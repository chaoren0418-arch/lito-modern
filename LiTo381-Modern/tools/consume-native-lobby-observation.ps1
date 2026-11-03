param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$sourceRoot='D:\lito-dev-runtime\20261003-supervision\launcher\observability-20261003-225704\manual-acceptance-20261004-015859'
$snapshot=Join-Path $logs ('native-lobby-observation-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $snapshot
function Read-Bounded([string]$path) {
    $stream=[IO.File]::Open($path,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::ReadWrite)
    try {
        if($stream.Length -gt 2MB){throw 'Observation exceeds read budget'}
        $memory=[IO.MemoryStream]::new()
        try {$stream.CopyTo($memory);if($memory.Length -gt 2MB){throw 'Observation grew beyond budget'};return ,$memory.ToArray()}
        finally {$memory.Dispose()}
    } finally {$stream.Dispose()}
}
$pins=@()
$observations=@{}
foreach($name in @('human-stage-observations.json','packet-delta-redacted.json')) {
    $source=Join-Path $sourceRoot $name
    $bytes=Read-Bounded $source
    $value=[Text.Encoding]::UTF8.GetString($bytes)|ConvertFrom-Json
    if($name -eq 'packet-delta-redacted.json' -and ($value.credentials_logged -ne $false -or $value.raw_payload_or_notes_logged -ne $false)){throw 'Consumer accepts only redacted metadata'}
    $copy=Join-Path $snapshot $name
    [IO.File]::WriteAllBytes($copy,$bytes)
    $hash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes))
    $pins += [ordered]@{source=$source;snapshot=$copy;observed_bytes=$bytes.Length;observed_sha256=$hash;producer_updated_at=$value.updated_at;live_source_may_change=$true}
    $observations[$name]=$value
}
$indexPath='D:\lineage開發\3-lito開發\LITO-監製索引.json'
$indexBytes=Read-Bounded $indexPath
$index=[Text.Encoding]::UTF8.GetString($indexBytes)|ConvertFrom-Json
$user=$index.pending_user_action
if($user.status -ne 'user_reported_lobby_entered_room_and_help_unresponsive'){throw 'Human report changed; inspect and revise mapping'}
$frames=@($observations['packet-delta-redacted.json'].frames)
$human=$observations['human-stage-observations.json']
$client=@($frames|Where-Object direction -eq 'client_to_server')
function Count-Opcode([string]$opcode) {return @($client|Where-Object opcode -eq $opcode).Count}
$value=[ordered]@{
    consumed_at=[DateTimeOffset]::Now.ToString('o');source_pins=$pins
    supervisor_index=[ordered]@{path=$indexPath;observed_sha256=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($indexBytes));observed_bytes=$indexBytes.Length;last_checked_at=$index.last_checked_at}
    native_pid_reported=$human.native_pid
    raw_process_or_GUI_inspected_by_Modern=$false
    user_report=[ordered]@{source='Supervisor user report; original chat response also read';reported_at=$user.last_user_report_at;status=$user.status;observation=$user.user_observation;room_action='User clarified clicking an existing room in the room list';automatic_screenshot=$false}
    packet_metadata=[ordered]@{
        frames=$frames.Count;first_time=$frames[0].time;last_time=$frames[-1].time
        c_login_count=(Count-Opcode '0x01');c_character_select_count=(Count-Opcode '0xC9')
        c_channel_count=(Count-Opcode '0xCA');c_room_read_count=(Count-Opcode '0xCD')
        c_room_join_count=(Count-Opcode '0xCF');c_room_return_count=(Count-Opcode '0xD0')
        c_ready_count=(Count-Opcode '0xD2');raw_payloads_and_credentials_saved=$false
        interpretation='Opcode counts and producer stage report are packet evidence, not proof of room screen, battle state, current live state or help-button cause'
    }
    producer_wire_stage_report=$human.wire_stages
    acceptance=[ordered]@{native_login_visible_user_report=$true;native_lobby_visible_user_report=$true;native_room_screen=$false;native_battle=$false;native_natural_exit=$false;Modern_GUI=$false;qualified_modes=16;new_native_modes=0}
    stage_mapping_updates=@(
        [ordered]@{step='authentication';native='Two C_LOGIN opcodes in producer redacted capture; later stages plus user reported lobby';modern='TLS HELLO/ACK → authenticating → source scoped roster';limit='No credentials consumed; protocol-stage mapping only'},
        [ordered]@{step='source_and_character';native='Source/character/channel producer stages and captured opcodes; user subsequently confirms lobby';modern='source_id/object_id/epoch → character → channel';limit='No matching IDs or native private APIs inferred'},
        [ordered]@{step='lobby_room';native='User confirms lobby; clicking existing room fails to enter. C_CF and responses present in earlier capture';modern='InLobby → revision-qualified join → confirmed room view';limit='Packet received/sent is insufficient for room-screen success; preserve visible failure until correlated diagnostic'},
        [ordered]@{step='help';native='User reports help button left of attributes unresponsive';modern='Need confirmed behavior/asset/handler contract before implementing parity';limit='Separate failure; no assumption of same cause as room join'},
        [ordered]@{step='world_load_and_exit';native='No battle countdown/ready or natural exit qualification in these snapshots';modern='Existing CPU/TLS/SQL tests remain distinct from formal PLAYER GUI';limit='Do not count C_D0 room return or LOGIN-to-GAME handoff C_QUIT as whole-process exit'}
    )
    follow_up='Await Launcher/Server time-correlated room/help diagnosis; do not change Rust behavior from unconfirmed native failure cause or interact with active native program'
}
$output=Join-Path $logs 'native-live-lobby-consumer.json'
Write-ModernEvidenceJson -Path $output -Value $value
Write-Output ([ordered]@{output=$output;snapshot=$snapshot;frames=$frames.Count;last_packet_time=$frames[-1].time;native_lobby_user_report=$true;room_screen=$false}|ConvertTo-Json)
