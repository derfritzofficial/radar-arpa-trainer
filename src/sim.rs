//! Simulationswelt: Eigenschiff, Zielschiffe, Umwelt (Strom, Seegang, Regen, Land) und Szenarien.

use crate::i18n::t;
use crate::util::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ShipType {
    Container,
    Tanker,
    Bulker,
    Ferry,
    Coaster,
    Fishing,
    Tug,
    SailingYacht,
    MotorBoat,
    Buoy,
}

pub struct ShipSpec {
    pub name: &'static str,
    pub length_m: f64,
    /// relative Echostärke (Radarquerschnitt)
    pub strength: f64,
    /// typische Entdeckungsentfernung bei Normal-Gain (nm)
    pub det_range: f64,
    pub def_speed: f64,
    pub max_speed: f64,
    /// max. Drehrate °/min
    pub rot: f64,
    /// Fahrtänderung kn/s
    pub accel: f64,
}

impl ShipType {
    pub const ALL: [ShipType; 10] = [
        ShipType::Container,
        ShipType::Tanker,
        ShipType::Bulker,
        ShipType::Ferry,
        ShipType::Coaster,
        ShipType::Fishing,
        ShipType::Tug,
        ShipType::SailingYacht,
        ShipType::MotorBoat,
        ShipType::Buoy,
    ];

    pub fn spec(self) -> ShipSpec {
        use ShipType::*;
        let s = |name, length_m, strength, det_range, def_speed, max_speed, rot, accel| ShipSpec {
            name,
            length_m,
            strength,
            det_range,
            def_speed,
            max_speed,
            rot,
            accel,
        };
        match self {
            Container => s(
                t("Containerschiff", "Container ship"),
                300.0,
                1.0,
                16.0,
                18.0,
                24.0,
                25.0,
                0.03,
            ),
            Tanker => s("Tanker", 250.0, 1.0, 15.0, 13.0, 16.0, 20.0, 0.02),
            Bulker => s(
                t("Massengutfrachter", "Bulk carrier"),
                190.0,
                0.9,
                14.0,
                12.0,
                15.0,
                25.0,
                0.025,
            ),
            Ferry => s(
                t("Fähre", "Ferry"),
                180.0,
                0.9,
                13.0,
                20.0,
                24.0,
                45.0,
                0.06,
            ),
            Coaster => s(
                t("Küstenmotorschiff", "Coaster"),
                90.0,
                0.7,
                10.0,
                11.0,
                13.0,
                50.0,
                0.04,
            ),
            Fishing => s(
                t("Fischereifahrzeug", "Fishing vessel"),
                25.0,
                0.5,
                7.0,
                5.0,
                11.0,
                90.0,
                0.08,
            ),
            Tug => s(
                t("Schlepper", "Tug"),
                32.0,
                0.55,
                7.0,
                9.0,
                13.0,
                100.0,
                0.1,
            ),
            SailingYacht => s(
                t("Segelyacht", "Sailing yacht"),
                12.0,
                0.3,
                3.5,
                6.0,
                9.0,
                120.0,
                0.1,
            ),
            MotorBoat => s(
                t("Motorboot", "Motor boat"),
                10.0,
                0.25,
                3.0,
                18.0,
                30.0,
                150.0,
                0.3,
            ),
            Buoy => s(
                t("Tonne (Reflektor)", "Buoy (reflector)"),
                3.0,
                0.35,
                3.5,
                0.0,
                0.0,
                0.0,
                0.0,
            ),
        }
    }

    pub fn name(self) -> &'static str {
        self.spec().name
    }
}

/// Geplante Kurs-/Fahrtänderung eines Ziels (für Szenarien).
#[derive(Clone, Copy)]
pub struct Maneuver {
    pub at: f64,
    pub course: f64,
    pub speed: f64,
    pub done: bool,
}

pub struct Target {
    pub id: u32,
    pub kind: ShipType,
    pub name: String,
    pub pos: V2,
    pub heading: f64,
    pub speed: f64,
    pub cmd_heading: f64,
    pub cmd_speed: f64,
    rot: f64,
    pub schedule: Vec<Maneuver>,
}

impl Target {
    pub fn water_vel(&self) -> V2 {
        V2::from_bearing(self.heading, self.speed)
    }

    pub fn ground_vel(&self, current: V2) -> V2 {
        if self.kind == ShipType::Buoy {
            V2::ZERO
        } else {
            self.water_vel() + current
        }
    }
}

