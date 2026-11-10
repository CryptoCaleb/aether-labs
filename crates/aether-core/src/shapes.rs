//! Initial vortex-filament geometries.

use crate::filament::Filament;
use glam::DVec3;
use std::f64::consts::TAU;

/// Orthonormal basis (u, v) perpendicular to `n`.
pub fn basis(n: DVec3) -> (DVec3, DVec3) {
    let n = n.normalize();
    let helper = if n.x.abs() < 0.9 { DVec3::X } else { DVec3::Y };
    let u = n.cross(helper).normalize();
    let v = n.cross(u);
    (u, v)
}

/// Circular ring whose self-induced motion (for Γ > 0) is along `normal`.
pub fn ring(center: DVec3, normal: DVec3, radius: f64, n: usize, gamma: f64, color: [u8; 3]) -> Filament {
    perturbed_ring(center, normal, radius, n, gamma, color, 0, 0.0)
}

/// Ring with a sinusoidal radial + axial perturbation of azimuthal mode `m`
/// (seeds Kelvin waves / the Widnall instability).
#[allow(clippy::too_many_arguments)]
pub fn perturbed_ring(
    center: DVec3,
    normal: DVec3,
    radius: f64,
    n: usize,
    gamma: f64,
    color: [u8; 3],
    m: u32,
    amp: f64,
) -> Filament {
    let (u, v) = basis(normal);
    let nn = normal.normalize();
    let pts = (0..n)
        .map(|i| {
            let t = TAU * i as f64 / n as f64;
            let r = radius * (1.0 + amp * (m as f64 * t).cos());
            let z = radius * amp * (m as f64 * t).sin();
            center + r * (t.cos() * u + t.sin() * v) + z * nn
        })
        .collect();
    Filament::new(pts, gamma, color)
}

/// (p, q) torus knot on a torus of major radius `big_r`, tube radius `small_r`.
/// (2, 3) is the trefoil used by Kleckner & Irvine (2013).
pub fn torus_knot(p: u32, q: u32, big_r: f64, small_r: f64, n: usize, gamma: f64, color: [u8; 3]) -> Filament {
    let pts = (0..n)
        .map(|i| {
            let t = TAU * i as f64 / n as f64;
            let (p, q) = (p as f64, q as f64);
            let r = big_r + small_r * (q * t).cos();
            DVec3::new(r * (p * t).cos(), r * (p * t).sin(), small_r * (q * t).sin())
        })
        .collect();
    Filament::new(pts, gamma, color)
}

/// Long straight-ish periodic line approximated as a very large closed loop is
/// awkward; instead we model a helix-perturbed line as a large-radius ring
/// segment. For Kelvin-wave studies we use a ring of large radius with mode m.
pub fn kelvin_wave_ring(radius: f64, m: u32, amp: f64, n: usize, gamma: f64) -> Filament {
    perturbed_ring(DVec3::ZERO, DVec3::Z, radius, n, gamma, [120, 200, 255], m, amp)
}
