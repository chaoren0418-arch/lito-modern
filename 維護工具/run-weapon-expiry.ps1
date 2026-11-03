#Requires -Version 7.0
[CmdletBinding()]param([Parameter(Mandatory)][string]$TestBinary,[Parameter(Mandatory)][string]$ReferenceJar)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot='D:\lineage開發\3-lito開發\lito重建版';$project=Join-Path $taskRoot 'LiTo381-Modern'
$allowed=[IO.Path]::GetFullPath((Join-Path $project 'target/current'))+[IO.Path]::DirectorySeparatorChar
foreach($path in @($TestBinary,$ReferenceJar)){if(![IO.Path]::GetFullPath($path).StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw 'Only owned test artifacts are accepted'}}
$jarSha=(Get-FileHash -LiteralPath $ReferenceJar).Hash
if($jarSha -ne 'BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43'){throw 'Review candidate drift'}
$existing=Get-CimInstance Win32_Process|Where-Object {$_.CommandLine -and $_.CommandLine.Contains('native-weapon-expiry') -and $_.Name -eq 'java.exe'}
if($existing){throw 'Owned weapon expiry lease already active; inspect its receipt first'}
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $project ('target/current/native-weapon-expiry/'+$stamp)
$output=Join-Path $taskRoot ('驗證紀錄/current/native-weapon-expiry/'+$stamp)
$classes=Join-Path $root 'oracle';New-Item -ItemType Directory -Path $classes,$output -Force|Out-Null
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
function Quote([string]$s){if($s.Contains('"')){throw 'Invalid process argument'};'"'+$s+'"'}
$server=$null;$client=$null
$receipt=@{status='running';runtime=$root;output=$output;test_binary=$TestBinary;test_binary_sha256=(Get-FileHash -LiteralPath $TestBinary).Hash;jar_sha256=$jarSha;test_scope='one opt-in integration test; two actual players, equipped formal dagger19002, true 3600s leases, received inventory/walk restoration; no clock/authority injection';formal_deployed=$false}
function Save {$receipt|ConvertTo-Json -Depth 14|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
Save
try {
 $pins=@{}
 foreach($name in @('修補提案/租裝返回/current-guard-candidate/NativePeerHarness.java','維護工具/run-weapon-expiry.ps1','LiTo381-Modern/tests/native_weapon_expiry.rs','LiTo381-Modern/src/modern_lito/native_login.rs','LiTo381-Modern/src/modern_lito/native_item_actions.rs','LiTo381-Modern/src/modern_lito/native_inventory.rs','LiTo381-Modern/src/modern_lito/native_world.rs')){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash}

 $receipt.source_pins=$pins;$receipt.expected_seconds=3600;$receipt.clock_or_authority_injection=$false;Save
 $TestBinaryOriginal=$TestBinary;$TestBinary=Join-Path $root 'immutable-weapon-test.exe';Copy-Item -LiteralPath $TestBinaryOriginal -Destination $TestBinary
 if((Get-FileHash -LiteralPath $TestBinary).Hash -ne $receipt.test_binary_sha256){throw 'Immutable test binary mismatch'}
 $receipt.immutable_test_binary=$TestBinary;Save
 & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $ReferenceJar -d $classes (Join-Path $taskRoot '修補提案/租裝返回/current-guard-candidate/NativePeerHarness.java') *> (Join-Path $output 'javac.log')
 if($LASTEXITCODE -ne 0){throw 'Harness compile failed'}
 $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-Xverify:all','-cp',(Quote ($classes+';'+$ReferenceJar)),'local.lito381.server.NativePeerHarness',(Quote $root),'rental-equipped-expiry') -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output 'server.log') -RedirectStandardError (Join-Path $output 'server.stderr.txt')
 $end=[DateTime]::UtcNow.AddSeconds(15)
 while(!(Test-Path -LiteralPath (Join-Path $root 'ready.json'))){if($server.HasExited -or [DateTime]::UtcNow -ge $end){throw 'Server not ready'};Start-Sleep -Milliseconds 50}
 $info=[Diagnostics.ProcessStartInfo]::new($TestBinary);$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
 $info.Environment['LITO_WEAPON_ROOT']=$root;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $info.ArgumentList.Add('two_equipped_weapons_expire_after_real_one_hour_and_restore_original_weapon')
 foreach($arg in @('--exact','--ignored','--nocapture')){$info.ArgumentList.Add($arg)}
 $client=[Diagnostics.Process]::Start($info);$stdout=$client.StandardOutput.ReadToEndAsync();$stderr=$client.StandardError.ReadToEndAsync()
 $receipt.owned_processes=@(@{pid=$server.Id;start_utc=$server.StartTime.ToUniversalTime().ToString('o');kind='formal_server'},@{pid=$client.Id;start_utc=$client.StartTime.ToUniversalTime().ToString('o');kind='immutable_weapon_test'});$receipt.status='real_equipped_weapon_expiry_running';$receipt.lease_deadline_utc=[DateTime]::UtcNow.AddSeconds(4300).ToString('o');Save
 $end=[DateTime]::UtcNow.AddSeconds(4300)
 while(!$client.HasExited){if([DateTime]::UtcNow -ge $end){throw 'Negative test timeout'};Start-Sleep -Milliseconds 100}
 $stdout.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output 'test.log') -Encoding utf8
 $stderr.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output 'test.stderr.txt') -Encoding utf8
 $receipt.test_exit_code=$client.ExitCode
 $end=[DateTime]::UtcNow.AddSeconds(8)
 while(!$server.HasExited -and [DateTime]::UtcNow -lt $end){Start-Sleep -Milliseconds 100}
 if(!$server.HasExited){throw 'Server cleanup did not complete'}
 $receipt.server_exit_code=$server.ExitCode
 if($client.ExitCode -ne 0 -or $server.ExitCode -ne 0){throw 'Negative test or cleanup failed; inspect saved logs'}
 $receipt.server=Get-Content -LiteralPath (Join-Path $root 'result.json') -Raw|ConvertFrom-Json
 if(!$receipt.server.memberships_empty -or $receipt.server.actual_client_sessions -ne 4 -or $receipt.server.authority_mutations -ne 0){throw 'Isolation/cleanup mismatch'}
 $receipt.status='verified_two_real_equipped_weapon_expiries'
}catch{$receipt.status='failed';$receipt.error=$_.Exception.Message;throw}
finally {
 foreach($proc in @($client,$server)){if($proc -and !$proc.HasExited){Stop-Process -Id $proc.Id -Force;$proc.WaitForExit(5000)|Out-Null}}
 foreach($name in @('ready.json','result.json','server-trace.jsonl','a-weapon-preparation.json','b-weapon-preparation.json','a-weapon-result.json','b-weapon-result.json','a-weapon-progress.json','b-weapon-progress.json','a-weapon-restored.png','b-weapon-restored.png')){if(Test-Path -LiteralPath (Join-Path $root $name)){Copy-Item -LiteralPath (Join-Path $root $name) -Destination (Join-Path $output $name)}}
 Save
}
Write-Output $output
