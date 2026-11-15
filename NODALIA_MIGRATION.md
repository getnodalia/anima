# Nodalia migration: Ánima

Category: Creator suite app (2D animation).
Status: renamed to Ánima and building from Nodalia sources, as of 2026-11-14. The full NDL
interface is still to come (below). Evidence: the pull request that adds this file.

## Current state

Pure-Rust 2D animation studio (egui/eframe 0.36, wgpu), MIT OR Apache-2.0. Every name is Ánima's
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

## NDL: what is left

Ánima belongs to the Creator suite (shared top bar with workspace selector, Home, dialogs). The
shared pieces are `nodalia_theme::creator` in nodalia-design (`packages/egui`), which needs egui
0.37; Ánima is on 0.36. In order:

1. **egui 0.37.** Move `egui` and `eframe` (and the wgpu they bring) to 0.37 in one pull
   request, with no visual change, and fix what the upgrade breaks (the screenshot and pseudo-locale
   tests in `src/` catch most of it). Then switch `nodalia-update` to its `egui` feature for the
   update windows.
2. **Theme.** Depend on `nodalia-theme` (`packages/egui/crate`, pinned by rev) and build
   `theme::DARK`/`LIGHT` from its tokens with Ánima's accent `#6B4CE8`, keeping High Contrast as
   it is (ADR 0022). Colours stay in `src/theme.rs` tokens.
3. **Window.** Draw the window without OS decorations on Linux and add NDL's round window
   controls (`nodalia_theme::window_controls`), switchable to the desktop's in Preferences; keep
   native controls on macOS (full-size content view, already in `src/platform.rs`) and Windows.
4. **Top bar and hamburger menu.** The Creator top bar with the workspace selector (Essentials,
   Animator and Designer: Ánima's existing workspaces) and a hamburger menu holding About,
   Preferences and Keyboard Shortcuts. Light/dark only in Preferences.
5. **Panels as islands** (`creator::island`) for the toolbar, timeline, stage and the Properties
   and Library panels, with the Creator buttons, tabs and fields.
6. **Home** (`creator::home`): recent documents, templates and samples, replacing the start screen.
7. **Dialogs** on the Creator look, with integrated close controls.
8. **Global menu.** On Linux, export the menus over D-Bus to the shell's top-bar menu and show the
   in-window menu bar only when the desktop has none.

Each step comes with light and dark screenshots from the app (`cargo run -- --screenshot`).

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

1. The NDL steps above, one pull request each.
2. A first release (`0.3.0`): version bump plus `docs/release-notes/v0.3.0.md`.
3. Ánima's page on getnodalia.com.
