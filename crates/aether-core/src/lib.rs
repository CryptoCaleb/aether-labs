//! Aether Labs physics core.
//!
//! Pure, GUI-free numerical models for testing the hypothesis that matter and
//! forces are structures (vortices, pulsations, flows) in a single continuous
//! medium. Every model ships with quantitative [`Check`]s against an
//! independent theoretical or experimental benchmark, so the app can report
//! PASS / FAIL rather than just draw pretty pictures.

pub mod acoustic;
pub mod bjerknes;
pub mod filament;
pub mod gpe;
pub mod pointvortex;
pub mod shapes;
pub mod sinegordon;
pub mod stats;

/// A quantitative validation: a measured number compared with an expected one.
#[derive(Clone, Debug)]
pub struct Check {
    pub name: String,
    pub expected: f64,
    pub measured: f64,
    /// Allowed deviation. Relative to |expected| unless `absolute` is set.
    pub tolerance: f64,
    pub absolute: bool,
    /// Where the expected value comes from (formula, paper, experiment).
    pub source: String,
}

impl Check {
    pub fn rel(name: &str, expected: f64, measured: f64, tol: f64, source: &str) -> Self {
        Self {
            name: name.into(),
            expected,
            measured,
            tolerance: tol,
            absolute: false,
            source: source.into(),
        }
    }

    pub fn abs(name: &str, expected: f64, measured: f64, tol: f64, source: &str) -> Self {
        Self {
            absolute: true,
            ..Self::rel(name, expected, measured, tol, source)
        }
    }

    pub fn boolean(name: &str, ok: bool, source: &str) -> Self {
        Self::abs(name, 1.0, if ok { 1.0 } else { 0.0 }, 0.5, source)
    }

    pub fn error(&self) -> f64 {
        let d = (self.measured - self.expected).abs();
        if self.absolute {
            d
        } else {
            d / self.expected.abs().max(1e-300)
        }
    }

    pub fn pass(&self) -> bool {
        self.measured.is_finite() && self.error() <= self.tolerance
    }
}

/// Tiny deterministic PRNG (xorshift64*), so results are reproducible.
#[derive(Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in [0, 1).
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.uniform()
    }
}
