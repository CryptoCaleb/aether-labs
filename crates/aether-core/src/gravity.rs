//! Tests of whether vortex-medium physics can behave like gravity.
//!
//! * **E4 Hamiltonian consistency.** For vortex rings, translation speed must
//!   equal dE/dP. This holds at every level of a hierarchy, which tells us
//!   which energy bookkeeping the code really implements.
//! * **E9 Metric universality.** Long-wavelength sound that circles a region
//!   picks up the acoustic Sagnac delay Δt ≈ 2Γ/c². This is the same whether
//!   the region holds one vortex or a cluster of many small ones. It is the
//!   *kinematic* side of emergent geometry, and it is expected to pass.
//! * **E11 Equivalence principle.** Two small vortex objects with the same
//!   impulse but different internal structure are placed in the same external
//!   flow. Gravity requires both to "fall" identically, independent of what
//!   they are made of (MICROSCOPE: η < 10⁻¹⁵). This is expected to FAIL beyond
//!   leading order.

use crate::filament::VortexSystem;
use crate::hierarchy::{hierarchical_ring, Level};
use crate::shapes;
use crate::Check;
use glam::{DVec2, DVec3};
use std::f64::consts::{PI, TAU};

// ---------------------------------------------------------------- E4

/// Velocity of the impulse centroid Z = Σ P_i z_i / Σ P_i of an axisymmetric
/// set of rings (P_i = Γ_i π R_i²). For a bundle whose strands churn around
/// each other this, not the plain mean node velocity, is the velocity of "the
/// ring" that enters the Hamiltonian relation U = dE/dP.
pub fn impulse_centroid_speed(sys: &VortexSystem) -> f64 {
    let vel = sys.velocities();
    let (mut num, mut den) = (0.0, 0.0);
    for (f, v) in sys.filaments.iter().zip(&vel) {
        let n = f.n() as f64;
        let (mut r, mut z, mut vr, mut vz) = (0.0, 0.0, 0.0, 0.0);
        for (p, u) in f.pts.iter().zip(v) {
            let rho = (p.x * p.x + p.y * p.y).sqrt();
            let rhat = DVec3::new(p.x / rho, p.y / rho, 0.0);
            r += rho;
            z += p.z;
            vr += u.dot(rhat);
            vz += u.z;
        }
        let (r, z, vr, vz) = (r / n, z / n, vr / n, vz / n);
        let pi = f.gamma * PI * r * r;
        num += pi * vz + 2.0 * f.gamma * PI * r * vr * z;
        den += pi;
    }
    num / den
}

/// Energy, impulse and impulse-centroid speed of a (possibly bundled) ring.
fn ring_state(r: f64, levels: &[Level], core: f64, nodes: usize) -> (f64, f64, f64) {
    let fils = hierarchical_ring(r, 1.0, nodes, levels);
    let sys = VortexSystem::new(fils, core, TAU * r / nodes as f64);
    let e = sys.energy();
    let p = sys.impulse().z;
    let u = impulse_centroid_speed(&sys);
    (e, p, u)
}

/// Returns (measured U, dE/dP) at radius r by central differences in r.
pub fn hamiltonian_pair(r: f64, levels: &[Level], core: f64, nodes: usize) -> (f64, f64) {
    let h = 0.03 * r;
    let (e1, p1, _) = ring_state(r - h, levels, core, nodes);
    let (_, _, u) = ring_state(r, levels, core, nodes);
    let (e2, p2, _) = ring_state(r + h, levels, core, nodes);
    (u, (e2 - e1) / (p2 - p1))
}

/// Like [`hamiltonian_pair`] but varies R at fixed bundle *volume* (a²R
/// constant), the constraint an incompressible thick core obeys (Kelvin).
pub fn hamiltonian_pair_volume(r: f64, levels: &[Level], core: f64, nodes: usize) -> (f64, f64) {
    let h = 0.03 * r;
    let scaled = |rr: f64| -> Vec<Level> {
        levels.iter().map(|l| Level { radius: l.radius * (r / rr).sqrt(), ..*l }).collect()
    };
    let (e1, p1, _) = ring_state(r - h, &scaled(r - h), core, nodes);
    let (_, _, u) = ring_state(r, levels, core, nodes);
    let (e2, p2, _) = ring_state(r + h, &scaled(r + h), core, nodes);
    (u, (e2 - e1) / (p2 - p1))
}

// ---------------------------------------------------------------- E9

