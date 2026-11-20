//! Kapitel 1: Welle, Frequenz, Wellenlänge, Radarbänder, Polarisation.

use eframe::egui::{self, pos2, vec2, Align2, CornerRadius, FontId, Rect, Sense, Shape, Stroke};

use super::charts::Chart;
use super::{canvas, formula, merke, pal, presets, see_also, slider, sub, text, try_it, Chapter};
use crate::i18n::t;

const C: f64 = 299_792_458.0;

pub struct State {
    f: f64,
    n: f64,
    run: bool,
    phase: f64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            f: 9.41,
            n: 6.0,
            run: true,
            phase: 0.0,
        }
    }
}

pub fn band_of(f: f64) -> Option<&'static str> {
    if (2.9..=3.1).contains(&f) {
        Some("S")
    } else if (9.2..=9.5).contains(&f) {
        Some("X")
    } else {
        None
    }
}

impl State {
    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Ein Radar sendet kurze Pulse elektromagnetischer Wellen (Mikrowellen) aus und misst die Laufzeit der Echos. Frequenz f und Wellenlänge λ hängen über die Lichtgeschwindigkeit zusammen. Für die Schifffahrt sind zwei Bänder zugelassen: das X-Band um 9,4 GHz (λ ≈ 3 cm) und das S-Band um 3 GHz (λ ≈ 10 cm).",
                "A radar transmits short pulses of electromagnetic waves (microwaves) and measures the travel time of the echoes. Frequency f and wavelength λ are linked by the speed of light. Two bands are allowed for shipping: X-band around 9.4 GHz (λ ≈ 3 cm) and S-band around 3 GHz (λ ≈ 10 cm).",
            ),
        );
        formula(
            ui,
            t(
                "λ = c / f      c ≈ 300 000 km/s = 300 m/µs",
                "λ = c / f      c ≈ 300,000 km/s = 300 m/µs",
            ),
        );

        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.f,
                2.0..=10.0,
                t("Frequenz f", "Frequency f"),
                " GHz",
                0.01,
            );
            presets(
                ui,
                &mut self.f,
                &[("S-Band 3,05 GHz", 3.05), ("X-Band 9,41 GHz", 9.41)],
            );
        });
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.n,
                1.0..=20.0,
                t("Perioden", "Periods"),
                "",
                1.0,
            );
            ui.checkbox(&mut self.run, t("Welle laufen lassen", "Animate wave"));
        });
        let lambda_cm = C / (self.f * 1e9) * 100.0;
        let period_ps = 1000.0 / self.f;
        let band = band_of(self.f)
            .map(|b| format!("  ·  {}-Band", b))
            .unwrap_or_default();
        ui.label(
            egui::RichText::new(tf!(
                "λ = {:.2} cm     Periodendauer T = {:.0} ps{}",
                "λ = {:.2} cm     period T = {:.0} ps{}",
                lambda_cm,
                period_ps,
                band
            ))
            .monospace()
            .size(15.0)
            .strong(),
        );
        if self.run {
            self.phase += ui.input(|i| i.stable_dt).min(0.1) as f64 * 1.5;
            // ohne Neuzeichnen-Auftrag malt egui nur bei Eingaben (z. B. Mausbewegung)
            ui.ctx().request_repaint();
        }

        // Welle über der Zeit
        let (rect, _, painter) = canvas(ui, 230.0, Sense::hover());
        let tmax = self.n * period_ps / 1000.0; // ns
        let ch = Chart::new(rect, (0.0, tmax), (-1.3, 1.3));
        ch.frame(
            &painter,
            &p,
            &tf!(
                "Welle: f = {:.2} GHz, λ = {:.1} cm",
                "Wave: f = {:.2} GHz, λ = {:.1} cm",
                self.f,
                lambda_cm
            ),
            t("Zeit (ns)", "time (ns)"),
            "E",
        );
        let n = (400.0 * self.n) as usize;
        let ph = self.phase;
        let f = self.f;
        ch.line(
            &painter,
            (0..=n).map(|i| {
                let tt = tmax * i as f64 / n as f64;
                (tt, (std::f64::consts::TAU * f * tt - ph).sin())
            }),
            Stroke::new(2.0_f32, p.accent),
        );

        // Frequenzachse mit Bändern und Wellenlängen im Maßstab
        let (rect, _, painter) = canvas(ui, 150.0, Sense::hover());
        draw_bands(&painter, rect, &p, self.f);

        sub(ui, t("Polarisation", "Polarisation"));
        text(
            ui,
            t(
                "Die Welle schwingt in einer Ebene. Schiffsradare sind meist horizontal polarisiert – das liefert weniger Seegangsclutter als vertikale Polarisation.",
                "The wave oscillates in one plane. Marine radars are usually horizontally polarised – this gives less sea clutter than vertical polarisation.",
            ),
        );
        let (rect, _, painter) = canvas(ui, 220.0, Sense::hover());
        draw_polarisation(&painter, rect, &p, self.n.min(8.0), self.phase);

        try_it(
            ui,
            t(
                "Schieben Sie die Frequenz vom S- ins X-Band: Die Welle wird gut dreimal kürzer. Genau dieser Unterschied erklärt später die schmalere Keule (Kapitel 3) und den stärkeren Regenclutter (Kapitel 2).",
                "Move the frequency from S- to X-band: the wave becomes about three times shorter. This difference later explains the narrower beam (chapter 3) and the stronger rain clutter (chapter 2).",
            ),
        );
        merke(
            ui,
            &[
                t("λ = c / f; mit c = 300 m/µs: 3 GHz ⇒ 10 cm, 10 GHz ⇒ 3 cm.", "λ = c / f; with c = 300 m/µs: 3 GHz ⇒ 10 cm, 10 GHz ⇒ 3 cm."),
                t("X-Band 9,2–9,5 GHz (λ ≈ 3,2 cm), S-Band 2,9–3,1 GHz (λ ≈ 10 cm).", "X-band 9.2–9.5 GHz (λ ≈ 3.2 cm), S-band 2.9–3.1 GHz (λ ≈ 10 cm)."),
                t("Kürzere Welle: schmalere Keule bei gleicher Antenne, aber stärkere Streuung an Regen und Seegang.", "Shorter wave: narrower beam with the same antenna, but more scattering from rain and sea."),
                t("Nur das X-Band löst SART-Transponder aus.", "Only X-band triggers SART transponders."),
            ],
        );
        see_also(
            ui,
            nav,
            &[Chapter::Bands, Chapter::Antenna, Chapter::RangeRes],
        );
    }
}

