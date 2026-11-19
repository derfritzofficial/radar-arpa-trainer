//! Kapitel 11: Verhalten des Relativvektors bei eigener Kursänderung,
//! Vektordreieck Vb = Va + Vbr, stehende Peilung in RM und TM.

use std::collections::VecDeque;

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Pos2, Rect, RichText, Sense, Stroke,
};

use super::{formula, merke, pal, see_also, slider, sub, text, try_it, Chapter, Pal};
use crate::i18n::t;
use crate::sim::course_for_cpa;
use crate::util::{cpa_calc, norm180, norm360, V2};

const RANGE: f64 = 8.0;

pub struct State {
    own_pos: V2,
    hdg: f64,
    cmd: f64,
    spd: f64,
    tgt_pos: V2,
    tgt_crs: f64,
    tgt_spd: f64,
    start_brg: f64,
    start_rng: f64,
    delta: f64,
    rot: f64,
    t: f64,
    warp: f64,
    playing: bool,
    tm: bool,
    tm_origin: V2,
    vec_min: f64,
    ebl: bool,
    hist: VecDeque<(V2, V2)>,
    last_hist: f64,
}

impl Default for State {
    fn default() -> Self {
        let mut s = Self {
            own_pos: V2::ZERO,
            hdg: 0.0,
            cmd: 0.0,
            spd: 12.0,
            tgt_pos: V2::ZERO,
            tgt_crs: 290.0,
            tgt_spd: 12.0,
            start_brg: 50.0,
            start_rng: 7.0,
            delta: 60.0,
            rot: 30.0,
            t: 0.0,
            warp: 10.0,
            playing: false,
            tm: false,
            tm_origin: V2::ZERO,
            vec_min: 12.0,
            ebl: true,
            hist: VecDeque::new(),
            last_hist: -1e9,
        };
        s.reset_positions();
        s
    }
}

impl State {
    fn reset_positions(&mut self) {
        self.own_pos = V2::ZERO;
        self.hdg = norm360(self.cmd);
        self.tgt_pos = V2::from_bearing(self.start_brg, self.start_rng);
        self.t = 0.0;
        self.hist.clear();
        self.last_hist = -1e9;
        self.tm_origin = V2::from_bearing(self.hdg, RANGE * 0.35);
    }

    fn own_vel(&self) -> V2 {
        V2::from_bearing(self.hdg, self.spd)
    }

    fn tgt_vel(&self) -> V2 {
        V2::from_bearing(self.tgt_crs, self.tgt_spd)
    }

