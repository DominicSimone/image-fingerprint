use image::imageops::FilterType::Gaussian;
use image::io::Reader as ImageReader;
use image::DynamicImage;

mod linalg;

/**
 * 0 0  0
 * 1 0 -1
 * 0 0  0
 */
const KERNEL_X: [f32; 9] = [0., 0., 0., 1., 0., -1., 0., 0., 0.];
/**
 * 0  1 0
 * 0  0 0
 * 0 -1 0
 */
const KERNEL_Y: [f32; 9] = [0., 1., 0., 0., 0., 0., 0., -1., 0.];

fn main() {
    let img = open("./test/succulent_512.png");
    shi_tomasi(&small(&img));
}

fn open(path: &str) -> DynamicImage {
    ImageReader::open(path).unwrap().decode().unwrap()
}

fn gradients(image: &DynamicImage) -> (DynamicImage, DynamicImage) {
    let gray = image.grayscale().blur(0.5);

    let grad_x = gray.filter3x3(&KERNEL_X);
    let grad_y = gray.filter3x3(&KERNEL_Y);

    (grad_x, grad_y)
}

// This can be optimized to only iterate over grad_x and grad_y once each, rather than twice
fn second_moment_prep(
    gradient_x: &DynamicImage,
    gradient_y: &DynamicImage,
) -> Vec<(u32, u32, u32)> {
    gradient_x
        .as_bytes()
        .iter()
        .zip(gradient_y.as_bytes().iter())
        .map(|(x, y)| ((*x as u32).pow(2), (*x as u32) * (*y as u32), (*y as u32).pow(2)))
        .collect()
}

pub fn shi_tomasi(image: &DynamicImage) {
    let (grad_x, grad_y) = gradients(image);
    let gradients_mult = second_moment_prep(&grad_x, &grad_y);
}

pub fn small(image: &DynamicImage) -> DynamicImage {
    image.resize(16, 16, Gaussian)
}

pub fn save(image: &DynamicImage) {
    image.save("./test/out.png").ok();
}
