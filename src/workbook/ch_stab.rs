//! Kapitel 10: See- und Grundstabilisierung – was wahre Vektoren bei Strom zeigen.

use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, RichText, Sense, Stroke};

use super::{formula, merke, pal, see_also, slider, sub, text, try_it, Chapter, Pal};
use crate::i18n::t;
use crate::util::{norm180, V2};

pub struct State {
    hdg: f64,
    stw: f64,
    set: f64,
    drift: f64,
    vec_min: f64,
    sea: bool,
    tgt_hdg: f64,
    tgt_stw: f64,
    show_rel: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            hdg: 0.0,
            stw: 10.0,
            set: 90.0,
            drift: 2.5,
            vec_min: 30.0,
            sea: false,
            tgt_hdg: 270.0,
            tgt_stw: 8.0,
            show_rel: false,
        }
    }
}

const BUOY: V2 = V2::new(1.6, 2.8);
const SHIP: V2 = V2::new(3.4, 1.4);
const RANGE: f64 = 6.0;

impl State {
    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Ein Schiff fährt durchs Wasser (Vorausrichtung HDG, Fahrt durchs Wasser STW) – und das Wasser selbst bewegt sich mit dem Strom. Über Grund ergibt sich daraus Kurs und Fahrt über Grund (COG, SOG). Ein „wahrer“ Vektor kann sich deshalb auf zwei Bezugssysteme beziehen: auf den Meeresgrund (Grundstabilisierung, Sensor GNSS) oder auf das Wasser (Seestabilisierung, Sensor Fahrtlog).",
                "A ship moves through the water (heading HDG, speed through water STW) – and the water itself moves with the current. Over the ground this gives course and speed over ground (COG, SOG). A “true” vector can therefore refer to two frames: the sea bed (ground stabilisation, sensor GNSS) or the water (sea stabilisation, sensor speed log).",
            ),
        );
        formula(
            ui,
            t(
                "Vektor über Grund = Vektor durchs Wasser + Strom",
                "vector over ground = vector through water + current",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(t("Stabilisierung:", "Stabilisation:")).strong());
            ui.selectable_value(&mut self.sea, false, t("Grund (GNSS)", "ground (GNSS)"));
            ui.selectable_value(&mut self.sea, true, t("See (Fahrtlog)", "sea (speed log)"));
            ui.checkbox(
                &mut self.show_rel,
                t("relative Vektoren zusätzlich", "relative vectors as well"),
            );
            slider(
                ui,
                &mut self.vec_min,
                6.0..=60.0,
                t("Vektorlänge", "vector length"),
                " min",
                1.0,
            );
        });
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.set,
                0.0..=359.0,
                t("Strom nach", "current setting"),
                "°",
                1.0,
            );
            slider(
                ui,
                &mut self.drift,
                0.0..=5.0,
                t("Stromstärke", "drift"),
                " kn",
                0.1,
            );
        });
        ui.horizontal_wrapped(|ui| {
            slider(ui, &mut self.hdg, 0.0..=359.0, "HDG", "°", 1.0);
            slider(ui, &mut self.stw, 0.0..=20.0, "STW", " kn", 0.5);
            slider(
                ui,
                &mut self.tgt_hdg,
                0.0..=359.0,
                t("Kreuzer HDG", "crossing ship HDG"),
                "°",
                1.0,
            );
            slider(
                ui,
                &mut self.tgt_stw,
                0.0..=20.0,
                t("Kreuzer STW", "crossing ship STW"),
                " kn",
                0.5,
            );
        });

        let vc = V2::from_bearing(self.set, self.drift);
        let vw = V2::from_bearing(self.hdg, self.stw);
        let vg = vw + vc;
        let tw = V2::from_bearing(self.tgt_hdg, self.tgt_stw);
        let tg = tw + vc;
        let drift_angle = if vg.len() > 0.05 {
            norm180(vg.bearing() - self.hdg)
        } else {
            0.0
        };
        ui.label(
            RichText::new(tf!(
                "Eigenschiff: HDG {:05.1}° / STW {:.1} kn   →   COG {:05.1}° / SOG {:.1} kn   (Driftwinkel {:+.1}°)",
                "Own ship: HDG {:05.1}° / STW {:.1} kn   →   COG {:05.1}° / SOG {:.1} kn   (drift angle {:+.1}°)",
                self.hdg,
                self.stw,
                vg.bearing(),
                vg.len(),
                drift_angle
            ))
            .monospace()
            .strong(),
        );
        let (buoy_v, ship_v, own_v) = if self.sea {
            (-vc, tw, vw)
        } else {
            (V2::ZERO, tg, vg)
        };
        ui.label(
            RichText::new(tf!(
                "Wahrer Vektor der Tonne: {:.1} kn nach {:05.1}°   ·   Kreuzer: {:.1} kn nach {:05.1}°",
                "True vector of the buoy: {:.1} kn towards {:05.1}°   ·   crossing ship: {:.1} kn towards {:05.1}°",
                buoy_v.len(),
                if buoy_v.len() > 0.01 { buoy_v.bearing() } else { 0.0 },
                ship_v.len(),
                ship_v.bearing()
            ))
            .monospace()
            .color(if self.sea && self.drift > 0.05 { p.warn } else { p.ink }),
        );

        let avail = ui.available_width();
        let size = (avail - 250.0).clamp(260.0, 560.0);
        let (rect, _) = ui.allocate_exact_size(vec2(avail, size), Sense::hover());
        let painter = ui.painter_at(rect);
        let c = pos2(rect.left() + size / 2.0, rect.center().y);
        let rad = size / 2.0 - 16.0;
        let ppn = rad / RANGE as f32;
        let tm = self.vec_min / 60.0;
        let w2s = |w: V2| c + vec2(w.x as f32, -w.y as f32) * ppn;
        painter.circle_filled(c, rad + 6.0, Color32::from_rgb(22, 40, 70));
        painter.circle_filled(c, rad, Color32::from_rgb(26, 62, 128));
        for k in 1..=3 {
            painter.circle_stroke(
                c,
                rad * k as f32 / 3.0,
                Stroke::new(0.7_f32, Color32::from_rgba_unmultiplied(160, 200, 240, 70)),
            );
        }
        let arrow = |from: V2, v: V2, col: Color32, w: f32| {
            let a = w2s(from);
            let b = w2s(from + v * tm);
            if (b - a).length() > 1.0 {
                super::arrow(&painter, a, b - a, Stroke::new(w, col));
            } else {
                painter.circle_filled(a, 3.0, col);
            }
        };
        // Echos
        for (pos, name) in [
            (BUOY, t("Tonne (fest)", "buoy (fixed)")),
            (SHIP, t("Kreuzer", "crossing ship")),
        ] {
            let s = w2s(pos);
            painter.circle_filled(s, 5.0, Color32::from_rgb(255, 230, 110));
            painter.text(
                s + vec2(8.0, 8.0),
                Align2::LEFT_TOP,
                name,
                FontId::proportional(12.0),
                Color32::WHITE,
            );
        }
        // wahre Vektoren
        arrow(BUOY, buoy_v, p.rel, 2.5);
        arrow(SHIP, ship_v, p.rel, 2.5);
        // Eigenschiff: Dreieck Wasser + Strom = Grund
        arrow(
            V2::ZERO,
            vw,
            Color32::from_rgb(120, 200, 255),
            if self.sea { 3.0 } else { 1.5 },
        );
        let _ = own_v;
        let a = w2s(V2::ZERO);
        let b = w2s(vg * tm);
        super::arrow(
            &painter,
            a,
            b - a,
            Stroke::new(if self.sea { 1.5_f32 } else { 3.0 }, Color32::WHITE),
        );
        // Strom am Vektorende (Dreieck schließen)
        let wend = w2s(vw * tm);
        super::arrow(&painter, wend, b - wend, Stroke::new(1.6_f32, p.cur));
        if self.show_rel {
            arrow(SHIP, tg - vg, p.tgt, 1.6);
            arrow(BUOY, -vg, p.tgt, 1.6);
        }
        painter.circle_stroke(c, 5.0, Stroke::new(1.5_f32, Color32::WHITE));
        legend(
            &painter,
            pos2(rect.left() + size + 16.0, rect.top() + 10.0),
            &p,
            self.sea,
            self.show_rel,
        );

        try_it(
            ui,
            t(
                "Schalten Sie auf Seestabilisierung: Die feste Tonne bekommt einen Vektor genau entgegen der Stromrichtung, so lang wie der Strom stark ist – so lässt sich der Strom an festen Objekten ablesen. Der Kreuzer zeigt jetzt seine Vorausrichtung, also seinen Aspekt. Die relativen Vektoren ändern sich beim Umschalten nicht.",
                "Switch to sea stabilisation: the fixed buoy gets a vector exactly opposite to the current, as long as the current is strong – this is how the current can be read from fixed objects. The crossing ship now shows its heading, i.e. its aspect. The relative vectors do not change when switching.",
            ),
        );
        sub(
            ui,
            t(
                "Welche Stabilisierung wofür?",
                "Which stabilisation for what?",
            ),
        );
        egui::Grid::new("stab_tab").striped(true).num_columns(2).spacing([18.0, 6.0]).show(ui, |ui| {
            ui.label(RichText::new(t("Grund (GNSS / Bodenlog)", "ground (GNSS / bottom-track log)")).strong());
            ui.label(t("Navigation, Kartenvergleich, Ankerwache, Parallel Indexing, Landfall; feste Objekte zeigen keinen Vektor.", "navigation, chart comparison, anchor watch, parallel indexing, landfall; fixed objects show no vector."));
            ui.end_row();
            ui.label(RichText::new(t("See (Wasserlog)", "sea (water-track log)")).strong());
            ui.label(t("Kollisionsverhütung: Der wahre Vektor zeigt Vorausrichtung und Aspekt der anderen (KVR-Beurteilung); Strom an festen Objekten ablesbar.", "collision avoidance: the true vector shows the others' heading and aspect (COLREG assessment); current readable at fixed objects."));
            ui.end_row();
            ui.label(RichText::new(t("relative Vektoren", "relative vectors")).strong());
            ui.label(t("unabhängig von der Stabilisierung – CPA/TCPA bleiben gleich.", "independent of the stabilisation – CPA/TCPA stay the same."));
            ui.end_row();
        });
        merke(
            ui,
            &[
                t("Grundstabilisiert: wahre Vektoren über Grund (COG/SOG); feste Objekte ohne Vektor.", "Ground-stabilised: true vectors over ground (COG/SOG); fixed objects without vector."),
                t("Seestabilisiert: wahre Vektoren durchs Wasser (HDG/STW); feste Objekte zeigen den Strom rückwärts.", "Sea-stabilised: true vectors through the water (HDG/STW); fixed objects show the current reversed."),
                t("Strom an festen Objekten ermitteln: wahre Vektoren + Seestabilisierung.", "Determine the current at fixed objects: true vectors + sea stabilisation."),
                t("Für die KVR-Beurteilung zählt der Aspekt – also die Bewegung durchs Wasser.", "For the COLREG assessment the aspect counts – i.e. the motion through the water."),
            ],
        );
        see_also(
            ui,
            nav,
            &[Chapter::RelVector, Chapter::Display, Chapter::Plotting],
        );
    }
}

