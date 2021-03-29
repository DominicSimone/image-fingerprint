pub fn eigenvalue2x2(a: u32, b: u32, c: u32, d: u32) -> (f32, f32) {
    (0.0, 0.0)
}

pub fn quadratic(a: f32, b: f32, c: f32) -> Result<(f32, f32), String> {
    let denom = 2. * a;
    let root = (b * b - 4. * a * c).sqrt();

    match root.is_nan() {
        true => return Err(String::from("Quadratic failed; complex numbers")),
        false => return Ok(((-1. * b + root) / denom, (-1. * b - root) / denom)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_tuple(left: (f32, f32), right: (f32, f32), d: f32) -> bool {
        if !(left.0 - right.0 < d || right.0 - left.0 < d) {
            return false
        }
        if !(left.1 - right.1 < d || right.1 - left.1 < d) {
            return false
        }
        true
    }

    #[test]
    fn test_quadratic_two_roots() {
        assert!(compare_tuple(quadratic(2., 5., 1.).unwrap(), (-0.21924, -2.2807), 0.0001));
    }

    #[test]
    fn test_quadratic_one_root() {
        assert!(compare_tuple(quadratic(2.5, 5.1, 0.).unwrap(), (-2.04, -2.04), 0.0001));
    }

    #[test]
    #[should_panic]
    fn test_quadratic_no_roots() {
        quadratic(6., 4., 1.).unwrap();
    }
}
