//! Kapitel 9: Darstellung – A-Scope und PPI, Orientierung (HU/NU/CU), RM/TM, Trails.

use std::collections::VecDeque;

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Shape, Stroke,
};

use super::charts::Chart;
use super::{merke, pal, see_also, slider, sub, text, try_it, Chapter};
use crate::app::{circle_clipped, clip_circle, line, View};
use crate::i18n::t;
use crate::util::{norm180, norm360, Rng, V2};

/// Gefüllter Kreis (Echo), auf die Radarscheibe (Mittelpunkt `c`, Radius `rad`) beschnitten.
fn disc_blob(p: &egui::Painter, c: Pos2, rad: f32, m: Pos2, r: f32, col: Color32) {
    let dist = (m - c).length();
    if dist + r <= rad {
        p.circle_filled(m, r, col);
        return;
    }
    if dist - r >= rad || r < 0.5 {
        return;
    }
    // Schnittmenge zweier Kreise: Randpunkte außerhalb radial auf den Scheibenrand ziehen
    let n = 64;
    let pts: Vec<Pos2> = (0..n)
        .map(|k| {
            let a = k as f32 / n as f32 * std::f32::consts::TAU;
            let q = m + vec2(a.cos(), a.sin()) * r;
            let d = q - c;
            let l = d.length();
            if l > rad {
                c + d * (rad / l)
            } else {
                q
            }
        })
        .collect();
    p.add(Shape::convex_polygon(pts, col, Stroke::NONE));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Orient {
    North,
    Head,
    Course,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Trails {
    Off,
    Rel,
    True,
}

struct Hist {
    own: V2,
    ship: V2,
}

pub struct State {
    own: V2,
    hdg: f64,
    cmd: f64,
    spd: f64,
    ship: V2,
    ship_crs: f64,
    ship_spd: f64,
    t: f64,
    warp: f64,
    playing: bool,
    orient: Orient,
    cu_ref: f64,
    tm: bool,
    tm_origin: V2,
    trails: Trails,
    trail_min: f64,
    hist: VecDeque<(f64, Hist)>,
    last_hist: f64,
    compass_fail: bool,
    yaw: bool,
    gain: f64,
    sweep: f64,
    frame: u64,
}

const BUOY: V2 = V2::new(1.2, 2.4);
const ISLAND: V2 = V2::new(-2.6, 3.6);
const RANGE: f64 = 6.0;

impl Default for State {
    fn default() -> Self {
        let mut s = Self::initial();
        // Vorführhilfe für Screenshots: RADAR_DEMO_WB=tm schaltet True Motion ein
        if super::demo_flag("tm") {
            s.tm = true;
            s.tm_origin = s.own + V2::from_bearing(s.hdg, 0.5 * RANGE);
        }
        s
    }
}

impl State {
    fn initial() -> Self {
        Self {
            own: V2::ZERO,
            hdg: 0.0,
            cmd: 0.0,
            spd: 12.0,
            ship: V2::new(4.6, 3.2),
            ship_crs: 250.0,
            ship_spd: 14.0,
            t: 0.0,
            warp: 20.0,
            playing: true,
            orient: Orient::North,
            cu_ref: 0.0,
            tm: false,
            tm_origin: V2::new(0.0, 2.5),
            trails: Trails::Rel,
            trail_min: 6.0,
            hist: VecDeque::new(),
            last_hist: -1e9,
            compass_fail: false,
            yaw: false,
            gain: 40.0,
            sweep: 0.0,
            frame: 0,
        }
    }
}

impl State {
    fn reset(&mut self) {
        let keep = (
            self.orient,
            self.tm,
            self.trails,
            self.trail_min,
            self.compass_fail,
            self.yaw,
            self.gain,
            self.warp,
        );
        *self = Self::initial();
        (
            self.orient,
            self.tm,
            self.trails,
            self.trail_min,
            self.compass_fail,
            self.yaw,
            self.gain,
            self.warp,
        ) = keep;
        if self.tm {
            self.tm_origin = self.own + V2::from_bearing(self.hdg, 0.5 * RANGE);
        }
    }

    fn yaw_offset(&self) -> f64 {
        if self.yaw {
            4.0 * (self.t / 9.0 * std::f64::consts::TAU).sin()
        } else {
            0.0
        }
    }

    fn step(&mut self, dt: f64) {
        self.t += dt;
        let err = norm180(self.cmd - self.hdg);
        let max = 30.0 / 60.0 * dt;
        self.hdg = norm360(self.hdg + err.clamp(-max, max));
        self.own += V2::from_bearing(self.hdg, self.spd) * (dt / 3600.0);
        self.ship += V2::from_bearing(self.ship_crs, self.ship_spd) * (dt / 3600.0);
        if self.t - self.last_hist >= 6.0 {
            self.hist.push_back((
                self.t,
                Hist {
                    own: self.own,
                    ship: self.ship,
                },
            ));
            self.last_hist = self.t;
            while self
                .hist
                .front()
                .is_some_and(|h| self.t - h.0 > 31.0 * 60.0)
            {
                self.hist.pop_front();
            }
        }
        if self.tm && (self.own - self.tm_origin).len() > 0.65 * RANGE {
            self.tm_origin = self.own + V2::from_bearing(self.hdg, 0.5 * RANGE);
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Das A-Scope zeigt die Echos eines einzigen Peilstrahls als Amplitude über der Entfernung. Das PPI (Plan Position Indicator) setzt die Strahlen eines ganzen Antennenumlaufs zu einer Draufsicht zusammen; das Nachleuchten hält das Bild bis zum nächsten Überstreichen. Rauschen erscheint auf beiden als Körnung.",
                "The A-scope shows the echoes of a single bearing as amplitude over range. The PPI (plan position indicator) combines the beams of a whole antenna revolution into a plan view; persistence holds the picture until the next sweep. Noise appears on both as speckle.",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if self.playing {
                    t("⏸ Pause", "⏸ Pause")
                } else {
                    t("▶ Weiter", "▶ Resume")
                })
                .clicked()
            {
                self.playing = !self.playing;
            }
            if ui.button(t("⟲ Neustart", "⟲ Restart")).clicked() {
                self.reset();
            }
            ui.label(t("Zeitraffer:", "Time warp:"));
            for w in [1.0, 10.0, 20.0, 60.0] {
                if ui
                    .selectable_label(self.warp == w, format!("{w}×"))
                    .clicked()
                {
                    self.warp = w;
                }
            }
            let mut c = self.cmd;
            if slider(
                ui,
                &mut c,
                0.0..=359.0,
                t("Eigener Sollkurs", "own set course"),
                "°",
                1.0,
            ) {
                self.cmd = c;
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(t("Orientierung:", "Orientation:"));
            ui.add_enabled_ui(!self.compass_fail, |ui| {
                ui.radio_value(&mut self.orient, Orient::North, "North Up");
            });
            ui.radio_value(&mut self.orient, Orient::Head, "Head Up");
            ui.add_enabled_ui(!self.compass_fail, |ui| {
                if ui
                    .radio(self.orient == Orient::Course, "Course Up")
                    .clicked()
                {
                    self.orient = Orient::Course;
                    self.cu_ref = self.hdg;
                }
                if ui.small_button("CU-Reset").clicked() {
                    self.cu_ref = self.hdg;
                }
            });
            ui.separator();
            ui.radio_value(&mut self.tm, false, "RM");
            ui.add_enabled_ui(!self.compass_fail, |ui| {
                if ui.radio(self.tm, "TM").clicked() && !self.tm {
                    self.tm = true;
                    self.tm_origin = self.own + V2::from_bearing(self.hdg, 0.5 * RANGE);
                }
            });
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Trails:");
            ui.radio_value(&mut self.trails, Trails::Off, t("aus", "off"));
            ui.radio_value(&mut self.trails, Trails::Rel, t("relativ", "relative"));
            ui.add_enabled_ui(!self.compass_fail, |ui| {
                ui.radio_value(&mut self.trails, Trails::True, t("wahr", "true"));
            });
            slider(
                ui,
                &mut self.trail_min,
                1.0..=30.0,
                t("Länge", "length"),
                " min",
                1.0,
            );
            ui.checkbox(&mut self.yaw, t("Gieren ±4°", "yawing ±4°"));
            slider(ui, &mut self.gain, 0.0..=100.0, "GAIN", "", 1.0);
            if ui
                .checkbox(
                    &mut self.compass_fail,
                    t("Kompass ausgefallen", "compass failed"),
                )
                .changed()
                && self.compass_fail
            {
                self.orient = Orient::Head;
                self.tm = false;
                if self.trails == Trails::True {
                    self.trails = Trails::Rel;
                }
            }
        });

        let dt = ui.input(|i| i.stable_dt).min(0.1) as f64;
        if self.playing {
            let mut rem = dt * self.warp;
            while rem > 1e-9 {
                let h = rem.min(1.0);
                self.step(h);
                rem -= h;
            }
        }
        self.sweep = (self.sweep + dt * 360.0 / 2.5) % 360.0;
        self.frame += 1;
        ui.ctx().request_repaint();

        let avail = ui.available_width();
        let size = (avail * 0.58).clamp(280.0, 520.0);
        let (rect, _) = ui.allocate_exact_size(vec2(avail, size), Sense::hover());
        let painter = ui.painter_at(rect);
        let ppi_c = pos2(rect.left() + size / 2.0, rect.center().y);
        let rad = size / 2.0 - 18.0;
        let ascope = Rect::from_min_max(
            pos2(rect.left() + size + 12.0, rect.top()),
            pos2(rect.right(), rect.top() + 230.0),
        );
        self.draw(&painter, ppi_c, rad, ascope, &p);
        let info = Rect::from_min_max(pos2(ascope.left(), ascope.bottom() + 8.0), rect.max);
        let mode = match (self.compass_fail, self.orient) {
            (true, _) => t("HEAD UP (unstabilisiert)", "HEAD UP (unstabilised)"),
            (_, Orient::North) => "NORTH UP",
            (_, Orient::Head) => "HEAD UP",
            (_, Orient::Course) => "COURSE UP",
        };
        let lines = [
            format!("{}  ·  {}", mode, if self.tm { "TM" } else { "RM" }),
            tf!(
                "HDG {:05.1}°  ·  {:.0} kn",
                "HDG {:05.1}°  ·  {:.0} kn",
                norm360(self.hdg + self.yaw_offset()),
                self.spd
            ),
            tf!(
                "Zeit {:02}:{:02}",
                "time {:02}:{:02}",
                (self.t / 60.0) as i64,
                (self.t % 60.0) as i64
            ),
        ];
        for (i, l) in lines.iter().enumerate() {
            painter.text(
                info.left_top() + vec2(4.0, 4.0 + 18.0 * i as f32),
                Align2::LEFT_TOP,
                l,
                FontId::monospace(13.0),
                p.ink,
            );
        }

        sub(ui, t("Orientierung und Bewegung", "Orientation and motion"));
        egui::Grid::new("disp_tab").striped(true).num_columns(2).spacing([18.0, 6.0]).show(ui, |ui| {
            let rows: [(&str, &str); 5] = [
                ("Head Up", t("Vorausrichtung oben, unstabilisiert: Das Bild dreht sich bei jeder Kursänderung und verschmiert beim Gieren. Einzige Möglichkeit ohne Kurssignal.", "heading up, unstabilised: the picture rotates with every course change and smears when yawing. The only option without a heading signal.")),
                ("North Up", t("Nord oben, kreiselstabilisiert: Das Bild bleibt beim Drehen stehen, nur der Vorausstrich dreht. Gut zum Kartenvergleich.", "north up, gyro-stabilised: the picture stays still when turning, only the heading line rotates. Good for comparing with the chart.")),
                ("Course Up", t("Sollkurs oben, stabilisiert: ruhiges Bild wie North Up, aber „voraus ist oben“. Nach Kursänderung zurücksetzen (CU-Reset).", "set course up, stabilised: steady picture like north up but “ahead is up”. Reset after a course change (CU reset).")),
                ("RM", t("Relative Motion: Eigenschiff fest, alle anderen Echos bewegen sich relativ – auch feste Objekte.", "relative motion: own ship fixed, all other echoes move relatively – fixed objects too.")),
                ("TM", t("True Motion: Eigenschiff fährt über den Schirm, feste Objekte stehen; Reset, wenn das Schiff den Rand erreicht.", "true motion: own ship moves across the screen, fixed objects stand still; reset when the ship reaches the edge.")),
            ];
            for (a, b) in rows {
                ui.label(RichText::new(a).strong());
                ui.label(b);
                ui.end_row();
            }
        });
        sub(
            ui,
            t("Relative und wahre Trails", "Relative and true trails"),
        );
        text(
            ui,
            t(
                "Relative Trails zeigen, wie sich ein Echo relativ zum Eigenschiff bewegt hat: Zeigt der Trail auf uns, besteht Kollisionsgefahr – ideal zur schnellen Gefahrenerkennung. Auch Tonne und Insel ziehen Trails, wenn wir fahren. Wahre Trails zeigen die tatsächliche Bewegung: Feste Objekte haben keinen Trail, Kurs und Aspekt der anderen sind erkennbar – ideal für das Lagebild; sie brauchen Kurs- und Fahrtsignal. Trails zeigen immer die Vergangenheit, Vektoren eine Vorausberechnung.",
                "Relative trails show how an echo has moved relative to own ship: if the trail points at us, there is a risk of collision – ideal for quick hazard detection. Buoy and island also leave trails when we move. True trails show the actual motion: fixed objects have no trail, course and aspect of others are visible – ideal for the overall picture; they need heading and speed input. Trails always show the past, vectors a prediction.",
            ),
        );
        try_it(
            ui,
            t(
                "Schalten Sie bei relativen Trails zwischen RM und TM um und dann auf wahre Trails: Der Trail der Tonne verschwindet. Setzen Sie den Sollkurs auf 60° in Head Up: Das ganze Bild dreht sich. Haken Sie „Kompass ausgefallen“ an – nur Head Up und relative Trails bleiben übrig.",
                "With relative trails switch between RM and TM, then to true trails: the buoy's trail disappears. Set the course to 60° in head up: the whole picture rotates. Tick “compass failed” – only head up and relative trails remain.",
            ),
        );
        merke(
            ui,
            &[
                t("PPI: Draufsicht aus allen Strahlen eines Umlaufs; A-Scope: ein fester Azimut, Amplitude über Entfernung.", "PPI: plan view from all beams of one revolution; A-scope: one fixed azimuth, amplitude over range."),
                t("Thermisches Rauschen ist auf dem PPI als Körnung sichtbar (Gain).", "Thermal noise is visible on the PPI as speckle (gain)."),
                t("Kompassausfall ⇒ Head Up (unstabilisiert).", "Compass failure ⇒ head up (unstabilised)."),
                t("Lagebild: wahre Trails · Kollisionsgefahr schnell erkennen: relative Trails.", "Overall picture: true trails · quick collision check: relative trails."),
                t("Trails = Vergangenheit, Vektoren = Zukunft.", "Trails = past, vectors = future."),
            ],
        );
        see_also(
            ui,
            nav,
            &[Chapter::Signal, Chapter::Stabilisation, Chapter::RelVector],
        );
    }

    fn draw(&self, p: &egui::Painter, c: Pos2, rad: f32, ascope: Rect, pal: &super::Pal) {
        let hdg_disp = norm360(self.hdg + self.yaw_offset());
        let rot = match (self.compass_fail, self.orient) {
            (true, _) | (_, Orient::Head) => hdg_disp,
            (_, Orient::North) => 0.0,
            (_, Orient::Course) => self.cu_ref,
        };
        let cw = if self.tm { self.tm_origin } else { self.own };
        let ppn = rad / RANGE as f32;
        let w2s = |w: V2| -> Pos2 {
            let rel = w - cw;
            let (s, co) = rot.to_radians().sin_cos();
            let dx = rel.x * co - rel.y * s;
            let dy = rel.x * s + rel.y * co;
            c + vec2(dx as f32, -dy as f32) * ppn
        };
        let dir = |brg: f64| {
            let a = (brg - rot).to_radians() as f32;
            vec2(a.sin(), -a.cos())
        };
        p.circle_filled(c, rad + 8.0, Color32::from_rgb(22, 40, 70));
        p.circle_filled(c, rad, Color32::from_rgb(26, 62, 128));
        // Alles auf dem Schirm wird auf die Radarscheibe beschnitten – wichtig in True Motion,
        // wenn das Eigenschiff (und damit die Ringe) nicht in der Mitte steht.
        let view = View {
            center: c,
            radius: rad,
            range: RANGE,
            rot,
            cw,
        };
        let own_s = w2s(self.own);
        for k in 1..=3 {
            circle_clipped(
                p,
                &view,
                own_s,
                rad * k as f32 / 3.0,
                Stroke::new(0.7_f32, Color32::from_rgba_unmultiplied(160, 200, 240, 70)),
                false,
            );
        }
        let clip = p.with_clip_rect(Rect::from_center_size(c, vec2(rad * 2.0, rad * 2.0)));
        // Rauschen (Körnung) abhängig vom Gain
        let mut rng = Rng::new(self.frame / 3 + 7);
        let dots = (self.gain * self.gain / 25.0) as usize;
        for _ in 0..dots {
            let a = rng.range(0.0, std::f64::consts::TAU) as f32;
            let r = (rng.f64().sqrt() as f32) * (rad - 1.0);
            clip.circle_filled(
                c + vec2(a.sin(), a.cos()) * r,
                1.0,
                Color32::from_rgba_unmultiplied(255, 230, 120, 90),
            );
        }
        // Trails
        if self.trails != Trails::Off {
            let col = Color32::from_rgba_unmultiplied(255, 230, 120, 120);
            for (tt, h) in self.hist.iter() {
                if self.t - tt > self.trail_min * 60.0 {
                    continue;
                }
                let objs: [V2; 3] = [h.ship, BUOY, ISLAND];
                for (i, o) in objs.iter().enumerate() {
                    let pos = match self.trails {
                        Trails::Rel => *o - h.own + self.own,
                        _ => *o,
                    };
                    let s = w2s(pos);
                    let r = if i == 2 { 2.5 } else { 1.8 };
                    if (s - c).length() < rad - r {
                        clip.circle_filled(s, r, col);
                    }
                }
                if self.trails == Trails::True && self.tm {
                    let s = w2s(h.own);
                    if (s - c).length() < rad - 1.5 {
                        clip.circle_filled(
                            s,
                            1.5,
                            Color32::from_rgba_unmultiplied(255, 255, 255, 90),
                        );
                    }
                }
            }
        }
        // Echos mit Nachleuchten
        let objs = [
            (self.ship, 0.12f32, self.ship_crs, 0.9f32),
            (BUOY, 0.04, 0.0, 0.8),
            (ISLAND, 0.45, 30.0, 1.0),
        ];
        for (i, (w, size, crs, amp)) in objs.iter().enumerate() {
            let rel = *w - self.own;
            let brg = rel.bearing();
            let age = (self.sweep - brg).rem_euclid(360.0) / 360.0 * 2.5;
            let glow = (0.3 + 0.7 * (-age / 2.5).exp()) as f32 * amp;
            let s = w2s(*w);
            let col = Color32::from_rgba_unmultiplied(255, 230, 110, (glow * 255.0) as u8);
            if i == 2 {
                disc_blob(p, c, rad, s, size * ppn, col);
                disc_blob(
                    p,
                    c,
                    rad,
                    s + vec2(size * ppn * 0.6, -size * ppn * 0.3),
                    size * ppn * 0.7,
                    col,
                );
            } else {
                let d = dir(*crs) * (size * ppn).max(3.0);
                if let Some((a, b)) = clip_circle(s - d, s + d, c, rad - 2.5) {
                    p.line_segment([a, b], Stroke::new(5.0_f32, col));
                }
            }
        }
        // Vorausstrich, Nordmarke, Sweep
        line(
            p,
            &view,
            own_s,
            own_s + dir(hdg_disp) * rad * 2.2,
            Stroke::new(1.3_f32, Color32::WHITE),
        );
        line(
            p,
            &view,
            own_s,
            own_s + dir(self.sweep) * rad * 2.2,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 240, 160, 110)),
        );
        p.circle_stroke(own_s, 5.0, Stroke::new(1.5_f32, Color32::WHITE));
        if !self.compass_fail {
            let n = c + dir(0.0) * (rad + 1.0);
            p.add(Shape::convex_polygon(
                vec![
                    n,
                    n - dir(0.0) * 10.0 + dir(90.0) * 5.0,
                    n - dir(0.0) * 10.0 - dir(90.0) * 5.0,
                ],
                pal.warn,
                Stroke::NONE,
            ));
            p.text(
                c + dir(0.0) * (rad + 6.0),
                Align2::CENTER_CENTER,
                "N",
                FontId::proportional(11.0),
                pal.warn,
            );
        }
        p.text(
            c + vec2(-rad, -rad),
            Align2::LEFT_TOP,
            "6 NM",
            FontId::monospace(12.0),
            Color32::WHITE,
        );

        // A-Scope des aktuellen Strahls
        let ch = Chart::new(ascope, (0.0, RANGE), (0.0, 1.3));
        ch.frame(
            p,
            pal,
            &tf!("A-Scope bei {:03.0}°", "A-scope at {:03.0}°", self.sweep),
            t("Entfernung (sm)", "range (nm)"),
            "",
        );
        let mut rng = Rng::new(self.frame + 11);
        let noise = self.gain / 100.0 * 0.35;
        let n = 300;
        let mut prof = vec![0.0f64; n + 1];
        for (w, size, _, amp) in objs.iter() {
            let rel = *w - self.own;
            let d = norm180(rel.bearing() - self.sweep).abs();
            let ang_w = (*size as f64 / rel.len().max(0.1)).to_degrees() + 1.5;
            if d < ang_w {
                let r0 = rel.len() - *size as f64 * 0.5;
                let r1 = rel.len() + *size as f64 * 0.5 + 0.05;
                for (i, v) in prof.iter_mut().enumerate() {
                    let r = RANGE * i as f64 / n as f64;
                    if r >= r0 && r <= r1 {
                        *v += *amp as f64 * (1.0 - d / ang_w * 0.5);
                    }
                }
            }
        }
        ch.line(
            p,
            (0..=n).map(|i| {
                (
                    RANGE * i as f64 / n as f64,
                    prof[i] + noise * rng.expo() as f64,
                )
            }),
            Stroke::new(1.3_f32, pal.echo),
        );
    }
}
