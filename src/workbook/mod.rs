//! Interaktives Arbeitsbuch: die gängigen Radarphänomene in Kapiteln zum
//! Ausprobieren – mit Querverweisen zwischen den Kapiteln und Übungsfragen,
//! deren Antworten man im jeweiligen Kapitel am Modell „erleben“ kann.

use std::collections::HashSet;

use eframe::egui::{self, vec2, Color32, Rect, RichText, Sense};

use crate::i18n::t;

mod ch_antenna;
mod ch_bands;
mod ch_bearing_res;
mod ch_display;
mod ch_false;
mod ch_intro;
mod ch_plotting;
mod ch_range;
mod ch_range_res;
mod ch_relvec;
mod ch_signal;
mod ch_stab;
mod ch_std;
mod ch_wave;
pub mod charts;
mod questions;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Chapter {
    Intro,
    Wave,
    Bands,
    Antenna,
    RangeRes,
    BearingRes,
    Range,
    FalseEchoes,
    Signal,
    Display,
    Stabilisation,
    RelVector,
    Plotting,
    SpeedTime,
}

impl Chapter {
    pub const ALL: [Chapter; 14] = [
        Chapter::Intro,
        Chapter::Wave,
        Chapter::Bands,
        Chapter::Antenna,
        Chapter::RangeRes,
        Chapter::BearingRes,
        Chapter::Range,
        Chapter::FalseEchoes,
        Chapter::Signal,
        Chapter::Display,
        Chapter::Stabilisation,
        Chapter::RelVector,
        Chapter::Plotting,
        Chapter::SpeedTime,
    ];

    pub fn number(self) -> usize {
        Chapter::ALL.iter().position(|c| *c == self).unwrap_or(0)
    }

    pub fn title(self) -> &'static str {
        match self {
            Chapter::Intro => t("Einführung", "Introduction"),
            Chapter::Wave => t("Welle & Frequenzbänder", "Waves & frequency bands"),
            Chapter::Bands => t("X- und S-Band im Vergleich", "X-band vs. S-band"),
            Chapter::Antenna => t("Antenne & Radarkeule", "Antenna & radar beam"),
            Chapter::RangeRes => t("Radiale Auflösung", "Range resolution"),
            Chapter::BearingRes => t("Azimutale Auflösung", "Bearing resolution"),
            Chapter::Range => t(
                "Reichweite & Radarhorizont",
                "Detection range & radar horizon",
            ),
            Chapter::FalseEchoes => t("Mehrfach- und Falschechos", "Multiple & false echoes"),
            Chapter::Signal => t("Signalverarbeitung", "Signal processing"),
            Chapter::Display => t(
                "Darstellung, Orientierung & Trails",
                "Display, orientation & trails",
            ),
            Chapter::Stabilisation => t(
                "See- und Grundstabilisierung",
                "Sea vs. ground stabilisation",
            ),
            Chapter::RelVector => t(
                "Relativvektor bei Kursänderung",
                "Relative vector in a course change",
            ),
            Chapter::Plotting => t("Manuelles Plotten", "Manual plotting"),
            Chapter::SpeedTime => t("Weg – Zeit – Fahrt", "Speed – time – distance"),
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Chapter::Intro => t(
                "Wie dieses Arbeitsbuch funktioniert",
                "How this workbook works",
            ),
            Chapter::Wave => t(
                "Frequenz, Wellenlänge und die Radarbänder der Schifffahrt",
                "Frequency, wavelength and the marine radar bands",
            ),
            Chapter::Bands => t(
                "Dasselbe Wetter, zwei Radare – live nebeneinander",
                "Same weather, two radars – live side by side",
            ),
            Chapter::Antenna => t(
                "Strahlbündelung, Keulenbreite, Nebenkeulen, Nah- und Fernfeld",
                "Beam forming, beamwidth, side lobes, near and far field",
            ),
            Chapter::RangeRes => t(
                "Pulslänge, Pulsfolgefrequenz und Pulskompression",
                "Pulse length, PRF and pulse compression",
            ),
            Chapter::BearingRes => t(
                "Keulenbreite, Echoverbreiterung und Entfernung",
                "Beamwidth, echo widening and range",
            ),
            Chapter::Range => t(
                "Radarhorizont und Radargleichung",
                "Radar horizon and the radar equation",
            ),
            Chapter::FalseEchoes => t(
                "Mehrfachechos, indirekte Echos, Nebenkeulen, Überreichweiten",
                "Multiple, indirect, side-lobe and second-trace echoes",
            ),
            Chapter::Signal => t(
                "Gain, STC, FTC, Tune und Abtastung am A-Scope",
                "Gain, STC, FTC, tune and sampling on the A-scope",
            ),
            Chapter::Display => t(
                "A-Scope und PPI, Head/North/Course Up, RM/TM, Trails",
                "A-scope and PPI, head/north/course up, RM/TM, trails",
            ),
            Chapter::Stabilisation => t(
                "Was wahre Vektoren bei Strom wirklich zeigen",
                "What true vectors really show in a current",
            ),
            Chapter::RelVector => t(
                "Warum der Relativvektor beim eigenen Manöver schwenkt",
                "Why the relative vector swings when you manoeuvre",
            ),
            Chapter::Plotting => t(
                "Schritt für Schritt vom Peilen bis zum Ausweichmanöver",
                "Step by step from bearings to the avoiding manoeuvre",
            ),
            Chapter::SpeedTime => t(
                "Nomogramm, Zirkelmethode und Rechenschieber",
                "Nomogram, dividers and slide rule",
            ),
        }
    }
}

