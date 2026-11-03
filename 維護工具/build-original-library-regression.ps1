#Requires -Version 7.0
[CmdletBinding()]param([ValidatePattern('^[a-z0-9-]+$')][string]$RunLabel='targeted-original-tests-v2',[string]$SourceManifest='驗證紀錄/current/native-teleport-wire/library-diagnostic-v2-inputs.json')
$ErrorActionPreference='Stop'
$taskRoot='D:\lineage開發\3-lito開發\lito重建版';$project=Join-Path $taskRoot 'LiTo381-Modern'
$stage=Join-Path $taskRoot '驗證紀錄/current/native-teleport-wire';$output=Join-Path $stage $RunLabel
if(Test-Path -LiteralPath $output){throw 'Existing run must be preserved and inspected'}
$manifestPath=[IO.Path]::GetFullPath((Join-Path $taskRoot $SourceManifest));if(!$manifestPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned source manifest required'}
$pins=Get-Content -LiteralPath $manifestPath -Raw|ConvertFrom-Json -AsHashtable
foreach($name in @('LiTo381-Modern/tests/native_library_regression.rs','LiTo381-Modern/tests/native_library_regression/modules.rs','LiTo381-Modern/tests/native_library_regression/city_runtime.rs')){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash}
function Check-Inputs {foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw "Source drift: $name"}}}
Check-Inputs
New-Item -ItemType Directory -Path $output|Out-Null
$pins|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $output 'source-inputs.json') -Encoding utf8
Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $output 'runner.ps1')
function Mains {$m=@{};foreach($profile in @('Debug','Release')){$p=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$profile.ToLowerInvariant()+'/LiToModern.exe');$m[$profile]=(Get-FileHash -LiteralPath $p).Hash};return $m}
$tools='C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin'
$info=[Diagnostics.ProcessStartInfo]::new((Join-Path $tools 'cargo.exe'));$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
$info.Environment['RUSTC']=Join-Path $tools 'rustc.exe';$info.Environment['RUSTDOC']=Join-Path $tools 'rustdoc.exe'
$info.Environment['TEMP']=Join-Path $project 'target/current/scratch';$info.Environment['TMP']=$info.Environment['TEMP']
$arguments=@('test','--no-run','--test','native_library_regression','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json')
foreach($argument in $arguments){$info.ArgumentList.Add($argument)}
$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
$receipt=@{status='compiling';started_utc=[DateTime]::UtcNow.ToString('o');source_files=$pins.Count;source_manifest_sha256=(Get-FileHash -LiteralPath (Join-Path $output 'source-inputs.json')).Hash;prestart_snapshot=$true;features=@();test_target='native_library_regression';arguments=$arguments;normal_mains_before=(Mains);whole_goal_complete=$false}
function Save {$receipt|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
Save
$process=[Diagnostics.Process]::Start($info);$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
$receipt.pid=$process.Id;$receipt.creation_utc=$process.StartTime.ToUniversalTime().ToString('o');Save
while(!$process.HasExited){Start-Sleep -Milliseconds 250}
$process.WaitForExit()
$stdout.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output 'cargo-artifacts.jsonl') -Encoding utf8
$stderr.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output 'cargo-stderr.txt') -Encoding utf8
$receipt.exit_code=$process.ExitCode;$receipt.finished_utc=[DateTime]::UtcNow.ToString('o');$receipt.normal_mains_after=Mains
Check-Inputs;$receipt.sources_unchanged=$true;$receipt.status=if($process.ExitCode -eq 0){'compiled_original_library_regression_subset'}else{'original_library_regression_subset_compile_failed'};Save
Write-Output ($receipt|ConvertTo-Json -Depth 8)
if($process.ExitCode -ne 0){exit $process.ExitCode}
