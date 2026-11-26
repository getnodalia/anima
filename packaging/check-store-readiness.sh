#!/usr/bin/env bash
# Store readiness (ADR 0094): checks the metadata app stores read, without submitting anything.
# Fails on errors; tools that aren't installed are skipped with a warning (CI installs them).
#
#   AppStream          the metainfo, filled in as the packages fill it, passes
#                      `appstreamcli validate --no-net` (errors fail; warnings are shown)
#   Desktop entry      `desktop-file-validate`
#   winget / MSIX      the required fields of any manifests under packaging/windows/{winget,msix}
#                      (none are committed yet; submissions are the team's job)
#
# Usage: packaging/check-store-readiness.sh      Needs: python3.
set -euo pipefail
# shellcheck source=env.sh
. "$(dirname "${BASH_SOURCE[0]}")/env.sh"
LINUX="$ROOT/packaging/linux"
WIN="$ROOT/packaging/windows"
FAILED=0
fail() { echo "error: $*" >&2; FAILED=1; }
skip() { echo "warning: $* (skipped)" >&2; }

echo "==> AppStream metainfo"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
META="$WORK/$APP_ID.metainfo.xml"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@DATE@/$ANIMA_BUILD_DATE/g" "$LINUX/$APP_ID.metainfo.xml.in" >"$META"
if grep -q '@[A-Z_]*@' "$META"; then
  fail "the metainfo still has a placeholder after filling in VERSION and DATE"
fi
if command -v appstreamcli >/dev/null; then
  appstreamcli validate --no-net --explain "$META" || fail "appstreamcli validate found errors in the metainfo"
else
  skip "appstreamcli is not installed"
fi

echo "==> Desktop entry"
if command -v desktop-file-validate >/dev/null; then
  desktop-file-validate "$LINUX/$APP_ID.desktop" || fail "desktop-file-validate found errors"
else
  skip "desktop-file-validate is not installed"
fi

echo "==> winget and MSIX manifests"
found=0
if [[ -d "$WIN/winget" ]]; then
  while IFS= read -r -d '' f; do
    found=1
    for key in PackageIdentifier PackageVersion ManifestType ManifestVersion; do
      grep -Eq "^${key}:[[:space:]]*[^[:space:]]" "$f" || fail "$f: no $key"
    done
    if grep -Eq '^ManifestType:[[:space:]]*defaultLocale' "$f"; then
      for key in PackageLocale Publisher PackageName License ShortDescription; do
        grep -Eq "^${key}:[[:space:]]*[^[:space:]]" "$f" || fail "$f: no $key"
      done
    fi
    if grep -Eq '^ManifestType:[[:space:]]*installer' "$f"; then
      grep -Eq 'InstallerUrl:[[:space:]]*https://' "$f" || fail "$f: no https InstallerUrl"
      grep -Eq 'InstallerSha256:[[:space:]]*[0-9A-Fa-f]{64}' "$f" || fail "$f: no InstallerSha256"
    fi
  done < <(find "$WIN/winget" -name '*.yaml' -print0)
fi
if [[ -d "$WIN/msix" ]]; then
  while IFS= read -r -d '' f; do
    found=1
    for attr in 'Identity Name=' 'Publisher=' 'Version=' 'ProcessorArchitecture='; do
      grep -q "$attr" "$f" || fail "$f: no $attr"
    done
    for el in DisplayName PublisherDisplayName Logo; do
      grep -q "<$el>" "$f" || fail "$f: no <$el>"
    done
  done < <(find "$WIN/msix" -name 'AppxManifest.xml' -print0)
fi
[[ "$found" = 1 ]] || echo "no winget or MSIX manifests in packaging/windows yet"

if [[ "$FAILED" != 0 ]]; then
  echo "==> store readiness: errors above" >&2
  exit 1
fi
echo "==> store readiness: ok"
