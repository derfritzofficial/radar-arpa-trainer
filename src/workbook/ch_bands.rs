//! Kapitel 2: X- und S-Band live nebeneinander (gleiche Szene, zwei Radare).

use eframe::egui::{
    self, pos2, vec2, Align2, Color32, ColorImage, FontId, Rect, RichText, Sense, Stroke,
    TextureHandle, TextureOptions,
};

use super::{merke, pal, see_also, slider, sub, text, try_it, Chapter};
use crate::app::{make_palette, ColorScheme};
use crate::i18n::t;
use crate::radar::{Band, PpiRenderer, Radar, ANT_PERIOD, IMG, RANGES, RING_INT};
use crate::sim::{RainCell, ShipType, World};
use crate::util::V2;

struct Sim {
    world: World,
    radars: [Radar; 2],
    rend: [PpiRenderer; 2],
    img: [ColorImage; 2],
    tex: [Option<TextureHandle>; 2],
    palette: [Color32; 256],
}

impl Sim {
    fn new(range_idx: usize) -> Self {
        let mut world = World::new();
        world.own.stw = 0.0;
        world.own.cmd_speed = 0.0;
        world.wind_from = 20.0;
        world.rain.push(RainCell {
            center: V2::new(1.0, 3.4),
            radius: 1.4,
            intensity: 1.2,
        });
        world.rain.push(RainCell {
            center: V2::new(-3.3, -1.6),
            radius: 1.0,
            intensity: 1.0,
        });
        world.add_target(ShipType::SailingYacht, V2::new(0.9, 3.3), 270.0, 2.0);
        world.add_target(ShipType::Container, V2::new(2.4, 5.1), 200.0, 4.0);
        world.add_target(ShipType::Buoy, V2::new(1.6, -1.2), 0.0, 0.0);
        world.add_target(ShipType::Fishing, V2::new(-2.6, 2.1), 120.0, 2.0);
        world.add_target(ShipType::MotorBoat, V2::new(0.55, -0.6), 300.0, 3.0);
        let mut rx = Radar::new();
        let mut rs = Radar::new();
        rx.set_band(Band::X);
        rs.set_band(Band::S);
        rx.set_range(range_idx);
        rs.set_range(range_idx);
        // eine volle Antennenumdrehung vorlaufen lassen, damit das Bild sofort vollständig ist
        let mut tt = 0.0;
        while tt < ANT_PERIOD * 1.05 {
            world.step(0.05);
            rx.update(0.05, &world);
            rs.update(0.05, &world);
            tt += 0.05;
        }
        rx.events.clear();
        rs.events.clear();
        Self {
            world,
            radars: [rx, rs],
            rend: [PpiRenderer::new(), PpiRenderer::new()],
            img: [
                ColorImage::new([IMG, IMG], Color32::TRANSPARENT),
                ColorImage::new([IMG, IMG], Color32::TRANSPARENT),
            ],
            tex: [None, None],
            palette: make_palette(ColorScheme::Blue, 1.0, 1.0),
        }
    }
}

pub struct State {
    sim: Option<Box<Sim>>,
    sea: f64,
    rain: f64,
    gain: f64,
    stc: f64,
    ftc: f64,
    range_idx: usize,
    labels: bool,
    d_s_ft: f64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            sim: None,
            sea: 4.0,
            rain: 1.2,
            gain: 55.0,
            stc: 35.0,
            ftc: 0.0,
            range_idx: 5,
            labels: true,
            d_s_ft: 10.0,
        }
    }
}

