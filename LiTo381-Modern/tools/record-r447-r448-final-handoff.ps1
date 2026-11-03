$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
$snapshotPath=Join-Path $logs 'r439-appearance-snapshot.json';$snapshot=Get-Content $snapshotPath -Raw|ConvertFrom-Json
$manifestPath=Join-Path $snapshot.candidate.root 'manifest.json';if((Get-FileHash $manifestPath).Hash -ne $snapshot.candidate.manifest_sha256){throw 'Resource root drift'}
$manifest=Get-Content $manifestPath -Raw|ConvertFrom-Json
$chains=@(foreach($stage in @('r447','r448')){
 $detailPath=Join-Path $logs $(if($stage -eq 'r447'){'r447-continuation-details.json'}else{'r448-layer-details.json'});$d=Get-Content $detailPath -Raw|ConvertFrom-Json
 $names=if($stage -eq 'r447'){@('resources/appearance-dependencies.nad','evidence/appearance-bindings.nab')+@($d.member_pins.PSObject.Properties.Name|ForEach-Object{"resources/$_"})}else{@($d.input_pins.PSObject.Properties.Name)}
 $pins=@(foreach($name in $names){$entry=@($manifest.files|Where-Object path -eq $name);if($entry.Count -ne 1){throw 'Selected resource not unique'};$p=Join-Path $snapshot.candidate.root $name;$pin=Pin $p;if($pin.sha256 -ne $entry[0].sha256 -or $pin.bytes -ne $entry[0].bytes){throw 'Selected resource drift'};$pin.relative_path=$name;$pin})
 $chain=[ordered]@{state='ACTUAL_TEST_SELECTED_RESOURCE_CHAIN_REVERIFIED';manifest=Pin $manifestPath;root_snapshot=Pin $snapshotPath;selected_files=$pins;count=$pins.Count;details=Pin $detailPath;original_members_subframes_no_substitution=$true}
 $chainPath=Join-Path $logs "$stage-resource-chain.json";Write-ModernEvidenceJson -Path $chainPath -Value $chain
 [ordered]@{stage=$stage;final_verification=Pin (Join-Path $logs "$stage-final-verification.json");result=Pin (Join-Path $logs "$stage-finite-study-result.json");resource_chain=Pin $chainPath}
})
$drafts=@(foreach($name in @('r447a-host-build.jsonl','r447b-host-build.jsonl','r447b-library-build.jsonl','r448a-host-build.jsonl')){Pin (Join-Path $logs $name)})
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$scope=Get-Content (Join-Path $logs 'source-change-scope.json') -Raw|ConvertFrom-Json
$partial=Join-Path $logs 'r446-partial-observation-consumption.json'
$handoff=Join-Path $logs 'Native-appearance-R446-R448本輪有限實作交接.txt'
[IO.File]::WriteAllText($handoff,@"
R447c：1 focused passed/0failed，0.25sec；8heading共48步、2遲到case。frame1首步接2..6→end reset0；continuing delay=max(1,duration-lateness)+2，expected與scheduled分離。末尾條件queue0/nonrenderplayer/counter0/base46/idle49pointer/配置成功明示，queue idle49delay584後停止；原expected不因end分支更新，未執行idle callback。39source/host/lib、10完整ASM函式、4選定resource及7frame CPUstrip固定/視閱。
R448b：1 focused passed/0failed，0.60sec；8heading×6subframe=48neutral CPU及48bit2 skip105 submission case。原4E0200提交body→105list→shadow，同bank88..95／同actor19subframe。按原半格/深度、新插入tie先畫及screen scan合成；每heading首sample與獨立literal key/pixel compositor全像素相同。24實際SPR＋4catalog/binding/trace輸入釘manifest SHA/bytes；所選105全部0fragment，所以isolated中格空白。firstheading17fragments ledger與2CPU圖固定/視閱。45source含4未變baseline raster依賴/host/lib、13完整ASM函式固定。actorflags0/105flags0、base shadow與所有pointer存在是明示conditional，非runtime觀察；其他未知flag/pointer/override分支拒絕，bit2只submission證據且CPUpaint拒絕。RGB555 reference，不宣告原版framebuffer565 parity。
R446 Server新partial已唯讀消費：3708candidate＋38evidence全SHA/bytes/exact set核對；Class/Type/Sex734/3/0為same statement、實際source7object10163同Work weapon10252/type2/equipped/rev0可用。BaseClass尚未納入retained owner/appearance rev；override有無/appearance rev/cutoff/revocation unknown，所以qualifiedfalse。BACK+backendexpiry是單一合併拒絕，不能拆算SQL-item或appearanceexpiry。本輪Rust未連新候選、未造lease/wire，不替換C88C/v2；無live整套重跑。
512src=479baseline/469unchanged/10allowedchanged/33new。R408與歷史R443/R444/R445/R447二進位/來源snapshot保持；compile-only草稿logs保留。LOADEDfalse/GUI0/16of40，整體仍in progress。後續需producer appearance owner/state/revision/cutoff與輸出撤銷來源；未足證不能入live。
R447 result: $($chains[0].result.path)
R448 result: $($chains[1].result.path)
R446 partial: $partial
"@,[Text.UTF8Encoding]::new($false))
$result=Join-Path $logs 'r447-r448-combined-handoff.json'
Write-ModernEvidenceJson -Path $result -Value ([ordered]@{state='R447_R448_FINITE_STUDIES_COMPLETE_LIVE_AUTHORITY_PENDING';checked_at=[DateTimeOffset]::Now.ToString('o');stages=$chains;compile_only_drafts_preserved=$drafts;partial_Server_diagnostic=Pin $partial;source_scope=Pin (Join-Path $logs 'source-change-scope.json');handoff=Pin $handoff;focused_new_passed=2;focused_failed=0;CPU_images_visually_reviewed=3;source_files=$scope.source_files;R408_changed=$false;original_BIN_executed=$false;live_appearance_qualified=$false;renderer_LOADED=$false;GUI=0;overall_status='in_progress'})
$statusPath=Join-Path (Split-Path $project -Parent) '監製進度.json';$status=Get-Content $statusPath -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o');$status.phase='native_R447_R448_finite_continuation_layers_CPU_passed_appearance_authority_pending'
$status.current_task='R447c1pass0.25sec/48steps2late/expected-scheduled分離/end reset0→queueidle49delay584停止/39source10ASM4resource1PNG；R448b1pass0.60sec/48CPU+48bit2 submission/獨立literal排序像素相等/105實際零fragment/45source13ASM28resource2PNG，全部封存視閱。R446partial Server3708+38pins/exact set唯讀消費，sameStatement734/3/0同Workweapon10252type2rev0可用，但Class/override owner/rev/cutoff未知、qualifiedfalse。512sources/R408及歷史/草稿保留，無舊live整套或新候選連線，LOADEDfalse/GUI0/16of40。'
$status|Add-Member native_appearance_R446_R448 (Pin $result) -Force
$status.next_step='需producer Native selected appearance owner/override presence-or-absence/revision/exclusive cutoff/retained-output revocation；未有權威資料前拒絕live base appearance。R447/448有限續步、原版圖層first branch與CPU已完成，不重跑既有live整套。'
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output 'R446/R447/R448 combined handoff pinned;512src/R408 verified;2new focused passed;3CPU images reviewed;live authority pending'
