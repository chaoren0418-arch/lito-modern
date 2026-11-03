param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$handoffPath='D:\lineage開發\3-lito開發\lito伺服器\Native-canonical-DATA-TLS唯讀交接.json'
$h=Get-Content -LiteralPath $handoffPath -Raw|ConvertFrom-Json
$producer='D:\lito-dev-runtime\20261003-supervision\server\native-canonical-service-frozen-a85f1e06a0d645c58f251f48e7dd99ab'
$hash='DDB28022F48B267A3F34E36E6C326784F4C1D5F3A7FDB3C7C8C318946E4B2F77'
if($h.root -ne $producer -or $h.manifest_sha256 -ne $hash -or $h.file_count -ne 2801){throw 'Fixed service handoff mismatch'}
$manifestPath=Join-Path $producer 'manifest.json'
if((Get-FileHash -LiteralPath $manifestPath).Hash -ne $hash){throw 'Service manifest drift'}
$manifest=Get-Content -LiteralPath $manifestPath -Raw|ConvertFrom-Json
if($manifest.schema -ne 1 -or $manifest.files.Count -ne 2801){throw 'Service manifest count/schema'}
$root=Join-Path $project ('target\r429-native-service-frozen-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $root
foreach($p in $manifest.files){
  if([IO.Path]::IsPathRooted($p.path) -or $p.path.Replace('\','/').Split('/') -contains '..'){throw 'Manifest path confinement'}
  $src=[IO.Path]::GetFullPath((Join-Path $producer $p.path))
  if(!$src.StartsWith($producer+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Source confinement'}
  if((Get-Item -LiteralPath $src).Length -ne $p.bytes -or (Get-FileHash -LiteralPath $src).Hash -ne $p.sha256){throw ('Source drift: '+$p.path)}
  $dst=Join-Path $root $p.path
  $null=New-Item -ItemType Directory -Path (Split-Path -Parent $dst) -Force
  [IO.File]::Copy($src,$dst,$false)
  if((Get-Item -LiteralPath $dst).Length -ne $p.bytes -or (Get-FileHash -LiteralPath $dst).Hash -ne $p.sha256 -or (Get-FileHash -LiteralPath $src).Hash -ne $p.sha256){throw ('Copy drift: '+$p.path)}
}
[IO.File]::Copy($manifestPath,(Join-Path $root 'manifest.json'),$false)
if((Get-FileHash -LiteralPath (Join-Path $root 'manifest.json')).Hash -ne $hash){throw 'Manifest copy drift'}
$input=Join-Path $root 'consumer-input.json'
Write-ModernEvidenceJson -Path $input -Value ([ordered]@{state='FROZEN_NATIVE_SERVICE_BYTES_ONLY';root=$root;manifest_sha256=$hash;files_verified=2801;DATA_sha256=$h.DATA_vectors.sha256;TLS_sha256=$h.TLS_vectors.sha256;SQL=$false;Rust_TLS=$false;renderer_LOADED=$false;GUI=0})
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\r429-service-snapshot.json') -Value ([ordered]@{input=$input;input_sha256=(Get-FileHash -LiteralPath $input).Hash;root=$root;files_verified=2801;manifest_sha256=$hash})
Write-Output ([ordered]@{root=$root;files_verified=2801}|ConvertTo-Json)
