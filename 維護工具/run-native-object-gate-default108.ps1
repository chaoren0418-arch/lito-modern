#Requires -Version 7.0
param([Parameter(Mandatory)][string]$NormalQualification)
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$normal=[IO.Path]::GetFullPath($NormalQualification)
$normalDirectory=Split-Path -Parent $normal
$statePath=Join-Path $normalDirectory 'default108-current.json'
if(Test-Path -LiteralPath $statePath){throw 'Retain the existing default108 pipeline; do not restart it.'}
$env:PYTHONDONTWRITEBYTECODE='1';$env:PYTHONIOENCODING='utf-8'
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$installedRust='C:/Users/CHAOREN/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
$env:RUSTC=Join-Path $installedRust 'rustc.exe';$env:RUSTDOC=Join-Path $installedRust 'rustdoc.exe'
$env:PATH=$installedRust+';C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin;'+$env:PATH
$completed=Get-Content -LiteralPath (Join-Path $normalDirectory 'GUI-completion-observation.json') -Raw|ConvertFrom-Json
if($completed.status -ne 'independently_readback_verified_current1019_TW14_GUI_and_terminal_process'){throw 'Current GUI has not been independently read back.'}
$identity=Get-CimInstance Win32_Process -Filter ('ProcessId='+$PID)
$state=[ordered]@{status='running_current_source1019_default108_both_profiles';pipeline_pid=$PID;pipeline_creation_utc=$identity.CreationDate.ToUniversalTime().ToString('o');normal_qualification=$normal;source_files=1019;integration_targets=108;main_unit_targets=1;library_unit_tests=$false;source_and_corresponding_tests_frozen=$true;active_profile=$null;profiles=[ordered]@{};whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 10|Set-Content -LiteralPath $statePath -Encoding utf8}
Save
try {
 foreach($profile in @('Debug','Release')){
  $state.active_profile=$profile;Save
  $runsRoot=Join-Path $taskRoot '驗證紀錄/current/default-native-regression'
  $before=@(Get-ChildItem -LiteralPath $runsRoot -Directory|ForEach-Object FullName)
  & python -X utf8 (Join-Path $PSScriptRoot 'run-native-object-gate-default-regression.py') --profile $profile --normal-qualification $normal 2>&1|Tee-Object -FilePath (Join-Path $normalDirectory ('default108-'+$profile+'-console.txt'))
  if($LASTEXITCODE -ne 0){throw ($profile+' default108 failed; preserve its source, products, logs and receipt.')}
  $created=@(Get-ChildItem -LiteralPath $runsRoot -Directory|Where-Object {$_.FullName -notin $before})
  if($created.Count -ne 1){throw 'Expected exactly one new default run.'}
  $run=$created[0].FullName
  $qualification=Join-Path $run 'profile-qualified.json'
  & python -X utf8 (Join-Path $PSScriptRoot 'qualify-native-object-gate-default-regression.py') $run --output $qualification --normal-qualification $normal
  if($LASTEXITCODE -ne 0){throw ($profile+' independent default108 qualification failed.')}
  $q=Get-Content -LiteralPath $qualification -Raw|ConvertFrom-Json
  if($q.status -ne 'qualified_current_default_Rust_results' -or $q.failed -ne 0 -or $q.integration_targets -ne 108 -or $q.products_retained -ne 109){throw 'Incomplete default108 result.'}
  $state.profiles[$profile]=[ordered]@{run=$run;qualification=$qualification;qualification_sha256=(Get-FileHash -LiteralPath $qualification -Algorithm SHA256).Hash;passed=$q.passed;failed=$q.failed;ignored=$q.ignored;products_retained=$q.products_retained}
  Save
 }
 $state.status='qualified_current_source1019_default108_both_profiles'
 $state.active_profile=$null
 Save
}catch{$state.status='failed_preserved';$state['error']=$_.Exception.Message;Save;throw}
Write-Output ($state|ConvertTo-Json -Depth 10)
