use super::{fmt, metric, plot, Experiment, Series};
use crate::view::Camera;
use aether_core::filament::{self, linking_number, writhe, VortexSystem};
use aether_core::{shapes, Check, Rng};
use egui::{Color32, Slider, Ui};
use glam::DVec3;
use std::f64::consts::TAU;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Ring,
    Leapfrog,
    Collision,
    Wavy,
    HopfLink,
    Trefoil,
}

pub struct FilamentExp {
    kind: Kind,
    sys: VortexSystem,
    cam: Camera,
    // Parameters
    radius: f64,
    core: f64,
    gamma: f64,
    nodes: usize,
    separation: f64,
    mode: u32,
    amp: f64,
    dt: f64,
    show_tracers: bool,
    show_velocity: bool,
    // Tracers: passive particles of the medium.
    tracers: Vec<DVec3>,
    trails: Vec<Vec<DVec3>>,
    // Diagnostics
    z0: Vec<f64>,
    energy0: f64,
    impulse0: DVec3,
    /// Σ|P_i|: the scale for impulse drift (total P can be ~0, e.g. head-on rings).
    impulse_scale: f64,
    s_z: Vec<Series>,
    s_r: Vec<Series>,
    s_energy: Series,
    s_topo: Series,
    last_energy: f64,
    last_topo: f64,
    frame: u64,
}

const COLORS: [[u8; 3]; 4] = [[90, 200, 255], [255, 140, 80], [180, 255, 120], [230, 120, 255]];

impl FilamentExp {
    pub fn new(kind: Kind) -> Self {
        let mut e = Self {
            kind,
            sys: VortexSystem::new(Vec::new(), 0.1, 0.05),
            cam: Camera::new(6.0),
            radius: 1.0,
            core: 0.1,
            gamma: 1.0,
            nodes: 160,
            separation: 0.6,
            mode: 6,
            amp: 0.04,
            dt: 0.04,
            show_tracers: true,
            show_velocity: false,
            tracers: Vec::new(),
            trails: Vec::new(),
            z0: Vec::new(),
            energy0: 0.0,
            impulse0: DVec3::ZERO,
            impulse_scale: 1.0,
            s_z: Vec::new(),
            s_r: Vec::new(),
            s_energy: Series::default(),
            s_topo: Series::default(),
            last_energy: 0.0,
            last_topo: f64::NAN,
            frame: 0,
        };
        match kind {
            Kind::Collision => e.separation = 3.0,
            Kind::Trefoil => {
                e.nodes = 300;
                e.core = 0.08;
            }
            Kind::HopfLink => e.nodes = 140,
            Kind::Wavy => {
                e.nodes = 240;
                e.core = 0.05;
                e.amp = 0.05;
            }
            _ => {}
        }
        e.reset();
        e
    }

    fn build(&self) -> Vec<filament::Filament> {
        let (r, n, g) = (self.radius, self.nodes, self.gamma);
        match self.kind {
            Kind::Ring => vec![shapes::ring(DVec3::ZERO, DVec3::Z, r, n, g, COLORS[0])],
            Kind::Leapfrog => vec![
                shapes::ring(DVec3::ZERO, DVec3::Z, r, n, g, COLORS[0]),
                shapes::ring(DVec3::new(0.0, 0.0, self.separation), DVec3::Z, r, n, g, COLORS[1]),
            ],
            Kind::Collision => vec![
                shapes::ring(DVec3::new(0.0, 0.0, -0.5 * self.separation), DVec3::Z, r, n, g, COLORS[0]),
                shapes::ring(DVec3::new(0.0, 0.0, 0.5 * self.separation), -DVec3::Z, r, n, g, COLORS[1]),
            ],
            Kind::Wavy => vec![shapes::perturbed_ring(DVec3::ZERO, DVec3::Z, r, n, g, COLORS[0], self.mode, self.amp)],
            Kind::HopfLink => vec![
                shapes::ring(DVec3::ZERO, DVec3::Z, r, n, g, COLORS[0]),
                shapes::ring(DVec3::new(r, 0.0, 0.0), DVec3::Y, r, n, g, COLORS[1]),
            ],
            Kind::Trefoil => vec![shapes::torus_knot(2, 3, r, 0.45 * r, n, g, COLORS[2])],
        }
    }

    fn seed_tracers(&mut self) {
        let mut rng = Rng::new(42);
        let s = 2.2 * self.radius;
        self.tracers = (0..150)
            .map(|_| DVec3::new(rng.range(-s, s), rng.range(-s, s), rng.range(-s, s * 1.5)))
            .collect();
        self.trails = vec![Vec::new(); self.tracers.len()];
    }

