#Requires -Version 7.0
[CmdletBinding()]param([switch]$SkipBuild,[switch]$ReturnServices,[switch]$ReturnShared,[switch]$BattleWithdrawal,[switch]$FormalWithdrawalBaseline,[ValidateSet('Debug','Release')][string]$BuildProfile='Debug',[string]$ReferenceJar,
    [ValidateSet('melee','heal','bolt','bolt-refused','fire-arrow','fire-arrow-refused','wind-cutter','wind-cutter-refused','wall','wall-refused','ranged','item-heal','doors','movement','honor-respawn','room-admin','classic-entry','rental-entry','rental-shop','rental-citizen','rental-hotel','rental-hotel-entry','rental-hotel-middle-entry','rental-hotel-share','rental-merchants','rental-weight','rental-item-expiry','rental-equipped-expiry','rental-hotel-expiry','rental-hotel-middle-expiry','rental-return')][string]$ActionProfile='melee')
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot='D:\lineage開發\3-lito開發\lito重建版'
if($ActionProfile -notin @('rental-return','melee')){throw 'Review candidate only supports explicitly selected validation profiles'}
if($ReturnServices -and $ReturnShared){throw 'Choose one rental return variant'}
if($BattleWithdrawal -and $ActionProfile -ne 'melee'){throw 'BattleWithdrawal requires melee'}
if($FormalWithdrawalBaseline -and !$BattleWithdrawal){throw 'Formal baseline is only for explicit battle withdrawal comparison'}
$project=Join-Path $taskRoot 'LiTo381-Modern'
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-review-candidate'
if($ActionProfile -in @('rental-hotel-expiry','rental-hotel-middle-expiry','rental-item-expiry','rental-equipped-expiry')){$stamp+='-'+$ActionProfile}
$root=Join-Path $project ('target/current/native-real-peers/'+$stamp)
$output=Join-Path $taskRoot ('驗證紀錄/current/repair-candidate-peers/'+$stamp)
$classes=Join-Path $root 'oracle'
$taskValidatorRoot=Join-Path $taskRoot '維護工具'
$snapshot=if($ReferenceJar){[IO.Path]::GetFullPath($ReferenceJar)}else{Join-Path $PSScriptRoot 'rental-return-review-candidate.jar'}
$allowedRoot=[IO.Path]::GetFullPath((Join-Path $project 'target/current'))+[IO.Path]::DirectorySeparatorChar
if(!$snapshot.StartsWith($allowedRoot,[StringComparison]::OrdinalIgnoreCase)){throw 'Review JAR must be built inside this project target/current'}
$source=$snapshot
$baseline=Join-Path $project 'target/current/server-reference/formal-server.jar'
$expectedCandidate='6887A337E347D561366E0EB7EB8E93A2B7EA03ADD7A3888FC4BE2B70C3479DA7'
if($FormalWithdrawalBaseline){$snapshot=$baseline;$source=$snapshot;$expectedCandidate='BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43'}
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLower()+'/LiToModern.exe')
New-Item -ItemType Directory -Path $root,$output,$classes -Force|Out-Null
$pin=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/reference-snapshot.json') -Raw|ConvertFrom-Json
$sha=(Get-FileHash -LiteralPath $snapshot -Algorithm SHA256).Hash
if($sha -ne $expectedCandidate -or (Get-FileHash -LiteralPath $baseline).Hash -ne $pin.sha256 -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $pin.sha256){throw 'Candidate or formal baseline drift'}
function Quote([string]$text){if($text.Contains('"')){throw 'Invalid process argument'};'"'+$text+'"'}
$server=$null;$clients=[System.Collections.Generic.List[object]]::new()
Push-Location $project
try {
    & (Join-Path $jdk 'javac.exe') -encoding UTF-8 -cp $snapshot -d $classes (Join-Path $PSScriptRoot 'NativePeerHarness.java') *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Real-peer harness compile failed'}
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile *> (Join-Path $output 'build.log');if($LASTEXITCODE -ne 0){throw 'Client build failed'}}
    $exeSha=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
    $longExpiry=$ActionProfile -in @('rental-hotel-expiry','rental-hotel-middle-expiry','rental-item-expiry','rental-equipped-expiry')
    if($longExpiry){
        # Long real-time leases must not hold the working EXE open while development continues.
        $originalExe=$exe;$exe=Join-Path $root 'qualified-client.exe'
        Copy-Item -LiteralPath $originalExe -Destination $exe
        if((Get-FileHash -LiteralPath $exe).Hash -ne $exeSha){throw 'Immutable expiry EXE copy differs'}
        $taskValidatorRoot=Join-Path $output 'validation-snapshot';New-Item -ItemType Directory -Path $taskValidatorRoot|Out-Null
        foreach($name in @('verify-real-peer-rental-entry.py','verify-real-peer-rental-hotel-entry.py','verify-real-peer-rental-hotel-expiry.py','verify-real-peer-rental-item-expiry.py','verify-real-peer-equipped-expiry.py','verify-real-peer-attributes.py')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination (Join-Path $taskValidatorRoot $name)}
        Copy-Item -LiteralPath (Join-Path $project 'tests/fixtures/native-portal-current.json') -Destination (Join-Path $taskValidatorRoot 'native-portal-current.json')
        Copy-Item -LiteralPath (Join-Path $project 'tests/fixtures/native-rental-merchant-current.json') -Destination (Join-Path $taskValidatorRoot 'native-rental-merchant-current.json')
        $pins=[ordered]@{}
        foreach($folder in @('LiTo381-Modern/src','LiTo381-Modern/tests','維護工具')){
            foreach($file in Get-ChildItem -LiteralPath (Join-Path $taskRoot $folder) -Recurse -File|Where-Object {$_.Extension -in @('.rs','.java','.json','.py','.ps1','.html')}){$pins[[IO.Path]::GetRelativePath($taskRoot,$file.FullName).Replace('\','/')]=(Get-FileHash -LiteralPath $file.FullName).Hash}
        }
        foreach($name in @('LiTo381-Modern/Cargo.toml','LiTo381-Modern/Cargo.lock','LiTo381-Modern/build.rs','啟動現代客戶端.ps1')){if(Test-Path -LiteralPath (Join-Path $taskRoot $name)){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash}}
        @{status='natural_expiry_running';exe_sha256=$exeSha;immutable_exe=$exe;original_exe=$originalExe;jar_sha256=$sha;development_inputs=$pins;expected_maximum_seconds=$(if($ActionProfile -in @('rental-item-expiry','rental-equipped-expiry')){3600}else{7200});clock_or_authority_injection=$false}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $output 'natural-expiry-start.json') -Encoding utf8
    }
    $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-Xverify:all','-cp',(Quote ($classes+';'+$snapshot)),'local.lito381.server.NativePeerHarness',(Quote $root),$ActionProfile) -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output 'server.log') -RedirectStandardError (Join-Path $output 'server.stderr.txt')
    $until=[DateTime]::UtcNow.AddSeconds(15);$ready=$null
    while(!$ready){
        if($server.HasExited -or [DateTime]::UtcNow -ge $until){throw 'Real-peer server readiness failed'}
        if(Test-Path -LiteralPath (Join-Path $root 'ready.json')){try{$ready=Get-Content -LiteralPath (Join-Path $root 'ready.json') -Raw|ConvertFrom-Json}catch{}}
        if(!$ready){Start-Sleep -Milliseconds 20}
    }
    if($ready.jar_sha256 -ne $sha){throw 'Server loaded different JAR'}
    foreach($role in @('a','b')){
        $info=[System.Diagnostics.ProcessStartInfo]::new($exe)
        $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
        $info.RedirectStandardInput=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
        $info.StandardInputEncoding=[System.Text.UTF8Encoding]::new($false);$info.StandardOutputEncoding=[System.Text.UTF8Encoding]::new($false)
        $info.Environment['LITO_NATIVE_PEER_ROOT']=$root
        $info.Environment['LITO_NATIVE_PEER_PROFILE']=$ActionProfile
        if($BattleWithdrawal){$info.Environment['LITO_NATIVE_BATTLE_WITHDRAWAL']='1'}
        if($ReturnServices){if($ActionProfile -ne 'rental-return'){throw 'ReturnServices requires rental-return'};$info.Environment['LITO_NATIVE_RETURN_SERVICES']='1'}
        if($ReturnShared){if($ActionProfile -ne 'rental-return'){throw 'ReturnShared requires rental-return'};$info.Environment['LITO_NATIVE_RETURN_SHARED']='1'}
        $info.ArgumentList.Add('--probe-native-lobby');$info.ArgumentList.Add('127.0.0.1:'+$ready.port)
        $process=[System.Diagnostics.Process]::Start($info)
        $entry=[pscustomobject]@{Role=$role;Process=$process;Out=$process.StandardOutput.ReadToEndAsync();Err=$process.StandardError.ReadToEndAsync();Saved=$false;PeakBytes=[long]0}
        $clients.Add($entry)
        # Credentials identify isolated MemoryAccountRepository records only.
        $account=if($role -eq 'a'){'PeerFixture'}else{'PeerFixture2'}
        $inputData=@{account=$account;password='TestOnlyPassword';source_index=1;lobby_mode=('room-world-peer-'+$role)}|ConvertTo-Json -Compress
        $process.StandardInput.Write($inputData);$process.StandardInput.Close()
    }
    $until=[DateTime]::UtcNow.AddSeconds($(if($longExpiry){7950}elseif($ActionProfile -eq 'rental-merchants'){885}elseif($ActionProfile -eq 'honor-respawn'){375}else{225}))
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
    foreach($role in @('a','b')){
        $r=Get-Content -LiteralPath (Join-Path $output ($role+'-client.json')) -Raw|ConvertFrom-Json
        $ownedClient=$clients|Where-Object {$_.Role -eq $role}|Select-Object -First 1
        $r|Add-Member -NotePropertyName process_peak_working_set_bytes -NotePropertyValue $(if($ownedClient.PeakBytes -gt 0){$ownedClient.PeakBytes}else{$null})
        $r|Add-Member -NotePropertyName memory_measurement -NotePropertyValue 'Windows process lifetime peak sampled while alive every 50 ms; lower bound if final peak occurs after last sample'
        if($ActionProfile -ne 'melee'){
            $status=if($ActionProfile -eq 'rental-return'){'real_peer_rental_return_observed'}elseif($ActionProfile -in @('rental-entry','rental-shop','rental-citizen','rental-hotel','rental-hotel-entry','rental-hotel-middle-entry','rental-hotel-share','rental-merchants','rental-weight','rental-item-expiry','rental-equipped-expiry','rental-hotel-expiry','rental-hotel-middle-expiry')){'real_peer_rental_entry_completed'}elseif($ActionProfile -eq 'classic-entry'){'real_peer_classic_entry_observed'}elseif($ActionProfile -eq 'room-admin'){'real_peer_room_admin_completed'}elseif($ActionProfile -in @('doors','movement')){'real_peer_world_completed'}elseif($ActionProfile -eq 'honor-respawn'){'real_peer_lifecycle_completed'}else{'real_peer_action_completed'}
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
            if($BattleWithdrawal){
                if(!$r.battle_withdrawal -or $row.withdrawal_requested -ne $won -or $row.attacks_sent -ne 0 -or $row.end.world.hp[0] -le 0 -or $row.end.world.death_confirmations_sent -ne 0 -or $row.end.world.battle_result -ne (1-$round)){throw 'Ordinary withdrawal state mismatch'}
            }else{
                if($won -and ($row.attacks_sent -lt 1 -or $row.end.world.hp[0] -le 0)){throw 'Winner did not fight/survive'}
                if(!$won -and ($row.end.world.hp[0] -ne 0 -or $row.end.world.death_confirmations_sent -ne 1)){throw 'Loser missing actual death/confirmation'}
            }
            if(!(Test-Path -LiteralPath $row.render.png) -or !(Test-Path -LiteralPath $row.start.render.png)){throw 'Missing original resource frames'}
        }
        if($r.rounds[1].start.world.generation -le $r.rounds[0].start.world.generation -or $r.transport.epoch -ne 2){throw 'World/connection lifetime mismatch'}
        $responses[$role]=$r
    }
    $ref=Get-Content -LiteralPath (Join-Path $root 'result.json') -Raw|ConvertFrom-Json
    if($ref.actual_client_sessions -ne 4 -or $ref.game_sessions -ne 2 -or $ref.authority_mutations -ne 0 -or $ref.sql_access -or $ref.state_persistent -or !$ref.memberships_empty -or $ref.jar_sha256 -ne $sha){throw 'Server isolation evidence invalid'}
    if(Test-Path -LiteralPath (Join-Path $root 'state.properties')){throw 'Unexpected persistence'}
    $trace=@(Get-Content -LiteralPath (Join-Path $root 'server-trace.jsonl')|ForEach-Object {$_|ConvertFrom-Json})
    foreach($id in @(1,3)){
        if($ActionProfile -eq 'classic-entry'){continue} # Rejected selection has no player object ID.
        $states=@($trace|Where-Object {$_.state.object_id -eq $id})
        $requiredPhase=if($ActionProfile -eq 'room-admin'){'ROOM'}else{'BATTLE'}
        if(($ActionProfile -eq 'melee' -and !$BattleWithdrawal -and @($states|Where-Object {$_.state.hp -eq 0}).Count -lt 1) -or @($states|Where-Object {$_.state.phase -eq $requiredPhase}).Count -lt 1){throw 'Server did not observe required player phase/death'}
    }
    $sourceAfter=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
    if((Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash -ne $exeSha){throw 'EXE changed during test'}
    $receipt=[ordered]@{status='verified_two_real_peer_rounds';profile=$BuildProfile;jar_sha256=$sha;exe_sha256=$exeSha;source_sha256_at_end=$sourceAfter;source_matches_snapshot=($sourceAfter -eq $sha);runtime=$root;output=$output;server=$ref;clients=$responses;scope='Two real peers, two melee rounds, winner rotation, death ACK, results, same GAME reentry; not full gameplay or GUI'}
    if($ActionProfile -ne 'melee'){
        $receipt.status='verified_real_peer_action';$receipt.action_profile=$ActionProfile
        $receipt.scope='Two actual players, isolated initial fixture, authoritative action and resource rendering; no GUI or full-game claim'
    }
    if($BattleWithdrawal){$receipt.scope='Two actual ordinary battle withdrawals, alternating requester, live HP preserved, results and same GAME reentry; combat cooldown and physical input separate'}
    $receipt.observer_source_sha256=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'NativePeerHarness.java')).Hash;$receipt.return_services=$ReturnServices.IsPresent;$receipt.return_shared=$ReturnShared.IsPresent;$receipt.candidate_server=(!$FormalWithdrawalBaseline);$receipt.battle_withdrawal=$BattleWithdrawal.IsPresent;$receipt.formal_baseline_sha256=$pin.sha256;$receipt.formal_server_unchanged=$true;$receipt.full_product_verified=$false
    $receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8
    Copy-Item -LiteralPath (Join-Path $root 'server-trace.jsonl') -Destination (Join-Path $output 'server-trace.jsonl')
    if($ActionProfile -eq 'room-admin'){$receipt.scope='Two actual players: room creation, password, settings, observer, owner transfer, kick and rejected rejoin; no battle or GUI claim';$receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
    if($ActionProfile -eq 'classic-entry'){$receipt.status='observed_classic_entry_boundary';$receipt.scope='C05 using exact native directory identity; diagnose reply and world-transition boundary, not a successful world-entry claim';$receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
    if($ActionProfile -eq 'rental-entry'){$receipt.scope='Enabled isolated rental-town profile: service directory, loadout refusal/acceptance, direct-room bootstrap and map500 rendering; no normal-return, shop, GUI or production-enabled claim';$receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
    if($ActionProfile -eq 'rental-shop'){$receipt.scope='Isolated rental entry and authoritative merchant refusal/rent/return; no GUI, normal world return or production-enabled claim';$receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
    if($ActionProfile -eq 'rental-return'){$receipt.status='observed_candidate_rental_return';$receipt.scope='Actual C10/S65/CD9 return request and bounded observation; no normal return or reentry success claim';$receipt|ConvertTo-Json -Depth 100|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
    if($ActionProfile -eq 'melee' -and !$BattleWithdrawal){& python -X utf8 (Join-Path $taskValidatorRoot 'verify-real-peer-authority.py') (Join-Path $output 'receipt.json');if($LASTEXITCODE -ne 0){throw 'Candidate ordinary battle authority comparison failed'}}
    if((Get-FileHash -LiteralPath $baseline).Hash -ne $pin.sha256 -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $pin.sha256){throw 'Formal baseline changed during isolated candidate run'}
    if($sourceAfter -ne $sha){throw 'Source changed during test; results qualify snapshot only'}
    Write-Output ('Observed isolated review-candidate peer run: '+$output)
} finally {
    foreach($c in $clients){
        if(!$c.Process.HasExited){$c.Process.Kill();$c.Process.WaitForExit()}
        if(!$c.Saved){
            $c.Out.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($c.Role+'-client.json')) -Encoding utf8
            $c.Err.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($c.Role+'-client.stderr.txt')) -Encoding utf8
        }
        $c.Process.Dispose()
    }
    if($null -ne $server){if(!$server.HasExited){$server.Kill();$server.WaitForExit()};$server.Dispose()}
    foreach($name in @('result.json','server-trace.jsonl','a-client-trace.json','b-client-trace.json','ready.json')){
        $file=Join-Path $root $name;if(Test-Path -LiteralPath $file){Copy-Item -LiteralPath $file -Destination (Join-Path $output $name)}
    }
    Pop-Location
}
