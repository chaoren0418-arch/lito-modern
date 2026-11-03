#Requires -Version 7.0
param()
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$stage=Join-Path $taskRoot '驗證紀錄/current/native-draw-flags/body-half'
$expected=Get-Content -LiteralPath (Join-Path $stage 'strengthened-build-inputs-before.json') -Raw|ConvertFrom-Json -AsHashtable
$reference=Join-Path $taskRoot 'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar'
$batch=[ordered]@{status='running';cases=@();active=$null;whole_goal_complete=$false}
function Save {$batch|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $stage 'runtime-revalidation.json') -Encoding utf8}
function VerifySources {foreach($name in $expected.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $expected[$name]){throw ('Compilation input drift: '+$name)}}}
Save
try {
 foreach($profile in @('Debug','Release')){
  VerifySources
  $related=Get-Content -LiteralPath (Join-Path $stage ($profile+'-strengthened-qualified.json')) -Raw|ConvertFrom-Json
  $binary=($related.products|Where-Object suite -eq 'native_world_entry').path
  $batch.active=$profile+' controller classic-lab';Save
  $caseRoot=Join-Path $taskRoot '驗證紀錄/current/native-world-entry'
  $before=@(Get-ChildItem -LiteralPath $caseRoot -Directory|Select-Object -ExpandProperty Name)
  & (Join-Path $PSScriptRoot 'run-native-world-entry.ps1') -TestBinary $binary -ReferenceJar $reference -BuildProfile $profile *> (Join-Path $stage ($profile+'-current-controller.log'))
  $cases=@(Get-ChildItem -LiteralPath $caseRoot -Directory|Where-Object {$_.Name -notin $before})
  if($cases.Count -ne 1){throw 'Cannot uniquely bind controller case'}
  $case=$cases[0].FullName
  & python -X utf8 (Join-Path $PSScriptRoot 'verify-native-world-entry.py') $case *> (Join-Path $stage ($profile+'-current-controller-comparison.log'))
  if($LASTEXITCODE -ne 0){throw 'Controller comparison failed'}
  VerifySources
  $batch.cases+=@{profile=$profile;kind='controller';scenario='classic-lab';path=$case;full962_sources_unchanged=$true};Save
  foreach($scenario in @('disabled','classic-lab')){
   $batch.active=$profile+' GUI '+$scenario;Save
   $caseRoot=Join-Path $taskRoot '驗證紀錄/current/native-world-tree-gui'
   $before=@(Get-ChildItem -LiteralPath $caseRoot -Directory|Select-Object -ExpandProperty Name)
   & (Join-Path $PSScriptRoot 'run-isolated-native-world-tree-window-test.ps1') -BuildProfile $profile -Scenario $scenario *> (Join-Path $stage ($profile+'-GUI-'+$scenario+'.log'))
   $cases=@(Get-ChildItem -LiteralPath $caseRoot -Directory|Where-Object {$_.Name -notin $before})
   if($cases.Count -ne 1){throw 'Cannot uniquely bind GUI case'}
   $case=$cases[0].FullName
   & python -X utf8 (Join-Path $PSScriptRoot 'verify-native-world-tree-gui.py') $case *> (Join-Path $stage ($profile+'-GUI-'+$scenario+'-comparison.log'))
   if($LASTEXITCODE -ne 0){throw 'GUI comparison failed'}
   VerifySources
   $batch.cases+=@{profile=$profile;kind='GUI';scenario=$scenario;path=$case;full962_sources_unchanged=$true};Save
  }
 }
 $batch.status='verified_two_profiles_controller_and_two_GUI_scenarios';$batch.active=$null;Save
}catch{$batch.status='failed';$batch.error=$_.Exception.Message;Save;throw}
