use image::{DynamicImage, GenericImageView};

use std::cmp::Ordering;
use std::collections::BinaryHeap;

const SOBEL_KERNEL_X: [f32; 9] = [1., 0., -1., 2., 0., -2., 1., 0., -1.];
const SOBEL_KERNEL_Y: [f32; 9] = [1., 2., 1., 0., 0., 0., -1., -2., -1.];

#[derive(Copy, Clone, Debug)]
pub struct Corner {
    pub index: usize,
    pub score: f32,
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
        let diff = self.score - other.score;
        match diff > 0. {
            true => Ordering::Greater,
            false => Ordering::Less,
        }
    }
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

    let mut grads = vec![(0_f32, 0_f32, 0_f32); bytes.len()];

    let mut grad_x = vec![0_u8; bytes.len()];
    let mut grad_y = vec![0_u8; bytes.len()];

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

/**
 * Returns a vector of corners for an image using a Harris corner detector.
 * 
 * A threshold of about 1,000,000 works decently well for finding only corners.
 */
pub fn harris(
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

            let harris = harris_corner_score(
                sumx2 as f32 / 9.,
                sumxy as f32 / 9.,
                sumy2 as f32 / 9.,
            );
            harris_scores[w_center] = harris;
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

fn harris_corner_score(xx: f32, xy: f32, yy: f32) -> f32 {
    let det = xx * yy - xy * xy;
    let trace = xx + yy;
    let k = 0.04;
    det - (k * trace * trace)
}