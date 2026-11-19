//! Kapitel 3: Antenne und Radarkeule (Richtdiagramm, Keulenbreite, Nebenkeulen,
//! Nah-/Fernfeld, gleichphasige Schlitzstrahler, Beam Steering).

use eframe::egui::{
    self, pos2, vec2, Align2, CornerRadius, FontId, Rect, RichText, Sense, Shape, Stroke,
};

use super::charts::Chart;
use super::{formula, merke, pal, presets, see_also, slider, sub, text, try_it, Chapter, Pal};
use crate::i18n::t;

const C: f64 = 299_792_458.0;

pub struct State {
    d: f64,
    f: f64,
    taper: bool,
    steer: f64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            d: 2.44,
            f: 9.41,
            taper: true,
            steer: 0.0,
        }
    }
}

/// Richtfaktor (Feldstärke, normiert) über u = π·D/λ·sin θ.
fn pattern(u: f64, taper: bool) -> f64 {
    if !taper {
        if u.abs() < 1e-9 {
            1.0
        } else {
            u.sin() / u
        }
    } else {
        // Kosinus-Belegung: E(u) = cos(u) / (1 − (2u/π)²)
        let q = 1.0 - (2.0 * u / std::f64::consts::PI).powi(2);
        if q.abs() < 1e-6 {
            std::f64::consts::FRAC_PI_4
        } else {
            u.cos() / q
        }
    }
}

/// Halbwertsbreite (Grad) numerisch aus dem Diagramm.
fn hpbw_deg(d: f64, lambda: f64, taper: bool) -> f64 {
    let mut a = 0.0f64;
    while a < 90.0 {
        let u = std::f64::consts::PI * d / lambda * a.to_radians().sin();
        if pattern(u, taper).abs() < std::f64::consts::FRAC_1_SQRT_2 {
            return 2.0 * a;
        }
        a += 0.002;
    }
    180.0
}

