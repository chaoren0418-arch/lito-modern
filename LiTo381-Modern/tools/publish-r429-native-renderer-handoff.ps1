param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$resultPath=Join-Path $logs 'r429-native-renderer-slice-result.json'
$r=Get-Content -LiteralPath $resultPath -Raw|ConvertFrom-Json
if($r.focused_passed -ne 31 -or $r.focused_failed -ne 0 -or $r.LOADED){throw 'R429 not qualified for partial handoff'}
$owner=Get-Content -LiteralPath $r.owner_chain -Raw|ConvertFrom-Json
$lines=@('family / vector / kind / source / object64HEX / epoch64HEX / channel')
foreach($row in $owner.rows){$lines+=('{0} / {1} / {2} / {3} / {4} / {5} / {6}' -f $row.family,$row.vector,$row.kind,$row.owner.source,$row.owner.object_u64_hex,$row.owner.epoch_u64_hex,$row.owner.channel)}
$handoff=Join-Path $logs 'Native-renderer-R429動作與publication交接.txt'
$text=@"
R429 Native renderer：來源 tick CPU 切片與 actual publication consumer 交接
更新：$((Get-Date).ToString('o'))

已完成的範圍
31 focused tests passed / 0 failed：15 local + 6 canonical + 6 existing extension + 3 Native resource units + 1 fixed actual publication consumer。最後 consumer 32.43 秒；offline/locked i686 library 編譯成功。沒有重跑舊272檔codec batch、全模式輪替或GUI。
固定2801檔Java service bytes只作Rust輸入，未執行其中classes。manifest SHA：$($r.service_snapshot.manifest_sha256)
自有快照：$($r.service_snapshot.root)
Rust固定host SHA：$($r.candidate.sha256)
Rust固定library SHA：$($r.candidate.production_library.sha256)
6份當前Rust來源與自有source snapshot逐SHA相符；來源總數486，baseline479中469相同/10項已列明變更，加7新檔。R408正式玩家保持既有SHA，未重建或替換。

逐筆 wire / owner / DATA / receipt
$($lines -join "`r`n")
8份canonical89均使用Native draft schema1，SourceAttachment exact reencode、既有canonical DATA parser、Follower語義admission；discovery/preparing沒有DATA，實際DATA供Loading/Loaded/Active需求盤點。另1份READ61與2份MOVE65沿用既有typed parser，非legacy Frame。expected88與MOVE64 request均由echo重建；本交接未宣稱捕捉到request實際bytes。
service MOVE65使用同family source7 Active89先行Observation；TLS MOVE65使用source7 READ61先行Observation。TLS Active89來自source42，雖同object3333，借用該guard仍正確拒絕。兩份MOVE receipt各用合法來源完整guard通過，未發現獨立receipt解析缺口。
先前life0推論已更正：既有Rust battle_motion::life在沒有resources時回Some(1)，與Java首代life1一致。SCENE_ONLY沒有vital/resources/buffs，這不是戰鬥、死亡或復活資格。
逐筆room/match/generation/entity/life/engineRevision、SourceAttachment/DATApresence詳見：$($r.owner_chain)

實際renderer需求
service Loading DATA：Native map564，room1/match1/generation7/revision1/phaseLoading，source7與42皆object9223372036854775807、class6/sex1/level80。
TLS Loading DATA：同圖/room1/match1，generation5/revision1/phaseLoading；source7 object3333 class1/sex0、source42 object3333 class3/sex1，皆level80。
兩family角色entity1/2位置(32729,32797)/(32729,32796)、heading0；public presentation2rows、personal gear null、NPC false，無vital/resources/buffs/weapon。服務class6與TLS class1/3的Native actor body/catalog尚未資格化；不得以Shared同號角色替代，gear absent不能宣稱裝備完成。

原版Native來源與最小action契約
原PE SHA4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9；IDA SHA2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A。只靜態讀取，未執行原PE。
LTtext protected old28 index353 entries，LTlist.spr offset145671/39135bytes，L1解碼逐byte等於原extract：F55779AEB5C3982D8B366068CDBE4003D802619074E54388F26EE87368F42603。自己的LTtext idx/pak及解碼副本已固定SHA。
原route：sub43DE10先list.spr/basecount再LTlist；sub43DDD0 Native row減basecount；sub4406C0產生L{id}-{group}.spr；sub42EA50由L前綴轉Native LTsprite。歷史sub440010會先查external SpriteXX；本地顯式Native inspector只讀固定LTsprite，不冒稱已消除歷史外部shadow歧義。
只允許#38/#123原physical catalog row，explicit action3 FirePot、group0、subframe0..10、每frame duration2、無events。純CPU解析和畫圖保留SPR signed A/B、depth/source insertion與Native half-cell排序、同key newest-first。
14張512x256 PNG使用tick0/1/2/3/10/20/21，兩sprite各7張，逐張視閱；相鄰2ticks同frame、後续tick有來源幀變化。圖像是獨立火盆半格anchor切片，沒有actor/完整DM564場景。
Native毫秒/FPS、scheduler start及loop未知：只接受tick0..21；22與u64max拒絕，未套用Shared預設24FPS。sub4E02C0有條件idle base+3分支，但ExtendedLayer8完整物件/default action/start/repeat仍未驗收。
路由contract：$($r.source_route)

