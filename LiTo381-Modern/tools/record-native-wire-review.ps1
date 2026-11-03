param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$auditPath=Join-Path $logs 'native-wire-v1-compatibility-audit.json'
$audit=Get-Content -LiteralPath $auditPath -Raw | ConvertFrom-Json
if($audit.differences.Count -ne 5 -or $audit.fixed_vectors.cross_language_verified){throw 'Unexpected review state'}
function Write-AtomicText([string]$path,[string]$value) {
    $temporary=$path+'.'+[Guid]::NewGuid().ToString('N')+'.tmp'
    try {
        [IO.File]::WriteAllText($temporary,$value,[Text.UTF8Encoding]::new($false))
        [IO.File]::Move($temporary,$path,$true)
    } finally { if(Test-Path -LiteralPath $temporary){Remove-Item -LiteralPath $temporary} }
}
$brief=@'
Native wire v1 / Rust v47 差異交接（草案審查，尚未改 codec）

1. base v47 / numeric schema1、HELLO80=40 bytes、ACK81=28 bytes相符。
   extension bit0x100未使用；base CAP_GM_BOTS也為0x100，但屬另一個word，沒有碰撞。
2. REQUEST：2+2+32+104=140 payload，加入16-byte header才是156 whole frame。
   請固定向量明確使用body104，不再嵌入一個LTM1 header。
3. 現有Rust KNOWN=255會拒絕Native bit；非零extension一律要求完整Battle dependencies。
   草案Native單独依賴0x6F；需依mask驗依賴，同時保留旧extension完整依賴與closure守門。
4. 必須先對齊正式控制器路徑：schema2/modern_v47 constructor選canonical，
   world lifecycle走kind66/67，request112 whole/96 body，保留attachment-owned DATA。
   草案86/87卻包plain20/21的body104/result，不能直接當成正式PLAYER已整合。
   請定義SourcePlan/raw-sector/archive/canonical proof與PREPARE/LOADED/READ/LEAVE、
   canonical DATA/attachment及後續MOVE/life/mutation的綁定與失效規則。
5. SourcePlan252及hash preimage241 bytes算術相符；result echo36+plan252。
   若沿用plain WORLD全frame最大131072，新full最大推導131360；尚非雙方核定上限。
   請提供固定最低/最高長度、正負HEX與manifest；新kind應按selected extension守門，
   不能放寬舊kind長度、把未知bit視為已協商、將Native binding走legacy mutation。

R425只讀NativeDM564資產/CPU已通過32必需LTtile、3434slots、18994refs與11負向拒絕。
1026.til原始256槽／106..255 EOF尾保留；預設strict API仍拒絕，顯式Native選槽API限實際引用。
390 opacity僅parsed/static objects；屋頂淡化资格0。5效果parsed，38-0/123-0 SPR各11frames
及獨立frame0 CPU樣本；世界action/animation資格0。TLS/SQL/結算/GUI未驗，mode仍16/40。

