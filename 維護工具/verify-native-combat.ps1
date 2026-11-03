#Requires -Version 7.0
[CmdletBinding()]param([ValidateSet('Debug','Release')][string]$BuildProfile='Debug',[string]$SourceManifest='',[string]$NormalQualification='',[switch]$SkipBuild,[switch]$CycleOnly,[switch]$RangedOnly,[switch]$SkillsOnly,[switch]$AttackSkillsOnly,[switch]$GroundSkillsOnly,[switch]$SelfOnly)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
. (Join-Path $PSScriptRoot 'owned-protocol-fixture.ps1')
. (Join-Path $PSScriptRoot 'owned-current-protocol-context.ps1')
$output=Join-Path $taskRoot '驗證紀錄\current\native-combat'
$runtime=Join-Path $project 'target\current\native-combat'
$classes=Join-Path $project 'target\current\compatibility-oracle'
if($SelfOnly){
    if($CycleOnly -or $RangedOnly -or $SkillsOnly -or $AttackSkillsOnly -or $GroundSkillsOnly){throw 'SelfOnly cannot be combined with other mode filters'}
    $output=Join-Path $taskRoot '驗證紀錄/current/native-combat-self'
    $runtime=Join-Path $project 'target/current/native-combat-self'
    $classes=Join-Path $runtime 'oracle'
}
$snapshot=Join-Path $project 'target\current\server-reference\formal-server.jar'
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project 'target\current\x86_64-pc-windows-msvc\debug\LiToModern.exe'
$context=New-OwnedCurrentProtocolContext -ToolPath $PSCommandPath -Group (Split-Path -Leaf $output) -BuildProfile $BuildProfile -SourceManifest $SourceManifest -NormalQualification $NormalQualification -ExtraInputs @('tests/fixtures/native-attack-skills-current.json','tests/fixtures/native-combat-current.json','tests/fixtures/native-ground-skills-current.json','tests/fixtures/native-ranged-current.json','tests/fixtures/native-skills-current.json','tests/oracle/NativeAttackSkillOracle.java','tests/oracle/NativeCombatOracle.java','tests/oracle/NativeGameHarness.java','tests/oracle/NativeGroundSkillOracle.java','tests/oracle/NativeRangedOracle.java','tests/oracle/NativeSkillOracle.java')
$output=$context.Output;$outputBase=$context.OutputBase;$runtime=$context.Runtime;$classes=$context.Classes;$exe=$context.Exe
Assert-OwnedCurrentProtocolContext $context
$parity=[System.Collections.Generic.List[object]]::new()
$pin=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄\current\reference-snapshot.json') -Raw|ConvertFrom-Json
$sha=(Get-FileHash -LiteralPath $snapshot -Algorithm SHA256).Hash
if($sha -ne $pin.sha256){throw 'Pinned JAR mismatch'}
function Quote([string]$text){if($text.Contains('"')){throw 'Invalid process argument'};'"'+$text+'"'}
function Test-GroundFrameSamples($frames,[int]$expectedCount) {
    # TCP follows real elapsed time; a slow debug frame may cross the 2-tick
    # terminal interval. Validate every observed sample, report actual coverage,
    # and leave required terminal dwell coverage to the independent offline audit.
    $seen=[System.Collections.Generic.HashSet[long]]::new()
    $finished=[System.Collections.Generic.HashSet[long]]::new()
    $terminal=0;$samples=0
    foreach($frame in $frames){foreach($effect in $frame.ground_effects.effects){
        if($effect.elapsed_ns -lt 0){throw 'Ground negative elapsed time'}
        $tick=[Math]::Floor([decimal]$effect.elapsed_ns*24/1000000000)
        $end=0;$source=0;$done=$true
        $durations=@(1,1,2,3,3,3,3,3,3,2,2)
        for($i=0;$i -lt $durations.Count;$i++){
            $end+=$durations[$i]
            if($tick -lt $end){$source=if($i -eq 10){0}else{$i+1};$done=$false;break}
        }
        $subframe=if($source -eq 0){10}else{$source-1}
        if($effect.graphics -ne 168 -or $effect.source_frame -ne $source -or $effect.subframe -ne $subframe -or $effect.finished -ne $done -or $effect.drawn -ne (!$done)){throw 'Ground sampled frame differs from original timeline'}
        [void]$seen.Add([long]$effect.sequence);$samples++
        if($done){[void]$finished.Add([long]$effect.sequence)}
        if(!$done -and $source -eq 0){$terminal++}
    }}
    if($seen.Count -ne $expectedCount -or $finished.Count -ne $expectedCount){throw 'Ground sampled effect lifecycle incomplete'}
    return [ordered]@{samples=$samples;effects=$seen.Count;terminal_samples=$terminal;terminal_observed=($terminal -gt 0);elapsed_timeline_verified=$true;continuous_frame_coverage_verified=$false}
}
$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$scratch=Join-Path $project 'target\current\scratch'
New-Item -ItemType Directory -Path $scratch -Force|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
$receipts=[System.Collections.Generic.List[object]]::new()
Push-Location $project
try {
    & (Join-Path $jdk 'javac.exe') -encoding UTF-8 -cp $snapshot -d $classes 'tests/oracle/NativeCombatOracle.java' 'tests/oracle/NativeRangedOracle.java' 'tests/oracle/NativeSkillOracle.java' 'tests/oracle/NativeAttackSkillOracle.java' 'tests/oracle/NativeGroundSkillOracle.java' 'tests/oracle/NativeGameHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Game harness compile failed'}
    & (Join-Path $jdk 'java.exe') -cp ($classes+';'+$snapshot) local.lito381.server.NativeCombatOracle > (Join-Path $output 'oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Game oracle failed'}
    $parityOutput=Join-Path $output 'parity-oracle';New-Item -ItemType Directory -Path $parityOutput|Out-Null
    $parity.Add((Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-combat-current.json') -ActualOracle (Join-Path $output 'oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeCombatOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $parityOutput))
    & (Join-Path $jdk 'java.exe') -cp ($classes+';'+$snapshot) local.lito381.server.NativeRangedOracle > (Join-Path $output 'ranged-oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Ranged oracle failed'}
    $parityOutput=Join-Path $output 'parity-ranged-oracle';New-Item -ItemType Directory -Path $parityOutput|Out-Null
    $parity.Add((Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-ranged-current.json') -ActualOracle (Join-Path $output 'ranged-oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeRangedOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $parityOutput))
    & (Join-Path $jdk 'java.exe') -cp ($classes+';'+$snapshot) local.lito381.server.NativeSkillOracle > (Join-Path $output 'skills-oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Skill oracle failed'}
    $parityOutput=Join-Path $output 'parity-skills-oracle';New-Item -ItemType Directory -Path $parityOutput|Out-Null
    $parity.Add((Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-skills-current.json') -ActualOracle (Join-Path $output 'skills-oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeSkillOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $parityOutput))
    & (Join-Path $jdk 'java.exe') -cp ($classes+';'+$snapshot) local.lito381.server.NativeAttackSkillOracle > (Join-Path $output 'attack-skills-oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Attack skill oracle failed'}
    $parityOutput=Join-Path $output 'parity-attack-skills-oracle';New-Item -ItemType Directory -Path $parityOutput|Out-Null
    $parity.Add((Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-attack-skills-current.json') -ActualOracle (Join-Path $output 'attack-skills-oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeAttackSkillOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $parityOutput))
    & (Join-Path $jdk 'java.exe') '-Dfile.encoding=UTF-8' -cp ($classes+';'+$snapshot) local.lito381.server.NativeGroundSkillOracle > (Join-Path $output 'ground-skills-oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Ground skill oracle failed'}
    $parityOutput=Join-Path $output 'parity-ground-skills-oracle';New-Item -ItemType Directory -Path $parityOutput|Out-Null
    $parity.Add((Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-ground-skills-current.json') -ActualOracle (Join-Path $output 'ground-skills-oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeGroundSkillOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $parityOutput))
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile|Out-Null}
    $modes=if($SelfOnly){@('room-world-shield','room-world-refused')}elseif($GroundSkillsOnly){@('room-world-wall','room-world-wall-refused')}elseif($AttackSkillsOnly){@('room-world-bolt','room-world-bolt-refused')}elseif($SkillsOnly){@('room-world-heal','room-world-shield')}elseif($RangedOnly){@('room-world-ranged')}elseif($CycleOnly){@('room-world-cycle')}else{@('room-world-attack','room-world-victory','room-world-defeat','room-world-refused','room-world-cycle','room-world-ranged')}
    foreach($mode in $modes){
        $boltMode=$mode -in @('room-world-bolt','room-world-bolt-refused')
        $groundMode=$mode -in @('room-world-wall','room-world-wall-refused')
        $skillMode=$groundMode -or $boltMode -or $mode -in @('room-world-heal','room-world-shield')
        $modeRoot=Join-Path $runtime $mode;New-Item -ItemType Directory -Path $modeRoot -Force|Out-Null
        $readyPath=Join-Path $modeRoot 'ready.json';$resultPath=Join-Path $modeRoot 'result.json'
        # Fixed files belong only to this isolated test; never traverse/delete a tree.
        foreach($file in @($readyPath,$resultPath)){if(Test-Path -LiteralPath $file){Remove-Item -LiteralPath $file -Force}}
        $server=$null;$client=$null
        try {
            $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-cp',(Quote ($classes+';'+$snapshot)),'local.lito381.server.NativeGameHarness',(Quote $modeRoot),$mode) -WorkingDirectory $modeRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output ($mode+'-server.log')) -RedirectStandardError (Join-Path $output ($mode+'-server.stderr.txt'))
            $until=[DateTime]::UtcNow.AddSeconds(15);$ready=$null
            while(!$ready){
                if($server.HasExited -or [DateTime]::UtcNow -ge $until){throw "Game harness failed readiness: $mode"}
                if(Test-Path -LiteralPath $readyPath){try{$ready=Get-Content -LiteralPath $readyPath -Raw|ConvertFrom-Json}catch{}}
                if(!$ready){Start-Sleep -Milliseconds 20}
            }
            if($ready.jar_sha256 -ne $sha){throw 'Session loaded wrong JAR'}
            $info=[System.Diagnostics.ProcessStartInfo]::new($exe)
            $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
            $info.RedirectStandardInput=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
            $info.StandardInputEncoding=[System.Text.UTF8Encoding]::new($false)
            $info.StandardOutputEncoding=[System.Text.UTF8Encoding]::new($false)
            $info.ArgumentList.Add('--probe-native-lobby');$info.ArgumentList.Add('127.0.0.1:'+$ready.port)
            $client=[System.Diagnostics.Process]::Start($info)
            $outTask=$client.StandardOutput.ReadToEndAsync();$errTask=$client.StandardError.ReadToEndAsync()
            # These are local MemoryAccountRepository fixture credentials, not player credentials.
            $fixtureInput=[ordered]@{account='重建帳號';password='測試密碼';retry_password=$null;source_index=1;lobby_mode=$mode}
            $client.StandardInput.Write(($fixtureInput|ConvertTo-Json -Compress));$client.StandardInput.Close()
            if(!$client.WaitForExit(120000)){throw 'Native GAME client timed out'}
            $clientText=$outTask.GetAwaiter().GetResult();$clientText|Set-Content -LiteralPath (Join-Path $output ($mode+'-client.json')) -Encoding utf8
            $errTask.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($mode+'-client.stderr.txt')) -Encoding utf8
            if(!$server.WaitForExit(8000)){throw 'Deployed session did not close'}
            if($client.ExitCode -ne 0 -or $server.ExitCode -ne 0){throw "Lobby test failed $mode; inspect mode logs"}
            $response=$clientText|ConvertFrom-Json;$reference=Get-Content -LiteralPath $resultPath -Raw|ConvertFrom-Json
            $combat=$response.combat;$world=$combat.result.world
            if(!$response.worker_quiescent -or $response.shutdown -ne 'closed' -or $world.own_attack_packets -ne $(if($mode -eq 'room-world-heal'){1}elseif($skillMode){0}elseif($mode -in @('room-world-defeat','room-world-cycle')){$combat.attacks_sent}elseif($mode -eq 'room-world-refused'){0}else{1}) -or $world.local_corrections -ne 0){throw 'Combat owner/attack acknowledgement mismatch'}
            if($world.map_id -ne 564 -or (!$skillMode -and $mode -ne 'room-world-ranged' -and $combat.route.Count -lt 1) -or $combat.frames.Count -lt 10){throw 'Combat route/render evidence incomplete'}
            $own=$world.actors.($world.status.object_id.ToString());$bot=$world.actors.'2'
            if($skillMode){
                $expectedSkills=switch($mode){'room-world-heal'{@(1,3)}'room-world-shield'{@(3,1)}'room-world-bolt'{@(4,4)}'room-world-bolt-refused'{@(4)}'room-world-wall'{@(58)}'room-world-wall-refused'{@(58)}}
                $selected=@($combat.skill_selection)
                if($selected.Count -ne $combat.skills_sent -or (@($selected|ForEach-Object{$_.skill}) -join ',') -ne ($expectedSkills -join ',')){throw 'Native learned-ID selection path missing'}
                foreach($selection in $selected){
                    if($selection.connection -ne $combat.epoch -or $selection.generation -ne $world.generation){throw 'Skill selection owner mismatch'}
                    if(($selection.skill -eq 4 -and $selection.target -ne 2) -or ($selection.skill -ne 4 -and $null -ne $selection.target)){throw 'Skill target policy mismatch'}
                }
            }
            if($own.x -ne $reference.world_x -or $own.y -ne $reference.world_y -or $world.hp[0] -ne $reference.final_player_hp){throw 'World state differs from formal session'}
            if($mode -eq 'room-world-attack'){
                if($response.game.phase -ne 'battle_pending' -or $world.attack_packets -ne 2 -or $world.hp[0] -ge $combat.before.hp[0] -or $bot.hp_ratio -ge 100 -or $bot.dead){throw 'Attack/counterattack did not settle'}
            }elseif($mode -eq 'room-world-victory'){
                if($response.game.phase -ne 'lobby_ready' -or $reference.game_phase -ne 'CHANNEL' -or $reference.joined_room -ne 0 -or $reference.fixture_bot_hp -ne 1 -or !$bot.dead -or $bot.hp_ratio -ne 0 -or $world.battle_result -ne 0 -or $world.scores.Count -lt 1){throw 'Victory/result/return did not settle'}
            }
            if($mode -in @('room-world-defeat','room-world-cycle')){
                if($response.game.phase -ne 'lobby_ready' -or $reference.game_phase -ne 'CHANNEL' -or $reference.joined_room -ne 0 -or $reference.fixture_bot_hp -ne -1 -or !$own.dead -or $world.hp[0] -ne 0 -or $bot.dead -or $world.battle_result -ne 1 -or $world.death_confirmations_sent -ne 1 -or !$combat.death_ack.own.action_complete -or $combat.death_ack.own.phase -ne 'death'){throw 'Natural defeat/animation/death confirmation/return failed'}
                if($combat.before.hp[0] -ne 50 -or ($combat.attacks_sent -lt 10 -or $combat.attacks_sent -gt 32 -or $world.attack_packets -ne (2*$combat.attacks_sent)) -or $world.scores.Count -lt 1){throw 'Defeat must use unmodified full starting HP and actual repeated attacks'}
            }
            if($mode -eq 'room-world-refused'){
                if($response.game.phase -ne 'battle_pending' -or $world.attack_packets -ne 0 -or $world.hp[0] -ne 50 -or $combat.attacks_sent -ne 2 -or $combat.result.attack_feedback -ne 'no_response'){throw 'Silent refusal/retry did not preserve authority'}
            }
            if($mode -eq 'room-world-ranged'){
                if($combat.before.actors.'1'.walk_action -ne 20 -or $combat.route.Count -ne 0 -or $world.projectiles.Count -ne 1 -or $world.projectiles[0].projectile.graphics -ne 66 -or $combat.attacks_sent -ne 1 -or $reference.remaining_arrows -ne 19){throw 'Ranged envelope/authority mismatch'}
                if(@($combat.frames|ForEach-Object {$_.projectiles.flights}|Where-Object drawn).Count -lt 1){throw 'Ranged original arrow was never drawn'}
            }
            if($boltMode){
                $refusedBolt=$mode -eq 'room-world-bolt-refused'
                $casts=if($refusedBolt){0}else{2};$sent=if($refusedBolt){1}else{2};$mp=if($refusedBolt){2}else{4}
                if($combat.skills_sent -ne $sent -or $world.own_cast_packets -ne $casts -or $world.mp[0] -ne $mp -or $world.mp[0] -ne $reference.final_player_mp){throw 'Bolt authority mismatch'}
                if($refusedBolt){if($combat.result.skill_feedback.Rejected -ne 278){throw 'Bolt insufficient MP refusal missing'}}
                else{
                    if($combat.result.skill_feedback -ne 'CastAction' -or $world.actors.'2'.hp_ratio -ge 100){throw 'Bolt hit/cast missing'}
                    $fx=$combat.frames[-1].spell_projectiles
                    if($fx.launched -ne 2 -or $fx.arrivals -ne 2 -or $fx.active -ne 0 -or $fx.omitted -ne 0){throw 'Bolt effect lifecycle mismatch'}
                    $gfx=@($combat.frames|ForEach-Object {$_.spell_projectiles.flights}|Where-Object drawn|ForEach-Object graphics)
                    if(167 -notin $gfx -or 219 -notin $gfx){throw 'Original bolt/impact never drawn'}
                }
            }elseif($groundMode){
                $refusedGround=$mode -eq 'room-world-wall-refused'
                $casts=if($refusedGround){0}else{1};$mp=if($refusedGround){2}else{60}
                if($combat.skills_sent -ne 1 -or $world.own_cast_packets -ne $casts -or $world.mp[0] -ne $mp -or $world.mp[0] -ne $reference.final_player_mp){throw 'Ground authority mismatch'}
                $choice=$combat.skill_selection[0]
                if(@($choice.ground).Count -ne 2 -or $choice.ground[0] -ne $bot.x -or $choice.ground[1] -ne $bot.y){throw 'Ground selected point mismatch'}
                $fx=$combat.frames[-1].ground_effects
                if($refusedGround){if($combat.result.skill_feedback -ne 'NoResponse' -or $fx.started -ne 0 -or $world.effects.Count -ne 0){throw 'Ground silent MP refusal mismatch'}}
                else{
                    if($combat.result.skill_feedback -ne 'CastAction' -or $bot.hp_ratio -ge 100 -or $fx.started -lt 1 -or $fx.started -gt 8 -or $fx.active -ne 0 -or $fx.omitted -ne 0){throw 'Ground pulse or visuals incomplete'}
                    $groundCoverage=Test-GroundFrameSamples $combat.frames $fx.started
                }
            }elseif($skillMode){
                $expectedMp=if($mode -eq 'room-world-heal'){6}else{2}
                if(($mode -eq 'room-world-heal' -and $combat.result.skill_feedback.Rejected -ne 278) -or ($mode -eq 'room-world-shield' -and $combat.result.skill_feedback -ne 'NoResponse')){throw 'Skill refusal feedback mismatch'}
                if($combat.skills_sent -ne 2 -or $world.own_cast_packets -ne 1 -or $world.mp[0] -ne $expectedMp -or $world.mp[1] -ne 10){throw 'Skill MP/refusal mismatch'}
                if($mode -eq 'room-world-heal' -and ($combat.hp_before_skill -ge 50 -or $world.hp[0] -le $combat.hp_before_skill)){throw 'Real damage/heal transition absent'}
            }
            if((!$skillMode -and !$combat.shared_held_attack) -or $combat.held_active_end){throw 'Shared held input did not stop'}
            if($mode -eq 'room-world-cycle'){
                $rounds=@($response.combat_rounds)
                if($rounds.Count -ne 2 -or $rounds[0].epoch -ne $rounds[1].epoch -or $rounds[1].before.generation -le $rounds[0].before.generation -or $rounds[1].previous_generation -ne $rounds[0].before.generation){throw 'Cross-battle owner/generation failed'}
                foreach($round in $rounds){
                    if($round.before.hp[0] -ne 50 -or $round.before.own_attack_packets -ne 0 -or $round.before.scores.Count -ne 0 -or $round.result.world.battle_result -ne 1 -or $round.result.world.death_confirmations_sent -ne 1 -or $round.held_active_end){throw 'Cross-battle state did not reset/complete'}
                    if($round.first_frame.actors.Where({$_.id -eq 1})[0].phase -ne 'idle' -or $round.first_frame.audio.Count -ne 0){throw 'Old action/audio survived new battle'}
                }
            }
            $phases=@($combat.frames | ForEach-Object {$_.actors} | ForEach-Object {$_.phase})
            if($mode -eq 'room-world-attack' -and ('attack' -notin $phases -or 'damage' -notin $phases -or 'idle' -notin $phases)){throw 'Combat animation phases missing'}
            if($mode -in @('room-world-victory','room-world-defeat','room-world-cycle') -and 'death' -notin $phases){throw 'Death animation missing'}
            $audio=@($combat.frames | ForEach-Object {$_.audio})
            if(@($audio|Where-Object {$_.prepared -eq $false}).Count -ne 0 -or ($mode -notin @('room-world-refused','room-world-bolt-refused','room-world-wall-refused') -and $audio.Count -eq 0)){throw 'Combat sound preparation failed'}
            foreach($f in $combat.frames){if($f.png -and !(Test-Path -LiteralPath $f.png)){throw 'Combat image absent'}}
            if($response.game.selected_source -ne '測試乙' -or $response.transport.epoch -ne '2'){throw 'World owner changed'}
            if($reference.status -ne 'verified' -or $reference.jar_sha256 -ne $sha -or !$reference.actual_client_sessions -or $reference.sql_access -or $reference.state_persistent -or (Test-Path -LiteralPath (Join-Path $modeRoot 'state.properties'))){throw 'Isolation/session evidence failed'}
            $caseReceipt=[ordered]@{mode=$mode;client=$response;server=$reference;verified=$true}
            if($groundMode){$caseReceipt['frame_coverage']=if($refusedGround){Test-GroundFrameSamples $combat.frames 0}else{$groundCoverage}}
            $caseReceipt.presentation_file_pins=Save-OwnedProtocolPresentationFiles -Response $response -Output $output -Mode $mode
            $receipts.Add($caseReceipt)
            Write-Output "$mode native combat and authority verified"
        }finally{
            foreach($owned in @($client,$server)){if($null -ne $owned){if(!$owned.HasExited){Stop-OwnedLitoValidationChild -Process $owned -ScopeRoot $project};$owned.Dispose()}}
        }
    }
    $receiptPath=Join-Path $output $(if($GroundSkillsOnly){'ground-skills-receipt.json'}elseif($AttackSkillsOnly){'attack-skills-receipt.json'}elseif($SkillsOnly){'skills-receipt.json'}elseif($RangedOnly){'ranged-receipt.json'}elseif($CycleOnly){'cycle-receipt.json'}else{'receipt.json'})
    Assert-OwnedCurrentProtocolContext $context
    if((Get-FileHash -LiteralPath $snapshot).Hash -ne $sha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $sha){throw 'Current reference identity drift'}
    [ordered]@{schema=2;profile=$BuildProfile;output=$output;runtime=$runtime;immutable_receipt=$receiptPath;source_pins=$context.SourcePins;compilation_inputs_checked=$context.Inputs.Count;compilation_source_manifest=$context.SourceManifest;compilation_source_manifest_sha256=$context.SourceManifestSHA;fixture_parity=$parity;status='verified_native_combat_subset';jar_sha256=$sha;exe_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash;cases=$receipts;sql_access=$false;game_endpoint_connected=$true;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 20|Set-Content -LiteralPath $receiptPath -Encoding utf8
    Write-Output ('PROTOCOL_RECEIPT='+$receiptPath)
}finally{Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp}
