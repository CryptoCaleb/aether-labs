use super::{fmt, metric, plot, Experiment, Series};
use crate::view::Camera;
use aether_core::bjerknes::{self, BjerknesSim, Pulsator};
use aether_core::stats::power_law_fit;
use aether_core::Check;
use egui::{Color32, Slider, Stroke, Ui};
use glam::DVec3;
use std::f64::consts::PI;

pub struct BjerknesExp {
    sim: BjerknesSim,
    cam: Camera,
    phase_deg: f64,
    third: bool,
    third_phase_deg: f64,
    separation: f64,
    eps: f64,
    drag: f64,
    dt: f64,
    s_sep: Series,
    law_measured: Vec<[f64; 2]>,
    law_analytic: Vec<[f64; 2]>,
    fit_exponent: f64,
    visual_t: f64,
    forces: Vec<DVec3>,
}

impl BjerknesExp {
    pub fn new() -> Self {
        let mut e = Self {
            sim: BjerknesSim::new(vec![]),
            cam: Camera::new(14.0),
            phase_deg: 0.0,
            third: false,
            third_phase_deg: 180.0,
            separation: 6.0,
            eps: 0.15,
            drag: 0.0,
            dt: 0.05,
            s_sep: Series::default(),
            law_measured: vec![],
            law_analytic: vec![],
            fit_exponent: f64::NAN,
            visual_t: 0.0,
            forces: Vec::new(),
        };
        e.reset();
        e
    }

    fn update_forces(&mut self) {
        self.forces = (0..self.sim.spheres.len()).map(|i| self.sim.averaged_force(i, self.sim.time)).collect();
    }

    fn compute_law(&mut self) {
        let dphi = self.phase_deg.to_radians();
        let data = bjerknes::sweep(1.0, 0.02, dphi);
        self.law_measured = data.iter().map(|(d, f)| [d.log10(), f.abs().log10()]).collect();
        self.law_analytic = data
            .iter()
            .map(|(d, _)| {
                let dv = 4.0 * PI * 0.02;
                let f = (2.0 * PI).powi(2) * dv * dv * dphi.cos().abs() / (8.0 * PI * d * d);
                [d.log10(), f.log10()]
            })
            .collect();
        let (xs, ys): (Vec<f64>, Vec<f64>) = data.iter().copied().unzip();
        self.fit_exponent = power_law_fit(&xs, &ys).1;
    }
}

