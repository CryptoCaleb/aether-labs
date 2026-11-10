//! The hypothesis scorecard: runs every validation suite and lays out what a
//! single-medium theory must reproduce versus what these models achieve.

use crate::experiments::{checks_table, FAIL, PASS};
use aether_core::{acoustic, bjerknes, filament, gpe, pointvortex, sinegordon, Check};
use egui::{Color32, RichText, Ui};
use std::sync::mpsc::{channel, Receiver};
use std::time::Instant;

pub struct Suite {
    pub name: &'static str,
    pub claim: &'static str,
    pub checks: Vec<Check>,
    pub seconds: f64,
}

#[derive(Default)]
pub struct Scorecard {
    rx: Option<Receiver<Suite>>,
    pub suites: Vec<Suite>,
    running: usize,
    include_slow: bool,
}

type SuiteFn = fn() -> Vec<Check>;

fn lattice_suite() -> Vec<Check> {
    let mut g = gpe::Gpe::new(192, 26.0, 1500.0, 0.85, 7);
    for _ in 0..6000 {
        g.step(0.004, true);
    }
    gpe::lattice_checks(&gpe::analyze_lattice(&g))
}

impl Scorecard {
    pub fn run(&mut self) {
        let mut list: Vec<(&'static str, &'static str, SuiteFn)> = vec![
            ("Vortex rings & topology", "Kelvin: matter = vortex rings in an ideal aether", filament::checks),
            ("2D vortex 'charges'", "Vortices interact like charges (2D Coulomb + Magnus)", pointvortex::checks),
            ("Bjerknes pulsation forces", "Pulsating bodies in the medium feel 1/d² forces", bjerknes::checks),
            ("Acoustic black hole", "Flowing medium = curved spacetime for its waves", acoustic::checks),
            ("Emergent relativity", "A medium's own solitons show Lorentz contraction & time dilation", sinegordon::checks),
        ];
        if self.include_slow {
            list.push(("Quantised vortex lattice (slow)", "A superfluid medium quantises circulation (Feynman rule)", lattice_suite));
        }
        let (tx, rx) = channel();
        self.running = list.len();
        self.suites.clear();
        for (name, claim, f) in list {
            let tx = tx.clone();
            std::thread::spawn(move || {
                let t0 = Instant::now();
                let checks = f();
                let _ = tx.send(Suite { name, claim, checks, seconds: t0.elapsed().as_secs_f64() });
            });
        }
        self.rx = Some(rx);
    }

    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            while let Ok(s) = rx.try_recv() {
                self.suites.push(s);
                self.running = self.running.saturating_sub(1);
            }
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        self.poll();
        if self.running > 0 {
            ui.ctx().request_repaint();
        }
        ui.heading("Hypothesis scorecard: is the aether the medium of reality?");
        ui.label(
            "Each suite runs a model of 'everything is structure in one continuous medium' and compares it with independent theory or experiment. \
             A PASS means the medium really reproduces that piece of physics. A FAIL marks a real obstacle for the hypothesis. Failures are reported, never hidden.",
        );
        ui.horizontal(|ui| {
            if ui.add_enabled(self.running == 0, egui::Button::new("▶ Run all validation suites")).clicked() {
                self.run();
            }
            ui.checkbox(&mut self.include_slow, "include slow superfluid lattice (~1 min)");
            if self.running > 0 {
                ui.spinner();
                ui.label(format!("{} suite(s) running…", self.running));
            }
        });
        let total: usize = self.suites.iter().map(|s| s.checks.len()).sum();
        let passed: usize = self.suites.iter().map(|s| s.checks.iter().filter(|c| c.pass()).count()).sum();
        if total > 0 {
            ui.label(RichText::new(format!("{passed} / {total} checks pass")).size(18.0).strong());
        }
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut suites: Vec<&Suite> = self.suites.iter().collect();
            suites.sort_by_key(|s| s.name);
            for s in suites {
                let ok = s.checks.iter().filter(|c| c.pass()).count();
                let col = if ok == s.checks.len() { PASS } else { Color32::from_rgb(240, 190, 80) };
                egui::CollapsingHeader::new(RichText::new(format!("{}   {}/{}   ({:.1}s)", s.name, ok, s.checks.len(), s.seconds)).color(col))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new(s.claim).italics());
                        checks_table(ui, s.name, &s.checks);
                    });
            }
            ui.add_space(12.0);
            ui.heading("What any single-medium theory must reproduce");
            ui.label("Summarised from the research brief (docs/RESEARCH.md). Status says what this app currently shows.");
            requirements_table(ui);
        });
    }
}

fn requirements_table(ui: &mut Ui) {
    // (requirement, observed constraint, status, colour key, note)
    let rows: [(&str, &str, &str, u8, &str); 12] = [
        ("Light-speed isotropy (Michelson–Morley)", "Δc/c < 10⁻¹⁷", "Mechanism shown", 1, "Sine-Gordon: medium-made rulers contract and clocks dilate by γ, which hides the aether wind."),
        ("One limiting speed for all species", "agree to 10⁻¹⁵–10⁻²²", "Not yet tested", 0, "Planned: two-field medium with different sound speeds."),
        ("No dispersion of light up to high energy", "beyond the Planck scale (Fermi-LAT)", "Open problem", 2, "Superfluids have a healing length ξ, so dispersion appears at kξ ~ 1."),
        ("Inverse-square laws", "Coulomb exponent to 10⁻¹⁶", "Shown", 1, "Bjerknes force exponent −2 within 2%."),
        ("Correct force signs (like charges repel, masses attract)", "observed", "FAILS", 2, "Bjerknes: like phases attract. One mechanism cannot give both signs."),
        ("Charge quantisation", "|q_e + q_p| < 10⁻²¹ e", "Shown (topology)", 1, "Quantised superfluid circulation; conserved linking number."),
        ("Stable particles of fixed mass", "proton > 10³⁴ yr", "FAILS for fluid vortices", 2, "Ring size is set by initial conditions; rings slow as they gain energy; knots untie by reconnection."),
        ("Spin-½ / fermions", "4π rotation symmetry", "Not reproduced", 2, "Needs Skyrmion-type field solitons, not fluid vortices."),
        ("Equivalence principle", "η < 10⁻¹⁵ (MICROSCOPE)", "FAILS (Bjerknes)", 2, "The force scales with ΔV₁ΔV₂, not with inertial mass."),
        ("E² = p²c² + m²c⁴", "all particles", "Shown for solitons, fails for rings", 1, "Kink energy = 8γ; vortex rings have U falling as E rises."),
        ("No drag on steady motion", "orbits stable for Gyr", "Shown (ideal fluid)", 1, "d'Alembert: an ideal medium gives zero drag; kinks keep their speed."),
        ("Gravity with Einstein dynamics and spin-2 waves", "light bending, GW speed = c", "Partial", 0, "Acoustic metric bends and traps rays, but it is kinematic and not sourced by mass."),
    ];
    egui::Grid::new("req").striped(true).num_columns(4).show(ui, |ui| {
        ui.strong("Requirement");
        ui.strong("Observed");
        ui.strong("Status here");
        ui.strong("Notes");
        ui.end_row();
        for (req, obs, status, key, note) in rows {
            let col = match key {
                1 => PASS,
                2 => FAIL,
                _ => Color32::from_rgb(240, 190, 80),
            };
            ui.label(req);
            ui.label(obs);
            ui.label(RichText::new(status).color(col).strong());
            ui.label(note);
            ui.end_row();
        }
    });
}
