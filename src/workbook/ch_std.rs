//! Kapitel 13: Weg – Zeit – Fahrt. Eine gemeinsame Aufgabe wird mit den klassischen
//! Hilfsmitteln gelöst: Nomogramm (Linealmethode), Zirkelmethode auf der
//! logarithmischen Skala des Plottblatts und Rechenschieber nach Art des Portland
//! Speed-Time-Distance Calculators. Als Variante folgt die Rechenscheibe.

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Pos2, Rect, RichText, Sense, Shape,
    Stroke,
};

use super::{demo_flag, formula, merke, pal, see_also, sub, text, try_it, Chapter, Pal};
use crate::i18n::t;
use crate::util::Rng;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Unknown {
    D,
    T,
    S,
}

/// Woran der Stechzirkel gerade gehalten wird.
#[derive(Clone, Copy, PartialEq)]
enum Grip {
    A,
    B,
    /// am Griff: beide Spitzen, Greifpunkt relativ zu Spitze A (Dekaden)
    Both(f64),
}

pub struct State {
    // gemeinsame Aufgabe
    s: f64,
    tmin: f64,
    d: f64,
    unknown: Unknown,
    hours: bool,
    hide: bool,
    seed: u64,
    // Nomogramm
    drag_scale: Option<usize>,
    // Stechzirkel: Stellung der Spitzen A und B (log10 des Skalenwerts)
    zk: [f64; 2],
    zk_step: usize,
    zk_play: Option<f32>,
    zk_grip: Option<Grip>,
    // Rechenschieber: Versatz des Schiebers (Dekaden)
    pl_o: f64,
    pl_show: bool,
    pl_drag: bool,
    /// Greifpunkt (x) und Versatz beim Drücken
    pl_grab: Option<(f32, f64)>,
    pl_hair: bool,
    // Rechenscheibe
    rot: f64,
    cursor: f64,
    drag_disc: Option<bool>,
    last_ang: f64,
}

impl Default for State {
    fn default() -> Self {
        let mut s = Self {
            s: 12.0,
            tmin: 6.0,
            d: 1.2,
            unknown: Unknown::D,
            hours: false,
            hide: false,
            seed: 0,
            drag_scale: None,
            zk: [0.3, 0.55],
            zk_step: 0,
            zk_play: None,
            zk_grip: None,
            pl_o: 6f64.log10(),
            pl_show: false,
            pl_drag: false,
            pl_grab: None,
            pl_hair: true,
            rot: 0.0,
            cursor: 0.0,
            drag_disc: None,
            last_ang: 0.0,
        };
        s.align_speed();
        s.cursor = mant_angle(18.0) + s.rot;
        // Vorführhilfen für Screenshots (RADAR_DEMO_WB=…)
        if demo_flag("task44") {
            s.s = 16.0;
            s.tmin = 165.0;
            s.hours = true;
            s.recompute();
        }
        if demo_flag("speed") {
            s.unknown = Unknown::S;
            s.d = 1.4;
            s.recompute();
        }
        if demo_flag("zk1") || demo_flag("zk2") {
            s.zk_step = if demo_flag("zk2") { 2 } else { 1 };
            let p = s.zk_plan(1.25);
            s.zk = if s.zk_step == 1 {
                [p.u[0], p.u[1]]
            } else {
                [p.u[2], p.u[3]]
            };
        }
        if demo_flag("pl") {
            s.pl_show = true;
            s.pl_o = s.pl_solution().o;
        }
        s
    }
}

const S_RANGE: (f64, f64) = (1.0, 60.0);
const T_RANGE: (f64, f64) = (0.25, 2880.0);
const D_RANGE: (f64, f64) = (0.01, 3000.0);
const UMAX: f64 = 2.08;
const LOG60: f64 = 1.778_151_250_383_644;

// Stechzirkel und logarithmische Skala 0,1 … 120
const ZK_RANGE: (f64, f64) = (-1.0, 2.079_181_246_047_625);
const ZK_LEG: f32 = 226.0;
const ZK_MAX_SIN: f32 = 0.97;

// Rechenschieber – Lage in Dekaden der Minutenskala (log10 Minuten)
const PL_VIS: (f64, f64) = (-0.95, 2.95);
const PL_BODY: (f64, f64) = (-0.72, 2.27);
const PL_SLIDE: (f64, f64) = (-1.13, 1.77);
const PL_O: (f64, f64) = (0.12, 2.88);
/// log10(1,2): Die Stundenskala steht so, dass „1 h“ und „60 min“ dieselbe Fahrt einstellen.
const LOG12: f64 = 0.079_181_246_047_624_83;

// Farben der Instrumente (unabhängig vom Oberflächenthema)
const YELLOW: Color32 = Color32::from_rgb(244, 194, 44);
const YELLOW_EDGE: Color32 = Color32::from_rgb(186, 136, 18);
const CHANNEL: Color32 = Color32::from_rgb(204, 156, 34);
const SLIDE: Color32 = Color32::from_rgb(248, 248, 244);
const INK: Color32 = Color32::from_rgb(22, 22, 22);
const PAPER: Color32 = Color32::from_rgb(250, 248, 240);
const MARK_BLUE: Color32 = Color32::from_rgb(20, 90, 205);
const MARK_RED: Color32 = Color32::from_rgb(212, 36, 36);

/// Teilungsabschnitt: von, bis, Teilung, Abstand der mittellangen Striche
type Seg = (f64, f64, f64, f64);

const MIN_SEGS: [Seg; 8] = [
    (0.25, 0.5, 1.0 / 60.0, 5.0 / 60.0),
    (0.5, 1.0, 2.0 / 60.0, 10.0 / 60.0),
    (1.0, 2.0, 0.1, 0.5),
    (2.0, 5.0, 0.2, 1.0),
    (5.0, 10.0, 0.5, 1.0),
    (10.0, 20.0, 1.0, 5.0),
    (20.0, 50.0, 2.0, 10.0),
    (50.0, 120.0, 5.0, 10.0),
];
const MIN_LABELS: [(f64, &str); 32] = [
    (0.25, "15"),
    (0.3, "18"),
    (0.35, "21"),
    (0.4, "24"),
    (0.5, "30"),
    (0.6, "36"),
    (0.7, "42"),
    (0.8, "48"),
    (0.9, "54"),
    (1.0, "1"),
    (1.5, "1.5"),
    (2.0, "2"),
    (2.5, "2.5"),
    (3.0, "3"),
    (4.0, "4"),
    (5.0, "5"),
    (6.0, "6"),
    (7.0, "7"),
    (8.0, "8"),
    (9.0, "9"),
    (10.0, "10"),
    (15.0, "15"),
    (20.0, "20"),
    (25.0, "25"),
    (30.0, "30"),
    (40.0, "40"),
    (50.0, "50"),
    (60.0, "60"),
    (70.0, "70"),
    (80.0, "80"),
    (100.0, "100"),
    (120.0, "120"),
];
const HOUR_SEGS: [Seg; 6] = [
    (1.0, 2.0, 0.1, 0.5),
    (2.0, 5.0, 0.25, 0.5),
    (5.0, 10.0, 0.5, 1.0),
    (10.0, 24.0, 1.0, 2.0),
    (24.0, 48.0, 2.0, 6.0),
    (48.0, 96.0, 4.0, 12.0),
];
const HOUR_LABELS: [(f64, &str); 23] = [
    (1.0, "1"),
    (1.5, "1.5"),
    (2.0, "2"),
    (2.5, "2.5"),
    (3.0, "3"),
    (4.0, "4"),
    (5.0, "5"),
    (6.0, "6"),
    (7.0, "7"),
    (8.0, "8"),
    (10.0, "10"),
    (12.0, "12"),
    (15.0, "15"),
    (18.0, "18"),
    (24.0, "24"),
    (30.0, "30"),
    (36.0, "36"),
    (42.0, "42"),
    (48.0, "48"),
    (60.0, "60"),
    (72.0, "72"),
    (84.0, "84"),
    (96.0, "96"),
];
const NM_SEGS: [Seg; 8] = [
    (0.1, 0.2, 0.01, 0.05),
    (0.2, 0.5, 0.02, 0.1),
    (0.5, 1.0, 0.05, 0.1),
    (1.0, 2.0, 0.1, 0.5),
    (2.0, 5.0, 0.2, 1.0),
    (5.0, 10.0, 0.5, 1.0),
    (10.0, 20.0, 1.0, 5.0),
    (20.0, 40.0, 2.0, 10.0),
];
const NM_LABELS: [(f64, &str); 25] = [
    (0.1, "0.1"),
    (0.15, "0.15"),
    (0.2, "0.2"),
    (0.3, "0.3"),
    (0.4, "0.4"),
    (0.5, "0.5"),
    (0.6, "0.6"),
    (0.8, "0.8"),
    (1.0, "1"),
    (1.5, "1.5"),
    (2.0, "2"),
    (2.5, "2.5"),
    (3.0, "3"),
    (4.0, "4"),
    (5.0, "5"),
    (6.0, "6"),
    (7.0, "7"),
    (8.0, "8"),
    (9.0, "9"),
    (10.0, "10"),
    (15.0, "15"),
    (20.0, "20"),
    (25.0, "25"),
    (30.0, "30"),
    (40.0, "40"),
];
const BIG_SEGS: [Seg; 8] = [
    (5.0, 10.0, 0.5, 1.0),
    (10.0, 20.0, 1.0, 5.0),
    (20.0, 50.0, 2.0, 10.0),
    (50.0, 100.0, 5.0, 10.0),
    (100.0, 200.0, 10.0, 50.0),
    (200.0, 500.0, 20.0, 100.0),
    (500.0, 1000.0, 50.0, 100.0),
    (1000.0, 2000.0, 100.0, 500.0),
];
const BIG_LABELS: [(f64, &str); 27] = [
    (5.0, "5"),
    (6.0, "6"),
    (7.0, "7"),
    (8.0, "8"),
    (9.0, "9"),
    (10.0, "10"),
    (15.0, "15"),
    (20.0, "20"),
    (25.0, "25"),
    (30.0, "30"),
    (40.0, "40"),
    (50.0, "50"),
    (60.0, "60"),
    (70.0, "70"),
    (80.0, "80"),
    (100.0, "100"),
    (150.0, "150"),
    (200.0, "200"),
    (250.0, "250"),
    (300.0, "300"),
    (400.0, "400"),
    (500.0, "500"),
    (600.0, "600"),
    (800.0, "800"),
    (1000.0, "1000"),
    (1500.0, "1500"),
    (2000.0, "2000"),
];
const ZK_SEGS: [Seg; 10] = [
    (0.1, 0.2, 0.01, 0.05),
    (0.2, 0.5, 0.02, 0.1),
    (0.5, 1.0, 0.05, 0.1),
    (1.0, 2.0, 0.1, 0.5),
    (2.0, 5.0, 0.2, 1.0),
    (5.0, 10.0, 0.5, 1.0),
    (10.0, 20.0, 1.0, 5.0),
    (20.0, 50.0, 2.0, 10.0),
    (50.0, 100.0, 5.0, 10.0),
    (100.0, 120.0, 5.0, 10.0),
];
const ZK_LABELS: [(f64, &str); 28] = [
    (0.1, "0.1"),
    (0.15, "0.15"),
    (0.2, "0.2"),
    (0.3, "0.3"),
    (0.4, "0.4"),
    (0.5, "0.5"),
    (0.6, "0.6"),
    (0.8, "0.8"),
    (1.0, "1"),
    (1.5, "1.5"),
    (2.0, "2"),
    (3.0, "3"),
    (4.0, "4"),
    (5.0, "5"),
    (6.0, "6"),
    (7.0, "7"),
    (8.0, "8"),
    (9.0, "9"),
    (10.0, "10"),
    (15.0, "15"),
    (20.0, "20"),
    (30.0, "30"),
    (40.0, "40"),
    (50.0, "50"),
    (60.0, "60"),
    (80.0, "80"),
    (100.0, "100"),
    (120.0, "120"),
];