    fn spacing(&self) -> f64 {
        let len: f64 = self.sys.filaments.iter().map(|f| f.length()).sum::<f64>()
            / self.sys.filaments.len().max(1) as f64;
        let n = self.sys.filaments.first().map(|f| f.n()).unwrap_or(self.nodes);
        len / n as f64
    }

    fn ring_speed_prediction(&self) -> f64 {
        filament::rm_ring_speed(self.gamma, self.radius, self.core)
    }

    fn measured_speed(&self) -> f64 {
        match self.sys.filaments.first() {
            Some(f) if self.sys.time > 0.0 => (f.centroid().z - self.z0[0]) / self.sys.time,
            _ => f64::NAN,
        }
    }

    fn sample_series(&mut self) {
        let t = self.sys.time;
        for (i, f) in self.sys.filaments.iter().enumerate() {
            if i < self.s_z.len() {
                self.s_z[i].push(t, f.centroid().z);
                self.s_r[i].push(t, f.impulse_radius());
            }
        }
        if self.frame % 30 == 0 {
            self.last_energy = self.sys.energy();
            self.s_energy.push(t, (self.last_energy - self.energy0) / self.energy0.abs().max(1e-12));
            let topo = match self.kind {
                Kind::HopfLink if self.sys.filaments.len() == 2 => linking_number(&self.sys.filaments[0], &self.sys.filaments[1]),
                Kind::Trefoil | Kind::Wavy | Kind::Ring => writhe(&self.sys.filaments[0]),
                _ => f64::NAN,
            };
            self.last_topo = topo;
            self.s_topo.push(t, topo);
        }
    }
}

