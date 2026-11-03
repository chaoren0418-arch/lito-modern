#Requires -Version 7.0
param([ValidateSet('Debug','Release')][string]$BuildProfile='Debug',[ValidateSet('disabled','classic-lab')][string]$Scenario='disabled')
$ReplayStaleCommands=[switch]$false
# Only owned native windows on the unswitched test desktop receive messages.
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot;$project=Join-Path $taskRoot 'LiTo381-Modern'
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $project ('target/current/native-world-tree-gui/'+$stamp)
$out=Join-Path $taskRoot ('驗證紀錄/current/native-world-tree-gui/'+$stamp)
$classes=Join-Path $root 'oracle';New-Item -ItemType Directory -Path $classes,$out -Force|Out-Null
$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP
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
$result=[ordered]@{status='running';replay_stale_commands=$ReplayStaleCommands.IsPresent;stale_command_replays=@();scenario=$Scenario;runtime=$root;output=$out;jar_sha256=$jarSha;exe_sha256=(Get-FileHash -LiteralPath $exe).Hash;steps=@();physical_input_verified=$false;private_desktop=$true;product_complete=$false}
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
  if(($name.StartsWith('refused-') -or $name -in @('world-ready','before-cast','walked'))){
   for($gy=0;$gy -lt 8;$gy++){for($gx=0;$gx -lt 16;$gx++){$x=[int]($bitmap.Width*(0.25+0.5*$gx/15));$y=[int]($bitmap.Height*(0.25+0.25*$gy/7));$pixel=$bitmap.GetPixel($x,$y);$high=[Math]::Max($pixel.R,[Math]::Max($pixel.G,$pixel.B));$low=[Math]::Min($pixel.R,[Math]::Min($pixel.G,$pixel.B));if($high -gt 20 -and $high-$low -gt 15){$colored++}}}
   $paintProbe+=@($colored)
   if($colored -lt 10){$bitmap.Dispose();if([DateTime]::UtcNow -ge $paintUntil){throw 'Ready portal status did not acquire actual colored terrain pixels'};Start-Sleep -Milliseconds 60;[void][OwnedTeleportWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)}
  }
 }while(($name.StartsWith('refused-') -or $name -in @('world-ready','before-cast','walked')) -and $colored -lt 10)

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
 WaitFor {(TextOf $client 304) -eq '頻道' -and (Enabled $client 103)} 'channel selector';SelectCombo $client 0;Click $client 103
 WaitFor {Enabled $client 119} 'lobby';Snapshot $client 'lobby'
}
function SelectWorldTree($client){
 SelectComboMatch $client 208 '#131 '
 $h=C $client 208;[void][OwnedTeleportWindow]::Send($client.Window,0x111,(208 -bor (1 -shl 16)),$h.ToInt64())
 WaitFor {Enabled $client 113} 'world tree skill selected and frame ready'
}
function WalkAfter($client){
 [void][OwnedTeleportWindow]::Send($client.Window,0x201,0,(20 -bor (20 -shl 16)));Start-Sleep -Milliseconds 850
 $before=Authority $client
 $key=if($Scenario -eq 'disabled'){0x68}elseif($client.Role -eq 'a'){0x66}else{0x64}
 [void][OwnedTeleportWindow]::PostMessage($client.Window,0x100,[IntPtr]$key,[IntPtr]::Zero)
 [void][OwnedTeleportWindow]::PostMessage($client.Window,0x101,[IntPtr]$key,[IntPtr]::Zero)
 $targetX=[int]$before.x+$(if($key -eq 0x66){1}elseif($key -eq 0x64){-1}else{0});$targetY=[int]$before.y-$(if($key -eq 0x68){1}else{0})
 WaitFor {$v=Authority $client;$v.x -eq $targetX -and $v.y -eq $targetY} 'exact authoritative followup cell'
 $after=Authority $client;$script:result.walks+=@{role=$client.Role;before=$before;after=$after;key=$key};Snapshot $client 'walked'
}
try {
 $compilationInputs=@{}
 foreach($folder in @('src','tests')){foreach($file in Get-ChildItem -LiteralPath (Join-Path $project $folder) -Recurse -File){$relative=[IO.Path]::GetRelativePath($taskRoot,$file.FullName).Replace('\','/');$compilationInputs[$relative]=(Get-FileHash -LiteralPath $file.FullName).Hash}}
 foreach($name in @('Cargo.toml','Cargo.lock','build.rs','.cargo/config.toml')){$relative='LiTo381-Modern/'+$name;$compilationInputs[$relative]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $relative)).Hash}
 & (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile *> (Join-Path $out 'build-before-server.log')
 $result.exe_sha256=(Get-FileHash -LiteralPath $exe).Hash;Save
 foreach($name in $compilationInputs.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $compilationInputs[$name]){throw ('Compilation input changed during build: '+$name)}}
 @{status='completed_build';target='x86_64-pc-windows-msvc';profile=$BuildProfile;files=$compilationInputs;main_sha256=$result.exe_sha256}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $out 'compilation-inputs.json') -Encoding utf8
 $result.compilation_inputs_sha256=(Get-FileHash -LiteralPath (Join-Path $out 'compilation-inputs.json')).Hash;$result.complete_build_inputs_verified=$true;Save
 $pins=@{};foreach($file in @('維護工具/verify-native-world-tree-window.ps1','維護工具/verify-native-world-tree-gui.py','LiTo381-Modern/src/modern_lito/native_actor_layers.rs','LiTo381-Modern/src/modern_lito/native_actor_states.rs','LiTo381-Modern/src/modern_lito/native_world_layers.rs','LiTo381-Modern/src/modern_lito/native_world_scene.rs','維護工具/run-isolated-native-world-tree-window-test.ps1','LiTo381-Modern/tests/oracle/NativePeerHarness.java','LiTo381-Modern/src/modern_lito/native_portal.rs','LiTo381-Modern/src/modern_lito/native_navigation.rs','LiTo381-Modern/src/modern_lito/native_login.rs','LiTo381-Modern/src/modern_lito/native_login_window.rs','LiTo381-Modern/Cargo.toml','LiTo381-Modern/.cargo/config.toml','LiTo381-Modern/src/modern_lito/native_world.rs','LiTo381-Modern/src/modern_lito/native_skills.rs','LiTo381-Modern/src/modern_lito/native_world_animation.rs')){$pins[$file]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $file)).Hash;$target=Join-Path $out ('source-snapshot/'+$file);New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force|Out-Null;Copy-Item -LiteralPath (Join-Path $taskRoot $file) -Destination $target};$result.source_pins=$pins;Save
 & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $jar -d $classes (Join-Path $project 'tests/oracle/NativePeerHarness.java') *> (Join-Path $out 'javac.log');if($LASTEXITCODE -ne 0){throw 'Harness compile failed'}
 $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-Xverify:all','-cp',('"'+$classes+';'+$jar+'"'),'local.lito381.server.NativePeerHarness',('"'+$root+'"'),$(if($Scenario -eq 'disabled'){'world-tree-disabled'}else{'world-entry'})) -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $out 'server.log') -RedirectStandardError (Join-Path $out 'server.stderr.txt')
 WaitFor {Test-Path -LiteralPath (Join-Path $root 'ready.json')} 'server ready';$ready=Get-Content -LiteralPath (Join-Path $root 'ready.json') -Raw|ConvertFrom-Json
 $result.walks=@();$result.compatibility_lab=($Scenario -eq 'classic-lab');Save
 $a=Launch 'a';$b=Launch 'b';Bootstrap $a;Bootstrap $b
 Click $a 119;SetText $a 213 '世界樹GUI';SetText $a 214 '2';Click $a 121;WaitFor {Enabled $a 112} 'room created'
 Click $b 108;WaitFor {Enabled $b 109} 'room list';SelectRow $b '世界樹GUI';Click $b 109;WaitFor {Enabled $b 112} 'room joined';Click $b 118
 WaitFor {$h=C $b 205;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0);$found=$false;for($n=0;$n -lt $count;$n++){if([OwnedTeleportWindow]::Row($h,$n).Contains('隊伍2')){$found=$true}};$found -and (Enabled $b 110)} 'opposite teams'
 Click $a 110;Click $b 110;WaitFor {(Authority $a).phase -eq 'BATTLE' -and (Authority $b).phase -eq 'BATTLE' -and (TextOf $a 204).StartsWith('地圖 564') -and (TextOf $b 204).StartsWith('地圖 564')} 'battle resources ready'
 $result.before_a=Authority $a;$result.before_b=Authority $b;Snapshot $a 'before-cast';Snapshot $b 'before-cast'
 SelectWorldTree $a;SelectWorldTree $b
 if($Scenario -eq 'disabled'){
  for($round=0;$round -lt 2;$round++){
   Click $a 113;Click $b 113
   foreach($client in $clients){WaitFor {(TextOf $client 204).Contains('目前地圖無法使用此技能') -and (Enabled $client 113)} 'visible map rejection and retry ready';Snapshot $client ('refused-'+$round)}
  }
 }else{
  Click $a 113;Click $b 113
  foreach($client in $clients){WaitFor {(Authority $client).phase -eq 'WORLD' -and (TextOf $client 204).StartsWith('地圖 4') -and (Enabled $client 104)} 'classic lab WORLD rendered';if(Enabled $client 113){throw 'Old skill selection survived WORLD rebuild'};Snapshot $client 'world-ready'}
 }
 WalkAfter $a;WalkAfter $b

 foreach($client in $clients){[void][OwnedTeleportWindow]::PostMessage($client.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero)}
 foreach($client in $clients){if(!$client.Process.WaitForExit(15000)){throw 'GUI shutdown exceeded'}}
 if(!$server.WaitForExit(15000) -or $server.ExitCode -ne 0){throw 'Server cleanup failed'}
 $result.server=Get-Content -LiteralPath (Join-Path $root 'result.json') -Raw|ConvertFrom-Json
 if($result.server.actual_client_sessions -ne 4 -or !$result.server.memberships_empty -or $result.server.authority_mutations -ne 0){throw 'Cleanup mismatch'}
 foreach($name in $compilationInputs.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $compilationInputs[$name]){throw ('Compilation input changed during GUI: '+$name)}}
 foreach($file in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $file)).Hash -ne $pins[$file]){throw 'Source drift'}}
 if((Get-FileHash -LiteralPath $exe).Hash -ne $result.exe_sha256 -or (Get-FileHash -LiteralPath $jar).Hash -ne $jarSha){throw 'Owned binary/reference drift'}
 $result.observed_source_before_sha256=$observedSourceBefore;$result.observed_source_after_sha256=(Get-FileHash -LiteralPath $source).Hash;$result.reference_is_current=($observedSourceBefore -eq $jarSha -and $result.observed_source_after_sha256 -eq $jarSha)
 $result.inputs_unchanged=$true;$result.status='verified_world_tree_native_GUI_defined_scenario';Save
}catch{$result.status='failed';$result.error=$_.Exception.Message;$result.error_stack=$_.ScriptStackTrace;foreach($client in $clients){try{Snapshot $client 'failure'}catch{}};Save;throw}
finally {
 foreach($client in $clients){if(!$client.Process.HasExited){[void][OwnedTeleportWindow]::PostMessage($client.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero);if(!$client.Process.WaitForExit(5000)){StopOwned $client.Process ('127.0.0.1:'+$ready.port)}};$client.Process.Dispose()}
 if($server){if(!$server.HasExited){StopOwned $server $root};$server.Dispose()}
 foreach($file in @('ready.json','result.json','server-trace.jsonl')){if(Test-Path -LiteralPath (Join-Path $root $file)){Copy-Item -LiteralPath (Join-Path $root $file) -Destination (Join-Path $out $file)}};Save
}
Write-Output $out
