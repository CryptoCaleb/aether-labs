//! Lightweight 3D orbit camera and 2D pan/zoom view drawn with egui's painter.

use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use glam::{DVec2, DVec3};

pub const BG: Color32 = Color32::from_rgb(10, 12, 20);

#[derive(Clone)]
pub struct Camera {
    pub yaw: f64,
    pub pitch: f64,
    pub dist: f64,
    pub target: DVec3,
    pub fov: f64,
    rect: Rect,
}

impl Camera {
    pub fn new(dist: f64) -> Self {
        Self {
            yaw: 0.8,
            pitch: 0.35,
            dist,
            target: DVec3::ZERO,
            fov: 0.9,
            rect: Rect::NOTHING,
        }
    }

    fn eye(&self) -> DVec3 {
        let (cp, sp) = (self.pitch.cos(), self.pitch.sin());
        self.target + self.dist * DVec3::new(cp * self.yaw.cos(), cp * self.yaw.sin(), sp)
    }

    fn basis(&self) -> (DVec3, DVec3, DVec3) {
        let f = (self.target - self.eye()).normalize();
        let r = f.cross(DVec3::Z).normalize_or(DVec3::X);
        let u = r.cross(f);
        (r, u, f)
    }

    /// Project a world point; returns screen position and depth.
    pub fn project(&self, p: DVec3) -> Option<(Pos2, f32)> {
        let (r, u, f) = self.basis();
        let d = p - self.eye();
        let z = d.dot(f);
        if z < 1e-3 {
            return None;
        }
        let s = (self.rect.height() as f64 * 0.5) / (self.fov * 0.5).tan();
        let c = self.rect.center();
        Some((
            Pos2::new(c.x + (d.dot(r) / z * s) as f32, c.y - (d.dot(u) / z * s) as f32),
            z as f32,
        ))
    }

    /// Allocate the view and handle orbit (left drag), pan (right drag), zoom (wheel).
    pub fn interact(&mut self, ui: &mut Ui) -> (Response, egui::Painter) {
        let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        self.rect = resp.rect;
        painter.rect_filled(resp.rect, 0.0, BG);
        if resp.dragged_by(egui::PointerButton::Primary) {
            let d = resp.drag_delta();
            self.yaw -= d.x as f64 * 0.008;
            self.pitch = (self.pitch + d.y as f64 * 0.008).clamp(-1.5, 1.5);
        }
        if resp.dragged_by(egui::PointerButton::Secondary) {
            let d = resp.drag_delta();
            let (r, u, _) = self.basis();
            let k = self.dist * 0.0015;
            self.target += (-r * d.x as f64 + u * d.y as f64) * k;
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.dist = (self.dist * (-scroll as f64 * 0.002).exp()).clamp(0.2, 500.0);
            }
        }
        (resp, painter)
    }

    pub fn line(&self, painter: &egui::Painter, a: DVec3, b: DVec3, width: f32, color: Color32) {
        if let (Some((pa, za)), Some((pb, zb))) = (self.project(a), self.project(b)) {
            let fade = depth_fade(0.5 * (za + zb), self.dist as f32);
            painter.line_segment([pa, pb], Stroke::new(width, scale_color(color, fade)));
        }
    }

    pub fn polyline(&self, painter: &egui::Painter, pts: &[DVec3], closed: bool, width: f32, color: Color32) {
        let n = pts.len();
        if n < 2 {
            return;
        }
        let segs = if closed { n } else { n - 1 };
        for i in 0..segs {
            self.line(painter, pts[i], pts[(i + 1) % n], width, color);
        }
    }

    pub fn grid(&self, painter: &egui::Painter, half: f64, z: f64, step: f64) {
        let c = Color32::from_rgb(40, 46, 66);
        let n = (half / step).round() as i32;
        for i in -n..=n {
            let t = i as f64 * step;
            self.line(painter, DVec3::new(t, -half, z), DVec3::new(t, half, z), 1.0, c);
            self.line(painter, DVec3::new(-half, t, z), DVec3::new(half, t, z), 1.0, c);
        }
        // Axes
        self.line(painter, DVec3::new(0.0, 0.0, z), DVec3::new(half * 0.3, 0.0, z), 1.5, Color32::from_rgb(160, 60, 60));
        self.line(painter, DVec3::new(0.0, 0.0, z), DVec3::new(0.0, half * 0.3, z), 1.5, Color32::from_rgb(60, 160, 60));
        self.line(painter, DVec3::new(0.0, 0.0, z), DVec3::new(0.0, 0.0, z + half * 0.3), 1.5, Color32::from_rgb(60, 90, 200));
    }
}