/// Aktion, die das Arbeitsbuch an die App zurückgibt.
pub enum WbAction {
    None,
    OpenInPlot {
        own_course: f64,
        own_speed: f64,
        target: crate::plot::PTarget,
    },
}

/// Farben passend zur Oberfläche (dunkel/hell).
pub struct Pal {
    pub panel: Color32,
    pub ink: Color32,
    pub faint: Color32,
    pub grid: Color32,
    pub accent: Color32,
    pub x: Color32,
    pub s: Color32,
    pub echo: Color32,
    pub warn: Color32,
    pub ok: Color32,
    pub own: Color32,
    pub tgt: Color32,
    pub rel: Color32,
    pub cur: Color32,
}

pub fn pal(ui: &egui::Ui) -> Pal {
    if ui.visuals().dark_mode {
        Pal {
            panel: Color32::from_rgb(20, 28, 40),
            ink: Color32::from_rgb(220, 228, 240),
            faint: Color32::from_rgb(140, 155, 175),
            grid: Color32::from_rgb(40, 52, 70),
            accent: Color32::from_rgb(120, 190, 255),
            x: Color32::from_rgb(60, 200, 120),
            s: Color32::from_rgb(90, 150, 255),
            echo: Color32::from_rgb(255, 214, 90),
            warn: Color32::from_rgb(255, 110, 90),
            ok: Color32::from_rgb(70, 210, 120),
            own: Color32::WHITE,
            tgt: Color32::from_rgb(255, 120, 120),
            rel: Color32::from_rgb(110, 225, 255),
            cur: Color32::from_rgb(255, 170, 60),
        }
    } else {
        Pal {
            panel: Color32::from_rgb(246, 247, 250),
            ink: Color32::from_rgb(25, 32, 45),
            faint: Color32::from_rgb(105, 115, 130),
            grid: Color32::from_rgb(222, 226, 232),
            accent: Color32::from_rgb(20, 100, 200),
            x: Color32::from_rgb(20, 150, 80),
            s: Color32::from_rgb(40, 90, 210),
            echo: Color32::from_rgb(200, 140, 0),
            warn: Color32::from_rgb(210, 50, 40),
            ok: Color32::from_rgb(20, 150, 70),
            own: Color32::from_rgb(20, 20, 20),
            tgt: Color32::from_rgb(210, 50, 50),
            rel: Color32::from_rgb(0, 130, 170),
            cur: Color32::from_rgb(220, 120, 0),
        }
    }
}

// ---------------------------------------------------------------------------
// Gemeinsame Bausteine der Kapitel
// ---------------------------------------------------------------------------

pub fn header(ui: &mut egui::Ui, ch: Chapter) {
    ui.add_space(6.0);
    ui.label(
        RichText::new(tf!("KAPITEL {}", "CHAPTER {}", ch.number()))
            .small()
            .strong()
            .color(pal(ui).accent),
    );
    ui.label(RichText::new(ch.title()).size(26.0).strong());
    ui.label(RichText::new(ch.subtitle()).italics().size(15.0));
    ui.separator();
}

pub fn text(ui: &mut egui::Ui, s: &str) {
    ui.label(RichText::new(s).size(14.5));
}

pub fn sub(ui: &mut egui::Ui, s: &str) {
    ui.add_space(10.0);
    ui.label(RichText::new(s).strong().size(17.0));
}

pub fn formula(ui: &mut egui::Ui, s: &str) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new(s).monospace().size(14.0));
    });
}

