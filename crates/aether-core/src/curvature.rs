//! Numerical spacetime curvature: does an emergent (acoustic) metric obey
//! Einstein's equations? (E10)
//!
//! For any metric g_μν(x) given as a function, we compute Christoffel symbols
//! and the Ricci tensor by 4th-order finite differences. In vacuum, Einstein's
//! equations reduce to R_μν = 0. The engine is validated on the Schwarzschild
//! black hole in Painlevé–Gullstrand form, ds² = −dt² + (dx − v dt)² with
//! v = −√(2M/r) r̂. That metric IS a "river" of space flowing into the hole,
//! and it is Ricci-flat.
//!
//! Then the same test is run on metrics a real fluid can make:
//!
//! * **Constant-density sink.** Continuity forces v ∝ 1/r², so the acoustic
//!   metric has the right form but the wrong profile. It gives "gravity" ∝ 1/r⁵,
//!   not 1/r², and R_μν ≠ 0 in empty space.
//! * **Schwarzschild-profile fluid.** v = √(2M/r) needs ρ ∝ r^(−3/2) by
//!   continuity. The acoustic metric carries the conformal factor ρ/c, and that
//!   factor alone makes R_μν ≠ 0. This is Visser's observation that a fluid
//!   reproduces Schwarzschild only up to a conformal factor.

use crate::Check;

pub type Metric = dyn Fn([f64; 4]) -> [[f64; 4]; 4] + Sync;

fn inv4(m: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut a = [[0.0; 8]; 4];
    for i in 0..4 {
        for j in 0..4 {
            a[i][j] = m[i][j];
        }
        a[i][4 + i] = 1.0;
    }
    for c in 0..4 {
        let p = (c..4).max_by(|&x, &y| a[x][c].abs().partial_cmp(&a[y][c].abs()).unwrap()).unwrap();
        a.swap(c, p);
        let d = a[c][c];
        for j in 0..8 {
            a[c][j] /= d;
        }
        for r in 0..4 {
            if r != c {
                let f = a[r][c];
                for j in 0..8 {
                    a[r][j] -= f * a[c][j];
                }
            }
        }
    }
    let mut out = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            out[i][j] = a[i][4 + j];
        }
    }
    out
}

fn shift(x: [f64; 4], a: usize, d: f64) -> [f64; 4] {
    let mut y = x;
    y[a] += d;
    y
}

/// 4th-order central difference of a 4×4 field along coordinate a.
fn d_metric(g: &Metric, x: [f64; 4], a: usize, h: f64) -> [[f64; 4]; 4] {
    let (p1, m1, p2, m2) = (g(shift(x, a, h)), g(shift(x, a, -h)), g(shift(x, a, 2.0 * h)), g(shift(x, a, -2.0 * h)));
    let mut out = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            out[i][j] = (8.0 * (p1[i][j] - m1[i][j]) - (p2[i][j] - m2[i][j])) / (12.0 * h);
        }
    }
    out
}

/// Christoffel symbols Γ^a_bc at x.
pub fn christoffel(g: &Metric, x: [f64; 4], h: f64) -> [[[f64; 4]; 4]; 4] {
    let gi = inv4(g(x));
    let dg: Vec<[[f64; 4]; 4]> = (0..4).map(|a| d_metric(g, x, a, h)).collect();
    let mut gam = [[[0.0; 4]; 4]; 4];
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                let mut s = 0.0;
                for d in 0..4 {
                    s += gi[a][d] * (dg[b][d][c] + dg[c][d][b] - dg[d][b][c]);
                }
                gam[a][b][c] = 0.5 * s;
            }
        }
    }
    gam
}

/// Ricci tensor R_bd and a curvature scale (max |∂Γ|) at x.
pub fn ricci(g: &Metric, x: [f64; 4], h: f64) -> ([[f64; 4]; 4], f64) {
    let gam = christoffel(g, x, h);
    // ∂_e Γ^a_bc by 4th-order differences of Γ.
    let mut dgam = [[[[0.0; 4]; 4]; 4]; 4]; // [e][a][b][c]
    let mut scale: f64 = 0.0;
    for e in 0..4 {
        let (p1, m1) = (christoffel(g, shift(x, e, h), h), christoffel(g, shift(x, e, -h), h));
        let (p2, m2) = (christoffel(g, shift(x, e, 2.0 * h), h), christoffel(g, shift(x, e, -2.0 * h), h));
        for a in 0..4 {
            for b in 0..4 {
                for c in 0..4 {
                    let v = (8.0 * (p1[a][b][c] - m1[a][b][c]) - (p2[a][b][c] - m2[a][b][c])) / (12.0 * h);
                    dgam[e][a][b][c] = v;
                    scale = scale.max(v.abs());
                }
            }
        }
    }
    let mut r = [[0.0; 4]; 4];
    for b in 0..4 {
        for d in 0..4 {
            let mut s = 0.0;
            for a in 0..4 {
                s += dgam[a][a][b][d] - dgam[d][a][b][a];
                for e in 0..4 {
                    s += gam[a][a][e] * gam[e][b][d] - gam[a][d][e] * gam[e][b][a];
                }
            }
            r[b][d] = s;
        }
    }
    (r, scale)
}

