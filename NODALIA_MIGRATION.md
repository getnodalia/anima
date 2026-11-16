# Nodalia migration: Ánima

Category: Creator suite app (2D animation).
Status: on the full Nodalia Design Language (Creator suite) as of 2026-11-15. Evidence: the
pull request that moves the interface to NDL (light and dark screenshots there).

## Current state

Pure-Rust 2D animation studio (egui/eframe 0.37, wgpu), MIT OR Apache-2.0. Every name is Ánima's
(`anima`, `dev.danielmiguelt.Anima`); upstream's history is kept and its weekly changes arrive
renamed (`docs/upstream.md`). Cargo git dependencies come from getnodalia/foundation (craft-libs at
`vendor/craft-libs/pin-f0f4758`) and nodalia-design (`nodalia-update`). CI calls Foundation's
reusable workflows pinned by commit, on standard GitHub-hosted runners only; releases use the
automatic flow (`release-auto.yml`, unsigned macOS DMG). Ánima has its own icon (Émile Reynaud's portrait,
Nodalia icon family, `assets/app-icon/`), Nodalia's violet accent and a Nodalia DMG window.

## Compatibility kept on purpose

- `.anicraft` documents keep opening and are registered on every OS; new documents save as
  `.anima`.
- The `"format": "anicraft"` signature inside a document (from `craft-anicraft-format`) and the
  WebAssembly import module of plug-ins keep their old value: existing files and plug-ins depend on
  them. `.github/rebrand.toml` keeps them.
- `build.rs` reads `anicraft.tsv` from a craft-i18n checkout when there is no `anima.tsv`.

## NDL: done

Ánima is a Creator suite app and uses `nodalia-theme` (nodalia-design, pinned by rev, feature
`creator`):

- **Theme.** `theme::GRAPHITE` and `theme::LIGHT` take their surfaces, text, controls and accent
  from NDL's palette with Ánima's accent (`Accent::Anima`); High Contrast stays Ánima's own
  (ADR 0022). Corner radii follow NDL (8 / 10 / 16).
- **Islands.** The tools bar, the docks, the timeline and the stage are rounded islands on the
  window colour, 8 pt apart.
- **Top bar.** NDL's 44 pt Creator bar (`src/header.rs`): the hamburger menu (About, Preferences,
  Keyboard Shortcuts, Quit), Home, the in-window menus when the desktop shows none, the
  document's name and the workspace selector (Essentials, Animator, Designer and saved ones, with
  Reset, Save and Manage).
- **Window.** On Linux the window has no system title bar: the top bar drags it and NDL's round
  controls sit at its end (`src/titlebar.rs`), switchable to the desktop's own in Preferences ▸
  Window controls. macOS and Windows keep their native controls.
- **Home** is `creator::home`: New file, Open, Home, User guide and the website in the rail;
  presets, the sample, recent files and templates in the body.
- **Dialogs.** Preferences and About have NDL's integrated close button and no system title bar;
  light and dark are chosen only in Preferences.
- **Icons.** Interface icons are NDL's bold set (`Icon::ndl`); the few NDL lacks keep Ánima's
  own drawings.
- **Global menu.** On Linux the menus are exported over D-Bus (dbusmenu) to the desktop's
  top-bar menu (`src/global_menu.rs`); the in-window menus show only when nothing took them.

Left for later: the remaining dialogs (Shortcuts, Publish, Licences…) still use egui windows in
NDL colours rather than the Creator dialog frame.

## Inherited pieces still pending

| What | Where | Replacement |
|---|---|---|
| JetBrains Mono | `build.rs` (`MONO_FONT`) | not in Foundation's craft-fonts yet; data fields use egui's Hack until it is added there |
| Translations | `build.rs` (`CRAFT_I18N_DIR`) | upstream has no catalog for this app; Ánima's would go to Foundation's `vendor/craft-i18n` as `locales/<locale>/anima.tsv` |
| Windows arm64 and x86, Linux aarch64 and riscv64, Flatpak, FreeBSD packages | the inherited release workflow (removed) | `release-*.yml` builds Linux x86_64, macOS universal and Windows x64; the scripts in `packaging/` still build the others |
| Code signing (Windows, macOS notarization) | `release-*.yml` | optional secrets; without them packages are unsigned, which is Nodalia's default |

## Reusable Nodalia components

getnodalia/foundation (vendored crates, fonts, reusable workflows); nodalia-design
(`nodalia-update`, `nodalia-theme`, the DMG background tool); nodalix-icon-theme (the icon at
Colloid size).

## Recommended order

1. The first release, `0.1.0`: version bump plus `docs/release-notes/v0.1.0.md`.
2. Ánima's page on getnodalia.com (`/anima`).
3. The remaining dialogs on the Creator dialog frame.
