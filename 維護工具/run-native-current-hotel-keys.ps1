#Requires -Version 7.0
param([Parameter(Mandatory)][string]$NormalQualification)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$normalPath=(Resolve-Path -LiteralPath $NormalQualification).Path
if(!$normalPath.StartsWith((Join-Path $taskRoot '驗證紀錄/current/'),[StringComparison]::OrdinalIgnoreCase)){throw 'Owned normal qualification required'}
$normal=Get-Content -LiteralPath $normalPath -Raw|ConvertFrom-Json
if($normal.status -ne 'independently_qualified_green_clock_related_scope_and_unique_entry_normal_builds' -or $normal.source_files -ne 1021 -or $normal.features.Count -ne 0){throw 'Exact source1021 normal mains required'}
$pins=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $normalPath) 'source-inputs.json') -Raw|ConvertFrom-Json -AsHashtable
$cohort=Join-Path $taskRoot ('驗證紀錄/current/native-hotel-current/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-keys')
New-Item -ItemType Directory -Path $cohort|Out-Null
$env:PYTHONDONTWRITEBYTECODE='1';$env:PYTHONIOENCODING='utf-8'
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$identity=Get-CimInstance Win32_Process -Filter ('ProcessId='+$PID)
$validationNames=@('run-native-current-hotel-keys.ps1','verify-native-real-peers.ps1','verify-real-peer-rental-entry.py','native_resident_map_reference.py','verify-real-peer-rental-hotel.py','verify-real-peer-attributes.py','owned-peer-validation-binding.ps1')
$validationPins=[ordered]@{}
foreach($name in $validationNames){$validationPins[$name]=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name)).Hash;Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination (Join-Path $cohort $name)}
$state=[ordered]@{status='running_source1021_hotel_keys_and_return';cohort=$cohort;pipeline_pid=$PID;pipeline_creation_utc=$identity.CreationDate.ToUniversalTime().ToString('o');normal_qualification=$normalPath;normal_qualification_sha256=(Get-FileHash -LiteralPath $normalPath).Hash;source_manifest_sha256=$normal.source_manifest_sha256;source_files=1021;resource_profile='tw13081901-v1';extended_item_details=$true;validation_pins=$validationPins;profiles=[ordered]@{};active_profile=$null;source_and_corresponding_QA_frozen=$true;GUI_verified=$false;whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $cohort 'receipt.json') -Encoding utf8}
function CheckInputs {
 foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw ('Source drift: '+$name)}}
 foreach($name in $validationNames){if((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name)).Hash -ne $validationPins[$name]){throw ('Validation source drift: '+$name)}}
 foreach($profile in @('Debug','Release')){if((Get-FileHash -LiteralPath $normal.profiles.$profile.path).Hash -ne $normal.profiles.$profile.sha256){throw 'Normal EXE drift'}}
}
Save;Write-Output ('HOTEL_COHORT='+$cohort)
try {
 foreach($profile in @('Debug','Release')){
  CheckInputs;$state.active_profile=$profile;Save
  $log=Join-Path $cohort ($profile+'-console.log')
  & pwsh.exe -NoProfile -File (Join-Path $PSScriptRoot 'verify-native-real-peers.ps1') -SkipBuild -BuildProfile $profile -ReferenceJar (Join-Path $taskRoot 'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar') -ResourceProfile tw13081901-v1 -CommonCatalogPath (Join-Path $taskRoot 'LiTo381-Modern/target/current/shared-profile-inputs/common_catalog-B64ED920.bin') -ExtendedItemDetails -ActionProfile rental-hotel *> $log
  $code=$LASTEXITCODE
  $lines=@(Get-Content -LiteralPath $log|Where-Object {$_ -match '^PEER_CASE_OUTPUT='})
  if($lines.Count -ne 1){throw 'Cannot bind exact hotel case'}
  $case=$lines[0].Substring('PEER_CASE_OUTPUT='.Length)
  $state.profiles[$profile]=[ordered]@{case=$case;exit_code=$code;main_sha256=$normal.profiles.$profile.sha256};Save
  if($code -ne 0){throw ('Hotel runtime or independent comparison failed: '+$profile)}
  $r=Get-Content -LiteralPath (Join-Path $case 'receipt.json') -Raw|ConvertFrom-Json
  if($r.action_profile -ne 'rental-hotel' -or $r.exe_sha256 -ne $normal.profiles.$profile.sha256 -or $r.jar_sha256 -ne 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539' -or $r.server.actual_client_sessions -ne 4 -or $r.server.authority_mutations -ne 0 -or !$r.server.memberships_empty -or $r.server.active_hotel_leases_after_disconnect -ne 0){throw 'Hotel case identity or cleanup mismatch'}
  CheckInputs
  Write-Output ($profile+' hotel keys/return runtime and independent comparisons passed')
 }
 $state.status='executed_source1021_both_profiles_hotel_keys_and_return';$state.active_profile=$null;$state.source_and_corresponding_QA_frozen=$false;Save
}catch{$state.status='failed_preserved';$state['error']=$_.Exception.Message;Save;throw}
Write-Output ($state|ConvertTo-Json -Depth 12)
