$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
function Check($p){if((Get-FileHash -LiteralPath $p.path).Hash -ne $p.sha256 -or ($p.bytes -and (Get-Item -LiteralPath $p.path).Length -ne $p.bytes)){throw ('Pin drift: '+$p.path)}}
function Snapshots($c){Check $c;Check $c.production_library;foreach($s in $c.source_pins){if((Get-FileHash -LiteralPath $s.snapshot).Hash -ne $s.sha256){throw 'Historical source snapshot drift'}}}
$final=Join-Path $logs 'r459-r461-combined-handoff.json'
if(Test-Path -LiteralPath $final){throw 'Immutable result already exists'}
$sp=Join-Path (Split-Path $project -Parent) '監製進度.json';$status=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json
Check $status.native_R459_R460
$prior=Get-Content -LiteralPath $status.native_R459_R460.path -Raw|ConvertFrom-Json
foreach($p in $prior.prior_R456_R458_protected){Check $p.candidate;Check $p.result;$c=Get-Content -LiteralPath $p.candidate.path -Raw|ConvertFrom-Json;Snapshots $c}
Check $prior.prior_combined;Check $prior.failed_R459a;Check $prior.unrun_R459b;Check $prior.handoff
$outputs=0;$negatives=0;$reviewed=0
$stages=@(foreach($stage in @('r459','r460','r461')){
 $rp=Join-Path $logs "$stage-finite-renderer-result.json";$r=Get-Content -LiteralPath $rp -Raw|ConvertFrom-Json
 Snapshots $r.candidate;Check $r.details;Check $r.ASM;Check $r.immutable_source_scope;Check $r.test
 if($r.focused_passed -ne 1 -or $r.renderer_LOADED -or $r.GUI -ne 0){throw 'Stage scope'}
 foreach($p in $r.selected_resources){Check $p};foreach($p in $r.extra_inputs){Check $p};foreach($p in $r.images){Check $p}
 $asm=Get-Content -LiteralPath $r.ASM.path -Raw|ConvertFrom-Json;Check $asm.original_disassembly;$text=[IO.File]::ReadAllText($asm.original_disassembly.path)
 foreach($p in $asm.files){Check $p;if(!$text.Contains([IO.File]::ReadAllText($p.path))){throw 'Whole original function not exact'}}
 $d=Get-Content -LiteralPath $r.details.path -Raw|ConvertFrom-Json
 $qaPath=Join-Path $logs "$stage-visual-review.json";$qa=Get-Content -LiteralPath $qaPath -Raw|ConvertFrom-Json;foreach($p in $qa.reviewed){Check $p}
 $outputs+=@($r.images).Count;$negatives+=[int]$d.negative_cases;$reviewed+=[int]$qa.individual_review_count
 [ordered]@{stage=$stage;result=Pin $rp;candidate=Pin (Join-Path $logs "$stage-candidate.json");host=Pin $r.candidate.path;library=$r.candidate.production_library;source_snapshots=@($r.candidate.source_pins).Count;original_functions=@($asm.files).Count;CPU_outputs=@($r.images).Count;negative_cases=$d.negative_cases;focused_passed=1;test=$r.test;details=$r.details;immutable_source_scope=$r.immutable_source_scope;visual_review=Pin $qaPath}
})
if($outputs -ne 89 -or $negatives -ne 123 -or $reviewed -ne 30){throw 'Final numeric totals'}
foreach($p in $prior.visual_review){Check $p};Check $prior.source_locomotion_contract;Check $prior.minimal_runtime_producers;Check $prior.source_world_reload_queue_contract
$exit=Get-Content -LiteralPath $prior.source_world_reload_queue_contract.path -Raw|ConvertFrom-Json;Check $exit.original_disassembly
$text=[IO.File]::ReadAllText($exit.original_disassembly.path);foreach($p in $exit.files){Check $p;if(!$text.Contains([IO.File]::ReadAllText($p.path))){throw 'Readonly exit source fragment drift'}}
$f=Get-Content -LiteralPath $prior.failed_R459a.path -Raw|ConvertFrom-Json;Check $f.candidate;Check $f.host;Check $f.stdout;Check $f.stderr
$fc=Get-Content -LiteralPath $f.candidate.path -Raw|ConvertFrom-Json;foreach($s in $fc.source_pins){if((Get-FileHash -LiteralPath $s.snapshot).Hash -ne $s.sha256){throw 'Failed source snapshot drift'}}
$u=Get-Content -LiteralPath $prior.unrun_R459b.path -Raw|ConvertFrom-Json;Check $u.host;Check $u.build;foreach($p in $u.source_snapshots){Check $p}
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$audit=Get-Content -LiteralPath (Join-Path $logs 'source-change-scope.json') -Raw|ConvertFrom-Json
if($audit.source_files -ne 536 -or @($audit.new_source_files).Count -ne 57){throw 'Final source scope'}
$root=Split-Path $stages[2].host.path -Parent;$scope=Join-Path $root 'r459-r461-combined-source-scope-536.json'
[IO.File]::Copy((Join-Path $logs 'source-change-scope.json'),$scope,$false)
$handoff=Join-Path $project 'Native-R459-R461移動待機與退出重入交接.txt'
Write-ModernEvidenceJson -Path $final -Value ([ordered]@{state='SOURCE_PLAYER_ONE_AND_TWO_CELL_MOVEMENT_OWNED_IDLE2637_PRIVATE_RETIREMENT_REENTRY_VERIFIED';checked_at=[DateTimeOffset]::Now.ToString('o');stages=$stages;focused_passed=3;CPU_outputs=$outputs;negative_cases=$negatives;individual_outputs_reviewed=$reviewed;remaining_byte_pinned_not_individually_reviewed=$outputs-$reviewed;source_files=536;baseline=479;unchanged_baseline=469;allowed_changed_baseline=10;new_sources=57;immutable_source_scope=Pin $scope;prior_R459_R460=$status.native_R459_R460;prior_R456_R458_protected=$prior.prior_R456_R458_protected;failed_R459a_preserved=$prior.failed_R459a;unrun_R459b_preserved=$prior.unrun_R459b;source_locomotion_contract=$prior.source_locomotion_contract;source_world_reload_queue_contract=$prior.source_world_reload_queue_contract;minimal_runtime_producer_gaps=$prior.minimal_runtime_producers;source_multistep_producer_boundary=Pin (Join-Path $logs 'r461-multistep-producer-boundary.json');handoff=Pin $handoff;true_player_global_clock_branch='R459/R461 explicitly conditional';nonplayer_ordinary_idle_clock_branch='R460 explicitly conditional';real_runtime_inputs_observed=$false;full_native_locomotion_execution_qualified=$false;real_world_reload_executed=$false;renderer_LOADED=$false;GUI=0;acceptance_coverage='16/40 unchanged';R408_rebuilt_or_replaced=$false;old_R456_R458_tests_rerun=$false;original_BIN_executed=$false;server_namespace_written=$false;overall_status='in_progress'})
$status.updated_at=[DateTimeOffset]::Now.ToString('o');$status.phase='R459_R461_source_multistep_locomotion_owned_idle_private_retirement_reentry_passed_real_bridge_pending'
$status.current_task='R459/R461 original4E2150 true-player conditional start->prequeue3callbacks->4draw steps->end0cell/roof->carry reselect nextstep or natural arrival->idle; actual2cells/roof136168to136 verified. R460owned first-idle seed runs2637through9frame end0/source166ms queue32 repeat3979; JSON cannot mint seed. Three necessary new focused passed47.53/35.93/50.41sec;89CPU123negative30directvisual;536sources469unchanged10allowedchanged57new;74/77/78source snapshots and41/14/41ASM. R459a failed andR459b unrun preserved. R456-R458 andR459-R460 immutable combined/snapshots retained, no old test rerun. Source whole-world reload whitelist cleanup conditional, real runtime Work/appearance/global/collision/camera/mouse/stop-retarget/audio/wire/UI/reentry bridge still unobserved. LOADEDfalseGUI0coverage16of40R408unchanged;all task build/test/seal processes completed;overall in progress.'
$status|Add-Member native_R459_R461 (Pin $final) -Force
Write-ModernEvidenceJson -Path $sp -Value $status
Write-Output 'R459-R461 final sealed:3focused89CPU123negative536sources; real runtime bridge pending'
