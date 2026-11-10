//! Vortices made of vortices: hierarchical vortex bundles.
//!
//! A level-0 vortex ring (radius R, circulation Γ) is replaced by N₁ thinner
//! filaments of circulation Γ/N₁ spread on a tube of radius a₁ around the
//! original centreline. Each of those can be replaced by N₂ filaments on a
//! tube of radius a₂ around *it*, and so on. Total circulation is the same at
//! every level; each piece carries less and less.
//!
//! The questions this module answers numerically:
//!
//! 1. **The limit "until it becomes nothing".** As N → ∞ with Γ fixed (each
//!    sub-vortex Γ/N → 0), does the bundle behave like a single classical ring?
//!    Theory: a uniform vortex sheet on a torus is a *hollow* ring, so
//!    U_∞ = Γ/(4πR)[ln(8R/a) − 1/2], approached with an O(1/N) correction from
//!    the sub-vortices' own self-induction (∝ (Γ/N)² per filament × N).
//! 2. **Scale separation.** Far from the ring, the flow depends only on total
//!    circulation and impulse: the microstructure is invisible, with a relative
//!    difference that falls off like (a/d)².
//! 3. **Exact bookkeeping.** Impulse of a bundle: P = Γπ(R² + a²/2), exactly.

use crate::filament::{rm_ring_speed, Filament, VortexSystem};
use crate::stats::power_law_fit;
use crate::Check;
use glam::DVec3;
use rayon::prelude::*;
use std::f64::consts::{PI, TAU};

/// One level of the hierarchy: `n` sub-filaments on a tube of radius `radius`,
/// rotated by `twist` full turns around the parent as you go along it.
#[derive(Clone, Copy, Debug)]
pub struct Level {
    pub n: usize,
    pub radius: f64,
    pub twist: i32,
}

/// Rotation-minimising frame (double reflection, Wang et al. 2008) along a
/// closed polyline, with the closure holonomy spread evenly so offset curves
/// close on themselves.
fn frames(pts: &[DVec3]) -> Vec<(DVec3, DVec3, DVec3)> {
    let n = pts.len();
    let tangent = |i: usize| (pts[(i + 1) % n] - pts[(i + n - 1) % n]).normalize();
    let t0 = tangent(0);
    let helper = if t0.z.abs() < 0.9 { DVec3::Z } else { DVec3::X };
    let mut r = t0.cross(helper).normalize();
    let mut out = Vec::with_capacity(n);
    out.push((t0, r, t0.cross(r)));
    for i in 0..n {
        let (xi, xj) = (pts[i], pts[(i + 1) % n]);
        let (ti, tj) = (tangent(i), tangent((i + 1) % n));
        let v1 = xj - xi;
        let c1 = v1.dot(v1);
        if c1 < 1e-300 {
            continue;
        }
        let rl = r - (2.0 / c1) * v1.dot(r) * v1;
        let tl = ti - (2.0 / c1) * v1.dot(ti) * v1;
        let v2 = tj - tl;
        let c2 = v2.dot(v2);
        r = if c2 < 1e-300 { rl } else { rl - (2.0 / c2) * v2.dot(rl) * v2 };
        r = (r - tj * r.dot(tj)).normalize();
        if i + 1 < n {
            out.push((tj, r, tj.cross(r)));
        }
    }
    // Holonomy: angle between the transported frame at the end and the start.
    let (t_s, r_s, b_s) = out[0];
    let r_end = (r - t_s * r.dot(t_s)).normalize();
    let hol = r_end.dot(b_s).atan2(r_end.dot(r_s));
    for (i, f) in out.iter_mut().enumerate() {
        let ang = -hol * i as f64 / n as f64;
        let (t, u, v) = *f;
        let (c, s) = (ang.cos(), ang.sin());
        *f = (t, c * u + s * v, -s * u + c * v);
    }
    out
}

/// Replace one closed filament by `n` filaments on a tube around it.
pub fn bundle(parent: &Filament, level: Level) -> Vec<Filament> {
    let fr = frames(&parent.pts);
    let np = parent.pts.len();
    (0..level.n)
        .map(|k| {
            let phase = TAU * k as f64 / level.n as f64;
            let pts = parent
                .pts
                .iter()
                .zip(&fr)
                .enumerate()
                .map(|(i, (p, (_, u, v)))| {
                    let th = phase + TAU * level.twist as f64 * i as f64 / np as f64;
                    *p + level.radius * (th.cos() * *u + th.sin() * *v)
                })
                .collect();
            Filament::new(pts, parent.gamma / level.n as f64, parent.color)
        })
        .collect()
}

