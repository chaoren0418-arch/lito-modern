$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Read($n){Get-Content -LiteralPath (Join-Path $logs $n) -Raw|ConvertFrom-Json}
$checked=0
function Check($v){if($null -eq $v -or $v -is [string] -or $v -is [ValueType]){return};if($v -is [array]){foreach($i in $v){Check $i};return};$n=@($v.PSObject.Properties.Name);if('path' -in $n -and 'bytes' -in $n -and 'sha256' -in $n){$a=Pin $v.path;if($a.bytes -ne $v.bytes -or $a.sha256 -ne $v.sha256){throw ('Pin drift: '+$v.path)};$script:checked++};foreach($prop in $v.PSObject.Properties){Check $prop.Value}}
$names=@('r478-source-fragments.json','r478-native-menu-skin-binding.json','r478-menu-source-contract.json','r478-pixel-references.json','r478-entry-source-contract.json','r478-mode-menu-details.json','r478a-unexecuted-focus-refinement.json','r478-player-candidate.json','r478-independent-contract.json')
foreach($n in $names){Check (Read $n)}
$candidate=Read 'r478-candidate.json';Check $candidate
if($candidate.source_pins.Count -ne 132){throw '132 exact source pairs required'}
foreach($v in $candidate.source_pins){if((Get-FileHash -LiteralPath $v.snapshot).Hash -ne $v.sha256){throw 'Snapshot drift'};$checked++}
$pass=Get-Content -LiteralPath (Join-Path $logs 'r478b-consumer.stdout.log') -Raw
if($pass -notmatch '1 passed; 0 failed; 0 ignored;' -or $pass -notmatch 'finished in 40.82s'){throw 'Actual one new focused PASS required'}
$candidate.validation='PASS one new modefilter/header/thumb/DPI/actual564514 filter-cancel focused test; historical PASS reruns0; unexecuted first host retained'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r478-candidate.json') -Value $candidate
$scope=Read 'source-change-scope.json';if($scope.source_files -ne 585 -or $scope.new_source_files.Count -ne 106 -or $scope.changed_source_files.Count -ne 10){throw 'Source scope drift'}
$previous=Join-Path $logs 'r477-native-multi-map-menu-handoff.json'
if((Get-FileHash -LiteralPath $previous).Hash -ne '00327C39FE4B3DA884C2BDFD1B286AB52E7BF8C7382ABEBB84ABB097F7200E73'){throw 'Historical R477 handoff drift'}
$r408=Join-Path $project 'target\r408-source-selector\i686-pc-windows-msvc\debug\LiToModern.exe'
if((Get-FileHash -LiteralPath $r408).Hash -ne '58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'){throw 'R408 drift'}
$player=Read 'r478-player-candidate.json';$details=Read 'r478-mode-menu-details.json';$contract=Read 'r478-menu-source-contract.json'
$history=[ordered]@{header_probe_compile=1;header_probe_execution=1;header_probe_PASS=1;host_compile_attempts=2;host_compile_failed=0;host_compile_passed=2;first_host_unexecuted_focus_refinement=1;focused_executions=1;focused_PASS=1;focused_failures=0;focused_seconds=40.82;historical_PASS_reruns=0;library_builds=1;player_compile=1;player_compile_PASS=1;player_executed_flags=@('--version');readonly_player_exit_code=0;GUI_executions=0;history_evidence=@(foreach($n in @('r478a-unexecuted-focus-refinement.json','r478a-host-build.jsonl','r478a-host-build.stderr.log','r478b-host-build.jsonl','r478b-host-build.stderr.log','r478b-consumer.stdout.log','r478b-consumer.stderr.log','r478a-library-build.jsonl','r478a-library-build.stderr.log','r478-player-build.stdout.log','r478-player-build.stderr.log')){Pin (Join-Path $logs $n)})}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r478-execution-history.json') -Value $history
$gui=[ordered]@{state='NATIVE_GUI_PENDING_SOURCE_AND_CPU_ONLY';GUI=0;LOADED=$false;pending=@('Mode COMBOBOX selection, own Up/Down/Enter/Esc routing and initially cleared selection focus','Actual100%/200% DPI thumb/panel/font/clip and bottom arrow-overlap appearance','Actual mode change while loading/pan/frame, clear old scene and return/reentry','Actual mouse capture cancellation on mode change/focus loss','Actual close during init/load and network handoff to existing profile/login');unqualified=$contract.gaps;special_entries='Exact Hot Random(-100)/Random(-1) source labels displayed and selectable; offline entry refused, no random map matching';repeat='Press/release only; native hold-repeat timer initialization/cadence/cancellation unqualified';thumb='Source-qualified display and image-rectangle hit only; hit cannot capture/scroll/drag; no native drag or track paging grant';mode_trigger='Native constructor/filter/clear state subset proven. Modern mode selector explicitly rebuilds. Original packet-to-open-menu update is unproved.'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r478-unverified-gui-checklist.json') -Value $gui
$tools=@(foreach($n in @('prepare-r478-resources.py','r478-extract-headers.rs','finalize-r478-resources.py','prepare-r478-pixel-references.py','prepare-r478-candidate.py','build-r478-player-candidate.ps1','start-r478-player-candidate.ps1','verify-r478-mode-menu.py','seal-r478-handoff.ps1','freeze-finite-appearance-study.ps1','audit-continuation-source-scope.ps1','evidence-json.ps1')){Pin (Join-Path $PSScriptRoot $n)})
$evidence=@(foreach($n in ($names+@('r478-candidate.json','r478-execution-history.json','r478-unverified-gui-checklist.json','source-change-scope.json'))){Pin (Join-Path $logs $n)})
$handoff=[ordered]@{state='R478_NATIVE_THUMB_MODE_FILTER_HEADERS_STATE_CLEAR_SEPARATE_CANDIDATE_CPU_VERIFIED_GUI_PENDING';previous_R477_handoff=Pin $previous;player=$player.executable;production_library=$player.production_library;candidate_main=$player.candidate_main;original_main=$player.original_main;map_binding=$player.binding;skin_binding=$player.skin_binding;focused_host=[ordered]@{path=$candidate.path;bytes=$candidate.bytes;sha256=$candidate.sha256};implemented=[ordered]@{native_thumb_id=6231;native_thumb_size=@(7,10);native_thumb_position='x143; count<=1:y15; otherwise y15+floor(41*top/(count-1)); mask copy independent of IMG flag0; thumb before child arrows';thumb_interaction='display-only, source image rectangle hit is inert; no drag/track/repeat';mode_map_counts=$details.mode_map_counts;headers=@([ordered]@{key=-100;index=269;text='Hot Random'},[ordered]@{key=-1;index=108;text='Random'});native_filter='Exact(mode,map). Explicit ordinary byte64AF9F=0; mode3 excludes549/550/even. Alternative source branch separately tested but runtime producer unobserved.';mode_rebuild='New mode or diagnostic view clears selection/frame/old work and top0, rebuilds header-plus-filtered map rows. Same-mode no-op and Back retain selected identity/top as Modern inspection policy. No automatic first-map fallback.';diagnostic_all40='Explicit diagnostic selector; no Native mode admission';qualified_preview_identities=@(@(1,564),@(1,514));unqualified_catalog_maps=38;special_entry_policy='Selectable header entries preserve native key/index/name, Enter refuses offline random matching';worker='Exact visible mode/map before Enter; filter/header/selection invalidate generation and cancel old load/pan, revoke held frame, reject queued old scene. Owned worker retires before close/network handoff.';Modern_UI='Mode COMBOBOX labels, clipped/scaled Segoe UI/highlight/keys, same-mode/Back retention and native-construction rebuild trigger are explicit adaptations. Mode combo keeps own keyboard/Esc.'};tests=$details;execution_history=$history;source_scope=[ordered]@{source_files=585;baseline=479;unchanged=469;allowed_changed=10;new_sources=106;snapshot_pairs=132;lib_test_source_snapshots=128;separate_R478_main=1;inherited_R475_R476_R477_main_reference_not_compiled_in_R478=3;tools=$tools};source_boundary=$contract;evidence=$evidence;frozen_R408=Pin $r408;unverified_GUI=$gui;complete_modes=16;total_modes=40;GUI=0;LOADED=$false;original_BIN_or_resources_written=$false;root_main_written=$false;Java_SQL_Server_used=$false;provider_or_observer_used=$false;R472_timers_advanced=$false;full_original_UI_or_mode_acceptance=$false}
$handoffPath=Join-Path $logs 'r478-native-mode-menu-thumb-handoff.json'
Write-ModernEvidenceJson -Path $handoffPath -Value $handoff
$textPath=Join-Path $project 'Native-R478_模式場地與滑塊候選交接.txt'
$text=@"
R478 模式／場地選單與原版滑塊候選（GUI 待驗）

候選：$($player.executable.path)
SHA256：$($player.executable.sha256)
啟動：$PSScriptRoot\start-r478-player-candidate.ps1
加 -OfflinePreview 直接到模式與場地選單；預設先顯示 Modern 玩家入口。

先選模式，再選該模式的場地；模式0..5原表場地數4/15/9/3/1/8。
各模式先列 Hot Random(-100)、Random(-1)，保留原文但不提供離線隨機配對。
可預覽：mode1/map564 Battle Ground、mode1/map514 Temple。其他38個原表場地仍待合格。
切換模式清除選取與舊畫面、捲動歸零；返回及同模式保留選取與捲動是明示Modern行為。
「診斷 · 全部40場地」保留原表所有身分，與一般模式選單分開。
原版6231.img滑塊7x10已接入，x143/y15+floor41top/(count-1)，thumb先畫、箭頭後畫。
滑塊只展示，點擊不會拖動／捲動。箭頭每次按下放開捲一列，top上限count-1。
Modern模式控制與字體／鍵盤適配明示；原版mode packet觸發、旗標來源、拖動與長按repeat仍未合格。

新focused 40.82秒1PASS、66拒絕案例、4096184次命中檢查、8組獨立合成像素全符。
兩張真Native場地的實際影像hash符合R477固定參考；切模式取消舊load/pan、拒絕舊frame與背景工作退休通過。
獨立EXE、132來源快照與原main逐byte還原已核對。首個host因焦點修正保留，未執行。
GUI0、LOADEDfalse、完整模式16/40；原main／R408／原版共享資源不修改。
完整證據：$handoffPath
"@
[IO.File]::WriteAllText($textPath,$text,[Text.UTF8Encoding]::new($false))
$progressPath=Join-Path (Split-Path -Parent $project) '監製進度.json';$progress=Get-Content -LiteralPath $progressPath -Raw|ConvertFrom-Json
$progress.updated_at=[DateTimeOffset]::Now.ToString('o');$progress.phase='R478_native_mode_filter_headers_thumb_state_clear_candidate_CPU_verified_GUI_pending';$progress.current_task='R478 native6231 thumb display/order and exact mode filter/header subset; ordinary flag0, mode rebuild clears top/selection; Modern same-mode/Back retention.1new focused40.82sPASS66negative/4096184coordinates/8composites/132pairs; actual564514 filter load/pan/frame cancellation and retirement verified. GUI0/16of40/LOADEDfalse.'
$progress|Add-Member -NotePropertyName native_R478_handoff -NotePropertyValue (Pin $handoffPath) -Force
$progress|Add-Member -NotePropertyName native_R478_candidate -NotePropertyValue $player.executable -Force
Write-ModernEvidenceJson -Path $progressPath -Value $progress
Write-ModernEvidenceJson -Path (Join-Path $logs 'r478-final-audit.json') -Value ([ordered]@{state='R478_SOURCE_RESOURCE_HEADER_PROBE_PIXEL_FOCUS_LIBRARY_EXE_HANDOFF_PIN_RECHECKED';checked_pin_instances=$checked;source_pairs=132;tools=$tools.Count;handoff=Pin $handoffPath;text=Pin $textPath;progress=Pin $progressPath;R408=Pin $r408;original_main=$player.original_main;GUI=0;LOADED=$false})
Write-Output (ConvertTo-Json (Pin $handoffPath) -Compress)
