#Requires -Version 7.0
param([Parameter(Mandatory)][string]$ManifestPath)
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'owned-validation-process.ps1')
$manifestPathFull=(Resolve-Path -LiteralPath $ManifestPath).Path
$ownedEvidence=[IO.Path]::GetFullPath((Join-Path $taskRoot '驗證紀錄/current'))+'\'
if(!$manifestPathFull.StartsWith($ownedEvidence,[StringComparison]::OrdinalIgnoreCase)){throw 'Loader probe manifest must stay in owned evidence'}
$manifest=Get-Content -LiteralPath $manifestPathFull -Raw|ConvertFrom-Json
$out=Join-Path (Split-Path -Parent $manifestPathFull) ('replay-'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out -Force|Out-Null
$sourcePins=@{}
foreach($name in @('維護工具/verify-native-relocated-loader.ps1','維護工具/owned-validation-process.ps1','維護工具/prepare-native-relocated-loader-probe.py')){
 $file=Join-Path $taskRoot $name;$sourcePins[$name]=(Get-FileHash -LiteralPath $file).Hash;$copy=Join-Path $out ('source/'+$name)
 New-Item -ItemType Directory -Path (Split-Path -Parent $copy) -Force|Out-Null;Copy-Item -LiteralPath $file -Destination $copy
}
$state=[ordered]@{status='running';output=$out;manifest=$manifestPathFull;manifest_sha256=(Get-FileHash -LiteralPath $manifestPathFull).Hash;source_pins=$sourcePins;cases=@();actual_formal_sessions=0;gameplay_verified=$false;clean_host_verified=$false;portable_delivery_complete=$false;whole_goal_complete=$false}
function Save-LoaderState {$state|ConvertTo-Json -Depth 15|Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding utf8}
Save-LoaderState
try{
 foreach($profile in @('Debug','Release')){
  $item=$manifest.products.$profile
  if((Get-FileHash -LiteralPath $item.main).Hash -ne $item.main_sha256 -or (Get-FileHash -LiteralPath $item.runtime_DLL).Hash -ne $item.runtime_DLL_sha256){throw 'Relocated image drift'}
  $signature=Get-AuthenticodeSignature -LiteralPath $item.runtime_DLL
  if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notlike '*Microsoft*'){throw 'Local CRT signature invalid'}
  $listener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$process=$null;$socket=$null
  try{
   $listener.Start();$endpoint='127.0.0.1:'+$listener.LocalEndpoint.Port
   $info=[Diagnostics.ProcessStartInfo]::new($item.main);$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$manifest.unrelated_cwd
   $info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.StandardOutputEncoding=[Text.UTF8Encoding]::new($false)
   $info.Environment['TEMP']=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$info.Environment['TMP']=$info.Environment['TEMP']
   $info.ArgumentList.Add('--probe-native-loopback');$info.ArgumentList.Add($endpoint)
   $process=[Diagnostics.Process]::Start($info);$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync();$started=$process.StartTime.ToUniversalTime().ToString('o')
   $until=[DateTime]::UtcNow.AddSeconds(10)
   while(!$listener.Pending()){if($process.HasExited -or [DateTime]::UtcNow -ge $until){throw 'Relocated native process failed before owned TCP connect'};Start-Sleep -Milliseconds 20}
   $socket=$listener.AcceptTcpClient();$process.Refresh()
   $module=@($process.Modules|Where-Object {$_.ModuleName -ieq 'vcruntime140.dll'})
   if($module.Count -ne 1 -or [IO.Path]::GetFullPath($module[0].FileName) -ine [IO.Path]::GetFullPath($item.runtime_DLL)){throw 'Relocated EXE did not load its own adjacent CRT'}
   $loadedPath=$module[0].FileName;$observedPID=$process.Id
   if(!$process.WaitForExit(12000)){throw 'Owned handshake diagnostic did not reach its original deadline'}
   $text=$stdout.GetAwaiter().GetResult();$text|Set-Content -LiteralPath (Join-Path $out ($profile+'-client.json')) -Encoding utf8
   $stderr.GetAwaiter().GetResult()|Set-Content -LiteralPath (Join-Path $out ($profile+'-stderr.txt')) -Encoding utf8
   $response=$text|ConvertFrom-Json
   if($process.ExitCode -eq 0 -or $response.status -ne 'failed' -or $response.error -notlike '*deadline*'){throw 'Expected original no-key handshake deadline was not observed'}
   $state.cases+=@{profile=$profile;PID=$observedPID;created_utc=$started;exe_sha256=$item.main_sha256;loaded_CRT_path=$loadedPath;loaded_CRT_sha256=(Get-FileHash -LiteralPath $loadedPath).Hash;CRT_signature='Valid';CRT_signer=$signature.SignerCertificate.Subject;unrelated_cwd=$manifest.unrelated_cwd;actual_formal_sessions=0;expected_no_key_failure=$response;exit_code=$process.ExitCode}
   Save-LoaderState
   Write-Output ($profile+' relocated signed x64 CRT loader verified')
  }finally{
   if($null -ne $process){if(!$process.HasExited){Stop-OwnedLitoValidationChild -Process $process -ScopeRoot $manifest.runtime};$process.Dispose()}
   if($null -ne $socket){$socket.Dispose()};$listener.Stop()
  }
 }
 foreach($name in $sourcePins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $sourcePins[$name]){throw 'Loader probe source drift'}}
 if((Get-FileHash -LiteralPath $manifestPathFull).Hash -ne $state.manifest_sha256){throw 'Loader probe manifest drift'}
 $state.status='verified_current_both_main_relocated_loader_with_adjacent_CRT';Save-LoaderState
}catch{$state.status='failed';$state.error=$_.Exception.Message;Save-LoaderState;throw}
