//! Emergent relativity in a medium: the sine-Gordon field.
//!
//! φ_tt = c² φ_xx − sin φ   (c = 1)
//!
//! This is the continuum limit of a chain of pendulums coupled by springs
//! (Frenkel–Kontorova) — a mechanical "aether". Its solitons are particle-like:
//!
//! * kink:     φ = 4 arctan exp(γ (x − v t)),  width ∝ 1/γ,  energy 8γ
//! * breather: an internal oscillator ("clock") of frequency ω at rest
//!
//! If rulers and clocks are themselves made of excitations of the same medium,
//! a moving ruler contracts by γ = 1/√(1−v²/c²) and a moving clock slows by γ
//! — dynamically, with no postulate. This is the FitzGerald–Larmor–Lorentz
//! mechanism, and the only way a medium can give a null Michelson–Morley result.

use crate::Check;
use std::f64::consts::PI;

pub struct SineGordon {
    pub dx: f64,
    pub dt: f64,
    pub x0: f64,
    pub phi: Vec<f64>,
    pub prev: Vec<f64>,
    pub time: f64,
}

pub fn gamma(v: f64) -> f64 {
    1.0 / (1.0 - v * v).sqrt()
}

pub fn kink(x: f64, t: f64, v: f64, center: f64) -> f64 {
    4.0 * (gamma(v) * (x - center - v * t)).exp().atan()
}

/// Breather with rest-frame frequency w, Lorentz-boosted to velocity v.
pub fn breather(x: f64, t: f64, w: f64, v: f64, center: f64) -> f64 {
    let g = gamma(v);
    let xp = g * ((x - center) - v * t);
    let tp = g * (t - v * (x - center));
    let s = (1.0 - w * w).sqrt();
    4.0 * ((s / w) * (w * tp).sin() / (s * xp).cosh()).atan()
}

impl SineGordon {
    /// Initialise from an analytic solution f(x, t) (sets φ at t=0 and t=−dt).
    pub fn from_solution(n: usize, length: f64, f: impl Fn(f64, f64) -> f64) -> Self {
        let dx = length / (n - 1) as f64;
        let dt = 0.4 * dx;
        let x0 = -0.5 * length;
        let phi = (0..n).map(|i| f(x0 + i as f64 * dx, 0.0)).collect();
        let prev = (0..n).map(|i| f(x0 + i as f64 * dx, -dt)).collect();
        Self { dx, dt, x0, phi, prev, time: 0.0 }
    }

    pub fn x(&self, i: usize) -> f64 {
        self.x0 + i as f64 * self.dx
    }

    /// Leapfrog step with Neumann (zero-slope) boundaries.
    pub fn step(&mut self) {
        let n = self.phi.len();
        let r = (self.dt / self.dx).powi(2);
        let mut next = vec![0.0; n];
        for i in 0..n {
            let l = self.phi[if i == 0 { 1 } else { i - 1 }];
            let rr = self.phi[if i == n - 1 { n - 2 } else { i + 1 }];
            let c = self.phi[i];
            next[i] = 2.0 * c - self.prev[i] + r * (l - 2.0 * c + rr) - self.dt * self.dt * c.sin();
        }
        self.prev = std::mem::replace(&mut self.phi, next);
        self.time += self.dt;
    }

    /// Energy E = ∫ ½φ_t² + ½φ_x² + (1 − cos φ) dx.
    pub fn energy(&self) -> f64 {
        let n = self.phi.len();
        let mut e = 0.0;
        for i in 0..n - 1 {
            let pt = (self.phi[i] - self.prev[i]) / self.dt;
            let px = (self.phi[i + 1] - self.phi[i]) / self.dx;
            e += 0.5 * pt * pt + 0.5 * px * px + (1.0 - self.phi[i].cos());
        }
        e * self.dx
    }

    /// Kink centre (where φ crosses π, linear interpolation).
    pub fn kink_center(&self) -> f64 {
        for i in 0..self.phi.len() - 1 {
            let (a, b) = (self.phi[i], self.phi[i + 1]);
            if (a - PI) * (b - PI) <= 0.0 && a != b {
                return self.x(i) + self.dx * (PI - a) / (b - a);
            }
        }
        f64::NAN
    }

