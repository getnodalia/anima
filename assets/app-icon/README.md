# Ánima app icon

**Figure:** Émile Reynaud (1844–1918). His Théâtre Optique projected the first animated films in
1892, hand-painted frame by frame: the animator's craft Ánima is for.

**Style:** a black-and-white portrait cut out over a full-bleed colour field, head and shoulders,
in the Nodalia icon family (one historical figure per app, one colour per app).

**Tile:** Ánima violet, `#6B4CE8` at the top to `#5034C6` at the bottom; `viewBox="0 0 512 512"`,
a rounded square with `rx=112`, the artwork clipped to it. macOS adds Apple's transparent margin
(tile = 824/1024 of the canvas); Windows uses the tile edge to edge. Linux puts it on the Colloid
application grid, like the Nodalix icon theme around it: a 224 px tile in a 256 px canvas, a
51.5 px corner radius and a soft shadow 5 px below.

**Provenance:** the portrait is "Émile Reynaud.png" from
[Wikimedia Commons](https://commons.wikimedia.org/wiki/File:%C3%89mile_Reynaud.png), a photograph
by an unknown photographer from before 1918 (public domain). The figure was separated from its
background, converted to black and white and placed over the app colour with nodalia-design's
`tools/app-portrait`. Licence: see `LICENSE.txt`.

## Files

| File | What | Used by |
|---|---|---|
| `anima.svg` | master: the portrait as a 1024 px image embedded in an SVG with the tile clip (canonical) | source for everything below |
| `anima-1024.png` | full tile, 1024 px | docs, store listings |
| `anima-padded-1024.png` | tile with Apple's margin, 1024 px | landing pages, store listings, `Anima.icns` |
| `anima-macos-512.png` | tile with Apple's margin, 512 px | runtime Dock icon on macOS (`src/platform.rs`) |
| `anima.ico` | Windows icon, 16 to 256 px | `build.rs` (embedded in `anima.exe`) |
| `hicolor/scalable/apps/dev.danielmiguelt.Anima.svg` | the tile on the Colloid grid | Linux packaging, nodalix-icon-theme |
| `hicolor/<size>/apps/dev.danielmiguelt.Anima.png` | Linux theme icons, 16 to 512 px, rendered from the grid version; the 256 one is also the runtime icon on Windows and Linux, the 128 one the mark in Help › About (`src/app.rs`) | Linux packaging, README |

## How it reaches each OS

- **macOS:** the app sets the Dock / app-switcher icon at runtime (`ViewportBuilder::with_icon`),
  and `packaging/macos/package.sh` builds `Anima.icns` for the bundle from the tile.
- **Windows:** `build.rs` embeds `anima.ico` with `winresource` (taskbar, Start menu, Explorer,
  Alt-Tab); the runtime icon covers the title bar.
- **Linux:** the runtime icon covers the window; the packages install `hicolor/` to
  `share/icons/` for launchers.

## Regenerate

```sh
# 1. the artwork, in nodalia-design (needs rembg):
python3 tools/app-portrait/portrait.py "Émile_Reynaud.png" anima.png --box 0,-30,470,440 \
  --colors '#6B4CE8' '#5034C6' --gamma 1.25 \
  --add-mask 300,300,340,300,400,330,440,380,470,420,470,572,300,572
# 2. the master SVG, here:
python3 scripts/icon-svg.py anima.png assets/app-icon
# 3. every render (needs resvg or rsvg-convert, and Python with Pillow):
scripts/icons.sh
```

The SVGs embed a raster portrait instead of traced paths, so a regenerated set can differ slightly
from the committed files in resampling.
