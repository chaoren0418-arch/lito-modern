$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Check($p){if((Get-FileHash -LiteralPath $p.path).Hash -ne $p.sha256 -or ($p.bytes -and (Get-Item -LiteralPath $p.path).Length -ne $p.bytes)){throw ('Pin drift '+$p.path)}}
$historical=@(foreach($old in @('r459','r460','r461','r462')){
 $rp=Join-Path $logs "$old-finite-renderer-result.json";$r=Get-Content -LiteralPath $rp -Raw|ConvertFrom-Json
 $fp=Join-Path $logs "$old-final-verification.json";$v=Get-Content -LiteralPath $fp -Raw|ConvertFrom-Json
 foreach($n in @('result','candidate','host','library','details','immutable_scope')){Check $v.$n}
 foreach($s in $r.candidate.source_pins){Check ([ordered]@{path=$s.snapshot;sha256=$s.sha256;bytes=$s.bytes})}
 Check $r.ASM;foreach($p in $r.images){Check $p};Check $r.test
 [ordered]@{stage=$old;result=Pin $rp;verification=Pin $fp;source_snapshots=$r.candidate.source_pins.Count;historical_test_rerun=$false}
})
$oldHandoff=Join-Path $logs 'r462-source-stop-retarget-handoff.json'
if((Get-FileHash -LiteralPath $oldHandoff).Hash -ne 'D53E95F94F00F8A0B4C210D850EE97B5FFE690453F4119E3480137C69DBA76CB'){throw 'R462 handoff drift'}
$rp=Join-Path $logs 'r463-finite-renderer-result.json';$r=Get-Content -LiteralPath $rp -Raw|ConvertFrom-Json
$fp=Join-Path $logs 'r463-final-verification.json';$v=Get-Content -LiteralPath $fp -Raw|ConvertFrom-Json
foreach($n in @('result','candidate','host','library','details','immutable_scope')){Check $v.$n}
foreach($s in $r.candidate.source_pins){Check $s;Check ([ordered]@{path=$s.snapshot;sha256=$s.sha256;bytes=$s.bytes})}
foreach($p in $r.selected_resources){Check $p};foreach($p in $r.extra_inputs){Check $p};foreach($p in $r.images){Check $p};Check $r.ASM;Check $r.test
$asm=Get-Content -LiteralPath $r.ASM.path -Raw|ConvertFrom-Json;foreach($p in $asm.files){Check $p}
$d=Get-Content -LiteralPath $r.details.path -Raw|ConvertFrom-Json
$qaPath=Join-Path $logs 'r463-visual-review.json';$qa=Get-Content -LiteralPath $qaPath -Raw|ConvertFrom-Json;foreach($p in $qa.direct_images){Check $p}
if($r.source_files -ne 540 -or $r.candidate.source_pins.Count -ne 82 -or $asm.files.Count -ne 61 -or $d.CPU_frames -ne 39 -or $d.negative_cases -ne 40 -or $qa.direct_count -ne 8){throw 'R463 count mismatch'}
if([IO.File]::ReadAllText($r.test.path) -notmatch '1 passed; 0 failed; 0 ignored;' -or [IO.File]::ReadAllText($r.test.path) -notmatch 'finished in 49.30s'){throw 'Focused mismatch'}
if(@($d.moves|Where-Object{$_.committed}).Count -ne 1 -or ($d.moves|Where-Object{$_.committed})[0].clock -ne 1498 -or (($d.moves|Where-Object{$_.action -eq 47}|ForEach-Object{$_.clock}) -join ',') -ne '1664,1832,2000,2168,2295,2422,2549'){throw 'Cancellation/action transition mismatch'}
if($d.moves[-1].clock -ne 3133 -or $d.moves[-1].action -ne 49 -or $d.moves[-1].prequeued[0] -ne 3301 -or -not $d.move64bbc3_unchanged_by_action -or $d.received_packet_observed -or $d.runtime_inputs_observed){throw 'Conditional scope mismatch'}
$audit=Get-Content -LiteralPath $r.immutable_source_scope.path -Raw|ConvertFrom-Json;foreach($p in $audit.frozen_binary_pins){Check $p}
$textPath=Join-Path $project 'Native-R463互動取消與攻擊銜接交接.txt';if(Test-Path -LiteralPath $textPath){throw 'Already handed off'}
$text=@'
R463：原版動作封包在完成格空檔取消移動carry，再切攻擊的有限條件式模型

