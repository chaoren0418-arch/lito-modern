param([Parameter(Mandatory=$true)][string]$Case)
$ErrorActionPreference='Stop'
$rootPath=[IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$casePath=[IO.Path]::GetFullPath((Resolve-Path -LiteralPath $Case).Path)
$allowedPrefix=[IO.Path]::Combine($rootPath,'驗證紀錄','current','native-installed-resource-profile-sky-gui')+[IO.Path]::DirectorySeparatorChar
if(-not $casePath.StartsWith($allowedPrefix,[StringComparison]::OrdinalIgnoreCase)){throw 'Outside owned GUI workspace'}
$queue=Get-Content -LiteralPath ([IO.Path]::Combine($rootPath,'工作佇列.json')) -Raw|ConvertFrom-Json
if($queue.active_workload_sessions.normal_gui -ne 77873 -or $queue.current_native_catalog_gui_active.case -ne $casePath){throw 'Original active session mismatch'}
$obstruction=$queue.current_GUI_input_runtime_obstruction
if($obstruction.original_session -ne 77873 -or $obstruction.case -ne $casePath){throw 'No source-matched observed input obstruction'}
if((Get-FileHash -LiteralPath $obstruction.evidence -Algorithm SHA256).Hash -ne $obstruction.evidence_sha256){throw 'Obstruction evidence changed'}
$cohort=Get-Content -LiteralPath ([IO.Path]::Combine($casePath,'cohort.json')) -Raw|ConvertFrom-Json
if($cohort.status -ne 'normal_windows_running_for_Sky_inputs' -or $cohort.processes.Count -ne 3){throw 'Cohort already changed'}
if($cohort.source_manifest_sha256 -ne 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'){throw 'Source changed'}
$targetReceipt=[IO.Path]::Combine($casePath,'owned-input-unavailable-stop.json')
if(Test-Path -LiteralPath $targetReceipt){throw 'Cleanup receipt already exists'}
$ownedHandles=@()
$records=@()
try {
 foreach($row in $cohort.processes){
  $processInfo=Get-CimInstance Win32_Process -Filter "ProcessId=$($row.pid)"
  if(-not $processInfo){throw "Owned process no longer live: $($row.role)"}
  $expectedBirth=[DateTimeOffset]$row.creation_utc
  $actualBirth=[DateTimeOffset]$processInfo.CreationDate.ToUniversalTime()
  if([Math]::Abs(($actualBirth-$expectedBirth).TotalSeconds) -ge 0.001){throw 'PID birth mismatch'}
  if(-not [string]::Equals($processInfo.ExecutablePath,$row.command[0],[StringComparison]::OrdinalIgnoreCase)){throw 'Executable mismatch'}
  if($row.role -eq 'server'){
   if($processInfo.CommandLine -notlike '*local.lito381.server.NativePeerHarness*' -or $processInfo.CommandLine -notlike ('*'+$casePath+'*') -or $processInfo.CommandLine -notlike '*gui-rental-hotel-return-reconnect*'){throw 'Owned server command mismatch'}
   if($processInfo.CommandLine -notlike ('*'+$cohort.prepared+'\classes*')){throw 'Owned classpath mismatch'}
  } elseif($row.role -in @('a','b')) {
   $product=$cohort.normal_products.($row.role)
   if($processInfo.ExecutablePath -ne $product.path -or (Get-FileHash -LiteralPath $product.path -Algorithm SHA256).Hash -ne $product.sha256){throw 'Owned GUI image mismatch'}
   if($processInfo.CommandLine -notlike ('*--native-login '+$cohort.endpoint+'*') -or $processInfo.CommandLine -notlike '*D:\lineage開發\0-客戶端*'){throw 'Owned GUI command mismatch'}
  } else {throw 'Unexpected owned role'}
  $ownedProcess=[Diagnostics.Process]::GetProcessById([int]$row.pid)
  $retainedSafeHandle=$ownedProcess.SafeHandle
  if($retainedSafeHandle.IsInvalid -or $ownedProcess.HasExited -or [Math]::Abs((([DateTimeOffset]$ownedProcess.StartTime.ToUniversalTime())-$expectedBirth).TotalSeconds) -ge 0.001){throw 'Retained process identity mismatch'}
  $ownedHandles+=@{process=$ownedProcess;safe_handle=$retainedSafeHandle;row=$row}
  $records+=@{role=$row.role;pid=$row.pid;creation_utc=$expectedBirth.ToString('o');actual_executable=$processInfo.ExecutablePath;actual_command_line=$processInfo.CommandLine;cleanup_started=$false;exit_code=$null}
 }
 # Validate the complete exact cohort before stopping any of its members.
 for($index=0;$index -lt $ownedHandles.Count;$index++){
  $entry=$ownedHandles[$index]
  if($entry.process.HasExited){throw 'Original process exited before cleanup action'}
  $records[$index].cleanup_started=$true
  $entry.process.Kill()
  if(-not $entry.process.WaitForExit(10000)){throw 'Owned cleanup wait exceeded 10 seconds'}
  $records[$index].exit_code=$entry.process.ExitCode
  $records[$index].finished_utc=[DateTimeOffset]::UtcNow.ToString('o')
 }
} finally {
 $receipt=@{status='owned_GUI_input_unavailable_failure_cleanup';case=$casePath;original_session=77873;source_manifest_sha256=$cohort.source_manifest_sha256;reason='Sky pure observation minimized; activate input guard; fresh recovery still minimized';original_GUI_normal_close_claimed=$false;GUI_qualification_granted=$false;records=$records;recorded_utc=[DateTimeOffset]::UtcNow.ToString('o')}
 $receipt|ConvertTo-Json -Depth 10|Set-Content -LiteralPath $targetReceipt -Encoding utf8NoBOM
 foreach($entry in $ownedHandles){$entry.process.Dispose()}
}
if($records.Count -ne 3 -or @($records|Where-Object {$null -eq $_.exit_code}).Count -ne 0){throw 'Incomplete owned cleanup'}
$receipt|ConvertTo-Json -Depth 10 -Compress
