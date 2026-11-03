#Requires -Version 7.0
param([ValidateSet('Debug','Release')][string]$BuildProfile='Release',[ValidateSet('probe','expiry')][string]$Scenario='probe',[ValidateSet('lito-legacy','tw13081901-v1')][string]$ResourceProfile='tw13081901-v1',[Parameter(Mandatory)][string]$NormalQualification)
$ReplayStaleCommands=[switch]$false
# Only owned native windows on the unswitched test desktop receive messages.
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot;$project=Join-Path $taskRoot 'LiTo381-Modern'
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $project ('target/current/native-light-lease-gui/'+$stamp)
$out=Join-Path $taskRoot ('驗證紀錄/current/native-light-lease-gui/'+$stamp)
$classes=Join-Path $root 'oracle';New-Item -ItemType Directory -Path $classes,$out -Force|Out-Null
$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP;$env:PYTHONDONTWRITEBYTECODE='1'
$jar=Join-Path $project 'target/current/server-reference/formal-server-C02AAAE8.jar'
$source='D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar'
$jarSha='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
if((Get-FileHash -LiteralPath $jar).Hash -ne $jarSha){throw 'Owned reference drift'}
$observedSourceBefore=(Get-FileHash -LiteralPath $source).Hash
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin'
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System;using System.Text;using System.Runtime.InteropServices;
public static class OwnedTeleportWindow {
 [StructLayout(LayoutKind.Sequential)]public struct GuiInfo{public int size,flags;public IntPtr active,focus,capture,menu,move,caret;public Rect caretRect;}
 [DllImport("user32.dll")]static extern bool GetGUIThreadInfo(uint thread,ref GuiInfo info);
 public static IntPtr FocusFor(IntPtr h){uint pid;uint tid=GetWindowThreadProcessId(h,out pid);var info=new GuiInfo{size=Marshal.SizeOf<GuiInfo>()};if(!GetGUIThreadInfo(tid,ref info))throw new Exception("Owned GUI thread info failed");return info.focus;}
 public delegate bool EnumProc(IntPtr h,IntPtr p);
 [DllImport("user32.dll")]static extern bool EnumWindows(EnumProc f,IntPtr p);
 [DllImport("user32.dll")]public static extern uint GetWindowThreadProcessId(IntPtr h,out uint p);
 [DllImport("user32.dll")]public static extern IntPtr GetDlgItem(IntPtr h,int id);
 [DllImport("user32.dll")]public static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")]public static extern bool IsWindowEnabled(IntPtr h);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]static extern int GetWindowText(IntPtr h,StringBuilder s,int max);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]static extern IntPtr SendMessageTimeout(IntPtr h,uint m,IntPtr w,IntPtr l,uint f,uint t,out IntPtr r);
 [DllImport("user32.dll",CharSet=CharSet.Unicode,EntryPoint="SendMessageTimeoutW")]static extern IntPtr SendText(IntPtr h,uint m,IntPtr w,string l,uint f,uint t,out IntPtr r);
 [DllImport("user32.dll",CharSet=CharSet.Unicode,EntryPoint="SendMessageTimeoutW")]static extern IntPtr ReadText(IntPtr h,uint m,IntPtr w,StringBuilder l,uint f,uint t,out IntPtr r);
 [DllImport("user32.dll")]public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")]public static extern bool RedrawWindow(IntPtr h,IntPtr rect,IntPtr region,uint flags);
 [DllImport("user32.dll")]public static extern bool PrintWindow(IntPtr h,IntPtr dc,uint flags);
 [DllImport("user32.dll")]public static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("user32.dll")]public static extern bool GetClientRect(IntPtr h,out Rect r);
 [DllImport("user32.dll")]public static extern uint GetDpiForWindow(IntPtr h);
 [DllImport("user32.dll")]static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
 public static Rect PhysicalRect(IntPtr h){var prior=SetThreadDpiAwarenessContext((IntPtr)(-4));try{Rect r;if(!GetWindowRect(h,out r))throw new Exception("Window rect failed");return r;}finally{SetThreadDpiAwarenessContext(prior);}}
 public static Rect PhysicalClientRect(IntPtr h){var prior=SetThreadDpiAwarenessContext((IntPtr)(-4));try{Rect r;if(!GetClientRect(h,out r))throw new Exception("Client rect failed");return r;}finally{SetThreadDpiAwarenessContext(prior);}}
 public static long SendPhysical(IntPtr h,uint m,long w,long l){var prior=SetThreadDpiAwarenessContext((IntPtr)(-4));try{return Send(h,m,w,l);}finally{SetThreadDpiAwarenessContext(prior);}}
 [StructLayout(LayoutKind.Sequential)]public struct Rect {public int Left,Top,Right,Bottom;}
 public static IntPtr Find(int pid){IntPtr found=IntPtr.Zero;EnumWindows((h,p)=>{uint id;GetWindowThreadProcessId(h,out id);if(id==pid && GetDlgItem(h,101)!=IntPtr.Zero){found=h;return false;}return true;},IntPtr.Zero);return found;}
 public static string Text(IntPtr h){var s=new StringBuilder(4096);IntPtr r;if(ReadText(h,13,(IntPtr)s.Capacity,s,2,2000,out r)==IntPtr.Zero)throw new Exception("Control text timeout");return s.ToString();}
 public static long Send(IntPtr h,uint m,long w,long l){IntPtr r;if(SendMessageTimeout(h,m,(IntPtr)w,(IntPtr)l,2,2000,out r)==IntPtr.Zero)throw new Exception("Window message timeout");return r.ToInt64();}
 public static void SetText(IntPtr h,string s){IntPtr r;if(SendText(h,12,IntPtr.Zero,s,2,2000,out r)==IntPtr.Zero)throw new Exception("Text message timeout");}
 public static string ComboRow(IntPtr h,int index){var s=new StringBuilder(1024);IntPtr r;if(ReadText(h,0x148,(IntPtr)index,s,2,2000,out r)==IntPtr.Zero)throw new Exception("Combo read timeout");return s.ToString();}
 public static string Row(IntPtr h,int index){var s=new StringBuilder(1024);IntPtr r;if(ReadText(h,0x189,(IntPtr)index,s,2,2000,out r)==IntPtr.Zero)throw new Exception("List read timeout");return s.ToString();}
}
"@
$clients=@();$server=$null;$steps=@();$ActionProfile='teleport'
if($ReplayStaleCommands -and $Scenario -ne 'cancel-retry'){throw 'Stale-command replay currently requires the cancel-retry lifecycle'}
$result=[ordered]@{status='running';replay_stale_commands=$ReplayStaleCommands.IsPresent;stale_command_replays=@();scenario=$Scenario;resource_profile=$ResourceProfile;extended_item_details=($ResourceProfile -eq 'tw13081901-v1');runtime=$root;output=$out;jar_sha256=$jarSha;exe_sha256=(Get-FileHash -LiteralPath $exe).Hash;steps=@();physical_input_verified=$false;private_desktop=$true;product_complete=$false}
function Save {$result.steps=$script:steps;$result|ConvertTo-Json -Depth 20|Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding utf8}
function WaitFor([scriptblock]$Condition,[string]$Reason){$until=[DateTime]::UtcNow.AddSeconds(35);while(!(& $Condition)){if([DateTime]::UtcNow -gt $until){throw ('Timeout: '+$Reason)};Start-Sleep -Milliseconds 30}}
function C($client,[int]$id){$h=[OwnedTeleportWindow]::GetDlgItem($client.Window,$id);if($h -eq [IntPtr]::Zero){throw ('Missing control '+$id)};[uint32]$owner=0;[void][OwnedTeleportWindow]::GetWindowThreadProcessId($h,[ref]$owner);if($owner -ne $client.Process.Id){throw 'Foreign control'};return $h}
function Enabled($client,[int]$id){return [OwnedTeleportWindow]::IsWindowEnabled((C $client $id))}
function TextOf($client,[int]$id){return [OwnedTeleportWindow]::Text((C $client $id))}
function SetText($client,[int]$id,[string]$text){[OwnedTeleportWindow]::SetText((C $client $id),$text)}
function Click($client,[int]$id){$h=C $client $id;if(![OwnedTeleportWindow]::IsWindowVisible($h) -or ![OwnedTeleportWindow]::IsWindowEnabled($h)){throw ('Control not available '+$id)};[void][OwnedTeleportWindow]::Send($h,0xf5,0,0)}
function SelectCombo($client,[int]$index){[void][OwnedTeleportWindow]::Send((C $client 203),0x14e,$index,0)}
function SelectRow($client,[string]$match){$h=C $client 205;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0);$found=-1;for($n=0;$n -lt $count;$n++){if([OwnedTeleportWindow]::Row($h,$n).Contains($match)){$found=$n;break}};if($found -lt 0){throw ('List row absent '+$match)};[void][OwnedTeleportWindow]::Send($h,0x186,$found,0);[void][OwnedTeleportWindow]::Send($client.Window,0x111,(205 -bor (1 -shl 16)),$h.ToInt64())}
function SelectComboMatch($client,[int]$id,[string]$match){$h=C $client $id;$count=[OwnedTeleportWindow]::Send($h,0x146,0,0);$labels=@();for($i=0;$i -lt $count;$i++){$label=[OwnedTeleportWindow]::ComboRow($h,$i);$labels+=@($label);if($label.Contains($match)){[void][OwnedTeleportWindow]::Send($h,0x14e,$i,0);return}};throw ('Combo match absent: '+$match+' in '+($labels -join ', '))}
function SelectMerchantRow($client,[string]$match){$h=C $client 221;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0);for($n=0;$n -lt $count;$n++){if([OwnedTeleportWindow]::Row($h,$n).Contains($match)){[void][OwnedTeleportWindow]::Send($h,0x186,$n,0);[void][OwnedTeleportWindow]::Send($client.Window,0x111,(221 -bor (1 -shl 16)),$h.ToInt64());return}};throw ('Merchant row absent '+$match)}
function Snapshot($client,[string]$name){
 # Allow queued child-control paints to follow the already verified state transition.
 Start-Sleep -Milliseconds 180
 [void][OwnedTeleportWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)
 $rect=[OwnedTeleportWindow]::PhysicalRect($client.Window)
 $paintProbe=@();$paintUntil=[DateTime]::UtcNow.AddSeconds(3)
 do {
  $bitmap=[System.Drawing.Bitmap]::new($rect.Right-$rect.Left,$rect.Bottom-$rect.Top);$graphics=[System.Drawing.Graphics]::FromImage($bitmap);$dc=$graphics.GetHdc()
  try{if(![OwnedTeleportWindow]::PrintWindow($client.Window,$dc,2)){throw 'PrintWindow failed'}}finally{$graphics.ReleaseHdc($dc);$graphics.Dispose()}
  $colored=0
  if(($name -ne 'lobby')){
   for($gy=0;$gy -lt 8;$gy++){for($gx=0;$gx -lt 16;$gx++){$x=[int]($bitmap.Width*(0.25+0.5*$gx/15));$y=[int]($bitmap.Height*(0.25+0.25*$gy/7));$pixel=$bitmap.GetPixel($x,$y);$high=[Math]::Max($pixel.R,[Math]::Max($pixel.G,$pixel.B));$low=[Math]::Min($pixel.R,[Math]::Min($pixel.G,$pixel.B));if($high -gt 20 -and $high-$low -gt 15){$colored++}}}
   $paintProbe+=@($colored)
   if($colored -lt 10){$bitmap.Dispose();if([DateTime]::UtcNow -ge $paintUntil){throw 'Ready portal status did not acquire actual colored terrain pixels'};Start-Sleep -Milliseconds 60;[void][OwnedTeleportWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)}
  }
 }while(($name -ne 'lobby') -and $colored -lt 10)

 $path=Join-Path $out ($client.Role+'-'+$name+'.png');try{$bitmap.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)}finally{$bitmap.Dispose()}
 $script:steps+=@{role=$client.Role;step=$name;terrain_paint_probe=$paintProbe;status=(TextOf $client 204);rental_money=$(if($true){TextOf $client 223}else{''});rental_body=$(if($true){TextOf $client 220}else{''});inventory_button=(TextOf $client 114);controls=@{connect=(Enabled $client 101);cancel=(Enabled $client 104);inventory=(Enabled $client 114);npc_visible=[OwnedTeleportWindow]::IsWindowVisible((C $client 220))};png=$path;png_sha256=(Get-FileHash -LiteralPath $path).Hash};Save
}

