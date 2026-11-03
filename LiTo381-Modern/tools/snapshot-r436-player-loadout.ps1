param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$handoff='D:\lineage開發\3-lito開發\lito伺服器\Native-player-loadout唯讀交接.json'
$h=Get-Content -LiteralPath $handoff -Raw|ConvertFrom-Json
if($h.manifest_sha256 -ne 'B900C9A9D85B4BEF5665D7D81D6CAA5D7CA19986A4A40F129C87EE329DC41225' -or $h.file_count -ne 3452 -or $h.frozen_replay.manifest_sha256 -ne 'B7E0BA729F465A24AD7DED3235E69FF1508C535ED45C1C807013A8331B50B73F' -or $h.frozen_replay.file_count -ne 31){throw 'Handoff changed'}
function CopyFixed($producer,$hash,$count,$kind){
 $mp=Join-Path $producer 'manifest.json';if((Get-FileHash $mp).Hash -ne $hash){throw 'Manifest drift'}
 $manifest=Get-Content $mp -Raw|ConvertFrom-Json;if($manifest.schema -ne 1 -or $manifest.files.Count -ne $count){throw 'Manifest shape'}
 $root=Join-Path $project ('target\r436-loadout-'+$kind+'-'+[Guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($root)|Out-Null
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
$candidate=CopyFixed $h.root $h.manifest_sha256 3452 'candidate';$replay=CopyFixed $h.frozen_replay.root $h.frozen_replay.manifest_sha256 31 'replay'
$hp=Join-Path $candidate.root 'consumer-handoff.json';[IO.File]::Copy($handoff,$hp,$false)
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\r436-loadout-snapshot.json') -Value ([ordered]@{state='FIXED_OFFLINE_LOADOUT_BYTES_COPIED';candidate=$candidate;replay=$replay;handoff=$hp;handoff_sha256=(Get-FileHash $hp).Hash;producer_SQL=$true;producer_TLS=$false;Rust_live_SQL_TLS=$false;game_wire=$false;LOADED=$false;GUI=0})
Write-Output 'R436 candidate3452/replay31 fixed source-copy-source pins verified'
