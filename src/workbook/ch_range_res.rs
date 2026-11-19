//! Kapitel 4: Radiale Auflösung – Pulslänge, PRF, Bereich, Pulskompression.

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Rect, RichText, Sense, Stroke,
};

use super::charts::Chart;
use super::{formula, merke, pal, presets, see_also, slider, sub, text, try_it, Chapter, Pal};
use crate::i18n::t;

pub struct State {
    tau: f64,
    sep: f64,
    compress: bool,
    bw: f64,
    amp2: f64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tau: 0.15,
            sep: 30.0,
            compress: false,
            bw: 20.0,
            amp2: 1.0,
        }
    }
}

fn sig(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

impl State {
    /// Echoamplitude eines Ziels bei x0 (m) am Ort x (m).
    fn echo(&self, x: f64, x0: f64) -> f64 {
        if self.compress {
            let w = 150.0 / self.bw; // komprimierte Echobreite in m
            let a = std::f64::consts::PI * (x - x0) / w;
            if a.abs() < 1e-9 {
                1.0
            } else {
                (a.sin() / a).abs()
            }
        } else {
            let l = 150.0 * self.tau;
            let r = (l * 0.04).max(1.5);
            sig((x - x0) / r) - sig((x - x0 - l) / r)
        }
    }

    fn signal(&self, x: f64) -> f64 {
        self.echo(x, 0.0) + self.amp2 * self.echo(x, self.sep)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Zwei Ziele liegen auf derselben Peilung hintereinander. Ein Puls der Dauer τ ist im Raum c·τ lang; da die Welle hin und zurück läuft, belegt sein Echo auf dem Schirm die Strecke c·τ/2. Liegen die Ziele näher beieinander, überlappen ihre Echos und verschmelzen zu einem.",
                "Two targets lie one behind the other on the same bearing. A pulse of duration τ is c·τ long in space; since the wave travels out and back, its echo occupies c·τ/2 on the screen. If the targets are closer than that, their echoes overlap and merge into one.",
            ),
        );
        formula(
            ui,
            t(
                "ΔR = c · τ / 2 = 150 m/µs · τ        (c = 300 m/µs)",
                "ΔR = c · τ / 2 = 150 m/µs · τ        (c = 300 m/µs)",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.tau,
                0.05..=1.2,
                t("Pulslänge τ", "pulse length τ"),
                " µs",
                0.01,
            );
            presets(
                ui,
                &mut self.tau,
                &[("SP 0,07 µs", 0.07), ("MP 0,3 µs", 0.3), ("LP 1,2 µs", 1.2)],
            );
        });
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.sep,
                2.0..=300.0,
                t("Abstand der Ziele", "target separation"),
                " m",
                1.0,
            );
            slider(
                ui,
                &mut self.amp2,
                0.05..=1.0,
                t("Echostärke Ziel 2", "echo strength target 2"),
                "",
                0.01,
            );
        });
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(
                &mut self.compress,
                t("Pulskompression (Chirp)", "pulse compression (chirp)"),
            );
            if self.compress {
                slider(
                    ui,
                    &mut self.bw,
                    2.0..=60.0,
                    t("Bandbreite B", "bandwidth B"),
                    " MHz",
                    0.5,
                );
            }
        });

        let l = if self.compress {
            150.0 / self.bw
        } else {
            150.0 * self.tau
        };
        let xmin = -40.0 - if self.compress { 3.0 * l } else { 0.0 };
        let xmax = self.sep + l * if self.compress { 4.0 } else { 1.0 } + 60.0;
        // Echos zählen (Schwelle 0,5)
        let thr = 0.5;
        let n = 1500;
        let mut count = 0;
        let mut above = false;
        for i in 0..=n {
            let x = xmin + (xmax - xmin) * i as f64 / n as f64;
            let a = self.signal(x) > thr;
            if a && !above {
                count += 1;
            }
            above = a;
        }
        let (verdict, col) = match count {
            1 => (
                t(
                    "1 Echo – die Ziele sind verschmolzen",
                    "1 echo – the targets have merged",
                ),
                p.warn,
            ),
            2 => (
                t(
                    "2 Echos – die Ziele sind getrennt",
                    "2 echoes – the targets are resolved",
                ),
                p.ok,
            ),
            0 => (
                t("kein Echo über der Schwelle", "no echo above threshold"),
                p.warn,
            ),
            _ => (
                t(
                    "mehr Echos als Ziele – Scheinechos durch Entfernungs-Nebenzipfel",
                    "more echoes than targets – false echoes from range side lobes",
                ),
                p.warn,
            ),
        };
        ui.label(
            RichText::new(tf!(
                "Echolänge auf dem Schirm: {:.1} m   ·   {}",
                "Echo length on screen: {:.1} m   ·   {}",
                l,
                verdict
            ))
            .strong()
            .size(15.0)
            .color(col),
        );

        // A-Scope und PPI-Ausschnitt
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 260.0), Sense::hover());
        let painter = ui.painter_at(rect);
        let a_rect = Rect::from_min_max(rect.min, pos2(rect.right() - 190.0, rect.bottom()));
        let ppi_rect = Rect::from_min_max(pos2(rect.right() - 180.0, rect.top()), rect.max);
        let ch = Chart::new(a_rect, (xmin, xmax), (0.0, 2.1));
        ch.frame(
            &painter,
            &p,
            t(
                "A-Scope: Echoamplitude über der Entfernung",
                "A-scope: echo amplitude over range",
            ),
            t(
                "Entfernung relativ zu Ziel 1 (m)",
                "range relative to target 1 (m)",
            ),
            t("Amplitude", "amplitude"),
        );
        ch.line(
            &painter,
            (0..=n).map(|i| {
                let x = xmin + (xmax - xmin) * i as f64 / n as f64;
                (x, self.echo(x, 0.0))
            }),
            Stroke::new(1.0_f32, p.s.gamma_multiply(0.7)),
        );
        ch.line(
            &painter,
            (0..=n).map(|i| {
                let x = xmin + (xmax - xmin) * i as f64 / n as f64;
                (x, self.amp2 * self.echo(x, self.sep))
            }),
            Stroke::new(1.0_f32, p.x.gamma_multiply(0.7)),
        );
        ch.line(
            &painter,
            (0..=n).map(|i| {
                let x = xmin + (xmax - xmin) * i as f64 / n as f64;
                (x, self.signal(x))
            }),
            Stroke::new(2.4_f32, p.echo),
        );
        ch.hline(
            &painter,
            thr,
            Stroke::new(1.0_f32, p.warn),
            t("Anzeigeschwelle", "display threshold"),
        );
        ch.vline(
            &painter,
            0.0,
            Stroke::new(1.0_f32, p.s),
            t("Ziel 1", "target 1"),
        );
        ch.vline(
            &painter,
            self.sep,
            Stroke::new(1.0_f32, p.x),
            t("Ziel 2", "target 2"),
        );
        draw_ppi_cut(&painter, ppi_rect, &p, self, xmin, xmax, thr);

        // ΔR über τ
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 220.0), Sense::hover());
        let painter = ui.painter_at(rect);
        let ch = Chart::new(rect, (0.0, 1.2), (0.0, 200.0));
        ch.frame(
            &painter,
            &p,
            t(
                "Radiale Auflösung ΔR über der Pulslänge",
                "Range resolution ΔR over pulse length",
            ),
            "τ (µs)",
            "ΔR (m)",
        );
        ch.line(
            &painter,
            [(0.0, 0.0), (1.2, 180.0)],
            Stroke::new(2.0_f32, p.accent),
        );
        ch.hline(
            &painter,
            40.0,
            Stroke::new(1.2_f32, p.warn),
            t("IMO: 40 m", "IMO: 40 m"),
        );
        ch.vline(
            &painter,
            40.0 / 150.0,
            Stroke::new(1.0_f32, p.warn),
            "τ = 0,27 µs",
        );
        ch.dot(&painter, self.tau, 150.0 * self.tau, 6.0, p.echo);

        sub(
            ui,
            t(
                "Bereich, Pulslänge und Pulsfolgefrequenz",
                "Range scale, pulse length and PRF",
            ),
        );
        text(
            ui,
            t(
                "Beim Umschalten auf einen größeren Bereich wählt das Radar automatisch einen längeren Puls (mehr Energie für große Entfernungen, dafür schlechtere radiale Auflösung) und eine kleinere Pulsfolgefrequenz (PRF): Das nächste Signal darf erst gesendet werden, wenn die Echos aus der größten Entfernung zurück sind.",
                "When switching to a larger range scale the radar automatically selects a longer pulse (more energy for long ranges, poorer range resolution) and a lower pulse repetition frequency (PRF): the next pulse may only be sent once the echoes from the greatest range have returned.",
            ),
        );
        formula(
            ui,
            t(
                "eindeutige Reichweite  R = c / (2 · PRF)",
                "unambiguous range  R = c / (2 · PRF)",
            ),
        );
        egui::Grid::new("pulse_tab")
            .striped(true)
            .num_columns(5)
            .spacing([22.0, 4.0])
            .show(ui, |ui| {
                for h in [
                    t("Bereich", "Range scale"),
                    t("Puls", "Pulse"),
                    "τ",
                    "PRF",
                    t("ΔR / eindeutig bis", "ΔR / unambiguous to"),
                ] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for (r, name, tau, prf) in [
                    (t("0,25 – 1,5 sm", "0.25 – 1.5 nm"), "SP", 0.07, 3000.0),
                    (t("3 – 6 sm", "3 – 6 nm"), "MP", 0.3, 1500.0),
                    (t("12 – 96 sm", "12 – 96 nm"), "LP", 1.2, 600.0),
                ] {
                    ui.label(r);
                    ui.label(name);
                    ui.label(format!("{tau} µs"));
                    ui.label(format!("{prf:.0} Hz"));
                    ui.label(format!(
                        "{:.0} m  /  {:.0} {}",
                        150.0 * tau,
                        3.0e8 / (2.0 * prf) / 1852.0,
                        t("sm", "nm")
                    ));
                    ui.end_row();
                }
            });

        sub(ui, t("Pulskompression", "Pulse compression"));
        text(
            ui,
            t(
                "Ein langer Puls bringt viel Energie (Reichweite), ein kurzer gute Auflösung. Die Pulskompression vereint beides: Ein langer Puls wird in der Frequenz moduliert (Chirp) oder phasencodiert und im Empfänger auf ein kurzes Echo der Breite ≈ c/(2B) zusammengeschoben. Dafür entstehen neben starken Echos kleine Entfernungs-Nebenzipfel, und während des langen Sendepulses ist das Radar im Nahbereich blind.",
                "A long pulse gives a lot of energy (range), a short one good resolution. Pulse compression combines both: a long pulse is frequency-modulated (chirp) or phase-coded and compressed in the receiver into a short echo of width ≈ c/(2B). The price: small range side lobes next to strong echoes, and the radar is blind at short range while the long pulse is transmitted.",
            ),
        );
        try_it(
            ui,
            t(
                "Schieben Sie den Abstand unter c·τ/2: Die Echos verschmelzen. Mit SP (0,07 µs) trennt das Radar schon 11 m. Schalten Sie dann die Pulskompression ein, senken Sie die Echostärke von Ziel 2 auf 0,15 und den Abstand auf ≈ 10 m: Das kleine Ziel verschwindet im Nebenzipfel des großen.",
                "Move the separation below c·τ/2: the echoes merge. With SP (0.07 µs) the radar already resolves 11 m. Then switch on pulse compression, lower target 2's echo strength to 0.15 and the separation to ≈ 10 m: the small target disappears in the side lobe of the large one.",
            ),
        );
        merke(
            ui,
            &[
                t("ΔR = c·τ/2 – kürzerer Puls ⇒ bessere radiale Auflösung, aber weniger Energie.", "ΔR = c·τ/2 – shorter pulse ⇒ better range resolution, but less energy."),
                t("Größerer Bereich ⇒ längerer Puls und kleinere PRF (größere eindeutige Reichweite).", "Larger range scale ⇒ longer pulse and lower PRF (greater unambiguous range)."),
                t("IMO: zwei Ziele in 40 m Abstand müssen auf kleinen Bereichen getrennt erscheinen.", "IMO: two targets 40 m apart must appear separated on short range scales."),
                t("FTC verkürzt Echos auf ihre Vorderflanke und verbessert so die Trennung.", "FTC shortens echoes to their leading edge and thus improves separation."),
                t("Pulskompression (Chirp/Phasencode): Reichweite und Auflösung zugleich – Preis: Nebenzipfel und Blindbereich im Nahfeld.", "Pulse compression (chirp/phase code): range and resolution at once – price: side lobes and blind zone at short range."),
            ],
        );
        see_also(
            ui,
            nav,
            &[
                Chapter::BearingRes,
                Chapter::Signal,
                Chapter::Range,
                Chapter::FalseEchoes,
            ],
        );
    }
}

