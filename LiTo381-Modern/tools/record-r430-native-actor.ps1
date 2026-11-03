$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function ReadEvidence($name){Get-Content -LiteralPath (Join-Path $logs $name) -Raw|ConvertFrom-Json}
function CheckPin($path,$sha){if((Get-FileHash -LiteralPath $path).Hash -ne $sha){throw "Pin drift $path"}}
$c=ReadEvidence 'r430-candidate.json'
CheckPin $c.path $c.sha256
CheckPin $c.production_library.path $c.production_library.sha256
foreach($p in $c.source_pins){CheckPin $p.path $p.sha256;CheckPin $p.snapshot $p.sha256}
$tests=@()
foreach($r in @(@('local',15),@('canonical',6),@('extensions',6),@('native_units',3),@('time-units',2),@('sql',1),@('e-actor',1))){
 $n=if($r[0] -eq 'e-actor'){'r430e-actor.stdout.log'}else{'r430-'+$r[0]+'.stdout.log'}
 $p=Join-Path $logs $n
 $m=[regex]::Match([IO.File]::ReadAllText($p),'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
 if(-not $m.Success -or [int]$m.Groups[1].Value -ne $r[1] -or [int]$m.Groups[2].Value){throw "Test count $n"}
 $tests+=[ordered]@{suite=$r[0];passed=[int]$m.Groups[1].Value;failed=0;log=$p;sha256=(Get-FileHash -LiteralPath $p).Hash}
}
$detail=ReadEvidence 'r430-native-prince-slice-details.json'
if($detail.state -ne 'PINNED_PRINCE_UNARMED_SOURCE_STUDY_PASSED' -or $detail.LOADED){throw 'Actor not qualified'}
$pngs=@(Get-ChildItem -LiteralPath (Join-Path $logs 'r430-native-prince-slices') -Filter '*.png'|ForEach-Object{
 $b=[IO.File]::ReadAllBytes($_.FullName)
 if([Convert]::ToHexString($b[0..7]) -ne '89504E470D0A1A0A' -or [Convert]::ToHexString($b[16..23]) -ne '0000020000000100'){throw 'PNG dimensions'}
 [ordered]@{path=$_.FullName;sha256=(Get-FileHash -LiteralPath $_.FullName).Hash;visually_reviewed=$true;width=512;height=256}
})
if($pngs.Count -ne 10){throw 'PNG count'}
$historical=@(foreach($n in @('r426','r427','r428','r429')){
 $old=ReadEvidence "$n-candidate.json";CheckPin $old.path $old.sha256;CheckPin $old.production_library.path $old.production_library.sha256
 foreach($p in $old.source_pins){CheckPin $p.snapshot $p.sha256}
 [ordered]@{candidate=$n;verified_pins=2+@($old.source_pins).Count;changed=$false}
})
$scope=ReadEvidence 'source-change-scope.json'
if($scope.source_files -ne 487){throw 'Source scope'}
$c.validation='PASSED:34 focused tests;16 fixed SQL publication rows;10 visual-reviewed prince CPU slices;original integer frame clock;full renderer/LOADED unavailable.'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r430-candidate.json') -Value $c
$result=[ordered]@{updated_at=(Get-Date).ToString('o');state='PARTIAL_NATIVE_PRINCE_AND_SQL_PUBLICATIONS_VERIFIED';candidate=$c;tests=$tests;focused_passed=34;focused_failed=0;pngs=$pngs;details=Join-Path $logs 'r430-native-prince-slice-details.json';sql_publications=Join-Path $logs 'r430-sql-publication-details.json';publication_rows=16;source_scope=$scope;original_contract=Join-Path $logs 'r430-native-actor-route-contract.json';base_snapshot=ReadEvidence 'r430-base-body-snapshot.json';sql_snapshot=ReadEvidence 'r430-sql-snapshot.json';historical_frozen_preservation=$historical;LOADED=$false;GUI=0;Rust_live_SQL_TLS=$false;qualified_modes=16;total_modes=40;formal_R408_changed=$false;pending=@('Authoritative base Text/Sprite and shard route identity','Equipped weapon/body appearance absent from SCENE_ONLY','All class/sex and heading routes beyond this prince study','Original scheduler phase, speed/brave/global modifiers and lifetime/loop','ExtendedLayer8 binding and full actor-roof-effect depth','Complete sealed Native PreparedResources and live transport')}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r430-native-actor-result.json') -Value $result
$handoff=Join-Path $logs 'Native-renderer-R430角色與SQL交接.txt'
$text=@"
R430 Native prince CPU / actual SQL publication handoff
34 focused tests passed, 0 failed; 16 candidate/replay publication rows; 10 PNGs visually reviewed. Host $($c.sha256), production rlib $($c.production_library.sha256). R426–R429 frozen hosts/libraries/source copies and R408 preserved. Source487=479 baseline(469 same/10 approved changed)+8 new.
SQL candidate/replay have source7/42 same object10275 but distinct complete owners. Class0/sex0/level60; entity1/2 coordinates32729,32797 /32729,32796, heading0. Each family has5 typed89,1 READ61,1 valid MOVE65,1 stale-life65 correctly refused. SCENE_ONLY public presentation is shot/item only; gear absent does not imply unarmed. Fixed wire bytes consumed, no Rust live SQL/TLS. Full owner/channel/room/match/generation/entity/life/revision in $($result.sql_publications).
Original C_CreateChar CLASS_LIST class/sex→logical0. Native base catalog11297 rows, row0 graphics alias3225. Action0 walk4frames, action3 idle8frames, all duration4/eightway. Actual MOVE changes heading0→1; four selected members3225-0/-1/-8/-9.spr. Original440010 hash shard selected12/13/04/05 indexes each pinned MISS; full base Sprite pair pinned. Unread shard PAK contents not qualified. Explicit BodyStudy::ExplicitUnarmedPrince, not equipped player-body completion.
Base dependencies: Text.idx/pak, decoded list.spr; Sprite.idx/pak; Sprite04/05/12/13.idx plus respective PAK size for validated index bounds. Exact pins/route contract in $($result.base_snapshot.root) and $($result.original_contract). New 90/91 LTtext catalog alone does not cover these base dependencies. Server must define separate authority/profile for real actor catalog, graphics alias, shard route precedence, selected body appearance and archive identity; no peer filename or silent Shared fallback.
Original43DE10 sets Native defaultFPS24 and opcode110 can replace it. Frame duration multiplied1000 then integer idiv FPS into encoded duration. Thus duration4→166ms; duration2→83ms. CPU uses unmodified catalog only and explicit phase0, single interval: walk0..663ms, idle0..1327ms, firepot0..912ms; next boundary/u64max rejected. 4E8750 speed/brave/global adjustment, scheduler start, lifetime/repeat and Layer8 remain unsupported. Shared default was not used as evidence. The R429 historical unknown-clock statement is superseded only for this bounded default-catalog conversion.
Ten isolated prince RGBA samples preserve SPR signed depth/source insertion/Native half-cell coordinates and source heading; no complete world scene. Opaque completion binds real source plan/full DATA/owner/ticket/cancel and actor pins. Accepted partial completion still LOADED=false; Native full PreparedResources unavailable,16/40,GUI0.
Result: $(Join-Path $logs 'r430-native-actor-result.json')
"@
[IO.File]::WriteAllText($handoff,$text,[Text.UTF8Encoding]::new($false))
$sp=Join-Path (Split-Path -Parent $project) '監製進度.json'
$s=Get-Content -LiteralPath $sp -Raw|ConvertFrom-Json
$s.updated_at=(Get-Date).ToString('o');$s.phase='native_catalog90_91_consumer_in_progress_after_r430_actor';$s.current_task='R430 34focused passed/16SQL publications/10 prince CPU samples已凍結；正在接入R431 catalog90/91 LTtext identity與partial completion，base actor resources另列。LOADEDfalse/16of40/GUI0/R408unchanged。'
$s|Add-Member NoteProperty r430_native_actor_work ([ordered]@{result=Join-Path $logs 'r430-native-actor-result.json';handoff=$handoff;LOADED=$false}) -Force
Write-ModernEvidenceJson -Path $sp -Value $s
Write-Output 'R430 frozen:34passed;16 publications;10PNG;LOADEDfalse'
