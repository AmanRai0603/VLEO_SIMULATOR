#!/usr/bin/env python3
"""Draw the desktop app's icon: crates/vleo-app/icon/vleo.{png,ico,icns}.

    python3 tools/make_icon.py

Drawn here rather than kept as an opaque image, so what the icon is can be
read and changed like any other source: a dark field, the limb of the Earth,
and one satellite on a very low orbit just above it. Needs Pillow; the three
files it writes are committed, so building the app never needs it.
"""
import os
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "..", "crates", "vleo-app", "icon")
S = 1024


def draw(size=S):
    k = 4  # drawn large, scaled down smooth
    n = size * k
    im = Image.new("RGBA", (n, n), (20, 27, 48, 255))
    d = ImageDraw.Draw(im)
    # The Earth's limb, low in the frame.
    R = int(n * 1.1)
    cx, cy = n // 2, int(n * 1.62)
    d.ellipse([cx - R, cy - R, cx + R, cy + R], fill=(38, 99, 170, 255))
    # A thin atmosphere above it: very low orbit lives just over this line.
    a = int(n * 0.018)
    d.ellipse([cx - R - a, cy - R - a, cx + R + a, cy + R + a], outline=(125, 190, 255, 255), width=a)
    # The orbit, just above the atmosphere.
    ow = int(n * 0.028)
    box = [int(n * 0.10), int(n * 0.28), int(n * 0.90), int(n * 0.66)]
    d.arc(box, start=195, end=345, fill=(236, 240, 247, 255), width=ow)
    # The satellite: a body and two panels.
    # On the orbit: the ellipse's centre is (0.5, 0.47), its half-axes 0.4 and 0.19.
    sx = int(n * 0.62)
    sy = int(n * (0.47 - 0.19 * (1 - ((0.62 - 0.5) / 0.4) ** 2) ** 0.5))
    b = int(n * 0.045)
    d.rectangle([sx - b, sy - b, sx + b, sy + b], fill=(236, 240, 247, 255))
    pw, ph = int(n * 0.11), int(n * 0.035)
    d.rectangle([sx - b - pw, sy - ph, sx - b - int(n * 0.012), sy + ph], fill=(250, 190, 70, 255))
    d.rectangle([sx + b + int(n * 0.012), sy - ph, sx + b + pw, sy + ph], fill=(250, 190, 70, 255))
    # Everything clipped to the rounded square the platforms expect.
    mask = Image.new("L", (n, n), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, n - 1, n - 1], radius=int(n * 0.2), fill=255)
    im.putalpha(mask)
    return im.resize((size, size), Image.LANCZOS)


def main():
    os.makedirs(OUT, exist_ok=True)
    big = draw()
    big.save(os.path.join(OUT, "vleo.png"))
    big.save(os.path.join(OUT, "vleo.ico"),
             sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    big.save(os.path.join(OUT, "vleo.icns"))
    for f in ("vleo.png", "vleo.ico", "vleo.icns"):
        p = os.path.join(OUT, f)
        print("%-10s %7d bytes" % (f, os.path.getsize(p)))


if __name__ == "__main__":
    main()
