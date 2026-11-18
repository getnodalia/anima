#!/usr/bin/env bash
# Lists every Ánima interface string for translators (ADR 0040, docs/localisation.md), as a
# nodalia-i18n catalog template: the header, then one commented-out row per string
# (`#<context><TAB><English><TAB>`), for locales/<locale>/anima.tsv in getnodalia/foundation (vendor/nodalia-i18n).
# The strings come from the shell's tr!/trc!/trn!/tr_noop! calls and the command registry's
# labels and menus.
#
#   scripts/i18n-extract.sh [out.tsv]                      # default: target/i18n/anima.tsv
#   NODALIA_I18N_DIR=/abs/nodalia-i18n scripts/i18n-extract.sh  # also: how much each locale covers
#   NODALIA_I18N_DIR=/abs/nodalia-i18n ANIMA_I18N_LOCALE=de scripts/i18n-extract.sh de.tsv
#                                                          # only what German is missing
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUT="${1:-$ROOT/target/i18n/anima.tsv}"
case "$OUT" in
  /* | [A-Za-z]:*) ;;
  *) OUT="$PWD/$OUT" ;;
esac
ANIMA_I18N_OUT="$OUT" cargo test --quiet --bin anima i18n::tests::extract -- --ignored --exact --nocapture
