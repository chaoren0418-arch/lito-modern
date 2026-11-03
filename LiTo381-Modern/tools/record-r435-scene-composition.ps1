param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function ReadEvidence($name){Get-Content (Join-Path $logs $name) -Raw|ConvertFrom-Json}
function CheckPin($path,$sha){if((Get-FileHash $path).Hash -ne $sha){throw "Pin drift $path"}}
$c=ReadEvidence 'r435-candidate.json';CheckPin $c.path $c.sha256
if((Split-Path $c.path -Leaf) -ne 'lito-modern-r435c-host-tests.exe' -or $c.source_pins.Count -ne 14){throw 'Final candidate'}
foreach($p in $c.source_pins){CheckPin $p.path $p.sha256;CheckPin $p.snapshot $p.sha256}
$build=@(Get-Content (Join-Path $logs 'r435c-lib-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Library incomplete'}
$rlib=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($rlib.Count -ne 1){throw 'Library count'}
$library=Join-Path (Split-Path $c.path -Parent) 'liblito_modern-r435.rlib';$sha=(Get-FileHash $rlib[0]).Hash
if(Test-Path $library){CheckPin $library $sha}else{[IO.File]::Copy($rlib[0],$library,$false);CheckPin $library $sha}
$c|Add-Member NoteProperty production_library ([ordered]@{path=$library;sha256=$sha;offline_locked=$true;target='i686-pc-windows-msvc'}) -Force
$tests=@(foreach($row in @(@('local',3),@('consumer',1))){$p=Join-Path $logs ('r435c-'+$row[0]+'.stdout.log');$m=[regex]::Match([IO.File]::ReadAllText($p),'test result: ok\. (\d+) passed; (\d+) failed;');if(!$m.Success -or [int]$m.Groups[1].Value -ne $row[1] -or $m.Groups[2].Value -ne '0'){throw 'Tests'};[ordered]@{suite=$row[0];passed=$row[1];failed=0;log=$p;sha256=(Get-FileHash $p).Hash}})
$d=ReadEvidence 'r435-scene-consumer-details.json';$v=ReadEvidence 'r435-visual-review.json';$s=ReadEvidence 'r435-original-scene-source-pins.json';$contract=ReadEvidence 'r435-original-scene-contract.json'
if($d.state -ne 'REAL_NATIVE_CONDITIONAL_SCENE_INTERLEAVING_AND_OPAQUE_PARTIAL_VERIFIED' -or $d.rows.Count -ne 8 -or $d.LOADED -or $d.roof_runtime_qualified -or $d.natural_lifetime_qualified -or $d.actual_equipment_inferred -or $v.images_reviewed -ne 8 -or !$v.accepted){throw 'Scene scope'}
foreach($key in @('body_occlusion_observed_in_conditional_study','foreign_owner_refused','single_interval_frame_bounds_refused','budget_and_retire_revoke_output','opaque_partial_completed','owner_retire_cancels')){if($d.$key -ne $true){throw "Guard $key"}}
foreach($p in $d.rows){CheckPin $p.path $p.sha256}
foreach($p in $s.files){CheckPin $p.path $p.sha256};CheckPin $s.source $s.source_sha256
if($s.files.Count -ne 18 -or $contract.rules.Count -ne 8){throw 'Source rule count'}
$historical=@(foreach($name in @('r426','r427','r428','r429','r430','r431','r432','r433','r434')){$old=ReadEvidence "$name-candidate.json";CheckPin $old.path $old.sha256;CheckPin $old.production_library.path $old.production_library.sha256;foreach($p in $old.source_pins){CheckPin $p.snapshot $p.sha256};[ordered]@{candidate=$name;verified_pins=2+@($old.source_pins).Count;changed=$false}})
$scope=ReadEvidence 'source-change-scope.json';if($scope.source_files -ne 496 -or $scope.new_source_files.Count -ne 17){throw 'Source scope'};foreach($p in $scope.frozen_binary_pins){CheckPin $p.path $p.sha256}
$c.validation='PASSED:4 new focused tests;8 visual-reviewed bounded scene PNG;18 original source fragments/8 asserted rules;actual terrain+5 FirePot+profile1 body half-cell interleaving, strict owner/retire/frame bounds;opaque partial only. LOADEDfalse.'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r435-candidate.json') -Value $c
$resultPath=Join-Path $logs 'r435-scene-composition-result.json';$handoff=Join-Path $logs 'Native-scene-R435場景組合與roof缺口交接.txt'
$result=[ordered]@{updated_at=(Get-Date).ToString('o');state='SOURCE_QUALIFIED_CONDITIONAL_CPU_SCENE_VERIFIED_FULL_NATIVE_PENDING';candidate=$c;tests=$tests;focused_passed=4;focused_failed=0;original_source_fragments=18;source_rules=8;source_contract=Join-Path $logs 'r435-original-scene-contract.json';details=Join-Path $logs 'r435-scene-consumer-details.json';visual_review=$v;actual_DATA_cell=$d.body_actual_DATA_cell;real_effect_records=5;opaque_partial_completed=$true;default_enabled=$false;cross_object_insertion_runtime_observed=$false;conditional_relocation_is_study_only=$true;actual_equipment_inferred=$false;roof_runtime_qualified=$false;natural_lifetime_qualified=$false;viewport_adjacent_sectors_qualified=$false;historical_frozen_preservation=$historical;source_scope=$scope;R434_tests_rerun=$false;Rust_live_SQL_TLS=$false;original_BIN_executed=$false;renderer_LOADED=$false;full_PreparedResources=$false;GUI=0;qualified_modes=16;total_modes=40;formal_R408_changed=$false;pending=$contract.unknowns;preliminary_failures=@('R435a cross-connection stage correctly refused; corrected test to first assert refusal then perform same-scope replacement','R435b effect-frame bound assertion on no-effect control was inapplicable; finalC only checks effect bounds when effects exist. Earlier logs/host/partial image preserved')}
Write-ModernEvidenceJson -Path $resultPath -Value $result
$text=@"
R435 fixed default-off owner-bound conditional Native CPU scene
4 new focused tests passed/0failed, finalC host $($c.sha256), library $sha;14 frozen source copies.18 original ASM function pins and8 source-rule assertions verified. R426-R434 hosts/libraries/frozen source snapshots and R408 preserved; R434 tests not rerun.496 sources=479baseline+17new;469 unchanged/10allowed changes. EarlierA scope-refusal test andB no-effect-bound test diagnostics/hosts retained; finalC consumer137seconds succeeded.
Scene composition retains actual Native terrain/object fragments, all5 map FirePot records and original profile1 prince body fragments in one half-cell raster. Native whole-cell X becomes2X-8000; SPR signed A/B translates half-cell; scan row/column controls interleaving, floor precedes same-cell depth-list, ascending depth/newest-first equal-depth. Explicit cross-object insertion vector describes this local study, not original update history. Eight800x600PNG viewed: actualDATA32729/32797 with/without effects, and conditional rear/same/front relative to a real source FirePot with/without effects. Rear partly hidden, same fully hidden, front visible before flame. Dark viewport areas lie outside the single pinned sector; adjacent sector coverage remains unqualified.
Default disabled; unknown insertion, roof runtime/alias state, natural lifetime or body placement refuse. Body frame>=8/effect frame>=11 refuse for their loaded intervals. Full DATA/Native plan/opaque cancellation identity bind model/output; crossowner and changedconnection reject, same-scope replacement cancels old worker, budget/localretire/ownerretire revoke output. Original4CDBE0 removes matching block-pointer list entry; no guessed objectID alias replacement. Existing opaque partial completion still verifies terrain/effects/body/catalog provenance and owner ticket. FullPreparedResources/LOADED unavailable.
SCENE_ONLY profile1 is an explicit unarmed asset study, never evidence that real selected player has no equipment. Original4E0200 inserts body/attachment/shadow; actual actor flags+1A, attachment/clothes/shadow selection and blended drawing remain unqualified. StaticSourceOnly roof keeps decoded source pixels; original roof selected-actor groups, pointer alias/history and timer phase remain gaps. Natural Layer8 end/removal/repeat remains unqualified. No originalBIN, Java/SQL writes or RustliveTLS;GUI0/mode16of40/R408unchanged.
R436 fixed loadout input copied3452candidate+31replay files after supervisor handoff; independent Rust byte/rawfacts/deadline consumer is next. It is offline evidence, not92/93 expansion or authentication, and does not qualify weapon appearance.
Result: $resultPath
Rows/pixel coverage/PNG pins: $($result.details)
Source rules and exact remaining gaps: $($result.source_contract)
"@
[IO.File]::WriteAllText($handoff,$text,[Text.UTF8Encoding]::new($false))
$sp=Join-Path (Split-Path $project -Parent) '監製進度.json';$status=Get-Content $sp -Raw|ConvertFrom-Json;$status.updated_at=(Get-Date).ToString('o');$status.phase='native_scene_study_verified_fixed_loadout_consumer_in_progress';$status.current_task='R4354新測試與8場景對照PNG、18原版source pins/8rules已凍結；半格terrain/body/FirePot遮擋與owner/cancel/partial通過。R4363452+31固定loadout已保存，續做Rust完整投影/rawfacts/期限consumer。roof/runtime/actualgear/完整Native pending；LOADEDfalse/GUI0/R408保持。'
$status|Add-Member NoteProperty r435_scene_work ([ordered]@{result=$resultPath;handoff=$handoff;focused_passed=4;PNG=8;LOADED=$false}) -Force
Write-ModernEvidenceJson -Path $sp -Value $status
[IO.File]::AppendAllText((Join-Path $logs '持續驗證與缺口報告.txt'),"`r`nR4354passed/8scenePNG/18sourcepins/8rules，roof/runtime/actualequipment/fullNativepending。$handoff`r`n",[Text.UTF8Encoding]::new($false))
Write-Output 'R435 frozen4passed/8visualPNG;opaquePartial;LOADEDfalse'