impl State {
    pub fn ui(&mut self, ui: &mut egui::Ui, nav: &mut Option<Chapter>) {
        let p = pal(ui);
        text(
            ui,
            t(
                "Links und rechts sehen Sie dieselbe Szene mit identischen Einstellungen – links mit einem X-Band-, rechts mit einem S-Band-Radar. Im Regenschauer voraus fährt eine Segelyacht, dahinter ein Containerschiff, nah am Schiff ein Motorboot und eine Tonne im Seegang.",
                "Left and right show the same scene with identical settings – on the left an X-band, on the right an S-band radar. A sailing yacht is inside the rain shower ahead, a container ship is behind it, and a motor boat and a buoy are close by in the sea clutter.",
            ),
        );
        ui.horizontal_wrapped(|ui| {
            slider(
                ui,
                &mut self.sea,
                0.0..=6.0,
                t("Seegang", "Sea state"),
                "",
                1.0,
            );
            slider(
                ui,
                &mut self.rain,
                0.0..=2.0,
                t("Regenstärke", "Rain"),
                "",
                0.05,
            );
            ui.label(t("Bereich:", "Range:"));
            for idx in [4usize, 5, 6] {
                if ui
                    .selectable_label(self.range_idx == idx, format!("{} NM", RANGES[idx]))
                    .clicked()
                {
                    self.range_idx = idx;
                }
            }
            ui.checkbox(&mut self.labels, t("Objekte beschriften", "Label objects"));
        });
        ui.horizontal_wrapped(|ui| {
            slider(ui, &mut self.gain, 0.0..=100.0, "GAIN", "", 1.0);
            slider(ui, &mut self.stc, 0.0..=100.0, "SEA (STC)", "", 1.0);
            slider(ui, &mut self.ftc, 0.0..=100.0, "RAIN (FTC)", "", 1.0);
        });

        // --- Simulation und Darstellung
        let range_idx = self.range_idx;
        let sim = self
            .sim
            .get_or_insert_with(|| Box::new(Sim::new(range_idx)));
        let dt = ui.input(|i| i.stable_dt).min(0.1) as f64;
        sim.world.sea_state = self.sea as u8;
        sim.world.rain[0].intensity = self.rain;
        sim.world.rain[1].intensity = self.rain * 0.8;
        sim.world.step(dt);
        for r in sim.radars.iter_mut() {
            r.gain = self.gain;
            r.sea = self.stc;
            r.rain = self.ftc;
            r.set_range(self.range_idx);
            r.update(dt, &sim.world);
            r.events.clear();
        }
        let avail = ui.available_width();
        let size = ((avail - 30.0) / 2.0).clamp(220.0, 430.0);
        let (rect, _) = ui.allocate_exact_size(vec2(avail, size + 56.0), Sense::hover());
        let painter = ui.painter_at(rect);
        let range = RANGES[self.range_idx];
        let ring = RING_INT[self.range_idx];
        for k in 0..2 {
            let r = &sim.radars[k];
            sim.rend[k].render(
                r,
                (0.0, 0.0),
                0.0,
                sim.world.time,
                &sim.palette,
                3.0 * ANT_PERIOD,
                &mut sim.img[k],
            );
            let tex = sim.tex[k].get_or_insert_with(|| {
                ui.ctx().load_texture(
                    format!("band{k}"),
                    sim.img[k].clone(),
                    TextureOptions::LINEAR,
                )
            });
            tex.set(sim.img[k].clone(), TextureOptions::LINEAR);
            let cx = rect.left() + avail * (0.25 + 0.5 * k as f32);
            let c = pos2(cx, rect.top() + 34.0 + size / 2.0);
            let rad = size / 2.0;
            painter.circle_filled(c, rad + 8.0, Color32::from_rgb(22, 40, 70));
            painter.image(
                tex.id(),
                Rect::from_center_size(c, vec2(size, size)),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            let n = (range / ring).round() as usize;
            for i in 1..=n {
                painter.circle_stroke(
                    c,
                    rad * (i as f64 * ring / range) as f32,
                    Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(160, 200, 240, 90)),
                );
            }
            painter.circle_stroke(
                c,
                rad,
                Stroke::new(1.5_f32, Color32::from_rgb(160, 200, 240)),
            );
            painter.line_segment(
                [c, c - vec2(0.0, rad)],
                Stroke::new(1.2_f32, Color32::WHITE),
            );
            let (title, col) = if k == 0 {
                (
                    t(
                        "X-Band  9,4 GHz  (λ ≈ 3,2 cm, Keule 1,2°)",
                        "X-band  9.4 GHz  (λ ≈ 3.2 cm, beam 1.2°)",
                    ),
                    p.x,
                )
            } else {
                (
                    t(
                        "S-Band  3,0 GHz  (λ ≈ 10 cm, Keule 1,9°)",
                        "S-band  3.0 GHz  (λ ≈ 10 cm, beam 1.9°)",
                    ),
                    p.s,
                )
            };
            painter.text(
                pos2(cx, rect.top() + 4.0),
                Align2::CENTER_TOP,
                title,
                FontId::proportional(15.0),
                col,
            );
            if self.labels {
                let ppn = rad / range as f32;
                for tg in &sim.world.targets {
                    let rel = tg.pos - sim.world.own.pos;
                    if rel.len() > range {
                        continue;
                    }
                    let s = c + vec2(rel.x as f32, -rel.y as f32) * ppn;
                    let name = match tg.kind {
                        ShipType::SailingYacht => t("Yacht im Regen", "yacht in rain"),
                        ShipType::Container => t("Containerschiff", "container ship"),
                        ShipType::Buoy => t("Tonne", "buoy"),
                        ShipType::Fishing => t("Fischer", "fishing vessel"),
                        _ => t("Motorboot", "motor boat"),
                    };
                    painter.circle_stroke(
                        s,
                        9.0,
                        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 120)),
                    );
                    painter.text(
                        s + vec2(11.0, -11.0),
                        Align2::LEFT_BOTTOM,
                        name,
                        FontId::proportional(11.0),
                        Color32::from_rgb(230, 235, 245),
                    );
                }
            }
        }
        ui.ctx().request_repaint();

        try_it(
            ui,
            t(
                "Drehen Sie die Regenstärke hoch: Links verschwindet die Yacht im Regenecho und das Containerschiff dahinter wird gedämpft, rechts bleiben beide sichtbar. Erhöhen Sie den Seegang: Im X-Band wächst der Clutter um das Schiff deutlich stärker. Versuchen Sie dann, im X-Band mit RAIN (FTC) und SEA (STC) die Ziele wieder herauszuholen.",
                "Turn up the rain: on the left the yacht disappears in the rain echo and the container ship behind is attenuated; on the right both stay visible. Increase the sea state: the clutter around the ship grows much more in X-band. Then try to recover the targets in X-band with RAIN (FTC) and SEA (STC).",
            ),
        );

        sub(ui, t("Vergleich", "Comparison"));
        egui::Grid::new("band_cmp")
            .striped(true)
            .num_columns(3)
            .spacing([24.0, 6.0])
            .show(ui, |ui| {
                ui.label(RichText::new(t("Eigenschaft", "Property")).strong());
                ui.label(RichText::new("X-Band").strong().color(p.x));
                ui.label(RichText::new("S-Band").strong().color(p.s));
                ui.end_row();
                let rows: [(&str, &str, &str); 11] = [
                    (t("Frequenz", "Frequency"), "9,2 – 9,5 GHz", "2,9 – 3,1 GHz"),
                    (t("Wellenlänge", "Wavelength"), "≈ 3,2 cm", "≈ 10 cm"),
                    (
                        t(
                            "Keulenbreite (übliche Antenne)",
                            "Beamwidth (usual antenna)",
                        ),
                        t("≈ 1° (8 ft)", "≈ 1° (8 ft)"),
                        t("≈ 2° (12 ft)", "≈ 2° (12 ft)"),
                    ),
                    (
                        t("Seegangsclutter", "Sea clutter"),
                        t("stark", "strong"),
                        t("gering", "weak"),
                    ),
                    (
                        t("Regen: Echo und Dämpfung", "Rain: echo and attenuation"),
                        t("stark", "strong"),
                        t("kaum", "hardly any"),
                    ),
                    (
                        t("Auflösung, kleine Ziele", "Resolution, small targets"),
                        t("besser", "better"),
                        t("schwächer", "weaker"),
                    ),
                    (
                        t("Reichweite bei schlechtem Wetter", "Range in bad weather"),
                        t("geringer", "shorter"),
                        t("größer", "longer"),
                    ),
                    (
                        t("typische Sendeleistung", "typical transmitter power"),
                        "12 – 25 kW",
                        "30 kW",
                    ),
                    (
                        t(
                            "IMO-Radarreflektor (Mindest-RCS)",
                            "IMO radar reflector (min. RCS)",
                        ),
                        "7,5 m²",
                        "0,5 m²",
                    ),
                    (
                        t("SART-Transponder", "SART transponder"),
                        t("ja", "yes"),
                        t("nein", "no"),
                    ),
                    (
                        t("Antenne", "Antenna"),
                        t("klein, leicht", "small, light"),
                        t("groß, schwer", "large, heavy"),
                    ),
                ];
                for (a, b, c) in rows {
                    ui.label(a);
                    ui.label(b);
                    ui.label(c);
                    ui.end_row();
                }
            });

        sub(
            ui,
            t(
                "Gleiche Keulenbreite – wie lang muss die Antenne sein?",
                "Same beamwidth – how long must the antenna be?",
            ),
        );
        text(
            ui,
            t(
                "Die Keulenbreite ist proportional zu λ / D. Soll ein X-Band-Radar dieselbe Keule haben wie ein S-Band-Radar, darf seine Antenne um das Verhältnis der Wellenlängen kürzer sein.",
                "The beamwidth is proportional to λ / D. For an X-band radar to have the same beam as an S-band radar, its antenna may be shorter by the ratio of the wavelengths.",
            ),
        );
        slider(
            ui,
            &mut self.d_s_ft,
            4.0..=20.0,
            t("S-Band-Antenne", "S-band antenna"),
            " ft",
            0.5,
        );
        let rounded = self.d_s_ft * 3.0 / 10.0;
        let real = self.d_s_ft * 3.05 / 9.41;
        ui.label(
            RichText::new(tf!(
                "D_X = D_S · λ_X / λ_S = {:.1} ft · 3/10 = {:.2} ft ({:.2} m)   ·   mit 3,05/9,41 GHz: {:.2} ft",
                "D_X = D_S · λ_X / λ_S = {:.1} ft · 3/10 = {:.2} ft ({:.2} m)   ·   with 3.05/9.41 GHz: {:.2} ft",
                self.d_s_ft,
                rounded,
                rounded * 0.3048,
                real
            ))
            .monospace()
            .strong(),
        );

        merke(
            ui,
            &[
                t("S-Band: weniger See- und Regenclutter, kaum Dämpfung im Regen ⇒ größere Reichweite bei schlechtem Wetter.", "S-band: less sea and rain clutter, hardly any attenuation in rain ⇒ longer range in bad weather."),
                t("X-Band: schmalere Keule bei gleicher Antennengröße ⇒ bessere Auflösung, kleine Ziele besser sichtbar; SART nur im X-Band.", "X-band: narrower beam with the same antenna size ⇒ better resolution, small targets more visible; SART only in X-band."),
                t("Gleiche Keulenbreite: D_X / D_S = λ_X / λ_S ≈ 0,3.", "Same beamwidth: D_X / D_S = λ_X / λ_S ≈ 0.3."),
                t("Deshalb fahren große Schiffe oft beide Radare – S-Band für Reichweite und Wetter, X-Band für Detail.", "That is why large ships often carry both – S-band for range and weather, X-band for detail."),
            ],
        );
        see_also(
            ui,
            nav,
            &[
                Chapter::Wave,
                Chapter::Antenna,
                Chapter::BearingRes,
                Chapter::Signal,
                Chapter::Range,
            ],
        );
    }
}
