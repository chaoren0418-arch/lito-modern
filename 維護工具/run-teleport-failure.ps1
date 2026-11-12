#Requires -Version 7.0
[CmdletBinding()]
param(
 [Parameter(Mandatory)][string]$TestBinary,
 [Parameter(Mandatory)][string]$ReferenceJar,
 [ValidateSet('timeout','cancel-expiry','cancel-expiry-source','cancel-retry','cancel-repeat','refused','refused-cancel-repeat','wrong-service')][string]$Scenario='timeout',
 [switch]$WhatIf
)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
$taskRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$project=Join-Path $taskRoot 'LiTo381-Modern'
$target=[IO.Path]::GetFullPath((Join-Path $project 'target/current'))
$allowed=$target+[IO.Path]::DirectorySeparatorChar
$TestBinary=[IO.Path]::GetFullPath($TestBinary)
$ReferenceJar=[IO.Path]::GetFullPath($ReferenceJar)
foreach($path in @($TestBinary,$ReferenceJar)){
 if(!$path.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw 'Only owned artifacts under this worktree target/current are accepted'}
}
$e149='E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
$whitelist=@($e149,'BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43','9719F45DB324206617726CE06D1A9E2184E9A3B3AEFAB3D50F3EBBDFBF09CFAC','6928A147DF1E784304B9080632C56121B173BF60B172023EBD263E82FD448628','76EBE06D89DBD19CDB577E229657883C16156C77A23EDE288880C25584D2E3CC','C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539')
$jarSha=(Get-FileHash -LiteralPath $ReferenceJar -Algorithm SHA256).Hash
$stamp=[DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffZ')+'-'+[guid]::NewGuid().ToString('N')
$runs=Join-Path $target 'native-teleport-failure'
$leasePath=Join-Path $runs 'runner.lease'
$root=Join-Path $runs $stamp
$runtime=Join-Path $root 'runtime'
$classes=Join-Path $root 'classes'
$tmp=Join-Path $root 'tmp'
$immutable=Join-Path $root 'immutable-test.exe'
$output=Join-Path $target ('verification/native-teleport-failure/'+$stamp)
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
$javac=Join-Path $jdk 'javac.exe';$java=Join-Path $jdk 'java.exe'
$sources=@((Join-Path $project 'tests/oracle/NativePeerHarness.java'),(Join-Path $project 'tests/oracle/NativeTeleportFailureHarness.java'))
$javacArgs=@(('-J-Djava.io.tmpdir='+$tmp),'-J-Dfile.encoding=UTF-8','-encoding','UTF-8','-cp',$ReferenceJar,'-d',$classes)+$sources
$javaArgs=@(('-Djava.io.tmpdir='+$tmp),'-Dfile.encoding=UTF-8','-Xverify:all','-cp',($classes+';'+$ReferenceJar),'local.lito381.server.NativeTeleportFailureHarness',$runtime,$Scenario)
$testNames=@{
 'timeout'='destination_timeout_retires_old_authority_and_allows_fresh_login'
 'cancel-expiry'='cancellation_retires_destination_and_expired_handoff_requires_fresh_selection'
 'cancel-expiry-source'='expired_stage_and_source_handoff_require_the_requested_source_directory'
 'cancel-retry'='cancellation_then_immediate_retry_consumes_staged_character_once'
 'cancel-repeat'='repeated_cancellation_preserves_only_one_authenticated_recovery'
 'refused'='refused_destination_retires_authority_and_allows_fresh_login'
 'refused-cancel-repeat'='failed_handoff_then_repeated_cancel_preserves_the_authenticated_recovery'
 'wrong-service'='wrong_service_destination_retires_authority_and_allows_fresh_login'
}
# Both refusal scenarios use a previously owned, closed loopback destination.
$refusedDestination=$Scenario -in @('refused','refused-cancel-repeat')
$testArgs=@($testNames[$Scenario],'--exact','--ignored','--nocapture')
$bindingArgs=@('recovery_registry::source_binding_matches_compiled_checkout','--exact','--nocapture')
$removedEnvironment=@('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')
function Format-Command([string]$Executable,[string[]]$Arguments){
 '& '+"'"+$Executable.Replace("'","''")+"' "+(($Arguments|ForEach-Object {"'"+$_.Replace("'","''")+"'"}) -join ' ')
}
# Read-only preview: no directory, lease, snapshot or process is created.
if($WhatIf){
 [ordered]@{what_if=$true;repo=$taskRoot;target=$target;test_binary=$TestBinary;reference_jar=$ReferenceJar;jar_sha256=$jarSha;whitelisted=($jarSha -in $whitelist);e149=($jarSha -eq $e149);lease=$leasePath;run=$root;runtime=$runtime;classes=$classes;tmp=$tmp;immutable=$immutable;receipt=(Join-Path $root 'receipt.json');output=$output;working_directory=$root;environment=@{TEMP=$tmp;TMP=$tmp;LITO_TELEPORT_ROOT=$runtime;LITO_TELEPORT_SCENARIO=$Scenario;LITO_TELEPORT_REFERENCE_SHA=$jarSha.ToLowerInvariant()};removed_environment=$removedEnvironment;commands=@((Format-Command $immutable $bindingArgs),(Format-Command $javac $javacArgs),(Format-Command $java $javaArgs),(Format-Command $immutable $testArgs))}|ConvertTo-Json -Depth 6
 return
}
if($jarSha -notin $whitelist){throw 'Review candidate drift: JAR is not whitelisted'}
# Historical pins remain documented, but this owned harness requires E149.
if($jarSha -ne $e149){throw 'This runner requires the reviewed E149 reference'}

function Assert-OwnedPath([string]$Path){
 $cursor=[IO.Path]::GetFullPath($Path)
 if($cursor -ne $target -and !$cursor.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw "Outside target/current: $cursor"}
 while($true){
  $item=Get-Item -LiteralPath $cursor -Force
  if($item.Attributes -band [IO.FileAttributes]::ReparsePoint){throw "ReparsePoint rejected: $cursor"}
  if($cursor -eq $target){break}
  $cursor=Split-Path -Parent $cursor
 }
}
function New-OwnedDirectory([string]$Path){
 if(!(Test-Path -LiteralPath $Path)){
  Assert-OwnedPath (Split-Path -Parent $Path)
  $null=New-Item -ItemType Directory -Path $Path
 }
 Assert-OwnedPath $Path
}
function Get-Pin([string]$Path){
 if(Test-Path -LiteralPath $Path -PathType Leaf){return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash}
 if(Test-Path -LiteralPath $Path){throw "Expected file: $Path"}
 return 'absent'
}
function Get-ClassPins {
 $pins=@{}
 foreach($file in Get-ChildItem -LiteralPath $classes -Filter '*.class' -Recurse -File){
  Assert-OwnedPath $file.FullName
  $pins[[IO.Path]::GetRelativePath($classes,$file.FullName)]=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
 }
 return $pins
}
function Save-Receipt {
 $json=$receipt|ConvertTo-Json -Depth 14
 [IO.File]::WriteAllText((Join-Path $root 'receipt.json'),$json,[Text.UTF8Encoding]::new($false))
 if(Test-Path -LiteralPath $output -PathType Container){[IO.File]::WriteAllText((Join-Path $output 'receipt.json'),$json,[Text.UTF8Encoding]::new($false))}
}
function Initialize-ProcessImageApi {
 if($null -eq ('TeleportProcessImage' -as [type])){
  Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;
public static class TeleportProcessImage {
 [DllImport("kernel32.dll", EntryPoint = "QueryFullProcessImageNameW", CharSet = CharSet.Unicode, ExactSpelling = true, SetLastError = true)]
 [return: MarshalAs(UnmanagedType.Bool)]
 private static extern bool QueryFullProcessImageName(SafeProcessHandle process, uint flags, StringBuilder name, ref uint size);
 public static string Read(SafeProcessHandle process) {
  var name = new StringBuilder(32768);
  uint size = (uint)name.Capacity;
  if (!QueryFullProcessImageName(process, 0, name, ref size))
   throw new Win32Exception(Marshal.GetLastWin32Error());
  return name.ToString();
 }
}
'@
 }
}
function Read-ProcessImage([Microsoft.Win32.SafeHandles.SafeProcessHandle]$Handle){
 return [TeleportProcessImage]::Read($Handle)
}
function Get-OwnedProcessImage($Owned,[int]$TimeoutMilliseconds=5000){
 $watch=[Diagnostics.Stopwatch]::StartNew()
 $lastError='No image returned'
 try {
  do {
   $Owned.entry.image_query_attempts++
   try {
    $image=Read-ProcessImage $Owned.handle
    if(![string]::IsNullOrWhiteSpace($image)){return $image}
    $lastError='Empty image returned'
   } catch {$lastError=$_.Exception.Message}
   $remaining=$TimeoutMilliseconds-$watch.ElapsedMilliseconds
   if($remaining -le 0){break}
   Start-Sleep -Milliseconds ([int][Math]::Min(100,$remaining))
  } while($watch.ElapsedMilliseconds -lt $TimeoutMilliseconds)
  throw "Blocked: process image unavailable within ${TimeoutMilliseconds}ms: $lastError"
 } finally {$Owned.entry.image_query_elapsed_ms=$watch.ElapsedMilliseconds}
}
function Assert-BoundIdentity($Owned){
 $entry=$Owned.entry
 if(!$entry.identity_bound -or $null -eq $entry.pid -or [string]::IsNullOrWhiteSpace($entry.start_utc) -or [string]::IsNullOrWhiteSpace($entry.image) -or ![string]::Equals($entry.image,$entry.executable,[StringComparison]::OrdinalIgnoreCase)){
  throw "Blocked: $($entry.kind) PID/start time/image identity is not bound"
 }
}
function Start-OwnedProcess([string]$Kind,[string]$Executable,[string[]]$Arguments,[hashtable]$ExtraEnvironment=@{}){
 # Compile interop before Start(): cold Add-Type must not consume the image deadline.
 Initialize-ProcessImageApi
 $info=[Diagnostics.ProcessStartInfo]::new($Executable)
 $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.WorkingDirectory=$root
 $info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 $info.StandardOutputEncoding=[Text.Encoding]::UTF8;$info.StandardErrorEncoding=[Text.Encoding]::UTF8
 foreach($argument in $Arguments){$info.ArgumentList.Add($argument)}
 $info.Environment['TEMP']=$tmp;$info.Environment['TMP']=$tmp
 foreach($name in $removedEnvironment){$null=$info.Environment.Remove($name)}
 foreach($name in $ExtraEnvironment.Keys){$info.Environment[$name]=$ExtraEnvironment[$name]}
 # All actual names, only controlled values: do not serialize inherited secrets.
 $entry=@{kind=$Kind;executable=$Executable;arguments=$Arguments;command=(Format-Command $Executable $Arguments);working_directory=$root;environment_names=@($info.Environment.Keys|Sort-Object);environment_overrides=(@{TEMP=$tmp;TMP=$tmp}+$ExtraEnvironment);environment_removed=$removedEnvironment;pid=$null;start_utc=$null;image=$null;identity_bound=$false;image_query_attempts=0;image_query_elapsed_ms=$null;identity_error=$null;cleanup_method=$null;exit_code=$null;killed=$false;wait_for_exit=$false;cleanup_errors=@()}
 $receipt.processes+=@($entry)
 $process=[Diagnostics.Process]::new();$process.StartInfo=$info
 if(!$process.Start()){throw "Could not start $Kind"}
 # Register before reading identity, so any later failure still reaches finally.
 $owned=@{process=$process;handle=$null;entry=$entry;stdout=$null;stderr=$null;log_files=@{}}
 $processes.Add($owned)
 $owned.handle=$process.SafeHandle
 $entry.pid=$process.Id
 foreach($stream in @('stdout','stderr')){
  # Disable FileStream buffering so partial output survives cleanup failures.
  $file=[IO.FileStream]::new((Join-Path $root ($Kind+'.'+$stream+'.txt')),[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read,1,$true)
  $owned.log_files[$stream]=$file
  $reader=if($stream -eq 'stdout'){$process.StandardOutput}else{$process.StandardError}
  $owned[$stream]=$reader.BaseStream.CopyToAsync($file)
 }
 try {
  $entry.start_utc=$process.StartTime.ToUniversalTime().ToString('o')
  $entry.image=Get-OwnedProcessImage $owned
  if(![string]::Equals($entry.image,$Executable,[StringComparison]::OrdinalIgnoreCase)){throw "Blocked: $Kind image mismatch"}
  $entry.identity_bound=$true
  Assert-BoundIdentity $owned
 } catch {$entry.identity_error=$_.Exception.Message;throw}
 return $owned
}
function Complete-OwnedLog($Owned,[string]$Stream){
 if($null -ne $Owned[$Stream]){
  if(!$Owned[$Stream].Wait(5000)){throw "$Stream drain timed out"}
  $null=$Owned[$Stream].GetAwaiter().GetResult()
 }
 if($null -ne $Owned.log_files[$Stream]){
  # CopyToAsync does not close its destination. Close the writer before any
  # ReadAllText (FileShare.Read); finally may safely call this again.
  $Owned.log_files[$Stream].Dispose()
  $Owned.log_files[$Stream]=$null
 }
}
function Stop-OwnedProcess($Owned){
 $process=$Owned.process;$entry=$Owned.entry
 try {
  if(!$process.HasExited){
   if(!$entry.identity_bound){
    # Only the Process returned by our Start(), with its retained handle.
    # Never re-open a PID to clean up an unbound identity.
    if($null -eq $Owned.handle -or $Owned.handle.IsInvalid -or $Owned.handle.IsClosed){throw 'Original process handle unavailable: kill refused'}
    $entry.cleanup_method=if([string]::IsNullOrWhiteSpace($entry.image)){'original_handle_unbound_image'}else{'original_handle_unbound_identity'}
    $process.Kill();$entry.killed=$true
   } else {
    $entry.cleanup_method='original_handle_verified_identity'
    $live=[Diagnostics.Process]::GetProcessById($entry.pid)
    try {
     if(!$live.HasExited){
      if($live.Id -ne $entry.pid -or $live.StartTime.ToUniversalTime().ToString('o') -ne $entry.start_utc -or ![string]::Equals($live.MainModule.FileName,$entry.image,[StringComparison]::OrdinalIgnoreCase)){throw 'PID/start time/image mismatch: kill refused'}
      # Kill via the retained original handle, never by a bare PID.
      $process.Kill();$entry.killed=$true
     }
    } finally {$live.Dispose()}
   }
  }
  $entry.wait_for_exit=$process.WaitForExit(5000)
  if(!$entry.wait_for_exit){throw 'WaitForExit timed out after cleanup'}
  $entry.exit_code=$process.ExitCode
 } catch {
  # A natural exit racing identity lookup is provable via the original handle.
  if($process.HasExited -and $process.WaitForExit(5000)){$entry.wait_for_exit=$true;$entry.exit_code=$process.ExitCode}
  else {$entry.cleanup_errors+=@($_.Exception.Message);throw}
 } finally {
  foreach($stream in @('stdout','stderr')){
   try {
    Complete-OwnedLog $Owned $stream
   } catch {$entry.cleanup_errors+=@($_.Exception.Message)}
   # Copy independently even after a drain error; the run retains partial logs.
   try {
    $logPath=Join-Path $root ($entry.kind+'.'+$stream+'.txt')
    if(Test-Path -LiteralPath $logPath){Copy-Item -LiteralPath $logPath -Destination (Join-Path $output ($entry.kind+'.'+$stream+'.txt'))}
   } catch {$entry.cleanup_errors+=@($_.Exception.Message)}
  }
  if($entry.wait_for_exit){$process.Dispose()}
 }
}
function Assert-Ready($Ready,$Server){
 if($Ready.scenario -cne $Scenario -or $Ready.requested_source -cne '測試乙' -or $Ready.default_source -cne '測試甲'){throw 'ready source/scenario binding mismatch'}
 if($Ready.jar_sha256 -ne $jarSha -or $Ready.jar_sha256 -ne $e149){throw 'ready JAR binding mismatch'}
 if($Server.process.HasExited -or $Ready.pid -ne $Server.entry.pid){throw 'ready PID binding mismatch'}
 if($Ready.login_host -cne '127.0.0.1' -or $Ready.town_host -cne '127.0.0.2' -or $Ready.destination_host -cne $(if($refusedDestination){'127.0.0.1'}else{'127.0.0.3'})){throw 'Unexpected ready loopback address'}
 if($Ready.destination_refused -ne $refusedDestination -or $Ready.destination_wrong_service -ne ($Scenario -eq 'wrong-service')){throw 'ready scenario mismatch'}
 foreach($endpoint in @(@{host=$Ready.login_host;port=$Ready.port;name='login'},@{host=$Ready.town_host;port=$Ready.town_port;name='town'},@{host=$Ready.destination_host;port=$Ready.destination_port;name='destination'})){
  $port=$endpoint.port
  if($port -isnot [long] -and $port -isnot [int]){throw 'Non-integer listener port'}
  if($port -lt 1024 -or $port -gt 65535 -or $port -in @(2000,3306)){throw "Forbidden listener port: $port"}
  $absent=$endpoint.name -eq 'destination' -and $refusedDestination
  if($absent){
   # Enumerate successfully before filtering, so an absent destination is not
   # confused with a failed exact query. Wildcard binds also violate refusal.
   $listeners=@(Get-NetTCPConnection -State Listen -ErrorAction Stop|Where-Object {$_.LocalPort -eq $port -and $_.LocalAddress -in @($endpoint.host,'0.0.0.0','::')})
  } else {
   $listeners=@(Get-NetTCPConnection -State Listen -LocalAddress $endpoint.host -LocalPort $port -ErrorAction Stop)
  }
  if($absent){if($listeners.Count -ne 0){throw 'Refused destination has a listener'}}
  elseif($listeners.Count -ne 1 -or $listeners[0].OwningProcess -ne $Server.entry.pid -or $listeners[0].OwningProcess -ne $Ready.pid){throw "Listener ownership mismatch: $($endpoint.name)"}
  $receipt.listeners+=@(@{name=$endpoint.name;host=$endpoint.host;port=$port;expected_absent=$absent;owners=@($listeners|ForEach-Object OwningProcess);checked_utc=[DateTime]::UtcNow.ToString('o')})
 }
}

# Only the lease container is created before acquiring the atomic lease.
Assert-OwnedPath $target
foreach($path in @($TestBinary,$ReferenceJar)){Assert-OwnedPath $path}
New-OwnedDirectory $runs
$lease=$null
try {$lease=[IO.File]::Open($leasePath,[IO.FileMode]::CreateNew,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)}
catch {throw "Cannot acquire lease $leasePath. Inspect its owning runner and receipt; existing leases are never removed automatically. $($_.Exception.Message)"}
$receipt=@{status='running';scenario=$Scenario;root=$root;runtime=$runtime;output=$output;jar_path=$ReferenceJar;jar_sha256=$jarSha;processes=@();listeners=@();cleanup_errors=@();drift_errors=@();source_pins=@{};class_pins=@{};formal_deployed=$false}
$processes=[Collections.Generic.List[object]]::new()
$rootCreated=$false;$classesRecorded=$false;$failure=$null
$leaseSeconds=if($Scenario -eq 'cancel-expiry-source'){460}elseif($Scenario -eq 'cancel-expiry'){340}else{220}
try {
 $runner=[Diagnostics.Process]::GetCurrentProcess()
 $leaseData=@{pid=$PID;start_utc=$runner.StartTime.ToUniversalTime().ToString('o');root=$root}|ConvertTo-Json
 $bytes=[Text.Encoding]::UTF8.GetBytes($leaseData);$lease.Write($bytes,0,$bytes.Length);$lease.Flush($true)
 $receipt.lease_owner=$leaseData|ConvertFrom-Json
 Assert-OwnedPath $runs
 $null=New-Item -ItemType Directory -Path $root
 $rootCreated=$true;Assert-OwnedPath $root
 foreach($path in @($classes,$tmp)){$null=New-Item -ItemType Directory -Path $path;Assert-OwnedPath $path}
 if(Test-Path -LiteralPath $runtime){throw 'runtime must not exist before Java starts'}
 New-OwnedDirectory (Join-Path $target 'verification')
 New-OwnedDirectory (Join-Path $target 'verification/native-teleport-failure')
 $null=New-Item -ItemType Directory -Path $output;Assert-OwnedPath $output
 $receipt.test_binary=$TestBinary;$receipt.test_binary_sha256=Get-Pin $TestBinary
 [IO.File]::Copy($TestBinary,$immutable,$false)
 Assert-OwnedPath $immutable
 if((Get-Pin $immutable) -ne $receipt.test_binary_sha256){throw 'Immutable binary differs'}
 $receipt.immutable_test_binary=$immutable
 foreach($name in @('LiTo381-Modern/Cargo.toml','LiTo381-Modern/Cargo.lock','LiTo381-Modern/.cargo/config.toml','LiTo381-Modern/tests/oracle/NativePeerHarness.java','LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java','維護工具/run-teleport-failure.ps1','LiTo381-Modern/tests/native_teleport_failure.rs','LiTo381-Modern/src/modern_lito/native_services.rs','LiTo381-Modern/src/modern_lito/native_teleport.rs','LiTo381-Modern/src/modern_lito/native_login.rs','LiTo381-Modern/src/modern_lito/native_connection.rs')){
  $receipt.source_pins[$name]=Get-Pin (Join-Path $taskRoot $name)
 }
 Save-Receipt
 # A binary built from another checkout or an older Java/runner revision must
 # fail before compilation/server startup. The registry comes from src, not
 # an untracked historical overlay or a previous linked-image qualification.
 $bindingCheck=Start-OwnedProcess 'source-registry' $immutable $bindingArgs
 if(!$bindingCheck.process.WaitForExit(60000)){throw 'Source registry check timeout'}
 Complete-OwnedLog $bindingCheck 'stdout'
 $bindingText=[IO.File]::ReadAllText((Join-Path $root 'source-registry.stdout.txt'),[Text.Encoding]::UTF8)
 if($bindingCheck.process.ExitCode -ne 0 -or $bindingText -notmatch 'running 1 test\b' -or $bindingText -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw 'Current source/fixture registry check failed'}
 $bindings=[regex]::Matches($bindingText,'(?m)LITO_TELEPORT_BINDING=(\{[^\r\n]+\})')
 if($bindings.Count -ne 1){throw 'Missing/ambiguous current source binding'}
 $binding=$bindings[0].Groups[1].Value|ConvertFrom-Json
 if($binding.schema -cne 'current-src-teleport-binding-v1' -or $binding.registry.Count -ne 13 -or $binding.fixture.Count -ne 14 -or $binding.scope -cne 'T023-1-current-controller-memory-fixture' -or $binding.historical_overlay_applied -ne $false -or $binding.historical_qualification_reused -ne $false){throw 'Unexpected source registry scope'}
 foreach($pin in $binding.source_pins.PSObject.Properties){
  $path=[IO.Path]::GetFullPath((Join-Path $project $pin.Name))
  if(!$path.StartsWith($taskRoot+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Registry path outside current worktree'}
  if((Get-Pin $path) -ne $pin.Value){throw "Registry source drift: $($pin.Name)"}
  $receipt.source_pins[[IO.Path]::GetRelativePath($taskRoot,$path)]=$pin.Value
 }
 $receipt.source_registry=$binding
 Save-Receipt
 $compiler=Start-OwnedProcess 'javac' $javac $javacArgs
 if(!$compiler.process.WaitForExit(60000)){throw 'Harness compile timeout'}
 if($compiler.process.ExitCode -ne 0){throw 'Harness compile failed'}
 $receipt.class_pins=Get-ClassPins;$classesRecorded=$true
 if($receipt.class_pins.Count -eq 0){throw 'No harness classes produced'}
 Save-Receipt
 if(Test-Path -LiteralPath $runtime){throw 'runtime must not exist before Java starts'}
 $server=Start-OwnedProcess 'java' $java $javaArgs
 $readyPath=Join-Path $runtime 'ready.json'
 $end=[DateTime]::UtcNow.AddSeconds(15)
 while(!(Test-Path -LiteralPath $readyPath)){
  if($server.process.HasExited -or [DateTime]::UtcNow -ge $end){throw 'Server not ready'}
  Start-Sleep -Milliseconds 50
 }
 Assert-OwnedPath $readyPath
 # File creation may precede its final write; retry parsing while Java lives.
 while($true){
  try {$ready=Get-Content -LiteralPath $readyPath -Raw -Encoding utf8|ConvertFrom-Json;break}
  catch {if($server.process.HasExited -or [DateTime]::UtcNow -ge $end){throw};Start-Sleep -Milliseconds 50}
 }
 Assert-Ready $ready $server
 $receipt.ready=$ready
 if((Get-Pin $ReferenceJar) -ne $jarSha){throw 'JAR changed before Rust startup'}
 Save-Receipt
 # Startup may proceed only after javac and Java each bind all three fields.
 foreach($owned in @($compiler,$server)){Assert-BoundIdentity $owned}
 $client=Start-OwnedProcess 'rust' $immutable $testArgs @{LITO_TELEPORT_SCENARIO=$Scenario;LITO_TELEPORT_ROOT=$runtime;LITO_TELEPORT_REFERENCE_SHA=$jarSha.ToLowerInvariant()}
 $receipt.lease_deadline_utc=[DateTime]::UtcNow.AddSeconds($leaseSeconds).ToString('o');Save-Receipt
 if(!$client.process.WaitForExit($leaseSeconds*1000)){throw 'Teleport handoff test timeout'}
 if(!$server.process.WaitForExit(8000)){throw 'Server cleanup did not complete'}
 if($client.process.ExitCode -ne 0 -or $server.process.ExitCode -ne 0){throw 'Teleport handoff test or cleanup failed'}
 Complete-OwnedLog $client 'stdout'
 $testText=[IO.File]::ReadAllText((Join-Path $root 'rust.stdout.txt'),[Text.Encoding]::UTF8)
 if($testText -notmatch 'running 1 test\b' -or $testText -notmatch 'test result: ok\. 1 passed; 0 failed; 0 ignored;'){throw 'Expected exactly one executed passing Rust test'}
 $resultPath=Join-Path $runtime 'result.json';Assert-OwnedPath $resultPath
 $result=Get-Content -LiteralPath $resultPath -Raw -Encoding utf8|ConvertFrom-Json
 if($result.status -ne 'sessions_closed' -or $result.memberships_empty -ne $true -or $result.actual_client_sessions -ne $(if($Scenario -eq 'wrong-service'){10}else{8}) -or $result.authority_mutations -ne 0 -or $result.staged_loadouts_after_reentry -ne 0 -or $result.sql_access -ne $false -or $result.state_persistent -ne $false -or $result.full_game_reentry_verified -ne $true -or $result.jar_sha256 -ne $jarSha){throw 'Isolation/cleanup result mismatch'}
 $receipt.server=$result;$receipt.status='verified_'+$Scenario
 if($Scenario -eq 'cancel-expiry-source'){
  foreach($role in @('a','b')){
   $player=Get-Content -LiteralPath (Join-Path $runtime ($role+'-failure.json')) -Raw -Encoding utf8|ConvertFrom-Json
   if($player.scenario -cne $Scenario -or $player.requested_source -cne '測試乙' -or $player.expiry_wait_ms -lt 33000 -or $player.source_wait_ms -lt 31000 -or $player.manual_selection -ne $true -or $player.full_game_reentry_verified -ne $true){throw 'Natural stage/source expiry evidence missing'}
  }
 }
 if($Scenario -in @('cancel-repeat','refused-cancel-repeat')){
  $minimumCalls=if($Scenario -eq 'cancel-repeat'){6}else{2}
  foreach($role in @('a','b')){
   $player=Get-Content -LiteralPath (Join-Path $runtime ($role+'-failure.json')) -Raw -Encoding utf8|ConvertFrom-Json
   if($player.scenario -cne $Scenario -or $player.requested_source -cne '測試乙' -or $player.cancellation_calls -lt $minimumCalls -or $player.full_game_reentry_verified -ne $true){throw 'Repeated cancellation evidence missing'}
  }
 }
} catch {$failure=$_.Exception.Message;$receipt.status=if(@($receipt.processes|Where-Object {$null -ne $_.identity_error}).Count){'blocked'}else{'failed'};$receipt.error=$failure}
finally {
 # Attempt each owned process independently, including failed compilation.
 foreach($kind in @('rust','java','javac','source-registry')){
  foreach($owned in @($processes|Where-Object {$_.entry.kind -eq $kind})){
   try {Stop-OwnedProcess $owned} catch {$receipt.cleanup_errors+=@($kind+': '+$_.Exception.Message)}
   $receipt.cleanup_errors+=@($owned.entry.cleanup_errors|ForEach-Object {$kind+': '+$_})
  }
 }
 foreach($name in $receipt.source_pins.Keys){
  try {if((Get-Pin (Join-Path $taskRoot $name)) -ne $receipt.source_pins[$name]){throw "Source drift: $name"}}
  catch {$receipt.drift_errors+=@($_.Exception.Message)}
 }
 if($classesRecorded){
  try {
   $after=Get-ClassPins
   if($after.Count -ne $receipt.class_pins.Count){throw 'Class inventory drift'}
   foreach($name in $receipt.class_pins.Keys){if(!$after.ContainsKey($name) -or $after[$name] -ne $receipt.class_pins[$name]){throw "Class drift: $name"}}
  } catch {$receipt.drift_errors+=@($_.Exception.Message)}
 }
 foreach($path in @($ReferenceJar,$TestBinary,$immutable)){
  try {
   $expected=if($path -eq $ReferenceJar){$jarSha}elseif($receipt.ContainsKey('test_binary_sha256')){$receipt.test_binary_sha256}else{'absent'}
   if((Get-Pin $path) -ne $expected){throw "Binary drift: $path"}
  } catch {$receipt.drift_errors+=@($_.Exception.Message)}
 }
 if($receipt.cleanup_errors.Count -or $receipt.drift_errors.Count){$receipt.status='failed';if(!$failure){$failure='Cleanup or input drift failed; inspect receipt'}}
 $receipt.inputs_unchanged=($receipt.drift_errors.Count -eq 0)
 try {
  if($rootCreated){
   foreach($name in @('ready.json','result.json','server-trace.jsonl','staged-trace.jsonl','a-failure.json','b-failure.json')){
    try {if(Test-Path -LiteralPath (Join-Path $runtime $name)){Assert-OwnedPath (Join-Path $runtime $name);Copy-Item -LiteralPath (Join-Path $runtime $name) -Destination (Join-Path $output $name)}}
    catch {$receipt.status='failed';$receipt.cleanup_errors+=@($_.Exception.Message);$failure='Evidence copy failed'}
   }
   Save-Receipt
  }
 } finally {
  # Only this successful CreateNew owner closes and removes this lease.
  $lease.Dispose()
  Remove-Item -LiteralPath $leasePath -ErrorAction Stop
 }
}
if($failure){throw $failure}
Write-Output $output
