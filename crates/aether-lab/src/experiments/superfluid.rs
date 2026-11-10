use super::{fmt, metric, plot, Experiment, Series};
use crate::view::{colormap, phase_color, View2D};
use aether_core::gpe::{analyze_lattice, lattice_checks, Gpe, LatticeReport};
use aether_core::Check;
use egui::{Color32, ColorImage, Slider, Stroke, TextureHandle, TextureOptions, Ui};
use glam::DVec2;
use std::f64::consts::PI;

pub struct SuperfluidExp {
    gpe: Gpe,
    view: View2D,
    tex: Option<TextureHandle>,
    n: usize,
    g: f64,
    omega: f64,
    dt: f64,
    imaginary: bool,
    show_phase: bool,
    seed: u64,
    report: Option<LatticeReport>,
    s_count: Series,
    dirty: bool,
    imprint_w: i32,
    vortex_cache: Vec<(f64, f64, i32)>,
}

impl SuperfluidExp {
    pub fn new() -> Self {
        let (n, len, g, omega) = (128, 24.0, 1000.0, 0.8);
        Self {
            gpe: Gpe::new(n, len, g, omega, 1),
            view: View2D::new(len),
            tex: None,
            n,
            g,
            omega,
            dt: 0.005,
            imaginary: true,
            show_phase: false,
            seed: 1,
            report: None,
            s_count: Series::default(),
            dirty: true,
            imprint_w: 1,
            vortex_cache: Vec::new(),
        }
    }

    fn image(&self) -> ColorImage {
        let n = self.gpe.n;
        let dens = self.gpe.density();
        let maxd = dens.iter().cloned().fold(1e-30, f64::max);
        let mut pixels = Vec::with_capacity(n * n);
        // Row 0 of the image is the top (largest y).
        for j in (0..n).rev() {
            for i in 0..n {
                let idx = j * n + i;
                let d = (dens[idx] / maxd).sqrt();
                pixels.push(if self.show_phase { phase_color(self.gpe.psi[idx].arg(), d) } else { colormap(d) });
            }
        }
        ColorImage::new([n, n], pixels)
    }
}

