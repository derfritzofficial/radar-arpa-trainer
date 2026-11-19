//! Kapitel 6: Reichweite – Radarhorizont und Radargleichung (Energiebilanz).

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Rect, RichText, Sense, Shape, Stroke,
};

use super::charts::Chart;
use super::{formula, merke, pal, presets, see_also, slider, sub, text, try_it, Chapter, Pal};
use crate::i18n::t;

const C: f64 = 299_792_458.0;

pub struct State {
    h_ant: f64,
    h_tgt: f64,
    k: f64,
    pt: f64,
    tau: f64,
    sig_log: f64,
    x_band: bool,
    d: f64,
    rmax: f64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            h_ant: 30.0,
            h_tgt: 50.0,
            k: 2.2,
            pt: 25.0,
            tau: 0.15,
            sig_log: 1.0,
            x_band: true,
            d: 2.44,
            rmax: 24.0,
        }
    }
}

fn prf_of(tau: f64) -> f64 {
    let tt = [0.07f64, 0.15, 1.2];
    let pp = [3000.0f64, 3000.0, 600.0];
    let tc = tau.clamp(tt[0], tt[2]);
    let (i0, i1) = if tc <= tt[1] { (0, 1) } else { (1, 2) };
    let u = (tc.ln() - tt[i0].ln()) / (tt[i1].ln() - tt[i0].ln());
    (pp[i0].ln() + u * (pp[i1].ln() - pp[i0].ln())).exp()
}

