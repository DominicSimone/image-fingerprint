/**
 * Takes in values from a 2x2 matrix and computes the eigenvalues.
 */
pub fn eigenvalues2x2(a: f32, b: f32, c: f32, d: f32) -> Result<(f32, f32), u32> {
    quadratic(1., -1. * (a + d), a * d - b * c)
}

pub fn harris_corner_score(xx: f32, xy: f32, yy: f32) -> f32 {
    let det = xx * yy - xy * xy;
    let trace = xx + yy;
    let k = 0.04;
    det - (k * trace * trace)
}

/**
 * Takes in coefficients for a second degree polynomial and solves the
 * quadratic formula, returning any existing real roots.
 */
pub fn quadratic(a: f32, b: f32, c: f32) -> Result<(f32, f32), u32> {
    let denom = 2. * a;
    let root = (b * b - 4. * a * c).sqrt();

    match root.is_nan() {
        true => return Err(404),
        false => return Ok(((-1. * b + root) / denom, (-1. * b - root) / denom)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_tuple(left: (f32, f32), right: (f32, f32), d: f32) -> bool {
        if !(left.0 - right.0 < d || right.0 - left.0 < d) {
            return false;
        }
        if !(left.1 - right.1 < d || right.1 - left.1 < d) {
            return false;
        }
        true
    }

    #[test]
    fn test_eigenvalues() {
        assert_eq!(eigenvalues2x2(1., 1., 1., 1.).unwrap(), (2., 0.));
    }

    #[test]
    fn test_quadratic_two_roots() {
        assert!(compare_tuple(
            quadratic(2., 5., 1.).unwrap(),
            (-0.21924, -2.2807),
            0.0001
        ));
    }

    #[test]
    fn test_quadratic_one_root() {
        assert!(compare_tuple(
            quadratic(2.5, 5.1, 0.).unwrap(),
            (-2.04, -2.04),
            0.0001
        ));
    }

    #[test]
    #[should_panic]
    fn test_quadratic_no_roots() {
        quadratic(6., 4., 1.).unwrap();
    }
}
