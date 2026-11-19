#!/usr/bin/env python3
"""Erzeugt das App-Icon (Radarschirm mit Antennenstrahl und ARPA-Zielen).

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

S = 2048  # Arbeitsauflösung (2× Supersampling für saubere Kanten)
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")


def lerp(a, b, t):
    return tuple(int(round(a[i] + (b[i] - a[i]) * t)) for i in range(len(a)))


def rounded_mask(size, box, radius):
    m = Image.new("L", (size, size), 0)
    ImageDraw.Draw(m).rounded_rectangle(box, radius=radius, fill=255)
    return m


def polar(cx, cy, r, deg):
    a = math.radians(deg)
    return cx + r * math.sin(a), cy - r * math.cos(a)


def make_master():
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))

    # macOS-Raster: Kachel ca. 80 % der Fläche, Rest Rand für den Schatten
    m = int(S * 0.098)
    box = (m, m, S - m, S - m)
    radius = int((S - 2 * m) * 0.225)

    # Schatten
    shadow = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).rounded_rectangle(
        (box[0], box[1] + int(S * 0.018), box[2], box[3] + int(S * 0.018)), radius=radius, fill=(0, 0, 0, 150)
    )
    shadow = shadow.filter(ImageFilter.GaussianBlur(S * 0.022))
    img.alpha_composite(shadow)

    # Kachel mit Verlauf (oben heller)
    tile = Image.new("RGBA", (S, S))
    td = ImageDraw.Draw(tile)
    top, bottom = (28, 78, 150), (8, 26, 62)
    for y in range(S):
        td.line([(0, y), (S, y)], fill=lerp(top, bottom, y / S) + (255,))
    img.paste(tile, (0, 0), rounded_mask(S, box, radius))

    cx, cy = S / 2, S / 2 + S * 0.005
    R = (S - 2 * m) * 0.40

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
    d = ImageDraw.Draw(img)

    # Antennenstrahl: Nachleuchtfächer (gelb-grün, nach hinten ausblendend)
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
    vx, vy = tx + vdx, ty + vdy
    d.line([(tx, ty), (vx, vy)], fill=(110, 235, 255, 255), width=int(lw * 1.8))

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

    # feine helle Kante an der Kachel
    edge = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    ImageDraw.Draw(edge).rounded_rectangle(box, radius=radius, outline=(255, 255, 255, 40), width=int(S * 0.004))
    img.alpha_composite(edge)

    return img.resize((1024, 1024), Image.LANCZOS)


def main():
    os.makedirs(ROOT, exist_ok=True)
    master = make_master()
    master.save(os.path.join(ROOT, "icon_1024.png"))
    master.resize((256, 256), Image.LANCZOS).save(os.path.join(ROOT, "icon_256.png"))
    master.save(os.path.join(ROOT, "icon.ico"), sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])

    if shutil.which("iconutil"):
        with tempfile.TemporaryDirectory() as tmp:
            iconset = os.path.join(tmp, "icon.iconset")
            os.makedirs(iconset)
            for base in (16, 32, 128, 256, 512):
                master.resize((base, base), Image.LANCZOS).save(os.path.join(iconset, f"icon_{base}x{base}.png"))
                master.resize((base * 2, base * 2), Image.LANCZOS).save(os.path.join(iconset, f"icon_{base}x{base}@2x.png"))
            subprocess.run(["iconutil", "-c", "icns", iconset, "-o", os.path.join(ROOT, "icon.icns")], check=True)
    print("Icons geschrieben nach", os.path.normpath(ROOT))


if __name__ == "__main__":
    main()
