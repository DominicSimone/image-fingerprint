#![allow(unused_parens)]

use image::imageops::FilterType::Gaussian;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, Rgb};

use std::cmp::Ordering;
use std::collections::BinaryHeap;

mod linalg;

const KERNEL_X: [f32; 9] = [0., 0., 0., 1., 0., -1., 0., 0., 0.];
const KERNEL_Y: [f32; 9] = [0., 1., 0., 0., 0., 0., 0., -1., 0.];

/**
 * 1. Add prints to eigen functions to debug (can plug in values to online calculator)
 * 2. Window macro? macro takes in one/two functions
 */

#[derive(Copy, Clone, Debug)]
struct Corner {
    index: usize,
    eigens: (f32, f32),
}

impl PartialEq for Corner {
    fn eq(&self, other: &Self) -> bool {
        self.eigens == other.eigens
    }
}

impl Eq for Corner {}

impl PartialOrd for Corner {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.eigens
            .0
            .max(self.eigens.1)
            .partial_cmp(&other.eigens.0.max(other.eigens.1))
    }
}

impl Ord for Corner {
    fn cmp(&self, other: &Self) -> Ordering {
        let diff = self.eigens.0.max(self.eigens.1) - &other.eigens.0.max(other.eigens.1);
        match diff > 0. {
            true => Ordering::Greater,
            false => Ordering::Less,
        }
    }
}

fn main() {
    let mut img = small(&open("./test/triangle_256.png"), 0.125);
    let corners = shi_tomasi(&img, 5, 10);
    for c in corners {
        println!("i:{}: {:?}", c.index, c.eigens);
        let x = c.index as u32 % img.width();
        let y = (c.index as u32 - x) / img.width();
        mark(&mut img, x, y, 1);
    }
    save(&img);
}

fn gradients(image: &DynamicImage) -> (DynamicImage, DynamicImage) {
    let gray = image.grayscale().blur(0.5);

    let grad_x = gray.filter3x3(&KERNEL_X);
    let grad_y = gray.filter3x3(&KERNEL_Y);

    (grad_x, grad_y)
}

fn second_moment_prep(
    gradient_x: &DynamicImage,
    gradient_y: &DynamicImage,
) -> Vec<(u32, u32, u32)> {
    gradient_x
        .as_bytes()
        .iter()
        .zip(gradient_y.as_bytes().iter())
        .map(|(x, y)| {
            (
                (*x as u32).pow(2),
                (*x as u32) * (*y as u32),
                (*y as u32).pow(2),
            )
        })
        .collect()
}

fn shi_tomasi(image: &DynamicImage, window_size: usize, num_corners: usize) -> Vec<Corner> {
    assert!(window_size % 2 != 0);

    let (grad_x, grad_y) = gradients(image);
    let sm_data = second_moment_prep(&grad_x, &grad_y);

    // Window over sm_data and find eigen values of constructed second moment matrix
    let width = image.width() as usize;
    let height = image.height() as usize;

    let mut eigenvalues: Vec<(f32, f32)> = vec![(0., 0.); width * height];
    let mut corner_heap: BinaryHeap<Corner> = BinaryHeap::new();

    for row in 0..=(height - window_size) {
        for w_start in 0..=(width - window_size) {
            let mut sumx2: u32 = 0;
            let mut sumxy: u32 = 0;
            let mut sumy2: u32 = 0;
            for w_row in 0..window_size {
                for p in 0..window_size {
                    let index = (row * width) + w_start + (w_row * width) + p;
                    sumx2 += sm_data[index].0;
                    sumxy += sm_data[index].1;
                    sumy2 += sm_data[index].2;
                }
            }
            // TODO error handling for when the eigenvalue calculation fails
            let w_center =
                (row * width) + w_start + (width * (window_size / 2)) + (window_size / 2);
            print!("{}:", w_center);
            let eigens =
                linalg::eigenvalues2x2(sumx2 as f32, sumxy as f32, sumxy as f32, sumy2 as f32)
                    .unwrap();
            eigenvalues[w_center] = eigens;
        }
    }

    //println!("{:?}", eigenvalues);

    // iterate over eigenvalues and select corners that are local maximums
    for row in 0..=(height - window_size) {
        for w_start in 0..=(width - window_size) {
            let w_center =
                (row * width) + w_start + (width * (window_size / 2)) + (window_size / 2);
            let mut local_max = true;
            let center_max = eigenvalues[w_center].0.max(eigenvalues[w_center].1);
            'window: for w_row in 0..window_size {
                for p in 0..window_size {
                    let i = (row * width) + w_start + (w_row * width) + p;
                    // TODO this is not working as intended, adjacent pixels are being added as corners (maybe fixed with window_center changes?)
                    if i != w_center && eigenvalues[i].0.max(eigenvalues[i].1) > center_max {
                        local_max = false;
                        break 'window;
                    }
                }
            }
            if local_max {
                corner_heap.push(Corner {
                    index: w_center,
                    eigens: eigenvalues[w_center],
                })
            }
        }
    }
    let mut corners: Vec<Corner> = Vec::new();
    let size = std::cmp::min(corner_heap.len(), num_corners);
    for _ in 0..size {
        let temp: Corner = corner_heap.pop().unwrap();
        if temp.eigens.0 != 0.0 || temp.eigens.1 != 0.0 {
            corners.push(corner_heap.pop().unwrap());
        }
    }
    corners
}

fn open(path: &str) -> DynamicImage {
    ImageReader::open(path).unwrap().decode().unwrap()
}

fn save(image: &DynamicImage) {
    image.save("./test/out.png").ok();
}

fn small(image: &DynamicImage, ratio: f32) -> DynamicImage {
    image.resize(
        (image.width() as f32 * ratio) as u32,
        (image.height() as f32 * ratio) as u32,
        Gaussian,
    )
}

fn mark(image: &mut DynamicImage, x: u32, y: u32, r: u32) {
    let img = image.as_mut_rgb8().unwrap();
    let xmin: i32 = x as i32 - r as i32;
    let xmax: i32 = x as i32 + r as i32;
    let ymin: i32 = y as i32 - r as i32;
    let ymax: i32 = y as i32 + r as i32;
    for j in xmin..=xmax {
        for k in ymin..=ymax {
            if (j == xmin || j == xmax || k == ymin || k == ymax) {
                if (j >= 0 && j < img.width() as i32 && k >= 0 && k < img.height() as i32) {
                    img.put_pixel(j as u32, k as u32, Rgb([255, 0, 0]));
                }
            }
        }
    }
}
