#!/usr/bin/env python3
"""Erzeugt das App-Icon (Radarschirm mit Antennenstrahl und ARPA-Zielen).

macOS 26 (Tahoe) steckt App-Icons, deren Umriss nicht der Systemform entspricht
(z. B. Pixel außerhalb der Kachel wie ein Schlagschatten), verkleinert in einen
grauen Rahmen. Die Kachel folgt deshalb genau Apples Icon-Raster:
824 × 824 px auf einer 1024er-Fläche (100 px Rand), kontinuierliche Ecken
(„Squircle“, r = 185,4 px), innen vollständig deckend, außen vollständig
transparent – ohne Schatten. Den Schatten zeichnet macOS selbst.

Ausgabe in assets/:
  icon_1024.png  – Master
  icon_256.png   – wird ins Programm eingebettet (Fenster, Dock, Taskleiste)
  icon.ico       – Windows (.exe)
  icon.icns      – macOS (.app), nur wenn `iconutil` vorhanden ist

Benötigt Pillow:  python3 -m pip install pillow
"""

import math
import os
import shutil
import subprocess
import tempfile

from PIL import Image, ImageDraw, ImageFilter

OUT = 1024  # Kantenlänge des Masters
S = 2048  # Arbeitsauflösung der Grafik (2× Supersampling)
MS = 4096  # Arbeitsauflösung der Maske (4× Supersampling für weiche, exakte Kanten)
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")

# Apples Raster für macOS-App-Icons (bezogen auf 1024 px)
TILE = 824.0
RADIUS = 185.4


def lerp(a, b, t):
    return tuple(int(round(a[i] + (b[i] - a[i]) * t)) for i in range(len(a)))


def polar(cx, cy, r, deg):
    a = math.radians(deg)
    return cx + r * math.sin(a), cy - r * math.cos(a)


# Eine Ecke des Rechtecks mit kontinuierlicher Krümmung (wie UIKit/AppKit „continuous corners“):
# drei kubische Bézierkurven, Koordinaten in Vielfachen von r, gemessen vom Eckpunkt nach innen
# (a entlang der ersten Kante, b entlang der zweiten).
_CORNER = [
    ((1.52866471, 0.0), (1.08849323, 0.0), (0.86840689, 0.0), (0.66993427, 0.06549600)),
    ((0.66993427, 0.06549600), (0.37282392, 0.16905899), (0.16906013, 0.37282401), (0.06549569, 0.66993493)),
    ((0.06549569, 0.66993493), (0.0, 0.86840701), (0.0, 1.08849299), (0.0, 1.52866483)),
]


def _bezier(p0, p1, p2, p3, t):
    u = 1.0 - t
    return tuple(u * u * u * p0[i] + 3 * u * u * t * p1[i] + 3 * u * t * t * p2[i] + t * t * t * p3[i] for i in range(2))


def squircle(x0, y0, x1, y1, r, steps=96):
    """Polygon eines Rechtecks mit kontinuierlichen Ecken (im Uhrzeigersinn)."""
    r = min(r, min(x1 - x0, y1 - y0) / 2 / 1.52866483)
    corners = [
        lambda a, b: (x1 - a * r, y0 + b * r),  # oben rechts
        lambda a, b: (x1 - b * r, y1 - a * r),  # unten rechts
        lambda a, b: (x0 + a * r, y1 - b * r),  # unten links
        lambda a, b: (x0 + b * r, y0 + a * r),  # oben links
    ]
    pts = []
    for to_xy in corners:
        for seg in _CORNER:
            for k in range(steps):
                pts.append(to_xy(*_bezier(*seg, k / steps)))
        pts.append(to_xy(*_CORNER[-1][3]))
    return pts


def rounded_rect(x0, y0, x1, y1, r, steps=96):
    """Polygon eines gewöhnlichen Rechtecks mit Kreisbogen-Ecken (nur zum Vergleich)."""
    pts = []
    for cx, cy, a0 in ((x1 - r, y0 + r, -90), (x1 - r, y1 - r, 0), (x0 + r, y1 - r, 90), (x0 + r, y0 + r, 180)):
        for k in range(steps + 1):
            a = math.radians(a0 + 90 * k / steps)
            pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def superellipse(x0, y0, x1, y1, n=5.0, steps=1440):
    """Polygon einer Superellipse |x|^n + |y|^n = 1 (nur zum Vergleich)."""
    cx, cy, ax, ay = (x0 + x1) / 2, (y0 + y1) / 2, (x1 - x0) / 2, (y1 - y0) / 2
    pts = []
    for k in range(steps):
        t = 2 * math.pi * k / steps
        c, s = math.cos(t), math.sin(t)
        pts.append((cx + ax * math.copysign(abs(c) ** (2 / n), c), cy + ay * math.copysign(abs(s) ** (2 / n), s)))
    return pts


