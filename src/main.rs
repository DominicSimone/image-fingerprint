use image::io::Reader as ImageReader;
use image::DynamicImage;
use image::imageops::FilterType::Gaussian;

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
    gradients(&img);   
}

fn open(path: &str) -> DynamicImage {
    ImageReader::open(path).unwrap().decode().unwrap()
}

pub fn gradients(image: &DynamicImage) -> (DynamicImage, DynamicImage) {
    let gray = image.grayscale().blur(0.5);

    let grad_x = gray.filter3x3(&KERNEL_X);
    let grad_y = gray.filter3x3(&KERNEL_Y);
    
    (grad_x, grad_y)
}

pub fn small(image: &DynamicImage) -> DynamicImage {
    image.resize(16, 16, Gaussian)
}

pub fn save(image: &DynamicImage) {
    image.save("./test/out.png").ok();
}