61個完整原版函式固定封存。4B2970 case32→4A5A90→4A2A50解析DC（DWORD物件ID小端序／signed action byte）。4E8160必須是原版真正local actor；離線NAB、物件ID或JSON不能證明指標或實際封包接收。
4E6AF0檢查忙碌：移動途中frame非零/current46!=base46+3會先排入動作佇列，不立即4E45F0。40B6F0/406CE0等本地UI入口也拒絕busy/834EB3。因此本次不虛構走到一半立即撤銷所有步驟或鏡頭回捲。
1498先完成原方向的一格，寫cell/camera/roof；frame0/queue0、待1664的4E2150 owner0 carry存在時，明確條件式packet-before-timer順序允許1664動作封包先處理。4E45F0 true local actor/834EB3路徑清EB3/EB4，4D6870匹配callback4E2150與owner0撤銷待執行carry；屋頂/火盆回呼保留。4D6970則忽略owner按callback取消；4D69F0是時鐘getter。
動作arg1/base46→attack47，仍heading4，bank92/sub0，座標/相機/屋頂保留已完成格。64BBC3和目的地沒有被該來源清掉，也不據此自動恢復移動。新ordinary4E34C0 owner是actor pointer，first1832；1664,1832,2000,2168,2295,2422,2549攻擊序列，2549 end0 bank76，再排3133 idle49 bank84/sub0；3301未執行timer在私有owner退出時撤銷。
此次採明確trueplayer neutral global object關係，speed/haste/variant零，沒有拿舊nonplayer20000時計當作trueplayer證據。1845晚時計preview仍排2000且不commit。ordinary branch reached條件另列原版decoded831590安全選值3和actor/class/action/frame界限；值與branch仍未被真runtime觀測。原版draw/cache/audio/UI未執行。

唯一新增focused49.30秒通過、40拒絕檢查、39CPU畫面；8已直接檢視、31只SHA驗證。same-clock缺順序／dispatch已開始／錯callback或owner／缺指標和metadata／busy途中／clock overflow拒絕；取消前的move/action準備、旧carry ticket、owner退出和重入均拒絕。source/raster輸入preflight在commit前。
540來源＝479baseline／469unchanged／10allowedchanges／61new；82source snapshots、61wholeASM、fixed host/library/原資源/輸出/來源契約/缺口/QA固定。R459–R462沒有重跑或重封；舊失敗和未執行候選保留。
實際封包接收、原生同時計處理順序、actor registry/pointers/timers、live appearance/Work及framebuffer都未qualified。只寫modern；Java不需要。R408、16/40、LOADEDfalse、GUI0不變，整體仍in_progress。busy queued-action/dequeue producer是下一個不同且待實作的邊界。
'@
[IO.File]::WriteAllText($textPath,$text,[Text.UTF8Encoding]::new($false))
$contracts=@(foreach($n in @('r463-source-action-cancel-contract.json','r463-minimal-runtime-producer-gaps.json','r463-source-ordinary-entry-boundary.json')){Pin (Join-Path $logs $n)})
$handoff=Join-Path $logs 'r463-action-cancel-handoff.json'
Write-ModernEvidenceJson -Path $handoff -Value ([ordered]@{state='R463_SOURCE_PACKET32_GAP_CANCEL_ATTACK_IDLE_PRIVATE_SCENE_VERIFIED_RUNTIME_PENDING';checked_at=[DateTimeOffset]::Now.ToString('o');result=Pin $rp;final_verification=Pin $fp;contracts=$contracts;visual_review=Pin $qaPath;handoff_text=Pin $textPath;historical_protected=$historical;historical_r462_handoff=Pin $oldHandoff;new_focused_passed=1;focused_seconds=49.30;negative_cases=40;CPU_outputs=39;direct_visual_count=8;not_individually_viewed=31;source_files=540;source_snapshots=82;whole_original_functions=61;historical_tests_rerun=0;aggregate_with_historical=[ordered]@{focused_passed=5;CPU_outputs=166;negative_cases=193;direct_visual_count=47};actual_received_packet_observed=$false;native_sameclock_order_observed=$false;runtime_inputs_observed=$false;renderer_LOADED=$false;GUI=0;acceptance='16/40 unchanged';overall_status='in_progress'})
$sp=Join-Path (Split-Path $project -Parent) '監製進度.json';$s=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json;$s.updated_at=[DateTimeOffset]::Now.ToString('o');$s.phase='R463_source_action_gap_cancel_verified_runtime_bridge_pending';$s.current_task='R463 completed one new focused49.30sec/40refusals/39CPU(8directlyviewed). Actual sourcepacket32/DC nonbusy frame0 gap cancelspending4E2150 owner0 under explicit packet-first1664 equal-clock order; keeps1498completedcell/camera/roof and other root timers; trueplayer neutralattack47 ends2549/idle3133/future3301retired. Busy midstep action queues in original and immediate profile refuses. 540sources/82snapshots/61wholeASM/host/library/resources/contracts/gaps/QA frozen. R459-R462 no rerun; runtime pointers/order/appearance/framebuffer unqualified. Next distinct boundary is source busy action queue/dequeue, not invented midstep cancellation. R408/16of40/LOADEDfalse/GUI0 and modern-only scope retained;overall in progress.';$s|Add-Member native_R463 (Pin $handoff) -Force;Write-ModernEvidenceJson -Path $sp -Value $s
Write-Output (ConvertTo-Json (Pin $handoff) -Compress)