fn draw_bands(p: &egui::Painter, rect: Rect, pal: &super::Pal, f: f64) {
    p.rect_filled(rect, CornerRadius::same(6), pal.panel);
    let left = rect.left() + 40.0;
    let right = rect.right() - 20.0;
    let y = rect.top() + 52.0;
    let fx = |g: f64| left + ((g - 2.0) / 8.0) as f32 * (right - left);
    p.text(
        rect.left_top() + vec2(10.0, 6.0),
        Align2::LEFT_TOP,
        t(
            "Frequenzachse und Radarbänder",
            "Frequency axis and radar bands",
        ),
        FontId::proportional(13.0),
        pal.ink,
    );
    p.line_segment(
        [pos2(left, y), pos2(right, y)],
        Stroke::new(1.5_f32, pal.faint),
    );
    for g in 2..=10 {
        let x = fx(g as f64);
        p.line_segment(
            [pos2(x, y - 4.0), pos2(x, y + 4.0)],
            Stroke::new(1.0_f32, pal.faint),
        );
        p.text(
            pos2(x, y + 6.0),
            Align2::CENTER_TOP,
            format!("{g} GHz"),
            FontId::proportional(11.0),
            pal.faint,
        );
    }
    for (a, b, name, col) in [(2.9, 3.1, "S", pal.s), (9.2, 9.5, "X", pal.x)] {
        let r = Rect::from_min_max(pos2(fx(a), y - 12.0), pos2(fx(b).max(fx(a) + 4.0), y + 2.0));
        p.rect_filled(r, CornerRadius::same(2), col);
        p.text(
            pos2(r.center().x, y - 14.0),
            Align2::CENTER_BOTTOM,
            format!("{name}-Band"),
            FontId::proportional(12.0),
            col,
        );
    }
    let xf = fx(f);
    p.add(Shape::convex_polygon(
        vec![
            pos2(xf, y - 2.0),
            pos2(xf - 6.0, y - 14.0),
            pos2(xf + 6.0, y - 14.0),
        ],
        pal.echo,
        Stroke::NONE,
    ));

    // Wellenlängen im gleichen Maßstab
    let scale = (right - left) / 12.0; // px je cm
    let rows = [
        (
            t("S-Band 3,05 GHz", "S-band 3.05 GHz"),
            C / 3.05e9 * 100.0,
            pal.s,
        ),
        (
            t("X-Band 9,41 GHz", "X-band 9.41 GHz"),
            C / 9.41e9 * 100.0,
            pal.x,
        ),
        (
            t("eingestellt", "selected"),
            C / (f * 1e9) * 100.0,
            pal.echo,
        ),
    ];
    for (i, (name, l, col)) in rows.iter().enumerate() {
        let yy = y + 34.0 + i as f32 * 22.0;
        let w = (*l as f32 * scale).min(right - left);
        p.rect_filled(
            Rect::from_min_size(pos2(left, yy), vec2(w, 10.0)),
            CornerRadius::same(2),
            *col,
        );
        p.text(
            pos2(left + w + 8.0, yy + 5.0),
            Align2::LEFT_CENTER,
            format!("λ = {:.1} cm  ({})", l, name),
            FontId::proportional(11.0),
            pal.ink,
        );
    }
}

