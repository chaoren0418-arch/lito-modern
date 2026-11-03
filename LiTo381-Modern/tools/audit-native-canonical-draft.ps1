param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$draft='D:\lineage開發\3-lito開發\lito伺服器\Native資源canonical整合契約.md'
$names=@('battle_world.rs','battle_read.rs','battle_action.rs','battle_operation.rs','battle_payload.rs','battle_payload_context.rs','battle_state.rs','battle_player.rs','battle_runtime.rs','transport.rs','session_extensions.rs','native_resources.rs')
$pins=@($names|ForEach-Object{$p=Join-Path $project ('src\modern_lito\'+$_);[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}})
$findings=@(
 @{id='attachment_presence';resolution='Freeze SourceAttachment presence for denied/busy/unavailable, aborted, successful LEAVE, detached READ with binding and wrong expected digest. Draft reader uses a conservative local matrix; not Java agreement.'},
 @{id='manifest_complete_equality';resolution='Compare map, canonical digest and every ordered sector coordinate/length/raw SHA against DATA.manifest. Wire40-byte sectors and DATA44-byte sectors compare typed values, never bytes or digest alone.'},
 @{id='before_engine_bounds';resolution='Apply CAP_LARGE_MAPS above16 sectors before loading even without DATA; cap sum sector sizes32MiB in addition to64 sectors and8MiB each. Local draft sorts lexicographically x then y; Java fixed vectors must confirm order.'},
 @{id='epoch_bits';resolution='Existing canonical epoch is opaque nonzero64-bit, including high bit. Preserve its BE bit pattern in i64 proof encoding; do not impose positive signed epoch. Source/object and world IDs retain existing predicates.'},
 @{id='pending_room_binding';resolution='No-match READ has room0; attachment owner must also capture current authenticated connection incarnation and joined room/revision outside SourcePlan, retire on navigation changes, then require exact PREPARE room/revision under authority fence.'},
 @{id='renderer_loaded_support';resolution='R426 proves local terrain pins and bounded static assets, not Native world actor/effect resource support. No owner API may mint LOADED proof from verified_plan alone. Required renderer support and canonical loading publication remain blockers.'},
 @{id='atomic_owner';resolution='Native source attachment, DATA, worker ticket and verified assets need one canonical attachment owner/fence. Reject stale worker completion after retire/change; draft interfaces remain disconnected from live Tracker and transport.'},
 @{id='selected_extensions';resolution='Production KNOWN255 currently rejects Native256; use separate explicitly enabled local draft gate. Full canonical dependencies required in addition to0x6F. Native/full-VIP lifecycle combination rejects without stripping negotiated appearance.'},
 @{id='server_authority_ledger';resolution='All actual60/61 reads,62/63 ACTION_LIFE,64/65 operations,68/69 revival and other mutation routes need enqueue and publication ledger gates; no plain20/21,66/67 or84/85 Native bypass. Rust codec cannot prove Java fence behavior.'}
)
$out=Join-Path $project 'logs\continuation-20261003\native-canonical-draft-compatibility-audit.json'
$audit=[ordered]@{audited_at=[DateTimeOffset]::Now.ToString('o');status='draft_local_interfaces_preparing_java_fixed_vectors_pending';draft=@{path=$draft;sha256=(Get-FileHash -LiteralPath $draft).Hash;bytes=(Get-Item -LiteralPath $draft).Length};source_pins=$pins;compatible=@{request_full=180;result_min=244;source_attachment_min=340;source_attachment_max=2860;before_engine_min=584;result_max_fixed_overhead=3104;canonical_body_request=96;metadata=56;index_max=8MB;pak_max=256MB;current_draft_kind_names_match_rust=$true};findings=$findings;java_fixed_vectors_received=$false;local_examples_are_java_vectors=$false;live_dispatch_changed=$false;formal_R408_replaced=$false;qualified_modes=16;total_modes=40;GUI=0}
Write-ModernEvidenceJson -Path $out -Value $audit
$statusPath=Join-Path (Split-Path -Parent $project) '監製進度.json'
$status=Get-Content -LiteralPath $statusPath -Raw|ConvertFrom-Json
$status|Add-Member -NotePropertyName native_canonical_draft_audit -NotePropertyValue $out -Force
$status|Add-Member -NotePropertyName native_canonical_phase -NotePropertyValue 'isolated_default_disabled_codec_owner_preparation_java_vectors_pending' -Force
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output ($audit|ConvertTo-Json -Depth 8)
