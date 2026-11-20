//! Plottumgebung: klassisches Radarplotten (Relativbewegung aus zwei Beobachtungen),
//! Vektordreieck, CPA/TCPA/BCR, Manöverplanung mit parallel verschobener
//! Relativbewegung, Wiederaufnahme des alten Kurses, Begründung nach KVR
//! (Sicht / verminderte Sicht) und Cockcroft-Diagramm.

use eframe::egui::{self, vec2, Align2, Color32, FontId, Pos2, RichText, Sense, Shape, Stroke};

use crate::app::{
    angle_edit, angle_edit_enabled, circle_clipped, clip_circle, dashed, fb, label_bg, line, View,
};
use crate::i18n::t;
use crate::util::*;

/// (Bereich, Ringabstand) in nm
const RANGES: [(f64, f64); 6] = [
    (3.0, 0.5),
    (6.0, 1.0),
    (8.0, 1.0),
    (12.0, 2.0),
    (16.0, 2.0),
    (24.0, 4.0),
];

const TCOLORS: [Color32; 6] = [
    Color32::from_rgb(235, 70, 70),
    Color32::from_rgb(60, 150, 255),
    Color32::from_rgb(235, 165, 20),
    Color32::from_rgb(190, 90, 230),
    Color32::from_rgb(20, 185, 130),
    Color32::from_rgb(255, 110, 180),
];
const C_MAN: Color32 = Color32::from_rgb(40, 200, 90);
const C_COCK: Color32 = Color32::from_rgb(255, 200, 60);

// ---------------------------------------------------------------------------
// Daten
// ---------------------------------------------------------------------------

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Obs {
    pub time: String,
    pub dist: f64,
    pub brg: f64,
    /// Peilung relativ (rv) statt rechtweisend (rw)
    pub rel: bool,
}

