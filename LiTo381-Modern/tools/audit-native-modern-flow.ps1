param([string]$OutputDirectory=(Join-Path $PSScriptRoot '..\logs\continuation-20261003'))
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$project=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$litoRoot=Split-Path (Split-Path $project)
$launcherStatus=Get-Content (Join-Path $litoRoot 'lito登入器\監製進度.json') -Raw|ConvertFrom-Json
$contractPath='D:\lito-dev-runtime\20261003-supervision\launcher\observability-20261003-225704\state-contract\native-login-close-audit.json'
$contract=Get-Content -LiteralPath $contractPath -Raw|ConvertFrom-Json
$contractChecks=@(foreach($entry in @($contract.source_artifacts)+@($contract.current_runs)+@($contract.historical_manual_comparison)+@($contract.current_server_packets)){
    $actual=(Get-FileHash -LiteralPath $entry.path).Hash
    if($actual -ne $entry.sha256){throw ('Native reference drift: '+$entry.path)}
    [ordered]@{path=$entry.path;sha256=$actual;matches=$true}
})
$nativeReference=[ordered]@{path=$contractPath;sha256=(Get-FileHash -LiteralPath $contractPath).Hash;updated_at=$contract.updated_at;input_checks=$contractChecks;current_stage='Current mode2/page0 prelogin state matches historical manual sessions; no valid credential submit capture or C_LOGIN';historical_scope='Two preserved manual sessions contain submit marker and later source/character/channel stages; not current GUI acceptance';capture_marker_limit=$contract.capture_marker_limit;close_scope='Static WM_CLOSE and SC_CLOSE conditional dispatch requires observed message/flags plus natural exit; current runs did not establish cause';product_change_decision=$contract.change_decision;analysis_pe_limit=$contract.analysis_pe_limit;addresses_scope='Native private addresses, flags and 640x480 coordinates are reference-only and are not Modern APIs'}
$capturePaths=@($launcherStatus.evidence_paths|Where-Object{$_ -like '*frames.jsonl'})
$captures=@($capturePaths|ForEach-Object{
    $path=$_
    $frames=@(Get-Content -LiteralPath $path|ForEach-Object{$_|ConvertFrom-Json})
    $version=@($frames|Where-Object{$_.direction -eq 'client_to_server' -and $_.opcode -eq '0x00'})
    $accepted=@($frames|Where-Object{$_.direction -eq 'server_to_client' -and $_.opcode -eq '0x3A' -and $_.note -eq 'S_VERSION_CHECK accepted'})
    [ordered]@{path=$path;sha256=(Get-FileHash -LiteralPath $path).Hash;frames=$frames.Count;client_version_frames=$version.Count;accepted_version_frames=$accepted.Count;client_version_times=@($version|ForEach-Object{$_.time});accepted_times=@($accepted|ForEach-Object{$_.time});later_client_opcodes=@($frames|Where-Object{$_.direction -eq 'client_to_server' -and $_.opcode -ne '0x00'}|Select-Object -ExpandProperty opcode -Unique);scope='實際原版程序版本握手；未證明登入／選角／建房／自然退出';encryption_key_omitted=$true}
})
$latestPath=Join-Path $project 'logs\continuation-20261003\native-live-lobby-consumer.json'
$latest=$null
if(Test-Path -LiteralPath $latestPath){
    $latest=Get-Content -LiteralPath $latestPath -Raw|ConvertFrom-Json
    foreach($pin in $latest.source_pins){if((Get-FileHash -LiteralPath $pin.snapshot).Hash -ne $pin.observed_sha256){throw ('Lobby observation snapshot drift: '+$pin.snapshot)}}
}
$rows=@(
    [ordered]@{step='version_handshake';native='C_VERSION 0x00 → S_VERSION_CHECK accepted 0x3A';native_evidence='本輪actual LiTo.bin capture';modern='verified TLS → independent HELLO/ACK → LoginReady';modern_evidence='R407/R408 controller tests; protocol.rs + transport.rs';equivalence='行為階段對照；wire不同'},
    [ordered]@{step='authentication';native='原生帳密登入';native_evidence='本輪prelogin與歷史人工登入前狀態相符；有效submit marker與C_LOGIN未觀察';modern='LoginReady → Authenticating → ChoosingSource';modern_evidence='session.rs + ordinary v47 controller SQL rotation';equivalence='只對照階段；false submit marker不證明callback完全未進入'},
    [ordered]@{step='source_and_character';native='原生來源／選角';native_evidence='兩份已核雜湊歷史manual report後續階段；非本輪GUI';modern='source_id/object_id + roster epoch → CharactersReady → ChoosingChannel';modern_evidence='application_protocol.rs + session.rs + canonical_map_rotation_tests.rs';equivalence='identity須來源限定；來源ID不假定相同'},
    [ordered]@{step='lobby_room';native='C_CA channel / C_CD room read / C_CF join / C_D2 ready';native_evidence='NATIVE-CHANNEL-ROOM-FLOW-EVIDENCE.json契約；舊capture缺失';modern='InLobby + source-qualified Rooms Request/revision';modern_evidence='rooms.rs + R407/R408';equivalence='操作與狀態對照；不借用native opcode'},
    [ordered]@{step='world_load';native='原生地圖／world啟動';native_evidence='本輪GUI未抵達';modern='PREPARE → pinned shared assets LOAD → Countdown → Active';modern_evidence='world_runtime.rs / battle_runtime.rs + R407/R408';equivalence='native尚待新原生資源流程取證'},
    [ordered]@{step='leave_exit';native='自然離場／關閉';native_evidence='WM_CLOSE/SC_CLOSE來源條件已核；當輪沒有received-message/flag因果證據；歷史exit非當輪退出';modern='World Leave → room → LeaveLobby → ReturnCharacters → Shutdown/quiescent';modern_evidence='R419/R420三回合controller最後返回與退休；正式EXE完整GUI仍待驗';equivalence='程序清理不替代native/Modern玩家自然退出；不移植native sentinel或addresses'}
)
if($latest){
    foreach($row in $rows){
        $update=$latest.stage_mapping_updates|Where-Object step -eq $row.step|Select-Object -First 1
        if($update){$row.native=$update.native;$row.native_evidence='已釘選本輪redacted packet及producer report，加上使用者確認大廳；詳native-live-lobby-consumer.json';$row.equivalence=$update.limit}
    }
    ($rows|Where-Object step -eq 'world_load').native_evidence='新本輪snapshot有packet movement與使用者大廳回報，但沒有已確認進房/world畫面或戰鬥；詳native-live-lobby-consumer.json'
    ($rows|Where-Object step -eq 'leave_exit').native_evidence='新本輪C_D0只為room return；handoff C_QUIT非整程式退出。使用者尚未驗收自然退出；舊WM_CLOSE來源另列'
}
$null=New-Item -ItemType Directory -Path $OutputDirectory -Force
Write-ModernEvidenceJson -Path (Join-Path $OutputDirectory 'native-modern-state-mapping.json') -Value ([ordered]@{generated_at=[DateTimeOffset]::Now.ToOffset([TimeSpan]::FromHours(8)).ToString('o');native_release_sha256='3025E3DF1C28F55019E79D5F21F9C1587B7268FB6417D6A75A591B6AECACB4B4';native_client_sha256='CE6EE68ADA8FD37E6C096B1A27E46CB6888F1179F85DEFB5007AC60287360292';native_status_updated_at=$launcherStatus.updated_at;native_reference=$nativeReference;captures=$captures;latest_native_lobby_observation=$latest;state_mapping=$rows;limitation='舊reference runs只到版本握手；新snapshot另列使用者確認大廳與進房/說明無反應。封包不替代畫面；戰鬥/退出/Modern GUI未驗；缺失歷史capture不當已驗證'})