impl State {
    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Die Schlitzantenne ist ein Hohlleiter mit vielen kleinen Schlitzen. Jeder Schlitz strahlt wie eine kleine Kugelwelle. Weil alle Schlitze gleichphasig strahlen, überlagern sich die Wellen genau in Hauptrichtung konstruktiv – seitlich löschen sie sich weitgehend aus. So entsteht die schmale horizontale Keule.",
                "The slotted waveguide antenna has many small slots. Each slot radiates like a small spherical wave. Because all slots radiate in phase, the waves add up constructively exactly in the main direction – sideways they largely cancel. This forms the narrow horizontal beam.",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.d,
                0.5..=5.0,
                t("Antennenlänge D", "Antenna length D"),
                " m",
                0.01,
            );
            presets(
                ui,
                &mut self.d,
                &[
                    ("4 ft", 1.22),
                    ("6,5 ft", 1.98),
                    ("8 ft", 2.44),
                    ("10 ft", 3.05),
                    ("12 ft", 3.66),
                ],
            );
        });
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
                &[
                    ("S 3,05 GHz", 3.05),
                    ("X 9,41 GHz", 9.41),
                    (t("gerundet S 3 GHz", "rounded S 3 GHz"), 3.0),
                    (t("gerundet X 10 GHz", "rounded X 10 GHz"), 10.0),
                ],
            );
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(t("Belegung:", "Illumination:"));
            ui.selectable_value(&mut self.taper, false, t("gleichmäßig", "uniform"));
            ui.selectable_value(
                &mut self.taper,
                true,
                t(
                    "Kosinus-Taper (Faustformel 70·λ/D)",
                    "cosine taper (rule of thumb 70·λ/D)",
                ),
            );
        });

        let lambda = C / (self.f * 1e9);
        let hp = hpbw_deg(self.d, lambda, self.taper);
        let hp70 = 70.0 * lambda / self.d;
        let hp51 = 50.8 * lambda / self.d;
        let ff = 2.0 * self.d * self.d / lambda;
        egui::Grid::new("ant_nums")
            .num_columns(4)
            .spacing([18.0, 4.0])
            .show(ui, |ui| {
                let mono = |s: String| RichText::new(s).monospace().strong();
                ui.label("λ");
                ui.label(mono(format!("{:.2} cm", lambda * 100.0)));
                ui.label(t(
                    "Halbwertsbreite (Diagramm)",
                    "half-power beamwidth (pattern)",
                ));
                ui.label(mono(format!("{:.2}°", hp)));
                ui.end_row();
                ui.label(t("Faustformel 70·λ/D", "rule 70·λ/D"));
                ui.label(mono(format!("{:.2}°", hp70)));
                ui.label(t("gleichmäßig 51·λ/D", "uniform 51·λ/D"));
                ui.label(mono(format!("{:.2}°", hp51)));
                ui.end_row();
                ui.label(t("erste Nebenkeule", "first side lobe"));
                ui.label(mono(if self.taper {
                    "≈ −23 dB".into()
                } else {
                    "≈ −13 dB".into()
                }));
                ui.label(t("Fernfeld ab 2·D²/λ", "far field from 2·D²/λ"));
                ui.label(mono(format!("{:.0} m", ff)));
                ui.end_row();
            });

        // Richtdiagramm kartesisch + polar
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 300.0), Sense::hover());
        let painter = ui.painter_at(rect);
        let left = Rect::from_min_max(
            rect.min,
            pos2(rect.left() + rect.width() * 0.58 - 6.0, rect.bottom()),
        );
        let right = Rect::from_min_max(pos2(left.right() + 12.0, rect.top()), rect.max);
        let span = (3.0 * (lambda / self.d).min(1.0).asin().to_degrees()).clamp(10.0, 90.0);
        let ch = Chart::new(left, (-span, span), (-40.0, 3.0));
        ch.frame(
            &painter,
            &p,
            &tf!(
                "Richtdiagramm, Halbwertsbreite {:.2}°",
                "Radiation pattern, half-power beamwidth {:.2}°",
                hp
            ),
            t("Winkel zur Antennenachse (°)", "angle from boresight (°)"),
            "dB",
        );
        let k = std::f64::consts::PI * self.d / lambda;
        let taper = self.taper;
        let db = |deg: f64| {
            (20.0
                * pattern(k * deg.to_radians().sin(), taper)
                    .abs()
                    .max(1e-5)
                    .log10())
            .max(-40.0)
        };
        let n = 1200;
        ch.line(
            &painter,
            (0..=n).map(|i| {
                let a = -span + 2.0 * span * i as f64 / n as f64;
                (a, db(a))
            }),
            Stroke::new(2.0_f32, p.accent),
        );
        ch.hline(&painter, -3.0, Stroke::new(1.0_f32, p.warn), "−3 dB");
        ch.vline(&painter, -hp / 2.0, Stroke::new(1.0_f32, p.warn), "");
        ch.vline(&painter, hp / 2.0, Stroke::new(1.0_f32, p.warn), "");
        draw_polar(&painter, right, &p, &db, hp);

        sub(
            ui,
            t(
                "Gleichphasige Strahler und Beam Steering",
                "In-phase radiators and beam steering",
            ),
        );
        text(
            ui,
            t(
                "Jeder Punkt unten ist ein Schlitz, die Bögen sind seine Wellenfronten. Strahlen alle gleichphasig, bilden die Bögen eine gerade Front – die Keule zeigt senkrecht zur Antenne. Wird jeder Strahler gegenüber dem Nachbarn etwas verzögert angesteuert, kippt die Front: Die Keule schwenkt elektronisch, ohne dass sich die Antenne dreht (Phased Array).",
                "Each dot below is a slot, the arcs are its wavefronts. If all radiate in phase, the arcs form a straight front – the beam points perpendicular to the antenna. If each radiator is driven slightly later than its neighbour, the front tilts: the beam is steered electronically without turning the antenna (phased array).",
            ),
        );
        slider(
            ui,
            &mut self.steer,
            -0.45..=0.45,
            t(
                "Phasenversatz je Strahler (λ)",
                "phase shift per element (λ)",
            ),
            "",
            0.01,
        );
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 230.0), Sense::hover());
        draw_slots(&ui.painter_at(rect), rect, &p, self.steer);

        formula(
            ui,
            t(
                "Halbwertsbreite θ ≈ 70 · λ / D  (°)      Fernfeld ab  R ≈ 2 · D² / λ",
                "half-power beamwidth θ ≈ 70 · λ / D  (°)      far field from  R ≈ 2 · D² / λ",
            ),
        );
        try_it(
            ui,
            t(
                "Stellen Sie „gerundet S 3 GHz“ und D = 2,0 m ein: Die Faustformel liefert 3,5°. Mit „gerundet X 10 GHz“ und D = 1,8 m beginnt das Fernfeld bei 216 m. Schalten Sie auf gleichmäßige Belegung: Die Keule wird schmaler, aber die Nebenkeulen steigen auf −13 dB.",
                "Select “rounded S 3 GHz” and D = 2.0 m: the rule of thumb gives 3.5°. With “rounded X 10 GHz” and D = 1.8 m the far field starts at 216 m. Switch to uniform illumination: the beam becomes narrower, but the side lobes rise to −13 dB.",
            ),
        );
        merke(
            ui,
            &[
                t("Strahlbündelung durch gleichphasige (kohärente) Abstrahlung vieler Schlitze ⇒ konstruktive Interferenz in Hauptrichtung.", "Beam forming by in-phase (coherent) radiation of many slots ⇒ constructive interference in the main direction."),
                t("Längere Antenne oder kürzere Welle ⇒ schmalere Keule ⇒ bessere azimutale Auflösung.", "Longer antenna or shorter wave ⇒ narrower beam ⇒ better bearing resolution."),
                t("Taper (zum Rand schwächer) senkt die Nebenkeulen, verbreitert aber die Hauptkeule.", "Taper (weaker towards the ends) lowers the side lobes but widens the main lobe."),
                t("Nebenkeulen erzeugen Falschechos bei starken, nahen Zielen.", "Side lobes cause false echoes from strong, close targets."),
                t("Vertikal ist die Keule breit (≈ 20–25°), damit Rollen und Stampfen das Bild nicht verlieren.", "Vertically the beam is wide (≈ 20–25°) so that rolling and pitching do not lose the picture."),
            ],
        );
        see_also(
            ui,
            nav,
            &[
                Chapter::BearingRes,
                Chapter::Bands,
                Chapter::FalseEchoes,
                Chapter::Wave,
            ],
        );
    }
}

