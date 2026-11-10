//! Bjerknes forces between pulsating spheres in an ideal incompressible medium.
//!
//! A sphere of volume V(t) acts as a point source of strength q = dV/dt:
//! φ = −q/(4πr), u = q r̂/(4πr²). From the unsteady Bernoulli equation
//! p = −ρ ∂φ/∂t − ρ|u|²/2, and the force on a small body is F = −V ∇p.
//!
//! Cycle-averaged, for V_i = V0 + ΔV_i sin(ωt + φ_i) (C. A. & V. Bjerknes,
//! "Fields of Force", 1906):
//!
//! ```text
//! ⟨F⟩ = −ρ ω² ΔV1 ΔV2 cos(φ1 − φ2) / (8π d²)   (negative = attraction)
//! ```
//!
//! In-phase pulsators ATTRACT; anti-phase REPEL — the opposite sign of
//! Coulomb's law. That sign inversion is one of the historical reasons the
//! pulsating-aether theory of electricity failed.

use crate::stats::power_law_fit;
use crate::Check;
use glam::DVec3;
use std::f64::consts::PI;

#[derive(Clone, Debug)]
pub struct Pulsator {
    pub pos: DVec3,
    pub vel: DVec3,
    /// Mean radius a0.
    pub a0: f64,
    /// Relative radius amplitude ε (a = a0 (1 + ε sin(ωt + φ))).
    pub eps: f64,
    pub phase: f64,
    /// Material density of the sphere (sets its inertia).
    pub density: f64,
}

impl Pulsator {
    pub fn new(pos: DVec3, a0: f64, eps: f64, phase: f64) -> Self {
        Self {
            pos,
            vel: DVec3::ZERO,
            a0,
            eps,
            phase,
            density: 1.0,
        }
    }

    pub fn radius(&self, w: f64, t: f64) -> f64 {
        self.a0 * (1.0 + self.eps * (w * t + self.phase).sin())
    }

    pub fn volume(&self, w: f64, t: f64) -> f64 {
        4.0 / 3.0 * PI * self.radius(w, t).powi(3)
    }

    /// Source strength q = dV/dt = 4π a² ȧ.
    pub fn q(&self, w: f64, t: f64) -> f64 {
        let a = self.radius(w, t);
        let adot = self.a0 * self.eps * w * (w * t + self.phase).cos();
        4.0 * PI * a * a * adot
    }

    /// dq/dt = 8π a ȧ² + 4π a² ä.
    pub fn qdot(&self, w: f64, t: f64) -> f64 {
        let a = self.radius(w, t);
        let s = w * t + self.phase;
        let adot = self.a0 * self.eps * w * s.cos();
        let addot = -self.a0 * self.eps * w * w * s.sin();
        8.0 * PI * a * adot * adot + 4.0 * PI * a * a * addot
    }

    /// Volume amplitude in the linear limit, ΔV = 4π a0³ ε.
    pub fn delta_v(&self) -> f64 {
        4.0 * PI * self.a0.powi(3) * self.eps
    }

    /// Inertial mass including the added mass ½ ρ_fluid V of a sphere.
    pub fn inertia(&self, rho_fluid: f64) -> f64 {
        let v0 = 4.0 / 3.0 * PI * self.a0.powi(3);
        (self.density + 0.5 * rho_fluid) * v0
    }
}

#[derive(Clone, Debug)]
pub struct BjerknesSim {
    pub rho: f64,
    pub omega: f64,
    pub spheres: Vec<Pulsator>,
    pub time: f64,
    /// Samples per oscillation period for cycle averaging.
    pub samples: usize,
    /// Linear drag coefficient (0 = ideal fluid, d'Alembert: no drag).
    pub drag: f64,
}

impl BjerknesSim {
    pub fn new(spheres: Vec<Pulsator>) -> Self {
        Self {
            rho: 1.0,
            omega: 2.0 * PI,
            spheres,
            time: 0.0,
            samples: 64,
            drag: 0.0,
        }
    }

    /// Instantaneous pressure-gradient force on sphere `i` from all others.
    pub fn instantaneous_force(&self, i: usize, t: f64) -> DVec3 {
        let w = self.omega;
        let si = &self.spheres[i];
        let vi = si.volume(w, t);
        let mut grad_dphidt = DVec3::ZERO; // ∇(∂φ/∂t) of the others
        let others_u = |x: DVec3| -> DVec3 {
            let mut u = DVec3::ZERO;
            for (j, sj) in self.spheres.iter().enumerate() {
                if j != i {
                    let r = x - sj.pos;
                    let d = r.length();
                    u += sj.q(w, t) * r / (4.0 * PI * d * d * d);
                }
            }
            u
        };
        for (j, sj) in self.spheres.iter().enumerate() {
            if j == i {
                continue;
            }
            let r = si.pos - sj.pos;
            let d = r.length();
            // ∂φ/∂t = −q̇/(4π d) → ∇ = q̇ r̂/(4π d²)
            grad_dphidt += sj.qdot(w, t) * r / (4.0 * PI * d * d * d);
        }
        // ∇(|u|²/2) by central differences (second-order, small).
        let h = 1e-4 * si.a0.max(1e-6);
        let ke = |x: DVec3| 0.5 * others_u(x).length_squared();
        let grad_ke = DVec3::new(
            ke(si.pos + DVec3::X * h) - ke(si.pos - DVec3::X * h),
            ke(si.pos + DVec3::Y * h) - ke(si.pos - DVec3::Y * h),
            ke(si.pos + DVec3::Z * h) - ke(si.pos - DVec3::Z * h),
        ) / (2.0 * h);
        // ∇p = −ρ ∇(∂φ/∂t) − ρ ∇(|u|²/2);  F = −V ∇p
        self.rho * vi * (grad_dphidt + grad_ke)
    }

