use image::DynamicImage;

pub struct Fingerprint {
    id: String,
}

/**
 * Using a vector of indices, create a fingerprint that is easily stored and comparable to other fingerprints.
 */
pub fn fingerprint(image: &DynamicImage, corners: Vec<usize>) -> String {
    todo!();
}

/**
 * Compares two fingerprints and returns a float between 0.0 and 100.0 that describes their similarity, 100.0 being the most similar.
 */
pub fn compare_prints(first: Fingerprint, second: Fingerprint) -> f32 {
    todo!();
}