fn draw_polar(p: &egui::Painter, r: Rect, pal: &Pal, db: &dyn Fn(f64) -> f64, hp: f64) {
    p.rect_filled(r, CornerRadius::same(6), pal.panel);
    p.text(
        r.left_top() + vec2(10.0, 6.0),
        Align2::LEFT_TOP,
        t("Keule polar (0° = voraus)", "Beam polar (0° = ahead)"),
        FontId::proportional(13.0),
        pal.ink,
    );
    let c = pos2(r.center().x, r.bottom() - 18.0);
    let rad = (r.width() / 2.0 - 16.0).min(r.height() - 46.0);
    for db_ring in [0.0, -10.0, -20.0, -30.0] {
        let rr = rad * ((db_ring + 40.0) / 40.0) as f32;
        let pts: Vec<_> = (0..=90)
            .map(|i| {
                let a = (-90.0 + i as f32 * 2.0).to_radians();
                c + vec2(a.sin(), -a.cos()) * rr
            })
            .collect();
        p.add(Shape::line(pts, Stroke::new(1.0_f32, pal.grid)));
        p.text(
            c + vec2(4.0, -rr),
            Align2::LEFT_BOTTOM,
            format!("{db_ring:.0}"),
            FontId::proportional(10.0),
            pal.faint,
        );
    }
    for a in (-90..=90).step_by(30) {
        let ar = (a as f32).to_radians();
        p.line_segment(
            [c, c + vec2(ar.sin(), -ar.cos()) * rad],
            Stroke::new(1.0_f32, pal.grid),
        );
    }
    let pts: Vec<_> = (0..=1800)
        .map(|i| {
            let a = -90.0 + i as f64 * 0.1;
            let rr = rad * ((db(a) + 40.0) / 40.0) as f32;
            let ar = (a as f32).to_radians();
            c + vec2(ar.sin(), -ar.cos()) * rr
        })
        .collect();
    p.add(Shape::line(pts, Stroke::new(2.0_f32, pal.accent)));
    let h = (hp / 2.0) as f32;
    for s in [-h, h] {
        let ar = s.to_radians();
        p.line_segment(
            [c, c + vec2(ar.sin(), -ar.cos()) * rad],
            Stroke::new(1.0_f32, pal.warn),
        );
    }
}

fn draw_slots(p: &egui::Painter, r: Rect, pal: &Pal, steer: f64) {
    p.rect_filled(r, CornerRadius::same(6), pal.panel);
    let n = 9;
    let spacing = 26.0f32; // px = λ/2
    let lambda_px = spacing * 2.0;
    let base_y = r.bottom() - 26.0;
    let x0 = r.center().x - spacing * (n as f32 - 1.0) / 2.0;
    p.rect_filled(
        Rect::from_min_max(
            pos2(x0 - 20.0, base_y),
            pos2(x0 + spacing * (n as f32 - 1.0) + 20.0, base_y + 10.0),
        ),
        CornerRadius::same(2),
        pal.faint,
    );
    let clip = p.with_clip_rect(r.shrink(2.0));
    for i in 0..n {
        let x = x0 + spacing * i as f32;
        p.rect_filled(
            Rect::from_center_size(pos2(x, base_y + 5.0), vec2(6.0, 8.0)),
            CornerRadius::ZERO,
            pal.echo,
        );
        // Verzögerung: späterer Start ⇒ kleinere Radien
        let delay = (steer * i as f64) as f32 * lambda_px;
        for k in 1..8 {
            let rr = k as f32 * lambda_px - delay - lambda_px * 0.5 + 40.0;
            if rr <= 2.0 {
                continue;
            }
            let pts: Vec<_> = (0..=60)
                .map(|j| {
                    let a = std::f32::consts::PI * (j as f32 / 60.0);
                    pos2(x - rr * a.cos(), base_y - rr * a.sin())
                })
                .collect();
            clip.add(Shape::line(
                pts,
                Stroke::new(1.0_f32, pal.accent.gamma_multiply(0.45)),
            ));
        }
    }
    // Hauptrichtung
    let ang = (steer * lambda_px as f64 / spacing as f64)
        .clamp(-1.0, 1.0)
        .asin();
    let c = pos2(r.center().x, base_y);
    let tip = c + vec2(ang.sin() as f32, -(ang.cos() as f32)) * (r.height() - 50.0);
    super::arrow(&p, c, tip - c, Stroke::new(2.5_f32, pal.warn));
    p.text(
        r.left_top() + vec2(10.0, 6.0),
        Align2::LEFT_TOP,
        &tf!("Hauptkeule {:+.0}°", "main beam {:+.0}°", ang.to_degrees()),
        FontId::proportional(13.0),
        pal.ink,
    );
}
