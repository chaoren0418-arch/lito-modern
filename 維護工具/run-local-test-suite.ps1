[CmdletBinding()]
param(
    [switch]$Quick,
    [ValidateSet('All','Baseline','Skills','G06')][string]$Category = 'All',
    [string]$ReferenceJar,
    [string]$PowerShellPath
)

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

function Read-SkillEntries {
    param(
        [string]$StatusPath = (Join-Path $repoRoot 'docs/SKILLS-STATUS.md'),
        [string]$ManifestPath = (Join-Path $PSScriptRoot 'test-whitelist/skills.tsv')
    )
    # The reviewed manifest resolves short names, duplicate names across targets,
    # nested modules and include! tests. Never discover/run arbitrary Rust tests.
    $entries = @(Read-Whitelist $ManifestPath -Integration)
    $names = @{}; $ids = @{}; $withoutTests = 0; $inTable = $false
    foreach ($line in [IO.File]::ReadAllLines($StatusPath, [Text.Encoding]::UTF8)) {
        if ($line -match '^\|\s*ID\s*\|') { $inTable = $true; continue }
        if (-not $inTable) { continue }
        if ($line -notmatch '^\|') { break }
        if ($line -notmatch '^\|\s*(\d+)\s*\|') { continue }
        $id = $Matches[1]; $cells = $line.Split([char]'|')
        if ($cells.Length -ne 17 -or $ids.ContainsKey($id)) { throw "Invalid skill row: $id" }
        $ids[$id] = $true
        $refs = [regex]::Matches($cells[12], '`([^`]+)`')
        if ($refs.Count -eq 0) {
            # Only the explicit T026 'no test yet' marker may omit a test.
            $noTest = ([char]0x5c1a).ToString() + [char]0x7121
            if ($cells[12].Trim() -cne $noTest) { throw "Unparseable skill tests: $id" }
            $withoutTests++; continue
        }
        foreach ($ref in $refs) {
            if ($ref.Groups[1].Value -notmatch '^([A-Za-z0-9_:]+)(?:$|\s|\uFF08)') { throw "Invalid skill test: $id" }
            $names[$Matches[1]] = $true
        }
    }
    if ($ids.Count -ne 181 -or $names.Count -eq 0) { throw 'Expected 181 skill rows and nonempty test references' }
    $mapped = @{}
    foreach ($entry in $entries) {
        $short = ($entry.Name -split '::')[-1]
        if (-not $names.ContainsKey($short) -and -not $names.ContainsKey($entry.Name)) {
            throw "Stale skill manifest entry: $($entry.Target)::$($entry.Name)"
        }
        $mapped[$short] = $true; $mapped[$entry.Name] = $true
    }
    foreach ($name in $names.Keys) {
        if (-not $mapped.ContainsKey($name)) { throw "Unmapped T026 test: $name; review and update test-whitelist/skills.tsv" }
    }
    Write-Host "Skills: $($ids.Count) catalog rows; $($names.Count) references; $($entries.Count) exact tests; $withoutTests rows have no tests (not verified)."
    $entries
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

function Invoke-SuiteProcess {
    param([string]$Name, [string]$Executable, [string[]]$Arguments, [int]$Index)
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $logPath = Join-Path $outputDirectory ('{0:D3}.log' -f $Index)
    # Windows paths cannot contain quotes. Quote every token and escape trailing
    # backslashes so spaces in worktree/tool paths survive on PowerShell 5.1 too.
    $quoted = @($Arguments | ForEach-Object {
        if ($_ -match '["\r\n]') { throw 'Invalid process argument' }
        '"' + [regex]::Replace($_, '(\\+)$', '$1$1') + '"'
    })
    $command = '"' + $Executable + '" ' + ($quoted -join ' ')
    Write-Host "[$Index] $Name"
    Write-Host $command
    $exitCode = -1
    $output = ''
    $process = New-Object Diagnostics.Process
    try {
        $info = New-Object Diagnostics.ProcessStartInfo
        $info.FileName = $Executable
        $info.Arguments = $quoted -join ' '
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
    [pscustomobject]@{ Output = $output; ExitCode = $exitCode; Seconds = $timer.Elapsed.TotalSeconds; Log = $logPath }
}

function Invoke-SuiteStep {
    param([string]$Name, [string[]]$CargoArgs, [string]$Kind, [int]$Index, [string]$Group = 'Baseline')
    $run = Invoke-SuiteProcess -Name $Name -Executable $cargoPath -Arguments $CargoArgs -Index $Index
    $exitCode = $run.ExitCode; $logPath = $run.Log
    $result = Measure-TestResult -Output $run.Output -ExitCode $exitCode -Kind $Kind
    Write-Host ("[$Index] {0}; exit={1}; log={2}" -f $result.Status, $exitCode, $logPath)
    [pscustomobject]@{
        Category = $Group; Step = $Name; ExitCode = $exitCode; Passed = $result.Passed; Failed = $result.Failed
        Ignored = $result.Ignored; Zero = $result.Zero
        Seconds = $run.Seconds.ToString('F3', $invariant); Status = $result.Status
    }
}

function New-FailedRow {
    param([string]$Group, [string]$Name, [string]$Reason)
    [pscustomobject]@{ Category = $Group; Step = $Name; ExitCode = 'N/A'; Passed = 'N/A'; Failed = 'N/A'
        Ignored = 'N/A'; Zero = 'N/A'; Seconds = '0.000'; Status = 'FAIL: ' + $Reason }
}

function Assert-ArtifactPath {
    param([string]$Path, [string]$Root)
    $full = [IO.Path]::GetFullPath($Path)
    $base = [IO.Path]::GetFullPath($Root).TrimEnd('\','/')
    if (-not $full.StartsWith($base + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Artifact outside expected root: $full"
    }
    $cursor = $full
    while ($true) {
        $item = Get-Item -LiteralPath $cursor -Force
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "ReparsePoint: $cursor" }
        if ($cursor -eq $base) { break }
        $cursor = Split-Path -Parent $cursor
    }
    $full
}

function Read-G06SupportedScenarios {
    param([string]$RunnerPath)
    # Get-Command cannot expose a #Requires 7 script's metadata on PS 5.1.
    # Read literal ValidateSet values without executing the owned runner.
    $tokens = $null; $errors = $null
    $ast = [Management.Automation.Language.Parser]::ParseFile($RunnerPath, [ref]$tokens, [ref]$errors)
    if ($errors.Count -gt 0) { throw 'Cannot parse owned G06 runner' }
    $parameter = @($ast.ParamBlock.Parameters | Where-Object { $_.Name.VariablePath.UserPath -eq 'Scenario' })
    if ($parameter.Count -ne 1) { throw 'Missing G06 runner scenario parameter' }
    $attributes = @($parameter[0].Attributes | Where-Object { $_.TypeName.FullName -eq 'ValidateSet' })
    if ($attributes.Count -ne 1) { throw 'Missing G06 runner scenario contract' }
    foreach ($argument in $attributes[0].PositionalArguments) {
        $value = $argument.SafeGetValue()
        if ($value -isnot [string] -or $value -notmatch '^[a-z-]+$') { throw 'Invalid G06 scenario token' }
        $value
    }
}

function Invoke-G06Suite {
    param([int]$StartIndex)
    # Names match native_E149_recovery_contract.py. The last three are supplied
    # by T023-2; unsupported scenarios must remain visible failures, never skips.
    $scenarios = [ordered]@{
        'timeout' = 'destination_timeout_retires_old_authority_and_allows_fresh_login'
        'refused' = 'refused_destination_retires_authority_and_allows_fresh_login'
        'wrong-service' = 'wrong_service_destination_retires_authority_and_allows_fresh_login'
        'cancel-retry' = 'cancellation_then_immediate_retry_consumes_staged_character_once'
        'cancel-expiry' = 'cancellation_retires_destination_and_expired_handoff_requires_fresh_selection'
        'cancel-repeat' = 'repeated_cancellation_preserves_only_one_authenticated_recovery'
        'refused-cancel-repeat' = 'failed_handoff_then_repeated_cancel_preserves_the_authenticated_recovery'
        'cancel-expiry-source' = 'expired_stage_and_source_handoff_require_the_requested_source_directory'
    }
    $index = $StartIndex
    try {
        # Check the complete contract before building or starting any harness.
        # Five passing scenarios cannot satisfy the eight-scenario M2 gate.
        $runnerPath = Join-Path $PSScriptRoot 'run-teleport-failure.ps1'
        $supported = @(Read-G06SupportedScenarios $runnerPath)
        $missing = @($scenarios.Keys | Where-Object { $_ -cnotin $supported })
        if ($missing.Count -gt 0) {
            throw ('incomplete-owned-scenarios (T023-2): ' + ($missing -join ', '))
        }
        if (-not $PowerShellPath) {
            if ($PSVersionTable.PSVersion.Major -ge 7) { $PowerShellPath = Join-Path $PSHOME 'pwsh.exe' }
            else { $PowerShellPath = (Get-Command pwsh -CommandType Application -ErrorAction Stop).Source }
        }
        $probe = Invoke-SuiteProcess 'G06-pwsh-version' $PowerShellPath @('-NoProfile','-Command','$PSVersionTable.PSVersion.ToString()') (++$index)
        if ($probe.ExitCode -ne 0 -or [version]$probe.Output.Trim() -lt [version]'7.0') { throw 'G06 requires PowerShell 7 or later' }
        Write-Host "G06 PowerShell: $PowerShellPath | $($probe.Output.Trim())"
        if (-not $ReferenceJar) { $ReferenceJar = Join-Path $project 'target/current/server-reference/formal-server-E1498871.jar' }
        $ReferenceJar = Assert-ArtifactPath $ReferenceJar (Join-Path $project 'target/current')
        $jarSha = (Get-FileHash -LiteralPath $ReferenceJar -Algorithm SHA256).Hash
        if ($jarSha -cne 'E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498') { throw 'G06 requires reviewed E149 JAR' }
        $buildArgs = @('test') + $common + @('--test','native_teleport_failure','--no-run','--message-format=json')
        $build = Invoke-SuiteProcess 'G06-build' $cargoPath $buildArgs (++$index)
        if ($build.ExitCode -ne 0) { throw "G06 build exit=$($build.ExitCode); log=$($build.Log)" }
        $binaries = @($build.Output -split "`n" | Where-Object { $_.StartsWith('{') } | ForEach-Object {
            $message = $_ | ConvertFrom-Json
            if ($message.reason -eq 'compiler-artifact' -and $message.target.name -eq 'native_teleport_failure' -and $message.profile.test -and $message.executable) { $message.executable }
        } | Select-Object -Unique)
        if ($binaries.Count -ne 1) { throw 'Expected one Cargo test executable artifact' }
        $binary = Assert-ArtifactPath $binaries[0] (Join-Path $project 'target/current')
        # --list only enumerates tests; it does not run ignored network fixtures.
        # Catch a runner updated ahead of its Rust tests before starting Java.
        $inventory = Invoke-SuiteProcess 'G06-test-inventory' $binary @('--list','--ignored','--format','terse') (++$index)
        if ($inventory.ExitCode -ne 0) { throw "G06 inventory exit=$($inventory.ExitCode); log=$($inventory.Log)" }
        foreach ($scenario in $scenarios.Keys) {
            $pattern = '(?m)^' + [regex]::Escape($scenarios[$scenario]) + ': test\r?$'
            if ([regex]::Matches($inventory.Output, $pattern).Count -ne 1) {
                throw "missing-or-duplicate-G06-test: $scenario -> $($scenarios[$scenario]); log=$($inventory.Log)"
            }
        }
    } catch {
        foreach ($scenario in $scenarios.Keys) { New-FailedRow 'G06' $scenario ('precondition: ' + $_.Exception.Message) }
        return
    }
    $stopped = $false
    foreach ($scenario in $scenarios.Keys) {
        if ($stopped) { New-FailedRow 'G06' $scenario 'not-run-after-G06-failure'; continue }
        # Redirected pwsh can inherit the Windows ANSI console encoding. Force
        # UTF-8 before invoking the runner so Chinese diagnostics are preserved.
        $quotedPaths = @($runnerPath,$binary,$ReferenceJar,$scenario | ForEach-Object { "'" + $_.Replace("'","''") + "'" })
        $invoke = '[Console]::OutputEncoding = New-Object Text.UTF8Encoding($false); & ' + $quotedPaths[0] +
            ' -TestBinary ' + $quotedPaths[1] + ' -ReferenceJar ' + $quotedPaths[2] + ' -Scenario ' + $quotedPaths[3]
        $run = Invoke-SuiteProcess ('G06:' + $scenario) $PowerShellPath @('-NoProfile','-Command',$invoke) (++$index)
        try {
            if ($run.ExitCode -ne 0) { throw "owned-runner-exit=$($run.ExitCode); log=$($run.Log)" }
            $evidence = Assert-ArtifactPath $run.Output.Trim() (Join-Path $project 'target/current/verification/native-teleport-failure')
            $receiptPath = Assert-ArtifactPath (Join-Path $evidence 'receipt.json') $evidence
            $receipt = [IO.File]::ReadAllText($receiptPath) | ConvertFrom-Json
            if ($receipt.status -cne ('verified_' + $scenario) -or $receipt.scenario -cne $scenario -or
                $receipt.test_binary -ne $binary -or $receipt.jar_sha256 -cne $jarSha -or
                $receipt.inputs_unchanged -ne $true -or @($receipt.cleanup_errors).Count -ne 0 -or @($receipt.drift_errors).Count -ne 0) {
                throw 'G06 receipt status, binding or cleanup mismatch'
            }
            $ownedRoot = Assert-ArtifactPath $receipt.root (Join-Path $project 'target/current/native-teleport-failure')
            $stdoutPath = Assert-ArtifactPath (Join-Path $ownedRoot 'rust.stdout.txt') $ownedRoot
            $testOutput = [IO.File]::ReadAllText($stdoutPath, [Text.Encoding]::UTF8)
            $result = Measure-TestResult $testOutput 0 'Exact'
            if ($testOutput -notmatch ('(?m)^test ' + [regex]::Escape($scenarios[$scenario]) + ' \.\.\. ok\s*\r?$')) { throw 'G06 exact test identity mismatch' }
            [IO.File]::AppendAllText($run.Log, "`n" + $testOutput, $utf8)
            $row = [pscustomobject]@{ Category = 'G06'; Step = $scenario; ExitCode = $run.ExitCode; Passed = $result.Passed
                Failed = $result.Failed; Ignored = $result.Ignored; Zero = $result.Zero
                Seconds = $run.Seconds.ToString('F3', $invariant); Status = $result.Status }
        } catch {
            $row = New-FailedRow 'G06' $scenario $_.Exception.Message
            $row.ExitCode = $run.ExitCode; $row.Seconds = $run.Seconds.ToString('F3', $invariant)
        }
        if ($row.Status -ne 'PASS') { $stopped = $true }
        Write-Host "G06 $scenario | $($row.Status)"
        $row
    }
}

try {
    if ($Quick -and $Category -notin @('All','Baseline')) { throw '-Quick only supports All or Baseline' }
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
    if (-not $Quick -and $Category -in @('All','Baseline')) {
        $libNames = @(Read-Whitelist -Path (Join-Path $PSScriptRoot 'test-whitelist/lib.txt'))
        $testEntries = @(Read-Whitelist -Path (Join-Path $PSScriptRoot 'test-whitelist/tests.tsv') -Integration)
    }
    $stamp = (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + '-' + $PID
    $outputDirectory = Join-Path $project ('target/current/verification/local-test-suite/' + $stamp)
    New-Item -ItemType Directory -Path $scratch,$outputDirectory -Force | Out-Null
    $common = @('--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current')
    $steps = New-Object 'System.Collections.Generic.List[object]'
    $libArgs = @('test') + $common + @('--lib','--no-fail-fast')
    if ($Category -in @('All','Baseline')) {
        $steps.Add(@{ Name = 'check'; Kind = 'Check'; Args = @('check') + $common + @('--all-targets') })
        $steps.Add(@{ Name = 'lib-debug'; Kind = 'Lib'; Args = $libArgs })
    }
    if (-not $Quick -and $Category -in @('All','Baseline')) {
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
    if (-not $Quick -and $Category -in @('All','Skills')) {
        try {
            $skillEntries = @(Read-SkillEntries)
            foreach ($entry in $skillEntries) {
                $skillArgs = @('test') + $common
                if ($entry.Target -eq '__lib') { $skillArgs += '--lib' } else { $skillArgs += @('--test',$entry.Target) }
                if ($entry.Features -ne '-') { $skillArgs += @('--features',$entry.Features) }
                $skillArgs += @('--no-fail-fast','--','--include-ignored','--exact',$entry.Name)
                $rows.Add((Invoke-SuiteStep -Name ($entry.Target + '::' + $entry.Name) -CargoArgs $skillArgs -Kind 'Exact' -Index (++$index) -Group 'Skills'))
            }
        } catch { $rows.Add((New-FailedRow 'Skills' 'T026-manifest' $_.Exception.Message)) }
    }
    if (-not $Quick -and $Category -in @('All','G06')) {
        foreach ($row in @(Invoke-G06Suite $index)) { $rows.Add($row) }
    }
    $finalExit = 0
    if (@($rows | Where-Object { $_.Status -ne 'PASS' }).Count -gt 0) { $finalExit = 1 }
    $mode = 'Full'
    if ($Quick) { $mode = 'Quick' }
    $summary = New-Object 'System.Collections.Generic.List[string]'
    $summary.Add("LOCAL TEST SUITE | Mode=$mode | Category=$Category | PowerShell=$($PSVersionTable.PSVersion)")
    if (-not $Quick -and $Category -in @('All','Skills')) {
        $summary.Add('Skills scope: named T026 tests only; catalog rows without tests are NOT verified.')
    }
    $summary.Add('Category | Step | Exit | passed | failed | ignored | running-0-tests | Seconds | Result')
    foreach ($row in $rows) {
        $label = $row.Category
        if ($label -eq 'Skills') { $label = ([char]0x6280).ToString() + [char]0x80fd + ' (Skills)' }
        $summary.Add(('{0} | {1} | {2} | {3} | {4} | {5} | {6} | {7} | {8}' -f
            $label,$row.Step,$row.ExitCode,$row.Passed,$row.Failed,$row.Ignored,$row.Zero,$row.Seconds,$row.Status))
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
