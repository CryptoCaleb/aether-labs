//! Small statistics helpers used by the validation checks.

/// Ordinary least squares fit y = a + b x. Returns (a, b, r²).
pub fn linear_fit(xs: &[f64], ys: &[f64]) -> (f64, f64, f64) {
    let n = xs.len().min(ys.len()) as f64;
    if n < 2.0 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for (x, y) in xs.iter().zip(ys) {
        sxx += (x - mx) * (x - mx);
        sxy += (x - mx) * (y - my);
        syy += (y - my) * (y - my);
    }
    let b = sxy / sxx;
    let a = my - b * mx;
    let r2 = if syy > 0.0 { sxy * sxy / (sxx * syy) } else { 1.0 };
    (a, b, r2)
}

/// Fit y = C x^p in log-log space. Returns (C, p, r²). Uses |y|.
pub fn power_law_fit(xs: &[f64], ys: &[f64]) -> (f64, f64, f64) {
    let lx: Vec<f64> = xs.iter().map(|x| x.ln()).collect();
    let ly: Vec<f64> = ys.iter().map(|y| y.abs().ln()).collect();
    let (a, b, r2) = linear_fit(&lx, &ly);
    (a.exp(), b, r2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_power_law() {
        let xs: Vec<f64> = (1..20).map(|i| i as f64).collect();
        let ys: Vec<f64> = xs.iter().map(|x| 3.0 * x.powf(-2.0)).collect();
        let (c, p, r2) = power_law_fit(&xs, &ys);
        assert!((c - 3.0).abs() < 1e-9 && (p + 2.0).abs() < 1e-9 && r2 > 0.999);
    }
}
