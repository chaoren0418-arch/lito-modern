#Requires -Version 7.0
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot;$stage=Join-Path $taskRoot '驗證紀錄/current/native-shared-basic-heal'
$follow=Get-Content -LiteralPath (Join-Path $stage 'default90-followup.json') -Raw|ConvertFrom-Json
if($follow.status -ne 'both_profiles_default90_qualified'){throw 'Both current default90 profiles must finish before GUI validation'}
$normal=Join-Path $stage 'normal/qualified.json';$debugRust=Join-Path $stage 'Debug-default90-qualified.json';$releaseRust=Join-Path $stage 'Release-default90-qualified.json'
$heavy=@(Get-CimInstance Win32_Process|Where-Object {($_.Name -in @('cargo.exe','rustc.exe')) -or ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($taskRoot,[StringComparison]::OrdinalIgnoreCase) -and ($_.Name -match '^(LiToModern|native_|qualified-client)' -or $_.ExecutablePath -match '\\deps\\.*\.exe$'))})
if($heavy.Count){throw 'Owned compiler/test/GUI load is live before explicit TW current14'}
$cohort=Join-Path $taskRoot ('驗證紀錄/current/native-teleport-current/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-TW-shared')
New-Item -ItemType Directory -Path $cohort|Out-Null
$state=@{status='running';cohort=$cohort;normal_qualification=$normal;resource_profile='tw13081901-v1';extended_item_details=$true;Debug_Rust=$debugRust;Release_Rust=$releaseRust;profiles=@();whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $stage 'current14-followup.json') -Encoding utf8}
Save
try {
 foreach($profile in @('Debug','Release')){
  $state.active_profile=$profile;Save
  $rust=if($profile -eq 'Debug'){$debugRust}else{$releaseRust}
  & (Join-Path $PSScriptRoot 'run-current-teleport-GUI-revalidation.ps1') -BuildProfile $profile -NormalQualification $normal -RustEvidence $rust -CohortDirectory $cohort -ReceiptLabel 'current-cohort' -ResourceProfile tw13081901-v1 -CommonCatalogPath (Join-Path $taskRoot 'LiTo381-Modern/target/current/shared-profile-inputs/common_catalog-B64ED920.bin') -ExtendedItemDetails *> (Join-Path $cohort ($profile+'-batch-console.txt'))
  if($LASTEXITCODE -ne 0){throw ('Current GUI batch failed: '+$profile)}
  $state.profiles+=$profile;Save
 }
 & python -X utf8 (Join-Path $PSScriptRoot 'qualify-native-teleport-current-cohort.py') $cohort --normal-qualification $normal --debug-rust-evidence $debugRust --release-rust-evidence $releaseRust --resource-profile tw13081901-v1 --extended-item-details *> (Join-Path $cohort 'qualification-console.txt')
 if($LASTEXITCODE -ne 0){throw 'Current fourteen-case independent qualification failed'}
 $state.status='qualified_current_explicit_TW14_GUI_cases';$state.active_profile=$null;$state.evidence=Join-Path $cohort 'qualified.json';Save
 Write-Output ($state|ConvertTo-Json -Depth 8)
}catch{$state.status='failed';$state.error=$_.Exception.Message;Save;throw}
