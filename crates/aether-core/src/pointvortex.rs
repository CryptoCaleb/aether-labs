//! 2D point vortices: the simplest vortex "matter".
//!
//! dz̄_k/dt = (1/2πi) Σ_{j≠k} Γ_j / (z_k − z_j)
//!
//! Vortices interact through a logarithmic (2D Coulomb) energy
//! H = −(1/4π) Σ_{j≠k} Γ_j Γ_k ln r_jk, but respond with a Magnus-type motion
//! perpendicular to the "force" — a pair of like vortices orbits, an opposite
//! pair translates.

use crate::Check;
use glam::DVec2;
use std::f64::consts::PI;

#[derive(Clone, Debug)]
pub struct PointVortices {
    pub pos: Vec<DVec2>,
    pub gamma: Vec<f64>,
    /// Smoothing length (0 = exact point vortices).
    pub core: f64,
    pub time: f64,
}

impl PointVortices {
    pub fn new(pos: Vec<DVec2>, gamma: Vec<f64>) -> Self {
        Self { pos, gamma, core: 0.0, time: 0.0 }
    }

    pub fn velocities(&self, pos: &[DVec2]) -> Vec<DVec2> {
        let d2 = self.core * self.core;
        (0..pos.len())
            .map(|k| {
                let mut u = DVec2::ZERO;
                for j in 0..pos.len() {
                    if j != k {
                        let r = pos[k] - pos[j];
                        let r2 = r.length_squared() + d2;
                        u += self.gamma[j] / (2.0 * PI * r2) * DVec2::new(-r.y, r.x);
                    }
                }
                u
            })
            .collect()
    }

    pub fn step(&mut self, dt: f64) {
        let y0 = self.pos.clone();
        let add = |y: &[DVec2], k: &[DVec2], h: f64| -> Vec<DVec2> { y.iter().zip(k).map(|(a, b)| *a + *b * h).collect() };
        let k1 = self.velocities(&y0);
        let k2 = self.velocities(&add(&y0, &k1, 0.5 * dt));
        let k3 = self.velocities(&add(&y0, &k2, 0.5 * dt));
        let k4 = self.velocities(&add(&y0, &k3, dt));
        for i in 0..y0.len() {
            self.pos[i] = y0[i] + (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]) * (dt / 6.0);
        }
        self.time += dt;
    }

    pub fn hamiltonian(&self) -> f64 {
        let mut h = 0.0;
        for i in 0..self.pos.len() {
            for j in (i + 1)..self.pos.len() {
                let r2 = (self.pos[i] - self.pos[j]).length_squared() + self.core * self.core;
                h -= self.gamma[i] * self.gamma[j] * r2.ln() / (4.0 * PI);
            }
        }
        h
    }

    /// Linear impulse (Σ Γ y, −Σ Γ x) and angular impulse Σ Γ |x|².
    pub fn impulses(&self) -> (DVec2, f64) {
        let mut p = DVec2::ZERO;
        let mut l = 0.0;
        for (x, g) in self.pos.iter().zip(&self.gamma) {
            p += *g * DVec2::new(x.y, -x.x);
            l += g * x.length_squared();
        }
        (p, l)
    }
}

pub fn checks() -> Vec<Check> {
    let d = 1.0;
    // Co-rotating pair.
    let mut pair = PointVortices::new(vec![DVec2::new(-0.5, 0.0), DVec2::new(0.5, 0.0)], vec![1.0, 1.0]);
    let h0 = pair.hamiltonian();
    let (_, l0) = pair.impulses();
    let steps = 2000;
    let dt = 0.005;
    for _ in 0..steps {
        pair.step(dt);
    }
    let ang = {
        let r = pair.pos[1] - pair.pos[0];
        let mut a = r.y.atan2(r.x);
        // Unwrap using the expected sign/rate.
        let expected = 2.0 / (2.0 * PI * d * d) * pair.time;
        while a < expected - PI {
            a += 2.0 * PI;
        }
        a
    };
    let omega = ang / pair.time;
    // Counter-rotating pair translates.
    let mut dip = PointVortices::new(vec![DVec2::new(-0.5, 0.0), DVec2::new(0.5, 0.0)], vec![1.0, -1.0]);
    let y0 = dip.pos[0].y;
    for _ in 0..steps {
        dip.step(dt);
    }
    let u = (dip.pos[0].y - y0).abs() / dip.time;
    vec![
        Check::rel("Like pair orbits at Ω = (Γ₁+Γ₂)/(2πd²)", 2.0 / (2.0 * PI * d * d), omega, 1e-3, "Helmholtz/Kirchhoff"),
        Check::rel("Opposite pair translates at U = Γ/(2πd)", 1.0 / (2.0 * PI * d), u, 1e-3, "Helmholtz/Kirchhoff"),
        Check::abs("Energy (2D Coulomb H) conserved", h0, pair.hamiltonian(), 1e-8, "Kirchhoff Hamiltonian"),
        Check::abs("Angular impulse conserved", l0, pair.impulses().1, 1e-8, "rotational symmetry"),
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn point_vortex_checks() {
        for c in super::checks() {
            assert!(c.pass(), "{c:?}");
        }
    }
}
