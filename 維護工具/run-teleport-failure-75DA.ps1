#Requires -Version 7.0
[CmdletBinding()]param([Parameter(Mandatory)][string]$TestBinary,[Parameter(Mandatory)][string]$ReferenceJar,[ValidateSet('timeout','cancel-expiry','cancel-retry','refused','wrong-service')][string]$Scenario='timeout')
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot='D:\lineage開發\3-lito開發\lito重建版';$project=Join-Path $taskRoot 'LiTo381-Modern'
$allowed=[IO.Path]::GetFullPath((Join-Path $project 'target/current'))+[IO.Path]::DirectorySeparatorChar
foreach($path in @($TestBinary,$ReferenceJar)){if(![IO.Path]::GetFullPath($path).StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw 'Only owned test artifacts are accepted'}}
$jarSha=(Get-FileHash -LiteralPath $ReferenceJar).Hash
if($jarSha -ne '75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'){throw 'Reviewed 75DA reference hash mismatch'}
$existing=Get-CimInstance Win32_Process|Where-Object {$_.CommandLine -and $_.CommandLine.Contains('target/current/native-teleport-failure') -and $_.Name -eq 'java.exe'}
if($existing){throw 'An owned teleport run is already active; inspect its receipt and lease before starting another'}
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $project ('target/current/native-teleport-failure/'+$stamp)
$output=Join-Path $taskRoot ('驗證紀錄/current/native-teleport-failure/'+$stamp)
$classes=Join-Path $root 'oracle';New-Item -ItemType Directory -Path $classes,$output -Force|Out-Null
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
function Quote([string]$s){if($s.Contains('"')){throw 'Invalid process argument'};'"'+$s+'"'}
function Stop-OwnedTeleport([Diagnostics.Process]$process){
 if($process.HasExited){return}
 $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
 if(!$live -and $process.HasExited){return}
 $command=if($live){$live.CommandLine.Replace('/','\').Replace('\\','\')}else{''}
 if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$command.Contains($root.Replace('/','\'))){throw 'Owned teleport PID/creation/command mismatch; stop refused'}
 $process.Kill();$process.WaitForExit(5000)|Out-Null
}
$sourceFormal='D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar'
$observedSourceBefore=(Get-FileHash -LiteralPath $sourceFormal).Hash
$server=$null;$client=$null;$leaseSeconds=if($Scenario -eq 'cancel-expiry'){340}else{220}
$receipt=@{status='running';runtime=$root;output=$output;test_binary=$TestBinary;test_binary_sha256=(Get-FileHash -LiteralPath $TestBinary).Hash;jar_sha256=$jarSha;test_scope='one opt-in integration test; two actual players, B/C no-response destination, bounded failure, fresh LOGIN/GAME authentication, staged character continuation and map500 reentry';formal_deployed=$false;scenario=$Scenario}
if($Scenario -eq 'cancel-expiry'){$receipt.test_scope='two players cancel while destination GAME connects; real 33s wait, expired staged loadout, fresh LOGIN/GAME, manual selection and map500 movement'}
if($Scenario -eq 'cancel-retry'){$receipt.test_scope='two players cancel destination GAME, fresh authenticated LOGIN/GAME before staged expiry, one-shot server character/loadout continuation and map500 movement'}
if($Scenario -eq 'refused'){$receipt.test_scope='previously owned closed ephemeral loopback destination, Java ConnectException independently confirms TCP refusal; client uses default bounded connect, authority retirement, fresh LOGIN/GAME, original character/loadout continuation and map500 movement'}
if($Scenario -eq 'wrong-service'){$receipt.test_scope='two players redirected to a real formal LOGIN service; successful native handshake/authentication then bounded GAME rejection, authority retirement and fresh source GAME recovery'}
function Save {$receipt|ConvertTo-Json -Depth 14|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
Save
try {
 $pins=@{}
 foreach($name in @('LiTo381-Modern/Cargo.toml','LiTo381-Modern/.cargo/config.toml','LiTo381-Modern/tests/oracle/NativePeerHarness.java','維護工具/run-teleport-failure.ps1','LiTo381-Modern/tests/native_teleport_failure.rs','LiTo381-Modern/src/modern_lito/native_services.rs','LiTo381-Modern/src/modern_lito/native_teleport.rs','LiTo381-Modern/src/modern_lito/native_login.rs','LiTo381-Modern/src/modern_lito/native_connection.rs','LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java')){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash}

 $receipt.source_pins=$pins;Save
 $sourceArchive=Join-Path $output 'source-snapshot';New-Item -ItemType Directory -Path $sourceArchive -Force|Out-Null
 foreach($name in $pins.Keys){$destination=Join-Path $sourceArchive $name;New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force|Out-Null;Copy-Item -LiteralPath (Join-Path $taskRoot $name) -Destination $destination}
 $originalTestBinary=$TestBinary;$TestBinary=Join-Path $root 'immutable-teleport-test.exe';Copy-Item -LiteralPath $originalTestBinary -Destination $TestBinary
 if((Get-FileHash -LiteralPath $TestBinary).Hash -ne $receipt.test_binary_sha256){throw 'Immutable binary differs'}
 $receipt.immutable_test_binary=$TestBinary;Save
 & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $ReferenceJar -d $classes (Join-Path $taskRoot 'LiTo381-Modern/tests/oracle/NativePeerHarness.java') (Join-Path $project 'tests/oracle/NativeTeleportFailureHarness.java') *> (Join-Path $output 'javac.log')
 if($LASTEXITCODE -ne 0){throw 'Harness compile failed'}
 $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-Xverify:all','-cp',(Quote ($classes+';'+$ReferenceJar)),'local.lito381.server.NativeTeleportFailureHarness',(Quote $root),$Scenario) -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output 'server.log') -RedirectStandardError (Join-Path $output 'server.stderr.txt')
 $end=[DateTime]::UtcNow.AddSeconds(15)
 while(!(Test-Path -LiteralPath (Join-Path $root 'ready.json'))){if($server.HasExited -or [DateTime]::UtcNow -ge $end){throw 'Server not ready'};Start-Sleep -Milliseconds 50}
 $info=[Diagnostics.ProcessStartInfo]::new($TestBinary);$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
 $info.Environment['LITO_TELEPORT_SCENARIO']=$Scenario;$info.Environment['LITO_TELEPORT_ROOT']=$root;$info.Environment['LITO_TELEPORT_REFERENCE_SHA']=$jarSha.ToLowerInvariant();$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $testName=if($Scenario -eq 'wrong-service'){'wrong_service_destination_retires_authority_and_allows_fresh_login'}elseif($Scenario -eq 'refused'){'refused_destination_retires_authority_and_allows_fresh_login'}elseif($Scenario -eq 'cancel-retry'){'cancellation_then_immediate_retry_consumes_staged_character_once'}elseif($Scenario -eq 'cancel-expiry'){'cancellation_retires_destination_and_expired_handoff_requires_fresh_selection'}else{'destination_timeout_retires_old_authority_and_allows_fresh_login'}
 $info.ArgumentList.Add($testName)
 foreach($arg in @('--exact','--ignored','--nocapture')){$info.ArgumentList.Add($arg)}
 $client=[Diagnostics.Process]::Start($info);$stdout=$client.StandardOutput.ReadToEndAsync();$stderr=$client.StandardError.ReadToEndAsync()
 $receipt.owned_processes=@(@{pid=$server.Id;start_utc=$server.StartTime.ToUniversalTime().ToString('o');kind='formal_loopback_server'},@{pid=$client.Id;start_utc=$client.StartTime.ToUniversalTime().ToString('o');kind='immutable_selected_rust_test'});$receipt.lease_deadline_utc=[DateTime]::UtcNow.AddSeconds($leaseSeconds).ToString('o');Save
 $end=[DateTime]::UtcNow.AddSeconds($leaseSeconds)
 while(!$client.HasExited){if([DateTime]::UtcNow -ge $end){throw 'Teleport handoff test timeout'};Start-Sleep -Milliseconds 100}
 $stdout.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output 'test.log') -Encoding utf8
 $stderr.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output 'test.stderr.txt') -Encoding utf8
 $receipt.test_exit_code=$client.ExitCode
 $end=[DateTime]::UtcNow.AddSeconds(8)
 while(!$server.HasExited -and [DateTime]::UtcNow -lt $end){Start-Sleep -Milliseconds 100}
 if(!$server.HasExited){throw 'Server cleanup did not complete'}
 $receipt.server_exit_code=$server.ExitCode
 if($client.ExitCode -ne 0 -or $server.ExitCode -ne 0){throw 'Teleport handoff test or cleanup failed; inspect saved logs'}
 $receipt.server=Get-Content -LiteralPath (Join-Path $root 'result.json') -Raw|ConvertFrom-Json
 if(!$receipt.server.memberships_empty -or $receipt.server.actual_client_sessions -ne $(if($Scenario -eq 'wrong-service'){10}else{8}) -or $receipt.server.authority_mutations -ne 0 -or $receipt.server.staged_loadouts_after_reentry -ne 0){throw 'Isolation/cleanup mismatch'}
 $receipt.status=if($Scenario -eq 'wrong-service'){'verified_wrong_service_destination_and_fresh_login'}elseif($Scenario -eq 'refused'){'verified_destination_refused_and_fresh_login'}elseif($Scenario -eq 'cancel-retry'){'verified_cancel_immediate_retry'}elseif($Scenario -eq 'cancel-expiry'){'verified_cancel_expiry_and_fresh_selection'}else{'verified_destination_timeout_and_fresh_login'}
 foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw 'Source drift during run'}}
 if((Get-FileHash -LiteralPath $TestBinary).Hash -ne $receipt.test_binary_sha256 -or (Get-FileHash -LiteralPath $ReferenceJar).Hash -ne $jarSha){throw 'Binary drift during run'}
 $receipt.observed_source_before_sha256=$observedSourceBefore;$receipt.observed_source_after_sha256=(Get-FileHash -LiteralPath $sourceFormal).Hash
 $receipt.reference_is_current=($receipt.observed_source_before_sha256 -eq $jarSha -and $receipt.observed_source_after_sha256 -eq $jarSha)
 if(!$receipt.reference_is_current){throw 'Reviewed current formal source drift during failure test'}
 $receipt.reference_scope='immutable owned version pin; changing readonly adjacent source is observed separately'
 $receipt.inputs_unchanged=$true
}catch{$receipt.status='failed';$receipt.error=$_.Exception.Message;throw}
finally {
 foreach($proc in @($client,$server)){if($proc -and !$proc.HasExited){Stop-OwnedTeleport $proc}}
 foreach($name in @('ready.json','result.json','server-trace.jsonl','staged-trace.jsonl','a-failure.json','b-failure.json')){if(Test-Path -LiteralPath (Join-Path $root $name)){Copy-Item -LiteralPath (Join-Path $root $name) -Destination (Join-Path $output $name)}}
 Save
}
Write-Output $output
