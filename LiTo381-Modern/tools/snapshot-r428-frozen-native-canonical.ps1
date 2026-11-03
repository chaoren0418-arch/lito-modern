param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$handoffPath='D:\lineage開發\3-lito開發\lito伺服器\Native-canonical-codec唯讀交接.json'
$handoff=Get-Content -LiteralPath $handoffPath -Raw|ConvertFrom-Json
$expectedManifest='B09515C8571B172AA47A06CCFE66C5E7D1E0C138E1A0242AFD179B26CD043468'
$expectedVectors='A40F157E411CA16DD71547C87D12F108D0E4D301601F7596DD2D928F94768B34'
$expectedSpec='D7BEAEBCDA130535E52D072C63D1D0145F453FC7524D02C07A29BD6434ECD805'
$producer='D:\lito-dev-runtime\20261003-supervision\server\native-canonical-codec-frozen-1cb8e16091c44edca4a754de5653da4f'
if($handoff.root -ne $producer -or $handoff.manifest_sha256 -ne $expectedManifest -or $handoff.vectors_sha256 -ne $expectedVectors -or $handoff.specification_sha256 -ne $expectedSpec){throw 'Frozen handoff does not match explicitly supplied pins'}
$manifestPath=Join-Path $producer 'manifest.json'
if((Get-FileHash -LiteralPath $manifestPath).Hash -ne $expectedManifest){throw 'Frozen manifest drift'}
$manifest=Get-Content -LiteralPath $manifestPath -Raw|ConvertFrom-Json
if($manifest.schema -ne 1 -or $manifest.files.Count -ne 272){throw 'Frozen manifest schema/count mismatch'}
$root=Join-Path $project ('target\r428-native-canonical-frozen-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $root
$rows=@(foreach($pin in $manifest.files){
    if([IO.Path]::IsPathRooted($pin.path) -or $pin.path.Replace('\','/').Split('/') -contains '..'){throw 'Unconfined manifest path'}
    $source=[IO.Path]::GetFullPath((Join-Path $producer $pin.path))
    if(!$source.StartsWith($producer+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Unconfined frozen source'}
    $dest=Join-Path $root $pin.path
    if((Get-Item -LiteralPath $source).Length -ne $pin.bytes -or (Get-FileHash -LiteralPath $source).Hash -ne $pin.sha256){throw ('Frozen producer pin drift: '+$pin.path)}
    $null=New-Item -ItemType Directory -Path (Split-Path -Parent $dest) -Force
    [IO.File]::Copy($source,$dest,$false)
    if((Get-Item -LiteralPath $dest).Length -ne $pin.bytes -or (Get-FileHash -LiteralPath $dest).Hash -ne $pin.sha256 -or (Get-FileHash -LiteralPath $source).Hash -ne $pin.sha256){throw ('Snapshot or producer drift: '+$pin.path)}
    [ordered]@{path=$pin.path;bytes=$pin.bytes;sha256=$pin.sha256}
})
[IO.File]::Copy($manifestPath,(Join-Path $root 'manifest.json'),$false)
if((Get-FileHash -LiteralPath (Join-Path $root 'manifest.json')).Hash -ne $expectedManifest){throw 'Copied manifest drift'}
$vectors=Join-Path $root 'canonical-codec-vectors.json'
$spec=Join-Path $root 'Native資源canonical整合契約.md'
if((Get-FileHash -LiteralPath $vectors).Hash -ne $expectedVectors -or (Get-FileHash -LiteralPath $spec).Hash -ne $expectedSpec){throw 'Required frozen inputs absent/drifted'}
$input=Join-Path $root 'consumer-input.json'
$value=[ordered]@{created_at=[DateTimeOffset]::Now.ToString('o');state='FROZEN_CODEC_PROTOTYPE_ONLY_NOT_FORMAL_WIRE';producer_root=$producer;root=$root;files=$rows;file_count=272;manifest_sha256=$expectedManifest;vectors=$vectors;vector_sha256=$expectedVectors;specification=$spec;specification_sha256=$expectedSpec;requests=6;pre_engine_results=8;producer_negative_assertions=55;actual_DATA_vector=$false;live_service_ledger=$false;TLS=$false;renderer_LOADED=$false;qualified_modes=16;GUI=0}
Write-ModernEvidenceJson -Path $input -Value $value
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\r428-frozen-prototype-snapshot.json') -Value ([ordered]@{input=$input;input_sha256=(Get-FileHash -LiteralPath $input).Hash;root=$root;files_verified=272;manifest_sha256=$expectedManifest;state=$value.state})
Write-Output ([ordered]@{input=$input;files_verified=272;manifest_sha256=$expectedManifest}|ConvertTo-Json)
