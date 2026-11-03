param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function CheckPin($path,$sha){if((Get-FileHash -LiteralPath $path).Hash -ne $sha){throw "Pin drift: $path"}}
function Pin($path){[ordered]@{path=$path;sha256=(Get-FileHash -LiteralPath $path).Hash;bytes=(Get-Item -LiteralPath $path).Length}}
$cp=Join-Path $logs 'r443-candidate.json';$c=Get-Content $cp -Raw|ConvertFrom-Json
CheckPin $c.path $c.sha256
if($c.source_pins.Count -ne 35){throw 'Source count'}
foreach($p in $c.source_pins){CheckPin $p.path $p.sha256;CheckPin $p.snapshot $p.sha256}
$test=Join-Path $logs 'r443b-consumer.stdout.log'
if([IO.File]::ReadAllText($test) -notmatch '2 passed; 0 failed; 0 ignored;.*finished in 0.45s'){throw 'Tests'}
$build=@(Get-Content (Join-Path $logs 'r443b-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Library build'}
$libs=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'})
if($libs.Count -ne 1){throw 'Library count'}
$lib=Join-Path (Split-Path $c.path -Parent) 'liblito_modern-r443.rlib'
if(!(Test-Path $lib)){[IO.File]::Copy($libs[0],$lib,$false)}
CheckPin $lib (Get-FileHash $libs[0]).Hash
$c|Add-Member production_library ([ordered]@{path=$lib;sha256=(Get-FileHash $lib).Hash;offline_locked=$true;target='i686-pc-windows-msvc'}) -Force
$clockPath=Join-Path $logs 'r443-clock-details.json';$cpuPath=Join-Path $logs 'r443-cpu-clock-details.json'
$clock=Get-Content $clockPath -Raw|ConvertFrom-Json;$cpu=Get-Content $cpuPath -Raw|ConvertFrom-Json
if($clock.plans -ne 40 -or $clock.boundary_checks -ne 368 -or $clock.unsupported_zero_tick_plans.Count -ne 24 -or $cpu.unique_members_decoded -ne 4 -or $cpu.outputs.Count -ne 2 -or $clock.renderer_LOADED -or $cpu.renderer_LOADED -or $clock.GUI -or $cpu.GUI){throw 'Study scope'}
$snapshot=Get-Content (Join-Path $logs 'r439-appearance-snapshot.json') -Raw|ConvertFrom-Json
$manifestPath=Join-Path $snapshot.candidate.root 'manifest.json';CheckPin $manifestPath $snapshot.candidate.manifest_sha256
$manifest=Get-Content $manifestPath -Raw|ConvertFrom-Json
$inputs=@(foreach($name in @('resources/appearance-dependencies.nad','evidence/appearance-bindings.nab')+@($cpu.member_pins.PSObject.Properties.Name|ForEach-Object{"resources/$_"})){
 $m=@($manifest.files|Where-Object path -eq $name);if($m.Count -ne 1){throw 'Input identity'};$p=Join-Path $snapshot.candidate.root $name;CheckPin $p $m[0].sha256;if((Get-Item $p).Length -ne $m[0].bytes){throw 'Input length'};Pin $p
})
$images=@(foreach($p in $cpu.outputs){CheckPin $p.path $p.sha256;[ordered]@{path=$p.path;sha256=$p.sha256;dimensions=$p.dimensions;catalog_samples=$p.cells.Count}})
if($images[0].catalog_samples -ne 12 -or $images[1].catalog_samples -ne 7){throw 'Samples'}
$visual=[ordered]@{state='ACTUAL_CPU_IMAGES_VISUALLY_REVIEWED';images_reviewed=2;accepted=$true;images=$images;observations=@('Walk: three rows of four distinct body poses, armed prince/unarmed prince/elf; centered cells with no visible clipping.','Attack: seven elf bow body poses in catalog order, including initial walk member followed by attack member; no projectile/effect claim.');scope='Individual body frames; no world placement, shadow/105 interleaving, scene depth, animation loop or live player admission.'}
$visualPath=Join-Path $logs 'r443-visual-review.json';Write-ModernEvidenceJson -Path $visualPath -Value $visual
$history=@(foreach($name in @('r435','r436','r437','r438','r439','r440','r441','r442','r442-first-attempt','r442-second-attempt','r443-first-scope-attempt')){
 $p=Join-Path $logs "$name-candidate.json";$h=Get-Content $p -Raw|ConvertFrom-Json;CheckPin $h.path $h.sha256;foreach($s in $h.source_pins){CheckPin $s.snapshot $s.sha256};if($h.production_library){CheckPin $h.production_library.path $h.production_library.sha256};[ordered]@{record=$p;source_copies=$h.source_pins.Count;preserved=$true}
})
$failedPath=Join-Path $logs 'r443-first-scope-attempt.json';$failed=Get-Content $failedPath -Raw|ConvertFrom-Json
CheckPin $failed.partial_image $failed.partial_image_sha256
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$scope=Get-Content (Join-Path $logs 'source-change-scope.json') -Raw|ConvertFrom-Json
if($scope.source_files -ne 506 -or $scope.new_source_files.Count -ne 27){throw 'Source audit'}
foreach($p in $scope.frozen_binary_pins){CheckPin $p.path $p.sha256}
$c.validation='PASSED:2 focused/0failed in0.45s;40 positive-tick single-interval plans/368boundary checks;24Type0 zero-tick attack plans explicitly refused;4actual SPR members/19body frames/2visually reviewed PNG. No live appearance/event/layer/LOADED grant.'
Write-ModernEvidenceJson -Path $cp -Value $c
$pending=@('Type0 attack1/51 first tick0: original entry/advance/first-frame residency/timer/event ordering requires source qualification; no skipped frame or invented positive duration.','Raw catalog events retained, not dispatched; hit/projectile/sound/motion remains unqualified.','Shadow/105 timing and interleaving, scene/world placement/depth, natural loop/end, movement phase require separate original evidence.','R439 historical binding/Class does not grant R441/R442 live appearance ownership; private generation and same-Work appearance fields await producer.','Full Native PreparedResources, formal player renderer dispatch and GUI remain pending.')
$result=[ordered]@{state='PASSED_OFFLINE_POSITIVE_TICK_SINGLE_INTERVAL_BODY_STUDY';updated_at=[DateTimeOffset]::Now.ToString('o');candidate=$c;tests=Pin $test;focused_passed=2;focused_failed=0;seconds=0.45;positive_tick_plans=40;boundary_checks=368;explicit_zero_tick_attack_refusals=24;actual_members_decoded=4;body_frame_samples=19;inputs=$inputs;details=@((Pin $clockPath),(Pin $cpuPath));visual_review=Pin $visualPath;images=$images;first_scope_attempt=Pin $failedPath;history=$history;source_files=506;baseline=479;unchanged=469;allowed_changed=10;new_sources=27;default_enabled=$false;raw_events_dispatched=$false;natural_loop_qualified=$false;live_player_appearance_qualified=$false;renderer_LOADED=$false;GUI=0;mode_coverage='16/40';original_BIN_executed=$false;formal_R408_changed=$false;pending=$pending}
$resultPath=Join-Path $logs 'r443-clock-study-result.json';Write-ModernEvidenceJson -Path $resultPath -Value $result
$handoff=Join-Path $logs 'Native-appearance-R443有界時鐘與CPU幀交接.txt'
[IO.File]::WriteAllText($handoff,@"
R443b:2 focused passed/0failed in0.45sec, host/lib/35source frozen. 506sources=479baseline/469unchanged/10allowedchanged+27new; historical candidates including R442 failed/partial and R443a retained; R408 unchanged.
40 bounded positive-tick plans (32 walk +8 Type3/Class734 weapon2 attack47), 368 exact integer-ms start/end-exclusive checks. Full binding/epoch/revision/hash, foreign identity, cutoff, retirement and finite interval guards passed. No natural loop or runtime start phase inferred.
R439 pinned NAD/NAB and4actual SPR members decoded:12 walk body frames (armed prince, unarmed prince, elf) and7 elf attack body frames;2PNG visually reviewed. Walk4x166=664ms; elf attack83/166/166/166/125/125/125=956ms. Centered body cells retain source offset and raw events; no world placement or layer grant.
24 Type0 attack1/51 plans correctly refused first tick0. R443a assumption failure retained with source/host and incomplete output; no positive duration invented or first frame skipped. Original zero-tick entry/advance semantics is the next bounded source study.
Historical Class/binding is separate from actual live R441/R442 ownership. Events, projectile/hit/sound, shadow/105 timing/interleaving, scene/depth/loop, full Native player renderer remain pending. LOADEDfalse/GUI0/16of40; no BIN execution or server/SQL writes.
Result: $resultPath
"@,[Text.UTF8Encoding]::new($false))
Write-ModernEvidenceJson -Path (Join-Path $logs 'r443-final-verification.json') -Value ([ordered]@{result=Pin $resultPath;handoff=Pin $handoff;host=Pin $c.path;library=Pin $lib;source_copies=35;detail_files=2;actual_input_files=6;images=2;first_attempt_preserved=$true;source_files=506;renderer_LOADED=$false;GUI=0})
$statusPath=Join-Path (Split-Path $project -Parent) '監製進度.json';$status=Get-Content $statusPath -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o');$status.phase='native_positive_tick_clock_body_study_passed_zero_tick_source_audit_next'
$status.current_task='R443b2focused/0failed0.45sec、40正tick有界plan/368boundary/24零tick王族attack明確拒絕，4actualSPR/19CPUbodyframes/2PNG已視閱；host/lib/35source/2details/6inputs固定、506source審核與R408保持、R443a失敗假設保留。續原版零tick首幀與advance有限靜態證據；R442不重跑，原有唯一工作槽設計與新schema待producer。LOADEDfalse/GUI0/16of40。'
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output 'R443 frozen:2tests/40plans/368checks/4SPR/19frames/2PNG;zero-tick refused;LOADEDfalse'