機器證據：native-wire-v1-compatibility-audit.json，含草案SHA、11個Rust來源pins、19個行號依據。
正式R408／權威JAR／共享資源不替換。等待Server固定向量與canonical整合契約後再實作。
'@
$briefPath=Join-Path $logs 'Native-wire-v1差異交接.txt'
Write-AtomicText $briefPath $brief
$reportPath=Join-Path $logs '持續驗證與缺口報告.txt'
$report=[IO.File]::ReadAllText($reportPath)
$report=$report.Replace('R409–R420 是隔離的測試／operator／診斷 host，不是另一份玩家成品。','R409–R425 是隔離的測試／operator／診斷 host與離線consumer，不是另一份玩家成品。')
$report=$report.Replace('src共479檔，471檔一致；5個測試輔助檔及3個含cfg(test)診斷的模組有差異，不宣稱所有產品來源bytes未變。','src共479檔，470檔一致；5個測試輔助檔、3個含cfg(test)診斷的模組及tile.rs顯式Native選槽API有差異。預設strict及Shared sparse reader不改，不宣稱所有產品來源bytes未變。')
$marker='R424/R425 Native離線資產與wire差異：'
if(!$report.Contains($marker)) {
    $addition=@'

R424/R425 Native離線資產與wire差異：
R424新增顯式decode_classic_til_selected_with_empty_tail API；預設full/selected仍strict。
僅允許有真實資料的TIL連續EOF尾，引用空槽、中間空洞、超界offset、截斷、budget與不支援payload均拒絕。
22 TIL unit +4 Shared稀疏reader +6真實Shared goldens通過，正式PLAYER未重建。
R425固定NativeDM564 snapshot43檔含4實際Native archives與sector；32必需LTtile/3434slots解碼，
18994floor/object refs靜態CPU全圖3072x1536及800x600相機樣本成功，11負向拒絕；PNG已視閱。
1026.til原始40295 bytes/256槽位保持，106..255 EOF尾，不用Shared同號資源代替。
390 opacity僅解析與靜態物件；玩家驅動roof政策0。Layer8五效果解析；Native38-0/123-0 SPR
各11frames解碼並有獨立frame0樣本，世界action/animation資格0。非原版framebuffer／GUI／新mode。
Native wire草案審查完成：v47/schema1/40-byte HELLO/28-byte ACK相符；bit0x100可用，
新mask需Native依賴0x6F，舊extension依賴不放寬。REQUEST payload140/full156須明確標示。
正式控制器走66/67+canonical attachment/DATA；草案86/87包20/21，整合契約尚待雙方確認。
SourcePlan252/hash preimage241相符，result最大長度須固定；等待正負HEX vectors及manifest。
Native TLS／SQL／戰鬥結算／動態屋頂與效果／正式PLAYER GUI仍未驗；mode聯集維持16/40，GUI0。
新證據：r424-candidate.json、native-dm564-fixture-snapshot.json、r425-native-assets-consumer-result.json、
native-wire-v1-compatibility-audit.json、Native-wire-v1差異交接.txt。
'@
    $report += $addition
}
Write-AtomicText $reportPath $report
$statusPath=Join-Path (Split-Path -Parent $project) '監製進度.json'
$status=Get-Content -LiteralPath $statusPath -Raw | ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o')
$status.phase='native_wire_contract_alignment_pending'
$status.current_task='Native564離線資產完成；wire草案五項差異已寫入交接：payload140/full156、mask依賴、正式canonical66/67與plain20/21整合、result bounds、Native profile/loader/LOADED proof。等待固定HEX與整合答覆。'
$status.next_step='讀取Server固定正負HEX/manifest與canonical66/67整合答覆，再實作selected-extension codec及proof gate；保持舊codec fail closed與16/40計數。'
$item=[pscustomobject]@{
    owner='伺服器組／總監製';priority='high';request='Native wire v1草案具體差異已核對：REQUEST payload140/full156；Native mask0x6F與舊Battle依賴分開；正式v47控制器走canonical66/67，86/87包plain20/21尚不能整合，請明確SourcePlan/LOADED及後續canonical mutation綁定。請提供固定result界限、正負HEX、manifest與整合路徑。'
    evidence=$auditPath;status='rust_draft_audit_complete_fixed_vectors_and_canonical_dispatch_agreement_pending'
}
$status.dependency_requests=@($status.dependency_requests | Where-Object {$_.evidence -ne $auditPath})+@($item)
foreach($path in @($auditPath,$briefPath,(Join-Path $PSScriptRoot 'audit-native-wire-v1.ps1'),$PSCommandPath)) {
    if($status.evidence_paths -notcontains $path -and $path -like '*.json'){$status.evidence_paths=@($status.evidence_paths)+@($path)}
    if($status.changed_files -notcontains $path){$status.changed_files=@($status.changed_files)+@($path)}
}
$verification='Native wire v1草案與11Rust source pins/19行號核對：握手相符；五項差異已交接，未改codec／未驗跨語言／未新增Native mode。'
if($status.verified -notcontains $verification){$status.verified=@($status.verified)+@($verification)}
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output ([ordered]@{audit=$auditPath;brief=$briefPath;report=$reportPath;status=$statusPath}|ConvertTo-Json)
