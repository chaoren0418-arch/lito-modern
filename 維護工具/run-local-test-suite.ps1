[CmdletBinding()]
param(
    [switch]$Quick,
    # T042: Debug + Release for every category, plus the T036 released items
    # (test-whitelist/full.tsv). Rows that cannot run are listed with a reason.
    [switch]$Full,
    [ValidateSet('All','Baseline','Skills','G06','T036')][string]$Category = 'All',
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
# Tests hardcode this read-only resource root. This file has no BOM, so the
# non-ASCII path is built from code points to stay correct on PowerShell 5.1.
$clientRoot = 'D:\lineage' + [char]0x958B + [char]0x767C + '\0-' + [char]0x5BA2 + [char]0x6236 + [char]0x7AEF

function Get-G06Scenarios {
    # Names match native_E149_recovery_contract.py. The last three are supplied
    # by T023-2; unsupported scenarios must remain visible failures, never skips.
    [ordered]@{
        'timeout' = 'destination_timeout_retires_old_authority_and_allows_fresh_login'
        'refused' = 'refused_destination_retires_authority_and_allows_fresh_login'
        'wrong-service' = 'wrong_service_destination_retires_authority_and_allows_fresh_login'
        'cancel-retry' = 'cancellation_then_immediate_retry_consumes_staged_character_once'
        'cancel-expiry' = 'cancellation_retires_destination_and_expired_handoff_requires_fresh_selection'
        'cancel-repeat' = 'repeated_cancellation_preserves_only_one_authenticated_recovery'
        'refused-cancel-repeat' = 'failed_handoff_then_repeated_cancel_preserves_the_authenticated_recovery'
        'cancel-expiry-source' = 'expired_stage_and_source_handoff_require_the_requested_source_directory'
    }
}

function Read-FullManifest {
    param([string]$Path = (Join-Path $PSScriptRoot 'test-whitelist/full.tsv'))
    # T036 ledger: one row per test outside the T010 whitelist. Eight columns:
    # target, name, ignored, features, verdict, requires, source, reason.
    #   run      - reviewed and released; -Full runs it in Debug and Release
    #   excluded - must not run; the reason is printed per item
    #   covered  - already run by lib.txt/tests.tsv/skills.tsv or the G06 suite
    $lines = [IO.File]::ReadAllLines($Path, [Text.Encoding]::UTF8)
    if ($lines.Length -eq 0) { throw "Empty T036 manifest: $Path" }
    $seen = @{}
    foreach ($line in $lines) {
        $fields = $line.Split([char]9)
        if ($fields.Length -ne 8 -or $fields[0] -notmatch '^[A-Za-z0-9_]+$' -or
            $fields[1] -notmatch '^[A-Za-z0-9_:]+$' -or $fields[2] -notmatch '^(yes|no)$' -or
            $fields[3] -notmatch '^(?:-|[A-Za-z0-9_-]+(?:,[A-Za-z0-9_-]+)*)$' -or
            $fields[4] -notmatch '^(run|excluded|covered)$' -or $fields[5] -notmatch '^(-|client)$' -or
            $fields[6] -notmatch '^[#A-Za-z0-9_-]+$' -or $fields[7].Trim().Length -eq 0) {
            throw "Invalid eight-column T036 manifest row: $line"
        }
        if ($fields[4] -eq 'excluded') {
            # An exclusion without a stated reason is not allowed.
            if ($fields[7] -notmatch '^[a-z0-9-]+: \S') { throw "T036 exclusion needs 'code: reason': $($fields[0])::$($fields[1])" }
        } elseif ($fields[7] -ne '-') { throw "T036 $($fields[4]) row must use '-' as reason: $($fields[0])::$($fields[1])" }
        if ($fields[4] -ne 'run' -and $fields[5] -ne '-') { throw "Only run rows may declare a requirement: $($fields[0])::$($fields[1])" }
        $key = $fields[0] + '::' + $fields[1]
        if ($seen.ContainsKey($key)) { throw "Duplicate T036 manifest item: $key" }
        $seen[$key] = $true
        [pscustomobject]@{ Target = $fields[0]; Name = $fields[1]; Features = $fields[3]; Verdict = $fields[4]
            Requires = $fields[5]; Source = $fields[6]; Reason = $fields[7]; Key = $key }
    }
}

function Get-CoveredTests {
    param([string]$ListRoot = (Join-Path $PSScriptRoot 'test-whitelist'))
    # Exact tests the other categories already run. Static file reads only.
    $covered = @{}
    foreach ($name in @(Read-Whitelist -Path (Join-Path $ListRoot 'lib.txt'))) { $covered['__lib::' + $name] = 'Baseline' }
    foreach ($entry in @(Read-Whitelist -Path (Join-Path $ListRoot 'tests.tsv') -Integration)) {
        $covered[$entry.Target + '::' + $entry.Name] = 'Baseline'
    }
    foreach ($entry in @(Read-Whitelist -Path (Join-Path $ListRoot 'skills.tsv') -Integration)) {
        $key = $entry.Target + '::' + $entry.Name
        if (-not $covered.ContainsKey($key)) { $covered[$key] = 'Skills' }
    }
    foreach ($test in (Get-G06Scenarios).Values) { $covered['native_teleport_failure::' + $test] = 'G06' }
    $covered
}

function Resolve-FullPlan {
    param([object[]]$Entries, [hashtable]$Covered)
    # Every manifest row gets exactly one action; contradictions fail the category.
    foreach ($entry in $Entries) {
        $elsewhere = $Covered.ContainsKey($entry.Key)
        $action = $entry.Verdict
        if ($entry.Verdict -eq 'excluded' -and $elsewhere) {
            throw "T036 row is excluded but $($Covered[$entry.Key]) runs it: $($entry.Key)"
        }
        if ($entry.Verdict -eq 'covered' -and -not $elsewhere) {
            throw "T036 row is marked covered but no other category runs it: $($entry.Key)"
        }
        # A released row that another list picked up later is run there, once.
        if ($entry.Verdict -eq 'run' -and $elsewhere) { $action = 'covered' }
        [pscustomobject]@{ Target = $entry.Target; Name = $entry.Name; Features = $entry.Features; Action = $action
            Requires = $entry.Requires; Source = $entry.Source; Reason = $entry.Reason; Key = $entry.Key }
    }
}

function Get-ExactTestArgs {
    param([string]$Target, [string]$Name, [string]$Features, [string]$BuildKind, [string[]]$Common)
    $testArgs = @('test') + $Common
    if ($BuildKind -eq 'Release') { $testArgs += '--release' }
    if ($Target -eq '__lib') { $testArgs += '--lib' } else { $testArgs += @('--test', $Target) }
    if ($Features -ne '-') { $testArgs += @('--features', $Features) }
    $testArgs + @('--no-fail-fast', '--', '--include-ignored', '--exact', $Name)
}

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

function Read-TargetTestNames {
    param([string]$Target, [string]$TestsRoot)
    # Static source scan only; nothing is compiled or executed here. Files with
    # modules or include! cannot be enumerated reliably and need manual mapping.
    if ($Target -notmatch '^[A-Za-z0-9_]+$') { throw "Invalid T026 test target: $Target" }
    $path = Join-Path $TestsRoot ($Target + '.rs')
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Unknown T026 test target: $Target" }
    $text = [IO.File]::ReadAllText($path, [Text.Encoding]::UTF8)
    if ($text -match '(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+[A-Za-z0-9_]+\s*[;{]' -or $text -match 'include!\s*\(') {
        throw "T026 target needs manual review (mod/include!): $Target"
    }
    $attributes = [regex]::Matches($text, '(?m)^[ \t]*#\[test\][ \t]*\r?$').Count
    $found = [regex]::Matches($text, '(?m)^[ \t]*#\[test\][ \t]*\r?\n(?:[ \t]*#\[[^\r\n]*\][ \t]*\r?\n)*[ \t]*fn[ \t]+([A-Za-z0-9_]+)[ \t]*\(')
    if ($attributes -eq 0 -or $found.Count -ne $attributes) { throw "Cannot enumerate tests in T026 target: $Target" }
    $result = @{}
    foreach ($match in $found) {
        $name = $match.Groups[1].Value
        if ($result.ContainsKey($name)) { throw "Duplicate test in T026 target: $($Target)::$($name)" }
        $result[$name] = $true
    }
    $result
}

function Read-SkillEntries {
    param(
        [string]$StatusPath = (Join-Path $repoRoot 'docs/SKILLS-STATUS.md'),
        [string]$ManifestPath = (Join-Path $PSScriptRoot 'test-whitelist/skills.tsv'),
        [string]$TestsRoot = (Join-Path $project 'tests')
    )
    # The reviewed manifest resolves short names, duplicate names across targets,
    # nested modules and include! tests. Never discover/run arbitrary Rust tests.
    # A bare target reference (e.g. native_t027_batch1) must list every #[test]
    # in that file; an optional "(N cases)" note must match the count.
    $entries = @(Read-Whitelist $ManifestPath -Integration)
    $entryTargets = @{}
    foreach ($entry in $entries) { $entryTargets[$entry.Target] = $true }
    $names = @{}; $ids = @{}; $targetCounts = @{}; $withoutTests = 0; $inTable = $false
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
            $name = $Matches[1]
            $names[$name] = $true
            $after = $cells[12].Substring($ref.Index + $ref.Length)
            if ($after -match '^\uFF08(\d+)\s*\u6848') {
                $count = [int]$Matches[1]
                if ($targetCounts.ContainsKey($name) -and $targetCounts[$name] -ne $count) { throw "Conflicting T026 case counts: $name" }
                $targetCounts[$name] = $count
            }
        }
    }
    if ($ids.Count -ne 181 -or $names.Count -eq 0) { throw 'Expected 181 skill rows and nonempty test references' }
    # Whole-target references: the manifest must match the file's #[test] set exactly.
    $targetRefs = @{}
    foreach ($name in @($names.Keys)) {
        if ($name -notmatch '::' -and $entryTargets.ContainsKey($name)) {
            $sourceNames = Read-TargetTestNames $name $TestsRoot
            $listed = @{}
            foreach ($entry in $entries) { if ($entry.Target -ceq $name) { $listed[$entry.Name] = $true } }
            foreach ($test in $sourceNames.Keys) {
                if (-not $listed.ContainsKey($test)) { throw "Unmapped T026 target test: $($name)::$($test); update test-whitelist/skills.tsv" }
            }
            foreach ($test in $listed.Keys) {
                if (-not $sourceNames.ContainsKey($test)) { throw "Stale skill manifest entry: $($name)::$($test)" }
            }
            if ($targetCounts.ContainsKey($name) -and $targetCounts[$name] -ne $sourceNames.Count) {
                throw "T026 case count mismatch: $name lists $($targetCounts[$name]), source has $($sourceNames.Count)"
            }
            $targetRefs[$name] = $true
        }
    }
    $mapped = @{}
    foreach ($entry in $entries) {
        $short = ($entry.Name -split '::')[-1]
        $qualified = $entry.Target + '::' + $entry.Name
        if (-not $names.ContainsKey($short) -and -not $names.ContainsKey($entry.Name) -and
            -not $names.ContainsKey($qualified) -and -not $targetRefs.ContainsKey($entry.Target)) {
            throw "Stale skill manifest entry: $qualified"
        }
        $mapped[$short] = $true; $mapped[$entry.Name] = $true; $mapped[$qualified] = $true
    }
    foreach ($name in $names.Keys) {
        if (-not $mapped.ContainsKey($name) -and -not $targetRefs.ContainsKey($name)) {
            throw "Unmapped T026 test: $name; review and update test-whitelist/skills.tsv"
        }
    }
    Write-Host "Skills: $($ids.Count) catalog rows; $($names.Count) references ($($targetRefs.Count) whole targets); $($entries.Count) exact tests; $withoutTests rows have no tests (not verified)."
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
    param([string]$Name, [string[]]$CargoArgs, [string]$Kind, [int]$Index, [string]$Group = 'Baseline', [string]$BuildKind = 'Debug')
    $run = Invoke-SuiteProcess -Name $Name -Executable $cargoPath -Arguments $CargoArgs -Index $Index
    $exitCode = $run.ExitCode; $logPath = $run.Log
    $result = Measure-TestResult -Output $run.Output -ExitCode $exitCode -Kind $Kind
    Write-Host ("[$Index] {0}; exit={1}; log={2}" -f $result.Status, $exitCode, $logPath)
    [pscustomobject]@{
        Category = $Group; Profile = $BuildKind; Step = $Name; ExitCode = $exitCode; Passed = $result.Passed; Failed = $result.Failed
        Ignored = $result.Ignored; Zero = $result.Zero
        Seconds = $run.Seconds.ToString('F3', $invariant); Status = $result.Status
    }
}