    fn step(&mut self, dt: f64) {
        self.t += dt;
        let err = norm180(self.cmd - self.hdg);
        let max = self.rot / 60.0 * dt;
        self.hdg = norm360(self.hdg + err.clamp(-max, max));
        self.own_pos += self.own_vel() * (dt / 3600.0);
        self.tgt_pos += self.tgt_vel() * (dt / 3600.0);
        if self.t - self.last_hist >= 10.0 {
            self.hist.push_back((self.own_pos, self.tgt_pos));
            self.last_hist = self.t;
            if self.hist.len() > 400 {
                self.hist.pop_front();
            }
        }
        if (self.tgt_pos - self.own_pos).len() > RANGE * 1.4 {
            self.playing = false;
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Der relative Vektor eines Ziels ist die Differenz aus seinem wahren Vektor und dem eigenen Vektor. Ändert das Eigenschiff Kurs oder Fahrt, ändert sich sofort der eigene Vektor – und damit der relative Vektor jedes Ziels, obwohl die anderen Schiffe gar nichts tun. Der wahre Vektor des Ziels bleibt dagegen unverändert.",
                "The relative vector of a target is the difference between its true vector and own ship's vector. If own ship changes course or speed, own vector changes immediately – and with it the relative vector of every target, although the other ships do nothing. The target's true vector stays unchanged.",
            ),
        );
        formula(
            ui,
            t(
                "Vb = Va + Vbr      ⇔      Vbr = Vb − Va",
                "Vb = Va + Vbr      ⇔      Vbr = Vb − Va",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if self.playing {
                    t("⏸ Pause", "⏸ Pause")
                } else {
                    t("▶ Laufen lassen", "▶ Run")
                })
                .clicked()
            {
                self.playing = !self.playing;
            }
            if ui.button(t("⟲ Zurücksetzen", "⟲ Reset")).clicked() {
                self.cmd = 0.0;
                self.reset_positions();
                self.playing = false;
            }
            ui.label(t("Zeitraffer:", "Time warp:"));
            for w in [1.0, 5.0, 10.0, 30.0] {
                if ui
                    .selectable_label(self.warp == w, format!("{w}×"))
                    .clicked()
                {
                    self.warp = w;
                }
            }
            ui.separator();
            ui.radio_value(&mut self.tm, false, "RM");
            if ui.radio(self.tm, "TM").clicked() && !self.tm {
                self.tm = true;
                self.tm_origin = self.own_pos + V2::from_bearing(self.hdg, RANGE * 0.35);
            }
            ui.checkbox(&mut self.ebl, t("EBL auf das Ziel", "EBL on target"));
            slider(
                ui,
                &mut self.vec_min,
                3.0..=30.0,
                t("Vektoren", "vectors"),
                " min",
                1.0,
            );
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(t("Ziel:", "Target:")).strong());
            let mut changed = slider(
                ui,
                &mut self.start_brg,
                0.0..=359.0,
                t("Peilung", "bearing"),
                "°",
                1.0,
            );
            changed |= slider(
                ui,
                &mut self.start_rng,
                2.0..=8.0,
                t("Distanz", "range"),
                t(" sm", " nm"),
                0.1,
            );
            slider(
                ui,
                &mut self.tgt_crs,
                0.0..=359.0,
                t("Kurs", "course"),
                "°",
                1.0,
            );
            slider(
                ui,
                &mut self.tgt_spd,
                0.0..=25.0,
                t("Fahrt", "speed"),
                " kn",
                0.5,
            );
            if changed && self.t == 0.0 {
                self.reset_positions();
            }
            if ui.button(t("Kollisionskurs", "collision course")).clicked() {
                if let Some(c) = course_for_cpa(
                    self.own_pos,
                    self.own_vel(),
                    self.tgt_pos,
                    self.tgt_spd,
                    0.0,
                ) {
                    self.tgt_crs = c.round();
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(t("Eigenschiff:", "Own ship:")).strong());
            slider(
                ui,
                &mut self.spd,
                0.0..=25.0,
                t("Fahrt", "speed"),
                " kn",
                0.5,
            );
            slider(
                ui,
                &mut self.delta,
                -90.0..=90.0,
                t("Kursänderung", "course change"),
                "°",
                5.0,
            );
            slider(
                ui,
                &mut self.rot,
                5.0..=120.0,
                t("Drehrate", "rate of turn"),
                " °/min",
                5.0,
            );
            if ui
                .button(t("▶ Manöver ausführen", "▶ Execute manoeuvre"))
                .clicked()
            {
                self.cmd = norm360(self.hdg + self.delta);
                self.playing = true;
            }
        });

        if self.playing {
            let mut rem = ui.input(|i| i.stable_dt).min(0.1) as f64 * self.warp;
            while rem > 1e-9 {
                let h = rem.min(1.0);
                self.step(h);
                rem -= h;
            }
            ui.ctx().request_repaint();
        }

        let va = self.own_vel();
        let vb = self.tgt_vel();
        let vbr = vb - va;
        let rel = self.tgt_pos - self.own_pos;
        let (cpa, th) = cpa_calc(rel, vbr);
        ui.label(
            RichText::new(tf!(
                "HDG {:05.1}°  ·  Peilung {:05.1}°  ·  Relativ: KBr {:05.1}° / vBr {:.1} kn  ·  CPA {:.2} sm  ·  TCPA {}",
                "HDG {:05.1}°  ·  bearing {:05.1}°  ·  relative: {:05.1}° / {:.1} kn  ·  CPA {:.2} nm  ·  TCPA {}",
                self.hdg,
                rel.bearing(),
                vbr.bearing(),
                vbr.len(),
                cpa,
                if th.is_finite() { format!("{:.1} min", th * 60.0) } else { "---".into() }
            ))
            .monospace()
            .strong()
            .color(if cpa < 1.0 && th > 0.0 { p.warn } else { p.ink }),
        );

        let avail = ui.available_width();
        let size = (avail * 0.6).clamp(280.0, 540.0);
        let (rect, _) = ui.allocate_exact_size(vec2(avail, size), Sense::hover());
        let painter = ui.painter_at(rect);
        let c = pos2(rect.left() + size / 2.0, rect.center().y);
        self.draw_ppi(&painter, c, size / 2.0 - 14.0, &p);
        let tri = Rect::from_min_max(pos2(rect.left() + size + 12.0, rect.top()), rect.max);
        draw_triangle(&painter, tri, &p, va, vb);

        try_it(
            ui,
            t(
                "Starten Sie das Manöver (60° nach Stb): Während der Drehung schwenkt der blaue Relativvektor, der rote wahre Vektor des Ziels bleibt stehen – und der Relativtrail bekommt einen Knick. Klicken Sie „Kollisionskurs“ und lassen Sie laufen: In RM läuft das Ziel schnurgerade auf die Mitte zu, in TM verschieben sich beide Schiffe, aber die EBL bleibt auf dem Ziel – stehende Peilung.",
                "Start the manoeuvre (60° to starboard): during the turn the blue relative vector swings, the red true vector of the target stays still – and the relative trail gets a bend. Click “collision course” and run: in RM the target moves straight towards the centre, in TM both ships move but the EBL stays on the target – steady bearing.",
            ),
        );
        sub(
            ui,
            t(
                "Was heißt das für die Praxis?",
                "What does this mean in practice?",
            ),
        );
        text(
            ui,
            t(
                "Während und kurz nach dem eigenen Manöver sind relative Vektoren unruhig, und ARPA braucht einige Umläufe, bis die Werte wieder stimmen. Wahre Vektoren bleiben dagegen stabil, weil sich der wahre Kurs des Ziels nicht ändert. Wer die Wirkung eines Manövers vorher prüfen will, nutzt das Trial-Manöver (Simulation) oder plottet es (Kapitel 12).",
                "During and shortly after own manoeuvre relative vectors are unsteady, and ARPA needs a few revolutions until the values are correct again. True vectors stay stable because the target's true course does not change. To check the effect of a manoeuvre beforehand, use the trial manoeuvre (simulation) or plot it (chapter 12).",
            ),
        );
        merke(
            ui,
            &[
                t("Vb = Va + Vbr – der Relativvektor ist die Differenz der wahren Vektoren.", "Vb = Va + Vbr – the relative vector is the difference of the true vectors."),
                t("Eigenes Manöver ⇒ Relativvektoren aller Ziele ändern sich, wahre Vektoren nicht.", "Own manoeuvre ⇒ relative vectors of all targets change, true vectors do not."),
                t("Stehende Peilung bei abnehmender Distanz ⇒ Kollisionskurs (RM: gerade auf die Mitte, TM: EBL bleibt auf dem Ziel).", "Steady bearing with decreasing range ⇒ collision course (RM: straight to the centre, TM: EBL stays on the target)."),
                t("Ziel-Manöver erscheinen erst mit Verzögerung im ARPA-Vektor (Glättung).", "Target manoeuvres appear in the ARPA vector only with a delay (smoothing)."),
            ],
        );
        see_also(
            ui,
            nav,
            &[Chapter::Plotting, Chapter::Stabilisation, Chapter::Display],
        );
    }

    fn draw_ppi(&self, p: &egui::Painter, c: Pos2, rad: f32, pal: &Pal) {
        let cw = if self.tm {
            self.tm_origin
        } else {
            self.own_pos
        };
        let ppn = rad / RANGE as f32;
        let w2s = |w: V2| c + vec2((w - cw).x as f32, -(w - cw).y as f32) * ppn;
        p.circle_filled(c, rad + 6.0, Color32::from_rgb(22, 40, 70));
        p.circle_filled(c, rad, Color32::from_rgb(26, 62, 128));
        let own_s = w2s(self.own_pos);
        for k in 1..=4 {
            p.circle_stroke(
                own_s,
                rad * k as f32 / 4.0,
                Stroke::new(0.7_f32, Color32::from_rgba_unmultiplied(160, 200, 240, 60)),
            );
        }
        let clip = p.with_clip_rect(Rect::from_center_size(c, vec2(rad * 2.0, rad * 2.0)));
        // Trails: relativ in RM, wahr in TM
        for (o, tg) in &self.hist {
            let pos = if self.tm {
                *tg
            } else {
                *tg - *o + self.own_pos
            };
            clip.circle_filled(
                w2s(pos),
                2.0,
                Color32::from_rgba_unmultiplied(255, 230, 120, 140),
            );
            if self.tm {
                clip.circle_filled(
                    w2s(*o),
                    1.5,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 100),
                );
            }
        }
        let tm = self.vec_min / 60.0;
        let tgt_s = w2s(self.tgt_pos);
        if self.ebl {
            let d = (tgt_s - own_s).normalized();
            clip.extend(egui::Shape::dashed_line(
                &[own_s, own_s + d * rad * 2.5],
                Stroke::new(1.2_f32, Color32::from_rgb(90, 255, 170)),
                10.0,
                6.0,
            ));
        }
        let va = self.own_vel();
        let vb = self.tgt_vel();
        let arrow = |from: Pos2, v: V2, col: Color32, w: f32| {
            let d = vec2(v.x as f32, -v.y as f32) * ppn * tm as f32;
            if d.length() > 1.0 {
                super::arrow(&clip, from, d, Stroke::new(w, col));
            }
        };
        arrow(own_s, va, Color32::WHITE, 2.2);
        arrow(tgt_s, vb, pal.tgt, 2.2);
        arrow(tgt_s, vb - va, pal.rel, 2.6);
        // Schiffe
        let hd = (self.hdg.to_radians() as f32, 0.0);
        let dir = vec2(hd.0.sin(), -hd.0.cos());
        let n = vec2(-dir.y, dir.x);
        clip.add(egui::Shape::closed_line(
            vec![
                own_s + dir * 12.0,
                own_s + n * 5.0 - dir * 8.0,
                own_s - n * 5.0 - dir * 8.0,
            ],
            Stroke::new(1.8_f32, Color32::WHITE),
        ));
        clip.circle_filled(tgt_s, 6.0, Color32::from_rgb(255, 230, 110));
        p.text(
            c + vec2(-rad, -rad),
            Align2::LEFT_TOP,
            &format!("NORTH UP  {}  ·  8 NM", if self.tm { "TM" } else { "RM" }),
            FontId::monospace(12.0),
            Color32::WHITE,
        );
    }
}

