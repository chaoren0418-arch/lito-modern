#Requires -Version 7.0
[CmdletBinding()]param([Parameter(Mandatory)][string]$TestBinary,[Parameter(Mandatory)][string]$ReferenceJar)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot='D:\lineage開發\3-lito開發\lito重建版';$project=Join-Path $taskRoot 'LiTo381-Modern'
$allowed=[IO.Path]::GetFullPath((Join-Path $project 'target/current'))+[IO.Path]::DirectorySeparatorChar
foreach($path in @($TestBinary,$ReferenceJar)){if(![IO.Path]::GetFullPath($path).StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw 'Only owned test artifacts are accepted'}}
$jarSha=(Get-FileHash -LiteralPath $ReferenceJar).Hash
if($jarSha -notin @('BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43','9719F45DB324206617726CE06D1A9E2184E9A3B3AEFAB3D50F3EBBDFBF09CFAC')){throw 'Review reference drift'}
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $project ('target/current/native-guardian-attack/'+$stamp)
$output=Join-Path $taskRoot ('驗證紀錄/current/native-guardian-attack/'+$stamp)
$classes=Join-Path $root 'oracle';New-Item -ItemType Directory -Path $classes,$output -Force|Out-Null
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
function Quote([string]$s){if($s.Contains('"')){throw 'Invalid process argument'};'"'+$s+'"'}
$server=$null;$client=$null
$receipt=@{status='running';runtime=$root;output=$output;test_binary=$TestBinary;test_binary_sha256=(Get-FileHash -LiteralPath $TestBinary).Hash;jar_sha256=$jarSha;test_scope='one opt-in integration test; two actual honor players after real rental-loadout/NPC endpoint handoff, actual3000HP guardian rendered cracks/destruction/held terminal frame and sound360 once, stale input retirement, automatic tower settlement, post-result input rejection and normal lobby return';formal_deployed=$false}
function Save {$receipt|ConvertTo-Json -Depth 14|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
Save
try {
 $pins=@{}
 foreach($name in @('LiTo381-Modern/tests/oracle/NativePeerHarness.java','維護工具/run-guardian-attack.ps1','LiTo381-Modern/tests/native_guardian_attack.rs','LiTo381-Modern/src/modern_lito/native_obstacles.rs','LiTo381-Modern/src/modern_lito/native_world.rs','LiTo381-Modern/src/modern_lito/native_combat.rs','LiTo381-Modern/src/modern_lito/native_world_animation.rs','LiTo381-Modern/src/modern_lito/native_combat_animation.rs','LiTo381-Modern/src/modern_lito/native_skill_selection.rs','LiTo381-Modern/src/modern_lito/native_skills.rs','LiTo381-Modern/tests/oracle/NativeGuardianHarness.java')){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash}

 $receipt.source_pins=$pins;Save
 $sourceArchive=Join-Path $output 'source-snapshot';New-Item -ItemType Directory -Path $sourceArchive -Force|Out-Null
 foreach($name in $pins.Keys){$destination=Join-Path $sourceArchive $name;New-Item -ItemType Directory -Path (Split-Path -Parent $destination) -Force|Out-Null;Copy-Item -LiteralPath (Join-Path $taskRoot $name) -Destination $destination}
 $originalTestBinary=$TestBinary;$TestBinary=Join-Path $root 'immutable-guardian-test.exe';Copy-Item -LiteralPath $originalTestBinary -Destination $TestBinary
 if((Get-FileHash -LiteralPath $TestBinary).Hash -ne $receipt.test_binary_sha256){throw 'Immutable binary differs'}
 $receipt.immutable_test_binary=$TestBinary;Save
 & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $ReferenceJar -d $classes (Join-Path $taskRoot 'LiTo381-Modern/tests/oracle/NativePeerHarness.java') (Join-Path $taskRoot 'LiTo381-Modern/tests/oracle/NativeGuardianHarness.java') *> (Join-Path $output 'javac.log')
 if($LASTEXITCODE -ne 0){throw 'Harness compile failed'}
 $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-Xverify:all','-cp',(Quote ($classes+';'+$ReferenceJar)),'local.lito381.server.NativeGuardianHarness',(Quote $root)) -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output 'server.log') -RedirectStandardError (Join-Path $output 'server.stderr.txt')
 $end=[DateTime]::UtcNow.AddSeconds(15)
 while(!(Test-Path -LiteralPath (Join-Path $root 'ready.json'))){if($server.HasExited -or [DateTime]::UtcNow -ge $end){throw 'Server not ready'};Start-Sleep -Milliseconds 50}
 $info=[Diagnostics.ProcessStartInfo]::new($TestBinary);$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
 $info.Environment['LITO_GUARDIAN_ROOT']=$root;$info.Environment['LITO_GUARDIAN_REFERENCE_SHA']=$jarSha.ToLowerInvariant();$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $info.ArgumentList.Add('actual_guardian_cracks_destruction_and_settlement_retire_old_inputs')
 foreach($arg in @('--exact','--ignored','--nocapture')){$info.ArgumentList.Add($arg)}
 $client=[Diagnostics.Process]::Start($info);$stdout=$client.StandardOutput.ReadToEndAsync();$stderr=$client.StandardError.ReadToEndAsync()
 $receipt.owned_processes=@(@{pid=$server.Id;start_utc=$server.StartTime.ToUniversalTime().ToString('o');kind='formal_guardian_server'},@{pid=$client.Id;start_utc=$client.StartTime.ToUniversalTime().ToString('o');kind='immutable_guardian_test'});$receipt.lease_deadline_utc=[DateTime]::UtcNow.AddSeconds(1100).ToString('o');Save
 $end=[DateTime]::UtcNow.AddSeconds(1100)
 while(!$client.HasExited){if([DateTime]::UtcNow -ge $end){throw 'Guardian integration timeout'};Start-Sleep -Milliseconds 100}
 $stdout.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output 'test.log') -Encoding utf8
 $stderr.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output 'test.stderr.txt') -Encoding utf8
 $receipt.test_exit_code=$client.ExitCode
 $end=[DateTime]::UtcNow.AddSeconds(8)
 while(!$server.HasExited -and [DateTime]::UtcNow -lt $end){Start-Sleep -Milliseconds 100}
 if(!$server.HasExited){throw 'Server cleanup did not complete'}
 $receipt.server_exit_code=$server.ExitCode
 if($client.ExitCode -ne 0 -or $server.ExitCode -ne 0){throw 'Guardian integration or cleanup failed; inspect saved logs'}
 $receipt.server=Get-Content -LiteralPath (Join-Path $root 'result.json') -Raw|ConvertFrom-Json
 if(!$receipt.server.memberships_empty -or $receipt.server.actual_client_sessions -ne 6 -or $receipt.server.authority_mutations -ne 0){throw 'Isolation/cleanup mismatch'}
 $receipt.status='verified_guardian_destruction_and_input_retirement'
}catch{$receipt.status='failed';$receipt.error=$_.Exception.Message;throw}
finally {
 foreach($proc in @($client,$server)){if($proc -and !$proc.HasExited){Stop-Process -Id $proc.Id -Force;$proc.WaitForExit(5000)|Out-Null}}
 foreach($name in @('ready.json','result.json','server-trace.jsonl','a-guardian-result.json','b-guardian-result.json')){if(Test-Path -LiteralPath (Join-Path $root $name)){Copy-Item -LiteralPath (Join-Path $root $name) -Destination (Join-Path $output $name)}}
 foreach($image in Get-ChildItem -LiteralPath $root -Filter '*-guardian-*.png' -File){Copy-Item -LiteralPath $image.FullName -Destination (Join-Path $output $image.Name)}
 Save
}
Write-Output $output
