#Requires -Version 7.0
param([Parameter(Mandatory)][string]$NormalQualification,[Parameter(Mandatory)][string]$FirstCase)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot;$project=Join-Path $taskRoot 'LiTo381-Modern'
$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP
$env:PYTHONDONTWRITEBYTECODE='1';$env:PYTHONIOENCODING='utf-8'
$normalPath=(Resolve-Path -LiteralPath $NormalQualification).Path
$first=(Resolve-Path -LiteralPath $FirstCase).Path
if(!$normalPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase) -or !$first.StartsWith((Join-Path $taskRoot '驗證紀錄/current/native-light-gui')+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned inputs required'}
$normal=Get-Content -LiteralPath $normalPath -Raw|ConvertFrom-Json
$pins=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $normalPath) 'source-inputs.json') -Raw|ConvertFrom-Json -AsHashtable
$toolPins=@{}
foreach($name in @('run-native-light-GUI-cases.ps1','run-isolated-native-light-window-test.ps1','verify-native-light-window.ps1','qualify-native-light-GUI.py','oracle/NativeLightHarness.java')){$toolPins[$name]=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name)).Hash}
$out=Join-Path $taskRoot ('驗證紀錄/current/native-light-spell/live/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out|Out-Null
Write-Output ('LIGHT_GUI_COHORT='+$out)
$state=@{status='running';active_case=$null;normal_qualification=$normalPath;source_files=$pins.Count;source_manifest_sha256=$normal.source_manifest_sha256;tool_pins=$toolPins;cases=@();whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 18|Set-Content -LiteralPath (Join-Path $out 'pipeline.json') -Encoding utf8}
function Stable {
 foreach($n in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $n)).Hash -ne $pins[$n]){throw ('Compilation input drift: '+$n)}}
 foreach($n in $toolPins.Keys){if((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $n)).Hash -ne $toolPins[$n]){throw 'GUI QA source drift'}}
 foreach($p in @('Debug','Release')){if((Get-FileHash -LiteralPath $normal.profiles.$p.path).Hash -ne $normal.profiles.$p.sha256){throw 'Normal main drift'}}
}
function Quiet {
 $busy=@(Get-CimInstance Win32_Process|Where-Object { $_.Name -match '^(cargo|rustc|native_.*|LiToModern.*)\.exe$' -and $_.CommandLine -and $_.CommandLine.Replace('/','\').Contains($project) })
 if($busy.Count){throw 'Wait for owned compilation/tests/UI before next GUI case'}
}
function AddCase([string]$path){
 $q=Get-Content -LiteralPath (Join-Path $path 'qualified.json') -Raw|ConvertFrom-Json
 if($q.status -ne 'qualified_actual_native_light_GUI_defined_scenario' -or $q.source_manifest_sha256 -ne $normal.source_manifest_sha256 -or $q.main_sha256 -ne $normal.profiles.($q.profile).sha256){throw 'Case identity mismatch'}
 $state.cases+=@{path=$path;profile=$q.profile;scenario=$q.scenario;resource_profile=$q.resource_profile;actual_sessions=$q.actual_sessions;qualification_sha256=(Get-FileHash -LiteralPath (Join-Path $path 'qualified.json')).Hash}
 Save
}
try {
 Stable;Quiet;AddCase $first
 if($state.cases[0].profile -ne 'Release' -or $state.cases[0].scenario -ne 'success' -or $state.cases[0].resource_profile -ne 'tw13081901-v1'){throw 'First case must be qualified current Release TW success'}
 foreach($case in @(@{profile='Release';scenario='recast';resource='tw13081901-v1'},@{profile='Release';scenario='refused';resource='tw13081901-v1'},@{profile='Release';scenario='success';resource='lito-legacy'},@{profile='Debug';scenario='success';resource='tw13081901-v1'})){
  Stable;Quiet;$label=$case.profile+'-'+$case.resource+'-'+$case.scenario;$state.active_case=$label;Save
  $log=Join-Path $out ($label+'.log')
  & pwsh -NoProfile -File (Join-Path $PSScriptRoot 'run-isolated-native-light-window-test.ps1') -BuildProfile $case.profile -Scenario $case.scenario -ResourceProfile $case.resource -NormalQualification $normalPath *> $log
  if($LASTEXITCODE -ne 0){throw ('Native light GUI runtime failed: '+$label)}
  $lines=@(Get-Content -LiteralPath $log|Where-Object {$_.StartsWith('LIGHT_GUI_OUTPUT=')})
  if($lines.Count -ne 1){throw 'Unique actual case path missing'}
  $path=[IO.Path]::GetFullPath($lines[0].Substring('LIGHT_GUI_OUTPUT='.Length))
  if(!$path.StartsWith((Join-Path $taskRoot '驗證紀錄/current/native-light-gui')+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Case outside owned evidence'}
  Stable;Quiet
  & python (Join-Path $PSScriptRoot 'qualify-native-light-GUI.py') $path *> (Join-Path $out ($label+'-qualification.txt'))
  if($LASTEXITCODE -ne 0){throw ('Independent native light GUI qualification failed: '+$label)}
  AddCase $path
 }
 Stable;Quiet;$state.active_case=$null;$state.status='five_defined_GUI_cases_qualified_pending_aggregate_archive';Save
}catch{$state.status='failed';$state.error=$_.Exception.Message;Save;throw}
