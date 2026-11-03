#Requires -Version 7.0
[CmdletBinding()]param([switch]$SkipBuild,[ValidateSet('room','world')][string]$Mode='world',[ValidateSet('Debug','Release')][string]$BuildProfile='Debug')
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot;$project=Join-Path $taskRoot 'LiTo381-Modern'
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-'+$BuildProfile+'-'+$Mode
$root=Join-Path $project ('target/current/native-recovery/'+$stamp)
$output=Join-Path $taskRoot ('驗證紀錄/current/native-recovery/'+$stamp)
$classes=Join-Path $root 'oracle'
New-Item -ItemType Directory -Path $root,$output,$classes -Force|Out-Null
$pin=Get-Content -Raw (Join-Path $taskRoot '驗證紀錄/current/reference-snapshot.json')|ConvertFrom-Json
$snapshot=$pin.snapshot;$sha=$pin.sha256
if((Get-FileHash -LiteralPath $snapshot).Hash -ne $sha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $sha){throw 'Reference drift'}
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
$sourcePins=@{}
foreach($name in @('維護工具/verify-native-recovery.ps1','維護工具/owned-validation-process.ps1','LiTo381-Modern/tests/oracle/NativeRecoveryHarness.java','驗證紀錄/current/reference-snapshot.json','驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/source-inputs.json')){
 $path=Join-Path $taskRoot $name;$sourcePins[$name]=(Get-FileHash -LiteralPath $path).Hash;$destination=Join-Path $output ('source/'+$name);New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force|Out-Null;Copy-Item -LiteralPath $path -Destination $destination
}
$inputPins=Get-Content -LiteralPath (Join-Path $taskRoot '驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/source-inputs.json') -Raw|ConvertFrom-Json -AsHashtable
function Assert-RecoveryInputs {
 foreach($name in $inputPins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $inputPins[$name]){throw ('Recovery compilation input drift: '+$name)}}
 foreach($name in $sourcePins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $sourcePins[$name]){throw ('Recovery tool/input drift: '+$name)}}
}
Assert-RecoveryInputs
$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$scratch=Join-Path $project 'target/current/scratch';New-Item -ItemType Directory -Path $scratch -Force|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
function Quote([string]$v){if($v.Contains('"')){throw 'Invalid argument'};'"'+$v+'"'}
$server=$null;$client=$null;$saved=$false
Push-Location $project
try{
    & (Join-Path $jdk 'javac.exe') -encoding UTF-8 -cp $snapshot -d $classes 'tests/oracle/NativeRecoveryHarness.java' *> (Join-Path $output 'javac.log')
    if($LASTEXITCODE -ne 0){throw 'Recovery harness compile failed'}
    if(!$SkipBuild){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile *> (Join-Path $output 'build.log');if($LASTEXITCODE -ne 0){throw 'Build failed'}}
    $exeSha=(Get-FileHash -LiteralPath $exe).Hash
    $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-cp',(Quote ($classes+';'+$snapshot)),'local.lito381.server.NativeRecoveryHarness',(Quote $root),$Mode) -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output 'server.log') -RedirectStandardError (Join-Path $output 'server.stderr.txt')
    $ready=$null;$until=[DateTime]::UtcNow.AddSeconds(15)
    while(!$ready){
        if($server.HasExited -or [DateTime]::UtcNow -gt $until){throw 'Recovery server startup'}
        if(Test-Path (Join-Path $root 'ready.json')){try{$ready=Get-Content -Raw (Join-Path $root 'ready.json')|ConvertFrom-Json}catch{}}
        if(!$ready){Start-Sleep -Milliseconds 20}
    }
    if($ready.jar_sha256 -ne $sha){throw 'Loaded reference differs'}
    $info=[System.Diagnostics.ProcessStartInfo]::new($exe);$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
    $info.RedirectStandardInput=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
    $info.StandardInputEncoding=[System.Text.UTF8Encoding]::new($false);$info.StandardOutputEncoding=[System.Text.UTF8Encoding]::new($false)
    $info.Environment['LITO_NATIVE_RECOVERY_ROOT']=$root
    $info.ArgumentList.Add('--probe-native-lobby');$info.ArgumentList.Add('127.0.0.1:'+$ready.port)
    $client=[System.Diagnostics.Process]::Start($info);$stdout=$client.StandardOutput.ReadToEndAsync();$stderr=$client.StandardError.ReadToEndAsync()
    $client.StandardInput.Write((@{account='RecoveryFixture';password='TestOnlyPassword';source_index=1;lobby_mode=('recovery-'+$Mode)}|ConvertTo-Json -Compress));$client.StandardInput.Close()
    $until=[DateTime]::UtcNow.AddSeconds(120)
    while(!$client.HasExited){if([DateTime]::UtcNow -gt $until){throw 'Recovery client timeout'};Start-Sleep -Milliseconds 50}
    $stdout.GetAwaiter().GetResult()|Set-Content (Join-Path $output 'client.json') -Encoding utf8
    $stderr.GetAwaiter().GetResult()|Set-Content (Join-Path $output 'client.stderr.txt') -Encoding utf8;$saved=$true
    if($client.ExitCode -ne 0){throw ('Recovery client failed: '+$output)}
    if(!$server.WaitForExit(8000) -or $server.ExitCode -ne 0){throw ('Recovery server failed: '+$output)}
    $c=Get-Content -Raw (Join-Path $output 'client.json')|ConvertFrom-Json
    $s=Get-Content -Raw (Join-Path $root 'result.json')|ConvertFrom-Json
    if($c.status -ne 'recovery_completed' -or !$c.worker_quiescent -or $c.first_epoch -ne 2 -or $c.last_epoch -ne 4 -or $c.failure.phase -ne 'failed'){throw 'Recovery lifecycle mismatch'}
    if($s.status -ne 'reconnected' -or $s.actual_client_sessions -ne 4 -or $s.socket_faults -ne 1 -or $s.authority_mutations -ne 0 -or $s.sql_access -or !$s.memberships_empty -or $s.jar_sha256 -ne $sha){throw 'Recovery isolation mismatch'}
    for($i=0;$i -lt 2;$i++){
        $state=if($i -eq 0){$c.before}else{$c.after};$obs=$s.observations[$i]
        if($state.joined_room.id -ne $obs.room -or $state.selected_source -ne '測試乙'){throw 'Recovery room/source mismatch'}
        if($Mode -eq 'world'){
            $own=$state.world.actors.($obs.id.ToString())
            if($state.world.status.object_id -ne $obs.id -or $state.world.hp[0] -ne $obs.hp -or $state.world.mp[0] -ne $obs.mp -or $own.x -ne $obs.x -or $own.y -ne $obs.y){throw 'Recovery world authority mismatch'}
        }
    }
    if($Mode -eq 'world' -and (!$c.stale_frame_rejected -or $c.after.world.generation -le $c.before.world.generation)){throw 'Recovery reused world'}
    Assert-RecoveryInputs
    if((Get-FileHash -LiteralPath $snapshot).Hash -ne $sha -or (Get-FileHash -LiteralPath $exe).Hash -ne $exeSha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $sha){throw 'Build/reference changed'}
    [ordered]@{schema=2;profile=$BuildProfile;output=$output;immutable_receipt=(Join-Path $output 'receipt.json');source_pins=$sourcePins;compilation_inputs_checked=$inputPins.Count;status='verified_controller_recovery';mode=$Mode;jar_sha256=$sha;exe_sha256=$exeSha;runtime=$root;server=$s;client=$c;gui_verified=$false}|ConvertTo-Json -Depth 100|Set-Content (Join-Path $output 'receipt.json') -Encoding utf8
    Write-Output ('Verified controller recovery '+$Mode+': '+$output)
}finally{
    if($null -ne $client){
        if(!$client.HasExited){Stop-OwnedLitoValidationChild -Process $client -ScopeRoot $project}
        if(!$saved){$stdout.GetAwaiter().GetResult()|Set-Content (Join-Path $output 'client.json') -Encoding utf8;$stderr.GetAwaiter().GetResult()|Set-Content (Join-Path $output 'client.stderr.txt') -Encoding utf8}
        $client.Dispose()
    }
    if($null -ne $server){if(!$server.HasExited){Stop-OwnedLitoValidationChild -Process $server -ScopeRoot $root};$server.Dispose()}
    foreach($name in @('ready.json','result.json')){if(Test-Path (Join-Path $root $name)){Copy-Item -LiteralPath (Join-Path $root $name) -Destination (Join-Path $output $name)}}
    Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp
}
