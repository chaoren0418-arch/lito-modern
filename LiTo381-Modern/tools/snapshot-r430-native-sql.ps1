param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$h=Get-Content -LiteralPath 'D:\lineage開發\3-lito開發\lito伺服器\Native-canonical-真實SQL唯讀交接.json' -Raw|ConvertFrom-Json
$expected='6590D75206C1AC3F2C7C318CA312E50DEB44A5B421A11B4BD530841979EAEACF'
$replayExpected='4962A0E19FAC4CCD2C1F3DE7F6A6F19396CA78A293A08B0C9249C2EB22BA1528'
if($h.manifest_sha256 -ne $expected -or $h.file_count -ne 2857 -or $h.frozen_replay.manifest_sha256 -ne $replayExpected -or $h.frozen_replay.files -ne 10){throw 'Fixed SQL handoff mismatch'}
function CopyFixed([string]$producer,[string]$hash,[int]$count,[string]$kind){
 $m=Join-Path $producer 'manifest.json'
 if((Get-FileHash -LiteralPath $m).Hash -ne $hash){throw 'Manifest drift'}
 $manifest=Get-Content -LiteralPath $m -Raw|ConvertFrom-Json
 if($manifest.schema -ne 1 -or $manifest.files.Count -ne $count){throw 'Manifest shape'}
 $root=Join-Path $project ('target\r430-'+$kind+'-'+[Guid]::NewGuid().ToString('N'))
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
 [ordered]@{root=$root;files_verified=$count;manifest_sha256=$hash;vectors=(Join-Path $root $(if($kind -eq 'native-sql'){'evidence\canonical-tls-vectors.json'}else{'canonical-tls-vectors.json'}));vectors_sha256=(Get-FileHash -LiteralPath (Join-Path $root $(if($kind -eq 'native-sql'){'evidence\canonical-tls-vectors.json'}else{'canonical-tls-vectors.json'}))).Hash}
}
$candidate=CopyFixed $h.root $expected 2857 'native-sql'
$replay=CopyFixed $h.frozen_replay.root $replayExpected 10 'sql-replay'
$input=Join-Path $candidate.root 'consumer-input.json'
Write-ModernEvidenceJson -Path $input -Value ([ordered]@{state='FROZEN_SQL_PUBLICATION_BYTES_ONLY';candidate=$candidate;replay=$replay;producer_SQL_TLS=$true;consumer_live_SQL_TLS=$false;LOADED=$false;GUI=0})
Write-ModernEvidenceJson -Path (Join-Path $project 'logs\continuation-20261003\r430-sql-snapshot.json') -Value ([ordered]@{input=$input;input_sha256=(Get-FileHash -LiteralPath $input).Hash;candidate=$candidate;replay=$replay})
Write-Output 'R430 SQL candidate2857/replay10 immutable bytes copied; no class execution'
