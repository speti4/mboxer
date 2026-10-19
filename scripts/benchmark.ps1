<#
.SYNOPSIS
Repeatable mboxer benchmark: index build (time, peak memory, size), re-open latency and a
fixed set of searches, written as a Markdown report.

.DESCRIPTION
Rebuilds the sidecar index of the given mbox (`index --force`), then opens it again, then runs
each benchmark query `-Repeat` times through `mboxer search` and reports the median open and
search time. Only counts and timings are recorded, never message content. Peak memory of the
build is the peak working set of the child process, sampled every 20 ms (see peak-memory.ps1);
the search processes are too short-lived to sample, so only the in-process timings are kept.

.EXAMPLE
scripts\benchmark.ps1 -Mbox "D:\mail\all.mbox" -Out docs\benchmarks\2026-10-17-14gb-takeout.md
#>
# PowerShell 7 reads this file as UTF-8 without a BOM; Windows PowerShell 5.1 would misread the accented
# benchmark queries below, so it stops with a clear message instead (#Requires applies wherever it stands).
#Requires -Version 7
param(
    [Parameter(Mandatory)] [string] $Mbox,
    [string] $Exe = (Join-Path $PSScriptRoot '..\target\release\mboxer-cli.exe'),
    [int] $Repeat = 3,
    [string] $Out
)

$ErrorActionPreference = 'Stop'
$Exe = (Resolve-Path $Exe).Path
$Mbox = (Resolve-Path $Mbox).Path

# One benchmark query per row: label, then the `search` arguments after the mbox path.
$queries = @(
    @{ Label = 'word, ASCII';            Args = @('szamla') },
    @{ Label = 'word, accented';         Args = @('számla') },
    @{ Label = 'phrase';                 Args = @('"számla melléklet"') },
    @{ Label = 'word + has attachment';  Args = @('meeting', '--has-attachment') },
    @{ Label = 'word + label';           Args = @('ajánlat', '--label', 'Inbox') },
    @{ Label = 'word + one year';        Args = @('számla', '--after', '2020-01-01', '--before', '2021-01-01') },
    @{ Label = 'filter only, count';     Args = @('--has-attachment', '--count') },
    @{ Label = 'word, by relevance';     Args = @('számla', '--relevance') }
)

