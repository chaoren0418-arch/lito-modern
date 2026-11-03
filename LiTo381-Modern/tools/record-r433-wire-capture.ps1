param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function ReadEvidence($name){Get-Content -LiteralPath (Join-Path $logs $name) -Raw|ConvertFrom-Json}
function CheckPin($path,$sha){if((Get-FileHash -LiteralPath $path).Hash -ne $sha){throw "Pin drift $path"}}
$c=ReadEvidence 'r433-candidate.json';CheckPin $c.path $c.sha256
foreach($p in $c.source_pins){CheckPin $p.path $p.sha256;CheckPin $p.snapshot $p.sha256}
$build=@(Get-Content -LiteralPath (Join-Path $logs 'r433-lib-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Library incomplete'}
$rlib=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'})
if($rlib.Count -ne 1){throw 'Library count'}
$library=Join-Path (Split-Path -Parent $c.path) 'liblito_modern-r433.rlib';$librarySha=(Get-FileHash -LiteralPath $rlib[0]).Hash
if(Test-Path -LiteralPath $library){CheckPin $library $librarySha}else{Copy-Item -LiteralPath $rlib[0] -Destination $library;CheckPin $library $librarySha}
$c|Add-Member NoteProperty production_library ([ordered]@{path=$library;sha256=$librarySha;offline_locked=$true;target='i686-pc-windows-msvc'}) -Force
$tests=@(foreach($suite in @('r433c-local','r433c-consumer')){$path=Join-Path $logs ($suite+'.stdout.log');$m=[regex]::Match([IO.File]::ReadAllText($path),'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;');if(!$m.Success -or $m.Groups[1].Value -ne '1' -or $m.Groups[2].Value -ne '0'){throw 'Focused test failure'};[ordered]@{suite=$suite;passed=1;failed=0;ignored=[int]$m.Groups[3].Value;log=$path;sha256=(Get-FileHash -LiteralPath $path).Hash}})
$snap=ReadEvidence 'r433-catalog-snapshot.json';CheckPin $snap.input $snap.input_sha256
foreach($family in @('candidate','replay')){$cfg=$snap.$family;$mp=Join-Path $cfg.root 'manifest.json';CheckPin $mp $cfg.manifest_sha256;$m=Get-Content -LiteralPath $mp -Raw|ConvertFrom-Json;if($m.files.Count -ne $cfg.files_verified){throw 'Manifest count'};foreach($p in $m.files){$path=[IO.Path]::GetFullPath((Join-Path $cfg.root $p.path));if(!$path.StartsWith($cfg.root+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Confinement'};if((Get-Item -LiteralPath $path).Length -ne $p.bytes){throw 'Length'};CheckPin $path $p.sha256};foreach($p in $cfg.vectors){CheckPin $p.path $p.sha256}}
$d=ReadEvidence 'r433-append-only-consumer-details.json'
if($d.state -ne 'FIXED_APPEND_ONLY_SQL_CAPTURE_CONSUMED' -or $d.renderer_LOADED -or $d.Rust_live_SQL_TLS -or $d.historical_lost91_reconstructed){throw 'Qualification'}
if($d.rows.Count -ne 271 -or $d.EOF_records.Count -ne 32 -or $d.new_initial_discoveries.Count -ne 8 -or $d.source_generation_proofs.Count -ne 16 -or $d.inventory_equipment.Count -ne 32 -or $d.typed_receipts.Count -ne 12){throw 'Counts'}
foreach($row in $d.rows|Where-Object{$_.data_owner}){foreach($key in @('room','match','generation')){if($row.data_owner.$key -ne $row.observed_world.$key){throw 'Observed DATA world metadata mismatch'}}}
$counts=@(@('candidate',90,78,60,7),@('candidate',92,95,75,9),@('replay',90,80,62,7),@('replay',92,94,74,9))
foreach($x in $counts){$s=@($d.summaries|Where-Object{$_.family -eq $x[0] -and $_.route -eq $x[1]})[0];if($s.records -ne $x[2] -or $s.resource_pairs -ne $x[3] -or $s.EOF_no_response -ne $x[4]){throw 'Family counts'}}
$historical=@(foreach($name in @('r426','r427','r428','r429','r430','r431','r432')){$old=ReadEvidence "$name-candidate.json";CheckPin $old.path $old.sha256;CheckPin $old.production_library.path $old.production_library.sha256;foreach($p in $old.source_pins){CheckPin $p.snapshot $p.sha256};[ordered]@{candidate=$name;verified_pins=2+@($old.source_pins).Count;changed=$false}})
$scope=ReadEvidence 'source-change-scope.json';if($scope.source_files -ne 492){throw 'Source scope'};foreach($p in $scope.frozen_binary_pins){CheckPin $p.path $p.sha256}
$c.validation='PASSED:2 new focused tests;347 append-only records;271 full resource pairs;8 new initial discoveries+nested89;16 source-generation domain proofs;32 typed inventory/equipment and12 READ/MOVE receipts.32 EOF have no response. R432 resource partial is separate; renderer LOADEDfalse.'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r433-candidate.json') -Value $c
$resultPath=Join-Path $logs 'r433-wire-capture-result.json'
$result=[ordered]@{updated_at=(Get-Date).ToString('o');state='FRESH_APPEND_ONLY_SQL_90_92_CAPTURE_CONSUMER_VERIFIED_FULL_RENDERER_PENDING';candidate=$c;tests=$tests;focused_passed=2;focused_failed=0;snapshot=$snap;snapshot_files_reverified=3436;details=Join-Path $logs 'r433-append-only-consumer-details.json';append_only_records=347;full_resource_pairs=271;new_initial_discovery_pairs=8;Native_catalog_proofs_90=8;Native_catalog_actor_proofs_92=8;typed_inventory_equipment=32;typed_READ_MOVE_receipts=12;EOF_no_response=32;Loading_actor_jobs=4;resources_loaded_or_partial_completed_in_this_consumer=$false;R432_actual_resource_partial_separate=$true;observed_DATA_world_metadata_verified=$true;historical_lost91_reconstructed=$false;historical_frozen_preservation=$historical;source_scope=$scope;Rust_live_SQL_TLS=$false;producer_SQL_TLS=$true;renderer_LOADED=$false;full_PreparedResources=$false;GUI=0;formal_known_extensions=255;formal_R408_changed=$false;qualified_modes=16;total_modes=40;pending=@('Original scheduler runtime phase/speed/brave/lifetime and full Layer8 insertion','Other classes/sex/headings/weapon appearance/combat loadout','Complete Native PreparedResources, formal live dispatch and player GUI')}
Write-ModernEvidenceJson -Path $resultPath -Value $result
$handoff=Join-Path $logs 'Native-wire-R433新SQL封包消費交接.txt'
$text=@"
R433 fresh append-only actualSQL90/91 and92/93 fixed consumer
Two new focused tests passed/0 failed; frozen host $($c.sha256), production library $librarySha; ten frozen source copies. Only test module wiring/test source added after R432; no production Native admission behavior changed.3436 own input manifest entries SHA/length/confinement reverified, no Java/SQL/TLS socket execution. R426–R432 frozen host/library/source copies and formal R408 preserved; historical R433a/b test source compile diagnostics preserved.
347 exact append-only records across four independent families: candidate90=78records/60resource pairs/7EOF, candidate92=95/75/9EOF, replay90=80/62/7EOF, replay92=94/74/9EOF. Every sequence1..N, exact key(sequence/session/source/requestID), session bounds,256record/12MiB wire cap and byte total checked. Full compatibility-vector prefix request/result/nested89 equals corresponding capture entry. Reused old source/action/requestID collision groups preserve all sequences; no map merge or overwrite.
271 complete resource pairs independently decoded with exact captured requests, echoed IDs, immutable catalog/actor identity, Native source and proof domains. Eight fresh main initial READ match0 discovery pairs (two sources per family) match directly selected discovery view and full91 or93→91→89; historical lost91 remains lost, not reconstructed.16 source/generation LOADED test-peer proofs (four per family) recomputed;90 uses Native+catalog domains,92 adds actor domain. ActualSQL source7/42 object10275 opaque epoch8000000000000000 retained. DATA owner world equals observed metadata; actual SCENE_ONLY equipment absent remains scope only.
32 typed inventory/equipment responses preserve29items/source. Perfamily source7 item10304 unequips with revision+1; source42 remains equipped.12 exact READ60/61 andMOVE64/65 paired records: eachfamily oneREAD and twoMOVE receipts (Applied/Rejected), foreign source42 Active guard refused. Four actualSQL92 Loading jobs stage exact opaque connection/session/source/fullDATA, connection replacement refused and retire cancels; this consumer does not load resources or complete partial.
32 EOF records have transmitted requests only, no fabricated response/nested89. Direct codec/authority refusal and valid byte proof sent across socket owner/old-generation are labelled separately; actual socket refusal remains producer evidence, no Rust live replay claim. Old-generation operation EOF is not invented as a Rejected65 response. Negative fixture sessions also contain valid setup exchanges;271 count includes all full validated setup pairs, not only main flow.
R432 realbody opaque partial qualifications stay separate (30focused/60synthetic-auth pairs/128negative). R433 does not repeat1.9GB body stream or promote renderer LOADED. Default-disabled1792 and formal KNOWN255 unchanged. FullPreparedResources/clock modifiers/Layer8/full class-sex-equipment/combat/liveplayerGUI pending;16/40/GUI0/R408 unchanged.
Result: $resultPath
Exact rows, sessions, proofs and EOF classifications: $($result.details)
R432 body and scheduler: $(Join-Path $logs 'Native-actor-R432依賴與scheduler交接.txt')
"@
[IO.File]::WriteAllText($handoff,$text,[Text.UTF8Encoding]::new($false))
$sp=Join-Path (Split-Path -Parent $project) '監製進度.json';$s=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json
$s.updated_at=(Get-Date).ToString('o');$s.phase='native_actor_and_fresh_sql_capture_verified_full_renderer_pending';$s.current_task='R432真實base body依賴/opaque partial與原版scheduler來源已凍結；R433新append-only真SQL90-91/92-93獨立consumer2passed/347records/271pairs/8freshdiscovery/16proofs/32EOF已驗。續作原版runtime clock/Layer8與完整Native renderer；LOADEDfalse/GUI0/16of40/R408unchanged。'
$s|Add-Member NoteProperty r433_wire_capture_work ([ordered]@{result=$resultPath;handoff=$handoff;records=347;resource_pairs=271;fresh_discovery=8;proofs=16;EOF=32;LOADED=$false}) -Force
$s.verified=@($s.verified)+@('R4332newfocused/3436fixedpins/347append-onlyrecords/271exactresourcepairs/8freshinitialdiscovery/16sourcegenproofs/32typedinventory/12READMOVE；32EOF不造response，历史lost91未重建；Rust无liveTLS/SQL，R432partial資格分開。')
Write-ModernEvidenceJson -Path $sp -Value $s
[IO.File]::AppendAllText((Join-Path $logs '持續驗證與缺口報告.txt'),"`r`nR433 $((Get-Date).ToString('o')) 2passed/347records/271pairs/8freshdiscovery/16proofs/32EOF；bodypartial資格分開，fullNative/LOADED/playerGUI pending。$handoff`r`n",[Text.UTF8Encoding]::new($false))
Write-Output 'R433 frozen2passed;347records;271pairs;8freshdiscovery;LOADEDfalse'
