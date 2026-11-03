#Requires -Version 7.0
[CmdletBinding()]param([string]$SourceManifest='驗證紀錄/current/native-teleport-wire/green-inputs-before.json',[ValidateSet(16,64,256)][int]$CodegenUnits=256,[ValidateSet(0,1)][int]$LibraryOptLevel=0,[ValidateRange(1,100000)][int]$ExpectedSourceFiles=965)
$ErrorActionPreference='Stop'
$taskRoot='D:\lineage開發\3-lito開發\lito重建版'
$project=Join-Path $taskRoot 'LiTo381-Modern'
$pinsPath=[IO.Path]::GetFullPath((Join-Path $taskRoot $SourceManifest))
if(!$pinsPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Source manifest must remain inside this rebuild project'}
$pins=Get-Content -LiteralPath $pinsPath -Raw|ConvertFrom-Json -AsHashtable
if($pins.Count -ne $ExpectedSourceFiles){throw 'Review current compilation input scope'}
function Check-Inputs {
 foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw "Compilation input drift: $name"}}
}
Check-Inputs
$existing=Get-CimInstance Win32_Process -Filter "Name = 'cargo.exe' OR Name = 'rustc.exe'"|Where-Object {$_.CommandLine -and $_.CommandLine.Replace('/','\').Contains($project)}
if($existing){throw 'Existing project compiler; inspect before starting another'}
$out=Join-Path $taskRoot ('驗證紀錄/current/native-teleport-wire/library-memory-'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out|Out-Null
Copy-Item -LiteralPath $pinsPath -Destination (Join-Path $out 'source-inputs.json')
$toolchain='C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin'
$cargo=Join-Path $toolchain 'cargo.exe';$rustc=Join-Path $toolchain 'rustc.exe'
foreach($tool in @($cargo,$rustc,(Join-Path $toolchain 'rustdoc.exe'))){if(!(Test-Path -LiteralPath $tool)){throw 'Existing toolchain binary unavailable; no install/update attempted'}}
Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $out 'runner.ps1')
$compilerPins=@{}
foreach($tool in @((Get-Item -LiteralPath $cargo),(Get-Item -LiteralPath $rustc))+(Get-ChildItem -LiteralPath $toolchain -Filter 'rustc_driver*.dll')){$compilerPins[$tool.FullName]=(Get-FileHash -LiteralPath $tool.FullName).Hash}
$info=[Diagnostics.ProcessStartInfo]::new($cargo)
$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$project
$info.Environment['TEMP']=Join-Path $project 'target/current/scratch';$info.Environment['TMP']=$info.Environment['TEMP']
$info.Environment['RUSTC']=$rustc;$info.Environment['RUSTDOC']=Join-Path $toolchain 'rustdoc.exe'
$arguments=@('test','--lib','--no-run','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--config','profile.test.debug=0','--message-format=json')
if($CodegenUnits -ne 256){$arguments+=@('--config',('profile.test.codegen-units='+$CodegenUnits))}
if($LibraryOptLevel -ne 0){$arguments+=@('--config',('profile.test.package.lito-modern.opt-level='+$LibraryOptLevel))}
foreach($argument in $arguments){$info.ArgumentList.Add($argument)}
$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
$receipt=@{status='building';started_utc=[DateTime]::UtcNow.ToString('o');source_files=$pins.Count;source_manifest_sha256=(Get-FileHash -LiteralPath $pinsPath).Hash;cargo=$cargo;rustc=$rustc;compiler_pins=$compilerPins;arguments=$arguments;features=@();test_debug_information=0;codegen_units=$CodegenUnits;library_opt_level=$LibraryOptLevel;product_source_changed=$false;whole_goal_complete=$false;output=$out}
function Save {$receipt|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding utf8}
$process=[Diagnostics.Process]::Start($info)
$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
$receipt.pid=$process.Id;$receipt.creation_utc=$process.StartTime.ToUniversalTime().ToString('o');Save
$samplePath=Join-Path $out 'memory-samples.jsonl';$owned=@{};$owned[$process.Id]=$process.StartTime.ToUniversalTime();$lastMemory=$null;$samples=0
while(!$process.HasExited){
 $rows=@(Get-CimInstance Win32_Process -Filter "Name = 'cargo.exe' OR Name = 'rustc.exe' OR Name = 'link.exe'")
 foreach($row in $rows){if($owned.ContainsKey([int]$row.ParentProcessId) -and $row.CreationDate.ToUniversalTime() -ge $owned[[int]$row.ParentProcessId]){$owned[[int]$row.ProcessId]=$row.CreationDate.ToUniversalTime()}}
 $children=@($rows|Where-Object {$owned.ContainsKey([int]$_.ProcessId) -and [Math]::Abs(($_.CreationDate.ToUniversalTime()-$owned[[int]$_.ProcessId]).TotalSeconds) -lt 1}|ForEach-Object {@{pid=$_.ProcessId;name=$_.Name;parent_pid=$_.ParentProcessId;private_bytes=[long]$_.PrivatePageCount;working_set_bytes=[long]$_.WorkingSetSize}})
 if($samples%5 -eq 0){$os=Get-CimInstance Win32_OperatingSystem;$lastMemory=@{total_commit_kib=$os.TotalVirtualMemorySize;available_commit_kib=$os.FreeVirtualMemory;available_physical_kib=$os.FreePhysicalMemory}}
 @{utc=[DateTime]::UtcNow.ToString('o');processes=$children;host_memory=$lastMemory}|ConvertTo-Json -Depth 6 -Compress|Add-Content -LiteralPath $samplePath -Encoding utf8
 $samples++;Start-Sleep -Milliseconds 1000
}
$process.WaitForExit()
$stdout.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $out 'cargo-artifacts.jsonl') -Encoding utf8
$stderr.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $out 'cargo-stderr.txt') -Encoding utf8
$receipt.exit_code=$process.ExitCode;$receipt.finished_utc=[DateTime]::UtcNow.ToString('o');$receipt.memory_samples=$samples
$receipt.status=if($process.ExitCode -eq 0){'compiled_library_test_debug0_runtime_not_yet_executed'}else{'library_test_debug0_build_failed_measured'}
Check-Inputs;$receipt.sources_unchanged=$true;Save
foreach($tool in $compilerPins.Keys){if((Get-FileHash -LiteralPath $tool).Hash -ne $compilerPins[$tool]){throw 'Compiler binary drift during measured build'}}
Write-Output ($receipt|ConvertTo-Json -Depth 6)
if($process.ExitCode -ne 0){exit $process.ExitCode}