function Run-Measured {
    <# Runs $Exe with $Arguments, returns stdout lines, elapsed seconds and peak working set in MB. #>
    param([string[]] $Arguments)
    $stdout = New-TemporaryFile
    # Windows argv quoting: wrap arguments with whitespace, escape embedded quotes so a phrase stays a phrase.
    $quoted = $Arguments | ForEach-Object { if ($_ -match '[\s"]') { '"' + $_.Replace('"', '\"') + '"' } else { $_ } }
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $process = Start-Process -FilePath $Exe -ArgumentList $quoted -PassThru -NoNewWindow `
        -RedirectStandardOutput $stdout.FullName
    $peak = 0
    while (-not $process.HasExited) {
        try {
            $process.Refresh()
            $peak = [Math]::Max($peak, $process.PeakWorkingSet64)
        } catch {}
        Start-Sleep -Milliseconds 20
    }
    $watch.Stop()
    if ($process.ExitCode -ne 0) { throw "mboxer $($Arguments -join ' ') exited with $($process.ExitCode)" }
    $lines = Get-Content -LiteralPath $stdout.FullName -Encoding utf8
    Remove-Item -LiteralPath $stdout.FullName
    [pscustomobject]@{ Lines = $lines; Seconds = $watch.Elapsed.TotalSeconds; PeakMB = $peak / 1MB }
}

function Get-Field {
    <# The value of a `key:   value` summary line. #>
    param([string[]] $Lines, [string] $Key)
    $line = $Lines | Where-Object { $_.StartsWith($Key) } | Select-Object -First 1
    if (-not $line) { throw "no `$Key` line in output" }
    $line.Substring($Key.Length).Trim()
}

function Get-Number {
    <# The leading number of a field value, e.g. "122.3 s (…)" → 122.3. #>
    param([string] $Value)
    [double](($Value -split '\s+')[0])
}

function Median {
    param([double[]] $Values)
    $sorted = $Values | Sort-Object
    $n = $sorted.Count
    if ($n % 2 -eq 1) { $sorted[($n - 1) / 2] } else { ($sorted[$n / 2 - 1] + $sorted[$n / 2]) / 2 }
}

# --- environment ---------------------------------------------------------------------------
$cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1).Name.Trim()
$ramGb = [Math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB, 0)
$os = Get-CimInstance Win32_OperatingSystem
$disk = 'unknown'
try {
    $drive = (Split-Path -Qualifier $Mbox).TrimEnd(':')
    $diskNumber = (Get-Partition -DriveLetter $drive).DiskNumber
    $physical = Get-PhysicalDisk | Where-Object { $_.DeviceId -eq "$diskNumber" } | Select-Object -First 1
    if ($physical) { $disk = "$($physical.MediaType) ($($physical.BusType))" }
} catch {}
$version = (& $Exe --version) -join ' '
$commit = try { (git -C $PSScriptRoot rev-parse --short HEAD 2>$null) } catch { 'n/a' }
if (git -C $PSScriptRoot status --porcelain 2>$null) { $commit += '-dirty' }
$mboxBytes = (Get-Item -LiteralPath $Mbox).Length

Write-Host "benchmark: $version @ $commit on $cpu, mbox $([Math]::Round($mboxBytes / 1GB, 2)) GB"

# --- build ---------------------------------------------------------------------------------
Write-Host 'index --force …'
$build = Run-Measured @('index', $Mbox, '--force', '--quiet')
$messages = Get-Field $build.Lines 'messages:'
$parseFailures = Get-Field $build.Lines 'parse failures:'
$buildSeconds = Get-Number (Get-Field $build.Lines 'elapsed:')
$indexBytes = [long](Get-Number (Get-Field $build.Lines 'index size:'))
$threads = Get-Field $build.Lines 'threads:'

Write-Host 'index (fresh, no rebuild) …'
$reopenLines = @()
$reopen = Measure-Command { $reopenLines = & $Exe index $Mbox --quiet }
if (-not ($reopenLines -join "`n").Contains('up to date')) { throw 'second index run rebuilt the index' }

# --- searches ------------------------------------------------------------------------------
$rows = foreach ($q in $queries) {
    Write-Host "search $($q.Args -join ' ') …"
    $opens = @(); $searches = @(); $hits = $null
    for ($i = 0; $i -lt $Repeat; $i++) {
        $run = Run-Measured (@('search', $Mbox) + $q.Args)
        $opens += Get-Number (Get-Field $run.Lines 'open:')
        $searches += Get-Number (Get-Field $run.Lines 'search:')
        $hits = Get-Field $run.Lines 'hits:'
    }
    [pscustomobject]@{
        Label = $q.Label
        Query = '`' + ($q.Args -join ' ') + '`'
        Hits = $hits
        OpenMs = Median $opens
        SearchMs = Median $searches
    }
}

# --- report --------------------------------------------------------------------------------
$report = @()
$report += "_Generated $(Get-Date -Format 'yyyy-MM-dd HH:mm') by ``scripts/benchmark.ps1``; $version, commit $commit._"
$report += ''
$report += '## Environment'
$report += ''
$report += "- CPU: $cpu; RAM: $ramGb GB; OS: $($os.Caption.Trim()) $($os.Version)"
$report += "- mbox: $([Math]::Round($mboxBytes / 1GB, 2)) GB on $disk; messages: $messages; parse failures: $parseFailures"
$report += "- indexing threads: $threads; search repeats: $Repeat (median reported)"
$report += ''
$report += '## Index build and re-open'
$report += ''
$report += '| Measurement | Value |'
$report += '|---|---|'
$report += "| build time (``index --force``) | $([Math]::Round($buildSeconds, 1)) s ($([Math]::Round($mboxBytes / 1e6 / $buildSeconds, 0)) MB/s) |"
$report += "| peak working set while building | $([Math]::Round($build.PeakMB, 1)) MB |"
$report += "| index size | $([Math]::Round($indexBytes / 1MB, 1)) MB ($([Math]::Round($indexBytes * 100.0 / $mboxBytes, 2)) % of the mbox) |"
$report += "| re-open, fresh index (``index``, whole process) | $([Math]::Round($reopen.TotalMilliseconds, 0)) ms |"
$report += ''
$report += '## Searches'
$report += ''
$report += '| Case | Query | Hits | Open | Search |'
$report += '|---|---|---:|---:|---:|'
foreach ($r in $rows) {
    $report += "| $($r.Label) | $($r.Query) | $($r.Hits) | $([Math]::Round($r.OpenMs, 1)) ms | $([Math]::Round($r.SearchMs, 1)) ms |"
}
$report += ''
$report += '_Open = `MboxIndex::open` (fingerprint check + mmap), Search = query execution; both inside one `mboxer search` process, file cache warm._'

$text = $report -join "`n"
if ($Out) {
    $utf8 = New-Object System.Text.UTF8Encoding $false
    $outPath = if ([IO.Path]::IsPathRooted($Out)) { $Out } else { Join-Path (Get-Location) $Out }
    [IO.File]::WriteAllText($outPath, $text + "`n", $utf8)
    Write-Host "written: $Out"
} else {
    $text
}
