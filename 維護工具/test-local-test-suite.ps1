# Regression tests for the runner itself; synthetic output is NOT Rust evidence.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 2.0
$repoRoot = Split-Path -Parent $PSScriptRoot
$project = Join-Path $repoRoot 'LiTo381-Modern'
$utf8 = New-Object Text.UTF8Encoding($false)
$invariant = [Globalization.CultureInfo]::InvariantCulture
$tokens = $null; $errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot 'run-local-test-suite.ps1'), [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw ($errors -join "`n") }
# Load actual function definitions without executing the suite's entry point.
foreach ($node in $ast.EndBlock.Statements) {
    if ($node -is [Management.Automation.Language.FunctionDefinitionAst]) {
        # Create() has no filename, so bind the original script directory literal.
        $definition = $node.Extent.Text.Replace('$PSScriptRoot', ("'" + $PSScriptRoot.Replace("'","''") + "'"))
        . ([scriptblock]::Create($definition))
    }
}
$checks = 0
function Assert([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
    $script:checks++
}
function Reject([scriptblock]$Action, [string]$Message) {
    $rejected = $false
    try { & $Action | Out-Null } catch { $rejected = $true }
    Assert $rejected $Message
}
$good = "running 1 test`ntest sample ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`n"
Assert ((Measure-TestResult $good 0 Exact).Status -eq 'PASS') 'valid exact result rejected'
Assert ((Measure-TestResult ($good.Replace("`n","`r`n")) 0 Exact).Status -eq 'PASS') 'CRLF result rejected'
Assert ((Measure-TestResult '' 0 Check).Status -eq 'PASS') 'check exit 0 rejected'
Assert ((Measure-TestResult '' 9 Check).Status -ne 'PASS') 'check nonzero accepted'
foreach ($bad in @(
    '', $good.Replace('running 1 test','running 0 tests'),
    $good.Replace('1 passed','0 passed'), $good.Replace('0 failed','1 failed'),
    $good.Replace('0 ignored','1 ignored'), $good.Replace('0 measured','1 measured'),
    $good.Replace('test result: ok.','test result: FAILED.'),
    $good.Replace('test result:','invalid result:'), $good.Replace('running 1 test',''),
    ($good + $good), $good.Replace('0.01s','unknown'),
    "running 0 tests`ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s"
)) { Assert ((Measure-TestResult $bad 0 Exact).Status -ne 'PASS') 'invalid exact result accepted' }
Assert ((Measure-TestResult $good 101 Exact).Status -ne 'PASS') 'nonzero test exit accepted'

$qa = Join-Path $repoRoot ('scratch/local-suite-qa-' + $PID)
New-Item -ItemType Directory $qa -Force | Out-Null
$statusPath = Join-Path $repoRoot 'docs/SKILLS-STATUS.md'
$manifestPath = Join-Path $PSScriptRoot 'test-whitelist/skills.tsv'
$entries = @(Read-SkillEntries $statusPath $manifestPath)
Assert ($entries.Count -gt 0) 'empty skill manifest'
$doc = [IO.File]::ReadAllText($statusPath, [Text.Encoding]::UTF8)
$manifest = [IO.File]::ReadAllText($manifestPath, [Text.Encoding]::UTF8)
$docFixture = Join-Path $qa 'status.md'; $manifestFixture = Join-Path $qa 'skills.tsv'
[IO.File]::WriteAllText($docFixture, $doc.Replace('ability_buffs_keep_native_screen_asymmetric_edges','unmapped_new_test'), $utf8)
Reject { Read-SkillEntries $docFixture $manifestPath } 'new T026 reference silently dropped'
[IO.File]::WriteAllText($docFixture, $doc.Replace('| 1 |','| 2 |'), $utf8)
Reject { Read-SkillEntries $docFixture $manifestPath } 'duplicate catalog ID accepted'
[IO.File]::WriteAllText($docFixture, '', $utf8)
Reject { Read-SkillEntries $docFixture $manifestPath } 'empty status table accepted'
[IO.File]::WriteAllText($manifestFixture, $manifest + "native_skills`tstale_test`tno`t-`n", $utf8)
Reject { Read-SkillEntries $statusPath $manifestFixture } 'stale manifest entry accepted'
[IO.File]::WriteAllText($manifestFixture, $manifest + ($manifest -split "`n")[0] + "`n", $utf8)
Reject { Read-SkillEntries $statusPath $manifestFixture } 'duplicate manifest entry accepted'
[IO.File]::WriteAllText($manifestFixture, "native_skills`tbad;name`tno`t-`n", $utf8)
Reject { Read-SkillEntries $statusPath $manifestFixture } 'invalid identifier accepted'

# Whole-target references (native_t027_batch1/2) and target::name references.
$testsRoot = Join-Path $project 'tests'
foreach ($target in @('native_t027_batch1','native_t027_batch2')) {
    $sourceCount = (Read-TargetTestNames $target $testsRoot).Count
    $listedCount = @($entries | Where-Object { $_.Target -ceq $target }).Count
    Assert ($sourceCount -gt 0 -and $sourceCount -eq $listedCount) "whole target not fully listed: $target"
}
Assert (@($entries | Where-Object { $_.Target -ceq 'native_t021' -and $_.Name -ceq 'wire_shapes_and_menu_keep_selectors_separate_from_inventory_objects' }).Count -eq 1) 'qualified target::name reference not listed'
$manifestLines = @($manifest -split "\r?\n" | Where-Object { $_ })
$dropped = @($manifestLines | Where-Object { $_ -notmatch '^native_t027_batch1\tall_eight_complete_wire_shapes_are_pure_and_use_learned_ids\t' })
Assert ($dropped.Count -eq $manifestLines.Count - 1) 'fixture did not drop exactly one target test'
[IO.File]::WriteAllText($manifestFixture, ($dropped -join "`n") + "`n", $utf8)
Reject { Read-SkillEntries $statusPath $manifestFixture } 'whole-target test silently dropped'
[IO.File]::WriteAllText($manifestFixture, $manifest + "native_t027_batch2`tinvented_case`tno`t-`n", $utf8)
Reject { Read-SkillEntries $statusPath $manifestFixture } 'stale whole-target entry accepted'
$withoutT021 = @($manifestLines | Where-Object { $_ -notmatch '^native_t021\t' })
[IO.File]::WriteAllText($manifestFixture, ($withoutT021 -join "`n") + "`n", $utf8)
Reject { Read-SkillEntries $statusPath $manifestFixture } 'qualified reference silently dropped'
Assert ($doc.Contains('`native_t027_batch1`' + [char]0xFF08 + '13 ')) 'batch1 case-count note missing'
[IO.File]::WriteAllText($docFixture, $doc.Replace('`native_t027_batch1`' + [char]0xFF08 + '13 ', '`native_t027_batch1`' + [char]0xFF08 + '14 '), $utf8)
Reject { Read-SkillEntries $docFixture $manifestPath } 'case-count mismatch accepted'
$fakeTests = Join-Path $qa 'tests'
New-Item -ItemType Directory $fakeTests -Force | Out-Null
foreach ($target in @('native_t027_batch1','native_t027_batch2')) {
    Copy-Item -LiteralPath (Join-Path $testsRoot ($target + '.rs')) -Destination $fakeTests
}
Assert ((Read-SkillEntries $statusPath $manifestPath $fakeTests).Count -eq $entries.Count) 'fake tests root fixture invalid'
$batch2 = Join-Path $fakeTests 'native_t027_batch2.rs'
$batch2Text = [IO.File]::ReadAllText($batch2, [Text.Encoding]::UTF8)
[IO.File]::WriteAllText($batch2, $batch2Text + "`ninclude!(`"extra.rs`");`n", $utf8)
Reject { Read-SkillEntries $statusPath $manifestPath $fakeTests } 'include! in whole target accepted'
[IO.File]::WriteAllText($batch2, $batch2Text + "`n#[test]`nfn added_after_review() {}`n", $utf8)
Reject { Read-SkillEntries $statusPath $manifestPath $fakeTests } 'new whole-target test silently dropped'
[IO.File]::WriteAllText($batch2, $batch2Text + "`n#[test]`n// comment between`nfn hidden() {}`n", $utf8)
Reject { Read-SkillEntries $statusPath $manifestPath $fakeTests } 'unparseable #[test] accepted'
Remove-Item -LiteralPath $batch2 -Confirm:$false
Reject { Read-SkillEntries $statusPath $manifestPath $fakeTests } 'missing whole-target source accepted'

# Actual subprocess round trip: spaces, trailing slash, environment isolation,
# UTF-8 and both output pipes. Only this QA child is started.
$scratch = Join-Path $qa 'temp with spaces'
$outputDirectory = Join-Path $qa 'logs'
New-Item -ItemType Directory $scratch,$outputDirectory -Force | Out-Null
$child = Join-Path $qa 'child with spaces.ps1'
[IO.File]::WriteAllText($child, @'
param([string]$Value)
[Console]::OutputEncoding = New-Object Text.UTF8Encoding($false)
[Console]::WriteLine($Value)
[Console]::WriteLine('TMP=' + $env:TMP)
[Console]::WriteLine('TEMP=' + $env:TEMP)
if (Get-ChildItem Env:LITO_*) { exit 9 }
[Console]::Error.WriteLine('stderr-drained')
exit 0
'@, $utf8)
$shell = (Get-Process -Id $PID).Path
$prior = $env:LITO_SUITE_QA
try {
    $env:LITO_SUITE_QA = 'must not inherit'
    $value = 'space path' + [char]0x6e2c + '\'
    $run = Invoke-SuiteProcess 'QA-child' $shell @('-NoProfile','-ExecutionPolicy','Bypass','-File',$child,'-Value',$value) 1
    Assert ($run.ExitCode -eq 0) 'child environment not cleared'
    Assert ($run.Output.Contains($value)) 'argument quoting changed trailing slash'
    Assert ($run.Output.Contains('TEMP=' + $scratch) -and $run.Output.Contains('TMP=' + $scratch)) 'child temp not isolated'
    Assert ($run.Output.Contains('stderr-drained')) 'stderr lost'
    Assert ($env:LITO_SUITE_QA -eq 'must not inherit') 'parent environment changed'
} finally { $env:LITO_SUITE_QA = $prior }
$failedRun = Invoke-SuiteProcess 'QA-missing-executable' (Join-Path $qa 'absent.exe') @() 2
Assert ($failedRun.ExitCode -ne 0) 'start failure accepted'
Reject { Assert-ArtifactPath $statusPath $qa } 'outside-root artifact accepted'
Assert ((Assert-ArtifactPath $child $qa) -eq $child) 'owned artifact rejected'

# G06 preconditions must produce all eight failed rows, even before Cargo starts.
$PowerShellPath = Join-Path $qa 'absent-pwsh.exe'
$ReferenceJar = ''; $common = @(); $cargoPath = 'must-not-run'
$rows = @(Invoke-G06Suite 2)
Assert ($rows.Count -eq 8) 'G06 precondition dropped scenarios'
Assert (@($rows | Where-Object Status -eq 'PASS').Count -eq 0) 'G06 incomplete preconditions passed'
Assert (($rows.Step -join ',') -eq 'timeout,refused,wrong-service,cancel-retry,cancel-expiry,cancel-repeat,refused-cancel-repeat,cancel-expiry-source') 'G06 inventory changed'
$supported = @(Read-G06SupportedScenarios (Join-Path $PSScriptRoot 'run-teleport-failure.ps1'))
Assert ($supported -contains 'timeout' -and $supported -contains 'cancel-expiry') 'owned runner metadata unavailable'
foreach ($name in @('cancel-repeat','refused-cancel-repeat','cancel-expiry-source')) {
    Assert ($supported -ccontains $name) "owned runner lacks T023-2 scenario: $name"
}

# Synthetic owned-runner adapter exercises selection, evidence parsing and
# fail-stop. No Java, Cargo, network or game process runs in this section.
$scenarioNames = @($rows.Step)
$testNames = @(
    'destination_timeout_retires_old_authority_and_allows_fresh_login',
    'refused_destination_retires_authority_and_allows_fresh_login',
    'wrong_service_destination_retires_authority_and_allows_fresh_login',
    'cancellation_then_immediate_retry_consumes_staged_character_once',
    'cancellation_retires_destination_and_expired_handoff_requires_fresh_selection',
    'repeated_cancellation_preserves_only_one_authenticated_recovery',
    'failed_handoff_then_repeated_cancel_preserves_the_authenticated_recovery',
    'expired_stage_and_source_handoff_require_the_requested_source_directory'
)
$project = Join-Path $qa 'synthetic project'
$target = Join-Path $project 'target/current'
New-Item -ItemType Directory $target -Force | Out-Null
$ReferenceJar = Join-Path $target 'synthetic.jar'
$binaryFixture = Join-Path $target 'synthetic-test.exe'
[IO.File]::WriteAllText($ReferenceJar, 'not a real JAR', $utf8)
[IO.File]::WriteAllText($binaryFixture, 'not executable', $utf8)
$PowerShellPath = 'synthetic-shell'; $supportedCount = 5; $fault = ''; $scenarioCalls = 0; $processCalls = 0
$fakeSha = 'E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
function Get-FileHash { param($LiteralPath,$Algorithm); [pscustomobject]@{ Hash = $fakeSha } }
function Read-G06SupportedScenarios { $scenarioNames[0..($supportedCount-1)] }
function Invoke-SuiteProcess {
    param($Name,$Executable,$Arguments,$Index)
    $script:processCalls++
    $log = Join-Path $qa ('synthetic-' + $Index + '.log')
    [IO.File]::WriteAllText($log, 'SYNTHETIC RUNNER QA ONLY', $utf8)
    $code = 0
    if ($Name -eq 'G06-pwsh-version') { $output = '7.0.0' }
    elseif ($Name -eq 'G06-build') {
        $output = @{ reason='compiler-artifact'; target=@{name='native_teleport_failure'}; profile=@{test=$true}; executable=$binaryFixture } | ConvertTo-Json -Compress
    } elseif ($Name -eq 'G06-test-inventory') {
        Assert (($Arguments -join ' ') -eq '--list --ignored --format terse') 'inventory could execute tests'
        $output = ($testNames | ForEach-Object { $_ + ': test' }) -join "`n"
        switch ($fault) {
            'inventory-missing' { $output = ($testNames[0..6] | ForEach-Object { $_ + ': test' }) -join "`n" }
            'inventory-duplicate' { $output += "`n" + $testNames[0] + ': test' }
            'inventory-case' { $output = $output.ToUpperInvariant() }
            'inventory-exit' { $code = 1 }
            'inventory-empty' { $output = '' }
        }
    } else {
        $script:scenarioCalls++
        $scenario = $Name.Substring('G06:'.Length)
        $position = [Array]::IndexOf($scenarioNames,$scenario)
        $owned = Join-Path $target ('native-teleport-failure/' + $scenario)
        $evidence = Join-Path $target ('verification/native-teleport-failure/' + $scenario)
        New-Item -ItemType Directory $owned,$evidence -Force | Out-Null
        $testText = $good.Replace('sample',$testNames[$position])
        $receipt = @{ status=('verified_'+$scenario); scenario=$scenario; test_binary=$binaryFixture; jar_sha256=$fakeSha; inputs_unchanged=$true; cleanup_errors=@(); drift_errors=@(); root=$owned }
        switch ($fault) {
            'zero' { $testText = $testText.Replace('running 1 test','running 0 tests').Replace('1 passed','0 passed') }
            'cleanup' { $receipt.cleanup_errors = @('synthetic cleanup failure') }
            'identity' { $testText = $good }
            'exit' { $code = 1 }
            'malformed' { $testText = 'missing result' }
        }
        [IO.File]::WriteAllText((Join-Path $owned 'rust.stdout.txt'),$testText,$utf8)
        [IO.File]::WriteAllText((Join-Path $evidence 'receipt.json'),($receipt | ConvertTo-Json),$utf8)
        $output = $evidence
    }
    [pscustomobject]@{ Output=$output; ExitCode=$code; Seconds=0.001; Log=$log }
}
$rows = @(Invoke-G06Suite 0)
Assert ($rows.Count -eq 8 -and $processCalls -eq 0) 'incomplete contract started a subprocess or dropped rows'
Assert (@($rows | Where-Object Status -eq 'PASS').Count -eq 0) 'incomplete contract partially passed'
Assert (@($rows | Where-Object Status -like '*incomplete-owned-scenarios (T023-2): cancel-repeat, refused-cancel-repeat, cancel-expiry-source*').Count -eq 8) 'missing scenarios not identified'
Assert (@($rows | Where-Object { $_.ExitCode -ne 'N/A' -or $_.Passed -ne 'N/A' }).Count -eq 0) 'unexecuted scenarios have invented results'
$supportedCount = 8; $scenarioCalls = 0
$rows = @(Invoke-G06Suite 0)
Assert ($scenarioCalls -eq 8 -and @($rows | Where-Object Status -eq 'PASS').Count -eq 8) 'eight supported synthetic scenarios did not run'
foreach ($fault in @('inventory-missing','inventory-duplicate','inventory-case','inventory-exit','inventory-empty')) {
    $scenarioCalls = 0
    $rows = @(Invoke-G06Suite 0)
    Assert ($rows.Count -eq 8 -and $scenarioCalls -eq 0) "invalid inventory started owned harness: $fault"
    Assert (@($rows | Where-Object Status -eq 'PASS').Count -eq 0) "invalid inventory passed: $fault"
}
foreach ($fault in @('zero','cleanup','identity','exit','malformed')) {
    $scenarioCalls = 0
    $rows = @(Invoke-G06Suite 0)
    Assert ($rows.Count -eq 8 -and $scenarioCalls -eq 1) "G06 did not stop after $fault"
    Assert (@($rows | Where-Object Status -eq 'PASS').Count -eq 0) "G06 accepted $fault"
    Assert (@($rows | Where-Object Status -like '*not-run-after-G06-failure*').Count -eq 7) "G06 fail-stop rows absent: $fault"
}
Write-Output "Runner QA: $checks assertions passed; PowerShell=$($PSVersionTable.PSVersion). Synthetic checks only."
