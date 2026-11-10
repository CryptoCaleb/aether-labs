//! Lagrangian vortex-filament dynamics in an ideal incompressible medium.
//!
//! Each vortex tube is a closed polygonal curve carrying circulation Γ. The
//! velocity it induces anywhere follows the Biot–Savart law, regularised with
//! the Rosenhead–Moore core δ:
//!
//! ```text
//! u(x) = Γ/4π ∮ dl × (x − l) / (|x − l|² + δ²)^{3/2}
//! ```
//!
//! Nodes are advected with the local velocity (Helmholtz: vortex lines are
//! material lines), integrated with RK4, and the curves are resampled to keep
//! segment lengths near a target spacing.
//!
//! For a thin circular ring of radius R this model gives the self-induced speed
//! U = Γ/(4πR) [ln(8R/δ) − 1], which corresponds to Kelvin's solid-core ring
//! U = Γ/(4πR) [ln(8R/a) − 1/4] with core radius a = δ·e^{3/4}.

use glam::DVec3;
use rayon::prelude::*;
use std::f64::consts::PI;

#[derive(Clone, Debug)]
pub struct Filament {
    /// Closed curve: the last node connects back to the first.
    pub pts: Vec<DVec3>,
    pub gamma: f64,
    pub color: [u8; 3],
}

impl Filament {
    pub fn new(pts: Vec<DVec3>, gamma: f64, color: [u8; 3]) -> Self {
        Self { pts, gamma, color }
    }

    pub fn n(&self) -> usize {
        self.pts.len()
    }

    pub fn segment(&self, i: usize) -> (DVec3, DVec3) {
        (self.pts[i], self.pts[(i + 1) % self.pts.len()])
    }

    pub fn length(&self) -> f64 {
        (0..self.n())
            .map(|i| {
                let (a, b) = self.segment(i);
                (b - a).length()
            })
            .sum()
    }

    /// Length-weighted centroid of the curve.
    pub fn centroid(&self) -> DVec3 {
        let mut c = DVec3::ZERO;
        let mut w = 0.0;
        for i in 0..self.n() {
            let (a, b) = self.segment(i);
            let l = (b - a).length();
            c += 0.5 * (a + b) * l;
            w += l;
        }
        if w > 0.0 { c / w } else { DVec3::ZERO }
    }

    /// Hydrodynamic impulse P = (Γ/2) ∮ x × dl. Conserved for an isolated
    /// system in an unbounded ideal fluid.
    pub fn impulse(&self) -> DVec3 {
        let mut p = DVec3::ZERO;
        for i in 0..self.n() {
            let (a, b) = self.segment(i);
            p += (0.5 * (a + b)).cross(b - a);
        }
        0.5 * self.gamma * p
    }

    /// Effective ring radius from the impulse: |P| = Γ π R².
    pub fn impulse_radius(&self) -> f64 {
        (self.impulse().length() / (self.gamma.abs() * PI)).sqrt()
    }

    /// Mean distance of the nodes from the centroid.
    pub fn mean_radius(&self) -> f64 {
        let c = self.centroid();
        self.pts.iter().map(|p| (*p - c).length()).sum::<f64>() / self.n() as f64
    }
}

/// Velocity induced at `p` by the straight segment a→b with circulation Γ,
/// using the exact Rosenhead–Moore regularisation of the straight-segment
/// Biot–Savart integral.
#[inline]
pub fn segment_velocity(p: DVec3, a: DVec3, b: DVec3, gamma: f64, delta2: f64) -> DVec3 {
    let r0 = b - a;
    let r1 = p - a;
    let r2 = p - b;
    let c = r1.cross(r2);
    let denom = c.length_squared() + delta2 * r0.length_squared();
    if denom < 1e-300 {
        return DVec3::ZERO;
    }
    let n1 = (r1.length_squared() + delta2).sqrt();
    let n2 = (r2.length_squared() + delta2).sqrt();
    let k = gamma / (4.0 * PI) * r0.dot(r1 / n1 - r2 / n2) / denom;
    c * k
}

#[derive(Clone, Copy)]
struct Seg {
    a: DVec3,
    b: DVec3,
    gamma: f64,
}

/// A collection of vortex filaments in an unbounded ideal medium.
#[derive(Clone, Debug)]
pub struct VortexSystem {
    pub filaments: Vec<Filament>,
    /// Rosenhead–Moore core parameter δ.
    pub core: f64,
    /// Target node spacing for resampling.
    pub spacing: f64,
    /// Uniform background flow of the medium ("aether wind").
    pub background: DVec3,
    pub time: f64,
    pub resample: bool,
    /// Per-step Laplacian smoothing weight (0 = off). Damps grid-scale
    /// sawtooth modes like a small numerical viscosity; it does not change the
    /// impulse of a closed curve to first order.
    pub smoothing: f64,
}