pub struct OwnShip {
    pub pos: V2,
    pub heading: f64,
    pub stw: f64,
    pub cmd_heading: f64,
    pub cmd_speed: f64,
    /// aktuelle Drehrate °/s
    pub rot: f64,
    /// max. Drehrate °/min
    pub max_rot: f64,
    pub max_speed: f64,
    pub accel: f64,
    pub length_m: f64,
    pub beam_m: f64,
}

impl OwnShip {
    fn new() -> Self {
        Self {
            pos: V2::ZERO,
            heading: 0.0,
            stw: 12.0,
            cmd_heading: 0.0,
            cmd_speed: 12.0,
            rot: 0.0,
            max_rot: 30.0,
            max_speed: 22.0,
            accel: 0.04,
            length_m: 180.0,
            beam_m: 28.0,
        }
    }

    pub fn water_vel(&self) -> V2 {
        V2::from_bearing(self.heading, self.stw)
    }

    pub fn ground_vel(&self, current: V2) -> V2 {
        self.water_vel() + current
    }
}

pub struct RainCell {
    pub center: V2,
    pub radius: f64,
    pub intensity: f64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scenario {
    Empty,
    CrossingStbd,
    HeadOn,
    Overtaking,
    MultiShip,
    CoastBuoys,
    Clutter,
    FishingField,
    TargetManeuver,
    CurrentStab,
}

impl Scenario {
    pub const ALL: [Scenario; 10] = [
        Scenario::Empty,
        Scenario::CrossingStbd,
        Scenario::HeadOn,
        Scenario::Overtaking,
        Scenario::MultiShip,
        Scenario::CoastBuoys,
        Scenario::Clutter,
        Scenario::FishingField,
        Scenario::TargetManeuver,
        Scenario::CurrentStab,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Scenario::Empty => t("Freies Fahren (leer)", "Free sailing (empty)"),
            Scenario::CrossingStbd => t("Kreuzer von Steuerbord", "Crossing from starboard"),
            Scenario::HeadOn => t(
                "Begegnung (entgegengesetzte Kurse)",
                "Head-on (reciprocal courses)",
            ),
            Scenario::Overtaking => t(
                "Überholen / Überholt werden",
                "Overtaking / being overtaken",
            ),
            Scenario::MultiShip => t("Mehrschiff-Situation", "Multi-ship situation"),
            Scenario::CoastBuoys => t(
                "Küste, Insel & Tonnen (Abschattung)",
                "Coast, island & buoys (shadowing)",
            ),
            Scenario::Clutter => t(
                "Seegang & Regen (Enttrübung üben)",
                "Sea & rain (clutter practice)",
            ),
            Scenario::FishingField => t("Fischereigebiet", "Fishing grounds"),
            Scenario::TargetManeuver => t(
                "Zielmanöver (ARPA-Verzögerung)",
                "Target manoeuvre (ARPA lag)",
            ),
            Scenario::CurrentStab => t(
                "Strom: Grund- vs. Seestabilisierung",
                "Current: ground vs. sea stabilisation",
            ),
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Scenario::Empty => t("Keine Ziele. Ziele selbst über das Ziel-Menü oder Rechtsklick hinzufügen.", "No targets. Add targets via the target menu or right-click."),
            Scenario::CrossingStbd => t("Containerschiff auf Kollisionskurs von Steuerbord. Ziel erfassen, CPA/TCPA bewerten, Ausweichmanöver mit Trial planen (KVR Regel 15/16).", "Container ship on collision course from starboard. Acquire, assess CPA/TCPA, plan avoiding action with trial (COLREGs Rules 15/16)."),
            Scenario::HeadOn => t("Tanker nahezu recht voraus auf Gegenkurs. Regel 14: beide nach Steuerbord. Dazu eine Segelyacht.", "Tanker nearly dead ahead on a reciprocal course. Rule 14: both alter to starboard. Plus a sailing yacht."),
            Scenario::Overtaking => t("Schnelle Fähre überholt von achtern, voraus ein langsamerer Frachter, den Sie überholen.", "Fast ferry overtaking from astern; ahead a slower freighter that you are overtaking."),
            Scenario::MultiShip => t("Sechs Fahrzeuge mit unterschiedlichen CPAs – Prioritäten setzen, Guard Zone nutzen.", "Six vessels with different CPAs – set priorities, use the guard zone."),
            Scenario::CoastBuoys => t("Insel an Steuerbord voraus, Fahrwassertonnen. Ein Schiff taucht aus dem Radarschatten der Insel auf.", "Island fine on the starboard bow, fairway buoys. A ship emerges from the island's radar shadow."),
            Scenario::Clutter => t("Seegang 5 und Regenschauer voraus. Kleine Ziele mit GAIN, SEA (STC) und RAIN (FTC) sichtbar machen.", "Sea state 5 and rain showers ahead. Make small targets visible with GAIN, SEA (STC) and RAIN (FTC)."),
            Scenario::FishingField => t("Viele langsame, manövrierende Fischer. Echospuren (Trails) und Vektoren interpretieren.", "Many slow, manoeuvring fishing vessels. Interpret echo trails and vectors."),
            Scenario::TargetManeuver => t("Ein Ziel passiert zunächst sicher und ändert nach 6 min den Kurs auf Kollisionskurs. Beobachten, wie lange ARPA braucht.", "A target first passes safely, then alters onto a collision course after 6 min. Observe how long ARPA takes."),
            Scenario::CurrentStab => t("2,5 kn Strom aus West. Tonnen und gestoppter Fischer: Unterschied wahre Vektoren GRUND vs. SEE.", "2.5 kn current setting east. Buoys and a stopped fishing vessel: true vectors GROUND vs. SEA."),
        }
    }
}

