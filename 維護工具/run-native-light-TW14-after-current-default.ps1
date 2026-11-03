#Requires -Version 7.0
param([Parameter(Mandatory)][string]$JobIdentityFile)
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$stage=Join-Path $taskRoot '驗證紀錄/current/native-light-spell/current-regression'
$identityPath=[IO.Path]::GetFullPath($JobIdentityFile)
if(!$identityPath.StartsWith($stage+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Expected an owned current-regression job identity'}
$identity=Get-Content -LiteralPath $identityPath -Raw|ConvertFrom-Json
$live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$identity.pid)
if(!$live -or $live.CommandLine -cne $identity.command -or $live.ExecutablePath -ine $identity.executable){throw 'Default job identity differs'}
if([Math]::Abs(($live.CreationDate.ToUniversalTime()-([DateTime]$identity.creation_utc).ToUniversalTime()).TotalMilliseconds) -gt 1){throw 'Default job creation time differs'}
if(!$live.CommandLine.Contains('run-native-light-default91-followup.ps1') -or !$live.CommandLine.Contains($taskRoot)){throw 'Expected this project default-regression parent'}
$parent=Get-Process -Id $identity.pid
if([Math]::Abs(($parent.StartTime.ToUniversalTime()-$live.CreationDate.ToUniversalTime()).TotalMilliseconds) -gt 1){throw 'Parent handle creation time differs'}
$env:PYTHONDONTWRITEBYTECODE='1';$env:PYTHONIOENCODING='utf-8'
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$state=[ordered]@{status='waiting_on_verified_current_default_parent';waiting_pid=$identity.pid;waiting_creation_utc=$identity.creation_utc;started_utc=[DateTime]::UtcNow.ToString('o');source_files=970;new_GUI_sessions_started=$false;whole_goal_complete=$false}
$statePath=Join-Path $stage 'TW14-after-default.json'
if(Test-Path -LiteralPath $statePath){throw 'A current TW14 dispatcher receipt already exists'}
function Save {$state|ConvertTo-Json -Depth 10|Set-Content -LiteralPath $statePath -Encoding utf8}
Save
try {
 while(!$parent.WaitForExit(30000)){
  $state['last_verified_wait_utc']=[DateTime]::UtcNow.ToString('o');Save
 }
 $state['parent_exit_code']=$parent.ExitCode
 $state['parent_exit_code_available']=($null -ne $parent.ExitCode)
 # Get-Process may expose no ExitCode for an externally started process.
 # WaitForExit proves termination; exact qualified receipts below remain mandatory.
 if($null -ne $parent.ExitCode -and $parent.ExitCode -ne 0){throw 'Original default-regression parent exited unsuccessfully'}
 $follow=Get-Content -LiteralPath (Join-Path $stage 'default91-followup.json') -Raw|ConvertFrom-Json
 if($follow.status -ne 'both_profiles_default91_qualified'){throw 'Both exact default profiles must be qualified'}
 $normalPath=Join-Path $taskRoot '驗證紀錄/current/native-light-spell/normal/qualified.json'
 $normal=Get-Content -LiteralPath $normalPath -Raw|ConvertFrom-Json
 $pins=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $normalPath) 'source-inputs.json') -Raw|ConvertFrom-Json -AsHashtable
 if($pins.Count -ne 970 -or $normal.source_files -ne 970){throw 'Current source manifest differs'}
 foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw ('Source drift: '+$name)}}
 foreach($profile in @('Debug','Release')){
  $qualified=Join-Path $stage ($profile+'-default91-qualified.json')
  if((Get-FileHash -LiteralPath $qualified).Hash -ne $follow.profiles.$profile.qualification_sha256){throw 'Default qualification drift'}
  $q=Get-Content -LiteralPath $qualified -Raw|ConvertFrom-Json
  if($q.failed -ne 0 -or $q.integration_targets -ne 91 -or $q.products_retained -ne 92 -or $q.normal_mains.$profile -ne $normal.profiles.$profile.sha256){throw 'Exact normal/default binding differs'}
  if((Get-FileHash -LiteralPath $normal.profiles.$profile.path).Hash -ne $normal.profiles.$profile.sha256){throw 'Normal executable drift'}
  $retention=Join-Path $stage ($profile+'-default91-retention.json')
  if(!(Test-Path -LiteralPath $retention)){
   & python -X utf8 (Join-Path $PSScriptRoot 'seal-native-light-default-run.py') --profile $profile *> (Join-Path $stage ($profile+'-retention-console.txt'))
   if($LASTEXITCODE -ne 0){throw ('Default evidence retention failed: '+$profile)}
  }
 }
 $state['status']='starting_TW14_after_both_default_profiles_qualified';Save
 & (Join-Path $PSScriptRoot 'run-native-light-TW14-followup.ps1') *> (Join-Path $stage 'TW14-after-default-console.txt')
 if($LASTEXITCODE -ne 0){throw 'Current TW14 follow-up failed'}
 $gui=Get-Content -LiteralPath (Join-Path $stage 'current14-followup.json') -Raw|ConvertFrom-Json
 if($gui.status -ne 'qualified_current_explicit_TW14_GUI_cases'){throw 'TW14 independent qualification incomplete'}
 $state['status']=$gui.status;$state['cohort']=$gui.cohort;$state['evidence']=$gui.evidence;$state['new_GUI_sessions_started']=$true;Save
}catch{$state['status']='failed';$state['error']=$_.Exception.Message;Save;throw}
finally{$parent.Dispose()}
Write-Output ($state|ConvertTo-Json -Depth 10)
