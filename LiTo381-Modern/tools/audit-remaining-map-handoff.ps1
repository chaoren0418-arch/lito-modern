$ErrorActionPreference='Stop'
$project=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$handoffPath=Join-Path (Split-Path (Split-Path $project)) 'lito伺服器\Modern-map-fixture交接.json'
$handoff=Get-Content -LiteralPath $handoffPath -Raw|ConvertFrom-Json
$refs=@($handoff.shared_component_evidence,$handoff.remaining24_evidence_r2)
foreach($ref in $refs){if((Get-FileHash -LiteralPath $ref.path).Hash -ne $ref.sha256){throw 'Remaining map evidence hash drift'}}
$components=Get-Content -LiteralPath $refs[0].path -Raw|ConvertFrom-Json
$remaining=Get-Content -LiteralPath $refs[1].path -Raw|ConvertFrom-Json
$consumerPath=Join-Path $logs 'remaining-map-qualification.json'
if((Get-FileHash -LiteralPath $consumerPath).Hash -ne $remaining.consumer_input.sha256){throw 'Consumer pending list drift'}
$consumer=Get-Content -LiteralPath $consumerPath -Raw|ConvertFrom-Json
$expected=@($consumer.pending_rows|ForEach-Object{"$($_.mode):$($_.logical_map)"}|Sort-Object)
$actual=@($remaining.rows|ForEach-Object{"$($_.mode):$($_.logical_map)"}|Sort-Object)
if($actual.Count -ne 24 -or (Compare-Object $expected $actual)){throw 'Must reconcile exact24 keys'}
if($remaining.new_qualified_modern_pending_bindings -ne 0 -or $remaining.identical_shared_native_sector_count -ne 0){throw 'Qualification scope changed; review new evidence'}
$checks=[Collections.Generic.List[object]]::new()
function Check-Bytes($Entry,[string]$Path,[string]$Namespace){
    $hash=(Get-FileHash -LiteralPath $Path).Hash
    if($hash -ne $Entry.sha256 -or (Get-Item -LiteralPath $Path).Length -ne $Entry.bytes){throw ('Map bytes drift: '+$Path)}
    $checks.Add([ordered]@{path=$Path;namespace=$Namespace;bytes=$Entry.bytes;sha256=$hash;matches=$true})
}
$rows=@(foreach($row in $remaining.rows){
    if($null -ne $row.modern_explicit_binding -or $row.identical_shared_native_files -ne 0){throw 'Unexpected pending binding or identical sector claim'}
    foreach($file in @($row.native_files)){Check-Bytes $file $file.path 'Native LiTo/LTmap'}
    foreach($file in @($row.shared_files)){Check-Bytes $file $file.path 'Shared Map'}
    [ordered]@{mode=$row.mode;logical_map=$row.logical_map;native_name=$row.native_catalog.native_name;native_category=$row.native_catalog.native_category;native_sector_count=$row.native_files.Count;shared_sector_count=$row.shared_files.Count;same_filename_count=$row.same_filename_count;identical_shared_native_files=$row.identical_shared_native_files;native_sql_rules=$row.pinned_native_sql_rules;native_sql_objects=$row.pinned_native_sql_objects;native_sql_npc_waves=$row.pinned_native_sql_npc_waves;lineage_same_number=$row.lineage_mapids_same_number;required=$row.required;binding=$null;program_accepted=$false;gui_accepted=$false}
})
foreach($component in $components.rows){
    if($component.mode_qualified){throw 'Geometry component is unexpectedly mode-qualified'}
    foreach($pin in $component.sector_pins){Check-Bytes $pin (Join-Path 'D:\lineage開發\0-客戶端' "Map\$($component.physical_map)\$($pin.sector).s32") 'Shared geometry component'}
}
Write-ModernEvidenceJson -Path (Join-Path $logs 'remaining-map-handoff-consumer.json') -Value ([ordered]@{verified_at=[DateTimeOffset]::Now.ToString('o');handoff=$handoffPath;references=$refs;pending_pairs=24;native_resources_present=24;shared_resources_present=9;geometry_components=$components.rows.Count;geometry_parse_scope='Producer geometry_parse passed; consumer independently verifies raw sector bytes/lengths. This audit does not claim independent geometry parsing or playable policy.';verified_byte_checks=$checks.Count;byte_checks=$checks.ToArray();new_qualified_bindings=0;rows=$rows;scope='Native LiTo/LTmap and Shared Map namespaces remain separate; matching numbers/names/filenames do not qualify source, spawn, encounter, race or objective binding. No map remap, SQL mutation, provider change or GUI acceptance.'})
[pscustomobject]@{pending_pairs=24;verified_byte_checks=$checks.Count;geometry_components=$components.rows.Count;new_qualified_bindings=0}
