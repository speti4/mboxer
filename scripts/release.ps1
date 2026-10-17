<#
.SYNOPSIS
Builds the release exe and packs the portable zip: dist\mboxer-<version>.zip (task 012).

.DESCRIPTION
The version is the workspace Cargo.toml one, the only place it is written. The zip holds
mboxer.exe and usage.md (docs\usage.md). CI runs this script on a v<version> tag with -Tag
set, which must match the version; by hand the tag check is skipped.

.EXAMPLE
scripts\release.ps1
scripts\release.ps1 -SkipBuild -Out C:\tmp\dist
#>
[CmdletBinding()]
param(
    # Pack what target\release already holds instead of building.
    [switch] $SkipBuild,
    # Where the zip goes.
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

if (-not $SkipBuild) {
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargo) {
        $fallback = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
        if (Test-Path $fallback) { $cargo = Get-Command $fallback } else { throw "cargo not found" }
    }
    & $cargo.Source build --release -p mboxer-app
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

New-Item -ItemType Directory -Force $Out | Out-Null
$zip = Join-Path (Resolve-Path $Out) "mboxer-$version.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zip -CompressionLevel Optimal
Remove-Item -Recurse -Force $stage

$exeMb = [math]::Round((Get-Item $exe).Length / 1MB, 1)
$zipMb = [math]::Round((Get-Item $zip).Length / 1MB, 1)
Write-Host "version:  $version (exe properties: $stamped)"
Write-Host "exe:      $exe ($exeMb MB)"
Write-Host "zip:      $zip ($zipMb MB)"
