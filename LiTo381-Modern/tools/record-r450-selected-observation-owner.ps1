$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$mapping='D:\lineage開發\3-lito開發\lito伺服器\Native-selected外觀owner生命週期與最小交接.json'
$candidate='D:\lineage開發\3-lito開發\lito伺服器\Native-selected外觀owner候選交接.json'
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash $p).Hash;bytes=(Get-Item $p).Length}}
$root=Join-Path $project ('target\r450-selected-observation-owner-readonly-'+[Guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($root)|Out-Null
$handoffs=@(foreach($p in @($mapping,$candidate)){$dst=Join-Path $root (Split-Path $p -Leaf);[IO.File]::Copy($p,$dst,$false);if((Get-FileHash $p).Hash -ne (Get-FileHash $dst).Hash){throw 'Handoff copy drift'};Pin $dst})
$c=Get-Content (Join-Path $root (Split-Path $candidate -Leaf)) -Raw|ConvertFrom-Json
$m=Get-Content (Join-Path $root (Split-Path $mapping -Leaf)) -Raw|ConvertFrom-Json
$sets=@(foreach($set in @([ordered]@{root=$c.root;sha=$c.manifest_sha256;count=$c.file_count},[ordered]@{root=$c.evidence.root;sha=$c.evidence.manifest_sha256;count=$c.evidence.file_count})){
 $mp=Join-Path $set.root 'manifest.json';if((Get-FileHash $mp).Hash -ne $set.sha){throw 'Manifest drift'};$manifest=Get-Content $mp -Raw|ConvertFrom-Json;if($manifest.files.Count -ne $set.count){throw 'Count drift'}
 foreach($p in $manifest.files){$path=Join-Path $set.root $p.path;if((Get-FileHash $path).Hash -ne $p.sha256 -or (Get-Item $path).Length -ne $p.bytes){throw 'Candidate/evidence drift'}}
 $actual=@(Get-ChildItem $set.root -Recurse -File|ForEach-Object{[IO.Path]::GetRelativePath($set.root,$_.FullName).Replace('\','/')}|Where-Object{$_ -ne 'manifest.json'}|Sort-Object);$expected=@($manifest.files.path|Sort-Object);if(@(Compare-Object $actual $expected).Count){throw 'Exact set differs'}
 $dst=Join-Path $root $(if($set.count -eq 3830){'candidate-manifest.json'}else{'evidence-manifest.json'});[IO.File]::Copy($mp,$dst,$false);[ordered]@{root=$set.root;manifest=Pin $dst;files=$set.count;all_SHA256_bytes_verified=$true;exact_set_verified=$true}
})
if($m.qualified -or $m.new_appearance_lease -or $m.actual_live.observed.qualified -or !$m.actual_live.old_retained_not_revived -or !$m.actual_live.retired_after_BACK -or !$m.actual_live.actual_disconnect_retired_new_owner -or $m.program_invariants.count -ne 16 -or $m.actual_live.physical_SQL_reads -ne 1){throw 'Owner observation scope differs'}
$result=Join-Path $logs 'r450-selected-observation-owner-consumption.json'
Write-ModernEvidenceJson -Path $result -Value ([ordered]@{state='SERVER_SELECTED_OBSERVATION_OWNER_EVIDENCE_READONLY_CONSUMED_NO_APPEARANCE_ADMISSION';checked_at=[DateTimeOffset]::Now.ToString('o');handoff_copies=$handoffs;sets=$sets;producer_program_invariants=16;producer_actual_SQL_Work_cases=1;producer_actual_observation=[ordered]@{source=7;object=10275;Type_Sex_Class=@(0,0,0);gear_revision=0;weapon_object=10304;retained_type=3;privategen_initial='0000000000000002';privategen_reselected='0000000000000005';owner_observation_rev_before_BACK=2;owner_observation_rev_after_BACK=3;old_retained_revoked=$true;disconnect_closes_new_owner=$true};revision_semantics=$m.revision_semantics;expiry_coverage=$m.expiry_coverage;minimum_use=$m.Modern_minimal_use;remaining=@('Override authority remains UNKNOWN; observation owner is not Classic/Full-VIP override owner','Observation revision cannot replace persisted appearance revision or exclusive override cutoff','External SQL updates have no push bridge; private producer handle cannot be reconstructed from JSON');candidate_launched=$false;Rust_connected_to_new_candidate=$false;old_v2_replaced=$false;private_handle_minted=$false;PreparedResources_minted=$false;live_appearance_qualified=$false;renderer_LOADED=$false;GUI=0})
Write-Output 'Read-only owner evidence:3830+26pins/exact sets;16program invariants separate from1actualWork;appearance authority still unknown'
