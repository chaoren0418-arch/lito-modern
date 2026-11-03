param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$handoffPath='D:\lineage開發\3-lito開發\lito伺服器\Native-canonical-wire進度交接.json'
$replyPath='D:\lineage開發\3-lito開發\lito伺服器\Native-canonical-九項審查回覆.json'
$handoff=Get-Content -LiteralPath $handoffPath -Raw|ConvertFrom-Json
$expectedSpec='D7BEAEBCDA130535E52D072C63D1D0145F453FC7524D02C07A29BD6434ECD805'
$expectedVectors='A40F157E411CA16DD71547C87D12F108D0E4D301601F7596DD2D928F94768B34'
if($handoff.specification.sha256 -ne $expectedSpec -or $handoff.vectors.sha256 -ne $expectedVectors){throw 'Producer handoff changed; await new explicit pins before consuming'}
$extra=@($handoffPath,$replyPath)|ForEach-Object{[ordered]@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash;bytes=(Get-Item -LiteralPath $_).Length}}
$pins=@($extra)+@($handoff.specification,$handoff.vectors,$handoff.result_file)+@($handoff.sources)+@($handoff.test_source)
foreach($pin in $pins){
    if((Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256 -or (Get-Item -LiteralPath $pin.path).Length -ne $pin.bytes){throw ('Producer pin drift; not consuming mixed version: '+$pin.path)}
}
$root=Join-Path $project ('target\r428-native-canonical-prototype-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $root
$copies=@(foreach($pin in $pins){
    $destination=Join-Path $root (Split-Path -Leaf $pin.path)
    [IO.File]::Copy($pin.path,$destination,$false)
    if((Get-FileHash -LiteralPath $destination).Hash -ne $pin.sha256 -or (Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256){throw ('Producer drift during snapshot: '+$pin.path)}
    [ordered]@{producer=$pin.path;path=$destination;bytes=$pin.bytes;sha256=$pin.sha256}
})
$input=Join-Path $root 'consumer-input.json'
$value=[ordered]@{created_at=[DateTimeOffset]::Now.ToString('o');state='UNFROZEN_JAVA_CODEC_PROTOTYPE_NOT_FORMAL_WIRE';root=$root;files=$copies;vectors=(Join-Path $root 'canonical-codec-vectors.json');vector_sha256=$expectedVectors;specification=(Join-Path $root 'Native資源canonical整合契約.md');specification_sha256=$expectedSpec;requests=6;pre_engine_results=8;producer_negative_assertions=55;actual_DATA_vector=$false;live_service_ledger=$false;TLS=$false;qualified_modes=16;GUI=0}
Write-ModernEvidenceJson -Path $input -Value $value
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\r428-prototype-snapshot.json') -Value ([ordered]@{input=$input;input_sha256=(Get-FileHash -LiteralPath $input).Hash;files=$copies;state=$value.state})
Write-Output ([ordered]@{input=$input;files=$copies.Count;vectors_sha256=$expectedVectors;specification_sha256=$expectedSpec}|ConvertTo-Json)
