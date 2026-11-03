$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash $p).Hash;bytes=(Get-Item $p).Length}}
function Check($p){if((Get-FileHash $p.path).Hash -ne $p.sha256){throw 'Pin drift'};if($p.bytes -and (Get-Item $p.path).Length -ne $p.bytes){throw 'Byte drift'}}
$stages=@(foreach($stage in @('r449','r450','r451')){$result=Join-Path $logs "$stage-finite-renderer-result.json";$v=Get-Content $result -Raw|ConvertFrom-Json;Check $v.candidate;Check $v.candidate.production_library;foreach($p in $v.candidate.source_pins){if((Get-FileHash $p.snapshot).Hash -ne $p.sha256){throw 'Source snapshot drift'}};foreach($p in @($v.images)+@($v.selected_resources)+@($v.extra_inputs)){Check $p};[ordered]@{stage=$stage;result=Pin $result;final_verification=Pin (Join-Path $logs "$stage-final-verification.json");source_snapshots=$v.candidate.source_pins.Count;ASM_functions=$v.original_fragments;resources=$v.selected_resources.Count;extra_inputs=$v.extra_inputs.Count;images=$v.images.Count;focused_passed=1}})
$failedPath=Join-Path $logs 'r450a-compile-failed-attempt.json';$failed=Get-Content $failedPath -Raw|ConvertFrom-Json;foreach($p in $failed.source_pins){if((Get-FileHash $p.snapshot).Hash -ne $p.sha256){throw 'Failed draft snapshot drift'}};Check $failed.compiler_log
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$scopePath=Join-Path $logs 'source-change-scope.json';$scope=Get-Content $scopePath -Raw|ConvertFrom-Json
$root=Join-Path $project ('target\r449-r451-fixed-handoff-'+[Guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($root)|Out-Null
$fixedScope=Join-Path $root 'source-change-scope-518.json';[IO.File]::Copy($scopePath,$fixedScope,$false);if((Get-FileHash $scopePath).Hash -ne (Get-FileHash $fixedScope).Hash){throw 'Scope copy drift'}
$oldSummary=Join-Path $logs 'r447-r448-combined-handoff.json';$old=Get-Content $oldSummary -Raw|ConvertFrom-Json
$correction=Join-Path $logs 'r447-r448-rolling-scope-reference-correction.json'
Write-ModernEvidenceJson -Path $correction -Value ([ordered]@{state='OLD_COMBINED_SUMMARY_REFERENCED_ROLLING_AUDIT_NOT_IMMUTABLE_SNAPSHOT';checked_at=[DateTimeOffset]::Now.ToString('o');old_summary_preserved=Pin $oldSummary;old_rolling_reference=$old.source_scope;current_rolling_audit=Pin $scopePath;old_scope_bytes_no_longer_at_rolling_path=$true;old_reference_must_not_be_treated_as_current_valid_pin=$true;R447_R448_host_library_source_snapshots_resources_images_unchanged=$true;correction='The old combined summary used source-change-scope.json, a rolling audit overwritten by later source audits. No historical hash is replaced or claimed recovered. Historical renderer snapshots remain valid; stage-specific source_files and frozen source hashes retain their original validation. This run copies current audit to an immutable path before pinning.';this_run_fixed_scope=Pin $fixedScope})
$owner=Join-Path $logs 'r450-selected-observation-owner-consumption.json'
$handoff=Join-Path $logs 'Native-appearance-R449-R451有限動作遮擋與真地形本輪交接.txt'
[IO.File]::WriteAllText($handoff,@"
R449a：1new focused0.34sec，單一finiteReplay含clock與layer preflight，七步frame1..6(bank88/sub0..5)→end0(bank72/sub0)。來源body3476/shadow3477/1056260皆同bank/subframe。end0=10body/0layer/4shadow fragments；clock1000/1168/1336/1504/1631/1758/1885→queueidle49due2469(delay584)後停，不執行idle。21missingpointer/flag與7wrongbank拒絕均不推進待執行clock。47source/21fullASM/9inputs/host/lib/1strip封存視閱。
R450b：1new focused7.45sec；實際Native38-0 FirePot rear/same/front source fragments，body可見1128→rear1128/front31。不同actual攻擊subframes0/3有11same-cell/depth交點，兩insertion順序像素/可見範圍不同；六情境像素全部等於独立literal半格scan/depth/newest-first參考。50source/9fullASM/6appearanceinputs＋4Nativearchive/provider/memberpins/host/lib/6PNG封存視閱。未改源depth或按objectID排序。r450a兩E0521 compilefailed草稿三source及compiler log保留，未執行測試。
R451a：1new focused24.42sec；實際Native564/7fff8000 encoded/canonical geometry與LTtile/LTsprite四archive角色驗證。8192floor/10802object refs與上段有限動作七state同bank/subframe/clock交錯，固定conditionalcell32724/32784、static-source roof與terrain-first policy，三欄缺失先拒絕；七budget用盡/end72停。body visible1012/1083/1060/1050/1034/938/807。59source/9fullASM/9appearanceinputs＋6provider/S32/archive pins/host/lib/7張512x384CPU封存視閱。非actual玩家worldposition；Layer8自然動畫/actor-drivenroof未執行。
Server selected observation owner新3830candidate＋26evidence全SHA/bytes/exact set唯讀消費；16program invariants與1actualsource7object10275Work分列，0/0/0、gearrev0weapon10304type3，BACK observationrev2→3撤銷，reselectprivategen2→5不復活舊handle，disconnect關新owner。觀察owner/revision不是override authority/appearance revision/cutoff，外部SQLupdate無pushbridge；qualifiedfalse。Modern未啟動新候選/未連線/未造privatehandle、lease或PreparedResources。
518src=479baseline/469unchanged/10allowedchanged/39new，R408及R447/448等歷史source/binary/resource/CPU snapshots保持，無舊live套件重跑。新增3focused全passed，14張CPU圖片視閱。LOADEDfalse/GUI0/16of40，overall仍in progress。
稽核引用更正：舊R447/R448彙總將會更新的source-change-scope.json當固定引用，原SHA已不在rollingpath，舊紀錄保持並明示失效，不宣告已恢復舊bytes；renderer历史固定快照不變。本輪改釘不可變copy：$fixedScope 。Correction: $correction
後續renderer界線：idle49 callback dispatch/切換、actor-driven roof alias/clock及Layer8/event branches仍需各自原版條件與足夠資源；live外觀需要真正override owner/revision/exclusivecutoff/revocation，不由CPU成功推出。
R449: $($stages[0].result.path)
R450: $($stages[1].result.path)
R451: $($stages[2].result.path)
Owner diagnostic: $owner
"@,[Text.UTF8Encoding]::new($false))
$result=Join-Path $logs 'r449-r451-combined-handoff.json'
Write-ModernEvidenceJson -Path $result -Value ([ordered]@{state='R449_R450_R451_FINITE_RENDERER_STAGES_COMPLETE_LIVE_AND_RUNTIME_BRANCHES_PENDING';checked_at=[DateTimeOffset]::Now.ToString('o');stages=$stages;focused_new_passed=3;focused_failed=0;compile_failed_attempt_preserved=Pin $failedPath;CPU_images_visually_reviewed=14;source_files=518;source_scope=Pin $fixedScope;rolling_scope_reference_correction=Pin $correction;selected_observation_owner=Pin $owner;handoff=Pin $handoff;R408_changed=$false;original_BIN_executed=$false;old_live_suites_rerun=$false;live_appearance_qualified=$false;renderer_LOADED=$false;GUI=0;overall_status='in_progress'})
$statusPath=Join-Path (Split-Path $project -Parent) '監製進度.json';$status=Get-Content $statusPath -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o');$status.phase='R449_R450_R451_finite_player_action_prop_occlusion_static_terrain_passed_runtime_authority_pending';$status.current_task='R449finite七步bank88→end72clock/layer整合0.34sec；R450actualFirePot前後遮擋1128→31/actualsubframes11depthties反向insertion像素不同7.45sec；R451真Native5648192floor10802object與七步玩家交錯24.42sec。3new focused pass/14CPU圖片逐張視閱，47/50/59source、21/9/9fullASM、所有resource/host/lib固定。518sources/R408保持、r450a compilefailure留存。Server selected observationowner3830+26pins/exactsets唯讀消費，16program與1actualWork分開，非override權威/qualifiedfalse。舊rollingaudit引用更正保留，本輪scope不可變copy。LOADEDfalse/GUI0/16of40，未重跑舊live或啟動新owner候選。'
$status|Add-Member native_R449_R451 (Pin $result) -Force
$status.next_step='Native idle49 callback條件、actor-driven roof alias/state/timer及Layer8/event足證分支待續；目前完成原版有限步進、shadow105/end72、實際場景遮擋與真地形CPU。live外觀override owner/revision/cutoff/revocation仍未知，不能提升LOADED或GUI。'
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output 'R449/R450/R451 fixed combined handoff:3newpass/14reviewedCPU/518sources;source audit copied immutable;live/runtime pending'
