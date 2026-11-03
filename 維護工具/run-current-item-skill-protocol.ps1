#Requires -Version 7.0
[CmdletBinding()]param([string]$SourceManifest='驗證紀錄/current/native-resource-optin/arrow-frame-comparison-repair/source-inputs.json',[string]$NormalQualification='驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/qualified.json')
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$map=[IO.Path]::GetFullPath((Join-Path $taskRoot $SourceManifest))
$normalPath=if([IO.Path]::IsPathFullyQualified($NormalQualification)){[IO.Path]::GetFullPath($NormalQualification)}else{[IO.Path]::GetFullPath((Join-Path $taskRoot $NormalQualification))}
if(!$normalPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Normal qualification outside rebuild root'}
$busy=Get-CimInstance Win32_Process|Where-Object {$_.Name -match '^(cargo|rustc|lito_modern-.*)\.exe$' -and $_.CommandLine -and $_.CommandLine.Replace('/','\').Contains((Join-Path $taskRoot 'LiTo381-Modern'))}
if($busy){throw 'Owned compilation or lib tests remain live; do not overlap this protocol run'}
$out=Join-Path $taskRoot ('驗證紀錄/current/native-item-skill-current/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff'));New-Item -ItemType Directory -Path $out|Out-Null
$index=@{status='running';output=$out;source_manifest=$map;normal_qualification=$normalPath;normal_qualification_sha256=(Get-FileHash -LiteralPath $normalPath).Hash;source_manifest_sha256=(Get-FileHash -LiteralPath $map).Hash;steps=@();whole_goal_complete=$false}
function Save {$index|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $out 'index.json') -Encoding utf8}
Save
$groups=@(
 @{group='inventory-actions';tool='verify-native-item-actions.ps1';flags=@('-InventoryOnly')},
 @{group='combat-self';tool='verify-native-combat.ps1';flags=@('-SelfOnly')},
 @{group='item-drop';tool='verify-native-item-actions.ps1';flags=@('-GroundOnly')},
 @{group='reagent-skills';tool='verify-native-reagent-skills.ps1';flags=@()},
 @{group='ground-pickup';tool='verify-native-ground-pickup.ps1';flags=@()}
)
try {
 foreach($profile in @('Debug','Release')){foreach($group in $groups){
  $log=Join-Path $out ($profile+'-'+$group.group+'.log')
  $args=@('-NoProfile','-File',(Join-Path $PSScriptRoot $group.tool),'-SkipBuild','-BuildProfile',$profile,'-SourceManifest',$map,'-NormalQualification',$normalPath)+$group.flags
  & pwsh @args *> $log
  if($LASTEXITCODE -ne 0){throw ('Protocol case group failed: '+$profile+'/'+$group.group+'; '+$log)}
  $line=Get-Content -LiteralPath $log|Where-Object {$_.StartsWith('PROTOCOL_RECEIPT=')}|Select-Object -Last 1
  if(!$line){throw 'Immutable receipt path missing from tool output'}
  $receipt=$line.Substring('PROTOCOL_RECEIPT='.Length)
  if(!(Test-Path -LiteralPath $receipt)){throw 'Immutable receipt missing'}
  $index.steps+=@{profile=$profile;group=$group.group;receipt=$receipt;receipt_sha256=(Get-FileHash -LiteralPath $receipt).Hash};Save
 }}
 $index.status='runtime_completed_pending_independent_qualification';Save
 & python -X utf8 (Join-Path $PSScriptRoot 'qualify-current-item-skill-protocol.py') $out --normal-qualification $normalPath
 if($LASTEXITCODE -ne 0){throw 'Independent protocol qualification failed'}
 @{status='qualified';index_sha256=(Get-FileHash -LiteralPath (Join-Path $out 'index.json')).Hash;qualification_sha256=(Get-FileHash -LiteralPath (Join-Path $out 'qualified.json')).Hash}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $out 'completed.json') -Encoding utf8
 Write-Output $out
}catch{$index.status='failed';$index.error=$_.Exception.Message;Save;throw}