pub struct World {
    pub time: f64,
    pub own: OwnShip,
    pub targets: Vec<Target>,
    /// Strom: Richtung wohin (set) in Grad, Stärke (drift) in kn
    pub current_set: f64,
    pub current_drift: f64,
    pub sea_state: u8,
    /// Wind aus Richtung (Grad)
    pub wind_from: f64,
    pub rain: Vec<RainCell>,
    pub land: Vec<Vec<V2>>,
    next_id: u32,
    pub rng: Rng,
}

fn steer(heading: &mut f64, rot: &mut f64, cmd: f64, max_rot_dpm: f64, dt: f64) {
    let max = max_rot_dpm / 60.0;
    let err = norm180(cmd - *heading);
    let desired = (err * 0.06).clamp(-max, max);
    let lag = if max_rot_dpm > 60.0 { 3.0 } else { 6.0 };
    let a = (dt / lag).min(1.0);
    *rot += (desired - *rot) * a;
    *heading = norm360(*heading + *rot * dt);
}

fn approach(v: &mut f64, target: f64, rate: f64, dt: f64) {
    let d = target - *v;
    let m = rate * dt;
    *v += d.clamp(-m, m);
}

/// Kurs eines Ziels (Fahrt `vt`), damit es das Eigenschiff mit vorgegebenem CPA passiert.
/// Vorzeichen von `cpa` wählt die Seite. `None`, wenn nicht erreichbar.
pub fn course_for_cpa(own_pos: V2, own_vel: V2, tgt_pos: V2, vt: f64, cpa: f64) -> Option<f64> {
    let r = tgt_pos - own_pos;
    let dist = r.len();
    if cpa.abs() >= dist || dist < 1e-6 {
        return None;
    }
    let alpha = (cpa / dist).asin().to_degrees();
    let base = (-r).bearing();
    let d = V2::from_bearing(base + alpha, 1.0);
    let b = own_vel.dot(d);
    let c = own_vel.len2() - vt * vt;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let k = -b + disc.sqrt();
    if k <= 1e-6 {
        return None;
    }
    let v = own_vel + d * k;
    Some(v.bearing())
}

pub fn make_island(center: V2, radius: f64, seed: u64, n: usize) -> Vec<V2> {
    let mut rng = Rng::new(seed);
    let harm: Vec<(f64, f64, f64)> = (2..8)
        .map(|k| {
            (
                k as f64,
                rng.range(0.04, 0.22) / (k as f64 * 0.5),
                rng.range(0.0, 360.0),
            )
        })
        .collect();
    (0..n)
        .map(|i| {
            let ang = i as f64 / n as f64 * 360.0;
            let mut f = 1.0;
            for (k, a, ph) in &harm {
                f += a * (k * ang + ph).to_radians().sin();
            }
            center + V2::from_bearing(ang, radius * f.max(0.3))
        })
        .collect()
}

/// Lange Küstenlinie im Westen (x < edge_x), zerklüftet.
pub fn make_coast(edge_x: f64, seed: u64) -> Vec<V2> {
    let mut rng = Rng::new(seed);
    let mut pts = Vec::new();
    let mut x = edge_x;
    let mut y: f64 = -40.0;
    while y <= 40.0 {
        x = (x + rng.range(-0.35, 0.35)).clamp(edge_x - 1.5, edge_x + 1.0);
        pts.push(V2::new(x + 0.4 * (y * 0.35).sin(), y));
        y += 0.4;
    }
    pts.push(V2::new(edge_x - 40.0, 40.0));
    pts.push(V2::new(edge_x - 40.0, -40.0));
    pts
}

