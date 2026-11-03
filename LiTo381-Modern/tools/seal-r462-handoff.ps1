$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
function Check($p){if((Get-FileHash -LiteralPath $p.path).Hash -ne $p.sha256 -or ($p.bytes -and (Get-Item -LiteralPath $p.path).Length -ne $p.bytes)){throw ('Pin drift '+$p.path)}}
$oldPath=Join-Path $logs 'r459-r461-combined-handoff.json'
if((Get-FileHash -LiteralPath $oldPath).Hash -ne '2ABFF8C97FC0405B730BF593EFB2AFD7DBEF44EEC37A8B1D664689614420B0E4'){throw 'Historical handoff drift'}
$old=Get-Content -LiteralPath $oldPath -Raw|ConvertFrom-Json
foreach($st in $old.stages){foreach($name in @('result','candidate','host','library','test','details','immutable_source_scope','visual_review')){Check $st.$name};$c=Get-Content -LiteralPath $st.candidate.path -Raw|ConvertFrom-Json;foreach($s in $c.source_pins){Check ([ordered]@{path=$s.snapshot;sha256=$s.sha256;bytes=$s.bytes})}}
foreach($name in @('r459a-test-failure-preservation.json','r459b-unrun-guard-review-preservation.json')){$p=Get-Content -LiteralPath (Join-Path $logs $name) -Raw|ConvertFrom-Json;Check $p.host;if($p.source_snapshots -is [Array]){foreach($s in $p.source_snapshots){Check $s}};if($p.candidate){Check $p.candidate;$c=Get-Content -LiteralPath $p.candidate.path -Raw|ConvertFrom-Json;foreach($s in $c.source_pins){Check ([ordered]@{path=$s.snapshot;sha256=$s.sha256;bytes=$s.bytes})}}}
$r=Get-Content -LiteralPath (Join-Path $logs 'r462-finite-renderer-result.json') -Raw|ConvertFrom-Json
$v=Get-Content -LiteralPath (Join-Path $logs 'r462-final-verification.json') -Raw|ConvertFrom-Json
foreach($name in @('result','candidate','host','library','details','immutable_scope')){Check $v.$name}
foreach($s in $r.candidate.source_pins){Check $s;Check ([ordered]@{path=$s.snapshot;sha256=$s.sha256;bytes=$s.bytes})};foreach($p in $r.selected_resources){Check $p};foreach($p in $r.extra_inputs){Check $p};foreach($p in $r.images){Check $p};Check $r.ASM
$asm=Get-Content -LiteralPath $r.ASM.path -Raw|ConvertFrom-Json;foreach($p in $asm.files){Check $p}
$d=Get-Content -LiteralPath $r.details.path -Raw|ConvertFrom-Json
$qaPath=Join-Path $logs 'r462-visual-review.json';$qa=Get-Content -LiteralPath $qaPath -Raw|ConvertFrom-Json
foreach($p in $qa.direct_images){Check $p}
if($d.CPU_frames -ne 38 -or $d.negative_cases -ne 30 -or $qa.direct_count -ne 9 -or $r.source_files -ne 538 -or $r.candidate.source_pins.Count -ne 80 -or $asm.files.Count -ne 45){throw 'R462 count mismatch'}
if([IO.File]::ReadAllText($r.test.path) -notmatch '1 passed; 0 failed; 0 ignored;' -or [IO.File]::ReadAllText($r.test.path) -notmatch 'finished in 49.55s'){throw 'Focused evidence mismatch'}
if($d.moves[-1].clock -ne 2162 -or $d.moves[-1].active -or $d.moves[-1].prequeued.Count -ne 0 -or @($d.moves|Where-Object{$_.action -eq 49}).Count){throw 'Stopped end0 invented continuation'}
if(($d.inputs[0].unchanged_prequeued -join ',') -ne '1332,1498' -or ($d.inputs[1].unchanged_prequeued -join ',') -ne '1996,2162'){throw 'Input canceled existing source steps'}
$audit=Get-Content -LiteralPath $r.immutable_source_scope.path -Raw|ConvertFrom-Json
foreach($p in $audit.frozen_binary_pins){Check $p}
$textPath=Join-Path $project 'Native-R462移動途中改向停止交接.txt'
if(Test-Path -LiteralPath $textPath){throw 'Already handed off'}
$text=@'
R462：原版移動途中改目的地／目前格停止的有限來源與私有場景驗證