fn legend(p: &egui::Painter, at: Pos2, pal: &Pal, sea: bool, rel: bool) {
    let rows: Vec<(Color32, &str)> = vec![
        (
            pal.rel,
            if sea {
                t(
                    "wahrer Vektor (durchs Wasser)",
                    "true vector (through water)",
                )
            } else {
                t("wahrer Vektor (über Grund)", "true vector (over ground)")
            },
        ),
        (
            Color32::from_rgb(120, 200, 255),
            t("Eigenschiff: HDG / STW", "own ship: HDG / STW"),
        ),
        (pal.cur, t("Strom", "current")),
        (
            Color32::WHITE,
            t("Eigenschiff: COG / SOG", "own ship: COG / SOG"),
        ),
    ];
    let all: Vec<(Color32, &str)> = rows
        .into_iter()
        .chain(if rel {
            vec![(pal.tgt, t("relativer Vektor", "relative vector"))]
        } else {
            vec![]
        })
        .collect();
    // Legende auf Schirmfarbe, damit die Linienfarben wie auf dem Schirm wirken
    let bg = egui::Rect::from_min_size(
        at - vec2(8.0, 6.0),
        vec2(230.0, 20.0 * all.len() as f32 + 10.0),
    );
    p.rect_filled(
        bg,
        egui::CornerRadius::same(6),
        Color32::from_rgb(26, 62, 128),
    );
    let mut y = at.y;
    for (c, s) in all {
        p.line_segment(
            [pos2(at.x, y + 7.0), pos2(at.x + 26.0, y + 7.0)],
            Stroke::new(3.0_f32, c),
        );
        p.text(
            pos2(at.x + 34.0, y),
            Align2::LEFT_TOP,
            s,
            FontId::proportional(12.0),
            Color32::WHITE,
        );
        y += 20.0;
    }
}
