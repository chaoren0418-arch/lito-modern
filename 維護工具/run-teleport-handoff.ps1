#Requires -Version 7.0
[CmdletBinding()]param([Parameter(Mandatory)][string]$TestBinary,[Parameter(Mandatory)][string]$ReferenceJar,[ValidateSet("cross-endpoint","same-endpoint")][string]$Scenario="cross-endpoint",[string]$HarnessSource)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot='D:\lineage開發\3-lito開發\lito重建版';$project=Join-Path $taskRoot 'LiTo381-Modern'
if(!$HarnessSource){$HarnessSource=Join-Path $project 'tests/oracle/NativeTeleportHarness.java'}
$HarnessSource=(Resolve-Path -LiteralPath $HarnessSource).Path
if(!$HarnessSource.StartsWith($taskRoot+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($HarnessSource) -ne 'NativeTeleportHarness.java'){throw 'Harness source must be an owned NativeTeleportHarness.java'}
$harnessRelative=[IO.Path]::GetRelativePath($taskRoot,$HarnessSource).Replace('\','/')
$allowed=[IO.Path]::GetFullPath((Join-Path $project 'target/current'))+[IO.Path]::DirectorySeparatorChar
foreach($path in @($TestBinary,$ReferenceJar)){if(![IO.Path]::GetFullPath($path).StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw 'Only owned test artifacts are accepted'}}
$jarSha=(Get-FileHash -LiteralPath $ReferenceJar).Hash
if($jarSha -notin @('BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43','9719F45DB324206617726CE06D1A9E2184E9A3B3AEFAB3D50F3EBBDFBF09CFAC','6928A147DF1E784304B9080632C56121B173BF60B172023EBD263E82FD448628','B33F9FA1D3173337A7BA8AC18872ADF4C037033D68F3266442C05C86A74896CC','C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539')){throw 'Review candidate drift'}
$existing=Get-CimInstance Win32_Process|Where-Object {$_.CommandLine -and $_.CommandLine.Contains('target/current/native-teleport-handoff') -and $_.Name -eq 'java.exe'}
if($existing){throw 'An owned teleport run is already active; inspect its receipt and lease before starting another'}
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $project ('target/current/native-teleport-handoff/'+$stamp)
$output=Join-Path $taskRoot ('驗證紀錄/current/native-teleport-handoff/'+$stamp)
$classes=Join-Path $root 'oracle';New-Item -ItemType Directory -Path $classes,$output -Force|Out-Null
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
function Quote([string]$s){if($s.Contains('"')){throw 'Invalid process argument'};'"'+$s+'"'}
function Stop-OwnedTeleport([Diagnostics.Process]$process){
 if($process.HasExited){return}
 $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
 if(!$live -and $process.HasExited){return}
 $command=if($live){$live.CommandLine.Replace('/','\')}else{''}
 if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$command.Contains($root.Replace('/','\'))){throw 'Owned teleport PID/creation/command mismatch; stop refused'}
 $process.Kill();$process.WaitForExit(5000)|Out-Null
}
$sourceFormal='D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar'
if($jarSha -in @('6928A147DF1E784304B9080632C56121B173BF60B172023EBD263E82FD448628','C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539') -and (Get-FileHash -LiteralPath $sourceFormal).Hash -ne $jarSha){throw 'Current formal source drift before teleport test'}
$server=$null;$client=$null
$receipt=@{status='running';runtime=$root;output=$output;test_binary=$TestBinary;test_binary_sha256=(Get-FileHash -LiteralPath $TestBinary).Hash;jar_sha256=$jarSha;test_scope='one opt-in integration test; two actual players, B/C endpoints, new epochs and character/loadout continuation into a shared battle';formal_deployed=$false;reference_kind=$(if($jarSha -eq 'B33F9FA1D3173337A7BA8AC18872ADF4C037033D68F3266442C05C86A74896CC'){'detached_single_method_candidate'}else{'pinned_formal'});scenario=$Scenario}
function Save {$receipt|ConvertTo-Json -Depth 14|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
Save
try {
 $pins=@{}
 foreach($name in @('LiTo381-Modern/tests/oracle/NativePeerHarness.java','維護工具/run-teleport-handoff.ps1','維護工具/verify-teleport-handoff.py','LiTo381-Modern/tests/native_teleport_handoff.rs','LiTo381-Modern/src/modern_lito/native_services.rs','LiTo381-Modern/src/modern_lito/native_teleport.rs','LiTo381-Modern/src/modern_lito/native_login.rs','LiTo381-Modern/src/modern_lito/native_connection.rs','LiTo381-Modern/tests/oracle/NativeTeleportHarness.java')){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash}

 $pins[$harnessRelative]=(Get-FileHash -LiteralPath $HarnessSource).Hash
 $receipt.harness_source=$HarnessSource;$receipt.harness_source_sha256=$pins[$harnessRelative]
 $receipt.source_pins=$pins;Save
 $sourceArchive=Join-Path $output 'source-snapshot';New-Item -ItemType Directory -Path $sourceArchive -Force|Out-Null
 foreach($name in $pins.Keys){$destination=Join-Path $sourceArchive $name;New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force|Out-Null;Copy-Item -LiteralPath (Join-Path $taskRoot $name) -Destination $destination}
 $originalTestBinary=$TestBinary;$TestBinary=Join-Path $root 'immutable-teleport-test.exe';Copy-Item -LiteralPath $originalTestBinary -Destination $TestBinary
 if((Get-FileHash -LiteralPath $TestBinary).Hash -ne $receipt.test_binary_sha256){throw 'Immutable binary differs'}
 $receipt.immutable_test_binary=$TestBinary;Save
 & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $ReferenceJar -d $classes (Join-Path $taskRoot 'LiTo381-Modern/tests/oracle/NativePeerHarness.java') $HarnessSource *> (Join-Path $output 'javac.log')
 if($LASTEXITCODE -ne 0){throw 'Harness compile failed'}
 $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-Xverify:all','-cp',(Quote ($classes+';'+$ReferenceJar)),'local.lito381.server.NativeTeleportHarness',(Quote $root),$Scenario) -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output 'server.log') -RedirectStandardError (Join-Path $output 'server.stderr.txt')
 $end=[DateTime]::UtcNow.AddSeconds(15)
 while(!(Test-Path -LiteralPath (Join-Path $root 'ready.json'))){if($server.HasExited -or [DateTime]::UtcNow -ge $end){throw 'Server not ready'};Start-Sleep -Milliseconds 50}
 $info=[Diagnostics.ProcessStartInfo]::new($TestBinary);$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
 $info.Environment['LITO_TELEPORT_SCENARIO']=$Scenario;$info.Environment['LITO_TELEPORT_ROOT']=$root;$info.Environment['LITO_TELEPORT_REFERENCE_SHA']=$jarSha.ToLowerInvariant();$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $info.ArgumentList.Add('actual_two_players_teleport_and_continue_on_new_game_endpoints')
 foreach($arg in @('--exact','--ignored','--nocapture')){$info.ArgumentList.Add($arg)}
 $client=[Diagnostics.Process]::Start($info);$stdout=$client.StandardOutput.ReadToEndAsync();$stderr=$client.StandardError.ReadToEndAsync()
 $receipt.owned_processes=@(@{pid=$server.Id;start_utc=$server.StartTime.ToUniversalTime().ToString('o');kind='formal_loopback_server'},@{pid=$client.Id;start_utc=$client.StartTime.ToUniversalTime().ToString('o');kind='immutable_selected_rust_test'});$receipt.lease_deadline_utc=[DateTime]::UtcNow.AddSeconds(220).ToString('o');Save
 $end=[DateTime]::UtcNow.AddSeconds(220)
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
 if(!$receipt.server.memberships_empty -or $receipt.server.actual_client_sessions -ne 6 -or $receipt.server.authority_mutations -ne 0){throw 'Isolation/cleanup mismatch'}
 $receipt.status='verified_two_player_teleport_handoff'
 foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw 'Source drift during run'}}
 if((Get-FileHash -LiteralPath $TestBinary).Hash -ne $receipt.test_binary_sha256 -or (Get-FileHash -LiteralPath $ReferenceJar).Hash -ne $jarSha){throw 'Binary drift during run'}
 if($jarSha -in @('6928A147DF1E784304B9080632C56121B173BF60B172023EBD263E82FD448628','C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539') -and (Get-FileHash -LiteralPath $sourceFormal).Hash -ne $jarSha){throw 'Current formal source drift during teleport test'}
 $receipt.inputs_unchanged=$true
}catch{$receipt.status='failed';$receipt.error=$_.Exception.Message;throw}
finally {
 foreach($proc in @($client,$server)){if($proc -and !$proc.HasExited){Stop-OwnedTeleport $proc}}
 foreach($name in @('ready.json','result.json','server-trace.jsonl','a-handoff.json','b-handoff.json')){if(Test-Path -LiteralPath (Join-Path $root $name)){Copy-Item -LiteralPath (Join-Path $root $name) -Destination (Join-Path $output $name)}}
 Save
}
Write-Output $output
