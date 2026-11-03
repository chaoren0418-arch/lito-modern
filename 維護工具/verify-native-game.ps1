#Requires -Version 7.0
[CmdletBinding()]param([switch]$SkipBuild,[ValidateSet('Debug','Release')][string]$BuildProfile='Debug')
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
. (Join-Path $PSScriptRoot 'owned-protocol-fixture.ps1')
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-'+$BuildProfile
$outputBase=Join-Path $taskRoot '驗證紀錄/current/native-game'
$output=Join-Path $outputBase $stamp
$runtime=Join-Path $project ('target/current/native-game/'+$stamp)
$classes=Join-Path $runtime 'oracle'
$snapshot=Join-Path $project 'target\current\server-reference\formal-server.jar'
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
New-Item -ItemType Directory -Path $output,$runtime,$classes -Force|Out-Null
$pin=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄\current\reference-snapshot.json') -Raw|ConvertFrom-Json
$sha=(Get-FileHash -LiteralPath $snapshot -Algorithm SHA256).Hash
if($sha -ne $pin.sha256){throw 'Pinned JAR mismatch'}
$sourcePins=@{}
foreach($name in @('維護工具/verify-native-game.ps1','維護工具/owned-validation-process.ps1','維護工具/owned-protocol-fixture.ps1','維護工具/compare-owned-protocol-fixture.py','LiTo381-Modern/tests/oracle/NativeGameOracle.java','LiTo381-Modern/tests/oracle/NativeGameHarness.java','LiTo381-Modern/tests/fixtures/native-game-current.json','驗證紀錄/current/reference-snapshot.json')){
 $path=Join-Path $taskRoot $name;$sourcePins[$name]=(Get-FileHash -LiteralPath $path).Hash;$destination=Join-Path $output ('source/'+$name);New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force|Out-Null;Copy-Item -LiteralPath $path -Destination $destination
}
$mainBefore=(Get-FileHash -LiteralPath $exe).Hash
function Quote([string]$text){if($text.Contains('"')){throw 'Invalid process argument'};'"'+$text+'"'}
$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$scratch=Join-Path $project 'target\current\scratch'
New-Item -ItemType Directory -Path $scratch -Force|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
$receipts=[System.Collections.Generic.List[object]]::new()
Push-Location $project
try {
    & (Join-Path $jdk 'javac.exe') -encoding UTF-8 -cp $snapshot -d $classes 'tests/oracle/NativeGameOracle.java' 'tests/oracle/NativeGameHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Game harness compile failed'}
    & (Join-Path $jdk 'java.exe') -cp ($classes+';'+$snapshot) local.lito381.server.NativeGameOracle > (Join-Path $output 'oracle.json')
    if($LASTEXITCODE -ne 0){throw 'Game oracle failed'}
    $parity=Assert-OwnedLitoProtocolFixture -FixturePath (Join-Path $project 'tests/fixtures/native-game-current.json') -ActualOracle (Join-Path $output 'oracle.json') -Classes $classes -OracleClass 'local.lito381.server.NativeGameOracle' -Jdk $jdk -ReferenceJar $snapshot -Output $output
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile|Out-Null;$mainBefore=(Get-FileHash -LiteralPath $exe).Hash}
    foreach($mode in @('select','notices','empty','unauthorized')){
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
            $info.ArgumentList.Add('--probe-native-game');$info.ArgumentList.Add('127.0.0.1:'+$ready.port)
            $client=[System.Diagnostics.Process]::Start($info)
            $outTask=$client.StandardOutput.ReadToEndAsync();$errTask=$client.StandardError.ReadToEndAsync()
            # These are local MemoryAccountRepository fixture credentials, not player credentials.
            $fixtureInput=[ordered]@{account='重建帳號';password='測試密碼';retry_password=$null;source_index=1}
            $client.StandardInput.Write(($fixtureInput|ConvertTo-Json -Compress));$client.StandardInput.Close()
            if(!$client.WaitForExit(35000)){throw 'Native GAME client timed out'}
            $clientText=$outTask.GetAwaiter().GetResult();$clientText|Set-Content -LiteralPath (Join-Path $output ($mode+'-client.json')) -Encoding utf8
            $errTask.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output ($mode+'-client.stderr.txt')) -Encoding utf8
            if(!$server.WaitForExit(8000)){throw 'Deployed session did not close'}
            $expectedExit=if($mode -eq 'unauthorized'){2}else{0}
            if($client.ExitCode -ne $expectedExit -or $server.ExitCode -ne 0){throw "Game test failed $mode; inspect mode logs"}
            $response=$clientText|ConvertFrom-Json;$reference=Get-Content -LiteralPath $resultPath -Raw|ConvertFrom-Json
            if($mode -eq 'unauthorized'){
                if($response.error -notlike '*native login response deadline exceeded*' -or $reference.game_phase -ne 'not_connected'){throw 'Unauthorized source advanced or unexpected failure'}
            }else{
                $phase=if($mode -eq 'empty'){'characters_ready'}else{'character_selected'}
                if($response.game.phase -ne $phase -or !$response.worker_quiescent -or $response.shutdown -ne 'closed' -or $response.pointer_bits -ne 64 -or !$response.game.game_connected){throw 'Native GAME final state mismatch'}
                if($response.login_epoch -ne '1' -or $response.transport.epoch -ne '2'){throw 'Handoff did not retire previous owner'}
                if($response.game.selected_source -ne '測試乙' -or $reference.source -ne '測試乙'){throw 'Source authority changed'}
                $acks=@($response.game.acknowledgement_opcodes) -join ','
                $expectedAcks=if($mode -eq 'notices'){'70,97,226'}else{'226'}
                if($acks -ne $expectedAcks){throw "Wrong bootstrap acknowledgements: $acks"}
                if($response.game.characters.Count -ne $reference.count -or ($mode -ne 'empty' -and ($response.game.character_result -ne 0 -or $response.game.selected_character -ne $reference.display_character))){throw 'Character authority mismatch'}
            }
            if($reference.status -ne 'verified' -or $reference.jar_sha256 -ne $sha -or !$reference.actual_client_sessions -or $reference.sql_access -or $reference.state_persistent -or (Test-Path -LiteralPath (Join-Path $modeRoot 'state.properties'))){throw 'Isolation/session evidence failed'}
            $receipts.Add([ordered]@{mode=$mode;client=$response;server=$reference;verified=$true})
            Write-Output "$mode native LOGIN-to-GAME verified"
        }finally{
            foreach($owned in @($client,$server)){if($null -ne $owned){if(!$owned.HasExited){Stop-OwnedLitoValidationChild -Process $owned -ScopeRoot $project};$owned.Dispose()}}
        }
    }
    foreach($name in $sourcePins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $sourcePins[$name]){throw 'GAME source drift'}}
    if((Get-FileHash -LiteralPath $exe).Hash -ne $mainBefore -or (Get-FileHash -LiteralPath $snapshot).Hash -ne $sha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $sha){throw 'GAME main/current-reference drift'}
    [ordered]@{schema=2;status='verified_native_game_handoff_characters';profile=$BuildProfile;output=$output;runtime=$runtime;immutable_receipt=(Join-Path $output 'receipt.json');source_pins=$sourcePins;fixture_parity=$parity;jar_sha256=$sha;exe_sha256=$mainBefore;cases=$receipts;sql_access=$false;game_endpoint_connected=$true;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json -Depth 20|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8
    $canonical=Join-Path $outputBase 'receipt.json';if(Test-Path -LiteralPath $canonical){Copy-Item -LiteralPath $canonical -Destination (Join-Path $output 'previous-canonical-receipt.json')};Copy-Item -LiteralPath (Join-Path $output 'receipt.json') -Destination $canonical
}finally{Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp}