    /// Force on sphere `i` averaged over one period starting at time t.
    pub fn averaged_force(&self, i: usize, t: f64) -> DVec3 {
        let period = 2.0 * PI / self.omega;
        let n = self.samples.max(8);
        (0..n)
            .map(|k| self.instantaneous_force(i, t + period * k as f64 / n as f64))
            .sum::<DVec3>()
            / n as f64
    }

    /// Analytic Bjerknes force on `i` due to `j` (linear theory).
    pub fn analytic_force(&self, i: usize, j: usize) -> DVec3 {
        let (si, sj) = (&self.spheres[i], &self.spheres[j]);
        let r = si.pos - sj.pos;
        let d = r.length();
        let mag = -self.rho * self.omega.powi(2) * si.delta_v() * sj.delta_v() * (si.phase - sj.phase).cos()
            / (8.0 * PI * d * d);
        mag * r / d
    }

    /// Advance positions using the cycle-averaged force (separation of
    /// time scales: the slow drift of the spheres vs the fast pulsation).
    pub fn step(&mut self, dt: f64) {
        let forces: Vec<DVec3> = (0..self.spheres.len()).map(|i| self.averaged_force(i, self.time)).collect();
        let rho = self.rho;
        let drag = self.drag;
        for (s, f) in self.spheres.iter_mut().zip(forces) {
            let m = s.inertia(rho);
            s.vel += (f - drag * s.vel) / m * dt;
            s.pos += s.vel * dt;
        }
        // Prevent overlap: stop at contact.
        let n = self.spheres.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let d = self.spheres[i].pos - self.spheres[j].pos;
                let min = (self.spheres[i].a0 + self.spheres[j].a0) * (1.0 + self.spheres[i].eps.max(self.spheres[j].eps));
                if d.length() < min {
                    self.spheres[i].vel = DVec3::ZERO;
                    self.spheres[j].vel = DVec3::ZERO;
                }
            }
        }
        self.time += dt;
    }
}

/// Measured force (along the line of centres; negative = attraction) between
/// two identical pulsators at separation d with phase difference dphi.
pub fn pair_force(d: f64, dphi: f64, a0: f64, eps: f64) -> (f64, f64) {
    let mut sim = BjerknesSim::new(vec![
        Pulsator::new(DVec3::ZERO, a0, eps, 0.0),
        Pulsator::new(DVec3::new(d, 0.0, 0.0), a0, eps, dphi),
    ]);
    sim.samples = 256;
    let measured = sim.averaged_force(1, 0.0).x;
    let analytic = sim.analytic_force(1, 0).x;
    (measured, analytic)
}

/// Force-law sweep: returns (d, measured force) samples.
pub fn sweep(a0: f64, eps: f64, dphi: f64) -> Vec<(f64, f64)> {
    (0..16)
        .map(|k| {
            let d = a0 * 6.0 * 1.2f64.powi(k);
            (d, pair_force(d, dphi, a0, eps).0)
        })
        .collect()
}

/// Validation suite for the Bjerknes model.
pub fn checks() -> Vec<Check> {
    let (a0, eps) = (1.0, 0.02);
    let data = sweep(a0, eps, 0.0);
    let (xs, ys): (Vec<f64>, Vec<f64>) = data.iter().copied().unzip();
    let (_, p, r2) = power_law_fit(&xs, &ys);
    let (m, a) = pair_force(20.0, 0.0, a0, eps);
    let (m_anti, _) = pair_force(20.0, PI, a0, eps);
    let (m_quad, _) = pair_force(20.0, PI / 2.0, a0, eps);
    vec![
        Check::abs("Force-law exponent (in-phase)", -2.0, p, 0.02, "Bjerknes 1906: F ∝ 1/d²"),
        Check::abs("Power-law fit quality r²", 1.0, r2, 1e-3, "clean power law expected"),
        Check::rel("Amplitude vs ρω²ΔV₁ΔV₂/(8πd²)", a, m, 0.05, "linear Bjerknes theory"),
        Check::boolean("In-phase pulsators attract", m < 0.0, "Bjerknes: like phases attract"),
        Check::boolean("Anti-phase pulsators repel", m_anti > 0.0, "Bjerknes: opposite phases repel"),
        Check::abs("Quadrature (90°) force ≈ 0 (relative)", 0.0, m_quad / m.abs(), 0.02, "cos Δφ dependence"),
        Check::boolean(
            "Sign matches Coulomb (like repels)?",
            m > 0.0,
            "EXPECTED TO FAIL: Bjerknes forces have the inverted sign — a known obstacle for aether electrostatics",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bjerknes_physics_checks() {
        for c in checks() {
            if c.name.starts_with("Sign matches Coulomb") {
                assert!(!c.pass(), "sign inversion must be reproduced");
            } else {
                assert!(c.pass(), "{c:?}");
            }
        }
    }
}
