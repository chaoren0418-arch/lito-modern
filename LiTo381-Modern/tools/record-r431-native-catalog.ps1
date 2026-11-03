param([Parameter(Mandatory)][string]$Revision)
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function ReadEvidence($name){Get-Content -LiteralPath (Join-Path $logs $name) -Raw|ConvertFrom-Json}
function CheckPin($p,$sha){if((Get-FileHash -LiteralPath $p).Hash -ne $sha){throw "Pin drift $p"}}
$c=ReadEvidence 'r431-candidate.json';CheckPin $c.path $c.sha256
foreach($p in $c.source_pins){CheckPin $p.path $p.sha256;CheckPin $p.snapshot $p.sha256}
$build=@(Get-Content -LiteralPath (Join-Path $logs 'r431-lib-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Production library build incomplete'}
$rlib=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'})
if($rlib.Count -ne 1){throw 'Library count'}
$library=Join-Path (Split-Path -Parent $c.path) 'liblito_modern-r431.rlib';$librarySha=(Get-FileHash -LiteralPath $rlib[0]).Hash
if(Test-Path -LiteralPath $library){CheckPin $library $librarySha}else{Copy-Item -LiteralPath $rlib[0] -Destination $library;CheckPin $library $librarySha}
$c|Add-Member NoteProperty production_library ([ordered]@{path=$library;sha256=$librarySha;offline_locked=$true;target='i686-pc-windows-msvc'}) -Force
$tests=@()
foreach($row in @(@("$Revision-local",4),@("$Revision-consumer",1),@("$Revision-sql",1),@('r431-native_local',15),@('r431-canonical',6),@('r431-extensions',6),@('r431-native_units',3),@('r431-time_units',2))){
 $path=Join-Path $logs ($row[0]+'.stdout.log');$m=[regex]::Match([IO.File]::ReadAllText($path),'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
 if(-not $m.Success -or [int]$m.Groups[1].Value -ne $row[1] -or [int]$m.Groups[2].Value){throw "Unexpected tests $path"}
 $tests+=[ordered]@{suite=$row[0];passed=[int]$m.Groups[1].Value;failed=0;ignored=[int]$m.Groups[3].Value;log=$path;sha256=(Get-FileHash -LiteralPath $path).Hash}
}
$snapshots=@()
foreach($name in @('r431-catalog-snapshot.json','r431-catalog-sql-snapshot.json')){
 $s=ReadEvidence $name;CheckPin $s.input $s.input_sha256
 foreach($family in @('candidate','replay')){
  $cfg=$s.$family;$mp=Join-Path $cfg.root 'manifest.json';CheckPin $mp $cfg.manifest_sha256
  $m=Get-Content -LiteralPath $mp -Raw|ConvertFrom-Json
  if($m.files.Count -ne $cfg.files_verified){throw 'Manifest count drift'}
  foreach($p in $m.files){$path=[IO.Path]::GetFullPath((Join-Path $cfg.root $p.path));if(-not $path.StartsWith($cfg.root+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Manifest confinement'};if((Get-Item -LiteralPath $path).Length -ne $p.bytes){throw 'Length drift'};CheckPin $path $p.sha256}
  foreach($v in $cfg.vectors){CheckPin $v.path $v.sha256}
 }$snapshots+=$s
}
$d=ReadEvidence 'r431-native-catalog-details.json';$sql=ReadEvidence 'r431-native-catalog-sql-details.json'
if($d.state -ne 'FIXED_CATALOG_PUBLICATIONS_AND_PARTIAL_VERIFIED' -or $d.LOADED -or $sql.state -ne 'FIXED_SQL_CATALOG_BYTES_CONSUMED' -or $sql.LOADED){throw 'Consumer qualification'}
if(@($d.rows).Count -ne 68 -or @($d.negatives).Count -ne 40 -or @($sql.rows).Count -ne 69 -or @($sql.negatives).Count -ne 10 -or @($sql.equipment_publications).Count -ne 16 -or @($sql.load_proofs).Count -ne 8){throw 'Publication/proof count'}
$historical=@(foreach($name in @('r426','r427','r428','r429','r430')){
 $old=ReadEvidence "$name-candidate.json";CheckPin $old.path $old.sha256;CheckPin $old.production_library.path $old.production_library.sha256
 foreach($p in $old.source_pins){CheckPin $p.snapshot $p.sha256};[ordered]@{candidate=$name;verified_pins=2+@($old.source_pins).Count;changed=$false}
})
$scope=ReadEvidence 'source-change-scope.json';if($scope.source_files -ne 489){throw 'Source count'}
foreach($p in $scope.frozen_binary_pins){CheckPin $p.path $p.sha256}
$SQLreadRows=@(foreach($family in @('candidate','replay')){
 $cfg=$snapshots[1].$family;$pin=@($cfg.vectors|Where-Object{$_.name -eq 'catalog-sql-tls-vectors.json'})[0]
 $v=Get-Content -LiteralPath $pin.path -Raw|ConvertFrom-Json
 $bytes=[Convert]::FromHexString($v.read61_source7_before_move_hex)
 $owner=@($sql.receipts|Where-Object{$_.family -eq $family -and $_.case -eq 'move65_source7_hex'})[0].baseline_owner
 [ordered]@{family=$family;kind=61;sha256=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes));bytes=$bytes.Length;owner=$owner;typed_followed_by_fixed_consumer=$true}
})
$c.validation='PASSED:38 focused tests;68 in-memory/TLS publication rows;69 SQL90/91 pairs;16 SQL inventory/equipment publications;8 source-generation proofs;opaque catalog partial completion. Full renderer/LOADED unavailable.'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r431-candidate.json') -Value $c
$resultPath=Join-Path $logs 'r431-native-catalog-result.json'
$result=[ordered]@{updated_at=(Get-Date).ToString('o');state='NATIVE_CATALOG_AND_FIXED_SQL_CONSUMERS_VERIFIED_FULL_RENDERER_PENDING';candidate=$c;tests=$tests;focused_passed=38;focused_failed=0;snapshots=$snapshots;snapshot_files_reverified=6157;
 catalog_details=Join-Path $logs 'r431-native-catalog-details.json';sql_details=Join-Path $logs 'r431-native-catalog-sql-details.json';catalog_publication_rows=68;catalog_negatives=40;SQL_positive90_91_pairs=69;SQL_inventory_equipment_publications=16;SQL_source_generation_proofs=8;SQL_MOVE_stale_receipts=4;
 SQL_READ61_records=$SQLreadRows;SQL_89_capture_records=10;SQL_main_discovery91_overwritten=$true;historical_frozen_preservation=$historical;source_scope=$scope;catalog_bytes=168;request90_bytes=264;extension_bits=768;formal_known_extensions=255;opaque_partial_completed=$true;LOADED=$false;full_PreparedResources=$false;Rust_live_SQL_TLS=$false;GUI=0;qualified_modes=16;total_modes=40;formal_R408_changed=$false;
 boundary='In-memory/synthetic-auth TLS and real SQL fixed byte families remain distinct. Cross-source socket refusal is producer evidence: attack carries valid source7 proof to source42 transport. Offline context comparison uses trusted source42 publication; no Rust socket replay claim.';
 pending=@('Base Text/Sprite + four MISS shard indexes lack authoritative body proof; Server independent schema pending','SCENE_ONLY intentionally lacks combat loadout; source15/17 equipment is independent','Other Native class/sex/body/heading/actions','Scheduler phase/speed/brave/global modifiers, lifetime/loop','ExtendedLayer8 binding/full actor-roof-effect insertion order','Sealed complete Native PreparedResources and live Rust TLS/SQL/GUI admission')}
Write-ModernEvidenceJson -Path $resultPath -Value $result
$handoff=Join-Path $logs 'Native-catalog-R431與SQL消費交接.txt'
$text=@"
R431 Native catalog90/91 + fixed actual SQL consumer handoff
38 focused tests passed/0 failed; offline locked i686 host $($c.sha256), library $librarySha. Eight frozen Rust source copies match current sources. Own frozen inputs3049+8+3086+14=6157 manifest entries reverified SHA/length/confinement. R426–R430 host/library/source snapshots and R408 player preserved; no full mode rotation, no original PE/Java candidate execution.
Catalog: 168bytes schema1/namespace3/profile1, LTtext.idx/pak hashes+byte lengths; fixed LTlist.spr offset145671/39135bytes, raw SHA8BAF964C... and decoded F55779AE... . Identity domain LITO-NATIVE-CATALOG-IDENTITY + NUL, digestce1906fc061364da6fe2e3c297c178a5fae9c27bb248e6cb8ebc9f2142c095a1. Request90=264bytes wraps exact88; result91 bounded selected maximum260+89, min336, catalogpresence iff sourcepresence. Catalog index minimum32bytes matches actual frozen Java source (the narrative1-byte lower bound does not permit malformed index). Exact provided requests reencode, echoed nested IDs/schema/reserved/lengths/dependency closure strict; streamed oversized header refused before body read. Default-disabled/crate-private, formal KNOWN remains255; no live Native dispatch. Old88 is structurally refused by catalog consumer; Java old-factory ledger bypass refusal was not replayed by Rust.
In-memory/synthetic-auth TLS: candidate/replay68 publication rows=16 provided90/91 pairs+38 full capturedTLS91+14 nested89/READ61/MOVE65. Requests for full captured91 are reconstructed from exact echo, not claimed captured request bytes. 40 negative inputs=20 each family with separate structure/presence vs proof authority labels. TLS source42 Active never borrows as source7 MOVE guard; source7 READ61-owned receipt passes.
Real SQL: candidate/replay69 provided actual90/91 pairs=candidate34/replay35; normal preparing poll count differs by one; four distinct source+generation proofs per family (8total), Native V1 proof and outer catalog domain recomputed from every LOADED pair. 16 typed inventory/equipment15/17 publications preserve source/object10275/opaque epoch,29 items each. Source7 item10304 unequip changed revision exactly+1 and retained staleCAS/cross-source rejection; source42 item10304 remains equipped=true with independent flags/revision unchanged. SCENE_ONLY DATA has features0/no combat loadout; PERSONAL equipment absent is intentional scope, not no-equipment evidence. SQL standalone89 captures also decoded separately: main discovery89 has no surviving full91 because producer source/action/requestID keys were reused by later negative connections; four other89 match exact validated91. This loses two original outer discovery91 captures (one per family), not DATA/proof acceptance. Use connection incarnation plus source/action/requestID for future producer evidence keys. READ61 + MOVE65/stale-life65 each family consumed; valid source7 MOVE Applied, stale rejected. Wire bytes only: Rust did not run a SQL/TLS socket or duplicate producer172-table/write/CAS/durable-reopen qualification.
Five actualTLS negative request types consumed twice: wrong catalog, old V1 outer proof, source7 proof for source42 directly fail structure/authority; old88 structurally fails. Cross-source_scope carries a valid source7 proof but was sent to source42 socket: codec/proof alone correctly accepts its syntax. It differs from trusted source42 publication context; actual socket identity/refusal remains producer ledger evidence, no false Rust live proof. All exact case/source/epoch/channel/world/guard rows in $($result.sql_details).
Local completion now requires immutable decodedCatalog91, real pinned LTtext index entry matches offset/length, raw hash beforeL1 decode, decoded hash after; worker cancelArc privately retained. CatalogOwner takes only private Catalog Reply90/91, CatalogRendererJob created only for exact admitted Loading fullDATA and inner owner/ticket/scope. Genuine verification from another worker, mixed catalog pins, cross-owner completion, replaced ticket/cancelled worker/retire all refuse. Legal real NativeStaticAssets + FirePot slice + verified LTtext accepts only opaque partial; LOADED admission alwaysfalse.
This catalog identity covers LTtext only. R430 prince base Text/list.spr, full Sprite pair, four SpriteNN MISS indexes and selected body appearance remain separate local study dependencies, absent from proof. The Server independent actor schema is pending; current90/91 unchanged. R430 native integer clock proof stands for unmodified catalog only; scheduler phase/speed/brave/loop and Layer8/full depth unsupported. CompleteNative PreparedResources and formal Rust transport/SQL/GUI remain pending;16/40,remaining24,GUI0,R408 unchanged.
Evidence: $resultPath
Catalog cases: $($result.catalog_details)
SQL cases: $($result.sql_details)
R430 body/clock handoff: $(Join-Path $logs 'Native-renderer-R430角色與SQL交接.txt')
Historical R431a test-field compile failure and R431b oversized-header fixture misuse remain preserved. R431d incorrectly assumed both polling families had35pairs; candidate34/replay35 are retained exactly. R431e misread source42 missing override as missing item; actual item10304 remains equipped=true. R431f exposed that overwritten outer discovery91 gap and now preserves it explicitly. All failures and logs remain history; the final fixed host is named here.
"@
[IO.File]::WriteAllText($handoff,$text,[Text.UTF8Encoding]::new($false))
$report=Join-Path $logs '持續驗證與缺口報告.txt'
[IO.File]::AppendAllText($report,"`r`n`r`nR430/R431 $((Get-Date).ToString('o'))：R43034passed/16SQL rows/10prince PNG；R43138passed/68catalog rows/69SQL pairs/16inventory-equipment rows/8source-generation proofs；真实LTtext raw/decoded/index worker绑定partial通过，LOADEDfalse。base actor authority、combat loadout、full clock/loop/Layer8/depth/PreparedResources pending；16of40/GUI0/R408unchanged。交接：$handoff`r`n结果：$resultPath`r`n",[Text.UTF8Encoding]::new($false))
$sp=Join-Path (Split-Path -Parent $project) '監製進度.json';$s=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json
$s.updated_at=(Get-Date).ToString('o');$s.phase='native_catalog_and_fixed_sql_consumers_verified_full_renderer_pending';$s.current_task='R43034focused/16SQL rows/10prince PNG與原版整數frame clock已凍結；R43138focused/68catalog發布/69真SQL90-91配對/16inventory-equipment/8source-generation proof已驗證。本地LTtext authority與partial completion已接入；base actor proof及完整PreparedResources/戰鬥/full renderer/live仍pending，LOADEDfalse/16of40/GUI0/R408unchanged。'
$s|Add-Member NoteProperty r431_native_catalog_work ([ordered]@{result=$resultPath;handoff=$handoff;in_memory_TLS_rows=68;real_SQL_pairs=69;Rust_live_SQL_TLS=$false;LOADED=$false}) -Force
$s.verified=@($s.verified)+@('R43034focused/16SQLpublication/10princeCPU切片逐張檢視；Native預設FPS24與逐幀整數毫秒有原版證據，未裝武器study不冒稱实际装备body。','R43138focused/6157fixedinputpins/68catalogrows/69realSQLpairs/16inventory-equipment/8source-generationproofs；真实LTtext raw/decoded/entry+opaqueownerpartial验证。Socket跨source拒绝仍producerledger，Rust无liveTLS/SQL。')
Write-ModernEvidenceJson -Path $sp -Value $s
Write-Output 'R431 frozen38passed;68catalog rows;69SQL pairs;LOADEDfalse'
