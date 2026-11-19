//! Kapitel 12: Manuelles Plotten Schritt für Schritt (mit der Rechnung der Plottumgebung).

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Shape, Stroke,
};

use super::{formula, merke, pal, see_also, text, Chapter, Pal, WbAction};
use crate::app::{angle_edit, fb};
use crate::i18n::t;
use crate::plot::{
    classify, cockcroft_sector, encounter_label, fmt_clock, maneuver, solve, ManGoal, ManKind,
    ManParams, ManPoint, ManResult, Obs, PTarget, Side, Solution, COCKCROFT,
};
use crate::util::{norm360, V2};

const STEPS: usize = 11;

pub struct State {
    own_crs: f64,
    own_spd: f64,
    t1: String,
    d1: f64,
    b1: f64,
    t2: String,
    d2: f64,
    b2: f64,
    man_after: f64,
    cpa_goal: f64,
    side: Side,
    step: usize,
}

impl Default for State {
    fn default() -> Self {
        let mut s = Self {
            own_crs: 0.0,
            own_spd: 12.0,
            t1: String::new(),
            d1: 0.0,
            b1: 0.0,
            t2: String::new(),
            d2: 0.0,
            b2: 0.0,
            man_after: 3.0,
            cpa_goal: 2.0,
            side: Side::Stbd,
            step: 0,
        };
        s.example();
        if super::demo_flag("exercise") {
            s.exercise();
        }
        if let Some(st) = std::env::var("RADAR_DEMO_STEP")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
        {
            s.step = st.min(STEPS - 1);
        }
        s
    }
}

impl State {
    fn example(&mut self) {
        self.own_crs = 0.0;
        self.own_spd = 12.0;
        self.t1 = "12:00".into();
        self.d1 = 9.0;
        self.b1 = 40.0;
        self.t2 = "12:06".into();
        self.d2 = 7.6;
        self.b2 = 39.0;
        self.man_after = 6.0;
        self.cpa_goal = 2.0;
        self.side = Side::Stbd;
        self.step = 0;
    }

    /// Übungsaufgabe zu den Fragen am Kapitelende (Kreuzer von Backbord, verminderte Sicht).
    fn exercise(&mut self) {
        self.own_crs = 30.0;
        self.own_spd = 13.0;
        self.t1 = "10:00".into();
        self.d1 = 8.0;
        self.b1 = 350.0;
        self.t2 = "10:06".into();
        self.d2 = 6.8;
        self.b2 = 351.0;
        self.man_after = 6.0;
        self.cpa_goal = 2.0;
        self.side = Side::Stbd;
        self.step = 0;
    }

    fn target(&self) -> PTarget {
        PTarget {
            name: "A".into(),
            o1: Obs {
                time: self.t1.clone(),
                dist: self.d1,
                brg: self.b1,
                rel: false,
            },
            o2: Obs {
                time: self.t2.clone(),
                dist: self.d2,
                brg: self.b2,
                rel: false,
            },
        }
    }