/// Max |R_μν| relative to the curvature scale, over sample points at radius r
/// (several directions). ~0 means Ricci-flat (Einstein vacuum).
pub fn ricci_ratio(g: &Metric, r: f64) -> f64 {
    let dirs = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.577, 0.577, 0.577], [0.6, -0.8, 0.0]];
    let h = 1e-3 * r;
    let mut worst: f64 = 0.0;
    for d in dirs {
        let x = [0.0, r * d[0], r * d[1], r * d[2]];
        let (ric, scale) = ricci(g, x, h);
        let m = ric.iter().flatten().fold(0.0f64, |a, v| a.max(v.abs()));
        worst = worst.max(m / scale.max(1e-300));
    }
    worst
}

/// Acoustic / Painlevé–Gullstrand metric for a radial flow v(r) r̂ with sound
/// speed c(r) and conformal factor Ω²(r) = ρ/c:
/// ds² = Ω² [ −c² dt² + (dx − v dt)² ].
pub fn radial_flow_metric(v: impl Fn(f64) -> f64 + Sync + 'static, c: impl Fn(f64) -> f64 + Sync + 'static, conformal: impl Fn(f64) -> f64 + Sync + 'static) -> Box<Metric> {
    Box::new(move |x: [f64; 4]| {
        let r = (x[1] * x[1] + x[2] * x[2] + x[3] * x[3]).sqrt();
        let vr = v(r);
        let vi = [vr * x[1] / r, vr * x[2] / r, vr * x[3] / r];
        let (cc, w) = (c(r), conformal(r));
        let mut g = [[0.0; 4]; 4];
        g[0][0] = w * (-(cc * cc) + vr * vr);
        for i in 0..3 {
            g[0][i + 1] = -w * vi[i];
            g[i + 1][0] = -w * vi[i];
            g[i + 1][i + 1] = w;
        }
        g
    })
}

pub struct EinsteinStudy {
    pub schwarzschild: f64,
    pub flat: f64,
    pub const_density_sink: f64,
    pub schwarzschild_profile_fluid: f64,
}

pub fn einstein_study() -> EinsteinStudy {
    let m = 1.0;
    let r = 6.0;
    let schw = radial_flow_metric(move |r| -(2.0 * m / r).sqrt(), |_| 1.0, |_| 1.0);
    let flat = radial_flow_metric(|_| 0.0, |_| 1.0, |_| 1.0);
    // Constant density: continuity ρ v r² = const ⇒ v = −A/r². Match v at r=6 to Schwarzschild.
    let a = (2.0 * m / r).sqrt() * r * r;
    let sink = radial_flow_metric(move |r| -a / (r * r), |_| 1.0, |_| 1.0);
    // Schwarzschild-profile velocity, but continuity forces ρ ∝ r^(−3/2): conformal factor ρ/c.
    let fluid = radial_flow_metric(move |r| -(2.0 * m / r).sqrt(), |_| 1.0, |r| r.powf(-1.5));
    EinsteinStudy {
        schwarzschild: ricci_ratio(schw.as_ref(), r),
        flat: ricci_ratio(flat.as_ref(), r),
        const_density_sink: ricci_ratio(sink.as_ref(), r),
        schwarzschild_profile_fluid: ricci_ratio(fluid.as_ref(), r),
    }
}

pub fn checks() -> Vec<Check> {
    let s = einstein_study();
    vec![
        Check::abs("Engine check: Schwarzschild (Painlevé–Gullstrand) is Ricci-flat", 0.0, s.schwarzschild, 1e-5, "GR: R_μν = 0 in vacuum"),
        Check::abs("Engine check: flat space has zero curvature", 0.0, s.flat, 1e-9, "Minkowski"),
        Check::abs(
            "E10 Constant-density fluid sink obeys Einstein vacuum eqs",
            0.0,
            s.const_density_sink,
            1e-3,
            "EXPECTED TO FAIL: continuity forces v ∝ 1/r², not √(2M/r)",
        ),
        Check::abs(
            "E10 Fluid with Schwarzschild velocity profile obeys Einstein eqs",
            0.0,
            s.schwarzschild_profile_fluid,
            1e-3,
            "EXPECTED TO FAIL: continuity forces ρ ∝ r^(−3/2); the conformal factor ρ/c spoils Ricci-flatness (Visser)",
        ),
    ]
}

/// Full E10 + E12 suite: Einstein's equations and the graviton search.
pub fn gravity_dynamics_checks() -> Vec<Check> {
    let mut v = checks();
    v.extend(crate::gpe::spectrum_checks(&crate::gpe::spectrum_study()));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn einstein_suite() {
        let s = einstein_study();
        eprintln!(
            "schw {:e} flat {:e} sink {:e} fluid {:e}",
            s.schwarzschild, s.flat, s.const_density_sink, s.schwarzschild_profile_fluid
        );
        for c in checks() {
            let expect_fail = c.name.starts_with("E10");
            assert_eq!(c.pass(), !expect_fail, "{c:?}");
        }
    }
}