pub fn merke(ui: &mut egui::Ui, lines: &[&str]) {
    let c = pal(ui).accent;
    egui::Frame::group(ui.style())
        .fill(c.gamma_multiply(0.12))
        .stroke(egui::Stroke::new(1.0_f32, c.gamma_multiply(0.6)))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(t("💡 Merke", "💡 Key points"))
                    .strong()
                    .color(c),
            );
            for l in lines {
                ui.label(format!("•  {l}"));
            }
        });
}

pub fn try_it(ui: &mut egui::Ui, s: &str) {
    let c = pal(ui).ok;
    egui::Frame::group(ui.style())
        .fill(c.gamma_multiply(0.10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(format!(
                "{}  {}",
                t("👉 Ausprobieren:", "👉 Try it:"),
                s
            )));
        });
}

pub fn see_also(ui: &mut egui::Ui, nav: &mut Option<Chapter>, chs: &[Chapter]) {
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(t("Siehe auch:", "See also:")).strong());
        for c in chs {
            if ui.link(format!("→ {} {}", c.number(), c.title())).clicked() {
                *nav = Some(*c);
            }
        }
    });
}

/// Pfeil mit Spitze fester Größe (egui skaliert die Spitze sonst mit der Länge).
pub fn arrow(p: &egui::Painter, from: egui::Pos2, vec: egui::Vec2, stroke: egui::Stroke) {
    let len = vec.length();
    if len < 0.5 {
        return;
    }
    let to = from + vec;
    p.line_segment([from, to], stroke);
    let u = vec / len;
    let n = egui::vec2(-u.y, u.x);
    let h = (6.0 + stroke.width * 2.0).min(len * 0.6);
    p.add(egui::Shape::convex_polygon(
        vec![to, to - u * h + n * h * 0.45, to - u * h - n * h * 0.45],
        stroke.color,
        egui::Stroke::NONE,
    ));
}

/// Vorführ-/Testschalter für Screenshots: RADAR_DEMO_WB=name1,name2
pub fn demo_flag(name: &str) -> bool {
    std::env::var("RADAR_DEMO_WB").is_ok_and(|v| v.split(',').any(|f| f.trim() == name))
}

/// Zeichenfläche fester Höhe über die volle Breite.
pub fn canvas(ui: &mut egui::Ui, h: f32, sense: Sense) -> (Rect, egui::Response, egui::Painter) {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), h), sense);
    let p = ui.painter_at(rect);
    (rect, resp, p)
}

/// Schieberegler mit Beschriftung.
pub fn slider(
    ui: &mut egui::Ui,
    v: &mut f64,
    r: std::ops::RangeInclusive<f64>,
    label: &str,
    suffix: &str,
    step: f64,
) -> bool {
    let mut s = egui::Slider::new(v, r)
        .text(label)
        .suffix(suffix.to_string());
    if step > 0.0 {
        s = s.step_by(step);
    }
    ui.add(s).changed()
}