impl Experiment for BjerknesExp {
    fn title(&self) -> &'static str {
        "Pulsating spheres (Bjerknes)"
    }
    fn category(&self) -> &'static str {
        "Forces through the medium"
    }
    fn tagline(&self) -> &'static str {
        "Can a breathing aether make gravity or electricity?"
    }
    fn theory(&self) -> &'static str {
        "BJERKNES FORCES (C. A. Bjerknes 1875–1881; V. Bjerknes, 'Fields of Force', 1906). Spheres that pulsate in an ideal incompressible fluid push on each other through the pressure field. Averaged over a cycle:\n\n⟨F⟩ = −ρ ω² ΔV₁ ΔV₂ cos(Δφ) / (8π d²)\n\nThis is an exact inverse-square law, transmitted purely by the medium. Kelvin and Bjerknes asked whether gravity could be universal in-phase pulsation.\n\nTHE CATCH. In-phase spheres ATTRACT and anti-phase spheres REPEL. Treat pulsation phase as 'charge' and like charges attract, which is the reverse of Coulomb's law. One mechanism cannot give both gravity and electrostatics. The force also scales with ΔV₁ΔV₂, not with mass, so the equivalence principle would need fine-tuning. And every body in the universe would have to pulsate in phase.\n\nWHAT IS MEASURED. The force from the unsteady Bernoulli pressure, integrated numerically over a cycle, swept over distance and fitted on log–log axes. Set the phase to 90° and the force vanishes (cos Δφ).\n\nThis model is incompressible (d ≪ wavelength). A compressible lattice-Boltzmann medium, where retardation can flip the sign at kd ~ π/2, is planned."
    }

    fn reset(&mut self) {
        let mut s = vec![
            Pulsator::new(DVec3::new(-0.5 * self.separation, 0.0, 0.0), 1.0, self.eps, 0.0),
            Pulsator::new(DVec3::new(0.5 * self.separation, 0.0, 0.0), 1.0, self.eps, self.phase_deg.to_radians()),
        ];
        if self.third {
            s.push(Pulsator::new(DVec3::new(0.0, 0.8 * self.separation, 0.0), 1.0, self.eps, self.third_phase_deg.to_radians()));
        }
        self.sim = BjerknesSim::new(s);
        self.sim.samples = 32;
        self.sim.drag = self.drag;
        self.s_sep.clear();
        self.compute_law();
        self.s_sep.push(0.0, self.separation);
        self.update_forces();
    }

    fn advance(&mut self, substeps: usize) {
        for _ in 0..substeps {
            self.sim.step(self.dt);
        }
        let d = (self.sim.spheres[1].pos - self.sim.spheres[0].pos).length();
        self.s_sep.push(self.sim.time, d);
        self.visual_t += 0.02 * substeps as f64;
        self.update_forces();
    }

    fn controls(&mut self, ui: &mut Ui) -> bool {
        let mut ch = false;
        ch |= ui.add(Slider::new(&mut self.phase_deg, 0.0..=180.0).text("phase difference Δφ (°)")).changed();
        ch |= ui.add(Slider::new(&mut self.separation, 2.5..=15.0).text("initial separation")).changed();
        ch |= ui.add(Slider::new(&mut self.eps, 0.01..=0.3).text("pulsation amplitude ε")).changed();
        ch |= ui.add(Slider::new(&mut self.drag, 0.0..=0.5).text("viscous drag")).changed();
        ch |= ui.checkbox(&mut self.third, "Add a third sphere").changed();
        if self.third {
            ch |= ui.add(Slider::new(&mut self.third_phase_deg, 0.0..=180.0).text("third sphere phase (°)")).changed();
        }
        ui.add(Slider::new(&mut self.dt, 0.005..=0.5).logarithmic(true).text("time step"));
        ch
    }

    fn view(&mut self, ui: &mut Ui) {
        let (_r, painter) = self.cam.interact(ui);
        self.cam.grid(&painter, 10.0, -3.0, 1.0);
        let w = 2.0 * PI;
        for (i, s) in self.sim.spheres.iter().enumerate() {
            // Visual pulsation is slowed down so it can be seen.
            let r = s.a0 * (1.0 + s.eps * (w * self.visual_t + s.phase).sin());
            let col = match i {
                0 => Color32::from_rgb(90, 200, 255),
                1 => Color32::from_rgb(255, 140, 80),
                _ => Color32::from_rgb(180, 255, 120),
            };
            if let Some((c, z)) = self.cam.project(s.pos) {
                let scale = (ui.max_rect().height() as f64 * 0.5 / (self.cam.fov * 0.5).tan()) / z as f64;
                let rad = (r * scale) as f32;
                painter.circle_filled(c, rad, col.gamma_multiply(0.35));
                painter.circle_stroke(c, rad, Stroke::new(2.0, col));
                // Pressure "ripples" from each pulsator.
                for k in 1..4 {
                    let phase = ((w * self.visual_t + s.phase) / (2.0 * PI) + k as f64 / 3.0).fract();
                    let rr = rad * (1.0 + 3.0 * phase as f32);
                    painter.circle_stroke(c, rr, Stroke::new(1.0, col.gamma_multiply(0.4 * (1.0 - phase as f32))));
                }
            }
        }
        // Force arrows
        for i in 0..self.sim.spheres.len() {
            let f = self.forces.get(i).copied().unwrap_or(DVec3::ZERO);
            let p = self.sim.spheres[i].pos;
            let scale = 40.0 / (self.eps * self.eps).max(1e-6);
            self.cam.line(&painter, p, p + f * scale.min(400.0), 2.0, Color32::from_rgb(255, 230, 120));
        }
    }

    fn diagnostics(&mut self, ui: &mut Ui) {
        let d = (self.sim.spheres[1].pos - self.sim.spheres[0].pos).length();
        let f = self.forces.get(1).map_or(0.0, |f| f.x);
        let fa = self.sim.analytic_force(1, 0).x;
        metric(ui, "time", fmt(self.sim.time));
        metric(ui, "separation", fmt(d));
        metric(ui, "force on B (numeric)", fmt(f));
        metric(ui, "force on B (Bjerknes formula)", fmt(fa));
        metric(ui, "verdict", if f < 0.0 { "ATTRACTION" } else if f > 0.0 { "REPULSION" } else { "none" });
        metric(ui, "fitted force-law exponent", fmt(self.fit_exponent));
        ui.label("Separation vs time");
        plot(ui, "bs", 110.0, &[("d(t)", &self.s_sep, Color32::from_rgb(90, 200, 255))]);
        ui.label("Force law: log₁₀|F| vs log₁₀ d (dots = simulation, line = 1/d²)");
        egui_plot::Plot::new("blaw").height(160.0).allow_scroll(false).show(ui, |p| {
            p.points(egui_plot::Points::new("simulation", self.law_measured.clone()).radius(3.0).color(Color32::from_rgb(255, 140, 80)));
            p.line(egui_plot::Line::new("Bjerknes 1/d²", self.law_analytic.clone()).color(Color32::from_rgb(90, 200, 255)));
        });
    }

    fn checks(&self) -> Vec<Check> {
        let d = (self.sim.spheres[1].pos - self.sim.spheres[0].pos).length();
        let mut v = vec![];
        if (self.phase_deg - 90.0).abs() > 5.0 {
            v.push(Check::abs("Force-law exponent", -2.0, self.fit_exponent, 0.03, "Bjerknes 1/d²"));
        }
        if d > 4.0 && self.eps <= 0.1 && (self.phase_deg - 90.0).abs() > 5.0 {
            v.push(Check::rel(
                "Force vs formula at current d",
                self.sim.analytic_force(1, 0).x,
                self.forces.get(1).map_or(0.0, |f| f.x),
                0.1,
                "linear Bjerknes theory (valid for small ε, d ≫ a)",
            ));
        }
        v
    }

    fn sim_time(&self) -> f64 {
        self.sim.time
    }
}
