//! 2D Gross–Pitaevskii superfluid ("quantum aether") in a rotating frame.
//!
//! Dimensionless (ħ = m = trap frequency = 1):
//!
//! ```text
//! i ∂ψ/∂t = [ −½∇² + ½r² + g|ψ|² − Ω L_z ] ψ,   L_z = −i (x∂_y − y∂_x)
//! ```
//!
//! Circulation around any loop is quantised in units of 2π (h/m). Rotating the
//! container nucleates a triangular (Abrikosov) vortex lattice with Feynman
//! areal density n_v = mΩ/(πħ) = Ω/π.
//!
//! Integrator: alternating-direction split-step Fourier. The rotation term is
//! split as T_x = ½p_x² + Ωy p_x (diagonal in k_x at fixed y) and
//! T_y = ½p_y² − Ωx p_y (diagonal in k_y at fixed x). Imaginary time (τ = it)
//! relaxes to the ground state; real time evolves the dynamics.

use crate::{Check, Rng};
use num_complex::Complex64 as C64;
use rustfft::{Fft, FftPlanner};
use std::f64::consts::PI;
use std::sync::Arc;

pub struct Gpe {
    pub n: usize,
    pub len: f64,
    pub g: f64,
    pub omega: f64,
    pub psi: Vec<C64>,
    pub time: f64,
    /// Optional extra potential (e.g. a stirring obstacle), indexed like psi.
    pub extra_potential: Vec<f64>,
    fwd: Arc<dyn Fft<f64>>,
    inv: Arc<dyn Fft<f64>>,
    scratch: Vec<C64>,
    col: Vec<C64>,
}

impl Gpe {
    pub fn new(n: usize, len: f64, g: f64, omega: f64, seed: u64) -> Self {
        let mut planner = FftPlanner::new();
        let fwd = planner.plan_fft_forward(n);
        let inv = planner.plan_fft_inverse(n);
        let scratch = vec![C64::new(0.0, 0.0); fwd.get_inplace_scratch_len().max(inv.get_inplace_scratch_len())];
        let mut s = Self {
            n,
            len,
            g,
            omega,
            psi: vec![C64::new(0.0, 0.0); n * n],
            time: 0.0,
            extra_potential: vec![0.0; n * n],
            fwd,
            inv,
            scratch,
            col: vec![C64::new(0.0, 0.0); n],
        };
        s.reset(seed);
        s
    }

    pub fn dx(&self) -> f64 {
        self.len / self.n as f64
    }

    pub fn coord(&self, i: usize) -> f64 {
        -0.5 * self.len + (i as f64 + 0.5) * self.dx()
    }

    fn k(&self, j: usize) -> f64 {
        let n = self.n as i64;
        let j = j as i64;
        let m = if j < n / 2 { j } else { j - n };
        2.0 * PI * m as f64 / self.len
    }

    /// Thomas–Fermi radius in the rotating frame (effective trap 1 − Ω²).
    pub fn tf_radius(&self) -> f64 {
        let w2 = (1.0 - self.omega * self.omega).max(1e-3);
        let mu = (self.g * w2 / PI).sqrt();
        (2.0 * mu / w2).sqrt()
    }

    /// Gaussian cloud with random phase/amplitude noise to seed vortices.
    pub fn reset(&mut self, seed: u64) {
        let mut rng = Rng::new(seed);
        let r0 = 0.7 * self.tf_radius();
        for j in 0..self.n {
            for i in 0..self.n {
                let (x, y) = (self.coord(i), self.coord(j));
                let amp = (-(x * x + y * y) / (2.0 * r0 * r0)).exp();
                let ph = rng.range(-0.3, 0.3);
                self.psi[j * self.n + i] = C64::from_polar(amp * (1.0 + 0.1 * rng.range(-1.0, 1.0)), ph);
            }
        }
        self.time = 0.0;
        self.normalize();
    }

    pub fn norm(&self) -> f64 {
        self.psi.iter().map(|z| z.norm_sqr()).sum::<f64>() * self.dx() * self.dx()
    }

    pub fn normalize(&mut self) {
        let s = 1.0 / self.norm().sqrt();
        for z in &mut self.psi {
            *z *= s;
        }
    }

    /// Imprint a vortex of winding `w` at (x0, y0).
    pub fn imprint_vortex(&mut self, x0: f64, y0: f64, w: i32) {
        for j in 0..self.n {
            for i in 0..self.n {
                let (x, y) = (self.coord(i) - x0, self.coord(j) - y0);
                let th = y.atan2(x) * w as f64;
                let idx = j * self.n + i;
                self.psi[idx] *= C64::from_polar(1.0, th);
            }
        }
    }