/// Point-vortex velocity field at x.
fn pv_velocity(pos: &[DVec2], gam: &[f64], x: DVec2) -> DVec2 {
    let mut u = DVec2::ZERO;
    for (p, g) in pos.iter().zip(gam) {
        let r = x - *p;
        u += *g / (TAU * r.length_squared()) * DVec2::new(-r.y, r.x);
    }
    u
}

/// Sunflower cluster of n vortices of total circulation `gamma` in a disk of radius a.
pub fn cluster(n: usize, a: f64, gamma: f64) -> (Vec<DVec2>, Vec<f64>) {
    let golden = PI * (3.0 - 5f64.sqrt());
    let pos = (0..n)
        .map(|k| {
            let rr = if n == 1 { 0.0 } else { a * ((k as f64 + 0.5) / n as f64).sqrt() };
            let th = k as f64 * golden;
            DVec2::new(rr * th.cos(), rr * th.sin())
        })
        .collect();
    (pos, vec![gamma / n as f64; n])
}

/// Acoustic Sagnac delay t_cw − t_ccw for sound guided around a circle of
/// radius `l` through the flow of the given vortices (sound speed c). The
/// ground speed along the path solves |s t̂ − v| = c.
pub fn sagnac_delay(pos: &[DVec2], gam: &[f64], l: f64, c: f64) -> f64 {
    let m = 20_000;
    let dl = TAU * l / m as f64;
    let mut t_ccw = 0.0;
    let mut t_cw = 0.0;
    for i in 0..m {
        let th = TAU * (i as f64 + 0.5) / m as f64;
        let x = DVec2::new(l * th.cos(), l * th.sin());
        let t = DVec2::new(-th.sin(), th.cos());
        let v = pv_velocity(pos, gam, x);
        let vt = v.dot(t);
        let vn2 = v.length_squared() - vt * vt;
        let root = (c * c - vn2).sqrt();
        t_ccw += dl / (root + vt);
        t_cw += dl / (root - vt);
    }
    t_cw - t_ccw
}

pub struct SagnacStudy {
    pub ls: Vec<f64>,
    pub single: Vec<f64>,
    pub cluster: Vec<f64>,
    pub leading: f64,
}

pub fn sagnac_study() -> SagnacStudy {
    let (c, a) = (20.0, 0.5);
    let (sp, sg) = cluster(1, a, 1.0);
    let (cp, cg) = cluster(61, a, 1.0);
    let ls = vec![1.0, 1.5, 2.0, 3.0, 4.0];
    let single = ls.iter().map(|&l| sagnac_delay(&sp, &sg, l, c)).collect();
    let cluster = ls.iter().map(|&l| sagnac_delay(&cp, &cg, l, c)).collect();
    SagnacStudy { ls, single, cluster, leading: 2.0 / (c * c) }
}

// ---------------------------------------------------------------- E11

/// Falls a small "test body" vortex ring through the flow of a big "planet"
/// ring and returns its extra displacement caused by the planet (trajectory
/// with the planet minus free trajectory), as a vector.
fn fall(body: Vec<crate::filament::Filament>, core: f64, spacing: f64, steps: usize, dt: f64) -> DVec3 {
    let centre = |s: &VortexSystem, n_body: usize| -> DVec3 {
        let fs = &s.filaments[..n_body];
        fs.iter().map(|f| f.centroid()).sum::<DVec3>() / n_body as f64
    };
    let nb = body.len();
    // Free motion.
    let mut free = VortexSystem::new(body.clone(), core, spacing);
    free.resample = false;
    let c0 = centre(&free, nb);
    for _ in 0..steps {
        free.step(dt);
    }
    let free_d = centre(&free, nb) - c0;
    // With the planet: a large ring ahead of the body, coaxial with it.
    let planet = shapes::ring(DVec3::new(0.0, 0.0, 2.5), DVec3::Z, 3.0, 300, 4.0, [255; 3]);
    let mut fils = body;
    fils.push(planet);
    let mut with = VortexSystem::new(fils, core, spacing);
    with.resample = false;
    for _ in 0..steps {
        with.step(dt);
    }
    let with_d = centre(&with, nb) - c0;
    with_d - free_d
}

pub struct EquivalenceStudy {
    /// Extra displacement for the thin single-filament body.
    pub thin: DVec3,
    /// Extra displacement for the bundle body with the same impulse.
    pub bundle: DVec3,
    pub eta: f64,
}

pub fn equivalence_study() -> EquivalenceStudy {
    equivalence_study_with(100)
}

