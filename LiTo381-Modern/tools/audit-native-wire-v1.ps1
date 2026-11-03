param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$draft='D:\lineage開發\3-lito開發\lito伺服器\Native資源wire契約.md'
$output=Join-Path $project 'logs\continuation-20261003\native-wire-v1-compatibility-audit.json'
$sources=@(
    'protocol.rs','session_extensions.rs','world_protocol.rs','battle_world.rs',
    'application_protocol.rs','profile.rs','session.rs','transport.rs',
    'battle_operation.rs','resources.rs','tile_reader.rs'
)
$pins=@($sources | ForEach-Object {
    $path=Join-Path $project ('src\modern_lito\'+$_)
    [ordered]@{path=$path;bytes=(Get-Item -LiteralPath $path).Length;sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash}
})
function Evidence([string]$file,[string]$literal) {
    $path=Join-Path $project ('src\modern_lito\'+$file)
    $matches=@(Select-String -LiteralPath $path -SimpleMatch $literal)
    if($matches.Count -eq 0){throw "Required source evidence absent: $file $literal"}
    [ordered]@{path=$path;lines=@($matches | ForEach-Object {$_.LineNumber});literal=$literal}
}
$evidence=@(
    (Evidence 'protocol.rs' 'pub const MAX_VERSION: u16 = 47;'),
    (Evidence 'protocol.rs' 'pub const NUMERIC_SCHEMA_I64_V1: u16 = 1;'),
    (Evidence 'protocol.rs' 'pub const CAP_GM_BOTS: u32 = 256;'),
    (Evidence 'protocol.rs' 'b[4..8].copy_from_slice(&40u32.to_be_bytes());'),
    (Evidence 'protocol.rs' 'b[4..8].copy_from_slice(&28u32.to_be_bytes());'),
    (Evidence 'session_extensions.rs' 'pub const KNOWN: u32 = 255;'),
    (Evidence 'session_extensions.rs' 'ext & !KNOWN != 0'),
    (Evidence 'world_protocol.rs' 'pub const REQUEST_BYTES: usize = 104;'),
    (Evidence 'world_protocol.rs' '16 + REQUEST_BYTES'),
    (Evidence 'battle_world.rs' 'pub const REQUEST_BYTES: usize = 112;'),
    (Evidence 'battle_world.rs' 'pub const REQUEST_KIND: u16 = 66;'),
    (Evidence 'battle_world.rs' 'pub const RESULT_KIND: u16 = 67;'),
    (Evidence 'session.rs' 'controller.canonical_mode = true;'),
    (Evidence 'session.rs' 'controller.handshake = modern_hello(1);'),
    (Evidence 'session.rs' 'if connection.supports_battle_world() {'),
    (Evidence 'transport.rs' 'super::battle_world::encode_request'),
    (Evidence 'transport.rs' 'super::battle_world::read_reply_selected'),
    (Evidence 'profile.rs' 'pub const ALL: [Self; 7]'),
    (Evidence 'application_protocol.rs' 'world_protocol::MAX_FRAME_BYTES')
)
$draftText=[IO.File]::ReadAllText($draft)
foreach($token in @('v47/schema1','0x00000100','[104]','156','[252]','86','87')) {
    if(!$draftText.Contains($token)){throw "Draft version changed; inspect before updating audit: $token"}
}
$domain=[Text.Encoding]::ASCII.GetBytes('LITO-WORLD-RESOURCES'+[char]0)
if($domain.Length -ne 21){throw 'Unexpected digest domain length'}
$result=[ordered]@{
    audited_at=[DateTimeOffset]::Now.ToString('o')
    status='draft_review_complete_cross_language_vectors_and_canonical_dispatch_agreement_pending'
    scope='Read-only draft and Rust source audit; no codec/runtime/profile implementation or live acceptance'
    draft=[ordered]@{path=$draft;bytes=(Get-Item -LiteralPath $draft).Length;sha256=(Get-FileHash -LiteralPath $draft -Algorithm SHA256).Hash}
    source_pins=$pins
    source_evidence=$evidence
    compatible=[ordered]@{
        base_version=47;numeric_schema=1;header_bytes=16;extended_hello_kind=80;extended_hello_full_bytes=40
        extended_ack_kind=81;extended_ack_full_bytes=28;native_extension_bit=256
        separate_words='extension 0x100 does not collide with base CAP_GM_BOTS 0x100'
        resource_namespace=2;resource_schema=1;archive_pin_bytes=44;archive_count=4
        source_plan_bytes=(2+2+32+4+32+2+2+4*44)
        digest_domain_bytes=$domain.Length;digest_preimage_bytes=($domain.Length+2+2+4+32+2+2+4*44)
    }
    differences=@(
        [ordered]@{
            id='request_length_wording';severity='contract_clarification'
            draft='REQUEST payload total156 wrapping unchanged WORLD body104'
            rust='WORLD REQUEST_BYTES104 means body; existing full request is120'
            calculation='2+2+32+104=140 payload;16+140=156 whole Native request'
            required_resolution='Fixed vector/spec must label156 as whole frame,140 as payload; embedded body excludes LTM1 header'
        },
        [ordered]@{
            id='extension_dependency_masks';severity='implementation_gap'
            draft='Native only requires stats/source identities/navigation/character data/rooms/world; existing extensions retain full Battle dependencies'
            rust='KNOWN255 rejects Native bit; any nonzero extension currently requires full DEPENDENCIES'
            native_base_dependencies_hex='0x0000006F'
            required_resolution='Add explicit known Native bit and validate dependencies per selected mask; retain old closure rules and full dependencies for old bits; unknown bits reject'
        },
        [ordered]@{
            id='formal_v47_canonical_dispatch';severity='requires_server_modern_agreement'
            draft='kinds86/87 wrap plain WORLD20/21 body104 and legacy world result'
            rust='Formal schema2/modern_v47 SessionController selects canonical state; live lifecycle is66/67 request112 full/96 body plus attachment-owned DATA; MOVE uses canonical operations'
            required_resolution='Define how SourcePlan and resource proof bind canonical PREPARE/LOADED/READ/LEAVE, DATA/attachments and later mutation/life requests. A20/21 wrapper alone is not formal PLAYER integration. Keep both existing codecs strict; do not synthesize legacy WorldFrame or downgrade controller'
        },
        [ordered]@{
            id='result_allocation_and_exact_bounds';severity='contract_clarification'
            draft='36 echo +252 SourcePlan + unchanged WORLD result body; no explicit total result bound'
            rust='WORLD result maximum131072 includes16-byte header; application reader uses per-kind/version/format whitelist'
            conditional_calculation='If unchanged legacy body limit131056 remains, Native full maximum=16+36+252+131056=131360; this is derived, not agreed limit'
            required_resolution='Publish exact minimum/maximum full lengths and vector lengths. Add selected-extension-specific kind87 limits before allocation without relaxing old kinds; canonical result limits require separate agreement'
        },
        [ordered]@{
            id='profile_loader_loaded_proof';severity='implementation_gap'
            draft='Selected Native bit, fixed namespace and SourcePlan derived bounded adapters; full proof before LOADED'
            rust='Seven existing profile extensions; no Native selection/plan/loader dispatch/proof. R425 Native adapter is standalone diagnostic only'
            required_resolution='Explicit opt-in profile and negotiated gate, fixed role namespace, resource-set hash and raw archive/sector/canonical pins, identity-bound proof retirement on stale generation; no map-ID routing or Shared fallback'
        }
    )
    fixed_vectors=[ordered]@{received=$false;cross_language_verified=$false;needed=@(
        'HELLO/ACK Native alone, combined old extensions, missing dependency, unknown extension and unselected Native',
        'Native request/result full bytes, exact bounds, request ID, trailing bytes and schema/namespace failures',
        'Resource-set digest preimage and four ordered archive pins; missing/extra/duplicate/wrong role/profile/size/hash/reserved failures',
        'READ zero digest without match; matched READ and PREPARE/LOADED/MOVE/LEAVE exact digest requirements',
        'Native versus Shared binding, raw-sector/canonical/archive proof, stale match/generation and no legacy mutation bypass',
        'Canonical integration agreement or explicit declaration that initial86/87 fixture is only a codec/legacy world diagnostic'
    )}
    preserved=[ordered]@{formal_R408_replaced=$false;authority_JAR_replaced=$false;shared_default_codec_relaxed=$false;wire_implementation_changed=$false;qualified_modes=16;total_modes=40;new_native_modes=0;gui=0}
}
Write-ModernEvidenceJson -Path $output -Value $result
Write-Output ([ordered]@{output=$output;draft_sha256=$result.draft.sha256;differences=$result.differences.Count;source_pins=$pins.Count;source_evidence=$evidence.Count}|ConvertTo-Json)
