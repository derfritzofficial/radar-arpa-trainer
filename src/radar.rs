//! Radarvideo-Simulation: rotierende Antenne, Echos, Seegangs-/Regenclutter,
//! Rauschen, STC/FTC/Gain, Landabschattung, Nachleuchten und Echospuren.

use std::collections::{HashMap, VecDeque};

use eframe::egui::{Color32, ColorImage};

use crate::sim::World;
use crate::util::*;

pub const N_BRG: usize = 1024;
pub const BRG_MASK: usize = N_BRG - 1;
pub const N_BIN: usize = 512;
pub const BRG_STEP: f64 = 360.0 / N_BRG as f64;
/// Videopuffer reicht über den Bereich hinaus (für Off-Center / True Motion).
pub const BUF_FACTOR: f64 = 1.7;
/// Antennenumlauf in s (24 U/min)
pub const ANT_PERIOD: f64 = 2.5;
const THRESH: f32 = 0.45;
const LAND_DET: f64 = 20.0;
/// Maximale ARPA-Verfolgungsentfernung, unabhängig vom eingestellten Bereich (nm)
pub const ARPA_MAX_RANGE: f64 = 32.0;

pub const RANGES: [f64; 8] = [0.25, 0.5, 0.75, 1.5, 3.0, 6.0, 12.0, 24.0];
pub const RING_INT: [f64; 8] = [0.05, 0.1, 0.25, 0.25, 0.5, 1.0, 2.0, 4.0];

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Pulse {
    Short,
    Medium,
    Long,
}

impl Pulse {
    pub fn label(self) -> &'static str {
        match self {
            Pulse::Short => "SP",
            Pulse::Medium => "MP",
            Pulse::Long => "LP",
        }
    }
    /// (Pulslänge in nm, Energiefaktor)
    fn params(self) -> (f64, f32) {
        match self {
            Pulse::Short => (0.008, 0.75),
            Pulse::Medium => (0.04, 1.0),
            Pulse::Long => (0.12, 1.3),
        }
    }
}

/// Frequenzband des Radars.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Band {
    /// 9,4 GHz, λ ≈ 3 cm: schmale Keule, gute Auflösung, empfindlich für kleine Ziele,
    /// aber starker See- und Regenclutter sowie Dämpfung im Regen.
    X,
    /// 3 GHz, λ ≈ 10 cm: breitere Keule, deutlich weniger Clutter und Regendämpfung,
    /// größere Reichweite bei schlechtem Wetter, kleine Ziele schwächer.
    S,
}

pub struct BandParams {
    /// horizontale Keulenbreite (Grad)
    pub beam_w: f64,
    pub sea_clutter: f32,
    pub rain_echo: f32,
    pub rain_atten: f32,
    /// Reichweitenfaktor kleiner Ziele (schwacher Radarquerschnitt)
    pub small_target: f64,
    /// Reichweitenfaktor großer Ziele und Land
    pub large_target: f64,
}

impl Band {
    pub fn label(self) -> &'static str {
        match self {
            Band::X => "X-Band",
            Band::S => "S-Band",
        }
    }
    pub fn params(self) -> BandParams {
        match self {
            Band::X => BandParams {
                beam_w: 1.2,
                sea_clutter: 1.0,
                rain_echo: 1.0,
                rain_atten: 0.6,
                small_target: 1.0,
                large_target: 1.0,
            },
            Band::S => BandParams {
                beam_w: 1.9,
                sea_clutter: 0.35,
                rain_echo: 0.07,
                rain_atten: 0.03,
                small_target: 0.8,
                large_target: 1.1,
            },
        }
    }
}

pub struct DetectionEvent {
    pub target_id: u32,
    pub time: f64,
    pub visible: bool,
    pub rel: V2,
    pub own_pos: V2,
    pub own_heading: f64,
}

#[derive(Clone, Copy)]
pub struct TrailPt {
    pub t: f64,
    pub world: V2,
    pub rel: V2,
}

