#![allow(unused_parens)]
#![allow(dead_code)]

use image::imageops::FilterType::Gaussian;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, GrayImage, Rgb};

use std::cmp::Ordering;
use std::collections::BinaryHeap;

mod linalg;

const KERNEL_X: [f32; 9] = [0., 0., 0., 1., 0., -1., 0., 0., 0.];
const KERNEL_Y: [f32; 9] = [0., 1., 0., 0., 0., 0., 0., -1., 0.];

const SOBEL_KERNEL_X: [f32; 9] = [1., 0., -1., 2., 0., -2., 1., 0., -1.];
const SOBEL_KERNEL_Y: [f32; 9] = [1., 2., 1., 0., 0., 0., -1., -2., -1.];

const DEBUG: bool = true;

#[derive(Copy, Clone, Debug)]
struct Corner {
    index: usize,
    score: f32,
}

impl PartialEq for Corner {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for Corner {}

impl PartialOrd for Corner {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Ord for Corner {
    fn cmp(&self, other: &Self) -> Ordering {
        let diff = self.score - &other.score;
        match diff > 0. {
            true => Ordering::Greater,
            false => Ordering::Less,
        }
    }
}

/**
 * - Harris corner should be picking up corners on the edges of the star, not just its points. Adjusting the threshold does not solve the problem, even with a threshold
 * of 0 it still does not pick up the edges very well, they seem to be offset from the actual edge. Problems with non maximal suppression?
 */

fn main() {
    let mut img = resize(&open("./test/succulent_512.png"), 1.);
    let corners = harris(&img, 0.1, 5, 50, 40000.);
    mark_corners(&mut img, &corners);
    save(&img);
}

fn grads(
    image: &DynamicImage,
    gaussian: f32,
    window_size: u32,
    kx: &[f32; 9],
    ky: &[f32; 9],
) -> Vec<(f32, f32, f32)> {
    let gray = image.grayscale().blur(gaussian);
    let bytes = gray.as_bytes();

    let mut grads = vec![(0 as f32, 0 as f32, 0 as f32); bytes.len()];

    let mut grad_x = vec![0 as u8; bytes.len()];
    let mut grad_y = vec![0 as u8; bytes.len()];

    let width = image.width();
    let height = image.height();
    for row in 0..=(height - window_size) {
        for w_start in 0..=(width - window_size) {
            let w_center =
                (row * width) + w_start + (width * (window_size / 2)) + (window_size / 2);
            let mut k_i = 0;
            let mut sumx = 0.;
            let mut sumy = 0.;
            for w_row in 0..window_size {
                for p in 0..window_size {
                    let index = (row * width) + w_start + (w_row * width) + p;

                    sumx += kx[k_i] * bytes[index as usize] as f32;
                    sumy += ky[k_i] * bytes[index as usize] as f32;

                    k_i += 1;
                }
            }
            grads[w_center as usize] = (sumx * sumx, sumx * sumy, sumy * sumy);
            grad_x[w_center as usize] = sumx.abs() as u8;
            grad_y[w_center as usize] = sumy.abs() as u8;
        }
    }

    grads
}

// Threshold of about 40,000 works decently well for corners
fn harris(
    image: &DynamicImage,
    gaussian: f32,
    window_size: usize,
    num_corners: usize,
    threshold: f32,
) -> Vec<Corner> {
    assert!(window_size % 2 != 0);

    let sm_data = grads(image, gaussian, 3, &SOBEL_KERNEL_X, &SOBEL_KERNEL_Y);

    let width = image.width() as usize;
    let height = image.height() as usize;

    let mut harris_scores: Vec<f32> = vec![0.; width * height];
    let mut corner_heap: BinaryHeap<Corner> = BinaryHeap::new();

    let mut max_score = 0.;

    for row in 0..=(height - window_size) {
        for w_start in 0..=(width - window_size) {
            let mut sumx2: f32 = 0.;
            let mut sumxy: f32 = 0.;
            let mut sumy2: f32 = 0.;
            for w_row in 0..window_size {
                for p in 0..window_size {
                    let index = (row * width) + w_start + (w_row * width) + p;
                    sumx2 += sm_data[index].0;
                    sumxy += sm_data[index].1;
                    sumy2 += sm_data[index].2;
                }
            }

            let w_center =
                (row * width) + w_start + (width * (window_size / 2)) + (window_size / 2);

            let harris = linalg::harris_corner_score(
                sumx2 as f32 / 9.,
                sumxy as f32 / 9.,
                sumy2 as f32 / 9.,
            );
            harris_scores[w_center] = harris;

            if harris > max_score {
                max_score = harris;
            }
        }
    }

    // NMS
    for row in 0..=(height - window_size) {
        for w_start in 0..=(width - window_size) {
            let w_center =
                (row * width) + w_start + (width * (window_size / 2)) + (window_size / 2);
            let mut local_max = true;
            'window: for w_row in 0..window_size {
                for p in 0..window_size {
                    let i = (row * width) + w_start + (w_row * width) + p;
                    if i != w_center && harris_scores[i] > harris_scores[w_center] {
                        local_max = false;
                        break 'window;
                    }
                }
            }
            if local_max {
                corner_heap.push(Corner {
                    index: w_center,
                    score: harris_scores[w_center],
                })
            }
        }
    }

