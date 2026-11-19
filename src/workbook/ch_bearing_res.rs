//! Kapitel 5: Azimutale Auflösung – Keulenbreite, Echoverbreiterung, Entfernung.

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Rect, RichText, Sense, Stroke,
};

use super::charts::Chart;
use super::{formula, merke, pal, presets, see_also, slider, sub, text, try_it, Chapter, Pal};
use crate::i18n::t;

pub struct State {
    x_band: bool,
    d: f64,
    r_nm: f64,
    sep: f64,
    sweep: f64,
    run: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            x_band: true,
            d: 2.44,
            r_nm: 6.0,
            sep: 300.0,
            sweep: 0.0,
            run: true,
        }
    }
}

impl State {
    fn lambda(&self) -> f64 {
        if self.x_band {
            0.0319
        } else {
            0.0983
        }
    }

    /// Keulenbreite in Grad (Faustformel 70·λ/D)
    fn theta_deg(&self) -> f64 {
        70.0 * self.lambda() / self.d
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Die Keule überstreicht ein Ziel nicht in einem Augenblick: Solange irgendein Teil der Keule das Ziel trifft, kommt ein Echo zurück – und das Radar malt es auf die Peilung, in die die Keulenmitte gerade zeigt. Jedes Echo wird deshalb um die Keulenbreite verbreitert. Zwei Ziele in gleicher Entfernung erscheinen nur getrennt, wenn ihr Abstand größer ist als die Keulenbreite an dieser Stelle.",
                "The beam does not sweep across a target in an instant: as long as any part of the beam hits the target, an echo returns – and the radar paints it on the bearing the beam centre is pointing at. Each echo is therefore widened by the beamwidth. Two targets at the same range only appear separate if their distance is greater than the beamwidth at that range.",
            ),
        );
        formula(
            ui,
            t(
                "Querauflösung  ΔA ≈ R · θ   (θ im Bogenmaß)      θ ≈ 70 · λ / D  (°)",
                "cross-range resolution  ΔA ≈ R · θ   (θ in radians)      θ ≈ 70 · λ / D  (°)",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.x_band, true, "X-Band");
            ui.selectable_value(&mut self.x_band, false, "S-Band");
            slider(
                ui,
                &mut self.d,
                1.0..=4.5,
                t("Antennenlänge", "antenna length"),
                " m",
                0.01,
            );
            presets(
                ui,
                &mut self.d,
                &[("4 ft", 1.22), ("8 ft", 2.44), ("12 ft", 3.66)],
            );
        });
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.r_nm,
                0.25..=24.0,
                t("Entfernung der Ziele", "range of targets"),
                t(" sm", " nm"),
                0.05,
            );
            slider(
                ui,
                &mut self.sep,
                10.0..=1500.0,
                t("Querabstand der Ziele", "lateral distance"),
                " m",
                5.0,
            );
            if ui
                .button(t("Beispiel: 300 m in 10 sm", "Example: 300 m at 10 nm"))
                .clicked()
            {
                self.sep = 300.0;
                self.r_nm = 10.0;
            }
            ui.checkbox(&mut self.run, t("Keule drehen", "rotate beam"));
        });

        let th = self.theta_deg();
        let r_m = self.r_nm * 1852.0;
        let da = r_m * th.to_radians();
        let need = (self.sep / r_m).to_degrees();
        let resolved = self.sep > da;
        ui.label(
            RichText::new(tf!(
                "Keulenbreite θ = {:.2}°   ·   Querauflösung bei {:.2} sm: ΔA = {:.0} m   ·   nötige Keule für {:.0} m: ≤ {:.2}°",
                "beamwidth θ = {:.2}°   ·   cross-range resolution at {:.2} nm: ΔA = {:.0} m   ·   beam needed for {:.0} m: ≤ {:.2}°",
                th,
                self.r_nm,
                da,
                self.sep,
                need
            ))
            .monospace()
            .strong(),
        );
        ui.label(
            RichText::new(if resolved {
                t("✔ Zwei getrennte Echos", "✔ Two separate echoes")
            } else {
                t(
                    "✘ Die Echos verschmelzen zu einem breiten Echo",
                    "✘ The echoes merge into one wide echo",
                )
            })
            .strong()
            .size(15.0)
            .color(if resolved { p.ok } else { p.warn }),
        );

        if self.run {
            self.sweep = (self.sweep + ui.input(|i| i.stable_dt).min(0.1) as f64 / 3.5) % 1.0;
            ui.ctx().request_repaint();
        }
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 270.0), Sense::hover());
        draw_sector(&ui.painter_at(rect), rect, &p, self.sep, da, self.sweep);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 230.0), Sense::hover());
        let painter = ui.painter_at(rect);
        let ymax =
            (24.0 * 1852.0 * 2.6f64.to_radians()).max(24.0 * 1852.0 * th.to_radians()) * 1.05;
        let ch = Chart::new(rect, (0.0, 24.0), (0.0, ymax));
        ch.frame(
            &painter,
            &p,
            t(
                "Querauflösung ΔA über der Entfernung",
                "Cross-range resolution ΔA over range",
            ),
            t("Entfernung (sm)", "range (nm)"),
            "ΔA (m)",
        );
        let thr = th.to_radians();
        ch.line(
            &painter,
            [(0.0, 0.0), (24.0, 24.0 * 1852.0 * thr)],
            Stroke::new(2.0_f32, p.accent),
        );
        ch.line(
            &painter,
            [(0.0, 0.0), (24.0, 24.0 * 1852.0 * 2.5f64.to_radians())],
            Stroke::new(1.2_f32, p.warn),
        );
        ch.dot(&painter, self.r_nm, da, 6.0, p.echo);
        ch.dot(&painter, self.r_nm, self.sep, 5.0, p.ok);
        painter.text(
            rect.right_top() + vec2(-16.0, 8.0),
            Align2::RIGHT_TOP,
            &tf!(
                "blau: θ = {:.2}°   rot: 2,5° (IMO)   grün: Zielabstand",
                "blue: θ = {:.2}°   red: 2.5° (IMO)   green: target distance",
                th
            ),
            FontId::proportional(11.0),
            p.faint,
        );

        try_it(
            ui,
            t(
                "Klicken Sie „Beispiel: 300 m in 10 sm“: Bei 10 sm sind 300 m nur 0,93° – mit 8-ft-X-Band-Antenne (0,91°) gerade noch getrennt, mit einer 4-ft-Antenne nicht mehr. Fahren Sie die Ziele näher heran: In Metern wird die Keule schmaler, die Echos trennen sich.",
                "Click “Example: 300 m at 10 nm”: at 10 nm, 300 m are only 0.93° – just resolved with an 8 ft X-band antenna (0.91°), no longer with a 4 ft antenna. Bring the targets closer: in metres the beam gets narrower and the echoes separate.",
            ),
        );
        sub(ui, t("Was hilft?", "What helps?"));
        text(
            ui,
            t(
                "Längere Antenne oder kürzere Wellenlänge (X-Band) machen die Keule schmaler. Weniger Gain verkleinert die Echos etwas, weil nur noch der stärkere Mittelteil der Keule über der Schwelle liegt. Ansonsten: kleinerer Bereich wählen und ggf. warten, bis die Ziele näher sind.",
                "A longer antenna or a shorter wavelength (X-band) narrow the beam. Less gain shrinks echoes slightly, because only the stronger central part of the beam stays above the threshold. Otherwise: choose a smaller range scale and wait until the targets are closer.",
            ),
        );
        merke(
            ui,
            &[
                t(
                    "Echos werden um die Keulenbreite verbreitert (Echoverbreiterung).",
                    "Echoes are widened by the beamwidth (echo widening).",
                ),
                t(
                    "ΔA = R · θ: In Metern wächst die Querauflösung mit der Entfernung.",
                    "ΔA = R · θ: in metres the cross-range resolution grows with range.",
                ),
                t(
                    "Nötige Keulenbreite für Abstand s in Entfernung R: θ ≤ s / R.",
                    "Beamwidth needed for distance s at range R: θ ≤ s / R.",
                ),
                t(
                    "Gleiche Keulenbreite im X-Band mit ≈ 0,3-facher Antennenlänge des S-Bands.",
                    "Same beamwidth in X-band with ≈ 0.3 times the S-band antenna length.",
                ),
            ],
        );
        see_also(
            ui,
            nav,
            &[
                Chapter::Antenna,
                Chapter::RangeRes,
                Chapter::Bands,
                Chapter::Range,
            ],
        );
    }
}