來源：45個完整原版函式固定封存。48C080 case512、popup空、左鍵旗標及viewport/HUD分支明確成立時呼叫48B9F0；選定plain mouse分支更新目的地與64BBC3，沒有立即呼叫移動、撤銷步驟或重畫。
1200改目的地：1332/1498既有timer保留，仍沿原heading4完成第一格1498。1664 carry才重取碰撞與三個時鐘，改heading2、bank74，排1830/1996/2162。
1850把目的地改成角色目前格：64BBC3清除，1996/2162仍保留。2162依旧heading2完成已起步一格、camera和屋頂更新；沒有2328 carry，也不創造idle49。
放開左鍵case514只清760098；plain無popup/action時沒有清64BBC3或取消timer，不能宣稱立即停止。4D69F0是時鐘取得函式；4D6970才是callback取消函式。4E45F0的真local actor取消4E2150(owner0)屬不同互動入口，此次不虛構觸發。

新增唯一focused通過49.55秒、30項拒絕檢查、38張CPU畫面；9張已直接檢视，其餘29張只做SHA驗證。新owner拒絕舊ticket/frame；同owner改向不撤銷預排timer、不改畫面generation。有限CPU場景保留最後46/frame0直到明確retire，不能當作原生可見性證明。
538來源＝479 baseline／469 unchanged／10 allowed changes／59 new。新fixed host與library、80個source snapshot、45 whole ASM、資源/CPU輸出/契約/缺口/QA固定封存。R459–R461未重跑或重封，失敗R459a及未執行R459b保留。
真實Windows input鏈、viewport資源、player pointer/queue、碰撞、Work/live appearance、原生framebuffer未觀測；neutral true-player分支是明確條件，未知與nonplayer拒絕。沒有執行wire/audio/UI。只寫modern範圍，Java fixture不需要。R408、16/40、LOADED=false、GUI=0不變，整體仍in_progress。
'@
[IO.File]::WriteAllText($textPath,$text,[Text.UTF8Encoding]::new($false))
$handoff=Join-Path $logs 'r462-source-stop-retarget-handoff.json'
$record=[ordered]@{state='R462_SOURCE_STOP_RETARGET_PRIVATE_SCENE_VERIFIED_RUNTIME_BRIDGE_PENDING';checked_at=[DateTimeOffset]::Now.ToString('o');result=Pin $v.result.path;final_verification=Pin (Join-Path $logs 'r462-final-verification.json');source_contract=Pin (Join-Path $logs 'r462-source-stop-retarget-contract.json');minimal_runtime_gaps=Pin (Join-Path $logs 'r462-minimal-runtime-producer-gaps.json');visual_review=Pin $qaPath;handoff_text=Pin $textPath;historical_r459_r461_handoff=Pin $oldPath;focused_passed=1;focused_seconds=49.55;negative_cases=30;CPU_outputs=38;direct_visual_count=9;not_individually_viewed=29;source_files=538;source_snapshots=80;whole_original_functions=45;historical_tests_rerun=0;aggregate_with_historical=[ordered]@{focused_passed=4;CPU_outputs=127;negative_cases=153;direct_visual_count=39};Java_fixture_required=$false;runtime_inputs_observed=$false;renderer_LOADED=$false;GUI=0;acceptance='16/40 unchanged';overall_status='in_progress'}
Write-ModernEvidenceJson -Path $handoff -Value $record
$sp=Join-Path (Split-Path $project -Parent) '監製進度.json';$s=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json;$s.updated_at=[DateTimeOffset]::Now.ToString('o');$s.phase='R462_source_stop_retarget_verified_runtime_bridge_pending';$s.current_task='R462 completed one new focused49.55sec/30 refusals/38CPU(9 directly viewed). Midstep target1200 preserves old timers and heading until1498, carry1664 adopts heading2; current-cell stop1850 finishes2162 with camera/roof but no carry/idle. Frozen538sources/80snapshots/45wholeASM, host/library/resources/contracts/QA. R459-R461 immutable snapshots and failed/unrun candidates retained, no rerun. Missing actual input/player/collision/native framebuffer/live authority remains unqualified; source interaction cancellation is distinct next boundary. R408/16of40/LOADEDfalse/GUI0, modern-only scope, overall in progress.';$s|Add-Member native_R462 (Pin $handoff) -Force;Write-ModernEvidenceJson -Path $sp -Value $s
Write-Output (ConvertTo-Json (Pin $handoff) -Compress)
