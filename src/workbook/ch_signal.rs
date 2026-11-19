//! Kapitel 8: Signalverarbeitung am A-Scope – Gain, STC, FTC, Tune, Abtastung.

use eframe::egui::{self, pos2, vec2, Color32, CornerRadius, Rect, RichText, Sense, Stroke};

use super::charts::Chart;
use super::{merke, pal, see_also, slider, sub, text, try_it, Chapter, Pal};
use crate::i18n::t;
use crate::util::Rng;

const N: usize = 900;
const RMAX: f64 = 6.0; // sm

pub struct State {
    gain: f64,
    stc: f64,
    ftc: f64,
    thresh: f64,
    noise: f64,
    sea: f64,
    rain: bool,
    tune: f64,
    minutes: f64,
    bits: f64,
    sample_m: f64,
    seed: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            gain: 50.0,
            stc: 0.0,
            ftc: 0.0,
            thresh: 0.5,
            noise: 0.10,
            sea: 3.0,
            rain: true,
            tune: 0.0,
            minutes: 0.0,
            bits: 3.0,
            sample_m: 60.0,
            seed: 1,
        }
    }
}

struct Chain {
    r: Vec<f64>,
    raw: Vec<f64>,
    stc_curve: Vec<f64>,
    after_stc: Vec<f64>,
    after_ftc: Vec<f64>,
    video: Vec<f64>,
    tune_factor: f64,
    drift: f64,
}

impl State {
    fn drift(&self) -> f64 {
        9.0 * (1.0 - (-self.minutes / 12.0).exp())
    }

