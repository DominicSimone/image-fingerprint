use image::{DynamicImage, GenericImageView};
use crate::corner::{Corner, harris};

pub struct Fingerprint {
    id: String,
    vec: Vec<f32>,
}

pub struct PointInfo {
    
}

/**
 * Using a vector of indices, create a fingerprint that is easily stored and comparable to other fingerprints.
 */
fn fingerprint(_image: &DynamicImage, _corners: Vec<usize>) -> Fingerprint {
    unimplemented!();
}

fn fingerprint_from_image(image: &DynamicImage) -> Fingerprint {
    let corners = harris(image, 0.1, 5, 50, 1000000_f32);
    
    unimplemented!();
}

/**
 * Compares two fingerprints and returns a float between 0.0 and 100.0 that describes their similarity, 100.0 being the most similar.
 */
fn compare_prints(_first: Fingerprint, _second: Fingerprint) -> f32 {
    unimplemented!();
}

#[test]
fn pokemon() {
    unimplemented!();
}
