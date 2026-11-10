use super::{fmt, metric, plot, Experiment, Series};
use crate::view::View2D;
use aether_core::pointvortex::PointVortices;
use aether_core::{Check, Rng};
use egui::{Color32, Slider, Stroke, Ui};
use glam::DVec2;
use std::f64::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Preset {
    LikePair,
    Dipole,
    Triangle,
    Gas,
    Street,
}

pub struct PointExp {
    preset: Preset,
    pv: PointVortices,
    view: View2D,
    trails: Vec<Vec<DVec2>>,
    dt: f64,
    count: usize,
    h0: f64,
    l0: f64,
    s_h: Series,
    click_gamma: f64,
}

impl PointExp {
    pub fn new() -> Self {
        let mut e = Self {
            preset: Preset::LikePair,
            pv: PointVortices::new(vec![], vec![]),
            view: View2D::new(6.0),
            trails: vec![],
            dt: 0.01,
            count: 60,
            h0: 0.0,
            l0: 0.0,
            s_h: Series::default(),
            click_gamma: 1.0,
        };
        e.reset();
        e
    }
}

impl Experiment for PointExp {
    fn title(&self) -> &'static str {
        "2D vortex 'charges'"
    }
    fn category(&self) -> &'static str {
        "Vortex matter (2D)"
    }
    fn tagline(&self) -> &'static str {
        "Coulomb-like energy, Magnus-like motion"
    }
    fn theory(&self) -> &'static str {
        "POINT VORTICES (Helmholtz 1858, Kirchhoff 1876). In a 2D slice of the medium, line vortices interact through a logarithmic energy H = −(1/4π) Σ Γ_iΓ_j ln r_ij. That is exactly the 2D Coulomb law, so vortices behave like charges.\n\nTHE TWIST. The response is not Newtonian. A vortex moves PERPENDICULAR to the force on it (the Magnus effect, the fluid version of the Lorentz force qv×B). Like vortices orbit each other at Ω = (Γ₁+Γ₂)/(2πd²). Opposite vortices travel together in a straight line at U = Γ/(2πd).\n\nIn 2D this 'particle–vortex duality' is exact: vortices are charges of a dual electromagnetism, and sound waves play the role of the photon. It works only in 2+1 dimensions.\n\nCLICK in the view to add a vortex with the chosen Γ. The 'gas' preset shows Onsager's like-sign clustering, where 2D vortex turbulence builds large structures out of small ones.\n\nNote: the vortex 'gas' starts with identical cores, so a few vortices can pass very close and the energy drifts. The exact presets conserve energy to 1e-10."
    }

    fn reset(&mut self) {
        let mut rng = Rng::new(3);
        let (pos, gam): (Vec<DVec2>, Vec<f64>) = match self.preset {
            Preset::LikePair => (vec![DVec2::new(-0.5, 0.0), DVec2::new(0.5, 0.0)], vec![1.0, 1.0]),
            Preset::Dipole => (vec![DVec2::new(-0.5, -2.0), DVec2::new(0.5, -2.0)], vec![1.0, -1.0]),
            Preset::Triangle => {
                let p = (0..3).map(|k| {
                    let a = 2.0 * PI * k as f64 / 3.0;
                    DVec2::new(a.cos(), a.sin())
                });
                (p.collect(), vec![1.0; 3])
            }
            Preset::Gas => {
                let mut p = vec![];
                let mut g = vec![];
                for i in 0..self.count {
                    p.push(DVec2::new(rng.range(-2.5, 2.5), rng.range(-2.5, 2.5)));
                    g.push(if i % 2 == 0 { 0.5 } else { -0.5 });
                }
                (p, g)
            }
            Preset::Street => {
                let mut p = vec![];
                let mut g = vec![];
                for i in 0..10 {
                    let x = -4.5 + i as f64;
                    p.push(DVec2::new(x, 0.4));
                    g.push(1.0);
                    p.push(DVec2::new(x + 0.5, -0.4));
                    g.push(-1.0);
                }
                (p, g)
            }
        };
        self.pv = PointVortices::new(pos, gam);
        if self.preset == Preset::Gas {
            self.pv.core = 0.05;
        }
        self.trails = vec![Vec::new(); self.pv.pos.len()];
        self.h0 = self.pv.hamiltonian();
        self.l0 = self.pv.impulses().1;
        self.s_h.clear();
    }

    fn advance(&mut self, substeps: usize) {
        for _ in 0..substeps {
            self.pv.step(self.dt);
        }
        for (i, p) in self.pv.pos.iter().enumerate() {
            if i < self.trails.len() {
                self.trails[i].push(*p);
                if self.trails[i].len() > 300 {
                    self.trails[i].remove(0);
                }
            }
        }
        self.s_h.push(self.pv.time, (self.pv.hamiltonian() - self.h0) / self.h0.abs().max(1e-12));
    }

    fn controls(&mut self, ui: &mut Ui) -> bool {
        let mut ch = false;
        egui::ComboBox::from_label("preset")
            .selected_text(match self.preset {
                Preset::LikePair => "Like pair (orbits)",
                Preset::Dipole => "Opposite pair (translates)",
                Preset::Triangle => "Triangle (rigid rotation)",
                Preset::Gas => "Vortex gas (Onsager clustering)",
                Preset::Street => "Vortex street",
            })
            .show_ui(ui, |ui| {
                for (p, n) in [
                    (Preset::LikePair, "Like pair (orbits)"),
                    (Preset::Dipole, "Opposite pair (translates)"),
                    (Preset::Triangle, "Triangle (rigid rotation)"),
                    (Preset::Gas, "Vortex gas (Onsager clustering)"),
                    (Preset::Street, "Vortex street"),
                ] {
                    ch |= ui.selectable_value(&mut self.preset, p, n).changed();
                }
            });
        if self.preset == Preset::Gas {
            ch |= ui.add(Slider::new(&mut self.count, 4..=300).text("vortices")).changed();
        }
        ui.add(Slider::new(&mut self.dt, 0.001..=0.05).logarithmic(true).text("time step"));
        ui.add(Slider::new(&mut self.click_gamma, -3.0..=3.0).text("Γ for click-to-add"));
        ch
    }

    fn view(&mut self, ui: &mut Ui) {
        let (resp, painter) = self.view.interact(ui);
        if resp.clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                let w = self.view.to_world(p);
                self.pv.pos.push(w);
                self.pv.gamma.push(self.click_gamma);
                self.trails.push(Vec::new());
                self.h0 = self.pv.hamiltonian();
                self.l0 = self.pv.impulses().1;
            }
        }
        for (i, tr) in self.trails.iter().enumerate() {
            let c = if self.pv.gamma.get(i).copied().unwrap_or(0.0) >= 0.0 {
                Color32::from_rgba_unmultiplied(90, 200, 255, 110)
            } else {
                Color32::from_rgba_unmultiplied(255, 120, 90, 110)
            };
            self.view.polyline(&painter, tr, Stroke::new(1.0, c));
        }
        for (p, g) in self.pv.pos.iter().zip(&self.pv.gamma) {
            let c = if *g >= 0.0 { Color32::from_rgb(90, 200, 255) } else { Color32::from_rgb(255, 120, 90) };
            painter.circle_filled(self.view.to_screen(*p), 3.0 + 2.0 * g.abs() as f32, c);
        }
        painter.text(
            ui.max_rect().left_top() + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            "Click: add vortex · Right-drag: pan · Wheel: zoom · blue = +Γ (counter-clockwise), red = −Γ",
            egui::FontId::proportional(12.0),
            Color32::GRAY,
        );
    }

    fn diagnostics(&mut self, ui: &mut Ui) {
        metric(ui, "time", fmt(self.pv.time));
        metric(ui, "vortices", self.pv.pos.len().to_string());
        metric(ui, "energy H (2D Coulomb)", fmt(self.pv.hamiltonian()));
        metric(ui, "angular impulse Σ Γ r²", fmt(self.pv.impulses().1));
        ui.label("Relative energy drift");
        plot(ui, "ph", 120.0, &[("ΔH/H", &self.s_h, Color32::from_rgb(230, 120, 255))]);
    }

    fn checks(&self) -> Vec<Check> {
        vec![
            Check::abs(
                "Energy conserved (relative)",
                0.0,
                (self.pv.hamiltonian() - self.h0) / self.h0.abs().max(1e-12),
                1e-4,
                "Kirchhoff Hamiltonian",
            ),
            Check::abs(
                "Angular impulse conserved (relative)",
                0.0,
                (self.pv.impulses().1 - self.l0) / self.l0.abs().max(1e-12),
                1e-4,
                "rotational symmetry",
            ),
        ]
    }

    fn sim_time(&self) -> f64 {
        self.pv.time
    }
}
