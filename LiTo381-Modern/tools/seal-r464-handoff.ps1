$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Check($p){if((Get-FileHash -LiteralPath $p.path).Hash -ne $p.sha256 -or ($p.bytes -and (Get-Item -LiteralPath $p.path).Length -ne $p.bytes)){throw ('Pin drift '+$p.path)}}
$historical=@(foreach($old in @('r459','r460','r461','r462','r463')){
 $rp=Join-Path $logs "$old-finite-renderer-result.json";$r=Get-Content -LiteralPath $rp -Raw|ConvertFrom-Json
 $fp=Join-Path $logs "$old-final-verification.json";$v=Get-Content -LiteralPath $fp -Raw|ConvertFrom-Json
 foreach($n in @('result','candidate','host','library','details','immutable_scope')){Check $v.$n}
 foreach($s in $r.candidate.source_pins){Check ([ordered]@{path=$s.snapshot;sha256=$s.sha256;bytes=$s.bytes})}
 Check $r.ASM;foreach($p in $r.images){Check $p};Check $r.test
 [ordered]@{stage=$old;result=Pin $rp;verification=Pin $fp;source_snapshots=$r.candidate.source_pins.Count;historical_test_rerun=$false}
})
$oldHandoff=Join-Path $logs 'r463-action-cancel-handoff.json'
if((Get-FileHash -LiteralPath $oldHandoff).Hash -ne 'B15293DB244C18FD064F0577A94B584AF2408218BE72C77EA1C9C66E68D48C61'){throw 'R463 handoff drift'}
$rp=Join-Path $logs 'r464-finite-renderer-result.json';$r=Get-Content -LiteralPath $rp -Raw|ConvertFrom-Json
$fp=Join-Path $logs 'r464-final-verification.json';$v=Get-Content -LiteralPath $fp -Raw|ConvertFrom-Json
foreach($n in @('result','candidate','host','library','details','immutable_scope')){Check $v.$n}
foreach($s in $r.candidate.source_pins){Check $s;Check ([ordered]@{path=$s.snapshot;sha256=$s.sha256;bytes=$s.bytes})}
foreach($p in $r.selected_resources){Check $p};foreach($p in $r.extra_inputs){Check $p};foreach($p in $r.images){Check $p};Check $r.ASM;Check $r.test
$asm=Get-Content -LiteralPath $r.ASM.path -Raw|ConvertFrom-Json;foreach($p in $asm.files){Check $p}
$d=Get-Content -LiteralPath $r.details.path -Raw|ConvertFrom-Json
$qaPath=Join-Path $logs 'r464-visual-review.json';$qa=Get-Content -LiteralPath $qaPath -Raw|ConvertFrom-Json;foreach($p in $qa.direct_images){Check $p}
if($r.source_files -ne 542 -or $r.candidate.source_pins.Count -ne 84 -or $asm.files.Count -ne 65 -or $d.CPU_frames -ne 39 -or $d.negative_cases -ne 67 -or $qa.direct_count -ne 8){throw 'R464 count mismatch'}
if([IO.File]::ReadAllText($r.test.path) -notmatch '1 passed; 0 failed; 0 ignored;' -or [IO.File]::ReadAllText($r.test.path) -notmatch 'finished in 50.22s'){throw 'Focused mismatch'}
if(@($d.moves|Where-Object{$_.committed}).Count -ne 1 -or ($d.moves|Where-Object{$_.committed})[0].clock -ne 1498 -or (($d.moves|Where-Object{$_.popped_serial}|ForEach-Object{$_.popped_serial}) -join ',') -ne '1,2'){throw 'Cell/pop order mismatch'}
if(($d.entries.capacity -join ',') -ne '2,2,4,4,8,8,8' -or @($d.entries|Where-Object{$_.suppressed}).Count -ne 1 -or $d.entries[-1].clock -ne 1206 -or $d.moves[-1].clock -ne 2632 -or $d.moves[-1].prequeued[0] -ne 2800 -or ($d.remaining_serials_removed -join ',') -ne '3,4,5,6' -or $d.actual_received_packet_observed -or $d.runtime_inputs_observed){throw 'Queue/source qualification mismatch'}
$zeroPath=Join-Path $logs 'r464a-zero-selected-preservation.json';$zero=Get-Content -LiteralPath $zeroPath -Raw|ConvertFrom-Json;foreach($name in @('candidate','stdout','stderr','host','corrected_focused')){Check $zero.$name};foreach($s in $zero.source_snapshots){Check ([ordered]@{path=$s.snapshot;sha256=$s.sha256;bytes=$s.bytes})};if($zero.focused_executed -ne 0 -or [IO.File]::ReadAllText($zero.stdout.path) -notmatch '0 passed;'){throw 'Zero selected preservation mismatch'}
$reloadPath=Join-Path $logs 'r464-source-reload-boundary.json';$reload=Get-Content -LiteralPath $reloadPath -Raw|ConvertFrom-Json;Check $reload.whitelist_data;Check $reload.ordinary_entry_inherited
$original=[IO.File]::ReadAllText($asm.original_disassembly.path);if(!$original.Contains([IO.File]::ReadAllText($reload.whitelist_data.path))){throw 'Reload whitelist data not original'}
$audit=Get-Content -LiteralPath $r.immutable_source_scope.path -Raw|ConvertFrom-Json;foreach($p in $audit.frozen_binary_pins){Check $p}
$textPath=Join-Path $project 'Native-R464忙碌動作佇列與出列交接.txt';if(Test-Path -LiteralPath $textPath){throw 'Already handed off'}
$text=@'
R464：移動途中接動作，完成目前格後按來源佇列出列攻擊

