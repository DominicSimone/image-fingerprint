#![allow(dead_code)]

use iced::{Application, Settings};
use image::imageops::FilterType::Gaussian;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, Rgb};

mod corner;
mod fingerprint;
mod gui;

fn main() -> iced::Result {
    // let mut img = resize(&open("./test/succulent_512.png"), 1. / 2.);
    // let corners = corner::harris(&img, 0.1, 5, 50, 1000000.);
    // mark_corners(&mut img, &corners);
    // save(&img);

    gui::Gui::run(Settings {
        ..Settings::default()
    })
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

fn mark_corners(image: &mut DynamicImage, corners: &[corner::Corner]) {
    let mut i = 0.;
    for c in corners {
        let x = c.index as u32 % image.width();
        let y = (c.index as u32 - x) / image.width();
        let color = (255. * (1. - i / (corners.len() as f32 * 2.))) as u8;
        mark_custom(image, x, y, 1, (color, 0, 0));
        i += 1.;
    }
}

fn mark_custom(image: &mut DynamicImage, x: u32, y: u32, r: u32, rgb: (u8, u8, u8)) {
    let img = image.as_mut_rgb8().unwrap();
    let xmin: i32 = x as i32 - r as i32;
    let xmax: i32 = x as i32 + r as i32;
    let ymin: i32 = y as i32 - r as i32;
    let ymax: i32 = y as i32 + r as i32;
    for j in xmin..=xmax {
        for k in ymin..=ymax {
            if (j == xmin || j == xmax || k == ymin || k == ymax) && j >= 0 && j < img.width() as i32 && k >= 0 && k < img.height() as i32 {
                img.put_pixel(j as u32, k as u32, Rgb([rgb.0, rgb.1, rgb.2]));
            }
        }
    }
}
