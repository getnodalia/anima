# Ánima upstream

| Field | Value |
|---|---|
| Upstream | `storytold/anicraft` (MIT OR Apache-2.0) |
| Last observed | `3435e9868150312e54694c0a70082609b2fe8573` |
| Last evaluated | `3435e9868150312e54694c0a70082609b2fe8573` |
| Last integrated | `3435e9868150312e54694c0a70082609b2fe8573`, merged on 2026-11-14 |
| Adapted | every name (app, crates, binaries, app id, installers), the app icon, the in-app updater (`nodalia-update`), the Nodalia accent |
| Kept for compatibility | the `"format"` signature inside documents, the plug-in host module, `.anicraft` as an extension that still opens |

## How the sync works

Upstream's code arrives renamed: `scripts/rebrand.py` applies `.github/rebrand.toml` (AniCraft
becomes Ánima, `anicraft` becomes `anima`, internal names included) to upstream's tree, and the
weekly sync (`.github/workflows/upstream-sync.yml`, Mondays) records the result on the
`upstream-anima` branch. Each commit there has the previous mirror commit and the upstream commit
it renames as parents, and names that commit in an `Upstream-Commit:` trailer. The sync merges the
newest mirror commit into a branch and opens a pull request; files listed in
`.github/upstream-keep` keep this repository's version.

The mirror starts at the renamed `3435e986` (the commit whose message is "Upstream 3435e986 with
this app's names"), merged into `main` with this repository's own files by the pull request that
renamed the app. Until `upstream-anima` exists, the sync finds that commit through its trailer.

`python3 scripts/rebrand.py check` (the CI job "No upstream names") fails when an old name comes
back. A line that must keep one says `rebrand: keep`.

The repository can push the mirror branch with its own token; to let the sync's pull request run
the checks, give it an `UPSTREAM_SYNC_TOKEN` secret (a fine-grained token with contents and pull
requests write on this repository). Without it the pull request opens and the checks start when
someone pushes to it.
