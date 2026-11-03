$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
function Check($p){if((Get-FileHash -LiteralPath $p.path).Hash -ne $p.sha256 -or ($p.bytes -and (Get-Item -LiteralPath $p.path).Length -ne $p.bytes)){throw ('Pin drift: '+$p.path)}}
function Snapshots($c){Check $c;Check $c.production_library;foreach($s in $c.source_pins){if((Get-FileHash -LiteralPath $s.snapshot).Hash -ne $s.sha256){throw ('Source snapshot drift: '+$s.snapshot)}}}
$combined=Join-Path $logs 'r459-r460-combined-handoff.json'
if(Test-Path -LiteralPath $combined){throw 'Already sealed; do not overwrite immutable handoff'}
$imagesTotal=0;$negativeTotal=0
$stages=@(foreach($stage in @('r459','r460')){
 $rp=Join-Path $logs "$stage-finite-renderer-result.json";$r=Get-Content -LiteralPath $rp -Raw|ConvertFrom-Json
 Snapshots $r.candidate;Check $r.details;Check $r.ASM;Check $r.immutable_source_scope;Check $r.test
 if($r.focused_passed -ne 1 -or $r.renderer_LOADED -or $r.GUI -ne 0){throw 'Conditional test scope'}
 $d=Get-Content -LiteralPath $r.details.path -Raw|ConvertFrom-Json
 foreach($p in $r.selected_resources){Check $p};foreach($p in $r.extra_inputs){Check $p};foreach($p in $r.images){Check $p}
 $asm=Get-Content -LiteralPath $r.ASM.path -Raw|ConvertFrom-Json;Check $asm.original_disassembly
 $text=[IO.File]::ReadAllText($asm.original_disassembly.path)
 foreach($p in $asm.files){Check $p;if(!$text.Contains([IO.File]::ReadAllText($p.path))){throw 'ASM not exact source'}}
 $imagesTotal+=@($r.images).Count;$negativeTotal+=[int]$d.negative_cases
 [ordered]@{stage=$stage;result=Pin $rp;host=Pin $r.candidate.path;library=$r.candidate.production_library;source_snapshots=@($r.candidate.source_pins).Count;whole_original_functions=@($asm.files).Count;CPU_outputs=@($r.images).Count;negative_cases=$d.negative_cases;focused_passed=1;test=$r.test;source_scope=$r.immutable_source_scope;details=$r.details}
})
if($imagesTotal -ne 48 -or $negativeTotal -ne 106){throw 'Combined totals differ'}
$protected=@(foreach($stage in @('r456','r457','r458')){
 $cp=Join-Path $logs "$stage-candidate.json";$c=Get-Content -LiteralPath $cp -Raw|ConvertFrom-Json;Snapshots $c
 $rp=Join-Path $logs "$stage-finite-renderer-result.json";$r=Get-Content -LiteralPath $rp -Raw|ConvertFrom-Json;Check $r.details;Check $r.immutable_source_scope
 foreach($p in $r.images){Check $p}
 [ordered]@{stage=$stage;candidate=Pin $cp;result=Pin $rp;source_snapshots=@($c.source_pins).Count;rerun=$false}
})
$older=Join-Path $logs 'r456-r458-combined-handoff-corrected.json'
if((Get-FileHash -LiteralPath $older).Hash -ne '29557FA20CF27981EE64BA394F2AC55F2B417A193CC94CBEF6AE69EB70EB8EF3'){throw 'Previous combined handoff drift'}
$failedp=Join-Path $logs 'r459a-test-failure-preservation.json';$failed=Get-Content -LiteralPath $failedp -Raw|ConvertFrom-Json
Check $failed.candidate;Check $failed.host;Check $failed.stdout;Check $failed.stderr
$fc=Get-Content -LiteralPath $failed.candidate.path -Raw|ConvertFrom-Json;Check $fc
foreach($s in $fc.source_pins){if((Get-FileHash -LiteralPath $s.snapshot).Hash -ne $s.sha256){throw 'Failed snapshot drift'}}
$unrunp=Join-Path $logs 'r459b-unrun-guard-review-preservation.json';$unrun=Get-Content -LiteralPath $unrunp -Raw|ConvertFrom-Json
Check $unrun.host;Check $unrun.build;foreach($p in $unrun.source_snapshots){Check $p}
$qa=@(foreach($name in @('r459-visual-review.json','r460-visual-review.json')){$p=Join-Path $logs $name;$q=Get-Content -LiteralPath $p -Raw|ConvertFrom-Json;foreach($i in $q.reviewed){Check $i};Pin $p})
$gap=Join-Path $logs 'r459-r460-minimal-runtime-producer-gaps.json';$contract=Join-Path $logs 'r459-source-locomotion-contract.json'
$exitp=Join-Path $logs 'r459-r460-source-world-reload-queue-contract.json';$exit=Get-Content -LiteralPath $exitp -Raw|ConvertFrom-Json;Check $exit.original_disassembly
$text=[IO.File]::ReadAllText($exit.original_disassembly.path);foreach($p in $exit.files){Check $p;if(!$text.Contains([IO.File]::ReadAllText($p.path))){throw 'Exit contract ASM drift'}}
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$audit=Get-Content -LiteralPath (Join-Path $logs 'source-change-scope.json') -Raw|ConvertFrom-Json
if($audit.source_files -ne 535 -or @($audit.new_source_files).Count -ne 56){throw 'Current source scope'}
$r460=$stages[1];$root=Split-Path $r460.host.path -Parent
$scope=Join-Path $root 'r459-r460-combined-source-scope-535.json'
[IO.File]::Copy((Join-Path $logs 'source-change-scope.json'),$scope,$false)
$handoff=Join-Path $project 'Native-R459-R460移動待機與退出重入交接.txt'
Write-ModernEvidenceJson -Path $combined -Value ([ordered]@{state='SOURCE_PLAYER_LOCOMOTION_AND_OWNED_IDLE2637_PRIVATE_SCENE_VERIFIED';checked_at=[DateTimeOffset]::Now.ToString('o');stages=$stages;focused_passed=2;CPU_outputs=$imagesTotal;negative_cases=$negativeTotal;source_files=535;baseline_source_files=479;unchanged_baseline=469;allowed_changed_baseline=10;new_sources=56;immutable_source_scope=Pin $scope;visual_review=$qa;individual_outputs_reviewed=22;remaining_R459_outputs_pinned_not_individually_reviewed=26;source_locomotion_contract=Pin $contract;minimal_runtime_producers=Pin $gap;source_world_reload_queue_contract=Pin $exitp;prior_R456_R458_protected=$protected;prior_combined=Pin $older;failed_R459a=Pin $failedp;unrun_R459b=Pin $unrunp;handoff=Pin $handoff;true_player_global_clock_branch='R459 conditional';nonplayer_ordinary_idle_clock_branch='R460 conditional';real_runtime_inputs_observed=$false;full_native_locomotion_execution_qualified=$false;real_world_reload_executed=$false;renderer_LOADED=$false;GUI=0;acceptance_coverage='16/40 unchanged';R408_rebuilt_or_replaced=$false;original_BIN_executed=$false;server_namespace_written=$false;overall_status='in_progress'})
$sp=Join-Path (Split-Path $project -Parent) '監製進度.json'
$status=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o');$status.phase='R459_R460_source_locomotion_owned_idle_cycle_private_retirement_reentry_passed_real_bridge_pending'
$status.current_task='R459 source true-player4E2150 starts/prequeues3 callbacks,4 draw steps then cell+roof,arrival1664 queuesidle2248; R460 actual private first-idle seed runs2637 through9-frame end0/source166ms requeue and3979repeat. Same owner/clock/depth scene; preflight before commit;106negative/48CPU/22directvisual;2necessaryfocused passed. 535sources=469unchanged/10allowedchanged/56new,74/77source snapshots,41/14ASM;R459a failed andR459b unrun preserved. Readonly world-reload retention whitelist proves conditional study-callback cleanup but real Work/runtime/mouse/collision/camera/audio/wire/global bridge remain unobserved;full native locomotion and original world reload execution unqualified. R456-R458 protected/no rerun;R408 unchanged;LOADEDfalseGUI0coverage16of40;all new processes completed.'
$status|Add-Member native_R459_R460 (Pin $combined) -Force
Write-ModernEvidenceJson -Path $sp -Value $status
Write-Output 'R459/R460 sealed:2 focused,48CPU,106negative,535sources; real runtime bridge still pending'
