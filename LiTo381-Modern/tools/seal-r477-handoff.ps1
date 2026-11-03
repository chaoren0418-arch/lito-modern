$ErrorActionPreference='Stop';$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Read($name){Get-Content -LiteralPath (Join-Path $logs $name) -Raw|ConvertFrom-Json}
$checked=0
function Check($v){if($null -eq $v -or $v -is [string] -or $v -is [ValueType]){return};if($v -is [array]){foreach($i in $v){Check $i};return};$n=@($v.PSObject.Properties.Name);if('path' -in $n -and 'bytes' -in $n -and 'sha256' -in $n){$a=Pin $v.path;if($a.bytes -ne $v.bytes -or $a.sha256 -ne $v.sha256){throw ('Pin drift: '+$v.path)};$script:checked++};foreach($prop in $v.PSObject.Properties){Check $prop.Value}}
$names=@('r477-resource-probe-input.json','r477-probe-candidate.json','r477-native514-probe.json','r477-multi-map-binding.json','r477-menu-source-contract.json','r477-entry-source-contract.json','r477-multi-map-details.json','r477a-compile-failure-preservation.json','r477b-unexecuted-layout-refinement.json','r477c-unexecuted-focus-refinement.json','r477-player-candidate.json','r477-independent-contract.json','r477-sealer-array-failure-preservation.json')
foreach($n in $names){Check (Read $n)}
$candidate=Read 'r477-candidate.json';Check $candidate;if($candidate.source_pins.Count -ne 126){throw '126 exact source pairs required'}
foreach($v in $candidate.source_pins){if((Get-FileHash -LiteralPath $v.snapshot).Hash -ne $v.sha256){throw 'Snapshot drift'};$script:checked++}
$pass=Get-Content -LiteralPath (Join-Path $logs 'r477d-consumer.stdout.log') -Raw;if($pass -notmatch '1 passed; 0 failed; 0 ignored;'){throw 'Actual one new focused PASS required'}
$candidate.validation='PASS one new focused Native564/514 multirow/cancel/identity/scene test; old PASS reruns0; all compile/refinement history retained'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r477-candidate.json') -Value $candidate
$scope=Read 'source-change-scope.json';if($scope.source_files -ne 580 -or $scope.new_source_files.Count -ne 101 -or $scope.changed_source_files.Count -ne 10){throw 'Source scope drift'}
$player=Read 'r477-player-candidate.json';$details=Read 'r477-multi-map-details.json';$binding=Read 'r477-multi-map-binding.json'
$previous=Join-Path $logs 'r476-original-ltsurf-map-menu-handoff.json';if((Get-FileHash -LiteralPath $previous).Hash -ne '0FA4F5C838CAFE02AC4E8F507F103F293820D2834110AA0C389164FA18F6A206'){throw 'Historical R476 handoff drift'}
$r408=Join-Path $project 'target\r408-source-selector\i686-pc-windows-msvc\debug\LiToModern.exe';if((Get-FileHash -LiteralPath $r408).Hash -ne '58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'){throw 'R408 changed'}
$history=[ordered]@{probe_compile=1;probe_execution=1;probe_PASS=1;probe_used_fixed_R476_library=(Read 'r477-probe-candidate.json').fixed_R476_library;host_compile_attempts=4;host_compile_failed=1;host_compile_passed=3;host_unexecuted_refinements=2;focused_executions=1;focused_PASS=1;focused_failures=0;focused_seconds=58.94;historical_PASS_reruns=0;library_builds=1;player_compile=1;player_compile_pass=1;player_executed_flags=@('--version');readonly_player_exit_code=0;GUI_executions=0;sealer_failed_attempts=1;history_evidence=@(foreach($n in @('r477a-compile-failure-preservation.json','r477b-unexecuted-layout-refinement.json','r477c-unexecuted-focus-refinement.json','r477-sealer-array-failure-preservation.json')){Pin (Join-Path $logs $n)})}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r477-execution-history.json') -Value $history
$gui=[ordered]@{state='NATIVE_GUI_PENDING_SOURCE_AND_CPU_PROOF_ONLY';GUI=0;LOADED=$false;pending=@('Actual multirow font scale/clipping/selection/arrow press appearance at100% and200% DPI','Actual row/mouse capture/keyboard focus and focused Close/Back Enter routing','Actual resize/letterbox/move and small-window clipping','Selecting while loading, Back/reentry, pan and old scene clear behavior in visible window','Close during init/load and network handoff to existing profile/login');unimplemented=@('Original thumb6231.img7x10 at x143/y15+floor41top/(count-1) is source-recorded but not wired','Original repeat timer/hold-repeat excluded; press-release only','Native glyph/font/color/language/shard runtime parity unobserved','Original modefilter/header entries/network map callbacks not reenacted; R477 explicitly uses cross-mode catalog40','38 catalog rows lack current qualified complete single-sector preview recipe; no Shared or564 fallback','Full renderer/actors/effects/spawn/network mode admission unchanged')}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r477-unverified-gui-checklist.json') -Value $gui
$tools=@(foreach($n in @('prepare-r477-resources.py','r477-native-map-probe.rs','build-r477-map-probe.ps1','prepare-r477-candidate.py','build-r477-player-candidate.ps1','start-r477-player-candidate.ps1','verify-r477-multi-map.py','seal-r477-handoff.ps1','freeze-finite-appearance-study.ps1','audit-continuation-source-scope.ps1','evidence-json.ps1')){Pin (Join-Path $PSScriptRoot $n)})
$evidence=@(foreach($n in ($names+@('r477-candidate.json','r477-execution-history.json','r477-unverified-gui-checklist.json','source-change-scope.json'))){Pin (Join-Path $logs $n)})
$handoff=[ordered]@{state='R477_SEPARATE_EXISTING_MAIN_NATIVE_MULTI_MAP_MENU_CANDIDATE_BUILT_TWO_TRUE_RESOURCE_SCENES_CPU_VERIFIED_GUI_PENDING';previous_R476_handoff=Pin $previous;player=$player.executable;production_library=$player.production_library;candidate_main=$player.candidate_main;original_main=$player.original_main;binding=$player.binding;skin_binding=$player.skin_binding;resource_plans=$binding.bindings;catalog40_source=$binding.catalog_source;additional_original_sector=$binding.additional_sector_origin;additional_canonical_digest=$binding.bindings[1].canonical_sha256;additional_reference_probe=Pin (Join-Path $logs 'r477-native514-probe.json');implemented=[ordered]@{catalog_rows=40;qualified_preview_identities=@(@(1,564),@(1,514));unqualified_rows=38;menu_native_source='46B7D0/4645D0 rowpitch12/type7;4646C0 six visible rows;474350 top-1>=0 /474360 top+1<count;414A80/414BD0/414C00 press/move/release half-open guards;476 panel and6232-35 arrows reused';native_top_upper_bound='count-1, independent of visible6';native_press_repeat='One accepted release advances one row; repeat timer excluded';Modern_UI='Exact original mode/map/name records across40 rows; available/unqualified marker. Scaled Segoe UI in ceil-projected per-row clip/hit rectangle, ellipsis and Modern highlight; Up/Down/Home/End/PageUp/PageDown selection and focused-parent Enter only';selection='First qualified catalog row initially; Back preserves selected mode/map and top; unavailable row can be selected but Enter refuses explicitly';worker='Each Enter carries selected exact mode/map and generation. Select cancels old load/pan, revokes old frame, resets Menu; old generated or wrong-map frame rejected. Missing selected map fails visibly with no564 fallback. Owned worker retires before close/network handoff';static_scope='Two complete single Native sectors and same four exact ordered Native LTtile/LTsprite archive pins. Common Modern camera1136,468,800x600 only, not native spawn.'};tests=[ordered]@{new_focused=1;PASS=1;negative_cases=$details.negative_cases;coordinate_checks=$details.coordinate_checks;actual_queued_old_map_and_pan_rejected=$details.actual_old_map_queued_frame_and_old_pan_discarded;actual_select_during_load_cancel=$true;actual_missing514_no564_fallback=$true;Back_selection_retained=$true;source_main_inverse_byte_exact=$true;PE_machine=332;PE_subsystem=2;scene_pixel_pairs=2;Temple_pixels='Matches separately built fixed R476 library actualNative514 probe';Battle_pixels='Matches fixed R474 source reference';directly_viewed_CPU_scenes=2};CPU_images=$details.images;execution_history=$history;source_scope=[ordered]@{source_files=580;baseline=479;unchanged=469;allowed_changed=10;new_sources=101;snapshot_pairs=126;lib_test_source_snapshots=123;separate_R477_main=1;inherited_R475_R476_main_reference_not_compiled_in_R477=2;tools=$tools};evidence=$evidence;frozen_R408=Pin $r408;unverified_GUI=$gui;complete_modes=16;total_modes=40;GUI=0;LOADED=$false;original_BIN_or_resources_written=$false;root_main_written=$false;Java_SQL_Server_used=$false;provider_or_observer_used=$false;R472_timers_advanced=$false;full_original_UI_or_mode_acceptance=$false}
$handoffPath=Join-Path $logs 'r477-native-multi-map-menu-handoff.json'
Write-ModernEvidenceJson -Path $handoffPath -Value $handoff
$textPath=Join-Path $project 'Native-R477_多場地選單候選交接.txt'
$text=@"
R477 多場地選單候選（原生 GUI 待驗）