/// Ausschnitt um die beiden Ziele: Keule streicht von links nach rechts.
fn draw_sector(p: &egui::Painter, r: Rect, pal: &Pal, sep: f64, da: f64, sweep: f64) {
    p.rect_filled(r, CornerRadius::same(6), pal.panel);
    p.text(
        r.left_top() + vec2(10.0, 6.0),
        Align2::LEFT_TOP,
        t(
            "PPI-Ausschnitt um die Ziele (quer zur Peilung)",
            "PPI cut-out around the targets (across the bearing)",
        ),
        FontId::proportional(13.0),
        pal.ink,
    );
    let view = Rect::from_min_max(r.min + vec2(16.0, 30.0), r.max - vec2(16.0, 26.0));
    p.rect_filled(view, CornerRadius::same(4), Color32::from_rgb(26, 62, 128));
    let half_w = (sep * 0.5 + da) * 1.35 + 20.0; // m
    let mpx = view.width() as f64 / (2.0 * half_w);
    let cx = view.center().x;
    let cy = view.center().y;
    let x_of = |m: f64| cx + (m * mpx) as f32;
    let xs = [-sep / 2.0, sep / 2.0];
    // Keulenmitte läuft über den Ausschnitt
    let beam_c = -half_w + 2.0 * half_w * sweep;
    // gemalte Echos bis zur aktuellen Keulenmitte
    let echo_h = 26.0;
    for &x in &xs {
        // Nachleuchten des vorigen Durchlaufs
        let full = Rect::from_min_max(
            pos2(x_of(x - da / 2.0), cy - echo_h / 2.0),
            pos2(x_of(x + da / 2.0), cy + echo_h / 2.0),
        );
        p.rect_filled(
            full,
            CornerRadius::same(6),
            Color32::from_rgba_unmultiplied(255, 230, 110, 95),
        );
        let a = x - da / 2.0;
        let b = (x + da / 2.0).min(beam_c);
        if b > a {
            let rr = Rect::from_min_max(
                pos2(x_of(a), cy - echo_h / 2.0),
                pos2(x_of(b), cy + echo_h / 2.0),
            );
            p.rect_filled(
                rr,
                CornerRadius::same(6),
                Color32::from_rgba_unmultiplied(255, 230, 110, 210),
            );
        }
    }
    // Keule
    let bl = x_of(beam_c - da / 2.0);
    let br = x_of(beam_c + da / 2.0);
    p.rect_filled(
        Rect::from_min_max(
            pos2(bl.max(view.left()), view.top()),
            pos2(br.min(view.right()), view.bottom()),
        ),
        CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(120, 200, 255, 40),
    );
    p.line_segment(
        [
            pos2(x_of(beam_c), view.top()),
            pos2(x_of(beam_c), view.bottom()),
        ],
        Stroke::new(1.5_f32, Color32::from_rgb(160, 220, 255)),
    );
    // wahre Ziele
    for &x in &xs {
        p.circle_filled(pos2(x_of(x), cy), 4.0, Color32::WHITE);
    }
    // Maßstab
    let bar_m = nice_bar(half_w);
    let y = view.bottom() - 12.0;
    p.line_segment(
        [
            pos2(view.left() + 12.0, y),
            pos2(view.left() + 12.0 + (bar_m * mpx) as f32, y),
        ],
        Stroke::new(2.0_f32, Color32::WHITE),
    );
    p.text(
        pos2(view.left() + 14.0, y - 4.0),
        Align2::LEFT_BOTTOM,
        format!("{bar_m:.0} m"),
        FontId::proportional(11.0),
        Color32::WHITE,
    );
    p.text(
        pos2(cx, r.bottom() - 4.0),
        Align2::CENTER_BOTTOM,
        t(
            "weiß: wahre Ziele · gelb: Echos · hellblau: Keule",
            "white: true targets · yellow: echoes · light blue: beam",
        ),
        FontId::proportional(11.0),
        pal.faint,
    );
}

fn nice_bar(half_w: f64) -> f64 {
    let raw = half_w * 0.4;
    let mag = 10f64.powf(raw.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .iter()
        .map(|m| m * mag)
        .filter(|v| *v <= raw)
        .last()
        .unwrap_or(mag)
}
