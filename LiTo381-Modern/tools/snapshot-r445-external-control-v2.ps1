param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$handoff='D:\lineage開發\3-lito開發\lito伺服器\Native-external-control-v2唯讀交接.json'
$h=Get-Content -LiteralPath $handoff -Raw|ConvertFrom-Json
if($h.manifest_sha256 -ne 'C88CBCC80A512A2F17766B72872D388EFC71D66C1CDBC3A46E5CDA69C7B1C1BE' -or $h.file_count -ne 3665){throw 'Handoff changed'}
if($h.schema -ne 2 -or $h.fixed_evidence.manifest_sha256 -ne '9B0F07B146337437EC8F146BE152FAD9BDF695AC0B2132763EE762E6B08F9E50' -or $h.fixed_evidence.file_count -ne 17){throw 'V2 evidence changed'}
function CopyFixed($producer,$hash,$count,$kind){
 $mp=Join-Path $producer 'manifest.json';if((Get-FileHash $mp).Hash -ne $hash){throw 'Manifest drift'}
 $manifest=Get-Content $mp -Raw|ConvertFrom-Json;if($manifest.schema -ne 1 -or $manifest.files.Count -ne $count){throw 'Manifest shape'}
 $root=Join-Path $project ('target\r445-external-control-v2-'+$kind+'-'+[Guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($root)|Out-Null
 foreach($p in $manifest.files){
  if([IO.Path]::IsPathRooted($p.path) -or $p.path.Replace('\','/').Split('/') -contains '..'){throw 'Path escape'}
  $src=[IO.Path]::GetFullPath((Join-Path $producer $p.path));if(!$src.StartsWith($producer+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Source confinement'}
  if((Get-Item $src).Length -ne $p.bytes -or (Get-FileHash $src).Hash -ne $p.sha256){throw 'Source drift'}
  $dst=Join-Path $root $p.path;[IO.Directory]::CreateDirectory((Split-Path $dst -Parent))|Out-Null;[IO.File]::Copy($src,$dst,$false)
  if((Get-Item $dst).Length -ne $p.bytes -or (Get-FileHash $dst).Hash -ne $p.sha256 -or (Get-FileHash $src).Hash -ne $p.sha256){throw 'Copy drift'}
 }
 [IO.File]::Copy($mp,(Join-Path $root 'manifest.json'),$false)
 if((Get-FileHash (Join-Path $root 'manifest.json')).Hash -ne $hash){throw 'Manifest copy'}
 [ordered]@{root=$root;files_verified=$count;manifest_sha256=$hash}
}
$candidate=CopyFixed $h.root $h.manifest_sha256 3665 'candidate'
$evidence=CopyFixed $h.fixed_evidence.root $h.fixed_evidence.manifest_sha256 17 'evidence'
$hp=Join-Path $project 'logs\continuation-20261003\r445-producer-handoff.json';[IO.File]::Copy($handoff,$hp,$false)
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\r445-external-control-v2-snapshot.json') -Value ([ordered]@{state='FIXED_EXTERNAL_FIXTURE_BYTES_COPIED_PENDING_READY_BINDING';candidate=$candidate;evidence=$evidence;handoff=$hp;handoff_sha256=(Get-FileHash $hp).Hash;producer_SQL=$true;producer_TLS=$true;Rust_live_SQL_TLS=$false;game_wire=$false;LOADED=$false;GUI=0})
Write-Output 'R441b candidate3665 fixed source-copy-source pins verified'
