#Requires -Version 7.0
[CmdletBinding()]param([switch]$SkipBuild,[ValidateSet('Debug','Release')][string]$BuildProfile='Debug',[string]$ReferenceJar,
    [ValidateSet('lito-legacy','tw13081901-v1')][string]$ResourceProfile='lito-legacy',[string]$CommonCatalogPath='',[switch]$ExtendedItemDetails,[string]$VisibilityNormalInputsPath='',
    [ValidateSet('detection-invisibility','detection-refused','invisibility-refused','invisibility-missing','melee','heal','heal19','heal19-full','heal19-refused','heal35','heal35-full','heal35-refused','heal57','heal57-full','heal57-refused','bolt','bolt-refused','fire-arrow','fire-arrow-refused','wind-cutter','wind-cutter-refused','ice-arrow','ice-arrow-refused','holy-weapon','holy-weapon-missing','holy-weapon-refused','poison-cure','poison-missing','poison-refused','cure-refused','chill-touch','chill-touch-full','chill-touch-refused','vampiric-touch','vampiric-touch-full','vampiric-touch-refused','stalac','stalac-refused','lightning','lightning-refused','lightning-multi','turn-undead-quest','turn-undead-refused','turn-undead-player','wall','wall-refused','ranged','item-heal','doors','movement','honor-respawn','room-admin','classic-entry','rental-entry','rental-shop','rental-citizen','rental-hotel','rental-hotel-entry','rental-hotel-middle-entry','rental-hotel-share','rental-merchants','rental-weight','rental-item-expiry','rental-equipped-expiry','rental-hotel-expiry','rental-hotel-middle-expiry','rental-return')][string]$ActionProfile='melee')