function StopOwned($process,[string]$marker) {
 if($process.HasExited){return};$live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
 if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$live.CommandLine.Replace('/','\').Replace('\\','\').Contains($marker.Replace('/','\'))){throw 'Owned GUI PID/creation/command mismatch'}
 $process.Kill();$process.WaitForExit(5000)|Out-Null
}
function Launch([string]$role) {
 $before=@(Get-Process LiToModern -ErrorAction SilentlyContinue|Select-Object -ExpandProperty Id)
 & (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildProfile $BuildProfile -NativeLoginEndpoint ('127.0.0.1:'+$ready.port) *> (Join-Path $out ('launch-'+$role+'.log'))
 $discoverUntil=[DateTime]::UtcNow.AddSeconds(5)
 do {
  $owned=@(Get-CimInstance Win32_Process -Filter "Name = 'LiToModern.exe'"|Where-Object {$_.ProcessId -notin $before -and $_.ExecutablePath -and $_.ExecutablePath.Replace('/','\').Replace('\\','\') -eq $exe.Replace('/','\').Replace('\\','\') -and $_.CommandLine.Contains('127.0.0.1:'+$ready.port)})
  if($owned.Count -gt 1){throw 'Multiple owned GUI identities'}
  if($owned.Count -eq 1){break};Start-Sleep -Milliseconds 50
 } while([DateTime]::UtcNow -lt $discoverUntil)
 if($owned.Count -ne 1){throw 'Owned GUI identity missing'}
 $process=Get-Process -Id $owned[0].ProcessId;$client=[pscustomobject]@{Role=$role;Account=$(if($role -eq 'a'){'PeerFixture'}else{'PeerFixture2'});Process=$process;Window=[IntPtr]::Zero};$script:clients+=@($client);$result.owned_clients=@($script:clients|ForEach-Object {@{pid=$_.Process.Id;start_utc=$_.Process.StartTime.ToUniversalTime().ToString("o");role=$_.Role}});Save
 WaitFor {$client.Window=[OwnedTeleportWindow]::Find($process.Id);$client.Window -ne [IntPtr]::Zero} 'window creation';return $client
}
function Authority($client) {
 $trace=Join-Path $root 'server-trace.jsonl';$needle='"test_account":"'+$client.Account+'"'
 $row=Get-Content -LiteralPath $trace -Tail 160|Where-Object {$_.Contains($needle)}|Select-Object -Last 1
 if(!$row){throw 'Current owned authority absent'};return ($row|ConvertFrom-Json).state
}
function Bootstrap($client){
 Click $client 101;WaitFor {Enabled $client 102} 'login';SetText $client 201 $client.Account;SetText $client 202 'TestOnlyPassword';Click $client 102
 WaitFor {Enabled $client 103} 'sources';SelectCombo $client 1;Click $client 103;WaitFor {Enabled $client 105} 'GAME';Click $client 105
 WaitFor {(Enabled $client 106) -or (Enabled $client 103)} 'notice/characters';if(Enabled $client 106){Click $client 106}
 WaitFor {Enabled $client 103} 'characters';SelectCombo $client 0;Click $client 103
 WaitFor {Enabled $client 127} 'original rental selection'
 [void][OwnedTeleportWindow]::Send((C $client 217),0x14e,0,0);Click $client 127;WaitFor {Enabled $client 128} 'rental accepted';Click $client 128
 WaitFor {(Enabled $client 129) -and (TextOf $client 204).StartsWith('地圖 500')} 'original rental town';Snapshot $client 'before-cast'
}

function SelectLight($client){
 SelectComboMatch $client 208 '#2 日光術'
 $h=C $client 208;[void][OwnedTeleportWindow]::Send($client.Window,0x111,(208 -bor (1 -shl 16)),$h.ToInt64())
 WaitFor {Enabled $client 113} 'learned light selected and current frame ready'
}
try {
 $normalPath=(Resolve-Path -LiteralPath $NormalQualification).Path
 if(!$normalPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned normal qualification required'}
 $normal=Get-Content -LiteralPath $normalPath -Raw|ConvertFrom-Json
 $manifest=Join-Path (Split-Path -Parent $normalPath) 'source-inputs.json'
 $compilationInputs=Get-Content -LiteralPath $manifest -Raw|ConvertFrom-Json -AsHashtable
 if($normal.source_files -ne 970 -or $compilationInputs.Count -ne 970 -or (Get-FileHash -LiteralPath $manifest).Hash -ne $normal.source_manifest_sha256){throw 'Current complete970 source map required'}
 foreach($name in $compilationInputs.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $compilationInputs[$name]){throw ('Source drift: '+$name)}}
 $main=$normal.profiles.$BuildProfile
 if(!$main.unique_entry -or $main.features.Count -ne 0 -or (Get-FileHash -LiteralPath $exe).Hash -ne $main.sha256){throw 'Qualified normal main required'}
 $busy=@(Get-CimInstance Win32_Process|Where-Object { $_.Name -match '^(cargo|rustc|native_.*|LiToModern.*)\.exe$' -and $_.CommandLine -and $_.CommandLine.Replace('/','\').Contains($project) })
 if($busy.Count){throw 'Wait for owned compiler/tests/UI before GUI'}
 Copy-Item -LiteralPath $manifest -Destination (Join-Path $out 'source-inputs.json')
 Copy-Item -LiteralPath $normalPath -Destination (Join-Path $out 'normal-qualification.json')
 $result.normal_qualification_sha256=(Get-FileHash -LiteralPath $normalPath).Hash
 $result.source_manifest_sha256=$normal.source_manifest_sha256
 $result.source_files=970;$result.exe_sha256=$main.sha256
 $result.audio_device_playback_count_verified=$false;$result.night_illumination_verified=$false;$result.natural3600s_expiry_verified=$false
 $result.resource_profile=$ResourceProfile;$result.extended_item_details=($ResourceProfile -eq 'tw13081901-v1')
 $pins=@{}
 foreach($file in @('維護工具/verify-native-light-lease-window.ps1','維護工具/run-isolated-native-light-lease-window-test.ps1','維護工具/oracle/NativeLightLeaseHarness.java','LiTo381-Modern/tests/oracle/NativePeerHarness.java')){
  $pins[$file]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $file)).Hash
  $target=Join-Path $out ('source-snapshot/'+$file);New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force|Out-Null
  Copy-Item -LiteralPath (Join-Path $taskRoot $file) -Destination $target
 }
 $result.source_pins=$pins;Save
 & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $jar -d $classes (Join-Path $taskRoot '維護工具/oracle/NativeLightLeaseHarness.java') *> (Join-Path $out 'javac.log')
 if($LASTEXITCODE -ne 0){throw 'Passive light harness compile failed'}
 $catalog=Join-Path $project 'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
 if((Get-FileHash -LiteralPath $catalog).Hash -ne 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'){throw 'Owned shared catalog identity differs'}
 $result.common_catalog_sha256=$(if($ResourceProfile -eq 'tw13081901-v1'){(Get-FileHash -LiteralPath $catalog).Hash}else{$null})
 $fixture='rental-entry'
 $result.initial_memory_fixture=$fixture
 $javaArgs=@('-Dfile.encoding=UTF-8','-Xverify:all',('-Dlito.resourceProfile='+$ResourceProfile),('-Dlito.extendedItemDetails='+($ResourceProfile -eq 'tw13081901-v1').ToString().ToLowerInvariant()))
 if($ResourceProfile -eq 'tw13081901-v1'){$javaArgs+=('-Dlito.commonCatalog="'+$catalog+'"')}
 $javaArgs+=@('-cp',('"'+$classes+';'+$jar+'"'),'local.lito381.server.NativeLightLeaseHarness',('"'+$root+'"'),$fixture)
 $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList $javaArgs -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $out 'server.log') -RedirectStandardError (Join-Path $out 'server.stderr.txt')
 $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$server.Id)
 $result.server_launch=@{pid=$server.Id;start_utc=$server.StartTime.ToUniversalTime().ToString('o');command=$live.CommandLine;jar_sha256=$jarSha};Save
 WaitFor {Test-Path -LiteralPath (Join-Path $root 'ready.json')} 'server ready';$ready=Get-Content -LiteralPath (Join-Path $root 'ready.json') -Raw|ConvertFrom-Json
 $a=Launch 'a';$b=Launch 'b';Bootstrap $a;Bootstrap $b
 WaitFor {(Authority $a).rental_town -and (Authority $b).rental_town} 'actual rental town authority'
 $result.before_a=Authority $a;$result.before_b=Authority $b;$result.casts=@();Save
 Snapshot $a 'before-cast';Snapshot $b 'before-cast';SelectLight $a
 $before=Authority $a;$clock=[Diagnostics.Stopwatch]::StartNew();Click $a 113
 WaitFor {[long](Authority $a).light_expires_at_nanos -gt [long]$before.light_expires_at_nanos} 'original3600s light activation in rental town'
 WaitFor {(TextOf $a 204).Contains('收到施法動作') -and (Enabled $a 113)} 'visible original cast acknowledgement'
 $result.activated_a=Authority $a;$result.casts=@(@{round=1;before=$before;after_a=(Authority $a);after_b=(Authority $b);GUI_status=(TextOf $a 204)});Save
 Snapshot $a 'cast-1';Snapshot $b 'cast-1'
 if($Scenario -eq 'expiry'){
  $result.natural_wait_started_utc=[DateTime]::UtcNow.ToString('o');$samples=0;Save
  while($clock.ElapsedMilliseconds -lt 3599000){
   if($a.Process.HasExited -or $b.Process.HasExited){throw 'Native GUI closed during natural light lease'}
   $state=Authority $a
   if(!$state.rental_town -or $state.phase -ne 'BATTLE' -or [long]$state.light_expires_at_nanos -ne [long]$result.activated_a.light_expires_at_nanos){throw 'Original light lease disappeared early'}
   if(!(TextOf $a 204).StartsWith('地圖 500') -or (TextOf $a 204).Contains('音效：')){throw 'Native world/audio error while lease live'}
   $samples++
   if($samples % 10 -eq 0){$result.natural_wait_elapsed_ms=$clock.ElapsedMilliseconds;$result.natural_live_samples=$samples;Save}
   Start-Sleep -Milliseconds 1000
  }
  $limit=[DateTime]::UtcNow.AddSeconds(25)
  while([long](Authority $a).light_expires_at_nanos -ne 0){if([DateTime]::UtcNow -gt $limit){throw 'Original light expiry was not observed'};Start-Sleep -Milliseconds 100}
  $result.natural_wait_elapsed_ms=$clock.ElapsedMilliseconds
  if($result.natural_wait_elapsed_ms -lt 3599000){throw 'Natural lease wait too short'}
  $result.natural_expiry_observed=$true;Snapshot $a 'natural-expired';Snapshot $b 'natural-expired';Save
 }
 $result.after_a=Authority $a;$result.after_b=Authority $b;Save
 foreach($client in $clients){[void][OwnedTeleportWindow]::PostMessage($client.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero)}
 foreach($client in $clients){if(!$client.Process.WaitForExit(15000)){throw 'Owned GUI shutdown exceeded'}}
 if(!$server.WaitForExit(15000) -or $server.ExitCode -ne 0){throw 'Owned server cleanup failed'}
 $result.server=Get-Content -LiteralPath (Join-Path $root 'result.json') -Raw|ConvertFrom-Json
 if($result.server.actual_client_sessions -ne 4 -or !$result.server.memberships_empty -or $result.server.authority_mutations -ne 0){throw 'Actual4 sessions/cleanup mismatch'}
 foreach($name in $compilationInputs.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $compilationInputs[$name]){throw ('Compilation input drift: '+$name)}}
 foreach($file in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $file)).Hash -ne $pins[$file]){throw 'GUI QA source drift'}}
 if((Get-FileHash -LiteralPath $exe).Hash -ne $result.exe_sha256 -or (Get-FileHash -LiteralPath $jar).Hash -ne $jarSha){throw 'Normal image/reference drift'}
 $result.observed_source_before_sha256=$observedSourceBefore;$result.observed_source_after_sha256=(Get-FileHash -LiteralPath $source).Hash
 $result.reference_is_current=($observedSourceBefore -eq $jarSha -and $result.observed_source_after_sha256 -eq $jarSha)
 if(!$result.reference_is_current){throw 'Readonly formal source changed'}
 $result.inputs_unchanged=$true;$result.status='completed_native_light_lease_GUI_pending_independent_wire_qualification';Save
}catch{
 $result.status='failed';$result.error=$_.Exception.Message;$result.error_stack=$_.ScriptStackTrace
 foreach($client in $clients){try{Snapshot $client 'failure'}catch{}}
 Save;throw
}finally{
 foreach($client in $clients){if(!$client.Process.HasExited){[void][OwnedTeleportWindow]::PostMessage($client.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero);if(!$client.Process.WaitForExit(5000)){StopOwned $client.Process ('127.0.0.1:'+$ready.port)}};$client.Process.Dispose()}
 if($server){if(!$server.HasExited){StopOwned $server $root};$server.Dispose()}
 foreach($file in @('ready.json','result.json','server-trace.jsonl')){if(Test-Path -LiteralPath (Join-Path $root $file)){Copy-Item -LiteralPath (Join-Path $root $file) -Destination (Join-Path $out $file)}}
 if(Test-Path -LiteralPath (Join-Path $root 'captures')){Copy-Item -LiteralPath (Join-Path $root 'captures') -Destination (Join-Path $out 'raw-capture') -Recurse}
 Save
}
Write-Output ('LIGHT_LEASE_GUI_OUTPUT='+$out)
