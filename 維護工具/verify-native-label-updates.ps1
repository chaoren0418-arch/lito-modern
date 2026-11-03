#Requires -Version 7.0
[CmdletBinding()]param([switch]$SkipBuild,[ValidateSet('Debug','Release')][string]$BuildProfile='Debug')
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
. (Join-Path $PSScriptRoot 'owned-fixed-probe-runtime.ps1')
. (Join-Path $PSScriptRoot 'owned-protocol-fixture.ps1')
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-'+$BuildProfile
$outputBase=Join-Path $taskRoot '驗證紀錄/current/native-label-updates'
$output=Join-Path $outputBase $stamp
$runtime=Join-Path $project 'target/current/native-label-updates'
$classes=Join-Path $project ('target/current/native-label-updates/'+$stamp+'/oracle')
$snapshot=Join-Path $project 'target\current\server-reference\formal-server.jar'
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
New-Item -ItemType Directory -Path $output,$runtime,$classes -Force|Out-Null
$pin=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄\current\reference-snapshot.json') -Raw|ConvertFrom-Json
$sha=(Get-FileHash -LiteralPath $snapshot -Algorithm SHA256).Hash
if($sha -ne $pin.sha256){throw 'Pinned JAR mismatch'}
$sourcePins=@{}
foreach($name in @('維護工具/owned-fixed-probe-runtime.ps1','維護工具/verify-native-label-updates.ps1','維護工具/owned-validation-process.ps1','維護工具/owned-protocol-fixture.ps1','維護工具/compare-owned-protocol-fixture.py','驗證紀錄/current/reference-snapshot.json','驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/source-inputs.json','LiTo381-Modern/tests/oracle/NativeGameHarness.java')){
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
    & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $snapshot -d $classes 'tests/oracle/NativeGameHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Label updates harness compile failed'}
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile|Out-Null;$mainBefore=(Get-FileHash -LiteralPath $exe).Hash}
    $modes=@('room-world-labels')
    foreach($mode in $modes){
        $modeRoot=Join-Path $runtime $mode;New-Item -ItemType Directory -Path $modeRoot -Force|Out-Null
        $previousRuntimeFiles=Save-OwnedFixedProbeFiles -Runtime $modeRoot -Destination (Join-Path $output 'previous-runtime-files')
        $readyPath=Join-Path $modeRoot 'ready.json';$resultPath=Join-Path $modeRoot 'result.json'
        # Fixed files belong only to this isolated test; never traverse/delete a tree.
        foreach($file in @($readyPath,$resultPath,(Join-Path $modeRoot 'label-ready'),(Join-Path $modeRoot 'label-step'),(Join-Path $modeRoot 'label-fixture.json')) + @(0..10|ForEach-Object {Join-Path $modeRoot "label-ack-$_"})){if(Test-Path -LiteralPath $file){Remove-Item -LiteralPath $file -Force}}
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
            if(!$response.worker_quiescent -or $response.shutdown -ne 'closed' -or $combat.attacks_sent -ne 0 -or $combat.skills_sent -ne 0 -or $world.attack_packets -ne 0 -or $world.local_corrections -ne 0){throw 'State fixture changed combat authority'}
            if(@($combat.frames|Select-Object -ExpandProperty label_step -Unique).Count -ne 11 -or $combat.frames.Count -lt 20){throw 'State capture incomplete'}
            if($world.hp[0] -ne $reference.final_player_hp -or $world.mp[0] -ne $reference.final_player_mp){throw 'State fixture HP/MP mismatch'}
            $stateFixture=Get-Content -LiteralPath (Join-Path $modeRoot 'label-fixture.json') -Raw|ConvertFrom-Json
            if($stateFixture.steps -ne 11 -or $stateFixture.packets -ne 10 -or $stateFixture.natural_clan_gameplay){throw 'State fixture scope mismatch'}
            if($response.game.selected_source -ne '測試乙' -or $response.transport.epoch -ne '2'){throw 'World owner changed'}
            if($reference.status -ne 'verified' -or $reference.jar_sha256 -ne $sha -or !$reference.actual_client_sessions -or $reference.sql_access -or $reference.state_persistent -or (Test-Path -LiteralPath (Join-Path $modeRoot 'state.properties'))){throw 'Isolation/session evidence failed'}
            $runtimeFilePins=Save-OwnedFixedProbeFiles -Runtime $modeRoot -Destination (Join-Path $output 'runtime-files')
            $receipts.Add([ordered]@{mode=$mode;client=$response;server=$reference;verified=$true;fixed_probe_coordination_root=$modeRoot;previous_runtime_files=$previousRuntimeFiles;runtime_file_pins=$runtimeFilePins})
            Write-Output "$mode native label updates and authority verified"
        }finally{
            foreach($owned in @($client,$server)){if($null -ne $owned){if(!$owned.HasExited){Stop-OwnedLitoValidationChild -Process $owned -ScopeRoot $project};$owned.Dispose()}}
        }
    }
    Assert-CurrentProtocolInputs
    if((Get-FileHash -LiteralPath $exe).Hash -ne $mainBefore -or (Get-FileHash -LiteralPath $snapshot).Hash -ne $sha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $sha){throw 'Protocol main/current-reference drift'}
    [ordered]@{schema=2;profile=$BuildProfile;output=$output;runtime=$runtime;immutable_receipt=(Join-Path $output 'receipt.json');source_pins=$sourcePins;compilation_inputs_checked=$inputPins.Count;fixture_parity=$parity;status='verified_native_label_updates_subset';jar_sha256=$sha;exe_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash;cases=$receipts;sql_access=$false;game_endpoint_connected=$true;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 20|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8
    $canonical=Join-Path $outputBase 'receipt.json';if(Test-Path -LiteralPath $canonical){Copy-Item -LiteralPath $canonical -Destination (Join-Path $output 'previous-canonical-receipt.json')};Copy-Item -LiteralPath (Join-Path $output 'receipt.json') -Destination $canonical
}finally{Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp}