pub struct Radar {
    pub transmit: bool,
    pub range_idx: usize,
    pub gain: f64,
    pub sea: f64,
    pub rain: f64,
    pub pulse_sel: Option<Pulse>,
    /// Interferenzunterdrückung
    pub ir: bool,
    band: Band,
    pub antenna: f64,
    pub cells: Vec<f32>,
    pub sweep_time: Vec<f64>,
    buffer_range: f64,
    pub events: Vec<DetectionEvent>,
    pub last_detect: HashMap<u32, (f64, bool)>,
    pub trails: HashMap<u32, VecDeque<TrailPt>>,
    rng: Rng,
}

impl Radar {
    pub fn new() -> Self {
        let range_idx = 6; // 12 NM
        Self {
            transmit: true,
            range_idx,
            gain: 55.0,
            sea: 35.0,
            rain: 0.0,
            pulse_sel: None,
            ir: true,
            band: Band::X,
            antenna: 0.0,
            cells: vec![0.0; N_BRG * N_BIN],
            sweep_time: vec![f64::NEG_INFINITY; N_BRG],
            buffer_range: RANGES[range_idx] * BUF_FACTOR,
            events: Vec::new(),
            last_detect: HashMap::new(),
            trails: HashMap::new(),
            rng: Rng::new(1234),
        }
    }

    pub fn range(&self) -> f64 {
        RANGES[self.range_idx]
    }

    pub fn ring_interval(&self) -> f64 {
        RING_INT[self.range_idx]
    }

    pub fn pulse(&self) -> Pulse {
        self.pulse_sel.unwrap_or(if self.range() <= 1.5 {
            Pulse::Short
        } else if self.range() <= 6.0 {
            Pulse::Medium
        } else {
            Pulse::Long
        })
    }

    pub fn set_range(&mut self, idx: usize) {
        let idx = idx.min(RANGES.len() - 1);
        if idx != self.range_idx {
            self.range_idx = idx;
            self.buffer_range = RANGES[idx] * BUF_FACTOR;
            self.clear_video();
        }
    }

    pub fn band(&self) -> Band {
        self.band
    }

    /// Umschalten auf den anderen Sender/Empfänger: Bild wird neu aufgebaut.
    pub fn set_band(&mut self, b: Band) {
        if b != self.band {
            self.band = b;
            self.clear_video();
        }
    }

    pub fn set_transmit(&mut self, on: bool) {
        self.transmit = on;
        if !on {
            self.clear_video();
            self.last_detect.clear();
        }
    }

    pub fn clear_video(&mut self) {
        self.cells.iter_mut().for_each(|c| *c = 0.0);
        self.sweep_time
            .iter_mut()
            .for_each(|t| *t = f64::NEG_INFINITY);
    }

    pub fn clear_trails(&mut self) {
        self.trails.clear();
    }

    /// Antenne um dt Sekunden weiterdrehen und alle überstrichenen Peilstrahlen berechnen.
    pub fn update(&mut self, dt: f64, world: &World) {
        if !self.transmit {
            return;
        }
        let start = self.antenna;
        let end = start + 360.0 / ANT_PERIOD * dt;
        let a = (start / BRG_STEP).floor() as i64 + 1;
        let e = (end / BRG_STEP).floor() as i64;
        for k in a..=e {
            self.paint_ray(k.rem_euclid(N_BRG as i64) as usize, world);
        }
        self.antenna = norm360(end);

        // alte Spurpunkte verwerfen
        let now = world.time;
        for tr in self.trails.values_mut() {
            while tr.front().is_some_and(|p| now - p.t > 31.0 * 60.0) {
                tr.pop_front();
            }
        }
        self.trails.retain(|_, v| !v.is_empty());
    }