impl State {
    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        sub(ui, t("Radarhorizont", "Radar horizon"));
        text(
            ui,
            t(
                "Radarwellen laufen nahezu geradlinig, werden aber in der Atmosphäre leicht zur Erde hin gebeugt – etwas stärker als Licht. Ein Ziel ist nur sichtbar, wenn der Strahl von der Antenne über die Kimm hinweg seine Spitze erreicht. Die Reichweite ist die Summe aus dem Horizont der Antenne und dem des Ziels.",
                "Radar waves travel almost in straight lines but are bent slightly towards the earth by the atmosphere – a little more than light. A target is only visible if the ray from the antenna reaches its top across the horizon. The range is the sum of the antenna's horizon and the target's horizon.",
            ),
        );
        formula(ui, t("d [sm] = 2,2 · ( √h_Antenne + √h_Ziel )      h in m   (optisch 2,08, geometrisch 1,93)", "d [nm] = 2.2 · ( √h_antenna + √h_target )      h in m   (optical 2.08, geometric 1.93)"));
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.h_ant,
                2.0..=60.0,
                t("Antennenhöhe", "antenna height"),
                " m",
                0.5,
            );
            slider(
                ui,
                &mut self.h_tgt,
                0.0..=120.0,
                t("Zielhöhe", "target height"),
                " m",
                0.5,
            );
        });
        ui.horizontal_wrapped(|ui| {
            slider(ui, &mut self.k, 1.9..=2.4, t("Faktor", "factor"), "", 0.01);
            presets(
                ui,
                &mut self.k,
                &[
                    (t("Radar 2,2", "radar 2.2"), 2.2),
                    (t("optisch 2,08", "optical 2.08"), 2.08),
                    (t("geometrisch 1,93", "geometric 1.93"), 1.93),
                ],
            );
            if ui
                .button(t(
                    "Beispiel: Steilküste 80 m, Antenne 25 m",
                    "Example: cliff 80 m, antenna 25 m",
                ))
                .clicked()
            {
                self.h_ant = 25.0;
                self.h_tgt = 80.0;
                self.k = 2.2;
            }
        });
        let d1 = self.k * self.h_ant.sqrt();
        let d2 = self.k * self.h_tgt.sqrt();
        ui.label(
            RichText::new(tf!(
                "d = {:.2} · (√{:.1} + √{:.1}) = {:.2} + {:.2} = {:.2} sm",
                "d = {:.2} · (√{:.1} + √{:.1}) = {:.2} + {:.2} = {:.2} nm",
                self.k,
                self.h_ant,
                self.h_tgt,
                d1,
                d2,
                d1 + d2
            ))
            .monospace()
            .strong()
            .size(15.0),
        );
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 250.0), Sense::hover());
        draw_horizon(
            &ui.painter_at(rect),
            rect,
            &p,
            self.h_ant,
            self.h_tgt,
            d1,
            d2,
        );

        sub(
            ui,
            t(
                "Radargleichung: Wie viel Energie kommt zurück?",
                "Radar equation: how much energy comes back?",
            ),
        );
        text(
            ui,
            t(
                "Auf dem Hinweg verteilt sich die Energie auf eine immer größere Kugelfläche (∝ 1/R²), auf dem Rückweg noch einmal. Am Empfänger kommt deshalb nur ein winziger Bruchteil ∝ 1/R⁴ an. Ein Ziel wird erkannt, solange dieser Bruchteil über der Rauschschwelle liegt. Doppelte Reichweite verlangt 16-fache Energie.",
                "On the way out the energy spreads over an ever larger sphere (∝ 1/R²), and again on the way back. Only a tiny fraction ∝ 1/R⁴ reaches the receiver. A target is detected as long as this fraction stays above the noise threshold. Twice the range needs 16 times the energy.",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.x_band, true, "X-Band 9,41 GHz");
            ui.selectable_value(&mut self.x_band, false, "S-Band 3,05 GHz");
            slider(
                ui,
                &mut self.d,
                1.2..=3.7,
                t("Antenne", "antenna"),
                " m",
                0.01,
            );
        });
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.pt,
                1.0..=60.0,
                t("Sendeleistung", "peak power"),
                " kW",
                0.5,
            );
            slider(
                ui,
                &mut self.tau,
                0.05..=1.2,
                t("Pulslänge", "pulse length"),
                " µs",
                0.01,
            );
        });
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.sig_log,
                -1.0..=4.0,
                t("Radarquerschnitt log₁₀ σ", "radar cross section log₁₀ σ"),
                "",
                0.01,
            );
            presets(
                ui,
                &mut self.sig_log,
                &[
                    (t("Jolle ≈ 0,5 m²", "dinghy ≈ 0.5 m²"), 0.5f64.log10()),
                    (
                        t("IMO-Reflektor X 7,5 m²", "IMO reflector X 7.5 m²"),
                        7.5f64.log10(),
                    ),
                    (t("Frachter ≈ 10⁴ m²", "freighter ≈ 10⁴ m²"), 4.0),
                ],
            );
            slider(
                ui,
                &mut self.rmax,
                1.0..=96.0,
                t("Achse bis", "axis to"),
                t(" sm", " nm"),
                1.0,
            );
        });

        // Radargleichung (Freiraum) wie im MATLAB-Vorbild
        let f = if self.x_band { 9.41e9 } else { 3.05e9 };
        let lambda = C / f;
        let th3 = 0.886 * lambda / self.d;
        let thv = (if self.x_band { 20.0f64 } else { 25.0 }).to_radians();
        let g = 0.6 * 4.0 * std::f64::consts::PI / (th3 * thv);
        let ls = 10f64.powf(0.8);
        let etx = self.pt * 1e3 * self.tau * 1e-6;
        let emin = 1.38e-23 * 290.0 * 10f64.powf(0.6) * 10f64.powf(1.3) / 10f64.powf(1.0);
        let sigma = 10f64.powf(self.sig_log);
        let fpi = 4.0 * std::f64::consts::PI;
        let frac_t = |r: f64| g * sigma / (fpi * r * r);
        let frac_e = |r: f64| g * g * lambda * lambda * sigma / (fpi.powi(3) * r.powi(4) * ls);
        let rdet =
            (etx * g * g * lambda * lambda * sigma / (fpi.powi(3) * ls * emin)).powf(0.25) / 1852.0;
        let prf = prf_of(self.tau);
        let rua = C / (2.0 * prf) / 1852.0;
        let rhor = self.k * (self.h_ant.sqrt() + self.h_tgt.sqrt());
        let thr = 10.0 * (emin / etx).log10();
        let r0 = 0.05;
        let r1 = self.rmax.max(0.2);
        let e_end = 10.0 * frac_e(r1 * 1852.0).log10();
        let t_start = 10.0 * frac_t(r0 * 1852.0).log10();
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 290.0), Sense::hover());
        let painter = ui.painter_at(rect);
        let ch = Chart::new(
            rect,
            (r0, r1),
            (e_end.min(thr) - 15.0, t_start.max(0.0) + 5.0),
        )
        .log_x();
        ch.frame(
            &painter,
            &p,
            &tf!(
                "Pulsenergie {:.2} mJ  ·  Keule {:.2}°  ·  Gewinn {:.0} dB",
                "pulse energy {:.2} mJ  ·  beam {:.2}°  ·  gain {:.0} dB",
                etx * 1e3,
                th3.to_degrees(),
                10.0 * g.log10()
            ),
            t(
                "Entfernung R (sm, logarithmisch)",
                "range R (nm, logarithmic)",
            ),
            t(
                "Anteil der Pulsenergie (dB)",
                "fraction of pulse energy (dB)",
            ),
        );
        let n = 300;
        let rs = |i: usize| (r0.ln() + (r1.ln() - r0.ln()) * i as f64 / n as f64).exp();
        ch.line(
            &painter,
            (0..=n).map(|i| (rs(i), 10.0 * frac_t(rs(i) * 1852.0).log10())),
            Stroke::new(1.3_f32, p.s),
        );
        ch.line(
            &painter,
            (0..=n).map(|i| (rs(i), 10.0 * frac_e(rs(i) * 1852.0).log10())),
            Stroke::new(2.4_f32, p.accent),
        );
        ch.hline(
            &painter,
            thr,
            Stroke::new(1.2_f32, p.warn),
            t("Detektionsschwelle", "detection threshold"),
        );
        ch.vline(
            &painter,
            rdet,
            Stroke::new(1.5_f32, p.ok),
            &tf!("Detektion bis {:.1} sm", "detection to {:.1} nm", rdet),
        );
        ch.vline(
            &painter,
            rua,
            Stroke::new(1.5_f32, p.cur),
            &tf!(
                "eindeutig bis {:.0} sm (PRF {:.0} Hz)",
                "unambiguous to {:.0} nm (PRF {:.0} Hz)",
                rua,
                prf
            ),
        );
        ch.vline(
            &painter,
            rhor,
            Stroke::new(1.5_f32, p.faint),
            &tf!("Horizont {:.1} sm", "horizon {:.1} nm", rhor),
        );
        painter.text(
            pos2(ch.plot.left() + 8.0, ch.plot.bottom() - 8.0),
            Align2::LEFT_BOTTOM,
            t("dünn: Anteil, der das Ziel trifft (∝ 1/R²)   ·   dick: Echo an der Antenne (∝ 1/R⁴)", "thin: fraction hitting the target (∝ 1/R²)   ·   thick: echo at the antenna (∝ 1/R⁴)"),
            FontId::proportional(11.0),
            p.faint,
        );
        let eff = rdet.min(rhor);
        ui.label(
            RichText::new(tf!(
                "Sichtbar bis ≈ {:.1} sm – begrenzt durch {}",
                "Visible to ≈ {:.1} nm – limited by {}",
                eff,
                if rdet < rhor {
                    t(
                        "die Energie (Radargleichung)",
                        "the energy (radar equation)",
                    )
                } else {
                    t("den Radarhorizont", "the radar horizon")
                }
            ))
            .strong()
            .size(15.0),
        );

        sub(
            ui,
            t(
                "Warum sehe ich die Jolle nicht?",
                "Why can't I see the dinghy?",
            ),
        );
        egui::Grid::new("dinghy")
            .num_columns(2)
            .spacing([16.0, 4.0])
            .show(ui, |ui| {
                let rows: [(&str, Chapter); 7] = [
                    (
                        t(
                            "unter dem Radarhorizont (niedrig, weit weg)",
                            "below the radar horizon (low, far away)",
                        ),
                        Chapter::Range,
                    ),
                    (
                        t(
                            "zu kleiner Radarquerschnitt (Holz, GFK, kein Reflektor)",
                            "too small a radar cross section (wood, GRP, no reflector)",
                        ),
                        Chapter::Range,
                    ),
                    (
                        t(
                            "im Seegangsclutter verborgen oder durch STC gelöscht",
                            "hidden in sea clutter or erased by STC",
                        ),
                        Chapter::Signal,
                    ),
                    (
                        t(
                            "im Regen verdeckt oder gedämpft (X-Band)",
                            "hidden or attenuated in rain (X-band)",
                        ),
                        Chapter::Bands,
                    ),
                    (
                        t(
                            "Gain zu gering, Tune verstellt, FTC zu stark",
                            "gain too low, tune off, FTC too strong",
                        ),
                        Chapter::Signal,
                    ),
                    (
                        t(
                            "im Schatten- oder Blindsektor (Mast, Schornstein, Land)",
                            "in a shadow or blind sector (mast, funnel, land)",
                        ),
                        Chapter::FalseEchoes,
                    ),
                    (
                        t(
                            "nicht aufgelöst neben einem großen Ziel",
                            "not resolved next to a large target",
                        ),
                        Chapter::BearingRes,
                    ),
                ];
                for (s, c) in rows {
                    ui.label(format!("•  {s}"));
                    if ui.link(format!("→ {} {}", c.number(), c.title())).clicked() {
                        *nav = Some(c);
                    }
                    ui.end_row();
                }
            });
        try_it(
            ui,
            t(
                "Wählen Sie „Jolle ≈ 0,5 m²“ und senken Sie die Zielhöhe auf 1 m: Der Horizont schrumpft auf wenige Seemeilen. Verdoppeln Sie dann die Sendeleistung – die Detektionsreichweite wächst nur um ≈ 19 %, denn sie steigt mit der vierten Wurzel der Energie.",
                "Select “dinghy ≈ 0.5 m²” and lower the target height to 1 m: the horizon shrinks to a few miles. Then double the transmitter power – the detection range grows by only ≈ 19 %, because it rises with the fourth root of the energy.",
            ),
        );
        merke(
            ui,
            &[
                t("Radarhorizont: d = 2,2 · (√h_A + √h_Z) sm, h in m.", "Radar horizon: d = 2.2 · (√h_A + √h_T) nm, h in m."),
                t("Echoenergie ∝ 1/R⁴ ⇒ Reichweite ∝ ⁴√(Pt · τ · σ).", "Echo energy ∝ 1/R⁴ ⇒ range ∝ ⁴√(Pt · τ · σ)."),
                t("Längerer Puls ⇒ mehr Energie und Reichweite, aber schlechtere radiale Auflösung.", "Longer pulse ⇒ more energy and range, but poorer range resolution."),
                t("Die PRF begrenzt die eindeutige Reichweite: R = c/(2·PRF).", "The PRF limits the unambiguous range: R = c/(2·PRF)."),
                t("Über- und Unterreichweiten (Ducting, Subrefraktion) verschieben den Horizont wetterabhängig.", "Super- and sub-refraction (ducting) shift the horizon depending on the weather."),
            ],
        );
        see_also(
            ui,
            nav,
            &[
                Chapter::Bands,
                Chapter::RangeRes,
                Chapter::FalseEchoes,
                Chapter::Signal,
            ],
        );
    }
}

