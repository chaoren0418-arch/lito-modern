param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function ReadEvidence($name){Get-Content -LiteralPath (Join-Path $logs $name) -Raw|ConvertFrom-Json}
function CheckPin($path,$sha){if((Get-FileHash -LiteralPath $path).Hash -ne $sha){throw "Pin drift $path"}}
$c=ReadEvidence 'r434-candidate.json';CheckPin $c.path $c.sha256
if((Split-Path $c.path -Leaf) -ne 'lito-modern-r434e-host-tests.exe' -or $c.source_pins.Count -ne 12){throw 'Final candidate required'}
foreach($p in $c.source_pins){CheckPin $p.path $p.sha256;CheckPin $p.snapshot $p.sha256}
$build=@(Get-Content -LiteralPath (Join-Path $logs 'r434e-lib-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Library incomplete'}
$rlib=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'})
if($rlib.Count -ne 1){throw 'Library count'}
$library=Join-Path (Split-Path -Parent $c.path) 'liblito_modern-r434.rlib';$librarySha=(Get-FileHash -LiteralPath $rlib[0]).Hash
if(Test-Path -LiteralPath $library){CheckPin $library $librarySha}else{Copy-Item -LiteralPath $rlib[0] -Destination $library;CheckPin $library $librarySha}
$c|Add-Member NoteProperty production_library ([ordered]@{path=$library;sha256=$librarySha;offline_locked=$true;target='i686-pc-windows-msvc'}) -Force
$tests=@(foreach($row in @(@('local',6),@('consumer',1),@('actor_local',4),@('catalog_local',4),@('native_units',3),@('clock',2))){
 $path=Join-Path $logs ('r434e-'+$row[0]+'.stdout.log');$m=[regex]::Match([IO.File]::ReadAllText($path),'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
 if(!$m.Success -or [int]$m.Groups[1].Value -ne $row[1] -or $m.Groups[2].Value -ne '0'){throw "Test failed $($row[0])"}
 [ordered]@{suite=$row[0];passed=$row[1];failed=0;ignored=[int]$m.Groups[3].Value;log=$path;sha256=(Get-FileHash -LiteralPath $path).Hash}
})
$d=ReadEvidence 'r434-conditional-event-details.json'
if($d.state -ne 'REAL_ASSET_CONDITIONAL_EVENT_SEQUENCES_AND_PARTIAL_VERIFIED' -or $d.rows.Count -ne 30 -or $d.images.Count -ne 6 -or $d.LOADED -or $d.original_BIN_executed -or $d.source_anchor_values_runtime_observed -or $d.original_loop_or_lifetime_inferred){throw 'Consumer scope'}
foreach($key in @('body_object_substitution_refused','body_clock_table_substitution_refused','model_retire_refuses_held_event','foreign_job_event_refused','opaque_partial_completed','owner_retire_cancels')){if($d.$key -ne $true){throw "Guard missing $key"}}
foreach($id in @(38,123)){
 $rows=@($d.rows|Where-Object{$_.asset -eq 'FirePot' -and $_.sprite -eq $id})
 if($rows.Count -ne 11){throw 'Effect count'}
 for($i=0;$i -lt 11;$i++){$s=$rows[$i].step;if($s.sequence -ne $i+1 -or $s.frame -ne (($i+1)%11) -or $s.due_ms -ne 584+85*$i -or $s.dispatch_ms -ne $s.due_ms -or $s.returned_duration_ms -ne 83 -or $s.actor_queue_after_pop -ne 0 -or $s.timer_queue_after_removal -ne 0){throw 'Effect event order'}}
 if($null -ne $rows[-1].step.next_due_ms){throw 'Unqualified loop scheduled'}
}
$body=@($d.rows|Where-Object{$_.asset -eq 'PrinceIdle'});if($body.Count -ne 8 -or $d.body_object_id -ne 10275 -or $d.body_conditional_player_global -ne 20000){throw 'Body identity'}
for($i=0;$i -lt 8;$i++){$s=$body[$i].step;if($s.object_id -ne 10275 -or $s.sequence -ne $i+1 -or $s.frame -ne (($i+1)%8) -or $s.due_ms -ne 168*$i -or $s.dispatch_ms -ne $s.due_ms -or $s.returned_duration_ms -ne 166){throw 'Body event order'}}
if($null -ne $body[-1].step.next_due_ms){throw 'Body loop scheduled'}
foreach($p in $d.images){CheckPin $p.path $p.sha256;if(![IO.Path]::GetFullPath($p.path).StartsWith((Join-Path $project 'target\r434e-conditional-event-cpu')+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Final image root'}}
$visual=ReadEvidence 'r434-visual-review.json';if($visual.images_reviewed -ne 6 -or $visual.accepted -ne $true){throw 'Visual review pending'}
foreach($p in $visual.images){CheckPin $p.path $p.sha256}
$source=ReadEvidence 'r434-original-source-pins.json';$sched=ReadEvidence 'r432-scheduler-new-source-pins.json';$clock=ReadEvidence 'r433-original-clock-source-pins.json'
$sourceCount=0;foreach($set in @($source,$sched,$clock)){foreach($p in $set.files){CheckPin $p.path $p.sha256;$sourceCount++}}
CheckPin $sched.source $sched.source_sha256
$contract=ReadEvidence 'r434-original-layer8-event-contract.json';if($contract.original_BIN_started -or $contract.default_enabled -or $contract.LOADED){throw 'Original contract scope'}
$snapshot=ReadEvidence 'r433-catalog-snapshot.json';CheckPin $snapshot.input $snapshot.input_sha256
$cfg=Get-Content -LiteralPath $snapshot.input -Raw|ConvertFrom-Json
$capture=@($cfg.candidate.vectors|Where-Object{$_.name -eq 'sql92/actor-sql-wire-capture.json'});if($capture.Count -ne 1){throw 'Capture identity'};CheckPin $capture[0].path $capture[0].sha256
$historical=@(foreach($name in @('r426','r427','r428','r429','r430','r431','r432','r433')){
 $old=ReadEvidence "$name-candidate.json";CheckPin $old.path $old.sha256;CheckPin $old.production_library.path $old.production_library.sha256
 foreach($p in $old.source_pins){CheckPin $p.snapshot $p.sha256}
 [ordered]@{candidate=$name;verified_pins=2+@($old.source_pins).Count;changed=$false}
})
$scope=ReadEvidence 'source-change-scope.json';if($scope.source_files -ne 494 -or $scope.new_source_files.Count -ne 15){throw 'Source scope'};foreach($p in $scope.frozen_binary_pins){CheckPin $p.path $p.sha256}
$c.validation='PASSED:20 focused;30 conditional real asset CPU events;6 visual-reviewed PNG;strict body object and clock substitution refusal;opaque partial only;LOADEDfalse. Original runtime globals/phase/loop remain unqualified.'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r434-candidate.json') -Value $c
$resultPath=Join-Path $logs 'r434-event-scheduler-result.json'
$result=[ordered]@{updated_at=(Get-Date).ToString('o');state='BOUNDED_CONDITIONAL_EVENT_CPU_VERIFIED_ORIGINAL_RUNTIME_FULL_RENDERER_PENDING';candidate=$c;tests=$tests;focused_passed=20;focused_failed=0;CPU_event_rows=30;FirePot_rows=22;body_rows=8;visual_review=$visual;details=Join-Path $logs 'r434-conditional-event-details.json';original_source_contract=Join-Path $logs 'r434-original-layer8-event-contract.json';original_source_fragments_verified=$sourceCount;fresh_SQL_Loading_input=$capture[0];body_object_id=10275;conditional_player_global=20000;conditional_effect_object_id=1073741823;conditional_effect_player_global=10275;conditional_anchor_ms=0;conditional_modifiers=@(0,0);initial_values_runtime_observed=$false;external_state_frozen_is_explicit_study=$true;player_global_branch_refused=$true;first_animation_end_continuation_refused=$true;default_enabled=$false;opaque_partial_completed=$true;historical_frozen_preservation=$historical;source_scope=$scope;early_C_consumer='Preserved r434c host/source/images/details; body scenario object10275 differed from old R432 high-bit owner. C is not final identity qualification. E uses fresh R433 SQL Loading object10275 and explicit object/table refusal.';Rust_live_SQL_TLS=$false;original_BIN_executed=$false;renderer_LOADED=$false;full_PreparedResources=$false;GUI=0;formal_known_extensions=255;formal_R408_changed=$false;qualified_modes=16;total_modes=40;pending=$contract.necessary_gaps;additional_pending=@('Other class/sex/headings/weapon and combat appearance','Complete Native PreparedResources, formal live dispatch and player GUI')}
Write-ModernEvidenceJson -Path $resultPath -Value $result
$handoff=Join-Path $logs 'Native-event-R434有界scheduler與原版runtime缺口交接.txt'
$text=@"
R434 default-off bounded conditional original event model; full Native player remains pending
20 focused passed/0 failed:6 scheduler local,1 actual resource event consumer,4 actor owner/identity,4 catalog,3 Native resource,2 integer frame clock. Frozen r434e host $($c.sha256), production library $librarySha;12 current/frozen source copies verified. $sourceCount original source fragments and whole disassembly hash verified;494 source files=479 baseline+15 new,469 unchanged/10 allowed changes. R426-R433 frozen host/library/source copies and formal R408 unchanged. Early R434c provisional details/source/host/images remain preserved: its body scenario object10275 did not match old high-bit owner; final E replaces input with fresh R433 actualSQL92/93 Loading object10275 and rejects object/table substitution.
Literal integer order:83+2=85;83/2 truncates41 then+2=43;166+2=168;>=2 queue short-circuits later4/8/16 CFG branches. Nonplayer modifiers apply sequentially with integer truncation. Dispatcher samples one explicit clock; removes timer before callback; equal deadlines retain FIFO; opcode32 pops actor queue before clock calculation, resets frame0 then advances frame1. Late callbacks use max(1,duration-lateness)+2; bounded budget and owner/model retirement drain/refuse. Player equality, unknown initial values, nonmonotonic/wrapping/signed-out-of-range clocks and first-end continuation refuse.
Actual FirePot38 and123 each emit11 CPU events at relative584,669,754,839,924,1009,1094,1179,1264,1349,1434ms with frames1..10,0; real prince0/sex0 explicit-unarmed Idle emits8 at0,168,336,504,672,840,1008,1176 with frames1..7,0. Every event retains state/action/frame/due/dispatch/queue/duration/next and RGBA hash. Six PNGs (first-frame1/end-frame0 per asset) visually reviewed. Actual pinned full Sprite stream/provenance,4 MISS indexes and4 member ranges verify through existing loader; foreign job, same-job wrong body object and wrong table refuse. Event-selected body object10275 matches opaque Loading DATA; player global20000 is explicitly conditional, so this is NOT an observed original player branch. Effects use conditional object3FFFFFFF/player10275/empty queue; modifiers0/anchor0/frozen external state are explicit studies, not runtime constants.
Layer8 source:62F328 allocation counter and75A848/84C range;75AA48 player global written indirectly through packet decoder4A36F0;4E8160 may reuse existing actor, so constructor queue0 applies only to fresh lookup MISS.4DFDD0 saved through unlabelled continuation to actual return, not truncated IDA malloc fragment. Parent modifier writes can change child1C/21.4E82F0/bulk removal cancel timer before destructor; ordinary end4E02C0 may requeue under guards. No natural infinite loop/lifetime inferred. Counter import identity/init/runtime phase remains unresolved.
Required original runtime observations:actual75AA48 and effective allocated object IDs at insertion;fresh/reuse outcome and actor queue;clock API/init/anchor;external queue/modifier updates or proof of a frozen window;natural removal/end guards and full Layer8/actor/roof insertion. Without these, the default-off conditional model is the completed independent implementation, not live renderer acceptance. Full other-class/equipment/combat appearance, PreparedResources, formal dispatch/player GUI remain pending. Existing opaque cancel/partial checks complete; renderer LOADEDfalse, formal KNOWN255, mode16/40,GUI0 preserved. No original BIN, Java/SQL edits or live Rust sockets.
Result: $resultPath
Exact30 event rows and6 PNG pins: $($result.details)
Source evidence and runtime gaps: $($result.original_source_contract)
"@
[IO.File]::WriteAllText($handoff,$text,[Text.UTF8Encoding]::new($false))
$sp=Join-Path (Split-Path -Parent $project) '監製進度.json';$s=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json
$s.updated_at=(Get-Date).ToString('o');$s.phase='native_conditional_event_scheduler_verified_original_runtime_full_renderer_pending';$s.current_task='R434預設關閉有界事件模型20passed；真實FirePot22與王族Idle8事件CPU序列、6PNG、object/時長表拒絕及opaque partial已驗。原版player global/clock phase/reuse queue/外部modifier/lifetime仍須runtime證據，完整Native renderer pending；LOADEDfalse/GUI0/16of40/R408unchanged。'
$s|Add-Member NoteProperty r434_event_scheduler_work ([ordered]@{result=$resultPath;handoff=$handoff;focused_passed=20;CPU_events=30;visual_images=6;original_runtime_observed=$false;LOADED=$false}) -Force
$s.verified=@($s.verified)+@('R43420focused/30realassetconditionalCPUevents/6visualPNG/strictbodyobject-clockguards/opaquePartial；defaultoff、原版runtime選支不臆測，R426–R433/R408保持。')
Write-ModernEvidenceJson -Path $sp -Value $s
[IO.File]::AppendAllText((Join-Path $logs '持續驗證與缺口報告.txt'),"`r`nR434 $((Get-Date).ToString('o')) 20passed/30conditionalCPUevents/6PNG；runtime globals/phase/reuse/external writes/lifetime and fullNative/LOADED/playerGUI pending。$handoff`r`n",[Text.UTF8Encoding]::new($false))
Write-Output 'R434 frozen20passed;30conditionalCPUevents;6visualPNG;LOADEDfalse'