function New-FailedRow {
    param([string]$Group, [string]$Name, [string]$Reason, [string]$BuildKind = '-')
    [pscustomobject]@{ Category = $Group; Profile = $BuildKind; Step = $Name; ExitCode = 'N/A'; Passed = 'N/A'; Failed = 'N/A'
        Ignored = 'N/A'; Zero = 'N/A'; Seconds = '0.000'; Status = 'FAIL: ' + $Reason }
}

function New-ExcludedRow {
    param([string]$Group, [string]$Name, [string]$Reason)
    # Not executed and never counted as passed; it does not fail the run either.
    [pscustomobject]@{ Category = $Group; Profile = '-'; Step = $Name; ExitCode = 'N/A'; Passed = 'N/A'; Failed = 'N/A'
        Ignored = 'N/A'; Zero = 'N/A'; Seconds = '0.000'; Status = 'EXCLUDED: ' + $Reason }
}

function Invoke-T036Suite {
    param([int]$StartIndex, [string[]]$Builds, [string[]]$Common,
        [string]$ManifestPath = (Join-Path $PSScriptRoot 'test-whitelist/full.tsv'),
        [string]$ListRoot = (Join-Path $PSScriptRoot 'test-whitelist'),
        [string]$ResourceRoot = $clientRoot)
    $index = $StartIndex
    try {
        $plan = @(Resolve-FullPlan @(Read-FullManifest $ManifestPath) (Get-CoveredTests $ListRoot))
    } catch {
        New-FailedRow 'T036' 'T036-manifest' $_.Exception.Message
        return
    }
    $run = @($plan | Where-Object { $_.Action -eq 'run' })
    $excluded = @($plan | Where-Object { $_.Action -eq 'excluded' })
    $elsewhere = @($plan | Where-Object { $_.Action -eq 'covered' })
    Write-Host "T036: $($plan.Count) ledger rows; run=$($run.Count) per profile; excluded=$($excluded.Count); run by Baseline/Skills/G06=$($elsewhere.Count) (not repeated here)."
    $hasClient = Test-Path -LiteralPath $ResourceRoot -PathType Container
    foreach ($BuildKind in $Builds) {
        foreach ($entry in $run) {
            if ($entry.Requires -eq 'client' -and -not $hasClient) {
                # A released test that cannot run on this host is a failure, not a pass.
                New-FailedRow 'T036' $entry.Key 'precondition: client-resource-root-missing' $BuildKind
                continue
            }
            $testArgs = Get-ExactTestArgs $entry.Target $entry.Name $entry.Features $BuildKind $Common
            Invoke-SuiteStep -Name $entry.Key -CargoArgs $testArgs -Kind 'Exact' -Index (++$index) -Group 'T036' -BuildKind $BuildKind
        }
    }
    foreach ($entry in $excluded) { New-ExcludedRow 'T036' $entry.Key ($entry.Reason + ' [' + $entry.Source + ']') }
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
    # ParseFile on PS 5.1 decodes BOM-less UTF-8 as ANSI; decode explicitly.
    $source = [IO.File]::ReadAllText($RunnerPath, (New-Object Text.UTF8Encoding($false, $true)))
    $ast = [Management.Automation.Language.Parser]::ParseInput($source, [ref]$tokens, [ref]$errors)
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
    param([int]$StartIndex, [string]$BuildKind = 'Debug')
    $scenarios = Get-G06Scenarios
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
        $buildArgs = @('test') + $common
        if ($BuildKind -eq 'Release') { $buildArgs += '--release' }
        $buildArgs += @('--test','native_teleport_failure','--no-run','--message-format=json')
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
        foreach ($scenario in $scenarios.Keys) { New-FailedRow 'G06' $scenario ('precondition: ' + $_.Exception.Message) $BuildKind }
        return
    }
    $stopped = $false
    foreach ($scenario in $scenarios.Keys) {
        if ($stopped) { New-FailedRow 'G06' $scenario 'not-run-after-G06-failure' $BuildKind; continue }
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
            $row = [pscustomobject]@{ Category = 'G06'; Profile = $BuildKind; Step = $scenario; ExitCode = $run.ExitCode; Passed = $result.Passed
                Failed = $result.Failed; Ignored = $result.Ignored; Zero = $result.Zero
                Seconds = $run.Seconds.ToString('F3', $invariant); Status = $result.Status }
        } catch {
            $row = New-FailedRow 'G06' $scenario $_.Exception.Message $BuildKind
            $row.ExitCode = $run.ExitCode; $row.Seconds = $run.Seconds.ToString('F3', $invariant)
        }
        if ($row.Status -ne 'PASS') { $stopped = $true }
        Write-Host "G06 $scenario | $($row.Status)"
        $row
    }
}

