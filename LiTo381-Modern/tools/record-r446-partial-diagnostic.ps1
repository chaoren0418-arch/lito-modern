$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$mappingPath='D:\lineage開發\3-lito開發\lito伺服器\Native-live外觀權威逐欄映射與只讀診斷交接.json'
$candidatePath='D:\lineage開發\3-lito開發\lito伺服器\Native-live外觀診斷候選交接.json'
$root=Join-Path $project ('target\r446-partial-observation-readonly-'+[Guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($root)|Out-Null
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
$copies=@(foreach($p in @($mappingPath,$candidatePath)){$dst=Join-Path $root (Split-Path $p -Leaf);[IO.File]::Copy($p,$dst,$false);if((Get-FileHash $p).Hash -ne (Get-FileHash $dst).Hash){throw 'Handoff copy drift'};Pin $dst})
$c=Get-Content (Join-Path $root (Split-Path $candidatePath -Leaf)) -Raw|ConvertFrom-Json
$m=Get-Content (Join-Path $root (Split-Path $mappingPath -Leaf)) -Raw|ConvertFrom-Json
$checks=@(foreach($set in @([ordered]@{root=$c.root;sha=$c.manifest_sha256;count=$c.file_count},[ordered]@{root=$c.evidence.root;sha=$c.evidence.manifest_sha256;count=$c.evidence.file_count})){
 $mp=Join-Path $set.root 'manifest.json';if((Get-FileHash $mp).Hash -ne $set.sha){throw 'Manifest drift'};$manifest=Get-Content $mp -Raw|ConvertFrom-Json
 if($manifest.files.Count -ne $set.count){throw 'File count'}
 foreach($entry in $manifest.files){$p=Join-Path $set.root $entry.path;if((Get-FileHash $p).Hash -ne $entry.sha256 -or (Get-Item $p).Length -ne $entry.bytes){throw 'Candidate/evidence drift'}}
 $actual=@(Get-ChildItem -LiteralPath $set.root -Recurse -File|ForEach-Object{[IO.Path]::GetRelativePath($set.root,$_.FullName).Replace('\','/')}|Where-Object{$_ -ne 'manifest.json'}|Sort-Object)
 $expected=@($manifest.files.path|Sort-Object);if(@(Compare-Object $expected $actual).Count){throw 'Exact set differs'}
 $dst=Join-Path $root $(if($set.count -eq 3708){'candidate-manifest.json'}else{'evidence-manifest.json'});[IO.File]::Copy($mp,$dst,$false)
 [ordered]@{root=$set.root;manifest=Pin $dst;files=$set.count;all_SHA256_bytes_verified=$true;exact_set_verified=$true;candidate_launched=$false;read_only=$true}
})
if($m.qualified -or $m.override_absence_proven -or $m.appearance_lease_issued -or $m.observed.qualified -or $m.observed.appearance_owner_bound -or $m.observed.persisted_render_Class -ne 734 -or $m.observed.profession_Type -ne 3 -or $m.observed.Sex -ne 0){throw 'Scope drift'}
$result=Join-Path $logs 'r446-partial-observation-consumption.json'
Write-ModernEvidenceJson -Path $result -Value ([ordered]@{state='READONLY_SERVER_PARTIAL_OBSERVATION_CONSUMED_LIVE_APPEARANCE_REFUSED';checked_at=[DateTimeOffset]::Now.ToString('o');handoff_copies=$copies;sets=$checks;available=[ordered]@{same_statement_Class_Type_Sex=@(734,3,0);source=7;object=10163;weapon_object=10252;retained_type=2;equipped=$true;gear_revision=0;actual_same_Work_diagnostic_witness=$true};remaining=@('Base Class not in retained owner material/checked appearance revision','Native selected appearance owner/override presence or absence unknown','Appearance revision, exclusive cutoff and retained-output revocation unknown');BACK_backend_case='Combined held BACK+backend expiration rejection only; no independent causal/SQL-item/appearance expiry claims';diagnostic_lease_minted=$false;old_v2_replaced=$false;Rust_connected_to_new_candidate=$false;candidate_launched=$false;live_appearance_qualified=$false;renderer_LOADED=$false;GUI=0})
Write-Output 'R446 partial diagnostic read-only:3708 candidate/38 evidence pins+exact sets checked;live authority still refused'