/// Schnellwahl-Knöpfe.
pub fn presets(ui: &mut egui::Ui, v: &mut f64, items: &[(&str, f64)]) {
    ui.horizontal_wrapped(|ui| {
        for (l, x) in items {
            if ui.selectable_label((*v - *x).abs() < 1e-9, *l).clicked() {
                *v = *x;
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Arbeitsbuch
// ---------------------------------------------------------------------------

pub struct Workbook {
    pub current: Chapter,
    history: Vec<Chapter>,
    revealed: HashSet<&'static str>,
    wave: ch_wave::State,
    bands: ch_bands::State,
    antenna: ch_antenna::State,
    range_res: ch_range_res::State,
    bearing_res: ch_bearing_res::State,
    range: ch_range::State,
    false_echoes: ch_false::State,
    signal: ch_signal::State,
    display: ch_display::State,
    stab: ch_stab::State,
    relvec: ch_relvec::State,
    plotting: ch_plotting::State,
    std: ch_std::State,
}

impl Workbook {
    pub fn new() -> Self {
        Self {
            current: Chapter::Intro,
            history: Vec::new(),
            revealed: HashSet::new(),
            wave: Default::default(),
            bands: Default::default(),
            antenna: Default::default(),
            range_res: Default::default(),
            bearing_res: Default::default(),
            range: Default::default(),
            false_echoes: Default::default(),
            signal: Default::default(),
            display: Default::default(),
            stab: Default::default(),
            relvec: Default::default(),
            plotting: Default::default(),
            std: Default::default(),
        }
    }

    pub fn goto(&mut self, ch: Chapter) {
        if ch != self.current {
            self.history.push(self.current);
            self.current = ch;
        }
    }

    /// Kapitelverzeichnis (linke Spalte).
    pub fn nav_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.label(
            RichText::new(t("ARBEITSBUCH", "WORKBOOK"))
                .strong()
                .color(pal(ui).accent),
        );
        ui.separator();
        let mut go = None;
        for ch in Chapter::ALL {
            let label = format!("{:>2}  {}", ch.number(), ch.title());
            if ui.selectable_label(self.current == ch, label).clicked() {
                go = Some(ch);
            }
        }
        ui.separator();
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !self.history.is_empty(),
                    egui::Button::new(t("◀ Zurück", "◀ Back")),
                )
                .clicked()
            {
                if let Some(c) = self.history.pop() {
                    self.current = c;
                }
            }
        });
        if let Some(c) = go {
            self.goto(c);
        }
    }

    /// Inhalt des aktuellen Kapitels.
    pub fn content(&mut self, ui: &mut egui::Ui) -> WbAction {
        let mut nav: Option<Chapter> = None;
        let mut action = WbAction::None;
        let ch = self.current;
        let mut area = egui::ScrollArea::vertical()
            .id_salt(("wb", ch.number()))
            .auto_shrink([false, false]);
        if let Some(off) = std::env::var("RADAR_DEMO_SCROLL")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
        {
            area = area.vertical_scroll_offset(off);
        }
        area.show(ui, |ui| {
            ui.set_max_width(1180.0);
            header(ui, ch);
            match ch {
                Chapter::Intro => ch_intro::ui(ui, &mut nav),
                Chapter::Wave => self.wave.ui(ui, &mut nav),
                Chapter::Bands => self.bands.ui(ui, &mut nav),
                Chapter::Antenna => self.antenna.ui(ui, &mut nav),
                Chapter::RangeRes => self.range_res.ui(ui, &mut nav),
                Chapter::BearingRes => self.bearing_res.ui(ui, &mut nav),
                Chapter::Range => self.range.ui(ui, &mut nav),
                Chapter::FalseEchoes => self.false_echoes.ui(ui, &mut nav),
                Chapter::Signal => self.signal.ui(ui, &mut nav),
                Chapter::Display => self.display.ui(ui, &mut nav),
                Chapter::Stabilisation => self.stab.ui(ui, &mut nav),
                Chapter::RelVector => self.relvec.ui(ui, &mut nav),
                Chapter::Plotting => self.plotting.ui(ui, &mut nav, &mut action),
                Chapter::SpeedTime => self.std.ui(ui, &mut nav),
            }
            questions::chapter_ui(ui, ch, &mut self.revealed, &mut nav);
            footer(ui, ch, &mut nav);
        });
        if let Some(c) = nav {
            self.goto(c);
        }
        action
    }
}

fn footer(ui: &mut egui::Ui, ch: Chapter, nav: &mut Option<Chapter>) {
    ui.add_space(18.0);
    ui.separator();
    ui.horizontal(|ui| {
        let i = ch.number();
        if i > 0 {
            let prev = Chapter::ALL[i - 1];
            if ui
                .button(format!("◀ {} {}", prev.number(), prev.title()))
                .clicked()
            {
                *nav = Some(prev);
            }
        }
        if i + 1 < Chapter::ALL.len() {
            let next = Chapter::ALL[i + 1];
            if ui
                .button(format!("{} {} ▶", next.number(), next.title()))
                .clicked()
            {
                *nav = Some(next);
            }
        }
    });
    ui.add_space(20.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Jedes Kapitel muss sich ohne Absturz zeichnen lassen (beide Sprachen, mehrere Frames).
    #[test]
    fn all_chapters_render() {
        for lang in [crate::i18n::Lang::De, crate::i18n::Lang::En] {
            crate::i18n::set_lang(lang);
            let ctx = egui::Context::default();
            let mut wb = Workbook::new();
            for ch in Chapter::ALL {
                wb.current = ch;
                for _ in 0..3 {
                    let input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1400.0, 900.0),
                        )),
                        ..Default::default()
                    };
                    let _ = ctx.run(input, |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            let _ = wb.content(ui);
                        });
                    });
                }
            }
        }
        crate::i18n::set_lang(crate::i18n::Lang::De);
    }

    /// Kapitel 1 muss sich selbst neu zeichnen lassen – sonst läuft die Welle nur bei Mausbewegung.
    #[test]
    fn wave_chapter_keeps_animating() {
        let ctx = egui::Context::default();
        let mut wb = Workbook::new();
        wb.current = Chapter::Wave;
        for _ in 0..2 {
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let _ = wb.content(ui);
                });
            });
        }
        assert!(ctx.has_requested_repaint());
    }
}
