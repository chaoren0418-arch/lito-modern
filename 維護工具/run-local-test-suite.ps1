[CmdletBinding()]
param([switch]$Quick)

# Keep this file compatible with Windows PowerShell 5.1 as well as PowerShell 7.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 2.0
$repoRoot = Split-Path -Parent $PSScriptRoot
$project = Join-Path $repoRoot 'LiTo381-Modern'
$scratch = Join-Path $project 'target/current/scratch'
$utf8 = New-Object System.Text.UTF8Encoding($false)
$invariant = [Globalization.CultureInfo]::InvariantCulture

function Read-Whitelist {
    param([string]$Path, [switch]$Integration)
    $lines = [IO.File]::ReadAllLines($Path, [Text.Encoding]::UTF8)
    if ($lines.Length -eq 0) { throw "Empty whitelist: $Path" }
    $seen = @{}
    foreach ($line in $lines) {
        if ($Integration) {
            $fields = $line.Split([char]9)
            if ($fields.Length -ne 4 -or $fields[0] -notmatch '^[A-Za-z0-9_]+$' -or
                $fields[1] -notmatch '^[A-Za-z0-9_:]+$' -or $fields[2] -notmatch '^(yes|no)$' -or
                $fields[3] -notmatch '^(?:-|[A-Za-z0-9_-]+(?:,[A-Za-z0-9_-]+)*)$') {
                throw "Invalid four-column whitelist row: $line"
            }
            $key = $fields[0] + '::' + $fields[1]
            if ($seen.ContainsKey($key)) { throw "Duplicate whitelist item: $key" }
            $seen[$key] = $true
            [pscustomobject]@{ Target = $fields[0]; Name = $fields[1]; Features = $fields[3] }
        } else {
            if ($line -notmatch '^[A-Za-z0-9_:]+$') { throw "Invalid lib whitelist row: $line" }
            if ($seen.ContainsKey($line)) { throw "Duplicate whitelist item: $line" }
            $seen[$line] = $true
            $line
        }
    }
}

function Measure-TestResult {
    param([string]$Output, [int]$ExitCode, [ValidateSet('Check','Lib','Exact')][string]$Kind)
    $reasons = New-Object 'System.Collections.Generic.List[string]'
    if ($ExitCode -ne 0) { $reasons.Add('command-exit') }
    $passed = 'N/A'; $failed = 'N/A'; $ignored = 'N/A'; $zero = 'N/A'
    if ($Kind -ne 'Check') {
        $running = [regex]::Matches($Output, '(?m)^running (\d+) tests?\s*\r?$')
        $zero = @($running | Where-Object { [long]$_.Groups[1].Value -eq 0 }).Count
        $resultLines = [regex]::Matches($Output, '(?m)^test result:.*$')
        $results = [regex]::Matches($Output, '(?m)^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out; finished in [0-9.]+s\s*\r?$')
        if ($zero -gt 0) { $reasons.Add('running-0-tests') }
        # One cargo --lib/--test invocation must report exactly one complete harness.
        if ($resultLines.Count -ne 1 -or $results.Count -ne 1) { $reasons.Add('missing-or-invalid-result') }
        if ($running.Count -ne 1) { $reasons.Add('missing-or-invalid-running') }
        if ($results.Count -eq 1) {
            $result = $results[0]
            $passed = [long]$result.Groups[2].Value
            $failed = [long]$result.Groups[3].Value
            $ignored = [long]$result.Groups[4].Value
            $measured = [long]$result.Groups[5].Value
            if ($failed -gt 0 -or $result.Groups[1].Value -ne 'ok') { $reasons.Add('test-failed') }
            if ($running.Count -eq 1 -and [long]$running[0].Groups[1].Value -ne ($passed + $failed + $ignored + $measured)) {
                $reasons.Add('count-mismatch')
            }
            if ($Kind -eq 'Exact' -and ($passed -ne 1 -or $failed -ne 0 -or $ignored -ne 0 -or $measured -ne 0)) {
                $reasons.Add('expected-1-passed')
            }
        }
        if ($Kind -eq 'Exact' -and ($running.Count -ne 1 -or $running[0].Groups[1].Value -ne '1')) {
            $reasons.Add('expected-running-1')
        }
    }
    $status = 'PASS'
    if ($reasons.Count -gt 0) { $status = 'FAIL: ' + ($reasons -join ',') }
    [pscustomobject]@{ Passed = $passed; Failed = $failed; Ignored = $ignored; Zero = $zero; Status = $status }
}

