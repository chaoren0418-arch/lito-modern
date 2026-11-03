$ErrorActionPreference='Stop';$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003';$root=Join-Path $project 'target\r483-city-visible-rows'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Read($n){Get-Content -LiteralPath (Join-Path $logs $n) -Raw|ConvertFrom-Json}
$proof=Read 'r483-independent-contract.json';$details=Read 'r483-city-details.json';$player=Read 'r483b-player-candidate.json';$binding=Read 'r483b-map-binding.json'
if($proof.source_pairs -ne 155 -or $proof.CPU_reference_pairs -ne 20 -or $proof.negative_cases -ne 45 -or $proof.focused_exit_code -ne 0 -or $proof.stale_results_rejected -ne 4){throw 'New city proof mismatch'}
$candidate=Read 'r483c-consumer-candidate.json'
foreach($s in $candidate.source_pins){if((Get-FileHash -LiteralPath $s.path).Hash -ne $s.sha256 -or (Get-FileHash -LiteralPath $s.snapshot).Hash -ne $s.sha256){throw 'R483 source drift'}}
$candidate.validation='PASS one finite new city API/actual worker consumer122.3840031seconds/20CPUreferencepairs/45negative/4stale; exit0; historical PASS reruns0'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r483-candidate.json') -Value $candidate
$scope=Read 'source-change-scope.json';if($scope.source_files -ne 601 -or $scope.new_source_files.Count -ne 122 -or $scope.changed_source_files.Count -ne 10){throw 'R483 source scope'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r483-source-scope.json') -Value $scope
$qualified=@($binding.bindings|ForEach-Object{[string]$_.mode+':'+[string]$_.map_id});$remainingRows=@($binding.catalog40|Where-Object{([string]$_.mode+':'+[string]$_.map_id) -notin $qualified});if($qualified.Count -ne 24 -or $remainingRows.Count -ne 16){throw '24/16 exact catalog identity split'}
$review=[ordered]@{previously_supervisor_accepted_CPU_previews=22;previous_acceptance='T293 R482 limited static/CPU scope';newly_submitted_whole_city_previews=2;current_candidate_CPU_evidenced_previews=24;current_round_supervisor_acceptance='PENDING';remaining_unqualified_catalog_maps=16}
$remaining=@('Qualify exact whole resources/shapes/source budgets for remaining16 catalog maps;13 have not received new whole-map player preview qualification','Investigate506 required Native Tile198/199 type22/23 using original decoder/blend evidence; no art skipping','Investigate573/563 Layer7 variants without guessing corruption','Visible GUI/DPI/focus/menu/selection/loading/pan/Back/close/network-handoff acceptance for24 CPU evidenced maps','Native mode/menu flag producer, thumb drag/track and hold repeat where proven','Native spawn/network/mode rules/actors/effects/roof/loading admission; static preview is separate from full mode')
$memory=[ordered]@{basis='Measured Rust GlobalAlloc request sizes; excludes allocator bookkeeping/OS working set, not a hard memory or OS latency guarantee';fixed_R482_reference_WorldMap_requested_heap=@(2555310,4171256);fixed_independent_full_vector_capacity_bytes=@(37748736,75497472);cities=$details.map_cases;row_index_payload_limit=8388608;row_count_limit=4096;visible_frame_fragment_limit=262144;decoded_pixel_limit=67108864;source_and_cumulative_encoded_byte_limit=33554432;all_original_sectors_and_required_tiles_validated=$true;encoded_members_dropped_after_selected_decode=$true;source_total_draw_cap_in_regular_API_unchanged=262144;floor_not_duplicated_in_row_index=$true;no_full_city_draw_vector_in_production=$true}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r483-memory-budget-measurement.json') -Value $memory
$failureCandidate=Read 'r483b-consumer-candidate.json'
foreach($s in $failureCandidate.source_pins){if((Get-FileHash -LiteralPath $s.snapshot).Hash -ne $s.sha256){throw 'Failed consumer snapshots drift'}}
$history=[ordered]@{
 reference_compile_attempts=1;reference_executions=1;reference_two_city_full_source_PASS=2;fixed_reference_library='R482 production library';reference_method='Complete full draw tuple vector and independently projected/sorted static CPU views; uses fixed R482 S32/TIL/WorldMap/Canvas. This vector is reference-only.'
 production_library_compile_attempts=2;production_library_compile_PASS=2;initial_library_unexecuted='40.32s initial library; caller-tightened fragment-budget API added before consumer testing, then final12.62s library. Initial rlib retained; no initial library runtime acceptance.'
 retained_initial_library=Pin (Join-Path $project 'target\r483a-unexecuted-library\liblito_modern.rlib')
 consumer_compile_attempts=3;consumer_compile_PASS=2;consumer_compile_failures=1;initial_compile_failure='Consumer called Runtime::start with extra None argument; corrected consumer only; failed source/compiler log retained'
 initial_compile_failed_source=Pin (Join-Path $PSScriptRoot 'r483a-city-consumer-compile-failed.rs')
 consumer_executions=2;consumer_failures=1;failed_consumer_before_map_tests='Initial assertion incorrectly required entire LTtext catalog choices to be40; actual parser includes Town. Corrected to40 exact mode/map rows and catalog membership. No city test/CPU PASS had run in failed execution.'
 failed_consumer_candidate=Pin (Join-Path $logs 'r483b-consumer-candidate.json');failed_consumer_exe=[ordered]@{path=$failureCandidate.path;bytes=$failureCandidate.bytes;sha256=$failureCandidate.sha256};failed_snapshot_pairs=155;failed_snapshot_pins_verified=$true;failed_candidate_current_pointer_graph_not_reinterpreted=$true
 final_consumer_executions=1;final_consumer_PASS=1;focused_seconds=$details.focused_seconds;focused_exit_code=0;historical_PASS_reruns=0
 player_compile_attempts=2;player_compile_PASS=2;player_flags_only=@('--version');initial_player_metadata='24 complete plans correct; inherited descriptive unqualified18 and city-measurement metadata remained from R482. Preserved first manifest/exe; final separate manifest corrects to16remaining, clears measurement-only city IDs, and names new row profile. Tested24 plans and final24 plans byte-equivalent JSON values; same fixed production library; no runtime/source change required.'
 GUI_executions=0
 logs=@(foreach($n in @('r483-reference-build.stdout.log','r483-reference-build.stderr.log','r483-reference.stdout.log','r483-reference.stderr.log','r483a-library-build.jsonl','r483a-library-build.stderr.log','r483b-library-build.jsonl','r483b-library-build.stderr.log','r483-consumer-build.stdout.log','r483-consumer-build.stderr.log','r483b-consumer-build.stdout.log','r483b-consumer-build.stderr.log','r483-consumer.stdout.log','r483-consumer.stderr.log','r483c-consumer-build.stdout.log','r483c-consumer-build.stderr.log','r483c-consumer.stdout.log','r483c-consumer.stderr.log','r483-player-build.stdout.log','r483-player-build.stderr.log','r483-player-version.stdout.log','r483-player-version.stderr.log','r483b-player-build.stdout.log','r483b-player-build.stderr.log','r483b-player-version.stdout.log','r483b-player-version.stderr.log')){Pin (Join-Path $logs $n)})
}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r483-execution-history.json') -Value $history
Write-ModernEvidenceJson -Path (Join-Path $logs 'r483-negative-fixture-pins.json') -Value ([ordered]@{state='R483_TWO_REAL_WHOLE_CITY_MISSING_LAST_SECTORS_INCOMPLETE_AND_WRONG_MODE_FIXTURES_RETAINED';files=$details.fixture_pins;physical_sector_copies=39;intentional_invalid_JSON_files=5;intentional_invalid_JSON_payload_not_valid_source_authority=$true;missing_last_sector_absent=$true;failures=$details.failure_cases;failed_initial_consumer=Pin (Join-Path $logs 'r483b-consumer-candidate.json')})
$gui=[ordered]@{state='R483_GUI_PENDING_STATIC_CPU_ONLY';GUI=0;LOADED=$false;pending=@('24 whole-map rows visible and16 unavailable, real select/load/pan/Back/close and network handoff','Actual DPI/font/focus/capture/UI event acceptance');Modern_adaptations=@('800x600 signed whole bounding-box camera; empty corner pixels background, no native spawn/collision inference','Canonical source insertion domain and NativeDrawCell/depth ordering, original streaming insertion history unobserved','Existing Modern menu/controls/keys/font/ordinary flag policy; thumb display only, native producer/drag/hold-repeat pending');cancel_scope='Observed phase1sector IO,2WorldMap,3index,4archive/decode phase entry requests and cooperative join; chunked64KB hash/read and per-source/per-slot checks. Existing bounded single-sector parse/single-entry table/sort/OS calls do not promise hard deadlines.';remaining_required_player_flow=$remaining}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r483-unverified-gui-checklist.json') -Value $gui
$tools=@(foreach($n in @('r483-city-reference.rs','build-r483-city-reference.ps1','prepare-r483-candidate.py','audit-r483-source-scope.ps1','build-r483-player-candidate.ps1','build-r483b-player-candidate.ps1','start-r483-player-candidate.ps1','r483-city-consumer.rs','r483a-city-consumer-compile-failed.rs','r483b-city-consumer-build-history.ps1','build-r483-city-consumer.ps1','verify-r483-city.py','seal-r483-handoff.ps1','evidence-json.ps1')){Pin (Join-Path $PSScriptRoot $n)})
$evidence=@(foreach($n in @('r483-resource-input.json','r483-reference-candidate.json','r483-map-binding.json','r483b-map-binding.json','r483-entry-source-contract.json','r483b-entry-source-contract.json','r483-player-candidate.json','r483b-player-candidate.json','r483-consumer-exit.json','r483c-consumer-exit.json','r483c-consumer-candidate.json','r483-candidate.json','r483-city-details.json','r483-independent-contract.json','r483-source-scope.json','r483-memory-budget-measurement.json','r483-execution-history.json','r483-negative-fixture-pins.json','r483-unverified-gui-checklist.json')){Pin (Join-Path $logs $n)})
$previous=Join-Path $logs 'r482-nine-cross-mode-complete-map-handoff.json';if((Get-FileHash -LiteralPath $previous).Hash -ne '4096AB39B677BDEFDAEFFA78C058E39F13459CB0FABCAE0A29C96F3E74EF26B9'){throw 'R482 handoff drift'}
$handoff=[ordered]@{state='R483_TWO_WHOLE_CITIES41SECTORS_BOUNDED_VISIBLE_ROWS_24CPU_EVIDENCED_PREVIEWS_NEW2_REVIEW_PENDING_GUI_PENDING';previous_R482_handoff=Pin $previous;review_status=$review;player=$player.executable;production_library=$player.production_library;candidate_main=$player.candidate_main;original_main=$player.original_main;map_binding=$player.binding;skin_binding=$player.skin_binding;focused_consumer=[ordered]@{path=$candidate.path;bytes=$candidate.bytes;sha256=$candidate.sha256};implemented=[ordered]@{new_identities=@(@(1,534),@(5,535));complete_sectors=41;canonical_anchor_and_full_source_identity=$true;all_required_selected_tiles_decoded=$true;compact_non_floor_rows=$true;implicit_floor_visible_rows=$true;source_draw_total_regular_cap_not_raised=$true;whole_source_not_cropped=$true;unsupported_types_not_skipped=$true;existing22_plans_unchanged=$true;separate_city_API_and_flow=$true;shared_native_resources_change='Append child module declaration only; original load_static body byte-equivalent text unchanged';shared_R480_renderer_flow_runtime_resources_unchanged=$true;no_decoder_change=$true};tests=$details;memory_and_budgets=$memory;fixed_R482_independent_reference=Pin (Join-Path $root 'reference\reference.json');reference_scope=$details.fixed_reference_method;source_scope=[ordered]@{source_files=601;baseline=479;unchanged_baseline=469;allowed_changed_baseline=10;new_sources=122;snapshot_pairs=155;prior_R482_source_pairs=149;new_library_modules=4;new_candidate_main=1;new_headless_consumer_source=1;R482_current_sources_changed=@('native_resources.rs','mod.rs');historical_snapshots_and_binaries_preserved=$true;tools=$tools};remaining_actual_catalog_rows=$remainingRows;remaining_required_player_flow=$remaining;execution_history=$history;evidence=$evidence;unverified_GUI=$gui;known_format_gaps=@('506 required Native Tile198/199 type22/23','573 Layer7 truncated170638 need109 available31','563 Layer7 truncated136068 need136 available18');corruption_inferred=$false;complete_modes=16;total_modes=40;GUI=0;LOADED=$false;root_main_written=$false;Java_SQL_Server_used=$false;provider_or_observer_used=$false;original_BIN_or_resources_written=$false;R472_timers_advanced=$false;full_original_UI_or_mode_acceptance=$false}
$path=Join-Path $logs 'r483-two-whole-city-visible-row-handoff.json';Write-ModernEvidenceJson -Path $path -Value $handoff
$textPath=Join-Path $project 'Native-R483_兩張完整城市有界可見列預覽交接.txt'
$text=@"
R483 兩張完整城市有界可見列預覽（CPU 完成，新增2張監製審查／GUI 待驗）

候選：$($player.executable.path)
SHA256：$($player.executable.sha256)
啟動：$PSScriptRoot\start-r483-player-candidate.ps1（-OfflinePreview 直接選圖）
候選僅 Hidden --version；GUI0。

mode1／534完整16sector、mode5／535完整25sector，共41個原版sector；全檔、geometry/canonical及全部必要tile驗證，不裁圖、不跳格式。
獨立city API保留完整geometry，以地板即時計算＋疊图/物件12byte列索引，按可見列產生暫存fragment；沒有完整城市draw Vec。
原regular load_static的262,144 total draw cap及函式本體保留；city profile每frame262,144fragment／64MB解碼／32MB原圖和累計encoded源／8MB index payload／4096index rows。
534：347,528源引用；2,597,472bytes index／511rows；32,601,600decoded bytes；資產drop實際釋放38,744,418bytes Rust配置量，載入增量峰值38,898,026bytes。
535：558,061源引用；4,239,132bytes index／639rows；29,475,072decoded bytes；資產drop實際釋放38,789,060bytes，載入增量峰值38,942,236bytes。
量測只含Rust allocator request，排除allocator metadata與OS working set。固定參考全向量容量37,748,736／75,497,472bytes只在參考程式存在。
5組相機中可見fragments最大534:7,218／535:7,522；可見列57，fragment容量最高393,216bytes，未掃全部索引列。signed全圖bounds/極值／Modern角落背景保留。

新finite consumer退出碼0／PASS，122.3840031秒；20組真CPU比對（direct API與實際worker各10），逐像素與固定R482獨立完整向量參考一致。
45項負向檢查，4個queued跨mode舊load/pan拒絕、held frame撤銷，2圖Back取消與active-load Close後worker退出；兩城市最後必要sector物理缺失／plan不完整／錯mode均拒絕、無partial或舊frame。
8次phase1..4入口取消join觀測0.0809..3.1189ms；不承諾OS、單sector parse、tile table decode/sort硬時限。
參考證明這些靜態CPU視圖、完整來源及canonical source插入政策；未證明原版framebuffer、原版stream insertion、actors/effects/roof或GUI。

初consumer編譯呼叫多一個None失敗；修正後首次consumer在「包含Town的全表必須40」誤假設拒絕，尚無城市測試PASS；改核對40精確mode場地後當輪新consumer PASS，失敗來源/exe/log/snapshot保留。
初候選24plans正確，但繼承舊18剩餘／city僅量測說明；final另檔修正成16剩餘、city獨立profile，24plans與consumer測試完全相同、同一production library，未重跑PASS。
601 source／469 baseline unchanged／10 allowed changed／122 new；155 current/snapshot pairs。R482與歷史候選/snapshot保留；native_resources僅加child module宣告，原regular函式本體與R480 renderer/flow/runtime不改。
先前22張有限接受，本輪新增2張CPU證據待審；當前24/40預覽有CPU證據、剩16張，完整模式仍16/40、GUI0、LOADEDfalse。
下一步：其餘13張完整整圖資格與506 type22/23、573/563 Layer7來源缺口；實際GUI與native producer/spawn/network/mode rules/actors/effects/roof/admission。
未用Java/SQL/server/provider/原版GUI，未寫原版資源，不推進R472 timers。
完整交接：$path
"@
[IO.File]::WriteAllText($textPath,$text,[Text.UTF8Encoding]::new($false))
$progressPath=Join-Path (Split-Path -Parent $project) '監製進度.json';$progress=Get-Content -LiteralPath $progressPath -Raw|ConvertFrom-Json
$progress.updated_at=[DateTimeOffset]::Now.ToString('o');$progress.phase='R483_two_whole_cities41sectors_bounded_visible_rows24CPU_previews_new2_review_pending_GUI_pending';$progress.current_task='R483 whole53416/53525sectors complete geometry/all tiles, implicit floor/compact rows; per-frame262144/decoded64MB regular total cap unchanged. One final finite consumer122.384sPASS/exit0/20CPU/45negative/4stale/8phase cancel/155sourcepairs. Prior22 accepted; current24CPU-evidenced new2review pending/16remaining;GUI0/16of40/LOADEDfalse.'
$progress|Add-Member -NotePropertyName native_R483_handoff -NotePropertyValue (Pin $path) -Force
$progress|Add-Member -NotePropertyName native_R483_candidate -NotePropertyValue $player.executable -Force
$progress|Add-Member -NotePropertyName native_R483_review_status -NotePropertyValue $review -Force
$progress|Add-Member -NotePropertyName native_R483_remaining_player_flow -NotePropertyValue $remaining -Force
$progress|Add-Member -NotePropertyName native_R482_supervisor_acceptance -NotePropertyValue ([ordered]@{state='T293_ACCEPTED_LIMITED_STATIC_CPU_SCOPE';accepted_catalog_previews=22;handoff=Pin $previous;GUI=0;LOADED=$false;complete_modes=16}) -Force
Write-ModernEvidenceJson -Path $progressPath -Value $progress
Write-ModernEvidenceJson -Path (Join-Path $logs 'r483-final-audit.json') -Value ([ordered]@{state='R483_FIXED_SOURCE_CANDIDATE_WHOLE_CITY_MEMORY_BUDGET_REFERENCE_CANCEL_FAILURE_HANDOFF_PROGRESS_SEALED';handoff=Pin $path;text=Pin $textPath;progress=Pin $progressPath;source_pairs=155;review_status=$review;focused_exit_code=0;GUI=0;LOADED=$false})
Write-Output (ConvertTo-Json (Pin $path) -Compress)
