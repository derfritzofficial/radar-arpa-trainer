//! Kapitel 7: Mehrfachechos, indirekte Echos, Nebenkeulenechos, Echos zweiter Ablenkung.

use eframe::egui::{self, vec2, Align2, Color32, FontId, Pos2, RichText, Sense, Shape, Stroke};

use super::{merke, pal, see_also, slider, sub, text, try_it, Chapter, Pal};
use crate::i18n::t;
use crate::util::Rng;

const RANGES: [f64; 5] = [0.75, 1.5, 3.0, 6.0, 12.0];

pub struct State {
    multiple: bool,
    indirect: bool,
    sidelobe: bool,
    second: bool,
    rays: bool,
    r_tgt: f64,
    brg_tgt: f64,
    obst_brg: f64,
    prf: f64,
    far_r: f64,
    far_brg: f64,
    jitter: bool,
    range_idx: usize,
    sweep: f64,
    jitter_off: f64,
    rng: Rng,
}

impl Default for State {
    fn default() -> Self {
        Self {
            multiple: true,
            indirect: false,
            sidelobe: false,
            second: false,
            rays: true,
            r_tgt: 0.3,
            brg_tgt: 90.0,
            obst_brg: 200.0,
            prf: 3000.0,
            far_r: 28.2,
            far_brg: 320.0,
            jitter: false,
            range_idx: 1,
            sweep: 0.0,
            jitter_off: 0.0,
            rng: Rng::new(5),
        }
    }
}

struct Echo {
    brg: f64,
    r: f64,
    amp: f32,
    width_deg: f64,
    len: f64,
    col: Color32,
}

impl State {
    fn rua(&self) -> f64 {
        3.0e8 / (2.0 * self.prf * (1.0 + self.jitter_off)) / 1852.0
    }

