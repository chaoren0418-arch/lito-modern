param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function ReadEvidence($name){Get-Content -LiteralPath (Join-Path $logs $name) -Raw|ConvertFrom-Json}
function CheckPin($path,$sha){if((Get-FileHash -LiteralPath $path).Hash -ne $sha){throw "Pin drift $path"}}
$c=ReadEvidence 'r432-candidate.json';CheckPin $c.path $c.sha256
foreach($p in $c.source_pins){CheckPin $p.path $p.sha256;CheckPin $p.snapshot $p.sha256}
$build=@(Get-Content -LiteralPath (Join-Path $logs 'r432-lib-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Production library incomplete'}
$rlib=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'})
if($rlib.Count -ne 1){throw 'Library count'}
$library=Join-Path (Split-Path -Parent $c.path) 'liblito_modern-r432.rlib';$librarySha=(Get-FileHash -LiteralPath $rlib[0]).Hash
if(Test-Path -LiteralPath $library){CheckPin $library $librarySha}else{Copy-Item -LiteralPath $rlib[0] -Destination $library;CheckPin $library $librarySha}
$c|Add-Member NoteProperty production_library ([ordered]@{path=$library;sha256=$librarySha;offline_locked=$true;target='i686-pc-windows-msvc'}) -Force
$tests=@(foreach($row in @(@('r432c-local',4),@('r432c-consumer',1),@('r432c-body',1),@('r432-native_local',15),@('r432-catalog_local',4),@('r432-native_units',3),@('r432-clock',2))){
 $path=Join-Path $logs ($row[0]+'.stdout.log');$m=[regex]::Match([IO.File]::ReadAllText($path),'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
 if(-not $m.Success -or [int]$m.Groups[1].Value -ne $row[1] -or [int]$m.Groups[2].Value){throw "Unexpected tests $path"}
 [ordered]@{suite=$row[0];passed=[int]$m.Groups[1].Value;failed=0;ignored=[int]$m.Groups[3].Value;log=$path;sha256=(Get-FileHash -LiteralPath $path).Hash}
})
$snap=ReadEvidence 'r432-catalog-snapshot.json';CheckPin $snap.input $snap.input_sha256
foreach($family in @('candidate','replay')){
 $cfg=$snap.$family;$mp=Join-Path $cfg.root 'manifest.json';CheckPin $mp $cfg.manifest_sha256
 $m=Get-Content -LiteralPath $mp -Raw|ConvertFrom-Json
 if($m.files.Count -ne $cfg.files_verified){throw 'Manifest count'}
 foreach($p in $m.files){$path=[IO.Path]::GetFullPath((Join-Path $cfg.root $p.path));if(-not $path.StartsWith($cfg.root+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Manifest confinement'};if((Get-Item -LiteralPath $path).Length -ne $p.bytes){throw 'Length drift'};CheckPin $path $p.sha256}
 foreach($v in $cfg.vectors){CheckPin $v.path $v.sha256}
}
$d=ReadEvidence 'r432-actor-publication-details.json';$body=ReadEvidence 'r432-actor-body-details.json'
if($d.state -ne 'FIXED_ACTOR_PROTOCOL_PUBLICATIONS_CONSUMED' -or $body.state -ne 'REAL_ACTOR_DEPENDENCY_PARTIAL_VERIFIED' -or $d.LOADED -or $body.LOADED){throw 'Qualification'}
if(@($d.rows).Count -ne 60 -or @($d.negatives).Count -ne 128 -or @($d.receipts).Count -ne 2){throw 'Row count'}
$historical=@(foreach($name in @('r426','r427','r428','r429','r430','r431')){
 $old=ReadEvidence "$name-candidate.json";CheckPin $old.path $old.sha256;CheckPin $old.production_library.path $old.production_library.sha256
 foreach($p in $old.source_pins){CheckPin $p.snapshot $p.sha256};[ordered]@{candidate=$name;verified_pins=2+@($old.source_pins).Count;changed=$false}
})
$scope=ReadEvidence 'source-change-scope.json';if($scope.source_files -ne 491){throw 'Source count'}
foreach($p in $scope.frozen_binary_pins){CheckPin $p.path $p.sha256}
$route=ReadEvidence 'r430-native-actor-route-contract.json';$original=ReadEvidence 'r430-original-route-snapshot.json'
$provPath=Join-Path $snap.candidate.root 'provenance\source-verification.json';$prov=Get-Content -LiteralPath $provPath -Raw|ConvertFrom-Json
$reconciled=@(foreach($p in $prov.assembly_pins){$own=if($p.name -eq 'C_CreateChar.java'){$route.class_source.path}else{Join-Path $original.root $p.name};CheckPin $own $p.sha256;[ordered]@{name=$p.name;own=$own;sha256=$p.sha256}})
CheckPin (Join-Path $logs 'r430-native-actor-route-contract.json') $prov.r430_contract_sha256
CheckPin (Join-Path $project 'src\modern_lito\resource_cipher.rs') $prov.decoder_source_sha256
CheckPin 'D:\lineage開發\0-客戶端\LiTo.bin' 'CE6EE68ADA8FD37E6C096B1A27E46CB6888F1179F85DEFB5007AC60287360292'
$scheduler=ReadEvidence 'r432-original-scheduler-layer8-contract.json'
$c.validation='PASSED:30 focused tests;60 paired92/93 publications;128 direct byte negatives;2 typed READ61/MOVE65 scope receipts;real base body streaming and opaque partial completion. Full renderer/LOADED pending.'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r432-candidate.json') -Value $c
$resultPath=Join-Path $logs 'r432-native-actor-result.json'
$result=[ordered]@{updated_at=(Get-Date).ToString('o');state='ACTOR_DEPENDENCY_AND_OPAQUE_PARTIAL_VERIFIED_FULL_RENDERER_PENDING';candidate=$c;tests=$tests;focused_passed=30;focused_failed=0;snapshot=$snap;snapshot_files_reverified=3307;publication_details=Join-Path $logs 'r432-actor-publication-details.json';body_details=Join-Path $logs 'r432-actor-body-details.json';paired92_93_rows=60;direct_byte_negatives=128;typed_READ_MOVE_receipts=$d.receipts;publication_families=@($d.rows|Group-Object family|Select-Object Name,Count);producer_negative_ledger_count=84;producer_negative_ledger_replayed=$false;provenance_assembly_reconciled=$reconciled;producer_extractor_executed=$false;producer_extractor_same_as_Modern=$false;historical_frozen_preservation=$historical;source_scope=$scope;actor_identity_bytes=952;request92_bytes=348;extension_bits=1792;formal_known_extensions=255;identity_digest=$body.identity_digest;opaque_partial_completed=$true;scheduler_contract=Join-Path $logs 'r432-original-scheduler-layer8-contract.json';LOADED=$false;full_PreparedResources=$false;Rust_live_SQL_TLS=$false;GUI=0;qualified_modes=16;total_modes=40;formal_R408_changed=$false;pending=@('Other class/sex/headings/weapon appearance and combat loadout','Scheduler clock API/init, actual start phase, speed/brave/global modifiers and lifetime','Layer8 full insertion/depth/roof/effect integration','Complete Native PreparedResources and live Rust transport/player GUI','Fresh append-only SQL90/91 and92/93 captures independently consume next')}
Write-ModernEvidenceJson -Path $resultPath -Value $result
$readiness=[ordered]@{updated_at=(Get-Date).ToString('o');state='NATIVE_PLAYER_ADMISSION_CLOSED';verified=@('Fixed92/93 actor dependency identity and nested catalog/Native proofs','Actual class0/sex0 body Text/Sprite/MISS/member stream verification','Opaque owner/ticket/source/fullDATA/cancel partial completion','Isolated body Walk/Idle headings0/1 integer frame intervals');gates=@([ordered]@{gate='formal Native hello/ack/dispatch';ready=$false;reason='Draft remains crate-private/default-disabled; formal KNOWN255'},[ordered]@{gate='complete Native renderer resources';ready=$false;reason='Only partial terrain/effect/body qualification; clock/Layer8/full insertion pending'},[ordered]@{gate='actual equipment/combat body';ready=$false;reason='Explicit unarmed profile1 study, SCENE_ONLY no loadout'},[ordered]@{gate='live Rust SQL/TLS and player GUI';ready=$false;reason='Fixed byte consumers; no live Rust actor session or GUI validation'});LOADED=$false;GUI=0;formal_R408_changed=$false}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r432-native-player-readiness.json') -Value $readiness
$handoff=Join-Path $logs 'Native-actor-R432依賴與scheduler交接.txt'
$text=@"
R432 actor dependency92/93 + original scheduler/Layer8 source handoff
30 focused tests/0 failed. Fixed host $($c.sha256); production library $librarySha. Nine source copies match current sources. 3292+15 own manifest entries reverified. R426–R431 frozen host/library/source copies and R408 preserved. No original PE/Java execution, no full mode rotation. R432b adapter failure retained: TLS map contains paired request and result keys; final consumer filters result93 and uses exact corresponding request92.
Default-disabled/crate-private bit1024 requires256+512 (1792); formal KNOWN remains255. Identity952 schema1/ns4/profile1 binds original LiTo.bin, provenance JSON, C_CreateChar, exact LTtext catalog168, base Text raw/decoded list, Sprite pair, ordered four MISS indexes (4/5/12/13) and four selected member extents (0/1/8/9). Digest $($body.identity_digest). Request92=348 wraps exact90; result93 min428, max1044+91max, exact nested presence/catalog/request IDs and actor-domain proof. Structural decode and context authority remain distinct.
Candidate/replay 60 paired rows:11+20 TLS and11+18 TLS. All provided92 and paired TLS requests checked/reencoded. Five legacy89 captures per family equal full validated93 nested89. 128 direct byte negatives=64 each family; producer84 codec/service cases are not all independently replayed. Two typed READ61/MOVE65 receipts preserve opaque epoch/source/object/channel/room/match/gen/entity/life/revision; valid source7 applies, source42 Active guard refused. Producer synthetic-auth TLS, not SQL; Rust has no live TLS/SQL. Wire LOADED test-peer responses do not authorize renderer LOADED.
Real body worker streamed full1.9GB Sprite PAK once, verifies Text index/selected list raw beforeL1 and decoded SHA after, exact selected Sprite offsets/raw hashes, and four ordered verified MISS indexes then unique base hits. Provenance bytes SHA and semantic resource/routes checked through fixed confined file; JSON paths never followed. All24 assembly/C_CreateChar provenance pins independently match own original source copies, R430 contract/decoder/source LiTo SHA match. Producer extractor source differs from own independent extraction tool; it was not executed or asserted identical.
Actual class0/sex0/logical0/alias3225; headings0/1, explicit unarmed study only. Wrong class/heading/provenance refuse; sealed verified body retains identity/cancelArc and real decoded assets. Legal real catalog+terrain/effect+body accepts only opaque partial; cross-owner completion refuses, retire cancels, LOADED alwaysfalse. Perframe166ms Walk/Idle boundaries remain finite; no unknown actions/loop default enabled.
Original scheduler source: constructor action0/heading0/frame0; conditional action0→3 route uses584ms when current differs from base+3. Queue opcode32 stores action; 4E9300 dispatches 4E45F0, resets frame0 then 4E5B50 advances to1 when within range, schedules4E34C0 at returned duration+2. 4D6570 samples clock once per dispatch and removes due queue entry before invoking callback. This is conditional static behavior, not proven actual object phase/infinite loop; opaque clock imports, speed/brave/global/lifetime and full Layer8 remain unresolved. Contract: $($result.scheduler_contract)
FullNative PreparedResources, formal dispatch/admission, player/combat/GUI remain pending.16/40,remaining24,GUI0,R408 unchanged. Fresh append-only realSQL captures are the next independent consumer; historical lost91 remains lost and is not reconstructed.
Result: $resultPath
Publication bytes: $($result.publication_details)
Body worker: $($result.body_details)
Player gates: $(Join-Path $logs 'r432-native-player-readiness.json')
"@
[IO.File]::WriteAllText($handoff,$text,[Text.UTF8Encoding]::new($false))
$sp=Join-Path (Split-Path -Parent $project) '監製進度.json';$s=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json
$s.updated_at=(Get-Date).ToString('o');$s.phase='native_actor_dependency_partial_verified_fresh_sql_capture_consumer_pending';$s.current_task='R43230focused/60paired92-93/128byte negatives/3307fixedpins及真實base body opaque partial通過；原版conditional584ms與queue32/dispatcher來源已釘選。續做新append-only真SQL90-91/92-93 capture獨立consumer；完整Native renderer/LOADED/player/GUI仍pending，16of40/R408unchanged。'
$s|Add-Member NoteProperty r432_native_actor_work ([ordered]@{result=$resultPath;handoff=$handoff;paired92_93=60;direct_byte_negatives=128;LOADED=$false}) -Force
$s.verified=@($s.verified)+@('R43230focused/60paired92-93/128directbyte negatives/3307fixedpins；真實base Text/list/Sprite/MISS/member dependencies進opaquepartial，跨owner拒絕與retire取消通過；LOADEDfalse。')
Write-ModernEvidenceJson -Path $sp -Value $s
[IO.File]::AppendAllText((Join-Path $logs '持續驗證與缺口報告.txt'),"`r`nR432 $((Get-Date).ToString('o')) 30passed/60paired92-93/128negative/3307pins；body authority僅profile1partial；584ms/queue32/dispatch static evidence；fullNative/LOADED/GUI pending。$handoff`r`n",[Text.UTF8Encoding]::new($false))
Write-Output 'R432 frozen30passed;60paired92/93;128negative;LOADEDfalse'
