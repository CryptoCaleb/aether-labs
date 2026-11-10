use super::{fmt, metric, Experiment};
use aether_core::sinegordon::{self, gamma, SineGordon};
use aether_core::Check;
use egui::{Color32, Slider, Ui};


#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Kink,
    Breather,
    Collision,
}

pub struct RelativityExp {
    mode: Mode,
    v: f64,
    omega: f64,
    sg: SineGordon,
    rest: SineGordon,
    steps_per_frame: usize,
    e0: f64,
    clock_crossings: Vec<f64>,
    rest_crossings: Vec<f64>,
    last_moving: f64,
    last_rest: f64,
}

const N: usize = 3001;
const L: f64 = 60.0;
const X0: f64 = -15.0;

impl RelativityExp {
    pub fn new() -> Self {
        let mut e = Self {
            mode: Mode::Kink,
            v: 0.8,
            omega: 0.5,
            sg: SineGordon::from_solution(10, 1.0, |_, _| 0.0),
            rest: SineGordon::from_solution(10, 1.0, |_, _| 0.0),
            steps_per_frame: 8,
            e0: 0.0,
            clock_crossings: vec![],
            rest_crossings: vec![],
            last_moving: 0.0,
            last_rest: 0.0,
        };
        e.reset();
        e
    }

    fn analytic(&self, x: f64, t: f64) -> f64 {
        match self.mode {
            Mode::Kink => sinegordon::kink(x, t, self.v, X0),
            Mode::Breather => sinegordon::breather(x, t, self.omega, self.v, X0),
            Mode::Collision => sinegordon::kink(x, t, self.v, X0) - sinegordon::kink(x, t, -self.v, -X0),
        }
    }

    fn period(c: &[f64]) -> f64 {
        if c.len() < 2 {
            return f64::NAN;
        }
        (c[c.len() - 1] - c[0]) / (c.len() - 1) as f64
    }
}

