use super::{fmt, metric, Experiment};
use crate::view::Camera;
use aether_core::filament::VortexSystem;
use aether_core::hierarchy::{self, hierarchical_ring, hollow_ring_speed, HierarchyStudy, Level};
use aether_core::Check;
use egui::{Color32, Slider, Ui};
use std::sync::mpsc::{channel, Receiver};
use std::f64::consts::TAU;

pub struct HierarchyExp {
    sys: VortexSystem,
    cam: Camera,
    depth: usize,
    n: [usize; 3],
    radius: [f64; 3],
    twist: [i32; 3],
    nodes: usize,
    dt: f64,
    smoothing: f64,
    z0: f64,
    study: Option<HierarchyStudy>,
    study_checks: Vec<Check>,
    rx: Option<Receiver<(HierarchyStudy, Vec<Check>)>>,
}

impl HierarchyExp {
    pub fn new() -> Self {
        let mut e = Self {
            sys: VortexSystem::new(vec![], 0.03, 0.05),
            cam: Camera::new(4.5),
            depth: 2,
            n: [5, 3, 3],
            radius: [0.22, 0.08, 0.03],
            twist: [1, 2, 3],
            nodes: 96,
            dt: 0.01,
            smoothing: 0.03,
            z0: 0.0,
            study: None,
            study_checks: vec![],
            rx: None,
        };
        e.reset();
        e
    }

    fn levels(&self) -> Vec<Level> {
        (0..self.depth)
            .map(|i| Level { n: self.n[i], radius: self.radius[i], twist: self.twist[i] })
            .collect()
    }

    fn start_study(&mut self) {
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let s = hierarchy::study();
            let mut c = hierarchy::checks_from(&s);
            c.extend(hierarchy::cluster_checks());
            let _ = tx.send((s, c));
        });
        self.rx = Some(rx);
    }

    fn total_filaments(&self) -> usize {
        self.levels().iter().map(|l| l.n).product::<usize>().max(1)
    }
}