def tile_polygon(shape, scale, inset=0.0):
    """Umriss der Kachel in Pixeln einer Fläche von OUT·scale (inset in 1024er-Pixeln, negativ = größer)."""
    m = (OUT - TILE) / 2 + inset
    x0, y0, x1, y1 = m * scale, m * scale, (OUT - m) * scale, (OUT - m) * scale
    if shape == "continuous":
        return squircle(x0, y0, x1, y1, RADIUS * scale)
    if shape == "rounded":
        return rounded_rect(x0, y0, x1, y1, RADIUS * scale)
    if shape == "superellipse":
        return superellipse(x0, y0, x1, y1)
    if shape == "full":
        return [(0, 0), (OUT * scale, 0), (OUT * scale, OUT * scale), (0, OUT * scale)]
    raise ValueError(shape)


def tile_mask(shape="continuous", inset=0.0):
    """Deckungsmaske der Kachel in OUT × OUT, kantenglatt per Flächenmittel (keine Überschwinger)."""
    m = Image.new("L", (MS, MS), 0)
    ImageDraw.Draw(m).polygon(tile_polygon(shape, MS / OUT, inset), fill=255)
    return m.resize((OUT, OUT), Image.BOX)


def make_art(shape="continuous", inset=0.0):
    """Farbige Grafik (deckend, ganze Fläche) in Arbeitsauflösung S."""
    img = Image.new("RGBA", (S, S))
    td = ImageDraw.Draw(img)
    # Kachel mit Verlauf (oben heller) – über die ganze Fläche, die Form kommt aus der Maske
    top, bottom = (28, 78, 150), (8, 26, 62)
    tile = TILE / OUT * S
    m = (S - tile) / 2
    for y in range(S):
        t = min(max((y - m) / tile, 0.0), 1.0)
        td.line([(0, y), (S, y)], fill=lerp(top, bottom, t) + (255,))

    cx, cy = S / 2, S / 2 + S * 0.005
    R = tile * 0.40

    # Schirm: radialer Verlauf
    scope = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    sd = ImageDraw.Draw(scope)
    steps = 120
    for i in range(steps, 0, -1):
        t = i / steps
        col = lerp((24, 92, 170), (6, 30, 72), t ** 1.3)
        r = R * t
        sd.ellipse((cx - r, cy - r, cx + r, cy + r), fill=col + (255,))
    img.alpha_composite(scope)

    lw = max(2, int(S * 0.004))

    # Entfernungsringe und Peillinien (eigene Ebene, damit die Transparenz überblendet)
    grid = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    gdr = ImageDraw.Draw(grid)
    for k in (1, 2, 3):
        r = R * k / 4
        gdr.ellipse((cx - r, cy - r, cx + r, cy + r), outline=(130, 200, 240, 95), width=lw)
    for deg in range(0, 360, 30):
        x, y = polar(cx, cy, R, deg)
        gdr.line([(cx, cy), (x, y)], fill=(130, 200, 240, 40), width=max(1, int(lw * 0.7)))
    img.alpha_composite(grid)

    # Antennenstrahl: Nachleuchtfächer (gelb, nach hinten ausblendend)
    sweep = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    wd = ImageDraw.Draw(sweep)
    head = 62.0  # Strahlrichtung in Grad
    span = 90
    for i in range(span):
        a0 = head - span + i
        alpha = int(150 * (i / span) ** 2.6)
        # PIL: 0° = 3 Uhr, im Uhrzeigersinn → Peilung - 90
        wd.pieslice((cx - R, cy - R, cx + R, cy + R), a0 - 90, a0 - 89 + 0.6, fill=(255, 225, 70, alpha))
    sweep = sweep.filter(ImageFilter.GaussianBlur(S * 0.003))
    img.alpha_composite(sweep)
    d = ImageDraw.Draw(img)
    x, y = polar(cx, cy, R, head)
    d.line([(cx, cy), (x, y)], fill=(255, 245, 160, 255), width=int(lw * 1.6))

    # Echos (amber) mit weichem Glühen
    glow = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glow)
    echoes = [(40, 0.62, 1.0), (300, 0.70, 0.8), (150, 0.45, 0.7), (230, 0.82, 0.55), (95, 0.86, 0.5)]
    for brg, rf, size in echoes:
        ex, ey = polar(cx, cy, R * rf, brg)
        er = S * 0.016 * size
        gd.ellipse((ex - er * 2.2, ey - er * 2.2, ex + er * 2.2, ey + er * 2.2), fill=(255, 200, 60, 90))
    glow = glow.filter(ImageFilter.GaussianBlur(S * 0.01))
    img.alpha_composite(glow)
    d = ImageDraw.Draw(img)
    for brg, rf, size in echoes:
        ex, ey = polar(cx, cy, R * rf, brg)
        er = S * 0.016 * size
        d.ellipse((ex - er, ey - er, ex + er, ey + er), fill=(255, 214, 90, 255))

    # ARPA: verfolgtes Ziel mit Kreis und Vektor (cyan)
    tx, ty = polar(cx, cy, R * 0.62, 40)
    cr = S * 0.038
    d.ellipse((tx - cr, ty - cr, tx + cr, ty + cr), outline=(110, 235, 255, 255), width=int(lw * 1.6))
    vdx, vdy = polar(0, 0, S * 0.12, 250)
    d.line([(tx, ty), (tx + vdx, ty + vdy)], fill=(110, 235, 255, 255), width=int(lw * 1.8))

    # Gefährliches Ziel: rotes Dreieck
    gx, gy = polar(cx, cy, R * 0.70, 300)
    tr = S * 0.042
    tri = [(gx, gy - tr), (gx + tr * 0.9, gy + tr * 0.6), (gx - tr * 0.9, gy + tr * 0.6)]
    d.line(tri + [tri[0]], fill=(255, 80, 80, 255), width=int(lw * 1.8), joint="curve")

    # Vorausstrich und Eigenschiff
    hx, hy = polar(cx, cy, R, 0)
    d.line([(cx, cy), (hx, hy)], fill=(245, 248, 255, 255), width=int(lw * 1.5))
    orr = S * 0.017
    d.ellipse((cx - orr, cy - orr, cx + orr, cy + orr), outline=(245, 248, 255, 255), width=int(lw * 1.5))

    # Peilskala-Ring mit Teilstrichen
    d.ellipse((cx - R, cy - R, cx + R, cy + R), outline=(200, 225, 245, 255), width=int(lw * 2.2))
    for deg in range(0, 360, 10):
        L = S * (0.03 if deg % 30 == 0 else 0.016)
        x0, y0 = polar(cx, cy, R, deg)
        x1, y1 = polar(cx, cy, R + L, deg)
        d.line([(x0, y0), (x1, y1)], fill=(200, 225, 245, 230), width=int(lw * (1.6 if deg % 30 == 0 else 1.0)))

    # feine helle Kante innen an der Kachel
    if shape != "full":
        edge = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        poly = tile_polygon(shape, S / OUT, inset + 1.5)
        ImageDraw.Draw(edge).line(poly + [poly[0]], fill=(255, 255, 255, 46), width=int(S * 0.003))
        img.alpha_composite(edge)
    return img