/// Seitenansicht: Strahl horizontal, Erde als Parabel (Absenkung (u/k)² m bei u sm vom Horizontpunkt).
fn draw_horizon(p: &egui::Painter, r: Rect, pal: &Pal, h1: f64, h2: f64, d1: f64, d2: f64) {
    p.rect_filled(r, CornerRadius::same(6), pal.panel);
    let total = d1 + d2;
    let left = r.left() + 50.0;
    let right = r.right() - 50.0;
    let top = r.top() + 50.0;
    let hmax = h1.max(h2).max(5.0) * 1.25;
    let sx = |x: f64| left + ((x / total) as f32) * (right - left);
    let sy = |y: f64| top + ((-y / hmax) as f32) * (r.bottom() - 20.0 - top); // y ≤ 0 nach unten
    let k = d1 / h1.sqrt();
    let ground = |x: f64| -((x - d1) / k).powi(2);
    // Meer/Erde
    let mut pts = vec![pos2(left - 40.0, r.bottom())];
    for i in 0..=200 {
        let x = -0.15 * total + 1.3 * total * i as f64 / 200.0;
        pts.push(pos2(sx(x), sy(ground(x).max(-hmax * 1.6)).min(r.bottom())));
    }
    pts.push(pos2(right + 40.0, r.bottom()));
    let clip = p.with_clip_rect(r.shrink(1.0));
    let surf: Vec<_> = pts[1..pts.len() - 1].to_vec();
    for w in surf.windows(2) {
        let poly = vec![
            w[0],
            w[1],
            pos2(w[1].x, r.bottom()),
            pos2(w[0].x, r.bottom()),
        ];
        clip.add(Shape::convex_polygon(
            poly,
            Color32::from_rgba_unmultiplied(40, 110, 190, 120),
            Stroke::NONE,
        ));
    }
    clip.add(Shape::line(surf, Stroke::new(1.5_f32, pal.s)));
    // Strahl
    clip.extend(Shape::dashed_line(
        &[pos2(sx(0.0), sy(0.0)), pos2(sx(total), sy(0.0))],
        Stroke::new(1.6_f32, pal.echo),
        8.0,
        5.0,
    ));
    // Antenne und Ziel
    p.line_segment(
        [pos2(sx(0.0), sy(-h1)), pos2(sx(0.0), sy(0.0))],
        Stroke::new(3.0_f32, pal.ink),
    );
    p.circle_filled(pos2(sx(0.0), sy(0.0)), 5.0, pal.accent);
    p.line_segment(
        [pos2(sx(total), sy(-h2)), pos2(sx(total), sy(0.0))],
        Stroke::new(5.0_f32, pal.tgt),
    );
    p.circle_filled(pos2(sx(d1), sy(0.0)), 4.0, pal.ok);
    p.text(
        pos2(sx(0.0), sy(0.0) - 8.0),
        Align2::CENTER_BOTTOM,
        &tf!("Radar {:.0} m", "radar {:.0} m", h1),
        FontId::proportional(12.0),
        pal.ink,
    );
    p.text(
        pos2(sx(total), sy(0.0) - 8.0),
        Align2::CENTER_BOTTOM,
        &tf!("Ziel {:.0} m", "target {:.0} m", h2),
        FontId::proportional(12.0),
        pal.tgt,
    );
    p.text(
        pos2(sx(d1 / 2.0), sy(0.0) - 6.0),
        Align2::CENTER_BOTTOM,
        format!("{:.2} {}", d1, t("sm", "nm")),
        FontId::proportional(12.0),
        pal.echo,
    );
    p.text(
        pos2(sx(d1 + d2 / 2.0), sy(0.0) - 6.0),
        Align2::CENTER_BOTTOM,
        format!("{:.2} {}", d2, t("sm", "nm")),
        FontId::proportional(12.0),
        pal.echo,
    );
    p.text(
        pos2(sx(d1), sy(0.0) + 8.0),
        Align2::CENTER_TOP,
        t("Radarhorizont", "radar horizon"),
        FontId::proportional(11.0),
        pal.ok,
    );
    p.text(
        r.left_top() + vec2(10.0, 6.0),
        Align2::LEFT_TOP,
        t(
            "Seitenansicht (Höhen stark überhöht)",
            "Side view (heights strongly exaggerated)",
        ),
        FontId::proportional(13.0),
        pal.ink,
    );
}