function Get-SuiteExitCode {
    param([object[]]$Rows)
    # EXCLUDED rows are reported, not executed; anything else must be PASS.
    if (@($Rows | Where-Object { $_.Status -ne 'PASS' -and $_.Status -notlike 'EXCLUDED: *' }).Count -gt 0) { return 1 }
    0
}

function Format-SuiteSummary {
    param([object[]]$Rows, [string]$Mode, [string]$Category, [int]$FinalExit, [bool]$SkillsNote)
    $full = $Mode -eq 'Full'
    $summary = New-Object 'System.Collections.Generic.List[string]'
    $summary.Add("LOCAL TEST SUITE | Mode=$Mode | Category=$Category | PowerShell=$($PSVersionTable.PSVersion)")
    if ($SkillsNote) {
        $summary.Add('Skills scope: named T026 tests only; catalog rows without tests are NOT verified.')
    }
    if ($full) {
        $summary.Add('Full scope: every category in Debug and Release, plus T036 released rows. EXCLUDED rows were not run and are not passes.')
        $summary.Add('Category | Profile | Step | Exit | passed | failed | ignored | running-0-tests | Seconds | Result')
    } else {
        $summary.Add('Category | Step | Exit | passed | failed | ignored | running-0-tests | Seconds | Result')
    }
    foreach ($row in $Rows) {
        $label = $row.Category
        if ($label -eq 'Skills') { $label = ([char]0x6280).ToString() + [char]0x80fd + ' (Skills)' }
        if ($full) { $label = $label + ' | ' + $row.Profile }
        $summary.Add(('{0} | {1} | {2} | {3} | {4} | {5} | {6} | {7} | {8}' -f
            $label,$row.Step,$row.ExitCode,$row.Passed,$row.Failed,$row.Ignored,$row.Zero,$row.Seconds,$row.Status))
    }
    if ($full) {
        $excluded = @($Rows | Where-Object { $_.Status -like 'EXCLUDED: *' })
        $counted = @($Rows | Where-Object { $_.Status -notlike 'EXCLUDED: *' })
        $passed = @($counted | Where-Object { $_.Status -eq 'PASS' }).Count
        $summary.Add("Counted=$($counted.Count) | PASS=$passed | FAIL=$($counted.Count - $passed) | EXCLUDED=$($excluded.Count) (not run; reasons listed per item above)")
        foreach ($name in @('Debug','Release')) {
            $part = @($counted | Where-Object { $_.Profile -eq $name })
            $partPassed = @($part | Where-Object { $_.Status -eq 'PASS' }).Count
            $summary.Add("${name}: rows=$($part.Count) | PASS=$partPassed | FAIL=$($part.Count - $partPassed)")
        }
        $byCode = [ordered]@{}
        foreach ($row in $excluded) {
            $code = 'other'
            if ($row.Status -match '^EXCLUDED: ([a-z0-9-]+):') { $code = $Matches[1] }
            if ($byCode.Contains($code)) { $byCode[$code]++ } else { $byCode[$code] = 1 }
        }
        if ($byCode.Count -gt 0) {
            $summary.Add('Excluded by reason: ' + ((@($byCode.Keys) | ForEach-Object { $_ + '=' + $byCode[$_] }) -join '; '))
        }
    }
    $summary.Add("Steps=$($Rows.Count) | Final exit code=$FinalExit")
    ($summary -join "`n") + "`n"
}