impl VortexSystem {
    pub fn new(filaments: Vec<Filament>, core: f64, spacing: f64) -> Self {
        Self {
            filaments,
            core,
            spacing,
            background: DVec3::ZERO,
            time: 0.0,
            resample: true,
            smoothing: 0.0,
        }
    }

    pub fn node_count(&self) -> usize {
        self.filaments.iter().map(|f| f.n()).sum()
    }

    fn flatten(&self) -> Vec<DVec3> {
        self.filaments.iter().flat_map(|f| f.pts.iter().copied()).collect()
    }

    fn segments_of(&self, state: &[DVec3]) -> Vec<Seg> {
        let mut segs = Vec::with_capacity(state.len());
        let mut off = 0;
        for f in &self.filaments {
            let n = f.n();
            for i in 0..n {
                segs.push(Seg {
                    a: state[off + i],
                    b: state[off + (i + 1) % n],
                    gamma: f.gamma,
                });
            }
            off += n;
        }
        segs
    }

    fn velocity_from(segs: &[Seg], p: DVec3, delta2: f64) -> DVec3 {
        let mut u = DVec3::ZERO;
        for s in segs {
            u += segment_velocity(p, s.a, s.b, s.gamma, delta2);
        }
        u
    }

    /// Velocity of the medium at an arbitrary point.
    pub fn velocity_at(&self, p: DVec3) -> DVec3 {
        let segs = self.segments_of(&self.flatten());
        self.background + Self::velocity_from(&segs, p, self.core * self.core)
    }

    fn node_velocities(&self, state: &[DVec3]) -> Vec<DVec3> {
        let segs = self.segments_of(state);
        let d2 = self.core * self.core;
        let bg = self.background;
        state
            .par_iter()
            .map(|&p| bg + Self::velocity_from(&segs, p, d2))
            .collect()
    }

    /// Velocities of every node, filament by filament.
    pub fn velocities(&self) -> Vec<Vec<DVec3>> {
        let v = self.node_velocities(&self.flatten());
        let mut out = Vec::new();
        let mut off = 0;
        for f in &self.filaments {
            out.push(v[off..off + f.n()].to_vec());
            off += f.n();
        }
        out
    }

    /// Advance one RK4 step of size `dt`.
    pub fn step(&mut self, dt: f64) {
        let y0 = self.flatten();
        let add = |y: &[DVec3], k: &[DVec3], h: f64| -> Vec<DVec3> {
            y.iter().zip(k).map(|(a, b)| *a + *b * h).collect()
        };
        let k1 = self.node_velocities(&y0);
        let k2 = self.node_velocities(&add(&y0, &k1, 0.5 * dt));
        let k3 = self.node_velocities(&add(&y0, &k2, 0.5 * dt));
        let k4 = self.node_velocities(&add(&y0, &k3, dt));
        let mut off = 0;
        for f in &mut self.filaments {
            for (i, p) in f.pts.iter_mut().enumerate() {
                let j = off + i;
                *p = y0[j] + (k1[j] + 2.0 * k2[j] + 2.0 * k3[j] + k4[j]) * (dt / 6.0);
            }
            off += f.n();
        }
        self.time += dt;
        if self.smoothing > 0.0 {
            let w = self.smoothing;
            for f in &mut self.filaments {
                let n = f.pts.len();
                let old = f.pts.clone();
                for i in 0..n {
                    f.pts[i] = old[i] + w * (old[(i + n - 1) % n] + old[(i + 1) % n] - 2.0 * old[i]);
                }
            }
        }
        if self.resample {
            self.resample_all();
        }
    }

    /// Keep node spacing within [0.5, 1.5] × target by Catmull–Rom insertion
    /// and node removal.
    pub fn resample_all(&mut self) {
        let h = self.spacing;
        for f in &mut self.filaments {
            let n = f.pts.len();
            if n < 4 {
                continue;
            }
            let p = &f.pts;
            let mut out = Vec::with_capacity(n + 8);
            let mut i = 0;
            while i < n {
                let a = p[i];
                let b = p[(i + 1) % n];
                out.push(a);
                let l = (b - a).length();
                if l > 1.5 * h {
                    let p0 = p[(i + n - 1) % n];
                    let p3 = p[(i + 2) % n];
                    out.push(0.5625 * (a + b) - 0.0625 * (p0 + p3));
                } else if l < 0.5 * h && n - (i + 1 - out.len().min(i + 1)) > 8 && i + 1 < n {
                    // Skip node i+1 (merge the short segment).
                    i += 1;
                }
                i += 1;
            }
            if out.len() >= 8 {
                f.pts = out;
            }
        }
    }

