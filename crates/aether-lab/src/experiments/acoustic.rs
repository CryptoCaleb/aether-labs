use super::{fmt, metric, Experiment};
use crate::view::View2D;
use aether_core::acoustic::{self, Bathtub, Ray, RayFate};
use aether_core::Check;
use egui::{Color32, Slider, Stroke, Ui};
use glam::DVec2;
use std::f64::consts::TAU;

pub struct AcousticExp {
    flow: Bathtub,
    view: View2D,
    source: DVec2,
    rays: Vec<Ray>,
    count: usize,
    reveal: usize,
    time: f64,
    show_flow: bool,
    beam: bool,
    beam_angle_deg: f64,
    cached_checks: Vec<Check>,
}

impl AcousticExp {
    pub fn new() -> Self {
        let mut e = Self {
            flow: Bathtub { a: 1.0, b: 1.5, c: 1.0 },
            view: View2D::new(12.0),
            source: DVec2::new(4.0, 0.5),
            rays: vec![],
            count: 72,
            reveal: 0,
            time: 0.0,
            show_flow: true,
            beam: false,
            beam_angle_deg: 180.0,
            cached_checks: Vec::new(),
        };
        e.reset();
        e
    }

    fn trace_all(&mut self) {
        let dt = 4e-3;
        let steps = 6000;
        self.rays = if self.beam {
            // Parallel beam with varying impact parameter.
            let a = self.beam_angle_deg.to_radians();
            let dir = DVec2::new(a.cos(), a.sin());
            let perp = DVec2::new(-dir.y, dir.x);
            (0..self.count)
                .map(|i| {
                    let b = -5.0 + 10.0 * i as f64 / (self.count - 1).max(1) as f64;
                    let start = -dir * 8.0 + perp * b;
                    acoustic::trace(&self.flow, Ray::new(start, dir), dt, steps)
                })
                .collect()
        } else {
            (0..self.count)
                .map(|i| {
                    let th = TAU * i as f64 / self.count as f64;
                    acoustic::trace(&self.flow, Ray::new(self.source, DVec2::new(th.cos(), th.sin())), dt, steps)
                })
                .collect()
        };
        self.reveal = 0;
        self.time = 0.0;
    }
}

impl AcousticExp {
    /// Rays launched inside the horizon must all be captured (run once per reset).
    fn horizon_checks(&self) -> Vec<Check> {
        let rh = self.flow.horizon();
        if rh <= 0.0 {
            return vec![];
        }
        let inside = (0..12).all(|i| {
            let th = TAU * i as f64 / 12.0;
            acoustic::trace(&self.flow, Ray::new(DVec2::new(0.95 * rh, 0.0), DVec2::new(th.cos(), th.sin())), 2e-3, 20_000).fate
                == RayFate::Captured
        });
        vec![Check::boolean("All rays launched inside r_H are captured", inside, "acoustic horizon (Unruh 1981)")]
    }
}

