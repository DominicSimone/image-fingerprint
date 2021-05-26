#![allow(dead_code)]

use eframe::epi::{App, IconData};
use image::imageops::FilterType::Gaussian;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, Rgb};

mod app;
mod corner;
mod fingerprint;
mod storage;

fn main() {
    // let mut img = resize(&open("./test/succulent_512.png"), 1. / 2.);
    // let corners = corner::harris(&img, 0.1, 5, 50, 1000000.);
    // mark_corners(&mut img, &corners);
    // save(&img);

    let local_storage = match storage::init("./storage/app_persistence.txt") {
        None => panic!("Failed to open persistent storage"),
        Some(ls) => ls
    };

    let gui = app::TemplateApp::default();
    let mut native_options = eframe::NativeOptions::default();
    native_options.icon_data = Some(IconData {
        rgba: open("./icon.png").to_rgba8().into_vec(),
        width: 128,
        height: 128,
    });
    eframe::run_native(Box::new(gui), native_options);    
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

fn mark_corners(image: &mut DynamicImage, corners: &Vec<corner::Corner>) {
    let mut i = 0.;
    for c in corners {
        let x = c.index as u32 % image.width();
        let y = (c.index as u32 - x) / image.width();
        let color = (255. * (1. - i / (corners.len() as f32 * 2.))) as u8;
        mark_custom(image, x, y, 1, (color, 0, 0));
        i = i + 1.;
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
            if j == xmin || j == xmax || k == ymin || k == ymax {
                if j >= 0 && j < img.width() as i32 && k >= 0 && k < img.height() as i32 {
                    img.put_pixel(j as u32, k as u32, Rgb([rgb.0, rgb.1, rgb.2]));
                }
            }
        }
    }
}
