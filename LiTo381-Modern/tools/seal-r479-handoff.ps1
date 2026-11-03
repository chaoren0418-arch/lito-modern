$ErrorActionPreference='Stop';$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003';$batchRoot=Join-Path $project 'target\r479-map-batch'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Read($n){Get-Content -LiteralPath (Join-Path $logs $n) -Raw|ConvertFrom-Json}
$checked=0
function Check($v){if($null -eq $v -or $v -is [string] -or $v -is [ValueType]){return};if($v -is [array]){foreach($i in $v){Check $i};return};$n=@($v.PSObject.Properties.Name);if('path' -in $n -and 'bytes' -in $n -and 'sha256' -in $n){$a=Pin $v.path;if($a.bytes -ne $v.bytes -or $a.sha256 -ne $v.sha256){throw ('Pin drift: '+$v.path)};$script:checked++};foreach($prop in $v.PSObject.Properties){Check $prop.Value}}
$names=@('r479-resource-inventory-input.json','r479-native38-inventory.json','r479-probe-candidate.json','r479b-map-binding.json','r479-resource-completeness-audit.json','r479-map-batch-details.json','r479b-entry-source-contract.json','r479-player-candidate.json','r479-independent-contract.json','r479-metadata-refinement.json')
foreach($n in $names){Check (Read $n)}
$probePath=Join-Path $batchRoot 'probe\batch-results.json';$probe=Get-Content -LiteralPath $probePath -Raw|ConvertFrom-Json;Check $probe
if($probe.passed -ne 4 -or $probe.failed -ne 0){throw 'Four complete new Native probes required'}
$candidate=Read 'r479-candidate.json';Check $candidate;if($candidate.source_pins.Count -ne 136){throw '136 source pairs required'}
foreach($p in $candidate.source_pins){if((Get-FileHash -LiteralPath $p.snapshot).Hash -ne $p.sha256){throw 'Snapshot drift'};$checked++}
$pass=Get-Content -LiteralPath (Join-Path $logs 'r479a-consumer.stdout.log') -Raw
if($pass -notmatch '1 passed; 0 failed; 0 ignored;' -or $pass -notmatch 'finished in 69.20s'){throw 'One actual new focused PASS required'}
$candidate.validation='PASS one actual new four-complete-map/player-flow/old-new generation/cancel/missing/corrupt resource focused test; historical PASS reruns0'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r479-candidate.json') -Value $candidate
$scope=Read 'source-change-scope.json';if($scope.source_files -ne 587 -or $scope.new_source_files.Count -ne 108 -or $scope.changed_source_files.Count -ne 10){throw 'Source scope drift'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r479-source-scope.json') -Value $scope
$previous=Join-Path $logs 'r478-native-mode-menu-thumb-handoff.json'
if((Get-FileHash -LiteralPath $previous).Hash -ne 'E4F316DBA83EAACD54B2A94A6109D41805F80387246A47660FEE6991EE740C56'){throw 'Historical R478 handoff drift'}
$r408=Join-Path $project 'target\r408-source-selector\i686-pc-windows-msvc\debug\LiToModern.exe';if((Get-FileHash -LiteralPath $r408).Hash -ne '58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'){throw 'R408 drift'}
$player=Read 'r479-player-candidate.json';$details=Read 'r479-map-batch-details.json';$audit=Read 'r479-resource-completeness-audit.json';$binding=Read 'r479b-map-binding.json'
if($details.negative_cases -ne 13 -or $details.images.Count -ne 4 -or $binding.bindings.Count -ne 6){throw 'New scope mismatch'}
$failures=@(foreach($n in @('missing509.json','missing508.json','missing505.json','missing516.json','bad509-sector-hash.json','bad509-truncated-sector.json','bad509-tile-index.json','bad-native-map\509\80008000.s32','truncated-native-map\509\80008000.s32','bad-native-archive\LTtile.idx')){Pin (Join-Path $batchRoot $n)})
foreach($id in @(509,508,505,516)){if(Test-Path -LiteralPath (Join-Path $batchRoot "missing-native$id\$id\80008000.s32")){throw 'Expected missing-sector fixture unexpectedly exists'}}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r479-negative-fixture-pins.json') -Value ([ordered]@{state='ACTUAL_NEW_MAP_FAILURE_INPUTS_RETAINED_BYTE_PINNED';files=$failures;failure_cases=$details.failure_cases;missing_sector_fixture_ids=@(509,508,505,516);missing_sector_files_absent=$true;expected_invalid_resource_pins_in_negative_input_JSON_not_treated_as_valid_source_evidence=$true})
$history=[ordered]@{descriptive_metadata_only_refinement=1;focused_source_and_production_library_unchanged_after_PASS=$true;fixed_R478_probe_compiles=1;fixed_R478_probe_executions=1;new_map_probe_PASS=4;new_map_probe_failures=0;readonly_inventory_rows=38;readonly_inventory_sectors=305;inventory_decoder_rejected_rows=2;host_compile_attempts=1;host_compile_PASS=1;host_compile_failures=0;focused_executions=1;focused_PASS=1;focused_failures=0;focused_seconds=69.20;historical_PASS_reruns=0;library_builds=1;player_compile=2;player_compile_PASS=2;player_executed_flags=@('--version');readonly_player_exit_code=0;GUI_executions=0;build_and_execution_logs=@(foreach($n in @('r479a-host-build.jsonl','r479a-host-build.stderr.log','r479a-consumer.stdout.log','r479a-consumer.stderr.log','r479a-library-build.jsonl','r479a-library-build.stderr.log','r479-player-build.stdout.log','r479-player-build.stderr.log','r479b-player-build.stdout.log','r479b-player-build.stderr.log')){Pin (Join-Path $logs $n)})}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r479-execution-history.json') -Value $history
$gui=[ordered]@{state='NATIVE_GUI_PENDING_SOURCE_AND_CPU_ONLY';GUI=0;LOADED=$false;pending=@('Visible menu availability of six exact mode1 maps and34 unqualified rows','Actual selection/Enter/Back/close and mode switch during load/pan for new maps','Actual DPI/font/thumb/focus/capture behavior inherited R478','Existing profile/network handoff after background worker retirement');preserved_R478_source_gaps=@('Original runtime byte64AF9F/mode packet-to-open-menu producer','Thumb drag/track and hold-repeat source chain not qualified');static_scope='Four new complete actual single sectors, all floor/overlay/object slots decoded. Sprite archive pins/index verified; effects/opacity/Layer6 remain retained metadata, no new playback/roof/runtime admission.';remaining_required_player_flow=$audit.next_required_player_flow}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r479-unverified-gui-checklist.json') -Value $gui
$tools=@(foreach($n in @('prepare-r479-resources.py','r479-native-batch-probe.rs','build-r479-map-probe.ps1','finalize-r479-binding.py','prepare-r479-candidate.py','build-r479-player-candidate.ps1','prepare-r479b-candidate.py','build-r479b-player-candidate.ps1','finalize-r479-metadata-variant.ps1','start-r479-player-candidate.ps1','verify-r479-map-batch.py','seal-r479-handoff.ps1','freeze-finite-appearance-study.ps1','audit-continuation-source-scope.ps1','evidence-json.ps1')){Pin (Join-Path $PSScriptRoot $n)})
$evidence=@(foreach($n in ($names+@('r479-candidate.json','r479-source-scope.json','r479-negative-fixture-pins.json','r479-execution-history.json','r479-unverified-gui-checklist.json'))){Pin (Join-Path $logs $n)})
$handoff=[ordered]@{state='R479_EXACT38_INVENTORY_FOUR_ADDITIONAL_COMPLETE_NATIVE_MAPS_SIX_PLAYER_PREVIEWS_SEPARATE_CANDIDATE_CPU_VERIFIED_GUI_PENDING';previous_R478_handoff=Pin $previous;player=$player.executable;production_library=$player.production_library;candidate_main=$player.candidate_main;original_main=$player.original_main;map_binding=$player.binding;skin_binding=$player.skin_binding;focused_host=[ordered]@{path=$candidate.path;bytes=$candidate.bytes;sha256=$candidate.sha256};implemented=[ordered]@{catalog_rows=40;previous_preview_maps=2;qualified_preview_maps=6;new_complete_map_names=@('mode1/map509 Fatality','mode1/map508 Fatality(Trap)','mode1/map505 Crossroad','mode1/map516 Secret Chamber');qualified_preview_identities=$details.qualified_maps;unqualified_catalog_maps=34;native_source_batch='All four remaining mode1 single-sector maps copied byte-exact from original complete native directory/R474 lists, no partial multi-sector recipe. Each exact map ID and sector coordinate bound to own snapshot, canonical geometry digest and four ordered Native LTtile/LTsprite archive pins.';required_decoder_changes='None for four selected complete S32 sectors. Existing Native adapter/resource loader/R478 mode runtime reused. R479 window differs from R478 only revision class/title/timer identifiers.';camera='Existing Modern inspection camera1136,468,800x600 relative to actual sector, no native spawn/permission/actor/mode admission';UI='R478 original LTsurf panel/arrows/thumb and verified header/mode filter subset retained; Modern COMBOBOX/font/keys/Back policy explicit';worker='Select exact filtered mode/map; old564 queued result rejected before new509 enters, all four new images correct/distinct, new516 queued pan rejected on old514 selection; new516 Back cancels load and old564 select/close retires worker. Missing/bad new resources fail visibly retaining selected identity and no fallback frame.'};tests=$details;resource_completeness=[ordered]@{original_rows=38;original_sectors=305;original_sector_name_byte_drift=0;fully_parsed_rows=36;known_decoder_rejections=$audit.decoder_rejected_rows;observed_missing_static_tile_index_member_rows=0;other34_not_decoder_or_player_qualified=$true;remaining_required_player_flow=$audit.next_required_player_flow;no_seam_implementation_needed_for_selected_complete_single_sector_batch=$true};CPU_images=$details.images;independent_fixed_library_probe=Pin $probePath;source_scope=[ordered]@{source_files=587;baseline=479;unchanged=469;allowed_changed=10;new_sources=108;snapshot_pairs=136;lib_test_source_snapshots=130;separate_final_R479b_main=1;inherited_R475_R476_R477_R478_R479a_main_reference_not_compiled_in_final_R479b=5;tools=$tools};execution_history=$history;negative_fixtures=Pin (Join-Path $logs 'r479-negative-fixture-pins.json');evidence=$evidence;frozen_R408=Pin $r408;unverified_GUI=$gui;complete_modes=16;total_modes=40;GUI=0;LOADED=$false;original_BIN_or_resources_written=$false;root_main_written=$false;Java_SQL_Server_used=$false;provider_or_observer_used=$false;R472_timers_advanced=$false;full_original_UI_or_mode_acceptance=$false}
$handoffPath=Join-Path $logs 'r479-six-native-map-preview-handoff.json';Write-ModernEvidenceJson -Path $handoffPath -Value $handoff
$textPath=Join-Path $project 'Native-R479_六場地預覽候選交接.txt'
$text=@"
R479b 六場地靜態預覽候選（修正兩個描述欄位）（GUI 待驗）

候選：$($player.executable.path)
SHA256：$($player.executable.sha256)
啟動：$PSScriptRoot\start-r479-player-candidate.ps1
加 -OfflinePreview 直接進入選單；一般模式入口先選 mode1，再點場地進入。

已合格靜態預覽：564 Battle Ground、514 Temple、509 Fatality、508 Fatality(Trap)、505 Crossroad、516 Secret Chamber。
新增4張全是原版目錄只有一個完整sector的mode1場地，整個source sector與所有靜態圖塊均驗證，沒有只取多sector地圖的一角。
沿用原版Tile/Sprite/UI精確pins、模式篩選及6231滑塊。camera1136,468/800x600、字體鍵盤及返回保留行為是Modern適配。
切換取消舊結果；缺sector／壞內容／壞索引清楚失敗，不會改載564或舊frame。

其餘38原表場地已盤點305個實際sector；36筆能完整解碼，目前索引未觀察到缺靜態Tile member。
573/563的實際Layer7被既有解碼器拒絕，錯誤與來源檔名已保留；不判定原檔損壞。
尚待合格34張全是多sector地圖，後續需完整綁定、camera/邊界與原版格式驗證。

新focused69.20秒1PASS、13拒絕案例、4張玩家流程CPU圖逐像素符合固定R478 library參考、136來源快照核對。
描述欄位已改為34張待合格／初始無選取；原測試來源與library完全相同，入口EXE單獨重建，未重跑focused。
GUI0、LOADEDfalse、完整模式16/40；新增預覽不等於模式完成。原main、R408、原版與既有候選不修改。
原版effects/roof/actor/network/spawn、GUI/DPI/focus、長按／拖動／執行期旗標仍待必要工作。
完整證據：$handoffPath
"@
[IO.File]::WriteAllText($textPath,$text,[Text.UTF8Encoding]::new($false))
$progressPath=Join-Path (Split-Path -Parent $project) '監製進度.json';$progress=Get-Content -LiteralPath $progressPath -Raw|ConvertFrom-Json
$progress.updated_at=[DateTimeOffset]::Now.ToString('o');$progress.phase='R479_exact38_inventory_four_additional_complete_native_maps_six_previews_CPU_verified_GUI_pending';$progress.current_task='R479 inventory38/305sectors;4complete mode1 maps509508505516 added,6qualified34pending.1new focused69.20sPASS13negative/4actualscene-referencepairs/136sources; old-new generation/cancel and missing/corrupt nofallback proven.573563Layer7 parser refusal retained. GUI0/16of40/LOADEDfalse.'
$progress|Add-Member -NotePropertyName native_R479_handoff -NotePropertyValue (Pin $handoffPath) -Force
$progress|Add-Member -NotePropertyName native_R479_candidate -NotePropertyValue $player.executable -Force
$progress|Add-Member -NotePropertyName native_R479_remaining_player_flow -NotePropertyValue $audit.next_required_player_flow -Force
Write-ModernEvidenceJson -Path $progressPath -Value $progress
Write-ModernEvidenceJson -Path (Join-Path $logs 'r479-final-audit.json') -Value ([ordered]@{state='R479_INVENTORY_SOURCE_PROBE_PIXELS_FOCUS_LIBRARY_EXE_FAILURE_HANDOFF_PINS_RECHECKED';checked_pin_instances=$checked;source_pairs=136;tools=$tools.Count;handoff=Pin $handoffPath;text=Pin $textPath;progress=Pin $progressPath;R408=Pin $r408;original_main=$player.original_main;GUI=0;LOADED=$false})
Write-Output (ConvertTo-Json (Pin $handoffPath) -Compress)