/// Control: two bodies with the SAME structure (thin rings at different node
/// counts). Their difference measures the numerical noise floor of the test.
pub fn equivalence_control() -> f64 {
    let (core, a) = (0.03, 0.5);
    let spacing = TAU * a / 100.0;
    let d1 = fall(hierarchical_ring(a, 1.0, 100, &[]), core, spacing, 60, 0.02);
    let d2 = fall(hierarchical_ring(a, 1.0, 140, &[]), core, spacing, 60, 0.02);
    2.0 * (d1 - d2).length() / (d1.length() + d2.length())
}

pub fn equivalence_study_with(nodes: usize) -> EquivalenceStudy {
    let core = 0.03;
    let a = 0.12;
    // Same impulse: bundle has P = Γπ(R² + a²/2), so shrink its R accordingly.
    let r_thin = 0.5f64;
    let r_bundle = (r_thin * r_thin - 0.5 * a * a).sqrt();
    let thin = hierarchical_ring(r_thin, 1.0, nodes, &[]);
    let bundle = hierarchical_ring(r_bundle, 1.0, nodes, &[Level { n: 6, radius: a, twist: 0 }]);
    let spacing = TAU * r_thin / nodes as f64;
    let (steps, dt) = (60, 0.02);
    let d_thin = fall(thin, core, spacing, steps, dt);
    let d_bundle = fall(bundle, core, spacing, steps, dt);
    let eta = 2.0 * (d_thin - d_bundle).length() / (d_thin.length() + d_bundle.length());
    EquivalenceStudy { thin: d_thin, bundle: d_bundle, eta }
}

// ---------------------------------------------------------------- checks

pub fn checks() -> Vec<Check> {
    let mut v = Vec::new();
    // E4
    let (u1, de1) = hamiltonian_pair(1.0, &[], 0.05, 800);
    let bundle = [Level { n: 16, radius: 0.075, twist: 0 }];
    let (u2, de2) = hamiltonian_pair_volume(1.0, &bundle, 0.015, 1200);
    let (_, de_wrong) = hamiltonian_pair(1.0, &bundle, 0.015, 1200);
    v.push(Check::rel("E4 thin ring: U = dE/dP", de1, u1, 0.01, "Hamiltonian vortex dynamics (Roberts 1972)"));
    v.push(Check::rel(
        "E4 16-filament bundle: U = dE/dP at fixed bundle volume",
        de2,
        u2,
        0.01,
        "Kelvin: thick cores conserve volume a²R; residual ∝ (a/R)²",
    ));
    v.push(Check::boolean(
        "E4 …but NOT at fixed bundle radius (constraint matters, >5% off)",
        (u2 - de_wrong).abs() / de_wrong > 0.05,
        "U = ∂E/∂P depends on which internal quantity is conserved",
    ));
    // E9
    let s = sagnac_study();
    let i3 = s.ls.iter().position(|&l| l == 3.0).unwrap();
    v.push(Check::rel("E9 Sagnac delay Δt → 2Γ/c² (single vortex, L=3)", s.leading, s.single[i3], 0.01, "acoustic metric, frame dragging"));
    v.push(Check::rel(
        "E9 Cluster of 61 vortices gives the same Δt as one (L=3, a=0.5)",
        s.single[i3],
        s.cluster[i3],
        1e-3,
        "long-wavelength metric is blind to microstructure",
    ));
    // E11
    let eq = equivalence_study();
    v.push(Check::abs(
        "E11 control: identical bodies fall alike (noise floor)",
        0.0,
        equivalence_control(),
        5e-3,
        "same structure must give the same fall",
    ));
    v.push(Check::abs(
        "E11 Equivalence principle: bodies of same impulse fall alike (η)",
        0.0,
        eq.eta,
        1e-3,
        "gravity needs η < 10⁻¹⁵ (MICROSCOPE); EXPECTED TO FAIL for vortex bodies",
    ));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gravity_suite_report() {
        let (u1, de1) = hamiltonian_pair(1.0, &[], 0.05, 800);
        eprintln!("E4 thin U={u1} dE/dP={de1}");
        let s = sagnac_study();
        eprintln!("E9 leading {} single {:?} cluster {:?}", s.leading, s.single, s.cluster);
        let eq = equivalence_study();
        eprintln!("E11 thin {:?} bundle {:?} eta {}", eq.thin, eq.bundle, eq.eta);
        eprintln!("E11 control {}", equivalence_control());
        for c in checks() {
            eprintln!("{} {} exp {} meas {}", if c.pass() { "PASS" } else { "FAIL" }, c.name, c.expected, c.measured);
            // Everything passes except the equivalence principle, which is the
            // physics result this suite exists to expose.
            assert_eq!(c.pass(), !c.name.starts_with("E11 Equivalence"), "{c:?}");
        }
    }
}
