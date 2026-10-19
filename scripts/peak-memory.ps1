<#
.SYNOPSIS
Runs a command and reports its peak working set.

.EXAMPLE
scripts\peak-memory.ps1 -- target\release\mboxer-cli.exe scan "D:\mail\all.mbox" --quiet
#>
param(
    [Parameter(Mandatory, ValueFromRemainingArguments)]
    [string[]] $Command
)

$exe, $rest = $Command
# Start-Process joins the arguments with spaces and does not quote them itself: wrap arguments with whitespace or
# quotes, escape embedded quotes so a phrase stays a phrase (the quoting of benchmark.ps1).
$quoted = $rest | ForEach-Object { if ($_ -match '[\s"]') { '"' + $_.Replace('"', '\"') + '"' } else { $_ } }
$process = Start-Process -FilePath $exe -ArgumentList $quoted -PassThru -NoNewWindow
$peak = 0
while (-not $process.HasExited) {
    try {
        $process.Refresh()
        $peak = [Math]::Max($peak, $process.PeakWorkingSet64)
    } catch {}
    Start-Sleep -Milliseconds 200
}
"peak working set: {0:N1} MB (exit code {1})" -f ($peak / 1MB), $process.ExitCode
