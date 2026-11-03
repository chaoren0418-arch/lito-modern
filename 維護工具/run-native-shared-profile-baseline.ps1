#Requires -Version 7.0
[CmdletBinding()]param([Parameter(Mandatory)][string]$NormalQualification,[ValidateSet('Debug','Release')][string]$BuildProfile='Release')
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
$normalPath=if([IO.Path]::IsPathFullyQualified($NormalQualification)){[IO.Path]::GetFullPath($NormalQualification)}else{[IO.Path]::GetFullPath((Join-Path $taskRoot $NormalQualification))}
if(!$normalPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned normal qualification required'}
$normal=Get-Content -LiteralPath $normalPath -Raw|ConvertFrom-Json
$sourceMap=Join-Path (Split-Path -Parent $normalPath) 'source-inputs.json'
$inputs=Get-Content -LiteralPath $sourceMap -Raw|ConvertFrom-Json -AsHashtable
if($normal.features.Count -ne 0 -or $normal.source_files -ne 968 -or $inputs.Count -ne 968 -or (Get-FileHash -LiteralPath $sourceMap).Hash -ne $normal.source_manifest_sha256){throw 'Reviewed complete input map required'}
function Assert-NoHeavyJob {
 $busy=@(Get-CimInstance Win32_Process|Where-Object {
  $_.Name -match '^(cargo|rustc|lito_modern-.*)\.exe$' -or
  ($_.Name -match '^(native_.*|rebuild_.*|LiToModern.*)\.exe$' -and $_.CommandLine -and $_.CommandLine.Replace('/','\').Contains($project))
 })
 if($busy.Count){throw 'Wait for existing compilation/tests/GUI to terminate before this baseline'}
}
function Assert-Stable {
 foreach($name in $inputs.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $inputs[$name]){throw ('Compilation input drift: '+$name)}}
 foreach($profile in @('Debug','Release')){
  $main=$normal.profiles.$profile
  if(!$main.unique_entry -or $main.features.Count -ne 0 -or $main.machine -ne 'AMD64' -or $main.optional_header -ne 'PE32+' -or (Get-FileHash -LiteralPath $main.path).Hash -ne $main.sha256){throw 'Normal main identity drift'}
 }
}
Assert-NoHeavyJob;Assert-Stable
$catalogSource=Join-Path (Split-Path -Parent $taskRoot) 'lito登入器/LiTo381-Launcher/app/common_catalog.bin'
$catalogSHA='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
if((Get-FileHash -LiteralPath $catalogSource).Hash -ne $catalogSHA){throw 'Readonly common catalog source changed'}
$inputRoot=Join-Path $project 'target/current/shared-profile-inputs'
New-Item -ItemType Directory -Path $inputRoot -Force|Out-Null
$catalog=Join-Path $inputRoot 'common_catalog-B64ED920.bin'
if(!(Test-Path -LiteralPath $catalog)){Copy-Item -LiteralPath $catalogSource -Destination $catalog}
if(((Get-Item -LiteralPath $catalog).Attributes -band [IO.FileAttributes]::ReparsePoint) -or (Get-FileHash -LiteralPath $catalog).Hash -ne $catalogSHA){throw 'Owned common catalog copy mismatch'}
$out=Join-Path $taskRoot ('驗證紀錄/current/native-shared-profile-baseline/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-'+$BuildProfile)
New-Item -ItemType Directory -Path $out|Out-Null
Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $out 'runner.ps1')
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'verify-native-real-peers.ps1') -Destination (Join-Path $out 'peer-wrapper.ps1')
Copy-Item -LiteralPath $normalPath -Destination (Join-Path $out 'normal-qualification.json')
Copy-Item -LiteralPath $sourceMap -Destination (Join-Path $out 'source-inputs.json')
$state=@{status='running_shared_profile_baseline';profile=$BuildProfile;action='heal';resource_profile='tw13081901-v1';extended_item_details=$true;normal_qualification=$normalPath;normal_qualification_sha256=(Get-FileHash -LiteralPath $normalPath).Hash;source_manifest_sha256=$normal.source_manifest_sha256;main_sha256=$normal.profiles.$BuildProfile.sha256;common_catalog_sha256=$catalogSHA;source_catalog_read_only=$catalogSource;owned_catalog=$catalog;no_heavy_job_before=$true;started_utc=[DateTime]::UtcNow.ToString('o');gameplay_or_GUI_qualified=$false;whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $out 'observation.json') -Encoding utf8}
Save
$oldTemp=$env:TEMP;$oldTmp=$env:TMP
$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP;$env:PYTHONDONTWRITEBYTECODE='1'
$resultCode=1
try {
 $log=Join-Path $out 'peer-console.txt'
 & pwsh -NoProfile -File (Join-Path $PSScriptRoot 'verify-native-real-peers.ps1') -SkipBuild -BuildProfile $BuildProfile -ActionProfile heal -ResourceProfile tw13081901-v1 -CommonCatalogPath $catalog -ExtendedItemDetails *> $log
 $resultCode=$LASTEXITCODE;$state.peer_exit_code=$resultCode
 Assert-Stable;Assert-NoHeavyJob
 $lines=@(Get-Content -LiteralPath $log|Where-Object {$_.StartsWith('PEER_CASE_OUTPUT=')})
 if($lines.Count -ne 1){throw 'Expected exactly one owned peer case path; inspect baseline log'}
 $case=[IO.Path]::GetFullPath($lines[0].Substring('PEER_CASE_OUTPUT='.Length))
 if(!$case.StartsWith((Join-Path $taskRoot '驗證紀錄/current/native-real-peers')+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Unexpected peer case output'}
 $state.case=$case;$state.no_heavy_job_after=$true;$state.sources_unchanged=$true
 $state.status='shared_profile_runtime_observed_pending_failure_or_success_review';$state.finished_utc=[DateTime]::UtcNow.ToString('o');Save
 Write-Output ('SHARED_BASELINE_OBSERVATION='+$out)
}catch{$state.status='baseline_observation_infrastructure_failed';$state.error=$_.Exception.Message;Save;throw}finally{$env:TEMP=$oldTemp;$env:TMP=$oldTmp}
exit $resultCode
