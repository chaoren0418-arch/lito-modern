#Requires -Version 7.0
# Read-only audit of completed real-session combat runs; does not start a server.
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$receipt=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/native-combat/receipt.json') -Raw|ConvertFrom-Json
$pin=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/reference-snapshot.json') -Raw|ConvertFrom-Json
$exe=Join-Path $taskRoot 'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe'
if($receipt.exe_sha256 -ne (Get-FileHash -LiteralPath $exe).Hash -or $receipt.jar_sha256 -ne $pin.sha256){throw 'Outcomes evidence uses a different executable or server'}
$defeat=@($receipt.cases|Where-Object mode -eq 'room-world-defeat')
$refused=@($receipt.cases|Where-Object mode -eq 'room-world-refused')
if($defeat.Count -ne 1 -or $refused.Count -ne 1){throw 'Missing distinct outcomes cases'}
$combat=$defeat[0].client.combat;$world=$combat.result.world
$own=$world.actors.($world.status.object_id.ToString())
$score=@($world.scores|Where-Object name -eq $own.name)
if($score.Count -ne 1 -or $score[0].deaths -ne 1 -or $score[0].kills -ne 0 -or $world.death_confirmations_sent -ne 1 -or $world.death_confirmation_pending -or !$combat.death_ack.own.action_complete){throw 'Own death score/confirmation mismatch'}
$hp=@($combat.frames|ForEach-Object {$_.world.hp[0]})
$recovery=0;for($i=1;$i -lt $hp.Count;$i++){if($hp[$i] -gt $hp[$i-1]){$recovery+=$hp[$i]-$hp[$i-1]}}
$events=0;$sounds=[System.Collections.Generic.HashSet[int]]::new()
foreach($case in $receipt.cases){
    $rounds=if($case.mode -eq 'room-world-cycle'){@($case.client.combat_rounds)}else{@($case.client.combat)}
    foreach($round in $rounds){
    if($round.shared_held_attack -and $round.held_active_end){throw 'Completed scenario retained held attack'}
    if($round.death_ack){
        $roundWorld=$round.result.world
        $roundOwn=$roundWorld.actors.($roundWorld.status.object_id.ToString())
        $roundScore=@($roundWorld.scores|Where-Object name -eq $roundOwn.name)
        if($roundScore.Count -ne 1 -or $roundScore[0].deaths -ne 1 -or $roundScore[0].kills -ne 0 -or $roundWorld.death_confirmations_sent -ne 1 -or $roundWorld.death_confirmation_pending -or !$round.death_ack.own.action_complete){throw 'Per-round own death result mismatch'}
    }
    $seen=[System.Collections.Generic.HashSet[string]]::new()
    foreach($frame in $round.frames){
        foreach($actor in $frame.actors){
            if(!$actor.rendered){throw 'Missing actor render'}
            foreach($event in $actor.entered_events){
                $key=@($event.actor,$event.sequence,$event.action,$event.frame,$event.event)-join '/'
                if(!$seen.Add($key)){throw 'Duplicate frame event'}
                $events++
            }
        }
        if($frame.audio_events_dropped -ne 0){throw 'Frame audio budget exceeded'}
        foreach($sound in $frame.audio){
            $id=[int]$sound.event.sound
            if(!$sound.prepared -or $id -lt 1 -or $id -gt 9999){throw 'Invalid prepared sound'}
            $file=Join-Path 'D:\lineage開發\0-客戶端\Sound' ($id.ToString()+'.wav')
            if((Get-Item -LiteralPath $file).Length -ne $sound.bytes -or (Get-FileHash -LiteralPath $file).Hash -ne $sound.sha256){throw 'Sound source differs from prepared PCM'}
            $null=$sounds.Add($id)
        }
    }
}
}
if($refused[0].client.combat.result.attack_feedback -ne 'no_response' -or $refused[0].client.combat.attacks_sent -ne 2){throw 'Missing timeout/retry evidence'}
[ordered]@{schema=1;status='verified_native_outcomes_audit';exe_sha256=$receipt.exe_sha256;jar_sha256=$receipt.jar_sha256;defeat_attacks=$combat.attacks_sent;observed_hp_recovery=$recovery;death_confirmations=1;own_deaths=$score[0].deaths;frame_events=$events;events_once=$true;sound_ids=@($sounds|Sort-Object);gui_tested=$false;audio_device_tested=$false;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/native-outcomes/audit.json') -Encoding utf8
Write-Output 'Native defeat, retry, scoreboard and frame events verified'
