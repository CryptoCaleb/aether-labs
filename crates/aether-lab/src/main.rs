#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod experiments;
mod library;
mod scorecard;
mod view;

use eframe::egui;
use egui::{Color32, RichText, Slider};
use experiments::{checks_table, Experiment};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Command-line options (also used for automated screenshots / verification).
#[derive(Default, Clone)]
struct Cli {
    experiment: Option<usize>,
    page: Option<String>,
    screenshot: Option<String>,
    frames: u64,
}

fn parse_cli() -> Cli {
    let mut cli = Cli { frames: 120, ..Default::default() };
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        let next = args.get(i + 1).cloned();
        match args[i].as_str() {
            "--experiment" => cli.experiment = next.and_then(|v| v.parse().ok()),
            "--page" => cli.page = next,
            "--screenshot" => cli.screenshot = next,
            "--frames" => cli.frames = next.and_then(|v| v.parse().ok()).unwrap_or(120),
            _ => {
                i += 1;
                continue;
            }
        }
        i += 2;
    }
    cli
}

fn save_png(path: &str, img: &egui::ColorImage) -> std::io::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), img.size[0] as u32, img.size[1] as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().map_err(std::io::Error::other)?;
    let bytes: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
    w.write_image_data(&bytes).map_err(std::io::Error::other)
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Page {
    Lab,
    Scorecard,
    Library,
}

struct AetherLab {
    experiments: Vec<Box<dyn Experiment>>,
    selected: usize,
    page: Page,
    running: bool,
    substeps: usize,
    scorecard: scorecard::Scorecard,
    library: library::Library,
    show_theory: bool,
    frame_ms: f64,
    cli: Cli,
    frame: u64,
    shot: Arc<Mutex<Option<(String, Arc<egui::ColorImage>)>>>,
    shot_requested: bool,
    status: String,
    checks_cache: Vec<aether_core::Check>,
    checks_at: Option<Instant>,
    checks_for: usize,
}

impl AetherLab {
    fn new(cc: &eframe::CreationContext<'_>, cli: Cli) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(16, 18, 28);
        visuals.window_fill = Color32::from_rgb(20, 22, 34);
        visuals.selection.bg_fill = Color32::from_rgb(40, 90, 150);
        cc.egui_ctx.set_visuals(visuals);
        Self {
            experiments: experiments::all(),
            selected: 0,
            page: Page::Lab,
            running: true,
            substeps: 1,
            scorecard: scorecard::Scorecard::default(),
            library: library::Library::default(),
            show_theory: true,
            frame_ms: 0.0,
            cli,
            frame: 0,
            shot: Arc::new(Mutex::new(None)),
            shot_requested: false,
            status: String::new(),
            checks_cache: Vec::new(),
            checks_at: None,
            checks_for: usize::MAX,
        }
        .with_cli()
    }

    fn with_cli(mut self) -> Self {
        if let Some(e) = self.cli.experiment {
            self.selected = e.min(self.experiments.len() - 1);
        }
        match self.cli.page.as_deref() {
            Some("scorecard") => {
                self.page = Page::Scorecard;
                self.scorecard.run();
            }
            Some("library") => self.page = Page::Library,
            _ => {}
        }
        self
    }

    fn request_snapshot(&mut self, ctx: &egui::Context, path: String) {
        let slot = self.shot.clone();
        ctx.request_screenshot(move |img| {
            *slot.lock().unwrap() = Some((path, img));
        });
        ctx.request_repaint();
    }

