//! Kleine Diagramm-Hilfe (Achsen, Gitter, Linien) für die Arbeitsbuch-Kapitel.

use eframe::egui::{self, pos2, vec2, Align2, CornerRadius, FontId, Pos2, Rect, Shape, Stroke};

use super::Pal;

pub struct Chart {
    pub area: Rect,
    pub plot: Rect,
    pub x: (f64, f64),
    pub y: (f64, f64),
    pub logx: bool,
}

impl Chart {
    pub fn new(area: Rect, x: (f64, f64), y: (f64, f64)) -> Self {
        let plot = Rect::from_min_max(area.min + vec2(56.0, 28.0), area.max - vec2(14.0, 36.0));
        Self {
            area,
            plot,
            x,
            y,
            logx: false,
        }
    }

    pub fn log_x(mut self) -> Self {
        self.logx = true;
        self
    }

    fn fx(&self, v: f64) -> f64 {
        if self.logx {
            v.max(1e-12).log10()
        } else {
            v
        }
    }

    pub fn sx(&self, v: f64) -> f32 {
        let (a, b) = (self.fx(self.x.0), self.fx(self.x.1));
        self.plot.left() + ((self.fx(v) - a) / (b - a)) as f32 * self.plot.width()
    }

    pub fn sy(&self, v: f64) -> f32 {
        let (a, b) = self.y;
        self.plot.bottom() - ((v - a) / (b - a)) as f32 * self.plot.height()
    }

    pub fn pt(&self, x: f64, y: f64) -> Pos2 {
        pos2(self.sx(x), self.sy(y))
    }

    /// Hintergrund, Gitter, Achsenbeschriftung.
    pub fn frame(&self, p: &egui::Painter, pal: &Pal, title: &str, xlabel: &str, ylabel: &str) {
        p.rect_filled(self.area, CornerRadius::same(6), pal.panel);
        let font = FontId::proportional(11.0);
        for xv in ticks(self.x, self.logx) {
            let sx = self.sx(xv);
            p.line_segment(
                [pos2(sx, self.plot.top()), pos2(sx, self.plot.bottom())],
                Stroke::new(1.0_f32, pal.grid),
            );
            p.text(
                pos2(sx, self.plot.bottom() + 3.0),
                Align2::CENTER_TOP,
                fmt_tick(xv),
                font.clone(),
                pal.faint,
            );
        }
        for yv in ticks(self.y, false) {
            let sy = self.sy(yv);
            p.line_segment(
                [pos2(self.plot.left(), sy), pos2(self.plot.right(), sy)],
                Stroke::new(1.0_f32, pal.grid),
            );
            p.text(
                pos2(self.plot.left() - 4.0, sy),
                Align2::RIGHT_CENTER,
                fmt_tick(yv),
                font.clone(),
                pal.faint,
            );
        }
        p.rect_stroke(
            self.plot,
            CornerRadius::ZERO,
            Stroke::new(1.0_f32, pal.faint),
            egui::StrokeKind::Inside,
        );
        p.text(
            self.area.left_top() + vec2(10.0, 6.0),
            Align2::LEFT_TOP,
            title,
            FontId::proportional(13.0),
            pal.ink,
        );
        p.text(
            pos2(self.plot.center().x, self.area.bottom() - 4.0),
            Align2::CENTER_BOTTOM,
            xlabel,
            font.clone(),
            pal.faint,
        );
        p.text(
            pos2(self.plot.left() + 4.0, self.plot.top() + 2.0),
            Align2::LEFT_TOP,
            ylabel,
            font,
            pal.faint,
        );
    }

    /// Polylinie (auf die Plotfläche begrenzt).
    pub fn line(&self, p: &egui::Painter, pts: impl IntoIterator<Item = (f64, f64)>, s: Stroke) {
        let v: Vec<Pos2> = pts.into_iter().map(|(x, y)| self.pt(x, y)).collect();
        if v.len() >= 2 {
            p.with_clip_rect(self.plot.expand(1.0))
                .add(Shape::line(v, s));
        }
    }

    pub fn hline(&self, p: &egui::Painter, y: f64, s: Stroke, label: &str) {
        let sy = self.sy(y);
        if sy < self.plot.top() || sy > self.plot.bottom() {
            return;
        }
        p.extend(Shape::dashed_line(
            &[pos2(self.plot.left(), sy), pos2(self.plot.right(), sy)],
            s,
            6.0,
            4.0,
        ));
        if !label.is_empty() {
            p.text(
                pos2(self.plot.right() - 4.0, sy - 2.0),
                Align2::RIGHT_BOTTOM,
                label,
                FontId::proportional(11.0),
                s.color,
            );
        }
    }

    pub fn vline(&self, p: &egui::Painter, x: f64, s: Stroke, label: &str) {
        let sx = self.sx(x);
        if sx < self.plot.left() - 0.5 || sx > self.plot.right() + 0.5 {
            return;
        }
        p.extend(Shape::dashed_line(
            &[pos2(sx, self.plot.top()), pos2(sx, self.plot.bottom())],
            s,
            6.0,
            4.0,
        ));
        if !label.is_empty() {
            p.text(
                pos2(sx + 4.0, self.plot.top() + 16.0),
                Align2::LEFT_TOP,
                label,
                FontId::proportional(11.0),
                s.color,
            );
        }
    }

    pub fn dot(&self, p: &egui::Painter, x: f64, y: f64, r: f32, c: egui::Color32) {
        let q = self.pt(x, y);
        if self.plot.expand(r).contains(q) {
            p.circle_filled(q, r, c);
        }
    }
}

/// Teilstriche: linear 1-2-5-Raster, logarithmisch Dekaden mit 2 und 5.
pub fn ticks(r: (f64, f64), log: bool) -> Vec<f64> {
    let (a, b) = (r.0.min(r.1), r.0.max(r.1));
    let mut out = Vec::new();
    if log {
        let mut e = a.max(1e-9).log10().floor() as i32;
        while 10f64.powi(e) <= b * 1.0001 {
            for m in [1.0, 2.0, 5.0] {
                let v = m * 10f64.powi(e);
                if v >= a * 0.9999 && v <= b * 1.0001 {
                    out.push(v);
                }
            }
            e += 1;
        }
        return out;
    }
    let span = (b - a).max(1e-12);
    let raw = span / 6.0;
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0]
        .iter()
        .map(|m| m * mag)
        .find(|s| *s >= raw)
        .unwrap_or(10.0 * mag);
    let mut v = (a / step).ceil() * step;
    while v <= b + step * 1e-6 {
        out.push(if v.abs() < step * 1e-9 { 0.0 } else { v });
        v += step;
    }
    out
}

pub fn fmt_tick(v: f64) -> String {
    let a = v.abs();
    if a >= 100.0 || a == 0.0 {
        format!("{:.0}", v)
    } else if a >= 1.0 {
        let s = format!("{:.1}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        let s = format!("{:.3}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
