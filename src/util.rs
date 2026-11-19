//! Kleine Mathematik-Helfer: 2D-Vektoren (Seemeilen), Winkel, Zufall, Rauschen.
//!
//! Koordinatensystem der Welt: x = Ost, y = Nord, Einheit Seemeilen (nm).
//! Peilungen in Grad rechtweisend (0° = Nord, im Uhrzeigersinn).

use std::ops::{Add, AddAssign, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V2 {
    pub x: f64,
    pub y: f64,
}

impl V2 {
    pub const ZERO: V2 = V2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Vektor aus Peilung (Grad) und Länge.
    pub fn from_bearing(brg_deg: f64, len: f64) -> Self {
        let r = brg_deg.to_radians();
        Self::new(len * r.sin(), len * r.cos())
    }

    pub fn len(self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn len2(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn dot(self, o: V2) -> f64 {
        self.x * o.x + self.y * o.y
    }

    pub fn cross(self, o: V2) -> f64 {
        self.x * o.y - self.y * o.x
    }

    /// Richtung des Vektors als Peilung 0..360°.
    pub fn bearing(self) -> f64 {
        norm360(self.x.atan2(self.y).to_degrees())
    }

    pub fn normalized(self) -> V2 {
        let l = self.len();
        if l > 1e-12 {
            self * (1.0 / l)
        } else {
            V2::ZERO
        }
    }
}

impl Add for V2 {
    type Output = V2;
    fn add(self, o: V2) -> V2 {
        V2::new(self.x + o.x, self.y + o.y)
    }
}
impl Sub for V2 {
    type Output = V2;
    fn sub(self, o: V2) -> V2 {
        V2::new(self.x - o.x, self.y - o.y)
    }
}
impl Mul<f64> for V2 {
    type Output = V2;
    fn mul(self, k: f64) -> V2 {
        V2::new(self.x * k, self.y * k)
    }
}
impl Neg for V2 {
    type Output = V2;
    fn neg(self) -> V2 {
        V2::new(-self.x, -self.y)
    }
}
impl AddAssign for V2 {
    fn add_assign(&mut self, o: V2) {
        self.x += o.x;
        self.y += o.y;
    }
}

pub fn norm360(a: f64) -> f64 {
    let r = a.rem_euclid(360.0);
    if r >= 360.0 {
        0.0
    } else {
        r
    }
}

pub fn norm180(a: f64) -> f64 {
    norm360(a + 180.0) - 180.0
}

/// Schneller Pseudozufallsgenerator (xorshift64*), reproduzierbar.
#[derive(Clone)]
pub struct Rng {
    s: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self {
            s: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.s;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.s = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Gleichverteilt in [0, 1).
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn f32(&mut self) -> f32 {
        self.f64() as f32
    }

    pub fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.f64()
    }

    /// Exponentialverteilt mit Mittelwert 1 (Rayleigh-Leistung → Speckle).
    pub fn expo(&mut self) -> f32 {
        -((1.0 - self.f64()).max(1e-12).ln()) as f32
    }

    /// Standardnormalverteilt.
    pub fn gauss(&mut self) -> f64 {
        let u1 = self.f64().max(1e-12);
        let u2 = self.f64();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

fn hash2(x: i64, y: i64) -> f64 {
    let mut h = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^= h >> 33;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// Glattes, ortsfestes Wertrauschen in [0,1) (für Landtextur).
pub fn value_noise(p: V2, cell: f64) -> f64 {
    let gx = p.x / cell;
    let gy = p.y / cell;
    let x0 = gx.floor();
    let y0 = gy.floor();
    let fx = gx - x0;
    let fy = gy - y0;
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);
    let (ix, iy) = (x0 as i64, y0 as i64);
    let a = hash2(ix, iy);
    let b = hash2(ix + 1, iy);
    let c = hash2(ix, iy + 1);
    let d = hash2(ix + 1, iy + 1);
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

pub fn fmt_hms(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

/// Schnittintervalle eines Strahls (Ursprung, Einheitsrichtung) mit einem Polygon.
/// Rückgabe: Liste (t_ein, t_aus) in nm entlang des Strahls.
pub fn ray_intervals(origin: V2, dir: V2, poly: &[V2]) -> Vec<(f64, f64)> {
    let mut ts: Vec<f64> = Vec::new();
    let n = poly.len();
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        let e = b - a;
        let denom = dir.cross(e);
        if denom.abs() < 1e-12 {
            continue;
        }
        let w = a - origin;
        let t = w.cross(e) / denom;
        let s = w.cross(dir) / denom;
        if (0.0..1.0).contains(&s) && t > 0.0 {
            ts.push(t);
        }
    }
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = Vec::new();
    let mut i = 0;
    if ts.len() % 2 == 1 {
        out.push((0.0, ts[0]));
        i = 1;
    }
    while i + 1 < ts.len() {
        out.push((ts[i], ts[i + 1]));
        i += 2;
    }
    out
}

/// CPA-Berechnung: relative Position r (nm), relative Geschwindigkeit v (kn).
/// Rückgabe (CPA in nm, TCPA in Stunden; INF wenn keine Relativbewegung).
pub fn cpa_calc(r: V2, v: V2) -> (f64, f64) {
    let v2 = v.len2();
    if v2 < 1e-6 {
        return (r.len(), f64::INFINITY);
    }
    let t = -r.dot(v) / v2;
    ((r + v * t).len(), t)
}

/// Punkt-in-Polygon (gerade/ungerade Regel).
pub fn point_in_poly(p: V2, poly: &[V2]) -> bool {
    let n = poly.len();
    let mut inside = false;
    let mut j = n.wrapping_sub(1);
    for i in 0..n {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        j = i;
    }
    inside
}
