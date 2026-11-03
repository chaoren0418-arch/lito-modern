param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$snap=Get-Content (Join-Path $logs 'r445-external-control-v2-snapshot.json') -Raw|ConvertFrom-Json
$adapter=Join-Path $snap.candidate.root 'source\main\NativePlayerLoadoutAdapter.java'
$pins=@(foreach($p in @($adapter,(Join-Path $project 'src\modern_lito\application_protocol.rs'),(Join-Path $project 'src\modern_lito\character_data.rs'),(Join-Path $project 'src\modern_lito\native_appearance_inspection.rs'),(Join-Path $logs 'r445-v2-same-work-result.json'))){[ordered]@{path=$p;sha256=(Get-FileHash $p).Hash;bytes=(Get-Item $p).Length}})
$result=[ordered]@{
 state='EXACT_MINIMUM_LIVE_APPEARANCE_DEPENDENCIES_OBSERVATION_FIRST_NO_AUTHORITY_GRANT'
 updated_at=[DateTimeOffset]::Now.ToString('o');source_pins=$pins
 current=@(
  'Actual selected source/object/opaque epoch/channel and registered-reader private generation exist;R445 Hold/terminal correlate the actual Work/Operation and deadline.',
  'Roster CharacterSummary.class is profession/Type, explicitly NOT sprite Class;sex is already present. CharacterData/equipment source retains the same selected CharacterSummary.',
  'Actual equipment flags/revision and selected effective weapon are available. Physical loadout has qualified retained weapon Type;diagnostic v2 exposes revision/weapon object but not all appearance source fields.',
  'SQL Class/body logical row in R439 NAB was separately observed historically;it is NOT a live same-Work appearance snapshot. R445 does not observe active graphics/transformation or issue an appearance capability.'
 )
 minimal_base_only=@(
  'Authoritative same-read persisted render Class/logical actor row, distinct from profession Type;for existing bounded matrix admit only exact Type/Sex/Class tuples0/0/0 or3/0/734.',
  'Type/Sex from the very same source bundle or an immutable bound reference to it;no second roster peek joined to a different Class read.',
  'Authoritative explicit absence of active graphics/transformation override during snapshot validity. Unknown override state must reject;base Class alone is insufficient.',
  'Effective weapon object/retained Type/equipped state and equipment revision from the same reconciliation;source persisted equipped flag is insufficient.',
  'Pinned catalog/action/member dependency identity. Existing NAD maps logical0→graphics3225 and734→3476;do not request duplicate alias graphics if that exact mapping remains pinned.'
 )
 active_override_alternative=@(
  'If overrides are in scope, provide source-qualified effective logical actor row/action family and override-active state from the authoritative appearance store, plus appearance revision and exclusive override cutoff.',
  'Type/Sex/Class are needed as derivation inputs only if the effective row is computed from them;an authoritative effective row with complete derivation/binding may avoid duplicate fields. Numeric activegfx alone without defined logical-row versus image-bank semantics is insufficient.',
  'Unsupported effective row/member/action family is rejected;receiving an override value does not widen current renderer catalog coverage.'
 )
 snapshot_and_commit=@(
  'Inside the actual authenticated registered selected-reader Work and its actual Operation/ModernBackendScope, read appearance and physical source under the same principal/source/object. Verify selected Type/Sex and source identity;bind render Class/override fields to owner material or a checked appearance revision.',
  'Reconcile the same loaded physical bundle with the real owner-bound equipment store.read at query observation time. Capture appearance state/revision/cutoff in that same Work;never substitute request revision or diagnostic UUID for owner capability.',
  'Before publication recheck current private attachment/expected generation/selection, backend deadline/cancellation, appearance revision and override/source/item exclusive cutoffs. Reject source changed or desired active item/override expired after read;do not silently turn it into a different base/unarmed appearance.',
  'Use the existing real Operation.matchesSelection/matchesKey and tryCommit gate;only the producer holding the private selected owner/lease may grant a narrowly scoped publication. Work operation UUID/private generation are observation fields, not capabilities.',
  'For retention beyond one committed observation require real owner/appearance/equipment revision invalidation and cutoff checks;BACK/reselect/disconnect/update/expiry must revoke retained outputs. A one-time Work witness alone cannot authorize long-lived renderer output.'
 )
 observation_versus_authority=[ordered]@{
  observation='A default-off safe diagnostic may report field values, actual Work identity and source provenance with qualified=false. No new game wire is required for this evidence step.'
  authorization='A consumer cannot create a lease from Class,owner hash,private generation,operation UUID or historical NAB. Any future game-wire/PreparedResources appearance admission requires an explicit producer-owned capability and versioned contract bound to current selection/appearance lifetime.'
  existing_profile='Native92/93 profile1 remains existing unarmed prince/head0 or1 SCENE_ONLY;do not widen it based on v2 diagnostics.'
 }
 excluded=@('No Java/SQL/formal DB writes requested','No original BIN/live-person test','No renderer LOADED/GUI or complete Native PreparedResources grant','No generalized polymorph support or runtime events/layers inferred')
 renderer_LOADED=$false;GUI=0
}
$path=Join-Path $logs 'r446-minimum-live-appearance-requirements.json';Write-ModernEvidenceJson -Path $path -Value $result
$handoff=Join-Path $logs 'Native-live-appearance-R446最小依賴與權威快照需求交接.txt'
[IO.File]::WriteAllText($handoff,@"
R446最小live外觀需求（給監製；觀察優先，非wire/lease授權）
現有真選角有source/object/epoch/channel、Type/Sex；CharacterSummary.class明確是職業Type，不是Sprite Class。R445真sameWork提供privategen/gearrev/operation/deadline；實際equipment source與loadout有flags、effective weapon/retained Type。同Work live render Class/active gfx或變身狀態仍未提供，R439歷史Class不可拼接成live快照。
最小base-only路徑：同一SQL/權威read bundle的render Class，加已存在且同bundle綁定的Type/Sex；明示由權威appearance store確認無active override，不接受unknown；同次equipment reconcile的有效weapon/retainedType/rev；固定catalog dependencies。只既有0/0/0與3/0/734矩陣；logical0→3225、734→3476由固定NAD推導，無需重複欄位。若無權威無override證據，不能把base Class當active外觀。
若要active override：要定義清楚effective logical row或image bank語意及action family、權威override狀態/appearance revision/截止。已給有效row且完整綁定可不用重複Type/Sex衍生欄位，但未知或unsupported row拒絕，不因收到activegfx就扩catalog資格。
足夠的publication條件：actual principal/selected private attachment/gen的同Work/Operation內同bundle读Class/override+physical、real owner-bound store.read reconcile；Class/override必須進source owner material或可檢查appearance revision。query observation到commit重新驗selection/gen/backend deadline/cancel/appearance revision與source/desired gear/override exclusive cutoff；變更或過期拒絕，不悄悄換base/unarmed。現有真Operation.matchesKey/matchesSelection/tryCommit由producer持有，Rust不得用diagnostic UUID/gen/hash鑄lease。
safe diagnostic只是qualified=false觀察，當前無需新game wire。若未來進PreparedResources/wire，須明示版本與真producer-owned appearance capability/retire/截止；需owner/appearance/gear更新撤銷持有輸出。Native92/93 profile1 unarmedprince/head0/1 SCENE_ONLY不擴大。LOADEDfalse/GUI0。此需求交接後繼續原版有限advance/end及shadow105分支，不等待。
JSON: $path
"@,[Text.UTF8Encoding]::new($false))
Write-ModernEvidenceJson -Path (Join-Path $logs 'r446-appearance-needs-verification.json') -Value ([ordered]@{result=$path;sha256=(Get-FileHash $path).Hash;handoff=$handoff;handoff_sha256=(Get-FileHash $handoff).Hash;no_authority_grant=$true;renderer_LOADED=$false;GUI=0})
Write-Output 'R446 precise appearance dependencies recorded;observation distinct from authorization'