fn draw_polarisation(p: &egui::Painter, rect: Rect, pal: &super::Pal, n: f64, phase: f64) {
    p.rect_filled(rect, CornerRadius::same(6), pal.panel);
    let x0 = rect.left() + 60.0;
    let x1 = rect.right() - 60.0;
    let cy = rect.center().y + 10.0;
    let amp = rect.height() * 0.28;
    let depth = vec2(0.3, -0.5) * amp; // Projektion der z-Achse (schräg nach hinten)
                                       // Achsen
    p.line_segment(
        [pos2(x0, cy), pos2(x1, cy)],
        Stroke::new(1.0_f32, pal.faint),
    );
    p.line_segment(
        [pos2(x0, cy - amp * 1.2), pos2(x0, cy + amp * 1.2)],
        Stroke::new(1.0_f32, pal.faint),
    );
    p.line_segment(
        [pos2(x0, cy) - depth * 1.2, pos2(x0, cy) + depth * 1.2],
        Stroke::new(1.0_f32, pal.faint),
    );
    let steps = 600;
    let mut v = Vec::with_capacity(steps + 1);
    let mut h = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let u = i as f64 / steps as f64;
        let x = x0 + (x1 - x0) * u as f32;
        let s = (std::f64::consts::TAU * n * u - phase).sin() as f32;
        v.push(pos2(x, cy - s * amp));
        h.push(pos2(x, cy) + depth * s);
    }
    p.add(Shape::line(v, Stroke::new(1.8_f32, pal.s)));
    p.add(Shape::line(h, Stroke::new(2.2_f32, pal.x)));
    p.text(
        rect.left_top() + vec2(10.0, 6.0),
        Align2::LEFT_TOP,
        t("Polarisationsebenen", "Planes of polarisation"),
        FontId::proportional(13.0),
        pal.ink,
    );
    p.text(
        pos2(x1, cy - amp * 1.05),
        Align2::RIGHT_BOTTOM,
        t("vertikal polarisiert", "vertically polarised"),
        FontId::proportional(12.0),
        pal.s,
    );
    p.text(
        pos2(x1, cy + amp * 0.9),
        Align2::RIGHT_TOP,
        t(
            "horizontal polarisiert (Schiffsradar)",
            "horizontally polarised (marine radar)",
        ),
        FontId::proportional(12.0),
        pal.x,
    );
}
