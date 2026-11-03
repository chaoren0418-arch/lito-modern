param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$candidatePath=Join-Path $logs 'r428-candidate.json'
$candidate=Get-Content -LiteralPath $candidatePath -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $candidate.path).Hash -ne $candidate.sha256){throw 'R428 binary drift'}
foreach($pin in $candidate.source_pins){foreach($p in @($pin.path,$pin.snapshot)){if((Get-FileHash -LiteralPath $p).Hash -ne $pin.sha256){throw 'R428 source drift'}}}
$snapshotPath=Join-Path $logs 'r428-frozen-prototype-snapshot.json'
$snapshot=Get-Content -LiteralPath $snapshotPath -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $snapshot.input).Hash -ne $snapshot.input_sha256){throw 'R428 input drift'}
$input=Get-Content -LiteralPath $snapshot.input -Raw|ConvertFrom-Json
$verified=0
foreach($pin in $input.files){$p=Join-Path $input.root $pin.path;if((Get-Item -LiteralPath $p).Length -ne $pin.bytes -or (Get-FileHash -LiteralPath $p).Hash -ne $pin.sha256){throw ('Copied frozen input drift: '+$pin.path)};$verified++}
if($verified -ne 272){throw 'Frozen snapshot file count mismatch'}
$runs=@(foreach($run in @(@{name='local';prefix='r428-local';passed=14;ignored=1},@{name='fixed_java_prototype';prefix='r428-prototype';passed=1;ignored=0})){
    $stdout=Join-Path $logs ($run.prefix+'.stdout.log');$stderr=Join-Path $logs ($run.prefix+'.stderr.log')
    if([IO.File]::ReadAllText($stdout) -notmatch ('test result: ok\. '+$run.passed+' passed; 0 failed; '+$run.ignored+' ignored;')){throw ('Test not passed '+$run.name)}
    [ordered]@{name=$run.name;passed=$run.passed;failed=0;ignored=$run.ignored;stdout=$stdout;stdout_sha256=(Get-FileHash -LiteralPath $stdout).Hash;stderr=$stderr;stderr_sha256=(Get-FileHash -LiteralPath $stderr).Hash;completion='Owned synchronous command exited0; no Java/service/SQL/TLS/GUI launched'}
})
$line=Get-Content -LiteralPath (Join-Path $logs 'r428-prototype.stderr.log')|Where-Object{$_ -like 'NATIVE_CANONICAL_PROTOTYPE_RESULT *'}|Select-Object -Last 1
if(!$line){throw 'Prototype consumer report absent'}
$probe=$line.Substring('NATIVE_CANONICAL_PROTOTYPE_RESULT '.Length)|ConvertFrom-Json
if($probe.requests.Count -ne 6 -or $probe.pre_engine_replies.Count -ne 8 -or $probe.proofs.Count -ne 2 -or $probe.negative_equivalents.Count -ne 43){throw 'Prototype coverage mismatch'}
$groups=@($probe.negative_equivalents|Group-Object category|ForEach-Object{[ordered]@{category=$_.Name;count=$_.Count}})
$proxies=@($probe.negative_equivalents|Where-Object{$_.category -eq 'draft_gate_equivalent_not_live_application_dispatch'}).Count
if($proxies -ne 4){throw 'Expected four explicit gate-only proxies'}
$probe|Add-Member -NotePropertyName directly_checked_codec_claim_metadata_selection -NotePropertyValue (43-$proxies) -Force
$probe|Add-Member -NotePropertyName default_disabled_gate_proxy_checks -NotePropertyValue $proxies -Force
$probe|Add-Member -NotePropertyName negative_breakdown -NotePropertyValue $groups -Force
$detailPath=Join-Path $logs 'r428-java-prototype-consumer-details.json'
Write-ModernEvidenceJson -Path $detailPath -Value $probe
$build=Get-Content -LiteralPath (Join-Path $logs 'r428-lib-build.jsonl')|ForEach-Object{$_|ConvertFrom-Json}
if(!($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success})){throw 'Production library not built'}
$rlib=($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|Select-Object -Last 1).filenames|Where-Object{$_ -like '*.rlib'}|Select-Object -First 1
$library=Join-Path (Split-Path -Parent $candidate.path) 'liblito_modern-r428.rlib'
if(!(Test-Path -LiteralPath $library)){[IO.File]::Copy($rlib,$library,$false)}
if((Get-FileHash -LiteralPath $library).Hash -ne (Get-FileHash -LiteralPath $rlib).Hash){throw 'Frozen production library drift'}
foreach($name in @('r426-candidate.json','r427-candidate.json')){
    $prior=Get-Content -LiteralPath (Join-Path $logs $name) -Raw|ConvertFrom-Json
    foreach($p in @(@{path=$prior.path;sha256=$prior.sha256},$prior.production_library)){if((Get-FileHash -LiteralPath $p.path).Hash -ne $p.sha256){throw ('Historical frozen candidate drift '+$name)}}
}
$candidate.validation='14 local +1 fixed Java prototype consumer passed;6requests/8pre-engine replies/2proofs and39 direct negative checks +4 default-disabled proxies; non-test lib built'
$candidate|Add-Member -NotePropertyName production_library -NotePropertyValue ([ordered]@{path=$library;sha256=(Get-FileHash -LiteralPath $library).Hash}) -Force
Write-ModernEvidenceJson -Path $candidatePath -Value $candidate
$rendererPath=Join-Path $logs 'r428-native-renderer-support-audit.json'
$renderer=Get-Content -LiteralPath $rendererPath -Raw|ConvertFrom-Json
foreach($e in $renderer.source_evidence){if((Get-FileHash -LiteralPath $e.path).Hash -ne $e.sha256){throw 'Renderer audit source drift'}}
$out=Join-Path $logs 'r428-native-canonical-prototype-consumer-result.json'
$result=[ordered]@{
    verified_at=[DateTimeOffset]::Now.ToString('o');status='passed_fixed_codec_prototype_consumer_not_formal_PLAYER_wire'
    candidate=$candidatePath;candidate_sha256=(Get-FileHash -LiteralPath $candidatePath).Hash;binary=$candidate.path;binary_sha256=$candidate.sha256;production_library=$candidate.production_library;source_pins=$candidate.source_pins
    frozen_snapshot=$snapshotPath;input=$snapshot.input;input_sha256=$snapshot.input_sha256;files_verified=272
    manifest_sha256=$input.manifest_sha256;vector_sha256=$input.vector_sha256;specification_sha256=$input.specification_sha256
    runs=$runs;tests_passed=15;requests_exact_roundtrip=6;pre_engine_results_consumed=8;load_proofs_matched=2
    prototype_details=$detailPath;prototype_details_sha256=(Get-FileHash -LiteralPath $detailPath).Hash
    negative_scope=@{wire_mutations_and_lengths=28;claim=4;metadata=3;selection=4;direct_checks_total=39;default_disabled_gate_proxy_checks=4;producer_ledger_state_assertions_not_proved_by_bytes=12;producer_total_assertions=55;all55_direct_bad_HEX_published=$false;java_live_application_dispatch_verified=$false}
    contract_alignment=@{nonzero_expected_requires_source=$true;prepare_denied_busy_unavailable_detached_source_present=$true;leave_success_detached_pre_retirement_source_present=$true;bootstrap_zero_no_source_OK_or_unavailable=$true;source_echo_does_not_start_error_abort_leave_worker=$true;sector_order='x then y';minimum_sector_bytes=49158;map_zero_codec_allowed=$true;opaque_epoch_high_bit_preserved=$true;proof_preimage_bytes=135;full_source_DATA_manifest_equality=$true}
    renderer_support_audit=$rendererPath;renderer_support_audit_sha256=(Get-FileHash -LiteralPath $rendererPath).Hash
    boundaries=@{frozen_scope='Codec prototype only; no actual canonical DATA in batch';mutable_server_source_consumed=$false;default_disabled=$true;LOADED_owner_admission=$false;legacy_WorldFrame_created=$false;live_profile_transport_controller_changed=$false;actual_DATA_cross_language_verified=$false;service_ledger_generation_fence_verified=$false;TLS_SQL_verified=$false;formal_R408_replaced=$false;whole_rotation_rerun=$false;qualified_modes=16;total_modes=40;mode_additions=0;GUI=0}
    next=@('Pinned Native action/catalog provenance and Native actor/effect timing/depth backend','Actual Java canonical DATA vector and required resource support tied to that publication','One live canonical owner/cancellation/generation fence and service ledger integration','TLS/SQL consumer and GUI acceptance after those prerequisites')
}
Write-ModernEvidenceJson -Path $out -Value $result
$statusPath=Join-Path (Split-Path -Parent $project) '監製進度.json'
$status=Get-Content -LiteralPath $statusPath -Raw|ConvertFrom-Json
$status.updated_at=$result.verified_at;$status.phase='native_canonical_frozen_codec_prototype_verified_renderer_support_pending'
$status.current_task='R428 frozen prototype272檔snapshot核對；14本地+1跨語言consumer pass，6request/8reply/2proof相符；39 direct拒絕+4關閉gate代理，12ledger不能由bytes證明。Native renderer14處source核對：actor/catalog/effect/depth/完整certificate未接通，LOADED仍false。'
$status.next_step='等待Native action/catalog固定來源身分與實際canonical DATA；先補必要Native actor/effect renderer support及單一attachment/generation fence，再服務ledger/TLS/SQL。現有Shared proof不能轉授Native，R408/16of40/GUI0保持，不重跑rotation。'
$status.native_canonical_phase='fixed_codec_prototype_consumed_renderer_loaded_unqualified'
foreach($pair in @(@{name='native_canonical_prototype_consumer';value=$out},@{name='native_renderer_support_audit';value=$rendererPath},@{name='native_canonical_previous_pin_drift_resolved';value='Frozen272-file manifestB09515C... consumed; mutable source drift evidence retained historically'})){$status|Add-Member -NotePropertyName $pair.name -NotePropertyValue $pair.value -Force}
$item='R428固定Java codec prototype同bytes：272pins自有snapshot；6requests精確重編碼、8pre-engine results/附件、2種epoch proof通過；14local+1consumer=15passed。28wire+4claim+3metadata+4selection direct checks39，4default-off代理另列、12ledger state不由bytes宣告；presence矩陣與49158/min/map0對齊。Renderer14 sourcepoints核對仍缺Native action/actor/effect/depth/full completion，Loaded=false、16/40、GUI0。'
if($status.verified -notcontains $item){$status.verified=@($status.verified)+@($item)}
foreach($p in @($out,$detailPath,$rendererPath,$snapshotPath,$candidatePath)){if($status.evidence_paths -notcontains $p){$status.evidence_paths=@($status.evidence_paths)+@($p)}}
foreach($p in @($candidate.source_pins.path)+@($PSCommandPath)){if($status.changed_files -notcontains $p){$status.changed_files=@($status.changed_files)+@($p)}}
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output ([ordered]@{result=$out;binary_sha256=$candidate.sha256;tests_passed=15;requests=6;replies=8;proofs=2;direct_negative_checks=39;gate_proxies=4;ledger_state_cases_excluded=12;LOADED=$false;qualified_modes=16;GUI=0}|ConvertTo-Json)
