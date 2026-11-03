#Requires -Version 7.0
param([Parameter(Mandatory)][string]$NormalQualification,[Parameter(Mandatory)][string]$G06Qualification)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$normalPath=(Resolve-Path -LiteralPath $NormalQualification).Path
$g06Path=(Resolve-Path -LiteralPath $G06Qualification).Path
foreach($path in @($normalPath,$g06Path)){if(!$path.StartsWith((Join-Path $taskRoot '驗證紀錄/current/'),[StringComparison]::OrdinalIgnoreCase)){throw 'Owned prerequisite required'}}
$normal=Get-Content -LiteralPath $normalPath -Raw|ConvertFrom-Json
$g06=Get-Content -LiteralPath $g06Path -Raw|ConvertFrom-Json
if($g06.status -ne 'independently_qualified_both_current_G06_controller_profiles_six_cases_each' -or $g06.actual_formal_sessions -ne 96 -or $g06.source_manifest_sha256 -ne $normal.source_manifest_sha256){throw 'Current terminal G06 controller proof required'}
$cohort=Join-Path $taskRoot ('驗證紀錄/current/native-default109/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $cohort|Out-Null
$identity=Get-CimInstance Win32_Process -Filter ('ProcessId='+$PID)
$qaNames=@('run-native-current-default109-both.ps1','run-native-current-default109.py','qualify-native-current-default109.py','preserve-legacy-library-reports.py')
$qa=[ordered]@{}
foreach($name in $qaNames){$qa[$name]=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name)).Hash;Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination (Join-Path $cohort $name)}
$state=[ordered]@{status='running_current_default109_both_profiles';cohort=$cohort;pipeline_pid=$PID;pipeline_creation_utc=$identity.CreationDate.ToUniversalTime().ToString('o');normal_qualification=$normalPath;G06_qualification=$g06Path;source_files=1021;compiled_input_files=1022;source_manifest_sha256=$normal.source_manifest_sha256;validation_pins=$qa;active_profile=$null;profiles=[ordered]@{};source_and_corresponding_QA_frozen=$true;GUI_verified=$false;whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $cohort 'cohort.json') -Encoding utf8}
function Check-QA {foreach($name in $qaNames){if((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name)).Hash -ne $qa[$name]){throw ('QA drift: '+$name)}}}
$env:PYTHONDONTWRITEBYTECODE='1';$env:PYTHONIOENCODING='utf-8'
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$rust='C:/Users/CHAOREN/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
$env:RUSTC=Join-Path $rust 'rustc.exe';$env:RUSTDOC=Join-Path $rust 'rustdoc.exe';$env:PATH=$rust+';'+$env:PATH
Save;Write-Output ('DEFAULT109_COHORT='+$cohort)
try {
 foreach($profile in @('Debug','Release')){
  Check-QA;$state.active_profile=$profile;Save
  $runs=Join-Path $taskRoot '驗證紀錄/current/default-native-regression'
  $before=@(Get-ChildItem -LiteralPath $runs -Directory|ForEach-Object FullName)
  & python -X utf8 (Join-Path $PSScriptRoot 'run-native-current-default109.py') --profile $profile --normal-qualification $normalPath --G06-qualification $g06Path 2>&1|Tee-Object -FilePath (Join-Path $cohort ($profile+'-console.log'))
  $code=$LASTEXITCODE
  $created=@(Get-ChildItem -LiteralPath $runs -Directory|Where-Object {$_.FullName -notin $before})
  if($created.Count -ne 1){throw 'Expected exactly one current default109 run'}
  $run=$created[0].FullName
  $state.profiles[$profile]=[ordered]@{run=$run;exit_code=$code;qualified=$false};Save
  if($code -ne 0){throw ($profile+' default109 failed; retain its original products/source/logs')}
  & python -X utf8 (Join-Path $PSScriptRoot 'qualify-native-current-default109.py') $run
  if($LASTEXITCODE -ne 0){throw ($profile+' independent default109 readback failed')}
  $qualification=Join-Path $run 'profile-qualified/qualified.json'
  $q=Get-Content -LiteralPath $qualification -Raw|ConvertFrom-Json
  if($q.status -ne 'independently_qualified_current_default109_integrations_and_main' -or $q.actual_test_executables -ne 110 -or $q.integration_targets -ne 109 -or $q.counts.failed -ne 0){throw 'Incomplete current default109 qualification'}
  $state.profiles[$profile].qualified=$true;$state.profiles[$profile]['qualification']=$qualification;$state.profiles[$profile]['qualification_sha256']=(Get-FileHash -LiteralPath $qualification).Hash;$state.profiles[$profile]['counts']=$q.counts;Save
  Check-QA
 }
 $state.status='independently_qualified_current_default109_both_profiles';$state.active_profile=$null;$state.source_and_corresponding_QA_frozen=$false;Save
}catch{$state.status='failed_preserved';$state['error']=$_.Exception.Message;Save;throw}
Write-Output ($state|ConvertTo-Json -Depth 10)
