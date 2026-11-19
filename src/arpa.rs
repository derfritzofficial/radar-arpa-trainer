//! ARPA: Zielerfassung, Verfolgung (Plot-Extraktion mit Messrauschen, adaptive
//! Regressionsfilterung mit Manövererkennung), CPA/TCPA/BCR/BCT, Alarme,
//! Guard Zone, vergangene Positionen und Probemanöver (Trial).

use std::collections::{HashSet, VecDeque};

use crate::radar::DetectionEvent;
use crate::sim::World;
use crate::util::*;

/// Erfassungsphase bis zum ersten Vektor (IMO: Trend innerhalb 1 min)
pub const ACQ_TIME: f64 = 60.0;
/// volle Genauigkeit (IMO: 3 min)
pub const FULL_TIME: f64 = 180.0;
/// ohne Plot so lange → verlorenes Ziel
pub const LOST_TIME: f64 = 20.0;
pub const MAX_TRACKS: usize = 40;
const RNG_SIGMA: f64 = 0.006;
const BRG_SIGMA: f64 = 0.15;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrackState {
    Acquiring,
    Tracking,
    Lost,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum VectorMode {
    True,
    Relative,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AlarmKind {
    Cpa,
    Lost,
    GuardZone,
}

#[derive(Clone, Copy)]
struct Plot {
    t: f64,
    p: V2,
}

pub struct Track {
    pub num: u32,
    pub target_id: u32,
    pub state: TrackState,
    pub auto: bool,
    pub acq_time: f64,
    pub last_plot: f64,
    plots: VecDeque<Plot>,
    window_start: f64,
    suspect: u32,
    pub maneuver_time: f64,
    pub pos: V2,
    /// geschätzte Geschwindigkeit über Grund (kn)
    pub vel: V2,
    pub est_time: f64,
    pub vel_valid: bool,
    /// vergangene Positionen: (Zeit, geschätzte Position, Eigenschiffposition)
    pub past: VecDeque<(f64, V2, V2)>,
    last_past: f64,
    pub dangerous: bool,
    lost_pos: V2,
}

impl Track {
    pub fn display_pos(&self, now: f64) -> V2 {
        match self.state {
            TrackState::Lost => self.lost_pos,
            _ if self.vel_valid => self.pos + self.vel * ((now - self.est_time) / 3600.0),
            _ => self.pos,
        }
    }

    pub fn age(&self, now: f64) -> f64 {
        now - self.acq_time
    }

    pub fn plot_count(&self) -> usize {
        self.plots.len()
    }

    pub fn maneuver_recent(&self, now: f64) -> bool {
        now - self.maneuver_time < 45.0
    }

    fn add_plot(&mut self, t: f64, p: V2, rng: f64) {
        if self.vel_valid {
            let pred = self.pos + self.vel * ((t - self.est_time) / 3600.0);
            let sigma = (RNG_SIGMA.powi(2) + (rng * BRG_SIGMA.to_radians()).powi(2)).sqrt();
            let thr = 3.5 * sigma + 0.012;
            if (p - pred).len() > thr {
                self.suspect += 1;
            } else {
                self.suspect = 0;
            }
            if self.suspect >= 3 {
                // Manöver erkannt: Filterfenster verkürzen
                self.window_start = t - 20.0;
                self.suspect = 0;
                self.maneuver_time = t;
            }
        }
        self.plots.push_back(Plot { t, p });
        self.last_plot = t;
        while self.plots.front().is_some_and(|q| q.t < t - FULL_TIME) {
            self.plots.pop_front();
        }
        self.estimate(t);
    }

    /// Lineare Regression über das (adaptive) Plotfenster.
    fn estimate(&mut self, now: f64) {
        let start = self.window_start.max(now - FULL_TIME);
        let pts: Vec<Plot> = self
            .plots
            .iter()
            .filter(|q| q.t >= start)
            .cloned()
            .collect();
        let n = pts.len();
        if n == 0 {
            return;
        }
        let span = pts[n - 1].t - pts[0].t;
        if n < 4 || span < 8.0 {
            // noch keine Geschwindigkeit: Mittelwert der Positionen
            let mut m = V2::ZERO;
            for q in &pts {
                m += q.p;
            }
            self.pos = m * (1.0 / n as f64);
            self.est_time = now;
            if !self.vel_valid {
                self.vel = V2::ZERO;
            }
            return;
        }
        let nf = n as f64;
        let tm = pts.iter().map(|q| q.t).sum::<f64>() / nf;
        let xm = pts.iter().map(|q| q.p.x).sum::<f64>() / nf;
        let ym = pts.iter().map(|q| q.p.y).sum::<f64>() / nf;
        let mut sxx = 0.0;
        let mut sx = 0.0;
        let mut sy = 0.0;
        for q in &pts {
            let dt = q.t - tm;
            sxx += dt * dt;
            sx += dt * (q.p.x - xm);
            sy += dt * (q.p.y - ym);
        }
        if sxx < 1e-9 {
            return;
        }
        let vx = sx / sxx;
        let vy = sy / sxx;
        self.pos = V2::new(xm + vx * (now - tm), ym + vy * (now - tm));
        self.vel = V2::new(vx * 3600.0, vy * 3600.0);
        self.est_time = now;
        self.vel_valid = true;
    }
}

pub struct Alarm {
    pub kind: AlarmKind,
    pub track: u32,
    pub acked: bool,
    pub time: f64,
}

impl Alarm {
    pub fn text(&self) -> String {
        match self.kind {
            AlarmKind::Cpa => tf!("CPA/TCPA  Ziel {}", "CPA/TCPA  target {}", self.track),
            AlarmKind::Lost => tf!("ZIEL VERLOREN  {}", "TARGET LOST  {}", self.track),
            AlarmKind::GuardZone => tf!("GUARD ZONE  Ziel {}", "GUARD ZONE  target {}", self.track),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Trial {
    pub active: bool,
    pub course: f64,
    pub speed: f64,
    pub delay_min: f64,
}

#[derive(Clone, Copy, Default)]
pub struct TrackInfo {
    pub brg: f64,
    pub rng: f64,
    pub cse: f64,
    pub spd: f64,
    pub cpa: f64,
    /// Minuten (INF = keine Relativbewegung)
    pub tcpa: f64,
    pub bcr: Option<f64>,
    pub bct: Option<f64>,
    pub valid: bool,
    pub rel_vel: V2,
    pub true_vel: V2,
    pub pos: V2,
}

#[derive(Clone, Copy)]
pub struct TrialInfo {
    pub cpa: f64,
    pub tcpa: f64,
    /// relative Position des Ziels am Ende der Verzögerung
    pub rel_d: V2,
    /// relative Geschwindigkeit nach dem Manöver
    pub rel_vel: V2,
    /// Zielposition (Grund) am Ende der Verzögerung
    pub pos_d: V2,
}

/// Momentaufnahme der Eigenschiff-Sensordaten.
#[derive(Clone, Copy)]
pub struct OwnRef {
    pub pos: V2,
    pub gvel: V2,
    pub wvel: V2,
    pub heading: f64,
}

impl OwnRef {
    pub fn from_world(w: &World) -> Self {
        Self {
            pos: w.own.pos,
            gvel: w.own.ground_vel(w.current()),
            wvel: w.own.water_vel(),
            heading: w.own.heading,
        }
    }
    pub fn current(&self) -> V2 {
        self.gvel - self.wvel
    }
}

fn compute_info(tr: &Track, o: &OwnRef, ref_vel: V2, now: f64) -> TrackInfo {
    let p = tr.display_pos(now);
    let rel = p - o.pos;
    let rel_vel = tr.vel - o.gvel;
    let true_vel = tr.vel - (o.gvel - ref_vel);
    let (cpa, t_h) = cpa_calc(rel, rel_vel);
    let d = V2::from_bearing(o.heading, 1.0);
    let denom = d.cross(rel_vel);
    let (mut bcr, mut bct) = (None, None);
    if denom.abs() > 1e-6 {
        let t = -d.cross(rel) / denom;
        if t > 0.0 && t * 60.0 < 120.0 {
            bcr = Some(d.dot(rel + rel_vel * t));
            bct = Some(t * 60.0);
        }
    }
    TrackInfo {
        brg: rel.bearing(),
        rng: rel.len(),
        cse: true_vel.bearing(),
        spd: true_vel.len(),
        cpa,
        tcpa: t_h * 60.0,
        bcr,
        bct,
        valid: tr.vel_valid && tr.state == TrackState::Tracking,
        rel_vel,
        true_vel,
        pos: p,
    }
}

pub struct Arpa {
    pub enabled: bool,
    pub tracks: Vec<Track>,
    next_num: u32,
    pub selected: Option<u32>,
    pub vector_mode: VectorMode,
    pub vector_min: f64,
    /// Intervall vergangener Positionen in s (0 = aus)
    pub pp_interval: f64,
    pub stab_ground: bool,
    pub cpa_limit: f64,
    pub tcpa_limit: f64,
    pub gz_on: bool,
    pub gz_inner: f64,
    pub gz_outer: f64,
    /// Polygon-Guard-Zone
    pub gzp_on: bool,
    /// Eckpunkte relativ zum Eigenschiff (nm). Schiffsfest: x = Steuerbord, y = voraus;
    /// sonst nordausgerichtet: x = Ost, y = Nord.
    pub gzp_points: Vec<V2>,
    pub gzp_ship_fixed: bool,
    gz_known: HashSet<u32>,
    pub trial: Trial,
    pub alarms: Vec<Alarm>,
    pub own_past: VecDeque<(f64, V2)>,
    last_own_past: f64,
    rng: Rng,
}

impl Arpa {
    pub fn new() -> Self {
        Self {
            enabled: true,
            tracks: Vec::new(),
            next_num: 1,
            selected: None,
            vector_mode: VectorMode::Relative,
            vector_min: 6.0,
            pp_interval: 60.0,
            stab_ground: true,
            cpa_limit: 1.0,
            tcpa_limit: 12.0,
            gz_on: false,
            gz_inner: 3.0,
            gz_outer: 4.0,
            gzp_on: false,
            gzp_points: Vec::new(),
            gzp_ship_fixed: true,
            gz_known: HashSet::new(),
            trial: Trial {
                active: false,
                course: 0.0,
                speed: 12.0,
                delay_min: 0.0,
            },
            alarms: Vec::new(),
            own_past: VecDeque::new(),
            last_own_past: f64::NEG_INFINITY,
            rng: Rng::new(99),
        }
    }

    pub fn reset(&mut self) {
        self.tracks.clear();
        self.alarms.clear();
        self.gz_known.clear();
        self.own_past.clear();
        self.last_own_past = f64::NEG_INFINITY;
        self.selected = None;
        self.next_num = 1;
        self.trial.active = false;
    }

    pub fn is_tracked(&self, target_id: u32) -> Option<u32> {
        self.tracks
            .iter()
            .find(|t| t.target_id == target_id)
            .map(|t| t.num)
    }

    pub fn track(&self, num: u32) -> Option<&Track> {
        self.tracks.iter().find(|t| t.num == num)
    }

    pub fn acquire(&mut self, target_id: u32, now: f64, init_pos: V2, auto: bool) -> Option<u32> {
        if self.tracks.len() >= MAX_TRACKS {
            return None;
        }
        let mut num = self.next_num;
        while self.tracks.iter().any(|t| t.num == num) {
            num = num % 99 + 1;
        }
        self.next_num = num % 99 + 1;
        self.tracks.push(Track {
            num,
            target_id,
            state: TrackState::Acquiring,
            auto,
            acq_time: now,
            last_plot: now,
            plots: VecDeque::new(),
            window_start: now,
            suspect: 0,
            maneuver_time: f64::NEG_INFINITY,
            pos: init_pos,
            vel: V2::ZERO,
            est_time: now,
            vel_valid: false,
            past: VecDeque::new(),
            last_past: now,
            dangerous: false,
            lost_pos: init_pos,
        });
        self.selected = Some(num);
        Some(num)
    }

    pub fn remove_track(&mut self, num: u32) {
        self.tracks.retain(|t| t.num != num);
        self.alarms.retain(|a| a.track != num);
        if self.selected == Some(num) {
            self.selected = None;
        }
    }

    pub fn clear_tracks(&mut self) {
        self.tracks.clear();
        self.alarms.clear();
        self.selected = None;
    }

    pub fn ack(&mut self, idx: usize) {
        if idx >= self.alarms.len() {
            return;
        }
        let (kind, num) = (self.alarms[idx].kind, self.alarms[idx].track);
        match kind {
            AlarmKind::Lost => self.remove_track(num),
            _ => self.alarms[idx].acked = true,
        }
    }

    pub fn ack_all(&mut self) {
        let lost: Vec<u32> = self
            .alarms
            .iter()
            .filter(|a| a.kind == AlarmKind::Lost)
            .map(|a| a.track)
            .collect();
        for n in lost {
            self.remove_track(n);
        }
        for a in &mut self.alarms {
            a.acked = true;
        }
    }

    pub fn has_unacked(&self) -> bool {
        self.alarms.iter().any(|a| !a.acked)
    }

    pub fn own_ref_vel(&self, o: &OwnRef) -> V2 {
        if self.stab_ground {
            o.gvel
        } else {
            o.wvel
        }
    }

    pub fn info(&self, tr: &Track, o: &OwnRef, now: f64) -> TrackInfo {
        compute_info(tr, o, self.own_ref_vel(o), now)
    }

    pub fn trial_ground_vel(&self, o: &OwnRef) -> V2 {
        V2::from_bearing(self.trial.course, self.trial.speed) + o.current()
    }

    pub fn trial_info(&self, tr: &Track, o: &OwnRef, now: f64) -> Option<TrialInfo> {
        if !tr.vel_valid || tr.state == TrackState::Lost {
            return None;
        }
        let dh = self.trial.delay_min.max(0.0) / 60.0;
        let p = tr.display_pos(now);
        let rel0 = p - o.pos;
        let v0 = tr.vel - o.gvel;
        // Phase 1: bis zum Manöver
        let (_, t1) = cpa_calc(rel0, v0);
        let t1c = if t1.is_finite() {
            t1.clamp(0.0, dh)
        } else {
            0.0
        };
        let d1 = (rel0 + v0 * t1c).len();
        // Phase 2: nach dem Manöver
        let rel_d = rel0 + v0 * dh;
        let v1 = tr.vel - self.trial_ground_vel(o);
        let (_, t2) = cpa_calc(rel_d, v1);
        let t2c = if t2.is_finite() { t2.max(0.0) } else { 0.0 };
        let d2 = (rel_d + v1 * t2c).len();
        let (cpa, tcpa) = if dh > 0.0 && d1 < d2 {
            (d1, t1c * 60.0)
        } else {
            (d2, (dh + t2c) * 60.0)
        };
        Some(TrialInfo {
            cpa,
            tcpa,
            rel_d,
            rel_vel: v1,
            pos_d: p + tr.vel * dh,
        })
    }

    /// Gespeicherten Polygonpunkt in eine relative Weltposition (nm) umrechnen.
    pub fn gzp_to_rel(&self, p: V2, heading: f64) -> V2 {
        if self.gzp_ship_fixed {
            V2::from_bearing(heading + 90.0, p.x) + V2::from_bearing(heading, p.y)
        } else {
            p
        }
    }

    /// Relative Weltposition in einen Polygonpunkt umrechnen.
    pub fn rel_to_gzp(&self, rel: V2, heading: f64) -> V2 {
        if self.gzp_ship_fixed {
            V2::new(
                rel.dot(V2::from_bearing(heading + 90.0, 1.0)),
                rel.dot(V2::from_bearing(heading, 1.0)),
            )
        } else {
            rel
        }
    }

    pub fn in_guard_zone(&self, rel: V2, heading: f64) -> bool {
        let r = rel.len();
        let ring = self.gz_on
            && r >= self.gz_inner.min(self.gz_outer)
            && r <= self.gz_outer.max(self.gz_inner);
        let poly = self.gzp_on
            && self.gzp_points.len() >= 3
            && point_in_poly(self.rel_to_gzp(rel, heading), &self.gzp_points);
        ring || poly
    }

    /// Beide Guard Zones ausschalten und löschen, zugehörige Alarme entfernen.
    pub fn clear_guard_zones(&mut self) {
        self.gz_on = false;
        self.gzp_on = false;
        self.gzp_points.clear();
        self.gz_known.clear();
        self.alarms.retain(|a| a.kind != AlarmKind::GuardZone);
    }

    pub fn is_dangerous(&self, cpa: f64, tcpa: f64) -> bool {
        tcpa.is_finite() && cpa <= self.cpa_limit && tcpa >= 0.0 && tcpa <= self.tcpa_limit
    }

    /// Neue Radardetektionen verarbeiten (Plots für Tracks, Guard Zone).
    pub fn process(&mut self, events: &[DetectionEvent]) {
        if !self.enabled {
            return;
        }
        for e in events {
            let mut tracked = false;
            for tr in self.tracks.iter_mut() {
                if tr.target_id != e.target_id {
                    continue;
                }
                tracked = true;
                if tr.state != TrackState::Lost && e.visible {
                    let rng = e.rel.len();
                    let brg = e.rel.bearing();
                    let m = V2::from_bearing(
                        brg + self.rng.gauss() * BRG_SIGMA,
                        rng + self.rng.gauss() * RNG_SIGMA,
                    );
                    tr.add_plot(e.time, e.own_pos + m, rng);
                }
            }
            if !tracked && e.visible && !self.gz_known.contains(&e.target_id) {
                if self.in_guard_zone(e.rel, e.own_heading) {
                    self.gz_known.insert(e.target_id);
                    if let Some(num) = self.acquire(e.target_id, e.time, e.own_pos + e.rel, true) {
                        self.alarms.push(Alarm {
                            kind: AlarmKind::GuardZone,
                            track: num,
                            acked: false,
                            time: e.time,
                        });
                    }
                }
            }
        }
    }

    /// Zustände, vergangene Positionen und Alarme aktualisieren.
    pub fn update(&mut self, world: &World) {
        let now = world.time;
        let o = OwnRef::from_world(world);
        for tr in self.tracks.iter_mut() {
            if tr.state != TrackState::Lost && now - tr.last_plot > LOST_TIME {
                tr.lost_pos = tr.display_pos(now);
                tr.state = TrackState::Lost;
                tr.dangerous = false;
                self.alarms
                    .retain(|a| !(a.track == tr.num && a.kind == AlarmKind::Cpa));
                self.alarms.push(Alarm {
                    kind: AlarmKind::Lost,
                    track: tr.num,
                    acked: false,
                    time: now,
                });
            }
            if tr.state == TrackState::Acquiring && tr.age(now) >= ACQ_TIME && tr.vel_valid {
                tr.state = TrackState::Tracking;
            }
            if tr.state != TrackState::Lost
                && self.pp_interval > 0.0
                && now - tr.last_past >= self.pp_interval
            {
                tr.past.push_back((now, tr.display_pos(now), o.pos));
                while tr.past.len() > 10 {
                    tr.past.pop_front();
                }
                tr.last_past = now;
            }
        }
        if self.pp_interval > 0.0 && now - self.last_own_past >= self.pp_interval {
            self.own_past.push_back((now, o.pos));
            while self.own_past.len() > 10 {
                self.own_past.pop_front();
            }
            self.last_own_past = now;
        }

        let ref_vel = self.own_ref_vel(&o);
        let infos: Vec<TrackInfo> = self
            .tracks
            .iter()
            .map(|t| compute_info(t, &o, ref_vel, now))
            .collect();
        for (tr, inf) in self.tracks.iter_mut().zip(infos.iter()) {
            if tr.state != TrackState::Tracking {
                continue;
            }
            let danger = inf.valid
                && inf.tcpa.is_finite()
                && inf.cpa <= self.cpa_limit
                && inf.tcpa >= 0.0
                && inf.tcpa <= self.tcpa_limit;
            if danger
                && !tr.dangerous
                && !self
                    .alarms
                    .iter()
                    .any(|a| a.track == tr.num && a.kind == AlarmKind::Cpa)
            {
                self.alarms.push(Alarm {
                    kind: AlarmKind::Cpa,
                    track: tr.num,
                    acked: false,
                    time: now,
                });
            }
            if !danger {
                self.alarms
                    .retain(|a| !(a.track == tr.num && a.kind == AlarmKind::Cpa));
            }
            tr.dangerous = danger;
        }
    }
}