impl World {
    pub fn new() -> Self {
        let mut w = Self {
            time: 0.0,
            own: OwnShip::new(),
            targets: Vec::new(),
            current_set: 0.0,
            current_drift: 0.0,
            sea_state: 1,
            wind_from: 0.0,
            rain: Vec::new(),
            land: Vec::new(),
            next_id: 1,
            rng: Rng::new(7),
        };
        w.load_scenario(Scenario::Empty);
        w
    }

    pub fn current(&self) -> V2 {
        V2::from_bearing(self.current_set, self.current_drift)
    }

    pub fn add_target(&mut self, kind: ShipType, pos: V2, course: f64, speed: f64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let speed = if kind == ShipType::Buoy { 0.0 } else { speed };
        self.targets.push(Target {
            id,
            kind,
            name: format!("{} #{}", kind.name(), id),
            pos,
            heading: norm360(course),
            speed,
            cmd_heading: norm360(course),
            cmd_speed: speed,
            rot: 0.0,
            schedule: Vec::new(),
        });
        id
    }

    /// Ziel bei Peilung/Distanz mit Kurs, der zum gewünschten CPA führt.
    pub fn add_target_cpa(
        &mut self,
        kind: ShipType,
        brg: f64,
        rng: f64,
        speed: f64,
        cpa: f64,
    ) -> u32 {
        let pos = self.own.pos + V2::from_bearing(brg, rng);
        let ov = self.own.ground_vel(self.current());
        let course = course_for_cpa(self.own.pos, ov, pos, speed, cpa)
            .or_else(|| course_for_cpa(self.own.pos, ov, pos, speed, -cpa))
            .unwrap_or(norm360(brg + 180.0));
        self.add_target(kind, pos, course, speed)
    }

    pub fn remove_target(&mut self, id: u32) {
        self.targets.retain(|t| t.id != id);
    }

    pub fn step(&mut self, dt: f64) {
        self.time += dt;
        let cur = self.current();
        let h = dt / 3600.0;

        let o = &mut self.own;
        o.cmd_heading = norm360(o.cmd_heading);
        steer(&mut o.heading, &mut o.rot, o.cmd_heading, o.max_rot, dt);
        approach(&mut o.stw, o.cmd_speed, o.accel, dt);
        o.pos += (o.water_vel() + cur) * h;

        let now = self.time;
        for t in &mut self.targets {
            if t.kind == ShipType::Buoy {
                continue;
            }
            for m in &mut t.schedule {
                if !m.done && now >= m.at {
                    m.done = true;
                    t.cmd_heading = m.course;
                    t.cmd_speed = m.speed;
                }
            }
            let spec = t.kind.spec();
            steer(&mut t.heading, &mut t.rot, t.cmd_heading, spec.rot, dt);
            approach(&mut t.speed, t.cmd_speed, spec.accel.max(0.01), dt);
            t.pos += (t.water_vel() + cur) * h;
        }

        let drift = V2::from_bearing(self.wind_from + 180.0, 8.0) * h;
        for r in &mut self.rain {
            r.center += drift;
        }
    }

    fn reset(&mut self) {
        self.time = 0.0;
        let (max_rot, accel) = (self.own.max_rot, self.own.accel);
        self.own = OwnShip::new();
        self.own.max_rot = max_rot;
        self.own.accel = accel;
        self.targets.clear();
        self.rain.clear();
        self.land.clear();
        self.current_set = 0.0;
        self.current_drift = 0.0;
        self.sea_state = 1;
        self.wind_from = 0.0;
        self.next_id = 1;
        self.rng = Rng::new(42);
    }

