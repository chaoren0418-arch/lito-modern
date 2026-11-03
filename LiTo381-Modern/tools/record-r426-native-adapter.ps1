param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$fixturePath=Join-Path $logs 'native-dm564-fixture-snapshot.json'
$fixture=Get-Content -LiteralPath $fixturePath -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $fixture.consumer_input).Hash -ne $fixture.consumer_input_sha256){throw 'Fixed Native fixture input drift'}
$candidatePath=Join-Path $logs 'r426-candidate.json'
$candidate=Get-Content -LiteralPath $candidatePath -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $candidate.path).Hash -ne $candidate.sha256){throw 'R426 candidate drift'}
$freeze=Split-Path -Parent $candidate.path
$sourceFolder=Join-Path $freeze 'source'
$null=New-Item -ItemType Directory -Path $sourceFolder -Force
foreach($pin in $candidate.source_pins) {
    if((Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256){throw ('R426 source drift: '+$pin.path)}
    $copy=Join-Path $sourceFolder (Split-Path -Leaf $pin.path)
    if(!(Test-Path -LiteralPath $copy)){[IO.File]::Copy($pin.path,$copy,$false)}
    if((Get-FileHash -LiteralPath $copy).Hash -ne $pin.sha256){throw 'R426 frozen source drift'}
    $pin|Add-Member -NotePropertyName snapshot -NotePropertyValue $copy -Force
}
$build=Get-Content -LiteralPath (Join-Path $logs 'r426-lib-build.jsonl')|ForEach-Object{$_|ConvertFrom-Json}
if(!($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success})){throw 'Production library build absent'}
$rlib=($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|Select-Object -Last 1).filenames|Where-Object{$_ -like '*.rlib'}|Select-Object -First 1
if(!$rlib){throw 'No production rlib'}
$library=Join-Path $freeze 'liblito_modern-r426.rlib'
if(!(Test-Path -LiteralPath $library)){[IO.File]::Copy($rlib,$library,$false)}
if((Get-FileHash -LiteralPath $library).Hash -ne (Get-FileHash -LiteralPath $rlib).Hash){throw 'Frozen library drift'}
$runs=@(
    @{name='adapter_unit';prefix='r426-adapter-unit';passed=3;ignored=1},
    @{name='actual_native564';prefix='r426-native564';passed=1;ignored=0},
    @{name='strict_til';prefix='r426-strict-til';passed=22;ignored=6},
    @{name='shared_sparse';prefix='r426-shared-sparse';passed=4;ignored=0}
)
$runRows=@(foreach($run in $runs) {
    $stdout=Join-Path $logs ($run.prefix+'.stdout.log');$stderr=Join-Path $logs ($run.prefix+'.stderr.log')
    $text=[IO.File]::ReadAllText($stdout)
    if($text -notmatch ('test result: ok\. '+$run.passed+' passed; 0 failed; '+$run.ignored+' ignored;')){throw ('Regression not passed: '+$run.name)}
    [ordered]@{name=$run.name;passed=$run.passed;ignored=$run.ignored;failed=0;stdout=$stdout;stdout_sha256=(Get-FileHash -LiteralPath $stdout).Hash;stderr=$stderr;stderr_sha256=(Get-FileHash -LiteralPath $stderr).Hash;completion='Synchronous owned command returned success; no SQL/server/GUI launched'}
})
$line=Get-Content -LiteralPath (Join-Path $logs 'r426-native564.stderr.log')|Where-Object{$_ -like 'NATIVE_ADAPTER_RESULT *'}|Select-Object -Last 1
if(!$line){throw 'No actual Native report'}
$report=$line.Substring('NATIVE_ADAPTER_RESULT '.Length)|ConvertFrom-Json
if($report.opacity_dynamic.status -ne 'passed_cpu_group_clock_fade_restore' -or $report.opacity_dynamic.rgba_hashes[0] -ne $report.opacity_dynamic.rgba_hashes[4]){throw 'Roof CPU result not established'}
$roofDirectory=[IO.File]::ReadAllText((Join-Path $logs 'r426-roof-output-path.txt'))
$images=@(Get-ChildItem -LiteralPath $roofDirectory -Filter '*.png' -File|ForEach-Object{[ordered]@{path=$_.FullName;bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName).Hash}})
if($images.Count -ne 5){throw 'Five roof CPU frames absent'}
$candidate.validation='3 adapter units +1 actual Native564 +22 strict TIL +4 Shared sparse passed; production lib built'
$candidate|Add-Member -NotePropertyName production_library -NotePropertyValue ([ordered]@{path=$library;sha256=(Get-FileHash -LiteralPath $library).Hash}) -Force
Write-ModernEvidenceJson -Path $candidatePath -Value $candidate
$resultPath=Join-Path $logs 'r426-native-adapter-consumer-result.json'
$result=[ordered]@{
    verified_at=[DateTimeOffset]::Now.ToString('o');validation_status='passed_default_disabled_pinned_native_adapter_cpu_regressions'
    candidate=$candidatePath;candidate_sha256=(Get-FileHash -LiteralPath $candidatePath).Hash
    binary=$candidate.path;binary_sha256=$candidate.sha256;production_library=$candidate.production_library;source_pins=$candidate.source_pins
    fixture_input=(Join-Path $project 'target\r425-native-dm564-fixture\consumer-input.json')
    fixture_input_sha256=(Get-FileHash -LiteralPath (Join-Path $project 'target\r425-native-dm564-fixture\consumer-input.json')).Hash
    immutable_fixture_snapshot=$fixturePath
    immutable_fixture_snapshot_sha256=(Get-FileHash -LiteralPath $fixturePath).Hash
    fixture_input_matches_R425=$true
    prior_assets_evidence=(Join-Path $logs 'r425-native-assets-consumer-result.json')
    runs=$runRows;report=$report;images=$images
    image_QA='Viewed complete0ms and451ms CPU images; source-layout coherent. Five RGBA hashes prove transitions and902ms exact restore. Not original framebuffer or a live GUI.'
    adapter_contract=[ordered]@{default_disabled=$true;namespace='Explicit local NativeLitoV1; not wire enum';map_id_source_inference=$false;shared_fallback=$false;full_pak_hash='64KiB cancellable stream on read-only path-verified handles';retained_full_paks=0;selected_entry_and_tile_budgets=$true;cancel_returns_partial_assets=$false;proof_scope='Local exact archive/sector/canonical pins; no authenticated match/generation or LOADED admission';shared_loader_changed=$false;shared_decoder_relaxed=$false}
    dynamic_scope=[ordered]@{opacity_records_linked=390;tested_roof_group=133;tested_roof_kind=0;tested_dynamic_groups=1;cpu_fade_restore=$true;all390_dynamic_combinations_qualified=$false;native_timer_alias_parity=$false;effects_records_retained=5;effect_action_or_playback_qualified=0;actor_depth_world_render_qualified=$false}
    interface_handoff=(Join-Path $logs 'Native-resource-adapter介面交接.txt')
    completed_modes=16;total_modes=40;mode_additions=0;GUI=0;wire_codec_implemented=$false;formal_R408_replaced=$false;whole_rotation_rerun=$false
}
Write-ModernEvidenceJson -Path $resultPath -Value $result
$statusPath=Join-Path (Split-Path -Parent $project) '監製進度.json'
$status=Get-Content -LiteralPath $statusPath -Raw|ConvertFrom-Json
$status.updated_at=$result.verified_at;$status.phase='native_adapter_candidate_verified_wire_contract_pending'
$status.current_task='R426預設關閉Native資源adapter及production rlib已固定：3守門/取消unit+1真Native564+22 strict TIL+4 Shared sparse全pass；32條目/3434slots靜態像素等值；390opacity接現有compositor，群組133 kind0 CPU淡化/復原pass，五effects資料保留、播放0。'
$status.next_step='Server固定wire向量/canonical整合待交接；R426本地介面與pins候選可供SourcePlan接入。原版C_D7/房間/說明診斷由別組處理；不改R408、不再重跑完整rotation。Native效果action/時序、其他屋頂群組/原版timeralias、正式controller/TLS/SQL/GUI仍待驗。'
$item='R426 reusable Native adapter/production rlib：預設關閉、固定namespace/roles/archive/sector/canonical pins、handle confinement、64KiB取消hash/entry及decode budget；30 tests passed（3+1+22+4）。Native564靜態RGBA等於R425；群組133 kind0 CPU淡化後902ms完全復原，390只資料接點／1動態sample，5effects播放0，mode16/40/GUI0。'
if($status.verified -notcontains $item){$status.verified=@($status.verified)+@($item)}
foreach($path in @($candidatePath,$resultPath,$result.interface_handoff)){if($status.evidence_paths -notcontains $path){$status.evidence_paths=@($status.evidence_paths)+@($path)}}
foreach($path in @($resultPath,$result.interface_handoff,$PSCommandPath)+@($candidate.source_pins.path)){if($status.changed_files -notcontains $path){$status.changed_files=@($status.changed_files)+@($path)}}
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output ([ordered]@{result=$resultPath;binary_sha256=$candidate.sha256;production_library=$library;tests_passed=30;roof_dynamic_groups=1;mode_additions=0}|ConvertTo-Json)