    fn handle_snapshots(&mut self, ctx: &egui::Context) {
        self.frame += 1;
        if let Some(path) = self.cli.screenshot.clone() {
            if !self.shot_requested && self.frame >= self.cli.frames {
                self.shot_requested = true;
                self.request_snapshot(ctx, path);
            }
            ctx.request_repaint();
        }
        let done = self.shot.lock().unwrap().take();
        if let Some((path, img)) = done {
            self.status = match save_png(&path, &img) {
                Ok(()) => format!("Saved snapshot to {path}"),
                Err(e) => format!("Snapshot failed: {e}"),
            };
            if self.cli.screenshot.is_some() {
                let _ = std::fs::write(format!("{path}.txt"), format!("{:.2} ms/frame physics+draw, substeps {}
", self.frame_ms, self.substeps));
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

impl eframe::App for AetherLab {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_snapshots(ui.ctx());
        // Keyboard shortcuts
        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) && self.page == Page::Lab {
                self.running = !self.running;
            }
        });

        egui::Panel::top("top").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("◎ AETHER LABS").strong().size(18.0).color(Color32::from_rgb(120, 200, 255)));
                ui.separator();
                ui.selectable_value(&mut self.page, Page::Lab, "🧪 Laboratory");
                ui.selectable_value(&mut self.page, Page::Scorecard, "📋 Hypothesis scorecard");
                ui.selectable_value(&mut self.page, Page::Library, "📚 Research library");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{:.1} ms/frame", self.frame_ms)).color(Color32::GRAY));
                    if ui.button("📷 Snapshot").on_hover_text("Save a PNG of the window to the current folder").clicked() {
                        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                        let path = format!("aether-snapshot-{ts}.png");
                        let ctx = ui.ctx().clone();
                        self.request_snapshot(&ctx, path);
                    }
                    if !self.status.is_empty() {
                        ui.label(RichText::new(&self.status).color(Color32::from_rgb(140, 200, 140)));
                    }
                });
            });
        });

        match self.page {
            Page::Scorecard => {
                egui::CentralPanel::default().show(ui, |ui| self.scorecard.ui(ui));
                return;
            }
            Page::Library => {
                egui::CentralPanel::default().show(ui, |ui| self.library.ui(ui));
                return;
            }
            Page::Lab => {}
        }

        // Left: experiment list
        egui::Panel::left("experiments").default_size(250.0).show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut last_cat = "";
                for (i, e) in self.experiments.iter().enumerate() {
                    if e.category() != last_cat {
                        last_cat = e.category();
                        ui.add_space(6.0);
                        ui.label(RichText::new(last_cat.to_uppercase()).small().color(Color32::from_rgb(140, 150, 180)));
                    }
                    let r = ui.selectable_label(self.selected == i, RichText::new(e.title()).strong());
                    if r.clicked() {
                        self.selected = i;
                    }
                    r.on_hover_text(e.tagline());
                }
            });
        });

        let exp = &mut self.experiments[self.selected];

        // Right: controls, diagnostics, checks
        egui::Panel::right("inspector").default_size(360.0).show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(exp.title());
                ui.label(RichText::new(exp.tagline()).italics().color(Color32::from_rgb(170, 180, 210)));
                ui.horizontal(|ui| {
                    if ui.button(if self.running { "⏸ Pause" } else { "▶ Run" }).clicked() {
                        self.running = !self.running;
                    }
                    if ui.button("⏭ Step").clicked() {
                        exp.advance(1);
                    }
                    if ui.button("⟲ Reset").clicked() {
                        exp.reset();
                    }
                });
                ui.add(Slider::new(&mut self.substeps, 1..=20).text("steps / frame"));
                ui.separator();
                egui::CollapsingHeader::new("Parameters").default_open(true).show(ui, |ui| {
                    if exp.controls(ui) {
                        exp.reset();
                    }
                });
                egui::CollapsingHeader::new("Live checks vs theory").default_open(true).show(ui, |ui| {
                    // Checks can be expensive: refresh at most twice per second.
                    let stale = self.checks_for != self.selected
                        || self.checks_at.is_none_or(|t| t.elapsed().as_millis() > 500);
                    if stale {
                        self.checks_cache = exp.checks();
                        self.checks_at = Some(Instant::now());
                        self.checks_for = self.selected;
                    }
                    let checks = &self.checks_cache;
                    if checks.is_empty() {
                        ui.label(RichText::new("No checks apply yet. Let it run, or press 'Analyze'.").color(Color32::GRAY));
                    } else {
                        checks_table(ui, "live", checks);
                    }
                });
                egui::CollapsingHeader::new("Diagnostics").default_open(true).show(ui, |ui| exp.diagnostics(ui));
            });
        });

        // Bottom: theory panel
        egui::Panel::bottom("theory").resizable(true).default_size(170.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Theory & what this tests");
                ui.checkbox(&mut self.show_theory, "show");
                ui.label(RichText::new(format!("t = {:.3}", exp.sim_time())).monospace().color(Color32::GRAY));
            });
            if self.show_theory {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(exp.theory());
                });
            }
        });

        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            let t0 = Instant::now();
            if self.running {
                exp.advance(self.substeps);
                // Keep the UI responsive: if physics takes too long, do less per frame.
                if t0.elapsed().as_secs_f64() > 0.03 && self.substeps > 1 {
                    self.substeps -= 1;
                }
            }
            exp.view(ui);
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            self.frame_ms = 0.9 * self.frame_ms + 0.1 * ms;
        });

        if self.running {
            ui.ctx().request_repaint();
        }
    }
}

fn main() -> eframe::Result {
    let cli = parse_cli();
    // Leave CPU cores free for the window and the rest of the system.
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let _ = rayon::ThreadPoolBuilder::new().num_threads((cores / 2).max(1)).build_global();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Aether Labs: vortex physics of a single medium")
            .with_inner_size([1500.0, 920.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native("Aether Labs", options, Box::new(move |cc| Ok(Box::new(AetherLab::new(cc, cli)))))
}
