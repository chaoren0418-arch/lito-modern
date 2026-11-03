param([Parameter(Mandatory=$true)][string]$Case)
$ErrorActionPreference='Stop'
$rootPath=[IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$casePath=[IO.Path]::GetFullPath((Resolve-Path -LiteralPath $Case).Path)
$prefix=[IO.Path]::Combine($rootPath,'驗證紀錄','current','native-installed-resource-profile-sky-gui')+[IO.Path]::DirectorySeparatorChar
if(-not $casePath.StartsWith($prefix,[StringComparison]::OrdinalIgnoreCase)){throw 'Outside owned GUI workspace'}
$queue=Get-Content -LiteralPath ([IO.Path]::Combine($rootPath,'工作佇列.json')) -Raw|ConvertFrom-Json
if($queue.active_workload_sessions.normal_gui -ne 23695 -or $queue.current_native_catalog_gui_active.case -ne $casePath){throw 'Active case mismatch'}
$c=Get-Content -LiteralPath ([IO.Path]::Combine($casePath,'cohort.json')) -Raw|ConvertFrom-Json
if($c.source_manifest_sha256 -ne '3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628' -or $c.scenario -ne 'rental-entry' -or $c.status -ne 'normal_windows_running_for_Sky_inputs'){throw 'Unexpected cohort'}
if(@($c.processes|Where-Object {$_.role -in @('a','b') -and $_.exit_code -eq 0}).Count -ne 2){throw 'Both GUI clients must already have exited normally'}
$server=@($c.processes|Where-Object {$_.role -eq 'server'});if($server.Count -ne 1){throw 'Server ambiguity'}
$receiptPath=[IO.Path]::Combine($casePath,'owned-server-stop-user-desktop.json')
if(Test-Path -LiteralPath $receiptPath){throw 'Receipt already exists'}
$row=$server[0];$info=Get-CimInstance Win32_Process -Filter "ProcessId=$($row.pid)"
if(-not $info -or $info.ExecutablePath -ne $row.command[0] -or $info.CommandLine -notlike ('*'+$casePath+'*') -or $info.CommandLine -notlike '*local.lito381.server.NativePeerHarness*' -or $info.CommandLine -notlike '*gui-rental-entry*'){throw 'Owned server command mismatch'}
$birth=[DateTimeOffset]$row.creation_utc
if([Math]::Abs((([DateTimeOffset]$info.CreationDate.ToUniversalTime())-$birth).TotalSeconds) -ge .001){throw 'Server birth mismatch'}
$owned=[Diagnostics.Process]::GetProcessById([int]$row.pid)
try {
 $retained=$owned.SafeHandle
 if($retained.IsInvalid -or $owned.HasExited -or [Math]::Abs((([DateTimeOffset]$owned.StartTime.ToUniversalTime())-$birth).TotalSeconds) -ge .001){throw 'Retained identity mismatch'}
 $owned.Kill();if(-not $owned.WaitForExit(10000)){throw 'Owned cleanup deadline'}
 $receipt=@{status='owned_waiting_server_stopped_user_desktop_requested';case=$casePath;original_session=23695;pid=$row.pid;creation_utc=$row.creation_utc;original_process_handle_retained=$true;exit_code=$owned.ExitCode;GUI_qualification_granted=$false;reason='User requested use of computer; defer tests and continue development';finished_utc=[DateTimeOffset]::UtcNow.ToString('o')}
 $receipt|ConvertTo-Json -Depth 8|Set-Content -LiteralPath $receiptPath -Encoding utf8NoBOM
 $receipt|ConvertTo-Json -Compress
} finally {$owned.Dispose()}
