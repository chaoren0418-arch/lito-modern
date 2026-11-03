#Requires -Version 7.0
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$OutputEncoding=[Text.UTF8Encoding]::new($false)
$taskRoot=Split-Path -Parent $PSScriptRoot
$run=Join-Path $taskRoot '驗證紀錄/current/native-environment/20261016-023507-851373'
$stage=Join-Path $run 'GUI-current'
if(Test-Path -LiteralPath (Join-Path $stage 'current14-followup.json')){throw 'Existing GUI followup retained; inspect it rather than restart'}
New-Item -ItemType Directory -Path $stage -Force|Out-Null
$normal=Join-Path $run 'normal/qualified.json'
$scope=Join-Path $run 'independent-qualified.json'
$env:PYTHONDONTWRITEBYTECODE='1';$env:PYTHONIOENCODING='utf-8'
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
& python -X utf8 (Join-Path $PSScriptRoot 'native_environment_GUI_evidence.py') $normal $scope *> (Join-Path $stage 'prerequisites-console.txt')
if($LASTEXITCODE -ne 0){throw 'Exact current environment normal/scoped/formal prerequisites failed'}
$heavy=@(Get-CimInstance Win32_Process|Where-Object {
 $_.Name -in @('cargo.exe','rustc.exe') -or
 ($_.Name -eq 'java.exe' -and $_.CommandLine -and $_.CommandLine.Contains($taskRoot)) -or
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -and
  ($_.Name -match '^(LiToModern|native_|qualified-client|actual-client)' -or $_.ExecutablePath -match '\deps\.*\.exe$'))
})
if($heavy.Count){throw 'Owned compiler/test/GUI/Java load live before explicit TW14'}
$cohort=Join-Path $taskRoot ('驗證紀錄/current/native-teleport-current/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-TW-environment')
New-Item -ItemType Directory -Path $cohort|Out-Null
$state=@{status='running';cohort=$cohort;normal_qualification=$normal;scoped_Rust_evidence=$scope;
 resource_profile='tw13081901-v1';extended_item_details=$true;profiles=@();source_files=978;additional_compile_fixtures=4;
 pipeline_pid=$PID;pipeline_creation_utc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o');
 source_and_corresponding_tests_frozen=$true;full_default_regression_verified=$false;whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $stage 'current14-followup.json') -Encoding utf8}
Save
try {
 foreach($profile in @('Debug','Release')){
  $state.active_profile=$profile;Save
  & (Join-Path $PSScriptRoot 'run-native-environment-current-teleport-GUI.ps1') -BuildProfile $profile -NormalQualification $normal -RustEvidence $scope -CohortDirectory $cohort -ReceiptLabel 'current-cohort' -ResourceProfile tw13081901-v1 -CommonCatalogPath (Join-Path $taskRoot 'LiTo381-Modern/target/current/shared-profile-inputs/common_catalog-B64ED920.bin') -ExtendedItemDetails *> (Join-Path $cohort ($profile+'-batch-console.txt'))
  if($LASTEXITCODE -ne 0){throw ('Current environment GUI batch failed: '+$profile)}
  $state.profiles+=$profile;Save
 }
 & python -X utf8 (Join-Path $PSScriptRoot 'qualify-native-environment-teleport-current-cohort.py') $cohort --normal-qualification $normal --debug-rust-evidence $scope --release-rust-evidence $scope --resource-profile tw13081901-v1 --extended-item-details *> (Join-Path $cohort 'qualification-console.txt')
 if($LASTEXITCODE -ne 0){throw 'Current fourteen-case independent qualification failed'}
 $state.status='qualified_current_environment_TW14_GUI_cases';$state.active_profile=$null
 $state.evidence=Join-Path $cohort 'qualified.json';$state.source_and_corresponding_tests_frozen=$false;Save
 Write-Output ($state|ConvertTo-Json -Depth 8)
}catch{$state.status='failed';$state.error=$_.Exception.Message;Save;throw}
