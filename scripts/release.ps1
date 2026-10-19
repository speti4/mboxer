<#
.SYNOPSIS
Builds the release exe and packs the portable zip: dist\mboxer-<version>.zip (task 012), with
its checksum and its release notes next to it (task 025).

.DESCRIPTION
The version is the workspace Cargo.toml one, the only place it is written. The zip holds
mboxer.exe, usage.md (docs\usage.md) and THIRD-PARTY-LICENSES.html (cargo-about over the crates
compiled into the exe, with about.toml and scripts\third-party-licenses.hbs). Next to the zip go
SHA256SUMS.txt (the zip's SHA-256 in `sha256sum -c` format) and release-notes-<version>.md
(scripts\changelog.py --notes, the body of the GitHub release). CI runs this script on a
v<version> tag with -Tag set, which must match the version; by hand the tag check is skipped.
Needs cargo, cargo-about (cargo install cargo-about --version 0.9.2 --locked --features cli),
Python 3 and git with the release tags.

.EXAMPLE
scripts\release.ps1
scripts\release.ps1 -SkipBuild -Out C:\tmp\dist
#>
[CmdletBinding()]
param(
    # Pack what target\release already holds instead of building.
    [switch] $SkipBuild,
    # Where the zip, the checksum file and the notes go.
    [string] $Out = "dist",
    # The git tag being released (CI); must equal "v<version>".
    [string] $Tag = ""
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$cargoToml = Get-Content (Join-Path $root "Cargo.toml") -Raw
if ($cargoToml -notmatch '(?m)^\[workspace\.package\][^\[]*?^version = "([^"]+)"') {
    throw "no [workspace.package] version in Cargo.toml"
}
$version = $Matches[1]
if ($Tag -and $Tag -ne "v$version") {
    throw "tag '$Tag' does not match the Cargo.toml version 'v$version'"
}

# A tool from PATH, else from ~\.cargo\bin (the tool shells here may run with a PATH that lacks it).
function Find-Tool([string] $Name) {
    $tool = Get-Command $Name -ErrorAction SilentlyContinue
    if (-not $tool) {
        $fallback = Join-Path $env:USERPROFILE ".cargo\bin\$Name.exe"
        if (Test-Path $fallback) { $tool = Get-Command $fallback } else { throw "$Name not found" }
    }
    $tool.Source
}

# What can fail fast runs before the build: the tools and the release notes.
$cargo = Find-Tool "cargo"
$cargoAbout = Find-Tool "cargo-about"
$python = (Get-Command python -ErrorAction SilentlyContinue).Source
if (-not $python) { throw "python not found" }

New-Item -ItemType Directory -Force $Out | Out-Null
$outDir = (Resolve-Path $Out).Path
$notes = Join-Path $outDir "release-notes-$version.md"
& $python -I (Join-Path $root "scripts\changelog.py") --notes $version --out $notes
if ($LASTEXITCODE -ne 0) { throw "release notes failed ($LASTEXITCODE)" }

if (-not $SkipBuild) {
    & $cargo build --release -p mboxer-app
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
}

$exe = Join-Path $root "target\release\mboxer.exe"
$guide = Join-Path $root "docs\usage.md"
foreach ($file in @($exe, $guide)) {
    if (-not (Test-Path $file)) { throw "missing: $file" }
}

$stamped = (Get-Item $exe).VersionInfo.ProductVersion
if ($stamped -ne $version) {
    throw "the exe carries version '$stamped', Cargo.toml says '$version' (stale build?)"
}

$stage = Join-Path ([System.IO.Path]::GetTempPath()) "mboxer-release-$version"
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item $exe (Join-Path $stage "mboxer.exe")
Copy-Item $guide (Join-Path $stage "usage.md")

# The notices of the crates compiled into the exe (their MIT, Apache-2.0, BSD and MPL-2.0 licences ask for
# them). cargo-about reads the graph through `cargo metadata`, which wants the sources of every platform's
# crates, more than the build fetched (task 041), so `cargo fetch` (all targets) runs first; then --frozen:
# Cargo.lock as it is and no network, so the same lock file gives the same page.
& $cargo fetch --locked
if ($LASTEXITCODE -ne 0) { throw "cargo fetch failed ($LASTEXITCODE)" }
$licences = Join-Path $stage "THIRD-PARTY-LICENSES.html"
& $cargoAbout generate --frozen --fail `
    --manifest-path (Join-Path $root "crates\mboxer-app\Cargo.toml") `
    --config (Join-Path $root "about.toml") `
    --output-file $licences `
    (Join-Path $root "scripts\third-party-licenses.hbs")
if ($LASTEXITCODE -ne 0) { throw "cargo-about failed ($LASTEXITCODE)" }
$licencesKb = [math]::Round((Get-Item $licences).Length / 1KB)

$zip = Join-Path $outDir "mboxer-$version.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zip -CompressionLevel Optimal
Remove-Item -Recurse -Force $stage

# "<sha256>  <file name>" and a LF, as sha256sum writes it, so `sha256sum -c SHA256SUMS.txt` checks the
# download; Get-FileHash prints the same value in capitals.
$hash = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLowerInvariant()
$sums = Join-Path $outDir "SHA256SUMS.txt"
[System.IO.File]::WriteAllText($sums, "$hash  mboxer-$version.zip`n", (New-Object System.Text.UTF8Encoding $false))

$exeMb = [math]::Round((Get-Item $exe).Length / 1MB, 1)
$zipMb = [math]::Round((Get-Item $zip).Length / 1MB, 1)
Write-Host "version:  $version (exe properties: $stamped)"
Write-Host "exe:      $exe ($exeMb MB)"
Write-Host "zip:      $zip ($zipMb MB; THIRD-PARTY-LICENSES.html $licencesKb KB inside)"
Write-Host "sha256:   $hash ($sums)"
Write-Host "notes:    $notes"
