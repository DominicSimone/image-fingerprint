use image::io::Reader as ImageReader;

fn main() {
    let img = open("./test/succulent_512.png");
    gradients(small(img));
}

fn open(path: &str) -> image::DynamicImage {
    ImageReader::open(path).unwrap().decode().unwrap()
}

fn gradients(image: image::DynamicImage) {
    let gray = image.grayscale();
    println!("{:?}", gray);
}

fn small(image: image::DynamicImage) -> image::DynamicImage {
    image.resize(16, 16, image::imageops::FilterType::Gaussian)
}