impl Experiment for AcousticExp {
    fn title(&self) -> &'static str {
        "Acoustic black hole"
    }
    fn category(&self) -> &'static str {
        "Gravity as flow (analog gravity)"
    }
    fn tagline(&self) -> &'static str {
        "Sound in a draining vortex sees curved spacetime"
    }
    fn theory(&self) -> &'static str {
        "ANALOG GRAVITY (Unruh 1981; Visser 1998). Sound in a moving fluid obeys the same wave equation as a field in curved spacetime. The acoustic metric is\n\nds² = (ρ/c)[−c²dt² + (dx − v dt)²]\n\nThat is the Painlevé–Gullstrand form of a black hole, with the medium 'falling' into the drain. This is the strongest modern support for 'gravity as aether flow'. It is real and has been tested: Weinfurtner 2011 (water waves) and Steinhauer 2016/2019 (Hawking radiation in a BEC).\n\nTHE BATHTUB. The flow v = (−A r̂ + B θ̂)/r has a horizon at r_H = A/c, where the inflow reaches the speed of sound and nothing escapes. It has an ergosurface at r_E = √(A²+B²)/c; inside it, waves are dragged around and cannot stand still (frame dragging, as around a spinning Kerr black hole). The analog surface gravity is κ = c²/A, giving a Hawking temperature ħκ/(2πk_B).\n\nHONEST LIMITS. The metric is KINEMATIC: it is set by how the fluid happens to flow, not by mass–energy through Einstein's equations. Different excitations generally see different metrics. And no fluid has been shown to carry spin-2 gravitational waves.\n\nClick in the view to move the sound source. Red rays are captured, cyan rays escape."
    }

    fn reset(&mut self) {
        self.trace_all();
        self.cached_checks = self.horizon_checks();
    }

    fn advance(&mut self, substeps: usize) {
        self.reveal += 8 * substeps;
        self.time += substeps as f64;
    }

    fn controls(&mut self, ui: &mut Ui) -> bool {
        let mut ch = false;
        ch |= ui.add(Slider::new(&mut self.flow.a, 0.0..=3.0).text("inflow A (drain)")).changed();
        ch |= ui.add(Slider::new(&mut self.flow.b, -4.0..=4.0).text("swirl B (circulation)")).changed();
        ch |= ui.add(Slider::new(&mut self.flow.c, 0.2..=3.0).text("speed of sound c")).changed();
        ch |= ui.add(Slider::new(&mut self.count, 8..=240).text("rays")).changed();
        ch |= ui.checkbox(&mut self.beam, "Parallel beam (impact parameter scan)").changed();
        if self.beam {
            ch |= ui.add(Slider::new(&mut self.beam_angle_deg, 0.0..=360.0).text("beam direction (°)")).changed();
        }
        ui.checkbox(&mut self.show_flow, "Show medium flow arrows");
        if ui.button("Replay wavefront").clicked() {
            self.reveal = 0;
        }
        ch
    }

    fn view(&mut self, ui: &mut Ui) {
        let (resp, painter) = self.view.interact(ui);
        if resp.clicked() && !self.beam {
            if let Some(p) = resp.interact_pointer_pos() {
                self.source = self.view.to_world(p);
                self.trace_all();
            }
        }
        if self.show_flow {
            let n = 14;
            for i in -n..=n {
                for j in -n..=n {
                    let p = DVec2::new(i as f64 * 0.5, j as f64 * 0.5);
                    if p.length() < 0.3 {
                        continue;
                    }
                    let v = self.flow.velocity(p);
                    let len = v.length();
                    let dir = v / len.max(1e-9) * (0.35 * (len / self.flow.c).min(1.0));
                    let col = if len > self.flow.c { Color32::from_rgb(110, 60, 60) } else { Color32::from_rgb(50, 60, 90) };
                    painter.arrow(self.view.to_screen(p), egui::vec2(self.view.len(dir.x), -self.view.len(dir.y)), Stroke::new(1.0, col));
                }
            }
        }
        // Ergosurface and horizon
        self.view.circle(&painter, DVec2::ZERO, self.flow.ergosurface(), Stroke::new(1.5, Color32::from_rgb(255, 170, 60)));
        self.view.circle(&painter, DVec2::ZERO, self.flow.horizon(), Stroke::new(2.5, Color32::from_rgb(255, 60, 60)));
        painter.circle_filled(self.view.to_screen(DVec2::ZERO), self.view.len(0.3 * self.flow.horizon()), Color32::BLACK);
        for r in &self.rays {
            let n = r.path.len().min(self.reveal.max(1));
            let col = match r.fate {
                RayFate::Captured => Color32::from_rgba_unmultiplied(255, 90, 90, 160),
                RayFate::Escaped => Color32::from_rgba_unmultiplied(90, 220, 255, 160),
                RayFate::Flying => Color32::from_rgba_unmultiplied(200, 200, 200, 120),
            };
            let pts: Vec<DVec2> = r.path[..n].iter().step_by(4).copied().collect();
            self.view.polyline(&painter, &pts, Stroke::new(1.2, col));
            if n < r.path.len() {
                painter.circle_filled(self.view.to_screen(r.path[n - 1]), 2.0, Color32::WHITE);
            }
        }
        if !self.beam {
            painter.circle_filled(self.view.to_screen(self.source), 5.0, Color32::from_rgb(255, 255, 140));
        }
        painter.text(
            ui.max_rect().left_top() + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            "Click: move source · Right-drag: pan · Wheel: zoom · red circle = horizon, orange = ergosurface",
            egui::FontId::proportional(12.0),
            Color32::GRAY,
        );
    }

    fn diagnostics(&mut self, ui: &mut Ui) {
        let cap = self.rays.iter().filter(|r| r.fate == RayFate::Captured).count();
        metric(ui, "horizon r_H = A/c", fmt(self.flow.horizon()));
        metric(ui, "ergosurface r_E", fmt(self.flow.ergosurface()));
        metric(ui, "surface gravity κ = c²/A", fmt(self.flow.surface_gravity()));
        metric(ui, "Hawking T (units ħ=k=1)", fmt(self.flow.surface_gravity() / TAU));
        metric(ui, "rays captured", format!("{cap} / {}", self.rays.len()));
        if self.beam {
            let min_b_escape = self
                .rays
                .iter()
                .enumerate()
                .filter(|(_, r)| r.fate == RayFate::Captured)
                .count();
            metric(ui, "capture fraction of beam", fmt(min_b_escape as f64 / self.rays.len().max(1) as f64));
        }
        if self.flow.b.abs() > 0.0 {
            ui.label("Rays are dragged with the swirl. The prograde and retrograde sides are asymmetric, like light around a rotating (Kerr) black hole.");
        }
    }

    fn checks(&self) -> Vec<Check> {
        self.cached_checks.clone()
    }

    fn sim_time(&self) -> f64 {
        self.time
    }
}
