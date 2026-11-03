#Requires -Version 7.0
[CmdletBinding()]param([ValidateSet('Debug','Release')][string]$BuildProfile='Debug',[string]$SourceManifest='',[string]$NormalQualification='',[switch]$SkipBuild,[switch]$InventoryOnly,[switch]$GroundOnly)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
. (Join-Path $PSScriptRoot 'owned-protocol-fixture.ps1')
. (Join-Path $PSScriptRoot 'owned-current-protocol-context.ps1')
$output=Join-Path $taskRoot '驗證紀錄\current\native-item-actions'
$runtime=Join-Path $project 'target\current\native-item-actions'
if($InventoryOnly){
    $output=Join-Path $taskRoot '驗證紀錄/current/native-item-actions-inventory'
    $runtime=Join-Path $project 'target/current/native-item-actions-inventory'
}
$classes=Join-Path $project 'target\current\compatibility-oracle'
if($GroundOnly){
    if($InventoryOnly){throw 'Choose InventoryOnly or GroundOnly'}
    $output=Join-Path $taskRoot '驗證紀錄/current/native-item-actions-ground'
    $runtime=Join-Path $project 'target/current/native-item-actions-ground'
    $classes=Join-Path $runtime 'oracle'
}
if($InventoryOnly){$classes=Join-Path $runtime 'oracle'}
$snapshot=Join-Path $project 'target\current\server-reference\formal-server.jar'
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project 'target\current\x86_64-pc-windows-msvc\debug\LiToModern.exe'
$context=New-OwnedCurrentProtocolContext -ToolPath $PSCommandPath -Group (Split-Path -Leaf $output) -BuildProfile $BuildProfile -SourceManifest $SourceManifest -NormalQualification $NormalQualification -ExtraInputs @('tests/fixtures/native-item-actions-current.json','tests/oracle/NativeGameHarness.java','tests/oracle/NativeItemActionsOracle.java')
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
    & (Join-Path $jdk 'javac.exe') -encoding UTF-8 -cp $snapshot -d $classes 'tests/oracle/NativeItemActionsOracle.java' 'tests/oracle/NativeGameHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Game harness compile failed'}
    & (Join-Path $jdk 'java.exe') -cp ($classes+';'+$snapshot) local.lito381.server.NativeItemActionsOracle > (Join-Path $output 'oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Game oracle failed'}
    $parityOutput=Join-Path $output 'parity-oracle';New-Item -ItemType Directory -Path $parityOutput|Out-Null
    $parity.Add((Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-item-actions-current.json') -ActualOracle (Join-Path $output 'oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeItemActionsOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $parityOutput))
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile|Out-Null}
    $modes=if($GroundOnly){@('room-world-item-drop','room-world-item-drop-refused')}elseif($InventoryOnly){@('inventory-actions','inventory-actions-refused')}else{@('inventory-actions','inventory-actions-refused','room-world-item-heal','room-world-item-drop','room-world-item-drop-refused')}
    foreach($mode in $modes){
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
            if($mode.StartsWith('room-world-item-drop')){
                $refused=$mode.EndsWith('refused');$combat=$response.combat;$world=$combat.result.world;$rows=@($combat.item_actions)
                if($response.game.phase -ne 'battle_pending' -or !$response.worker_quiescent -or $response.shutdown -ne 'closed' -or !$response.inventory_shutdown_cleared){throw 'Drop lifecycle'}
                if($rows.Count -ne 2 -or $combat.attacks_sent -ne 0 -or $world.own_attack_packets -ne 0 -or $world.hp[0] -ne 50 -or $reference.final_player_hp -ne 50 -or $world.local_corrections -ne 0){throw 'Drop changed combat authority'}
                foreach($row in $rows){
                    if(!$row.duplicate_pending_rejected -or $row.owner.connection -ne 2 -or $row.quantity -ne 2){throw 'Drop owner/quantity'}
                    if($refused){if($row.feedback -ne 'NoResponse' -or $row.before.count -ne 4 -or ($row.before|ConvertTo-Json -Depth 6 -Compress) -cne ($row.after|ConvertTo-Json -Depth 6 -Compress) -or $row.ground.Count -ne 0){throw 'Refused drop authority'}}
                    else{if($row.feedback -ne 'InventoryChanged'){throw 'Drop update missing'}}
                }
                if(!$refused){
                    if($rows[0].before.count -ne 4 -or $rows[0].after.count -ne 2 -or $rows[1].before.count -ne 2 -or $null -ne $rows[1].after -or $rows[0].ground.Count -ne 1 -or $rows[1].ground.Count -ne 2){throw 'Drop stack/ground lifecycle'}
                    foreach($g in $rows[1].ground){
                        if($g.graphics -ne 773 -or $g.experience -ne 2 -or $g.x -ne $reference.world_x -or $g.y -ne $reference.world_y -or $g.id -eq $rows[0].before.id){throw 'Ground object wire identity'}
                        if(@($combat.frames[-1].actors|Where-Object {$_.id -eq $g.id -and $_.rendered -and $_.phase -eq 'ground_item'}).Count -ne 1){throw 'Ground item not rendered'}
                    }
                }
                $view=$combat.result.inventory
                if(@($view.items.PSObject.Properties).Count -ne $reference.inventory.Count){throw 'Drop inventory size'}
                foreach($item in $reference.inventory){
                    $actual=$view.items.PSObject.Properties[[string]$item.id].Value
                    foreach($field in @('id','use_type','charge','icon','blessing','count','identified','name')){if($actual.$field -cne $item.$field){throw "Drop authority differs: $field"}}
                    if(($actual.detail -join ',') -ne ($item.detail -join ',')){throw 'Drop detail mismatch'}
                }
                foreach($f in $combat.frames){if($f.png -and !(Test-Path -LiteralPath $f.png)){throw 'Drop PNG missing'}}
            }elseif($mode -eq 'room-world-item-heal'){
                $combat=$response.combat;$world=$combat.result.world;$rows=@($combat.item_actions)
                if($response.game.phase -ne 'battle_pending' -or !$response.worker_quiescent -or $response.shutdown -ne 'closed' -or !$response.inventory_shutdown_cleared){throw 'World potion lifecycle'}
                if($rows.Count -ne 2 -or $combat.attacks_sent -ne 1 -or $world.own_attack_packets -ne 1 -or $world.own_cast_packets -ne 0 -or $combat.held_active_end -or $combat.route.Count -lt 1){throw 'World potion combat authority'}
                if($rows[0].hp_before[0] -ge 50 -or $rows[0].hp_after[0] -ne 50 -or $rows[1].hp_after[0] -ne 50 -or $reference.final_player_hp -ne 50 -or $world.hp[0] -ne 50){throw 'World potion HP restoration'}
                if($rows[0].before.count -ne 2 -or $rows[0].after.count -ne 1 -or $rows[1].before.count -ne 1 -or $null -ne $rows[1].after -or $rows[0].before.id -ne $rows[1].before.id){throw 'Potion stack lifecycle'}
                foreach($row in $rows){if(!$row.duplicate_pending_rejected -or !$row.effect_finished -or $row.feedback -ne 'InventoryChanged' -or $row.owner.connection -ne 2){throw 'Potion owner/wait mismatch'}}
                $fx=$combat.frames[-1].item_effects
                if($fx.started -ne 2 -or $fx.active -ne 0 -or $fx.omitted -ne 0 -or @($combat.frames|ForEach-Object{$_.item_effects.effects}|Where-Object drawn).Count -lt 2){throw 'Potion original effect lifecycle'}
                if($reference.inventory.Count -ne 1 -or @($combat.result.inventory.items.PSObject.Properties).Count -ne 1){throw 'Potion exhausted stock remains'}
                $view=$combat.result.inventory
                foreach($item in $reference.inventory){
                    $actual=$view.items.PSObject.Properties[[string]$item.id].Value
                    foreach($field in @('id','use_type','charge','icon','blessing','count','identified','name')){if($actual.$field -cne $item.$field){throw "Potion final authority differs: $field"}}
                    if(($actual.detail -join ',') -ne ($item.detail -join ',')){throw 'Potion detail mismatch'}
                }
                foreach($f in $combat.frames){if($f.png -and !(Test-Path -LiteralPath $f.png)){throw 'Potion PNG missing'}}
            }else{
            if($response.game.phase -ne 'lobby_ready' -or !$response.worker_quiescent -or $response.shutdown -ne 'closed' -or $response.pointer_bits -ne 64){throw 'Inventory client did not settle'}
            $refused=$mode -eq 'inventory-actions-refused'
            $expected=if($refused){2}else{4}
            if($response.item_actions.Count -ne $expected -or $response.lobby_history.Count -ne 1 -or !$response.inventory_shutdown_cleared){throw 'Item action lifecycle/count'}
            $prior=$null
            foreach($row in $response.item_actions){
                if($row.owner.connection -ne 2 -or $row.owner.selection -ne 1 -or !$row.duplicate_pending_rejected){throw 'Item request owner/pending'}
                if($row.before.id -ne $row.after.id -or $row.before.count -ne $row.after.count){throw 'Item request authority'}
                if($refused){
                    if($row.feedback -ne 'NoResponse' -or ($row.before|ConvertTo-Json -Depth 6 -Compress) -cne ($row.after|ConvertTo-Json -Depth 6 -Compress)){throw 'Refused potion changed authority'}
                }else{
                    if($row.feedback -ne 'InventoryChanged' -or $row.before.name -ceq $row.after.name){throw 'Equipment did not update'}
                }
            }
            $view=$response.lobby_history[0].inventory
            if(@($view.items.PSObject.Properties).Count -ne 4 -or $reference.inventory.Count -ne 4){throw 'Action stock fixture'}
            foreach($item in $reference.inventory){
                $actual=$view.items.PSObject.Properties[[string]$item.id].Value
                if($null -eq $actual){throw 'Missing authoritative item'}
                foreach($field in @('id','use_type','charge','icon','blessing','count','identified','name')){if($actual.$field -cne $item.$field){throw "Item action final authority differs: $field"}}
                if(($actual.detail -join ',') -ne ($item.detail -join ',')){throw 'Item action detail changed'}
            }
            if(!$refused){
                $rows=$response.item_actions
                if($rows[0].before.use_type -ne 1 -or $rows[1].before.use_type -ne 23 -or $rows[0].before.id -ne $rows[2].before.id -or $rows[1].before.id -ne $rows[3].before.id){throw 'Equipment sequence'}
                for($n=0;$n -lt 2;$n++){
                    if($rows[$n].after.name -cne $rows[$n+2].before.name -or $rows[$n].before.name -cne $rows[$n+2].after.name){throw 'Equipment toggle did not restore presentation'}
                }
            }
            }
            if($reference.status -ne 'verified' -or $reference.jar_sha256 -ne $sha -or !$reference.actual_client_sessions -or $reference.sql_access -or $reference.state_persistent -or (Test-Path -LiteralPath (Join-Path $modeRoot 'state.properties'))){throw 'Isolation/session evidence failed'}
            $presentationPins=Save-OwnedProtocolPresentationFiles -Response $response -Output $output -Mode $mode
            $receipts.Add([ordered]@{mode=$mode;client=$response;server=$reference;verified=$true;presentation_file_pins=$presentationPins})
            Write-Output "$mode native item actions/authority verified"
        }finally{
            foreach($owned in @($client,$server)){if($null -ne $owned){if(!$owned.HasExited){Stop-OwnedLitoValidationChild -Process $owned -ScopeRoot $project};$owned.Dispose()}}
        }
    }
    $receiptPath=Join-Path $output 'receipt.json'
    Assert-OwnedCurrentProtocolContext $context
    if((Get-FileHash -LiteralPath $snapshot).Hash -ne $sha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $sha){throw 'Current reference identity drift'}
    [ordered]@{schema=2;profile=$BuildProfile;output=$output;runtime=$runtime;immutable_receipt=$receiptPath;source_pins=$context.SourcePins;compilation_inputs_checked=$context.Inputs.Count;compilation_source_manifest=$context.SourceManifest;compilation_source_manifest_sha256=$context.SourceManifestSHA;fixture_parity=$parity;status='verified_native_item_action_subset';jar_sha256=$sha;exe_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash;cases=$receipts;sql_access=$false;game_endpoint_connected=$true;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 20|Set-Content -LiteralPath $receiptPath -Encoding utf8
    Write-Output ('PROTOCOL_RECEIPT='+$receiptPath)
}finally{Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp}