impl Experiment for SuperfluidExp {
    fn title(&self) -> &'static str {
        "Quantum aether (superfluid BEC)"
    }
    fn category(&self) -> &'static str {
        "Quantised vortices"
    }
    fn tagline(&self) -> &'static str {
        "Spin the medium: vortices appear one quantum at a time"
    }
    fn theory(&self) -> &'static str {
        "SUPERFLUID VACUUM (Volovik, 'The Universe in a Helium Droplet', 2003). The most serious modern 'single medium' programme treats the vacuum as a quantum superfluid. Its low-energy excitations look like relativistic fields, and its vortices are quantised topological defects. This experiment solves the Gross–Pitaevskii equation for such a medium:\n\ni∂ψ/∂t = [−½∇² + ½r² + g|ψ|² − ΩL_z]ψ\n\nWHAT HAPPENS. Unlike a classical fluid, a superfluid cannot rotate smoothly. Circulation comes only in whole units of h/m (Onsager 1949, Feynman 1955). When the container spins, vortices nucleate one by one and settle into a triangular Abrikosov lattice. Their density follows Feynman's rule n_v = mΩ/(πħ), which in these units is Ω/π. This was seen with over 100 vortices by Abo-Shaeer et al. (2001).\n\nWHY IT MATTERS. Quantisation, which classical vortex atoms lacked, falls out of the medium automatically. It is the closest analogue of charge quantisation.\n\nTHE CATCH. Quantised vortices still reconnect when they meet, so topology is not permanent. Vortex 'particles' are not identical solitons with a fixed mass.\n\nUSE IT. Leave 'Imaginary time' on to relax towards the ground state; the lattice forms in roughly 2000–6000 steps. Then press 'Analyze lattice'. Switch to real time to watch the dynamics, and click in the view to imprint a vortex."
    }

    fn reset(&mut self) {
        let len = 24.0f64.max(2.6 * self.gpe_tf_estimate());
        self.gpe = Gpe::new(self.n, len, self.g, self.omega, self.seed);
        self.view = View2D::new(len);
        self.report = None;
        self.s_count.clear();
        self.dirty = true;
    }

    fn advance(&mut self, substeps: usize) {
        for _ in 0..substeps * 4 {
            self.gpe.step(self.dt, self.imaginary);
        }
        self.dirty = true;
        self.vortex_cache = self.gpe.vortices(0.15);
        self.s_count.push(self.gpe.time, self.vortex_cache.len() as f64);
    }

    fn controls(&mut self, ui: &mut Ui) -> bool {
        let mut ch = false;
        egui::ComboBox::from_label("grid")
            .selected_text(format!("{0}×{0}", self.n))
            .show_ui(ui, |ui| {
                for n in [64usize, 96, 128, 192, 256] {
                    ch |= ui.selectable_value(&mut self.n, n, format!("{n}×{n}")).changed();
                }
            });
        ch |= ui.add(Slider::new(&mut self.g, 10.0..=4000.0).logarithmic(true).text("interaction g")).changed();
        ch |= ui.add(Slider::new(&mut self.omega, 0.0..=0.95).text("rotation Ω (trap units)")).changed();
        if ui.add(Slider::new(&mut self.gpe.omega, 0.0..=0.95).text("live Ω (no reset)")).changed() {
            self.omega = self.gpe.omega;
        }
        ui.add(Slider::new(&mut self.dt, 0.0005..=0.02).logarithmic(true).text("time step"));
        ui.checkbox(&mut self.imaginary, "Imaginary time (relax to ground state)");
        if ui.checkbox(&mut self.show_phase, "Show phase (colour wheel) instead of density").changed() {
            self.dirty = true;
        }
        ui.horizontal(|ui| {
            ui.label("click-imprint winding");
            ui.add(egui::DragValue::new(&mut self.imprint_w).range(-3..=3));
        });
        ui.horizontal(|ui| {
            if ui.button("Analyze lattice").clicked() {
                self.report = Some(analyze_lattice(&self.gpe));
            }
            if ui.button("New random seed").clicked() {
                self.seed += 1;
                ch = true;
            }
        });
        ch
    }

    fn view(&mut self, ui: &mut Ui) {
        if self.dirty || self.tex.is_none() {
            let img = self.image();
            match &mut self.tex {
                Some(t) => t.set(img, TextureOptions::LINEAR),
                None => self.tex = Some(ui.ctx().load_texture("gpe", img, TextureOptions::LINEAR)),
            }
            self.dirty = false;
        }
        let (resp, painter) = self.view.interact(ui);
        let half = 0.5 * self.gpe.len;
        let rect = egui::Rect::from_two_pos(self.view.to_screen(DVec2::new(-half, half)), self.view.to_screen(DVec2::new(half, -half)));
        if let Some(t) = &self.tex {
            painter.image(t.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        }
        if resp.clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                let w = self.view.to_world(p);
                self.gpe.imprint_vortex(w.x, w.y, self.imprint_w);
                self.vortex_cache = self.gpe.vortices(0.15);
                self.dirty = true;
            }
        }
        for &(x, y, w) in &self.vortex_cache {
            let c = if w > 0 { Color32::from_rgb(255, 255, 255) } else { Color32::from_rgb(255, 80, 80) };
            painter.circle_stroke(self.view.to_screen(DVec2::new(x, y)), 4.0, Stroke::new(1.2, c));
        }
        let rtf = self.gpe.tf_radius();
        self.view.circle(&painter, DVec2::ZERO, rtf, Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 50)));
        if let Some(r) = &self.report {
            self.view.circle(&painter, DVec2::ZERO, r.inner_radius, Stroke::new(1.0, Color32::from_rgb(255, 200, 80)));
        }
        painter.text(
            ui.max_rect().left_top() + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            "Click: imprint vortex · circles mark detected vortices (white +1, red −1) · Right-drag: pan · Wheel: zoom",
            egui::FontId::proportional(12.0),
            Color32::GRAY,
        );
    }

    fn diagnostics(&mut self, ui: &mut Ui) {
        let vs = &self.vortex_cache;
        let rtf = self.gpe.tf_radius();
        metric(ui, "time τ", fmt(self.gpe.time));
        metric(ui, "norm ∫|ψ|²", fmt(self.gpe.norm()));
        metric(ui, "vortices detected", vs.len().to_string());
        metric(ui, "net winding", vs.iter().map(|v| v.2).sum::<i32>().to_string());
        metric(ui, "Thomas–Fermi radius", fmt(rtf));
        metric(ui, "Feynman estimate Ω·R²", fmt(self.gpe.omega * rtf * rtf));
        metric(ui, "healing length ξ ≈ 1/√(2μ)", fmt(1.0 / (2.0 * self.gpe.chemical_scale()).sqrt()));
        metric(ui, "grid spacing dx", fmt(self.gpe.dx()));
        ui.label("Vortex count vs time");
        plot(ui, "gc", 110.0, &[("vortices", &self.s_count, Color32::from_rgb(90, 200, 255))]);
        if let Some(r) = &self.report {
            ui.separator();
            ui.strong("Lattice analysis (inside the gold circle)");
            metric(ui, "vortices inside", r.inner_count.to_string());
            metric(ui, "measured density", fmt(r.measured_density));
            metric(ui, "Feynman Ω/π", fmt(r.feynman_density));
            metric(ui, "mean NN spacing", fmt(r.nn_distance));
            metric(ui, "triangular-lattice spacing", fmt(r.triangular_nn));
        }
        let _ = PI;
    }

    fn checks(&self) -> Vec<Check> {
        match &self.report {
            Some(r) => lattice_checks(r),
            None => vec![],
        }
    }

    fn sim_time(&self) -> f64 {
        self.gpe.time
    }
}

impl SuperfluidExp {
    fn gpe_tf_estimate(&self) -> f64 {
        let w2 = (1.0 - self.omega * self.omega).max(1e-3);
        let mu = (self.g * w2 / PI).sqrt();
        (2.0 * mu / w2).sqrt()
    }
}