fn draw_triangle(p: &egui::Painter, r: Rect, pal: &Pal, va: V2, vb: V2) {
    p.rect_filled(r, CornerRadius::same(6), pal.panel);
    p.text(
        r.left_top() + vec2(10.0, 6.0),
        Align2::LEFT_TOP,
        t("Vektordreieck", "Vector triangle"),
        FontId::proportional(14.0),
        pal.ink,
    );
    let maxv = va.len().max(vb.len()).max((vb - va).len()).max(1.0);
    let scale = (r.width().min(r.height()) * 0.36) / maxv as f32;
    let o = r.center() + vec2(0.0, 14.0);
    let s = |v: V2| vec2(v.x as f32, -v.y as f32) * scale;
    super::arrow(&p, o, s(va), Stroke::new(2.5_f32, pal.own));
    super::arrow(&p, o, s(vb), Stroke::new(2.5_f32, pal.tgt));
    super::arrow(&p, o + s(va), s(vb - va), Stroke::new(2.8_f32, pal.rel));
    p.circle_filled(o, 3.0, pal.ink);
    let lab = |at: Pos2, txt: String, col: Color32| {
        p.text(
            at,
            Align2::CENTER_CENTER,
            txt,
            FontId::proportional(13.0),
            col,
        );
    };
    lab(
        o + s(va) * 0.5 + vec2(-14.0, 0.0),
        format!("Va {:.1} kn", va.len()),
        pal.own,
    );
    lab(
        o + s(vb) * 0.5 + vec2(14.0, 10.0),
        format!("Vb {:.1} kn", vb.len()),
        pal.tgt,
    );
    lab(
        o + s(va) + s(vb - va) * 0.5 + vec2(0.0, -14.0),
        format!("Vbr {:.1} kn", (vb - va).len()),
        pal.rel,
    );
    let y = r.bottom() - 46.0;
    for (i, (c, s2)) in [
        (
            pal.own,
            t("Va: eigener wahrer Vektor", "Va: own true vector"),
        ),
        (
            pal.tgt,
            t("Vb: wahrer Vektor des Ziels", "Vb: target's true vector"),
        ),
        (pal.rel, t("Vbr: relativer Vektor", "Vbr: relative vector")),
    ]
    .iter()
    .enumerate()
    {
        let yy = y + i as f32 * 15.0;
        p.line_segment(
            [
                pos2(r.left() + 12.0, yy + 6.0),
                pos2(r.left() + 30.0, yy + 6.0),
            ],
            Stroke::new(3.0_f32, *c),
        );
        p.text(
            pos2(r.left() + 36.0, yy),
            Align2::LEFT_TOP,
            *s2,
            FontId::proportional(11.0),
            pal.ink,
        );
    }
}
