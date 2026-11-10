//! Interactive experiments. Each one owns its simulation state, controls,
//! viewport and live validation checks.

pub mod acoustic;
pub mod bjerknes;
pub mod filaments;
pub mod hierarchy;
pub mod point;
pub mod relativity;
pub mod superfluid;

use aether_core::Check;
use egui::{Color32, RichText, Ui};

pub trait Experiment {
    fn title(&self) -> &'static str;
    fn category(&self) -> &'static str;
    /// One-line hook shown in the sidebar.
    fn tagline(&self) -> &'static str;
    /// Longer explanation: what aether theory says, what physics says, what
    /// this measures.
    fn theory(&self) -> &'static str;
    fn reset(&mut self);
    /// Advance the simulation by `substeps` internal steps.
    fn advance(&mut self, substeps: usize);
    /// Parameter controls. Return true if parameters changed and a reset is needed.
    fn controls(&mut self, ui: &mut Ui) -> bool;
    /// Main viewport.
    fn view(&mut self, ui: &mut Ui);
    /// Live diagnostics and plots.
    fn diagnostics(&mut self, ui: &mut Ui);
    /// Quantitative checks against theory for the current run.
    fn checks(&self) -> Vec<Check> {
        Vec::new()
    }
    fn sim_time(&self) -> f64;
}

pub fn all() -> Vec<Box<dyn Experiment>> {
    use filaments::{FilamentExp, Kind};
    vec![
        Box::new(FilamentExp::new(Kind::Ring)),
        Box::new(FilamentExp::new(Kind::Leapfrog)),
        Box::new(FilamentExp::new(Kind::Collision)),
        Box::new(FilamentExp::new(Kind::Wavy)),
        Box::new(FilamentExp::new(Kind::HopfLink)),
        Box::new(FilamentExp::new(Kind::Trefoil)),
        Box::new(point::PointExp::new()),
        Box::new(bjerknes::BjerknesExp::new()),
        Box::new(acoustic::AcousticExp::new()),
        Box::new(relativity::RelativityExp::new()),
        Box::new(superfluid::SuperfluidExp::new()),
        Box::new(hierarchy::HierarchyExp::new()),
    ]
}

pub const PASS: Color32 = Color32::from_rgb(90, 210, 120);
pub const FAIL: Color32 = Color32::from_rgb(235, 90, 90);

/// Render a list of checks as a compact table.
pub fn checks_table(ui: &mut Ui, id: &str, checks: &[Check]) {
    egui::Grid::new(id).striped(true).num_columns(4).show(ui, |ui| {
        ui.strong("");
        ui.strong("Check");
        ui.strong("Expected");
        ui.strong("Measured");
        ui.end_row();
        for c in checks {
            let (txt, col) = if c.pass() { ("PASS", PASS) } else { ("FAIL", FAIL) };
            ui.label(RichText::new(txt).color(col).strong());
            ui.label(&c.name).on_hover_text(format!("Source: {}\nTolerance: {}{}", c.source, c.tolerance, if c.absolute { " (absolute)" } else { " (relative)" }));
            ui.monospace(fmt(c.expected));
            ui.monospace(fmt(c.measured));
            ui.end_row();
        }
    });
}

pub fn fmt(x: f64) -> String {
    if !x.is_finite() {
        return "—".into();
    }
    let a = x.abs();
    if a != 0.0 && !(1e-3..1e5).contains(&a) {
        format!("{x:.4e}")
    } else {
        format!("{x:.5}")
    }
}

pub fn metric(ui: &mut Ui, name: &str, value: impl Into<String>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(name).color(Color32::GRAY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.monospace(value.into());
        });
    });
}

/// Bounded time-series buffer for plots.
#[derive(Default, Clone)]
pub struct Series {
    pub pts: Vec<[f64; 2]>,
}

impl Series {
    pub fn push(&mut self, t: f64, v: f64) {
        if v.is_finite() {
            self.pts.push([t, v]);
            if self.pts.len() > 4000 {
                // Decimate: keep every other point.
                self.pts = self.pts.iter().step_by(2).copied().collect();
            }
        }
    }
    pub fn clear(&mut self) {
        self.pts.clear();
    }
}

pub fn plot(ui: &mut Ui, id: &str, height: f32, lines: &[(&str, &Series, Color32)]) {
    egui_plot::Plot::new(id)
        .height(height)
        .legend(egui_plot::Legend::default())
        .allow_scroll(false)
        .show(ui, |p| {
            for (name, s, c) in lines {
                p.line(egui_plot::Line::new(*name, s.pts.clone()).color(*c));
            }
        });
}
