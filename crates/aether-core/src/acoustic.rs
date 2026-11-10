//! Analog gravity: sound rays in a flowing medium.
//!
//! Linear sound in a barotropic, irrotational flow v obeys a wave equation on
//! the acoustic metric (Unruh 1981; Visser 1998):
//!
//! ```text
//! ds² = (ρ/c) [ −c² dt² + (dx − v dt)·(dx − v dt) ]
//! ```
//!
//! Rays are its null geodesics, equivalently Hamilton's equations for
//! ω = c|k| + v·k:  ẋ = c k̂ + v,  k̇ = −∇(v·k).
//!
//! Background: the "draining bathtub" v = (−A r̂ + B θ̂)/r, which has an
//! acoustic horizon at r_H = A/c and an ergosurface at r_E = √(A²+B²)/c.

use crate::Check;
use glam::DVec2;
use std::f64::consts::{PI, TAU};

#[derive(Clone, Copy, Debug)]
pub struct Bathtub {
    /// Radial sink strength A (inflow).
    pub a: f64,
    /// Circulation strength B (swirl).
    pub b: f64,
    /// Speed of sound c.
    pub c: f64,
}

impl Bathtub {
    pub fn velocity(&self, x: DVec2) -> DVec2 {
        let r2 = x.length_squared().max(1e-12);
        DVec2::new(-self.a * x.x - self.b * x.y, -self.a * x.y + self.b * x.x) / r2
    }

    /// (J^T k) where J_ij = ∂v_i/∂x_j; i.e. ∇(v·k) at fixed k.
    pub fn grad_v_dot_k(&self, x: DVec2, k: DVec2) -> DVec2 {
        let (a, b) = (self.a, self.b);
        let r2 = x.length_squared().max(1e-12);
        let r4 = r2 * r2;
        let (px, py) = (x.x, x.y);
        let dvx_dx = (-a * r2 + 2.0 * px * (a * px + b * py)) / r4;
        let dvx_dy = (-b * r2 + 2.0 * py * (a * px + b * py)) / r4;
        let dvy_dx = (b * r2 + 2.0 * px * (a * py - b * px)) / r4;
        let dvy_dy = (-a * r2 + 2.0 * py * (a * py - b * px)) / r4;
        DVec2::new(dvx_dx * k.x + dvy_dx * k.y, dvx_dy * k.x + dvy_dy * k.y)
    }

    pub fn horizon(&self) -> f64 {
        self.a / self.c
    }

    pub fn ergosurface(&self) -> f64 {
        (self.a * self.a + self.b * self.b).sqrt() / self.c
    }

