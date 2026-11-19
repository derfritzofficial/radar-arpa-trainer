//! Bedienoberfläche: PPI-Hauptanzeige mit Mausbedienung, Datenfeld links,
//! Bedienfelder rechts, Szenario-/Zeitrafferleiste oben.

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Key, Pos2, Rect, RichText, Sense,
    Shape, Stroke, Vec2,
};

use crate::arpa::*;
use crate::i18n::t;
use crate::radar::*;
use crate::sim::*;
use crate::util::*;

const WARPS: [f64; 7] = [1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 60.0];
const VEC_PRESETS: [f64; 6] = [1.0, 3.0, 6.0, 12.0, 15.0, 30.0];
const TRAIL_PRESETS: [f64; 6] = [0.5, 1.0, 3.0, 6.0, 15.0, 30.0];
const PP_INTERVALS: [(f64, &str); 5] = [
    (0.0, "aus"),
    (30.0, "30 s"),
    (60.0, "1 min"),
    (120.0, "2 min"),
    (180.0, "3 min"),
];

const C_RING: Color32 = Color32::from_rgb(70, 120, 140);
const C_SCALE: Color32 = Color32::from_rgb(160, 190, 200);
const C_HDG: Color32 = Color32::from_rgb(235, 235, 235);
const C_ARPA: Color32 = Color32::from_rgb(110, 225, 255);
const C_DANGER: Color32 = Color32::from_rgb(255, 60, 60);
const C_LOST: Color32 = Color32::from_rgb(255, 160, 40);
const C_EBL: Color32 = Color32::from_rgb(90, 255, 170);
const C_TRIAL: Color32 = Color32::from_rgb(255, 165, 60);
const C_OWN: Color32 = Color32::from_rgb(255, 255, 255);
const C_CMD: Color32 = Color32::from_rgb(120, 255, 120);
const C_TRUTH: Color32 = Color32::from_rgb(230, 90, 230);
const C_PI: Color32 = Color32::from_rgb(120, 170, 255);
const C_GZ: Color32 = Color32::from_rgb(255, 220, 120);

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Orientation {
    NorthUp,
    HeadUp,
    CourseUp,
}

