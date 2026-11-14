# AGENTS.md: guide for AI agents and contributors

Engineering rules for every Nodalia repository: [getnodalia/foundation](https://github.com/getnodalia/foundation)
([engineering](https://github.com/getnodalia/foundation/blob/main/standards/engineering.md),
[CI](https://github.com/getnodalia/foundation/blob/main/standards/ci.md),
[security](https://github.com/getnodalia/foundation/blob/main/standards/security.md),
[dependencies and licences](https://github.com/getnodalia/foundation/blob/main/standards/dependencies.md),
[release](https://github.com/getnodalia/foundation/blob/main/standards/release.md),
[NDL](https://github.com/getnodalia/foundation/blob/main/standards/ndl.md)).

## What this is

Ánima is an open-source, native, clean-room 2D animation studio (Adobe Animate class), written
in **Rust only**. There is no Tauri, Electron or webview. The UI is egui/eframe. The product name
is written **Ánima**; machine names are lowercase (`anima-*`). Read `docs/` first.

## Architecture: workspace map

```text
crates/geom     L0  Vec2, Affine (decompose/compose), Rect, Path, flattening, hit testing
crates/xml      L0  Bounded XML element reader shared by the importers (XFL, SVG)
crates/plugins  L0  Sandboxed WebAssembly plug-ins: wasmi host, ABI v1, limits, registry (ADR 0017)
crates/audio    L0  Sound decoding (WAV, Ogg Vorbis, MP3), pure mixing and resampling, WAV, peaks (ADR 0020)
crates/actions  L0  Timeline action IR: stop/play/goto, targets, button events, frame scripts (ADR 0024)
crates/guide    L0  The user guide (docs/guide) as a bundled static HTML book, with its link checks (ADR 0069)
crates/doc      L1  Document model, easing, timeline evaluation, the interactive player, sample scene
crates/text     L1  Fonts on this computer, shaping and line layout into outlines (ADR 0012)
crates/render   L2  CPU oracle renderer (coverage accumulation, paints, strokes, PNG)
crates/xfl      L2  XFL/FLA import: own XML + zip readers, edges → paths (ADR 0009)
crates/svg      L2  SVG import: shapes, groups, use, styles, gradients, clips/masks → mask layers, text
crates/engine   L3  Session, undo (with gesture coalescing), command registry, .anima format
crates/gpu      L3  wgpu stage renderer (the default stage): tile binning, compute coverage, bitmaps, filters (ADR 0019, 0029, 0032)
crates/cli      L4  anima-cli: info/render/render-seq/commands/exec/run/new/guide/serve/mcp
src/            L5  egui shell (app, menus, stage, timeline, panels, theme)
```

A crate depends only on lower layers. Nothing below `src/` may use egui, eframe or winit.
`src/architecture_tests.rs` enforces both, and that this map lists every crate with its layer:
a new crate needs a row here and in its `LAYERS` table.

## Standing rules (read `docs/product-principles.md`)

* **Principal-engineer check.** For every proposal ask: "Is this how a principal engineer
  would do it?" If not, use the better method or ask the project lead. Record hard-to-reverse
  decisions as ADRs in `docs/adr/`. **ADR numbers never move:** take the next number above
  every ADR and every reservation in `docs/adr/README.md`, and reserve it there in the same
  PR. An open PR keeps its number until it merges or closes, so no one renumbers because
  another PR merged first; maintainers reserve the numbers outside PRs already use.
* **Users are artists and creatives** (many coming from Adobe Animate). The product must be
  professional, fast, reliable and easy, with smooth, predictable flow: common actions one
  click or key away, shortcuts for power users, no modal interruptions for routine edits.
* **UI guidelines** (`docs/ui-guidelines.md`, ADR 0005): text goes through `theme::Type`
  roles. Data (numbers, timecode, rulers, values, shortcuts, paths) is monospace; prose
  (labels, menus, buttons, names, dialog text) is proportional. No raw font sizes or colours
  outside `theme.rs` (enforced by `guidelines_tests`). Typefaces: Inter + JetBrains Mono via
  `CRAFT_FONTS_DIR`, with egui defaults as fallback.
* **Interface text is translatable** (ADR 0040, `docs/localisation.md`): write English through
  `tr!` / `trn!` / `tr_noop!`, one template per sentence, and translate keys only where they are
  drawn. Command ids, the CLI, MCP and the file format stay English. Check with `--lang pseudo`.
* **Cross-platform** (ADR 0006): macOS, Windows and Linux are first-class. OS-specific
  behaviour lives only in `src/platform.rs` (shortcut labels with ⌘ on macOS, app menu,
  window chrome, settings folder, native dialogs, GPU backend fallback). Shortcut text always
  goes through `platform::shortcut_label`.
* **Edits are live**: anything that changes the document must show in the same frame (Auto
  render mode). Never leave an edit waiting for the next input event.
* **Destructive actions** follow the tiers in ADR 0004: irreversible loss always confirms
  (engine-enforced `"confirm": true`); large undoable deletions confirm by default; routine
  edits rely on undo with a visible hint.

## Golden rules

1. **Never crash.** Non-test code never panics: no `unwrap`/`expect`/`panic!`/indexing on
   input-derived values. Crates carry `#![deny(clippy::unwrap_used, …)]`. Commands validate params
   and return `Err`. The `panic_hunt_every_command` test fuzzes every command and must stay green.
2. **Everything is a command.** New behaviour means a `CommandSpec` in `crates/engine/src/cmds_*.rs`
   with id, label, menu path, shortcut, params doc and tests. Menus and shortcuts are generated
   from the registry. Only view state (zoom, pan, tool, onion skin, playback) lives in the shell.
3. **Time is a function.** Never animate by mutating the document. Add behaviour to
   `doc::evaluate` and keep it pure.
4. **The CPU renderer is the oracle.** Renderer changes come with analytic or golden tests. Any
   future GPU renderer is tested against it.
5. **No unsafe** (`unsafe_code = "forbid"`).
6. **Clean room.** Learn behaviour from public docs and observation only. Never copy proprietary
   code, presets, shaders or assets.
7. **Verify UI changes visually**: `cargo run -- --screenshot shots/now.png`, then look at the PNG.

## Licensing and legal compliance (read `docs/legal/compliance.md`)

* Ánima is **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`). `NOTICE` keeps the
  upstream copyright line and says where the code base began; credit the origin there and in
  About only, never in the README, the website or the interface.
* **Dependencies**: permissive only, from crates.io only. `deny.toml` is the policy, and
  `cargo deny check licenses bans sources` must pass. Adding a crate with a new licence needs
  a deliberate `deny.toml` change and a note in `compliance.md`. No copyleft, "non-commercial"
  or source-available terms.
* **Assets**: every font, icon, image, sound or sample file gets a row in `ATTRIBUTION.md`
  in the same change, with its licence text under `licenses/`. Never commit font files
  (use the shared craft-fonts input). Never commit Adobe or other proprietary material.
* **Patents**: implement only from public papers, open specifications and permissively
  licensed references; credit them in `ATTRIBUTION.md`. Never bundle encoders for
  patent-pool codecs (H.264/HEVC/AAC); use royalty-free formats or a user-installed ffmpeg.
* **Trademarks**: Adobe, Animate and Flash only nominatively (docs, and the non-affiliation
  disclaimer in legal notices), never in the product name, branding, feature UI or marketing.

## Version control

* Git. Remote `origin` = `https://github.com/getnodalia/anima` (public). `main` is the
  development branch; it begins with "Initial commit v0.1.0 early alpha", and contributors are
  credited in `AUTHORS.md`.
* **Every change reaches `main` through a pull request** from a topic branch (`feat/…`, `fix/…`,
  `docs/…`), with one roadmap step per PR, green on build, tests, clippy and `cargo deny`.
  Never commit or push to `main` directly. `core.hooksPath` points at `scripts/git-hooks`:
  `pre-commit` refuses commits on `main`, and `pre-push` refuses pushes to the remote `main`.
  Run `git config core.hooksPath scripts/git-hooks` once per clone.
* Branches marked hidden in your local git config (`git config --add anima.hidden <branch>`)
  never leave the machine: `pre-push` refuses them and anything built on them.
* **Milestone tracks.** Milestones that don't depend on unfinished work run in parallel, one
  agent per track (`docs/roadmap.md` › Parallel tracks). Each track works in its own git
  worktree under `.worktrees/<track>` (git-ignored), on branches named `m<N>/<topic>`, and
  every commit subject and PR title ends with its milestone, e.g.
  `feat(audio): audio layers with waveforms (M5)`. Each worktree builds into its own folder
  inside the shared `target/` (`CARGO_TARGET_DIR=target/tracks/<track>`): worktrees of one
  workspace produce artifacts with the same names, so a single shared folder lets one track's
  build overwrite another's and tests run against the wrong code. Before opening or updating
  a PR, bring in `origin/main` and keep shared files (CHANGELOG, roadmap, Cargo.lock, the
  workspace map) conflict-free. ADR numbers are reserved in `docs/adr/README.md` when a task
  starts and never move (see below). A track never merges its own PR: the lead reviews it
  (principal engineer and senior designer) and merges it when CI is green, one at a time.
* **A new command namespace** needs a plug-in policy (`plugin_may_call` in
  `crates/engine/src/cmds_plugin.rs`); a test fails until it has one.
* **Outside pull requests** that match a roadmap task are reviewed and their good ideas
  considered. The work stays first-party: a track waits on an outside PR only if it is
  complete and fits our architecture and flow.
* Commit in small, green steps (build + tests pass), with conventional messages:
  `feat(render): …`, `fix(engine): …`, `docs: …`, `perf: …`, `test: …`. Record measured
  numbers in `perf:` commit bodies.
* Never commit `target/`, screenshots (`shots/`), exports or `.anima` user files.

## Nodalia

* **Names.** Every name is Ánima's, internal ones included: `anima` in machine names, **Ánima**
  in prose and the interface, app id `dev.danielmiguelt.Anima`. Documents save as `.anima`;
  `.anicraft` files keep opening, and the `"format"` signature inside a document and the plug-in <!-- rebrand: keep -->
  host module keep their old value, because existing files and plug-ins depend on them (each such
  line says `rebrand: keep`). `python3 scripts/rebrand.py check` fails when an old name comes back.
* **Upstream sync.** `.github/workflows/upstream-sync.yml` brings upstream's changes in weekly,
  renamed by `scripts/rebrand.py` with `.github/rebrand.toml` (`docs/upstream.md`). Files this
  project owns are listed in `.github/upstream-keep`; add any new one there.
* **Dependencies** come from crates.io or from getnodalia (`getnodalia/foundation`'s `vendor/*`
  branches, `danielmigueltejedor/nodalia-design`), pinned by revision. Nothing depends on
  storytold or ArtCraft repositories or services. <!-- rebrand: keep -->
* **NDL.** Ánima belongs to the Creator suite of the Nodalia Design Language. What is done and
  what is left is in `NODALIA_MIGRATION.md`; colours come from the theme tokens, NDL window
  controls on Linux only, native ones on macOS and Windows. No GTK or libadwaita.
* **CI** runs on standard GitHub-hosted runners only, actions pinned to a commit SHA, least
  permissions. Keep CI changes in their own pull request unless a change cannot work without them.

## Release

A pull request that sets `[workspace.package] version` to a release
version and adds `docs/release-notes/vX.Y.Z.md` (format in that folder's README) releases it
when it merges: `release-auto.yml` tags it through Foundation's `release-tag`, builds the
Linux, macOS (unsigned DMG) and Windows packages and publishes the release. Nobody pushes a tag.

## Build hygiene (disk first, then time; `docs/development.md` › Build footprint)

* One checkout uses the **single `target/`**. Don't make extra target folders for one
  checkout (each is a full build); parallel worktrees are the exception above
  (`target/tracks/<track>`, removed with the worktree when its track is done).
* Don't turn incremental compilation or full debug info back on in `Cargo.toml`. Do that
  locally if you need a debugger.
* After profile or feature changes, run `cargo clean` once (with the app closed).

## Build and test (before you finish)

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo deny check licenses bans sources
cargo run -- --screenshot shots/now.png
```

Update `docs/roadmap.md` › Honest status when you move a capability.
