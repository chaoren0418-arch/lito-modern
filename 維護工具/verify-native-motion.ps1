#Requires -Version 7.0
[CmdletBinding()]param([switch]$SkipBuild,[ValidateSet('Debug','Release')][string]$BuildProfile='Debug')
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
. (Join-Path $PSScriptRoot 'owned-protocol-fixture.ps1')
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-'+$BuildProfile
$outputBase=Join-Path $taskRoot '驗證紀錄/current/native-motion'
$output=Join-Path $outputBase $stamp
$runtime=Join-Path $project ('target/current/native-motion/'+$stamp)
$classes=Join-Path $runtime 'oracle'
$snapshot=Join-Path $project 'target\current\server-reference\formal-server.jar'
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
New-Item -ItemType Directory -Path $output,$runtime,$classes -Force|Out-Null
$pin=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄\current\reference-snapshot.json') -Raw|ConvertFrom-Json
$sha=(Get-FileHash -LiteralPath $snapshot -Algorithm SHA256).Hash
if($sha -ne $pin.sha256){throw 'Pinned JAR mismatch'}
$sourcePins=@{}
foreach($name in @('維護工具/verify-native-motion.ps1','維護工具/owned-validation-process.ps1','維護工具/owned-protocol-fixture.ps1','維護工具/compare-owned-protocol-fixture.py','驗證紀錄/current/reference-snapshot.json','驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/source-inputs.json','LiTo381-Modern/tests/fixtures/native-motion-current.json','LiTo381-Modern/tests/oracle/NativeGameHarness.java','LiTo381-Modern/tests/oracle/NativeMotionOracle.java')){
 $path=Join-Path $taskRoot $name;$sourcePins[$name]=(Get-FileHash -LiteralPath $path).Hash;$destination=Join-Path $output ('source/'+$name);New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force|Out-Null;Copy-Item -LiteralPath $path -Destination $destination
}
$inputPins=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/source-inputs.json') -Raw|ConvertFrom-Json -AsHashtable
function Assert-CurrentProtocolInputs {
 foreach($name in $inputPins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $inputPins[$name]){throw ('Protocol compilation input drift: '+$name)}}
 foreach($name in $sourcePins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $sourcePins[$name]){throw ('Protocol tool/input drift: '+$name)}}
}
Assert-CurrentProtocolInputs
$mainBefore=(Get-FileHash -LiteralPath $exe).Hash
$parity=$null
function Quote([string]$text){if($text.Contains('"')){throw 'Invalid process argument'};'"'+$text+'"'}
$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$scratch=Join-Path $project 'target\current\scratch'
New-Item -ItemType Directory -Path $scratch -Force|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
$receipts=[System.Collections.Generic.List[object]]::new()
Push-Location $project
try {
    & (Join-Path $jdk 'javac.exe') -encoding UTF-8 -cp $snapshot -d $classes 'tests/oracle/NativeMotionOracle.java' 'tests/oracle/NativeGameHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Game harness compile failed'}
    & (Join-Path $jdk 'java.exe') -cp ($classes+';'+$snapshot) local.lito381.server.NativeMotionOracle > (Join-Path $output 'oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Game oracle failed'}
    $parity=Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-motion-current.json') -ActualOracle (Join-Path $output 'oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeMotionOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $output
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile|Out-Null;$mainBefore=(Get-FileHash -LiteralPath $exe).Hash}
    foreach($mode in @('room-world-move','room-world-correct')){
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
            if(!$client.WaitForExit(45000)){throw 'Native GAME client timed out'}
            $clientText=$outTask.GetAwaiter().GetResult();$clientText|Set-Content -LiteralPath (Join-Path $output ($mode+'-client.json')) -Encoding utf8
            $errTask.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($mode+'-client.stderr.txt')) -Encoding utf8
            if(!$server.WaitForExit(8000)){throw 'Deployed session did not close'}
            if($client.ExitCode -ne 0 -or $server.ExitCode -ne 0){throw "Lobby test failed $mode; inspect mode logs"}
            $response=$clientText|ConvertFrom-Json;$reference=Get-Content -LiteralPath $resultPath -Raw|ConvertFrom-Json
            if($response.game.phase -ne 'battle_pending' -or !$response.game.world.started -or !$response.worker_quiescent -or $response.shutdown -ne 'closed'){throw 'World client did not settle'}
            $world=$response.game.world
            if($world.map_id -ne 564 -or $null -eq $world.status -or $null -eq $world.integrity_challenge){throw 'World bootstrap incomplete'}
            $own=$world.actors.($world.status.object_id.ToString())
            if($null -eq $own -or $own.name -ne $response.game.selected_character){throw 'World own actor identity mismatch'}
            if($own.id -ne $reference.world_object -or $own.x -ne $reference.world_x -or $own.y -ne $reference.world_y -or $own.graphics -ne $reference.world_gfx){throw 'World state differs from formal session'}
            $before=$response.motion_before
            $previous=$before.actors.($own.id.ToString())
            if($null -eq $previous -or $world.predicted_moves -ne 1 -or $reference.movement_lock_pending -or $response.movement_self_ack){throw 'Movement evidence invalid'}
            if($mode -eq 'room-world-move'){
                if($own.x -ne $previous.x -or $own.y -ne ($previous.y-1) -or !$own.position_predicted -or $world.local_corrections -ne 0){throw 'Accepted move did not reach predicted destination'}
                if($response.world_render.frame.rgba_sha256 -eq $response.world_render.before.frame.rgba_sha256){throw 'Movement did not redraw scene'}
            }else{
                if($own.x -ne $previous.x -or $own.y -ne $previous.y -or $own.position_predicted -or $world.local_corrections -lt 1 -or $world.movement_locks -lt 1){throw 'Rejected move did not restore authoritative position'}
            }
            if(!$response.world_render.model_reused -or $response.world_render.frame.visual_revision -le $response.world_render.before.frame.visual_revision -or !(Test-Path -LiteralPath $response.world_render.before.png)){throw 'Cached model update evidence missing'}
            $image=$response.world_render.frame
            $renderedOwn=@($image.actors | Where-Object {$_.id -eq $own.id -and $_.rendered})
            if($image.map_id -ne $world.map_id -or $renderedOwn.Count -ne 1 -or ($renderedOwn[0].screen -join ',') -ne '400,300' -or !(Test-Path -LiteralPath $response.world_render.png)){throw 'World image lacks authoritative own actor'}
            if($response.game.selected_source -ne '測試乙' -or $response.transport.epoch -ne '2'){throw 'World owner changed'}
            if($reference.status -ne 'verified' -or $reference.jar_sha256 -ne $sha -or !$reference.actual_client_sessions -or $reference.sql_access -or $reference.state_persistent -or (Test-Path -LiteralPath (Join-Path $modeRoot 'state.properties'))){throw 'Isolation/session evidence failed'}
            $receipts.Add([ordered]@{mode=$mode;client=$response;server=$reference;verified=$true})
            Write-Output "$mode native movement/correction verified"
        }finally{
            foreach($owned in @($client,$server)){if($null -ne $owned){if(!$owned.HasExited){Stop-OwnedLitoValidationChild -Process $owned -ScopeRoot $project};$owned.Dispose()}}
        }
    }
    Assert-CurrentProtocolInputs
    if((Get-FileHash -LiteralPath $exe).Hash -ne $mainBefore -or (Get-FileHash -LiteralPath $snapshot).Hash -ne $sha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $sha){throw 'Protocol main/current-reference drift'}
    [ordered]@{schema=2;profile=$BuildProfile;output=$output;runtime=$runtime;immutable_receipt=(Join-Path $output 'receipt.json');source_pins=$sourcePins;compilation_inputs_checked=$inputPins.Count;fixture_parity=$parity;status='verified_native_world_motion_subset';jar_sha256=$sha;exe_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash;cases=$receipts;sql_access=$false;game_endpoint_connected=$true;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 20|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8
    $canonical=Join-Path $outputBase 'receipt.json';if(Test-Path -LiteralPath $canonical){Copy-Item -LiteralPath $canonical -Destination (Join-Path $output 'previous-canonical-receipt.json')};Copy-Item -LiteralPath (Join-Path $output 'receipt.json') -Destination $canonical
}finally{Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp}