    fn man(&self, s: &Solution) -> Result<ManResult, &'static str> {
        let mp = ManParams {
            point: ManPoint::Time,
            time: fmt_clock(s.t2 + self.man_after),
            dist: 0.0,
            kind: ManKind::Course,
            goal: ManGoal::Cpa,
            cpa: self.cpa_goal,
            course: 0.0,
            speed: 0.0,
            side: self.side,
        };
        maneuver(s, self.own_crs, self.own_spd, &mp)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>, action: &mut WbAction) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Hier wird ein Radarplot wie auf dem Plottblatt Schritt für Schritt aufgebaut – mit der Rechnung in Zahlen. Das Eigenschiff steht in der Mitte (relative Darstellung, North Up). Geben Sie eigene Werte ein oder laden Sie eine Vorlage, dann gehen Sie mit „Weiter“ durch die Schritte.",
                "Here a radar plot is built up step by step like on the plotting sheet – with the calculation in numbers. Own ship is in the centre (relative display, north up). Enter your own values or load a template, then go through the steps with “next”.",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(t("Vorlage: Beispiel", "template: example"))
                .clicked()
            {
                self.example();
            }
            if ui
                .button(t("Vorlage: Übung", "template: exercise"))
                .clicked()
            {
                self.exercise();
            }
        });
        egui::Grid::new("plot_inputs")
            .num_columns(8)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new(t("Eigenschiff", "Own ship")).strong());
                ui.label(t("Kurs", "course"));
                angle_edit(ui, &mut self.own_crs, 0.5, 0, "°");
                ui.label(t("Fahrt", "speed"));
                ui.add(
                    egui::DragValue::new(&mut self.own_spd)
                        .range(0.0..=30.0)
                        .speed(0.1)
                        .fixed_decimals(1)
                        .suffix(" kn"),
                );
                ui.label(t("Manöver nach", "manoeuvre after"));
                ui.add(
                    egui::DragValue::new(&mut self.man_after)
                        .range(0.0..=60.0)
                        .speed(0.1)
                        .fixed_decimals(1)
                        .suffix(" min"),
                );
                ui.label("");
                ui.end_row();
                for (k, tt, d, b) in [
                    (1, &mut self.t1, &mut self.d1, &mut self.b1),
                    (2, &mut self.t2, &mut self.d2, &mut self.b2),
                ] {
                    ui.label(RichText::new(tf!("Beobachtung {}", "observation {}", k)).strong());
                    ui.label(t("Zeit", "time"));
                    ui.add(
                        egui::TextEdit::singleline(tt)
                            .desired_width(60.0)
                            .hint_text("HH:MM"),
                    );
                    ui.label(t("Distanz", "range"));
                    ui.add(
                        egui::DragValue::new(d)
                            .range(0.0..=24.0)
                            .speed(0.02)
                            .fixed_decimals(2)
                            .suffix(t(" sm", " nm")),
                    );
                    ui.label(t("Peilung rw", "true bearing"));
                    angle_edit(ui, b, 0.25, 1, "°");
                    ui.label("");
                    ui.end_row();
                }
                ui.label(RichText::new(t("Ziel-CPA", "CPA goal")).strong());
                ui.add(
                    egui::DragValue::new(&mut self.cpa_goal)
                        .range(0.2..=6.0)
                        .speed(0.05)
                        .fixed_decimals(1)
                        .suffix(t(" sm", " nm")),
                );
                ui.selectable_value(&mut self.side, Side::Stbd, t("Stb", "stbd"));
                ui.selectable_value(&mut self.side, Side::Port, t("Bb", "port"));
                ui.end_row();
            });

        let sol = solve(self.own_crs, self.own_spd, &self.target());
        let s = match &sol {
            Ok(s) => *s,
            Err(e) => {
                ui.label(RichText::new(*e).color(p.warn).strong());
                return;
            }
        };
        let man = self.man(&s);

        // Schrittnavigation
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(self.step > 0, egui::Button::new(t("◀ Zurück", "◀ Back")))
                .clicked()
            {
                self.step -= 1;
            }
            for i in 0..STEPS {
                if ui
                    .selectable_label(self.step == i, format!("{}", i))
                    .clicked()
                {
                    self.step = i;
                }
            }
            if ui
                .add_enabled(
                    self.step + 1 < STEPS,
                    egui::Button::new(t("Weiter ▶", "Next ▶")),
                )
                .clicked()
            {
                self.step += 1;
            }
        });

        let avail = ui.available_width();
        let size = (avail * 0.56).clamp(300.0, 560.0);
        let (rect, _) = ui.allocate_exact_size(vec2(avail, size), Sense::hover());
        let painter = ui.painter_at(rect);
        let c = pos2(rect.left() + size / 2.0, rect.center().y);
        self.draw(
            &painter,
            c,
            size / 2.0 - 30.0,
            &p,
            &s,
            man.as_ref().ok(),
            None,
        );
        let side = Rect::from_min_max(pos2(rect.left() + size + 14.0, rect.top()), rect.max);
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(side)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        self.explain(&mut child, &p, &s, &man, action);

        // Lupe auf die Konstruktion (O, A, W, M, O′)
        if self.step >= 1 {
            let pts = self.focus_points(&s, man.as_ref().ok());
            let (mut lo, mut hi) = (pts[0], pts[0]);
            for q in &pts {
                lo = V2::new(lo.x.min(q.x), lo.y.min(q.y));
                hi = V2::new(hi.x.max(q.x), hi.y.max(q.y));
            }
            let mid = (lo + hi) * 0.5;
            let half = ((hi.x - lo.x).max(hi.y - lo.y) * 0.5 * 1.35).max(0.4);
            let (mrect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 330.0), Sense::hover());
            let mp = ui.painter_at(mrect);
            let mrad = mrect.width().min(mrect.height()) / 2.0 - 12.0;
            self.draw(
                &mp,
                mrect.center(),
                mrad,
                &p,
                &s,
                man.as_ref().ok(),
                Some((mrect, mid, half)),
            );
        }

        merke(
            ui,
            &[
                t("Relativweg O→A in Δt ⇒ vBr = OA · 60 / Δt (kn); Richtung = KBr.", "Relative track O→A in Δt ⇒ vBr = OA · 60 / Δt (kn); direction = course of relative motion."),
                t("CPA = Lot vom Eigenschiff auf die verlängerte Relativbewegung; TCPA = Weg A→CPA / vBr.", "CPA = perpendicular from own ship to the extended relative motion line; TCPA = distance A→CPA / vBr."),
                t("Vektordreieck: W→O eigener Weg (Kurs/Fahrt · Δt), O→A relativ, W→A wahrer Weg des Ziels: Vb = Va + Vbr.", "Vector triangle: W→O own track (course/speed · Δt), O→A relative, W→A target's true track: Vb = Va + Vbr."),
                t("Manöver: neue Relativbewegung von M als Tangente an den CPA-Kreis, parallel durch A verschieben, Schnitt mit dem Kreis um W ⇒ neuer Kurs W→O′.", "Manoeuvre: new relative motion from M as tangent to the CPA circle, shift it parallel through A, intersect with the circle around W ⇒ new course W→O′."),
                t("Genauigkeit: Peil-, Entfernungs-, Zeit- und Kurs-/Fahrtfehler, Zeichengenauigkeit; längeres Intervall verkleinert den Einfluss.", "Accuracy: bearing, range, time and course/speed errors, drawing accuracy; a longer interval reduces their influence."),
            ],
        );
        see_also(
            ui,
            nav,
            &[
                Chapter::SpeedTime,
                Chapter::RelVector,
                Chapter::Stabilisation,
            ],
        );
    }

    fn explain(
        &self,
        ui: &mut egui::Ui,
        p: &Pal,
        s: &Solution,
        man: &Result<ManResult, &'static str>,
        action: &mut WbAction,
    ) {
        let oa = (s.p2 - s.p1).len();
        let own_d = self.own_spd * s.dt / 60.0;
        let wa = (s.p2 - (s.p1 - s.vo * (s.dt / 60.0))).len();
        let (title, body): (&str, String) = match self.step {
            0 => (
                t("0 · Aufgabe", "0 · Task"),
                tf!(
                    "Eigenschiff {} mit {:.1} kn. Ziel A: um {} in {:.2} sm auf {}, um {} in {:.2} sm auf {}. Gesucht: Relativbewegung, CPA/TCPA, Kurs und Fahrt des Ziels, Lage nach KVR und eine Kursänderung {:.1} min nach der 2. Beobachtung für {:.1} sm CPA.",
                    "Own ship {} at {:.1} kn. Target A: at {} {:.2} nm on {}, at {} {:.2} nm on {}. Find: relative motion, CPA/TCPA, target course and speed, COLREG situation and a course change {:.1} min after the 2nd observation for a {:.1} nm CPA.",
                    fb(self.own_crs), self.own_spd, self.t1, self.d1, fb(self.b1), self.t2, self.d2, fb(self.b2), self.man_after, self.cpa_goal
                ),
            ),
            1 => (
                t("1 · Beobachtungen eintragen", "1 · Plot the observations"),
                tf!(
                    "O ({}): {:.2} sm auf {}.  A ({}): {:.2} sm auf {}.  Zeitabstand Δt = {:.1} min. Peilungen mit dem Kursdreieck am Rand ablesen, Distanzen mit dem Zirkel aus der Entfernungsskala.",
                    "O ({}): {:.2} nm on {}.  A ({}): {:.2} nm on {}.  Interval Δt = {:.1} min. Read bearings at the edge of the sheet, take ranges with the dividers from the range scale.",
                    self.t1, self.d1, fb(self.b1), self.t2, self.d2, fb(self.b2), s.dt
                ),
            ),
            2 => (
                t("2 · Relativbewegung", "2 · Relative motion"),
                tf!(
                    "Strecke OA = {:.2} sm in {:.1} min.  vBr = {:.2} · 60 / {:.1} = {:.1} kn.  Richtung O→A: KBr = {}. Weg-Zeit-Fahrt lässt sich auch mit Nomogramm, Zirkel oder Rechenschieber bestimmen (Kapitel 13).",
                    "Distance OA = {:.2} nm in {:.1} min.  Relative speed = {:.2} · 60 / {:.1} = {:.1} kn.  Direction O→A: {}. Speed–time–distance can also be found with the nomogram, dividers or slide rule (chapter 13).",
                    oa, s.dt, oa, s.dt, s.vbr, fb(s.kbr)
                ),
            ),
            3 => (
                t("3 · CPA", "3 · CPA"),
                tf!(
                    "Die Linie O→A über A hinaus verlängern. Das Lot vom Eigenschiff auf diese Linie ist der CPA: {:.2} sm in der Peilung {} (Seitenpeilung {}).{}",
                    "Extend the line O→A beyond A. The perpendicular from own ship onto it is the CPA: {:.2} nm on bearing {} (relative {}).{}",
                    s.cpa, fb(s.pcpa), fb(s.spcpa),
                    match s.bcr { Some(b) if b > 0.0 => tf!(" Das Ziel kreuzt {:.2} sm vor dem Bug.", " The target crosses {:.2} nm ahead.", b), Some(b) => tf!(" Das Ziel kreuzt {:.2} sm hinter dem Heck.", " The target crosses {:.2} nm astern.", -b), None => String::new() }
                ),
            ),
            4 => (
                t("4 · TCPA", "4 · TCPA"),
                tf!(
                    "Weg von A bis zum CPA-Punkt: {:.2} sm. TCPA = {:.2} · 60 / {:.1} kn = {:.1} min nach der 2. Beobachtung, also um {}.",
                    "Distance from A to the CPA point: {:.2} nm. TCPA = {:.2} · 60 / {:.1} kn = {:.1} min after the 2nd observation, i.e. at {}.",
                    s.vbr * s.tcpa / 60.0, s.vbr * s.tcpa / 60.0, s.vbr, s.tcpa, fmt_clock(s.t2 + s.tcpa)
                ),
            ),
            5 => (
                t("5 · Eigener Weg", "5 · Own track"),
                tf!(
                    "In Δt = {:.1} min legt das Eigenschiff {:.1} · {:.1} / 60 = {:.2} sm zurück. Von O aus entgegen dem eigenen Kurs ({}) abtragen: Punkt W. Der Pfeil W→O ist der eigene Weg.",
                    "In Δt = {:.1} min own ship covers {:.1} · {:.1} / 60 = {:.2} nm. Plot it from O against own course ({}): point W. The arrow W→O is own track.",
                    s.dt, self.own_spd, s.dt, own_d, fb(norm360(self.own_crs + 180.0))
                ),
            ),
            6 => (
                t("6 · Wahrer Kurs und Fahrt des Ziels", "6 · True course and speed of the target"),
                tf!(
                    "Der Pfeil W→A ist der wahre Weg des Ziels: {:.2} sm in {:.1} min ⇒ vB = {:.1} kn, Richtung KB = {}. Kontrolle: Vb = Va + Vbr.",
                    "The arrow W→A is the target's true track: {:.2} nm in {:.1} min ⇒ speed {:.1} kn, course {}. Check: Vb = Va + Vbr.",
                    wa, s.dt, s.vb, fb(s.kb)
                ),
            ),
            7 => {
                let enc = classify(s, self.own_crs, self.cpa_goal);
                let sec = COCKCROFT[cockcroft_sector(s.rb2)];
                (
                    t("7 · Lage beurteilen", "7 · Assess the situation"),
                    tf!(
                        "{}. Seitenpeilung {}, Aspekt {} {:.0}°. Bei verminderter Sicht (Regel 19) empfiehlt das Cockcroft-Diagramm für diesen Sektor: {}.",
                        "{}. Relative bearing {}, aspect {} {:.0}°. In restricted visibility (Rule 19) the Cockcroft diagram recommends for this sector: {}.",
                        encounter_label(enc).0,
                        fb(s.rb2),
                        if s.aspect >= 0.0 { t("grün", "green") } else { t("rot", "red") },
                        s.aspect.abs(),
                        if crate::i18n::is_de() { sec.4 } else { sec.5 }
                    ),
                )
            }
            8 => {
                let pm = s.p2 + s.v * (self.man_after / 60.0);
                (
                    t("8 · Manöverpunkt M", "8 · Manoeuvre point M"),
                    tf!(
                        "Das Ziel läuft weiter auf der Relativbewegung. {:.1} min nach der 2. Beobachtung ({}) steht es in M: {:.2} sm auf {}. Weg A→M = {:.1} kn · {:.1} / 60 = {:.2} sm.",
                        "The target continues on the relative motion line. {:.1} min after the 2nd observation ({}) it is at M: {:.2} nm on {}. Distance A→M = {:.1} kn · {:.1} / 60 = {:.2} nm.",
                        self.man_after,
                        fmt_clock(s.t2 + self.man_after),
                        pm.len(),
                        fb(pm.bearing()),
                        s.vbr,
                        self.man_after,
                        s.vbr * self.man_after / 60.0
                    ),
                )
            }
            9 => (
                t("9 · Gewünschte neue Relativbewegung", "9 · Desired new relative motion"),
                match man {
                    Ok(m) => tf!(
                        "Kreis mit dem gewünschten CPA ({:.1} sm) um das Eigenschiff schlagen. Von M die Tangente an diesen Kreis legen (Passieren an der gewählten Seite): neue Relativbewegung KBr′ = {} (Änderung {:+.1}°).",
                        "Draw a circle with the desired CPA ({:.1} nm) around own ship. From M draw the tangent to this circle (passing on the chosen side): new relative motion {} (change {:+.1}°).",
                        self.cpa_goal, fb(m.kbr), m.dkbr
                    ),
                    Err(e) => tf!(
                        "{} – mit diesen Werten ist der gewünschte CPA durch eine Kursänderung zur gewählten Seite nicht erreichbar (aktueller CPA {:.2} sm).",
                        "{} – with these values the desired CPA cannot be reached by a course change to the chosen side (current CPA {:.2} nm).",
                        e, s.cpa
                    ),
                },
            ),
            _ => (
                if self.step == 10 { t("10 · Parallel verschieben ⇒ neuer Kurs", "10 · Shift in parallel ⇒ new course") } else { "" },
                match man {
                    Ok(m) => tf!(
                        "Die neue Relativrichtung parallel durch A verschieben (rückwärts verlängern). Kreis um W mit dem eigenen Weg {:.2} sm schlagen. Schnittpunkt O′: W→O′ ist der neue eigene Kurs {} – Kursänderung {:.1}° nach {}. Neuer CPA {:.2} sm um {}. Wiederaufnahme von Kurs und Fahrt ab ca. {}.",
                        "Shift the new relative direction in parallel through A (extend backwards). Draw a circle around W with own track {:.2} nm. Intersection O′: W→O′ is the new own course {} – alteration {:.1}° to {}. New CPA {:.2} nm at {}. Resume course and speed from approx. {}.",
                        own_d,
                        fb(m.course),
                        m.dk.abs(),
                        if m.dk >= 0.0 { t("Steuerbord", "starboard") } else { t("Backbord", "port") },
                        m.cpa,
                        fmt_clock(m.t_m + m.tcpa),
                        m.resume.map_or("--:--".into(), |r| fmt_clock(r.0))
                    ),
                    Err(e) => (*e).to_string(),
                },
            ),
        };
        ui.label(RichText::new(title).strong().size(17.0).color(p.accent));
        ui.add_space(4.0);
        ui.label(RichText::new(body).size(14.5));
        ui.add_space(10.0);
        if self.step + 1 == STEPS {
            if ui
                .button(t(
                    "In der Plottumgebung öffnen",
                    "Open in the plotting environment",
                ))
                .clicked()
            {
                *action = WbAction::OpenInPlot {
                    own_course: self.own_crs,
                    own_speed: self.own_spd,
                    target: self.target(),
                };
            }
        }
        ui.add_space(8.0);
        formula(ui, &tf!(
            "KBr {}  vBr {:.1} kn\nKB  {}  vB  {:.1} kn\nCPA {:.2} sm  TCPA {:.1} min",
            "rel. course {}  rel. speed {:.1} kn\ncourse {}  speed {:.1} kn\nCPA {:.2} nm  TCPA {:.1} min",
            fb(s.kbr), s.vbr, fb(s.kb), s.vb, s.cpa, s.tcpa
        ));
    }

    /// Punkte, die die Lupe umfassen soll (abhängig vom Schritt).
    fn focus_points(&self, s: &Solution, man: Option<&ManResult>) -> Vec<V2> {
        let mut v = vec![s.p1, s.p2];
        if self.step >= 5 {
            v.push(s.p1 - s.vo * (s.dt / 60.0));
        }
        if self.step >= 8 {
            v.push(s.p2 + s.v * (self.man_after / 60.0));
        }
        if let (true, Some(m)) = (self.step >= 10, man) {
            v.push(s.p2 - m.v * (s.dt / 60.0));
        }
        v
    }

    /// Plottblatt zeichnen. `view = None`: ganzes Blatt (Eigenschiff in der Mitte);
    /// `Some(rect, Mitte, Halbbreite)`: Lupe auf einen Ausschnitt.
    fn draw(
        &self,
        p: &egui::Painter,
        c: Pos2,
        rad: f32,
        pal: &Pal,
        s: &Solution,
        man: Option<&ManResult>,
        view: Option<(Rect, V2, f64)>,
    ) {
        let maxd = self.d1.max(self.d2).max(1.0);
        let sheet_range = [3.0, 6.0, 8.0, 12.0, 16.0, 24.0]
            .into_iter()
            .find(|r| *r >= maxd * 1.15)
            .unwrap_or(24.0);
        let ring = if sheet_range <= 3.0 {
            0.5
        } else if sheet_range <= 8.0 {
            1.0
        } else {
            2.0
        };
        let (cw, range, full) = match view {
            None => (V2::ZERO, sheet_range, true),
            Some((_, cw, r)) => (cw, r, false),
        };
        let ppn = rad / range as f32;
        let w2s = |w: V2| c + vec2((w - cw).x as f32, -(w - cw).y as f32) * ppn;
        let sheet = if pal.panel.r() < 100 {
            Color32::from_rgb(18, 34, 64)
        } else {
            Color32::from_rgb(252, 252, 246)
        };
        let ink = pal.ink;
        let own_s = w2s(V2::ZERO);
        let clip_rect = match view {
            None => Rect::from_center_size(c, vec2(rad * 2.0, rad * 2.0)),
            Some((r, _, _)) => r,
        };
        if full {
            p.circle_filled(c, rad + 26.0, pal.panel);
            p.circle_filled(c, rad, sheet);
        } else {
            p.rect_filled(clip_rect, egui::CornerRadius::same(6), sheet);
        }
        let ringclip = p.with_clip_rect(clip_rect);
        let n = (sheet_range / ring).round() as usize;
        for k in 1..=n {
            ringclip.circle_stroke(
                own_s,
                (k as f64 * ring) as f32 * ppn,
                Stroke::new(
                    if k == n { 1.4_f32 } else { 0.7 },
                    if k == n { ink } else { pal.grid },
                ),
            );
        }
        if full {
            for deg in (0..360).step_by(10) {
                let a = (deg as f32).to_radians();
                let d = vec2(a.sin(), -a.cos());
                p.line_segment(
                    [
                        c + d * rad,
                        c + d * (rad + if deg % 30 == 0 { 9.0 } else { 5.0 }),
                    ],
                    Stroke::new(1.0_f32, ink),
                );
                if deg % 30 == 0 {
                    p.text(
                        c + d * (rad + 18.0),
                        Align2::CENTER_CENTER,
                        format!("{deg:03}"),
                        FontId::monospace(10.0),
                        pal.faint,
                    );
                }
            }
            p.text(
                c + vec2(-rad, -rad - 14.0),
                Align2::LEFT_TOP,
                &tf!(
                    "{} sm, Ringe {} sm",
                    "{} nm, rings {} nm",
                    sheet_range,
                    ring
                ),
                FontId::proportional(11.0),
                pal.faint,
            );
        } else {
            let bar = [0.1, 0.2, 0.25, 0.5, 1.0, 2.0]
                .into_iter()
                .filter(|b| (*b as f32) * ppn < clip_rect.width() * 0.3)
                .last()
                .unwrap_or(0.1);
            let y = clip_rect.bottom() - 14.0;
            let x = clip_rect.left() + 14.0;
            p.line_segment(
                [pos2(x, y), pos2(x + bar as f32 * ppn, y)],
                Stroke::new(2.0_f32, ink),
            );
            p.text(
                pos2(x, y - 4.0),
                Align2::LEFT_BOTTOM,
                &tf!("{} sm", "{} nm", bar),
                FontId::proportional(11.0),
                ink,
            );
            p.text(
                clip_rect.left_top() + vec2(10.0, 6.0),
                Align2::LEFT_TOP,
                t(
                    "Lupe: Konstruktion vergrößert",
                    "Magnifier: construction enlarged",
                ),
                FontId::proportional(13.0),
                ink,
            );
        }
        let hd = (self.own_crs as f32).to_radians();
        ringclip.line_segment(
            [
                own_s,
                own_s + vec2(hd.sin(), -hd.cos()) * (sheet_range as f32 * ppn),
            ],
            Stroke::new(1.4_f32, pal.own),
        );
        ringclip.circle_stroke(own_s, 4.0, Stroke::new(1.5_f32, pal.own));
        let clip = p.with_clip_rect(clip_rect);
        let st = self.step;
        let tgt = pal.tgt;
        let o = w2s(s.p1);
        let a = w2s(s.p2);
        if st >= 1 {
            for (q, lbl) in [(o, format!("O {}", self.t1)), (a, format!("A {}", self.t2))] {
                clip.circle_filled(q, 4.5, tgt);
                clip.text(
                    q + vec2(7.0, -7.0),
                    Align2::LEFT_BOTTOM,
                    lbl,
                    FontId::monospace(11.0),
                    tgt,
                );
            }
        }
        if st >= 2 {
            super::arrow(&clip, o, a - o, Stroke::new(2.2_f32, pal.rel));
        }
        if st >= 3 {
            let far = s.p2 + s.v.normalized() * (range * 3.0);
            clip.extend(Shape::dashed_line(
                &[a, w2s(far)],
                Stroke::new(1.3_f32, pal.rel),
                8.0,
                5.0,
            ));
            if s.tcpa.is_finite() && s.tcpa > 0.0 {
                let cp = w2s(s.cpa_pt);
                clip.extend(Shape::dashed_line(
                    &[own_s, cp],
                    Stroke::new(1.2_f32, pal.rel),
                    3.0,
                    3.0,
                ));
                clip.circle_stroke(cp, 5.0, Stroke::new(1.6_f32, pal.rel));
                clip.text(
                    cp + vec2(8.0, 6.0),
                    Align2::LEFT_TOP,
                    format!("CPA {:.2}", s.cpa),
                    FontId::monospace(11.0),
                    pal.rel,
                );
            }
        }
        let w = s.p1 - s.vo * (s.dt / 60.0);
        let ws = w2s(w);
        if st >= 5 {
            super::arrow(&clip, ws, o - ws, Stroke::new(2.0_f32, pal.own));
            clip.text(
                ws + vec2(-8.0, 0.0),
                Align2::RIGHT_CENTER,
                "W",
                FontId::monospace(12.0),
                pal.own,
            );
        }
        if st >= 6 {
            super::arrow(&clip, ws, a - ws, Stroke::new(2.0_f32, tgt));
        }
        if st >= 7 {
            // Cockcroft-Sektor des Ziels
            let sec = COCKCROFT[cockcroft_sector(s.rb2)];
            for b in [sec.0, sec.1] {
                let ang = ((self.own_crs + b) as f32).to_radians();
                clip.extend(Shape::dashed_line(
                    &[
                        own_s,
                        own_s + vec2(ang.sin(), -ang.cos()) * (sheet_range as f32 * ppn),
                    ],
                    Stroke::new(1.0_f32, pal.cur),
                    6.0,
                    6.0,
                ));
            }
        }
        let pm = w2s(s.p2 + s.v * (self.man_after / 60.0));
        if st >= 8 {
            clip.circle_filled(pm, 5.0, pal.ok);
            clip.text(
                pm + vec2(8.0, 0.0),
                Align2::LEFT_CENTER,
                format!("M {}", fmt_clock(s.t2 + self.man_after)),
                FontId::monospace(11.0),
                pal.ok,
            );
        }
        if let Some(m) = man {
            if st >= 9 {
                clip.circle_stroke(
                    own_s,
                    self.cpa_goal as f32 * ppn,
                    Stroke::new(1.2_f32, pal.ok.gamma_multiply(0.8)),
                );
                let cp = w2s(m.cpa_pt);
                super::arrow(&clip, pm, cp - pm, Stroke::new(2.2_f32, pal.ok));
                let far = m.cpa_pt + m.v.normalized() * (range * 2.0);
                clip.extend(Shape::dashed_line(
                    &[cp, w2s(far)],
                    Stroke::new(1.2_f32, pal.ok),
                    8.0,
                    5.0,
                ));
            }
            if st >= 10 {
                let back = s.p2 - m.v.normalized() * (range * 3.0);
                clip.extend(Shape::dashed_line(
                    &[a, w2s(back)],
                    Stroke::new(1.0_f32, pal.ok),
                    4.0,
                    4.0,
                ));
                clip.circle_stroke(
                    ws,
                    (self.own_spd * s.dt / 60.0) as f32 * ppn,
                    Stroke::new(1.0_f32, pal.own.gamma_multiply(0.6)),
                );
                let o2 = w2s(s.p2 - m.v * (s.dt / 60.0));
                super::arrow(&clip, ws, o2 - ws, Stroke::new(2.4_f32, pal.ok));
                clip.text(
                    o2 + vec2(8.0, 0.0),
                    Align2::LEFT_CENTER,
                    "O′",
                    FontId::monospace(12.0),
                    pal.ok,
                );
                if let Some((tr, pr)) = m.resume {
                    let rs = w2s(pr);
                    clip.circle_stroke(rs, 5.0, Stroke::new(1.5_f32, pal.ok));
                    clip.text(
                        rs + vec2(8.0, 0.0),
                        Align2::LEFT_CENTER,
                        format!("R {}", fmt_clock(tr)),
                        FontId::monospace(11.0),
                        pal.ok,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plot::Encounter;

    /// Die Zahlen in den Lösungen der Übungsfragen 12.1–12.3 müssen zur Rechnung
    /// der Vorlage „Übung“ passen.
    #[test]
    fn exercise_matches_answers() {
        let mut st = State::default();
        st.exercise();
        let s = solve(st.own_crs, st.own_spd, &st.target()).unwrap();
        assert!((s.kbr - 164.4).abs() < 0.5, "KBr {}", s.kbr);
        assert!((s.vbr - 12.1).abs() < 0.1, "vBr {}", s.vbr);
        assert!((s.kb - 92.0).abs() < 0.6, "KB {}", s.kb);
        assert!((s.vb - 9.8).abs() < 0.1, "vB {}", s.vb);
        assert!((s.cpa - 0.8).abs() < 0.05, "CPA {}", s.cpa);
        assert!((s.tcpa - 34.0).abs() < 0.6, "TCPA {}", s.tcpa);
        assert!((s.t2 + s.tcpa - 640.0).abs() < 0.6, "CPA-Zeit {}", fmt_clock(s.t2 + s.tcpa));
        assert!((s.bcr.unwrap() - 1.1).abs() < 0.05, "BCR {:?}", s.bcr);
        assert!(matches!(classify(&s, st.own_crs, 2.0), Encounter::CrossingPort));
        let m = st.man(&s).unwrap();
        assert!((m.course - 61.0).abs() < 0.6, "Kurs {}", m.course);
        assert!((m.dk - 31.0).abs() < 0.6, "Kursänderung {}", m.dk);
        assert!((m.dist_m - 5.6).abs() < 0.05, "Distanz {}", m.dist_m);
        assert!((m.cpa - 2.0).abs() < 0.02, "CPA neu {}", m.cpa);
        assert!((m.t_m + m.tcpa - 658.0).abs() < 0.6, "CPA-Zeit neu {}", fmt_clock(m.t_m + m.tcpa));
        assert!((m.resume.unwrap().0 - 662.0).abs() < 0.6, "Wiederaufnahme {:?}", m.resume);
        // Empfindlichkeit: zweite Peilung um 1° geändert
        st.b2 = 352.0;
        let s2 = solve(st.own_crs, st.own_spd, &st.target()).unwrap();
        assert!((s2.cpa - 1.5).abs() < 0.06, "CPA bei 352° {}", s2.cpa);
    }
}