65完整原版函式固定封存。4A5A90→4E6AF0忙碌→4E63C0；opcode32/arg1會保留每筆，超過三筆也不合併。只有count>3、尾端opcode32且參數相同、requestedarg2才在配置前略過。新節點8bytes；count達capacity時capacity加倍、複製既有指標並釋放舊陣列。初始capacity2是條件式输入，私有8筆上限不是原版容量上限。缺少或失敗配置条件一律拒絕，不推導ACK。
4E85F0取array[0]並前移其餘指標、減count，FIFO由完整原碼證明。4E9300 case32釋放optional68、呼叫4E45F0，然後釋放取出的節點。忙碌封包不立即取消移動或寫新位置。
1200至1206依序接1,1,1,1,1,2,2，保留serial1至6；最後2略過且不要求未呼叫的配置輸入，cap2→4→8。原1332/1498步驟、屋頂/火盆root回呼和heldframe都保留；所有输入嚴格先於下一個roottimer，不猜同時計處理順序。
1498 end0才寫cell/camera/48AFC0屋頂，queue非空排actor-owner4E9300於1664，不排owner0移動carry。1664取serial1，開始47/bank92/sub0；1832,2000,2168,2295,2422普通攻擊續幀，2549 end0/bank76。queue仍五筆，所以4E02C0用endduration83排2632出列，不排3133 idle。2632取serial2再次47/frame1，剩3,4,5,6（1,1,1,2）；未執行2800timer連同四筆在私有owner移除撤銷。全程僅1498格子/鏡頭/屋頂承諾，64BBC3/目的地不被動作清掉。
4E82F0 no-opcode11/非class68選定移除路徑：原版unlink/registry移除→4D68F0取消所有actor-owner回呼→virtualdestructor4DFED0/4DFF00釋放queue和陣列。解構本身不能證明取消timer；owner0也不被actor-ownerall取消。本模型只在已沒有owner0timer的攻擊階段套用此移除profile，缺少任一步驟拒絕。
48BC80 reload另外追到：decoded8315F0非零，actor解構後4D63A0依九個callback白名單過濾；4E2150/4E9300/4E34C0不在保留表。資料片段另固定，不能把該來源當作4E82F0相同的取消順序。本輪不實作reload整體root/global再建立，也不授予原生執行已觀測。