    fn potential_half_step(&mut self, h: C64) {
        let n = self.n;
        for j in 0..n {
            let y = self.coord(j);
            for i in 0..n {
                let x = self.coord(i);
                let idx = j * n + i;
                let v = 0.5 * (x * x + y * y) + self.g * self.psi[idx].norm_sqr() + self.extra_potential[idx];
                self.psi[idx] *= (h * v).exp();
            }
        }
    }

    /// One step. `imaginary = true` relaxes (dτ), else evolves in real time.
    pub fn step(&mut self, dt: f64, imaginary: bool) {
        let n = self.n;
        // h multiplies the operator in the exponent: exp(h H).
        let h = if imaginary { C64::new(-dt, 0.0) } else { C64::new(0.0, -dt) };
        self.potential_half_step(h * 0.5);
        let norm = 1.0 / n as f64;
        // T_x on rows (fixed y)
        for j in 0..n {
            let y = self.coord(j);
            let row = &mut self.psi[j * n..(j + 1) * n];
            self.fwd.process_with_scratch(row, &mut self.scratch);
            for (i, z) in row.iter_mut().enumerate() {
                let kx = {
                    let nn = n as i64;
                    let m = if (i as i64) < nn / 2 { i as i64 } else { i as i64 - nn };
                    2.0 * PI * m as f64 / self.len
                };
                *z *= (h * (0.5 * kx * kx + self.omega * y * kx)).exp() * norm;
            }
            self.inv.process_with_scratch(row, &mut self.scratch);
        }
        // T_y on columns (fixed x)
        for i in 0..n {
            let x = self.coord(i);
            for j in 0..n {
                self.col[j] = self.psi[j * n + i];
            }
            self.fwd.process_with_scratch(&mut self.col, &mut self.scratch);
            for j in 0..n {
                let ky = self.k(j);
                self.col[j] *= (h * (0.5 * ky * ky - self.omega * x * ky)).exp() * norm;
            }
            self.inv.process_with_scratch(&mut self.col, &mut self.scratch);
            for j in 0..n {
                self.psi[j * n + i] = self.col[j];
            }
        }
        self.potential_half_step(h * 0.5);
        if imaginary {
            self.normalize();
        }
        self.time += dt;
    }

    pub fn density(&self) -> Vec<f64> {
        self.psi.iter().map(|z| z.norm_sqr()).collect()
    }

    /// Detect vortices by the phase winding around each grid plaquette in
    /// regions with appreciable density. Returns (x, y, winding).
    pub fn vortices(&self, density_frac: f64) -> Vec<(f64, f64, i32)> {
        let n = self.n;
        let dens = self.density();
        let maxd = dens.iter().cloned().fold(0.0, f64::max);
        let wrap = |d: f64| {
            let mut d = d;
            while d > PI {
                d -= 2.0 * PI;
            }
            while d < -PI {
                d += 2.0 * PI;
            }
            d
        };
        let ph = |i: usize, j: usize| self.psi[j * n + i].arg();
        let mut out = Vec::new();
        for j in 0..n - 1 {
            for i in 0..n - 1 {
                let davg = 0.25 * (dens[j * n + i] + dens[j * n + i + 1] + dens[(j + 1) * n + i] + dens[(j + 1) * n + i + 1]);
                // Vortex cores are density holes; require surrounding bulk density.
                let (x, y) = (self.coord(i) + 0.5 * self.dx(), self.coord(j) + 0.5 * self.dx());
                if self.local_bulk(&dens, i, j) < density_frac * maxd {
                    continue;
                }
                let _ = davg;
                let s = wrap(ph(i + 1, j) - ph(i, j))
                    + wrap(ph(i + 1, j + 1) - ph(i + 1, j))
                    + wrap(ph(i, j + 1) - ph(i + 1, j + 1))
                    + wrap(ph(i, j) - ph(i, j + 1));
                let w = (s / (2.0 * PI)).round() as i32;
                if w != 0 {
                    out.push((x, y, w));
                }
            }
        }
        out
    }

    /// Max density in a small neighbourhood (so cores inside the bulk count).
    fn local_bulk(&self, dens: &[f64], i: usize, j: usize) -> f64 {
        let n = self.n as i64;
        let r = (1.5 / self.dx()).ceil() as i64;
        let mut m: f64 = 0.0;
        for dj in [-r, 0, r] {
            for di in [-r, 0, r] {
                let (ii, jj) = (i as i64 + di, j as i64 + dj);
                if ii >= 0 && jj >= 0 && ii < n && jj < n {
                    m = m.max(dens[(jj * n + ii) as usize]);
                }
            }
        }
        m
    }