/// Winkel (Grad, im Uhrzeigersinn ab oben) einer Zahl auf einer logarithmischen Kreisskala (eine Dekade je Umlauf).
fn mant_angle(x: f64) -> f64 {
    360.0 * x.log10().rem_euclid(1.0)
}

fn mant_of(angle: f64) -> f64 {
    10f64.powf(angle.rem_euclid(360.0) / 360.0)
}

/// Zahl mit höchstens drei signifikanten Stellen, ohne überflüssige Nullen.
fn fmt_num(x: f64) -> String {
    let a = x.abs();
    let s = if a >= 100.0 {
        format!("{:.0}", x)
    } else if a >= 10.0 {
        format!("{:.1}", x)
    } else if a >= 1.0 {
        format!("{:.2}", x)
    } else {
        format!("{:.3}", x)
    };
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Zeit in Minuten lesbar, bei Bedarf mit Stunden.
fn fmt_time(min: f64) -> String {
    if min >= 90.0 {
        let h = (min / 60.0).floor();
        let m = min - h * 60.0;
        if m.abs() < 0.05 {
            format!("{} h", h)
        } else {
            format!("{} h {} min", h, fmt_num(m))
        }
    } else if min < 1.0 {
        format!("{} s", fmt_num(min * 60.0))
    } else {
        format!("{} min", fmt_num(min))
    }
}

/// Teilung entlang einer waagerechten Kante: Striche ab `y` in Richtung `dir`
/// (+1 nach unten, −1 nach oben), Beschriftung hinter den langen Strichen.
#[allow(clippy::too_many_arguments)]
fn edge_scale(
    p: &egui::Painter,
    segs: &[Seg],
    labels: &[(f64, &str)],
    x_of: &dyn Fn(f64) -> f32,
    y: f32,
    dir: f32,
    xr: (f32, f32),
    font: f32,
) {
    let stroke = Stroke::new(1.0_f32, INK);
    for &(lo, hi, step, mid) in segs {
        let n = ((hi - lo) / step).round() as i64;
        for i in 0..=n {
            let v = lo + step * i as f64;
            let x = x_of(v);
            if x < xr.0 || x > xr.1 {
                continue;
            }
            let long = labels.iter().any(|(lv, _)| (lv - v).abs() < 1e-6);
            let m = v / mid;
            let len = if long {
                12.0
            } else if (m - m.round()).abs() < 1e-6 {
                8.5
            } else {
                5.0
            };
            p.line_segment([pos2(x, y), pos2(x, y + dir * len)], stroke);
        }
    }
    let al = if dir > 0.0 {
        Align2::CENTER_TOP
    } else {
        Align2::CENTER_BOTTOM
    };
    for &(v, s) in labels {
        let x = x_of(v);
        if x < xr.0 + 3.0 || x > xr.1 - 3.0 || s.is_empty() {
            continue;
        }
        p.text(
            pos2(x, y + dir * 13.5),
            al,
            s,
            FontId::proportional(font),
            INK,
        );
    }
}

/// Text mit Hintergrund.
fn tag(p: &egui::Painter, pos: Pos2, al: Align2, s: &str, col: Color32, bg: Color32) {
    let g = p.layout_no_wrap(s.to_string(), FontId::monospace(11.5), col);
    let rect = al.anchor_size(pos, g.size());
    p.rect_filled(rect.expand(3.0), CornerRadius::same(3), bg);
    p.galley(rect.min, g, col);
}

/// Gelenk des Stechzirkels über den beiden Spitzen.
fn hinge(a: Pos2, b: Pos2) -> Pos2 {
    let half = (b.x - a.x).abs() / 2.0;
    let h = (ZK_LEG * ZK_LEG - half * half).max(0.0).sqrt();
    pos2((a.x + b.x) / 2.0, a.y - h)
}

fn seg_dist(q: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let k = ((q - a).dot(ab) / ab.length_sq().max(1e-6)).clamp(0.0, 1.0);
    (a + ab * k - q).length()
}

/// Ablauf der Zirkelmethode: Zirkel von A nach B öffnen, Spitze A nach C versetzen – die andere Spitze zeigt X.
struct ZkPlan {
    u: [f64; 4],
    lbl: [String; 4],
    alt: bool,
    ok: bool,
}

/// Lösung am Rechenschieber.
struct PlSol {
    o: f64,
    ok: bool,
    hours: bool,
    /// Lage der Zeit (und des Wegs) auf dem Körper
    u_t: f64,
    steps: Vec<String>,
}

impl State {
    fn recompute(&mut self) {
        match self.unknown {
            Unknown::D => self.d = self.s * self.tmin / 60.0,
            Unknown::T => {
                self.tmin = (60.0 * self.d / self.s.max(1e-6)).clamp(T_RANGE.0, T_RANGE.1)
            }
            Unknown::S => self.s = 60.0 * self.d / self.tmin.max(1e-6),
        }
    }

    fn align_speed(&mut self) {
        self.rot = mant_angle(self.s) - mant_angle(60.0);
    }

    fn what(&self) -> &'static str {
        match self.unknown {
            Unknown::D => t("Weg", "distance"),
            Unknown::T => t("Zeit", "time"),
            Unknown::S => t("Fahrt", "speed"),
        }
    }

    /// Neue Übungsaufgabe mit runden Werten.
    fn random_task(&mut self) {
        const V: [f64; 14] = [
            6.0, 7.5, 8.0, 9.0, 10.0, 11.0, 12.0, 13.5, 14.0, 15.0, 16.0, 18.0, 20.0, 24.0,
        ];
        const TM: [f64; 16] = [
            3.0, 4.0, 5.0, 6.0, 8.0, 9.0, 10.0, 12.0, 15.0, 18.0, 20.0, 24.0, 30.0, 36.0, 40.0,
            45.0,
        ];
        const TH: [f64; 6] = [90.0, 150.0, 165.0, 240.0, 330.0, 450.0];
        self.seed = self.seed.wrapping_add(1);
        let mut rng = Rng::new(self.seed.wrapping_mul(2_654_435_761) ^ 0x5eed_1234);
        let mut pick = |n: usize| ((rng.f64() * n as f64) as usize).min(n - 1);
        for _ in 0..40 {
            self.s = V[pick(V.len())];
            self.hours = pick(5) == 0;
            self.tmin = if self.hours {
                TH[pick(TH.len())]
            } else {
                TM[pick(TM.len())]
            };
            self.d = self.s * self.tmin / 60.0;
            let cents = self.d * 100.0;
            if (cents - cents.round()).abs() < 1e-6 {
                break;
            }
        }
        self.unknown = [Unknown::D, Unknown::T, Unknown::S][pick(3)];
        self.hide = true;
        self.zk_step = 0;
        self.zk_play = None;
        self.pl_show = false;
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        let dt = ui.input(|i| i.stable_dt).min(0.1);
        text(
            ui,
            t(
                "Beim Plotten wird ständig Weg, Zeit und Fahrt umgerechnet: Wie schnell ist das Ziel relativ, wenn es in 6 Minuten 1,4 sm zurücklegt? Wann erreicht es den CPA? Neben dem Taschenrechner gibt es drei klassische Hilfsmittel: das Nomogramm, den Stechzirkel auf der logarithmischen Skala und den Rechenschieber. Alle drei lösen unten dieselbe Aufgabe.",
                "Plotting constantly converts distance, time and speed: how fast is the target relatively if it covers 1.4 nm in 6 minutes? When does it reach the CPA? Besides the calculator there are three classic aids: the nomogram, the dividers on the logarithmic scale and the slide rule. Below, all three solve the same task.",
            ),
        );
        formula(ui, t("Weg [sm] = Fahrt [kn] · Zeit [min] / 60          6-Minuten-Regel: in 6 min legt man Fahrt/10 sm zurück", "distance [nm] = speed [kn] · time [min] / 60          6-minute rule: in 6 min you cover speed/10 nm"));

        sub(ui, t("Aufgabe", "Task"));
        self.task_ui(ui, &p);

        sub(ui, t("1 · Nomogramm – mit dem Lineal", "1 · Nomogram – with a ruler"));
        text(
            ui,
            t(
                "Drei logarithmische Leitern: links Fahrt, rechts Zeit, in der Mitte Weg. Ein Lineal durch zwei bekannte Werte schneidet die dritte Leiter im gesuchten Wert. Ziehen Sie die Marken auf den bekannten Leitern mit der Maus.",
                "Three logarithmic scales: speed on the left, time on the right, distance in the middle. A ruler through two known values crosses the third scale at the wanted value. Drag the marks on the known scales with the mouse.",
            ),
        );
        let (rect, resp) = ui.allocate_exact_size(
            vec2(ui.available_width().min(760.0), 380.0),
            Sense::click_and_drag(),
        );
        self.nomogram(&ui.painter_at(rect), rect, &resp, &p);

        self.dividers_ui(ui, &p, dt);
        self.portland_ui(ui, &p, dt);

        ui.add_space(8.0);
        egui::CollapsingHeader::new(
            RichText::new(t(
                "Variante: Rechenscheibe (Kreisrechenschieber)",
                "Variant: circular calculator",
            ))
            .strong()
            .size(16.0),
        )
        .id_salt("std_disc")
        .default_open(demo_flag("disc"))
        .show(ui, |ui| self.disc_ui(ui, &p));

        try_it(
            ui,
            t(
                "Klicken Sie „⟳ Neue Aufgabe“ und lösen Sie sie nacheinander mit allen drei Hilfsmitteln: am Nomogramm ablesen, den Zirkel selbst öffnen und versetzen, den Schieber ziehen – die Anzeige meldet ✔, sobald alles richtig steht. Erst dann „Ergebnis verdecken“ abwählen. Für lange Strecken die Zeit auf Stunden umstellen: 16 kn in 2¾ h liest man auf den unteren Skalen ab.",
                "Click “⟳ New task” and solve it with all three aids in turn: read the nomogram, open and transfer the dividers yourself, drag the slide – the display reports ✔ as soon as everything is set correctly. Only then untick “hide result”. For long passages switch the time to hours: 16 kn for 2¾ h is read on the lower scales.",
            ),
        );
        merke(
            ui,
            &[
                t("Weg = Fahrt · Zeit / 60 (sm, kn, min); 6-Minuten-Regel: in 6 min legt man Fahrt/10 sm zurück.", "distance = speed · time / 60 (nm, kn, min); 6-minute rule: in 6 min you cover speed/10 nm."),
                t("Nomogramm: Lineal durch zwei bekannte Werte – der Schnitt mit der dritten Leiter ist das Ergebnis.", "Nomogram: ruler through two known values – where it crosses the third scale is the result."),
                t("Zirkelmethode: Strecke 60 → Fahrt = Strecke Zeit → Weg. Auf der logarithmischen Skala sind gleiche Verhältnisse gleich lange Strecken.", "Dividers: span 60 → speed = span time → distance. On the logarithmic scale equal ratios are equal lengths."),
                t("Rechenschieber: Fahrt unter den Pfeil ▼ (60 MINUTES) – dann steht unter jeder Zeit der Weg; Stunden auf den unteren Skalen.", "Slide rule: speed under the ▼ arrow (60 MINUTES) – then the distance stands below every time; hours on the lower scales."),
                t("3-Minuten-Regel: Fahrt in kn · 100 ≈ Weg in Metern in 3 min.", "3-minute rule: speed in kn · 100 ≈ distance in metres in 3 min."),
            ],
        );
        see_also(ui, nav, &[Chapter::Plotting, Chapter::RelVector]);
    }

    /// Aufgabe für alle Hilfsmittel: gesuchte Größe und bekannte Werte.
    fn task_ui(&mut self, ui: &mut egui::Ui, p: &Pal) {
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(t("Gesucht:", "Wanted:")).strong());
            for (u, name) in [
                (Unknown::D, t("Weg", "distance")),
                (Unknown::T, t("Zeit", "time")),
                (Unknown::S, t("Fahrt", "speed")),
            ] {
                if ui.selectable_value(&mut self.unknown, u, name).clicked() {
                    changed = true;
                }
            }
            ui.separator();
            let hidden = |u: Unknown| self.hide && self.unknown == u;
            if hidden(Unknown::S) {
                ui.label(RichText::new(t("Fahrt ? kn", "speed ? kn")).strong().color(p.warn));
            } else {
                ui.add_enabled_ui(self.unknown != Unknown::S, |ui| {
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut self.s)
                                .range(S_RANGE.0..=S_RANGE.1)
                                .speed(0.1)
                                .fixed_decimals(1)
                                .prefix(t("Fahrt ", "speed "))
                                .suffix(" kn"),
                        )
                        .changed();
                });
            }
            if hidden(Unknown::T) {
                ui.label(RichText::new(t("Zeit ? ", "time ? ")).strong().color(p.warn));
            } else {
                ui.add_enabled_ui(self.unknown != Unknown::T, |ui| {
                    if self.hours {
                        let mut h = self.tmin / 60.0;
                        if ui
                            .add(
                                egui::DragValue::new(&mut h)
                                    .range(T_RANGE.0 / 60.0..=T_RANGE.1 / 60.0)
                                    .speed(0.01)
                                    .fixed_decimals(2)
                                    .prefix(t("Zeit ", "time "))
                                    .suffix(" h"),
                            )
                            .changed()
                        {
                            self.tmin = h * 60.0;
                            changed = true;
                        }
                    } else {
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.tmin)
                                    .range(T_RANGE.0..=T_RANGE.1)
                                    .speed(0.1)
                                    .fixed_decimals(1)
                                    .prefix(t("Zeit ", "time "))
                                    .suffix(" min"),
                            )
                            .changed();
                    }
                });
            }
            ui.selectable_value(&mut self.hours, false, "min");
            ui.selectable_value(&mut self.hours, true, "h");
            if hidden(Unknown::D) {
                ui.label(RichText::new(t("Weg ? sm", "distance ? nm")).strong().color(p.warn));
            } else {
                ui.add_enabled_ui(self.unknown != Unknown::D, |ui| {
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut self.d)
                                .range(D_RANGE.0..=D_RANGE.1)
                                .speed(0.01)
                                .fixed_decimals(2)
                                .prefix(t("Weg ", "distance "))
                                .suffix(t(" sm", " nm")),
                        )
                        .changed();
                });
            }
            ui.separator();
            if ui.button(t("⟳ Neue Aufgabe", "⟳ New task")).clicked() {
                self.random_task();
            }
            ui.checkbox(&mut self.hide, t("Ergebnis verdecken", "hide result"));
        });
        if changed {
            self.recompute();
        }
        let q = |u: Unknown, s: String| {
            if self.hide && self.unknown == u {
                "?".to_string()
            } else {
                s
            }
        };
        ui.label(
            RichText::new(tf!(
                "{} sm  =  {} kn  ·  {}  / 60",
                "{} nm  =  {} kn  ·  {}  / 60",
                q(Unknown::D, fmt_num(self.d)),
                q(Unknown::S, fmt_num(self.s)),
                q(Unknown::T, fmt_time(self.tmin))
            ))
            .monospace()
            .strong()
            .size(16.0),
        );
    }

    fn nomogram(&mut self, p: &egui::Painter, r: Rect, resp: &egui::Response, pal: &Pal) {
        p.rect_filled(r, CornerRadius::same(6), pal.panel);
        let top = r.top() + 34.0;
        let bot = r.bottom() - 24.0;
        let xs = [
            r.left() + r.width() * 0.18,
            r.center().x,
            r.right() - r.width() * 0.18,
        ];
        let y_of = |u: f64| bot - (u / UMAX) as f32 * (bot - top);
        let u_of_y = |y: f32| ((bot - y) / (bot - top)) as f64 * UMAX;
        // Interaktion: Leiter wählen und Wert setzen
        if resp.drag_started() || resp.clicked() {
            if let Some(pp) = resp.interact_pointer_pos() {
                let i = (0..3)
                    .min_by(|a, b| {
                        (xs[*a] - pp.x)
                            .abs()
                            .partial_cmp(&(xs[*b] - pp.x).abs())
                            .unwrap()
                    })
                    .unwrap();
                self.drag_scale = Some(i);
            }
        }
        if let (Some(i), Some(pp)) = (self.drag_scale, resp.interact_pointer_pos()) {
            if resp.dragged() || resp.clicked() {
                let u = u_of_y(pp.y).clamp(0.0, UMAX);
                match i {
                    0 if self.unknown != Unknown::S => {
                        self.s = 10f64.powf(u).clamp(S_RANGE.0, S_RANGE.1)
                    }
                    2 if self.unknown != Unknown::T => self.tmin = 10f64.powf(u).clamp(1.0, 120.0),
                    1 if self.unknown != Unknown::D => {
                        self.d = (10f64.powf(2.0 * u - LOG60)).clamp(0.02, 100.0)
                    }
                    _ => {}
                }
                self.recompute();
            }
        }
        if resp.drag_stopped() {
            self.drag_scale = None;
        }
        let names = [
            t("Fahrt (kn)", "speed (kn)"),
            t("Weg (sm)", "distance (nm)"),
            t("Zeit (min)", "time (min)"),
        ];
        let ticks: [&[f64]; 3] = [
            &[
                1.0, 1.5, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 25.0, 30.0, 40.0,
                50.0, 60.0,
            ],
            &[
                0.02, 0.05, 0.1, 0.2, 0.3, 0.5, 1.0, 1.5, 2.0, 3.0, 5.0, 10.0, 20.0, 30.0, 50.0,
                100.0,
            ],
            &[
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 30.0, 40.0, 60.0, 90.0,
                120.0,
            ],
        ];
        let u_val = |i: usize, v: f64| {
            if i == 1 {
                (v.log10() + LOG60) / 2.0
            } else {
                v.log10()
            }
        };
        for i in 0..3 {
            let (u0, u1) = match i {
                0 => (S_RANGE.0.log10(), S_RANGE.1.log10()),
                1 => (u_val(1, 0.02), u_val(1, 100.0).min(UMAX)),
                _ => (0.0, 120f64.log10()),
            };
            p.line_segment(
                [pos2(xs[i], y_of(u0)), pos2(xs[i], y_of(u1))],
                Stroke::new(2.0_f32, pal.ink),
            );
            p.text(
                pos2(xs[i], r.top() + 8.0),
                Align2::CENTER_TOP,
                names[i],
                FontId::proportional(13.0),
                pal.ink,
            );
            for v in ticks[i] {
                let u = u_val(i, *v);
                if u > UMAX + 1e-9 {
                    continue;
                }
                let y = y_of(u);
                let (dx, al) = if i == 2 {
                    (8.0, Align2::LEFT_CENTER)
                } else {
                    (-8.0, Align2::RIGHT_CENTER)
                };
                p.line_segment(
                    [pos2(xs[i] - 5.0, y), pos2(xs[i] + 5.0, y)],
                    Stroke::new(1.0_f32, pal.ink),
                );
                p.text(
                    pos2(xs[i] + dx, y),
                    al,
                    crate::workbook::charts::fmt_tick(*v),
                    FontId::proportional(11.0),
                    pal.faint,
                );
            }
        }
        let us = [
            u_val(0, self.s),
            u_val(1, self.d),
            u_val(2, self.tmin),
        ];
        let lims = [
            (0.0, S_RANGE.1.log10()),
            (u_val(1, 0.02), u_val(1, 100.0)),
            (0.0, 120f64.log10()),
        ];
        let outside = (0..3).any(|i| us[i] < lims[i].0 - 1e-9 || us[i] > lims[i].1 + 1e-9);
        if outside {
            p.text(
                pos2(r.center().x, r.bottom() - 6.0),
                Align2::CENTER_BOTTOM,
                t(
                    "Ein Wert liegt außerhalb der Leitern (Zeit bis 120 min) – hier hilft der Rechenschieber mit seiner Stundenskala.",
                    "A value is outside the scales (time up to 120 min) – the slide rule with its hours scale helps here.",
                ),
                FontId::proportional(12.0),
                pal.warn,
            );
            return;
        }
        let pts = [
            pos2(xs[0], y_of(us[0])),
            pos2(xs[1], y_of(us[1])),
            pos2(xs[2], y_of(us[2])),
        ];
        // Linie durch die drei Punkte (kollinear)
        let dir = (pts[2] - pts[0]).normalized();
        p.line_segment(
            [pts[0] - dir * 30.0, pts[2] + dir * 30.0],
            Stroke::new(1.6_f32, pal.warn),
        );
        let unknown_idx = match self.unknown {
            Unknown::S => 0,
            Unknown::D => 1,
            Unknown::T => 2,
        };
        for (i, q) in pts.iter().enumerate() {
            let col = if i == unknown_idx {
                pal.warn
            } else {
                pal.accent
            };
            p.circle_filled(*q, 7.0, col);
            p.circle_stroke(*q, 7.0, Stroke::new(1.5_f32, Color32::WHITE));
        }
    }

    // -----------------------------------------------------------------------
    // Zirkelmethode
    // -----------------------------------------------------------------------

    fn zk_plan(&self, max_dec: f64) -> ZkPlan {
        let (v, tm, d) = (self.s, self.tmin, self.d);
        let l60 = "60".to_string();
        let lv = format!("{} kn", fmt_num(v));
        let lt = format!("{} min", fmt_num(tm));
        let ld = format!("{} {}", fmt_num(d), t("sm", "nm"));
        let (classic, alt) = match self.unknown {
            Unknown::D => (
                ([60.0, v, tm, d], [l60.clone(), lv.clone(), lt.clone(), ld.clone()]),
                ([60.0, tm, v, d], [l60, lt, lv, ld]),
            ),
            Unknown::T => (
                ([v, 60.0, d, tm], [lv.clone(), l60.clone(), ld.clone(), lt.clone()]),
                ([v, d, 60.0, tm], [lv, ld, l60, lt]),
            ),
            Unknown::S => (
                ([tm, d, 60.0, v], [lt.clone(), ld.clone(), l60.clone(), lv.clone()]),
                ([tm, 60.0, d, v], [lt, l60, ld, lv]),
            ),
        };
        let spread = |a: &[f64; 4]| (a[1] / a[0]).log10().abs();
        let use_alt = spread(&classic.0) > max_dec && spread(&alt.0) < spread(&classic.0);
        let (vals, lbl) = if use_alt { alt } else { classic };
        let u = vals.map(|x| x.max(1e-9).log10());
        let ok = u
            .iter()
            .all(|x| *x >= ZK_RANGE.0 - 1e-9 && *x <= ZK_RANGE.1 + 1e-9)
            && spread(&vals) <= max_dec + 1e-9;
        ZkPlan {
            u,
            lbl,
            alt: use_alt,
            ok,
        }
    }

    fn dividers_ui(&mut self, ui: &mut egui::Ui, p: &Pal, dt: f32) {
        sub(
            ui,
            t(
                "2 · Zirkelmethode – die logarithmische Skala mit dem Stechzirkel",
                "2 · Dividers method – the logarithmic scale with dividers",
            ),
        );
        text(
            ui,
            t(
                "Viele Plottblätter haben eine einzelne logarithmische Skala. Auf ihr hängt die Strecke zwischen zwei Zahlen nur von deren Verhältnis ab. Weil Weg : Zeit = Fahrt : 60 ist, ist die Strecke von 60 bis zur Fahrt genauso lang wie die von der Zeit bis zum Weg. Diese Strecke greift man mit dem Stechzirkel ab und überträgt sie. An den Spitzen ziehen öffnet den Zirkel, am Griff ziehen versetzt ihn, ohne die Öffnung zu ändern.",
                "Many plotting sheets carry a single logarithmic scale. On it, the length between two numbers depends only on their ratio. Since distance : time = speed : 60, the span from 60 to the speed is as long as the span from the time to the distance. You pick up this span with the dividers and transfer it. Dragging a point opens the dividers, dragging the handle moves them without changing the spread.",
            ),
        );
        formula(
            ui,
            t(
                "log Weg − log Zeit = log Fahrt − log 60      ⇒      Strecke 60 → Fahrt  =  Strecke Zeit → Weg",
                "log distance − log time = log speed − log 60      ⇒      span 60 → speed  =  span time → distance",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            if ui
                .selectable_label(self.zk_step == 1, t("1 · Zirkel öffnen", "1 · open the dividers"))
                .clicked()
            {
                self.zk_step = 1;
                self.zk_play = None;
            }
            if ui
                .selectable_label(
                    self.zk_step == 2,
                    t("2 · Übertragen und ablesen", "2 · transfer and read"),
                )
                .clicked()
            {
                self.zk_step = 2;
                self.zk_play = None;
            }
            if ui
                .button(t("▶ Ablauf zeigen", "▶ show the sequence"))
                .clicked()
            {
                self.zk_step = 1;
                self.zk_play = Some(0.0);
            }
            if ui
                .selectable_label(self.zk_step == 0, t("✋ Selbst versuchen", "✋ Try it yourself"))
                .clicked()
            {
                self.zk_step = 0;
                self.zk_play = None;
            }
        });
        let (rect, resp) =
            ui.allocate_exact_size(vec2(ui.available_width(), 330.0), Sense::click_and_drag());
        let plan = self.dividers(&ui.painter_at(rect), rect, &resp, p, dt);
        if self.zk_play.is_some() || self.zk_grip.is_some() || self.zk_step > 0 {
            ui.ctx().request_repaint();
        }

        let what = self.what();
        let lbl = &plan.lbl;
        if !plan.ok {
            ui.label(RichText::new(t(
                "Ein Wert der Aufgabe liegt außerhalb der Skala (0,1 … 120) oder die nötige Öffnung ist zu groß. Komma verschieben – z. B. 165 min als 2,75 h rechnen – oder eine andere Aufgabe wählen.",
                "A value of the task is outside the scale (0.1 … 120) or the spread needed is too large. Shift the decimal point – e.g. work with 165 min as 2.75 h – or choose another task.",
            )).color(p.warn));
            return;
        }
        let tol = 0.01;
        let near = |x: f64, y: f64| (x - y).abs() < tol;
        let pair = |i: usize, j: usize| {
            (near(self.zk[0], plan.u[i]) && near(self.zk[1], plan.u[j]))
                || (near(self.zk[0], plan.u[j]) && near(self.zk[1], plan.u[i]))
        };
        let msg = match self.zk_step {
            1 => tf!(
                "Schritt 1: Den Zirkel zwischen {} und {} öffnen.",
                "Step 1: open the dividers between {} and {}.",
                lbl[0],
                lbl[1]
            ),
            2 => tf!(
                "Schritt 2: Öffnung beibehalten und die Spitze von {} auf {} setzen. Die andere Spitze zeigt {} ⇒ {} = {}.",
                "Step 2: keep the spread and move the point from {} to {}. The other point shows {} ⇒ {} = {}.",
                lbl[0],
                lbl[2],
                lbl[3],
                what,
                lbl[3]
            ),
            _ if pair(2, 3) => tf!(
                "✔ Richtig übertragen: Die zweite Spitze steht auf {} ⇒ {} = {}.",
                "✔ Transferred correctly: the second point is on {} ⇒ {} = {}.",
                lbl[3],
                what,
                lbl[3]
            ),
            _ if pair(0, 1) => tf!(
                "✔ Richtig geöffnet ({} ↔ {}). Jetzt am Griff ziehen und die Spitze von {} auf {} setzen.",
                "✔ Opened correctly ({} ↔ {}). Now drag the handle and move the point from {} to {}.",
                lbl[0],
                lbl[1],
                lbl[0],
                lbl[2]
            ),
            _ => tf!(
                "Selbst versuchen: Eine Spitze auf {} und die andere auf {} ziehen, dann am Griff die erste Spitze auf {} setzen.",
                "Try it: drag one point to {} and the other to {}, then move the first point to {} with the handle.",
                lbl[0],
                lbl[1],
                lbl[2]
            ),
        };
        let strong = msg.starts_with('✔') || self.zk_step == 2;
        ui.label(RichText::new(msg).size(14.5).strong().color(if strong {
            p.ok
        } else {
            p.ink
        }));
        if plan.alt {
            ui.label(
                RichText::new(t(
                    "Hinweis: Die übliche Öffnung wäre für den Zirkel zu weit – deshalb die gleichwertige Variante mit kleinerer Öffnung.",
                    "Note: the usual spread would be too wide for the dividers – hence the equivalent variant with a smaller spread.",
                ))
                .small()
                .italics(),
            );
        }
    }

    fn dividers(
        &mut self,
        p: &egui::Painter,
        r: Rect,
        resp: &egui::Response,
        pal: &Pal,
        dt: f32,
    ) -> ZkPlan {
        let pad = 34.0;
        let dec = (r.width() - 2.0 * pad) / (ZK_RANGE.1 - ZK_RANGE.0) as f32;
        let x_of = |v: f64| r.left() + pad + (v.max(1e-9).log10() - ZK_RANGE.0) as f32 * dec;
        let xu = |u: f64| r.left() + pad + (u - ZK_RANGE.0) as f32 * dec;
        let ux = |x: f32| ZK_RANGE.0 + ((x - r.left() - pad) / dec) as f64;
        let yb = r.top() + 250.0;
        let max_dec = (2.0 * ZK_LEG * ZK_MAX_SIN / dec) as f64;
        let plan = self.zk_plan(max_dec);

        // automatischer Ablauf
        if let Some(tp) = self.zk_play.as_mut() {
            *tp += dt;
            if *tp > 1.7 && self.zk_step == 1 {
                self.zk_step = 2;
            }
            if *tp > 3.6 {
                self.zk_play = None;
            }
        }

        // Maus: Spitze, Griff oder Skala
        let tip = |u: f64| pos2(xu(u), yb);
        if resp.drag_started() || resp.clicked() {
            let press = if resp.drag_started() {
                resp.ctx.input(|i| i.pointer.press_origin())
            } else {
                None
            };
            if let Some(pp) = press.or(resp.interact_pointer_pos()) {
                let (a, b) = (tip(self.zk[0]), tip(self.zk[1]));
                let h = hinge(a, b);
                let on_tool = (pp - h).length() < 28.0
                    || seg_dist(pp, h, a) < 12.0
                    || seg_dist(pp, h, b) < 12.0
                    || ((pp.x - h.x).abs() < 10.0 && pp.y < h.y && pp.y > h.y - 40.0);
                self.zk_grip = if (pp - a).length() < 16.0 {
                    Some(Grip::A)
                } else if (pp - b).length() < 16.0 {
                    Some(Grip::B)
                } else if pp.y < yb && on_tool {
                    Some(Grip::Both(ux(pp.x) - self.zk[0]))
                } else if pp.y >= yb {
                    if (pp.x - a.x).abs() <= (pp.x - b.x).abs() {
                        Some(Grip::A)
                    } else {
                        Some(Grip::B)
                    }
                } else {
                    None
                };
                if self.zk_grip.is_some() {
                    self.zk_step = 0;
                    self.zk_play = None;
                }
            }
        }
        if let (Some(g), Some(pp)) = (self.zk_grip, resp.interact_pointer_pos()) {
            if resp.dragged() || resp.clicked() {
                let u = ux(pp.x).clamp(ZK_RANGE.0, ZK_RANGE.1);
                match g {
                    Grip::A => self.zk[0] = self.zk[1] + (u - self.zk[1]).clamp(-max_dec, max_dec),
                    Grip::B => self.zk[1] = self.zk[0] + (u - self.zk[0]).clamp(-max_dec, max_dec),
                    Grip::Both(off) => {
                        let sp = self.zk[1] - self.zk[0];
                        let lo = ZK_RANGE.0 - sp.min(0.0);
                        let hi = ZK_RANGE.1 - sp.max(0.0);
                        let a = (ux(pp.x) - off).clamp(lo, hi);
                        self.zk = [a, a + sp];
                    }
                }
            }
        }
        if resp.drag_stopped() || resp.clicked() {
            self.zk_grip = None;
        }
        // geführte Schritte: Spitzen gleiten in die Sollstellung (bei gleicher Öffnung)
        if self.zk_step > 0 && plan.ok && self.zk_grip.is_none() {
            let tg = if self.zk_step == 1 {
                [plan.u[0], plan.u[1]]
            } else {
                [plan.u[2], plan.u[3]]
            };
            let k = 1.0 - (-(dt as f64) * 6.0).exp();
            for (z, g) in self.zk.iter_mut().zip(tg) {
                *z += (g - *z) * k;
            }
        }

        // Zeichnen: Skala auf Papier
        p.rect_filled(r, CornerRadius::same(6), pal.panel);
        let strip = Rect::from_min_max(pos2(r.left() + 10.0, yb), pos2(r.right() - 10.0, yb + 66.0));
        p.rect_filled(strip, CornerRadius::same(3), PAPER);
        p.rect_stroke(
            strip,
            CornerRadius::same(3),
            Stroke::new(1.0_f32, Color32::from_gray(150)),
            egui::StrokeKind::Inside,
        );
        p.line_segment(
            [pos2(xu(ZK_RANGE.0), yb), pos2(xu(ZK_RANGE.1), yb)],
            Stroke::new(1.3_f32, INK),
        );
        edge_scale(p, &ZK_SEGS, &ZK_LABELS, &x_of, yb, 1.0, (strip.left(), strip.right()), 11.5);
        p.text(
            pos2(r.right() - 10.0, r.top() + 8.0),
            Align2::RIGHT_TOP,
            t(
                "logarithmische Skala des Plottblatts",
                "logarithmic scale of the plotting sheet",
            ),
            FontId::proportional(12.0),
            pal.faint,
        );
        // Sollwerte des aktuellen Schritts unter der Skala, Übertragung als Pfeil darüber
        if self.zk_step > 0 && plan.ok {
            let idx: [usize; 2] = if self.zk_step == 1 { [0, 1] } else { [2, 3] };
            for (k, i) in idx.into_iter().enumerate() {
                let x = xu(plan.u[i]);
                let col = if self.zk_step == 2 && k == 1 {
                    MARK_RED
                } else {
                    MARK_BLUE
                };
                p.add(Shape::convex_polygon(
                    vec![
                        pos2(x, yb + 30.0),
                        pos2(x - 5.0, yb + 39.0),
                        pos2(x + 5.0, yb + 39.0),
                    ],
                    col,
                    Stroke::NONE,
                ));
                p.text(
                    pos2(x, yb + 40.0),
                    Align2::CENTER_TOP,
                    &plan.lbl[i],
                    FontId::proportional(11.5),
                    col,
                );
            }
            if self.zk_step == 2 {
                for (from, to) in [(0usize, 2usize), (1, 3)] {
                    let (x0, x1) = (xu(plan.u[from]), xu(plan.u[to]));
                    if (x1 - x0).abs() > 12.0 {
                        let y = yb - 34.0 - 12.0 * from as f32;
                        super::arrow(
                            p,
                            pos2(x0, y),
                            vec2(x1 - x0, 0.0),
                            Stroke::new(1.6_f32, pal.accent.gamma_multiply(0.8)),
                        );
                    }
                }
            }
        }
        // Stechzirkel
        let a = tip(self.zk[0]);
        let b = tip(self.zk[1]);
        let h = hinge(a, b);
        let steel = Color32::from_rgb(170, 178, 192);
        let edge = Color32::from_rgb(64, 70, 82);
        for q in [a, b] {
            let d = q - h;
            let len = d.length().max(1.0);
            let u = d / len;
            let n = vec2(-u.y, u.x);
            let k = h + u * (len - 26.0).max(4.0);
            p.add(Shape::convex_polygon(
                vec![h + n * 5.5, k + n * 3.0, k - n * 3.0, h - n * 5.5],
                steel,
                Stroke::new(1.0_f32, edge),
            ));
            p.add(Shape::convex_polygon(
                vec![k + n * 2.4, q, k - n * 2.4],
                edge,
                Stroke::NONE,
            ));
        }
        p.rect_filled(
            Rect::from_center_size(h - vec2(0.0, 22.0), vec2(9.0, 28.0)),
            CornerRadius::same(3),
            edge,
        );
        p.circle_filled(h, 10.0, steel);
        p.circle_stroke(h, 10.0, Stroke::new(1.5_f32, edge));
        p.circle_filled(h, 3.0, edge);
        // abgelesene Werte an den Spitzen
        let (left, right) = if a.x <= b.x { (0, 1) } else { (1, 0) };
        for (i, al, dx) in [
            (left, Align2::RIGHT_BOTTOM, -8.0),
            (right, Align2::LEFT_BOTTOM, 8.0),
        ] {
            let q = tip(self.zk[i]);
            tag(
                p,
                pos2(q.x + dx, yb - 6.0),
                al,
                &fmt_num(10f64.powf(self.zk[i])),
                pal.ink,
                pal.panel,
            );
        }
        p.text(
            r.left_top() + vec2(10.0, 8.0),
            Align2::LEFT_TOP,
            t(
                "Spitzen ziehen = öffnen · Griff ziehen = versetzen · Klick auf die Skala setzt die nächste Spitze",
                "drag a point = open · drag the handle = move · click on the scale moves the nearest point",
            ),
            FontId::proportional(12.0),
            pal.faint,
        );
        plan
    }

    // -----------------------------------------------------------------------
    // Rechenschieber (Portland Speed-Time-Distance Calculator)
    // -----------------------------------------------------------------------

    fn pl_solution(&self) -> PlSol {
        let (v, tm, d) = (self.s, self.tmin, self.d);
        let o = (60.0 / v.max(1e-6)).log10();
        let hours = tm > 120.0 || d > 40.0;
        let u_t = if hours {
            (1.2 * tm / 60.0).log10()
        } else {
            tm.log10()
        };
        let in_scale = if hours {
            (1.0..=96.0).contains(&(tm / 60.0)) && (5.0..=2000.0).contains(&d)
        } else {
            (0.25..=120.0).contains(&tm) && (0.1..=40.0).contains(&d)
        };
        let ok = in_scale && (PL_O.0..=PL_O.1).contains(&o);
        let fv = format!("{} kn", fmt_num(v));
        let ft = if hours {
            format!("{} h", fmt_num(tm / 60.0))
        } else {
            fmt_time(tm)
        };
        let fd = format!("{} {}", fmt_num(d), t("sm", "nm"));
        let set_speed = tf!(
            "Schieber so stellen, dass die Fahrt {} (obere Schieberskala) unter dem Pfeil ▼ bei 60 MINUTES steht.",
            "Move the slide until the speed {} (upper slide scale) is under the ▼ arrow at 60 MINUTES.",
            fv
        );
        let steps = if !ok {
            vec![t(
                "Die Werte liegen außerhalb der Skalen. Zehnerpotenz verschieben: z. B. 45 kn als 4,5 einstellen und den abgelesenen Weg mit 10 malnehmen.",
                "The values are outside the scales. Shift the decimal point: e.g. set 45 kn as 4.5 and multiply the distance read off by 10.",
            )
            .to_string()]
        } else {
            match (self.unknown, hours) {
                (Unknown::D, false) => vec![
                    set_speed,
                    tf!("Auf der MINUTES-Skala die Zeit {} aufsuchen.", "Find the time {} on the MINUTES scale.", ft),
                    tf!("Direkt darunter auf der oberen Schieberskala ablesen: Weg = {}.", "Read directly below it on the upper slide scale: distance = {}.", fd),
                ],
                (Unknown::D, true) => vec![
                    set_speed,
                    tf!("Auf der HOURS-Skala die Zeit {} aufsuchen.", "Find the time {} on the HOURS scale.", ft),
                    tf!("Direkt darüber auf der unteren Schieberskala ablesen: Weg = {}.", "Read directly above it on the lower slide scale: distance = {}.", fd),
                ],
                (Unknown::T, false) => vec![
                    set_speed,
                    tf!("Den Weg {} auf der oberen Schieberskala suchen.", "Find the distance {} on the upper slide scale.", fd),
                    tf!("Direkt darüber auf der MINUTES-Skala ablesen: Zeit = {}.", "Read directly above it on the MINUTES scale: time = {}.", ft),
                ],
                (Unknown::T, true) => vec![
                    set_speed,
                    tf!("Den Weg {} auf der unteren Schieberskala suchen.", "Find the distance {} on the lower slide scale.", fd),
                    tf!("Direkt darunter auf der HOURS-Skala ablesen: Zeit = {}.", "Read directly below it on the HOURS scale: time = {}.", ft),
                ],
                (Unknown::S, false) => vec![
                    tf!("Schieber so stellen, dass der Weg {} (obere Schieberskala) unter der Zeit {} (MINUTES) steht.", "Move the slide until the distance {} (upper slide scale) is under the time {} (MINUTES).", fd, ft),
                    tf!("Unter dem Pfeil ▼ bei 60 MINUTES ablesen: Fahrt = {}.", "Read under the ▼ arrow at 60 MINUTES: speed = {}.", fv),
                ],
                (Unknown::S, true) => vec![
                    tf!("Schieber so stellen, dass der Weg {} (untere Schieberskala) über der Zeit {} (HOURS) steht.", "Move the slide until the distance {} (lower slide scale) is above the time {} (HOURS).", fd, ft),
                    tf!("Unter dem Pfeil ▼ bei 60 MINUTES ablesen: Fahrt = {}.", "Read under the ▼ arrow at 60 MINUTES: speed = {}.", fv),
                ],
            }
        };
        PlSol {
            o,
            ok,
            hours,
            u_t,
            steps,
        }
    }

    fn portland_ui(&mut self, ui: &mut egui::Ui, p: &Pal, dt: f32) {
        sub(
            ui,
            t(
                "3 · Rechenschieber (Portland Speed-Time-Distance Calculator)",
                "3 · Slide rule (Portland speed-time-distance calculator)",
            ),
        );
        text(
            ui,
            t(
                "Auf dem gelben Körper steht oben die Zeit in MINUTES (links davon SECONDS), unten in HOURS. Der weiße Schieber trägt Seemeilen bzw. Knoten – oben 0,1 bis 40, unten 5 bis 2000. Stellt man die Fahrt unter den Pfeil ▼ bei 60 Minuten, steht unter jeder Zeit der zurückgelegte Weg: für Minuten oben, für Stunden unten. Ziehen Sie den Schieber mit der Maus.",
                "The yellow body carries the time in MINUTES at the top (SECONDS to the left) and in HOURS at the bottom. The white slide carries sea miles or knots – 0.1 to 40 at the top, 5 to 2000 at the bottom. With the speed under the ▼ arrow at 60 minutes, every time stands above the distance covered: minutes at the top, hours at the bottom. Drag the slide with the mouse.",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            if ui
                .selectable_label(
                    self.pl_show,
                    t("Lösungsweg für die Aufgabe zeigen", "show the solution for the task"),
                )
                .clicked()
            {
                self.pl_show = !self.pl_show;
            }
            ui.checkbox(
                &mut self.pl_hair,
                t("Ablesehilfe (Haarlinie unter der Maus)", "reading aid (hairline under the mouse)"),
            );
        });
        let (rect, resp) =
            ui.allocate_exact_size(vec2(ui.available_width(), 282.0), Sense::click_and_drag());
        let sol = self.portland(&ui.painter_at(rect), rect, &resp, dt);
        if self.pl_show || self.pl_drag || resp.hovered() {
            ui.ctx().request_repaint();
        }
        let v_ind = 60.0 / 10f64.powf(self.pl_o);
        ui.label(
            RichText::new(tf!(
                "Unter dem Pfeil ▼ (60 MINUTES) steht {} kn – der Schieber ist auf diese Fahrt eingestellt.",
                "Under the ▼ arrow (60 MINUTES) stands {} kn – the slide is set to this speed.",
                fmt_num(v_ind)
            ))
            .monospace(),
        );
        if self.pl_show {
            for (i, s) in sol.steps.iter().enumerate() {
                let last = i + 1 == sol.steps.len() && sol.ok;
                ui.label(
                    RichText::new(format!("{}. {}", i + 1, s))
                        .size(14.5)
                        .strong()
                        .color(if last { p.ok } else { p.ink }),
                );
            }
        } else if sol.ok {
            if (self.pl_o - sol.o).abs() < 0.007 {
                ui.label(
                    RichText::new(tf!(
                        "✔ Der Schieber steht richtig – jetzt ablesen: {}",
                        "✔ The slide is set correctly – now read off: {}",
                        sol.steps.last().cloned().unwrap_or_default()
                    ))
                    .size(14.5)
                    .strong()
                    .color(p.ok),
                );
            } else {
                ui.label(
                    RichText::new(tf!(
                        "Für die Aufgabe: {}",
                        "For the task: {}",
                        sol.steps[0]
                    ))
                    .size(14.5),
                );
            }
        } else {
            ui.label(RichText::new(&sol.steps[0]).color(p.warn));
        }
    }

    fn portland(&mut self, p: &egui::Painter, r: Rect, resp: &egui::Response, dt: f32) -> PlSol {
        let sol = self.pl_solution();
        let pad = 6.0;
        let dec = (r.width() - 2.0 * pad) / (PL_VIS.1 - PL_VIS.0) as f32;
        let xu = |u: f64| r.left() + pad + (u - PL_VIS.0) as f32 * dec;
        // Schieber ziehen
        if resp.drag_started() {
            self.pl_drag = true;
            self.pl_show = false;
            let x0 = resp
                .ctx
                .input(|i| i.pointer.press_origin())
                .or(resp.interact_pointer_pos())
                .map_or(0.0, |q| q.x);
            self.pl_grab = Some((x0, self.pl_o));
        }
        if let (true, Some((x0, o0)), Some(q)) =
            (self.pl_drag, self.pl_grab, resp.interact_pointer_pos())
        {
            self.pl_o = (o0 + ((q.x - x0) / dec) as f64).clamp(PL_O.0, PL_O.1);
        }
        if resp.drag_stopped() {
            self.pl_drag = false;
            self.pl_grab = None;
        }
        if self.pl_show && sol.ok && !self.pl_drag {
            let k = 1.0 - (-(dt as f64) * 6.0).exp();
            self.pl_o += (sol.o - self.pl_o) * k;
        }
        let o = self.pl_o;

        let y0 = r.top() + 4.0;
        let st = y0 + 92.0; // Oberkante des Schlitzes
        let sb = st + 80.0; // Unterkante
        let y1 = sb + 96.0;
        let bx = (xu(PL_BODY.0), xu(PL_BODY.1));
        let body = Rect::from_min_max(pos2(bx.0, y0), pos2(bx.1, y1));
        p.rect_filled(body, CornerRadius::same(8), YELLOW);
        p.rect_stroke(
            body,
            CornerRadius::same(8),
            Stroke::new(1.5_f32, YELLOW_EDGE),
            egui::StrokeKind::Inside,
        );
        p.rect_filled(
            Rect::from_min_max(pos2(bx.0, st), pos2(bx.1, sb)),
            CornerRadius::ZERO,
            CHANNEL,
        );
        // Schieber
        let sx = (xu(PL_SLIDE.0 + o), xu(PL_SLIDE.1 + o));
        let slide = Rect::from_min_max(pos2(sx.0, st + 1.0), pos2(sx.1, sb - 1.0));
        p.rect_filled(slide, CornerRadius::same(4), SLIDE);
        p.rect_stroke(
            slide,
            CornerRadius::same(4),
            Stroke::new(1.0_f32, Color32::from_gray(165)),
            egui::StrokeKind::Inside,
        );
        let top_x = |m: f64| xu(m.log10() + o);
        let bot_x = |l: f64| xu((l / 50.0).log10() + o);
        let sr = (sx.0.max(r.left()), sx.1.min(r.right()));
        edge_scale(p, &NM_SEGS, &NM_LABELS, &top_x, st, 1.0, sr, 11.0);
        edge_scale(p, &BIG_SEGS, &BIG_LABELS, &bot_x, sb, -1.0, sr, 11.0);
        p.text(
            pos2(xu(0.3 + o), (st + sb) / 2.0),
            Align2::CENTER_CENTER,
            "SEA MILES (Distance) or KNOTS (Speed) or Dep.",
            FontId::proportional(10.5),
            INK,
        );
        // Körper-Skalen
        let min_x = |m: f64| xu(m.log10());
        let hour_x = |h: f64| xu(h.log10() + LOG12);
        edge_scale(p, &MIN_SEGS, &MIN_LABELS, &min_x, st, -1.0, bx, 11.0);
        edge_scale(p, &HOUR_SEGS, &HOUR_LABELS, &hour_x, sb, 1.0, bx, 11.0);
        // Pfeile
        let x60 = xu(LOG60);
        p.add(Shape::convex_polygon(
            vec![pos2(x60, st - 0.5), pos2(x60 - 6.5, st - 13.0), pos2(x60 + 6.5, st - 13.0)],
            INK,
            Stroke::NONE,
        ));
        let x1h = xu(LOG12);
        p.add(Shape::convex_polygon(
            vec![pos2(x1h, sb + 0.5), pos2(x1h - 6.5, sb + 13.0), pos2(x1h + 6.5, sb + 13.0)],
            INK,
            Stroke::NONE,
        ));
        // Beschriftung des Körpers
        p.text(
            pos2(bx.0 + 16.0, y0 + 14.0),
            Align2::LEFT_TOP,
            "SPEED – TIME – DISTANCE CALCULATOR",
            FontId::proportional(16.0),
            INK,
        );
        let small = FontId::proportional(10.5);
        p.text(pos2(xu(-0.33), st - 31.0), Align2::CENTER_BOTTOM, "SECONDS", small.clone(), INK);
        p.text(
            pos2(xu(0.95), st - 31.0),
            Align2::CENTER_BOTTOM,
            "MINUTES or D. Long.",
            FontId::proportional(12.0),
            INK,
        );
        p.text(
            pos2(x60, st - 31.0),
            Align2::CENTER_BOTTOM,
            "Set Speed against arrow",
            small.clone(),
            INK,
        );
        p.text(
            pos2(x1h, sb + 31.0),
            Align2::CENTER_TOP,
            "Set Speed\nagainst arrow",
            small.clone(),
            INK,
        );
        p.text(
            pos2(xu(0.8), sb + 33.0),
            Align2::CENTER_TOP,
            "HOURS",
            FontId::proportional(12.0),
            INK,
        );
        for (h, s) in [(24.0, "1 Day"), (48.0, "2"), (72.0, "3"), (96.0, "4")] {
            p.text(
                pos2(hour_x(h), sb + 33.0),
                Align2::CENTER_TOP,
                s,
                FontId::proportional(12.0),
                INK,
            );
        }

        // Lösungsweg: Marken an den Ableseorten
        if self.pl_show && sol.ok {
            let xt = xu(sol.u_t);
            let (cv, ct, cd) = match self.unknown {
                Unknown::S => (MARK_RED, MARK_BLUE, MARK_BLUE),
                Unknown::T => (MARK_BLUE, MARK_RED, MARK_BLUE),
                Unknown::D => (MARK_BLUE, MARK_BLUE, MARK_RED),
            };
            let dash = |x: f32, c: Color32| {
                p.extend(Shape::dashed_line(
                    &[pos2(x, y0 + 40.0), pos2(x, y1 - 30.0)],
                    Stroke::new(1.2_f32, c),
                    5.0,
                    4.0,
                ));
            };
            dash(x60, cv);
            dash(xt, ct);
            let ring = |c: Pos2, col: Color32, s: &str| {
                p.circle_stroke(c, 8.0, Stroke::new(2.5_f32, col));
                let g = p.layout_no_wrap(s.to_string(), FontId::proportional(12.0), Color32::WHITE);
                let rr = Align2::LEFT_CENTER.anchor_size(c + vec2(13.0, 0.0), g.size());
                p.rect_filled(rr.expand2(vec2(3.0, 1.0)), CornerRadius::same(3), col);
                p.galley(rr.min, g, Color32::WHITE);
            };
            ring(pos2(x60, st + 7.0), cv, "v");
            if sol.hours {
                ring(pos2(xt, sb + 7.0), ct, "t");
                ring(pos2(xt, sb - 7.0), cd, "d");
            } else {
                ring(pos2(xt, st - 7.0), ct, "t");
                ring(pos2(xt, st + 7.0), cd, "d");
            }
        }

        // Ablesehilfe: Haarlinie mit allen vier Skalenwerten
        if self.pl_hair {
            if let Some(hp) = resp.hover_pos().or(resp.interact_pointer_pos()) {
                if r.contains(hp) {
                    p.line_segment(
                        [pos2(hp.x, y0 + 2.0), pos2(hp.x, y1 - 2.0)],
                        Stroke::new(1.2_f32, MARK_RED),
                    );
                    let u = PL_VIS.0 + ((hp.x - r.left() - pad) / dec) as f64;
                    let m = 10f64.powf(u);
                    let top = 10f64.powf(u - o);
                    let big = top * 50.0;
                    let hrs = m / 1.2;
                    let show = |ok: bool, s: String| if ok { s } else { "–".to_string() };
                    let lines = [
                        format!(
                            "MINUTES     {}",
                            show((0.25..=120.0).contains(&m), fmt_time(m))
                        ),
                        format!(
                            "SM / KN     {}",
                            show((0.1..=40.0).contains(&top), fmt_num(top))
                        ),
                        format!(
                            "SM (unten)  {}",
                            show((5.0..=2000.0).contains(&big), fmt_num(big))
                        ),
                        format!(
                            "HOURS       {}",
                            show((1.0..=96.0).contains(&hrs), fmt_num(hrs))
                        ),
                    ];
                    let txt = lines.join("\n");
                    let g = p.layout_no_wrap(txt, FontId::monospace(12.0), INK);
                    let rr = Align2::RIGHT_TOP.anchor_size(r.right_top() + vec2(-8.0, 8.0), g.size());
                    p.rect_filled(rr.expand(6.0), CornerRadius::same(4), PAPER);
                    p.rect_stroke(
                        rr.expand(6.0),
                        CornerRadius::same(4),
                        Stroke::new(1.0_f32, MARK_RED),
                        egui::StrokeKind::Inside,
                    );
                    p.galley(rr.min, g, INK);
                }
            }
        }
        sol
    }

    // -----------------------------------------------------------------------
    // Rechenscheibe
    // -----------------------------------------------------------------------

    fn disc_ui(&mut self, ui: &mut egui::Ui, p: &Pal) {
        text(
            ui,
            t(
                "Dasselbe Prinzip auf einer Scheibe: Außen steht eine feste logarithmische Skala (Weg bzw. Fahrt), innen eine drehbare Zeitskala mit der Fahrtmarke ▲ bei 60 Minuten. Drehen Sie die innere Scheibe, bis ▲ auf der Fahrt steht – dann steht jeder Zeit innen der zugehörige Weg außen gegenüber. Den Haarstrich zum Ablesen ziehen Sie am äußeren Ring. Die Skalen umfassen eine Dekade; das Komma setzt man durch Überschlag.",
                "The same principle on a disc: outside is a fixed logarithmic scale (distance or speed), inside a rotating time scale with the speed mark ▲ at 60 minutes. Turn the inner disc until ▲ points at the speed – then every time inside faces the corresponding distance outside. Drag the hairline at the outer ring to read off values. The scales cover one decade; place the decimal point by estimation.",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(tf!(
                    "▲ auf Fahrt {} kn stellen",
                    "set ▲ to speed {} kn",
                    fmt_num(self.s)
                ))
                .clicked()
            {
                self.align_speed();
            }
            if ui
                .button(tf!(
                    "Haarstrich auf {} min",
                    "hairline to {} min",
                    fmt_num(self.tmin)
                ))
                .clicked()
            {
                self.cursor = mant_angle(self.tmin) + self.rot;
            }
        });
        let (rect, resp) = ui.allocate_exact_size(
            vec2(ui.available_width().min(760.0), 440.0),
            Sense::click_and_drag(),
        );
        self.calculator(&ui.painter_at(rect), rect, &resp, p);
        let speed_ind = mant_of(mant_angle(60.0) + self.rot);
        let outer = mant_of(self.cursor);
        let inner = mant_of(self.cursor - self.rot);
        ui.label(
            RichText::new(tf!(
                "▲ zeigt auf {:.3} (Fahrt)   ·   Haarstrich: außen {:.3}, innen {:.3}   ⇒   außen = ▲-Wert · innen / 60 (Ziffernfolge)",
                "▲ points at {:.3} (speed)   ·   hairline: outer {:.3}, inner {:.3}   ⇒   outer = ▲ value · inner / 60 (digits)",
                speed_ind,
                outer,
                inner
            ))
            .monospace(),
        );
    }

    fn calculator(&mut self, p: &egui::Painter, r: Rect, resp: &egui::Response, pal: &Pal) {
        p.rect_filled(r, CornerRadius::same(6), pal.panel);
        let c = r.center();
        let ro = r.height() / 2.0 - 16.0;
        let ri = ro * 0.72;
        let ang_at = |pp: Pos2| -> f64 {
            let d = pp - c;
            (d.x as f64).atan2(-(d.y as f64)).to_degrees()
        };
        if resp.drag_started() {
            if let Some(pp) = resp.interact_pointer_pos() {
                self.drag_disc = Some((pp - c).length() < ri);
                self.last_ang = ang_at(pp);
            }
        }
        if let (Some(inner), Some(pp)) = (self.drag_disc, resp.interact_pointer_pos()) {
            if resp.dragged() {
                let a = ang_at(pp);
                if inner {
                    let mut d = a - self.last_ang;
                    if d > 180.0 {
                        d -= 360.0;
                    }
                    if d < -180.0 {
                        d += 360.0;
                    }
                    self.rot = (self.rot + d).rem_euclid(360.0);
                    self.last_ang = a;
                } else {
                    self.cursor = a.rem_euclid(360.0);
                }
            }
        }
        if resp.drag_stopped() {
            self.drag_disc = None;
        }
        let dir = |a: f64| {
            let r = a.to_radians() as f32;
            vec2(r.sin(), -r.cos())
        };
        // äußere feste Skala
        p.circle_filled(c, ro + 12.0, Color32::from_rgb(235, 232, 220));
        p.circle_stroke(
            c,
            ro + 12.0,
            Stroke::new(1.5_f32, Color32::from_rgb(90, 90, 90)),
        );
        let dark = Color32::from_rgb(30, 30, 30);
        let scale_ticks =
            |rad: f32, inward: bool, rot: f64, painter: &egui::Painter, col: Color32| {
                let mut v = 1.0;
                while v < 10.0 - 1e-9 {
                    let step = if v < 2.0 {
                        0.05
                    } else if v < 5.0 {
                        0.1
                    } else {
                        0.2
                    };
                    let a = mant_angle(v) + rot;
                    let tenth = v * 10.0;
                    let on_tenth = (tenth - tenth.round()).abs() < 1e-6;
                    let major = (v - v.round()).abs() < 1e-6
                        || (v - 1.5).abs() < 1e-6
                        || (v - 2.5).abs() < 1e-6;
                    let mid = on_tenth && (tenth.round() as i64) % 5 == 0;
                    let len = if major {
                        12.0
                    } else if mid {
                        8.0
                    } else {
                        4.0
                    };
                    let s = if inward { 1.0 } else { -1.0 };
                    painter.line_segment(
                        [c + dir(a) * rad, c + dir(a) * (rad - s * len)],
                        Stroke::new(1.0_f32, col),
                    );
                    if major {
                        let lbl = if (v - v.round()).abs() < 1e-6 {
                            format!("{}", v.round() as i64 * 10)
                        } else {
                            format!("{}", (v * 10.0).round() as i64)
                        };
                        painter.text(
                            c + dir(a) * (rad - s * 22.0),
                            Align2::CENTER_CENTER,
                            lbl,
                            FontId::proportional(11.0),
                            col,
                        );
                    }
                    v = ((v + step) * 100.0).round() / 100.0;
                }
            };
        scale_ticks(ro + 10.0, true, 0.0, p, dark);
        // innere drehbare Scheibe (Zeit)
        p.circle_filled(c, ri, Color32::from_rgb(250, 248, 240));
        p.circle_stroke(c, ri, Stroke::new(1.5_f32, Color32::from_rgb(90, 90, 90)));
        let blue = Color32::from_rgb(30, 70, 160);
        scale_ticks(ri - 2.0, true, self.rot, p, blue);
        // Fahrtmarke ▲ bei 60
        let a60 = mant_angle(60.0) + self.rot;
        let tip = c + dir(a60) * (ri + 2.0);
        let n = vec2(-dir(a60).y, dir(a60).x);
        p.add(Shape::convex_polygon(
            vec![
                tip,
                tip - dir(a60) * 14.0 + n * 7.0,
                tip - dir(a60) * 14.0 - n * 7.0,
            ],
            Color32::from_rgb(210, 40, 40),
            Stroke::NONE,
        ));
        p.text(
            c + dir(a60) * (ri - 44.0),
            Align2::CENTER_CENTER,
            "60",
            FontId::proportional(12.0),
            Color32::from_rgb(210, 40, 40),
        );
        p.text(
            c + vec2(0.0, -ri * 0.25),
            Align2::CENTER_CENTER,
            t("innen: Zeit (min)", "inside: time (min)"),
            FontId::proportional(12.0),
            blue,
        );
        p.text(
            c + vec2(0.0, ri * 0.25),
            Align2::CENTER_CENTER,
            t("außen: Weg / Fahrt", "outside: distance / speed"),
            FontId::proportional(12.0),
            dark,
        );
        // Haarstrich
        p.line_segment(
            [
                c + dir(self.cursor) * (ri * 0.35),
                c + dir(self.cursor) * (ro + 14.0),
            ],
            Stroke::new(2.0_f32, Color32::from_rgb(210, 40, 40)),
        );
        p.circle_filled(c, 4.0, dark);
        p.text(
            r.left_top() + vec2(10.0, 8.0),
            Align2::LEFT_TOP,
            t(
                "innere Scheibe drehen · Haarstrich am Außenring ziehen",
                "turn the inner disc · drag the hairline at the outer ring",
            ),
            FontId::proportional(12.0),
            pal.faint,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Zirkelmethode liefert für jede gesuchte Größe das richtige Ergebnis.
    #[test]
    fn dividers_plan_is_consistent() {
        let mut st = State::default();
        for unk in [Unknown::D, Unknown::T, Unknown::S] {
            st.unknown = unk;
            st.s = 9.0;
            st.tmin = 40.0;
            st.d = 6.0;
            st.recompute();
            let p = st.zk_plan(1.25);
            assert!(p.ok);
            // gleiche Öffnung: B − A = X − C
            assert!(((p.u[1] - p.u[0]) - (p.u[3] - p.u[2])).abs() < 1e-9);
        }
        st.unknown = Unknown::D;
        st.s = 9.0;
        st.tmin = 40.0;
        st.recompute();
        let p = st.zk_plan(1.25);
        assert!((10f64.powf(p.u[3]) - 6.0).abs() < 1e-9);
    }

    /// Rechenschieber: Fahrt unter ▼ und Weg unter der Zeit stehen auf einer Linie.
    #[test]
    fn slide_rule_alignment() {
        let mut st = State::default();
        // 16 kn in 2 h 45 min = 44 sm (Stundenskala)
        st.unknown = Unknown::D;
        st.s = 16.0;
        st.tmin = 165.0;
        st.recompute();
        let sol = st.pl_solution();
        assert!(sol.ok && sol.hours);
        let u_d = (st.d / 50.0).log10() + sol.o;
        assert!((u_d - sol.u_t).abs() < 1e-9, "{} {}", u_d, sol.u_t);
        assert!((st.d - 44.0).abs() < 1e-9);
        // 14 kn, 4,2 sm ⇒ 18 min (Minutenskala)
        st.unknown = Unknown::T;
        st.s = 14.0;
        st.d = 4.2;
        st.recompute();
        let sol = st.pl_solution();
        assert!(sol.ok && !sol.hours);
        assert!((st.tmin - 18.0).abs() < 1e-9);
        assert!((st.d.log10() + sol.o - sol.u_t).abs() < 1e-9);
    }

    fn run(ctx: &egui::Context, st: &mut State, t: f64, events: Vec<egui::Event>) -> (Rect, Rect) {
        let mut rects = (Rect::NOTHING, Rect::NOTHING);
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1300.0, 900.0))),
            time: Some(t),
            events,
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let p = pal(ui);
                let (r, resp) =
                    ui.allocate_exact_size(vec2(1100.0, 330.0), Sense::click_and_drag());
                st.dividers(&ui.painter_at(r), r, &resp, &p, 0.016);
                let (r2, resp2) =
                    ui.allocate_exact_size(vec2(1100.0, 282.0), Sense::click_and_drag());
                st.portland(&ui.painter_at(r2), r2, &resp2, 0.016);
                rects = (r, r2);
            });
        });
        rects
    }

    fn drag(ctx: &egui::Context, st: &mut State, t0: f64, from: Pos2, to: Pos2) {
        use egui::{Event, Modifiers, PointerButton};
        let button = |pos: Pos2, pressed: bool| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        run(ctx, st, t0, vec![Event::PointerMoved(from), button(from, true)]);
        for k in 1..=6 {
            let q = from + (to - from) * (k as f32 / 6.0);
            run(ctx, st, t0 + 0.05 * k as f64, vec![Event::PointerMoved(q)]);
        }
        run(ctx, st, t0 + 0.4, vec![button(to, false)]);
        run(ctx, st, t0 + 0.45, vec![]);
    }

    /// Spitze ziehen öffnet, Griff ziehen versetzt bei gleicher Öffnung, Schieber folgt der Maus.
    #[test]
    fn dragging_dividers_and_slide() {
        let ctx = egui::Context::default();
        let mut st = State::default();
        let (zr, pr) = run(&ctx, &mut st, 0.0, vec![]);
        let pad = 34.0;
        let dec = (zr.width() - 2.0 * pad) / (ZK_RANGE.1 - ZK_RANGE.0) as f32;
        let xu = |u: f64| zr.left() + pad + (u - ZK_RANGE.0) as f32 * dec;
        let yb = zr.top() + 250.0;
        let a0 = st.zk[0];
        let b0 = st.zk[1];
        let tip_b = pos2(xu(b0), yb);
        drag(&ctx, &mut st, 1.0, tip_b, tip_b + vec2(80.0, 0.0));
        let want = b0 + (80.0 / dec) as f64;
        assert!((st.zk[1] - want).abs() < 0.01, "B {} statt {}", st.zk[1], want);
        assert!((st.zk[0] - a0).abs() < 1e-9, "A hat sich bewegt");
        let spread = st.zk[1] - st.zk[0];
        let h = hinge(pos2(xu(st.zk[0]), yb), pos2(xu(st.zk[1]), yb));
        drag(&ctx, &mut st, 2.0, h, h + vec2(-120.0, 0.0));
        assert!(((st.zk[1] - st.zk[0]) - spread).abs() < 1e-9, "Öffnung verändert");
        let want = a0 - (120.0 / dec) as f64;
        assert!((st.zk[0] - want).abs() < 0.01, "A {} statt {}", st.zk[0], want);
        let pdec = (pr.width() - 12.0) / (PL_VIS.1 - PL_VIS.0) as f32;
        let o0 = st.pl_o;
        let c = pr.center();
        drag(&ctx, &mut st, 3.0, c, c + vec2(60.0, 0.0));
        let want = o0 + (60.0 / pdec) as f64;
        assert!((st.pl_o - want).abs() < 0.005, "Versatz {} statt {}", st.pl_o, want);
    }

    #[test]
    fn random_tasks_are_round() {
        let mut st = State::default();
        for _ in 0..50 {
            st.random_task();
            let c = st.d * 100.0;
            assert!((c - c.round()).abs() < 1e-6, "Weg {}", st.d);
            assert!((st.s * st.tmin / 60.0 - st.d).abs() < 1e-9);
        }
    }
}