    /// Total hydrodynamic impulse.
    pub fn impulse(&self) -> DVec3 {
        self.filaments.iter().map(|f| f.impulse()).sum()
    }

    /// Regularised kinetic energy
    /// E = 1/(8π) Σ Γ_i Γ_j ∮∮ dl_i·dl_j / sqrt(|x_i − x_j|² + δ²)
    /// (midpoint rule; the Hamiltonian of Rosenhead–Moore dynamics).
    pub fn energy(&self) -> f64 {
        let segs = self.segments_of(&self.flatten());
        let d2 = self.core * self.core;
        let e: f64 = segs
            .par_iter()
            .map(|s| {
                let m = 0.5 * (s.a + s.b);
                let dl = s.b - s.a;
                let mut acc = 0.0;
                for t in &segs {
                    let m2 = 0.5 * (t.a + t.b);
                    acc += t.gamma * dl.dot(t.b - t.a) / ((m - m2).length_squared() + d2).sqrt();
                }
                s.gamma * acc
            })
            .sum();
        e / (8.0 * PI)
    }
}

/// Gauss linking number of two closed curves (midpoint quadrature).
pub fn linking_number(a: &Filament, b: &Filament) -> f64 {
    let mut s = 0.0;
    for i in 0..a.n() {
        let (a0, a1) = a.segment(i);
        let ma = 0.5 * (a0 + a1);
        let da = a1 - a0;
        for j in 0..b.n() {
            let (b0, b1) = b.segment(j);
            let mb = 0.5 * (b0 + b1);
            let db = b1 - b0;
            let r = ma - mb;
            let r3 = r.length().powi(3).max(1e-300);
            s += r.dot(da.cross(db)) / r3;
        }
    }
    s / (4.0 * PI)
}

/// Writhe of a single closed curve (Gauss double integral, i ≠ j).
pub fn writhe(f: &Filament) -> f64 {
    let n = f.n();
    let mut s = 0.0;
    for i in 0..n {
        let (a0, a1) = f.segment(i);
        let ma = 0.5 * (a0 + a1);
        let da = a1 - a0;
        for j in 0..n {
            if i == j {
                continue;
            }
            let (b0, b1) = f.segment(j);
            let mb = 0.5 * (b0 + b1);
            let r = ma - mb;
            s += r.dot(da.cross(b1 - b0)) / r.length().powi(3).max(1e-300);
        }
    }
    s / (4.0 * PI)
}

/// Kelvin/Rosenhead–Moore prediction for a thin ring's self-induced speed.
pub fn rm_ring_speed(gamma: f64, radius: f64, core: f64) -> f64 {
    gamma / (4.0 * PI * radius) * ((8.0 * radius / core).ln() - 1.0)
}

/// Kelvin (1867) speed of a thin ring with a uniform-vorticity core of radius a.
pub fn kelvin_ring_speed(gamma: f64, radius: f64, a: f64) -> f64 {
    gamma / (4.0 * PI * radius) * ((8.0 * radius / a).ln() - 0.25)
}

/// Mean self-induced axial speed of a discretised ring.
pub fn measured_ring_speed(radius: f64, core: f64, nodes: usize) -> f64 {
    let ring = crate::shapes::ring(DVec3::ZERO, DVec3::Z, radius, nodes, 1.0, [0; 3]);
    let sys = VortexSystem::new(vec![ring], core, 2.0 * PI * radius / nodes as f64);
    let v = sys.velocities();
    v[0].iter().map(|u| u.z).sum::<f64>() / v[0].len() as f64
}

