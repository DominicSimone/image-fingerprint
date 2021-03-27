use image::io::Reader as ImageReader;

fn main() {
    let img = open("./test/tree_256.png");
    let img = img.resize(16, 16, image::imageops::FilterType::Gaussian);
    println!("{:?}", img);
}

fn open(path: &str) -> image::DynamicImage {
    ImageReader::open(path).unwrap().decode().unwrap()
}