def compose(rgb, alpha, size):
    """Farbe (Lanczos) und Deckung (Flächenmittel) getrennt skalieren – keine Überschwinger an der Kante."""
    c = rgb.convert("RGB").resize((size, size), Image.LANCZOS)
    a = alpha.resize((size, size), Image.BOX) if alpha.size[0] != size else alpha
    out = c.convert("RGBA")
    out.putalpha(a)
    return out


def make_master(shape="continuous", inset=0.0, shadow=False):
    art = make_art(shape, inset)
    mask = tile_mask(shape, inset)
    master = compose(art, mask, OUT)
    if shadow:  # nur für Vergleichstests: weicher Schatten außerhalb der Kachel
        sh = Image.new("RGBA", (OUT, OUT), (0, 0, 0, 0))
        sa = mask.filter(ImageFilter.GaussianBlur(10)).point(lambda v: int(v * 0.35))
        sh.putalpha(sa)
        sh = sh.transform(sh.size, Image.AFFINE, (1, 0, 0, 0, 1, -10))
        sh.alpha_composite(master)
        master = sh
    return master, art, mask


def write_icns(rgb, mask, path):
    with tempfile.TemporaryDirectory() as tmp:
        iconset = os.path.join(tmp, "icon.iconset")
        os.makedirs(iconset)
        for base in (16, 32, 128, 256, 512):
            compose(rgb, mask, base).save(os.path.join(iconset, f"icon_{base}x{base}.png"))
            compose(rgb, mask, base * 2).save(os.path.join(iconset, f"icon_{base}x{base}@2x.png"))
        subprocess.run(["iconutil", "-c", "icns", iconset, "-o", path], check=True)


def main():
    os.makedirs(ROOT, exist_ok=True)
    master, art, mask = make_master()
    master.save(os.path.join(ROOT, "icon_1024.png"))
    compose(art, mask, 256).save(os.path.join(ROOT, "icon_256.png"))
    master.save(os.path.join(ROOT, "icon.ico"), sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    if shutil.which("iconutil"):
        write_icns(art, mask, os.path.join(ROOT, "icon.icns"))
    print("Icons geschrieben nach", os.path.normpath(ROOT))


if __name__ == "__main__":
    main()
