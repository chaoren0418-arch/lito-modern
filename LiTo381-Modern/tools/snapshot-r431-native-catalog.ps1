param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$h=Get-Content -LiteralPath 'D:\lineage開發\3-lito開發\lito伺服器\Native-catalog-canonical唯讀交接.json' -Raw|ConvertFrom-Json
if($h.manifest_sha256 -ne 'D3DB8A157D1E49ED5F4E8F9A3FF2A4B7B33CEE2C33B02684F40293B0D955662D' -or $h.file_count -ne 3049 -or $h.frozen_replay.manifest_sha256 -ne '53A0E37BC8D2FC5121C7D809F40F3203A8A08788531385B0971FE530CED57CF5' -or $h.frozen_replay.file_count -ne 8){throw 'Fixed catalog handoff mismatch'}
function CopyFixed([string]$producer,[string]$hash,[int]$count,[string]$kind){
 $m=Join-Path $producer 'manifest.json'
 if((Get-FileHash -LiteralPath $m).Hash -ne $hash){throw 'Manifest drift'}
 $manifest=Get-Content -LiteralPath $m -Raw|ConvertFrom-Json
 if($manifest.schema -ne 1 -or $manifest.files.Count -ne $count){throw 'Manifest shape'}
 $root=Join-Path $project ('target\r431-'+$kind+'-'+[Guid]::NewGuid().ToString('N'))
 $null=New-Item -ItemType Directory -Path $root
 foreach($p in $manifest.files){
  if([IO.Path]::IsPathRooted($p.path) -or $p.path.Replace('\','/').Split('/') -contains '..'){throw 'Manifest path confinement'}
  $src=[IO.Path]::GetFullPath((Join-Path $producer $p.path))
  if(!$src.StartsWith($producer+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Source confinement'}
  if((Get-Item -LiteralPath $src).Length -ne $p.bytes -or (Get-FileHash -LiteralPath $src).Hash -ne $p.sha256){throw ('Source drift '+$p.path)}
  $dst=Join-Path $root $p.path
  $null=New-Item -ItemType Directory -Path (Split-Path -Parent $dst) -Force
  [IO.File]::Copy($src,$dst,$false)
  if((Get-Item -LiteralPath $dst).Length -ne $p.bytes -or (Get-FileHash -LiteralPath $dst).Hash -ne $p.sha256 -or (Get-FileHash -LiteralPath $src).Hash -ne $p.sha256){throw ('Copy drift '+$p.path)}
 }
 [IO.File]::Copy($m,(Join-Path $root 'manifest.json'),$false)
 if((Get-FileHash -LiteralPath (Join-Path $root 'manifest.json')).Hash -ne $hash){throw 'Manifest copy drift'}
 $vectors=@(foreach($name in @('catalog-vectors.json','catalog-negative-vectors.json','catalog-tls-vectors.json')){$relative=if($kind -eq 'catalog-candidate'){'evidence\'+$name}else{$name};$p=Join-Path $root $relative;[ordered]@{name=$name;path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash}})
 [ordered]@{root=$root;files_verified=$count;manifest_sha256=$hash;vectors=$vectors}
}
$candidate=CopyFixed $h.root $h.manifest_sha256 3049 'catalog-candidate'
$replay=CopyFixed $h.frozen_replay.root $h.frozen_replay.manifest_sha256 8 'catalog-replay'
$input=Join-Path $candidate.root 'consumer-input.json'
Write-ModernEvidenceJson -Path $input -Value ([ordered]@{state='FROZEN_CATALOG_90_91_BYTES_ONLY';candidate=$candidate;replay=$replay;producer_TLS=$true;producer_SQL=$false;consumer_live_TLS_SQL=$false;LOADED=$false;GUI=0})
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\r431-catalog-snapshot.json') -Value ([ordered]@{input=$input;input_sha256=(Get-FileHash -LiteralPath $input).Hash;candidate=$candidate;replay=$replay})
Write-Output 'R431 catalog candidate3049/replay8 immutable bytes copied; no class execution'