impl Obs {
    fn new(time: &str, dist: f64, brg: f64) -> Self {
        Self {
            time: time.into(),
            dist,
            brg,
            rel: false,
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PTarget {
    pub name: String,
    pub o1: Obs,
    pub o2: Obs,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Orient {
    North,
    Course,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ManPoint {
    Time,
    Distance,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ManKind {
    Course,
    Speed,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ManGoal {
    /// gewünschten CPA erreichen
    Cpa,
    /// vorgegebenen Kurs / vorgegebene Fahrt prüfen
    Value,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Side {
    Stbd,
    Port,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Visibility {
    InSight,
    Restricted,
}

/// Ergebnis der Auswertung eines Ziels.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Solution {
    pub t1: f64,
    pub t2: f64,
    pub dt: f64,
    pub p1: V2,
    pub p2: V2,
    /// relative Geschwindigkeit (kn)
    pub v: V2,
    pub vo: V2,
    pub vt: V2,
    pub kbr: f64,
    pub vbr: f64,
    pub kb: f64,
    pub vb: f64,
    pub cpa: f64,
    /// Minuten ab 2. Beobachtung (INF = keine Relativbewegung)
    pub tcpa: f64,
    pub cpa_pt: V2,
    pub pcpa: f64,
    pub spcpa: f64,
    pub bcr: Option<f64>,
    pub bct: Option<f64>,
    /// rechtweisende / relative Peilung bei Beobachtung 2
    pub brg2: f64,
    pub rb2: f64,
    pub rng2: f64,
    /// Seitenpeilung des Eigenschiffs vom Ziel aus (+ = an dessen Steuerbordseite)
    pub aspect: f64,
}

#[derive(Clone, Copy)]
pub struct ManResult {
    pub t_m: f64,
    pub pm: V2,
    pub dist_m: f64,
    pub course: f64,
    pub speed: f64,
    /// Kursänderung (+ = Steuerbord)
    pub dk: f64,
    pub vo: V2,
    pub v: V2,
    pub kbr: f64,
    pub vbr: f64,
    pub dkbr: f64,
    pub cpa: f64,
    pub tcpa: f64,
    pub cpa_pt: V2,
    pub pcpa: f64,
    pub spcpa: f64,
    pub bcr: Option<f64>,
    pub bct: Option<f64>,
    /// Wiederaufnahme von Kurs und Fahrt: (Uhrzeit, relative Position)
    pub resume: Option<(f64, V2)>,
}

pub const PLOT_TEMPLATE: &str = "plot_standard";

impl Default for ManParams {
    fn default() -> Self {
        Self {
            point: ManPoint::Distance,
            time: String::new(),
            dist: 5.0,
            kind: ManKind::Course,
            goal: ManGoal::Cpa,
            cpa: 2.0,
            course: 0.0,
            speed: 0.0,
            side: Side::Stbd,
        }
    }
}

/// Speicherbare Voreinstellung der Plottumgebung.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct PlotTemplate {
    pub orient: Orient,
    pub range_idx: usize,
    pub show_heading: bool,
    pub show_vectors: bool,
    pub show_rml: bool,
    pub show_maneuver: bool,
    pub show_cockcroft: bool,
    pub own_course: f64,
    pub own_speed: f64,
    pub cpa_limit: f64,
    pub vis: Visibility,
    pub targets: Vec<PTarget>,
    pub mp: ManParams,
}

/// Werkseinstellung: nur Ziel A, alle Eingaben leer.
impl Default for PlotTemplate {
    fn default() -> Self {
        Self {
            orient: Orient::North,
            range_idx: 3,
            show_heading: true,
            show_vectors: true,
            show_rml: true,
            show_maneuver: true,
            show_cockcroft: false,
            own_course: 0.0,
            own_speed: 0.0,
            cpa_limit: 2.0,
            vis: Visibility::Restricted,
            targets: vec![PTarget {
                name: "A".into(),
                o1: Obs::new("", 0.0, 0.0),
                o2: Obs::new("", 0.0, 0.0),
            }],
            mp: ManParams::default(),
        }
    }
}

/// Uhrzeit "HH:MM", "HH:MM:SS", "HHMM" oder "HH.MM" → Minuten seit 0 Uhr.
pub fn parse_time(s: &str) -> Option<f64> {
    let s = s.trim();
    let parts: Vec<&str> = s.split([':', '.', 'h']).filter(|p| !p.is_empty()).collect();
    let (h, m, sec): (u32, u32, u32) = match parts.len() {
        1 if (3..=4).contains(&parts[0].len()) => {
            let n: u32 = parts[0].parse().ok()?;
            (n / 100, n % 100, 0)
        }
        2 => (parts[0].parse().ok()?, parts[1].parse().ok()?, 0),
        3 => (
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ),
        _ => return None,
    };
    if h > 23 || m > 59 || sec > 59 {
        return None;
    }
    Some(h as f64 * 60.0 + m as f64 + sec as f64 / 60.0)
}

pub fn fmt_clock(min: f64) -> String {
    if !min.is_finite() {
        return "--:--".into();
    }
    let secs = (min.rem_euclid(1440.0) * 60.0).round() as i64;
    let (h, m, s) = (secs / 3600 % 24, secs / 60 % 60, secs % 60);
    if s == 0 {
        format!("{:02}:{:02}", h, m)
    } else {
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
}

fn fmin(m: f64) -> String {
    if m.is_finite() {
        format!("{:.1} min", m)
    } else {
        "---".into()
    }
}

fn obs_pos(o: &Obs, own_course: f64) -> V2 {
    let b = if o.rel { own_course + o.brg } else { o.brg };
    V2::from_bearing(b, o.dist)
}

fn bow_crossing(heading: f64, r: V2, v: V2) -> (Option<f64>, Option<f64>) {
    let d = V2::from_bearing(heading, 1.0);
    let denom = d.cross(v);
    if denom.abs() > 1e-6 {
        let th = -d.cross(r) / denom;
        if th > 0.0 {
            return (Some(d.dot(r + v * th)), Some(th * 60.0));
        }
    }
    (None, None)
}

pub fn solve(own_course: f64, own_speed: f64, tg: &PTarget) -> Result<Solution, &'static str> {
    if tg.o1.time.trim().is_empty() && tg.o2.time.trim().is_empty() {
        return Err(t(
            "Beobachtungen eingeben (Zeit, Distanz, Peilung)",
            "Enter observations (time, distance, bearing)",
        ));
    }
    let t1 = parse_time(&tg.o1.time).ok_or(t(
        "Uhrzeit Beobachtung 1 ungültig",
        "Invalid time observation 1",
    ))?;
    let mut t2 = parse_time(&tg.o2.time).ok_or(t(
        "Uhrzeit Beobachtung 2 ungültig",
        "Invalid time observation 2",
    ))?;
    if t2 < t1 {
        t2 += 1440.0;
    }
    let dt = t2 - t1;
    if dt <= 0.0 {
        return Err(t(
            "Beobachtung 2 muss nach Beobachtung 1 liegen",
            "Observation 2 must be after observation 1",
        ));
    }
    if tg.o1.dist <= 0.0 || tg.o2.dist <= 0.0 {
        return Err(t("Distanzen eingeben", "Enter distances"));
    }
    let p1 = obs_pos(&tg.o1, own_course);
    let p2 = obs_pos(&tg.o2, own_course);
    let v = (p2 - p1) * (60.0 / dt);
    let vo = V2::from_bearing(own_course, own_speed);
    let vt = vo + v;
    let (cpa, th) = cpa_calc(p2, v);
    let cpa_pt = if th.is_finite() { p2 + v * th } else { p2 };
    let (bcr, bct) = bow_crossing(own_course, p2, v);
    let brg2 = p2.bearing();
    Ok(Solution {
        t1,
        t2,
        dt,
        p1,
        p2,
        v,
        vo,
        vt,
        kbr: v.bearing(),
        vbr: v.len(),
        kb: vt.bearing(),
        vb: vt.len(),
        cpa,
        tcpa: th * 60.0,
        cpa_pt,
        pcpa: cpa_pt.bearing(),
        spcpa: norm360(cpa_pt.bearing() - own_course),
        bcr,
        bct,
        brg2,
        rb2: norm360(brg2 - own_course),
        rng2: p2.len(),
        aspect: norm180((-p2).bearing() - vt.bearing()),
    })
}

/// zukünftiger Passierabstand (bei auseinanderlaufender Bewegung = aktueller Abstand)
fn future_cpa(r: V2, v: V2) -> (f64, f64) {
    let (c, th) = cpa_calc(r, v);
    if !th.is_finite() || th < 0.0 {
        (r.len(), 0.0)
    } else {
        (c, th)
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ManParams {
    pub point: ManPoint,
    pub time: String,
    pub dist: f64,
    pub kind: ManKind,
    pub goal: ManGoal,
    pub cpa: f64,
    pub course: f64,
    pub speed: f64,
    pub side: Side,
}

/// Manöverpunkt (Uhrzeit, Minuten nach Beobachtung 2).
fn maneuver_time(sol: &Solution, mp: &ManParams) -> Result<(f64, f64), &'static str> {
    match mp.point {
        ManPoint::Time => {
            let mut tm =
                parse_time(&mp.time).ok_or(t("Manöverzeit ungültig", "Invalid manoeuvre time"))?;
            while tm < sol.t2 - 1e-6 && tm + 1440.0 <= sol.t2 + 720.0 {
                tm += 1440.0;
            }
            if tm < sol.t2 - 1e-6 {
                return Err(t(
                    "Manöverzeit liegt vor Beobachtung 2",
                    "Manoeuvre time is before observation 2",
                ));
            }
            Ok((tm, tm - sol.t2))
        }
        ManPoint::Distance => {
            let d = mp.dist;
            let r = sol.p2;
            let v = sol.v;
            if r.len() <= d {
                return Ok((sol.t2, 0.0));
            }
            let a = v.len2();
            let b = 2.0 * r.dot(v);
            let c = r.len2() - d * d;
            let disc = b * b - 4.0 * a * c;
            if a < 1e-9 || disc < 0.0 {
                return Err(t(
                    "Ziel erreicht diese Distanz nicht",
                    "Target never reaches this distance",
                ));
            }
            let th = (-b - disc.sqrt()) / (2.0 * a);
            if th < 0.0 {
                return Err(t(
                    "Ziel erreicht diese Distanz nicht",
                    "Target never reaches this distance",
                ));
            }
            Ok((sol.t2 + th * 60.0, th * 60.0))
        }
    }
}

pub fn maneuver(
    sol: &Solution,
    own_course: f64,
    own_speed: f64,
    mp: &ManParams,
) -> Result<ManResult, &'static str> {
    let (t_m, tau) = maneuver_time(sol, mp)?;
    let pm = sol.p2 + sol.v * (tau / 60.0);
    let vt = sol.vt;
    let (course, speed) = match (mp.kind, mp.goal) {
        (ManKind::Course, ManGoal::Value) => (norm360(mp.course), own_speed),
        (ManKind::Speed, ManGoal::Value) => (own_course, mp.speed.max(0.0)),
        (ManKind::Course, ManGoal::Cpa) => {
            let c = mp.cpa;
            let dist = pm.len();
            if c >= dist {
                return Err(t(
                    "Gewünschter CPA ≥ Abstand beim Manöver",
                    "Requested CPA ≥ range at manoeuvre",
                ));
            }
            if own_speed <= 0.0 {
                return Err(t("Eigene Fahrt ist 0", "Own speed is 0"));
            }
            let alpha = (c / dist).asin().to_degrees();
            let base = (-pm).bearing();
            let mut cands: Vec<f64> = Vec::new();
            for sgn in [1.0, -1.0] {
                let d = V2::from_bearing(base + sgn * alpha, 1.0);
                let b = vt.dot(d);
                let disc = b * b - vt.len2() + own_speed * own_speed;
                if disc < 0.0 {
                    continue;
                }
                for k in [b + disc.sqrt(), b - disc.sqrt()] {
                    if k > 1e-6 {
                        cands.push((vt - d * k).bearing());
                    }
                }
            }
            let best = cands
                .iter()
                .map(|c| (*c, norm180(*c - own_course)))
                .filter(|(_, dk)| match mp.side {
                    Side::Stbd => *dk > 0.05,
                    Side::Port => *dk < -0.05,
                })
                .min_by(|a, b| {
                    a.1.abs()
                        .partial_cmp(&b.1.abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            match best {
                Some((c, _)) => (c, own_speed),
                None => {
                    return Err(match mp.side {
                        Side::Stbd => t(
                            "CPA mit Kursänderung nach Stb nicht erreichbar",
                            "CPA not reachable with a starboard alteration",
                        ),
                        Side::Port => t(
                            "CPA mit Kursänderung nach Bb nicht erreichbar",
                            "CPA not reachable with a port alteration",
                        ),
                    })
                }
            }
        }
        (ManKind::Speed, ManGoal::Cpa) => {
            let u = V2::from_bearing(own_course, 1.0);
            let max = own_speed.max(1.0) * 1.5 + 5.0;
            let mut best: Option<f64> = None;
            let mut s = 0.0;
            while s <= max {
                let (c, _) = future_cpa(pm, vt - u * s);
                if c >= mp.cpa && best.is_none_or(|b| (s - own_speed).abs() < (b - own_speed).abs())
                {
                    best = Some(s);
                }
                s += 0.05;
            }
            match best {
                Some(s) => (own_course, s),
                None => {
                    return Err(t(
                        "CPA durch Fahrtänderung nicht erreichbar",
                        "CPA not reachable by a speed change",
                    ))
                }
            }
        }
    };
    let vo = V2::from_bearing(course, speed);
    let v = vt - vo;
    let (cpa, th) = future_cpa(pm, v);
    let cpa_pt = pm + v * th;
    let (bcr, bct) = bow_crossing(course, pm, v);

    // Wiederaufnahme von Kurs/Fahrt: sobald der alte Relativkurs den CPA nicht mehr unterschreitet
    let target_cpa = if mp.goal == ManGoal::Cpa { mp.cpa } else { cpa };
    let mut resume = None;
    let mut tr = 0.0;
    while tr <= 600.0 {
        let p = pm + v * (tr / 60.0);
        let (c, _) = future_cpa(p, sol.v);
        if tr > 0.0 && c >= target_cpa - 0.005 {
            resume = Some((t_m + tr, p));
            break;
        }
        tr += 0.1;
    }

    Ok(ManResult {
        t_m,
        pm,
        dist_m: pm.len(),
        course,
        speed,
        dk: norm180(course - own_course),
        vo,
        v,
        kbr: v.bearing(),
        vbr: v.len(),
        dkbr: norm180(v.bearing() - sol.kbr),
        cpa,
        tcpa: th * 60.0,
        cpa_pt,
        pcpa: cpa_pt.bearing(),
        spcpa: norm360(cpa_pt.bearing() - course),
        bcr,
        bct,
        resume,
    })
}

// ---------------------------------------------------------------------------
// KVR-Bewertung
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Encounter {
    NoRisk,
    HeadOn,
    CrossingStbd,
    CrossingPort,
    WeOvertake,
    BeingOvertaken,
}

pub fn classify(sol: &Solution, own_course: f64, cpa_limit: f64) -> Encounter {
    let risk = sol.tcpa.is_finite() && sol.tcpa > 0.0 && sol.cpa < cpa_limit;
    if !risk {
        return Encounter::NoRisk;
    }
    let rb = sol.rb2;
    // wir sind mehr als 22,5° achterlicher als querab des anderen → wir überholen
    if sol.aspect.abs() > 112.5 && sol.vo.len() > sol.vb {
        return Encounter::WeOvertake;
    }
    if (112.5..=247.5).contains(&rb) && sol.vb > sol.vo.len() {
        return Encounter::BeingOvertaken;
    }
    let recip = norm180(sol.kb - own_course - 180.0).abs();
    if recip <= 10.0 && (rb <= 10.0 || rb >= 350.0) {
        return Encounter::HeadOn;
    }
    if rb < 180.0 {
        Encounter::CrossingStbd
    } else {
        Encounter::CrossingPort
    }
}

/// Cockcroft-Sektoren: (von, bis, Kurztext DE, Kurztext EN, Langtext DE, Langtext EN, Richtung +1 Stb / −1 Bb)
type CockSector = (
    f64,
    f64,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    i8,
);
pub const COCKCROFT: [CockSector; 7] = [
    (
        330.0,
        30.0,
        "Stb 60°–90°",
        "Stbd 60°–90°",
        "Kursänderung nach Steuerbord um 60° bis 90°",
        "Alter course to starboard by 60° to 90°",
        1,
    ),
    (
        30.0,
        67.5,
        "Stb → Ziel ≥30° an Bb-Bug",
        "Stbd → tgt ≥30° on port bow",
        "Nach Steuerbord drehen, bis das Ziel mindestens 30° an Backbord voraus steht",
        "Turn to starboard until the vessel is at least 30° on the port bow",
        1,
    ),
    (
        67.5,
        112.5,
        "Bb → Ziel (fast) achteraus",
        "Port → tgt (nearly) astern",
        "Nach Backbord drehen, bis das Ziel achteraus oder nahezu achteraus steht",
        "Turn to port until the vessel is astern or nearly astern",
        -1,
    ),
    (
        112.5,
        150.0,
        "Bb → Ziel achteraus",
        "Port → tgt astern",
        "Nach Backbord drehen, bis das Ziel achteraus steht",
        "Turn to port until the vessel is astern",
        -1,
    ),
    (
        150.0,
        210.0,
        "Bb ca. 30°",
        "Port ~30°",
        "Kursänderung nach Backbord um etwa 30°",
        "Turn to port about 30°",
        -1,
    ),
    (
        210.0,
        292.5,
        "Stb → Ziel achteraus",
        "Stbd → tgt astern",
        "Nach Steuerbord drehen, bis das Ziel achteraus steht",
        "Turn to starboard until the vessel is astern",
        1,
    ),
    (
        292.5,
        330.0,
        "Stb → Ziel querab Bb",
        "Stbd → tgt abeam port",
        "Nach Steuerbord drehen, bis das Ziel querab an Backbord steht",
        "Turn to starboard until the vessel is abeam to port",
        1,
    ),
];

pub fn cockcroft_sector(rb: f64) -> usize {
    let rb = norm360(rb);
    COCKCROFT
        .iter()
        .position(|s| {
            if s.0 > s.1 {
                rb >= s.0 || rb < s.1
            } else {
                rb >= s.0 && rb < s.1
            }
        })
        .unwrap_or(0)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Head,
    Text,
    Ok,
    Bad,
    Warn,
}

/// Begründung als Liste (Art, Text).
pub fn reasoning(
    sol: &Solution,
    own_course: f64,
    own_speed: f64,
    cpa_limit: f64,
    vis: Visibility,
    man: Option<&ManResult>,
    target_name: &str,
) -> Vec<(Mark, String)> {
    let mut r: Vec<(Mark, String)> = Vec::new();
    let enc = classify(sol, own_course, cpa_limit);
    let rb = sol.rb2;
    let forward = !(90.0..=270.0).contains(&rb);

    r.push((
        Mark::Head,
        tf!("Lage Ziel {}", "Situation target {}", target_name),
    ));
    r.push((
        Mark::Text,
        tf!(
            "Peilung {} (Seitenpeilung {}), Distanz {:.2} sm. KB {} / vB {:.1} kn. CPA {:.2} sm, TCPA {}.",
            "Bearing {} (relative {}), range {:.2} nm. Target course {} / speed {:.1} kn. CPA {:.2} nm, TCPA {}.",
            fb(sol.brg2),
            fb(rb),
            sol.rng2,
            fb(sol.kb),
            sol.vb,
            sol.cpa,
            fmin(sol.tcpa)
        ),
    ));
    let bearing_steady = sol.tcpa.is_finite() && sol.tcpa > 0.0 && sol.cpa < 0.5;
    if enc == Encounter::NoRisk {
        r.push((
            Mark::Ok,
            tf!(
                "Regel 7: Keine Kollisionsgefahr – CPA ≥ {:.1} sm bzw. Ziel läuft ab. Kurs und Fahrt beibehalten, weiter beobachten.",
                "Rule 7: No risk of collision – CPA ≥ {:.1} nm or target opening. Keep course and speed, keep monitoring.",
                cpa_limit
            ),
        ));
    } else {
        r.push((
            Mark::Warn,
            if bearing_steady {
                tf!(
                    "Regel 7(d)(i): Peilung steht nahezu bei abnehmender Distanz: Kollisionsgefahr (CPA {:.2} sm < {:.1} sm).",
                    "Rule 7(d)(i): Bearing nearly steady with decreasing range: risk of collision (CPA {:.2} nm < {:.1} nm).",
                    sol.cpa,
                    cpa_limit
                )
            } else {
                tf!(
                    "Regel 7: Kollisionsgefahr bzw. Nahbereichslage – CPA {:.2} sm < {:.1} sm.",
                    "Rule 7: Risk of collision / close-quarters situation – CPA {:.2} nm < {:.1} nm.",
                    sol.cpa,
                    cpa_limit
                )
            },
        ));
    }

    let sector = COCKCROFT[cockcroft_sector(rb)];
    let mut recommended: i8 = 0;
    match vis {
        Visibility::Restricted => {
            r.push((
                Mark::Head,
                t(
                    "Verminderte Sicht – Regel 19 (Ziel nur durch Radar geortet)",
                    "Restricted visibility – Rule 19 (vessel detected by radar alone)",
                )
                .into(),
            ));
            r.push((Mark::Text, t(
                "Regel 19(a)/(b): Es gibt keine Kurshalte- oder Ausweichpflicht nach Regel 11–18. Sichere Geschwindigkeit nach Regel 6, Maschine manöverbereit.",
                "Rule 19(a)/(b): Rules 11–18 (stand-on / give-way) do not apply. Safe speed according to Rule 6, engines ready for immediate manoeuvre.",
            ).into()));
            if enc != Encounter::NoRisk {
                r.push((Mark::Text, t(
                    "Regel 19(d): Bei Nahbereichslage und/oder Kollisionsgefahr rechtzeitig Maßnahmen zur Vermeidung treffen.",
                    "Rule 19(d): If a close-quarters situation is developing and/or risk of collision exists, take avoiding action in ample time.",
                ).into()));
                let being_overtaken_by_us = enc == Encounter::WeOvertake;
                if forward && !being_overtaken_by_us {
                    r.push((Mark::Warn, t(
                        "Regel 19(d)(i): Ziel vorlicher als querab (und wird nicht von uns überholt): Kursänderung nach Backbord vermeiden.",
                        "Rule 19(d)(i): Vessel forward of the beam (not being overtaken): avoid an alteration of course to port.",
                    ).into()));
                } else if forward {
                    r.push((Mark::Text, t(
                        "Regel 19(d)(i): Ziel wird von uns überholt – die Einschränkung nach Bb gilt hier nicht.",
                        "Rule 19(d)(i): The vessel is being overtaken by us – the port restriction does not apply.",
                    ).into()));
                } else {
                    let side = if rb < 180.0 {
                        t("Steuerbord", "starboard")
                    } else {
                        t("Backbord", "port")
                    };
                    r.push((Mark::Warn, tf!(
                        "Regel 19(d)(ii): Ziel querab oder achterlicher als querab an {}: keine Kursänderung auf das Ziel zu (nicht nach {}).",
                        "Rule 19(d)(ii): Vessel abeam or abaft the beam on the {} side: avoid an alteration of course towards it (not to {}).",
                        side,
                        side
                    )));
                }
                r.push((Mark::Text, t(
                    "Regel 19(e): Nebelsignal vorlicher als querab gehört: Fahrt auf das Mindestmaß verringern, notfalls aufstoppen.",
                    "Rule 19(e): Fog signal heard forward of the beam: reduce speed to the minimum, take all way off if necessary.",
                ).into()));
                recommended = sector.6;
                r.push((
                    Mark::Head,
                    t("Cockcroft-Diagramm", "Cockcroft diagram").into(),
                ));
                r.push((
                    Mark::Text,
                    tf!(
                        "Ziel in Sektor {:05.1}°–{:05.1}° (Seitenpeilung {}): {}.",
                        "Vessel in sector {:05.1}°–{:05.1}° (relative bearing {}): {}.",
                        sector.0,
                        sector.1,
                        fb(rb),
                        if crate::i18n::is_de() {
                            sector.4
                        } else {
                            sector.5
                        }
                    ),
                ));
                let d = sol.rng2;
                r.push((
                    if d < 3.0 { Mark::Warn } else { Mark::Text },
                    if d < 3.0 {
                        tf!("Distanz {:.1} sm < 3 sm: Notmanöver (escape action) – sofort, deutlich, ggf. Fahrt reduzieren.", "Range {:.1} nm < 3 nm: escape action – immediately, substantially, reduce speed if necessary.", d)
                    } else {
                        tf!("Normale Ausweichmanöver vorzugsweise bei 4–6 sm (aktuell {:.1} sm).", "Normal turns preferably at 4–6 nm (currently {:.1} nm).", d)
                    },
                ));
            }
        }
        Visibility::InSight => {
            r.push((
                Mark::Head,
                t(
                    "In Sicht – Regeln 11 bis 18",
                    "In sight of one another – Rules 11 to 18",
                )
                .into(),
            ));
            match enc {
                Encounter::NoRisk => {}
                Encounter::HeadOn => {
                    recommended = 1;
                    r.push((Mark::Text, t(
                        "Regel 14: Entgegengesetzte oder fast entgegengesetzte Kurse – jedes Fahrzeug ändert seinen Kurs nach Steuerbord, so dass beide einander an Backbordseite passieren.",
                        "Rule 14: Reciprocal or nearly reciprocal courses – each vessel alters course to starboard so that each passes on the port side of the other.",
                    ).into()));
                }
                Encounter::CrossingStbd => {
                    recommended = 1;
                    r.push((Mark::Text, t(
                        "Regel 15: Kreuzende Kurse, Ziel an unserer Steuerbordseite: wir sind ausweichpflichtig und vermeiden es, vor dem Bug des anderen zu kreuzen.",
                        "Rule 15: Crossing situation, vessel on our starboard side: we keep out of the way and avoid crossing ahead of her.",
                    ).into()));
                    r.push((Mark::Text, t(
                        "Regel 16: Frühzeitig und durchgreifend handeln, um gut klarzukommen – i. d. R. Kursänderung nach Stb (hinter dem Heck passieren) oder Fahrt reduzieren.",
                        "Rule 16: Take early and substantial action to keep well clear – usually alter to starboard (pass astern) or reduce speed.",
                    ).into()));
                }
                Encounter::CrossingPort => {
                    r.push((Mark::Text, t(
                        "Regel 15/17: Ziel an unserer Backbordseite ist ausweichpflichtig. Regel 17(a)(i): Wir halten Kurs und Geschwindigkeit bei.",
                        "Rule 15/17: The vessel on our port side is the give-way vessel. Rule 17(a)(i): we keep our course and speed.",
                    ).into()));
                    r.push((Mark::Text, t(
                        "Regel 17(a)(ii): Manöver erst, wenn klar ist, dass der andere nicht ausweicht – dann gemäß 17(c) nicht nach Backbord ändern. 17(b): Manöver des letzten Augenblicks.",
                        "Rule 17(a)(ii): manoeuvre only once it is apparent the other is not taking action – then per 17(c) do not alter to port. 17(b): last-moment action.",
                    ).into()));
                }
                Encounter::WeOvertake => {
                    r.push((Mark::Text, t(
                        "Regel 13: Wir überholen (von mehr als 22,5° achterlicher als querab): wir sind ausweichpflichtig, gleich ob nach Stb oder Bb, bis wir klar vorbei sind.",
                        "Rule 13: We are overtaking (from more than 22.5° abaft her beam): we keep out of the way, to either side, until finally past and clear.",
                    ).into()));
                }
                Encounter::BeingOvertaken => {
                    r.push((Mark::Text, t(
                        "Regel 13/17: Wir werden überholt: Kurshalter: Kurs und Geschwindigkeit beibehalten.",
                        "Rule 13/17: We are being overtaken: stand-on vessel: keep course and speed.",
                    ).into()));
                }
            }
            if enc != Encounter::NoRisk {
                r.push((Mark::Text, t(
                    "Regel 8: Manöver rechtzeitig, deutlich (für andere erkennbar, ≥ 30°) und so, dass in sicherem Abstand passiert wird; Wirksamkeit überwachen.",
                    "Rule 8: Action in ample time, large enough to be readily apparent (≥ 30°), resulting in passing at a safe distance; check its effectiveness.",
                ).into()));
            }
        }
    }

    if let Some(m) = man {
        r.push((
            Mark::Head,
            t(
                "Bewertung des geplanten Manövers",
                "Assessment of the planned manoeuvre",
            )
            .into(),
        ));
        let dir: i8 = if m.dk > 0.5 {
            1
        } else if m.dk < -0.5 {
            -1
        } else {
            0
        };
        let dv = m.speed - own_speed;
        r.push((
            Mark::Text,
            tf!(
                "Manöver um {} bei {:.1} sm: Kurs {} ({:+.1}°), Fahrt {:.1} kn ({:+.1} kn).",
                "Manoeuvre at {} at {:.1} nm: course {} ({:+.1}°), speed {:.1} kn ({:+.1} kn).",
                fmt_clock(m.t_m),
                m.dist_m,
                fb(m.course),
                m.dk,
                m.speed,
                dv
            ),
        ));
        if dir != 0 {
            r.push((
                if m.dk.abs() >= 30.0 { Mark::Ok } else { Mark::Warn },
                if m.dk.abs() >= 30.0 {
                    t("Regel 8(b): Kursänderung groß genug, um für andere deutlich erkennbar zu sein.", "Rule 8(b): Alteration large enough to be readily apparent.").into()
                } else {
                    t("Regel 8(b): Kursänderung < 30° – möglicherweise nicht deutlich erkennbar (Folge kleiner Änderungen vermeiden).", "Rule 8(b): Alteration < 30° – may not be readily apparent (avoid a succession of small alterations).").into()
                },
            ));
        } else if dv < 0.0 {
            let big = own_speed > 0.0 && -dv / own_speed >= 0.5;
            r.push((
                if big { Mark::Ok } else { Mark::Warn },
                if big {
                    t(
                        "Regel 8(b)/(e): Deutliche Fahrtreduzierung.",
                        "Rule 8(b)/(e): Substantial reduction of speed.",
                    )
                    .into()
                } else {
                    t(
                        "Regel 8(b): Fahrtänderung gering – für andere kaum erkennbar.",
                        "Rule 8(b): Small speed change – hardly apparent to others.",
                    )
                    .into()
                },
            ));
        }
        r.push((
            if m.cpa >= cpa_limit {
                Mark::Ok
            } else {
                Mark::Bad
            },
            tf!(
                "Regel 8(d): Neuer CPA {:.2} sm {} Grenzwert {:.1} sm.",
                "Rule 8(d): New CPA {:.2} nm {} limit {:.1} nm.",
                m.cpa,
                if m.cpa >= cpa_limit { "≥" } else { "<" },
                cpa_limit
            ),
        ));
        match vis {
            Visibility::Restricted => {
                if dir == -1 && forward && enc != Encounter::WeOvertake {
                    r.push((Mark::Bad, t(
                        "Regel 19(d)(i) verletzt: Kursänderung nach Backbord gegenüber einem Fahrzeug vorlicher als querab.",
                        "Rule 19(d)(i) violated: alteration to port for a vessel forward of the beam.",
                    ).into()));
                } else if !forward && ((dir == 1 && rb < 180.0) || (dir == -1 && rb >= 180.0)) {
                    r.push((Mark::Bad, t(
                        "Regel 19(d)(ii) verletzt: Kursänderung auf ein Fahrzeug zu, das querab oder achterlicher als querab ist.",
                        "Rule 19(d)(ii) violated: alteration towards a vessel abeam or abaft the beam.",
                    ).into()));
                } else if dir != 0 {
                    r.push((
                        Mark::Ok,
                        t(
                            "Regel 19(d): Richtung der Kursänderung zulässig.",
                            "Rule 19(d): Direction of alteration permitted.",
                        )
                        .into(),
                    ));
                }
            }
            Visibility::InSight => match enc {
                Encounter::HeadOn if dir == -1 => {
                    r.push((
                        Mark::Bad,
                        t(
                            "Regel 14 verletzt: Kursänderung nach Backbord bei Begegnung.",
                            "Rule 14 violated: alteration to port in a head-on situation.",
                        )
                        .into(),
                    ));
                }
                Encounter::CrossingStbd if m.bcr.is_some_and(|b| b > 0.0) => {
                    r.push((Mark::Bad, t(
                        "Regel 15: Nach dem Manöver würden wir vor dem Bug des anderen kreuzen – vermeiden!",
                        "Rule 15: After the manoeuvre we would cross ahead of the other vessel – avoid!",
                    ).into()));
                }
                Encounter::CrossingPort | Encounter::BeingOvertaken
                    if dir != 0 || dv.abs() > 0.1 =>
                {
                    r.push((
                        Mark::Warn,
                        t(
                            "Regel 17: Als Kurshalter nur nach 17(a)(ii)/(b) manövrieren.",
                            "Rule 17: As stand-on vessel only manoeuvre per 17(a)(ii)/(b).",
                        )
                        .into(),
                    ));
                    if enc == Encounter::CrossingPort && dir == -1 {
                        r.push((Mark::Bad, t("Regel 17(c) verletzt: Kursänderung nach Backbord gegenüber Fahrzeug an Backbord.", "Rule 17(c) violated: alteration to port for a vessel on our port side.").into()));
                    }
                }
                _ => {
                    if dir != 0 && enc != Encounter::NoRisk {
                        r.push((
                            Mark::Ok,
                            t(
                                "Richtung der Kursänderung regelkonform.",
                                "Direction of alteration complies with the rules.",
                            )
                            .into(),
                        ));
                    }
                }
            },
        }
        if recommended != 0 && dir != 0 {
            r.push((
                if recommended == dir {
                    Mark::Ok
                } else {
                    Mark::Warn
                },
                if recommended == dir {
                    t(
                        "Entspricht der empfohlenen Drehrichtung.",
                        "Matches the recommended direction of turn.",
                    )
                    .into()
                } else {
                    t(
                        "Weicht von der empfohlenen Drehrichtung ab.",
                        "Differs from the recommended direction of turn.",
                    )
                    .into()
                },
            ));
        }
        if let Some((tr, _)) = m.resume {
            r.push((
                Mark::Text,
                tf!(
                    "Wiederaufnahme von Kurs/Fahrt ab ca. {} möglich (Ziel passiert dann mit ausreichendem Abstand).",
                    "Resume course/speed from approx. {} (target will then pass at a sufficient distance).",
                    fmt_clock(tr)
                ),
            ));
        }
    }
    r
}

// ---------------------------------------------------------------------------
// Zustand & Oberfläche
// ---------------------------------------------------------------------------

pub struct PlotState {
    pub orient: Orient,
    pub range_idx: usize,
    pub show_heading: bool,
    pub show_vectors: bool,
    pub show_rml: bool,
    pub show_maneuver: bool,
    pub show_cockcroft: bool,
    pub show_reason: bool,
    pub own_course: f64,
    pub own_speed: f64,
    pub cpa_limit: f64,
    pub vis: Visibility,
    pub targets: Vec<PTarget>,
    pub sel: usize,
    pub mp: ManParams,
    cursor: Option<V2>,
    ctx_pos: Option<V2>,
    status: Option<String>,
}

impl PlotState {
    pub fn new() -> Self {
        let mut s = Self {
            orient: Orient::North,
            range_idx: 3,
            show_heading: true,
            show_vectors: true,
            show_rml: true,
            show_maneuver: true,
            show_cockcroft: false,
            show_reason: false,
            own_course: 0.0,
            own_speed: 12.0,
            cpa_limit: 2.0,
            vis: Visibility::Restricted,
            targets: Vec::new(),
            sel: 0,
            mp: ManParams {
                point: ManPoint::Distance,
                time: "12:12".into(),
                dist: 5.0,
                kind: ManKind::Course,
                goal: ManGoal::Cpa,
                cpa: 2.0,
                course: 60.0,
                speed: 6.0,
                side: Side::Stbd,
            },
            cursor: None,
            ctx_pos: None,
            status: None,
        };
        s.load_standard();
        s
    }

    /// Aktuelle Einstellungen (inkl. Ziele) als Template.
    pub fn template(&self) -> PlotTemplate {
        PlotTemplate {
            orient: self.orient,
            range_idx: self.range_idx,
            show_heading: self.show_heading,
            show_vectors: self.show_vectors,
            show_rml: self.show_rml,
            show_maneuver: self.show_maneuver,
            show_cockcroft: self.show_cockcroft,
            own_course: self.own_course,
            own_speed: self.own_speed,
            cpa_limit: self.cpa_limit,
            vis: self.vis,
            targets: self.targets.clone(),
            mp: self.mp.clone(),
        }
    }

    pub fn apply_template(&mut self, tp: PlotTemplate) {
        self.orient = tp.orient;
        self.range_idx = tp.range_idx.min(RANGES.len() - 1);
        self.show_heading = tp.show_heading;
        self.show_vectors = tp.show_vectors;
        self.show_rml = tp.show_rml;
        self.show_maneuver = tp.show_maneuver;
        self.show_cockcroft = tp.show_cockcroft;
        self.own_course = tp.own_course;
        self.own_speed = tp.own_speed;
        self.cpa_limit = tp.cpa_limit;
        self.vis = tp.vis;
        self.targets = if tp.targets.is_empty() {
            PlotTemplate::default().targets
        } else {
            tp.targets
        };
        self.mp = tp.mp;
        self.sel = 0;
    }

    /// Gespeichertes Standard-Template laden, sonst Werkseinstellung (nur Ziel A, leer).
    pub fn load_standard(&mut self) {
        let tp = crate::templates::load::<PlotTemplate>(PLOT_TEMPLATE).unwrap_or_default();
        self.apply_template(tp);
    }

    pub fn load_example(&mut self) {
        self.own_course = 0.0;
        self.own_speed = 12.0;
        self.targets = vec![
            PTarget {
                name: "A".into(),
                o1: Obs::new("12:00", 9.0, 40.0),
                o2: Obs::new("12:06", 7.6, 39.0),
            },
            PTarget {
                name: "B".into(),
                o1: Obs::new("12:00", 7.0, 300.0),
                o2: Obs::new("12:06", 6.4, 293.0),
            },
        ];
        self.sel = 0;
    }

    fn range(&self) -> (f64, f64) {
        RANGES[self.range_idx]
    }

    fn rot(&self) -> f64 {
        match self.orient {
            Orient::North => 0.0,
            Orient::Course => self.own_course,
        }
    }

    fn next_name(&self) -> String {
        for c in 'A'..='Z' {
            let n = c.to_string();
            if !self.targets.iter().any(|t| t.name == n) {
                return n;
            }
        }
        format!("{}", self.targets.len() + 1)
    }

    pub fn solutions(&self) -> Vec<Result<Solution, &'static str>> {
        self.targets
            .iter()
            .map(|t| solve(self.own_course, self.own_speed, t))
            .collect()
    }

    pub fn maneuver_result(
        &self,
        sols: &[Result<Solution, &'static str>],
    ) -> Option<Result<ManResult, &'static str>> {
        let sol = sols.get(self.sel)?.as_ref().ok()?;
        Some(maneuver(sol, self.own_course, self.own_speed, &self.mp))
    }

    // --- Panels -------------------------------------------------------------

    pub fn left_panel(&mut self, ui: &mut egui::Ui) {
        crate::app::section(ui, t("DARSTELLUNG", "DISPLAY"));
        ui.horizontal(|ui| {
            ui.label(t("Ausrichtung:", "Orientation:"));
            ui.selectable_value(&mut self.orient, Orient::North, "North Up");
            ui.selectable_value(&mut self.orient, Orient::Course, "Course Up");
        });
        ui.horizontal(|ui| {
            ui.label(t("Bereich:", "Range:"));
            egui::ComboBox::from_id_salt("plot_range")
                .selected_text(format!("{} nm", self.range().0))
                .show_ui(ui, |ui| {
                    for (i, (r, _)) in RANGES.iter().enumerate() {
                        ui.selectable_value(&mut self.range_idx, i, format!("{} nm", r));
                    }
                });
        });
        ui.checkbox(&mut self.show_heading, t("Vorausrichtung", "Heading line"));
        ui.checkbox(
            &mut self.show_rml,
            t("Relativbewegung (RML)", "Relative motion line (RML)"),
        );
        ui.checkbox(
            &mut self.show_vectors,
            t("Vektordreieck", "Vector triangle"),
        );
        ui.checkbox(
            &mut self.show_maneuver,
            t(
                "Manöver (parallel verschoben)",
                "Manoeuvre (parallel shifted)",
            ),
        );
        ui.checkbox(
            &mut self.show_cockcroft,
            t(
                "Cockcroft-Diagramm (vor Manöver)",
                "Cockcroft diagram (before manoeuvre)",
            ),
        );

        crate::app::section(ui, t("EIGENSCHIFF", "OWN SHIP"));
        egui::Grid::new("plot_own").num_columns(2).show(ui, |ui| {
            ui.label(t("Kurs", "Course"));
            let mut c = self.own_course;
            if angle_edit(ui, &mut c, 0.5, 1, "°").changed() {
                self.own_course = norm360(c);
            }
            ui.end_row();
            ui.label(t("Fahrt", "Speed"));
            ui.add(
                egui::DragValue::new(&mut self.own_speed)
                    .range(0.0..=40.0)
                    .speed(0.1)
                    .fixed_decimals(1)
                    .suffix(" kn"),
            );
            ui.end_row();
            ui.label(t("CPA-Grenze", "CPA limit"));
            ui.add(
                egui::DragValue::new(&mut self.cpa_limit)
                    .range(0.1..=10.0)
                    .speed(0.05)
                    .fixed_decimals(1)
                    .suffix(" nm"),
            );
            ui.end_row();
        });

        crate::app::section(ui, t("SICHT", "VISIBILITY"));
        ui.radio_value(
            &mut self.vis,
            Visibility::InSight,
            t("In Sicht (Regeln 11–18)", "In sight (Rules 11–18)"),
        );
        ui.radio_value(
            &mut self.vis,
            Visibility::Restricted,
            t("Keine Sicht (Regel 19)", "Not in sight (Rule 19)"),
        );
        ui.add_space(8.0);
        let label = if self.show_reason {
            t("✖ Begründung schließen", "✖ Close reasoning")
        } else {
            t("📖 Begründung anzeigen", "📖 Show reasoning")
        };
        if ui
            .add_sized(
                [ui.available_width(), 30.0],
                egui::Button::new(RichText::new(label).strong()),
            )
            .clicked()
        {
            self.show_reason = !self.show_reason;
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button(t("Beispiel laden", "Load example")).clicked() {
                self.load_example();
                self.status = None;
            }
            if ui
                .button(t("Neu (Standard)", "New (standard)"))
                .on_hover_text(t(
                    "Neu beginnen mit dem Standard-Template",
                    "Start over with the standard template",
                ))
                .clicked()
            {
                self.load_standard();
                self.status = None;
            }
        });
        ui.horizontal(|ui| {
            if ui
                .button(RichText::new("💾").size(15.0))
                .on_hover_text(t(
                    "Als Standard speichern: aktuelle Einstellungen inkl. Ziele als Standard-Template speichern – wird beim Start geladen.",
                    "Save as standard: the current settings incl. targets as standard template – loaded at start-up.",
                ))
                .clicked()
            {
                self.status = Some(match crate::templates::save(PLOT_TEMPLATE, &self.template()) {
                    Ok(_) => t("Standard-Template gespeichert.", "Standard template saved.").into(),
                    Err(e) => tf!("Speichern fehlgeschlagen: {}", "Saving failed: {}", e),
                });
            }
            if ui
                .button(RichText::new("↺").size(15.0))
                .on_hover_text(t(
                    "Standard zurücksetzen: Standard-Template auf Werkseinstellung zurücksetzen (nur Ziel A, leer). Die aktuelle Eingabe bleibt erhalten.",
                    "Reset standard: back to factory settings (target A only, empty). Current input is kept.",
                ))
                .clicked()
            {
                self.status = Some(match crate::templates::remove(PLOT_TEMPLATE) {
                    Ok(()) => t("Standard auf Werkseinstellung zurückgesetzt.", "Standard reset to factory settings.").into(),
                    Err(e) => tf!("Zurücksetzen fehlgeschlagen: {}", "Reset failed: {}", e),
                });
            }
        });
        if let Some(st) = &self.status {
            ui.label(RichText::new(st).small());
        }
        ui.add_space(6.0);
        ui.label(
            RichText::new(t(
                "Rechtsklick auf das Plottblatt setzt Beobachtungen des gewählten Ziels.",
                "Right-click on the plotting sheet sets observations of the selected target.",
            ))
            .small()
            .italics(),
        );
    }

    pub fn right_panel(&mut self, ui: &mut egui::Ui) {
        crate::app::section(ui, t("ZIELDATEN", "TARGET DATA"));
        ui.horizontal(|ui| {
            for (i, tg) in self.targets.iter().enumerate() {
                let c = TCOLORS[i % TCOLORS.len()];
                let txt = RichText::new(format!(" {} ", tg.name)).strong().color(c);
                if ui.selectable_label(self.sel == i, txt).clicked() {
                    self.sel = i;
                }
            }
            if self.targets.len() < 12
                && ui
                    .button("➕")
                    .on_hover_text(t("Ziel hinzufügen", "Add target"))
                    .clicked()
            {
                let name = self.next_name();
                let (o1, o2) = match self.targets.get(self.sel) {
                    Some(tg) => (
                        Obs {
                            dist: 0.0,
                            ..tg.o1.clone()
                        },
                        Obs {
                            dist: 0.0,
                            ..tg.o2.clone()
                        },
                    ),
                    None => (Obs::new("12:00", 0.0, 0.0), Obs::new("12:06", 0.0, 0.0)),
                };
                self.targets.push(PTarget { name, o1, o2 });
                self.sel = self.targets.len() - 1;
            }
            if self.targets.len() > 1
                && ui
                    .button("➖")
                    .on_hover_text(t("Gewähltes Ziel entfernen", "Remove selected target"))
                    .clicked()
            {
                self.targets.remove(self.sel);
                self.sel = self.sel.min(self.targets.len() - 1);
            }
        });
        if let Some(tg) = self.targets.get_mut(self.sel) {
            for (k, o) in [(1, &mut tg.o1), (2, &mut tg.o2)] {
                ui.label(RichText::new(tf!("Beobachtung {}", "Observation {}", k)).strong());
                egui::Grid::new(format!("obs{k}"))
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(t("Zeit", "Time"));
                        let ok = parse_time(&o.time).is_some();
                        let mut te = egui::TextEdit::singleline(&mut o.time)
                            .desired_width(70.0)
                            .hint_text("HH:MM");
                        if !ok {
                            te = te.text_color(Color32::from_rgb(230, 80, 80));
                        }
                        ui.add(te);
                        ui.end_row();
                        ui.label(t("Distanz", "Distance"));
                        ui.add(
                            egui::DragValue::new(&mut o.dist)
                                .range(0.0..=48.0)
                                .speed(0.02)
                                .fixed_decimals(2)
                                .suffix(" nm"),
                        );
                        ui.end_row();
                        ui.label(t("Peilung", "Bearing"));
                        ui.horizontal(|ui| {
                            let mut b = o.brg;
                            if angle_edit(ui, &mut b, 0.25, 1, "°").changed() {
                                o.brg = norm360(b);
                            }
                            ui.selectable_value(&mut o.rel, false, t("rw", "T"))
                                .on_hover_text(t("rechtweisend", "true bearing"));
                            ui.selectable_value(&mut o.rel, true, t("rv", "R"))
                                .on_hover_text(t(
                                    "relativ zur Vorausrichtung (Seitenpeilung)",
                                    "relative to heading",
                                ));
                        });
                        ui.end_row();
                    });
            }
        }

        // erst nach dem Bearbeiten der Ziele berechnen (Ziele können eben entfernt worden sein)
        let sols = self.solutions();
        crate::app::section(ui, t("ERGEBNISSE", "RESULTS"));
        match sols.get(self.sel) {
            Some(Ok(s)) => {
                egui::Grid::new("plot_res")
                    .num_columns(4)
                    .striped(true)
                    .show(ui, |ui| {
                        let kv = |ui: &mut egui::Ui, k: &str, v: String, tip: &str| {
                            ui.label(RichText::new(k).monospace()).on_hover_text(tip);
                            ui.label(RichText::new(v).monospace().strong());
                        };
                        kv(
                            ui,
                            "Δt",
                            format!("{:.1} min", s.dt),
                            t(
                                "Zeit zwischen den Beobachtungen",
                                "Time between observations",
                            ),
                        );
                        kv(
                            ui,
                            "KBr",
                            fb(s.kbr),
                            t("Kurs der Relativbewegung", "Course of relative motion"),
                        );
                        ui.end_row();
                        kv(
                            ui,
                            "vBr",
                            format!("{:.1} kn", s.vbr),
                            t(
                                "Geschwindigkeit der Relativbewegung",
                                "Speed of relative motion",
                            ),
                        );
                        kv(
                            ui,
                            "KB",
                            fb(s.kb),
                            t("Kurs des Ziels (wahr)", "Target true course"),
                        );
                        ui.end_row();
                        kv(
                            ui,
                            "vB",
                            format!("{:.1} kn", s.vb),
                            t("Fahrt des Ziels (wahr)", "Target true speed"),
                        );
                        kv(
                            ui,
                            "CPA",
                            format!("{:.2} nm", s.cpa),
                            t("Passierabstand", "Closest point of approach"),
                        );
                        ui.end_row();
                        kv(
                            ui,
                            "TCPA",
                            fmin(s.tcpa),
                            t(
                                "Zeit bis CPA ab Beobachtung 2",
                                "Time to CPA from observation 2",
                            ),
                        );
                        kv(
                            ui,
                            "tCPA",
                            fmt_clock(s.t2 + s.tcpa),
                            t("Uhrzeit des CPA", "Clock time of CPA"),
                        );
                        ui.end_row();
                        kv(
                            ui,
                            "PCPA",
                            fb(s.pcpa),
                            t("Peilung des CPA-Punkts (rw)", "True bearing of CPA point"),
                        );
                        kv(
                            ui,
                            "SPCPA",
                            fb(s.spcpa),
                            t(
                                "Seitenpeilung des CPA-Punkts",
                                "Relative bearing of CPA point",
                            ),
                        );
                        ui.end_row();
                        kv(
                            ui,
                            "BCR",
                            s.bcr.map_or("---".into(), |b| format!("{:+.2} nm", b)),
                            t(
                                "Abstand beim Kreuzen der Vorausrichtung (+ voraus, − achteraus)",
                                "Bow crossing range (+ ahead, − astern)",
                            ),
                        );
                        kv(
                            ui,
                            "BCT",
                            s.bct.map_or("---".into(), fmin),
                            t(
                                "Zeit bis zum Kreuzen der Vorausrichtung",
                                "Bow crossing time",
                            ),
                        );
                        ui.end_row();
                        kv(
                            ui,
                            "tBC",
                            s.bct.map_or("--:--".into(), |b| fmt_clock(s.t2 + b)),
                            t("Uhrzeit des Kreuzens", "Clock time of bow crossing"),
                        );
                        kv(
                            ui,
                            t("Asp.", "Asp."),
                            format!(
                                "{} {:.0}°",
                                if s.aspect >= 0.0 {
                                    t("grün", "green")
                                } else {
                                    t("rot", "red")
                                },
                                s.aspect.abs()
                            ),
                            t(
                                "Aspekt: unter welcher Seitenpeilung das Ziel uns sieht",
                                "Aspect: relative bearing at which the target sees us",
                            ),
                        );
                        ui.end_row();
                    });
                let enc = classify(s, self.own_course, self.cpa_limit);
                let (txt, col) = encounter_label(enc);
                ui.label(
                    RichText::new(txt)
                        .strong()
                        .color(crate::app::adapt(ui, col)),
                );
            }
            Some(Err(e)) => {
                ui.label(RichText::new(*e).color(Color32::from_rgb(230, 120, 60)));
            }
            None => {}
        }

        crate::app::section(ui, t("MANÖVERPLANUNG", "MANOEUVRE PLANNING"));
        let mp = &mut self.mp;
        ui.horizontal(|ui| {
            ui.label(t("Manöverpunkt:", "Manoeuvre point:"));
            ui.selectable_value(&mut mp.point, ManPoint::Time, t("Zeit", "Time"));
            ui.selectable_value(&mut mp.point, ManPoint::Distance, t("Distanz", "Distance"));
            match mp.point {
                ManPoint::Time => {
                    ui.add(
                        egui::TextEdit::singleline(&mut mp.time)
                            .desired_width(70.0)
                            .hint_text("HH:MM"),
                    );
                }
                ManPoint::Distance => {
                    ui.add(
                        egui::DragValue::new(&mut mp.dist)
                            .range(0.1..=24.0)
                            .speed(0.05)
                            .fixed_decimals(1)
                            .suffix(" nm"),
                    );
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label(t("Manöverart:", "Type:"));
            ui.selectable_value(
                &mut mp.kind,
                ManKind::Course,
                t("Kursänderung", "Course change"),
            );
            ui.selectable_value(
                &mut mp.kind,
                ManKind::Speed,
                t("Fahrtänderung", "Speed change"),
            );
        });
        ui.horizontal(|ui| {
            ui.radio_value(&mut mp.goal, ManGoal::Cpa, t("für CPA", "for CPA"));
            ui.add_enabled(
                mp.goal == ManGoal::Cpa,
                egui::DragValue::new(&mut mp.cpa)
                    .range(0.1..=10.0)
                    .speed(0.05)
                    .fixed_decimals(1)
                    .suffix(" nm"),
            );
            if mp.kind == ManKind::Course && mp.goal == ManGoal::Cpa {
                ui.selectable_value(&mut mp.side, Side::Stbd, t("Stb", "Stbd"));
                ui.selectable_value(&mut mp.side, Side::Port, t("Bb", "Port"));
            }
        });
        ui.horizontal(|ui| match mp.kind {
            ManKind::Course => {
                ui.radio_value(&mut mp.goal, ManGoal::Value, t("für Kurs", "for course"));
                let mut c = mp.course;
                if angle_edit_enabled(ui, mp.goal == ManGoal::Value, &mut c, 0.5, 0, "°").changed()
                {
                    mp.course = norm360(c);
                }
            }
            ManKind::Speed => {
                ui.radio_value(&mut mp.goal, ManGoal::Value, t("für Fahrt", "for speed"));
                ui.add_enabled(
                    mp.goal == ManGoal::Value,
                    egui::DragValue::new(&mut mp.speed)
                        .range(0.0..=40.0)
                        .speed(0.1)
                        .fixed_decimals(1)
                        .suffix(" kn"),
                );
            }
        });

        match self.maneuver_result(&sols) {
            Some(Ok(m)) => {
                ui.label(
                    RichText::new(t("NACH DEM MANÖVER", "AFTER MANOEUVRE"))
                        .small()
                        .strong(),
                );
                egui::Grid::new("plot_man")
                    .num_columns(4)
                    .striped(true)
                    .show(ui, |ui| {
                        let kv = |ui: &mut egui::Ui, k: &str, v: String| {
                            ui.label(RichText::new(k).monospace());
                            ui.label(RichText::new(v).monospace().strong());
                        };
                        kv(ui, t("Zeit", "Time"), fmt_clock(m.t_m));
                        kv(ui, t("Dist.", "Range"), format!("{:.2} nm", m.dist_m));
                        ui.end_row();
                        kv(
                            ui,
                            "ΔK",
                            format!(
                                "{:05.1}° {}",
                                m.dk.abs(),
                                if m.dk >= 0.0 {
                                    t("Stb", "Stbd")
                                } else {
                                    t("Bb", "Port")
                                }
                            ),
                        );
                        kv(ui, t("Kurs", "Course"), fb(m.course));
                        ui.end_row();
                        kv(ui, t("Fahrt", "Speed"), format!("{:.1} kn", m.speed));
                        kv(ui, "KBr", fb(m.kbr));
                        ui.end_row();
                        kv(ui, "vBr", format!("{:.1} kn", m.vbr));
                        kv(ui, "Δ KBr", format!("{:+.1}°", m.dkbr));
                        ui.end_row();
                        kv(ui, "CPA", format!("{:.2} nm", m.cpa));
                        kv(ui, "TCPA", fmin(m.tcpa));
                        ui.end_row();
                        kv(ui, "tCPA", fmt_clock(m.t_m + m.tcpa));
                        kv(ui, "PCPA", fb(m.pcpa));
                        ui.end_row();
                        kv(ui, "SPCPA", fb(m.spcpa));
                        kv(
                            ui,
                            "BCR",
                            m.bcr.map_or("---".into(), |b| format!("{:+.2} nm", b)),
                        );
                        ui.end_row();
                        kv(ui, "BCT", m.bct.map_or("---".into(), fmin));
                        kv(
                            ui,
                            t("Wiederaufn.", "Resume"),
                            m.resume.map_or("--:--".into(), |r| fmt_clock(r.0)),
                        );
                        ui.end_row();
                    });
                // Auswirkung auf die übrigen Ziele
                let others: Vec<(usize, &Solution)> = sols
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != self.sel)
                    .filter_map(|(i, s)| s.as_ref().ok().map(|s| (i, s)))
                    .collect();
                if !others.is_empty() {
                    ui.label(
                        RichText::new(t(
                            "ÜBRIGE ZIELE NACH DEM MANÖVER",
                            "OTHER TARGETS AFTER MANOEUVRE",
                        ))
                        .small()
                        .strong(),
                    );
                    egui::Grid::new("plot_others").striped(true).show(ui, |ui| {
                        for h in [t("Ziel", "Tgt"), "CPA", "tCPA", t("vorher", "before")] {
                            ui.label(RichText::new(h).small());
                        }
                        ui.end_row();
                        for (i, s) in others {
                            let (cpa, tcpa) = other_after(s, &m);
                            let c = TCOLORS[i % TCOLORS.len()];
                            let name = self.targets.get(i).map_or("?", |t| t.name.as_str());
                            ui.label(RichText::new(name).strong().color(c));
                            let bad = cpa < self.cpa_limit;
                            let col = if bad {
                                Color32::from_rgb(230, 70, 70)
                            } else {
                                ui.visuals().text_color()
                            };
                            ui.label(
                                RichText::new(format!("{:.2} nm", cpa))
                                    .monospace()
                                    .color(col),
                            );
                            ui.label(RichText::new(fmt_clock(m.t_m + tcpa)).monospace());
                            ui.label(RichText::new(format!("{:.2} nm", s.cpa)).monospace());
                            ui.end_row();
                        }
                    });
                }
            }
            Some(Err(e)) => {
                ui.label(RichText::new(e).color(Color32::from_rgb(230, 120, 60)));
            }
            None => {}
        }
    }

    pub fn reason_window(&mut self, ctx: &egui::Context) {
        if !self.show_reason {
            return;
        }
        let sols = self.solutions();
        let man = self.maneuver_result(&sols).and_then(|r| r.ok());
        let mut open = true;
        egui::Window::new(t("Begründung", "Reasoning"))
            .open(&mut open)
            .default_width(520.0)
            .default_pos(ctx.screen_rect().center() - vec2(260.0, 250.0))
            .vscroll(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t("Sicht:", "Visibility:"));
                    ui.radio_value(&mut self.vis, Visibility::InSight, t("In Sicht", "In sight"));
                    ui.radio_value(&mut self.vis, Visibility::Restricted, t("Keine Sicht", "Not in sight"));
                });
                ui.separator();
                let Some(tg) = self.targets.get(self.sel) else { return };
                match sols.get(self.sel) {
                    Some(Ok(s)) => {
                        let mm = if self.show_maneuver { man.as_ref() } else { None };
                        for (mark, text) in reasoning(s, self.own_course, self.own_speed, self.cpa_limit, self.vis, mm, &tg.name) {
                            let (prefix, col) = match mark {
                                Mark::Head => ("", None),
                                Mark::Text => ("•  ", None),
                                Mark::Ok => ("✔  ", Some(Color32::from_rgb(40, 190, 90))),
                                Mark::Bad => ("✘  ", Some(Color32::from_rgb(235, 70, 70))),
                                Mark::Warn => ("⚠  ", Some(Color32::from_rgb(240, 170, 30))),
                            };
                            if mark == Mark::Head {
                                ui.add_space(4.0);
                                ui.label(RichText::new(text).strong().size(15.0));
                            } else {
                                let mut rt = RichText::new(format!("{}{}", prefix, text));
                                if let Some(c) = col {
                                    rt = rt.color(crate::app::adapt(ui, c));
                                }
                                ui.label(rt);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        ui.label(*e);
                    }
                    None => {}
                }
                ui.add_space(6.0);
                ui.label(RichText::new(t(
                    "Hinweis: Ausbildungshilfe – ersetzt nicht die eigene Lagebeurteilung nach KVR.",
                    "Note: training aid – does not replace your own assessment under the COLREGs.",
                )).small().italics());
            });
        if !open {
            self.show_reason = false;
        }
    }

    // --- Plottblatt ---------------------------------------------------------

    pub fn sheet(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let resp = ui.allocate_rect(rect, Sense::click());
        let side = rect.width().min(rect.height());
        let radius = (side / 2.0 - 40.0).max(60.0);
        let (range, ring) = self.range();
        let view = View {
            center: rect.center(),
            radius,
            range,
            rot: self.rot(),
            cw: V2::ZERO,
        };
        let p = ui.painter_at(rect);
        let dark = ui.visuals().dark_mode;
        let (sheet, ink, faint) = if dark {
            (
                Color32::from_rgb(18, 34, 64),
                Color32::from_rgb(200, 215, 235),
                Color32::from_rgb(70, 95, 130),
            )
        } else {
            (
                Color32::from_rgb(252, 252, 246),
                Color32::from_rgb(30, 40, 55),
                Color32::from_rgb(170, 180, 195),
            )
        };
        let own_col = if dark { Color32::WHITE } else { Color32::BLACK };

        // Eingaben
        self.cursor = resp
            .hover_pos()
            .filter(|h| view.inside(*h))
            .map(|h| view.s2w(h));
        if resp.secondary_clicked() {
            self.ctx_pos = resp.interact_pointer_pos().map(|h| view.s2w(h));
        }
        resp.context_menu(|ui| {
            let Some(w) = self.ctx_pos else {
                ui.close_menu();
                return;
            };
            ui.label(
                RichText::new(format!("{}  {:.2} nm", fb(w.bearing()), w.len()))
                    .monospace()
                    .strong(),
            );
            let own_course = self.own_course;
            if let Some(tg) = self.targets.get_mut(self.sel) {
                for (k, o) in [(1, &mut tg.o1), (2, &mut tg.o2)] {
                    if ui
                        .button(tf!(
                            "Beobachtung {} von Ziel {} hierher",
                            "Observation {} of target {} here",
                            k,
                            tg.name.clone()
                        ))
                        .clicked()
                    {
                        o.dist = (w.len() * 100.0).round() / 100.0;
                        o.brg = if o.rel {
                            norm360((w.bearing() - own_course) * 10.0).round() / 10.0
                        } else {
                            (w.bearing() * 10.0).round() / 10.0
                        };
                        ui.close_menu();
                    }
                }
            }
        });

        // Blatt
        p.circle_filled(
            view.center,
            radius + 36.0,
            if dark {
                Color32::from_rgb(22, 40, 70)
            } else {
                Color32::from_rgb(228, 230, 234)
            },
        );
        p.circle_filled(view.center, radius, sheet);
        let n = (range / ring).round() as usize;
        for k in 1..=n {
            let r = (k as f64 * ring) as f32 * view.ppn();
            p.circle_stroke(
                view.center,
                r,
                Stroke::new(
                    if k == n { 1.5_f32 } else { 0.8 },
                    if k == n { ink } else { faint },
                ),
            );
            let lp = view.center + view.dir_s(self.rot() + 45.0) * r;
            p.text(
                lp,
                Align2::LEFT_BOTTOM,
                format!("{}", k as f64 * ring),
                FontId::monospace(10.0),
                faint,
            );
        }
        for deg in 0..360 {
            let d = view.dir_s(deg as f64);
            let len = if deg % 10 == 0 {
                10.0
            } else if deg % 5 == 0 {
                6.0
            } else {
                3.0
            };
            p.line_segment(
                [view.center + d * radius, view.center + d * (radius + len)],
                Stroke::new(1.0_f32, ink),
            );
            if deg % 10 == 0 && (radius > 300.0 || deg % 30 == 0) {
                p.text(
                    view.center + d * (radius + 22.0),
                    Align2::CENTER_CENTER,
                    format!("{:03}", deg),
                    FontId::monospace(11.0),
                    ink,
                );
            }
            if deg % 30 == 0 {
                line(
                    &p,
                    &view,
                    view.center,
                    view.center + d * radius,
                    Stroke::new(0.5_f32, faint),
                );
            }
        }
        if self.show_heading {
            line(
                &p,
                &view,
                view.center,
                view.center + view.dir_s(self.own_course) * radius,
                Stroke::new(1.6_f32, own_col),
            );
        }
        p.circle_stroke(view.center, 4.0, Stroke::new(1.5_f32, own_col));

        let sols = self.solutions();

        // Cockcroft (vor dem Manöver)
        if self.show_cockcroft {
            let rbsel = sols
                .get(self.sel)
                .and_then(|s| s.as_ref().ok())
                .map(|s| s.rb2);
            draw_cockcroft(&p, &view, self.own_course, rbsel, dark);
        }

        // Ziele
        for (i, (tg, sol)) in self.targets.iter().zip(sols.iter()).enumerate() {
            let col = TCOLORS[i % TCOLORS.len()];
            let selected = i == self.sel;
            let w: f32 = if selected { 2.2 } else { 1.4 };
            let Ok(s) = sol else {
                // nur vorhandene Beobachtungen markieren
                for o in [&tg.o1, &tg.o2] {
                    if o.dist > 0.0 {
                        let q = view.w2s(obs_pos(o, self.own_course));
                        p.circle_stroke(q, 4.0, Stroke::new(1.5_f32, col));
                    }
                }
                continue;
            };
            let a = view.w2s(s.p1);
            let b = view.w2s(s.p2);
            if self.show_rml {
                line(&p, &view, a, b, Stroke::new(w, col));
                let far = s.p2 + s.v.normalized() * (range * 3.0);
                dashed(
                    &p,
                    &view,
                    b,
                    view.w2s(far),
                    Stroke::new(w * 0.7, col),
                    8.0,
                    5.0,
                );
                arrow_head(&p, a, b, col, 9.0);
                if s.tcpa.is_finite() && s.tcpa > 0.0 {
                    let c = view.w2s(s.cpa_pt);
                    if view.inside(c) {
                        dashed(
                            &p,
                            &view,
                            view.center,
                            c,
                            Stroke::new(1.0_f32, col),
                            3.0,
                            3.0,
                        );
                        p.circle_stroke(c, 4.5, Stroke::new(1.6_f32, col));
                        p.text(
                            c + vec2(7.0, 6.0),
                            Align2::LEFT_TOP,
                            format!("CPA {:.2}", s.cpa),
                            FontId::monospace(11.0),
                            col,
                        );
                    }
                }
            }
            if self.show_vectors {
                let wpt = s.p1 - s.vo * (s.dt / 60.0);
                let ws = view.w2s(wpt);
                line(&p, &view, ws, a, Stroke::new(1.6_f32, own_col));
                arrow_head(&p, ws, a, own_col, 8.0);
                line(
                    &p,
                    &view,
                    ws,
                    b,
                    Stroke::new(1.6_f32, col.gamma_multiply(0.8)),
                );
                arrow_head(&p, ws, b, col, 8.0);
                if view.inside(ws) {
                    p.text(
                        ws + vec2(-6.0, 0.0),
                        Align2::RIGHT_CENTER,
                        "W",
                        FontId::monospace(11.0),
                        own_col,
                    );
                }
            }
            for (pt, o, k) in [(a, &tg.o1, "1"), (b, &tg.o2, "2")] {
                if view.inside(pt) {
                    p.circle_filled(pt, if selected { 4.5 } else { 3.5 }, col);
                    let tstr = parse_time(&o.time).map(fmt_clock).unwrap_or_default();
                    p.text(
                        pt + vec2(7.0, -7.0),
                        Align2::LEFT_BOTTOM,
                        format!("{}{} {}", tg.name, k, tstr),
                        FontId::monospace(11.0),
                        col,
                    );
                }
            }
        }

        // Manöver mit parallel verschobener Relativbewegung
        if self.show_maneuver {
            if let (Some(Ok(s)), Some(Ok(m))) = (sols.get(self.sel), self.maneuver_result(&sols)) {
                let col = C_MAN;
                let pm = view.w2s(m.pm);
                if self.mp.goal == ManGoal::Cpa {
                    circle_clipped(
                        &p,
                        &view,
                        view.center,
                        self.mp.cpa as f32 * view.ppn(),
                        Stroke::new(1.2_f32, col.gamma_multiply(0.8)),
                        true,
                    );
                }
                // neue Relativbewegung ab Manöverpunkt
                line(&p, &view, pm, view.w2s(m.cpa_pt), Stroke::new(2.2_f32, col));
                arrow_head(&p, pm, view.w2s(m.cpa_pt), col, 9.0);
                let far = m.cpa_pt + m.v.normalized() * (range * 3.0);
                dashed(
                    &p,
                    &view,
                    view.w2s(m.cpa_pt),
                    view.w2s(far),
                    Stroke::new(1.4_f32, col),
                    8.0,
                    5.0,
                );
                let c = view.w2s(m.cpa_pt);
                if view.inside(c) {
                    p.circle_stroke(c, 4.5, Stroke::new(1.6_f32, col));
                    dashed(
                        &p,
                        &view,
                        view.center,
                        c,
                        Stroke::new(1.0_f32, col),
                        3.0,
                        3.0,
                    );
                    p.text(
                        c + vec2(7.0, 6.0),
                        Align2::LEFT_TOP,
                        format!("CPA' {:.2}", m.cpa),
                        FontId::monospace(11.0),
                        col,
                    );
                }
                p.circle_filled(pm, 5.0, col);
                p.text(
                    pm + vec2(8.0, 0.0),
                    Align2::LEFT_CENTER,
                    format!("M {}", fmt_clock(m.t_m)),
                    FontId::monospace(11.0),
                    col,
                );

                // Vektorkonstruktion: Linie parallel zur neuen Relativbewegung durch A (Beobachtung 2)
                let wpt = s.p1 - s.vo * (s.dt / 60.0);
                let o_new = s.p2 - m.v * (s.dt / 60.0);
                let ws = view.w2s(wpt);
                let a2 = view.w2s(s.p2);
                let back = s.p2 - m.v.normalized() * (range * 3.0);
                dashed(
                    &p,
                    &view,
                    a2,
                    view.w2s(back),
                    Stroke::new(0.9_f32, col.gamma_multiply(0.7)),
                    4.0,
                    4.0,
                );
                let os = view.w2s(o_new);
                line(&p, &view, ws, os, Stroke::new(1.8_f32, col));
                arrow_head(&p, ws, os, col, 8.0);
                line(&p, &view, os, a2, Stroke::new(1.4_f32, col));
                arrow_head(&p, os, a2, col, 8.0);
                if view.inside(os) {
                    p.text(
                        os + vec2(6.0, 0.0),
                        Align2::LEFT_CENTER,
                        "O'",
                        FontId::monospace(11.0),
                        col,
                    );
                }
                // Wiederaufnahme
                if let Some((tr, pr)) = m.resume {
                    let rs = view.w2s(pr);
                    if view.inside(rs) {
                        p.circle_stroke(rs, 5.0, Stroke::new(1.6_f32, col));
                        p.text(
                            rs + vec2(8.0, 0.0),
                            Align2::LEFT_CENTER,
                            format!("R {}", fmt_clock(tr)),
                            FontId::monospace(11.0),
                            col,
                        );
                        let far = pr + s.v.normalized() * (range * 3.0);
                        dashed(
                            &p,
                            &view,
                            rs,
                            view.w2s(far),
                            Stroke::new(1.0_f32, col.gamma_multiply(0.6)),
                            2.0,
                            4.0,
                        );
                    }
                }
                // übrige Ziele: neue Relativbewegung ab Manöverzeit
                for (i, sol) in sols.iter().enumerate() {
                    if i == self.sel {
                        continue;
                    }
                    if let Ok(o) = sol {
                        let dtm = (m.t_m - o.t2) / 60.0;
                        let po = o.p2 + o.v * dtm;
                        let vn = o.vt - m.vo;
                        let a = view.w2s(po);
                        let b = view.w2s(po + vn.normalized() * (range * 3.0));
                        dashed(
                            &p,
                            &view,
                            a,
                            b,
                            Stroke::new(1.0_f32, TCOLORS[i % TCOLORS.len()].gamma_multiply(0.7)),
                            6.0,
                            6.0,
                        );
                    }
                }
            }
        }

        // Cursor + Ecken
        if let Some(c) = self.cursor {
            p.text(
                view.w2s(c) + vec2(12.0, 12.0),
                Align2::LEFT_TOP,
                format!("{} {:.2}nm", fb(c.bearing()), c.len()),
                FontId::monospace(11.0),
                ink,
            );
        }
        let tl = rect.left_top() + vec2(12.0, 10.0);
        let orient = match self.orient {
            Orient::North => "NORTH UP",
            Orient::Course => "COURSE UP",
        };
        label_bg(
            &p,
            tl,
            Align2::LEFT_TOP,
            &format!("RADARPLOT  {}  {} nm", orient, range),
            13.0,
            Color32::from_rgb(160, 190, 200),
        );
        let vis = match self.vis {
            Visibility::InSight => t("IN SICHT", "IN SIGHT"),
            Visibility::Restricted => t("KEINE SICHT", "NOT IN SIGHT"),
        };
        label_bg(
            &p,
            rect.right_top() + vec2(-12.0, 10.0),
            Align2::RIGHT_TOP,
            &format!(
                "{}  {:.1} kn   {}",
                fb(self.own_course),
                self.own_speed,
                vis
            ),
            13.0,
            Color32::from_rgb(160, 190, 200),
        );
        let _ = clip_circle;
    }
}

fn other_after(s: &Solution, m: &ManResult) -> (f64, f64) {
    let dtm = (m.t_m - s.t2) / 60.0;
    let po = s.p2 + s.v * dtm;
    let vn = s.vt - m.vo;
    let (c, th) = future_cpa(po, vn);
    (c, th * 60.0)
}

pub fn encounter_label(e: Encounter) -> (&'static str, Color32) {
    match e {
        Encounter::NoRisk => (
            t("Keine Kollisionsgefahr", "No risk of collision"),
            Color32::from_rgb(40, 190, 90),
        ),
        Encounter::HeadOn => (
            t(
                "Kollisionsgefahr: Begegnung (Regel 14)",
                "Risk: head-on (Rule 14)",
            ),
            Color32::from_rgb(235, 70, 70),
        ),
        Encounter::CrossingStbd => (
            t(
                "Kollisionsgefahr: Kreuzer von Stb (wir weichen aus)",
                "Risk: crossing from starboard (we give way)",
            ),
            Color32::from_rgb(235, 70, 70),
        ),
        Encounter::CrossingPort => (
            t(
                "Kollisionsgefahr: Kreuzer von Bb (wir sind Kurshalter)",
                "Risk: crossing from port (we stand on)",
            ),
            Color32::from_rgb(240, 170, 30),
        ),
        Encounter::WeOvertake => (
            t(
                "Kollisionsgefahr: wir überholen (Regel 13)",
                "Risk: we are overtaking (Rule 13)",
            ),
            Color32::from_rgb(235, 70, 70),
        ),
        Encounter::BeingOvertaken => (
            t(
                "Kollisionsgefahr: wir werden überholt",
                "Risk: we are being overtaken",
            ),
            Color32::from_rgb(240, 170, 30),
        ),
    }
}

fn arrow_head(p: &egui::Painter, a: Pos2, b: Pos2, col: Color32, size: f32) {
    let d = b - a;
    let len = d.length();
    if len < 2.0 {
        return;
    }
    let u = d / len;
    let n = vec2(-u.y, u.x);
    let pts = vec![
        b,
        b - u * size + n * size * 0.45,
        b - u * size - n * size * 0.45,
    ];
    p.add(Shape::convex_polygon(pts, col, Stroke::NONE));
}

fn draw_cockcroft(p: &egui::Painter, v: &View, heading: f64, rb_sel: Option<f64>, dark: bool) {
    let col = if dark {
        C_COCK
    } else {
        Color32::from_rgb(170, 110, 0)
    };
    let r = v.radius;
    let sel = rb_sel.map(cockcroft_sector);
    // markierter Sektor
    if let Some(si) = sel {
        let s = COCKCROFT[si];
        let span = norm360(s.1 - s.0);
        let mut pts = vec![v.center];
        let n = 24;
        for k in 0..=n {
            let b = heading + s.0 + span * k as f64 / n as f64;
            pts.push(v.center + v.dir_s(b) * r);
        }
        p.add(Shape::convex_polygon(
            pts,
            col.gamma_multiply(0.18),
            Stroke::NONE,
        ));
    }
    for (i, s) in COCKCROFT.iter().enumerate() {
        let b = heading + s.0;
        let st = if s.0 == 330.0 || s.0 == 30.0 || s.0 == 112.5 || s.0 == 292.5 {
            Stroke::new(1.6_f32, col)
        } else {
            Stroke::new(1.0_f32, col.gamma_multiply(0.8))
        };
        line(p, v, v.center, v.center + v.dir_s(b) * r, st);
        let mid = heading + s.0 + norm360(s.1 - s.0) / 2.0;
        let lp = v.center + v.dir_s(mid) * (r * 0.7);
        let txt = if crate::i18n::is_de() { s.2 } else { s.3 };
        let size = if Some(i) == sel { 13.0 } else { 11.0 };
        label_bg(p, lp, Align2::CENTER_CENTER, txt, size, col);
    }
    // 112,5°–292,5°-Linie (querab-Grenze) betonen, 3 sm und 4–6 sm
    let ppn = v.ppn();
    if 3.0 < v.range {
        circle_clipped(
            p,
            v,
            v.center,
            3.0 * ppn,
            Stroke::new(1.2_f32, Color32::from_rgb(230, 80, 80)),
            true,
        );
        label_bg(
            p,
            v.center + v.dir_s(heading + 180.0) * (3.0 * ppn),
            Align2::CENTER_CENTER,
            t("< 3 sm: Notmanöver", "< 3 nm: escape action"),
            10.0,
            Color32::from_rgb(230, 80, 80),
        );
    }
    for d in [4.0, 6.0] {
        if d < v.range {
            circle_clipped(
                p,
                v,
                v.center,
                d as f32 * ppn,
                Stroke::new(1.0_f32, Color32::from_rgb(60, 190, 110)),
                true,
            );
        }
    }
    if 6.0 < v.range {
        label_bg(
            p,
            v.center + v.dir_s(heading + 200.0) * (5.0 * ppn),
            Align2::CENTER_CENTER,
            t("4–6 sm: normale Manöver", "4–6 nm: normal turns"),
            10.0,
            Color32::from_rgb(60, 190, 110),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tgt(t1: &str, d1: f64, b1: f64, t2: &str, d2: f64, b2: f64) -> PTarget {
        PTarget {
            name: "A".into(),
            o1: Obs::new(t1, d1, b1),
            o2: Obs::new(t2, d2, b2),
        }
    }

    /// Entfernen eines Ziels im selben Frame darf nicht abstürzen.
    #[test]
    fn remove_target_then_render_panel() {
        let mut st = PlotState::new();
        st.load_example();
        st.sel = 1;
        let ctx = egui::Context::default();
        let mut removed = false;
        for frame in 0..3 {
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if frame == 1 && !removed {
                        removed = true;
                        st.targets.remove(st.sel);
                        st.sel = st.sel.min(st.targets.len() - 1);
                    }
                    st.right_panel(ui);
                    st.left_panel(ui);
                    st.sheet(ui);
                });
            });
        }
        assert_eq!(st.targets.len(), 1);
    }

    #[test]
    fn factory_template_is_single_empty_target() {
        let tp = PlotTemplate::default();
        assert_eq!(tp.targets.len(), 1);
        assert_eq!(tp.targets[0].name, "A");
        assert!(tp.targets[0].o1.time.is_empty() && tp.targets[0].o1.dist == 0.0);
        assert_eq!(tp.own_speed, 0.0);
        // leeres Ziel liefert eine Eingabeaufforderung statt Fehler/Absturz
        assert!(solve(0.0, 0.0, &tp.targets[0]).is_err());
    }

    #[test]
    fn template_save_load_reset() {
        let dir = std::env::temp_dir().join(format!("radar_tpl_test_{}", std::process::id()));
        std::env::set_var("RADAR_CONFIG_DIR", &dir);
        let mut st = PlotState::new();
        st.load_example();
        st.own_course = 123.0;
        crate::templates::save(PLOT_TEMPLATE, &st.template()).unwrap();
        let mut st2 = PlotState::new();
        st2.load_standard();
        assert_eq!(st2.targets.len(), 2);
        assert_eq!(st2.own_course, 123.0);
        crate::templates::remove(PLOT_TEMPLATE).unwrap();
        st2.load_standard();
        assert_eq!(st2.targets.len(), 1);
        assert_eq!(st2.own_course, 0.0);
        // unvollständige/alte Datei: fehlende Felder = Werkseinstellung
        let tp: PlotTemplate = serde_json::from_str(r#"{"own_speed": 9.5}"#).unwrap();
        assert_eq!(tp.own_speed, 9.5);
        assert_eq!(tp.targets.len(), 1);
        std::env::remove_var("RADAR_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn time_parsing() {
        assert_eq!(parse_time("12:06"), Some(726.0));
        assert_eq!(parse_time("1206"), Some(726.0));
        assert_eq!(parse_time("12.06"), Some(726.0));
        assert_eq!(parse_time("25:00"), None);
        assert_eq!(fmt_clock(726.0), "12:06");
    }

    #[test]
    fn stationary_target_appears_reciprocal() {
        // Ziel steht still, wir fahren 000° 12 kn: Relativbewegung 180° mit 12 kn
        let s = solve(0.0, 12.0, &tgt("12:00", 6.0, 0.0, "12:06", 4.8, 0.0)).unwrap();
        assert!((s.vbr - 12.0).abs() < 1e-6);
        assert!((s.kbr - 180.0).abs() < 1e-6);
        assert!(s.vb < 1e-6);
        assert!(s.cpa < 1e-6);
        assert!((s.tcpa - 24.0).abs() < 1e-6);
    }

    #[test]
    fn course_change_reaches_requested_cpa() {
        let s = solve(0.0, 12.0, &tgt("12:00", 9.0, 40.0, "12:06", 7.6, 39.0)).unwrap();
        let mp = ManParams {
            point: ManPoint::Distance,
            time: String::new(),
            dist: 6.0,
            kind: ManKind::Course,
            goal: ManGoal::Cpa,
            cpa: 2.0,
            course: 0.0,
            speed: 0.0,
            side: Side::Stbd,
        };
        let m = maneuver(&s, 0.0, 12.0, &mp).unwrap();
        assert!(m.dk > 0.0, "Stb-Manöver erwartet");
        assert!((m.cpa - 2.0).abs() < 0.02, "CPA {}", m.cpa);
        assert!((m.dist_m - 6.0).abs() < 0.01);
        assert!(m.resume.is_some());
    }

    #[test]
    fn speed_change_reaches_requested_cpa() {
        let s = solve(0.0, 12.0, &tgt("12:00", 9.0, 40.0, "12:06", 7.6, 39.0)).unwrap();
        let mp = ManParams {
            point: ManPoint::Distance,
            time: String::new(),
            dist: 6.0,
            kind: ManKind::Speed,
            goal: ManGoal::Cpa,
            cpa: 1.5,
            course: 0.0,
            speed: 0.0,
            side: Side::Stbd,
        };
        let m = maneuver(&s, 0.0, 12.0, &mp).unwrap();
        assert!(m.speed < 12.0);
        assert!(m.cpa >= 1.5 - 0.01);
    }

    #[test]
    fn cockcroft_sectors() {
        assert_eq!(cockcroft_sector(0.0), 0);
        assert_eq!(cockcroft_sector(345.0), 0);
        assert_eq!(cockcroft_sector(45.0), 1);
        assert_eq!(cockcroft_sector(90.0), 2);
        assert_eq!(cockcroft_sector(120.0), 3);
        assert_eq!(cockcroft_sector(180.0), 4);
        assert_eq!(cockcroft_sector(250.0), 5);
        assert_eq!(cockcroft_sector(300.0), 6);
    }

    #[test]
    fn rule19_flags_port_turn_forward_of_beam() {
        let s = solve(0.0, 12.0, &tgt("12:00", 9.0, 40.0, "12:06", 7.6, 39.0)).unwrap();
        let mp = ManParams {
            point: ManPoint::Distance,
            time: String::new(),
            dist: 6.0,
            kind: ManKind::Course,
            goal: ManGoal::Value,
            cpa: 2.0,
            course: 300.0,
            speed: 0.0,
            side: Side::Port,
        };
        let m = maneuver(&s, 0.0, 12.0, &mp).unwrap();
        let r = reasoning(&s, 0.0, 12.0, 2.0, Visibility::Restricted, Some(&m), "A");
        assert!(r
            .iter()
            .any(|(k, txt)| *k == Mark::Bad && txt.contains("19(d)(i)")));
        assert_eq!(classify(&s, 0.0, 2.0), Encounter::CrossingStbd);
    }
}
