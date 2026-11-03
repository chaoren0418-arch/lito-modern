$ErrorActionPreference='Stop';$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003';$root=Join-Path $project 'target\r484-remaining-batch'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Read($n){Get-Content -LiteralPath (Join-Path $logs $n) -Raw|ConvertFrom-Json}
$proof=Read 'r484-independent-contract.json';$details=Read 'r484-batch-details.json';$player=Read 'r484-player-candidate.json';$binding=Read 'r484-map-binding.json'
if($proof.source_pairs -ne 157 -or $proof.CPU_reference_pairs -ne 70 -or $proof.negative_cases -ne 178 -or $proof.focused_exit_code -ne 0 -or $proof.stale_results_rejected -ne 26){throw 'New thirteen-map proof mismatch'}
$candidate=Read 'r484-consumer-candidate.json'
foreach($s in $candidate.source_pins){if((Get-FileHash -LiteralPath $s.path).Hash -ne $s.sha256 -or (Get-FileHash -LiteralPath $s.snapshot).Hash -ne $s.sha256){throw 'R484 source drift'}}
$candidate.validation="PASS one new thirteen-map finite consumer$($details.focused_seconds)seconds/70CPUreferencepairs/178negative/26stale;exit0;historical PASS reruns0"
Write-ModernEvidenceJson -Path (Join-Path $logs 'r484-candidate.json') -Value $candidate
$scope=Read 'source-change-scope.json';if($scope.source_files -ne 601 -or $scope.new_source_files.Count -ne 122 -or $scope.changed_source_files.Count -ne 10){throw 'R484 source scope'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r484-source-scope.json') -Value $scope
$qualified=@($binding.bindings|ForEach-Object{[string]$_.mode+':'+[string]$_.map_id});$remainingRows=@($binding.catalog40|Where-Object{([string]$_.mode+':'+[string]$_.map_id) -notin $qualified})
if($qualified.Count -ne 37 -or $remainingRows.Count -ne 3 -or @($remainingRows|Where-Object{$_.map_id -notin @(506,573,563)}).Count){throw 'Exact37/3 catalog split'}
$review=[ordered]@{previously_supervisor_accepted_CPU_previews=24;previous_acceptance='T300 R483 limited static/CPU scope';newly_submitted_complete_map_previews=13;current_candidate_CPU_evidenced_previews=37;current_round_supervisor_acceptance='PENDING';remaining_unqualified_catalog_maps=3}
$remaining=@('Investigate actual original Native Tile198/199 type22/23 dispatch/decode/blend for mode1/map506; required slots retain exact source/member/offset pins; no skipping or speculative decode','Investigate actual Layer7 variants of mode3/maps573/563 using direct original source/decoder evidence; corruption not inferred','Visible GUI/DPI/focus/menu/select/loading/pan/Back/close/network handoff acceptance for37 CPU evidenced previews','Original mode/menu state producer, thumb drag/track and hold repeat where proven','Native spawn/network/mode rules/actors/effects/roof/loading admission and original state machine; static preview does not grant full mode')
$negative=@();foreach($id in @(566,570,554,513,527,518,521,522,532,512,517,524,530)){
 $negative+=@(Get-ChildItem -LiteralPath (Join-Path $root "missing-native-$id") -Recurse -File|ForEach-Object{Pin $_.FullName})
 foreach($name in @("missing-$id.json","incomplete-$id.json")){$negative+=Pin (Join-Path $root $name)}
 $map=@($binding.bindings|Where-Object{$_.map_id -eq $id})[0];$absent=Join-Path $root ("missing-native-$id\$id\"+(Split-Path $map.sectors[-1].pin.path -Leaf));if(Test-Path -LiteralPath $absent){throw 'Missing fixture last required sector exists'}
}
$negative+=Pin (Join-Path $root 'wrong-mode-map566.json');if($negative.Count -ne 129){throw 'Expected102 copied sectors and27 intentional invalid-input JSON files'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r484-negative-fixture-pins.json') -Value ([ordered]@{state='R484_THIRTEEN_REAL_FULL_MAP_MISSING_LAST_SECTOR_INCOMPLETE_PLAN_AND_WRONG_MODE_FIXTURES_RETAINED';files=$negative;physical_sector_copies=102;intentional_invalid_JSON_files=27;intentional_invalid_JSON_payload_not_valid_source_authority=$true;missing_last_required_sector_files_absent=$true;failure_cases=$details.failure_cases})
$rowReference=Get-Content -LiteralPath (Join-Path $root 'row-reference\reference.json') -Raw|ConvertFrom-Json
$memory=[ordered]@{basis='Rust GlobalAlloc allocation request bytes; excludes allocator bookkeeping/OS working set; no hard OS memory/latency guarantee';map527_fixed_R483_reference_WorldMap_heap=$rowReference.results[0].world_retained_heap_requested_bytes;map527_reference_full_vector_capacity=$rowReference.results[0].full_reference_draw_vec_capacity_bytes;new_generic_catalog_row_API=$details.generic_row_map527;regular_maps=12;row_maps=1;regular_total_draw_cap=262144;row_visible_fragment_cap=262144;decoded_pixel_cap=67108864;row_index_payload_cap=8388608;row_count_cap=4096;sector_and_cumulative_encoded_source_cap=33554432;no_budget_expansion=$true;complete_original_source_and_all_required_tiles_validated=$true}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r484-memory-budget-measurement.json') -Value $memory
$history=[ordered]@{
 resource_preparer_attempts=1;new_regular_probe_compile_attempts=1;new_regular_probe_executions=1;regular_probe_qualified=12;regular_probe_measured_refusal=1;regular_refusal='527 complete18sectors/362852references exceed unchanged regular262144 total cap. All4740required slots decode10920960bytes; no unsupported members.'
 probe_console_label_caveat='Generic per-map PASS for527 in initial probe stdout means full measurement finished only; JSON actual regular adapter budget refusal is authoritative. Later separately referenced/catalog-bound row API and actual worker consumer qualify527.'
 fixed_R483_527_full_vector_reference_compile_attempts=1;fixed_R483_527_reference_executions=1;fixed_R483_527_reference_PASS=1;previous_PASS_batches_rerun=0
 production_library_builds=1;production_library_compile_PASS=1;production_library_seconds=19.13;player_compile_attempts=1;player_compile_PASS=1;player_flags_only=@('--version')
 new_consumer_compile_attempts=1;new_consumer_compile_PASS=1;new_consumer_executions=1;new_consumer_PASS=1;new_consumer_failures=0;focused_seconds=$details.focused_seconds;focused_exit_code=0;negative_cases=178;stale_results_rejected=26;background_queue_transient_busy_retries=$details.background_queue_transient_busy_retries;historical_PASS_reruns=0;GUI_executions=0
 logs=@(foreach($n in @('r484-probe-build.stdout.log','r484-probe-build.stderr.log','r484-probe.stdout.log','r484-probe.stderr.log','r484-row-reference-build.stdout.log','r484-row-reference-build.stderr.log','r484-row-reference.stdout.log','r484-row-reference.stderr.log','r484a-library-build.jsonl','r484a-library-build.stderr.log','r484-player-build.stdout.log','r484-player-build.stderr.log','r484-player-version.stdout.log','r484-player-version.stderr.log','r484-consumer-build.stdout.log','r484-consumer-build.stderr.log','r484-consumer.stdout.log','r484-consumer.stderr.log')){Pin (Join-Path $logs $n)})
}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r484-execution-history.json') -Value $history
$gui=[ordered]@{state='R484_GUI_PENDING_STATIC_CPU_ONLY';GUI=0;LOADED=$false;pending=@('37 full map rows available and3 known format refusals; visible mode/menu/select/load/pan/Back/close','Actual DPI/font/focus/capture/UI and original network handoff after retirement');Modern_adaptations=@('Whole signed bounding camera800x600; absent sector diamonds/corner pixels background, no native spawn/collision grant','NativeDrawCell scan/depth ordering with canonical source insertion snapshot; original streaming history unobserved','Existing Modern controls/keys/font/ordinary flag policy, thumb display only; source producer/drag/hold-repeat pending','At least16 complete sectors choose row allocation profile; smaller maps choose existing regular API. Explicit Modern memory policy, not a native mode rule');cancel_scope='New527 phase1..4 entry requests plus13actual worker active-load Back cancellation and stale result guards. Chunked64KB reads/hash, per-source/per-slot checks; original catalog loader and bounded parse/table decode/sort/OS calls have no hard latency promise.';remaining_required_player_flow=$remaining}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r484-unverified-gui-checklist.json') -Value $gui
$tools=@(foreach($n in @('prepare-r484-resources.py','r484-complete-map-probe.rs','build-r484-map-probe.ps1','r484-map527-reference.rs','build-r484-map527-reference.ps1','finalize-r484-binding.py','prepare-r484-candidate.py','audit-r484-source-scope.ps1','build-r484-player-candidate.ps1','start-r484-player-candidate.ps1','r484-batch-consumer.rs','build-r484-batch-consumer.ps1','verify-r484-batch.py','seal-r484-handoff.ps1','evidence-json.ps1')){Pin (Join-Path $PSScriptRoot $n)})
$evidence=@(foreach($n in @('r484-resource-input.json','r484-probe-candidate.json','r484-row-reference-input.json','r484-row-reference-candidate.json','r484-map-binding.json','r484-entry-source-contract.json','r484-player-candidate.json','r484-consumer-candidate.json','r484-candidate.json','r484-consumer-exit.json','r484-batch-details.json','r484-independent-contract.json','r484-source-scope.json','r484-negative-fixture-pins.json','r484-memory-budget-measurement.json','r484-execution-history.json','r484-unverified-gui-checklist.json')){Pin (Join-Path $logs $n)})
$previous=Join-Path $logs 'r483-two-whole-city-visible-row-handoff.json';if((Get-FileHash -LiteralPath $previous).Hash -ne '85085C729B72B8CB1A44F7887F1B54248E0AC08608F796F0DADD3227A12A96F5'){throw 'R483 handoff drift'}
$handoff=[ordered]@{state='R484_THIRTEEN_WHOLE_MAPS115SECTORS_37CPU_EVIDENCED_PREVIEWS_NEW13_REVIEW_PENDING_GUI_PENDING';previous_R483_handoff=Pin $previous;review_status=$review;player=$player.executable;production_library=$player.production_library;candidate_main=$player.candidate_main;original_main=$player.original_main;map_binding=$player.binding;skin_binding=$player.skin_binding;focused_consumer=[ordered]@{path=$candidate.path;bytes=$candidate.bytes;sha256=$candidate.sha256};implemented=[ordered]@{new_identities=$details.new_identities;complete_source_sectors=115;new_regular_maps=12;new_generic_row_map527=1;complete_source_original_and_owned_bytes=$true;all_required_tiles_decoded=$true;existing24_plans_unchanged=$true;regular_API_body_and_total_draw_cap_unchanged=$true;generic_row_public_API='load_catalog(inputs,exact_mode,mapID,...) uses existing strict LTtext/R474 authority loader before row backend';legacy_R483_city_API_guard_retained=$true;row_backend_index_decoder_render_body_unchanged=$true;row_flow_policy='whole sorted plan>=16sectors chooses rows, otherwise original regular API';shared_R483_runtime_window_byte_unchanged=$true;new_runtime_or_window_clones=0;no_decoder_or_budget_expansion=$true;whole_source_not_cropped=$true;unsupported_types_not_skipped=$true};tests=$details;memory_and_budgets=$memory;fixed_R483_independent_reference=Pin (Join-Path $root 'reference-combined.json');reference_scope='12 new full regular loads + one new527 complete full-vector independently projected tuple-key static reference, fixed R483 decoder/WorldMap/Canvas. These static CPU views and canonical source order only; no original framebuffer/stream insertion/actors/effects/roof/GUI grant.';source_scope=[ordered]@{source_files=601;baseline=479;unchanged_baseline=469;allowed_changed_baseline=10;new_sources=122;snapshot_pairs=157;prior_R483_source_pairs=155;new_candidate_main=1;new_headless_consumer_source=1;new_Rust_library_modules=0;R483_current_sources_changed=@('native_city_resources.rs','native_city_flow.rs');historical_snapshots_candidates_preserved=$true;tools=$tools};remaining_actual_catalog_rows=$remainingRows;remaining_required_player_flow=$remaining;execution_history=$history;evidence=$evidence;unverified_GUI=$gui;known_format_gaps=@('506 required Native Tile198/199 type22/23','573 Layer7 truncated170638 need109 available31','563 Layer7 truncated136068 need136 available18');new_unsupported_formats=@();corruption_inferred=$false;complete_modes=16;total_modes=40;GUI=0;LOADED=$false;root_main_written=$false;Java_SQL_Server_used=$false;provider_or_observer_used=$false;original_BIN_or_resources_written=$false;R472_timers_advanced=$false;full_original_UI_or_mode_acceptance=$false}
$path=Join-Path $logs 'r484-thirteen-complete-map-batch-handoff.json';Write-ModernEvidenceJson -Path $path -Value $handoff
$textPath=Join-Path $project 'Native-R484_十三張完整場地批次預覽交接.txt'
$text=@"
R484 十三張完整場地批次（CPU完成，新增13張監製審查／GUI待驗）

候選：$($player.executable.path)
SHA256：$($player.executable.sha256)
啟動：$PSScriptRoot\start-r484-player-candidate.ps1（-OfflinePreview直接選圖）
候選僅Hidden --version；GUI0。

mode0:566/570；mode2:554/513/527/518/521/522；mode5:532/512/517/524/530。
13整圖共115個完整原版sector，全目录、original/owned bytes/SHA、全必要tile、canonical身份/錨定/signed bounds驗證，未裁圖或跳格式。
12張原regular API直接支援；最大regular 513有14sector／245,189源引用。
527有18sector／362,852源引用，原regular API明確262,144 total cap拒絕；全部4,740必要tile槽可解碼10,920,960bytes，沒有新不支援格式。
僅527補獨立固定R483全向量參考，未重跑12張已完成probe或R483歷史批次。
generic load_catalog要求既有exact LTtext/R474完整plan驗證，crate-private row backend解除僅534/535限制；舊city guard保留。原row index/decoder/render本體與所有budget不變；共用R483 runtime/window，沒有新增runtime同義副本。
>=16個完整sector使用rows是Modern配置政策，其他圖原regular；534/535舊候選／source snapshots不動。R484只測新13張，沒有宣稱重驗舊24。
527列索引2,584,752bytes、保留geometry與完整必要tile；Rust配置記憶體與每frame可見fragments／取消入口觀測附JSON，排除allocator metadata/OS working set、不承諾硬時限。
每frame262,144fragment／64MBdecoded／32MB原圖與累計encoded／8MB index payload／4096rows，原regular總上限未提高。

新finite consumer退出碼0／PASS，$($details.focused_seconds)秒；70組CPU參考比較（65實際worker＋527直接API5），178負向／邊界、26queued舊load/pan拒絕、13次active-load Back取消/切圖、held frame撤銷、worker實際退出。
每圖最後必要sector物理缺失／整圖plan不完整拒絕，保留exact mode/map且無partial/fallback；527公共catalog API另驗錯mode及4個phase入口取消。
參考證明上述靜態CPU視圖／完整來源／canonical插入政策；原版framebuffer、stream插入、spawn/network/rules/actors/effects/roof/GUI仍未授權。
整輪新probe12合格／527量測拒絕→新row參考1PASS→一次新consumer PASS；沒有编译或consumer失敗，歷史PASS重跑0。

601 source／469 baseline unchanged／10 allowed changed／122 new；157固定current/snapshot pairs。現行只修改R483 city_resources/flow兩個自有檔，parent native_resources、mod、runtime、window、regular/shared render均不變；歷史指紋保留。
先前24張有限接受，本輪13張新CPU證據待審；當前37/40預覽有CPU證據，剩3張506/573/563是真實格式缺口，不推判corrupt。
完整模式仍16/40、GUI0、LOADEDfalse；未用Java/SQL/server/provider/原版GUI，不寫原版資源，不推進R472 timers。
下一實際玩家流程：查Native198/199 type22/23原版dispatch/blend及573/563 Layer7，再做可見GUI與原版mode/menu producer／spawn/network/rules/actors/effects/roof/admission狀態機。
完整交接：$path
"@
[IO.File]::WriteAllText($textPath,$text,[Text.UTF8Encoding]::new($false))
$progressPath=Join-Path (Split-Path -Parent $project) '監製進度.json';$progress=Get-Content -LiteralPath $progressPath -Raw|ConvertFrom-Json
$progress.updated_at=[DateTimeOffset]::Now.ToString('o');$progress.phase='R484_thirteen_whole_maps115sectors37CPU_previews_new13_review_pending_three_format_gaps_GUI_pending';$progress.current_task="R48413wholemaps115sectors;12regular+527generic catalog-bound rows;budgets unchanged/sharedruntimewindow unchanged. One finite consumer$($details.focused_seconds)sPASS/exit0/70CPU/178negative/26stale/157sourcepairs. Prior24accepted;current37CPU-evidenced new13review pending;remaining50622_23/573563Layer7. GUI0/16of40/LOADEDfalse. Next actual source decoder/menu producer/state-machine/visible GUI gaps."
$progress|Add-Member -NotePropertyName native_R484_handoff -NotePropertyValue (Pin $path) -Force
$progress|Add-Member -NotePropertyName native_R484_candidate -NotePropertyValue $player.executable -Force
$progress|Add-Member -NotePropertyName native_R484_review_status -NotePropertyValue $review -Force
$progress|Add-Member -NotePropertyName native_R484_remaining_player_flow -NotePropertyValue $remaining -Force
$progress|Add-Member -NotePropertyName native_R483_supervisor_acceptance -NotePropertyValue ([ordered]@{state='T300_ACCEPTED_LIMITED_STATIC_CPU_SCOPE';accepted_catalog_previews=24;handoff=Pin $previous;GUI=0;LOADED=$false;complete_modes=16}) -Force
Write-ModernEvidenceJson -Path $progressPath -Value $progress
Write-ModernEvidenceJson -Path (Join-Path $logs 'r484-final-audit.json') -Value ([ordered]@{state='R484_FIXED_SOURCE_CANDIDATE_WHOLE_MAP_BUDGET_REFERENCE_GENERIC_ROWS_CANCEL_FAILURE_HANDOFF_PROGRESS_SEALED';handoff=Pin $path;text=Pin $textPath;progress=Pin $progressPath;source_pairs=157;review_status=$review;focused_exit_code=0;GUI=0;LOADED=$false})
Write-Output (ConvertTo-Json (Pin $path) -Compress)
