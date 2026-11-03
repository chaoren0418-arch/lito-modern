param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Evidence([string]$name,[string]$token){
    $path=Join-Path $project ('src\modern_lito\'+$name)
    $matches=@(Select-String -LiteralPath $path -SimpleMatch $token)
    if(!$matches.Count){throw ('Renderer source evidence absent '+$name+' '+$token)}
    [ordered]@{path=$path;sha256=(Get-FileHash -LiteralPath $path).Hash;lines=@($matches|ForEach-Object{$_.LineNumber});literal=$token}
}
$evidence=@(
    (Evidence 'native_resources.rs' 'let _sprites = NativeArchive::open('),
    (Evidence 'native_resources.rs' 'pub fn effect_records(&self)'),
    (Evidence 'native_resources.rs' 'pub fn verified_plan(&self)'),
    (Evidence 'native_canonical_draft.rs' 'pub fn loaded_admission_available(&self)'),
    (Evidence 'battle_runtime.rs' 'Ok(Box::new(SharedScene::load_with_effects(root, observation,'),
    (Evidence 'battle_runtime.rs' '|| surface.effects_skipped'),
    (Evidence 'battle_runtime.rs' 'pub(crate) fn loaded_request('),
    (Evidence 'battle_scene.rs' 'pub(crate) struct PreparedResources'),
    (Evidence 'battle_scene.rs' 'WorldMap::load_shared(root, &input.bindings, cancel)?'),
    (Evidence 'battle_scene.rs' 'let catalog = SharedActionCatalog::load(root, cancel)?;'),
    (Evidence 'shared_action_catalog.rs' 'SharedArchiveKind::Text'),
    (Evidence 'shared_action_catalog.rs' 'archive.entry("list.spr")?'),
    (Evidence 'actor_assets.rs' 'UniqueAcrossShared'),
    (Evidence 'actor_assets.rs' 'It does not consult LT archives or native code.')
)
$r426path=Join-Path $logs 'r426-native-adapter-consumer-result.json'
$r426=Get-Content -LiteralPath $r426path -Raw|ConvertFrom-Json
$r425path=Join-Path $logs 'r425-native-assets-consumer-result.json'
$snapshotPath=Join-Path $logs 'r428-frozen-prototype-snapshot.json'
$snapshot=Get-Content -LiteralPath $snapshotPath -Raw|ConvertFrom-Json
$input=Get-Content -LiteralPath $snapshot.input -Raw|ConvertFrom-Json
$out=Join-Path $logs 'r428-native-renderer-support-audit.json'
$result=[ordered]@{
    audited_at=[DateTimeOffset]::Now.ToString('o');status='required_native_renderer_support_incomplete_LOADED_stays_disabled'
    audit_scope='Read-only source and existing pinned CPU evidence; no repeated rendering, SQL, server, GUI or full rotations'
    prototype_manifest=$input.manifest_sha256;actual_DATA_present=$false;source_evidence=$evidence
    prior_evidence=@(@{path=$r426path;sha256=(Get-FileHash -LiteralPath $r426path).Hash},@{path=$r425path;sha256=(Get-FileHash -LiteralPath $r425path).Hash})
    support=@(
        @{requirement='Native terrain/geometry exact namespace';state='LOCAL_CPU_QUALIFIED_R426';evidence='32TIL/3434selected slots/raw sectors/canonical/4archive pins and static RGBA already qualified; not repeated'},
        @{requirement='Native actor actions/heading/subframes/timing/depth';state='MISSING_REQUIRED_SOURCE_AND_BACKEND';evidence='Actor assets require Shared Text/list.spr and explicit/unique17 Shared Sprite archives. Native adapter retains terrain tiles, not native actor action snapshots. No pinned Native catalog/actions supplied in frozen prototype.'},
        @{requirement='Native Layer8 world effects';state='METADATA_AND_DECODER_SAMPLES_ONLY';evidence='R426 retains5 map effects and verifies LTsprite pins/index, then drops archive; no action/frame/duration playback owner. R425 two SPR11frame decode/frame0CPU samples do not establish world actions/timing.'},
        @{requirement='Native opacity/depth and renderer clock';state='ONE_CPU_GROUP_SAMPLE_ONLY';evidence='390 records linked; group133 kind0 fade/restore proved. Native timer alias, all390 combinations and actor/effect depth integration unqualified.'},
        @{requirement='Publication-specific actor/gear/NPC/skill/effect resource needs';state='ACTUAL_DATA_REQUIRED';evidence='Frozen batch has no canonical DATA. A matched manifest and proof do not enumerate or qualify actual publication resources.'},
        @{requirement='Sealed canonical completion and one attachment owner';state='LIVE_NATIVE_BACKEND_NOT_CONNECTED';evidence='PreparedResources only minted by SharedScene after complete rendering. Runtime LOADED also requires latest attachment, Loading scene/navigation and effects_skipped=false. Native draft owner exposes no LOADED permission.'}
    )
    next_independent_work=@('Obtain a pinned Native action/catalog provenance contract before creating bounded Native actor/effect snapshots; never reuse Shared catalog by map ID','Define a resource-completion certificate tied to SourceAttachment, admitted DATA, current canonical attachment and worker cancellation; keep unavailable until required resources succeed','Preserve existing Shared loader/gates and Native default-disabled access while DATA/service candidates are pending')
    dynamic_scope=$r426.dynamic_scope;renderer_LOADED_admission=$false;qualified_modes=16;mode_additions=0;GUI=0;formal_R408_replaced=$false
}
Write-ModernEvidenceJson -Path $out -Value $result
Write-Output ([ordered]@{audit=$out;source_evidence=$evidence.Count;LOADED=$false;renderer_backend='pending_pinned_native_actions_and_live_owner'}|ConvertTo-Json)
