$ErrorActionPreference='Stop'
$project=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$handoffPath=Join-Path (Split-Path (Split-Path $project)) 'lito伺服器\Native-LTmap-candidate交接.json'
$handoff=Get-Content -LiteralPath $handoffPath -Raw|ConvertFrom-Json
$expected='2D343EE5415E335B011ED0DB65BDF98D1B186D6FAF7C7F1095CDC2AECF11B45F'
if($handoff.manifest_sha256 -ne $expected -or (Get-FileHash -LiteralPath $handoff.manifest).Hash -ne $expected){throw 'Native candidate manifest mismatch'}
$manifest=Get-Content -LiteralPath $handoff.manifest -Raw|ConvertFrom-Json
if(-not $manifest.immutable -or $manifest.files.Count -ne 38 -or $manifest.namespace -ne 'NATIVE_LITO_LTMAP_V1'){throw 'Unexpected Native candidate scope'}
foreach($dependency in $manifest.dependencies){if((Get-FileHash -LiteralPath $dependency.path).Hash -ne $dependency.sha256){throw 'Native dependency chain drift'}}
$destination=Join-Path $project 'target\r425-native-dm564-fixture'
if(Test-Path -LiteralPath $destination){throw 'Native consumer snapshot already exists'}
$null=New-Item -ItemType Directory -Path $destination
$checks=[Collections.Generic.List[object]]::new()
function Copy-Pinned([string]$Source,[string]$Relative,[long]$Bytes,[string]$Hash){
    $target=[IO.Path]::GetFullPath((Join-Path $destination $Relative))
    if(-not $target.StartsWith($destination+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Snapshot path escapes own destination'}
    if((Get-Item -LiteralPath $Source).Length -ne $Bytes -or (Get-FileHash -LiteralPath $Source).Hash -ne $Hash){throw ('Native source drift: '+$Source)}
    $null=New-Item -ItemType Directory -Path (Split-Path $target) -Force
    Copy-Item -LiteralPath $Source -Destination $target
    if((Get-Item -LiteralPath $target).Length -ne $Bytes -or (Get-FileHash -LiteralPath $target).Hash -ne $Hash){throw 'Native copied bytes drift'}
    $checks.Add([ordered]@{source=$Source;snapshot=$target;bytes=$Bytes;sha256=$Hash})
    return $target
}
foreach($entry in $manifest.files){
    $source=[IO.Path]::GetFullPath((Join-Path $handoff.immutable_root $entry.relative_path))
    if(-not $source.StartsWith($handoff.immutable_root+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Producer source path escapes immutable root'}
    $null=Copy-Pinned $source ('producer\'+$entry.relative_path) $entry.bytes $entry.sha256
}
$archives=@(foreach($pin in $handoff.native_archive_pins){
    if($pin.namespace -notin @('NATIVE_LITO_LTTILE_V1','NATIVE_LITO_LTSPRITE_V1')){throw 'Unknown Native archive namespace'}
    $path=Copy-Pinned $pin.path ('native-archives\'+(Split-Path $pin.path -Leaf)) $pin.bytes $pin.sha256
    [ordered]@{namespace=$pin.namespace;path=$path;bytes=$pin.bytes;sha256=$pin.sha256}
})
$config=Get-Content -LiteralPath $handoff.config|ConvertFrom-StringData
if($config.'source.namespace' -ne 'NATIVE_LITO_LTMAP_V1' -or $config.'map.id' -ne '564' -or $config.sectors -ne '7fff8000'){throw 'Require the explicit selected Native candidate'}
$mapSource=Join-Path $config.'source.root' '564\7fff8000.s32'
$mapPath=Copy-Pinned $mapSource 'native-map\564\7fff8000.s32' ([long]$config.'sector.7fff8000.bytes') $config.'sector.7fff8000.sha256'
$geometry=Get-Content -LiteralPath (Join-Path $destination 'producer\rust-format-geometry.json') -Raw|ConvertFrom-Json
$row=@($geometry.maps|Where-Object{$_.map -eq 564})
if($row.Count -ne 1 -or $row[0].native_tile_audit.dependencies.Count -ne 32){throw 'Require exactly32Native tile dependencies'}
$input=[ordered]@{candidate_id=$handoff.candidate_id;native_map=[ordered]@{namespace='NATIVE_LITO_LTMAP_V1';logical_map=564;sector='7fff8000';path=$mapPath;bytes=[long]$config.'sector.7fff8000.bytes';sha256=$config.'sector.7fff8000.sha256';canonical_sha256=$config.'canonical.sha256';standable_cells=760;opacity_records=390;sprite_effects=5};archive_pins=$archives;tile_dependencies=$row[0].native_tile_audit.dependencies;empty_tail_variant=[ordered]@{namespace='NATIVE_LITO_LTTILE_V1';entry='1026.til';bytes=40295;sha256='AB2A6C4A3B490C058589DF36CA8DAC7A57AC4EDEFDD606E7D69290A213656A31';table_slots=256;first_empty_slot=106};repro=(Join-Path $destination 'producer\minimal-native-1026.til');scope='Read-only Native format/asset/CPU consumer, no wire/TLS/SQL/combat settlement/GUI and zero mode additions'}
$inputPath=Join-Path $destination 'consumer-input.json'
Write-ModernEvidenceJson -Path $inputPath -Value $input
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\native-dm564-fixture-snapshot.json') -Value ([ordered]@{verified_at=[DateTimeOffset]::Now.ToString('o');handoff=$handoffPath;handoff_sha256=(Get-FileHash -LiteralPath $handoffPath).Hash;producer_manifest=$handoff.manifest;producer_manifest_sha256=$expected;checks=$checks.ToArray();consumer_input=$inputPath;consumer_input_sha256=(Get-FileHash -LiteralPath $inputPath).Hash;scope=$input.scope;formal_player_replaced=$false;formal_server_jar_replaced=$false})
[pscustomobject]@{copied_pinned_files=$checks.Count;input=$inputPath}
