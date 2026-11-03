$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$root=Join-Path $project 'target\r482-cross-mode-batch'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Read($n){Get-Content -LiteralPath (Join-Path $logs $n) -Raw|ConvertFrom-Json}
$proof=Read 'r482-independent-contract.json'
$details=Read 'r482-cross-mode-details.json'
$player=Read 'r482-player-candidate.json'
$binding=Read 'r482-map-binding.json'
if($proof.source_pairs -ne 149 -or $proof.actual_player_reference_pairs -ne 45 -or $proof.negative_cases -ne 122 -or $proof.focused_seconds -ne 270.20 -or $proof.stale_results_rejected -ne 18){throw 'Independent R482 proof mismatch'}
$candidate=Read 'r482a-candidate.json'
foreach($s in $candidate.source_pins){if((Get-FileHash -LiteralPath $s.path).Hash -ne $s.sha256 -or (Get-FileHash -LiteralPath $s.snapshot).Hash -ne $s.sha256){throw 'Final source drift'}}
$candidate.validation='PASS one focused execution270.20seconds122negative18stale45actualCPUreferencepairs; freeze process final exit0 verified by resumed session; historical PASS reruns0'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r482-candidate.json') -Value $candidate
$scope=Read 'source-change-scope.json'
if($scope.source_files -ne 597 -or $scope.new_source_files.Count -ne 118 -or $scope.changed_source_files.Count -ne 10){throw 'Source scope count mismatch'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r482-source-scope.json') -Value $scope
$qualified=@($binding.bindings|ForEach-Object{[string]$_.mode+':'+[string]$_.map_id})
$remainingRows=@($binding.catalog40|Where-Object{([string]$_.mode+':'+[string]$_.map_id) -notin $qualified})
if($qualified.Count -ne 22 -or $remainingRows.Count -ne 18){throw 'Catalog identity accounting'}
$remaining=@('Qualify18 remaining exact whole Native maps; current22 have CPU evidence, previously reviewed13 plus9 newly submitted this round','Implement and qualify separate bounded visible-row terrain API for complete534/535 after retained memory/cancel measurements; do not raise existing total cap blindly','Investigate506 Native Tile198/199 required block22/23 from direct decoder/blend evidence','Investigate573/563 Layer7 source variants without inferring corruption','Visible GUI/DPI/focus/select/load/pan/Back/close/network handoff acceptance','Original mode/menu flag producer, thumb drag/track/hold repeat where proven','Native spawn/network/mode rules, actors/effects/roof/loading admission; static previews do not grant complete mode')
$negative=@()
foreach($id in @(565,569,519,520,553,547,540,510,528)){
  $negative+=@(Get-ChildItem -LiteralPath (Join-Path $root "missing-native-$id") -Recurse -File|ForEach-Object{Pin $_.FullName})
  foreach($name in @("missing-$id.json","incomplete-$id.json")){$negative+=Pin (Join-Path $root $name)}
  $map=@($binding.bindings|Where-Object{$_.map_id -eq $id})[0]
  $absent=Join-Path $root ("missing-native-$id\$id\"+(Split-Path $map.sectors[-1].pin.path -Leaf))
  if(Test-Path -LiteralPath $absent){throw 'Last required sector unexpectedly exists'}
}
foreach($name in @('wrong-mode-same-map519.json','complete-city-534-draw-budget.json','complete-city-535-draw-budget.json')){$negative+=Pin (Join-Path $root $name)}
if($negative.Count -ne 90){throw 'Expected69 physical missing-fixture copies plus21 intentional invalid-input JSON files'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r482-negative-fixture-pins.json') -Value ([ordered]@{state='R482_NINE_WHOLE_MAP_MISSING_INCOMPLETE_WRONG_MODE_AND_TWO_FULL_CITY_BUDGET_INPUTS_RETAINED';files=$negative;physical_sector_copies=69;fixture_JSON_files=21;missing_last_required_sector_files_absent=$true;failure_cases=$details.failure_cases;intentional_invalid_input_JSON_not_valid_source_authority=$true})
$history=[ordered]@{
  resource_preparer_attempts=1;fixed_R481_probe_compiles=1;fixed_probe_executions=1;whole_map_preview_qualification_PASS=9;whole_city_measurement_only=2;failed_probe_records=0
  city_console_label_caveat='Generic per-map PASS in probe stdout for534/535 means measurement completed only. JSON measurement_only/state and actual adapter budget refusal are authoritative; neither city granted preview.'
  host_compile_attempts=1;host_compile_PASS=1;focused_executions=1;focused_PASS=1;focused_failures=0;freeze_and_focused_final_exit_code=0;exit_code_evidence='Resumed existing unified exec session32100 returned exit_code0; no rerun'
  focused_seconds=270.20;negative_cases=122;stale_results_rejected=18;background_queue_busy_retries=$details.background_queue_transient_busy_retries;historical_PASS_reruns=0
  production_library_builds=1;production_library_seconds=49.18;player_compiles=1;player_compile_PASS=1;player_executed_flags=@('--version');GUI_executions=0
  logs=@(foreach($n in @('r482-probe-build.stdout.log','r482-probe-build.stderr.log','r482-probe.stdout.log','r482-probe.stderr.log','r482a-host-build.jsonl','r482a-host-build.stderr.log','r482a-consumer.stdout.log','r482a-consumer.stderr.log','r482a-library-build.jsonl','r482a-library-build.stderr.log','r482-player-build.stdout.log','r482-player-build.stderr.log','r482-player-version.stdout.log','r482-player-version.stderr.log')){Pin (Join-Path $logs $n)})
}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r482-execution-history.json') -Value $history
$review=[ordered]@{previously_supervisor_reviewed_catalog_previews=13;newly_submitted_complete_map_evidence=9;current_candidate_CPU_evidenced_catalog_previews=22;current_round_supervisor_acceptance='PENDING';remaining_unqualified_catalog_maps=18}
$gui=[ordered]@{state='R482_GUI_PENDING_CPU_FLOW_ONLY';GUI=0;LOADED=$false;pending=@('Visible22row availability and18explicit refusals across modes; nine new maps selection/load/pan/Back/close','Actual DPI/font/focus/capture/menu and network handoff after worker retirement');Modern_adaptations=@('Canonical immutable sector anchor and signed whole projected bounding camera; absent sector diamonds background, not native spawn/collision','Canonical sector/source insertion order for equal cell/depth; original streaming insertion history unobserved','Modern controls/font/keys/Back and explicit ordinary filter/mode producer policy unchanged','Thumb display only; native drag/hold-repeat/menu flag producer not granted');remaining_required_player_flow=$remaining}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r482-unverified-gui-checklist.json') -Value $gui
$tools=@(foreach($n in @('prepare-r482-resources.py','r482-cross-mode-probe.rs','build-r482-map-probe.ps1','finalize-r482-binding.py','prepare-r482-candidate.py','freeze-r482-study.ps1','audit-r482-source-scope.ps1','build-r482-player-candidate.ps1','start-r482-player-candidate.ps1','verify-r482-cross-mode.py','seal-r482-handoff.ps1','evidence-json.ps1')){Pin (Join-Path $PSScriptRoot $n)})
$evidence=@(foreach($n in @('r482-resource-input.json','r482-probe-candidate.json','r482-map-binding.json','r482-large-map-measurement-design.json','r482-entry-source-contract.json','r482-player-candidate.json','r482a-candidate.json','r482-candidate.json','r482-cross-mode-details.json','r482-source-scope.json','r482-negative-fixture-pins.json','r482-independent-contract.json','r482-execution-history.json','r482-unverified-gui-checklist.json')){Pin (Join-Path $logs $n)})
$previous=Join-Path $logs 'r481-six-complete-multisector-batch-handoff.json'
if((Get-FileHash -LiteralPath $previous).Hash -ne 'D812A428F1C7568AEFF891089A24B94D0C1F8A27F12D7483FBD940BCA275D4B7'){throw 'R481 historical handoff drift'}
$handoff=[ordered]@{
 state='R482_NINE_COMPLETE_CROSS_MODE_MAPS78SECTORS_22CPU_EVIDENCED_PREVIEWS_NEW9_SUPERVISOR_REVIEW_PENDING_GUI_PENDING'
 previous_R481_handoff=Pin $previous;player=$player.executable;production_library=$player.production_library;candidate_main=$player.candidate_main;original_main=$player.original_main;map_binding=$player.binding;skin_binding=$player.skin_binding
 focused_host=[ordered]@{path=$candidate.path;bytes=$candidate.bytes;sha256=$candidate.sha256}
 review_status=$review
 implemented=[ordered]@{catalog_rows=40;qualified_catalog_previews=22;remaining_unqualified_catalog_maps=18;new_identities=$details.new_identities;new_whole_maps=@('mode0:565 four sectors','mode0:569 four sectors','mode2:519 eight sectors','mode2:520 eight sectors','mode2:553 fifteen sectors','mode3:547 twelve sectors','mode4:540 thirteen sectors','mode5:510 nine sectors','mode5:528 five sectors');new_complete_source_sectors=78;all_input_sectors_including_measured_cities=119;source_qualification='All exact original directory/catalog sectors, original and owned byte pins, all required Native tile slots, whole canonical geometry and source bounds; no sector crop or format skipping';shared_R480_adapter_renderer_flow_runtime_unchanged=$true;no_decoder_or_budget_expansion=$true;largest_new_draw_case=547;largest_new_draw_references=213271;largest_new_decoded_bytes=31198464;source_identical_519_520_pixels_equal_identity_and_canonical_distinct=$true;original_catalog_same_mapID_across_modes=$false;wrong_mode_same_ID_refused=$true;actual_active_load_Back_cancel_cases=9;queued_load_and_pan_stale_results_rejected=18;actual_worker_retired=$true;missing_sector_cases=9;full_city_runtime_budget_refusal_cases=2}
 tests=$details;CPU_images=$details.images;fixed_R481_library_independent_reference=Pin (Join-Path $root 'reference\reference.json');independent_contract=Pin (Join-Path $logs 'r482-independent-contract.json')
 full_city_measurements=$proof.full_city_measurements;large_map_minimal_design=Pin (Join-Path $logs 'r482-large-map-measurement-design.json');large_map_design_implemented=$false;cities_preview_granted=$false;remaining_actual_catalog_rows=$remainingRows
 source_scope=[ordered]@{source_files=597;baseline=479;unchanged_baseline=469;allowed_changed_baseline=10;new_sources=118;snapshot_pairs=149;lib_test_source_pairs=140;prior_candidate_main_references=8;separate_R482_main=1;R481_current_source_changes=@('mod.rs');historical_candidates_sources_snapshots_preserved=$true;tools=$tools}
 execution_history=$history;evidence=$evidence;unverified_GUI=$gui;remaining_required_player_flow=$remaining
 known_unsupported506=@(22,23);known_Layer7_gaps=@('573 Layer7 truncated170638 need109 available31','563 Layer7 truncated136068 need136 available18');new_batch_unsupported_types=@();corruption_inferred=$false
 complete_modes=16;total_modes=40;GUI=0;LOADED=$false;root_main_written=$false;original_BIN_or_resources_written=$false;Java_SQL_Server_used=$false;provider_or_observer_used=$false;R472_timers_advanced=$false;full_original_UI_or_mode_acceptance=$false
}
$path=Join-Path $logs 'r482-nine-cross-mode-complete-map-handoff.json'
Write-ModernEvidenceJson -Path $path -Value $handoff
$textPath=Join-Path $project 'Native-R482_九張跨模式完整場地預覽交接.txt'
$text=@"
R482 九張跨模式完整場地預覽交接（CPU 證據完成，本輪監製審查／GUI 待驗）

候選：$($player.executable.path)
SHA256：$($player.executable.sha256)
啟動：$PSScriptRoot\start-r482-player-candidate.ps1
使用 -OfflinePreview 可直接開離線選單。此次候選僅 Hidden --version，未執行 GUI。

新增 mode0:565/569；mode2:519/520/553；mode3:547；mode4:540；mode5:510/528，共9張整圖78個完整原版sector。
全目錄、原表、原檔與owned bytes/SHA、全必要tile、canonical geometry/bounds與資源上限已核實，未裁圖或跳格式。
共用R480 adapter/renderer/flow/runtime保持byte不變；597 source（469 baseline unchanged、10 allowed changed、118 new），149固定current/source pairs（140lib/test、8舊candidate main參考、1新main）。

固定consumer最終退出碼0，1PASS／0failed，270.20秒；45張實際CPU圖逐像素符合固定R481 library獨立參考。
122負向／邊界檢查；18個queued舊load/pan拒絕，9次實際active-load Back取消、held frame撤銷與最後worker退休。
519/520原sector內容及5組畫面相同，但map身份/canonical不同；切換撤銷舊frame。原表無跨mode同mapID；錯mode同ID明確拒絕。
每張圖最後必要sector真實缺失／plan不完整均拒絕，保留正確selection且沒有舊圖回退。queue busy重試實際0。

534完整16sector：347,528 draw refs、32,601,600 decoded bytes；535完整25sector：558,061 refs、29,475,072 decoded bytes。
全geometry/必要tile驗證成功，但兩者超過現有262,144 total draw cap，adapter及實際runtime明確拒絕，沒有partial frame，未列入預覽合格。
probe stdout通用PASS僅代表上述量測完成；JSON measurement_only與budget refusal決定資格。
最小設計已記錄：保留整圖來源驗證，以可取消terrain iterator／row-band index生成有界可見draw refs，替換adapter與Scene的全圖重複Vec邊界；先量retained memory/cancel，再定source bound。目前僅設計，未實作、未提高限制。

先前已審13張；本輪新增9張CPU完整證據待監製審查，當前候選22/40預覽有CPU證據、剩18張。完整模式仍16/40、GUI0、LOADEDfalse。
506必要tile type22/23與573/563 Layer7缺口維持；native spawn/network/mode rules/actors/effects/roof/admission、原版producer/拖動長按與可見GUI待驗。
root main及歷史候選／snapshots未改；未啟Java/SQL/server/provider/原版BIN，不寫共用原版資源。
完整交接：$path
"@
[IO.File]::WriteAllText($textPath,$text,[Text.UTF8Encoding]::new($false))
$progressPath=Join-Path (Split-Path -Parent $project) '監製進度.json'
$progress=Get-Content -LiteralPath $progressPath -Raw|ConvertFrom-Json
$progress.updated_at=[DateTimeOffset]::Now.ToString('o')
$progress.phase='R482_nine_complete_cross_mode_maps78sectors_22CPU_evidenced_previews_new9_review_pending_GUI_pending'
$progress.current_task='R4829new complete cross-mode maps78sectors;1focused270.20sPASS/exit0/45actualCPUreferencepairs/122negative/18stale/149sourcepairs. Previously reviewed13; current22CPU-evidenced with new9 supervisor review pending;18remaining. Whole534/535 measured347528/558061refs actual draw-cap refusal; bounded-row design only. GUI0/16of40/LOADEDfalse.'
$progress|Add-Member -NotePropertyName native_R482_handoff -NotePropertyValue (Pin $path) -Force
$progress|Add-Member -NotePropertyName native_R482_candidate -NotePropertyValue $player.executable -Force
$progress|Add-Member -NotePropertyName native_R482_review_status -NotePropertyValue $review -Force
$progress|Add-Member -NotePropertyName native_R482_remaining_player_flow -NotePropertyValue $remaining -Force
Write-ModernEvidenceJson -Path $progressPath -Value $progress
Write-ModernEvidenceJson -Path (Join-Path $logs 'r482-final-audit.json') -Value ([ordered]@{state='R482_SOURCE_PLAYER_FULL_MAP_IDENTITY_BUDGET_PIXEL_FAILURE_HANDOFF_PROGRESS_SEALED';handoff=Pin $path;text=Pin $textPath;progress=Pin $progressPath;source_pairs=149;review_status=$review;new_scope_focused_PASS=1;focused_exit_code=0;GUI=0;LOADED=$false})
Write-Output (ConvertTo-Json (Pin $path) -Compress)
