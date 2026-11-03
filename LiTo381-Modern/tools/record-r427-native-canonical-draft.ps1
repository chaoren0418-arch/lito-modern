param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$candidatePath=Join-Path $logs 'r427-candidate.json'
$candidate=Get-Content -LiteralPath $candidatePath -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $candidate.path).Hash -ne $candidate.sha256){throw 'R427 binary drift'}
foreach($pin in $candidate.source_pins){
    foreach($p in @($pin.path,$pin.snapshot)){if((Get-FileHash -LiteralPath $p).Hash -ne $pin.sha256){throw 'R427 source drift'}}
}
$auditPath=Join-Path $logs 'native-canonical-draft-compatibility-audit.json'
$audit=Get-Content -LiteralPath $auditPath -Raw|ConvertFrom-Json
foreach($pin in $audit.source_pins){if((Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256){throw 'Audited existing source drift'}}
$runs=@(foreach($r in @(@{prefix='r427b-local';passed=13},@{prefix='r427b-canonical-regression';passed=6},@{prefix='r427b-extension-regression';passed=6})){
    $stdout=Join-Path $logs ($r.prefix+'.stdout.log');$stderr=Join-Path $logs ($r.prefix+'.stderr.log')
    if([IO.File]::ReadAllText($stdout) -notmatch ('test result: ok\. '+$r.passed+' passed; 0 failed; 0 ignored;')){throw ('Run did not pass '+$r.prefix)}
    [ordered]@{name=$r.prefix;passed=$r.passed;failed=0;stdout=$stdout;stdout_sha256=(Get-FileHash -LiteralPath $stdout).Hash;stderr=$stderr;stderr_sha256=(Get-FileHash -LiteralPath $stderr).Hash;completion='Owned synchronous command exited0, no SQL/Java/GUI started'}
})
$build=Get-Content -LiteralPath (Join-Path $logs 'r427b-lib-build.jsonl')|ForEach-Object{$_|ConvertFrom-Json}
if(!($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success})){throw 'Production library not built'}
$rlib=($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|Select-Object -Last 1).filenames|Where-Object{$_ -like '*.rlib'}|Select-Object -First 1
if(!$rlib){throw 'Production library artifact absent'}
$library=Join-Path (Split-Path -Parent $candidate.path) 'liblito_modern-r427b.rlib'
if(!(Test-Path -LiteralPath $library)){[IO.File]::Copy($rlib,$library,$false)}
if((Get-FileHash -LiteralPath $library).Hash -ne (Get-FileHash -LiteralPath $rlib).Hash){throw 'Frozen library drift'}
$exampleLine=Get-Content -LiteralPath (Join-Path $logs 'r427b-local.stderr.log')|Where-Object{$_ -like 'NATIVE_CANONICAL_LOCAL_EXAMPLES *'}|Select-Object -Last 1
if(!$exampleLine){throw 'Local examples absent'}
$examples=$exampleLine.Substring('NATIVE_CANONICAL_LOCAL_EXAMPLES '.Length)|ConvertFrom-Json
foreach($p in @('request88','detached89','before_engine89','data89')){
    $bytes=[Convert]::FromHexString($examples.($p+'_hex'))
    if($bytes.Length -ne $examples.($p+'_full_bytes')){throw 'Example byte count mismatch'}
}
foreach($p in @('resource_set','load_proof')){
    $bytes=[Convert]::FromHexString($examples.($p+'_preimage_hex'))
    if($bytes.Length -ne $examples.($p+'_preimage_bytes')){throw 'Preimage byte count mismatch'}
    $hash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
    if($hash -ne $examples.($p+'_digest_hex')){throw 'Example SHA mismatch'}
}
$examplePath=Join-Path $logs 'r427-rust-local-synthetic-examples.json'
Write-ModernEvidenceJson -Path $examplePath -Value $examples
$r426=Get-Content -LiteralPath (Join-Path $logs 'r426-candidate.json') -Raw|ConvertFrom-Json
foreach($pin in @(@{path=$r426.path;sha256=$r426.sha256},$r426.production_library)){
    if((Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256){throw 'R426 frozen evidence drift'}
}
$candidate.validation='13 local Native draft +6 unchanged canonical +6 unchanged extension tests passed; production lib built'
$candidate|Add-Member -NotePropertyName production_library -NotePropertyValue ([ordered]@{path=$library;sha256=(Get-FileHash -LiteralPath $library).Hash}) -Force
Write-ModernEvidenceJson -Path $candidatePath -Value $candidate
$resultPath=Join-Path $logs 'r427-native-canonical-draft-result.json'
$handoff=Join-Path $logs 'Native-canonical-draft介面交接.txt'
$result=[ordered]@{
    verified_at=[DateTimeOffset]::Now.ToString('o');status='passed_isolated_default_disabled_rust_local_draft_java_vectors_pending'
    candidate=$candidatePath;candidate_sha256=(Get-FileHash -LiteralPath $candidatePath).Hash
    binary=$candidate.path;binary_sha256=$candidate.sha256;production_library=$candidate.production_library;source_pins=$candidate.source_pins
    audit=$auditPath;audited_draft_sha256=$audit.draft.sha256;current_draft_sha256=(Get-FileHash -LiteralPath $audit.draft.path).Hash
    draft_unchanged_since_audit=((Get-FileHash -LiteralPath $audit.draft.path).Hash -eq $audit.draft.sha256)
    runs=$runs;tests_passed=25;local_examples=$examplePath;local_examples_sha256=(Get-FileHash -LiteralPath $examplePath).Hash
    local_example_origin='Synthetic Rust fixtures: no Java fixed vectors, no real Native pin ownership, no live HELLO/ACK or accepted server ledger'
    interface_handoff=$handoff;interface_handoff_sha256=(Get-FileHash -LiteralPath $handoff).Hash
    boundaries=@{crate_private=$true;default_disabled=$true;live_profile_or_transport_changed=$false;production_extension_KNOWN=255;canonical_DTO_parser_reused=$true;legacy_WorldFrame_created=$false;whole_manifest_compared=$true;identity_proof_preimage_bytes=135;proof_domain_bytes=23;opaque_epoch_high_bit_supported=$true;owner_cancel_on_replace_retire_drop=$true;LOADED_owner_admission_available=$false;server_ledger_fence_verified=$false;java_fixed_vectors_received=$false;java_interop_verified=$false;TLS_SQL_verified=$false;qualified_modes=16;total_modes=40;mode_additions=0;GUI=0;formal_R408_replaced=$false;whole_rotation_rerun=$false}
    remaining=@('Java exact vectors and complete SourceAttachment presence/order/epoch contract','Renderer required Native actor/effect support before LOADED','Integration into one live canonical owner and generation fence','Server ledger enqueue/publication and legacy/full-VIP bypass rejection','True TLS/SQL consumer and GUI acceptance')
    preliminary_failures='Preserved preliminary logs: synthetic DATA schema mismatch, proof preimage byte-count assertion and over-limit test-header construction corrected. Successful fixed candidate supersedes these local fixture failures.'
}
Write-ModernEvidenceJson -Path $resultPath -Value $result
$audit.status='local_draft_interfaces_verified_java_vectors_and_live_owner_fence_pending'
$audit|Add-Member -NotePropertyName local_result -NotePropertyValue $resultPath -Force
Write-ModernEvidenceJson -Path $auditPath -Value $audit
$statusPath=Join-Path (Split-Path -Parent $project) '監製進度.json'
$status=Get-Content -LiteralPath $statusPath -Raw|ConvertFrom-Json
$status.updated_at=$result.verified_at;$status.phase='native_canonical_local_draft_verified_java_vectors_pending'
$status.current_task='R427獨立預設關閉88/89 canonical codec/owner草稿已固定：13本地正反例+6 canonical+6 extensions，25pass；135-byte LOAD proof preimage與Rust合成HEX已交接，非Java向量或TLS/SQL/GUI。'
$status.next_step='等待Server固定88/89跨語言正負bytes、SourceAttachment presence/排序/epoch語義；Native renderer必要support及單一canonical owner/generation fence/ledger接線完成前不開LOADED。R408不替換、完整rotation不重跑，16/40與GUI0。'
$status|Add-Member -NotePropertyName native_canonical_result -NotePropertyValue $resultPath -Force
$status.native_canonical_phase='local_codec_owner_verified_java_vectors_pending'
$item='R427 crate-private/default-disabled canonical88/89 codec/owner：13 Rust合成正反例+6既有canonical+6extension regression=25pass。SourceAttachment完整pins/Manifest、preallocation selected bound、opaque高位epoch、owner序號/取消/退休守門；Loaded owner admission=false，沒有live dispatch/Java互通/Native模式新增。'
if($status.verified -notcontains $item){$status.verified=@($status.verified)+@($item)}
foreach($p in @($resultPath,$candidatePath,$auditPath,$examplePath,$handoff)){if($status.evidence_paths -notcontains $p){$status.evidence_paths=@($status.evidence_paths)+@($p)}}
foreach($p in @($candidate.source_pins.path)+@($PSCommandPath,$handoff)){if($status.changed_files -notcontains $p){$status.changed_files=@($status.changed_files)+@($p)}}
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output ([ordered]@{result=$resultPath;binary_sha256=$candidate.sha256;tests_passed=25;java_vectors_verified=$false;LOADED_admission=$false;qualified_modes=16;GUI=0}|ConvertTo-Json)
