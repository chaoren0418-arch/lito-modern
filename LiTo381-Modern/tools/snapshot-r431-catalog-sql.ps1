$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$h=Get-Content -LiteralPath 'D:\lineage開發\3-lito開發\lito伺服器\Native-catalog-真實SQL唯讀交接.json' -Raw|ConvertFrom-Json
if($h.manifest_sha256 -ne '9294928365A0C5A2269688BCBCDD1CF31E7772E979C4750A530B80F9D1EE1277' -or $h.file_count -ne 3086 -or $h.frozen_replay.manifest_sha256 -ne 'FF8D3E7F2D153FB1B65A3AAF6B87638B742062B70095F5C80ED3F7DEC38BC746' -or $h.frozen_replay.file_count -ne 14){throw 'Fixed SQL catalog handoff mismatch'}
function CopyFixed($producer,$sha,$count,$kind){
 $mp=Join-Path $producer 'manifest.json';if((Get-FileHash -LiteralPath $mp).Hash -ne $sha){throw 'Manifest drift'}
 $m=Get-Content -LiteralPath $mp -Raw|ConvertFrom-Json;if($m.files.Count -ne $count -or $m.schema -ne 1){throw 'Manifest shape'}
 $root=Join-Path $project ('target\r431-catalog-sql-'+$kind+'-'+[Guid]::NewGuid().ToString('N'))
 [IO.Directory]::CreateDirectory($root)|Out-Null
 foreach($p in $m.files){
  if([IO.Path]::IsPathRooted($p.path) -or $p.path.Replace('\','/').Split('/') -contains '..'){throw 'Manifest confinement'}
  $src=[IO.Path]::GetFullPath((Join-Path $producer $p.path));$dst=[IO.Path]::GetFullPath((Join-Path $root $p.path))
  if(-not $src.StartsWith($producer+'\',[StringComparison]::OrdinalIgnoreCase) -or -not $dst.StartsWith($root+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Path escapes root'}
  if((Get-Item -LiteralPath $src).Length -ne $p.bytes -or (Get-FileHash -LiteralPath $src).Hash -ne $p.sha256){throw 'Source pin drift'}
  [IO.Directory]::CreateDirectory((Split-Path -Parent $dst))|Out-Null;[IO.File]::Copy($src,$dst,$false)
  if((Get-Item -LiteralPath $dst).Length -ne $p.bytes -or (Get-FileHash -LiteralPath $dst).Hash -ne $p.sha256 -or (Get-FileHash -LiteralPath $src).Hash -ne $p.sha256){throw 'Copy pin drift'}
 }
 [IO.File]::Copy($mp,(Join-Path $root 'manifest.json'),$false)
 $vectors=@(foreach($name in @('catalog-sql-wire90-91.json','catalog-sql-tls-vectors.json')){$relative=if($kind -eq 'candidate'){'evidence\'+$name}else{$name};$p=Join-Path $root $relative;[ordered]@{name=$name;path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash}})
 [ordered]@{root=$root;manifest_sha256=$sha;files_verified=$count;vectors=$vectors}
}
$candidate=CopyFixed $h.root $h.manifest_sha256 3086 'candidate'
$replay=CopyFixed $h.frozen_replay.root $h.frozen_replay.manifest_sha256 14 'replay'
$input=Join-Path $candidate.root 'consumer-input.json'
Write-ModernEvidenceJson -Path $input -Value ([ordered]@{state='FIXED_SQL_CATALOG_BYTES_ONLY';candidate=$candidate;replay=$replay;producer_SQL_TLS=$true;Rust_live_SQL_TLS=$false;LOADED=$false})
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\r431-catalog-sql-snapshot.json') -Value ([ordered]@{input=$input;input_sha256=(Get-FileHash -LiteralPath $input).Hash;candidate=$candidate;replay=$replay})
Write-Output 'R431 SQL catalog3086/replay14 fixed bytes copied; no Java execution'