    /// Analog surface gravity κ = c²/A at the horizon (constant c), which
    /// sets the analog Hawking temperature T = ħκ/(2π k_B).
    pub fn surface_gravity(&self) -> f64 {
        self.c * self.c / self.a.max(1e-12)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RayFate {
    Flying,
    Captured,
    Escaped,
}

#[derive(Clone, Debug)]
pub struct Ray {
    pub x: DVec2,
    pub k: DVec2,
    pub path: Vec<DVec2>,
    pub fate: RayFate,
}

impl Ray {
    pub fn new(x: DVec2, dir: DVec2) -> Self {
        Self {
            x,
            k: dir.normalize(),
            path: vec![x],
            fate: RayFate::Flying,
        }
    }
}

fn deriv(f: &Bathtub, x: DVec2, k: DVec2) -> (DVec2, DVec2) {
    let kh = k.normalize_or_zero();
    (f.c * kh + f.velocity(x), -f.grad_v_dot_k(x, k))
}

/// Advance a ray by one RK4 step.
pub fn step_ray(f: &Bathtub, ray: &mut Ray, dt: f64, capture_r: f64, escape_r: f64) {
    if ray.fate != RayFate::Flying {
        return;
    }
    let (x, k) = (ray.x, ray.k);
    let (a1, b1) = deriv(f, x, k);
    let (a2, b2) = deriv(f, x + a1 * dt * 0.5, k + b1 * dt * 0.5);
    let (a3, b3) = deriv(f, x + a2 * dt * 0.5, k + b2 * dt * 0.5);
    let (a4, b4) = deriv(f, x + a3 * dt, k + b3 * dt);
    ray.x = x + (a1 + 2.0 * a2 + 2.0 * a3 + a4) * dt / 6.0;
    ray.k = k + (b1 + 2.0 * b2 + 2.0 * b3 + b4) * dt / 6.0;
    // Keep |k| normalised (ray direction only matters).
    ray.k = ray.k.normalize_or_zero();
    ray.path.push(ray.x);
    let r = ray.x.length();
    if r < capture_r {
        ray.fate = RayFate::Captured;
    } else if r > escape_r {
        ray.fate = RayFate::Escaped;
    }
}

/// Integrate a ray to its fate (or until `max_steps`).
pub fn trace(f: &Bathtub, mut ray: Ray, dt: f64, max_steps: usize) -> Ray {
    let cap = 0.3 * f.horizon().max(1e-3);
    for _ in 0..max_steps {
        step_ray(f, &mut ray, dt, cap, 40.0 * f.ergosurface().max(1.0));
        if ray.fate != RayFate::Flying {
            break;
        }
    }
    ray
}

/// Time-time component of the acoustic metric (up to the conformal factor
/// ρ/c): g_tt = −(c² − |v|²). It changes sign on the ergosurface; inside it,
/// no sound wave can stay at rest relative to the lab.
pub fn g_tt(f: &Bathtub, r: f64) -> f64 {
    let v = f.velocity(DVec2::new(r, 0.0)).length();
    -(f.c * f.c - v * v)
}

/// Can some ray direction hold a wave packet stationary at radius r?
pub fn can_hover(f: &Bathtub, r: f64) -> bool {
    let x = DVec2::new(r, 0.0);
    // Choose the direction opposing the flow; check the residual lab speed.
    let v = f.velocity(x);
    let n = -v.normalize_or_zero();
    (f.c * n + v).length() < 1e-9 || v.length() <= f.c
}

pub fn checks() -> Vec<Check> {
    let f = Bathtub { a: 1.0, b: 1.5, c: 1.0 };
    let rh = f.horizon();
    let re = f.ergosurface();
    // 1. Every ray launched just inside the horizon is captured.
    let inside_all_captured = (0..36).all(|i| {
        let th = TAU * i as f64 / 36.0;
        let ray = trace(&f, Ray::new(DVec2::new(0.97 * rh, 0.0), DVec2::new(th.cos(), th.sin())), 2e-3, 20_000);
        ray.fate == RayFate::Captured
    });
    // 2. Outward rays just outside the horizon escape.
    let out_escape = {
        let ray = trace(&f, Ray::new(DVec2::new(1.05 * rh, 0.0), DVec2::X), 2e-3, 400_000);
        ray.fate == RayFate::Escaped
    };
    // 3. Ergosurface: find radius where min lab speed hits zero by bisection.
    let (mut lo, mut hi) = (1.0001 * rh, 3.0 * re);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if g_tt(&f, mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let measured_re = 0.5 * (lo + hi);
    // 4. Find the numerical horizon: largest radius where an outward ray is captured.
    let (mut lo, mut hi) = (0.5 * rh, 2.0 * rh);
    for _ in 0..30 {
        let mid = 0.5 * (lo + hi);
        let ray = trace(&f, Ray::new(DVec2::new(mid, 0.0), DVec2::X), 1e-3, 400_000);
        if ray.fate == RayFate::Captured {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let measured_rh = 0.5 * (lo + hi);
    let hover_ok = !can_hover(&f, 0.99 * re) && can_hover(&f, 1.01 * re);
    vec![
        Check::boolean("Waves can hover outside r_E but not inside", hover_ok, "ergoregion: no static observers"),
        Check::rel("Acoustic horizon radius r_H = A/c", rh, measured_rh, 0.01, "Visser 1998 (outward ray marginally trapped)"),
        Check::rel("Ergosurface radius √(A²+B²)/c", re, measured_re, 1e-3, "Visser 1998 (no stationary sound inside)"),
        Check::boolean("All rays inside r_H are captured", inside_all_captured, "horizon = one-way membrane"),
        Check::boolean("Outward ray just outside r_H escapes", out_escape, "horizon is the boundary"),
    ]
}

/// A fan of rays from a source point, for visualisation.
pub fn ray_fan(f: &Bathtub, src: DVec2, count: usize, dt: f64, steps: usize) -> Vec<Ray> {
    (0..count)
        .map(|i| {
            let th = PI * 2.0 * i as f64 / count as f64;
            trace(f, Ray::new(src, DVec2::new(th.cos(), th.sin())), dt, steps)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acoustic_checks() {
        for c in checks() {
            assert!(c.pass(), "{c:?}");
        }
    }

    #[test]
    fn still_medium_rays_are_straight() {
        let f = Bathtub { a: 0.0, b: 0.0, c: 1.0 };
        let mut ray = Ray::new(DVec2::new(5.0, 1.0), DVec2::new(-1.0, 0.0));
        for _ in 0..1000 {
            step_ray(&f, &mut ray, 1e-3, 0.0, 100.0);
        }
        assert!((ray.x - DVec2::new(4.0, 1.0)).length() < 1e-9);
    }
}