/// Build a hierarchical ring: a circle of radius `r` refined through `levels`.
pub fn hierarchical_ring(r: f64, gamma: f64, nodes: usize, levels: &[Level]) -> Vec<Filament> {
    let base = crate::shapes::ring(DVec3::ZERO, DVec3::Z, r, nodes, gamma, [90, 200, 255]);
    let mut fils = vec![base];
    for (depth, lv) in levels.iter().enumerate() {
        fils = fils.iter().flat_map(|f| bundle(f, *lv)).collect();
        let palette = [[90, 200, 255], [255, 140, 80], [180, 255, 120], [230, 120, 255]];
        let c = palette[(depth + 1) % palette.len()];
        for (i, f) in fils.iter_mut().enumerate() {
            let k = 0.65 + 0.35 * ((i as f64 * 0.618).fract());
            f.color = [(c[0] as f64 * k) as u8, (c[1] as f64 * k) as u8, (c[2] as f64 * k) as u8];
        }
    }
    fils
}

/// Hollow-core Kelvin ring speed, U = Γ/(4πR)[ln(8R/a) − 1/2]: the N → ∞ limit
/// of a ring made of N filaments spread over a tube of radius a.
pub fn hollow_ring_speed(gamma: f64, r: f64, a: f64) -> f64 {
    gamma / (4.0 * PI * r) * ((8.0 * r / a).ln() - 0.5)
}

/// Mean axial (self-propulsion) velocity of a bundle ring made of `n` filaments
/// on a tube of radius `a`, each with Rosenhead–Moore core `core`.
/// Evaluated at one node per filament (the configuration is axisymmetric).
pub fn bundle_ring_speed(r: f64, gamma: f64, n: usize, a: f64, core: f64, nodes: usize) -> f64 {
    let fils = hierarchical_ring(r, gamma, nodes, &[Level { n, radius: a, twist: 0 }]);
    let samples: Vec<DVec3> = fils.iter().map(|f| f.pts[0]).collect();
    let sys = VortexSystem::new(fils, core, TAU * r / nodes as f64);
    let vz: Vec<f64> = samples.par_iter().map(|p| sys.velocity_at(*p).z).collect();
    vz.iter().sum::<f64>() / vz.len() as f64
}

/// Total impulse of a bundle ring; exact value Γπ(R² + a²/2) for one level.
pub fn bundle_impulse(r: f64, gamma: f64, n: usize, a: f64, nodes: usize) -> f64 {
    hierarchical_ring(r, gamma, nodes, &[Level { n, radius: a, twist: 0 }])
        .iter()
        .map(|f| f.impulse().z)
        .sum()
}

/// Radius of the plain ring with the same impulse as a hierarchical ring:
/// every level adds a²/2 to the mean squared radius.
pub fn equivalent_radius(r: f64, levels: &[Level]) -> f64 {
    (r * r + levels.iter().map(|l| 0.5 * l.radius * l.radius).sum::<f64>()).sqrt()
}

/// Relative difference between the far-field velocity of a hierarchical ring
/// and a plain ring of radius `plain_r`, averaged over a sphere of radius `d`.
pub fn far_field_difference(r: f64, plain_r: f64, levels: &[Level], d: f64, nodes: usize) -> f64 {
    let plain = VortexSystem::new(hierarchical_ring(plain_r, 1.0, nodes, &[]), 0.02, TAU * r / nodes as f64);
    let fine = VortexSystem::new(hierarchical_ring(r, 1.0, nodes, levels), 0.02, TAU * r / nodes as f64);
    // Points on a sphere (Fibonacci lattice).
    let m = 24;
    let pts: Vec<DVec3> = (0..m)
        .map(|i| {
            let z = 1.0 - 2.0 * (i as f64 + 0.5) / m as f64;
            let rho = (1.0 - z * z).sqrt();
            let phi = i as f64 * PI * (3.0 - 5f64.sqrt());
            d * DVec3::new(rho * phi.cos(), rho * phi.sin(), z)
        })
        .collect();
    let (num, den) = pts
        .par_iter()
        .map(|p| {
            let (u0, u1) = (plain.velocity_at(*p), fine.velocity_at(*p));
            ((u1 - u0).length(), u0.length())
        })
        .reduce(|| (0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1));
    num / den
}