impl Experiment for HierarchyExp {
    fn title(&self) -> &'static str {
        "Vortex of vortices"
    }
    fn category(&self) -> &'static str {
        "Scale hierarchy (quantum → classical)"
    }
    fn tagline(&self) -> &'static str {
        "A vortex made of smaller vortices… until they become nothing"
    }
    fn theory(&self) -> &'static str {
        "THE IDEA. Build a vortex ring out of N thinner vortices. Build each of those out of smaller ones, and so on. Total circulation Γ is the same at every level, while each piece carries Γ/N, then Γ/N², heading toward zero. Does the whole thing behave like one smooth classical vortex? And can the large-scale behaviour 'see' what it's made of?\n\nWHAT THE STUDY MEASURES ('Run study' button):\n• THE LIMIT. As N grows (4 → 64), the bundle's speed converges to Kelvin's HOLLOW ring, Γ/(4πR)[ln(8R/a) − ½], to within 1%. The sub-vortices really do 'become nothing': their imprint fades faster than 1/N.\n• SCALE BLINDNESS. Far away, the flow of a two-level hierarchy (6×4 = 24 filaments) matches a single plain ring to about 1 part in 10⁵, PROVIDED the plain ring carries the same impulse.\n• IMPULSE COUNTS. The microstructure adds exactly Γπa²/2 of impulse per level. Internal structure shows up at large scale only through this conserved total, much as a bound system's internal energy adds to its mass. That is how effective theories, and coarse-graining in general, work.\n• QUANTUM → CLASSICAL. Two clusters of 64 tiny vortices orbit exactly like two single vortices. In the superfluid experiment, the coarse-grained velocity of many QUANTISED vortices equals classical rigid rotation v = Ωr (Feynman 1955).\n\nWHAT THIS DOES NOT DO. It does not unify gravity with quantum mechanics. It shows how classical, large-scale vortex physics EMERGES from many small quantised vortices, which is the part of the idea that standard physics supports. The hard part is still open: making the emergent large-scale flow obey EINSTEIN's equations, with spin-2 gravitational waves and the equivalence principle. Volovik, Sakharov and the analog-gravity community have taken this route, and none has closed that gap. See the scorecard and the research library."
    }

    fn reset(&mut self) {
        let fils = hierarchical_ring(1.0, 1.0, self.nodes, &self.levels());
        // The core must be resolved by the node spacing (δ ≳ ds) or the filaments go
        // unstable; it must also stay thinner than the finest tube radius.
        let ds = TAU * 1.2 / self.nodes as f64;
        let finest = self.levels().last().map_or(0.3, |l| l.radius);
        let core = (0.9 * ds).max(0.3 * finest).min(0.12);
        let mut fils = fils;
        // Colour each braid by its top-level parent so the hierarchy is visible.
        let groups = self.levels().first().map_or(1, |l| l.n);
        let per = (fils.len() / groups).max(1);
        let palette = [[90, 200, 255], [255, 140, 80], [180, 255, 120], [230, 120, 255], [255, 220, 90], [120, 255, 230], [255, 120, 160], [160, 160, 255], [220, 255, 140], [255, 170, 210], [140, 220, 255], [255, 200, 140]];
        for (i, f) in fils.iter_mut().enumerate() {
            let c = palette[(i / per) % palette.len()];
            let k = 0.75 + 0.25 * ((i % per) as f64 / per.max(1) as f64);
            f.color = [(c[0] as f64 * k) as u8, (c[1] as f64 * k) as u8, (c[2] as f64 * k) as u8];
        }
        let mut sys = VortexSystem::new(fils, core, TAU / self.nodes as f64);
        sys.resample = false;
        sys.smoothing = self.smoothing;
        self.z0 = sys.filaments.iter().map(|f| f.centroid().z).sum::<f64>() / sys.filaments.len() as f64;
        self.sys = sys;
    }

    fn advance(&mut self, substeps: usize) {
        for _ in 0..substeps {
            self.sys.step(self.dt);
        }
    }

    fn controls(&mut self, ui: &mut Ui) -> bool {
        let mut ch = false;
        ch |= ui.add(Slider::new(&mut self.depth, 0..=3).text("hierarchy levels")).changed();
        for i in 0..self.depth {
            ui.label(format!("Level {}", i + 1));
            ch |= ui.add(Slider::new(&mut self.n[i], 2..=12).text("sub-vortices N")).changed();
            ch |= ui.add(Slider::new(&mut self.radius[i], 0.005..=0.3).logarithmic(true).text("tube radius a")).changed();
            ch |= ui.add(Slider::new(&mut self.twist[i], -4..=4).text("twist (turns)")).changed();
        }
        ch |= ui.add(Slider::new(&mut self.nodes, 32..=200).text("nodes per filament")).changed();
        ui.add(Slider::new(&mut self.dt, 0.002..=0.05).logarithmic(true).text("time step"));
        if ui.add(Slider::new(&mut self.smoothing, 0.0..=0.2).text("smoothing (numerical viscosity)")).changed() {
            self.sys.smoothing = self.smoothing;
        }
        ui.label(format!("{} filaments, {} nodes total, core δ = {:.3}", self.total_filaments(), self.sys.node_count(), self.sys.core));
        if self.levels().last().is_some_and(|l| self.sys.core >= l.radius) {
            ui.colored_label(Color32::from_rgb(240, 190, 80), "Finest level is thinner than the core: add nodes per filament to resolve it.");
        }
        ui.separator();
        let busy = self.rx.is_some();
        ui.horizontal(|ui| {
            if ui.add_enabled(!busy, egui::Button::new("▶ Run study (N → ∞ limit)")).clicked() {
                self.start_study();
            }
            if busy {
                ui.spinner();
                ui.label("computing (~20 s)…");
            }
        });
        ch
    }

    fn view(&mut self, ui: &mut Ui) {
        if let Some(rx) = &self.rx {
            if let Ok((s, c)) = rx.try_recv() {
                self.study = Some(s);
                self.study_checks = c;
                self.rx = None;
            } else {
                ui.ctx().request_repaint();
            }
        }
        // Follow the bundle as it flies.
        let zc = self.sys.filaments.iter().map(|f| f.centroid().z).sum::<f64>() / self.sys.filaments.len().max(1) as f64;
        self.cam.target.z += 0.1 * (zc - self.cam.target.z);
        let (_r, painter) = self.cam.interact(ui);
        self.cam.grid(&painter, 3.0, zc - 1.5, 0.5);
        let width = if self.sys.filaments.len() > 30 { 1.2 } else { 2.0 };
        for f in &self.sys.filaments {
            let c = Color32::from_rgb(f.color[0], f.color[1], f.color[2]);
            self.cam.polyline(&painter, &f.pts, true, width, c);
        }
    }

    fn diagnostics(&mut self, ui: &mut Ui) {
        let zc = self.sys.filaments.iter().map(|f| f.centroid().z).sum::<f64>() / self.sys.filaments.len().max(1) as f64;
        let a = self.levels().first().map_or(0.0, |l| l.radius);
        metric(ui, "time", fmt(self.sys.time));
        metric(ui, "filaments", self.sys.filaments.len().to_string());
        metric(ui, "circulation per filament", fmt(1.0 / self.total_filaments() as f64));
        metric(ui, "bundle speed (measured)", fmt(if self.sys.time > 0.0 { (zc - self.z0) / self.sys.time } else { f64::NAN }));
        if a > 0.0 {
            metric(ui, "hollow-ring limit (N→∞)", fmt(hollow_ring_speed(1.0, 1.0, a)));
        }
        if let Some(s) = &self.study {
            ui.separator();
            ui.strong("Convergence: ring of N sub-vortices (R=1, a=0.1)");
            let pts: Vec<[f64; 2]> = s.ns.iter().zip(&s.speeds).map(|(n, u)| [1.0 / *n as f64, *u]).collect();
            let lim = vec![[0.0, s.hollow], [0.26, s.hollow]];
            egui_plot::Plot::new("hconv").height(150.0).allow_scroll(false).legend(egui_plot::Legend::default()).show(ui, |p| {
                p.points(egui_plot::Points::new("U(N) vs 1/N", pts.clone()).radius(4.0).color(Color32::from_rgb(255, 140, 80)));
                p.line(egui_plot::Line::new("U(N)", pts).color(Color32::from_rgb(255, 140, 80)));
                p.line(egui_plot::Line::new("hollow ring (1/N → 0)", lim).color(Color32::from_rgb(90, 200, 255)));
            });
            ui.label("x-axis: 1/N (circulation of each sub-vortex). At x → 0 they 'become nothing' and the ring is classical.");
            ui.strong("Far-field difference vs a plain ring with the same impulse");
            let fd: Vec<[f64; 2]> = s.far_d.iter().zip(&s.far_diff).map(|(d, v)| [*d, *v]).collect();
            egui_plot::Plot::new("hfar").height(110.0).allow_scroll(false).show(ui, |p| {
                p.line(egui_plot::Line::new("relative difference", fd).color(Color32::from_rgb(180, 255, 120)));
            });
            metric(ui, "same radius, no impulse match", fmt(s.naive_far_diff));
            metric(ui, "predicted impulse excess Σa²/2R²", fmt(s.impulse_excess));
        }
    }

    fn checks(&self) -> Vec<Check> {
        self.study_checks.clone()
    }

    fn sim_time(&self) -> f64 {
        self.sys.time
    }
}