    fn echoes(&self) -> Vec<Echo> {
        let yellow = Color32::from_rgb(255, 230, 110);
        let mut v = vec![Echo {
            brg: self.brg_tgt,
            r: self.r_tgt,
            amp: 1.0,
            width_deg: 3.0,
            len: 0.04,
            col: yellow,
        }];
        let abeam = (self.brg_tgt.to_radians().sin()).abs();
        if self.multiple {
            let k = if abeam > 0.9 { 1.0 } else { 0.35 };
            v.push(Echo {
                brg: self.brg_tgt,
                r: 2.0 * self.r_tgt,
                amp: 0.65 * k,
                width_deg: 3.0,
                len: 0.04,
                col: yellow,
            });
            v.push(Echo {
                brg: self.brg_tgt,
                r: 3.0 * self.r_tgt,
                amp: 0.38 * k,
                width_deg: 3.0,
                len: 0.04,
                col: yellow,
            });
        }
        if self.indirect {
            v.push(Echo {
                brg: self.obst_brg,
                r: self.r_tgt + 0.02,
                amp: 0.6,
                width_deg: 4.0,
                len: 0.05,
                col: yellow,
            });
        }
        if self.sidelobe {
            for k in 1..=7 {
                for s in [-1.0, 1.0] {
                    let off = s * (2.5 + 3.0 * k as f64);
                    v.push(Echo {
                        brg: self.brg_tgt + off,
                        r: self.r_tgt,
                        amp: (0.55 - 0.06 * k as f32).max(0.12),
                        width_deg: 1.8,
                        len: 0.025,
                        col: yellow,
                    });
                }
            }
        }
        if self.second {
            let rua = self.rua();
            if self.far_r > rua {
                let app = self.far_r - (self.far_r / rua).floor() * rua;
                v.push(Echo {
                    brg: self.far_brg,
                    r: app,
                    amp: 0.85,
                    width_deg: 5.0,
                    len: 0.12,
                    col: yellow,
                });
            }
        }
        v
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Nicht jedes Echo auf dem Schirm ist ein Ziel. Die Energie kann mehrfach zwischen Schiffen hin und her laufen, Umwege über eigene Aufbauten nehmen, über die Nebenkeulen empfangen werden oder erst nach dem nächsten Sendepuls zurückkommen. Schalten Sie die Effekte einzeln zu und beobachten Sie, wo die Falschechos erscheinen.",
                "Not every echo on the screen is a target. The energy can bounce back and forth between ships, take a detour via own superstructures, be received through the side lobes, or only return after the next pulse. Switch on the effects one by one and watch where the false echoes appear.",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.multiple, t("Mehrfachechos", "multiple echoes"));
            ui.checkbox(
                &mut self.indirect,
                t("indirektes Echo (Schornstein)", "indirect echo (funnel)"),
            );
            ui.checkbox(
                &mut self.sidelobe,
                t("Nebenkeulenechos", "side-lobe echoes"),
            );
            ui.checkbox(
                &mut self.second,
                t("Echo zweiter Ablenkung", "second-trace echo"),
            );
            ui.checkbox(&mut self.rays, t("Strahlwege zeigen", "show ray paths"));
        });
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.r_tgt,
                0.1..=0.7,
                t("Abstand großes Schiff", "range of large ship"),
                t(" sm", " nm"),
                0.01,
            );
            slider(
                ui,
                &mut self.brg_tgt,
                0.0..=359.0,
                t("Seitenpeilung", "relative bearing"),
                "°",
                1.0,
            );
            slider(
                ui,
                &mut self.obst_brg,
                120.0..=240.0,
                t("Schornstein (Seitenpeilung)", "funnel (relative bearing)"),
                "°",
                1.0,
            );
        });
        if self.second {
            ui.horizontal_wrapped(|ui| {
                slider(ui, &mut self.prf, 1000.0..=4000.0, "PRF", " Hz", 10.0);
                slider(
                    ui,
                    &mut self.far_r,
                    10.0..=60.0,
                    t("fernes Ziel (Überreichweite)", "distant target (ducting)"),
                    t(" sm", " nm"),
                    0.1,
                );
                ui.checkbox(
                    &mut self.jitter,
                    t("PRF-Staffelung (Jitter)", "staggered PRF (jitter)"),
                );
            });
            ui.label(RichText::new(tf!(
                "eindeutige Reichweite c/(2·PRF) = {:.1} sm   ·   fernes Ziel erscheint bei {:.2} sm",
                "unambiguous range c/(2·PRF) = {:.1} nm   ·   distant target appears at {:.2} nm",
                self.rua(),
                if self.far_r > self.rua() { self.far_r % self.rua() } else { self.far_r }
            )).monospace());
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(t("Bereich:", "Range:"));
            for (i, r) in RANGES.iter().enumerate() {
                if ui
                    .selectable_label(self.range_idx == i, format!("{r} {}", t("sm", "nm")))
                    .clicked()
                {
                    self.range_idx = i;
                }
            }
        });

        // Antennendrehung und PRF-Jitter je Umlauf
        let dt = ui.input(|i| i.stable_dt).min(0.1) as f64;
        let old = self.sweep;
        self.sweep = (self.sweep + dt * 360.0 / 2.5) % 360.0;
        if self.sweep < old {
            self.jitter_off = if self.jitter {
                self.rng.range(-0.06, 0.06)
            } else {
                0.0
            };
        }
        ui.ctx().request_repaint();

        let avail = ui.available_width();
        let size = avail.min(560.0);
        let (rect, _) = ui.allocate_exact_size(vec2(avail, size), Sense::hover());
        let painter = ui.painter_at(rect);
        let c = rect.center();
        let rad = size / 2.0 - 24.0;
        self.draw_ppi(&painter, c, rad, &p);

        sub(
            ui,
            t(
                "So erkennen Sie die Falschechos",
                "How to recognise the false echoes",
            ),
        );
        egui::Grid::new("false_tab").striped(true).num_columns(3).spacing([18.0, 6.0]).show(ui, |ui| {
            for h in [t("Art", "type"), t("Entstehung", "cause"), t("Erkennungsmerkmal", "how to recognise")] {
                ui.label(RichText::new(h).strong());
            }
            ui.end_row();
            let rows: [(&str, &str, &str); 4] = [
                (
                    t("Mehrfachechos", "multiple echoes"),
                    t("Energie läuft zwischen Eigenschiff und nahem, großem Schiff (meist querab) mehrfach hin und her.", "energy bounces between own ship and a close, large ship (usually abeam)."),
                    t("gleiche Peilung, 2-, 3-fache Entfernung, nach außen schwächer; wandern mit dem echten Echo.", "same bearing, 2×, 3× the range, weaker outwards; move with the real echo."),
                ),
                (
                    t("indirekte Echos", "indirect echoes"),
                    t("Umweg über Mast, Schornstein oder Ladung des Eigenschiffs (oder ein anderes Objekt).", "detour via mast, funnel or cargo of own ship (or another object)."),
                    t("richtige Entfernung, aber auf der Peilung des Hindernisses – oft im Schattensektor.", "correct range, but on the bearing of the obstruction – often in the shadow sector."),
                ),
                (
                    t("Nebenkeulenechos", "side-lobe echoes"),
                    t("starkes, nahes Ziel wird auch über die Nebenkeulen der Antenne empfangen.", "a strong, close target is also received through the antenna's side lobes."),
                    t("Bogen bzw. Punktreihe in gleicher Entfernung beiderseits des echten Echos; weniger Gain/mehr STC hilft.", "arc or row of dots at the same range on both sides of the real echo; less gain/more STC helps."),
                ),
                (
                    t("zweite Ablenkung", "second trace"),
                    t("Echo eines sehr fernen Ziels (Überreichweite) kommt erst nach dem nächsten Sendepuls zurück.", "echo of a very distant target (ducting) returns only after the next pulse."),
                    t("falsche, zu kleine Entfernung, verzerrte Form; springt oder verschmiert bei Bereichs- bzw. PRF-Wechsel.", "wrong, too short range, distorted shape; jumps or smears when changing range scale or PRF."),
                ),
            ];
            for (a, b, c2) in rows {
                ui.label(RichText::new(a).strong());
                ui.label(b);
                ui.label(c2);
                ui.end_row();
            }
        });
        try_it(
            ui,
            t(
                "Ein Schiff, drei Echos: Lassen Sie nur die Mehrfachechos an – ein echtes und zwei falsche Echos auf gleicher Peilung. Drehen Sie das Schiff aus der Querab-Position: Die Mehrfachechos werden schwächer. Schalten Sie stattdessen das indirekte Echo ein: Das dritte Echo liegt jetzt achteraus in der richtigen Entfernung.",
                "One ship, three echoes: keep only the multiple echoes on – one real and two false echoes on the same bearing. Move the ship away from abeam: the multiple echoes weaken. Switch on the indirect echo instead: the third echo now lies aft at the correct range.",
            ),
        );
        merke(
            ui,
            &[
                t("Mehrfachechos: gleiche Peilung, gleiche Abstände, nach außen schwächer.", "Multiple echoes: same bearing, equal spacing, weaker outwards."),
                t("Indirekte Echos: richtige Entfernung, falsche Peilung (Richtung des Hindernisses).", "Indirect echoes: correct range, wrong bearing (direction of the obstruction)."),
                t("Nebenkeulenechos: gleiche Entfernung, Bogen um das echte Echo – Gain/STC reduzieren.", "Side-lobe echoes: same range, arc around the real echo – reduce gain/STC."),
                t("Zweite Ablenkung: Entfernung R − c/(2·PRF); mit PRF- oder Bereichswechsel entlarven.", "Second trace: range R − c/(2·PRF); expose by changing PRF or range scale."),
            ],
        );
        see_also(
            ui,
            nav,
            &[
                Chapter::Antenna,
                Chapter::RangeRes,
                Chapter::Range,
                Chapter::Signal,
            ],
        );
    }

    fn draw_ppi(&self, p: &egui::Painter, c: Pos2, rad: f32, pal: &Pal) {
        let range = RANGES[self.range_idx];
        let scope = Color32::from_rgb(26, 62, 128);
        p.circle_filled(c, rad + 10.0, Color32::from_rgb(22, 40, 70));
        p.circle_filled(c, rad, scope);
        let ring = range / 3.0;
        for k in 1..=3 {
            p.circle_stroke(
                c,
                rad * (k as f64 * ring / range) as f32,
                Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(160, 200, 240, 90)),
            );
        }
        p.circle_stroke(
            c,
            rad,
            Stroke::new(1.5_f32, Color32::from_rgb(160, 200, 240)),
        );
        let ppn = rad / range as f32;
        let dir = |b: f64| {
            let a = b.to_radians() as f32;
            vec2(a.sin(), -a.cos())
        };
        let at = |b: f64, r: f64| c + dir(b) * (r as f32 * ppn);
        // Schattensektor hinter dem Schornstein
        let shadow: Vec<Pos2> = std::iter::once(c)
            .chain((0..=10).map(|i| c + dir(self.obst_brg - 4.0 + 0.8 * i as f64) * rad))
            .collect();
        p.add(Shape::convex_polygon(
            shadow,
            Color32::from_rgba_unmultiplied(0, 0, 0, 70),
            Stroke::NONE,
        ));
        // Echos mit Nachleuchten
        for e in self.echoes() {
            if e.r > range {
                continue;
            }
            let age = (self.sweep - e.brg).rem_euclid(360.0) / 360.0 * 2.5;
            let glow = (0.25 + 0.75 * (-age / 2.2).exp()) as f32 * e.amp;
            let n = 8;
            for i in 0..=n {
                let b = e.brg - e.width_deg / 2.0 + e.width_deg * i as f64 / n as f64;
                let a = at(b, e.r - e.len * range / 2.0);
                let z = at(b, e.r + e.len * range / 2.0);
                let col = Color32::from_rgba_unmultiplied(
                    e.col.r(),
                    e.col.g(),
                    e.col.b(),
                    (glow * 255.0).clamp(0.0, 255.0) as u8,
                );
                p.line_segment(
                    [a, z],
                    Stroke::new(
                        (rad * e.width_deg as f32 / 57.3 * e.r as f32 / range as f32 / n as f32
                            * 1.6)
                            .max(2.0),
                        col,
                    ),
                );
            }
        }
        // Strahlwege
        if self.rays {
            let st = Stroke::new(1.2_f32, pal.cur.gamma_multiply(0.9));
            let tgt = at(self.brg_tgt, self.r_tgt);
            if self.multiple {
                let n = dir(self.brg_tgt + 90.0) * 6.0;
                let pts = vec![
                    c,
                    tgt + n,
                    c + n * 2.0,
                    tgt + n * 3.0,
                    c + n * 4.0,
                    tgt + n * 5.0,
                    c + n * 6.0,
                ];
                p.add(Shape::line(pts, st));
            }
            if self.indirect {
                let funnel = at(self.obst_brg, range * 0.035);
                p.add(Shape::line(vec![c, funnel, tgt], st));
                p.extend(Shape::dashed_line(
                    &[c, at(self.obst_brg, self.r_tgt)],
                    Stroke::new(1.0_f32, pal.warn),
                    4.0,
                    4.0,
                ));
            }
        }
        // Sweep-Linie, Eigenschiff, Ziele
        p.line_segment(
            [c, c + dir(self.sweep) * rad],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 240, 160, 110)),
        );
        p.line_segment(
            [c, c + dir(0.0) * rad],
            Stroke::new(1.3_f32, Color32::WHITE),
        );
        let l = (ppn * 0.1).clamp(10.0, 26.0);
        let hull = vec![
            c + vec2(0.0, -l),
            c + vec2(l * 0.3, -l * 0.4),
            c + vec2(l * 0.3, l),
            c + vec2(-l * 0.3, l),
            c + vec2(-l * 0.3, -l * 0.4),
        ];
        p.add(Shape::closed_line(
            hull,
            Stroke::new(1.5_f32, Color32::WHITE),
        ));
        p.circle_filled(c + dir(self.obst_brg) * (l * 0.55), 3.5, pal.cur);
        let tgt = at(self.brg_tgt, self.r_tgt);
        p.circle_stroke(
            tgt,
            10.0,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 120)),
        );
        p.text(
            tgt + vec2(12.0, -12.0),
            Align2::LEFT_BOTTOM,
            t("großes Schiff", "large ship"),
            FontId::proportional(11.0),
            Color32::WHITE,
        );
        if self.second && self.far_r > range {
            let edge = c + dir(self.far_brg) * (rad + 2.0);
            p.text(
                edge,
                Align2::CENTER_BOTTOM,
                &tf!(
                    "wahres fernes Ziel: {:.1} sm",
                    "true distant target: {:.1} nm",
                    self.far_r
                ),
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
        p.text(
            c + vec2(-rad, -rad),
            Align2::LEFT_TOP,
            &tf!("HEAD UP  ·  {} sm", "HEAD UP  ·  {} nm", range),
            FontId::monospace(12.0),
            Color32::WHITE,
        );
    }
}