唯一新focused50.22秒通過、67拒絕檢查、39CPU；8張直接檢視、31張只SHA驗證。foreign/stale准备、錯count/capacity/owner、配置未知/失败、同時計/晚输入、缺屋頂承諾、queue限制、早期owner0移除、缺移除producer、退出和新owner舊ticket/frame均拒絕。trueplayer中性時鐘，非nonplayerqueue加速；opaqueordinaryentry沿用R463明示條件、rawaudio/wire/UI只記錄不執行。
542來源＝479baseline／469unchanged／10allowedchanges／63new；84source snapshots、65wholeASM、host/library/assets/images/contracts/QA封存。第一次錯exactname選到0tests紀錄與候選快照保留，未算通過；工具已加必須恰一測試通過的檢查。正確focused在同一固定主機獨立輸出執行一次。R459–R463未重跑或重封，R408未取代。
仍待實作：reload白名單清理與新sourceowner/root重建；action8清佇列/保留opcode11、queuedarg2執行、真receivedpacket/registry/allocator/order與live appearance/nativeframebuffer。只寫modern，Java不需要；16/40、LOADEDfalse、GUI0與overallin_progress保留。
'@
[IO.File]::WriteAllText($textPath,$text,[Text.UTF8Encoding]::new($false))
$contracts=@(foreach($n in @('r464-source-busy-queue-contract.json','r464-minimal-runtime-producer-gaps.json','r464-source-reload-boundary.json')){Pin (Join-Path $logs $n)})
$handoff=Join-Path $logs 'r464-busy-queue-handoff.json'
Write-ModernEvidenceJson -Path $handoff -Value ([ordered]@{state='R464_SOURCE_BUSY_QUEUE_FIFO_ATTACK_OWNER_REMOVAL_VERIFIED_RUNTIME_PENDING';checked_at=[DateTimeOffset]::Now.ToString('o');result=Pin $rp;final_verification=Pin $fp;contracts=$contracts;reload_whitelist_data=$reload.whitelist_data;ordinary_entry_inherited=$reload.ordinary_entry_inherited;visual_review=Pin $qaPath;handoff_text=Pin $textPath;zero_selected_preserved=Pin $zeroPath;historical_protected=$historical;historical_r463_handoff=Pin $oldHandoff;new_focused_passed=1;focused_seconds=50.22;negative_cases=67;CPU_outputs=39;direct_visual_count=8;not_individually_viewed=31;source_files=542;source_snapshots=84;whole_original_functions=65;historical_tests_rerun=0;aggregate_with_historical=[ordered]@{focused_passed=6;CPU_outputs=205;negative_cases=260;direct_visual_count=55};actual_received_packet_observed=$false;native_sameclock_order_observed=$false;runtime_inputs_observed=$false;renderer_LOADED=$false;GUI=0;acceptance='16/40 unchanged';remaining=@('Distinct48BC80 reload whitelist cleanup/newsourceowner root reconstruction','action8/opcode11 retention and queuedarg2 execution','actual receive/registry/allocator/clock order/runtime appearance bridge');overall_status='in_progress'})
$sp=Join-Path (Split-Path $project -Parent) '監製進度.json';$s=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json;$s.updated_at=[DateTimeOffset]::Now.ToString('o');$s.phase='R464_source_busy_queue_verified_reload_boundary_pending';$s.current_task='R464 one new focused50.22sec/67refusals/39CPU(8direct) verified: busy packet32 preserves duplicate1, conditionalcap2->4->8, special repeated2skip;1498onlycell/camera/roofcommit->1664serial1attack->2549end/83ms->2632serial2attack;remaining4nodes/future2800retired via selectedactor-ownerall cancellation+destructor profile. 542sources/84snapshots/65wholeASM/fixedhost/library/contracts/resources/QAsealed;zero-selected exactname mistake separatelypreserved,not counted. R459-R463 no rerun/R408unchanged. Source48BC80 reload destructor-before-whitelist cleanup proved with9keptcallbacks; distinctreloadroot/newowner integration next. Actualruntime/receive/allocations/appearance/framebuffer unqualified;16of40/LOADEDfalse/GUI0/modernonly/overallinprogress.';$s|Add-Member native_R464 (Pin $handoff) -Force;Write-ModernEvidenceJson -Path $sp -Value $s
Write-Output (ConvertTo-Json (Pin $handoff) -Compress)
