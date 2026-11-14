# Ánima app icon

**Motif:** a ball's arc through three keyframes. The earlier poses stay behind as onion skins, the
dotted path is the in-between frames, and the ball lands squashed. Under it, the same three
keyframes sit on a tween span in the timeline: movement, frames and interpolation in one picture.

**Style:** a flat, Colloid-style application tile in the Nodalia family: one colour field per app,
bold simple shapes that still read at 16 px, a light rim on top and a darker one below, no text and
no logo.

**Palette:**

| Colour | Hex | Used for |
|---|---|---|
| Ánima violet | `#6b4ce8` → `#5034c6` | the field, a soft top-to-bottom shade |
| Onion skin | `#8f76f0`, `#b4a4f7` | the earlier poses, paler the further back; the tween span |
| Ball | `#ff7a59` | the current pose and its keyframe |
| Timeline | `#2a1a78` at 60 % | the timeline strip |

**Tile:** `viewBox="0 0 512 512"`, a rounded square with `rx=112`, the artwork clipped to it.
macOS adds Apple's transparent margin (tile = 824/1024 of the canvas); Windows uses the tile edge
to edge. Linux puts it on the Colloid application grid, like the Nodalix icon theme around it: a
224 px tile in a 256 px canvas, a 51.5 px corner radius and a soft shadow 5 px below.

**Provenance:** original artwork drawn for Ánima as plain SVG geometry (no embedded images, no
third-party material). Licence: see `LICENSE.txt`.

## Files

| File | What | Used by |
|---|---|---|
| `anima.svg` | master vector, hand-written SVG (canonical) | source for everything below |
| `anima-1024.png` | full tile, 1024 px | docs, store listings |
| `anima-padded-1024.png` | tile with Apple's margin, 1024 px | landing pages, store listings |
| `anima-macos-512.png` | tile with Apple's margin, 512 px | runtime Dock icon on macOS (`src/main.rs`) |
| `anima.ico` | Windows icon, 16 to 256 px | `build.rs` (embedded in `anima.exe`) |
| `hicolor/scalable/apps/dev.danielmiguelt.Anima.svg` | the tile on the Colloid grid, vector | Linux packaging |
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
scripts/icons.sh   # needs resvg or rsvg-convert, and Python with Pillow
```