/// Validation suite for the filament engine.
pub fn checks() -> Vec<crate::Check> {
    use crate::Check;
    let core = 0.05;
    let u1 = measured_ring_speed(1.0, core, 600);
    let u2 = measured_ring_speed(2.0, core, 1200);
    // Impulse/energy conservation and topology during a leapfrog run.
    let a = crate::shapes::ring(DVec3::ZERO, DVec3::Z, 1.0, 120, 1.0, [0; 3]);
    let b = crate::shapes::ring(DVec3::new(0.0, 0.0, 0.6), DVec3::Z, 1.0, 120, 1.0, [0; 3]);
    let mut sys = VortexSystem::new(vec![a, b], 0.1, 2.0 * PI / 120.0);
    let (p0, e0) = (sys.impulse(), sys.energy());
    for _ in 0..80 {
        sys.step(0.05);
    }
    let (p1, e1) = (sys.impulse(), sys.energy());
    // Hopf link keeps its linking number under evolution (no reconnection).
    let la = crate::shapes::ring(DVec3::ZERO, DVec3::Z, 1.0, 120, 1.0, [0; 3]);
    let lb = crate::shapes::ring(DVec3::new(1.0, 0.0, 0.0), DVec3::Y, 1.0, 120, 1.0, [0; 3]);
    let mut link = VortexSystem::new(vec![la, lb], 0.1, 2.0 * PI / 120.0);
    for _ in 0..40 {
        link.step(0.05);
    }
    let lk = linking_number(&link.filaments[0], &link.filaments[1]).abs();
    vec![
        Check::rel("Ring speed vs Γ/(4πR)[ln(8R/δ) − 1]", rm_ring_speed(1.0, 1.0, core), u1, 0.01, "Kelvin 1867 / Saffman (RM core)"),
        Check::rel(
            "Same speed as Kelvin solid core a = δe^{3/4}",
            kelvin_ring_speed(1.0, 1.0, core * 0.75f64.exp()),
            u1,
            0.01,
            "Kelvin 1867: ln(8R/a) − 1/4",
        ),
        Check::boolean("Bigger ring is slower (U(2R) < U(R))", u2 < u1, "vortex rings: 'negative effective mass'"),
        Check::abs("Impulse conserved during leapfrogging (rel. drift)", 0.0, (p1 - p0).length() / p0.length(), 1e-3, "translation symmetry"),
        Check::abs("Energy conserved during leapfrogging (rel. drift)", 0.0, (e1 - e0).abs() / e0.abs(), 1e-2, "Hamiltonian RM dynamics"),
        Check::rel("Hopf link stays linked (|Lk| = 1)", 1.0, lk, 0.03, "Helmholtz: topology frozen in ideal flow"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes;

    #[test]
    fn filament_suite() {
        for c in checks() {
            assert!(c.pass(), "{c:?}");
        }
    }

    #[test]
    fn infinite_line_limit() {
        // A long straight segment approximates an infinite line: |u| = Γ/(2πd).
        let u = segment_velocity(
            DVec3::new(1.0, 0.0, 0.0),
            DVec3::new(0.0, 0.0, -1e4),
            DVec3::new(0.0, 0.0, 1e4),
            1.0,
            0.0,
        );
        assert!((u.length() - 1.0 / (2.0 * PI)).abs() < 1e-6);
        // Right-hand rule: vorticity +z, point at +x → flow along +y.
        assert!(u.y > 0.0);
    }

    #[test]
    fn ring_speed_matches_kelvin() {
        let (r, core) = (1.0, 0.1);
        let ring = shapes::ring(DVec3::ZERO, DVec3::Z, r, 400, 1.0, [255, 255, 255]);
        let sys = VortexSystem::new(vec![ring], core, 2.0 * PI * r / 400.0);
        let v = sys.velocities();
        let mean_vz = v[0].iter().map(|u| u.z).sum::<f64>() / v[0].len() as f64;
        let expected = rm_ring_speed(1.0, r, core);
        let err = (mean_vz - expected).abs() / expected;
        assert!(err < 0.02, "vz={mean_vz} expected={expected} err={err}");
    }

    #[test]
    fn ring_translates_and_conserves() {
        let ring = shapes::ring(DVec3::ZERO, DVec3::Z, 1.0, 160, 1.0, [255, 255, 255]);
        let mut sys = VortexSystem::new(vec![ring], 0.15, 2.0 * PI / 160.0);
        let p0 = sys.impulse();
        let e0 = sys.energy();
        let z0 = sys.filaments[0].centroid().z;
        for _ in 0..50 {
            sys.step(0.05);
        }
        let speed = (sys.filaments[0].centroid().z - z0) / sys.time;
        let expected = rm_ring_speed(1.0, 1.0, 0.15);
        assert!((speed - expected).abs() / expected < 0.03);
        assert!((sys.impulse() - p0).length() / p0.length() < 1e-3);
        assert!((sys.energy() - e0).abs() / e0.abs() < 1e-2);
    }

    #[test]
    fn hopf_link_has_linking_number_one() {
        let a = shapes::ring(DVec3::ZERO, DVec3::Z, 1.0, 200, 1.0, [0; 3]);
        let b = shapes::ring(DVec3::new(1.0, 0.0, 0.0), DVec3::Y, 1.0, 200, 1.0, [0; 3]);
        assert!((linking_number(&a, &b).abs() - 1.0).abs() < 0.02);
        let c = shapes::ring(DVec3::new(5.0, 0.0, 0.0), DVec3::Y, 1.0, 200, 1.0, [0; 3]);
        assert!(linking_number(&a, &c).abs() < 0.02);
    }
}