    pub fn load_scenario(&mut self, sc: Scenario) {
        use ShipType::*;
        self.reset();
        match sc {
            Scenario::Empty => {}
            Scenario::CrossingStbd => {
                self.add_target_cpa(Container, 50.0, 9.0, 15.0, 0.05);
                self.add_target_cpa(Coaster, 315.0, 7.5, 10.0, 2.5);
            }
            Scenario::HeadOn => {
                self.add_target_cpa(Tanker, 2.0, 11.0, 13.0, 0.15);
                self.add_target(SailingYacht, V2::from_bearing(25.0, 4.0), 280.0, 5.0);
            }
            Scenario::Overtaking => {
                self.add_target_cpa(Ferry, 190.0, 3.5, 21.0, 0.3);
                self.add_target(Bulker, V2::from_bearing(4.0, 6.0), 358.0, 7.5);
            }
            Scenario::MultiShip => {
                self.add_target_cpa(Container, 40.0, 10.0, 18.0, 0.5);
                self.add_target_cpa(Tanker, 345.0, 12.0, 12.0, 1.5);
                self.add_target_cpa(Ferry, 285.0, 7.0, 19.0, 0.8);
                self.add_target(Fishing, V2::from_bearing(15.0, 5.0), 95.0, 4.0);
                self.add_target(SailingYacht, V2::from_bearing(100.0, 3.0), 200.0, 5.5);
                self.add_target_cpa(Coaster, 200.0, 5.0, 14.0, 1.0);
                self.sea_state = 2;
            }
            Scenario::CoastBuoys => {
                self.land.push(make_island(V2::new(3.2, 6.5), 1.4, 11, 90));
                self.land.push(make_coast(-5.5, 5));
                for (x, y) in [
                    (-0.6, 3.0),
                    (0.6, 3.0),
                    (-0.6, 6.0),
                    (0.6, 6.0),
                    (-0.8, 9.0),
                    (0.8, 9.0),
                ] {
                    self.add_target(Buoy, V2::new(x, y), 0.0, 0.0);
                }
                // Schiff im Radarschatten hinter der Insel
                let id = self.add_target(Coaster, V2::new(4.9, 10.0), 235.0, 10.0);
                let _ = id;
                self.add_target(Ferry, V2::new(-1.0, 14.0), 180.0, 16.0);
                self.sea_state = 1;
            }
            Scenario::Clutter => {
                self.sea_state = 5;
                self.wind_from = 30.0;
                self.rain.push(RainCell {
                    center: V2::new(1.5, 4.5),
                    radius: 1.6,
                    intensity: 1.3,
                });
                self.rain.push(RainCell {
                    center: V2::new(-4.0, 8.0),
                    radius: 2.2,
                    intensity: 0.9,
                });
                self.add_target(SailingYacht, V2::new(1.1, 4.3), 270.0, 5.0);
                self.add_target(MotorBoat, V2::from_bearing(300.0, 1.2), 60.0, 8.0);
                self.add_target(Fishing, V2::from_bearing(30.0, 6.0), 200.0, 3.0);
                self.add_target_cpa(Tanker, 340.0, 12.0, 12.0, 1.2);
            }
            Scenario::FishingField => {
                for _ in 0..10 {
                    let brg = self.rng.range(-45.0, 45.0);
                    let rng = self.rng.range(3.5, 9.0);
                    let crs = self.rng.range(0.0, 360.0);
                    let spd = self.rng.range(1.5, 6.0);
                    let id = self.add_target(Fishing, V2::from_bearing(brg, rng), crs, spd);
                    let mut t_at = self.rng.range(120.0, 400.0);
                    let mut sched = Vec::new();
                    for _ in 0..6 {
                        sched.push(Maneuver {
                            at: t_at,
                            course: self.rng.range(0.0, 360.0),
                            speed: self.rng.range(1.0, 7.0),
                            done: false,
                        });
                        t_at += self.rng.range(180.0, 500.0);
                    }
                    if let Some(t) = self.targets.iter_mut().find(|t| t.id == id) {
                        t.schedule = sched;
                    }
                }
                self.add_target_cpa(Coaster, 160.0, 6.0, 13.0, 0.8);
                self.sea_state = 2;
            }
            Scenario::TargetManeuver => {
                let id = self.add_target_cpa(Coaster, 300.0, 8.0, 12.0, 2.2);
                // Position nach 6 min vorausberechnen und Kollisionskurs bestimmen
                let at = 360.0;
                let own_at = self.own.pos + self.own.ground_vel(V2::ZERO) * (at / 3600.0);
                if let Some(t) = self.targets.iter_mut().find(|t| t.id == id) {
                    let t_at = t.pos + t.water_vel() * (at / 3600.0);
                    let c = course_for_cpa(own_at, self.own.ground_vel(V2::ZERO), t_at, 12.0, 0.05)
                        .unwrap_or(t.heading + 40.0);
                    t.schedule.push(Maneuver {
                        at,
                        course: c,
                        speed: 12.0,
                        done: false,
                    });
                }
                self.add_target(Bulker, V2::from_bearing(170.0, 7.0), 30.0, 11.0);
            }
            Scenario::CurrentStab => {
                self.current_set = 90.0;
                self.current_drift = 2.5;
                for (x, y) in [(-0.7, 2.5), (0.7, 2.5), (1.5, 6.0), (-2.0, 5.0)] {
                    self.add_target(Buoy, V2::new(x, y), 0.0, 0.0);
                }
                self.add_target(Fishing, V2::new(-2.5, 3.5), 0.0, 0.0);
                self.add_target_cpa(Container, 60.0, 10.0, 16.0, 1.0);
                self.sea_state = 2;
            }
        }
    }
}