function Invoke-SuiteStep {
    param([string]$Name, [string[]]$CargoArgs, [string]$Kind, [int]$Index)
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $logPath = Join-Path $outputDirectory ('{0:D3}.log' -f $Index)
    $command = 'cargo ' + ($CargoArgs -join ' ')
    Write-Host "[$Index] $Name"
    Write-Host $command
    $exitCode = -1
    $output = ''
    $process = New-Object Diagnostics.Process
    try {
        $info = New-Object Diagnostics.ProcessStartInfo
        $info.FileName = $cargoPath
        # All arguments are fixed tokens or strictly validated whitelist identifiers.
        $info.Arguments = $CargoArgs -join ' '
        $info.WorkingDirectory = $project
        $info.UseShellExecute = $false
        $info.CreateNoWindow = $true
        $info.RedirectStandardOutput = $true
        $info.RedirectStandardError = $true
        $info.StandardOutputEncoding = $utf8
        $info.StandardErrorEncoding = $utf8
        # Modify only the child's environment; the caller's values never change.
        foreach ($key in @($info.EnvironmentVariables.Keys)) {
            if ($key -like 'LITO_*') { $info.EnvironmentVariables.Remove($key) }
        }
        $info.EnvironmentVariables['TEMP'] = $scratch
        $info.EnvironmentVariables['TMP'] = $scratch
        $info.EnvironmentVariables['CARGO_TERM_COLOR'] = 'never'
        $process.StartInfo = $info
        if (-not $process.Start()) { throw 'Could not start cargo' }
        # Drain both pipes concurrently so compiler diagnostics cannot deadlock the process.
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $lastHeartbeat = 0
        while (-not $process.WaitForExit(1000)) {
            if ($timer.Elapsed.TotalSeconds -ge ($lastHeartbeat + 30)) {
                $lastHeartbeat = [int]$timer.Elapsed.TotalSeconds
                Write-Host "[$Index] still running ($lastHeartbeat s)"
            }
        }
        $process.WaitForExit()
        $output = $stdout.GetAwaiter().GetResult() + "`n" + $stderr.GetAwaiter().GetResult()
        $exitCode = $process.ExitCode
    } catch {
        $output += "`nRunner error: " + $_.Exception.Message
    } finally {
        $process.Dispose()
        $timer.Stop()
    }
    [IO.File]::WriteAllText($logPath, ($command + "`n" + $output), $utf8)
    $result = Measure-TestResult -Output $output -ExitCode $exitCode -Kind $Kind
    Write-Host ("[$Index] {0}; exit={1}; log={2}" -f $result.Status, $exitCode, $logPath)
    [pscustomobject]@{
        Step = $Name; ExitCode = $exitCode; Passed = $result.Passed; Failed = $result.Failed
        Ignored = $result.Ignored; Zero = $result.Zero
        Seconds = $timer.Elapsed.TotalSeconds.ToString('F3', $invariant); Status = $result.Status
    }
}

try {
    if (-not (Test-Path -LiteralPath (Join-Path $project 'Cargo.toml') -PathType Leaf)) { throw 'Cargo.toml not found' }
    $cargo = Get-Command cargo -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($cargo) {
        $cargoPath = $cargo.Source
    } elseif ($env:CARGO_HOME -and (Test-Path -LiteralPath (Join-Path $env:CARGO_HOME 'bin/cargo.exe') -PathType Leaf)) {
        $cargoPath = Join-Path $env:CARGO_HOME 'bin/cargo.exe'
    } else {
        throw 'cargo not found on PATH or in CARGO_HOME/bin'
    }
    $libNames = @(); $testEntries = @()
    if (-not $Quick) {
        $libNames = @(Read-Whitelist -Path (Join-Path $PSScriptRoot 'test-whitelist/lib.txt'))
        $testEntries = @(Read-Whitelist -Path (Join-Path $PSScriptRoot 'test-whitelist/tests.tsv') -Integration)
    }
    $stamp = (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + '-' + $PID
    $outputDirectory = Join-Path $project ('target/current/verification/local-test-suite/' + $stamp)
    New-Item -ItemType Directory -Path $scratch,$outputDirectory -Force | Out-Null
    $common = @('--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current')
    $steps = New-Object 'System.Collections.Generic.List[object]'
    $steps.Add(@{ Name = 'check'; Kind = 'Check'; Args = @('check') + $common + @('--all-targets') })
    $libArgs = @('test') + $common + @('--lib','--no-fail-fast')
    $steps.Add(@{ Name = 'lib-debug'; Kind = 'Lib'; Args = $libArgs })
    if (-not $Quick) {
        $steps.Add(@{ Name = 'lib-release'; Kind = 'Lib'; Args = $libArgs + @('--release') })
        foreach ($name in $libNames) {
            $steps.Add(@{ Name = 'ignored-lib:' + $name; Kind = 'Exact'; Args = $libArgs + @('--','--ignored','--exact',$name) })
        }
        foreach ($entry in $testEntries) {
            $testArgs = @('test') + $common + @('--test',$entry.Target)
            if ($entry.Features -ne '-') { $testArgs += @('--features',$entry.Features) }
            $testArgs += @('--no-fail-fast','--','--include-ignored','--exact',$entry.Name)
            $steps.Add(@{ Name = $entry.Target + '::' + $entry.Name; Kind = 'Exact'; Args = $testArgs })
        }
    }
    $rows = New-Object 'System.Collections.Generic.List[object]'
    $index = 0
    foreach ($step in $steps) {
        $index++
        $rows.Add((Invoke-SuiteStep -Name $step.Name -CargoArgs $step.Args -Kind $step.Kind -Index $index))
    }
    $finalExit = 0
    if (@($rows | Where-Object { $_.Status -ne 'PASS' }).Count -gt 0) { $finalExit = 1 }
    $mode = 'Full'
    if ($Quick) { $mode = 'Quick' }
    $summary = New-Object 'System.Collections.Generic.List[string]'
    $summary.Add("LOCAL TEST SUITE | Mode=$mode | PowerShell=$($PSVersionTable.PSVersion)")
    $summary.Add('Step | Exit | passed | failed | ignored | running-0-tests | Seconds | Result')
    foreach ($row in $rows) {
        $summary.Add(('{0} | {1} | {2} | {3} | {4} | {5} | {6} | {7}' -f
            $row.Step,$row.ExitCode,$row.Passed,$row.Failed,$row.Ignored,$row.Zero,$row.Seconds,$row.Status))
    }
    $summary.Add("Steps=$($rows.Count) | Final exit code=$finalExit")
    $summaryText = ($summary -join "`n") + "`n"
    $summaryPath = Join-Path $outputDirectory 'summary.txt'
    [IO.File]::WriteAllText($summaryPath, $summaryText, $utf8)
    Write-Output $summaryText
    Write-Host "Summary: $summaryPath"
    exit $finalExit
} catch {
    Write-Error $_ -ErrorAction Continue
    exit 1
}
