$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Read($name){Get-Content -LiteralPath (Join-Path $logs $name) -Raw|ConvertFrom-Json}
$checked=0
function Check($v){
 if($null -eq $v -or $v -is [string] -or $v -is [ValueType]){return}
 if($v -is [array]){foreach($item in $v){Check $item};return}
 $names=@($v.PSObject.Properties.Name)
 if('path' -in $names -and 'bytes' -in $names -and 'sha256' -in $names){$actual=Pin $v.path;if($actual.bytes -ne $v.bytes -or $actual.sha256 -ne $v.sha256){throw ('Evidence pin drift: '+$v.path)};$script:checked++}
 foreach($prop in $v.PSObject.Properties){Check $prop.Value}
}
$names=@('r476-native-menu-skin-binding.json','r476-img-namespace-source-contract.json','r476-entry-source-contract.json','r476-menu-skin-details.json','r476a-coordinate-failure-preservation.json','r476-independent-contract.json','r476-player-candidate.json')
foreach($name in $names){Check (Read $name)}
$candidate=Read 'r476-candidate.json';Check $candidate
if($candidate.source_pins.Count -ne 120){throw 'Expected 120 explicit source snapshots'}
foreach($source in $candidate.source_pins){if((Get-FileHash -LiteralPath $source.snapshot).Hash -ne $source.sha256){throw 'Snapshot drift'};$script:checked++}
$stdout=Get-Content -LiteralPath (Join-Path $logs 'r476b-consumer.stdout.log') -Raw
if($stdout -notmatch '1 passed; 0 failed; 0 ignored;'){throw 'New focused actual PASS required'}
$candidate.validation='PASS one new focused real LTsurf/member/pixel/state/resize/worker test; one earlier coordinate overflow failure retained; zero R475 PASS reruns'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r476-candidate.json') -Value $candidate
$scope=Read 'source-change-scope.json'
if($scope.source_files -ne 575 -or $scope.new_source_files.Count -ne 96 -or $scope.changed_source_files.Count -ne 10){throw 'Source scope count drift'}
$player=Read 'r476-player-candidate.json';$details=Read 'r476-menu-skin-details.json'
$r408=Join-Path $project 'target\r408-source-selector\i686-pc-windows-msvc\debug\LiToModern.exe'
if((Get-FileHash -LiteralPath $r408).Hash -ne '58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'){throw 'R408 main changed'}
$old=Join-Path $logs 'r475-player-entry-async-ui-handoff.json'
if((Get-FileHash -LiteralPath $old).Hash -ne '76F43FD4600271318778A21189F2D00E6F7D0A56E184265453CEF8053AC16E11'){throw 'R475 historical handoff changed'}
$history=[ordered]@{host_compile_attempts=2;host_compile_success=2;focused_executions=2;focused_failures=1;focused_pass=1;focused_final_seconds=23.84;failure='First actual CPU test exposed Layout right/bottom extent overflow; repaired checked extents in separate final r476b host';failure_preservation=Pin (Join-Path $logs 'r476a-coordinate-failure-preservation.json');library_builds=1;player_builds=1;player_build_success=1;executed_player_flags=@('--version');player_version_exit_code=0;GUI_executions=0;R475_PASS_reruns=0;namespace_discovery='One failed lookup for undeclared42EC90/42EC80; retained precise initializer post-endp loc42EEEE fragment without invented proc authority'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r476-execution-history.json') -Value $history
$gui=[ordered]@{state='GUI_PENDING_CPU_EVIDENCE_ONLY';GUI=0;native_GUI_executed=$false;LOADED=$false;pending=@('Visible panel/arrow crop and Modern row text alignment in actual candidate window','Letterbox/resize/move/100% and200% DPI hit coordinates on actual display','Focus/keyboard navigation/one-row selection/disabled arrows','Loading/back clears panel/frame; late completion does not restore retired scene','Network choice retires worker then existing profile/login path','Window close during skin init/native load and source failure behavior');native_parity_gaps=@('Native glyph/font and exact language/shard runtime priority unobserved','Native full UI/room/network/map selection callbacks are not reenacted','Arrow repeat timer and multirow scroll excluded; only one qualified mode1/map564 row','RGB555 reference verified; native device555->565 conversion branch not admitted')}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r476-unverified-gui-checklist.json') -Value $gui
$tools=@(foreach($name in @('prepare-r476-skin.py','prepare-r476-candidate.py','build-r476-player-candidate.ps1','start-r476-player-candidate.ps1','verify-r476-menu-evidence.py','seal-r476-handoff.ps1','freeze-finite-appearance-study.ps1','audit-continuation-source-scope.ps1','evidence-json.ps1')){Pin (Join-Path $PSScriptRoot $name)})
$evidence=@(foreach($name in ($names+@('r476-candidate.json','r476-execution-history.json','r476-unverified-gui-checklist.json','source-change-scope.json'))){Pin (Join-Path $logs $name)})
$handoff=[ordered]@{state='R476_SEPARATE_EXISTING_MAIN_CANDIDATE_ORIGINAL_LTSURF_MAP_MENU_SUBSET_BUILT_CPU_VERIFIED_GUI_PENDING';previous_R475_handoff=Pin $old;player=$player.executable;production_library=$player.production_library;original_main=$player.original_main;candidate_main=$player.candidate_main;mode_map_binding=$player.binding;skin_binding=$player.skin_binding;selected_subset=[ordered]@{panel_id=6113;member_dimensions=@(158,82);native_crop=@(0,0,157,81);native_destination=@(424,310);up_normal_pressed=@(6232,6233);up_dimensions=@(11,12);up_hit_half_open=@(566,313,577,325);down_normal_pressed=@(6234,6235);down_hit_half_open=@(565,375,576,387);arrow_mask_word=6342;palette='No palette: signed-dimension IMG header8 and exact uncompressed little-endian RGB555 pixels';native_state='414BD0 sets button+2Dh1 on press;414C00 clears on release;4E9A40 reads+2Dh and uses resourceId+1;414A80 half-open bounds and enabled14h';native_panel='46B7D0 ->474280/vtable4F9058 ->4744F0 table834BD4 byte5F84h=6113 ->opaque4B9F60 source at0,0 ->157x81 crop at424,310';native_namespace='4D4D90 decimal IMG filename ->43FFC0 optional shard ->42EA50 baseSurf64798C tries LTsurf6479A0 first; exact post-endp initializer retained';candidate_behavior='Original panel and normal arrow pixels on menu; one qualified Battle Ground564 row means arrows disabled. Modern row label/focus/enter/back/close/network remain explicit adaptations; pressed variants CPU only';worker='Optional R476 skin pins/read/IMG decode/three BGRA composites owned by same background worker; UI paints retained buffer only. Root default start and R475 window remain unchanged'};tests=[ordered]@{new_focused_actual_tests=1;member_pixel_comparisons=5;independent_composite_all_pixel_comparisons=3;negative_cases=$details.negative_cases;resize_hit_pixel_checks=$details.resize_hit_pixel_checks;actual_transparent_arrow_pixels=$details.actual_arrow_transparent_pixels;skin_and_native564_enter_back_close_executed=$true;skin_bad_pin_refused_before_choice=$true;independent_main_inverse_byte_exact=$true;PE_machine=332;PE_subsystem=2;directly_viewed_CPU_images=3};CPU_images=$details.images;history=$history;source_scope=[ordered]@{files=575;baseline=479;unchanged=469;allowed_changed=10;new_sources=96;snapshot_pairs=120;compiled_lib_test_source_snapshots=118;separate_R476_candidate_main=1;inherited_R475_main_reference_not_compiled_in_R476=1;tools=$tools};evidence=$evidence;frozen_R408=Pin $r408;unverified_GUI=$gui;complete_modes=16;total_modes=40;GUI=0;LOADED=$false;original_BIN_or_resources_written=$false;root_main_written=$false;Java_SQL_server_used=$false;R472_timers_advanced=$false;native_resource_resolution_or_full_UI_parity_granted=$false}
$handoffPath=Join-Path $logs 'r476-original-ltsurf-map-menu-handoff.json'
Write-ModernEvidenceJson -Path $handoffPath -Value $handoff
$txtPath=Join-Path $project 'Native-R476_原版場地面板候選交接.txt'
$text=@"
R476 原版 LTsurf 場地面板候選（GUI 待驗）

候選：$($player.executable.path)
SHA256：$($player.executable.sha256)
啟動工具：$PSScriptRoot\start-r476-player-candidate.ps1
加 -OfflinePreview 直接到場地選單；預設為 Modern 玩家入口。

原版 panel6113：158x82，依4744F0來源裁出157x81。
上箭頭6232/6233，下箭頭6234/6235：11x12，透明色6342。
面板和正常箭頭已接入獨立 R476 視窗；唯一已驗證場地 Battle Ground564，捲動箭頭停用。
行文字、進入、返回、退出、連線選擇保留 Modern 操作；按下圖像僅 CPU 驗證。

新 focused PASS 1，23.84秒；16拒絕案例；$($details.resize_hit_pixel_checks)個縮放命中像素。
五張原版資源與獨立 Python 參考全部像素相同；三張合成圖獨立逐像素比對。
背景 worker 實際初始化 skin、地圖564進場、返回與退出；壞 skin pin 拒絕。
首次測試的矩形溢位缺口已修正，失敗 host/source/log 原樣保留。

原生 GUI 未操作，字型/語系/完整原版 UI 相等性未宣告；16/40、GUI0、LOADEDfalse。
原 main、R408、原 BIN與原資源保持；未使用 Java/SQL/server 或推進 R472 timer。
證據：$handoffPath
"@
[IO.File]::WriteAllText($txtPath,$text,[Text.UTF8Encoding]::new($false))
$progressPath=Join-Path (Split-Path -Parent $project) '監製進度.json'
$progress=Get-Content -LiteralPath $progressPath -Raw|ConvertFrom-Json
$progress.updated_at=[DateTimeOffset]::Now.ToString('o');$progress.phase='R476_original_LTsurf_map_menu_subset_separate_candidate_CPU_verified_GUI_pending';$progress.current_task='R476 qualified6113panel/6232-6235arrows IMGdecoder+optionalworker/newwindow candidate built;1newfocusedPASS/16negative/source120pairs;GUIpending,16of40/LOADEDfalse'
$progress|Add-Member -NotePropertyName native_R476_handoff -NotePropertyValue (Pin $handoffPath) -Force
$progress|Add-Member -NotePropertyName native_R476_candidate -NotePropertyValue $player.executable -Force
Write-ModernEvidenceJson -Path $progressPath -Value $progress
Write-ModernEvidenceJson -Path (Join-Path $logs 'r476-final-audit.json') -Value ([ordered]@{state='ALL_SELECTED_R476_SOURCE_RESOURCE_BUILD_EXE_CPU_HANDOFF_PINS_RECHECKED';checked_pin_instances=$checked;source_snapshot_pairs=120;tool_count=9;handoff=Pin $handoffPath;handoff_text=Pin $txtPath;progress=Pin $progressPath;R408=Pin $r408;original_main=$player.original_main;GUI=0;LOADED=$false})
Write-Output (ConvertTo-Json (Pin $handoffPath) -Compress)