/// Ausschnitt eines PPI entlang der Peilung: Helligkeit = Echoamplitude über Schwelle.
fn draw_ppi_cut(p: &egui::Painter, r: Rect, pal: &Pal, st: &State, xmin: f64, xmax: f64, thr: f64) {
    p.rect_filled(r, CornerRadius::same(6), pal.panel);
    p.text(
        r.left_top() + vec2(8.0, 6.0),
        Align2::LEFT_TOP,
        t("PPI-Ausschnitt", "PPI cut-out"),
        FontId::proportional(13.0),
        pal.ink,
    );
    let view = Rect::from_min_max(r.min + vec2(20.0, 28.0), r.max - vec2(20.0, 22.0));
    p.rect_filled(view, CornerRadius::same(4), Color32::from_rgb(26, 62, 128));
    let rows = view.height() as usize;
    for i in 0..rows {
        // oben = weiter entfernt
        let x = xmax - (xmax - xmin) * i as f64 / rows as f64;
        let a = st.signal(x);
        if a > thr {
            let v = (1.0 - (-(a - thr) * 2.0).exp()) as f32;
            let c = Color32::from_rgba_unmultiplied(255, 235, 120, (60.0 + 195.0 * v) as u8);
            let y = view.top() + i as f32;
            p.line_segment(
                [
                    pos2(view.center().x - 34.0, y),
                    pos2(view.center().x + 34.0, y),
                ],
                Stroke::new(1.2_f32, c),
            );
        }
    }
    p.text(
        pos2(view.center().x, view.bottom() + 4.0),
        Align2::CENTER_TOP,
        t("↑ Entfernung", "↑ range"),
        FontId::proportional(11.0),
        pal.faint,
    );
}
