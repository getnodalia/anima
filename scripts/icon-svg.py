#!/usr/bin/env python3
"""Writes Ánima's master icon `assets/app-icon/anima.svg` from its portrait artwork. Then run
`scripts/icons.sh`, which makes every other icon file from it.

    python3 scripts/icon-svg.py PORTRAIT.png assets/app-icon

PORTRAIT is the 1024 px full-bleed artwork (the portrait already cut out and laid on Ánima's
violet), made by nodalia-design's tools/app-portrait (see assets/app-icon/README.md). The master
SVG embeds it, so it is the canonical artwork afterwards. Needs Pillow.
"""
import base64
import io
import sys
from pathlib import Path

from PIL import Image

QUALITY = 90


def jpeg(image, size):
    buffer = io.BytesIO()
    image.convert('RGB').resize((size, size), Image.LANCZOS).save(
        buffer, 'JPEG', quality=QUALITY, optimize=True, progressive=True)
    return base64.b64encode(buffer.getvalue()).decode()


def master(art, comment):
    """The 512 full-bleed tile: the portrait clipped to a rounded square, corner radius 112."""
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512">\n'
        f'  {comment}\n'
        '  <defs><clipPath id="tile"><rect width="512" height="512" rx="112"/></clipPath></defs>\n'
        f'  <image width="512" height="512" clip-path="url(#tile)" href="data:image/jpeg;base64,{jpeg(art, 1024)}"/>\n'
        '</svg>\n')


COMMENT = ('<!-- Ánima app icon: Émile Cohl, maker of Fantasmagorie, the first hand-drawn '
           'animated cartoon (public-domain photograph), cut out in black and white on Ánima\'s violet. '
           'MIT OR Apache-2.0 (assets/app-icon/README.md). -->')

if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    art = Image.open(sys.argv[1]).convert('RGB')
    (Path(sys.argv[2]) / 'anima.svg').write_text(master(art, COMMENT), encoding='utf-8')
