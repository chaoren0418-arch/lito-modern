#Requires -Version 7.0
[CmdletBinding()]param([switch]$SkipBuild,[ValidateSet('Debug','Release')][string]$BuildProfile='Debug')
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
$outputBase=Join-Path $taskRoot '驗證紀錄\current\native-handshake'
$output=Join-Path $outputBase ([DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-'+$BuildProfile)
$snapshot=Join-Path $project 'target\current\server-reference\formal-server.jar'
$pinPath=Join-Path $taskRoot '驗證紀錄\current\reference-snapshot.json'
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$classes=Join-Path $output 'oracle'
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
New-Item -ItemType Directory -Path $output,$classes -Force|Out-Null
if(!(Test-Path -LiteralPath $snapshot)){throw 'Missing pinned test JAR; restore the recorded read-only reference snapshot first'}
$pin=Get-Content -LiteralPath $pinPath -Raw|ConvertFrom-Json
$jarSha=(Get-FileHash -LiteralPath $snapshot -Algorithm SHA256).Hash
if($jarSha -ne $pin.sha256){throw 'Reference snapshot SHA256 mismatch'}
$sourcePins=@{}
foreach($name in @('維護工具/verify-native-handshake.ps1','LiTo381-Modern/tests/oracle/FormalCipherOracle.java','LiTo381-Modern/tests/oracle/NativeHandshakeHarness.java','LiTo381-Modern/tests/fixtures/native-version-current.json','驗證紀錄/current/reference-snapshot.json')){
 $path=Join-Path $taskRoot $name;$sourcePins[$name]=(Get-FileHash -LiteralPath $path).Hash
 $destination=Join-Path $output ('source/'+$name);New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force|Out-Null;Copy-Item -LiteralPath $path -Destination $destination
}
$mainBefore=(Get-FileHash -LiteralPath $exe).Hash
function Stop-Owned($process){
 if($process.HasExited){return}
 $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
 if(!$live -and $process.HasExited){return}
 $command=if($live){$live.CommandLine.Replace('/','\')}else{''}
 if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or
    (!$command.Contains($output.Replace('/','\')) -and !$command.Contains($exe.Replace('/','\')))){throw 'Owned handshake PID/creation/command mismatch; stop refused'}
 $process.Kill();$process.WaitForExit()
}
function Quoted([string]$value){if($value.Contains('"')){throw 'Unexpected quote in process argument'};'"'+$value+'"'}
function Wait-Owned($process,[int]$timeout){
    if(!$process.WaitForExit($timeout)){
        Stop-Owned $process;throw "Owned test process timed out: $($process.Id)"
    }
}
$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$scratch=Join-Path $project 'target\current\scratch'
New-Item -ItemType Directory -Path $scratch -Force|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
$results=[System.Collections.Generic.List[object]]::new()
Push-Location $project
try {
    & (Join-Path $jdk 'javac.exe') -d $classes 'tests/oracle/FormalCipherOracle.java' 'tests/oracle/NativeHandshakeHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Isolated oracle compilation failed'}
    & (Join-Path $jdk 'java.exe') -cp $classes FormalCipherOracle $snapshot --version-contract > (Join-Path $output 'version-contract.json')
    if($LASTEXITCODE -ne 0){throw 'Deployed version oracle failed'}
    $contract=Get-Content -LiteralPath (Join-Path $output 'version-contract.json') -Raw|ConvertFrom-Json
    if($contract.jar_sha256 -ne $jarSha -or !$contract.supported){throw 'Version contract source mismatch'}
    $fixture=Get-Content -LiteralPath 'tests/fixtures/native-version-current.json' -Raw|ConvertFrom-Json
    if(!$fixture.supported -or $fixture.request -ne $contract.request -or $fixture.accepted -ne $contract.accepted -or $fixture.rejected -ne $contract.rejected){throw 'Rust fixture bytes differ from pinned current Protocol'}
    $fixtureSourceVerified=$fixture.jar_sha256 -eq $jarSha
    if(!$fixtureSourceVerified){
        $historical=Join-Path $project 'target/current/server-reference/candidate-BBC59308.jar'
        if((Get-FileHash -LiteralPath $historical).Hash -ne $fixture.jar_sha256){throw 'Historical fixture JAR is not retained at the expected SHA'}
        & (Join-Path $jdk 'java.exe') -cp $classes FormalCipherOracle $historical --version-contract > (Join-Path $output 'fixture-source-version-contract.json')
        if($LASTEXITCODE -ne 0){throw 'Historical fixture source oracle failed'}
        $original=Get-Content -LiteralPath (Join-Path $output 'fixture-source-version-contract.json') -Raw|ConvertFrom-Json
        if($original.jar_sha256 -ne $fixture.jar_sha256 -or !$original.supported -or $original.request -ne $fixture.request -or $original.accepted -ne $fixture.accepted -or $original.rejected -ne $fixture.rejected){throw 'Historical unit fixture provenance mismatch'}
        $fixtureSourceVerified=$true
    }
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile|Out-Null;$mainBefore=(Get-FileHash -LiteralPath $exe).Hash}
    if(!(Test-Path -LiteralPath $exe)){throw 'Missing current x64 executable'}
    $expectedErrors=@{reject='native version rejected';malformed='native version tail length';'header-eof'='EOF (PartialHeader)';'body-eof'='EOF (PartialBody)';stall='native handshake deadline exceeded'}
    foreach($mode in @('accept','reject','malformed','header-eof','body-eof','stall')){
        $readyPath=Join-Path $output ('ready-'+$mode+'-'+[guid]::NewGuid().ToString('N')+'.json')
        $serverOut=Join-Path $output ($mode+'-server.json');$serverErr=Join-Path $output ($mode+'-server.stderr.txt')
        $clientOut=Join-Path $output ($mode+'-client.json');$clientErr=Join-Path $output ($mode+'-client.stderr.txt')
        $server=$null;$client=$null
        try {
            $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-cp',(Quoted $classes),'NativeHandshakeHarness',(Quoted $snapshot),(Quoted $readyPath),$mode) -WorkingDirectory $project -WindowStyle Hidden -PassThru -RedirectStandardOutput $serverOut -RedirectStandardError $serverErr
            $deadline=[DateTime]::UtcNow.AddSeconds(10)
            while(!(Test-Path -LiteralPath $readyPath)){
                if($server.HasExited -or [DateTime]::UtcNow -ge $deadline){throw "Harness failed readiness: $mode"}
                Start-Sleep -Milliseconds 25
            }
            # CREATE_NEW publishes a small file; wait for complete JSON before using its port.
            $ready=$null
            while(!$ready){
                try{$ready=Get-Content -LiteralPath $readyPath -Raw|ConvertFrom-Json}catch{if([DateTime]::UtcNow -ge $deadline){throw};Start-Sleep -Milliseconds 10}
            }
            if($ready.jar_sha256 -ne $jarSha -or $ready.mode -ne $mode -or $ready.port -le 0){throw 'Unexpected isolated harness identity'}
            $client=Start-Process -FilePath $exe -ArgumentList @('--probe-native-loopback',('127.0.0.1:'+$ready.port)) -WorkingDirectory $project -WindowStyle Hidden -PassThru -RedirectStandardOutput $clientOut -RedirectStandardError $clientErr
            Wait-Owned $client 12000
            Wait-Owned $server 12000
            if($server.ExitCode -ne 0){throw "Harness failed: $mode; see $serverErr"}
            $actual=Get-Content -LiteralPath $clientOut -Raw|ConvertFrom-Json
            $reference=Get-Content -LiteralPath $serverOut -Raw|ConvertFrom-Json
            if(!$reference.version_valid -or $reference.request_bytes -ne 10 -or $reference.sql_access -or $reference.jar_sha256 -ne $jarSha){throw 'Deployed Protocol did not validate the actual client request'}
            if($mode -eq 'accept'){
                if($client.ExitCode -ne 0 -or $actual.status -ne 'native_version_verified' -or $actual.pointer_bits -ne 64 -or !$actual.worker_quiescent -or $actual.shutdown.phase -ne 'closed'){throw 'Native x64 version handshake or owner shutdown failed'}
                if($actual.account_authenticated -or $actual.sql_access -or $actual.native_bin_started -or $actual.playable_game){throw 'Probe scope changed'}
                if($actual.connection.negotiated.rx_counter -ne $reference.tx_counter -or $actual.connection.negotiated.tx_counter -ne $reference.rx_counter){throw 'Actual socket RX/TX counter mismatch'}
            }else{
                if($client.ExitCode -ne 2 -or !$actual.error.Contains($expectedErrors[$mode])){throw "Missing expected failure: $mode"}
            }
            $results.Add([ordered]@{mode=$mode;verified=$true;endpoint=('127.0.0.1:'+$ready.port);client_exit=$client.ExitCode;server_exit=$server.ExitCode;version_valid=$reference.version_valid;client=$actual;server=$reference})
            Write-Output "$mode verified"
        }finally{
            foreach($owned in @($client,$server)){if($null -ne $owned){if(!$owned.HasExited){Stop-Owned $owned};$owned.Dispose()}}
            # Delete only this script's generated readiness file, preserving raw mode logs.
            if(Test-Path -LiteralPath $readyPath){Remove-Item -LiteralPath $readyPath -Force}
        }
    }
    foreach($name in $sourcePins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $sourcePins[$name]){throw 'Handshake source drift'}}
    if((Get-FileHash -LiteralPath $exe).Hash -ne $mainBefore -or (Get-FileHash -LiteralPath $snapshot).Hash -ne $jarSha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $jarSha){throw 'Handshake main/current-reference drift'}
    $receipt=[ordered]@{schema=2;status='verified_native_version_subset';profile=$BuildProfile;output=$output;immutable_receipt=(Join-Path $output 'receipt.json');jar_sha256=$jarSha;exe_sha256=$mainBefore;source_pins=$sourcePins;fixture_source_jar_sha256=$fixture.jar_sha256;fixture_source_verified=$fixtureSourceVerified;fixture_bytes_equal_current_protocol=$true;cases=$results;formal_server_started=$false;isolated_protocol_harness=$true;sql_access=$false;account_authenticated=$false;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}
    $receipt|ConvertTo-Json -Depth 20|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8
    $canonical=Join-Path $outputBase 'receipt.json';if(Test-Path -LiteralPath $canonical){Copy-Item -LiteralPath $canonical -Destination (Join-Path $output 'previous-canonical-receipt.json')};Copy-Item -LiteralPath (Join-Path $output 'receipt.json') -Destination $canonical
}finally{Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp}