    /// Kink width defined from the maximum slope: φ'_max = 2γ → w = 2/φ'_max.
    pub fn kink_width(&self) -> f64 {
        let mut m: f64 = 0.0;
        for i in 0..self.phi.len() - 1 {
            m = m.max((self.phi[i + 1] - self.phi[i]).abs() / self.dx);
        }
        2.0 / m
    }

    /// Value of φ at an arbitrary x (linear interpolation).
    pub fn sample(&self, x: f64) -> f64 {
        let s = (x - self.x0) / self.dx;
        let i = (s.floor() as usize).min(self.phi.len() - 2);
        let f = s - i as f64;
        self.phi[i] * (1.0 - f) + self.phi[i + 1] * f
    }
}

/// Run a moving kink and return (measured width × γ, energy / 8γ, speed / v).
pub fn kink_test(v: f64) -> (f64, f64, f64) {
    let mut sg = SineGordon::from_solution(6001, 60.0, |x, t| kink(x, t, v, -10.0));
    let c0 = sg.kink_center();
    let steps = (10.0 / sg.dt) as usize;
    for _ in 0..steps {
        sg.step();
    }
    let speed = (sg.kink_center() - c0) / sg.time;
    let g = gamma(v);
    let speed_ratio = if v == 0.0 { 1.0 + speed } else { speed / v };
    (sg.kink_width() * g, sg.energy() / (8.0 * g), speed_ratio)
}

/// Measure the lab-frame period of a breather moving at v (its "clock").
pub fn breather_period(w: f64, v: f64) -> f64 {
    let mut sg = SineGordon::from_solution(8001, 80.0, |x, t| breather(x, t, w, v, -15.0));
    let mut crossings = Vec::new();
    let mut last = sg.sample(-15.0);
    let rest_period = 2.0 * PI / w;
    let t_end = 3.2 * rest_period * gamma(v);
    while sg.time < t_end {
        sg.step();
        let xc = -15.0 + v * sg.time;
        let val = sg.sample(xc);
        if last <= 0.0 && val > 0.0 {
            // Interpolate the crossing time.
            let frac = -last / (val - last);
            crossings.push(sg.time - sg.dt + frac * sg.dt);
        }
        last = val;
    }
    if crossings.len() < 2 {
        return f64::NAN;
    }
    (crossings[crossings.len() - 1] - crossings[0]) / (crossings.len() - 1) as f64
}

pub fn checks() -> Vec<Check> {
    let (w0, _, _) = kink_test(0.0);
    let (w6, e6, s6) = kink_test(0.6);
    let (w9, e9, _) = kink_test(0.9);
    let w = 0.5;
    let p0 = breather_period(w, 0.0);
    let p6 = breather_period(w, 0.6);
    vec![
        Check::rel("Kink rest width (2/φ'max) = 1", 1.0, w0, 0.01, "analytic kink"),
        Check::rel("Length contraction at v=0.6c: w·γ = w₀", w0, w6, 0.01, "Lorentz–FitzGerald"),
        Check::rel("Length contraction at v=0.9c: w·γ = w₀", w0, w9, 0.02, "Lorentz–FitzGerald"),
        Check::rel("Kink energy E = 8γ at v=0.6c", 1.0, e6, 0.01, "relativistic E = γ m c²"),
        Check::rel("Kink energy E = 8γ at v=0.9c", 1.0, e9, 0.02, "relativistic E = γ m c²"),
        Check::rel("Kink keeps its speed (no aether drag)", 1.0, s6, 0.01, "Lorentz invariance of the medium"),
        Check::rel("Breather rest period 2π/ω", 2.0 * PI / w, p0, 0.01, "analytic breather"),
        Check::rel("Moving clock slows by γ at v=0.6c", gamma(0.6), p6 / p0, 0.02, "time dilation"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emergent_lorentz_checks() {
        for c in checks() {
            assert!(c.pass(), "{c:?}");
        }
    }
}