    fn compute(&self) -> Chain {
        let mut rng = Rng::new(self.seed);
        let bin = RMAX / N as f64;
        let r: Vec<f64> = (0..N).map(|i| (i as f64 + 0.5) * bin).collect();
        let drift = self.drift();
        let mismatch = drift - self.tune;
        let tune_factor = 1.0 / (1.0 + (mismatch / 2.5).powi(2));
        const SEA: [f64; 7] = [0.0, 0.4, 0.8, 1.4, 2.2, 3.2, 4.4];
        let st = self.sea.round().clamp(0.0, 6.0) as usize;
        let r0 = 0.25 + 0.15 * st as f64;
        let targets = [(0.55, 0.04, 1.8), (3.2, 0.03, 1.4), (5.0, 0.05, 0.85)];
        let raw: Vec<f64> = r
            .iter()
            .map(|&x| {
                let mut s = SEA[st] * (-x / r0).exp() * rng.expo() as f64;
                if self.rain && (2.8..3.6).contains(&x) {
                    let u = (x - 3.2) / 0.4;
                    s += 1.0 * (1.0 - u * u) * rng.expo() as f64;
                }
                for (tr, len, a) in targets {
                    if x >= tr && x <= tr + len {
                        s += a * (0.85 + 0.3 * rng.f64());
                    }
                }
                s * tune_factor + self.noise * rng.expo() as f64
            })
            .collect();
        let k = self.stc / 100.0 * 14.0;
        let stc_curve: Vec<f64> = r
            .iter()
            .map(|&x| 1.0 / (1.0 + k * (-x / 1.3).exp()))
            .collect();
        let after_stc: Vec<f64> = raw.iter().zip(&stc_curve).map(|(a, s)| a * s).collect();
        let kf = self.ftc / 100.0 * 1.1;
        let w = ((0.08 / bin) as usize).max(3);
        let mut after_ftc = after_stc.clone();
        if kf > 0.0 {
            let mut run = 0.0;
            for i in 0..N {
                let mean = run / w as f64;
                after_ftc[i] = (after_stc[i] - kf * mean).max(0.0);
                run += after_stc[i];
                if i >= w {
                    run -= after_stc[i - w];
                }
            }
        }
        let g = 10f64.powf((self.gain - 50.0) / 40.0);
        let video: Vec<f64> = after_ftc.iter().map(|v| v * g).collect();
        Chain {
            r,
            raw,
            stc_curve,
            after_stc,
            after_ftc,
            video,
            tune_factor,
            drift,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Das A-Scope zeigt die Echoamplitude eines einzigen Peilstrahls über der Entfernung – also genau das, was der Empfänger nach jedem Puls sieht. Von oben nach unten durchläuft das Signal die Bearbeitungsschritte; ganz unten steht, was als Helligkeit auf dem PPI landet. Im Signal: Rauschen, Seegangsclutter, ein Regenschauer und drei Ziele (Boot im Clutter bei 0,55 sm, Tonne im Regen bei 3,2 sm, Schiff bei 5 sm).",
                "The A-scope shows the echo amplitude of a single bearing over range – exactly what the receiver sees after each pulse. From top to bottom the signal passes through the processing steps; at the bottom is what ends up as brightness on the PPI. In the signal: noise, sea clutter, a rain shower and three targets (boat in clutter at 0.55 nm, buoy in rain at 3.2 nm, ship at 5 nm).",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            slider(ui, &mut self.gain, 0.0..=100.0, "GAIN", "", 1.0);
            slider(ui, &mut self.stc, 0.0..=100.0, "SEA (STC)", "", 1.0);
            slider(ui, &mut self.ftc, 0.0..=100.0, "RAIN (FTC)", "", 1.0);
        });
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.sea,
                0.0..=6.0,
                t("Seegang", "sea state"),
                "",
                1.0,
            );
            ui.checkbox(&mut self.rain, t("Regenschauer", "rain shower"));
            slider(
                ui,
                &mut self.noise,
                0.0..=0.4,
                t("Rauschen", "noise"),
                "",
                0.01,
            );
            if ui.button(t("neues Rauschen", "new noise")).clicked() {
                self.seed = self.seed.wrapping_add(1);
            }
        });
        let ch = self.compute();
        let h = 150.0;
        let xr = (0.0, RMAX);
        let plot = |ui: &mut egui::Ui,
                    title: &str,
                    data: &[f64],
                    col: Color32,
                    extra: &dyn Fn(&Chart, &egui::Painter)| {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
            let painter = ui.painter_at(rect);
            let ymax = 3.0;
            let c = Chart::new(rect, xr, (0.0, ymax));
            c.frame(&painter, &p, title, t("Entfernung (sm)", "range (nm)"), "");
            c.line(
                &painter,
                ch.r.iter().zip(data).map(|(x, y)| (*x, y.min(ymax * 1.02))),
                Stroke::new(1.2_f32, col),
            );
            extra(&c, &painter);
        };
        plot(
            ui,
            t(
                "1 · Empfangssignal (Rohvideo) – gestrichelt: STC-Verstärkungskurve",
                "1 · received signal (raw video) – dashed: STC gain curve",
            ),
            &ch.raw,
            p.s,
            &|c, painter| {
                let pts: Vec<(f64, f64)> =
                    ch.r.iter()
                        .zip(&ch.stc_curve)
                        .map(|(x, s)| (*x, s * 2.8))
                        .collect();
                for w in pts.windows(2).step_by(3) {
                    c.line(painter, [w[0], w[1]], Stroke::new(1.5_f32, p.cur));
                }
            },
        );
        plot(
            ui,
            t(
                "2 · nach STC (Seegangsenttrübung)",
                "2 · after STC (sea clutter control)",
            ),
            &ch.after_stc,
            p.accent,
            &|_, _| {},
        );
        plot(
            ui,
            t(
                "3 · nach FTC (Regenenttrübung, Differenzierer)",
                "3 · after FTC (rain clutter control, differentiator)",
            ),
            &ch.after_ftc,
            p.x,
            &|_, _| {},
        );
        let thr = self.thresh;
        let bits = self.bits.round() as u32;
        let step = (self.sample_m / 1852.0).max(RMAX / N as f64);
        plot(
            ui,
            t(
                "4 · nach Gain mit Anzeigeschwelle und A/D-Abtastung",
                "4 · after gain, with display threshold and A/D sampling",
            ),
            &ch.video,
            p.echo,
            &|c, painter| {
                c.hline(
                    painter,
                    thr,
                    Stroke::new(1.0_f32, p.warn),
                    t("Schwelle", "threshold"),
                );
                // Abtastung (Zeit) und Quantisierung (Wert)
                let levels = (1u32 << bits) - 1;
                let vmax = 3.0;
                let mut x = 0.0;
                let mut prev: Option<(f64, f64)> = None;
                while x < RMAX {
                    let i = ((x / RMAX) * N as f64) as usize;
                    let v = ch.video[i.min(N - 1)].clamp(0.0, vmax);
                    let q = (v / vmax * levels as f64).round() / levels as f64 * vmax;
                    if let Some((px, pq)) = prev {
                        c.line(
                            painter,
                            [(px, pq), (x, pq), (x, q)],
                            Stroke::new(1.4_f32, p.warn.gamma_multiply(0.8)),
                        );
                    }
                    c.dot(painter, x, q, 2.5, p.warn);
                    prev = Some((x, q));
                    x += step;
                }
            },
        );
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.thresh,
                0.1..=1.5,
                t("Anzeigeschwelle", "display threshold"),
                "",
                0.01,
            );
            slider(
                ui,
                &mut self.sample_m,
                12.0..=300.0,
                t("Abtastabstand", "sample spacing"),
                " m",
                1.0,
            );
            slider(
                ui,
                &mut self.bits,
                1.0..=8.0,
                t("A/D-Auflösung", "A/D resolution"),
                " bit",
                1.0,
            );
        });
        // Ergebnis-Streifen wie eine PPI-Radiallinie
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::hover());
        draw_strip(&ui.painter_at(rect), rect, &p, &ch.video, thr);

        sub(
            ui,
            t(
                "Tune: Abstimmung nach dem Kaltstart",
                "Tune: tuning after a cold start",
            ),
        );
        text(
            ui,
            t(
                "Das Magnetron erwärmt sich nach dem Einschalten, seine Sendefrequenz driftet um einige MHz. Der Empfänger ist schmalbandig – passt seine Abstimmung nicht mehr, kommen alle Echos geschwächt an, das Rauschen aber nicht. Mehr Gain hilft dann nicht, nur Nachstimmen (Tune).",
                "The magnetron heats up after switching on and its transmit frequency drifts by a few MHz. The receiver is narrow-band – if its tuning no longer matches, all echoes arrive weakened, but the noise does not. More gain does not help then, only re-tuning.",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.minutes,
                0.0..=60.0,
                t("Minuten seit Kaltstart", "minutes since cold start"),
                " min",
                0.5,
            );
            slider(ui, &mut self.tune, -5.0..=15.0, "Tune", " MHz", 0.1);
            if ui.button(t("Auto-Tune", "auto tune")).clicked() {
                self.tune = (self.drift() * 10.0).round() / 10.0;
            }
        });
        let tf = ch.tune_factor;
        ui.label(
            RichText::new(tf!(
                "Magnetron-Drift {:.1} MHz  ·  Abstimmung {:.1} MHz  ·  Echos erreichen den Empfänger mit {:.0} %",
                "magnetron drift {:.1} MHz  ·  tuning {:.1} MHz  ·  echoes reach the receiver at {:.0} %",
                ch.drift,
                self.tune,
                tf * 100.0
            ))
            .monospace()
            .strong()
            .color(if tf > 0.8 { p.ok } else { p.warn }),
        );
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 170.0), Sense::hover());
        let painter = ui.painter_at(rect);
        let c = Chart::new(rect, (-10.0, 20.0), (0.0, 1.1));
        c.frame(
            &painter,
            &p,
            t(
                "Empfängerdurchlass und Sendefrequenz",
                "receiver passband and transmit frequency",
            ),
            t("Frequenzabweichung (MHz)", "frequency offset (MHz)"),
            "",
        );
        let tune = self.tune;
        c.line(
            &painter,
            (0..=300).map(|i| {
                let x = -10.0 + 30.0 * i as f64 / 300.0;
                (x, 1.0 / (1.0 + ((x - tune) / 2.5).powi(2)))
            }),
            Stroke::new(2.0_f32, p.accent),
        );
        c.vline(
            &painter,
            ch.drift,
            Stroke::new(2.0_f32, p.warn),
            t("Magnetron", "magnetron"),
        );

        try_it(
            ui,
            t(
                "Drehen Sie den Seegang hoch und dann SEA (STC): Der Nahclutter verschwindet, das Boot bei 0,55 sm bleibt – bis zu viel STC auch das Boot löscht. FTC schneidet aus dem Regenblock nur die Vorderkante heraus und lässt die Tonne darin erscheinen. Stellen Sie 20 Minuten seit Kaltstart ein: Alle Echos schrumpfen; Gain verstärkt auch das Rauschen, Auto-Tune holt die Echos zurück.",
                "Turn up the sea state and then SEA (STC): the close clutter disappears, the boat at 0.55 nm remains – until too much STC erases the boat too. FTC cuts only the leading edge out of the rain block and lets the buoy inside appear. Set 20 minutes since cold start: all echoes shrink; gain also amplifies the noise, auto tune brings the echoes back.",
            ),
        );
        merke(
            ui,
            &[
                t("Gain: verstärkt alles – Echos und Rauschen. Richtig: leichtes Rauschen sichtbar.", "Gain: amplifies everything – echoes and noise. Correct: light noise visible."),
                t("STC (Sensitivity Time Control): Verstärkung im Nahbereich abgesenkt, steigt mit der Entfernung – gegen Seegangsclutter.", "STC (sensitivity time control): gain reduced at short range, rising with range – against sea clutter."),
                t("FTC (Fast Time Constant): differenziert – Echos auf Vorderflanke verkürzt, flächige Echos (Regen, Land) bis auf die Kanten unterdrückt.", "FTC (fast time constant): differentiates – echoes shortened to the leading edge, extended echoes (rain, land) suppressed except for edges."),
                t("Tune: nach dem Kaltstart nachstimmen, weil die Magnetronfrequenz driftet.", "Tune: re-tune after a cold start because the magnetron frequency drifts."),
                t("Sampling (A/D-Wandler): Zeit wird diskretisiert (Abtastrate), der Wert quantisiert (Bitauflösung).", "Sampling (A/D converter): time is discretised (sampling rate), the value quantised (bit resolution)."),
            ],
        );
        see_also(
            ui,
            nav,
            &[
                Chapter::Bands,
                Chapter::RangeRes,
                Chapter::Display,
                Chapter::Range,
            ],
        );
    }
}

fn draw_strip(p: &egui::Painter, r: Rect, pal: &Pal, video: &[f64], thr: f64) {
    p.rect_filled(r, CornerRadius::same(6), pal.panel);
    let inner = Rect::from_min_max(r.min + vec2(56.0, 8.0), r.max - vec2(14.0, 8.0));
    p.rect_filled(inner, CornerRadius::same(3), Color32::from_rgb(26, 62, 128));
    let n = video.len();
    for i in 0..n {
        let v = video[i];
        if v > thr {
            let a = (1.0 - (-(v - thr) * 1.6).exp()) as f32;
            let x = inner.left() + inner.width() * i as f32 / n as f32;
            p.line_segment(
                [pos2(x, inner.top()), pos2(x, inner.bottom())],
                Stroke::new(
                    1.0_f32,
                    Color32::from_rgba_unmultiplied(255, 235, 120, (40.0 + 215.0 * a) as u8),
                ),
            );
        }
    }
    p.text(
        r.left_center() + vec2(6.0, 0.0),
        egui::Align2::LEFT_CENTER,
        "PPI",
        egui::FontId::proportional(12.0),
        pal.ink,
    );
}
