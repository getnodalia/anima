# Ánima upstream

| Field | Value |
|---|---|
| Upstream | `storytold/anicraft` (MIT OR Apache-2.0) |
| Last observed | `3435e9868150312e54694c0a70082609b2fe8573` |
| Last evaluated | `3435e9868150312e54694c0a70082609b2fe8573` |
| Last integrated | `8729eba624f41935967af1fefd27c2739d157e6b` (653 commits since the last sync) on 2026-11-18 |
| Adapted | every name (app, crates, binaries, app id, installers), the app icon, the in-app updater (`nodalia-update`), the Nodalia accent |
| Kept for compatibility | the `"format"` signature inside documents, the plug-in host module, `.anicraft` as an extension that still opens |

## How the sync works

Upstream's code arrives renamed: `scripts/rebrand.py` applies `.github/rebrand.toml` (AniCraft
becomes Ánima, `anicraft` becomes `anima`, internal names included) to upstream's tree, and the
weekly sync (`.github/workflows/upstream-sync.yml`, Mondays) records the result as one mirror
commit: its only parent is the previous mirror commit, its subject is `Upstream <sha> with this
app's names`, and an `Upstream-Commit:` trailer names the upstream commit it renames. The sync
merges that commit into a branch and opens a pull request; files listed in `.github/upstream-keep`
keep this repository's version. No mirror branch is pushed, so merge sync pull requests with a merge
commit: a squash would drop the mirror commit the next sync starts from.

The mirror starts at the renamed `3435e986` (the commit whose message is "Upstream 3435e986 with
this app's names"), merged into `main` with this repository's own files by the pull request that
renamed the app.

`python3 scripts/rebrand.py check` (the CI job "No upstream names") fails when an old name comes
back. A line that must keep one says `rebrand: keep`.

No mirror branch is pushed. To let the sync's pull request run
the checks, give it an `UPSTREAM_SYNC_TOKEN` secret (a fine-grained token with contents and pull
requests write on this repository). Without it the pull request opens and the checks start when
someone pushes to it.