不可偽造的本地partial completion
crate-private/default-disabled RendererJob私有欄位绑定unique owner/ticket、connection/room fence、完整SourceAttachment、canonical attachment、Follower已admit完整DATA與共享cancel token。
finish_partial只接受真正NativeStaticAssets與真正NativeActionSlice，核對相同plan/catalog，slice sprite必須存在同assets.effect_records。complete_partial_renderer再驗current ticket/owner/Scope/full DATA/attachment/Arc cancel身分；跨owner、替換job、相同DATA新ticket舊completion、取消、retire均拒絕。
marker故意只稱PartialRendererCompletion，不能替代完整PreparedResources。接受合法partial後loaded_admission_available仍false，沒有Native live dispatch/transport/controller/LOADED API。
LTtext目前只是supplemental local catalog pins，並不在 authoritative SourceAttachment；這項身份缺口明列而非補造wire欄位。

後續必要缺口
Native actor catalog/body來源與實際class6、class1/3 mapping；其base list.spr/baseSprite或Native alias archive必須依原route證據釘identity。Authoritative resource plan需能涵蓋實際catalog/body來源。ExtendedLayer8物件/預設action、Native時鐘/loop、完整actor-roof-effect insertion仍需證據。全部必要renderer完成並封裝真正Native PreparedResources，才可做正式LOADED與Rust live TLS/SQL/GUI資格。
本次服務actualDATA已收到並驗證，不再列為等待服務交接；也不等待全Server結案。既有mode仍16/40、remaining24、GUI0。失敗中間host與life0診斷保留歷史，最後R429f為唯一通過候選。
總證據：$resultPath
詳細14切片、11publication需求及負向守門：$($r.details)
"@
[IO.File]::WriteAllText($handoff,$text,[Text.UTF8Encoding]::new($false))
$report=Join-Path $logs '持續驗證與缺口報告.txt'
$append="`r`n`r`nR429 $((Get-Date).ToString('o'))：31 focused tests全pass；固定2801檔bytes重核SHA，8 Native89/1READ61/2MOVE65 owner與receipt驗證。TLS source42 Active89不能借給source7 MOVE，合法source7 READ61通過；life首代1無獨立gap。Native38/123 action3 14個source-tick CPU圖逐張視閱；opaque partial completion含完整DATA/source/attachment/fence/ticket/cancel。完整Native actor/時鐘/Layer8/深度及authoritative LTtext identity待補，LOADEDfalse；16/40、GUI0、R408未改。交接：$handoff`r`n證據：$resultPath`r`n"
[IO.File]::AppendAllText($report,$append,[Text.UTF8Encoding]::new($false))
$statusPath=Join-Path (Split-Path -Parent $project) '監製進度.json'
$s=Get-Content -LiteralPath $statusPath -Raw|ConvertFrom-Json
$s.updated_at=(Get-Date).ToString('o')
$s.phase='native_renderer_partial_cpu_and_actual_publications_verified_full_support_pending'
$s.current_task='R429已完成31focused tests、2801固定service bytes、11publication逐筆owner/DATA/receipt、14Native FirePot source-tick/depth CPU切片與opaque partial completion。TLS跨source guard正確拒絕，合法兩MOVE通過；life1推論已更正。完整Native actor/catalog身分、時鐘及Layer8/full深度待补，LOADEDfalse、16/40、GUI0、R408未改。'
$s.verified+= 'R429固定host60A04B25...與libraryD2619964...：31focused全pass；8Native89/1READ61/2MOVE65逐筆驗證，2801檔manifest bytes重核；14Native38/123 action3來源tick CPU PNG逐張視閱，partial completion守門成功但Loaded admission=false。'
foreach($dep in $s.dependency_requests){
 if($dep.status -eq 'rust_draft_audit_complete_fixed_vectors_and_canonical_dispatch_agreement_pending'){$dep.status='resolved_R428_codec_and_R429_actual_publications_consumed';$dep.request='固定codec及actual Native canonical89/READ61/MOVE65已由Modern consumer驗證；service2801檔bytes與owner/guard鏈已固定。完整Native renderer/LOADED/liveTLS仍屬後續。'}
 if($dep.request -like '需固定Native action/catalog*'){$dep.status='partially_resolved_R429_actual_DATA_and_firepot_ticks_full_renderer_pending';$dep.request='Actual DATA/service generation與兩MOVE receipt已驗證，Native38/123 action3來源ticks/depth完成；仍需Native actor/body/class映射、時鐘/loop/full Layer8深度，以及authoritative SourceAttachment catalog/body身份。'}
}
$s.next_step='繼續獨立Native actor body/catalog原route與實際class6/1/3來源身份，釘完整resource plan；查Native time/ExtendedLayer8/full depth，完成必要renderer與真正封裝PreparedResources後再驗live Native LOADED/TLS。'
$s.evidence_paths+=@($resultPath,$handoff,$r.owner_chain,$r.source_route,$r.details,(Join-Path $logs 'r429-candidate.json'))
$s.changed_files+=@((Join-Path $project 'src\modern_lito\native_action_inspection.rs'),(Join-Path $project 'src\modern_lito\native_renderer_slice_tests.rs'),(Join-Path $PSScriptRoot 'record-r429-native-renderer-slice.ps1'),(Join-Path $PSScriptRoot 'publish-r429-native-renderer-handoff.ps1'))
$s.native_renderer_work=[ordered]@{state='partial_cpu_and_actual_publication_verified_full_renderer_pending';service_files_verified=2801;manifest_sha256=$r.service_snapshot.manifest_sha256;focused_passed=31;publication_rows=11;CPU_PNG=14;LTlist_entry_offset=145671;LTlist_bytes=39135;LTlist_decoded_sha256='F55779AEB5C3982D8B366068CDBE4003D802619074E54388F26EE87368F42603';native_ms_clock=$false;actor=$false;partial_completion=$true;LOADED=$false;GUI=0;result=$resultPath;handoff=$handoff}
Write-ModernEvidenceJson -Path $statusPath -Value $s
Write-Output $handoff
