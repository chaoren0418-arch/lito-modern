#Requires -Version 7.0
[CmdletBinding()]param([ValidateSet('Debug','Release')][string]$BuildProfile='Debug',[string]$SourceManifest='',[string]$NormalQualification='',[switch]$SkipBuild)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
. (Join-Path $PSScriptRoot 'owned-protocol-fixture.ps1')
. (Join-Path $PSScriptRoot 'owned-current-protocol-context.ps1')
$output=Join-Path $taskRoot '驗證紀錄\current\native-ground-pickup'
$runtime=Join-Path $project 'target\current\native-ground-pickup'
$classes=Join-Path $project 'target\current\compatibility-oracle'
$snapshot=Join-Path $project 'target\current\server-reference\formal-server.jar'
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project 'target\current\x86_64-pc-windows-msvc\debug\LiToModern.exe'
$context=New-OwnedCurrentProtocolContext -ToolPath $PSCommandPath -Group (Split-Path -Leaf $output) -BuildProfile $BuildProfile -SourceManifest $SourceManifest -NormalQualification $NormalQualification -ExtraInputs @('tests/fixtures/native-ground-pickup-current.json','tests/oracle/NativeGameHarness.java','tests/oracle/NativeGroundPickupOracle.java')
$output=$context.Output;$outputBase=$context.OutputBase;$runtime=$context.Runtime;$classes=$context.Classes;$exe=$context.Exe
Assert-OwnedCurrentProtocolContext $context
$parity=[System.Collections.Generic.List[object]]::new()
$pin=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄\current\reference-snapshot.json') -Raw|ConvertFrom-Json
$sha=(Get-FileHash -LiteralPath $snapshot -Algorithm SHA256).Hash
if($sha -ne $pin.sha256){throw 'Pinned JAR mismatch'}
function Quote([string]$text){if($text.Contains('"')){throw 'Invalid process argument'};'"'+$text+'"'}
$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$scratch=Join-Path $project 'target\current\scratch'
New-Item -ItemType Directory -Path $scratch -Force|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
$receipts=[System.Collections.Generic.List[object]]::new()
Push-Location $project
try {
    & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $snapshot -d $classes 'tests/oracle/NativeGroundPickupOracle.java' 'tests/oracle/NativeGameHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Game harness compile failed'}
    & (Join-Path $jdk 'java.exe') '-Dfile.encoding=UTF-8' -cp ($classes+';'+$snapshot) local.lito381.server.NativeGroundPickupOracle > (Join-Path $output 'oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Game oracle failed'}
    $parityOutput=Join-Path $output 'parity-oracle';New-Item -ItemType Directory -Path $parityOutput|Out-Null
    $parity.Add((Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-ground-pickup-current.json') -ActualOracle (Join-Path $output 'oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeGroundPickupOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $parityOutput))
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile|Out-Null}
    foreach($mode in @('room-world-pickup','room-world-pickup-stale')){
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
            if(!$client.WaitForExit(100000)){throw 'Native GAME client timed out'}
            $clientText=$outTask.GetAwaiter().GetResult();$clientText|Set-Content -LiteralPath (Join-Path $output ($mode+'-client.json')) -Encoding utf8
            $errTask.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($mode+'-client.stderr.txt')) -Encoding utf8
            if(!$server.WaitForExit(8000)){throw 'Deployed session did not close'}
            if($client.ExitCode -ne 0 -or $server.ExitCode -ne 0){throw "Lobby test failed $mode; inspect mode logs"}
            $response=$clientText|ConvertFrom-Json;$reference=Get-Content -LiteralPath $resultPath -Raw|ConvertFrom-Json
            $stale=$mode.EndsWith('stale');$combat=$response.combat;$rows=@($combat.pickup_actions);$world=$combat.result.world
            if($response.game.phase -ne 'battle_pending' -or !$response.worker_quiescent -or $response.shutdown -ne 'closed' -or !$response.inventory_shutdown_cleared -or $response.pointer_bits -ne 64){throw 'Pickup lifecycle'}
            if($rows.Count -ne 2 -or $combat.attacks_sent -ne 0 -or $combat.skills_sent -ne 0 -or $world.own_attack_packets -ne 0 -or $world.own_cast_packets -ne 0 -or $world.hp[0] -ne 50 -or $reference.final_player_hp -ne 50 -or $combat.route.Count -ne 0){throw 'Pickup combat authority'}
            if(@($combat.pickup_drop.before.items.PSObject.Properties).Count -ne 2 -or @($combat.pickup_drop.after.items.PSObject.Properties).Count -ne 1 -or $combat.pickup_drop.choice.item.count -ne 4){throw 'Pickup fixture drop'}
            for($i=0;$i -lt 2;$i++){
                $row=$rows[$i]
                if(!$row.duplicate_pending_rejected -or !$row.item_pending_rejected -or $row.quantity -ne 2 -or $row.choice.owner.connection -ne 2 -or $row.choice.owner.selection -ne 1){throw 'Pickup ownership/pending'}
                if($row.wire.Count -ne 14 -or $row.wire[0] -ne 11 -or $row.wire[13] -ne 0){throw 'Pickup wire shape'}
                if($row.choice.item.id -ne $rows[0].choice.item.id -or $row.choice.item.graphics -ne 773 -or $row.choice.item.x -ne $reference.world_x -or $row.choice.item.y -ne $reference.world_y){throw 'Pickup ground identity'}
                if($stale){
                    if($row.feedback -ne 'NoResponse' -or $reference.forced_pickup_removal -ne $row.choice.item.id -or ($row.before|ConvertTo-Json -Depth 12 -Compress) -cne ($row.after|ConvertTo-Json -Depth 12 -Compress)){throw 'Stale pickup changed authority'}
                }else{
                    if($row.feedback -ne 'ObservedUpdates' -or $reference.forced_pickup_removal -ne 0 -or $row.before.ground.experience -ne 4-$i*2){throw 'Pickup update authority'}
                    if(($i -eq 0 -and ($row.after.ground.experience -ne 2 -or $row.after.ground.incarnation -eq $row.before.ground.incarnation)) -or ($i -eq 1 -and $null -ne $row.after.ground)){throw 'Pickup partial/full world update'}
                    $before=@($row.before.inventory.items.PSObject.Properties|ForEach-Object Value);$after=@($row.after.inventory.items.PSObject.Properties|ForEach-Object Value)
                    if(($after|Measure-Object count -Sum).Sum-($before|Measure-Object count -Sum).Sum -ne 2){throw 'Pickup inventory quantity'}
                    if(@($after|Where-Object id -eq $row.choice.item.id).Count -ne 0){throw 'Ground object mistaken for inventory object'}
                }
            }
            $view=$combat.result.inventory
            if(@($view.items.PSObject.Properties).Count -ne $reference.inventory.Count){throw 'Pickup inventory size'}
            foreach($item in $reference.inventory){
                $actual=$view.items.PSObject.Properties[[string]$item.id].Value
                foreach($field in @('id','use_type','charge','icon','blessing','count','identified','name')){if($actual.$field -cne $item.$field){throw "Pickup final authority differs: $field"}}
                # Opaque details are last received wire bytes; count-only deltas do not resend regenerated details.
                # The independent capture replay audits every client field, including details.
            }
            $groundId=$rows[0].choice.item.id
            if($stale){if(!$world.actors.PSObject.Properties[[string]$groundId] -or $world.actors.PSObject.Properties[[string]$groundId].Value.experience -ne 4){throw 'Stale pickup locally removed ground'}}
            elseif($world.actors.PSObject.Properties[[string]$groundId]){throw 'Pickup left ground object'}
            $actions=@($combat.frames|ForEach-Object {$_.actors}|Where-Object {$_.id -eq $reference.world_object -and $_.action -eq 15})
            if(($stale -and $actions.Count -ne 0) -or (!$stale -and $actions.Count -eq 0)){throw 'Pickup original action15'}
            foreach($f in $combat.frames){if($f.png -and !(Test-Path -LiteralPath $f.png)){throw 'Pickup PNG missing'}}
            if($reference.status -ne 'verified' -or $reference.jar_sha256 -ne $sha -or !$reference.actual_client_sessions -or $reference.sql_access -or $reference.state_persistent -or (Test-Path -LiteralPath (Join-Path $modeRoot 'state.properties'))){throw 'Isolation/session evidence failed'}
            $presentationPins=Save-OwnedProtocolPresentationFiles -Response $response -Output $output -Mode $mode
            $receipts.Add([ordered]@{mode=$mode;client=$response;server=$reference;verified=$true;presentation_file_pins=$presentationPins})
            Write-Output "$mode native ground pickup/authority verified"
        }finally{
            foreach($owned in @($client,$server)){if($null -ne $owned){if(!$owned.HasExited){Stop-OwnedLitoValidationChild -Process $owned -ScopeRoot $project};$owned.Dispose()}}
        }
    }
    $receiptPath=Join-Path $output 'receipt.json'
    Assert-OwnedCurrentProtocolContext $context
    if((Get-FileHash -LiteralPath $snapshot).Hash -ne $sha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $sha){throw 'Current reference identity drift'}
    [ordered]@{schema=2;profile=$BuildProfile;output=$output;runtime=$runtime;immutable_receipt=$receiptPath;source_pins=$context.SourcePins;compilation_inputs_checked=$context.Inputs.Count;compilation_source_manifest=$context.SourceManifest;compilation_source_manifest_sha256=$context.SourceManifestSHA;fixture_parity=$parity;status='verified_native_ground_pickup_subset';jar_sha256=$sha;exe_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash;cases=$receipts;sql_access=$false;game_endpoint_connected=$true;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 20|Set-Content -LiteralPath $receiptPath -Encoding utf8
    Write-Output ('PROTOCOL_RECEIPT='+$receiptPath)
}finally{Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp}