try {
    if ($Quick -and $Full) { throw '-Quick and -Full cannot be combined' }
    if ($Quick -and $Category -notin @('All','Baseline')) { throw '-Quick only supports All or Baseline' }
    if ($Category -eq 'T036' -and -not $Full) { throw '-Category T036 requires -Full' }
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
    # Creating this also creates target/current/verification, which the T036
    # evidence-writing tests expect to exist (#306).
    $outputDirectory = Join-Path $project ('target/current/verification/local-test-suite/' + $stamp)
    New-Item -ItemType Directory -Path $scratch,$outputDirectory -Force | Out-Null
    $common = @('--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current')
    # Without -Full the whitelists run in Debug only, exactly as before.
    $builds = @('Debug')
    if ($Full) { $builds = @('Debug','Release') }
    $steps = New-Object 'System.Collections.Generic.List[object]'
    $libArgs = @('test') + $common + @('--lib','--no-fail-fast')
    if ($Category -in @('All','Baseline')) {
        $steps.Add(@{ Name = 'check'; Kind = 'Check'; Build = 'Debug'; Args = @('check') + $common + @('--all-targets') })
        $steps.Add(@{ Name = 'lib-debug'; Kind = 'Lib'; Build = 'Debug'; Args = $libArgs })
    }
    if (-not $Quick -and $Category -in @('All','Baseline')) {
        $steps.Add(@{ Name = 'lib-release'; Kind = 'Lib'; Build = 'Release'; Args = $libArgs + @('--release') })
        foreach ($build in $builds) {
            $releaseFlag = @()
            if ($build -eq 'Release') { $releaseFlag = @('--release') }
            foreach ($name in $libNames) {
                $steps.Add(@{ Name = 'ignored-lib:' + $name; Kind = 'Exact'; Build = $build
                    Args = $libArgs + $releaseFlag + @('--','--ignored','--exact',$name) })
            }
            foreach ($entry in $testEntries) {
                $steps.Add(@{ Name = $entry.Target + '::' + $entry.Name; Kind = 'Exact'; Build = $build
                    Args = (Get-ExactTestArgs $entry.Target $entry.Name $entry.Features $build $common) })
            }
        }
    }
    $rows = New-Object 'System.Collections.Generic.List[object]'
    $index = 0
    foreach ($step in $steps) {
        $index++
        $rows.Add((Invoke-SuiteStep -Name $step.Name -CargoArgs $step.Args -Kind $step.Kind -Index $index -BuildKind $step.Build))
    }
    if (-not $Quick -and $Category -in @('All','Skills')) {
        try {
            $skillEntries = @(Read-SkillEntries)
            foreach ($build in $builds) {
                foreach ($entry in $skillEntries) {
                    $skillArgs = Get-ExactTestArgs $entry.Target $entry.Name $entry.Features $build $common
                    $rows.Add((Invoke-SuiteStep -Name ($entry.Target + '::' + $entry.Name) -CargoArgs $skillArgs -Kind 'Exact' -Index (++$index) -Group 'Skills' -BuildKind $build))
                }
            }
        } catch { $rows.Add((New-FailedRow 'Skills' 'T026-manifest' $_.Exception.Message)) }
    }
    if ($Full -and $Category -in @('All','T036')) {
        $ledgerRows = @(Invoke-T036Suite -StartIndex $index -Builds $builds -Common $common)
        foreach ($row in $ledgerRows) { $rows.Add($row) }
        $index += @($ledgerRows | Where-Object { $_.ExitCode -ne 'N/A' }).Count
    }
    if (-not $Quick -and $Category -in @('All','G06')) {
        foreach ($build in $builds) {
            foreach ($row in @(Invoke-G06Suite $index $build)) { $rows.Add($row) }
            # One G06 pass writes at most 3 preparation logs and 8 scenario logs.
            $index += 11
        }
    }
    $finalExit = Get-SuiteExitCode $rows.ToArray()
    $mode = 'Standard'
    if ($Quick) { $mode = 'Quick' }
    if ($Full) { $mode = 'Full' }
    $summaryText = Format-SuiteSummary -Rows $rows.ToArray() -Mode $mode -Category $Category -FinalExit $finalExit -SkillsNote (-not $Quick -and $Category -in @('All','Skills'))
    $summaryPath = Join-Path $outputDirectory 'summary.txt'
    [IO.File]::WriteAllText($summaryPath, $summaryText, $utf8)
    Write-Output $summaryText
    Write-Host "Summary: $summaryPath"
    exit $finalExit
} catch {
    Write-Error $_ -ErrorAction Continue
    exit 1
}