impl Experiment for RelativityExp {
    fn title(&self) -> &'static str {
        "Emergent relativity (sine-Gordon)"
    }
    fn category(&self) -> &'static str {
        "Relativity from a medium"
    }
    fn tagline(&self) -> &'static str {
        "Why Michelson–Morley sees no aether wind"
    }
    fn theory(&self) -> &'static str {
        "THE MICHELSON–MORLEY PROBLEM. A plain fluid aether predicts an 'aether wind': light should be slower against the flow. Michelson and Morley (1887) found nothing, and modern tests put isotropy at better than 1 part in 10¹⁷.\n\nTHE LORENTZ–LARMOR ESCAPE. If rulers and clocks are themselves made of excitations of the same medium, they distort with motion in exactly the way that hides the wind. This experiment shows that happening mechanically. The medium is a chain of pendulums coupled by springs, whose continuum limit is the sine-Gordon equation φ_tt = c²φ_xx − sin φ.\n\n• A KINK is a particle-like twist. Moving at v, it CONTRACTS by γ = 1/√(1−v²/c²), and its energy grows as 8γ (E = γmc²) — nobody put relativity in.\n• A BREATHER is an internal oscillator, a 'clock'. Moving, it TICKS SLOWER by γ (time dilation).\n\nThe grey curve is a copy at rest, for comparison.\n\nWHAT THIS DOES AND DOESN'T SHOW. A medium whose excitations all obey one wave equation with one speed c makes motion through that medium undetectable from inside. That is Lorentz Ether Theory, which is empirically equivalent to special relativity. The open problem is UNIVERSALITY: every species of particle must share the same c to about 1 part in 10²⁰, and a generic medium does not deliver that."
    }

    fn reset(&mut self) {
        let a = |x: f64, t: f64| -> f64 {
            match self.mode {
                Mode::Kink => sinegordon::kink(x, t, self.v, X0),
                Mode::Breather => sinegordon::breather(x, t, self.omega, self.v, X0),
                Mode::Collision => sinegordon::kink(x, t, self.v, X0) - sinegordon::kink(x, t, -self.v, -X0),
            }
        };
        self.sg = SineGordon::from_solution(N, L, a);
        let (mode, om) = (self.mode, self.omega);
        self.rest = SineGordon::from_solution(N, L, move |x, t| match mode {
            Mode::Breather => sinegordon::breather(x, t, om, 0.0, X0),
            _ => sinegordon::kink(x, t, 0.0, X0),
        });
        self.e0 = self.sg.energy();
        self.clock_crossings.clear();
        self.rest_crossings.clear();
        self.last_moving = self.sg.sample(X0);
        self.last_rest = self.rest.sample(X0);
    }

    fn advance(&mut self, substeps: usize) {
        for _ in 0..substeps * self.steps_per_frame {
            self.sg.step();
            self.rest.step();
            if self.mode == Mode::Breather {
                let xm = X0 + self.v * self.sg.time;
                let vm = self.sg.sample(xm);
                if self.last_moving <= 0.0 && vm > 0.0 {
                    self.clock_crossings.push(self.sg.time);
                }
                self.last_moving = vm;
                let vr = self.rest.sample(X0);
                if self.last_rest <= 0.0 && vr > 0.0 {
                    self.rest_crossings.push(self.rest.time);
                }
                self.last_rest = vr;
            }
        }
    }

    fn controls(&mut self, ui: &mut Ui) -> bool {
        let mut ch = false;
        ui.horizontal(|ui| {
            ch |= ui.selectable_value(&mut self.mode, Mode::Kink, "Kink (ruler)").changed();
            ch |= ui.selectable_value(&mut self.mode, Mode::Breather, "Breather (clock)").changed();
            ch |= ui.selectable_value(&mut self.mode, Mode::Collision, "Kink–antikink").changed();
        });
        ch |= ui.add(Slider::new(&mut self.v, 0.0..=0.97).text("velocity v / c")).changed();
        if self.mode == Mode::Breather {
            ch |= ui.add(Slider::new(&mut self.omega, 0.2..=0.95).text("breather frequency ω")).changed();
        }
        ui.add(Slider::new(&mut self.steps_per_frame, 1..=40).text("solver steps per tick"));
        ui.label(format!("γ = {:.4}", gamma(self.v)));
        ch
    }

    fn view(&mut self, ui: &mut Ui) {
        let moving: Vec<[f64; 2]> = (0..self.sg.phi.len()).step_by(3).map(|i| [self.sg.x(i), self.sg.phi[i]]).collect();
        let rest: Vec<[f64; 2]> = (0..self.rest.phi.len()).step_by(3).map(|i| [self.rest.x(i), self.rest.phi[i]]).collect();
        let t = self.sg.time;
        let analytic: Vec<[f64; 2]> = (0..self.sg.phi.len()).step_by(6).map(|i| [self.sg.x(i), self.analytic(self.sg.x(i), t)]).collect();
        egui_plot::Plot::new("sg")
            .legend(egui_plot::Legend::default())
            .allow_scroll(false)
            .include_y(-1.0)
            .include_y(7.0)
            .show(ui, |p| {
                p.line(egui_plot::Line::new("at rest (reference)", rest).color(Color32::from_gray(110)));
                p.line(egui_plot::Line::new("simulated medium φ(x,t)", moving).color(Color32::from_rgb(90, 200, 255)).width(2.0));
                p.line(
                    egui_plot::Line::new("exact Lorentz-boosted solution", analytic)
                        .color(Color32::from_rgb(255, 140, 80))
                        .style(egui_plot::LineStyle::dashed_loose()),
                );
            });
    }

    fn diagnostics(&mut self, ui: &mut Ui) {
        let g = gamma(self.v);
        metric(ui, "time", fmt(self.sg.time));
        metric(ui, "γ", fmt(g));
        metric(ui, "energy drift", fmt((self.sg.energy() - self.e0) / self.e0.abs().max(1e-12)));
        match self.mode {
            Mode::Kink => {
                let w = self.sg.kink_width();
                metric(ui, "kink width w", fmt(w));
                metric(ui, "w × γ (should be 1)", fmt(w * g));
                metric(ui, "energy / 8γ (should be 1)", fmt(self.sg.energy() / (8.0 * g)));
            }
            Mode::Breather => {
                let pm = Self::period(&self.clock_crossings);
                let pr = Self::period(&self.rest_crossings);
                metric(ui, "moving clock period", fmt(pm));
                metric(ui, "rest clock period", fmt(pr));
                metric(ui, "ratio (should be γ)", fmt(pm / pr));
            }
            Mode::Collision => {
                ui.label("Kink and antikink attract, pass through each other and come out unchanged apart from a phase shift. That is soliton integrability, a particle-like collision with no debris.");
            }
        }
    }

    fn checks(&self) -> Vec<Check> {
        let g = gamma(self.v);
        match self.mode {
            Mode::Kink => vec![
                Check::rel("Length contraction w·γ = w₀", 1.0, self.sg.kink_width() * g, 0.02, "Lorentz–FitzGerald"),
                Check::rel("Energy E = 8γ", 1.0, self.sg.energy() / (8.0 * g), 0.02, "E = γmc²"),
            ],
            Mode::Breather => {
                let pm = Self::period(&self.clock_crossings);
                let pr = Self::period(&self.rest_crossings);
                if pm.is_finite() && pr.is_finite() {
                    vec![Check::rel("Moving clock slows by γ", g, pm / pr, 0.03, "time dilation")]
                } else {
                    vec![]
                }
            }
            Mode::Collision => vec![],
        }
    }

    fn sim_time(&self) -> f64 {
        self.sg.time
    }
}
