#Requires -Version 7.0
[CmdletBinding()]param([ValidateSet('Debug','Release')][string]$BuildProfile='Debug',[string]$SourceManifest='',[string]$NormalQualification='',[switch]$SkipBuild)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
. (Join-Path $PSScriptRoot 'owned-protocol-fixture.ps1')
. (Join-Path $PSScriptRoot 'owned-current-protocol-context.ps1')
$output=Join-Path $taskRoot '驗證紀錄\current\native-reagent-skills'
$runtime=Join-Path $project 'target\current\native-reagent-skills'
$classes=Join-Path $project 'target\current\compatibility-oracle'
$snapshot=Join-Path $project 'target\current\server-reference\formal-server.jar'
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project 'target\current\x86_64-pc-windows-msvc\debug\LiToModern.exe'
$context=New-OwnedCurrentProtocolContext -ToolPath $PSCommandPath -Group (Split-Path -Leaf $output) -BuildProfile $BuildProfile -SourceManifest $SourceManifest -NormalQualification $NormalQualification -ExtraInputs @('tests/fixtures/native-reagent-skills-current.json','tests/oracle/NativeGameHarness.java','tests/oracle/NativeReagentSkillOracle.java')
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
    & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $snapshot -d $classes 'tests/oracle/NativeReagentSkillOracle.java' 'tests/oracle/NativeGameHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Game harness compile failed'}
    & (Join-Path $jdk 'java.exe') '-Dfile.encoding=UTF-8' -cp ($classes+';'+$snapshot) local.lito381.server.NativeReagentSkillOracle > (Join-Path $output 'oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Game oracle failed'}
    $parityOutput=Join-Path $output 'parity-oracle';New-Item -ItemType Directory -Path $parityOutput|Out-Null
    $parity.Add((Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-reagent-skills-current.json') -ActualOracle (Join-Path $output 'oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeReagentSkillOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $parityOutput))
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile|Out-Null}
    foreach($mode in @('room-world-stream','room-world-stream-no-reagent','room-world-stream-no-mana')){
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
            $success=$mode -eq 'room-world-stream';$lowMp=$mode.EndsWith('no-mana')
            $combat=$response.combat;$world=$combat.result.world;$rows=@($combat.reagent_actions)
            $count=if($success){3}else{2};$casts=if($success){2}else{0};$maxMp=if($lowMp){2}else{80};$finalMp=$maxMp-20*$casts
            if($response.game.phase -ne 'battle_pending' -or !$response.worker_quiescent -or $response.shutdown -ne 'closed' -or !$response.inventory_shutdown_cleared -or $response.pointer_bits -ne 64){throw 'Reagent lifecycle'}
            if($rows.Count -ne $count -or $combat.skills_sent -ne $count -or $combat.attacks_sent -ne 0 -or $world.own_attack_packets -ne 0 -or $world.own_cast_packets -ne $casts -or $world.hp[0] -ne 50 -or $reference.final_player_hp -ne 50 -or $world.mp[0] -ne $finalMp -or $reference.final_player_mp -ne $finalMp -or $world.local_corrections -ne 0 -or $combat.route.Count -ne 0){throw 'Reagent combat authority'}
            for($i=0;$i -lt $count;$i++){
                $row=$rows[$i];$accepted=$success -and $i -lt 2
                if($row.accepted -ne $accepted -or !$row.effect_finished){throw 'Reagent acceptance mismatch'}
                if($accepted){
                    if($row.feedback -ne 'CastAction' -or $row.before.mp[0]-$row.after.mp[0] -ne 20){throw 'Reagent accepted MP/feedback'}
                    $before=@($row.before.inventory.items.PSObject.Properties|ForEach-Object Value)
                    $after=@($row.after.inventory.items.PSObject.Properties|ForEach-Object Value)
                    # Match unchanged authoritative object IDs; catalog identity exists only in server fixture.
                    $changed=@($before|Where-Object { $item=$_;$next=$after|Where-Object id -eq $item.id; !$next -or $next.count -ne $item.count })
                    if($changed.Count -ne 1 -or $changed[0].count -ne 2-$i){throw 'Reagent consumed wrong number of objects'}
                    $remaining=@($after|Where-Object id -eq $changed[0].id)
                    if(($i -eq 0 -and ($remaining.Count -ne 1 -or $remaining[0].count -ne 1)) -or ($i -eq 1 -and $remaining.Count -ne 0)){throw 'Reagent stack decrement/removal'}
                    $unchanged=@($before|Where-Object id -ne $changed[0].id)
                    if($unchanged.Count -ne 1 -or ($unchanged[0]|ConvertTo-Json -Depth 8 -Compress) -cne (($after|Where-Object id -eq $unchanged[0].id)|ConvertTo-Json -Depth 8 -Compress)){throw 'Unrelated potion changed'}
                }else{
                    if($row.feedback -ne 'NoResponse' -or ($row.before|ConvertTo-Json -Depth 10 -Compress) -cne ($row.after|ConvertTo-Json -Depth 10 -Compress)){throw 'Reagent silent refusal changed authority'}
                }
                if($row.before.inventory.owner.connection -ne 2 -or $row.before.inventory.owner.selection -ne 1){throw 'Reagent inventory owner'}
                if($i -gt 0 -and ($rows[$i-1].after|ConvertTo-Json -Depth 10 -Compress) -cne ($row.before|ConvertTo-Json -Depth 10 -Compress)){throw 'Reagent retry state discontinuity'}
            }
            if($combat.skill_selection.Count -ne $count){throw 'Reagent selection count'}
            foreach($selected in $combat.skill_selection){if($selected.skill -ne 63 -or $selected.connection -ne 2 -or $null -ne $selected.target -or $selected.ground[0] -ne $reference.world_x -or $selected.ground[1] -ne $reference.world_y){throw 'Reagent own-cell choice'}}
            $fx=$combat.frames[-1].reagent_effects
            if($fx.started -ne $casts -or $fx.active -ne 0 -or $fx.omitted -ne 0 -or $fx.presentation -ne 'modern_explicit_action3_once'){throw 'Reagent presentation lifecycle'}
            $audio=@($combat.frames|ForEach-Object {$_.audio}|Where-Object {$_.event.sound -eq 841})
            if($audio.Count -ne $casts){throw 'Reagent sound count'}
            foreach($a in $audio){if(!$a.prepared -or $a.same_id_suppressed){throw 'Reagent sound preparation'}}
            foreach($f in $combat.frames){
                if($f.png -and !(Test-Path -LiteralPath $f.png)){throw 'Reagent PNG missing'}
                if($f.reagent_effects.omitted -ne 0 -or $f.reagent_effects.load_error -or $f.skill_effects.omitted -ne 0 -or $f.item_effects.omitted -ne 0){throw 'Reagent resource omission'}
                foreach($e in $f.reagent_effects.effects){if($e.action -ne 3 -or $e.graphics -ne 2231 -or $null -ne $e.target -or $e.position[0] -ne $reference.world_x -or $e.position[1] -ne $reference.world_y){throw 'Reagent effect identity/location'}}
            }
            $view=$combat.result.inventory
            if(@($view.items.PSObject.Properties).Count -ne $reference.inventory.Count){throw 'Reagent final inventory size'}
            foreach($item in $reference.inventory){
                $actual=$view.items.PSObject.Properties[[string]$item.id].Value
                foreach($field in @('id','use_type','charge','icon','blessing','count','identified','name')){if($actual.$field -cne $item.$field){throw "Reagent final authority differs: $field"}}
                if(($actual.detail -join ',') -ne ($item.detail -join ',')){throw 'Reagent detail mismatch'}
            }
            if($reference.status -ne 'verified' -or $reference.jar_sha256 -ne $sha -or !$reference.actual_client_sessions -or $reference.sql_access -or $reference.state_persistent -or (Test-Path -LiteralPath (Join-Path $modeRoot 'state.properties'))){throw 'Isolation/session evidence failed'}
            $presentationPins=Save-OwnedProtocolPresentationFiles -Response $response -Output $output -Mode $mode
            $receipts.Add([ordered]@{mode=$mode;client=$response;server=$reference;verified=$true;presentation_file_pins=$presentationPins})
            Write-Output "$mode native reagent skill/authority verified"
        }finally{
            foreach($owned in @($client,$server)){if($null -ne $owned){if(!$owned.HasExited){Stop-OwnedLitoValidationChild -Process $owned -ScopeRoot $project};$owned.Dispose()}}
        }
    }
    $receiptPath=Join-Path $output 'receipt.json'
    Assert-OwnedCurrentProtocolContext $context
    if((Get-FileHash -LiteralPath $snapshot).Hash -ne $sha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $sha){throw 'Current reference identity drift'}
    [ordered]@{schema=2;profile=$BuildProfile;output=$output;runtime=$runtime;immutable_receipt=$receiptPath;source_pins=$context.SourcePins;compilation_inputs_checked=$context.Inputs.Count;compilation_source_manifest=$context.SourceManifest;compilation_source_manifest_sha256=$context.SourceManifestSHA;fixture_parity=$parity;status='verified_native_reagent_skill_subset';jar_sha256=$sha;exe_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash;cases=$receipts;sql_access=$false;game_endpoint_connected=$true;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 20|Set-Content -LiteralPath $receiptPath -Encoding utf8
    Write-Output ('PROTOCOL_RECEIPT='+$receiptPath)
}finally{Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp}
