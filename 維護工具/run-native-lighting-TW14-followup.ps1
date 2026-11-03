#Requires -Version 7.0
param([string]$NormalQualification='',[string]$RustEvidence='')
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$application=Join-Path $taskRoot '驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788'
$normal=if($NormalQualification){[IO.Path]::GetFullPath($NormalQualification)}else{Join-Path $application 'normal-current/qualified.json'}
$rust=if($RustEvidence){[IO.Path]::GetFullPath($RustEvidence)}else{Join-Path $application 'normal-current/scoped-qualified.json'}
$env:PYTHONDONTWRITEBYTECODE='1';$env:PYTHONIOENCODING='utf-8'
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$installedRust='C:/Users/CHAOREN/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
$env:RUSTC=Join-Path $installedRust 'rustc.exe';$env:RUSTDOC=Join-Path $installedRust 'rustdoc.exe'
$env:PATH=$installedRust+';C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin;'+$env:PATH
$live=@(Get-CimInstance Win32_Process|Where-Object {
 $_.Name -in @('cargo.exe','rustc.exe') -or
 ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($taskRoot)) -or
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -and
 ($_.Name -match '^(LiToModern|native_|qualified-client|actual-client)' -or $_.ExecutablePath -match '\\deps\\.*\.exe$'))
})
if($live.Count){throw 'Owned compilation/test/server/client load must finish before current GUI'}
& python -X utf8 (Join-Path $PSScriptRoot 'native_lighting_GUI_evidence.py') $normal $rust
if($LASTEXITCODE -ne 0){throw 'Current lighting source/test/main binding failed'}
$cohort=Join-Path $taskRoot ('驗證紀錄/current/native-teleport-current/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-TW-lighting')
New-Item -ItemType Directory -Path $cohort|Out-Null
$statePath=Join-Path (Split-Path -Parent $normal) 'GUI-current.json'
if(Test-Path -LiteralPath $statePath){throw 'Existing lighting GUI dispatcher retained'}
$identity=Get-CimInstance Win32_Process -Filter ('ProcessId='+$PID)
$state=[ordered]@{status='running_current_lighting_TW14_GUI';pipeline_pid=$PID;pipeline_creation_utc=$identity.CreationDate.ToUniversalTime().ToString('o');cohort=$cohort;source_files=(Get-Content -LiteralPath $normal -Raw|ConvertFrom-Json).source_files;source_and_corresponding_tests_frozen=$true;normal_qualification=$normal;scoped_Rust_evidence=$rust;active_profile=$null;resource_profile='tw13081901-v1';extended_item_details=$true;whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 12|Set-Content -LiteralPath $statePath -Encoding utf8}
Save
try {
 foreach($profile in @('Debug','Release')){
  $state.active_profile=$profile;Save
  & (Join-Path $PSScriptRoot 'run-native-lighting-current-teleport-GUI.ps1') -BuildProfile $profile -NormalQualification $normal -RustEvidence $rust -CohortDirectory $cohort -ReceiptLabel 'current-cohort' -ResourceProfile tw13081901-v1 -CommonCatalogPath (Join-Path $taskRoot 'LiTo381-Modern/target/current/shared-profile-inputs/common_catalog-B64ED920.bin') -ExtendedItemDetails *> (Join-Path $cohort ($profile+'-batch-console.txt'))
 }
 & python -X utf8 (Join-Path $PSScriptRoot 'qualify-native-lighting-teleport-current-cohort.py') $cohort --normal-qualification $normal --debug-rust-evidence $rust --release-rust-evidence $rust --resource-profile tw13081901-v1 --extended-item-details *> (Join-Path $cohort 'independent-qualification-console.txt')
 if($LASTEXITCODE -ne 0){throw 'Current lighting GUI cohort independent qualification failed'}
 $q=Get-Content -LiteralPath (Join-Path $cohort 'qualified.json') -Raw|ConvertFrom-Json
 if($q.status -ne 'qualified_current_lighting_normal_mains14_GUI_teleport_and_recovery_cases'){throw 'Incomplete current lighting cohort'}
 $state.status='qualified_current_lighting_TW14_GUI_cases';$state.active_profile=$null;$state['evidence']=Join-Path $cohort 'qualified.json';$state.source_and_corresponding_tests_frozen=$false;Save
}catch{$state.status='failed_preserved';$state['error']=$_.Exception.Message;Save;throw}
Write-Output ($state|ConvertTo-Json -Depth 12)