impl Orientation {
    fn label(self) -> &'static str {
        match self {
            Orientation::NorthUp => "NORTH UP",
            Orientation::HeadUp => "HEAD UP",
            Orientation::CourseUp => "COURSE UP",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Motion {
    Relative,
    True,
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum TrailMode {
    Off,
    True,
    Relative,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppMode {
    /// Auswahlbildschirm beim Start
    Start,
    Sim,
    Plot,
}

/// Grundfarbe der Bedienoberfläche – unabhängig von der Systemeinstellung.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiTheme {
    Dark,
    Gray,
    Light,
}

impl UiTheme {
    pub(crate) fn label(self) -> &'static str {
        match self {
            UiTheme::Dark => t("Dunkel", "Dark"),
            UiTheme::Gray => t("Grau", "Gray"),
            UiTheme::Light => t("Hell", "Light"),
        }
    }

    pub(crate) fn visuals(self) -> egui::Visuals {
        match self {
            UiTheme::Dark => egui::Visuals::dark(),
            UiTheme::Light => egui::Visuals::light(),
            UiTheme::Gray => {
                let mut v = egui::Visuals::dark();
                let g = |x: u8| Color32::from_rgb(x, x, x + 4);
                v.panel_fill = g(62);
                v.window_fill = g(68);
                v.extreme_bg_color = g(44);
                v.faint_bg_color = g(70);
                v.widgets.noninteractive.bg_fill = g(62);
                v.widgets.noninteractive.weak_bg_fill = g(62);
                v.widgets.inactive.bg_fill = g(88);
                v.widgets.inactive.weak_bg_fill = g(84);
                v.widgets.hovered.bg_fill = g(104);
                v.widgets.hovered.weak_bg_fill = g(100);
                v.widgets.active.bg_fill = g(118);
                v.widgets.active.weak_bg_fill = g(114);
                v
            }
        }
    }

    /// Fläche um den Radarschirm
    pub(crate) fn surround(self) -> Color32 {
        match self {
            UiTheme::Dark => Color32::from_rgb(14, 18, 24),
            UiTheme::Gray => Color32::from_rgb(50, 52, 56),
            UiTheme::Light => Color32::from_rgb(205, 208, 212),
        }
    }

    /// Systemeinstellung überschreiben und Farben setzen.
    pub(crate) fn apply(self, ctx: &egui::Context) {
        ctx.set_theme(egui::ThemePreference::Dark);
        ctx.set_visuals_of(egui::Theme::Dark, self.visuals());
    }
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum ColorScheme {
    Blue,
    Yellow,
    Green,
    Orange,
}

impl ColorScheme {
    fn label(self) -> &'static str {
        match self {
            ColorScheme::Blue => t("Blau (hell)", "Blue (light)"),
            ColorScheme::Yellow => t("Gelb (Tag)", "Yellow (day)"),
            ColorScheme::Green => t("Grün (klassisch)", "Green (classic)"),
            ColorScheme::Orange => t("Orange (Nacht)", "Orange (night)"),
        }
    }
    /// (Hintergrund, schwaches Echo, starkes Echo)
    fn colors(self) -> ([u8; 3], [u8; 3], [u8; 3]) {
        match self {
            ColorScheme::Blue => ([26, 62, 128], [190, 150, 40], [255, 245, 120]),
            ColorScheme::Yellow => ([4, 10, 30], [120, 80, 0], [255, 235, 80]),
            ColorScheme::Green => ([0, 14, 8], [0, 90, 25], [160, 255, 150]),
            ColorScheme::Orange => ([6, 4, 3], [80, 30, 0], [255, 140, 30]),
        }
    }
    /// Hintergrund bei maximaler Helligkeit (Regler = 2,0)
    fn light_bg(self) -> [u8; 3] {
        match self {
            ColorScheme::Blue => [70, 120, 200],
            ColorScheme::Yellow => [40, 60, 120],
            ColorScheme::Green => [25, 80, 45],
            ColorScheme::Orange => [70, 45, 25],
        }
    }
    /// Farbe von Ringen/Skala passend zum Hintergrund
    fn ring_color(self) -> Color32 {
        match self {
            ColorScheme::Blue => Color32::from_rgb(160, 200, 240),
            _ => C_RING,
        }
    }
    /// Rand um den Schirm
    fn bezel(self) -> Color32 {
        match self {
            ColorScheme::Blue => Color32::from_rgb(22, 40, 70),
            _ => Color32::from_rgb(20, 26, 34),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CourseMode {
    Manual,
    Collision,
    Cpa,
}

#[derive(Clone, Copy, PartialEq)]
enum DragMode {
    None,
    OwnVector,
    Course,
    TrialVector,
    Vrm(usize),
    Ebl(usize),
}

#[derive(Clone, Copy)]
struct Ebl {
    on: bool,
    brg: f64,
}

#[derive(Clone, Copy)]
struct Vrm {
    on: bool,
    range: f64,
}

struct AddForm {
    kind: ShipType,
    brg: f64,
    rng: f64,
    mode: CourseMode,
    course: f64,
    speed: f64,
    cpa: f64,
}

/// Abbildung Welt ↔ Bildschirm für ein Frame.
#[derive(Clone, Copy)]
pub(crate) struct View {
    pub(crate) center: Pos2,
    pub(crate) radius: f32,
    pub(crate) range: f64,
    pub(crate) rot: f64,
    pub(crate) cw: V2,
}

impl View {
    pub(crate) fn ppn(&self) -> f32 {
        self.radius / self.range as f32
    }
    pub(crate) fn vec2s(&self, v: V2) -> Vec2 {
        let (s, c) = self.rot.to_radians().sin_cos();
        let dx = v.x * c - v.y * s;
        let dy = v.x * s + v.y * c;
        vec2(dx as f32, -dy as f32) * self.ppn()
    }
    pub(crate) fn w2s(&self, p: V2) -> Pos2 {
        self.center + self.vec2s(p - self.cw)
    }
    pub(crate) fn s2vec(&self, d: Vec2) -> V2 {
        let (s, c) = self.rot.to_radians().sin_cos();
        let dx = (d.x / self.ppn()) as f64;
        let dy = (-d.y / self.ppn()) as f64;
        V2::new(dx * c + dy * s, -dx * s + dy * c)
    }
    pub(crate) fn s2w(&self, p: Pos2) -> V2 {
        self.cw + self.s2vec(p - self.center)
    }
    pub(crate) fn inside(&self, p: Pos2) -> bool {
        (p - self.center).length() <= self.radius
    }
    /// Einheitsvektor auf dem Bildschirm für eine rechtweisende Peilung.
    pub(crate) fn dir_s(&self, brg: f64) -> Vec2 {
        let a = (brg - self.rot).to_radians();
        vec2(a.sin() as f32, -a.cos() as f32)
    }
}

pub(crate) fn clip_circle(a: Pos2, b: Pos2, c: Pos2, r: f32) -> Option<(Pos2, Pos2)> {
    let d = b - a;
    let f = a - c;
    let aa = d.dot(d);
    if aa < 1e-6 {
        return if f.length() <= r { Some((a, b)) } else { None };
    }
    let bb = 2.0 * f.dot(d);
    let cc = f.dot(f) - r * r;
    let disc = bb * bb - 4.0 * aa * cc;
    if disc < 0.0 {
        return None;
    }
    let sq = disc.sqrt();
    let t0 = ((-bb - sq) / (2.0 * aa)).max(0.0);
    let t1 = ((-bb + sq) / (2.0 * aa)).min(1.0);
    if t0 > t1 {
        None
    } else {
        Some((a + d * t0, a + d * t1))
    }
}

pub(crate) fn line(p: &egui::Painter, v: &View, a: Pos2, b: Pos2, s: Stroke) {
    if let Some((a, b)) = clip_circle(a, b, v.center, v.radius) {
        p.line_segment([a, b], s);
    }
}

pub(crate) fn dashed(
    p: &egui::Painter,
    v: &View,
    a: Pos2,
    b: Pos2,
    s: Stroke,
    dash: f32,
    gap: f32,
) {
    if let Some((a, b)) = clip_circle(a, b, v.center, v.radius) {
        p.extend(Shape::dashed_line(&[a, b], s, dash, gap));
    }
}

pub(crate) fn circle_clipped(p: &egui::Painter, v: &View, c: Pos2, r: f32, s: Stroke, dash: bool) {
    if r < 1.0 {
        return;
    }
    let n = ((r * 0.6) as usize).clamp(48, 360);
    for k in 0..n {
        if dash && (k / 2) % 2 == 1 {
            continue;
        }
        let a0 = k as f32 / n as f32 * std::f32::consts::TAU;
        let a1 = (k + 1) as f32 / n as f32 * std::f32::consts::TAU;
        let pa = c + vec2(a0.sin(), -a0.cos()) * r;
        let pb = c + vec2(a1.sin(), -a1.cos()) * r;
        line(p, v, pa, pb, s);
    }
}

pub(crate) fn label_bg(
    p: &egui::Painter,
    pos: Pos2,
    anchor: Align2,
    text: &str,
    size: f32,
    color: Color32,
) {
    let galley = p.layout_no_wrap(text.to_string(), FontId::monospace(size), color);
    let rect = anchor.anchor_size(pos, galley.size());
    p.rect_filled(
        rect.expand(3.0),
        CornerRadius::same(3),
        Color32::from_black_alpha(170),
    );
    p.galley(rect.min, galley, color);
}

pub(crate) fn fb(b: f64) -> String {
    format!("{:05.1}°", norm360(b))
}

pub(crate) fn ftcpa(t: f64) -> String {
    if t.is_finite() {
        format!("{:5.1} min", t)
    } else {
        "  --- min".into()
    }
}

/// Minuten als m:ss
pub(crate) fn fmt_ms(min: f64) -> String {
    let secs = (min * 60.0).round() as i64;
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Zeitwert in 30-s-Schritten: [−30 s] [Schieber] [+30 s]
fn half_minute_stepper(ui: &mut egui::Ui, v: &mut f64, max: f64) {
    ui.horizontal(|ui| {
        if ui.small_button("−30 s").clicked() {
            *v = (*v - 0.5).max(0.5);
        }
        ui.add(
            egui::Slider::new(v, 0.5..=max)
                .step_by(0.5)
                .custom_formatter(|m, _| fmt_ms(m))
                .suffix(" min"),
        );
        if ui.small_button("+30 s").clicked() {
            *v = (*v + 0.5).min(max);
        }
    });
    *v = ((*v * 2.0).round() / 2.0).clamp(0.5, max);
}

/// Winkelfeld (Kurs/Peilung): Ziehen und Eingabe laufen fließend über 359° ↔ 0° in beide Richtungen.
pub(crate) fn angle_edit(
    ui: &mut egui::Ui,
    v: &mut f64,
    speed: f64,
    decimals: usize,
    suffix: &str,
) -> egui::Response {
    angle_edit_enabled(ui, true, v, speed, decimals, suffix)
}

pub(crate) fn angle_edit_enabled(
    ui: &mut egui::Ui,
    enabled: bool,
    v: &mut f64,
    speed: f64,
    decimals: usize,
    suffix: &str,
) -> egui::Response {
    let mut x = *v;
    // kein Bereich: der Wert darf beim Ziehen über 360 bzw. unter 0 laufen und wird danach normiert
    let r = ui.add_enabled(
        enabled,
        egui::DragValue::new(&mut x)
            .speed(speed)
            .fixed_decimals(decimals)
            .suffix(suffix.to_string()),
    );
    if r.changed() {
        *v = wrap_angle(x, decimals);
    }
    r
}

/// Auf die angezeigten Nachkommastellen runden und in 0..360 normieren (360,0 → 0,0).
pub(crate) fn wrap_angle(x: f64, decimals: usize) -> f64 {
    let k = 10f64.powi(decimals as i32);
    norm360((x * k).round() / k)
}

pub(crate) fn mono(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(RichText::new(text.into()).monospace());
}

/// Signalfarben (für dunklen Hintergrund gewählt) im hellen Thema lesbar machen.
pub(crate) fn adapt(ui: &egui::Ui, c: Color32) -> Color32 {
    if ui.visuals().dark_mode {
        c
    } else {
        Color32::from_rgb(
            (c.r() as f32 * 0.5) as u8,
            (c.g() as f32 * 0.5) as u8,
            (c.b() as f32 * 0.5) as u8,
        )
    }
}

pub(crate) fn mono_c(ui: &mut egui::Ui, text: impl Into<String>, c: Color32) {
    let c = adapt(ui, c);
    ui.label(RichText::new(text.into()).monospace().color(c));
}

pub(crate) fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(6.0);
    ui.label(
        RichText::new(title)
            .strong()
            .color(adapt(ui, Color32::from_rgb(140, 200, 255))),
    );
    ui.separator();
}

fn make_palette(scheme: ColorScheme, brill: f32, bg_level: f32) -> [Color32; 256] {
    let (bg0, lo, hi) = scheme.colors();
    // unter 1: abdunkeln; über 1: Richtung hellerer Tönung des Schemas aufhellen
    // (funktioniert auch bei fast schwarzen Hintergründen)
    let tint = scheme.light_bg();
    let bg: [u8; 3] = std::array::from_fn(|i| {
        let b = bg0[i] as f32;
        let v = if bg_level <= 1.0 {
            b * bg_level
        } else {
            b + (tint[i] as f32 - b) * (bg_level - 1.0)
        };
        v.clamp(0.0, 255.0) as u8
    });
    let mut pal = [Color32::BLACK; 256];
    for (k, c) in pal.iter_mut().enumerate() {
        let f = k as f32 / 255.0;
        let t = f.powf(0.8);
        let mix = (f * 3.0).min(1.0);
        let ch = |i: usize| {
            let col = (lo[i] as f32 + (hi[i] as f32 - lo[i] as f32) * t) * brill;
            (bg[i] as f32 + (col - bg[i] as f32) * mix).clamp(0.0, 255.0) as u8
        };
        *c = if k == 0 {
            Color32::from_rgb(bg[0], bg[1], bg[2])
        } else {
            Color32::from_rgb(ch(0), ch(1), ch(2))
        };
    }
    pal
}

const SIM_TEMPLATE: &str = "sim_standard";

/// Speicherbare Voreinstellung des Simulators (ohne Szenario).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct SimTemplate {
    band: Band,
    range_idx: usize,
    gain: f64,
    sea: f64,
    rain: f64,
    pulse: Option<Pulse>,
    ir: bool,
    sweep_line: bool,
    rings: bool,
    persistence: f64,
    brilliance: f32,
    bg_level: f32,
    scheme: ColorScheme,
    orientation: Orientation,
    motion: Motion,
    trail_mode: TrailMode,
    trail_min: f64,
    vector_mode: VectorMode,
    vector_min: f64,
    pp_interval: f64,
    stab_ground: bool,
    cpa_limit: f64,
    tcpa_limit: f64,
    arpa_on: bool,
    show_own_vector: bool,
    show_autopilot: bool,
    show_reltrack: bool,
    max_rot: f64,
    accel: f64,
    gz_inner: f64,
    gz_outer: f64,
}

/// Werkseinstellung
impl Default for SimTemplate {
    fn default() -> Self {
        Self {
            band: Band::X,
            range_idx: 6,
            gain: 55.0,
            sea: 35.0,
            rain: 0.0,
            pulse: None,
            ir: true,
            sweep_line: true,
            rings: true,
            persistence: 3.0,
            brilliance: 1.0,
            bg_level: 1.0,
            scheme: ColorScheme::Blue,
            orientation: Orientation::NorthUp,
            motion: Motion::Relative,
            trail_mode: TrailMode::True,
            trail_min: 3.0,
            vector_mode: VectorMode::Relative,
            vector_min: 6.0,
            pp_interval: 60.0,
            stab_ground: true,
            cpa_limit: 1.0,
            tcpa_limit: 12.0,
            arpa_on: true,
            show_own_vector: true,
            show_autopilot: true,
            show_reltrack: true,
            max_rot: 30.0,
            accel: 0.04,
            gz_inner: 3.0,
            gz_outer: 4.0,
        }
    }
}

pub struct RadarApp {
    world: World,
    radar: Radar,
    arpa: Arpa,
    renderer: PpiRenderer,
    image: egui::ColorImage,
    tex: Option<egui::TextureHandle>,

    orientation: Orientation,
    motion: Motion,
    cu_ref: f64,
    /// Off-Center in Bruchteilen des Radius (Bildschirmrahmen)
    offc: V2,
    tm_origin: V2,

    ebl: [Ebl; 2],
    vrm: [Vrm; 2],
    ebl_relative: bool,
    pi_on: bool,
    pi_brg: f64,
    pi_offset: f64,

    trail_mode: TrailMode,
    trail_min: f64,
    scheme: ColorScheme,
    brilliance: f32,
    persistence: f64,
    sweep_line: bool,
    rings: bool,
    palette: [Color32; 256],
    palette_key: (ColorScheme, f32, f32),
    bg_level: f32,
    show_autopilot: bool,
    ui_theme: UiTheme,
    show_own_vector: bool,
    /// Polygon-Guard-Zone in Bearbeitung (Punkte im Zonen-Rahmen)
    gzp_draw: Option<Vec<V2>>,

    paused: bool,
    warp_idx: usize,
    scenario: Scenario,
    form: AddForm,
    show_help: bool,
    show_truth: bool,
    show_reltrack: bool,
    snap_path: Option<std::path::PathBuf>,
    demo: Option<Demo>,
    mode: AppMode,
    plot: crate::plot::PlotState,
    demo_frames: u32,

    real_time: f64,
    msg: Option<(String, f64)>,
    ctx_world: Option<V2>,
    ctx_screen: Option<Pos2>,
    drag: DragMode,
    cursor: Option<V2>,
    scroll_acc: f32,
}

impl RadarApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Systemsprache übernehmen (für Tests per RADAR_LANG=de|en überschreibbar)
        let lang = match std::env::var("RADAR_LANG").ok().as_deref() {
            Some("de") => crate::i18n::Lang::De,
            Some("en") => crate::i18n::Lang::En,
            _ => crate::i18n::detect(),
        };
        crate::i18n::set_lang(lang);
        UiTheme::Dark.apply(&cc.egui_ctx);
        cc.egui_ctx.style_mut(|s| {
            s.spacing.item_spacing = vec2(6.0, 4.0);
            s.spacing.slider_width = 150.0;
        });
        let scheme = ColorScheme::Blue;
        let brilliance = 1.0;
        let mut app = Self {
            world: World::new(),
            radar: Radar::new(),
            arpa: Arpa::new(),
            renderer: PpiRenderer::new(),
            image: egui::ColorImage::new([IMG, IMG], Color32::TRANSPARENT),
            tex: None,
            orientation: Orientation::NorthUp,
            motion: Motion::Relative,
            cu_ref: 0.0,
            offc: V2::ZERO,
            tm_origin: V2::ZERO,
            ebl: [
                Ebl {
                    on: false,
                    brg: 45.0,
                },
                Ebl {
                    on: false,
                    brg: 315.0,
                },
            ],
            vrm: [
                Vrm {
                    on: false,
                    range: 2.0,
                },
                Vrm {
                    on: false,
                    range: 4.0,
                },
            ],
            ebl_relative: false,
            pi_on: false,
            pi_brg: 0.0,
            pi_offset: 1.0,
            trail_mode: TrailMode::True,
            trail_min: 3.0,
            scheme,
            brilliance,
            persistence: 3.0,
            sweep_line: true,
            rings: true,
            palette: make_palette(scheme, brilliance, 1.0),
            palette_key: (scheme, brilliance, 1.0),
            bg_level: 1.0,
            show_autopilot: true,
            ui_theme: UiTheme::Dark,
            show_own_vector: true,
            gzp_draw: None,
            paused: false,
            warp_idx: 0,
            scenario: Scenario::Empty,
            form: AddForm {
                kind: ShipType::Container,
                brg: 45.0,
                rng: 8.0,
                mode: CourseMode::Collision,
                course: 270.0,
                speed: 18.0,
                cpa: 0.5,
            },
            show_help: false,
            show_truth: false,
            show_reltrack: true,
            real_time: 0.0,
            msg: None,
            ctx_world: None,
            ctx_screen: None,
            drag: DragMode::None,
            cursor: None,
            scroll_acc: 0.0,
            snap_path: None,
            demo: Demo::from_env(),
            mode: AppMode::Start,
            plot: crate::plot::PlotState::new(),
            demo_frames: 0,
        };
        app.load_scenario(Scenario::Empty);
        if app.demo.is_none() || std::env::var_os("RADAR_DEMO_TEMPLATE").is_some() {
            // gespeichertes Standard-Template (ohne Szenario) anwenden
            if let Some(tp) = crate::templates::load::<SimTemplate>(SIM_TEMPLATE) {
                app.apply_sim_template(&tp);
            }
        }
        if app.demo.is_some() {
            if std::env::var("RADAR_DEMO_MODE").is_ok_and(|m| m == "plot")
                && std::env::var_os("RADAR_DEMO_TEMPLATE").is_none()
            {
                app.plot.load_example();
            }
            app.mode = match std::env::var("RADAR_DEMO_MODE").as_deref() {
                Ok("plot") => AppMode::Plot,
                Ok("start") => AppMode::Start,
                _ => AppMode::Sim,
            };
            if std::env::var_os("RADAR_DEMO_COCK").is_some() {
                app.plot.show_cockcroft = true;
            }
            if std::env::var_os("RADAR_DEMO_REASON").is_some() {
                app.plot.show_reason = true;
            }
            if let Ok(t) = std::env::var("RADAR_DEMO_THEME") {
                app.ui_theme = match t.as_str() {
                    "gray" => UiTheme::Gray,
                    "light" => UiTheme::Light,
                    _ => UiTheme::Dark,
                };
                app.ui_theme.apply(&cc.egui_ctx);
            }
        }
        app
    }

    fn sim_template(&self) -> SimTemplate {
        SimTemplate {
            band: self.radar.band(),
            range_idx: self.radar.range_idx,
            gain: self.radar.gain,
            sea: self.radar.sea,
            rain: self.radar.rain,
            pulse: self.radar.pulse_sel,
            ir: self.radar.ir,
            sweep_line: self.sweep_line,
            rings: self.rings,
            persistence: self.persistence,
            brilliance: self.brilliance,
            bg_level: self.bg_level,
            scheme: self.scheme,
            orientation: self.orientation,
            motion: self.motion,
            trail_mode: self.trail_mode,
            trail_min: self.trail_min,
            vector_mode: self.arpa.vector_mode,
            vector_min: self.arpa.vector_min,
            pp_interval: self.arpa.pp_interval,
            stab_ground: self.arpa.stab_ground,
            cpa_limit: self.arpa.cpa_limit,
            tcpa_limit: self.arpa.tcpa_limit,
            arpa_on: self.arpa.enabled,
            show_own_vector: self.show_own_vector,
            show_autopilot: self.show_autopilot,
            show_reltrack: self.show_reltrack,
            max_rot: self.world.own.max_rot,
            accel: self.world.own.accel,
            gz_inner: self.arpa.gz_inner,
            gz_outer: self.arpa.gz_outer,
        }
    }

    fn apply_sim_template(&mut self, tp: &SimTemplate) {
        self.radar.set_band(tp.band);
        self.set_range(tp.range_idx.min(RANGES.len() - 1));
        self.radar.gain = tp.gain;
        self.radar.sea = tp.sea;
        self.radar.rain = tp.rain;
        self.radar.pulse_sel = tp.pulse;
        self.radar.ir = tp.ir;
        self.sweep_line = tp.sweep_line;
        self.rings = tp.rings;
        self.persistence = tp.persistence;
        self.brilliance = tp.brilliance;
        self.bg_level = tp.bg_level;
        self.scheme = tp.scheme;
        self.orientation = tp.orientation;
        self.motion = tp.motion;
        if self.motion == Motion::True && self.orientation == Orientation::HeadUp {
            self.orientation = Orientation::NorthUp;
        }
        if self.motion == Motion::True {
            self.tm_reset();
        }
        self.trail_mode = tp.trail_mode;
        self.trail_min = tp.trail_min;
        self.arpa.vector_mode = tp.vector_mode;
        self.arpa.vector_min = tp.vector_min;
        self.arpa.pp_interval = tp.pp_interval;
        self.arpa.stab_ground = tp.stab_ground;
        self.arpa.cpa_limit = tp.cpa_limit;
        self.arpa.tcpa_limit = tp.tcpa_limit;
        self.arpa.enabled = tp.arpa_on;
        self.show_own_vector = tp.show_own_vector;
        self.show_autopilot = tp.show_autopilot;
        self.show_reltrack = tp.show_reltrack;
        self.world.own.max_rot = tp.max_rot;
        self.world.own.accel = tp.accel;
        self.arpa.gz_inner = tp.gz_inner;
        self.arpa.gz_outer = tp.gz_outer;
    }

    fn say(&mut self, s: impl Into<String>) {
        self.msg = Some((s.into(), self.real_time + 4.0));
    }

    fn load_scenario(&mut self, sc: Scenario) {
        self.world.load_scenario(sc);
        self.arpa.reset();
        self.radar.clear_video();
        self.radar.clear_trails();
        self.radar.last_detect.clear();
        self.radar.events.clear();
        self.cu_ref = self.world.own.heading;
        self.offc = V2::ZERO;
        self.tm_origin =
            self.world.own.pos + V2::from_bearing(self.world.own.heading, 0.5 * self.radar.range());
        self.arpa.trial.course = self.world.own.cmd_heading;
        self.arpa.trial.speed = self.world.own.cmd_speed;
        self.say(tf!(
            "Szenario geladen: {}",
            "Scenario loaded: {}",
            sc.name()
        ));
    }

    fn own_ref(&self) -> OwnRef {
        OwnRef::from_world(&self.world)
    }

    fn display_rot(&self) -> f64 {
        match self.orientation {
            Orientation::NorthUp => 0.0,
            Orientation::HeadUp => self.world.own.heading,
            Orientation::CourseUp => self.cu_ref,
        }
    }

    fn screen_frac_to_world(&self, f: V2, range: f64) -> V2 {
        let (s, c) = self.display_rot().to_radians().sin_cos();
        let (dx, dy) = (f.x * range, f.y * range);
        V2::new(dx * c + dy * s, -dx * s + dy * c)
    }

    fn center_world(&self) -> V2 {
        let range = self.radar.range();
        match self.motion {
            Motion::Relative => self.world.own.pos - self.screen_frac_to_world(self.offc, range),
            Motion::True => self.tm_origin,
        }
    }

    fn tm_reset(&mut self) {
        let o = self.own_ref();
        let cog = if o.gvel.len() > 0.2 {
            o.gvel.bearing()
        } else {
            self.world.own.heading
        };
        self.tm_origin = self.world.own.pos + V2::from_bearing(cog, 0.6 * self.radar.range());
    }

    fn check_tm(&mut self) {
        if self.motion == Motion::True
            && (self.world.own.pos - self.tm_origin).len() > 0.66 * self.radar.range()
        {
            self.tm_reset();
        }
    }

    fn set_range(&mut self, idx: usize) {
        self.radar.set_range(idx);
        self.check_tm();
    }

    fn step_sim(&mut self, ctx: &egui::Context) {
        let real_dt = ctx.input(|i| i.stable_dt).min(0.1) as f64;
        if !self.paused {
            let mut remaining = real_dt * WARPS[self.warp_idx];
            while remaining > 1e-9 {
                let h = remaining.min(0.2);
                self.world.step(h);
                self.radar.update(h, &self.world);
                let ev = std::mem::take(&mut self.radar.events);
                self.arpa.process(&ev);
                self.arpa.update(&self.world);
                remaining -= h;
            }
        }
        self.check_tm();
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        struct K {
            enter: bool,
            a: bool,
            space: bool,
            plus: bool,
            minus: bool,
            left: bool,
            right: bool,
            up: bool,
            down: bool,
            del: bool,
            esc: bool,
            shift: bool,
            t: bool,
            h: bool,
            nums: [bool; 7],
        }
        let k = ctx.input(|i| K {
            space: i.key_pressed(Key::Space) || i.key_pressed(Key::P),
            plus: i.key_pressed(Key::Plus)
                || i.key_pressed(Key::Equals)
                || i.key_pressed(Key::PageUp),
            minus: i.key_pressed(Key::Minus) || i.key_pressed(Key::PageDown),
            left: i.key_pressed(Key::ArrowLeft),
            right: i.key_pressed(Key::ArrowRight),
            up: i.key_pressed(Key::ArrowUp),
            down: i.key_pressed(Key::ArrowDown),
            del: i.key_pressed(Key::Delete),
            esc: i.key_pressed(Key::Escape),
            enter: i.key_pressed(Key::Enter),
            a: i.key_pressed(Key::A),
            shift: i.modifiers.shift,
            t: i.key_pressed(Key::T),
            h: i.key_pressed(Key::F1) || i.key_pressed(Key::H),
            nums: [
                i.key_pressed(Key::Num1),
                i.key_pressed(Key::Num2),
                i.key_pressed(Key::Num3),
                i.key_pressed(Key::Num4),
                i.key_pressed(Key::Num5),
                i.key_pressed(Key::Num6),
                i.key_pressed(Key::Num7),
            ],
        });
        if k.space {
            self.paused = !self.paused;
        }
        if k.plus && self.radar.range_idx + 1 < RANGES.len() {
            self.set_range(self.radar.range_idx + 1);
        }
        if k.minus && self.radar.range_idx > 0 {
            self.set_range(self.radar.range_idx - 1);
        }
        let dc = if k.shift { 10.0 } else { 1.0 };
        let own = &mut self.world.own;
        if k.left {
            own.cmd_heading = norm360(own.cmd_heading - dc);
        }
        if k.right {
            own.cmd_heading = norm360(own.cmd_heading + dc);
        }
        let ds = if k.shift { 2.0 } else { 0.5 };
        if k.up {
            own.cmd_speed = (own.cmd_speed + ds).min(own.max_speed);
        }
        if k.down {
            own.cmd_speed = (own.cmd_speed - ds).max(0.0);
        }
        if k.del {
            if let Some(n) = self.arpa.selected {
                self.arpa.remove_track(n);
            }
        }
        if k.esc {
            if self.gzp_draw.is_some() {
                self.gzp_draw = None;
                self.say(t(
                    "Polygon-Guard-Zone abgebrochen",
                    "Polygon guard zone cancelled",
                ));
            } else {
                self.arpa.selected = None;
            }
        }
        if k.enter && self.gzp_draw.is_some() {
            self.finish_polygon();
        }
        if k.a {
            self.show_autopilot = !self.show_autopilot;
        }
        if k.t {
            self.toggle_trial();
        }
        if k.h {
            self.show_help = !self.show_help;
        }
        for (i, n) in k.nums.iter().enumerate() {
            if *n {
                self.warp_idx = i;
            }
        }
    }

    fn start_polygon(&mut self) {
        self.gzp_draw = Some(Vec::new());
        self.say(t(
            "Polygon-Guard-Zone: Eckpunkte per Linksklick setzen, Enter = fertig",
            "Polygon guard zone: set corners with left-click, Enter = done",
        ));
    }

    fn finish_polygon(&mut self) {
        if let Some(pts) = self.gzp_draw.take() {
            if pts.len() >= 3 {
                self.arpa.gzp_points = pts;
                self.arpa.gzp_on = true;
                self.say(tf!(
                    "Polygon-Guard-Zone aktiv ({} Punkte)",
                    "Polygon guard zone active ({} points)",
                    self.arpa.gzp_points.len()
                ));
            } else {
                self.say(t(
                    "Polygon braucht mindestens 3 Punkte – verworfen",
                    "Polygon needs at least 3 points – discarded",
                ));
            }
        }
    }

    fn toggle_trial(&mut self) {
        self.arpa.trial.active = !self.arpa.trial.active;
        if self.arpa.trial.active {
            self.arpa.vector_mode = VectorMode::Relative;
            self.say(t(
                "TRIAL-Manöver aktiv – Endpunkt des orangen Vektors ziehen",
                "TRIAL manoeuvre active – drag the end of the orange vector",
            ));
        }
    }

    fn place_target(&mut self, kind: ShipType, pos: V2) {
        let speed = if kind == ShipType::Buoy {
            0.0
        } else {
            self.form.speed
        };
        let o = self.own_ref();
        let course = match self.form.mode {
            CourseMode::Manual => Some(self.form.course),
            CourseMode::Collision => course_for_cpa(o.pos, o.gvel, pos, speed, 0.0),
            CourseMode::Cpa => course_for_cpa(o.pos, o.gvel, pos, speed, self.form.cpa),
        };
        let course = match course {
            Some(c) => c,
            None => {
                self.say(t("Gewünschter CPA mit dieser Fahrt nicht erreichbar – Kurs auf Eigenschiff gesetzt", "Requested CPA not reachable at this speed – course set towards own ship"));
                (o.pos - pos).bearing()
            }
        };
        let id = self.world.add_target(kind, pos, course, speed);
        self.say(tf!(
            "{} #{} platziert: Kurs {}, {:.1} kn",
            "{} #{} placed: course {}, {:.1} kn",
            kind.name(),
            id,
            fb(course),
            speed
        ));
    }

    fn nearest_track(&self, p: Pos2, view: &View, max_px: f32) -> Option<u32> {
        let now = self.world.time;
        let mut best: Option<(u32, f32)> = None;
        for tr in &self.arpa.tracks {
            let d = (view.w2s(tr.display_pos(now)) - p).length();
            if d < max_px && best.is_none_or(|b| d < b.1) {
                best = Some((tr.num, d));
            }
        }
        best.map(|b| b.0)
    }

    fn acquire_at(&mut self, p: Pos2, view: &View) {
        if !self.radar.transmit {
            self.say(t(
                "Radar in STANDBY – keine Zielerfassung möglich",
                "Radar in STANDBY – acquisition not possible",
            ));
            return;
        }
        if !self.arpa.enabled {
            self.say(t("ARPA ist ausgeschaltet", "ARPA is switched off"));
            return;
        }
        let now = self.world.time;
        let gate = (0.12 * view.ppn()).max(16.0);
        let mut best: Option<(u32, f32, V2)> = None;
        for t in &self.world.targets {
            if let Some(&(tm, vis)) = self.radar.last_detect.get(&t.id) {
                if vis && now - tm < ANT_PERIOD * 2.5 {
                    let d = (view.w2s(t.pos) - p).length();
                    if d < gate && best.is_none_or(|b| d < b.1) {
                        best = Some((t.id, d, t.pos));
                    }
                }
            }
        }
        match best {
            Some((id, _, pos)) => {
                if let Some(n) = self.arpa.is_tracked(id) {
                    self.arpa.selected = Some(n);
                } else if let Some(n) = self.arpa.acquire(id, now, pos, false) {
                    self.say(tf!(
                        "Ziel {} erfasst – Erfassungsphase (ca. 1 min)",
                        "Target {} acquired – acquisition phase (approx. 1 min)",
                        n
                    ));
                } else {
                    self.say(t(
                        "Maximale Anzahl Tracks erreicht",
                        "Maximum number of tracks reached",
                    ));
                }
            }
            None => self.say(t(
                "Kein Echo im Erfassungsfenster",
                "No echo in acquisition gate",
            )),
        }
    }

    fn ring_bearing(&self, view: &View, p: Pos2) -> f64 {
        let d = p - view.center;
        norm360((d.x as f64).atan2(-d.y as f64).to_degrees() + view.rot)
    }

    fn trial_origin(&self, o: &OwnRef) -> V2 {
        o.pos + o.gvel * (self.arpa.trial.delay_min / 60.0)
    }

    fn hit_drag(&self, at: Pos2, view: &View) -> DragMode {
        let own = &self.world.own;
        let o = self.own_ref();
        let t = self.arpa.vector_min / 60.0;
        let own_s = view.w2s(own.pos);
        let tip = view.w2s(own.pos + V2::from_bearing(own.cmd_heading, own.cmd_speed * t));
        if self.show_autopilot && (at - tip).length() < 12.0 {
            return DragMode::OwnVector;
        }
        if self.arpa.trial.active {
            let tip = view.w2s(self.trial_origin(&o) + self.arpa.trial_ground_vel(&o) * t);
            if (at - tip).length() < 12.0 {
                return DragMode::TrialVector;
            }
        }
        let dc = (at - view.center).length();
        if self.show_autopilot && dc > view.radius && dc < view.radius + 34.0 {
            return DragMode::Course;
        }
        for i in 0..2 {
            if self.vrm[i].on
                && ((at - own_s).length() - self.vrm[i].range as f32 * view.ppn()).abs() < 7.0
            {
                return DragMode::Vrm(i);
            }
        }
        for i in 0..2 {
            if self.ebl[i].on {
                let d = at - own_s;
                let dir = view.dir_s(self.ebl[i].brg);
                let along = d.dot(dir);
                let perp = (d.x * dir.y - d.y * dir.x).abs();
                if along > 10.0 && perp < 7.0 {
                    return DragMode::Ebl(i);
                }
            }
        }
        DragMode::None
    }

    fn apply_drag(&mut self, p: Pos2, view: &View) {
        let t = self.arpa.vector_min / 60.0;
        let o = self.own_ref();
        match self.drag {
            DragMode::None => {}
            DragMode::OwnVector => {
                let w = view.s2w(p) - o.pos;
                let own = &mut self.world.own;
                own.cmd_heading = norm360(w.bearing().round());
                own.cmd_speed = ((w.len() / t * 10.0).round() / 10.0).clamp(0.0, own.max_speed);
            }
            DragMode::Course => {
                self.world.own.cmd_heading = norm360(self.ring_bearing(view, p).round());
            }
            DragMode::TrialVector => {
                let w = view.s2w(p) - self.trial_origin(&o);
                let g = w * (1.0 / t) - o.current();
                self.arpa.trial.course = norm360(g.bearing().round());
                self.arpa.trial.speed =
                    ((g.len() * 10.0).round() / 10.0).clamp(0.0, self.world.own.max_speed);
            }
            DragMode::Vrm(i) => self.vrm[i].range = (view.s2w(p) - o.pos).len(),
            DragMode::Ebl(i) => self.ebl[i].brg = (view.s2w(p) - o.pos).bearing(),
        }
    }

    fn click(&mut self, p: Pos2, view: &View) {
        let dc = (p - view.center).length();
        if let Some(pts) = self.gzp_draw.as_mut() {
            if dc <= view.radius {
                let rel = view.s2w(p) - self.world.own.pos;
                let q = self.arpa.rel_to_gzp(rel, self.world.own.heading);
                pts.push(q);
            }
            return;
        }
        if self.show_autopilot && dc > view.radius && dc < view.radius + 34.0 {
            let b = norm360(self.ring_bearing(view, p).round());
            self.world.own.cmd_heading = b;
            self.say(tf!("Sollkurs {}", "Set course {}", fb(b)));
        } else if dc <= view.radius {
            if let Some(n) = self.nearest_track(p, view, 14.0) {
                self.arpa.selected = Some(n);
            } else {
                self.acquire_at(p, view);
            }
        }
    }

    fn context_menu(&mut self, ui: &mut egui::Ui, view: &View) {
        let Some(wp) = self.ctx_world else {
            ui.close_menu();
            return;
        };
        let rel = wp - self.world.own.pos;
        if self.gzp_draw.is_some() {
            ui.label(
                RichText::new(t("Polygon-Guard-Zone", "Polygon guard zone"))
                    .strong()
                    .color(adapt(ui, C_GZ)),
            );
            if ui
                .button(t("Punkt hier hinzufügen", "Add point here"))
                .clicked()
            {
                let q = self.arpa.rel_to_gzp(rel, self.world.own.heading);
                if let Some(d) = self.gzp_draw.as_mut() {
                    d.push(q);
                }
                ui.close_menu();
            }
            if ui
                .button(t("Letzten Punkt entfernen", "Remove last point"))
                .clicked()
            {
                if let Some(d) = self.gzp_draw.as_mut() {
                    d.pop();
                }
                ui.close_menu();
            }
            if ui
                .button(t("✔ Polygon abschließen", "✔ Close polygon"))
                .clicked()
            {
                self.finish_polygon();
                ui.close_menu();
            }
            if ui.button(t("Abbrechen", "Cancel")).clicked() {
                self.gzp_draw = None;
                ui.close_menu();
            }
            return;
        }
        ui.label(
            RichText::new(format!("{}   {:.2} nm", fb(rel.bearing()), rel.len()))
                .monospace()
                .strong(),
        );
        ui.separator();
        if ui
            .button(t("ARPA: Ziel hier erfassen", "ARPA: acquire target here"))
            .clicked()
        {
            if let Some(p) = self.ctx_screen {
                self.acquire_at(p, view);
            }
            ui.close_menu();
        }
        if let Some(p) = self.ctx_screen {
            if let Some(n) = self.nearest_track(p, view, 16.0) {
                if ui
                    .button(tf!("ARPA: Track {} löschen", "ARPA: cancel track {}", n))
                    .clicked()
                {
                    self.arpa.remove_track(n);
                    ui.close_menu();
                }
            }
        }
        ui.menu_button(
            t(
                "Ziel hier platzieren  (Kurs gemäß Ziel-Formular)",
                "Place target here  (course per target form)",
            ),
            |ui| {
                for k in ShipType::ALL {
                    if ui.button(k.name()).clicked() {
                        self.place_target(k, wp);
                        ui.close_menu();
                    }
                }
            },
        );
        ui.separator();
        for i in 0..2 {
            if ui
                .button(tf!("EBL{} hierher", "EBL{} here", i + 1))
                .clicked()
            {
                self.ebl[i] = Ebl {
                    on: true,
                    brg: rel.bearing(),
                };
                ui.close_menu();
            }
        }
        for i in 0..2 {
            if ui
                .button(tf!("VRM{} hierher", "VRM{} here", i + 1))
                .clicked()
            {
                self.vrm[i] = Vrm {
                    on: true,
                    range: rel.len(),
                };
                ui.close_menu();
            }
        }
        if ui
            .button(t("EBL1 + VRM1 hierher", "EBL1 + VRM1 here"))
            .clicked()
        {
            self.ebl[0] = Ebl {
                on: true,
                brg: rel.bearing(),
            };
            self.vrm[0] = Vrm {
                on: true,
                range: rel.len(),
            };
            ui.close_menu();
        }
        if ui
            .button(t(
                "PI-Linie durch diesen Punkt (Sollkurs)",
                "PI line through this point (set course)",
            ))
            .clicked()
        {
            self.pi_on = true;
            self.pi_brg = self.world.own.cmd_heading;
            let perp = V2::from_bearing(self.pi_brg + 90.0, 1.0);
            self.pi_offset = rel.dot(perp);
            ui.close_menu();
        }
        ui.separator();
        if ui
            .button(t(
                "Sollkurs auf diese Peilung",
                "Set course to this bearing",
            ))
            .clicked()
        {
            self.world.own.cmd_heading = norm360(rel.bearing().round());
            ui.close_menu();
        }
        if ui
            .button(t(
                "Off-Center: Eigenschiff hierher",
                "Off-centre: own ship here",
            ))
            .clicked()
        {
            if let Some(p) = self.ctx_screen {
                let d = p - view.center;
                let mut f = V2::new(d.x as f64, -d.y as f64) * (1.0 / view.radius as f64);
                if f.len() > 0.66 {
                    f = f.normalized() * 0.66;
                }
                match self.motion {
                    Motion::Relative => self.offc = f,
                    Motion::True => {
                        self.tm_origin =
                            self.world.own.pos - self.screen_frac_to_world(f, view.range);
                    }
                }
            }
            ui.close_menu();
        }
        if ui.button(t("Zentrieren", "Centre")).clicked() {
            self.offc = V2::ZERO;
            if self.motion == Motion::True {
                self.tm_origin = self.world.own.pos;
            }
            ui.close_menu();
        }
        if ui
            .button(t(
                "Guard Zone auf diese Entfernung (±0,5 nm)",
                "Guard zone at this range (±0.5 nm)",
            ))
            .clicked()
        {
            self.arpa.gz_on = true;
            self.arpa.gz_inner = (rel.len() - 0.5).max(0.1);
            self.arpa.gz_outer = rel.len() + 0.5;
            ui.close_menu();
        }
        if ui
            .button(t("Polygon-Guard-Zone zeichnen", "Draw polygon guard zone"))
            .clicked()
        {
            self.start_polygon();
            ui.close_menu();
        }
        if (self.arpa.gz_on || self.arpa.gzp_on)
            && ui
                .button(t("Guard Zones löschen", "Delete guard zones"))
                .clicked()
        {
            self.arpa.clear_guard_zones();
            self.say(t("Guard Zones gelöscht", "Guard zones deleted"));
            ui.close_menu();
        }
    }

    // -----------------------------------------------------------------------
    // PPI
    // -----------------------------------------------------------------------

    fn ppi(&mut self, ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let resp = ui.allocate_rect(rect, Sense::click_and_drag());
        let side = rect.width().min(rect.height());
        let radius = (side / 2.0 - 40.0).max(60.0);
        let view = View {
            center: rect.center(),
            radius,
            range: self.radar.range(),
            rot: self.display_rot(),
            cw: self.center_world(),
        };

        // --- Eingaben
        if resp.hovered() {
            let dy = ui.input(|i| i.raw_scroll_delta.y);
            self.scroll_acc += dy;
            if self.scroll_acc > 40.0 {
                self.scroll_acc = 0.0;
                if self.radar.range_idx > 0 {
                    self.set_range(self.radar.range_idx - 1);
                }
            } else if self.scroll_acc < -40.0 {
                self.scroll_acc = 0.0;
                if self.radar.range_idx + 1 < RANGES.len() {
                    self.set_range(self.radar.range_idx + 1);
                }
            }
        }
        self.cursor = resp
            .hover_pos()
            .filter(|hp| view.inside(*hp))
            .map(|hp| view.s2w(hp) - self.world.own.pos);
        if resp.drag_started_by(egui::PointerButton::Primary) {
            let origin = ui
                .input(|i| i.pointer.press_origin())
                .or(resp.interact_pointer_pos());
            if let Some(o) = origin {
                self.drag = self.hit_drag(o, &view);
            }
        }
        if self.drag != DragMode::None && resp.dragged_by(egui::PointerButton::Primary) {
            if let Some(p) = resp.interact_pointer_pos() {
                self.apply_drag(p, &view);
            }
        }
        if resp.drag_stopped() {
            self.drag = DragMode::None;
        }
        if resp.clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                self.click(p, &view);
            }
        }
        if resp.secondary_clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                self.ctx_screen = Some(p);
                self.ctx_world = Some(view.s2w(p));
            }
        }
        resp.context_menu(|ui| self.context_menu(ui, &view));

        // Ansicht nach Eingaben neu bestimmen (Bereich kann sich geändert haben)
        let view = View {
            range: self.radar.range(),
            rot: self.display_rot(),
            cw: self.center_world(),
            ..view
        };

        // --- Radarvideo rendern
        let own_s = view.w2s(self.world.own.pos);
        let k = (IMG as f32 / 2.0) / view.radius;
        let off = ((own_s.x - view.center.x) * k, (own_s.y - view.center.y) * k);
        let key = (self.scheme, self.brilliance, self.bg_level);
        if self.palette_key != key {
            self.palette = make_palette(self.scheme, self.brilliance, self.bg_level);
            self.palette_key = key;
        }
        self.renderer.render(
            &self.radar,
            off,
            view.rot,
            self.world.time,
            &self.palette,
            self.persistence * ANT_PERIOD,
            &mut self.image,
        );
        let opts = egui::TextureOptions::LINEAR;
        match &mut self.tex {
            Some(t) => t.set(self.image.clone(), opts),
            None => self.tex = Some(ui.ctx().load_texture("ppi", self.image.clone(), opts)),
        }

        let painter = ui.painter_at(rect);
        self.draw_ppi(&painter, &view, rect);
    }

    fn draw_ppi(&self, p: &egui::Painter, v: &View, rect: Rect) {
        let now = self.world.time;
        let own = &self.world.own;
        let o = self.own_ref();
        let own_s = v.w2s(own.pos);
        let ppn = v.ppn();
        let tmin = self.arpa.vector_min / 60.0;
        let blink = (self.real_time * 2.0) as i64 % 2 == 0;
        let (_, _, hi) = self.scheme.colors();
        let echo_col = Color32::from_rgb(hi[0], hi[1], hi[2]);

        // Hintergrund + Video
        p.circle_filled(v.center, v.radius + 34.0, self.scheme.bezel());
        if let Some(tex) = &self.tex {
            p.image(
                tex.id(),
                Rect::from_center_size(v.center, vec2(v.radius * 2.0, v.radius * 2.0)),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        if !self.radar.transmit {
            label_bg(
                p,
                v.center + vec2(0.0, -60.0),
                Align2::CENTER_CENTER,
                "STANDBY",
                22.0,
                C_LOST,
            );
        }

        // Echospuren
        if self.trail_mode != TrailMode::Off && self.radar.transmit {
            let len = self.trail_min * 60.0;
            for pts in self.radar.trails.values() {
                let mut prev: Option<Pos2> = None;
                for tp in pts.iter().filter(|tp| now - tp.t <= len) {
                    let s = match self.trail_mode {
                        TrailMode::True => v.w2s(tp.world),
                        _ => v.w2s(own.pos + tp.rel),
                    };
                    if let Some(a) = prev {
                        let age = ((now - tp.t) / len) as f32;
                        let c = echo_col.gamma_multiply(0.15 + 0.4 * (1.0 - age));
                        line(p, v, a, s, Stroke::new(3.0_f32, c));
                    }
                    prev = Some(s);
                }
            }
        }

        // Peilskala
        p.circle_stroke(v.center, v.radius, Stroke::new(1.5_f32, C_SCALE));
        let label_step = if v.radius > 330.0 { 10 } else { 30 };
        for deg in 0..360 {
            let dir = v.dir_s(deg as f64);
            let len = if deg % 10 == 0 {
                10.0
            } else if deg % 5 == 0 {
                6.0
            } else {
                3.0
            };
            p.line_segment(
                [v.center + dir * v.radius, v.center + dir * (v.radius + len)],
                Stroke::new(1.0_f32, C_SCALE),
            );
            if deg % label_step == 0 {
                p.text(
                    v.center + dir * (v.radius + 22.0),
                    Align2::CENTER_CENTER,
                    format!("{:03}", deg),
                    FontId::monospace(11.0),
                    C_SCALE,
                );
            }
        }
        // Soll-/Istkursmarken an der Skala
        let mark = |brg: f64, col: Color32, filled: bool| {
            let d = v.dir_s(brg);
            let n = vec2(-d.y, d.x);
            let tip = v.center + d * (v.radius + 1.0);
            let pts = vec![tip, tip + d * 12.0 + n * 6.0, tip + d * 12.0 - n * 6.0];
            if filled {
                p.add(Shape::convex_polygon(pts, col, Stroke::NONE));
            } else {
                p.add(Shape::closed_line(pts, Stroke::new(1.5_f32, col)));
            }
        };
        mark(own.heading, C_HDG, true);
        if self.show_autopilot {
            mark(own.cmd_heading, C_CMD, false);
        }
        if self.arpa.trial.active {
            mark(self.arpa.trial.course, C_TRIAL, false);
        }

        // Ringe
        if self.rings {
            let ri = self.radar.ring_interval();
            let n = (v.range * BUF_FACTOR / ri).ceil() as usize;
            for k in 1..=n {
                circle_clipped(
                    p,
                    v,
                    own_s,
                    (k as f64 * ri) as f32 * ppn,
                    Stroke::new(1.0_f32, self.scheme.ring_color().gamma_multiply(0.7)),
                    false,
                );
            }
        }

        // Vorausstrich
        line(
            p,
            v,
            own_s,
            own_s + v.dir_s(own.heading) * v.radius * 3.0,
            Stroke::new(1.3_f32, C_HDG),
        );
        dashed(
            p,
            v,
            own_s,
            own_s - v.dir_s(own.heading) * v.radius * 3.0,
            Stroke::new(1.0_f32, C_HDG.gamma_multiply(0.35)),
            4.0,
            8.0,
        );

        // Antennenstrahl
        if self.sweep_line && self.radar.transmit {
            line(
                p,
                v,
                own_s,
                own_s + v.dir_s(self.radar.antenna) * v.radius * 3.0,
                Stroke::new(1.0_f32, echo_col.gamma_multiply(0.35)),
            );
        }

        // Guard Zone (Ring)
        if self.arpa.gz_on {
            let s = Stroke::new(1.2_f32, C_GZ.gamma_multiply(0.8));
            circle_clipped(p, v, own_s, self.arpa.gz_inner as f32 * ppn, s, true);
            circle_clipped(p, v, own_s, self.arpa.gz_outer as f32 * ppn, s, true);
        }
        // Guard Zone (Polygon) und Polygon in Bearbeitung
        let to_s = |q: &V2| v.w2s(own.pos + self.arpa.gzp_to_rel(*q, own.heading));
        if self.arpa.gzp_on && self.arpa.gzp_points.len() >= 3 {
            let pts: Vec<Pos2> = self.arpa.gzp_points.iter().map(to_s).collect();
            let s = Stroke::new(1.4_f32, C_GZ.gamma_multiply(0.9));
            for i in 0..pts.len() {
                dashed(p, v, pts[i], pts[(i + 1) % pts.len()], s, 8.0, 5.0);
            }
            if let Some(c) = pts.first() {
                if v.inside(*c) {
                    p.text(
                        *c + vec2(6.0, -6.0),
                        Align2::LEFT_BOTTOM,
                        "GZ",
                        FontId::monospace(11.0),
                        C_GZ,
                    );
                }
            }
        }
        if let Some(draw) = &self.gzp_draw {
            let mut pts: Vec<Pos2> = draw.iter().map(to_s).collect();
            if let Some(c) = self.cursor {
                pts.push(v.w2s(own.pos + c));
            }
            let s = Stroke::new(1.6_f32, C_GZ);
            for w in pts.windows(2) {
                line(p, v, w[0], w[1], s);
            }
            if pts.len() >= 3 {
                dashed(
                    p,
                    v,
                    pts[pts.len() - 1],
                    pts[0],
                    Stroke::new(1.0_f32, C_GZ.gamma_multiply(0.6)),
                    4.0,
                    4.0,
                );
            }
            for q in draw.iter().map(to_s) {
                p.circle_filled(q, 3.5, C_GZ);
            }
            label_bg(
                p,
                rect.center_top() + vec2(0.0, 40.0),
                Align2::CENTER_TOP,
                &tf!("GUARD-ZONE-POLYGON: Punkte setzen per Linksklick ({}) – Enter = fertig, Esc = abbrechen", "GUARD ZONE POLYGON: set points with left-click ({}) – Enter = done, Esc = cancel",
                    draw.len()
                ),
                13.0,
                C_GZ,
            );
        }

        // PI-Linie
        if self.pi_on {
            let d = V2::from_bearing(self.pi_brg, 1.0);
            let off = V2::from_bearing(self.pi_brg + 90.0, self.pi_offset);
            let a = v.w2s(own.pos + off - d * (v.range * 3.0));
            let b = v.w2s(own.pos + off + d * (v.range * 3.0));
            dashed(p, v, a, b, Stroke::new(1.2_f32, C_PI), 10.0, 5.0);
        }

        // EBL / VRM
        for i in 0..2 {
            let (dash, gap) = if i == 0 { (12.0, 6.0) } else { (4.0, 5.0) };
            if self.ebl[i].on {
                let end = own_s + v.dir_s(self.ebl[i].brg) * v.radius * 3.0;
                dashed(p, v, own_s, end, Stroke::new(1.3_f32, C_EBL), dash, gap);
                if let Some((_, b)) = clip_circle(own_s, end, v.center, v.radius) {
                    p.text(
                        b - v.dir_s(self.ebl[i].brg) * 18.0,
                        Align2::CENTER_CENTER,
                        format!("E{}", i + 1),
                        FontId::monospace(11.0),
                        C_EBL,
                    );
                }
            }
            if self.vrm[i].on {
                let r = self.vrm[i].range as f32 * ppn;
                if i == 0 {
                    circle_clipped(p, v, own_s, r, Stroke::new(1.3_f32, C_EBL), true);
                } else {
                    circle_clipped(
                        p,
                        v,
                        own_s,
                        r,
                        Stroke::new(1.0_f32, C_EBL.gamma_multiply(0.8)),
                        true,
                    );
                }
            }
        }

        // Eigenschiff
        if v.range <= 1.5 {
            let l = own.length_m / 1852.0;
            let b = own.beam_m / 1852.0 / 2.0;
            let shape = [
                (l / 2.0, 0.0),
                (l * 0.3, b),
                (-l / 2.0, b),
                (-l / 2.0, -b),
                (l * 0.3, -b),
            ];
            let pts: Vec<Pos2> = shape
                .iter()
                .map(|(al, ac)| {
                    v.w2s(
                        own.pos
                            + V2::from_bearing(own.heading, *al)
                            + V2::from_bearing(own.heading + 90.0, *ac),
                    )
                })
                .collect();
            p.add(Shape::closed_line(pts, Stroke::new(1.5_f32, C_OWN)));
        } else {
            p.circle_stroke(own_s, 5.0, Stroke::new(1.5_f32, C_OWN));
        }
        // eigene vergangene Positionen (wahr)
        if self.arpa.vector_mode == VectorMode::True {
            for (_, pp) in &self.arpa.own_past {
                let s = v.w2s(*pp);
                if v.inside(s) {
                    p.circle_filled(s, 1.8, C_OWN.gamma_multiply(0.7));
                }
            }
        }
        // eigener Vektor (wahr, gemäß Stabilisierung)
        if self.show_own_vector {
            let ov = self.arpa.own_ref_vel(&o) * tmin;
            line(
                p,
                v,
                own_s,
                v.w2s(own.pos + ov),
                Stroke::new(2.0_f32, C_OWN),
            );
        }
        // Soll-Vektor mit Griff (Autopilot)
        let tip = v.w2s(own.pos + V2::from_bearing(own.cmd_heading, own.cmd_speed * tmin));
        if self.show_autopilot {
            dashed(p, v, own_s, tip, Stroke::new(1.2_f32, C_CMD), 6.0, 4.0);
        }
        if self.show_autopilot && v.inside(tip) {
            p.circle_stroke(tip, 5.0, Stroke::new(1.5_f32, C_CMD));
            if self.drag == DragMode::OwnVector || self.drag == DragMode::Course {
                label_bg(
                    p,
                    tip + vec2(10.0, -10.0),
                    Align2::LEFT_BOTTOM,
                    &tf!(
                        "SOLL {}  {:.1} kn",
                        "SET {}  {:.1} kn",
                        fb(own.cmd_heading),
                        own.cmd_speed
                    ),
                    12.0,
                    C_CMD,
                );
            }
        }

        // Trial: Eigenschiff
        let trial = self.arpa.trial;
        if trial.active {
            let od = self.trial_origin(&o);
            let ods = v.w2s(od);
            if trial.delay_min > 0.0 {
                dashed(p, v, own_s, ods, Stroke::new(1.0_f32, C_TRIAL), 3.0, 3.0);
                p.circle_stroke(ods, 4.0, Stroke::new(1.5_f32, C_TRIAL));
            }
            let tt = v.w2s(od + self.arpa.trial_ground_vel(&o) * tmin);
            line(p, v, ods, tt, Stroke::new(2.5_f32, C_TRIAL));
            if v.inside(tt) {
                p.circle_stroke(tt, 6.0, Stroke::new(1.8_f32, C_TRIAL));
                p.text(
                    tt + vec2(9.0, -9.0),
                    Align2::LEFT_BOTTOM,
                    "T",
                    FontId::proportional(16.0),
                    C_TRIAL,
                );
            }
            // Trial-Kennzeichnung: großes „T“ (blinkend) wie am Gerät
            let tpos = rect.left_top() + vec2(14.0, 40.0);
            let trect = Rect::from_min_size(tpos, vec2(40.0, 40.0));
            p.rect_filled(trect, CornerRadius::same(4), Color32::from_black_alpha(170));
            p.rect_stroke(
                trect,
                CornerRadius::same(4),
                Stroke::new(2.0_f32, C_TRIAL),
                egui::StrokeKind::Inside,
            );
            if blink {
                p.text(
                    trect.center(),
                    Align2::CENTER_CENTER,
                    "T",
                    FontId::proportional(30.0),
                    C_TRIAL,
                );
            }
            if self.drag == DragMode::TrialVector {
                label_bg(
                    p,
                    tt + vec2(10.0, -10.0),
                    Align2::LEFT_BOTTOM,
                    &format!("TRIAL {}  {:.1} kn", fb(trial.course), trial.speed),
                    12.0,
                    C_TRIAL,
                );
            }
        }

        // ARPA-Ziele
        for tr in &self.arpa.tracks {
            let info = self.arpa.info(tr, &o, now);
            let pos = info.pos;
            let s = v.w2s(pos);
            if !v.inside(s) {
                continue;
            }
            let col = match tr.state {
                TrackState::Lost => C_LOST,
                _ if tr.dangerous => C_DANGER,
                _ => C_ARPA,
            };
            let stroke = Stroke::new(1.6_f32, col);
            // vergangene Positionen
            for (_, pp, own_then) in &tr.past {
                let q = match self.arpa.vector_mode {
                    VectorMode::True => *pp,
                    VectorMode::Relative => *pp - *own_then + own.pos,
                };
                let qs = v.w2s(q);
                if v.inside(qs) {
                    p.circle_filled(qs, 1.8, col.gamma_multiply(0.8));
                }
            }
            // Vektor
            if info.valid {
                let vel = match self.arpa.vector_mode {
                    VectorMode::True => info.true_vel,
                    VectorMode::Relative => info.rel_vel,
                };
                let thin = trial.active && self.arpa.vector_mode == VectorMode::Relative;
                let st = if thin {
                    Stroke::new(1.0_f32, col.gamma_multiply(0.6))
                } else {
                    Stroke::new(1.8_f32, col)
                };
                line(p, v, s, v.w2s(pos + vel * tmin), st);
                if trial.active {
                    if let Some(ti) = self.arpa.trial_info(tr, &o, now) {
                        let tc = if self.arpa.is_dangerous(ti.cpa, ti.tcpa) {
                            C_DANGER
                        } else {
                            C_TRIAL
                        };
                        match self.arpa.vector_mode {
                            VectorMode::Relative => {
                                let a = v.w2s(o.pos + ti.rel_d);
                                let b = v.w2s(o.pos + ti.rel_d + ti.rel_vel * tmin);
                                if trial.delay_min > 0.0 {
                                    dashed(p, v, s, a, Stroke::new(1.0_f32, tc), 3.0, 3.0);
                                }
                                line(p, v, a, b, Stroke::new(2.2_f32, tc));
                            }
                            VectorMode::True => {
                                let a = v.w2s(ti.pos_d);
                                if v.inside(a) {
                                    p.circle_stroke(a, 3.5, Stroke::new(1.3_f32, tc));
                                }
                            }
                        }
                    }
                }
            }
            // Symbol
            match tr.state {
                TrackState::Acquiring => {
                    for k in 0..8 {
                        let a0 = k as f32 / 8.0 * std::f32::consts::TAU;
                        let a1 = a0 + std::f32::consts::TAU / 16.0;
                        p.line_segment(
                            [
                                s + vec2(a0.sin(), -a0.cos()) * 10.0,
                                s + vec2(a1.sin(), -a1.cos()) * 10.0,
                            ],
                            stroke,
                        );
                    }
                }
                TrackState::Tracking => {
                    if tr.dangerous {
                        let unacked = self
                            .arpa
                            .alarms
                            .iter()
                            .any(|a| a.track == tr.num && !a.acked);
                        if blink || !unacked {
                            let pts = vec![
                                s + vec2(0.0, -12.0),
                                s + vec2(10.5, 7.0),
                                s + vec2(-10.5, 7.0),
                            ];
                            p.add(Shape::closed_line(pts, Stroke::new(2.2_f32, C_DANGER)));
                        }
                    } else {
                        p.circle_stroke(s, 7.0, stroke);
                    }
                }
                TrackState::Lost => {
                    let pts = vec![
                        s + vec2(0.0, -10.0),
                        s + vec2(10.0, 0.0),
                        s + vec2(0.0, 10.0),
                        s + vec2(-10.0, 0.0),
                    ];
                    p.add(Shape::closed_line(pts, stroke));
                    p.line_segment([s + vec2(-7.0, -7.0), s + vec2(7.0, 7.0)], stroke);
                    p.line_segment([s + vec2(-7.0, 7.0), s + vec2(7.0, -7.0)], stroke);
                }
            }
            p.text(
                s + vec2(11.0, -11.0),
                Align2::LEFT_BOTTOM,
                format!("{}", tr.num),
                FontId::monospace(11.0),
                col,
            );
            if self.arpa.selected == Some(tr.num) {
                p.rect_stroke(
                    Rect::from_center_size(s, vec2(28.0, 28.0)),
                    CornerRadius::ZERO,
                    Stroke::new(1.2_f32, Color32::WHITE),
                    egui::StrokeKind::Middle,
                );
                // Relativbahn mit CPA-Punkt
                if self.show_reltrack && info.valid && info.tcpa.is_finite() && info.tcpa > 0.0 {
                    let rel = pos - o.pos;
                    let cpa_pt = o.pos + rel + info.rel_vel * (info.tcpa / 60.0);
                    let ext = o.pos + rel + info.rel_vel * (info.tcpa / 60.0 * 1.4);
                    dashed(
                        p,
                        v,
                        s,
                        v.w2s(ext),
                        Stroke::new(1.0_f32, col.gamma_multiply(0.7)),
                        5.0,
                        5.0,
                    );
                    let cs = v.w2s(cpa_pt);
                    if v.inside(cs) {
                        p.circle_stroke(cs, 4.0, Stroke::new(1.2_f32, col));
                        dashed(
                            p,
                            v,
                            own_s,
                            cs,
                            Stroke::new(0.8_f32, col.gamma_multiply(0.5)),
                            2.0,
                            3.0,
                        );
                        p.text(
                            cs + vec2(7.0, 7.0),
                            Align2::LEFT_TOP,
                            format!("CPA {:.2}", info.cpa),
                            FontId::monospace(11.0),
                            col,
                        );
                    }
                }
            }
        }

        // Ausbilderansicht: wahre Positionen
        if self.show_truth {
            let cur = self.world.current();
            for t in &self.world.targets {
                let s = v.w2s(t.pos);
                if !v.inside(s) {
                    continue;
                }
                let st = Stroke::new(1.0_f32, C_TRUTH);
                p.line_segment([s + vec2(-4.0, -4.0), s + vec2(4.0, 4.0)], st);
                p.line_segment([s + vec2(-4.0, 4.0), s + vec2(4.0, -4.0)], st);
                line(p, v, s, v.w2s(t.pos + t.ground_vel(cur) * tmin), st);
                p.text(
                    s + vec2(6.0, 6.0),
                    Align2::LEFT_TOP,
                    format!("#{}", t.id),
                    FontId::monospace(10.0),
                    C_TRUTH,
                );
            }
        }

        // Cursor
        if let Some(c) = self.cursor {
            let cs = v.w2s(own.pos + c);
            let st = Stroke::new(1.0_f32, Color32::from_white_alpha(160));
            p.line_segment([cs + vec2(-9.0, 0.0), cs + vec2(-3.0, 0.0)], st);
            p.line_segment([cs + vec2(3.0, 0.0), cs + vec2(9.0, 0.0)], st);
            p.line_segment([cs + vec2(0.0, -9.0), cs + vec2(0.0, -3.0)], st);
            p.line_segment([cs + vec2(0.0, 3.0), cs + vec2(0.0, 9.0)], st);
            p.text(
                cs + vec2(12.0, 12.0),
                Align2::LEFT_TOP,
                format!("{} {:.2}nm", fb(c.bearing()), c.len()),
                FontId::monospace(11.0),
                Color32::from_white_alpha(200),
            );
        }

        // Eckanzeigen
        let tl = rect.left_top() + vec2(12.0, 10.0);
        let mot = match self.motion {
            Motion::Relative => "RM",
            Motion::True => "TM",
        };
        let vec = match self.arpa.vector_mode {
            VectorMode::True => "T",
            VectorMode::Relative => "R",
        };
        label_bg(
            p,
            tl,
            Align2::LEFT_TOP,
            &format!("{}  {}({})", self.orientation.label(), mot, vec),
            13.0,
            C_SCALE,
        );
        label_bg(
            p,
            rect.right_top() + vec2(-12.0, 10.0),
            Align2::RIGHT_TOP,
            &format!(
                "{} NM  /  {} NM   {}  {}",
                v.range,
                self.radar.ring_interval(),
                self.radar.pulse().label(),
                if self.radar.band() == Band::X {
                    "X"
                } else {
                    "S"
                }
            ),
            13.0,
            C_SCALE,
        );
        let warp = WARPS[self.warp_idx];
        let tstr = if self.paused {
            tf!("PAUSE   {}", "PAUSED   {}", fmt_hms(now))
        } else {
            format!("x{}   {}", warp, fmt_hms(now))
        };
        label_bg(
            p,
            rect.left_bottom() + vec2(12.0, -10.0),
            Align2::LEFT_BOTTOM,
            &tstr,
            14.0,
            if self.paused { C_LOST } else { C_SCALE },
        );
        label_bg(
            p,
            rect.right_bottom() + vec2(-12.0, -10.0),
            Align2::RIGHT_BOTTOM,
            &format!("HDG {}  SPD {:.1} kn", fb(own.heading), own.stw),
            14.0,
            C_SCALE,
        );
        if let Some((m, until)) = &self.msg {
            if self.real_time < *until {
                label_bg(
                    p,
                    rect.center_top() + vec2(0.0, 12.0),
                    Align2::CENTER_TOP,
                    m,
                    13.0,
                    Color32::WHITE,
                );
            }
        }
        if self.arpa.has_unacked() && blink {
            label_bg(
                p,
                rect.center_bottom() + vec2(0.0, -12.0),
                Align2::CENTER_BOTTOM,
                "⚠ ALARM",
                16.0,
                C_DANGER,
            );
        }
    }

    // -----------------------------------------------------------------------
    // Panels
    // -----------------------------------------------------------------------

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("RADAR / ARPA TRAINER").strong().color(adapt(ui, Color32::from_rgb(140, 200, 255))));
            ui.separator();
            ui.label(t("Szenario:", "Scenario:"));
            let mut sc = self.scenario;
            egui::ComboBox::from_id_salt("scenario")
                .width(300.0)
                .selected_text(sc.name())
                .show_ui(ui, |ui| {
                    for s in Scenario::ALL {
                        ui.selectable_value(&mut sc, s, s.name()).on_hover_text(s.description());
                    }
                });
            self.scenario = sc;
            if ui.button(t("⟲ Laden / Neustart", "⟲ Load / restart")).clicked() {
                self.load_scenario(self.scenario);
            }
            ui.separator();
            if ui.button(if self.paused { t("▶ Weiter", "▶ Resume") } else { "⏸ Pause" }).clicked() {
                self.paused = !self.paused;
            }
            ui.label(t("Zeitraffer:", "Time warp:"));
            for (i, w) in WARPS.iter().enumerate() {
                if ui.selectable_label(self.warp_idx == i, format!("{}×", w)).clicked() {
                    self.warp_idx = i;
                }
            }
            ui.separator();
            ui.label(RichText::new(tf!("Simzeit {}", "Sim time {}", fmt_hms(self.world.time))).monospace());
            ui.separator();
            if self.arpa.has_unacked() {
                let blink = (self.real_time * 2.0) as i64 % 2 == 0;
                let c = if blink { C_DANGER } else { Color32::from_rgb(120, 30, 30) };
                ui.label(RichText::new("⚠ ALARM").strong().color(adapt(ui, c)));
                if ui.button(t("Quittieren", "Acknowledge")).clicked() {
                    self.arpa.ack_all();
                }
                ui.separator();
            }
            ui.checkbox(&mut self.show_truth, t("Ausbilder: wahre Ziele", "Instructor: true targets"))
                .on_hover_text(t("Zeigt die tatsächlichen Positionen/Vektoren aller simulierten Ziele (magenta) zum Vergleich mit ARPA.", "Shows the actual positions/vectors of all simulated targets (magenta) for comparison with ARPA."));
            self.mode_lang_bar(ui);
        });
        // zweite Reihe: Oberfläche, Foto, Hilfe, Szenariobeschreibung
        ui.horizontal(|ui| {
            self.theme_photo_bar(ui);
            if ui.button(t("❓ Hilfe", "❓ Help")).clicked() {
                self.show_help = !self.show_help;
            }
            ui.separator();
            if ui
                .button(t("💾 Als Standard", "💾 Save as standard"))
                .on_hover_text(t(
                    "Aktuelle Radar-, ARPA-, Vektor-, Trail-, Darstellungs- und Eigenschiff-Einstellungen als Standard speichern. Das Szenario ist nicht enthalten – der Start erfolgt immer mit „leer“.",
                    "Save the current radar, ARPA, vector, trail, display and own-ship settings as standard. The scenario is not included – start-up is always “empty”.",
                ))
                .clicked()
            {
                match crate::templates::save(SIM_TEMPLATE, &self.sim_template()) {
                    Ok(_) => self.say(t("Standard-Template gespeichert – gilt ab dem nächsten Start", "Standard template saved – applies from the next start")),
                    Err(e) => self.say(tf!("Speichern fehlgeschlagen: {}", "Saving failed: {}", e)),
                }
            }
            if ui
                .button(t("↺ Standard zurücksetzen", "↺ Reset standard"))
                .on_hover_text(t(
                    "Standard-Template auf Werkseinstellung zurücksetzen (X-Band, 12 NM …). Die aktuellen Einstellungen bleiben bis zum nächsten Start erhalten.",
                    "Reset the standard template to factory settings (X-band, 12 NM …). Current settings stay until the next start.",
                ))
                .clicked()
            {
                match crate::templates::remove(SIM_TEMPLATE) {
                    Ok(()) => self.say(t("Standard auf Werkseinstellung zurückgesetzt", "Standard reset to factory settings")),
                    Err(e) => self.say(tf!("Zurücksetzen fehlgeschlagen: {}", "Reset failed: {}", e)),
                }
            }
            ui.separator();
            ui.add(egui::Label::new(RichText::new(self.scenario.description()).italics().small()).truncate());
        });
    }

    /// Moduswechsel, Sprache, Oberfläche, Foto – in beiden Modi oben rechts.
    fn common_bar(&mut self, ui: &mut egui::Ui) {
        self.mode_lang_bar(ui);
        self.theme_photo_bar(ui);
    }

    /// Moduswechsel und Sprache.
    fn mode_lang_bar(&mut self, ui: &mut egui::Ui) {
        if ui
            .button(t("⟲ Modus", "⟲ Mode"))
            .on_hover_text(t(
                "Zurück zur Auswahl Simulation / Plottumgebung",
                "Back to the choice simulation / plotting",
            ))
            .clicked()
        {
            self.mode = AppMode::Start;
        }
        self.lang_combo(ui);
    }

    /// Oberfläche (Dunkel/Grau/Hell) und Bildschirmfoto.
    fn theme_photo_bar(&mut self, ui: &mut egui::Ui) {
        ui.label(t("Oberfläche:", "Interface:"));
        let before = self.ui_theme;
        egui::ComboBox::from_id_salt("ui_theme")
            .width(80.0)
            .selected_text(self.ui_theme.label())
            .show_ui(ui, |ui| {
                for th in [UiTheme::Dark, UiTheme::Gray, UiTheme::Light] {
                    ui.selectable_value(&mut self.ui_theme, th, th.label());
                }
            });
        if self.ui_theme != before {
            self.ui_theme.apply(ui.ctx());
        }
        if ui
            .button(t("📷 Foto", "📷 Photo"))
            .on_hover_text(t(
                "Bildschirmfoto in den Benutzerordner speichern (F12)",
                "Save screenshot to the user folder (F12)",
            ))
            .clicked()
        {
            let ctx = ui.ctx().clone();
            self.user_snapshot(&ctx);
        }
    }

    fn lang_combo(&mut self, ui: &mut egui::Ui) {
        use crate::i18n::{lang, set_lang, Lang};
        let mut l = lang();
        egui::ComboBox::from_id_salt("lang")
            .width(80.0)
            .selected_text(l.label())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut l, Lang::De, Lang::De.label());
                ui.selectable_value(&mut l, Lang::En, Lang::En.label());
            });
        if l != lang() {
            set_lang(l);
        }
    }

    fn start_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(self.ui_theme.surround()))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space((ui.available_height() * 0.16).max(20.0));
                    ui.label(RichText::new("RADAR / ARPA TRAINER").size(34.0).strong().color(adapt(ui, Color32::from_rgb(140, 200, 255))));
                    ui.label(RichText::new(t("Bitte Arbeitsumgebung wählen", "Please choose a working environment")).size(17.0));
                    ui.add_space(30.0);
                });
                let w = 340.0;
                let total = w * 2.0 + 40.0;
                ui.horizontal(|ui| {
                    ui.add_space(((ui.available_width() - total) / 2.0).max(0.0));
                    let card = |ui: &mut egui::Ui, title: &str, text: &str| -> bool {
                        let b = egui::Button::new(
                            RichText::new(format!("{}\n\n{}", title, text)).size(15.0),
                        )
                        .min_size(vec2(w, 190.0))
                        .wrap();
                        ui.add_sized([w, 190.0], b).clicked()
                    };
                    if card(
                        ui,
                        t("📡  SIMULATION", "📡  SIMULATION"),
                        t(
                            "Radarbedienung mit simuliertem Video, ARPA, Trial-Manöver, Szenarien und Zeitraffer.",
                            "Radar operation with simulated video, ARPA, trial manoeuvre, scenarios and time warp.",
                        ),
                    ) {
                        self.mode = AppMode::Sim;
                    }
                    ui.add_space(40.0);
                    if card(
                        ui,
                        t("📝  PLOTTUMGEBUNG", "📝  PLOTTING"),
                        t(
                            "Klassisches Radarplotten: Relativbewegung, Vektordreieck, Manöverplanung, KVR-Begründung und Cockcroft-Diagramm.",
                            "Classic radar plotting: relative motion, vector triangle, manoeuvre planning, COLREG reasoning and Cockcroft diagram.",
                        ),
                    ) {
                        self.mode = AppMode::Plot;
                    }
                });
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(((ui.available_width() - 420.0) / 2.0).max(0.0));
                        ui.label(t("Sprache:", "Language:"));
                        self.lang_combo(ui);
                        ui.add_space(20.0);
                        ui.label(t("Oberfläche:", "Interface:"));
                        let before = self.ui_theme;
                        egui::ComboBox::from_id_salt("ui_theme_start")
                            .width(80.0)
                            .selected_text(self.ui_theme.label())
                            .show_ui(ui, |ui| {
                                for th in [UiTheme::Dark, UiTheme::Gray, UiTheme::Light] {
                                    ui.selectable_value(&mut self.ui_theme, th, th.label());
                                }
                            });
                        if self.ui_theme != before {
                            self.ui_theme.apply(ui.ctx());
                        }
                    });
                });
            });
    }

    /// Testlauf ohne Simulationszeit: nach einigen Frames Bildschirmfoto.
    fn demo_frame_snapshot(&mut self, ctx: &egui::Context) {
        if let Some(d) = self.demo.as_mut() {
            self.demo_frames += 1;
            if self.demo_frames == 30 && !d.snapped {
                d.snapped = true;
                let path = d.path.clone();
                self.request_snapshot(ctx, path);
            }
            ctx.request_repaint();
        }
    }

    fn plot_mode(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("plot_top").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(t("RADARPLOT – Plottumgebung", "RADARPLOT – plotting"))
                        .strong()
                        .color(adapt(ui, Color32::from_rgb(140, 200, 255))),
                );
                ui.separator();
                self.common_bar(ui);
            });
        });
        egui::SidePanel::left("plot_left")
            .resizable(false)
            .exact_width(310.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.plot.left_panel(ui));
            });
        egui::SidePanel::right("plot_right")
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.plot.right_panel(ui));
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(self.ui_theme.surround()))
            .show(ctx, |ui| self.plot.sheet(ui));
        self.plot.reason_window(ctx);
    }

    fn info_panel(&mut self, ui: &mut egui::Ui) {
        let o = self.own_ref();
        let now = self.world.time;
        {
            let own = &self.world.own;
            section(ui, t("EIGENSCHIFF", "OWN SHIP"));
            mono(
                ui,
                tf!(
                    "HDG {}  SOLL {}",
                    "HDG {}  SET {}",
                    fb(own.heading),
                    fb(own.cmd_heading)
                ),
            );
            mono(
                ui,
                tf!(
                    "STW {:5.1} kn  SOLL {:4.1}",
                    "STW {:5.1} kn  SET {:4.1}",
                    own.stw,
                    own.cmd_speed
                ),
            );
            mono(
                ui,
                format!("COG {}  SOG {:5.1}", fb(o.gvel.bearing()), o.gvel.len()),
            );
            mono(ui, format!("ROT {:+6.1} °/min", own.rot * 60.0));
            if self.world.current_drift > 0.0 {
                mono(
                    ui,
                    tf!(
                        "STROM {} / {:.1} kn",
                        "CURRENT {} / {:.1} kn",
                        fb(self.world.current_set),
                        self.world.current_drift
                    ),
                );
            }
        }

        section(ui, "RADAR");
        let mot = match self.motion {
            Motion::Relative => "RM",
            Motion::True => "TM",
        };
        mono(
            ui,
            format!(
                "{}  {}  {}",
                self.orientation.label(),
                mot,
                if self.radar.transmit { "TX" } else { "STBY" }
            ),
        );
        mono(
            ui,
            tf!(
                "Bereich {} NM  Ringe {}",
                "Range {} NM  Rings {}",
                self.radar.range(),
                self.radar.ring_interval()
            ),
        );
        mono(
            ui,
            tf!(
                "{}  Puls {}  IR {}",
                "{}  Pulse {}  IR {}",
                self.radar.band().label(),
                self.radar.pulse().label(),
                if self.radar.ir {
                    t("an", "on")
                } else {
                    t("aus", "off")
                }
            ),
        );
        mono(
            ui,
            format!(
                "GAIN {:3.0} SEA {:3.0} RAIN {:3.0}",
                self.radar.gain, self.radar.sea, self.radar.rain
            ),
        );
        let vm = match self.arpa.vector_mode {
            VectorMode::True => t("WAHR", "TRUE"),
            VectorMode::Relative => "REL",
        };
        mono(
            ui,
            tf!(
                "Vektor {} {} min",
                "Vector {} {} min",
                vm,
                fmt_ms(self.arpa.vector_min)
            ),
        );
        mono(
            ui,
            format!(
                "Stab. {}",
                if self.arpa.stab_ground {
                    t("GRUND (GPS)", "GROUND (GPS)")
                } else {
                    t("SEE (Log)", "SEA (log)")
                }
            ),
        );
        let tm = match self.trail_mode {
            TrailMode::Off => t("aus", "off").to_string(),
            TrailMode::True => tf!("WAHR {} min", "TRUE {} min", fmt_ms(self.trail_min)),
            TrailMode::Relative => format!("REL {} min", fmt_ms(self.trail_min)),
        };
        mono(ui, format!("Trails {}", tm));

        section(ui, "EBL / VRM");
        for i in 0..2 {
            let e = if self.ebl[i].on {
                if self.ebl_relative {
                    format!("{} R", fb(self.ebl[i].brg - self.world.own.heading))
                } else {
                    format!("{} T", fb(self.ebl[i].brg))
                }
            } else {
                "  ----  ".into()
            };
            let r = if self.vrm[i].on {
                format!("{:6.2} nm", self.vrm[i].range)
            } else {
                "  ----".into()
            };
            mono_c(ui, format!("EBL{} {}  VRM{} {}", i + 1, e, i + 1, r), C_EBL);
        }

        section(ui, "CURSOR");
        if let Some(c) = self.cursor {
            mono(
                ui,
                format!("BRG {}  RNG {:.2} nm", fb(c.bearing()), c.len()),
            );
            mono(
                ui,
                format!("rel. {}", fb(c.bearing() - self.world.own.heading)),
            );
        } else {
            mono_c(ui, "–", Color32::GRAY);
        }

        section(ui, t("ZIELDATEN", "TARGET DATA"));
        let mut del = None;
        if let Some(tr) = self.arpa.selected.and_then(|n| self.arpa.track(n)) {
            let info = self.arpa.info(tr, &o, now);
            let col = if tr.dangerous {
                C_DANGER
            } else if tr.state == TrackState::Lost {
                C_LOST
            } else {
                C_ARPA
            };
            let status = match tr.state {
                TrackState::Acquiring => tf!("ERFASSUNG {:2.0}s", "ACQUIRING {:2.0}s", tr.age(now)),
                TrackState::Tracking if tr.age(now) < FULL_TIME => "TREND".into(),
                TrackState::Tracking => t("STABIL", "STABLE").into(),
                TrackState::Lost => t("VERLOREN", "LOST").into(),
            };
            let man = if tr.maneuver_recent(now) {
                t("  MANÖVER", "  MANOEUVRE")
            } else {
                ""
            };
            mono_c(
                ui,
                tf!("ZIEL {:2}  {}{}", "TARGET {:2}  {}{}", tr.num, status, man),
                col,
            );
            mono_c(
                ui,
                format!("BRG {}  RNG {:5.2}", fb(info.brg), info.rng),
                col,
            );
            if info.valid {
                mono_c(
                    ui,
                    tf!(
                        "KüG {}  FüG {:5.1}",
                        "CRS {}  SPD {:5.1}",
                        fb(info.cse),
                        info.spd
                    ),
                    col,
                );
                mono_c(
                    ui,
                    format!("CPA {:5.2} nm  TCPA {}", info.cpa, ftcpa(info.tcpa)),
                    col,
                );
                match (info.bcr, info.bct) {
                    (Some(r), Some(t)) => {
                        mono_c(ui, format!("BCR {:+5.2} nm  BCT {}", r, ftcpa(t)), col)
                    }
                    _ => mono_c(ui, "BCR  ---      BCT  ---", col),
                }
                if self.arpa.trial.active {
                    if let Some(ti) = self.arpa.trial_info(tr, &o, now) {
                        let c = if self.arpa.is_dangerous(ti.cpa, ti.tcpa) {
                            C_DANGER
                        } else {
                            C_TRIAL
                        };
                        mono_c(
                            ui,
                            format!("TRIAL CPA {:5.2} TCPA {}", ti.cpa, ftcpa(ti.tcpa)),
                            c,
                        );
                    }
                }
            } else {
                mono_c(
                    ui,
                    t(
                        "Vektor nach ca. 1 min verfügbar",
                        "Vector available after approx. 1 min",
                    ),
                    Color32::GRAY,
                );
            }
            mono_c(
                ui,
                tf!(
                    "Plots {}  Alter {}",
                    "Plots {}  age {}",
                    tr.plot_count(),
                    fmt_hms(tr.age(now))
                ),
                Color32::GRAY,
            );
            if ui.button(t("Track löschen", "Cancel track")).clicked() {
                del = Some(tr.num);
            }
        } else {
            mono_c(
                ui,
                t("Kein Ziel gewählt –", "No target selected –"),
                Color32::GRAY,
            );
            mono_c(
                ui,
                t("Echo anklicken zum Erfassen", "click an echo to acquire"),
                Color32::GRAY,
            );
        }
        if let Some(n) = del {
            self.arpa.remove_track(n);
        }

        section(ui, t("ALARME", "ALARMS"));
        let mut ack = None;
        if self.arpa.alarms.is_empty() {
            mono_c(ui, t("keine", "none"), Color32::GRAY);
        }
        for (i, a) in self.arpa.alarms.iter().enumerate() {
            ui.horizontal(|ui| {
                let c = if a.acked { Color32::GRAY } else { C_DANGER };
                mono_c(ui, format!("{} {}", fmt_hms(a.time), a.text()), c);
                if !a.acked || a.kind == AlarmKind::Lost {
                    if ui.small_button(t("Quitt.", "Ack.")).clicked() {
                        ack = Some(i);
                    }
                }
            });
        }
        if let Some(i) = ack {
            self.arpa.ack(i);
        }
    }

    fn controls_panel(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(RichText::new("📡 Radar").strong()).default_open(true).show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.radar.transmit, t("TX  Senden", "TX  Transmit")).clicked() {
                    self.radar.set_transmit(true);
                }
                if ui.selectable_label(!self.radar.transmit, "STANDBY").clicked() {
                    self.radar.set_transmit(false);
                    self.arpa.clear_tracks();
                }
            });
            ui.horizontal(|ui| {
                ui.label(t("Frequenzband:", "Frequency band:"));
                let mut b = self.radar.band();
                ui.radio_value(&mut b, Band::X, "X-Band (3 cm)")
                    .on_hover_text(t("9,4 GHz: schmale Keule (1,2°), hohe Auflösung, gut für kleine Ziele und SART – aber starker See-/Regenclutter und Dämpfung im Regen.", "9.4 GHz: narrow beam (1.2°), high resolution, good for small targets and SART – but strong sea/rain clutter and attenuation in rain."));
                ui.radio_value(&mut b, Band::S, "S-Band (10 cm)")
                    .on_hover_text(t("3 GHz: breitere Keule (1,9°), wenig See-/Regenclutter, kaum Regendämpfung, große Reichweite bei schlechtem Wetter – kleine Ziele schwächer.", "3 GHz: wider beam (1.9°), little sea/rain clutter, hardly any rain attenuation, long range in bad weather – small targets weaker."));
                if b != self.radar.band() {
                    self.radar.set_band(b);
                    self.say(tf!("{} gewählt – Bild wird neu aufgebaut", "{} selected – picture is rebuilt", b.label()));
                }
            });
            ui.horizontal(|ui| {
                ui.label(t("Bereich:", "Range:"));
                if ui.button(" − ").clicked() && self.radar.range_idx > 0 {
                    self.set_range(self.radar.range_idx - 1);
                }
                ui.label(RichText::new(format!("{:>5} NM", self.radar.range())).monospace().strong());
                if ui.button(" + ").clicked() && self.radar.range_idx + 1 < RANGES.len() {
                    self.set_range(self.radar.range_idx + 1);
                }
                ui.checkbox(&mut self.rings, t("Ringe", "Rings"));
            });
            ui.horizontal(|ui| {
                ui.label(t("Puls:", "Pulse:"));
                let mut ps = self.radar.pulse_sel;
                egui::ComboBox::from_id_salt("pulse")
                    .selected_text(match ps {
                        None => format!("Auto ({})", self.radar.pulse().label()),
                        Some(p) => p.label().to_string(),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut ps, None, "Auto");
                        ui.selectable_value(&mut ps, Some(Pulse::Short), t("SP kurz", "SP short"));
                        ui.selectable_value(&mut ps, Some(Pulse::Medium), t("MP mittel", "MP medium"));
                        ui.selectable_value(&mut ps, Some(Pulse::Long), t("LP lang", "LP long"));
                    });
                if ps != self.radar.pulse_sel {
                    self.radar.pulse_sel = ps;
                }
                ui.checkbox(&mut self.radar.ir, "IR").on_hover_text(t("Interferenzunterdrückung (Fremdradar)", "Interference rejection (other radars)"));
                ui.checkbox(&mut self.sweep_line, t("Strahl", "Sweep"));
            });
            ui.add(egui::Slider::new(&mut self.radar.gain, 0.0..=100.0).text("GAIN"))
                .on_hover_text(t("Empfängerverstärkung. Optimal: leichtes Rauschen (Speckle) sichtbar.", "Receiver gain. Optimal: light noise speckle visible."));
            ui.add(egui::Slider::new(&mut self.radar.sea, 0.0..=100.0).text("SEA (STC)"))
                .on_hover_text(t("Seegangsenttrübung: reduziert die Verstärkung im Nahbereich. Zu viel: kleine Ziele verschwinden!", "Sea clutter control: reduces gain at short range. Too much: small targets disappear!"));
            ui.add(egui::Slider::new(&mut self.radar.rain, 0.0..=100.0).text("RAIN (FTC)"))
                .on_hover_text(t("Regenenttrübung (Differenzierer): flächige Echos werden auf Vorderkanten reduziert.", "Rain clutter control (differentiator): extended echoes are reduced to their leading edges."));
            ui.add(egui::Slider::new(&mut self.persistence, 0.3..=20.0).logarithmic(true).text(t("Nachleuchten", "Persistence")));
            ui.add(egui::Slider::new(&mut self.brilliance, 0.3..=1.5).text(t("Helligkeit Echos", "Echo brightness")));
            ui.add(egui::Slider::new(&mut self.bg_level, 0.3..=2.0).text(t("Helligkeit Hintergrund", "Background brightness")));
            ui.horizontal(|ui| {
                ui.label(t("Farbe:", "Colour:"));
                egui::ComboBox::from_id_salt("scheme").selected_text(self.scheme.label()).show_ui(ui, |ui| {
                    for s in [ColorScheme::Blue, ColorScheme::Yellow, ColorScheme::Green, ColorScheme::Orange] {
                        ui.selectable_value(&mut self.scheme, s, s.label());
                    }
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(t("Darstellung:", "Orientation:"));
                let before = self.orientation;
                ui.radio_value(&mut self.orientation, Orientation::NorthUp, "N-Up");
                ui.radio_value(&mut self.orientation, Orientation::HeadUp, "H-Up");
                ui.radio_value(&mut self.orientation, Orientation::CourseUp, "C-Up");
                if self.orientation != before {
                    if self.orientation == Orientation::CourseUp {
                        self.cu_ref = self.world.own.heading;
                    }
                    if self.orientation == Orientation::HeadUp && self.motion == Motion::True {
                        self.motion = Motion::Relative;
                        self.say(t("Head-Up nur in Relativ-Darstellung (RM) – auf RM umgeschaltet", "Head-up only in relative motion (RM) – switched to RM"));
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label(t("Bewegung:", "Motion:"));
                let before = self.motion;
                ui.radio_value(&mut self.motion, Motion::Relative, t("Relativ (RM)", "Relative (RM)"));
                ui.radio_value(&mut self.motion, Motion::True, t("Wahr (TM)", "True (TM)"));
                if self.motion != before && self.motion == Motion::True {
                    if self.orientation == Orientation::HeadUp {
                        self.orientation = Orientation::NorthUp;
                        self.say(t("True Motion nicht in Head-Up – auf North-Up umgeschaltet", "True motion not in head-up – switched to north-up"));
                    }
                    self.tm_reset();
                }
            });
            ui.horizontal(|ui| {
                if ui.button("CU-Reset").clicked() {
                    self.cu_ref = self.world.own.heading;
                }
                if ui.button("TM-Reset").clicked() {
                    self.tm_reset();
                }
                if ui.button(t("Zentrieren", "Centre")).clicked() {
                    self.offc = V2::ZERO;
                    if self.motion == Motion::True {
                        self.tm_origin = self.world.own.pos;
                    }
                }
            });
        });

        egui::CollapsingHeader::new(
            RichText::new(t("➡ Vektoren & Trails", "➡ Vectors & trails")).strong(),
        )
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(t("Vektor:", "Vector:"));
                ui.radio_value(
                    &mut self.arpa.vector_mode,
                    VectorMode::True,
                    t("wahr (T)", "true (T)"),
                );
                ui.radio_value(
                    &mut self.arpa.vector_mode,
                    VectorMode::Relative,
                    t("relativ (R)", "relative (R)"),
                );
            });
            ui.label(t(
                "Vektorlänge (30-s-Schritte):",
                "Vector length (30 s steps):",
            ));
            half_minute_stepper(ui, &mut self.arpa.vector_min, 60.0);
            ui.horizontal(|ui| {
                for l in VEC_PRESETS {
                    if ui
                        .selectable_label(self.arpa.vector_min == l, format!("{} min", l))
                        .clicked()
                    {
                        self.arpa.vector_min = l;
                    }
                }
            });
            ui.checkbox(
                &mut self.show_own_vector,
                t("Eigener Vektor (weiß)", "Own vector (white)"),
            );
            ui.horizontal(|ui| {
                ui.label(t("Verg. Positionen:", "Past positions:"));
                for (v, n) in PP_INTERVALS {
                    if ui
                        .selectable_label(
                            self.arpa.pp_interval == v,
                            if n == "aus" { t("aus", "off") } else { n },
                        )
                        .clicked()
                    {
                        self.arpa.pp_interval = v;
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Trails:");
                ui.radio_value(&mut self.trail_mode, TrailMode::Off, t("aus", "off"));
                ui.radio_value(&mut self.trail_mode, TrailMode::True, t("wahr", "true"));
                ui.radio_value(
                    &mut self.trail_mode,
                    TrailMode::Relative,
                    t("relativ", "relative"),
                );
                if ui.small_button(t("löschen", "clear")).clicked() {
                    self.radar.clear_trails();
                }
            });
            ui.label(t(
                "Trail-Länge (30-s-Schritte):",
                "Trail length (30 s steps):",
            ));
            half_minute_stepper(ui, &mut self.trail_min, 30.0);
            ui.horizontal(|ui| {
                for l in TRAIL_PRESETS {
                    if ui
                        .selectable_label(self.trail_min == l, format!("{} min", l))
                        .clicked()
                    {
                        self.trail_min = l;
                    }
                }
            });
        });

        egui::CollapsingHeader::new(RichText::new("📐 EBL / VRM / PI").strong())
            .default_open(false)
            .show(ui, |ui| {
                let hdg = self.world.own.heading;
                for i in 0..2 {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.ebl[i].on, format!("EBL{}", i + 1));
                        if self.ebl_relative {
                            let mut r = norm360(self.ebl[i].brg - hdg);
                            if angle_edit(ui, &mut r, 0.5, 1, "° R").changed() {
                                self.ebl[i].brg = norm360(r + hdg);
                            }
                        } else {
                            let mut b = self.ebl[i].brg;
                            if angle_edit(ui, &mut b, 0.5, 1, "° T").changed() {
                                self.ebl[i].brg = norm360(b);
                            }
                        }
                        ui.checkbox(&mut self.vrm[i].on, format!("VRM{}", i + 1));
                        ui.add(
                            egui::DragValue::new(&mut self.vrm[i].range)
                                .range(0.0..=48.0)
                                .speed(0.01)
                                .fixed_decimals(2)
                                .suffix(" nm"),
                        );
                    });
                }
                ui.checkbox(
                    &mut self.ebl_relative,
                    t(
                        "EBL relativ zur Vorausrichtung anzeigen",
                        "Show EBL relative to heading",
                    ),
                );
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.pi_on, t("PI-Linie", "PI line"));
                    angle_edit(ui, &mut self.pi_brg, 0.5, 1, "°");
                    ui.add(
                        egui::DragValue::new(&mut self.pi_offset)
                            .range(-24.0..=24.0)
                            .speed(0.01)
                            .fixed_decimals(2)
                            .suffix(" nm"),
                    )
                    .on_hover_text(t(
                        "Querabstand: + = Steuerbord der Linienrichtung, − = Backbord",
                        "Offset: + = starboard of the line direction, − = port",
                    ));
                });
                ui.label(
                    RichText::new(t(
                        "Tipp: EBL/VRM direkt auf dem Schirm ziehen oder per Rechtsklick setzen.",
                        "Tip: drag EBL/VRM directly on the screen or set them via right-click.",
                    ))
                    .small()
                    .italics(),
                );
            });

        egui::CollapsingHeader::new(RichText::new(t("🚢 Eigenschiff", "🚢 Own ship")).strong()).default_open(true).show(ui, |ui| {
            let own = &mut self.world.own;
            ui.horizontal(|ui| {
                ui.label(t("Sollkurs:", "Set course:"));
                for d in [-10.0, -1.0] {
                    if ui.small_button(format!("{:+}", d)).clicked() {
                        own.cmd_heading = norm360(own.cmd_heading + d);
                    }
                }
                let mut c = own.cmd_heading;
                if angle_edit(ui, &mut c, 0.5, 1, "°").changed() {
                    own.cmd_heading = norm360(c);
                }
                for d in [1.0, 10.0] {
                    if ui.small_button(format!("{:+}", d)).clicked() {
                        own.cmd_heading = norm360(own.cmd_heading + d);
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label(t("Sollfahrt:", "Set speed:"));
                if ui.small_button("−1").clicked() {
                    own.cmd_speed = (own.cmd_speed - 1.0).max(0.0);
                }
                ui.add(egui::DragValue::new(&mut own.cmd_speed).range(0.0..=own.max_speed).speed(0.1).fixed_decimals(1).suffix(" kn"));
                if ui.small_button("+1").clicked() {
                    own.cmd_speed = (own.cmd_speed + 1.0).min(own.max_speed);
                }
                if ui.small_button("STOP").clicked() {
                    own.cmd_speed = 0.0;
                }
            });
            ui.add(egui::Slider::new(&mut own.max_rot, 5.0..=90.0).text(t("max. Drehrate °/min", "max. rate of turn °/min")));
            ui.checkbox(&mut self.show_autopilot, t("Autopilot-Bedienung auf dem Schirm (Taste A)", "Autopilot control on screen (key A)"))
                .on_hover_text(t("Grüner Sollvektor mit Griff, Sollkursmarke und Kurswahl über die Peilskala ein-/ausblenden.", "Show/hide the green set-course vector with handle, set-course marker and course selection via the bearing scale."));
            ui.add(egui::Slider::new(&mut own.accel, 0.01..=0.3).logarithmic(true).text(t("Beschleunigung kn/s", "Acceleration kn/s")));
            ui.label(
                RichText::new(
                    t("Direkt auf dem Schirm: Peilskala anklicken/ziehen = Sollkurs; grünen Vektor-Endpunkt ziehen = Kurs + Fahrt. Pfeiltasten links/rechts = Kurs, hoch/runter = Fahrt.", "On screen: click/drag the bearing scale = set course; drag the green vector tip = course + speed. Arrow keys left/right = course, up/down = speed."),
                )
                .small()
                .italics(),
            );
        });

        egui::CollapsingHeader::new(RichText::new("🎯 ARPA").strong())
            .default_open(true)
            .show(ui, |ui| {
                ui.checkbox(&mut self.arpa.enabled, t("ARPA ein", "ARPA on"));
                ui.horizontal(|ui| {
                    ui.label(t("Stabilisierung:", "Stabilisation:"));
                    ui.radio_value(
                        &mut self.arpa.stab_ground,
                        true,
                        t("Grund (GPS)", "Ground (GPS)"),
                    );
                    ui.radio_value(
                        &mut self.arpa.stab_ground,
                        false,
                        t("See (Log)", "Sea (log)"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(t("CPA-Grenze", "CPA limit"));
                    ui.add(
                        egui::DragValue::new(&mut self.arpa.cpa_limit)
                            .range(0.1..=5.0)
                            .speed(0.05)
                            .fixed_decimals(1)
                            .suffix(" nm"),
                    );
                    ui.label(t("TCPA-Grenze", "TCPA limit"));
                    ui.add(
                        egui::DragValue::new(&mut self.arpa.tcpa_limit)
                            .range(1.0..=60.0)
                            .speed(0.5)
                            .fixed_decimals(0)
                            .suffix(" min"),
                    );
                });
                ui.checkbox(
                    &mut self.show_reltrack,
                    t(
                        "Relativbahn + CPA-Punkt des gewählten Ziels",
                        "Relative track + CPA point of selected target",
                    ),
                );
                ui.horizontal(|ui| {
                    if ui
                        .button(t("Gewählten Track löschen", "Cancel selected track"))
                        .clicked()
                    {
                        if let Some(n) = self.arpa.selected {
                            self.arpa.remove_track(n);
                        }
                    }
                    if ui
                        .button(t("Alle Tracks löschen", "Cancel all tracks"))
                        .clicked()
                    {
                        self.arpa.clear_tracks();
                    }
                });

                ui.add_space(4.0);
                let o = self.own_ref();
                let now = self.world.time;
                let mut select = None;
                egui::Grid::new("tracks")
                    .striped(true)
                    .min_col_width(10.0)
                    .show(ui, |ui| {
                        for h in [
                            "Nr",
                            "St",
                            t("Peil", "Brg"),
                            "Dist",
                            t("KüG", "Crs"),
                            t("FüG", "Spd"),
                            "CPA",
                            "TCPA",
                        ] {
                            ui.label(RichText::new(h).small().strong());
                        }
                        ui.end_row();
                        for tr in &self.arpa.tracks {
                            let info = self.arpa.info(tr, &o, now);
                            let col = if tr.dangerous {
                                C_DANGER
                            } else if tr.state == TrackState::Lost {
                                C_LOST
                            } else {
                                C_ARPA
                            };
                            let sel = self.arpa.selected == Some(tr.num);
                            if ui
                                .selectable_label(
                                    sel,
                                    RichText::new(format!("{:2}", tr.num))
                                        .monospace()
                                        .color(adapt(ui, col)),
                                )
                                .clicked()
                            {
                                select = Some(tr.num);
                            }
                            let st = match tr.state {
                                TrackState::Acquiring if tr.auto => "ACQ*",
                                TrackState::Acquiring => "ACQ",
                                TrackState::Tracking => {
                                    if tr.maneuver_recent(now) {
                                        "MAN"
                                    } else {
                                        "TRK"
                                    }
                                }
                                TrackState::Lost => "LOST",
                            };
                            let m = |ui: &mut egui::Ui, s: String| {
                                ui.label(RichText::new(s).monospace().small().color(adapt(ui, col)))
                            };
                            m(ui, st.into());
                            m(ui, format!("{:05.1}", info.brg));
                            m(ui, format!("{:5.2}", info.rng));
                            if info.valid {
                                m(ui, format!("{:05.1}", info.cse));
                                m(ui, format!("{:4.1}", info.spd));
                                m(ui, format!("{:4.2}", info.cpa));
                                m(
                                    ui,
                                    if info.tcpa.is_finite() {
                                        format!("{:5.1}", info.tcpa)
                                    } else {
                                        "---".into()
                                    },
                                );
                            } else {
                                for _ in 0..4 {
                                    m(ui, "--".into());
                                }
                            }
                            ui.end_row();
                        }
                    });
                if let Some(n) = select {
                    self.arpa.selected = Some(n);
                }
                if self.arpa.tracks.is_empty() {
                    ui.label(
                        RichText::new(t(
                            "Keine Tracks. Echo auf dem Schirm anklicken.",
                            "No tracks. Click an echo on the screen.",
                        ))
                        .small()
                        .italics(),
                    );
                }
            });

        egui::CollapsingHeader::new(RichText::new("🛡 Guard Zone").strong()).default_open(false).show(ui, |ui| {
            ui.label(RichText::new(t("Neue Echos in einer aktiven Zone werden automatisch erfasst und alarmiert.", "New echoes in an active zone are acquired automatically and raise an alarm.")).small().italics());
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.arpa.gz_on, "Ring");
                ui.add(egui::DragValue::new(&mut self.arpa.gz_inner).range(0.1..=24.0).speed(0.05).fixed_decimals(1).suffix(" nm"));
                ui.label(t("bis", "to"));
                ui.add(egui::DragValue::new(&mut self.arpa.gz_outer).range(0.2..=24.0).speed(0.05).fixed_decimals(1).suffix(" nm"));
            });
            ui.horizontal(|ui| {
                let n = self.arpa.gzp_points.len();
                ui.add_enabled(n >= 3, egui::Checkbox::new(&mut self.arpa.gzp_on, tf!("Polygon ({} Punkte)", "Polygon ({} points)", n)));
                if self.gzp_draw.is_none() {
                    if ui.button(t("✏ Polygon zeichnen", "✏ Draw polygon")).clicked() {
                        self.start_polygon();
                    }
                } else {
                    let k = self.gzp_draw.as_ref().map_or(0, |d| d.len());
                    if ui.add_enabled(k >= 3, egui::Button::new(tf!("✔ Fertig ({})", "✔ Done ({})", k))).clicked() {
                        self.finish_polygon();
                    }
                    if ui.button(t("Abbrechen", "Cancel")).clicked() {
                        self.gzp_draw = None;
                    }
                }
            });
            let mut fixed = self.arpa.gzp_ship_fixed;
            if ui
                .checkbox(&mut fixed, t("Polygon schiffsfest (dreht mit der Vorausrichtung)", "Polygon ship-fixed (rotates with heading)"))
                .on_hover_text(t("Aus: Polygon bleibt nordausgerichtet und wandert nur mit dem Eigenschiff mit.", "Off: polygon stays north-aligned and only moves with own ship."))
                .changed()
            {
                // vorhandene Punkte so umrechnen, dass die Zone auf dem Schirm liegen bleibt
                let hdg = self.world.own.heading;
                let rels: Vec<V2> = self.arpa.gzp_points.iter().map(|q| self.arpa.gzp_to_rel(*q, hdg)).collect();
                self.arpa.gzp_ship_fixed = fixed;
                self.arpa.gzp_points = rels.iter().map(|r| self.arpa.rel_to_gzp(*r, hdg)).collect();
            }
            if ui.button(t("🗑 Guard Zones ausschalten & löschen", "🗑 Switch off & delete guard zones")).clicked() {
                self.arpa.clear_guard_zones();
                self.gzp_draw = None;
                self.say(t("Guard Zones gelöscht", "Guard zones deleted"));
            }
            ui.label(RichText::new(t("Polygon: Eckpunkte per Linksklick auf den Schirm, Enter = fertig, Esc = abbrechen. Auch per Rechtsklick-Menü.", "Polygon: corners with left-click on the screen, Enter = done, Esc = cancel. Also via right-click menu.")).small().italics());
        });

        egui::CollapsingHeader::new(RichText::new(t("🧭 Trial-Manöver", "🧭 Trial manoeuvre")).strong()).default_open(false).show(ui, |ui| {
            let mut active = self.arpa.trial.active;
            if ui.checkbox(&mut active, t("Trial aktiv  (Taste T)", "Trial active  (key T)")).changed() {
                self.toggle_trial();
            }
            let tri = &mut self.arpa.trial;
            ui.horizontal(|ui| {
                ui.label(t("Kurs", "Course"));
                let mut c = tri.course;
                if angle_edit(ui, &mut c, 0.5, 0, "°").changed() {
                    tri.course = norm360(c);
                }
                ui.label(t("Fahrt", "Speed"));
                ui.add(egui::DragValue::new(&mut tri.speed).range(0.0..=30.0).speed(0.1).fixed_decimals(1).suffix(" kn"));
                ui.label(t("Verzögerung", "Delay"));
                ui.add(egui::DragValue::new(&mut tri.delay_min).range(0.0..=30.0).speed(0.1).fixed_decimals(1).suffix(" min"));
            });
            ui.horizontal(|ui| {
                if ui.button(t("Werte vom Eigenschiff", "Values from own ship")).clicked() {
                    self.arpa.trial.course = self.world.own.cmd_heading;
                    self.arpa.trial.speed = self.world.own.cmd_speed;
                }
                if ui.button(t("✔ Manöver ausführen", "✔ Execute manoeuvre")).on_hover_text(t("Trial-Kurs und -Fahrt als Sollwerte übernehmen", "Use trial course and speed as set values")).clicked() {
                    self.world.own.cmd_heading = self.arpa.trial.course;
                    self.world.own.cmd_speed = self.arpa.trial.speed.min(self.world.own.max_speed);
                    self.arpa.trial.active = false;
                    self.say(tf!("Manöver: Sollkurs {}  {:.1} kn", "Manoeuvre: set course {}  {:.1} kn", fb(self.arpa.trial.course), self.arpa.trial.speed));
                }
            });
            if self.arpa.trial.active {
                let o = self.own_ref();
                let now = self.world.time;
                egui::Grid::new("trial").striped(true).show(ui, |ui| {
                    for h in ["Nr", t("CPA jetzt", "CPA now"), "TCPA", "CPA Trial", "TCPA Trial"] {
                        ui.label(RichText::new(h).small().strong());
                    }
                    ui.end_row();
                    for tr in &self.arpa.tracks {
                        let info = self.arpa.info(tr, &o, now);
                        if !info.valid {
                            continue;
                        }
                        if let Some(ti) = self.arpa.trial_info(tr, &o, now) {
                            let c0 = if tr.dangerous { C_DANGER } else { C_ARPA };
                            let c1 = if self.arpa.is_dangerous(ti.cpa, ti.tcpa) { C_DANGER } else { C_TRIAL };
                            ui.label(RichText::new(format!("{}", tr.num)).monospace().color(adapt(ui, c0)));
                            ui.label(RichText::new(format!("{:5.2}", info.cpa)).monospace().color(adapt(ui, c0)));
                            ui.label(RichText::new(ftcpa(info.tcpa)).monospace().color(adapt(ui, c0)));
                            ui.label(RichText::new(format!("{:5.2}", ti.cpa)).monospace().color(adapt(ui, c1)));
                            ui.label(RichText::new(ftcpa(ti.tcpa)).monospace().color(adapt(ui, c1)));
                            ui.end_row();
                        }
                    }
                });
                ui.label(RichText::new(t("Orangen Vektor-Endpunkt auf dem Schirm ziehen, um Kurs/Fahrt zu probieren.", "Drag the orange vector tip on the screen to try course/speed.")).small().italics());
            }
        });

        egui::CollapsingHeader::new(
            RichText::new(t("➕ Ziele (Ausbilder)", "➕ Targets (instructor)")).strong(),
        )
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(t("Typ:", "Type:"));
                let before = self.form.kind;
                egui::ComboBox::from_id_salt("kind")
                    .width(170.0)
                    .selected_text(self.form.kind.name())
                    .show_ui(ui, |ui| {
                        for k in ShipType::ALL {
                            ui.selectable_value(&mut self.form.kind, k, k.name());
                        }
                    });
                if before != self.form.kind {
                    self.form.speed = self.form.kind.spec().def_speed;
                }
            });
            ui.horizontal(|ui| {
                ui.label(t("Peilung", "Bearing"));
                angle_edit(ui, &mut self.form.brg, 1.0, 0, "°");
                ui.label(t("Distanz", "Distance"));
                ui.add(
                    egui::DragValue::new(&mut self.form.rng)
                        .range(0.1..=30.0)
                        .speed(0.05)
                        .fixed_decimals(2)
                        .suffix(" nm"),
                );
            });
            ui.horizontal(|ui| {
                ui.label(t("Kurswahl:", "Course mode:"));
                egui::ComboBox::from_id_salt("cmode")
                    .selected_text(match self.form.mode {
                        CourseMode::Manual => t("Kurs manuell", "Manual course"),
                        CourseMode::Collision => t("Kollisionskurs", "Collision course"),
                        CourseMode::Cpa => t("CPA vorgeben", "Given CPA"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.form.mode,
                            CourseMode::Manual,
                            t("Kurs manuell", "Manual course"),
                        );
                        ui.selectable_value(
                            &mut self.form.mode,
                            CourseMode::Collision,
                            t("Kollisionskurs", "Collision course"),
                        );
                        ui.selectable_value(
                            &mut self.form.mode,
                            CourseMode::Cpa,
                            t("CPA vorgeben", "Given CPA"),
                        );
                    });
            });
            ui.horizontal(|ui| {
                if self.form.mode == CourseMode::Manual {
                    ui.label(t("Kurs", "Course"));
                    angle_edit(ui, &mut self.form.course, 1.0, 0, "°");
                }
                ui.label(t("Fahrt", "Speed"));
                ui.add(
                    egui::DragValue::new(&mut self.form.speed)
                        .range(0.0..=35.0)
                        .speed(0.1)
                        .fixed_decimals(1)
                        .suffix(" kn"),
                );
                if self.form.mode == CourseMode::Cpa {
                    ui.label("CPA");
                    ui.add(
                        egui::DragValue::new(&mut self.form.cpa)
                            .range(-10.0..=10.0)
                            .speed(0.02)
                            .fixed_decimals(2)
                            .suffix(" nm"),
                    )
                    .on_hover_text(t(
                        "Vorzeichen wählt die Passierseite",
                        "Sign selects the passing side",
                    ));
                }
            });
            if ui
                .button(t("➕ Ziel hinzufügen", "➕ Add target"))
                .clicked()
            {
                let pos = self.world.own.pos + V2::from_bearing(self.form.brg, self.form.rng);
                self.place_target(self.form.kind, pos);
            }
            ui.label(
                RichText::new(t(
                    "Oder: Rechtsklick auf den Schirm, dann „Ziel hier platzieren“.",
                    "Or: right-click on the screen, then “Place target here”.",
                ))
                .small()
                .italics(),
            );
            ui.separator();

            let own_pos = self.world.own.pos;
            let mut remove = None;
            egui::Grid::new("targets").striped(true).show(ui, |ui| {
                for h in [
                    t("Ziel", "Target"),
                    t("Peil/Dist", "Brg/Dist"),
                    t("Kurs", "Course"),
                    t("Fahrt", "Speed"),
                    "",
                ] {
                    ui.label(RichText::new(h).small().strong());
                }
                ui.end_row();
                for t in self.world.targets.iter_mut() {
                    let rel = t.pos - own_pos;
                    ui.label(RichText::new(&t.name).small());
                    ui.label(
                        RichText::new(format!("{:05.1}° {:.1}", rel.bearing(), rel.len()))
                            .monospace()
                            .small(),
                    );
                    if t.kind != ShipType::Buoy {
                        let max = t.kind.spec().max_speed;
                        let mut c = t.cmd_heading;
                        if angle_edit(ui, &mut c, 1.0, 0, "°").changed() {
                            t.cmd_heading = norm360(c);
                        }
                        ui.add(
                            egui::DragValue::new(&mut t.cmd_speed)
                                .range(0.0..=max)
                                .speed(0.1)
                                .fixed_decimals(1)
                                .suffix(" kn"),
                        );
                    } else {
                        ui.label("–");
                        ui.label("–");
                    }
                    if ui
                        .small_button("✖")
                        .on_hover_text(crate::i18n::t("Ziel entfernen", "Remove target"))
                        .clicked()
                    {
                        remove = Some(t.id);
                    }
                    ui.end_row();
                }
            });
            if let Some(id) = remove {
                self.world.remove_target(id);
            }
            if !self.world.targets.is_empty()
                && ui
                    .button(t("Alle Ziele entfernen", "Remove all targets"))
                    .clicked()
            {
                self.world.targets.clear();
            }
        });

        egui::CollapsingHeader::new(RichText::new(t("🌊 Umwelt", "🌊 Environment")).strong())
            .default_open(false)
            .show(ui, |ui| {
                let mut ss = self.world.sea_state as i32;
                ui.add(egui::Slider::new(&mut ss, 0..=6).text(t("Seegang", "Sea state")));
                self.world.sea_state = ss as u8;
                ui.horizontal(|ui| {
                    ui.label(t("Wind aus", "Wind from"));
                    angle_edit(ui, &mut self.world.wind_from, 1.0, 0, "°");
                });
                ui.horizontal(|ui| {
                    ui.label(t("Strom nach", "Current setting"));
                    angle_edit(ui, &mut self.world.current_set, 1.0, 0, "°");
                    ui.add(
                        egui::DragValue::new(&mut self.world.current_drift)
                            .range(0.0..=6.0)
                            .speed(0.05)
                            .fixed_decimals(1)
                            .suffix(" kn"),
                    );
                });
                ui.horizontal(|ui| {
                    if ui
                        .button(t("Regenschauer voraus", "Rain shower ahead"))
                        .clicked()
                    {
                        let own = &self.world.own;
                        let c = own.pos
                            + V2::from_bearing(
                                own.heading + 10.0,
                                0.4 * self.radar.range().max(3.0),
                            );
                        self.world.rain.push(RainCell {
                            center: c,
                            radius: 1.2,
                            intensity: 1.2,
                        });
                    }
                    if ui.button(t("Regen entfernen", "Remove rain")).clicked() {
                        self.world.rain.clear();
                    }
                });
            });
    }

    fn help_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_help;
        egui::Window::new(t("Bedienungshilfe", "Help")).open(&mut open).default_width(560.0).vscroll(true).show(ctx, |ui| {
            ui.heading(t("Maus auf dem Radarschirm", "Mouse on the radar screen"));
            for l in [
                t("Linksklick auf ein Echo: ARPA-Erfassung (bzw. Auswahl eines vorhandenen Tracks).", "Left-click on an echo: ARPA acquisition (or select an existing track)."),
                t("Linksklick / Ziehen auf der Peilskala (Außenring): Sollkurs setzen.", "Left-click / drag on the bearing scale (outer ring): set course."),
                t("Grünen Endpunkt des Sollvektors ziehen: Sollkurs und Sollfahrt zugleich.", "Drag the green tip of the set vector: set course and speed together."),
                t("EBL-Linie oder VRM-Kreis ziehen: verstellen.", "Drag an EBL line or VRM circle: adjust."),
                t("Orangen Trial-Vektor ziehen: Probemanöver einstellen.", "Drag the orange trial vector: set up a trial manoeuvre."),
                t("Rechtsklick: Kontextmenü (Ziel platzieren, EBL/VRM, Off-Center, Guard Zone, Track löschen …).", "Right-click: context menu (place target, EBL/VRM, off-centre, guard zone, cancel track …)."),
                t("Mausrad: Bereich wechseln.", "Mouse wheel: change range."),
            ] {
                ui.label(format!("•  {}", l));
            }
            ui.add_space(6.0);
            ui.heading(t("Tastatur", "Keyboard"));
            for l in [
                t("Leertaste / P: Pause      1–7: Zeitraffer 1× … 60×", "Space / P: pause      1–7: time warp 1× … 60×"),
                t("+ / − (Bild↑/Bild↓): Bereich", "+ / − (PgUp/PgDn): range"),
                t("← / →: Sollkurs ±1° (mit Shift ±10°)    ↑ / ↓: Sollfahrt ±0,5 kn (Shift ±2)", "← / →: set course ±1° (Shift ±10°)    ↑ / ↓: set speed ±0.5 kn (Shift ±2)"),
                t("Entf: gewählten Track löschen    Esc: Auswahl aufheben", "Del: cancel selected track    Esc: deselect"),
                t("T: Trial-Manöver an/aus    A: Autopilot-Anzeige an/aus    H / F1: diese Hilfe", "T: trial on/off    A: autopilot display on/off    H / F1: this help"),
                t("Enter: Guard-Zone-Polygon abschließen    Esc: Polygon abbrechen", "Enter: close guard zone polygon    Esc: cancel polygon"),
            ] {
                ui.label(RichText::new(l).monospace());
            }
            ui.add_space(6.0);
            ui.heading(t("ARPA-Symbole", "ARPA symbols"));
            for l in [
                t("Gestrichelter Kreis: Erfassungsphase (ca. 1 min, noch kein Vektor).", "Dashed circle: acquisition phase (approx. 1 min, no vector yet)."),
                t("Kreis mit Vektor: verfolgtes Ziel. Daten bis 3 min als TREND, danach STABIL.", "Circle with vector: tracked target. Data shown as TREND up to 3 min, then STABLE."),
                t("Rotes Dreieck (blinkend): gefährliches Ziel – CPA und TCPA unter den Grenzwerten.", "Red triangle (flashing): dangerous target – CPA and TCPA below the limits."),
                t("Raute mit Kreuz: verlorenes Ziel (kein Echo seit 20 s). Quittieren entfernt den Track.", "Diamond with cross: lost target (no echo for 20 s). Acknowledging removes the track."),
                t("Quadrat: gewähltes Ziel; gestrichelt dessen Relativbahn mit CPA-Punkt.", "Square: selected target; dashed its relative track with CPA point."),
                t("Punkte: vergangene Positionen (wahr oder relativ, je nach Vektormodus).", "Dots: past positions (true or relative, depending on vector mode)."),
                t("MAN: Manöver erkannt – Filter wird verkürzt, Werte sind vorübergehend unsicher.", "MAN: manoeuvre detected – filter shortened, values temporarily uncertain."),
            ] {
                ui.label(format!("•  {}", l));
            }
            ui.add_space(6.0);
            ui.heading(t("Übungshinweise", "Training notes"));
            for l in [
                t("Relative Vektoren zeigen die Kollisionsgefahr direkt: zeigt der Vektor auf das Eigenschiff, besteht Kollisionskurs.", "Relative vectors show the risk of collision directly: a vector pointing at own ship means collision course."),
                t("Wahre Vektoren zeigen Kurs und Fahrt der anderen – wichtig zur Anwendung der KVR.", "True vectors show the other vessels' course and speed – important for applying the COLREGs."),
                t("Seestabilisiert (Log) erscheinen feste Tonnen bei Strom als bewegt – Fehlerquelle bei der Lagebeurteilung!", "Sea-stabilised (log), fixed buoys appear to move in a current – a source of error when assessing the situation!"),
                t("Nach einer Kursänderung des Ziels braucht ARPA Zeit, bis der Vektor wieder stimmt (Szenario Zielmanöver).", "After a target alters course, ARPA needs time until the vector is correct again (scenario target manoeuvre)."),
                t("Zu viel SEA (STC) oder RAIN (FTC) löscht kleine Ziele. Gain so, dass leichtes Rauschen sichtbar ist.", "Too much SEA (STC) or RAIN (FTC) erases small targets. Set gain so that light noise is visible."),
                t("Hinter Land (Insel) gibt es Radarschatten: Ziele dort sind unsichtbar.", "Behind land (island) there is radar shadow: targets there are invisible."),
                t("Trial: Manöver planen, CPA-Ergebnis prüfen, dann „Manöver ausführen“.", "Trial: plan the manoeuvre, check the CPA result, then “Execute manoeuvre”."),
            ] {
                ui.label(format!("•  {}", l));
            }
        });
        self.show_help = open;
    }
}