impl Experiment for FilamentExp {
    fn title(&self) -> &'static str {
        match self.kind {
            Kind::Ring => "Kelvin vortex ring",
            Kind::Leapfrog => "Leapfrogging rings",
            Kind::Collision => "Head-on ring collision",
            Kind::Wavy => "Kelvin waves on a ring",
            Kind::HopfLink => "Linked rings (Hopf link)",
            Kind::Trefoil => "Trefoil vortex knot",
        }
    }

    fn category(&self) -> &'static str {
        "Vortex matter (3D filaments)"
    }

    fn tagline(&self) -> &'static str {
        match self.kind {
            Kind::Ring => "Does a ring of aether move like Kelvin predicted?",
            Kind::Leapfrog => "Two 'atoms' passing through each other",
            Kind::Collision => "Opposite rings meet and spread",
            Kind::Wavy => "Vibrations of a vortex atom: Kelvin's spectral lines",
            Kind::HopfLink => "Topology frozen into an ideal medium",
            Kind::Trefoil => "Kelvin's knotted atom, Kleckner & Irvine 2013",
        }
    }

    fn theory(&self) -> &'static str {
        match self.kind {
            Kind::Ring => "KELVIN'S VORTEX ATOM (1867). Kelvin proposed that atoms are vortex rings in a perfect-fluid aether. Helmholtz (1858) proved that vortex lines move with the fluid and their circulation Γ never changes, so such atoms would be indestructible.\n\nWHAT IS MEASURED. A thin ring of radius R moves itself forward at U = Γ/(4πR)[ln(8R/a) − 1/4] (Kelvin, uniform core of radius a). This engine uses a Rosenhead–Moore core δ, for which U = Γ/(4πR)[ln(8R/δ) − 1]; the two match when a = δ·e^{3/4}. The live check compares the simulated speed with the formula. Try a larger R: the ring gets SLOWER. More energy giving less speed is the opposite of how particles behave.\n\nVERDICT SO FAR. The fluid mechanics is correct; Kelvin's formula holds. As particles, though, nothing fixes the ring's size, so nothing fixes its 'mass'. Thin rings are also unstable (Widnall 1974).",
            Kind::Leapfrog => "LEAPFROGGING (Helmholtz 1858). Two coaxial rings of the same sign. The rear ring is squeezed by the front ring's flow, shrinks, speeds up and passes through, and then the roles swap. Watch the centroid plot weave.\n\nTHE ATOM ANALOGY. Vortex atoms interact only through the medium's flow, and here two of them form a bound, oscillating 'molecule'. Total impulse and energy are conserved (see the drift readouts). In real fluids leapfrogging lasts only a few cycles before the rings merge. Acheson (2000) found the 2D model stable only for size ratios above about 0.38.",
            Kind::Collision => "HEAD-ON COLLISION. Two rings of opposite circulation move towards each other. Each slows down while its radius grows sharply, which conserves impulse: the radii run away to infinity in an ideal fluid. Real experiments show reconnection and many small secondary rings. This engine does not yet model reconnection (planned).",
            Kind::Wavy => "KELVIN WAVES. Kelvin hoped the vibration modes of vortex atoms would explain spectral lines. A ring perturbed by mode m carries helical waves whose dispersion is ω ≈ (Γk²/4π)[ln(1/ka)+c]. That is quadratic, unlike the discrete, Rydberg-like spectra of real atoms.\n\nAt larger amplitude the Widnall instability grows and the ring breaks up. This is one reason vortex atoms are not stable.",
            Kind::HopfLink => "LINKED VORTICES. Two rings threaded through each other. In an ideal medium the vortex lines are frozen in, so the Gauss linking number Lk is a conserved topological charge. Helicity H = Σ Γ_iΓ_j Lk_ij + self-terms is the fluid version of magnetic helicity.\n\nThis is the strongest part of the vortex-atom idea: topology gives integer, conserved quantities, much like charge quantisation. The catch: in real fluids and superfluids vortices RECONNECT and topology changes (Bewley et al. 2008; Kleckner & Irvine 2013).",
            Kind::Trefoil => "TREFOIL KNOT. Tait tabulated knots so that each knot type could be a chemical element in Kelvin's theory. Kleckner & Irvine (2013) made real trefoil vortices in water. Every one stretched, reconnected, and came apart into unknotted rings. No knot survived.\n\nThis engine is ideal and has no reconnection, so the knot type cannot change here. Watch for the strands coming close together; that is where a real medium would reconnect. The writhe readout tracks how the coiling moves around.",
        }
    }

    fn reset(&mut self) {
        let fils = self.build();
        let spacing = TAU * self.radius / self.nodes as f64;
        self.sys = VortexSystem::new(fils, self.core, spacing);
        self.sys.spacing = self.spacing();
        self.z0 = self.sys.filaments.iter().map(|f| f.centroid().z).collect();
        self.energy0 = self.sys.energy();
        self.impulse0 = self.sys.impulse();
        self.impulse_scale = self.sys.filaments.iter().map(|f| f.impulse().length()).sum::<f64>().max(1e-12);
        self.last_energy = self.energy0;
        self.s_z = vec![Series::default(); self.sys.filaments.len()];
        self.s_r = vec![Series::default(); self.sys.filaments.len()];
        self.s_energy.clear();
        self.s_topo.clear();
        self.frame = 0;
        self.seed_tracers();
        self.cam.target = DVec3::new(0.0, 0.0, if self.kind == Kind::Leapfrog { 1.5 } else { 0.0 });
        self.sample_series();
    }

    fn advance(&mut self, substeps: usize) {
        for _ in 0..substeps {
            self.sys.step(self.dt);
            // Stretching filaments keep adding nodes; coarsen the target spacing
            // so the O(N²) Biot–Savart cost stays interactive.
            if self.sys.node_count() > 1200 {
                self.sys.spacing *= 1.15;
            }
        }
        if self.show_tracers {
            let dt = self.dt * substeps as f64;
            let sys = &self.sys;
            use rayon::prelude::*;
            let vel: Vec<DVec3> = self.tracers.par_iter().map(|p| sys.velocity_at(*p)).collect();
            for (i, p) in self.tracers.iter_mut().enumerate() {
                *p += vel[i] * dt;
                self.trails[i].push(*p);
                if self.trails[i].len() > 40 {
                    self.trails[i].remove(0);
                }
            }
        }
        self.frame += 1;
        self.sample_series();
    }

    fn controls(&mut self, ui: &mut Ui) -> bool {
        let mut ch = false;
        ch |= ui.add(Slider::new(&mut self.radius, 0.3..=3.0).text("ring radius R")).changed();
        ch |= ui.add(Slider::new(&mut self.core, 0.01..=0.4).logarithmic(true).text("core δ")).changed();
        ch |= ui.add(Slider::new(&mut self.gamma, -3.0..=3.0).text("circulation Γ")).changed();
        ch |= ui.add(Slider::new(&mut self.nodes, 40..=600).text("nodes per filament")).changed();
        match self.kind {
            Kind::Leapfrog | Kind::Collision => {
                ch |= ui.add(Slider::new(&mut self.separation, 0.2..=6.0).text("separation")).changed();
            }
            Kind::Wavy => {
                ch |= ui.add(Slider::new(&mut self.mode, 1..=16).text("mode m")).changed();
                ch |= ui.add(Slider::new(&mut self.amp, 0.0..=0.3).text("amplitude")).changed();
            }
            _ => {}
        }
        ui.add(Slider::new(&mut self.dt, 0.005..=0.2).logarithmic(true).text("time step"));
        ui.checkbox(&mut self.show_tracers, "Show aether tracers (passive particles)");
        ui.checkbox(&mut self.show_velocity, "Show node velocity vectors");
        ui.checkbox(&mut self.sys.resample, "Adaptive resampling");
        ui.horizontal(|ui| {
            ui.label("Aether wind (background flow) z:");
            ui.add(egui::DragValue::new(&mut self.sys.background.z).speed(0.01));
        });
        ch
    }

    fn view(&mut self, ui: &mut Ui) {
        let (_resp, painter) = self.cam.interact(ui);
        self.cam.grid(&painter, 4.0, -2.5, 0.5);
        if self.show_tracers {
            for tr in &self.trails {
                for w in tr.windows(2) {
                    self.cam.line(&painter, w[0], w[1], 1.0, Color32::from_rgba_unmultiplied(120, 130, 170, 90));
                }
            }
        }
        let vels = if self.show_velocity { Some(self.sys.velocities()) } else { None };
        for (fi, f) in self.sys.filaments.iter().enumerate() {
            let c = Color32::from_rgb(f.color[0], f.color[1], f.color[2]);
            self.cam.polyline(&painter, &f.pts, true, 2.5, c);
            if let Some(v) = &vels {
                for (p, u) in f.pts.iter().zip(&v[fi]).step_by(4) {
                    self.cam.line(&painter, *p, *p + *u * 0.6, 1.0, Color32::from_rgb(255, 230, 120));
                }
            }
        }
        painter.text(
            ui.max_rect().left_top() + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            "Left-drag: orbit · Right-drag: pan · Wheel: zoom",
            egui::FontId::proportional(12.0),
            Color32::GRAY,
        );
    }

    fn diagnostics(&mut self, ui: &mut Ui) {
        metric(ui, "time", fmt(self.sys.time));
        metric(ui, "nodes", self.sys.node_count().to_string());
        let p = self.sys.impulse();
        metric(ui, "impulse drift |ΔP|/Σ|Pᵢ|", fmt((p - self.impulse0).length() / self.impulse_scale));
        metric(ui, "energy drift ΔE/E", fmt((self.last_energy - self.energy0) / self.energy0.abs().max(1e-12)));
        match self.kind {
            Kind::Ring => {
                metric(ui, "measured speed", fmt(self.measured_speed()));
                metric(ui, "Kelvin/RM prediction", fmt(self.ring_speed_prediction()));
                metric(ui, "equivalent Kelvin core a", fmt(self.core * 0.75f64.exp()));
            }
            Kind::HopfLink if self.sys.filaments.len() == 2 => {
                metric(ui, "linking number Lk", fmt(self.last_topo));
            }
            Kind::Trefoil | Kind::Wavy => {
                metric(ui, "writhe", fmt(self.last_topo));
                metric(ui, "length", fmt(self.sys.filaments[0].length()));
            }
            _ => {}
        }
        ui.separator();
        let cols = [Color32::from_rgb(90, 200, 255), Color32::from_rgb(255, 140, 80)];
        ui.label("Centroid z(t)");
        let lines: Vec<(&str, &Series, Color32)> =
            self.s_z.iter().enumerate().take(2).map(|(i, s)| (if i == 0 { "ring A" } else { "ring B" }, s, cols[i])).collect();
        plot(ui, "fz", 120.0, &lines);
        ui.label("Effective radius √(|P|/πΓ)");
        let lines: Vec<(&str, &Series, Color32)> =
            self.s_r.iter().enumerate().take(2).map(|(i, s)| (if i == 0 { "ring A" } else { "ring B" }, s, cols[i])).collect();
        plot(ui, "fr", 110.0, &lines);
        if !self.s_topo.pts.is_empty() && self.s_topo.pts.iter().all(|p| p[1].is_finite()) {
            ui.label(if self.kind == Kind::HopfLink { "Linking number" } else { "Writhe" });
            plot(ui, "ft", 90.0, &[("topology", &self.s_topo, Color32::from_rgb(180, 255, 120))]);
        }
        ui.label("Relative energy drift");
        plot(ui, "fe", 80.0, &[("ΔE/E", &self.s_energy, Color32::from_rgb(230, 120, 255))]);
    }

    fn checks(&self) -> Vec<Check> {
        let mut v = vec![Check::abs(
            "Impulse conserved (relative drift)",
            0.0,
            (self.sys.impulse() - self.impulse0).length() / self.impulse_scale,
            1e-3,
            "Helmholtz: total impulse is invariant",
        )];
        if self.kind == Kind::Ring && self.sys.time > 0.5 && self.sys.background == DVec3::ZERO {
            v.push(Check::rel(
                "Ring speed vs Kelvin/RM formula",
                self.ring_speed_prediction(),
                self.measured_speed(),
                0.03,
                "Kelvin 1867; Saffman 1992",
            ));
        }
        if self.kind == Kind::HopfLink && self.sys.filaments.len() == 2 {
            v.push(Check::rel(
                "Linking number stays ±1",
                1.0,
                self.last_topo.abs(),
                0.05,
                "topology frozen in ideal flow",
            ));
        }
        v
    }

    fn sim_time(&self) -> f64 {
        self.sys.time
    }
}