    fn paint_ray(&mut self, idx: usize, world: &World) {
        let theta = idx as f64 * BRG_STEP;
        let dir = V2::from_bearing(theta, 1.0);
        let own = world.own.pos;
        let br = self.buffer_range;
        let bin_nm = br / N_BIN as f64;
        let (pulse_nm, pulse_gain) = self.pulse().params();
        let bp = self.band.params();
        let beam_w = bp.beam_w;

        let mut sig = [0f32; N_BIN];
        let mut rain_d = [0f32; N_BIN];

        // --- Regen: Dichte entlang des Strahls
        for cell in &world.rain {
            let c = cell.center - own;
            let proj = c.dot(dir);
            let perp2 = c.len2() - proj * proj;
            let r2 = cell.radius * cell.radius;
            if perp2 >= r2 {
                continue;
            }
            let half = (r2 - perp2).sqrt();
            let t0 = (proj - half).max(0.0);
            let t1 = (proj + half).min(br);
            if t1 <= t0 {
                continue;
            }
            let i0 = (t0 / bin_nm) as usize;
            let i1 = ((t1 / bin_nm) as usize).min(N_BIN - 1);
            for (i, rd) in rain_d.iter_mut().enumerate().take(i1 + 1).skip(i0) {
                let r = (i as f64 + 0.5) * bin_nm;
                let d2 = perp2 + (r - proj) * (r - proj);
                let f = 1.0 - d2 / r2;
                if f > 0.0 {
                    *rd += (cell.intensity * f) as f32;
                }
            }
        }
        // Dämpfung durch Regen
        let mut att = [1f32; N_BIN];
        let mut acc = 0f32;
        for i in 0..N_BIN {
            att[i] = (-acc).exp();
            acc += rain_d[i] * bin_nm as f32 * bp.rain_atten;
        }

        // --- Land (mit Abschattung dahinter)
        let mut shadow = f64::INFINITY;
        for poly in &world.land {
            for (t0, t1) in ray_intervals(own, dir, poly) {
                shadow = shadow.min(t0);
                if t0 >= br {
                    continue;
                }
                let i0 = (t0 / bin_nm) as usize;
                let i1 = ((t1.min(br)) / bin_nm) as usize;
                for i in i0..i1.min(N_BIN) {
                    let r = (i as f64 + 0.5) * bin_nm;
                    let depth = r - t0;
                    let p = own + dir * r;
                    let tex = value_noise(p, 0.12) * 0.7 + value_noise(p, 0.5) * 0.3;
                    let coast = if depth < pulse_nm.max(bin_nm * 2.0) + 0.04 {
                        2.0
                    } else {
                        1.0
                    };
                    let range_f =
                        ((LAND_DET * bp.large_target / r.max(0.05)).powi(2) * 0.5).min(40.0);
                    let a = range_f * (0.15 + 1.3 * tex * tex) * coast * (-depth / 2.5).exp();
                    sig[i] += a as f32 * pulse_gain * att[i];
                }
            }
        }

        // --- Ziele
        let mut pending: Vec<(u32, usize, usize, V2)> = Vec::new();
        for t in &world.targets {
            let rel = t.pos - own;
            let rng = rel.len();
            if rng < 0.005 {
                continue;
            }
            // Ziele jenseits des angezeigten Videobereichs werden von ARPA weiter verfolgt
            let beyond = rng >= br;
            let brg = rel.bearing();
            let dth = norm180(brg - theta).abs();
            let is_center = dth <= BRG_STEP * 0.5;
            if dth > 25.0 && !is_center {
                continue;
            }
            let spec = t.kind.spec();
            let rel_ang = (t.heading - brg).to_radians();
            let len = spec.length_m / 1852.0;
            let beam = (len * 0.18).max(0.002);
            let across = len * rel_ang.sin().abs() + beam * rel_ang.cos().abs();
            let along = len * rel_ang.cos().abs() + beam * rel_ang.sin().abs();
            let half_ang = beam_w / 2.0 + (across / 2.0 / rng).atan().to_degrees();
            let shadowed = rng > shadow + 0.02;
            // det_range = Entfernung, bei der das Echo bei Normal-Gain gerade sicher sichtbar ist
            let band_f = if spec.strength < 0.6 {
                bp.small_target
            } else {
                bp.large_target
            };
            let det = spec.det_range * band_f;
            let mut raw = (0.9 * (det / rng).powi(2)) as f32 * pulse_gain;
            raw = raw.min(60.0);
            if rng > det * 1.5 {
                raw = 0.0; // jenseits des Radarhorizonts
            }
            if beyond {
                if is_center && rng <= ARPA_MAX_RANGE {
                    let g = 10f64.powf((self.gain - 50.0) / 50.0) as f32;
                    let visible = !shadowed && raw * att[N_BIN - 1] * g > THRESH * 1.3;
                    self.events.push(DetectionEvent {
                        target_id: t.id,
                        time: world.time,
                        visible,
                        rel,
                        own_pos: own,
                        own_heading: world.own.heading,
                    });
                    self.last_detect.insert(t.id, (world.time, visible));
                }
                continue;
            }
            // am Pufferrand begrenzen, damit i0 < i1 <= N_BIN - 1 gilt
            let i0 = ((((rng - along / 2.0) / bin_nm).max(0.0)) as usize).min(N_BIN - 2);
            let i1 = ((((rng + along / 2.0 + pulse_nm) / bin_nm).ceil()) as usize)
                .clamp(i0 + 1, N_BIN - 1);
            if !shadowed && raw > 0.0 {
                if dth <= half_ang {
                    let edge = ((dth - (half_ang - beam_w / 2.0)).max(0.0) / (beam_w / 2.0))
                        .min(1.0) as f32;
                    // kleine Ziele fluktuieren stärker (Swerling)
                    let fl = (1.0 - spec.strength as f32).clamp(0.0, 1.0);
                    let amp = raw
                        * (1.0 - 0.75 * edge * edge)
                        * (0.75 - 0.35 * fl + (0.5 + 0.7 * fl) * self.rng.f32());
                    for i in i0..=i1 {
                        sig[i] += amp * att[i];
                    }
                } else if raw > 6.0 && dth < 18.0 {
                    // Nebenkeulen starker Nahziele
                    let amp = raw * 0.012 * (1.0 - dth as f32 / 18.0);
                    for i in i0..=i1 {
                        sig[i] += amp * att[i];
                    }
                }
            }
            if is_center {
                pending.push((t.id, i0.saturating_sub(1), (i1 + 1).min(N_BIN - 1), rel));
            }
        }

        // --- Seegangsclutter (luvseitig stärker)
        const SEA_AMP: [f32; 7] = [0.0, 0.5, 1.0, 1.8, 2.8, 4.0, 5.5];
        let st = world.sea_state.min(6) as usize;
        if st > 0 {
            let r0 = 0.25 + 0.15 * st as f64;
            let wind = 1.0 + 0.6 * (theta - world.wind_from).to_radians().cos();
            let amp0 = SEA_AMP[st] * wind as f32 * pulse_gain * bp.sea_clutter;
            for i in 0..N_BIN {
                let r = (i as f64 + 0.5) * bin_nm;
                if r > r0 * 8.0 {
                    break;
                }
                let c = amp0 * (-r / r0).exp() as f32 * self.rng.expo();
                sig[i] += c * att[i];
            }
        }

        // --- Regenechos (Speckle)
        for i in 0..N_BIN {
            if rain_d[i] > 0.0 {
                sig[i] += rain_d[i] * 1.4 * bp.rain_echo * self.rng.expo() * att[i];
            }
        }

        // --- Fremdradar-Interferenz
        if !self.ir && self.rng.f64() < 0.025 {
            let step = 3 + (self.rng.f64() * 6.0) as usize;
            let mut i = (self.rng.f64() * 40.0) as usize;
            while i < N_BIN {
                sig[i] += 2.5;
                i += step;
            }
        }

        // --- Empfänger: Rauschen, STC, Gain
        let g = 10f64.powf((self.gain - 50.0) / 50.0) as f32;
        let sea_k = (self.sea / 100.0) as f32;
        let mut out = [0f32; N_BIN];
        for i in 0..N_BIN {
            let r = (i as f64 + 0.5) * bin_nm;
            let noise = 0.08 * self.rng.expo();
            let stc = 1.0 / (1.0 + sea_k * 14.0 * (-r / 1.3).exp() as f32);
            out[i] = (sig[i] + noise) * stc * g;
        }

        // --- FTC (Differenzierer gegen flächige Echos)
        let rain_k = (self.rain / 100.0) as f32 * 1.1;
        if rain_k > 0.0 {
            let w = ((0.08 / bin_nm) as usize).clamp(3, 40);
            let mut run = 0f32;
            let mut ftc = [0f32; N_BIN];
            for i in 0..N_BIN {
                let mean = run / w as f32;
                ftc[i] = (out[i] - rain_k * mean).max(0.0);
                run += out[i];
                if i >= w {
                    run -= out[i - w];
                }
            }
            out = ftc;
        }

        // --- Schwelle & Speicherung
        let base = idx * N_BIN;
        for i in 0..N_BIN {
            let v = out[i];
            self.cells[base + i] = if v > THRESH {
                1.0 - (-(v - THRESH) * 1.6).exp()
            } else {
                0.0
            };
        }
        self.sweep_time[idx] = world.time;

        // --- Detektionen für ARPA / Echospuren
        for (id, i0, i1, rel) in pending {
            let peak = self.cells[base + i0..=base + i1]
                .iter()
                .cloned()
                .fold(0.0f32, f32::max);
            let visible = peak > 0.12;
            self.events.push(DetectionEvent {
                target_id: id,
                time: world.time,
                visible,
                rel,
                own_pos: own,
                own_heading: world.own.heading,
            });
            self.last_detect.insert(id, (world.time, visible));
            if visible {
                let tr = self.trails.entry(id).or_default();
                if tr.back().is_none_or(|p| world.time - p.t >= 4.0) {
                    tr.push_back(TrailPt {
                        t: world.time,
                        world: own + rel,
                        rel,
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Darstellung: polarer Videopuffer → kartesisches Bild
// ---------------------------------------------------------------------------

pub const IMG: usize = 800;
const OUTSIDE: u16 = u16::MAX;
const BEYOND: u16 = u16::MAX - 1;

pub struct PpiRenderer {
    lut_ang: Vec<u16>,
    lut_bin: Vec<u16>,
    off: (i32, i32),
    built: bool,
}

impl PpiRenderer {
    pub fn new() -> Self {
        Self {
            lut_ang: vec![0; IMG * IMG],
            lut_bin: vec![OUTSIDE; IMG * IMG],
            off: (0, 0),
            built: false,
        }
    }

    /// Nachschlagetabelle Pixel → (Bildschirmwinkel-Index, Entfernungszelle).
    /// `off` = Eigenschiffposition in Bildpixeln relativ zur Bildmitte (y nach unten).
    fn build(&mut self, off: (i32, i32)) {
        let half = IMG as f32 / 2.0;
        for py in 0..IMG {
            for px in 0..IMG {
                let i = py * IMG + px;
                let x = px as f32 + 0.5 - half;
                let y = half - (py as f32 + 0.5);
                if x * x + y * y > half * half {
                    self.lut_bin[i] = OUTSIDE;
                    continue;
                }
                let dx = x - off.0 as f32;
                let dy = y + off.1 as f32;
                let ang = dx.atan2(dy).to_degrees();
                let a =
                    ((ang / 360.0 * N_BRG as f32).round() as i32).rem_euclid(N_BRG as i32) as u16;
                let r = (dx * dx + dy * dy).sqrt();
                let bin = r / half / BUF_FACTOR as f32 * N_BIN as f32;
                self.lut_ang[i] = a;
                self.lut_bin[i] = if bin >= N_BIN as f32 {
                    BEYOND
                } else {
                    bin as u16
                };
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        radar: &Radar,
        off_px: (f32, f32),
        rot: f64,
        now: f64,
        palette: &[Color32; 256],
        tau: f64,
        img: &mut ColorImage,
    ) {
        let off = (
            ((off_px.0 / 2.0).round() * 2.0) as i32,
            ((off_px.1 / 2.0).round() * 2.0) as i32,
        );
        if !self.built || off != self.off {
            self.build(off);
            self.off = off;
            self.built = true;
        }
        let rot_idx = ((rot / BRG_STEP).round() as i64).rem_euclid(N_BRG as i64) as usize;
        let mut decay = [0f32; N_BRG];
        for (b, d) in decay.iter_mut().enumerate() {
            let st = radar.sweep_time[b];
            *d = if st.is_finite() {
                (-((now - st).max(0.0) / tau)).exp() as f32
            } else {
                0.0
            };
        }
        let bg = palette[0];
        for (i, px) in img.pixels.iter_mut().enumerate() {
            let bin = self.lut_bin[i];
            *px = match bin {
                OUTSIDE => Color32::TRANSPARENT,
                BEYOND => bg,
                _ => {
                    let b = (self.lut_ang[i] as usize + rot_idx) & BRG_MASK;
                    let v = radar.cells[b * N_BIN + bin as usize] * decay[b];
                    palette[((v * 255.0) as usize).min(255)]
                }
            };
        }
    }
}