fn depth_fade(z: f32, dist: f32) -> f32 {
    (1.4 - 0.5 * z / dist.max(1e-3)).clamp(0.35, 1.0)
}

pub fn scale_color(c: Color32, k: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c.r() as f32 * k) as u8,
        (c.g() as f32 * k) as u8,
        (c.b() as f32 * k) as u8,
        c.a(),
    )
}

/// 2D pan/zoom view mapping world (x, y) to screen.
#[derive(Clone)]
pub struct View2D {
    pub center: DVec2,
    /// World units visible across the shorter screen dimension.
    pub span: f64,
    rect: Rect,
}

impl View2D {
    pub fn new(span: f64) -> Self {
        Self {
            center: DVec2::ZERO,
            span,
            rect: Rect::NOTHING,
        }
    }

    fn scale(&self) -> f64 {
        self.rect.width().min(self.rect.height()) as f64 / self.span
    }

    pub fn to_screen(&self, p: DVec2) -> Pos2 {
        let s = self.scale();
        let c = self.rect.center();
        Pos2::new(c.x + ((p.x - self.center.x) * s) as f32, c.y - ((p.y - self.center.y) * s) as f32)
    }

    pub fn to_world(&self, p: Pos2) -> DVec2 {
        let s = self.scale();
        let c = self.rect.center();
        DVec2::new(self.center.x + (p.x - c.x) as f64 / s, self.center.y - (p.y - c.y) as f64 / s)
    }

    pub fn len(&self, l: f64) -> f32 {
        (l * self.scale()) as f32
    }

    /// Allocate; right-drag pans, wheel zooms. Left clicks are left to the caller.
    pub fn interact(&mut self, ui: &mut Ui) -> (Response, egui::Painter) {
        let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        self.rect = resp.rect;
        painter.rect_filled(resp.rect, 0.0, BG);
        if resp.dragged_by(egui::PointerButton::Secondary) || resp.dragged_by(egui::PointerButton::Middle) {
            let d: Vec2 = resp.drag_delta();
            let s = self.scale();
            self.center += DVec2::new(-d.x as f64 / s, d.y as f64 / s);
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.span = (self.span * (-scroll as f64 * 0.002).exp()).clamp(0.05, 1e4);
            }
        }
        (resp, painter)
    }

    pub fn circle(&self, painter: &egui::Painter, c: DVec2, r: f64, stroke: Stroke) {
        painter.circle_stroke(self.to_screen(c), self.len(r), stroke);
    }

    pub fn polyline(&self, painter: &egui::Painter, pts: &[DVec2], stroke: Stroke) {
        if pts.len() < 2 {
            return;
        }
        let v: Vec<Pos2> = pts.iter().map(|p| self.to_screen(*p)).collect();
        painter.add(egui::Shape::line(v, stroke));
    }
}

/// Blue → cyan → white → orange → red colormap for t ∈ [0, 1].
pub fn colormap(t: f64) -> Color32 {
    let t = t.clamp(0.0, 1.0) as f32;
    let stops: [(f32, [f32; 3]); 5] = [
        (0.0, [8.0, 10.0, 30.0]),
        (0.25, [30.0, 70.0, 170.0]),
        (0.5, [60.0, 190.0, 220.0]),
        (0.75, [240.0, 240.0, 210.0]),
        (1.0, [255.0, 140.0, 50.0]),
    ];
    for w in stops.windows(2) {
        let (t0, c0) = w[0];
        let (t1, c1) = w[1];
        if t <= t1 {
            let f = (t - t0) / (t1 - t0);
            return Color32::from_rgb(
                (c0[0] + f * (c1[0] - c0[0])) as u8,
                (c0[1] + f * (c1[1] - c0[1])) as u8,
                (c0[2] + f * (c1[2] - c0[2])) as u8,
            );
        }
    }
    Color32::WHITE
}

/// HSV-ish phase wheel colour for an angle in radians.
pub fn phase_color(a: f64, brightness: f64) -> Color32 {
    let h = (a / std::f64::consts::TAU).rem_euclid(1.0) * 6.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as i32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    let k = brightness.clamp(0.0, 1.0) * 255.0;
    Color32::from_rgb((r * k) as u8, (g * k) as u8, (b * k) as u8)
}
