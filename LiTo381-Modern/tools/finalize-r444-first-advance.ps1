param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function CheckPin($p,$sha){if((Get-FileHash -LiteralPath $p).Hash -ne $sha){throw "Drift: $p"}}
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
$cp=Join-Path $logs 'r444-candidate.json';$c=Get-Content $cp -Raw|ConvertFrom-Json;CheckPin $c.path $c.sha256
foreach($p in $c.source_pins){CheckPin $p.path $p.sha256;CheckPin $p.snapshot $p.sha256}
$build=@(Get-Content (Join-Path $logs 'r444b-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Library build incomplete'}
$libs=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($libs.Count -ne 1){throw 'Library count'}
$lib=Join-Path (Split-Path $c.path -Parent) 'liblito_modern-r444.rlib';if(!(Test-Path $lib)){[IO.File]::Copy($libs[0],$lib,$false)};CheckPin $lib (Get-FileHash $libs[0]).Hash
$c|Add-Member production_library ([ordered]@{path=$lib;sha256=(Get-FileHash $lib).Hash;offline_locked=$true;target='i686-pc-windows-msvc'}) -Force
$test=Join-Path $logs 'r444b-consumer.stdout.log';if([IO.File]::ReadAllText($test) -notmatch '1 passed; 0 failed; 0 ignored;.*finished in 0.23s'){throw 'Consumer test'}
$detailsPath=Join-Path $logs 'r444-first-advance-details.json';$d=Get-Content $detailsPath -Raw|ConvertFrom-Json
if($d.rows.Count -ne 32 -or $d.initial_zero_tick_cases -ne 24 -or $d.initial_positive_tick_cases -ne 8 -or $d.actual_runtime_inputs_observed -or $d.generic_zero_tick_skip -or $d.events_dispatched -or $d.renderer_LOADED){throw 'Qualification'}
if(@($d.rows|Where-Object{$_.initial_ticks -eq 0}).Count -ne 24 -or @($d.rows|Where-Object{$_.selected_frame -ne 1 -or $_.initial_frame -ne 0 -or $_.visible_frame0_residency_qualified -or $_.next_delay_ms -ne $_.returned_ms+2 -or $_.conditional_next_due_ms -ne 1000+$_.next_delay_ms}).Count){throw 'Source sequence'}
$contractPath=Join-Path $logs 'r444-original-first-advance-contract.json';CheckPin $contractPath $c.original_source_contract.sha256;$contract=Get-Content $contractPath -Raw|ConvertFrom-Json
foreach($p in $contract.source){CheckPin $p.path $p.sha256}
$historical=@(foreach($name in @('r443','r443-first-scope-attempt','r442')){
 $p=Join-Path $logs "$name-candidate.json";$o=Get-Content $p -Raw|ConvertFrom-Json;CheckPin $o.path $o.sha256;foreach($s in $o.source_pins){CheckPin $s.snapshot $s.sha256};if($o.production_library){CheckPin $o.production_library.path $o.production_library.sha256};[ordered]@{record=Pin $p;source_copies=$o.source_pins.Count;preserved=$true}
})
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$c.validation='PASSED:1new focused0.23sec;32conditional ordinary first-advances(24initialtick0+8positive);8original fragments match full disassembly. Frame1 returned duration+2 only; visible frame0 residency/player runtime/event/loop not qualified. R443clock unchanged and preserved;noR442rerun.'
Write-ModernEvidenceJson -Path $cp -Value $c
$pending=@('Actual original player global and encoded player clock table/current Class/source registration generation are not observed.','Ordinary versus special/death/player runtime branch and queue/modifier inputs must be observed or explicitly source-qualified; these inputs were conditional.','Visible frame0 residency during loading/render pass ordering is not proved by frame0 resource load or ordinary reset/increment.','Raw events have guarded execution; projectile/hit/sound/layer effects remain unqualified.','Later zero-tick effective-duration scheduling and animation-end/requeue/loop are not implemented; source+2/floor alone does not grant a visible animation clock.','Shadow/105 placement/timing/depth and full Native live renderer/GUI remain pending; R442same-Work control schema awaits producer.')
$resultPath=Join-Path $logs 'r444-original-first-advance-result.json'
Write-ModernEvidenceJson -Path $resultPath -Value ([ordered]@{state='PASSED_CONDITIONAL_ORIGINAL_ORDINARY_FIRST_ADVANCE_STATIC_CONSUMER';updated_at=[DateTimeOffset]::Now.ToString('o');candidate=$c;focused_passed=1;focused_failed=0;seconds=0.23;conditional_cases=32;initial_zero_tick=24;initial_positive_tick=8;original_source_fragments=8;original_source_contract=Pin $contractPath;details=Pin $detailsPath;test=Pin $test;compile_only_first_attempt=Pin (Join-Path $logs 'r444a-host-build.jsonl');R443_preserved=$historical;generic_zero_tick_skip=$false;positive_duration_invented=$false;frame0_visible_residency_qualified=$false;live_player_appearance_qualified=$false;events_dispatched=$false;natural_loop_qualified=$false;default_enabled=$false;original_BIN_executed=$false;Rust_live_rerun=$false;renderer_LOADED=$false;GUI=0;mode_coverage='16/40';formal_R408_changed=$false;source_files=506;pending=$pending})
$handoff=Join-Path $logs 'Native-appearance-R444原版零tick首步與資格界線交接.txt'
[IO.File]::WriteAllText($handoff,@"
R444b:1新focused0.23sec/0failed，32條件首步（24initialtick0+8positive）通過；host/lib/35source固定、8原版函式copy SHA與完整ASM內容相符。R443b2passed/40plan/368checks/4SPR/19CPUframes/2PNG既有正tick時鐘與失敗假設保留；R442不重跑。506source/R408保留。
原版43DE10 ticks*1000/FPS可儲存0，沒有通用正時長夾值。4E7A40初始化frame0並call4E5C00(arg0=0)，讀入第0幀資源；一般4E45F0切換resetframe0→4E5B50先increment→frame1再讀取4E5C00(arg0=1)，與firsttick0或positive無關。4E8750回傳selected frame1有效時長；明示非player/queue0/modifiers0/counter0/scalingdisabled條件下等於catalog duration。4E45F0只對其下一次callback加2，並非替frame0補2ms。
Type0 unarmed action1 frame1=125→127ms；Type0weapon3 action51及Type3weapon2 action47 frame1=166→168ms。anchor1000/playerglobal20000是明示conditional值，從未冒稱原版runtime觀察。未知分支/同playerglobal/queue或modifier非neutral/scaling未知/無clock/overflow/fullowner mismatch/retired以及後續selectedtick0均拒絕。第0幀catalog與events保留，不通用skip，不授予visible residency或event execution。
4E34C0 continuing路徑先advance後max(1,duration-lateness)+2；end重新frame0且回傳false，由4E02C0等state決定後續。尚未以此推論loop、player初始phase、原版render ordering、projectile/hit/sound、shadow/105/world/depth。
實際live外觀歸屬與sameWork Class/generation待producer，R442唯一工作槽BACK-held BackendUnavailable已被監製通知為設計限制，新control schema待交付；本輪無livefixtures/JavaSQL改動/BIN執行。LOADEDfalse/GUI0/16of40。可續独立layer/source研究，不能聲稱完整玩家renderer。
Result: $resultPath
"@,[Text.UTF8Encoding]::new($false))
Write-ModernEvidenceJson -Path (Join-Path $logs 'r444-final-verification.json') -Value ([ordered]@{result=Pin $resultPath;handoff=Pin $handoff;host=Pin $c.path;library=Pin $lib;source_copies=35;original_fragments=8;details=Pin $detailsPath;source_files=506;R443_preserved=$true;R442_rerun=$false;renderer_LOADED=$false;GUI=0})
$statusPath=Join-Path (Split-Path $project -Parent) '監製進度.json';$status=Get-Content $statusPath -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o');$status.phase='native_original_zero_tick_first_advance_conditional_study_passed'
$status.current_task='R443交接已收尾2pass0.45sec/40plan368checks/4SPR19frames2PNG固定；R444新1focused0.23sec/32conditional firstadvance(24tick0+8positive)通過，8original source片段完整ASM對照、host/lib35source/detail固定。原版loadframe0與ordinary reset0立即advance1分離，只selectedframe1時長+2，無補零tick時長/visible residency/event/loop授權。506source/R408保持、R443a/444compilefailed保留。R442不重跑，SessionGate唯一槽設計及新sameWork schema待producer；liveappearance/fullrenderer/shadow105/worlddepth與GUI未完成，LOADEDfalse/GUI0/16of40。'
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output 'R444 frozen:1test/32conditional first advances/8original fragments;visible residency andLOADEDfalse'
