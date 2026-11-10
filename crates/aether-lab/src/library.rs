//! In-app research library: renders the research briefs bundled at compile time.

use egui::{Color32, RichText, Ui};

const DOCS: [(&str, &str); 3] = [
    ("Physics & history", include_str!("../../../docs/RESEARCH-physics.md")),
    ("Numerical methods", include_str!("../../../docs/RESEARCH-methods.md")),
    ("Roadmap", include_str!("../../../docs/ROADMAP.md")),
];

#[derive(Default)]
pub struct Library {
    tab: usize,
    filter: String,
}

impl Library {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for (i, (name, _)) in DOCS.iter().enumerate() {
                ui.selectable_value(&mut self.tab, i, *name);
            }
            ui.separator();
            ui.label("🔎");
            ui.text_edit_singleline(&mut self.filter);
        });
        ui.separator();
        let text = DOCS[self.tab].1;
        let f = self.filter.to_lowercase();
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_max_width(980.0);
            for para in text.split("\n\n") {
                if !f.is_empty() && !para.to_lowercase().contains(&f) {
                    continue;
                }
                render_block(ui, para);
            }
        });
    }
}

fn render_block(ui: &mut Ui, block: &str) {
    for line in block.lines() {
        let t = line.trim_end();
        if let Some(h) = t.strip_prefix("### ") {
            ui.add_space(4.0);
            ui.label(RichText::new(h).size(16.0).strong().color(Color32::from_rgb(170, 200, 255)));
        } else if let Some(h) = t.strip_prefix("## ") {
            ui.add_space(8.0);
            ui.label(RichText::new(h).size(19.0).strong().color(Color32::from_rgb(120, 200, 255)));
        } else if let Some(h) = t.strip_prefix("# ") {
            ui.label(RichText::new(h).size(24.0).strong());
        } else if t.starts_with("```") {
            continue;
        } else if t.starts_with('|') {
            ui.monospace(t);
        } else if t.starts_with("    ") || t.starts_with('\t') {
            ui.monospace(t.trim());
        } else if !t.is_empty() {
            ui.label(t.replace("**", ""));
        }
    }
    ui.add_space(6.0);
}