候選：$($player.executable.path)
SHA256：$($player.executable.sha256)
啟動：$PSScriptRoot\start-r477-player-candidate.ps1
加 -OfflinePreview 直接到場地選單；預設先顯示 Modern 玩家入口。

原表40個 mode/map 逐筆保留名稱與來源身份。
可預覽：mode1/map564 Battle Ground、mode1/map514 Temple。
另外38列標示「待」，不合格無法進入，缺資源不會改載564。
原版面板與正常／按下箭頭接入；每次捲動一列，top上限count-1，最多可見6列。
Modern字體隨面板倍率縮放，文字裁切和命中共用每列矩形；選取高亮與鍵盤為Modern行為。
點選或方向/Home/End/Page鍵切換；返回保留選取。切換時取消舊load/pan，舊map結果不能復活。

新測試58.94秒1PASS、16拒絕案例、$($details.coordinate_checks)個列命中像素。
兩場地真實資源載入／返回／重入／取消／缺資源已執行，兩張CPU圖逐像素符合固定參考。
原 main 可逐byte還原，126來源快照相同；編譯失敗與兩次未執行修正host保留。

原生GUI、實際DPI/焦點仍待驗；原版字體/語系/滑塊6231與長按重複未接入或未合格。
16/40、GUI0、LOADEDfalse；預設登入/R408/原資源/原BIN保持，無Java/SQL/server/provider/observer。
完整證據：$handoffPath
"@
[IO.File]::WriteAllText($textPath,$text,[Text.UTF8Encoding]::new($false))
$progressPath=Join-Path (Split-Path -Parent $project) '監製進度.json';$progress=Get-Content -LiteralPath $progressPath -Raw|ConvertFrom-Json
$progress.updated_at=[DateTimeOffset]::Now.ToString('o');$progress.phase='R477_native_catalog40_two_qualified_maps_multiselect_scroll_cancel_candidate_CPU_verified_GUI_pending';$progress.current_task='R477 originalmenu catalog40/Native564+514 truepreview;native12pitch6rows/topcount-1 pressrelease,scaledclippedModernfont;1focused58.94sPASS16negative/126sourcepairs. Oldmap/pan/loadcancel and missing514 nofallback proven;GUI0/16of40/LOADEDfalse.'
$progress|Add-Member -NotePropertyName native_R477_handoff -NotePropertyValue (Pin $handoffPath) -Force
$progress|Add-Member -NotePropertyName native_R477_candidate -NotePropertyValue $player.executable -Force
Write-ModernEvidenceJson -Path $progressPath -Value $progress
Write-ModernEvidenceJson -Path (Join-Path $logs 'r477-final-audit.json') -Value ([ordered]@{state='R477_SELECTED_SOURCE_RESOURCE_PROBE_BUILD_EXE_SCENE_HISTORY_HANDOFF_PINS_RECHECKED';checked_pin_instances=$checked;source_pairs=126;tools=11;handoff=Pin $handoffPath;text=Pin $textPath;progress=Pin $progressPath;R408=Pin $r408;original_main=$player.original_main;GUI=0;LOADED=$false})
Write-Output (ConvertTo-Json (Pin $handoffPath) -Compress)
