// Auf Windows im Release-Build kein Konsolenfenster öffnen
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
mod i18n;
mod app;
mod arpa;
mod plot;
mod radar;
mod sim;
mod templates;
mod util;

/// App-Icon (Fenster, Dock, Taskleiste) aus dem eingebetteten PNG.
fn app_icon() -> Option<eframe::egui::IconData> {
    let bytes: &[u8] = include_bytes!("../assets/icon_256.png");
    let mut dec = png::Decoder::new(bytes);
    dec.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = dec.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf
            .chunks(3)
            .flat_map(|c| [c[0], c[1], c[2], 255])
            .collect(),
        _ => return None,
    };
    Some(eframe::egui::IconData {
        rgba,
        width: info.width,
        height: info.height,
    })
}

fn main() -> eframe::Result {
    let mut viewport = eframe::egui::ViewportBuilder::default();
    if let Some(icon) = app_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport: viewport
            .with_title("Radar / ARPA Trainer")
            .with_inner_size([1560.0, 960.0])
            .with_min_inner_size([1100.0, 700.0])
            // Testläufe (RADAR_DEMO_SNAP) sollen keinen Fokus stehlen
            .with_active(std::env::var_os("RADAR_DEMO_SNAP").is_none()),
        ..Default::default()
    };
    eframe::run_native(
        "Radar / ARPA Trainer",
        options,
        Box::new(|cc| Ok(Box::new(app::RadarApp::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use crate::arpa::*;
    use crate::radar::*;
    use crate::sim::*;

    fn run(sc: Scenario, range_idx: usize, secs: f64) -> (World, Radar, Arpa) {
        let mut w = World::new();
        w.load_scenario(sc);
        let mut r = Radar::new();
        r.set_range(range_idx);
        let mut a = Arpa::new();
        let mut acquired = false;
        let dt = 0.2;
        let mut t = 0.0;
        while t < secs {
            w.step(dt);
            r.update(dt, &w);
            let ev = std::mem::take(&mut r.events);
            a.process(&ev);
            a.update(&w);
            if !acquired && w.time > 6.0 {
                acquired = true;
                for tg in &w.targets {
                    if r.last_detect.get(&tg.id).is_some_and(|x| x.1) && tg.kind != ShipType::Buoy {
                        println!("acquire target #{} {}", tg.id, tg.kind.name());
                        a.acquire(tg.id, w.time, tg.pos, false);
                    }
                }
            }
            t += dt;
        }
        (w, r, a)
    }

    #[test]
    fn app_icon_decodes() {
        let icon = crate::app_icon().expect("Icon muss dekodierbar sein");
        assert_eq!((icon.width, icon.height), (256, 256));
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
    }

    #[test]
    fn angle_inputs_wrap_both_ways() {
        use crate::app::wrap_angle;
        assert_eq!(wrap_angle(360.0, 1), 0.0);
        assert_eq!(wrap_angle(359.96, 1), 0.0);
        assert_eq!(wrap_angle(361.5, 1), 1.5);
        assert_eq!(wrap_angle(-0.5, 1), 359.5);
        assert_eq!(wrap_angle(-1.0, 0), 359.0);
        assert_eq!(wrap_angle(725.0, 0), 5.0);
        assert_eq!(wrap_angle(359.0, 0), 359.0);
    }

    #[test]
    fn multiship_tracking() {
        let (w, _r, a) = run(Scenario::MultiShip, 6, 240.0);
        let o = OwnRef::from_world(&w);
        for tr in &a.tracks {
            let i = a.info(tr, &o, w.time);
            let t = w.targets.iter().find(|t| t.id == tr.target_id).unwrap();
            let gv = t.ground_vel(w.current());
            println!(
                "track {} -> #{} {:?} est cse {:.1} spd {:.1} | true cse {:.1} spd {:.1} | cpa {:.2} tcpa {:.1}",
                tr.num, tr.target_id, tr.state, i.cse, i.spd, gv.bearing(), gv.len(), i.cpa, i.tcpa
            );
        }
        for al in &a.alarms {
            println!("alarm {}", al.text());
        }
        assert!(a.tracks.iter().all(|t| t.state == TrackState::Tracking));
    }

    #[test]
    fn multiship_no_lost_with_polygon_gz() {
        use crate::util::V2;
        let mut w = World::new();
        w.load_scenario(Scenario::MultiShip);
        let mut r = Radar::new();
        r.set_range(6);
        let mut a = Arpa::new();
        a.gzp_points = vec![
            V2::new(-1.5, 2.0),
            V2::new(1.5, 2.0),
            V2::new(2.5, 5.0),
            V2::new(0.0, 6.5),
            V2::new(-2.5, 5.0),
        ];
        a.gzp_on = true;
        let mut acquired = false;
        let mut misses: std::collections::HashMap<u32, u32> = Default::default();
        while w.time < 240.0 {
            w.step(0.2);
            r.update(0.2, &w);
            let ev = std::mem::take(&mut r.events);
            for e in &ev {
                if e.target_id == 2 && !e.visible {
                    *misses.entry(2).or_default() += 1;
                    println!("t={:.1} tanker invisible rel={:.2}", e.time, e.rel.len());
                }
            }
            a.process(&ev);
            a.update(&w);
            if !acquired && w.time > 6.0 {
                acquired = true;
                for tg in &w.targets {
                    if r.last_detect.get(&tg.id).is_some_and(|x| x.1)
                        && tg.kind != ShipType::Buoy
                        && a.is_tracked(tg.id).is_none()
                    {
                        a.acquire(tg.id, w.time, tg.pos, false);
                    }
                }
            }
        }
        for al in &a.alarms {
            println!("alarm {} at {:.0}", al.text(), al.time);
        }
        assert!(a.alarms.iter().all(|al| al.kind != AlarmKind::Lost));
    }

    /// Bereich verkleinern darf keine Tracks außerhalb des Videobereichs verlieren.
    #[test]
    fn tracks_survive_range_reduction() {
        let mut w = World::new();
        w.load_scenario(Scenario::MultiShip);
        let mut r = Radar::new();
        r.set_range(6); // 12 NM
        let mut a = Arpa::new();
        let mut acquired = false;
        while w.time < 200.0 {
            if w.time > 60.0 {
                r.set_range(4); // 3 NM: fast alle Ziele außerhalb des Videos
            }
            w.step(0.2);
            r.update(0.2, &w);
            let ev = std::mem::take(&mut r.events);
            a.process(&ev);
            a.update(&w);
            if !acquired && w.time > 6.0 {
                acquired = true;
                for tg in &w.targets {
                    if r.last_detect.get(&tg.id).is_some_and(|x| x.1) && tg.kind != ShipType::Buoy {
                        a.acquire(tg.id, w.time, tg.pos, false);
                    }
                }
            }
        }
        assert!(a.tracks.len() >= 4);
        assert!(
            a.tracks.iter().all(|t| t.state == TrackState::Tracking),
            "kein Track darf verloren gehen"
        );
    }

    /// Ziel genau am Rand des Videopuffers darf keinen Absturz auslösen.
    #[test]
    fn target_at_buffer_edge_no_panic() {
        use crate::util::V2;
        let mut w = World::new();
        w.load_scenario(Scenario::Empty);
        let mut r = Radar::new();
        r.set_range(5); // 6 NM → Puffer 10,2 nm
        let br = 6.0 * BUF_FACTOR;
        let bin = br / N_BIN as f64;
        // Ziele rund um die Puffergrenze in allen Richtungen
        for k in 0..40 {
            let d = br - bin * 1.5 + bin * 0.075 * k as f64;
            w.add_target(
                ShipType::Buoy,
                V2::from_bearing(k as f64 * 9.0, d),
                0.0,
                0.0,
            );
        }
        for _ in 0..30 {
            w.step(0.2);
            r.update(0.2, &w);
        }
    }

    #[test]
    fn polygon_guard_zone_auto_acquires() {
        use crate::util::V2;
        let mut w = World::new();
        w.load_scenario(Scenario::Empty);
        // Ziel 5 nm voraus, im schiffsfesten Polygon (Trapez voraus)
        w.add_target(ShipType::Coaster, V2::new(0.0, 5.0), 180.0, 5.0);
        // Ziel querab Steuerbord, außerhalb
        w.add_target(ShipType::Coaster, V2::new(4.0, 0.0), 0.0, 5.0);
        let mut r = Radar::new();
        let mut a = Arpa::new();
        a.gzp_points = vec![
            V2::new(-1.0, 2.0),
            V2::new(1.0, 2.0),
            V2::new(2.0, 6.0),
            V2::new(-2.0, 6.0),
        ];
        a.gzp_on = true;
        for _ in 0..100 {
            w.step(0.2);
            r.update(0.2, &w);
            let ev = std::mem::take(&mut r.events);
            a.process(&ev);
            a.update(&w);
        }
        assert_eq!(a.tracks.len(), 1, "nur das Ziel im Polygon wird erfasst");
        assert_eq!(a.tracks[0].target_id, 1);
        assert!(a.alarms.iter().any(|al| al.kind == AlarmKind::GuardZone));
        a.clear_guard_zones();
        assert!(!a.gzp_on && a.gzp_points.is_empty());
        assert!(a.alarms.iter().all(|al| al.kind != AlarmKind::GuardZone));
    }

    /// Mittlere Videohelligkeit (Seegangsclutter) für ein Band.
    fn clutter_level(band: Band) -> f32 {
        let mut w = World::new();
        w.load_scenario(Scenario::Clutter);
        w.targets.clear();
        w.rain.clear();
        let mut r = Radar::new();
        r.set_band(band);
        for _ in 0..20 {
            w.step(0.2);
            r.update(0.2, &w);
        }
        r.cells.iter().sum::<f32>() / r.cells.len() as f32
    }

    #[test]
    fn s_band_has_less_sea_clutter() {
        let x = clutter_level(Band::X);
        let s = clutter_level(Band::S);
        println!("clutter X {x:.4}  S {s:.4}");
        assert!(s < x * 0.5);
    }
}
