#Requires -Version 7.0
[CmdletBinding()]param([Parameter(Mandatory)][string]$TestBinary,[Parameter(Mandatory)][string]$ReferenceJar)
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot='D:\lineage開發\3-lito開發\lito重建版';$project=Join-Path $taskRoot 'LiTo381-Modern'
$allowed=[IO.Path]::GetFullPath((Join-Path $project 'target/current'))+[IO.Path]::DirectorySeparatorChar
foreach($path in @($TestBinary,$ReferenceJar)){if(![IO.Path]::GetFullPath($path).StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw 'Only owned test artifacts are accepted'}}
$jarSha=(Get-FileHash -LiteralPath $ReferenceJar).Hash
if($jarSha -ne '6887A337E347D561366E0EB7EB8E93A2B7EA03ADD7A3888FC4BE2B70C3479DA7'){throw 'Review candidate drift'}
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $project ('target/current/negative-rental-return/'+$stamp)
$output=Join-Path $taskRoot ('驗證紀錄/current/negative-rental-return/'+$stamp)
$classes=Join-Path $root 'oracle';New-Item -ItemType Directory -Path $classes,$output -Force|Out-Null
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
function Quote([string]$s){if($s.Contains('"')){throw 'Invalid process argument'};'"'+$s+'"'}
$server=$null;$client=$null
$receipt=@{status='running';runtime=$root;output=$output;test_binary=$TestBinary;test_binary_sha256=(Get-FileHash -LiteralPath $TestBinary).Hash;jar_sha256=$jarSha;test_scope='one opt-in unit test; two genuine LOGIN/GAME clients; ten wire stages each';formal_deployed=$false}
function Save {$receipt|ConvertTo-Json -Depth 14|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
Save
try {
 $pins=@{}
 foreach($name in @('NativePeerHarness.java','run-negative-return.ps1')){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name)).Hash}
 foreach($name in @('native_login.rs','native_return_server_negative_tests.rs')){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $project ('src/modern_lito/'+$name))).Hash}
 $receipt.source_pins=$pins;Save
 & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $ReferenceJar -d $classes (Join-Path $PSScriptRoot 'NativePeerHarness.java') *> (Join-Path $output 'javac.log')
 if($LASTEXITCODE -ne 0){throw 'Harness compile failed'}
 $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-Xverify:all','-cp',(Quote ($classes+';'+$ReferenceJar)),'local.lito381.server.NativePeerHarness',(Quote $root),'rental-return') -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output 'server.log') -RedirectStandardError (Join-Path $output 'server.stderr.txt')
 $end=[DateTime]::UtcNow.AddSeconds(15)
 while(!(Test-Path -LiteralPath (Join-Path $root 'ready.json'))){if($server.HasExited -or [DateTime]::UtcNow -ge $end){throw 'Server not ready'};Start-Sleep -Milliseconds 50}
 $info=[Diagnostics.ProcessStartInfo]::new($TestBinary);$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
 $info.Environment['LITO_NEGATIVE_ROOT']=$root;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $info.ArgumentList.Add('modern_lito::native_login::return_server_negative_tests::actual_server_rejects_out_of_order_and_duplicate_return_requests')
 foreach($arg in @('--exact','--ignored','--nocapture')){$info.ArgumentList.Add($arg)}
 $client=[Diagnostics.Process]::Start($info);$stdout=$client.StandardOutput.ReadToEndAsync();$stderr=$client.StandardError.ReadToEndAsync()
 $end=[DateTime]::UtcNow.AddSeconds(90)
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
 $receipt.status='verified_candidate_negative_return_requests'
}catch{$receipt.status='failed';$receipt.error=$_.Exception.Message;throw}
finally {
 foreach($proc in @($client,$server)){if($proc -and !$proc.HasExited){Stop-Process -Id $proc.Id -Force;$proc.WaitForExit(5000)|Out-Null}}
 foreach($name in @('ready.json','result.json','server-trace.jsonl','a-negative.json','b-negative.json')){if(Test-Path -LiteralPath (Join-Path $root $name)){Copy-Item -LiteralPath (Join-Path $root $name) -Destination (Join-Path $output $name)}}
 Save
}
Write-Output $output