    /// Energy in the rotating frame (diagnostic): E = ∫ ψ* H ψ with ½g|ψ|⁴.
    pub fn chemical_scale(&self) -> f64 {
        (self.g * (1.0 - self.omega * self.omega).max(1e-3) / PI).sqrt()
    }
}

/// Mean nearest-neighbour distance among points.
pub fn mean_nn_distance(pts: &[(f64, f64)]) -> f64 {
    if pts.len() < 2 {
        return f64::NAN;
    }
    let mut s = 0.0;
    for (i, a) in pts.iter().enumerate() {
        let mut best = f64::INFINITY;
        for (j, b) in pts.iter().enumerate() {
            if i != j {
                best = best.min(((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt());
            }
        }
        s += best;
    }
    s / pts.len() as f64
}

/// Analysis of a relaxed rotating condensate against Feynman's rule.
pub struct LatticeReport {
    pub total: usize,
    pub inner_count: usize,
    pub inner_radius: f64,
    pub measured_density: f64,
    pub feynman_density: f64,
    pub nn_distance: f64,
    pub triangular_nn: f64,
    pub all_unit_winding: bool,
}

pub fn analyze_lattice(gpe: &Gpe) -> LatticeReport {
    let vs = gpe.vortices(0.15);
    let r_in = 0.6 * gpe.tf_radius();
    let inner: Vec<(f64, f64)> = vs
        .iter()
        .filter(|v| (v.0 * v.0 + v.1 * v.1).sqrt() < r_in)
        .map(|v| (v.0, v.1))
        .collect();
    let area = PI * r_in * r_in;
    let measured = inner.len() as f64 / area;
    let feynman = gpe.omega / PI;
    LatticeReport {
        total: vs.len(),
        inner_count: inner.len(),
        inner_radius: r_in,
        measured_density: measured,
        feynman_density: feynman,
        nn_distance: mean_nn_distance(&inner),
        // Triangular lattice with density n: a = sqrt(2 / (√3 n)).
        triangular_nn: (2.0 / (3f64.sqrt() * feynman)).sqrt(),
        all_unit_winding: vs.iter().all(|v| v.2 == 1),
    }
}

pub fn lattice_checks(rep: &LatticeReport) -> Vec<Check> {
    vec![
        Check::rel(
            "Vortex density vs Feynman rule Ω/π",
            rep.feynman_density,
            rep.measured_density,
            0.15,
            "Feynman 1955: n_v = 2mΩ/h",
        ),
        Check::rel(
            "Nearest-neighbour spacing vs triangular lattice",
            rep.triangular_nn,
            rep.nn_distance,
            0.15,
            "Abrikosov lattice; Abo-Shaeer et al. 2001",
        ),
        Check::boolean("Every vortex carries exactly one quantum (+1)", rep.all_unit_winding, "Onsager–Feynman quantisation"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imprinted_vortex_is_detected_with_unit_charge() {
        let mut g = Gpe::new(64, 16.0, 100.0, 0.0, 1);
        for _ in 0..200 {
            g.step(0.01, true);
        }
        g.imprint_vortex(0.3, -0.2, 1);
        let vs = g.vortices(0.2);
        assert_eq!(vs.len(), 1, "{vs:?}");
        assert_eq!(vs[0].2, 1);
        assert!((vs[0].0 - 0.3).abs() < 0.5 && (vs[0].1 + 0.2).abs() < 0.5);
    }

    #[test]
    fn real_time_conserves_norm() {
        let mut g = Gpe::new(64, 16.0, 100.0, 0.3, 2);
        let n0 = g.norm();
        for _ in 0..100 {
            g.step(0.002, false);
        }
        assert!((g.norm() - n0).abs() < 1e-9);
    }

    #[test]
    #[ignore = "slow (~tens of seconds): run with --ignored"]
    fn rotating_lattice_obeys_feynman_rule() {
        let mut g = Gpe::new(192, 26.0, 1500.0, 0.85, 7);
        for _ in 0..6000 {
            g.step(0.004, true);
        }
        let rep = analyze_lattice(&g);
        for c in lattice_checks(&rep) {
            assert!(c.pass(), "{c:?} total={} inner={}", rep.total, rep.inner_count);
        }
    }
}