    if DEBUG {
        let mut harris_as_pixels: Vec<u8> = vec![0; width * height];
        for (i, f) in harris_scores.into_iter().enumerate() {
            harris_as_pixels[i] = (255. * (f / max_score)) as u8;
        }

        GrayImage::from_raw(width as u32, height as u32, harris_as_pixels)
            .unwrap()
            .save("./test/_harris_as_pixels.png");
    }

    let mut corners: Vec<Corner> = Vec::new();
    let size = std::cmp::min(corner_heap.len(), num_corners);
    for _ in 0..size {
        let temp: Corner = corner_heap.pop().unwrap();
        if temp.score > threshold {
            corners.push(temp);
        }
    }
    corners
}

fn open(path: &str) -> DynamicImage {
    ImageReader::open(path).unwrap().decode().unwrap()
}

fn save(image: &DynamicImage) {
    image.save("./test/_out.png").ok();
}

fn resize(image: &DynamicImage, ratio: f32) -> DynamicImage {
    image.resize(
        (image.width() as f32 * ratio) as u32,
        (image.height() as f32 * ratio) as u32,
        Gaussian,
    )
}

fn mark_corners(image: &mut DynamicImage, corners: &Vec<Corner>) {
    let mut i = 0.;
    for c in corners {
        let x = c.index as u32 % image.width();
        let y = (c.index as u32 - x) / image.width();
        let color = (255. * (1. - i / (corners.len() as f32 * 2.))) as u8;
        mark_custom(image, x, y, 1, (color, 0, 0));
        i = i + 1.;
    }
}

fn mark(image: &mut DynamicImage, x: u32, y: u32, r: u32) {
    mark_custom(image, x, y, r, (255, 0, 0))
}

fn mark_custom(image: &mut DynamicImage, x: u32, y: u32, r: u32, rgb: (u8, u8, u8)) {
    let img = image.as_mut_rgb8().unwrap();
    let xmin: i32 = x as i32 - r as i32;
    let xmax: i32 = x as i32 + r as i32;
    let ymin: i32 = y as i32 - r as i32;
    let ymax: i32 = y as i32 + r as i32;
    for j in xmin..=xmax {
        for k in ymin..=ymax {
            if (j == xmin || j == xmax || k == ymin || k == ymax) {
                if (j >= 0 && j < img.width() as i32 && k >= 0 && k < img.height() as i32) {
                    img.put_pixel(j as u32, k as u32, Rgb([rgb.0, rgb.1, rgb.2]));
                }
            }
        }
    }
}
