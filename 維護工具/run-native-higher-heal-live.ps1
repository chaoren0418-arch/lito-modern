#Requires -Version 7.0
[CmdletBinding()]param([Parameter(Mandatory)][string]$NormalQualification)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
$normalPath=(Resolve-Path -LiteralPath $NormalQualification).Path
if(!$normalPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned normal qualification required'}
$normal=Get-Content -LiteralPath $normalPath -Raw|ConvertFrom-Json
$mapPath=Join-Path (Split-Path -Parent $normalPath) 'source-inputs.json'
$pins=Get-Content -LiteralPath $mapPath -Raw|ConvertFrom-Json -AsHashtable
if($pins.Count -ne 968 -or $normal.source_files -ne 968 -or (Get-FileHash -LiteralPath $mapPath).Hash -ne $normal.source_manifest_sha256){throw 'Complete current source map required'}
function Stable {
 foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw ('Compilation input drift: '+$name)}}
 foreach($profile in @('Debug','Release')){
  $main=$normal.profiles.$profile
  if(!$main.unique_entry -or $main.features.Count -ne 0 -or $main.machine -ne 'AMD64' -or (Get-FileHash -LiteralPath $main.path).Hash -ne $main.sha256){throw 'Qualified normal main drift'}
 }
}
function Quiet {
 $busy=@(Get-CimInstance Win32_Process|Where-Object {
  $_.Name -match '^(cargo|rustc|lito_modern-.*)\.exe$' -or
  ($_.Name -match '^(native_.*|rebuild_.*|LiToModern.*|qualified-client)\.exe$' -and $_.CommandLine -and $_.CommandLine.Replace('/','\').Contains($project))
 })
 if($busy.Count){throw 'Wait for owned compilation/tests/GUI to terminate'}
}
Stable;Quiet
$catalog=Join-Path $project 'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
if((Get-FileHash -LiteralPath $catalog).Hash -ne 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'){throw 'Owned common catalog identity differs'}
$out=Join-Path $taskRoot ('驗證紀錄/current/native-shared-higher-heals/live/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out|Out-Null
Copy-Item -LiteralPath $mapPath -Destination (Join-Path $out 'source-inputs.json')
Copy-Item -LiteralPath $normalPath -Destination (Join-Path $out 'normal-qualification.json')
$toolPins=@{}
foreach($name in @('run-native-higher-heal-live.ps1','verify-native-real-peers.ps1','verify-real-peer-higher-heal.py','owned-peer-validation-binding.ps1')){
 $tool=Join-Path $PSScriptRoot $name;Copy-Item -LiteralPath $tool -Destination (Join-Path $out $name);$toolPins[$name]=(Get-FileHash -LiteralPath $tool).Hash
}
$state=@{status='running';normal_qualification=$normalPath;normal_qualification_sha256=(Get-FileHash -LiteralPath $normalPath).Hash;source_manifest_sha256=$normal.source_manifest_sha256;source_files=968;normal_mains=$normal.profiles;tool_pins=$toolPins;cases=@();actual_higher_heal_sessions_observed=0;whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 20|Set-Content -LiteralPath (Join-Path $out 'pipeline.json') -Encoding utf8}
$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP;$env:PYTHONDONTWRITEBYTECODE='1'
Save;Write-Output ('HIGHER_HEAL_LIVE_OUTPUT='+$out)
try {
 $cases=@()
 foreach($skill in @(19,35,57)){
  foreach($suffix in @('','-full','-refused')){$cases+=@{build='Release';resource='tw13081901-v1';action=('heal'+$skill+$suffix)}}
  $cases+=@{build='Release';resource='lito-legacy';action=('heal'+$skill)}
  $cases+=@{build='Debug';resource='tw13081901-v1';action=('heal'+$skill)}
 }
 foreach($case in $cases){
  Stable;Quiet
  foreach($name in $toolPins.Keys){if((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name)).Hash -ne $toolPins[$name]){throw 'Live validation tool changed'}}
  $label=$case.build+'-'+$case.resource+'-'+$case.action
  $log=Join-Path $out ($label+'.txt')
  $args=@('-NoProfile','-File',(Join-Path $PSScriptRoot 'verify-native-real-peers.ps1'),'-SkipBuild','-BuildProfile',$case.build,'-ActionProfile',$case.action,'-ResourceProfile',$case.resource)
  if($case.resource -eq 'tw13081901-v1'){$args+=@('-CommonCatalogPath',$catalog,'-ExtendedItemDetails')}
  & pwsh @args *> $log
  $exitCode=$LASTEXITCODE
  $lines=@(Get-Content -LiteralPath $log|Where-Object {$_.StartsWith('PEER_CASE_OUTPUT=')})
  if($lines.Count -ne 1){throw ('Missing unique case output: '+$label)}
  $path=[IO.Path]::GetFullPath($lines[0].Substring('PEER_CASE_OUTPUT='.Length))
  if(!$path.StartsWith((Join-Path $taskRoot '驗證紀錄/current/native-real-peers')+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Case path outside owned evidence'}
  $record=@{label=$label;build_profile=$case.build;resource_profile=$case.resource;action_profile=$case.action;exit_code=$exitCode;case=$path;log=$log;log_sha256=(Get-FileHash -LiteralPath $log).Hash;no_heavy_job_before=$true}
  $result=Join-Path $path 'result.json'
  if(Test-Path -LiteralPath $result){$server=Get-Content -LiteralPath $result -Raw|ConvertFrom-Json;$record.actual_client_sessions=$server.actual_client_sessions;$state.actual_higher_heal_sessions_observed+=$server.actual_client_sessions}
  $state.cases+=,$record;Save
  Stable;Quiet
  if($exitCode -ne 0){throw ('Actual higher heal case failed; retained for review: '+$label)}
  $marker=Get-Content -LiteralPath (Join-Path $path 'validation-completed.json') -Raw|ConvertFrom-Json
  if($marker.receipt_sha256 -ne (Get-FileHash -LiteralPath (Join-Path $path 'receipt.json')).Hash -or $marker.runtime_receipt_rewritten){throw 'Validation marker binding differs'}
  Write-Output ('HIGHER_HEAL_CASE_COMPLETED='+$label)
 }
 Stable;Quiet;$state.status='completed_pending_independent_qualification';Save
}catch{$state.status='failed_retained';$state.error=$_.Exception.Message;Save;throw}
