#Requires -Version 7.0
[CmdletBinding()]param([Parameter(Mandatory)][string]$TestBinary,[ValidateSet('Debug','Release')][string]$BuildProfile='Debug')
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$allowed=[IO.Path]::GetFullPath((Join-Path $taskRoot 'LiTo381-Modern/target/current'))+'\'
$binary=[IO.Path]::GetFullPath($TestBinary)
if(!$binary.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($binary) -notmatch '^native_audio_device-[a-f0-9]+\.exe$'){throw 'Only the owned native audio integration executable is accepted'}
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$runtime=Join-Path $allowed ('native-audio-device/'+$stamp)
$output=Join-Path $taskRoot ('驗證紀錄/current/native-audio-device/'+$stamp)
New-Item -ItemType Directory -Path $runtime,$output -Force|Out-Null
$immutable=Join-Path $runtime 'immutable-audio-test.exe';Copy-Item -LiteralPath $binary -Destination $immutable
$pins=@{}
foreach($name in @('LiTo381-Modern/src/modern_lito/audio_output.rs','LiTo381-Modern/src/modern_lito/audio.rs','LiTo381-Modern/src/modern_lito/combat_audio.rs','LiTo381-Modern/src/modern_lito/resources.rs','LiTo381-Modern/tests/native_audio_device.rs','LiTo381-Modern/Cargo.toml','LiTo381-Modern/.cargo/config.toml','維護工具/run-native-audio-device.ps1')){
 $pins[$name]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash
 $copy=Join-Path $output ('source-snapshot/'+$name);New-Item -ItemType Directory -Path (Split-Path -Parent $copy) -Force|Out-Null;Copy-Item -LiteralPath (Join-Path $taskRoot $name) -Destination $copy
}
$receipt=@{status='running';build_profile=$BuildProfile;test_binary_sha256=(Get-FileHash $immutable).Hash;immutable_test_binary=$immutable;runtime=$runtime;source_pins=$pins;physical_audibility_verified=$false;native_GUI_verified=$false;full_game_complete=$false}
function Save {$receipt|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
$resourcePins=@{};foreach($id in @(97,99,102)){$path='D:/lineage開發/0-客戶端/Sound/'+$id+'.wav';$resourcePins[$path]=(Get-FileHash -LiteralPath $path).Hash};$receipt.readonly_resource_pins=$resourcePins
$process=$null;Save
try {
 $info=[Diagnostics.ProcessStartInfo]::new($immutable);$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$runtime
 $info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.Environment['LITO_AUDIO_TEST_ROOT']=$runtime
 foreach($arg in @('original_PCM_actual_device_completion_reset_replacement_multivoice_and_drop','--exact','--ignored','--nocapture')){$info.ArgumentList.Add($arg)}
 $process=[Diagnostics.Process]::Start($info);$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
 $receipt.owned_process=@{pid=$process.Id;start_utc=$process.StartTime.ToUniversalTime().ToString('o')};Save
 $until=[DateTime]::UtcNow.AddSeconds(60)
 while(!$process.HasExited){if([DateTime]::UtcNow -ge $until){throw 'Actual audio device test exceeded its 60s lease'};Start-Sleep -Milliseconds 50}
 $stdout.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output 'test.log') -Encoding utf8
 $stderr.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $output 'test.stderr.txt') -Encoding utf8
 $receipt.test_exit_code=$process.ExitCode
 if($process.ExitCode -ne 0){throw 'Actual audio device test failed; inspect saved logs'}
 $receipt.report=Get-Content -LiteralPath (Join-Path $runtime 'actual-audio-device.json') -Raw|ConvertFrom-Json
 foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw 'Source changed during audio test'}}
 if((Get-FileHash $immutable).Hash -ne $receipt.test_binary_sha256){throw 'Test executable changed'}
 foreach($path in $resourcePins.Keys){if((Get-FileHash -LiteralPath $path).Hash -ne $resourcePins[$path]){throw 'Readonly source WAV changed during audio test'}}
 $receipt.inputs_unchanged=$true;$receipt.status='verified_actual_native_audio_device_lifecycle'
}catch{$receipt.status='failed';$receipt.error=$_.Exception.Message;throw}
finally {
 if($process -and !$process.HasExited){
  $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
  if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$live.CommandLine.Replace('/','\').Replace('\\','\').Contains($immutable.Replace('/','\'))){throw 'Owned audio PID/creation/command mismatch'}
  $process.Kill();$process.WaitForExit(5000)|Out-Null
 }
 if(Test-Path -LiteralPath (Join-Path $runtime 'actual-audio-device.json')){Copy-Item -LiteralPath (Join-Path $runtime 'actual-audio-device.json') -Destination $output}
 Save
}
Write-Output $output
