# Lists every Ánima interface string for translators (ADR 0040, docs/localisation.md), as a
# nodalia-i18n catalog template. Same as scripts/i18n-extract.sh:
#
#   scripts/i18n-extract.ps1 [out.tsv]                     # default: target/i18n/anima.tsv
#   $env:NODALIA_I18N_DIR = 'C:\src\nodalia-i18n'; scripts/i18n-extract.ps1    # + coverage per locale
#   $env:ANIMA_I18N_LOCALE = 'de'; scripts/i18n-extract.ps1 de.tsv     # only what German lacks
param([string]$Out = '')
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
if ($Out -eq '') { $Out = Join-Path $root 'target/i18n/anima.tsv' }
elseif (-not [System.IO.Path]::IsPathRooted($Out)) { $Out = Join-Path (Get-Location) $Out }
Push-Location $root
try {
    $env:ANIMA_I18N_OUT = $Out
    cargo test --quiet --bin anima i18n::tests::extract -- --ignored --exact --nocapture
    if ($LASTEXITCODE -ne 0) { throw "extraction failed ($LASTEXITCODE)" }
} finally {
    Remove-Item Env:ANIMA_I18N_OUT -ErrorAction SilentlyContinue
    Pop-Location
}
