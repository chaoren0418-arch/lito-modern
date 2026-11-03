#Requires -Version 7.0
# Owned static SQL; optional untouched formal-session gameplay fixture.
[CmdletBinding()]param([switch]$GameTest,[switch]$GuiTest,[ValidateSet('Debug','Release')][string]$BuildProfile='Debug')
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
if($GameTest -and $GuiTest){throw 'Choose one live test mode'}
if($GuiTest){& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile|Out-Null}
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $taskRoot ('LiTo381-Modern/target/current/travel-sql/'+$stamp)
$data=Join-Path $root 'data';$sqlTmp=Join-Path $root 'tmp';$output=Join-Path $taskRoot ('驗證紀錄/current/travel-sql/'+$stamp)
New-Item -ItemType Directory -Path $data,$sqlTmp,$output -Force|Out-Null
$allowed=[IO.Path]::GetFullPath((Join-Path $taskRoot 'LiTo381-Modern/target/current'))+'\'
if(![IO.Path]::GetFullPath($data).StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase) -or (Get-ChildItem -LiteralPath $data -Force).Count){throw 'SQL fixture must be a new empty owned directory'}
$mysqlBase='C:\Program Files\MySQL\MySQL Server 5.5';$bin=Join-Path $mysqlBase 'bin'
function RunWithInput([string]$exe,[string[]]$arguments,[string]$inputText,[string]$log){
 if($exe.EndsWith('mysqld.exe') -and ($arguments -notcontains ('--datadir='+$data) -or $arguments -notcontains '--no-defaults')){throw 'Owned explicit datadir/defaults arguments missing'}
 $info=[Diagnostics.ProcessStartInfo]::new($exe);$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$root;$info.Environment['TEMP']=$sqlTmp;$info.Environment['TMP']=$sqlTmp
 foreach($arg in $arguments){$info.ArgumentList.Add($arg)}
 $info.RedirectStandardInput=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.StandardInputEncoding=[Text.UTF8Encoding]::new($false);$info.StandardOutputEncoding=[Text.UTF8Encoding]::new($false);$info.StandardErrorEncoding=[Text.UTF8Encoding]::new($false)
 $process=[Diagnostics.Process]::Start($info);$stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
 $writeError=$null;try{$process.StandardInput.Write($inputText);$process.StandardInput.Close()}catch{$writeError=$_.Exception.Message}
 $process.WaitForExit()
 ($stdout.GetAwaiter().GetResult()+$stderr.GetAwaiter().GetResult())|Set-Content -LiteralPath $log -Encoding utf8
 $code=$process.ExitCode;$process.Dispose();if($writeError){Add-Content -LiteralPath $log -Value $writeError};if($code -ne 0){throw ('Owned SQL helper failed: '+$code+'; '+$log)}
}
function StopOwned($process){
 if($process.HasExited){return};$live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
 if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$live.CommandLine.Replace('/','\').Replace('\\','\').Contains($data.Replace('/','\').Replace('\\','\'))){throw 'Owned MySQL identity mismatch'}
 $process.Kill();$process.WaitForExit(5000)|Out-Null
}
Add-Type -TypeDefinition 'using System.Runtime.InteropServices;public static class OwnedSqlACP{[DllImport("kernel32.dll")]public static extern uint GetACP();}'
$gameProcess=$null;$testProcess=$null;$server=$null;$result=@{status='running';runtime=$root;output=$output;player_tables_accessed=$false;source_project_modified=$false}
function StopChildOwned($process,[string]$marker){
 if(!$process -or $process.HasExited){return}
 $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
 if(!$live){$process.Refresh();if($process.HasExited){return};throw 'Owned travel child handle live but PID missing'}
 if([Math]::Abs(($live.CreationDate.ToUniversalTime()-$process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$live.CommandLine.Replace('/','\').Replace('\\','\').Contains($marker.Replace('/','\').Replace('\\','\'))){throw 'Owned travel child identity mismatch'}
 $process.Kill();$process.WaitForExit(5000)|Out-Null
}
function StartChild([string]$exe,[string[]]$arguments,[string]$log,[string]$testRoot=''){
 $info=[Diagnostics.ProcessStartInfo]::new($exe);$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$root
 $info.Environment['TEMP']=$sqlTmp;$info.Environment['TMP']=$sqlTmp
 if($testRoot){$info.Environment['LITO_TRAVEL_ROOT']=$testRoot}
 foreach($arg in $arguments){$info.ArgumentList.Add($arg)}
 $info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true;$info.StandardOutputEncoding=[Text.UTF8Encoding]::new($false);$info.StandardErrorEncoding=[Text.UTF8Encoding]::new($false)
 $process=[Diagnostics.Process]::Start($info)
 return @{process=$process;stdout=$process.StandardOutput.ReadToEndAsync();stderr=$process.StandardError.ReadToEndAsync();log=$log}
}
function SaveChild($child){($child.stdout.GetAwaiter().GetResult()+$child.stderr.GetAwaiter().GetResult())|Set-Content -LiteralPath $child.log -Encoding utf8}
function Save {$result|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $output 'receipt.json') -Encoding utf8}
Save
$sourceNames=@('LiTo381-Modern/src/modern_lito/native_item_actions.rs','LiTo381-Modern/src/modern_lito/native_login.rs','LiTo381-Modern/src/modern_lito/native_login_window.rs','LiTo381-Modern/src/modern_lito/native_portal.rs','LiTo381-Modern/src/modern_lito/native_skill_effects.rs','LiTo381-Modern/src/modern_lito/native_world_scene.rs','LiTo381-Modern/tests/native_item_actions.rs','LiTo381-Modern/tests/native_travel.rs','LiTo381-Modern/tests/native_painted_input.rs','LiTo381-Modern/tests/fixtures/native-painted-input-current.json','LiTo381-Modern/tests/fixtures/travel-static-fixture.sql','LiTo381-Modern/tests/fixtures/native-target-rejection-current.json','LiTo381-Modern/tests/oracle/NativePeerHarness.java','LiTo381-Modern/tests/oracle/GmTravelSourceOracle.java','LiTo381-Modern/tests/oracle/NativeTravelHarness.java','維護工具/check-isolated-travel-sql.ps1','維護工具/verify-native-travel-authority.py','維護工具/verify-native-travel-gui-authority.py','維護工具/verify-native-travel-window.ps1','維護工具/run-isolated-native-travel-window-test.ps1','LiTo381-Modern/Cargo.toml','LiTo381-Modern/.cargo/config.toml')
$result.source_pins=@{}
foreach($name in $sourceNames){
 $source=Join-Path $taskRoot $name;$dest=Join-Path (Join-Path $output 'source-snapshot') $name
 New-Item -ItemType Directory -Path (Split-Path -Parent $dest) -Force|Out-Null
 Copy-Item -LiteralPath $source -Destination $dest
 $result.source_pins[$name]=(Get-FileHash -LiteralPath $source).Hash
 if((Get-FileHash -LiteralPath $dest).Hash -ne $result.source_pins[$name]){throw 'Travel source snapshot mismatch'}
}
Save
try {
 $bootstrap='CREATE DATABASE mysql;'+[Environment]::NewLine+'USE mysql;'+[Environment]::NewLine+(Get-Content -LiteralPath (Join-Path $mysqlBase 'share/mysql_system_tables.sql') -Raw)+(Get-Content -LiteralPath (Join-Path $mysqlBase 'share/mysql_system_tables_data.sql') -Raw)
 RunWithInput (Join-Path $bin 'mysqld.exe') @('--no-defaults','--bootstrap',('--basedir='+$mysqlBase),('--datadir='+$data),'--skip-innodb',('--tmpdir='+$sqlTmp),'--default-storage-engine=MyISAM','--skip-networking',('--log-error='+(Join-Path $root 'bootstrap-error.log'))) $bootstrap (Join-Path $output 'bootstrap.log')
 $listener=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$listener.Start();$port=$listener.LocalEndpoint.Port;$listener.Stop()
 $info=[Diagnostics.ProcessStartInfo]::new((Join-Path $bin 'mysqld.exe'));$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$root;$info.Environment['TEMP']=$sqlTmp;$info.Environment['TMP']=$sqlTmp
 foreach($arg in @('--no-defaults',('--basedir='+$mysqlBase),('--datadir='+$data),'--bind-address=127.0.0.1',('--port='+$port),'--skip-innodb',('--tmpdir='+$sqlTmp),'--default-storage-engine=MyISAM',('--pid-file='+(Join-Path $root 'mysql.pid')),('--log-error='+(Join-Path $root 'mysql-error.log')))){$info.ArgumentList.Add($arg)}
 $server=[Diagnostics.Process]::Start($info);$result.port=$port;$result.owned_process=@{pid=$server.Id;start_utc=$server.StartTime.ToUniversalTime().ToString('o');datadir=$data};Save
 $end=[DateTime]::UtcNow.AddSeconds(15)
 while(!(Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue|Where-Object {$_.OwningProcess -eq $server.Id -and $_.LocalAddress -eq '127.0.0.1'})){if($server.HasExited -or [DateTime]::UtcNow -gt $end){throw 'Owned MySQL did not bind its expected loopback socket'};Start-Sleep -Milliseconds 100}
 $args=@('--no-defaults','--protocol=TCP','--host=127.0.0.1',('--port='+$port),'--user=root','--batch','--raw','--skip-column-names','--default-character-set=utf8')
 RunWithInput (Join-Path $bin 'mysql.exe') $args 'SELECT HEX(@@datadir);' (Join-Path $output 'datadir.log')
 $hex=(Get-Content -LiteralPath (Join-Path $output 'datadir.log') -Raw).Trim()
 $actual=[Text.Encoding]::GetEncoding([int][OwnedSqlACP]::GetACP()).GetString([Convert]::FromHexString($hex)).Replace('/','\').Replace('\\','\').TrimEnd('\')
 if($actual -ne $data.Replace('/','\').TrimEnd('\')){throw 'SQL target data directory mismatch; fixture import refused'}
 $fixture=Join-Path $taskRoot 'LiTo381-Modern/tests/fixtures/travel-static-fixture.sql'
 $sql=(Get-Content -LiteralPath $fixture -Raw)+'CREATE USER ''travel_read''@''127.0.0.1'' IDENTIFIED BY ''TravelFixtureOnly''; GRANT SELECT ON travel_static.* TO ''travel_read''@''127.0.0.1'';'
 RunWithInput (Join-Path $bin 'mysql.exe') $args $sql (Join-Path $output 'import.log')
 RunWithInput (Join-Path $bin 'mysql.exe') $args 'SELECT item_id,name,classname,use_type,invgfx FROM travel_static.etcitem; SHOW TABLES FROM travel_static;' (Join-Path $output 'static-row.log')
 $jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin';$classes=Join-Path $root 'oracle';New-Item -ItemType Directory -Path $classes -Force|Out-Null
 $jar=Join-Path $taskRoot 'LiTo381-Modern/target/current/server-reference/formal-server-C02AAAE8.jar'
 $driver='C:\Users\CHAOREN\.m2\repository\com\mysql\mysql-connector-j\8.0.33\mysql-connector-j-8.0.33.jar'
 if((Get-FileHash -LiteralPath $jar).Hash -ne 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'){throw 'Fixed travel reference hash mismatch'}
 & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp ($jar+';'+$driver) -d $classes (Join-Path $taskRoot 'LiTo381-Modern/tests/oracle/GmTravelSourceOracle.java') *> (Join-Path $output 'javac.log')
 if($LASTEXITCODE -ne 0){throw 'Travel source oracle compile failed'}
 & (Join-Path $jdk 'java.exe') '-Dfile.encoding=UTF-8' '-Xverify:all' -cp ($classes+';'+$jar+';'+$driver) local.lito381.server.GmTravelSourceOracle $root $port *> (Join-Path $output 'oracle.log')
 if($LASTEXITCODE -ne 0){throw 'Actual SQL/formal travel source oracle failed'}
 $result.oracle=Get-Content -LiteralPath (Join-Path $root 'gm-travel-source-oracle.json') -Raw|ConvertFrom-Json
 Copy-Item -LiteralPath (Join-Path $root 'gm-travel-source-oracle.json') -Destination (Join-Path $output 'gm-travel-source-oracle.json')
 $result.jar_sha256=(Get-FileHash -LiteralPath $jar).Hash;$result.driver_sha256=(Get-FileHash -LiteralPath $driver).Hash
 $result.static_fixture_sha256=(Get-FileHash -LiteralPath $fixture).Hash;$result.status='verified_isolated_actual_SQL_static42501_row';Save
 if($GameTest -or $GuiTest){
  if(!$GuiTest){
  $profile=$BuildProfile.ToLowerInvariant();$testLog=Join-Path $taskRoot ('驗證紀錄/current/travel-'+$profile+'.log')
  $logText=Get-Content -LiteralPath $testLog -Raw
  $foundTests=[regex]::Matches($logText,'Running tests\\native_travel.rs \(([^\r\n]+)\)')
  if($foundTests.Count -ne 1){throw 'Exactly one current native_travel test executable receipt required'}
  $testExe=[IO.Path]::GetFullPath((Join-Path (Join-Path $taskRoot 'LiTo381-Modern') $foundTests[0].Groups[1].Value))
  if(!$testExe.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase) -or !(Test-Path -LiteralPath $testExe)){throw 'Owned test executable missing/outside current target'}
  }
  & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp ($jar+';'+$driver) -d $classes (Join-Path $taskRoot 'LiTo381-Modern/tests/oracle/NativePeerHarness.java') (Join-Path $taskRoot 'LiTo381-Modern/tests/oracle/GmTravelSourceOracle.java') (Join-Path $taskRoot 'LiTo381-Modern/tests/oracle/NativeTravelHarness.java') *> (Join-Path $output 'game-javac.log')
  if($LASTEXITCODE -ne 0){throw 'Travel formal-session harness compile failed'}
  $gameArgs=@('-Dfile.encoding=UTF-8','-Xverify:all','-cp',($classes+';'+$jar+';'+$driver),'local.lito381.server.NativeTravelHarness',$root,[string]$port)
  if($GuiTest){$gameArgs+='gui'}
  $gameChild=StartChild (Join-Path $jdk 'java.exe') $gameArgs (Join-Path $output 'game-harness.log')
  $gameProcess=$gameChild.process
  $result.game_process=@{pid=$gameProcess.Id;start_utc=$gameProcess.StartTime.ToUniversalTime().ToString('o');root=$root};Save
  $end=[DateTime]::UtcNow.AddSeconds(15)
  while(!(Test-Path -LiteralPath (Join-Path $root 'travel-ready.json'))){if($gameProcess.HasExited -or [DateTime]::UtcNow -gt $end){StopChildOwned $gameProcess $root;SaveChild $gameChild;throw 'Owned travel formal-session harness did not become ready'};Start-Sleep -Milliseconds 100}
  $ready=Get-Content -LiteralPath (Join-Path $root 'travel-ready.json') -Raw|ConvertFrom-Json
  foreach($endpoint in @(@{port=$ready.port;address='127.0.0.1'},@{port=$ready.game_port;address='127.0.0.2'})){
   if(!(Get-NetTCPConnection -LocalPort $endpoint.port -State Listen -ErrorAction SilentlyContinue|Where-Object {$_.OwningProcess -eq $gameProcess.Id -and $_.LocalAddress -eq $endpoint.address})){throw 'Formal game listener ownership mismatch'}
  }
  if($GuiTest){
   & (Join-Path $taskRoot '維護工具/verify-native-travel-window.ps1') -Runtime $root -Output $output -BuildProfile $BuildProfile
   $result.gui=Get-Content -LiteralPath (Join-Path $output 'travel-gui.json') -Raw|ConvertFrom-Json
  }else{
  $testChild=StartChild $testExe @('--ignored','--exact','actual_sql_wand_portal_retires_owner_and_preserves_peer_scene','--nocapture') (Join-Path $output 'game-test.log') $root
  $testProcess=$testChild.process
  $result.game_test=@{exe=$testExe;sha256=(Get-FileHash -LiteralPath $testExe).Hash;profile=$BuildProfile;pid=$testProcess.Id;start_utc=$testProcess.StartTime.ToUniversalTime().ToString('o')};Save
  $end=[DateTime]::UtcNow.AddSeconds(140)
  while(!$testProcess.HasExited){if([DateTime]::UtcNow -gt $end){StopChildOwned $testProcess $testExe;SaveChild $testChild;throw 'Owned travel native test timeout'};Start-Sleep -Milliseconds 100}
  SaveChild $testChild
  if($testProcess.ExitCode -ne 0){$gameProcess.WaitForExit(5000)|Out-Null;throw 'Native formal travel gameplay test failed'}
  }
  $end=[DateTime]::UtcNow.AddSeconds(10)
  while(!$gameProcess.HasExited){if([DateTime]::UtcNow -gt $end){throw 'Owned game harness did not stop after test'};Start-Sleep -Milliseconds 100}
  SaveChild $gameChild
  if($gameProcess.ExitCode -ne 0){throw 'Formal session travel harness failed'}
  if(!$GuiTest){
  $retained=Join-Path $output 'native-travel-test.exe'
  Copy-Item -LiteralPath $testExe -Destination $retained
  if((Get-FileHash -LiteralPath $retained).Hash -ne $result.game_test.sha256){throw 'Travel executed test binary changed before retention'}
  @{original_path=$testExe;retained_path=$retained;sha256=$result.game_test.sha256}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $output 'test-artifact-retention.json') -Encoding utf8
  }
  $result.game_harness=Get-Content -LiteralPath (Join-Path $root 'travel-harness-result.json') -Raw|ConvertFrom-Json
  $artifactNames=@('travel-server-trace.jsonl','travel-harness-result.json','travel-ready.json')
  if(!$GuiTest){$artifactNames+=@('a-travel.json','b-travel.json','a-before.png','a-after.png','b-peer-effect169.png')}
  foreach($name in $artifactNames){Copy-Item -LiteralPath (Join-Path $root $name) -Destination (Join-Path $output $name)}
  Copy-Item -LiteralPath (Join-Path $root 'travel-captures') -Destination (Join-Path $output 'travel-captures') -Recurse
  $result.status=if($GuiTest){'native_GUI_travel_test_passed_independent_capture_verification_pending'}else{'native_formal_travel_test_passed_independent_capture_verification_pending'};Save
 }

}catch{$result.status='failed';$result.error=$_.Exception.Message;Save;throw}
finally {
 $cleanupErrors=@()
 try {if($testProcess){StopChildOwned $testProcess $testExe;SaveChild $testChild;$testProcess.Dispose()}}catch{$cleanupErrors+=@($_.Exception.Message)}
 try {if($gameProcess){StopChildOwned $gameProcess $root;SaveChild $gameChild;$gameProcess.Dispose()}}catch{$cleanupErrors+=@($_.Exception.Message)}
 try {if($server){if(!$server.HasExited){StopOwned $server};$server.Dispose()}}catch{$cleanupErrors+=@($_.Exception.Message)}
 $result.owned_processes_stopped=($cleanupErrors.Count -eq 0)
 if($cleanupErrors.Count){$result.cleanup_errors=$cleanupErrors}
 Save
 if($cleanupErrors.Count){throw ('Owned fixture cleanup identity checks failed: '+($cleanupErrors -join '; '))}
}
Write-Output $output