/// Results of the "vortex made of smaller vortices" study.
pub struct HierarchyStudy {
    pub ns: Vec<usize>,
    pub speeds: Vec<f64>,
    pub hollow: f64,
    pub conv_exponent: f64,
    pub far_d: Vec<f64>,
    pub far_diff: Vec<f64>,
    pub far_exponent: f64,
    /// Far-field difference against a plain ring of the SAME radius (no impulse matching).
    pub naive_far_diff: f64,
    pub impulse_excess: f64,
}

pub const STUDY_R: f64 = 1.0;
pub const STUDY_A: f64 = 0.1;
pub const STUDY_CORE: f64 = 0.004;

pub fn study() -> HierarchyStudy {
    let (r, a, core) = (STUDY_R, STUDY_A, STUDY_CORE);
    let nodes = 8000; // ds ≈ 8e-4 ≪ δ, so each sub-filament is resolved.
    let ns = vec![4, 8, 16, 32, 64];
    let speeds: Vec<f64> = ns.iter().map(|&n| bundle_ring_speed(r, 1.0, n, a, core, nodes)).collect();
    let hollow = hollow_ring_speed(1.0, r, a);
    let devs: Vec<f64> = speeds.iter().map(|u| (u - hollow).abs()).collect();
    let nsf: Vec<f64> = ns.iter().map(|&n| n as f64).collect();
    // Successive differences shrink faster than 1/N (there is a ln N factor),
    // so record the effective exponent of |U_N − U_{2N}|.
    let steps: Vec<f64> = speeds.windows(2).map(|w| (w[0] - w[1]).abs()).collect();
    let conv_exponent = power_law_fit(&nsf[..steps.len()], &steps).1;
    let _ = devs;
    let levels = [Level { n: 6, radius: 0.12, twist: 1 }, Level { n: 4, radius: 0.03, twist: 2 }];
    let far_d = vec![3.0, 4.0, 6.0, 8.0];
    let r_eq = equivalent_radius(r, &levels);
    let far_diff: Vec<f64> = far_d.iter().map(|&d| far_field_difference(r, r_eq, &levels, d, 240)).collect();
    let far_exponent = power_law_fit(&far_d, &far_diff).1;
    let naive_far_diff = far_field_difference(r, r, &levels, 8.0, 240);
    let impulse_excess = (r_eq * r_eq - r * r) / (r * r);
    HierarchyStudy { ns, speeds, hollow, conv_exponent, far_d, far_diff, far_exponent, naive_far_diff, impulse_excess }
}

pub fn checks_from(s: &HierarchyStudy) -> Vec<Check> {
    let last = *s.speeds.last().unwrap();
    let p_exact = PI * (1.0 + 0.5 * STUDY_A * STUDY_A);
    let p_num = bundle_impulse(1.0, 1.0, 16, STUDY_A, 2000);
    vec![
        Check::rel(
            "N=64 sub-vortices → hollow classical ring speed",
            s.hollow,
            last,
            0.01,
            "Kelvin/Lamb hollow ring: Γ/(4πR)[ln(8R/a) − ½]",
        ),
        Check::boolean(
            "Converges as sub-vortices shrink (|U₆₄−U₃₂| < |U₈−U₄|/20)",
            (s.speeds[4] - s.speeds[3]).abs() < (s.speeds[1] - s.speeds[0]).abs() / 20.0,
            "sub-vortex self-induction ∝ N·(Γ/N)²·ln(…) → 0",
        ),
        Check::boolean(
            "Single filament (N=1 limit) differs from the hollow ring",
            (rm_ring_speed(1.0, 1.0, STUDY_CORE) - s.hollow).abs() / s.hollow > 0.05,
            "the bundle is NOT just a thin ring: its tube size matters",
        ),
        Check::rel("Bundle impulse = Γπ(R² + a²/2)", p_exact, p_num, 1e-4, "exact for a circular tube of filaments"),
        Check::rel(
            "Microstructure adds impulse a²/2R² ('internal energy has weight')",
            s.impulse_excess,
            s.naive_far_diff,
            0.1,
            "P = Γπ(R² + Σa²/2): same-R comparison differs by exactly this",
        ),
        Check::abs(
            "Far field blind to 2-level microstructure (same impulse, d = 3–8R)",
            0.0,
            s.far_diff.iter().cloned().fold(0.0, f64::max),
            1e-4,
            "multipole expansion: only Γ and impulse matter far away",
        ),
        Check::boolean(
            "Matching impulse removes >99% of the far-field difference",
            s.far_diff.last().unwrap() * 100.0 < s.naive_far_diff,
            "impulse (dipole moment) is the only large-scale imprint",
        ),
    ]
}