fn save_png(path: &std::path::Path, img: &egui::ColorImage) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(
        std::io::BufWriter::new(file),
        img.size[0] as u32,
        img.size[1] as u32,
    );
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().map_err(|e| e.to_string())?;
    let bytes: Vec<u8> = img
        .pixels
        .iter()
        .flat_map(|c| c.to_srgba_unmultiplied())
        .collect();
    w.write_image_data(&bytes).map_err(|e| e.to_string())
}

impl RadarApp {
    /// Bildschirmfoto anfordern (Speicherung beim nächsten Frame).
    fn request_snapshot(&mut self, ctx: &egui::Context, path: std::path::PathBuf) {
        self.snap_path = Some(path);
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
    }

    fn user_snapshot(&mut self, ctx: &egui::Context) {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.request_snapshot(ctx, home.join(format!("radar_{}.png", secs)));
    }

    fn handle_snapshots(&mut self, ctx: &egui::Context) {
        let shots: Vec<std::sync::Arc<egui::ColorImage>> = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
                .collect()
        });
        for img in shots {
            if let Some(path) = self.snap_path.take() {
                match save_png(&path, &img) {
                    Ok(()) => self.say(tf!(
                        "Bildschirmfoto gespeichert: {}",
                        "Screenshot saved: {}",
                        path.display()
                    )),
                    Err(e) => self.say(tf!(
                        "Bildschirmfoto fehlgeschlagen: {}",
                        "Screenshot failed: {}",
                        e
                    )),
                }
                if self.demo.as_ref().is_some_and(|d| d.snapped) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    /// Automatischer Vorführ-/Testlauf über Umgebungsvariablen (RADAR_DEMO_SNAP=Datei.png).
    fn run_demo(&mut self, ctx: &egui::Context) {
        let Some(d) = self.demo.as_mut() else { return };
        let now = self.world.time;
        if !d.started {
            d.started = true;
            let (sc, warp, trial, truth) = (d.scenario, d.warp, d.trial, d.truth);
            self.scenario = Scenario::ALL[sc.min(Scenario::ALL.len() - 1)];
            self.load_scenario(self.scenario);
            self.warp_idx = warp;
            self.show_truth = truth;
            if let Some(r) = std::env::var("RADAR_DEMO_RANGE")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
            {
                self.set_range(r);
            }
            self.vrm[0] = Vrm {
                on: true,
                range: 3.0,
            };
            if std::env::var("RADAR_DEMO_BAND").is_ok_and(|v| v == "S") {
                self.radar.set_band(Band::S);
            }
            if std::env::var_os("RADAR_DEMO_GZP").is_some() {
                self.arpa.gzp_points = vec![
                    V2::new(-1.5, 2.0),
                    V2::new(1.5, 2.0),
                    V2::new(2.5, 5.0),
                    V2::new(0.0, 6.5),
                    V2::new(-2.5, 5.0),
                ];
                self.arpa.gzp_on = true;
            }
            if let Ok(t) = std::env::var("RADAR_DEMO_THEME") {
                self.ui_theme = match t.as_str() {
                    "gray" => UiTheme::Gray,
                    "light" => UiTheme::Light,
                    _ => UiTheme::Dark,
                };
                self.ui_theme.apply(ctx);
            }
            if std::env::var("RADAR_DEMO_SCHEME").is_ok_and(|v| v == "orange") {
                self.scheme = ColorScheme::Orange;
                self.bg_level = 1.8;
            }
            if std::env::var_os("RADAR_DEMO_NOAP").is_some() {
                self.show_autopilot = false;
            }
            self.ebl[0] = Ebl {
                on: true,
                brg: 50.0,
            };
            if trial {
                self.arpa.trial.course = norm360(self.world.own.heading + 60.0);
            }
            return;
        }
        if !d.acquired && now > 6.0 {
            d.acquired = true;
            let ids: Vec<(u32, V2)> = self
                .world
                .targets
                .iter()
                .filter(|t| {
                    self.radar.last_detect.get(&t.id).is_some_and(|x| x.1)
                        && t.kind != ShipType::Buoy
                })
                .map(|t| (t.id, t.pos))
                .collect();
            for (id, p) in ids {
                if self.arpa.is_tracked(id).is_none() {
                    self.arpa.acquire(id, now, p, false);
                }
            }
            if let Some(first) = self.arpa.tracks.first() {
                self.arpa.selected = Some(first.num);
            }
            if self.demo.as_ref().is_some_and(|d| d.trial) {
                self.arpa.trial.active = true;
            }
            return;
        }
        let Some(d) = self.demo.as_mut() else { return };
        if !d.snapped && now >= d.at {
            d.snapped = true;
            self.paused = true;
            let path = d.path.clone();
            self.request_snapshot(ctx, path);
        }
    }
}

pub struct Demo {
    path: std::path::PathBuf,
    at: f64,
    scenario: usize,
    warp: usize,
    trial: bool,
    truth: bool,
    started: bool,
    acquired: bool,
    snapped: bool,
}

impl Demo {
    fn from_env() -> Option<Self> {
        let path = std::env::var_os("RADAR_DEMO_SNAP")?;
        let num = |k: &str, d: f64| {
            std::env::var(k)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(d)
        };
        Some(Self {
            path: path.into(),
            at: num("RADAR_DEMO_AT", 240.0),
            scenario: num("RADAR_DEMO_SCENARIO", 1.0) as usize,
            warp: num("RADAR_DEMO_WARP", 4.0) as usize,
            trial: num("RADAR_DEMO_TRIAL", 0.0) > 0.0,
            truth: num("RADAR_DEMO_TRUTH", 0.0) > 0.0,
            started: false,
            acquired: false,
            snapped: false,
        })
    }
}

impl eframe::App for RadarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.real_time = ctx.input(|i| i.time);
        self.handle_snapshots(ctx);
        if ctx.input(|i| i.key_pressed(Key::F12)) && self.mode != AppMode::Sim {
            self.user_snapshot(ctx);
        }
        match self.mode {
            AppMode::Start => {
                self.start_screen(ctx);
                self.demo_frame_snapshot(ctx);
                return;
            }
            AppMode::Plot => {
                self.plot_mode(ctx);
                self.demo_frame_snapshot(ctx);
                return;
            }
            AppMode::Sim => {}
        }
        self.run_demo(ctx);
        self.handle_keys(ctx);
        if ctx.input(|i| i.key_pressed(Key::F12)) {
            self.user_snapshot(ctx);
        }
        self.step_sim(ctx);

        egui::TopBottomPanel::top("top").show(ctx, |ui| self.top_bar(ui));
        egui::SidePanel::left("info")
            .resizable(false)
            .exact_width(270.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.info_panel(ui));
            });
        egui::SidePanel::right("controls")
            .resizable(true)
            .default_width(390.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.controls_panel(ui));
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(self.ui_theme.surround()))
            .show(ctx, |ui| self.ppi(ui));
        if self.show_help {
            self.help_window(ctx);
        }
        ctx.request_repaint();
    }
}
