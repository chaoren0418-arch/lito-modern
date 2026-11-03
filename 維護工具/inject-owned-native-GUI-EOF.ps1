param([Parameter(Mandatory=$true)][string]$Case)
$ErrorActionPreference='Stop'
$taskRoot=[IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$caseRoot=[IO.Path]::GetFullPath($Case)
$validationRoot=[IO.Path]::GetFullPath((Join-Path $taskRoot '驗證紀錄/current/native-installed-resource-profile-sky-gui'))
if(!$caseRoot.StartsWith($validationRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Case must remain in the owned rebuild GUI validation directory'}
$cohort=Get-Content -LiteralPath (Join-Path $caseRoot 'cohort.json') -Raw|ConvertFrom-Json
$plan=Get-Content -LiteralPath (Join-Path $caseRoot 'EOF-negative-plan.json') -Raw|ConvertFrom-Json
$queue=Get-Content -LiteralPath (Join-Path $taskRoot '工作佇列.json') -Raw|ConvertFrom-Json
if($cohort.source_manifest_sha256 -ne 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6' -or $plan.source_manifest_sha256 -ne $cohort.source_manifest_sha256 -or $queue.current_product_identity.source_manifest_sha256 -ne $cohort.source_manifest_sha256){throw 'Current source identity differs'}
if($cohort.scenario -ne 'rental-entry' -or $cohort.status -ne 'normal_windows_running_for_Sky_inputs' -or $queue.current_native_catalog_gui_active.case -ne $caseRoot -or !$queue.current_full_default_library_verified){throw 'Expected active GUI negative scope missing'}
$events=Get-Content -LiteralPath (Join-Path $caseRoot 'Sky-events.json') -Raw|ConvertFrom-Json
$lastEvent=@($events)[-1]
if($lastEvent.role -ne 'a' -or $lastEvent.label -ne 'eof-world-restored-before' -or $null -ne $lastEvent.action -or $lastEvent.tree -notmatch '地圖 500' -or $lastEvent.tree -notmatch 'ID: 318' -or @($events|Where-Object {$_.role -eq 'b' -and $_.label -eq 'login'}).Count -ne 0){throw 'Actual current owned world before EOF has not been observed'}
$rows=@($cohort.processes|Where-Object {$_.role -eq 'server'})
if($rows.Count -ne 1){throw 'Expected exactly one recorded server child'}
$row=$rows[0]
$target=Get-Process -Id $row.pid
$retainedHandle=$target.SafeHandle
$actualBirth=[DateTimeOffset]$target.StartTime.ToUniversalTime()
$expectedBirth=[DateTimeOffset]$row.creation_utc
if([Math]::Abs(($actualBirth-$expectedBirth).TotalSeconds) -gt 0.001){throw 'Recorded server creation identity differs'}
$live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$row.pid)
$java='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe'
if(!$live -or $live.ExecutablePath -ne $java -or $row.command[0] -ne $java -or !$live.CommandLine.Contains($caseRoot) -or !$live.CommandLine.Contains('local.lito381.server.NativePeerHarness') -or !$live.CommandLine.Contains('gui-rental-entry')){throw 'Owned server image or command scope differs'}
$record=[ordered]@{status='intentional_owned_server_EOF_injection';pid=$row.pid;creation_utc=$actualBirth.ToString('o');CIM_before=$live.CommandLine;image=$live.ExecutablePath;original_process_handle_retained=(!$retainedHandle.IsInvalid);scope=$caseRoot;plan_sha256=(Get-FileHash -LiteralPath (Join-Path $caseRoot 'EOF-negative-plan.json') -Algorithm SHA256).Hash;pre_fault_label=$lastEvent.label;pre_fault_observed_utc=$lastEvent.observed_utc;reason='Declared negative test of ordinary connection failure retirement; synthetic owned loopback server only';started_utc=[DateTime]::UtcNow.ToString('o');GUI_qualification_granted=$false}
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
Stop-OwnedLitoValidationChild -Process $target -ScopeRoot $caseRoot
$record.exit_code=$target.ExitCode
$record.finished_utc=[DateTime]::UtcNow.ToString('o')
$record|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $caseRoot 'owned-EOF-injection.json') -Encoding utf8
$record|ConvertTo-Json -Depth 8 -Compress
