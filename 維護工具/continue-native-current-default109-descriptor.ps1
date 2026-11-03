#Requires -Version 7.0
param([Parameter(Mandatory)][string]$FailedCohort,[Parameter(Mandatory)][string]$RetainedDebugRun)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$failedPath=(Resolve-Path -LiteralPath $FailedCohort).Path
$debugPath=(Resolve-Path -LiteralPath $RetainedDebugRun).Path
foreach($path in @($failedPath,$debugPath)){if(!$path.StartsWith((Join-Path $taskRoot '驗證紀錄/current/'),[StringComparison]::OrdinalIgnoreCase)){throw 'Owned prior evidence required'}}
$prior=Get-Content -LiteralPath (Join-Path $failedPath 'cohort.json') -Raw|ConvertFrom-Json
$debug=Get-Content -LiteralPath (Join-Path $debugPath 'receipt.json') -Raw|ConvertFrom-Json
if($prior.status -ne 'failed_preserved' -or $prior.error -ne 'Debug independent default109 readback failed' -or $prior.profiles.Debug.run -ne $debugPath -or $prior.profiles.Debug.exit_code -ne 0){throw 'Only the retained descriptor-readback failure may continue here'}
if($debug.status -ne 'verified_current_default109_integration_targets_and_main_units' -or $debug.failed -ne 0 -or $debug.owned_process){throw 'Actual Debug run must already be terminal and successful'}
$env:PYTHONDONTWRITEBYTECODE='1';$env:PYTHONIOENCODING='utf-8'
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$cohort=Join-Path $taskRoot ('驗證紀錄/current/native-default109/descriptor-continuation-'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $cohort|Out-Null
$identity=Get-CimInstance Win32_Process -Filter ('ProcessId='+$PID)
$qaNames=@('continue-native-current-default109-descriptor.ps1','run-native-current-default109.py','qualify-native-current-default109-descriptor.py','preserve-legacy-library-reports.py')
$qa=[ordered]@{}
foreach($name in $qaNames){$qa[$name]=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name)).Hash;Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination (Join-Path $cohort $name)}
$state=[ordered]@{status='continuing_current_default109_with_exact_descriptor_pair_readback';cohort=$cohort;pipeline_pid=$PID;pipeline_creation_utc=$identity.CreationDate.ToUniversalTime().ToString('o');original_failed_cohort=$failedPath;retained_Debug_run=$debugPath;Debug_tests_rerun=$false;normal_qualification=$prior.normal_qualification;G06_qualification=$prior.G06_qualification;source_manifest_sha256=$prior.source_manifest_sha256;source_files=1021;compiled_input_files=1022;validation_pins=$qa;active_profile='Debug-readback-only';profiles=[ordered]@{};source_and_corresponding_QA_frozen=$true;whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $cohort 'cohort.json') -Encoding utf8}
function Check-QA {foreach($name in $qaNames){if((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name)).Hash -ne $qa[$name]){throw ('QA drift: '+$name)}}}
Save;Write-Output ('DEFAULT109_CONTINUATION='+$cohort)
try {
 $debugQ=Join-Path $debugPath 'profile-qualified-descriptor/qualified.json'
 if(!(Test-Path -LiteralPath $debugQ)){
  & python -X utf8 (Join-Path $PSScriptRoot 'qualify-native-current-default109-descriptor.py') $debugPath
  if($LASTEXITCODE -ne 0){throw 'Retained Debug exact descriptor-pair readback failed'}
 }
 $q=Get-Content -LiteralPath $debugQ -Raw|ConvertFrom-Json
 if($q.status -ne 'independently_qualified_current_default109_integrations_and_main' -or $q.profile -ne 'Debug' -or $q.actual_test_executables -ne 110 -or $q.counts.failed -ne 0 -or $q.source_manifest_sha256 -ne $state.source_manifest_sha256){throw 'Retained Debug qualification mismatch'}
 $state.profiles['Debug']=[ordered]@{run=$debugPath;qualification=$debugQ;qualification_sha256=(Get-FileHash -LiteralPath $debugQ).Hash;counts=$q.counts;tests_rerun=$false};$state.active_profile='Release';Save
 Check-QA
 $runs=Join-Path $taskRoot '驗證紀錄/current/default-native-regression';$before=@(Get-ChildItem -LiteralPath $runs -Directory|ForEach-Object FullName)
 & python -X utf8 (Join-Path $PSScriptRoot 'run-native-current-default109.py') --profile Release --normal-qualification $prior.normal_qualification --G06-qualification $prior.G06_qualification 2>&1|Tee-Object -FilePath (Join-Path $cohort 'Release-console.log')
 $code=$LASTEXITCODE;$created=@(Get-ChildItem -LiteralPath $runs -Directory|Where-Object {$_.FullName -notin $before})
 if($created.Count -ne 1){throw 'Expected exactly one Release continuation run'}
 $run=$created[0].FullName;$state.profiles['Release']=[ordered]@{run=$run;exit_code=$code;qualified=$false};Save
 if($code -ne 0){throw 'Release actual default109 failed; preserve it'}
 & python -X utf8 (Join-Path $PSScriptRoot 'qualify-native-current-default109-descriptor.py') $run
 if($LASTEXITCODE -ne 0){throw 'Release exact descriptor-pair readback failed'}
 $qualified=Join-Path $run 'profile-qualified-descriptor/qualified.json';$q=Get-Content -LiteralPath $qualified -Raw|ConvertFrom-Json
 if($q.status -ne 'independently_qualified_current_default109_integrations_and_main' -or $q.actual_test_executables -ne 110 -or $q.counts.failed -ne 0 -or $q.source_manifest_sha256 -ne $state.source_manifest_sha256){throw 'Release qualification mismatch'}
 $state.profiles.Release.qualified=$true;$state.profiles.Release['qualification']=$qualified;$state.profiles.Release['qualification_sha256']=(Get-FileHash -LiteralPath $qualified).Hash;$state.profiles.Release['counts']=$q.counts
 Check-QA;$state.status='independently_qualified_current_default109_both_profiles_with_retained_Debug_and_exact_descriptor_pair';$state.active_profile=$null;$state.source_and_corresponding_QA_frozen=$false;Save
}catch{$state.status='failed_preserved';$state['error']=$_.Exception.Message;Save;throw}
Write-Output ($state|ConvertTo-Json -Depth 12)
