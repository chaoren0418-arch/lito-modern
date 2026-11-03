#Requires -Version 7.0
[CmdletBinding()]param([Parameter(Mandatory)][string]$NormalQualification)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
$normalPath=if([IO.Path]::IsPathFullyQualified($NormalQualification)){[IO.Path]::GetFullPath($NormalQualification)}else{[IO.Path]::GetFullPath((Join-Path $taskRoot $NormalQualification))}
if(!$normalPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Normal qualification outside rebuild root'}
$normal=Get-Content -LiteralPath $normalPath -Raw|ConvertFrom-Json
$map=Join-Path (Split-Path -Parent $normalPath) 'source-inputs.json'
$inputs=Get-Content -LiteralPath $map -Raw|ConvertFrom-Json -AsHashtable
if($normal.features.Count -ne 0 -or $inputs.Count -ne 968 -or (Get-FileHash -LiteralPath $map).Hash -ne $normal.source_manifest_sha256){throw 'Reviewed normal input map required'}
function Assert-NoCompilation {
 $busy=@(Get-CimInstance Win32_Process|Where-Object {
  $_.Name -match '^(cargo|rustc|lito_modern-.*)\.exe$' -or
  ($_.Name -match '^(native_.*|rebuild_.*|LiToModern.*)\.exe$' -and $_.CommandLine -and $_.CommandLine.Replace('/','\').Contains($project))
 })
 if($busy.Count){throw 'Compilation or library tests remain live; peer cases must run without this load'}
}
function Assert-Stable {
 foreach($name in $inputs.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $inputs[$name]){throw ('Compilation input drift: '+$name)}}
 foreach($profile in @('Debug','Release')){
  $main=$normal.profiles.$profile
  if(!$main.unique_entry -or $main.features.Count -ne 0 -or $main.machine -ne 'AMD64' -or $main.optional_header -ne 'PE32+' -or (Get-FileHash -LiteralPath $main.path).Hash -ne $main.sha256){throw 'Normal main drift or unsupported image'}
 }
 if((Get-FileHash -LiteralPath $normalPath).Hash -ne $normalHash){throw 'Normal qualification drift'}
}
$normalHash=(Get-FileHash -LiteralPath $normalPath).Hash
Assert-NoCompilation;Assert-Stable
$out=Join-Path $taskRoot ('驗證紀錄/current/native-peer-current-cohorts/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out|Out-Null
$index=@{status='running';normal_qualification=$normalPath;normal_qualification_sha256=$normalHash;source_manifest_sha256=$normal.source_manifest_sha256;steps=@();no_compilation_before_and_after_each_case=$true;whole_goal_complete=$false}
function Save {$index|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $out 'index.json') -Encoding utf8}
$oldTemp=$env:TEMP;$oldTmp=$env:TMP
$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP;$env:PYTHONDONTWRITEBYTECODE='1'
Save
try {
 foreach($profile in @('Debug','Release')){foreach($action in @('melee','movement')){
  Assert-NoCompilation;Assert-Stable
  $log=Join-Path $out ($profile+'-'+$action+'.log');$started=[DateTime]::UtcNow.ToString('o')
  & pwsh -NoProfile -File (Join-Path $PSScriptRoot 'verify-native-real-peers.ps1') -SkipBuild -BuildProfile $profile -ActionProfile $action *> $log
  if($LASTEXITCODE -ne 0){throw ('Peer case failed: '+$profile+'/'+$action+'; '+$log)}
  Assert-NoCompilation;Assert-Stable
  $line=Get-Content -LiteralPath $log|Where-Object {$_.StartsWith('Verified real-peer rounds: ')}|Select-Object -Last 1
  if(!$line){throw 'Missing immutable peer case output'}
  $case=[IO.Path]::GetFullPath($line.Substring('Verified real-peer rounds: '.Length))
  if(!$case.StartsWith((Join-Path $taskRoot '驗證紀錄/current/native-real-peers')+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Peer case outside expected owned output'}
  $receipt=Join-Path $case 'receipt.json';$receiptHash=(Get-FileHash -LiteralPath $receipt).Hash
  $binding=Get-Content -LiteralPath (Join-Path $case 'validation-completed.json') -Raw|ConvertFrom-Json
  if($binding.receipt_sha256 -ne $receiptHash -or $binding.runtime_receipt_rewritten){throw 'Peer validation completion binding mismatch'}
  $index.steps+=@{profile=$profile;action=$action;case=$case;receipt_sha256=$receiptHash;started_utc=$started;finished_utc=[DateTime]::UtcNow.ToString('o');no_compile_before=$true;no_compile_after=$true};Save
 }}
 $index.status='runtime_completed_pending_independent_qualification';Save
 & python -X utf8 (Join-Path $PSScriptRoot 'qualify-current-peer-cohort.py') $out
 if($LASTEXITCODE -ne 0){throw 'Independent peer cohort qualification failed'}
 @{status='qualified';index_sha256=(Get-FileHash -LiteralPath (Join-Path $out 'index.json')).Hash;qualification_sha256=(Get-FileHash -LiteralPath (Join-Path $out 'qualified.json')).Hash}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $out 'completed.json') -Encoding utf8
 Write-Output $out
}catch{$index.status='failed';$index.error=$_.Exception.Message;Save;throw}finally{$env:TEMP=$oldTemp;$env:TMP=$oldTmp}