/// Two clusters of `n` point vortices (Γ/n each, sunflower disk of radius `a`),
/// centres 1 apart. Returns the orbital angular velocity of the line joining
/// the clusters' centres of vorticity. Classical prediction for two Γ = 1
/// vortices: Ω = 2/(2π) = 1/π, whatever the clusters are made of.
pub fn cluster_pair_omega(n: usize, a: f64) -> f64 {
    use crate::pointvortex::PointVortices;
    use glam::DVec2;
    let golden = PI * (3.0 - 5f64.sqrt());
    let mut pos = Vec::new();
    let mut gam = Vec::new();
    for c in [-0.5, 0.5] {
        for k in 0..n {
            let rr = if n == 1 { 0.0 } else { a * ((k as f64 + 0.5) / n as f64).sqrt() };
            let th = k as f64 * golden;
            pos.push(DVec2::new(c + rr * th.cos(), rr * th.sin()));
            gam.push(1.0 / n as f64);
        }
    }
    let mut pv = PointVortices::new(pos, gam);
    pv.core = 0.2 * a / (n as f64).sqrt().max(1.0); // tiny smoothing, below the inter-vortex spacing
    let centre = |pv: &PointVortices, side: usize| -> DVec2 {
        let s = &pv.pos[side * n..(side + 1) * n];
        s.iter().copied().sum::<DVec2>() / n as f64
    };
    let dt = 0.002;
    let steps = 1500;
    let mut angle = 0.0;
    let mut last = {
        let d = centre(&pv, 1) - centre(&pv, 0);
        d.y.atan2(d.x)
    };
    for _ in 0..steps {
        pv.step(dt);
        let d = centre(&pv, 1) - centre(&pv, 0);
        let a2 = d.y.atan2(d.x);
        let mut da = a2 - last;
        if da > PI {
            da -= TAU;
        }
        if da < -PI {
            da += TAU;
        }
        angle += da;
        last = a2;
    }
    angle / pv.time
}

/// Quantum → classical: point-vortex clusters behave like single vortices.
pub fn cluster_checks() -> Vec<Check> {
    let target = 1.0 / PI;
    let w1 = cluster_pair_omega(1, 0.1);
    let w64 = cluster_pair_omega(64, 0.1);
    vec![
        Check::rel("Single vortices orbit at Ω = 1/π", target, w1, 1e-3, "Kirchhoff"),
        Check::rel(
            "Clusters of 64 tiny vortices orbit like single vortices",
            target,
            w64,
            0.01,
            "coarse-graining: only total Γ matters at separation ≫ cluster size",
        ),
    ]
}

pub fn checks() -> Vec<Check> {
    let mut v = checks_from(&study());
    v.extend(cluster_checks());
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_conserves_circulation_and_closes() {
        let fils = hierarchical_ring(1.0, 1.0, 200, &[Level { n: 5, radius: 0.1, twist: 1 }, Level { n: 3, radius: 0.02, twist: 2 }]);
        assert_eq!(fils.len(), 15);
        let total: f64 = fils.iter().map(|f| f.gamma).sum();
        assert!((total - 1.0).abs() < 1e-12);
        for f in &fils {
            // Closed curves: last node is next to the first.
            let gap = (f.pts[0] - *f.pts.last().unwrap()).length();
            let ds = (f.pts[1] - f.pts[0]).length();
            assert!(gap < 3.0 * ds, "gap {gap} ds {ds}");
        }
    }

    #[test]
    fn hierarchy_suite() {
        let s = study();
        eprintln!("speeds {:?} hollow {} exp {} far {:?} exp {}", s.speeds, s.hollow, s.conv_exponent, s.far_diff, s.far_exponent);
        for c in checks_from(&s).into_iter().chain(cluster_checks()) {
            assert!(c.pass(), "{c:?}");
        }
    }
}
