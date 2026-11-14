#!/usr/bin/env bash
# Regenerate every app icon file from assets/app-icon/anima.svg (the canonical artwork).
#
# Needs: resvg (cargo install resvg) or rsvg-convert, and Python with Pillow (for the .ico). The
# outputs are committed, so building never needs these tools.
#
#   scripts/icons.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/assets/app-icon"
SVG="$DIR/anima.svg"
ID="dev.danielmiguelt.Anima"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

if command -v resvg >/dev/null; then
  render() { resvg -w "$2" -h "$2" "$1" "$3" </dev/null; }
elif command -v rsvg-convert >/dev/null; then
  render() { rsvg-convert -w "$2" -h "$2" "$1" -o "$3"; }
else
  echo "error: neither resvg (cargo install resvg) nor rsvg-convert found" >&2
  exit 1
fi

# The SVG is a full-bleed 512 tile, used as-is for macOS, Windows and the 1024 render. macOS
# icons follow Apple's grid: the tile is 824/1024 of the canvas with a transparent margin, so
# widen the viewBox around it.
MAC="$TMP/mac.svg"
sed 's/viewBox="0 0 512 512"/viewBox="-62.136 -62.136 636.272 636.272"/' "$SVG" >"$MAC"
grep -q 'viewBox="-62.136' "$MAC" || { echo "error: $SVG has no 512 viewBox" >&2; exit 1; }

# Linux puts the tile on the Colloid application grid, like the icons around it in the Nodalix
# icon theme: a 224 px tile inside a 256 px canvas, a 51.5 px corner radius and a soft shadow
# 5 px below. Still vector: the artwork is scaled into place and the shadow is a few stacked,
# fading rounded squares rather than a blur filter, which not every icon renderer applies.
LINUX="$DIR/hicolor/scalable/apps/$ID.svg"
mkdir -p "$(dirname "$LINUX")"
python3 - "$SVG" "$LINUX" <<'PY'
import re, sys
src, out = sys.argv[1], sys.argv[2]
art = open(src, encoding="utf-8").read()
inner = re.search(r"<svg[^>]*>(.*)</svg>", art, re.S).group(1)
inner = re.sub(r"\s*<title>.*?</title>", "", inner, flags=re.S)
# The artwork's own tile is rx 112 on 512; on the grid the radius is 51.5 on 224.
inner = inner.replace('<rect width="512" height="512" rx="112"/>', '<rect width="512" height="512" rx="117.71"/>')
scale = 224 / 512
shadow = []
for i, (grow, opacity) in enumerate([(0.0, 0.05), (1.2, 0.04), (2.4, 0.03), (3.5, 0.025)]):
    x, w = 16 - grow, 224 + 2 * grow
    shadow.append(f'<rect x="{x:g}" y="{21 - grow:g}" width="{w:g}" height="{w:g}" rx="{51.5 + grow:g}" opacity="{opacity}"/>')
svg = (
    '<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">\n'
    "  <title>Ánima</title>\n"
    "  <!-- Ánima app icon on the Colloid application grid (scripts/icons.sh). -->\n"
    f"  <g fill=\"#000\">{''.join(shadow)}</g>\n"
    f'  <g transform="translate(16 16) scale({scale:g})">{inner}</g>\n'
    "</svg>\n"
)
open(out, "w", encoding="utf-8").write(svg)
PY

render "$SVG" 1024 "$DIR/anima-1024.png"
# The tile on Apple's grid: the runtime Dock icon (src/main.rs) and store/landing pages.
render "$MAC" 1024 "$DIR/anima-padded-1024.png"
render "$MAC" 512 "$DIR/anima-macos-512.png"

# Linux hicolor theme, from the grid version (the 256 one is also the runtime icon on Windows and
# Linux, the 128 one the mark in Help › About).
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${s}x${s}/apps"
  render "$LINUX" "$s" "$DIR/hicolor/${s}x${s}/apps/$ID.png"
done

# Windows .ico (embedded in anima.exe by build.rs): each size rendered, not scaled down.
ICO_SIZES=(16 20 24 32 40 48 64 128 256)
for s in "${ICO_SIZES[@]}"; do render "$SVG" "$s" "$TMP/ico-$s.png"; done
python3 - "$DIR/anima.ico" "$TMP" "${ICO_SIZES[@]}" <<'PY'
import sys
from PIL import Image
out, tmp, sizes = sys.argv[1], sys.argv[2], [int(s) for s in sys.argv[3:]]
imgs = [Image.open(f"{tmp}/ico-{s}.png").convert("RGBA") for s in sizes]
imgs[-1].save(out, format="ICO", sizes=[(s, s) for s in sizes], append_images=imgs[:-1])
PY
echo "icons written to $DIR"