$peerRoles=if($ActionProfile -eq 'lightning-multi'){@('a','b','c')}elseif($ActionProfile -eq 'turn-undead-quest'){@('b','a')}else{@('a','b')}
$peerCount=$peerRoles.Count
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
if($ActionProfile -in @('rental-hotel-expiry','rental-hotel-middle-expiry','rental-item-expiry','rental-equipped-expiry')){$stamp+='-'+$ActionProfile}
$root=[IO.Path]::GetFullPath((Join-Path $project ('target/current/native-real-peers/'+$stamp)))
$output=Join-Path $taskRoot ('驗證紀錄/current/native-real-peers/'+$stamp)
$classes=Join-Path $root 'oracle'
$taskValidatorRoot=$PSScriptRoot
$snapshot=Join-Path $project 'target/current/server-reference/formal-server.jar'
if($ReferenceJar){
    $snapshot=(Resolve-Path -LiteralPath $ReferenceJar).Path
    $allowed=(Resolve-Path -LiteralPath (Join-Path $project 'target/current/server-reference')).Path+[IO.Path]::DirectorySeparatorChar
    if(!$snapshot.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw 'Reference outside owned snapshot directory'}
}
$source='D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar'
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLower()+'/LiToModern.exe')
$commonCatalogSHA=$null
if($ResourceProfile -eq 'tw13081901-v1'){
    if(!$CommonCatalogPath){throw 'Shared resource profile requires an explicit owned common catalog'}
    $CommonCatalogPath=(Resolve-Path -LiteralPath $CommonCatalogPath).Path
    if(!$CommonCatalogPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase) -or ((Get-Item -LiteralPath $CommonCatalogPath).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Shared catalog outside owned rebuild files'}
    $commonCatalogSHA=(Get-FileHash -LiteralPath $CommonCatalogPath).Hash
    if($commonCatalogSHA -ne 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'){throw 'Shared catalog identity differs from the reviewed formal catalog'}
}elseif($CommonCatalogPath){throw 'Common catalog is only valid for the explicitly selected shared profile'}
New-Item -ItemType Directory -Path $root,$output,$classes -Force|Out-Null
Write-Output ('PEER_CASE_OUTPUT='+$output)
$pin=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/reference-snapshot.json') -Raw|ConvertFrom-Json
$sha=(Get-FileHash -LiteralPath $snapshot -Algorithm SHA256).Hash
if((!$ReferenceJar -and $sha -ne $pin.sha256) -or (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash -ne $sha){throw 'Formal reference drift; revalidate snapshot first'}
function Stop-OwnedPeer([Diagnostics.Process]$process){
    if($process.HasExited){return}
    $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
    if(!$live -and $process.HasExited){return}
    $ownedCommand=if($live){$live.CommandLine.Replace('/','\')}else{''}
    if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or
        (!$ownedCommand.Contains($root.Replace('/','\')) -and !$ownedCommand.Contains($exe.Replace('/','\')))){throw 'Owned peer PID/creation/command mismatch; stop refused'}
    $process.Kill();$process.WaitForExit()
}
function Quote([string]$text){if($text.Contains('"')){throw 'Invalid process argument'};'"'+$text+'"'}
$server=$null;$clients=[System.Collections.Generic.List[object]]::new()
Push-Location $project
try {
    & (Join-Path $jdk 'javac.exe') -encoding UTF-8 -cp $snapshot -d $classes 'tests/oracle/NativePeerHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Real-peer harness compile failed'}
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile *> (Join-Path $output 'build.log');if($LASTEXITCODE -ne 0){throw 'Client build failed'}}
    $exeSha=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
    $originalExe=$exe;$exe=Join-Path $root 'qualified-client.exe'
    Copy-Item -LiteralPath $originalExe -Destination $exe
    if((Get-FileHash -LiteralPath $exe).Hash -ne $exeSha){throw 'Immutable peer EXE differs'}
    $sourceSnapshot=Join-Path $output 'source-snapshot';New-Item -ItemType Directory -Path $sourceSnapshot|Out-Null
    $sourcePins=[ordered]@{}
    foreach($relative in @('tests/native_skills.rs','tests/native_skill_selection.rs','src/modern_lito/native_combat_animation.rs','tests/oracle/NativeQuestSummonBoundary.java','tests/native_combat.rs','src/modern_lito/native_class1.rs','src/modern_lito/native_attack_many.rs','src/modern_lito/native_turn_undead_probe.rs','src/modern_lito/native_navigation.rs','src/modern_lito/native_world_effects.rs','src/modern_lito/native_world_layers.rs','src/modern_lito/native_actor_layers.rs','src/modern_lito/native_combat.rs','src/modern_lito/native_skills.rs','src/modern_lito/native_skill_selection.rs','src/modern_lito/native_skill_effects.rs','src/modern_lito/native_peer_actions.rs','src/modern_lito/native_poison_probe.rs','src/modern_lito/tower_assets.rs','src/modern_lito/sprite_actions.rs','src/modern_lito/native_actor_states.rs','src/modern_lito/mod.rs','src/modern_lito/native_peer_probe.rs','src/modern_lito/native_world.rs','src/modern_lito/native_world_scene.rs','src/modern_lito/native_login.rs','tests/oracle/NativePeerHarness.java','tests/native_holy_weapon.rs','tests/native_poison_cure.rs','tests/native_chill_touch.rs','tests/native_vampiric_touch.rs','tests/native_stalac_layers.rs','tests/native_lightning.rs','tests/native_turn_undead.rs','tests/support/native_fragment_reference.rs','Cargo.toml','Cargo.lock')){
        $file=Join-Path $project $relative;$copy=Join-Path $sourceSnapshot $relative
        New-Item -ItemType Directory -Path (Split-Path -Parent $copy) -Force|Out-Null
        Copy-Item -LiteralPath $file -Destination $copy;$sourcePins[$relative]=(Get-FileHash -LiteralPath $file).Hash
    }
    $sourcePins|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $output 'source-pins.json') -Encoding utf8
    Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $sourceSnapshot 'runner.ps1')
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'verify-real-peer-actions.py') -Destination (Join-Path $sourceSnapshot 'verify-real-peer-actions.py')
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'verify-real-peer-poison.py') -Destination (Join-Path $sourceSnapshot 'verify-real-peer-poison.py')
    Copy-Item -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/native-world-integration/stalac-original-sources.json') -Destination (Join-Path $sourceSnapshot 'stalac-original-sources.json')
    Copy-Item -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/native-world-integration/lightning-original-sources.json') -Destination (Join-Path $sourceSnapshot 'lightning-original-sources.json')
    if($ActionProfile -in @('turn-undead-quest','turn-undead-player')){Copy-Item -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/native-world-integration/turn-undead-original-sources.json') -Destination (Join-Path $sourceSnapshot 'turn-undead-original-sources.json');Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'verify-real-peer-turn-undead.py') -Destination (Join-Path $sourceSnapshot 'verify-real-peer-turn-undead.py')}
    if($ActionProfile -match '^heal(19|35|57)(-full|-refused)?$'){Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'verify-real-peer-higher-heal.py') -Destination (Join-Path $sourceSnapshot 'verify-real-peer-higher-heal.py')}
    $longExpiry=$ActionProfile -in @('rental-hotel-expiry','rental-hotel-middle-expiry','rental-item-expiry','rental-equipped-expiry')
    if($longExpiry){
        # Long real-time leases must not hold the working EXE open while development continues.
        $exe=Join-Path $root 'qualified-client.exe'
        if((Get-FileHash -LiteralPath $exe).Hash -ne $exeSha){throw 'Immutable expiry EXE copy differs'}
        $taskValidatorRoot=Join-Path $output 'validation-snapshot';New-Item -ItemType Directory -Path $taskValidatorRoot|Out-Null
        foreach($name in @('verify-real-peer-rental-entry.py','verify-real-peer-rental-hotel-entry.py','verify-real-peer-rental-hotel-expiry.py','verify-real-peer-rental-item-expiry.py','verify-real-peer-equipped-expiry.py','verify-real-peer-attributes.py','native_resident_map_reference.py','native_hotel_catalog_reference.py')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination (Join-Path $taskValidatorRoot $name)}
        Copy-Item -LiteralPath (Join-Path $project 'tests/fixtures/native-portal-current.json') -Destination (Join-Path $taskValidatorRoot 'native-portal-current.json')
        Copy-Item -LiteralPath (Join-Path $project 'tests/fixtures/native-rental-merchant-current.json') -Destination (Join-Path $taskValidatorRoot 'native-rental-merchant-current.json')
        $pins=[ordered]@{}
        foreach($folder in @('LiTo381-Modern/src','LiTo381-Modern/tests','維護工具')){
            foreach($file in Get-ChildItem -LiteralPath (Join-Path $taskRoot $folder) -Recurse -File|Where-Object {$_.Extension -in @('.rs','.java','.json','.py','.ps1','.html')}){$pins[[IO.Path]::GetRelativePath($taskRoot,$file.FullName).Replace('\','/')]=(Get-FileHash -LiteralPath $file.FullName).Hash}
        }
        foreach($name in @('LiTo381-Modern/Cargo.toml','LiTo381-Modern/Cargo.lock','LiTo381-Modern/build.rs','啟動現代客戶端.ps1')){if(Test-Path -LiteralPath (Join-Path $taskRoot $name)){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash}}
        @{status='natural_expiry_running';exe_sha256=$exeSha;immutable_exe=$exe;original_exe=$originalExe;jar_sha256=$sha;development_inputs=$pins;expected_maximum_seconds=$(if($ActionProfile -in @('rental-item-expiry','rental-equipped-expiry')){3600}else{7200});clock_or_authority_injection=$false}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $output 'natural-expiry-start.json') -Encoding utf8
    }
    $serverJvmArguments=@('-Dfile.encoding=UTF-8',('-Dlito.resourceProfile='+$ResourceProfile),('-Dlito.extendedItemDetails='+$ExtendedItemDetails.IsPresent.ToString().ToLowerInvariant()))
    if($CommonCatalogPath){$serverJvmArguments+=(Quote ('-Dlito.commonCatalog='+$CommonCatalogPath))}
    $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList ($serverJvmArguments+@('-cp',(Quote ($classes+';'+$snapshot)),'local.lito381.server.NativePeerHarness',(Quote $root),$ActionProfile)) -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output 'server.log') -RedirectStandardError (Join-Path $output 'server.stderr.txt')
    $serverIdentity=Get-CimInstance Win32_Process -Filter ('ProcessId='+$server.Id)
    if(!$serverIdentity -or [Math]::Abs(($serverIdentity.CreationDate.ToUniversalTime()-$server.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$serverIdentity.CommandLine.Contains('-Dlito.resourceProfile='+$ResourceProfile)){throw 'Owned server launch identity/profile mismatch'}
    @{pid=$server.Id;creation_utc=$server.StartTime.ToUniversalTime().ToString('o');resource_profile=$ResourceProfile;extended_item_details=$ExtendedItemDetails.IsPresent;common_catalog_sha256=$commonCatalogSHA;exe_sha256=$exeSha;jar_sha256=$sha;command_line=$serverIdentity.CommandLine}|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $output 'server-launch-identity.json') -Encoding utf8
    $until=[DateTime]::UtcNow.AddSeconds(15);$ready=$null
    while(!$ready){
        if($server.HasExited -or [DateTime]::UtcNow -ge $until){throw 'Real-peer server readiness failed'}
        if(Test-Path -LiteralPath (Join-Path $root 'ready.json')){try{$ready=Get-Content -LiteralPath (Join-Path $root 'ready.json') -Raw|ConvertFrom-Json}catch{}}
        if(!$ready){Start-Sleep -Milliseconds 20}
    }
    if($ready.jar_sha256 -ne $sha){throw 'Server loaded different JAR'}
    foreach($role in $peerRoles){
        $info=[System.Diagnostics.ProcessStartInfo]::new($exe)
        $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
        $info.RedirectStandardInput=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
        $info.StandardInputEncoding=[System.Text.UTF8Encoding]::new($false);$info.StandardOutputEncoding=[System.Text.UTF8Encoding]::new($false)
        $info.Environment['LITO_NATIVE_PEER_ROOT']=$root
        $info.Environment['LITO_NATIVE_PEER_PROFILE']=$ActionProfile
        $info.ArgumentList.Add('--probe-native-lobby');$info.ArgumentList.Add('127.0.0.1:'+$ready.port)
        $process=[System.Diagnostics.Process]::Start($info)
        $entry=[pscustomobject]@{Role=$role;Process=$process;Out=$process.StandardOutput.ReadToEndAsync();Err=$process.StandardError.ReadToEndAsync();Saved=$false;PeakBytes=[long]0}
        $clients.Add($entry)
        @{pid=$process.Id;creation_utc=$process.StartTime.ToUniversalTime().ToString('o');exe_sha256=$exeSha;role=$role;command_line=(Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)).CommandLine}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $output ($role+'-launch-identity.json')) -Encoding utf8
        # Credentials identify isolated MemoryAccountRepository records only.
        $account=if($role -eq 'a'){'PeerFixture'}elseif($role -eq 'b'){'PeerFixture2'}else{'PeerFixture5'}
        $inputData=@{account=$account;password='TestOnlyPassword';source_index=1;lobby_mode=('room-world-peer-'+$role)}|ConvertTo-Json -Compress
        $process.StandardInput.Write($inputData);$process.StandardInput.Close()
        if($ActionProfile -eq 'turn-undead-quest' -and $role -eq 'b'){
            $joinUntil=[DateTime]::UtcNow.AddSeconds(30);$joinedFirst=$false
            while(!$joinedFirst){
                if($process.HasExited -or $server.HasExited){throw 'First Quest participant exited before room membership'}
                $tracePath=Join-Path $root 'server-trace.jsonl'
                if(Test-Path -LiteralPath $tracePath){
                    foreach($line in Get-Content -LiteralPath $tracePath -Tail 12){try{$row=$line|ConvertFrom-Json;if($row.state.object_id -eq 3 -and $row.state.room -gt 0){$joinedFirst=$true}}catch{}}
                }
                if([DateTime]::UtcNow -gt $joinUntil){throw 'First Quest participant did not establish observed room membership'}
                if(!$joinedFirst){Start-Sleep -Milliseconds 50}
            }
        }

    }
    $until=[DateTime]::UtcNow.AddSeconds($(if($longExpiry){7950}elseif($ActionProfile -eq 'rental-merchants'){885}elseif($ActionProfile -eq 'turn-undead-quest'){615}elseif($ActionProfile -eq 'honor-respawn'){375}else{225}))
    while(@($clients|Where-Object {!$_.Process.HasExited}).Count -gt 0){
        foreach($c in $clients){
            if(!$c.Process.HasExited){
                try{$c.Process.Refresh();$c.PeakBytes=[Math]::Max($c.PeakBytes,$c.Process.PeakWorkingSet64)}catch{if(!$c.Process.HasExited){throw}}
            }
            if($c.Process.HasExited -and !$c.Saved){
                $c.Out.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($c.Role+'-client.json')) -Encoding utf8
                $c.Err.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($c.Role+'-client.stderr.txt')) -Encoding utf8
                $c.Saved=$true
                if($c.Process.ExitCode -ne 0){throw ('Peer '+$c.Role+' failed; inspect '+$output)}
            }
        }
        if([DateTime]::UtcNow -ge $until){throw 'Real-peer test timeout'}
        Start-Sleep -Milliseconds 50
    }
    foreach($c in $clients){
        if(!$c.Saved){
            $c.Out.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($c.Role+'-client.json')) -Encoding utf8
            $c.Err.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($c.Role+'-client.stderr.txt')) -Encoding utf8
            $c.Saved=$true
        }
        if($c.Process.ExitCode -ne 0){throw ('Peer '+$c.Role+' failed; inspect '+$output)}
    }
    if(!$server.WaitForExit(8000) -or $server.ExitCode -ne 0){throw 'Real-peer server failed shutdown'}
    $responses=@{}
    foreach($role in $peerRoles){
        $r=Get-Content -LiteralPath (Join-Path $output ($role+'-client.json')) -Raw|ConvertFrom-Json
        $ownedClient=$clients|Where-Object {$_.Role -eq $role}|Select-Object -First 1
        $r|Add-Member -NotePropertyName process_peak_working_set_bytes -NotePropertyValue $(if($ownedClient.PeakBytes -gt 0){$ownedClient.PeakBytes}else{$null})
        $r|Add-Member -NotePropertyName memory_measurement -NotePropertyValue 'Windows process lifetime peak sampled while alive every 50 ms; lower bound if final peak occurs after last sample'
        if($ActionProfile -ne 'melee'){
            $status=if($ActionProfile -eq 'turn-undead-quest'){'real_peer_turn_undead_completed'}elseif($ActionProfile -eq 'rental-return'){'real_peer_rental_return_observed'}elseif($ActionProfile -in @('rental-entry','rental-shop','rental-citizen','rental-hotel','rental-hotel-entry','rental-hotel-middle-entry','rental-hotel-share','rental-merchants','rental-weight','rental-item-expiry','rental-equipped-expiry','rental-hotel-expiry','rental-hotel-middle-expiry')){'real_peer_rental_entry_completed'}elseif($ActionProfile -eq 'classic-entry'){'real_peer_classic_entry_observed'}elseif($ActionProfile -eq 'room-admin'){'real_peer_room_admin_completed'}elseif($ActionProfile -in @('doors','movement')){'real_peer_world_completed'}elseif($ActionProfile -eq 'honor-respawn'){'real_peer_lifecycle_completed'}else{'real_peer_action_completed'}
            if($r.status -ne $status -or $r.action_profile -ne $ActionProfile -or !$r.worker_quiescent -or $r.shutdown -ne 'closed'){throw 'Incomplete peer action'}
            $responses[$role]=$r
            continue
        }
        if($r.status -ne 'two_real_peer_rounds_completed' -or $r.rounds.Count -ne 2 -or !$r.worker_quiescent -or $r.shutdown -ne 'closed'){throw 'Incomplete real-peer client result'}
        $id=if($role -eq 'a'){1}else{3}
        for($round=0;$round -lt 2;$round++){
            $row=$r.rounds[$round];$won=($role -eq 'a') -eq ($round -eq 0)
            if($row.start.world.status.object_id -ne $id -or $row.end.world.status.object_id -ne $id){throw 'Player identity mismatch'}
            if($row.end.world.scores.Count -lt 2 -or $null -eq $row.end.world.battle_result){throw 'Missing peer scoreboard'}
            if($won -and ($row.attacks_sent -lt 1 -or $row.end.world.hp[0] -le 0)){throw 'Winner did not fight/survive'}
            if(!$won -and ($row.end.world.hp[0] -ne 0 -or $row.end.world.death_confirmations_sent -ne 1)){throw 'Loser missing actual death/confirmation'}
            if(!(Test-Path -LiteralPath $row.render.png) -or !(Test-Path -LiteralPath $row.start.render.png)){throw 'Missing original resource frames'}
        }
        if($r.rounds[1].start.world.generation -le $r.rounds[0].start.world.generation -or $r.transport.epoch -ne 2){throw 'World/connection lifetime mismatch'}
        $responses[$role]=$r
    }
    $ref=Get-Content -LiteralPath (Join-Path $root 'result.json') -Raw|ConvertFrom-Json
    if($ref.actual_client_sessions -ne ($peerCount*2) -or $ref.game_sessions -ne $peerCount -or $ref.authority_mutations -ne 0 -or $ref.sql_access -or $ref.state_persistent -or !$ref.memberships_empty -or $ref.jar_sha256 -ne $sha){throw 'Server isolation evidence invalid'}
    if(Test-Path -LiteralPath (Join-Path $root 'state.properties')){throw 'Unexpected persistence'}
    $trace=@(Get-Content -LiteralPath (Join-Path $root 'server-trace.jsonl')|ForEach-Object {$_|ConvertFrom-Json})
    foreach($id in $(if($peerCount -eq 3){@(1,3,6)}else{@(1,3)})){
        if($ActionProfile -eq 'classic-entry'){continue} # Rejected selection has no player object ID.
        $states=@($trace|Where-Object {$_.state.object_id -eq $id})
        $requiredPhase=if($ActionProfile -eq 'room-admin'){'ROOM'}else{'BATTLE'}
        if(($ActionProfile -eq 'melee' -and @($states|Where-Object {$_.state.hp -eq 0}).Count -lt 1) -or @($states|Where-Object {$_.state.phase -eq $requiredPhase}).Count -lt 1){throw 'Server did not observe required player phase/death'}
    }
    $sourceAfter=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
    if((Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash -ne $exeSha){throw 'EXE changed during test'}
    $receipt=[ordered]@{status='verified_two_real_peer_rounds';profile=$BuildProfile;resource_profile=$ResourceProfile;extended_item_details=$ExtendedItemDetails.IsPresent;common_catalog_sha256=$commonCatalogSHA;jar_sha256=$sha;exe_sha256=$exeSha;source_sha256_at_end=$sourceAfter;source_matches_snapshot=($sourceAfter -eq $sha);runtime=$root;output=$output;server=$ref;clients=$responses;scope='Two real peers, two melee rounds, winner rotation, death ACK, results, same GAME reentry; not full gameplay or GUI'}
    if($ActionProfile -ne 'melee'){
        $receipt.status='verified_real_peer_action';$receipt.action_profile=$ActionProfile
        $receipt.scope=$(if($peerCount -eq 3){'Three actual players, original source characters; C moves normally into line; two accepted target markers and authoritative damage; no GUI or full-game claim'}else{'Two actual players, isolated initial fixture, authoritative action and resource rendering; no GUI or full-game claim'})
    }
    $receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8
    Copy-Item -LiteralPath (Join-Path $root 'server-trace.jsonl') -Destination (Join-Path $output 'server-trace.jsonl')
    if($ActionProfile -eq 'room-admin'){$receipt.scope='Two actual players: room creation, password, settings, observer, owner transfer, kick and rejected rejoin; no battle or GUI claim';$receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
    if($ActionProfile -eq 'classic-entry'){$receipt.status='observed_classic_entry_boundary';$receipt.scope='C05 using exact native directory identity; diagnose reply and world-transition boundary, not a successful world-entry claim';$receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
    if($ActionProfile -eq 'rental-entry'){$receipt.scope='Enabled isolated rental-town profile: service directory, loadout refusal/acceptance, direct-room bootstrap and map500 rendering; no normal-return, shop, GUI or production-enabled claim';$receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
    if($ActionProfile -eq 'rental-shop'){$receipt.scope='Isolated rental entry and authoritative merchant refusal/rent/return; no GUI, normal world return or production-enabled claim';$receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
    if($ActionProfile -eq 'rental-return'){$receipt.status='observed_rental_return_boundary';$receipt.scope='Actual C10/S65/CD9 return request and bounded observation; no normal return or reentry success claim';$receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
    $validator=if($ActionProfile -in @('detection-invisibility','detection-refused','invisibility-refused','invisibility-missing')){'verify-selector-peer-visibility.py'}elseif($ActionProfile -match '^heal(19|35|57)(-full|-refused)?$'){'verify-real-peer-higher-heal.py'}elseif($ActionProfile -eq 'turn-undead-quest'){'verify-real-peer-turn-undead.py'}elseif($ActionProfile.StartsWith('poison-') -or $ActionProfile -eq 'cure-refused'){'verify-real-peer-poison.py'}elseif($ActionProfile -eq 'rental-return'){'verify-real-peer-rental-return.py'}elseif($ActionProfile -in @('rental-entry','rental-shop','rental-citizen','rental-hotel','rental-hotel-entry','rental-hotel-middle-entry','rental-hotel-share','rental-merchants','rental-weight','rental-item-expiry','rental-equipped-expiry','rental-hotel-expiry','rental-hotel-middle-expiry')){'verify-real-peer-rental-entry.py'}elseif($ActionProfile -eq 'classic-entry'){'verify-real-peer-classic-entry.py'}elseif($ActionProfile -eq 'room-admin'){'verify-real-peer-room-admin.py'}elseif($ActionProfile -eq 'melee'){'verify-real-peer-authority.py'}elseif($ActionProfile -in @('doors','movement')){'verify-real-peer-world.py'}elseif($ActionProfile -eq 'honor-respawn'){'verify-real-peer-lifecycle.py'}else{'verify-real-peer-actions.py'}
    $validatorArguments=@('-X','utf8',(Join-Path $taskValidatorRoot $validator),(Join-Path $output 'receipt.json'))
    if($VisibilityNormalInputsPath){
        if($validator -ne 'verify-selector-peer-visibility.py'){throw 'Visibility normal inputs require a visibility action profile'}
        $validatorArguments+=@('--normal-inputs',$VisibilityNormalInputsPath)
    }
    & python @validatorArguments
    if($LASTEXITCODE -ne 0){throw 'Cross-client authority comparison failed'}
    if($ActionProfile -eq 'rental-citizen'){& python -X utf8 (Join-Path $PSScriptRoot 'verify-real-peer-rental-citizen.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Citizen authority comparison failed'}}
    if($ActionProfile -eq 'rental-hotel'){& python -X utf8 (Join-Path $PSScriptRoot 'verify-real-peer-rental-hotel.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Hotel authority comparison failed'}}
    if($ActionProfile -in @('rental-hotel-entry','rental-hotel-middle-entry','rental-hotel-expiry','rental-hotel-middle-expiry')){& python -X utf8 (Join-Path $taskValidatorRoot 'verify-real-peer-rental-hotel-entry.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Hotel entry comparison failed'}}
    if($ActionProfile -eq 'rental-hotel-share'){& python -X utf8 (Join-Path $PSScriptRoot 'verify-real-peer-rental-hotel-share.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Hotel shared key comparison failed'}}
    if($ActionProfile -eq 'rental-merchants'){& python -X utf8 (Join-Path $PSScriptRoot 'verify-real-peer-rental-merchants.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'All merchant comparison failed'}}
    if($longExpiry -and $ActionProfile -notin @('rental-item-expiry','rental-equipped-expiry')){& python -X utf8 (Join-Path $taskValidatorRoot 'verify-real-peer-rental-hotel-expiry.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Natural hotel expiry comparison failed'}}
    if($ActionProfile -eq 'rental-item-expiry'){& python -X utf8 (Join-Path $taskValidatorRoot 'verify-real-peer-rental-item-expiry.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Natural merchant expiry comparison failed'}}
    if($ActionProfile -eq 'rental-equipped-expiry'){& python -X utf8 (Join-Path $taskValidatorRoot 'verify-real-peer-equipped-expiry.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Equipped natural expiry comparison failed'}}
    if($ActionProfile -eq 'rental-weight'){& python -X utf8 (Join-Path $PSScriptRoot 'verify-real-peer-rental-weight.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Weight rejection comparison failed'}}
    if($ActionProfile -like 'rental-*'){& python -X utf8 (Join-Path $taskValidatorRoot 'verify-real-peer-attributes.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Received attribute comparison failed'}}
    if($ActionProfile -in @('movement','rental-shop')){& python -X utf8 (Join-Path $PSScriptRoot 'verify-real-peer-minimap.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Minimap wire/render comparison failed'}}
    if($sourceAfter -ne $sha){throw 'Source changed during test; results qualify snapshot only'}
    . (Join-Path $PSScriptRoot 'owned-peer-validation-binding.ps1')
    [void](Complete-OwnedPeerValidation -ReceiptPath (Join-Path $output 'receipt.json'))
    Write-Output ('Verified real-peer rounds: '+$output)
} catch {
    # Keep any emitted runtime receipt immutable when an independent check fails.
    @{status='failed';error=$_.Exception.Message}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $output 'runner-error.json') -Encoding utf8
    throw
} finally {
    foreach($c in $clients){
        if(!$c.Process.HasExited){Stop-OwnedPeer $c.Process}
        if(!$c.Saved){
            $c.Out.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($c.Role+'-client.json')) -Encoding utf8
            $c.Err.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($c.Role+'-client.stderr.txt')) -Encoding utf8
        }
        $c.Process.Dispose()
    }
    if($null -ne $server){if(!$server.HasExited){Stop-OwnedPeer $server};$server.Dispose()}
    foreach($name in @('result.json','server-trace.jsonl','a-client-trace.json','b-client-trace.json','c-client-trace.json','a-tu-error.json','b-tu-error.json','a-poison-journal.json','b-poison-journal.json','a-poison-error-snapshot.json','b-poison-error-snapshot.json','a-visibility-journal.json','b-visibility-journal.json','a-visibility-error-snapshot.json','b-visibility-error-snapshot.json','ready.json')){
        $file=Join-Path $root $name;if(Test-Path -LiteralPath $file){Copy-Item -LiteralPath $file -Destination (Join-Path $output $name)}
    }
    Pop-Location
}
