#Requires -Version 7.0
param([ValidateSet('Debug','Release')][string]$BuildProfile='Release',[ValidateSet(573,563)][int]$MapId=563,[Parameter(Mandatory)][string]$SupportBinary,[Parameter(Mandatory)][string]$BuildInputManifest,[switch]$VerifyStatusScroll)
$Scenario='siege-portals'
# Only owned native windows on the unswitched test desktop receive messages.
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot;$project=Join-Path $taskRoot 'LiTo381-Modern'
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $project ('target/current/native-siege-portal-gui/'+$stamp)
$out=Join-Path $taskRoot ('驗證紀錄/current/native-siege-portal-gui/'+$stamp)
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
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]static extern int GetClassName(IntPtr h,StringBuilder s,int size);
 [DllImport("user32.dll",EntryPoint="GetWindowLongW")]static extern int GetStyleWord(IntPtr h,int index);
 [DllImport("user32.dll")]static extern IntPtr GetDC(IntPtr h);
 [DllImport("user32.dll")]static extern int ReleaseDC(IntPtr h,IntPtr dc);
 [DllImport("gdi32.dll")]static extern IntPtr SelectObject(IntPtr dc,IntPtr obj);
 [StructLayout(LayoutKind.Sequential)]struct Size{public int cx,cy;}
 [DllImport("gdi32.dll",CharSet=CharSet.Unicode)]static extern bool GetTextExtentPoint32(IntPtr dc,string text,int length,out Size size);
 public static string ClassName(IntPtr h){var s=new StringBuilder(128);if(GetClassName(h,s,s.Capacity)==0)throw new Exception("Owned class name unavailable");return s.ToString();}
 public static uint Style(IntPtr h){return unchecked((uint)GetStyleWord(h,-16));}
 public static int CharacterY(IntPtr h,int index){long value=SendPhysical(h,0xd6,index,0);if(value==-1)throw new Exception("Status character position unavailable");return unchecked((short)((value>>16)&65535));}
 public static int LinePitch(IntPtr h){long lines=Send(h,0xba,0,0);if(lines<2)throw new Exception("Status has no adjacent lines");int last=checked((int)Send(h,0xbb,lines-1,0)),previous=checked((int)Send(h,0xbb,lines-2,0));if(last<0||previous<0)throw new Exception("Status line index unavailable");int pitch=CharacterY(h,last)-CharacterY(h,previous);if(pitch<=0||pitch>256)throw new Exception("Status line pitch outside bound");return pitch;}
 public static string Text(IntPtr h){var s=new StringBuilder(4096);IntPtr r;if(ReadText(h,13,(IntPtr)s.Capacity,s,2,2000,out r)==IntPtr.Zero)throw new Exception("Control text timeout");return s.ToString();}
 public static long Send(IntPtr h,uint m,long w,long l){IntPtr r;if(SendMessageTimeout(h,m,(IntPtr)w,(IntPtr)l,2,2000,out r)==IntPtr.Zero)throw new Exception("Window message timeout");return r.ToInt64();}
 public static void SetText(IntPtr h,string s){IntPtr r;if(SendText(h,12,IntPtr.Zero,s,2,2000,out r)==IntPtr.Zero)throw new Exception("Text message timeout");}
 public static string ComboRow(IntPtr h,int index){var s=new StringBuilder(1024);IntPtr r;if(ReadText(h,0x148,(IntPtr)index,s,2,2000,out r)==IntPtr.Zero)throw new Exception("Combo read timeout");return s.ToString();}
 public static string Row(IntPtr h,int index){var s=new StringBuilder(1024);IntPtr r;if(ReadText(h,0x189,(IntPtr)index,s,2,2000,out r)==IntPtr.Zero)throw new Exception("List read timeout");return s.ToString();}
}
"@
function C($client,[int]$id){$h=[OwnedTeleportWindow]::GetDlgItem($client.Window,$id);if($h -eq [IntPtr]::Zero){throw ('Missing control '+$id)};[uint32]$owner=0;[void][OwnedTeleportWindow]::GetWindowThreadProcessId($h,[ref]$owner);if($owner -ne $client.Process.Id){throw 'Foreign control'};return $h}
function Enabled($client,[int]$id){return [OwnedTeleportWindow]::IsWindowEnabled((C $client $id))}
function TextOf($client,[int]$id){return [OwnedTeleportWindow]::Text((C $client $id))}
function SetText($client,[int]$id,[string]$text){[OwnedTeleportWindow]::SetText((C $client $id),$text)}
function Click($client,[int]$id){$h=C $client $id;if(![OwnedTeleportWindow]::IsWindowVisible($h) -or ![OwnedTeleportWindow]::IsWindowEnabled($h)){throw ('Control not available '+$id)};[void][OwnedTeleportWindow]::Send($h,0xf5,0,0)}
function SelectCombo($client,[int]$index){[void][OwnedTeleportWindow]::Send((C $client 203),0x14e,$index,0)}
function SelectRow($client,[string]$match){$h=C $client 205;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0);$found=-1;for($n=0;$n -lt $count;$n++){if([OwnedTeleportWindow]::Row($h,$n).Contains($match)){$found=$n;break}};if($found -lt 0){throw ('List row absent '+$match)};[void][OwnedTeleportWindow]::Send($h,0x186,$found,0);[void][OwnedTeleportWindow]::Send($client.Window,0x111,(205 -bor (1 -shl 16)),$h.ToInt64())}
function SelectComboMatch($client,[int]$id,[string]$match){$h=C $client $id;$count=[OwnedTeleportWindow]::Send($h,0x146,0,0);$labels=@();for($i=0;$i -lt $count;$i++){$label=[OwnedTeleportWindow]::ComboRow($h,$i);$labels+=@($label);if($label.Contains($match)){[void][OwnedTeleportWindow]::Send($h,0x14e,$i,0);return}};throw ('Combo match absent: '+$match+' in '+($labels -join ', '))}
function SelectMerchantRow($client,[string]$match){$h=C $client 221;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0);for($n=0;$n -lt $count;$n++){if([OwnedTeleportWindow]::Row($h,$n).Contains($match)){[void][OwnedTeleportWindow]::Send($h,0x186,$n,0);[void][OwnedTeleportWindow]::Send($client.Window,0x111,(221 -bor (1 -shl 16)),$h.ToInt64());return}};throw ('Merchant row absent '+$match)}
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

$clients=@();$server=$null;$support=$null;$steps=@();$ready=$null
$manifestPath=[IO.Path]::GetFullPath($BuildInputManifest)
$allowed=[IO.Path]::GetFullPath((Join-Path $project 'target/current'))+[IO.Path]::DirectorySeparatorChar
if(![IO.Path]::GetFullPath($SupportBinary).StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw 'Support binary outside owned target'}
if(!$manifestPath.StartsWith($taskRoot+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Manifest outside workspace'}
$buildInputs=Get-Content -LiteralPath $manifestPath -Raw|ConvertFrom-Json -AsHashtable
if($buildInputs.status -ne 'completed_build' -or $buildInputs.profile -ne $BuildProfile -or (Get-FileHash -LiteralPath $SupportBinary).Hash -ne $buildInputs.products.native_arena_portal.sha256){throw 'Support build mismatch'}
function Stable {
 foreach($n in $buildInputs.files.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $n)).Hash -ne $buildInputs.files[$n]){throw ('Compiled input changed '+$n)}}
 if((Get-FileHash -LiteralPath $exe).Hash -ne $buildInputs.main.sha256){throw 'Main build mismatch'}
}
Stable
Copy-Item -LiteralPath $manifestPath -Destination (Join-Path $out 'build-inputs.json')
$immutableSupport=Join-Path $root 'immutable-siege-support.exe';Copy-Item -LiteralPath $SupportBinary -Destination $immutableSupport
$result=[ordered]@{status='running';profile=$BuildProfile;map_id=$MapId;runtime=$root;output=$out;exe_sha256=(Get-FileHash -LiteralPath $exe).Hash;support_binary_sha256=(Get-FileHash -LiteralPath $immutableSupport).Hash;build_manifest_sha256=(Get-FileHash -LiteralPath $manifestPath).Hash;jar_sha256=$jarSha;steps=@();physical_input_verified=$false;private_desktop=$true;actual_player_target=30;source_only_entrance=$true;complete_game=$false;verify_status_scroll=[bool]$VerifyStatusScroll}
function Save {$result.steps=$script:steps;$result|ConvertTo-Json -Depth 30|Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding utf8}
function WaitFor([scriptblock]$Condition,[string]$Reason){$until=[DateTime]::UtcNow.AddSeconds(40);while(!(& $Condition)){if($support -and $support.HasExited -and $support.ExitCode -ne 0){throw 'Support controller failed; inspect support.stderr.txt'};if([DateTime]::UtcNow -gt $until){throw ('Timeout: '+$Reason)};Start-Sleep -Milliseconds 25}}
function Snapshot($client,[string]$Name){
 $worldShot=$Name -eq 'battle' -or $Name -eq 'near-entrance' -or $Name -eq 'post-portal-walk' -or $Name.StartsWith('sector-') -or $Name.Contains('entry-') -or $Name.StartsWith('status-')
 $settle=[Diagnostics.Stopwatch]::StartNew();if($worldShot){WaitFor {Enabled $client 113} 'current rendered screenshot guard'}
 Start-Sleep -Milliseconds 180;[void][OwnedTeleportWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)
 $rect=[OwnedTeleportWindow]::PhysicalRect($client.Window);$until=[DateTime]::UtcNow.AddSeconds(3);$probe=@()
 do {
  $bitmap=[System.Drawing.Bitmap]::new($rect.Right-$rect.Left,$rect.Bottom-$rect.Top);$graphics=[System.Drawing.Graphics]::FromImage($bitmap);$dc=$graphics.GetHdc()
  try{if(![OwnedTeleportWindow]::PrintWindow($client.Window,$dc,2)){throw 'Owned screenshot failed'}}finally{$graphics.ReleaseHdc($dc);$graphics.Dispose()}
  $colored=0;if($worldShot){for($gy=0;$gy -lt 8;$gy++){for($gx=0;$gx -lt 16;$gx++){$c=$bitmap.GetPixel([int][Math]::Round($bitmap.Width*(0.25+0.5*$gx/15)),[int][Math]::Round($bitmap.Height*(0.25+0.25*$gy/7)));$maximum=[Math]::Max($c.R,[Math]::Max($c.G,$c.B));$minimum=[Math]::Min($c.R,[Math]::Min($c.G,$c.B));if($maximum -gt 20 -and $maximum-$minimum -gt 15){$colored++}}};$probe+=@($colored)}
  if($worldShot -and $colored -lt 10){$bitmap.Dispose();if([DateTime]::UtcNow -ge $until){throw 'Current rendered scene did not acquire original colored terrain'};Start-Sleep -Milliseconds 60;[void][OwnedTeleportWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)}
 }while($worldShot -and $colored -lt 10)
 $path=Join-Path $out ($client.Role+'-'+$Name+'.png');try{$bitmap.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)}finally{$bitmap.Dispose()};$settle.Stop();$client.Process.Refresh()
 $rows=@();$selectedIndex=-1;$selectedRow=$null;$list=C $client 205;if([OwnedTeleportWindow]::IsWindowVisible($list)){$count=[OwnedTeleportWindow]::Send($list,0x18b,0,0);for($i=0;$i -lt $count;$i++){$rows+=@([OwnedTeleportWindow]::Row($list,$i))};$selectedIndex=[OwnedTeleportWindow]::Send($list,0x188,0,0);if($selectedIndex -ge 0){$selectedRow=[OwnedTeleportWindow]::Row($list,$selectedIndex)}}
 $script:steps+=@{role=$client.Role;step=$Name;status=(TextOf $client 204);authority=(Authority $client);room_rows=$rows;selected_room_index=$selectedIndex;selected_room_row=$selectedRow;terrain_paint_probe=$probe;scene_settle_ms=$settle.ElapsedMilliseconds;private_bytes=$client.Process.PrivateMemorySize64;handles=$client.Process.HandleCount;DPI=[OwnedTeleportWindow]::GetDpiForWindow($client.Window);png=$path;png_sha256=(Get-FileHash -LiteralPath $path).Hash};Save
}
function WriteMarkerJson([string]$Name,$Data){
 $path=Join-Path $root $Name;$temp=$path+'.tmp';[IO.File]::WriteAllText($temp,($Data|ConvertTo-Json -Depth 25),[Text.UTF8Encoding]::new($false));[IO.File]::Move($temp,$path)
}
function EnterLobby($client){
 Click $client 101;WaitFor {Enabled $client 102} 'login ready';SetText $client 201 $client.Account;SetText $client 202 'TestOnlyPassword';Click $client 102
 WaitFor {Enabled $client 103} 'source ready';SelectCombo $client 1;Click $client 103;WaitFor {Enabled $client 105} 'GAME ready';Click $client 105
 WaitFor {(Enabled $client 106) -or (Enabled $client 103)} 'directory';if(Enabled $client 106){Click $client 106};WaitFor {Enabled $client 103} 'characters';SelectCombo $client 0;Click $client 103
 WaitFor {(TextOf $client 304) -eq '頻道' -and (Enabled $client 103)} 'channel';SelectCombo $client 0;Click $client 103;WaitFor {Enabled $client 119} 'lobby';Snapshot $client 'lobby'
}
function ParticipantRow($client,[string]$Name){$h=C $client 205;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0);for($n=0;$n -lt $count;$n++){$row=[OwnedTeleportWindow]::Row($h,$n);if($row.Contains($Name)){return $row}};return ''}
function SelectReadySkill($client){
 # Select the existing self-heal locally; never cast. Its public enabled guard
 # observes movement_ready for the current world, including scene completion.
 SelectComboMatch $client 208 '#1 治癒術';$h=C $client 208;[void][OwnedTeleportWindow]::Send($client.Window,0x111,(208 -bor (1 -shl 16)),$h.ToInt64())
 WaitFor {Enabled $client 113} 'current rendered scene movement guard'
}
function FocusWorld($client){[void][OwnedTeleportWindow]::Send($client.Window,0x201,0,(20 -bor (20 -shl 16)));WaitFor {[OwnedTeleportWindow]::FocusFor($client.Window) -eq $client.Window} 'owned world focus'}
function StepWorld($client,$step){
 $frameTimer=[Diagnostics.Stopwatch]::StartNew();WaitFor {Enabled $client 113} 'current movement-ready frame';$frameTimer.Stop();FocusWorld $client;$before=Authority $client;$walkTimer=[Diagnostics.Stopwatch]::StartNew()
 if($before.x -ne $step.before[0] -or $before.y -ne $step.before[1]){throw 'Planned route differs from current authority'}
 [void][OwnedTeleportWindow]::PostMessage($client.Window,0x100,[IntPtr]$step.key,[IntPtr]::Zero);[void][OwnedTeleportWindow]::PostMessage($client.Window,0x101,[IntPtr]$step.key,[IntPtr]::Zero)
 WaitFor {$s=Authority $client;$s.x -eq $step.after[0] -and $s.y -eq $step.after[1]} 'single tapped GUI authoritative step'
 $after=Authority $client;if($after.x -ne $step.after[0] -or $after.y -ne $step.after[1]){throw 'GUI tap movement corrected or repeated'}
 $walkTimer.Stop();$script:steps+=@{role=$client.Role;step='GUI-key-walk';before=$step.before;after=$step.after;key=$step.key;frame_ready_wait_ms=$frameTimer.ElapsedMilliseconds;authority_walk_ms=$walkTimer.ElapsedMilliseconds};Save
}
function VerifyStatusMessage($client){
 $control=C $client 204;$text=TextOf $client 204
 if(!$text.Contains('入口：沒有回覆')){throw 'Scroll check requires actual original attacker no-response feedback'}
 $class=[OwnedTeleportWindow]::ClassName($control);$style=[OwnedTeleportWindow]::Style($control)
 if($class -ne 'Edit' -or ($style -band 0x800) -eq 0 -or ($style -band 0x200000) -eq 0){throw 'Status is not a readonly vertically scrolling edit'}
 $before=Authority $client
 [void][OwnedTeleportWindow]::SendPhysical($control,0x201,1,(5 -bor (5 -shl 16)));[void][OwnedTeleportWindow]::SendPhysical($control,0x202,0,(5 -bor (5 -shl 16)))
 WaitFor {[OwnedTeleportWindow]::FocusFor($client.Window) -eq $control} 'owned status focus'
 [void][OwnedTeleportWindow]::Send($control,0x102,88,0)
 if((TextOf $client 204) -ne $text){throw 'Readonly status was changed by character input'}
 [void][OwnedTeleportWindow]::PostMessage($control,0x100,[IntPtr]0x28,[IntPtr]::Zero);[void][OwnedTeleportWindow]::PostMessage($control,0x101,[IntPtr]0x28,[IntPtr]::Zero)
 Start-Sleep -Milliseconds 350;$after=Authority $client
 if($before.x -ne $after.x -or $before.y -ne $after.y -or $before.object_id -ne $after.object_id){throw 'Focused status navigation moved the player'}
 [void][OwnedTeleportWindow]::Send($control,0x115,6,0);$top=[OwnedTeleportWindow]::Send($control,0xce,0,0)
 [void][OwnedTeleportWindow]::Send($control,0x115,7,0);$bottom=[OwnedTeleportWindow]::Send($control,0xce,0,0);$lines=[OwnedTeleportWindow]::Send($control,0xba,0,0)
 $rect=[OwnedTeleportWindow]::PhysicalClientRect($control);$lastY=[OwnedTeleportWindow]::CharacterY($control,($text.Length-1));$glyphHeight=[OwnedTeleportWindow]::LinePitch($control)
 if($top -ne 0 -or $bottom -le $top -or $lastY -lt 0 -or $lastY+$glyphHeight -gt $rect.Bottom){throw 'Final status text is not fully visible after scroll'}
 if((TextOf $client 204) -ne $text){throw 'Status content changed during scroll check'}
 $script:steps+=@{role=$client.Role;step='status-scroll-proof';class_name=$class;readonly=$true;vertical_scroll=$true;character_input_rejected=$true;status_text=$text;wrapped_lines=$lines;first_visible_top=$top;first_visible_bottom=$bottom;last_character_y=$lastY;glyph_height=$glyphHeight;glyph_height_basis='native_edit_line_pitch';line_pitch=$glyphHeight;client_height=$rect.Bottom;authority_before=$before;authority_after=$after};Save
 Snapshot $client 'status-message-bottom'
 [void][OwnedTeleportWindow]::Send($control,0x115,6,0);FocusWorld $client
}
function PortalRoundtrip($client){
 $transitions=@();$attacker=(Authority $client).siege_attacker_force;$force=if($client.Role -eq 'a'){0}else{1};$admitted=$force -ne $attacker
 for($n=0;$n -lt 2;$n++){
  SelectReadySkill $client;FocusWorld $client;$before=Authority $client;Snapshot $client ('before-entry-'+$n)
  [void][OwnedTeleportWindow]::PostMessage($client.Window,0x100,[IntPtr]0x50,[IntPtr]::Zero);[void][OwnedTeleportWindow]::PostMessage($client.Window,0x101,[IntPtr]0x50,[IntPtr]::Zero)
  if($admitted){WaitFor {$s=Authority $client;($s.x -ne $before.x -or $s.y -ne $before.y) -and (TextOf $client 204).Contains('入口：場景已更新')} 'defender source portal'}else{WaitFor {(TextOf $client 204).Contains('入口：等待回覆')} 'one attacker request pending';WaitFor {(TextOf $client 204).Contains('入口：沒有回覆，請重新操作')} 'bounded original attacker no-response'}
  SelectReadySkill $client;$after=Authority $client
  if(!$admitted -and ($after.x -ne $before.x -or $after.y -ne $before.y)){throw 'Attacker refusal changed authority'}
  $transitions+=@{before=$before;after=$after;admitted=$admitted};$result[($client.Role+'-portals')]=$transitions;Snapshot $client ('after-entry-'+$n)
  if($VerifyStatusScroll -and !$admitted -and $n -eq 1){VerifyStatusMessage $client}
 }
}
Save
try {
 & (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile *> (Join-Path $out 'build-before-server.log');if($LASTEXITCODE -ne 0){throw 'Main build failed'};Stable
 $pins=@{};foreach($n in @('維護工具/verify-native-siege-portal-window.ps1','維護工具/run-isolated-native-siege-portal-window-test.ps1','維護工具/verify-native-siege-portal-gui.py','LiTo381-Modern/tests/native_arena_portal.rs','LiTo381-Modern/tests/oracle/NativePeerHarness.java','LiTo381-Modern/src/modern_lito/native_room.rs','LiTo381-Modern/src/modern_lito/native_login_window.rs')){$pins[$n]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $n)).Hash;$dest=Join-Path $out ('source-snapshot/'+$n);New-Item -ItemType Directory -Path (Split-Path -Parent $dest) -Force|Out-Null;Copy-Item -LiteralPath (Join-Path $taskRoot $n) -Destination $dest};$result.source_pins=$pins;Save
 & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $jar -d $classes (Join-Path $project 'tests/oracle/NativePeerHarness.java') *> (Join-Path $out 'javac.log');if($LASTEXITCODE -ne 0){throw 'Harness compile failed'}
 $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-Xverify:all','-cp',('"'+$classes+';'+$jar+'"'),'local.lito381.server.NativePeerHarness',('"'+$root+'"'),'siege-native') -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $out 'server.log') -RedirectStandardError (Join-Path $out 'server.stderr.txt')
 WaitFor {Test-Path -LiteralPath (Join-Path $root 'ready.json')} 'server ready';$ready=Get-Content -LiteralPath (Join-Path $root 'ready.json') -Raw|ConvertFrom-Json
 $env:LITO_ARENA_PORTAL_ROOT=$root;$env:LITO_ARENA_PORTAL_REFERENCE_SHA=$jarSha.ToLowerInvariant();$env:LITO_ARENA_PORTAL_MAP=$MapId.ToString();$env:LITO_ARENA_GUI_SUPPORT='1'
 $support=Start-Process -FilePath $immutableSupport -ArgumentList @('--exact','actual_siege_gui_supports_twenty_eight_players','--ignored','--nocapture') -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $out 'support.stdout.txt') -RedirectStandardError (Join-Path $out 'support.stderr.txt')
 $a=Launch 'a';$b=Launch 'b';EnterLobby $a;EnterLobby $b;WaitFor {@(Get-ChildItem -LiteralPath $root -Filter 'e*-lobby').Count -eq 28} 'twenty-eight normal support logins'
 Click $a 119;SetText $a 213 '攻城入口測試';SetText $a 214 '30';Click $a 121;WaitFor {Enabled $a 112} 'created thirty-player room'
 for($modeStep=0;$modeStep -lt 6;$modeStep++){if((TextOf $a 204).Contains('模式 攻城')){break};$beforeMode=TextOf $a 204;Click $a 126;WaitFor {(Enabled $a 126) -and (TextOf $a 204) -ne $beforeMode -and (TextOf $a 204).Contains('模式 ')} 'acknowledged normal room mode';Snapshot $a ('mode-step-'+$modeStep)}
 WaitFor {(TextOf $a 204).Contains('模式 攻城')} 'original siege mode';if($MapId -eq 563){Click $a 137;WaitFor {(TextOf $a 204).Contains('地圖 563') -and (Enabled $a 137)} 'normal map selection'}else{WaitFor {(TextOf $a 204).Contains('地圖 573')} 'original siege map'}
 $room=Authority $a;WriteMarkerJson 'created-room.json' @{id=$room.room}
 WaitFor {(Authority $a).room_state.members.Count -eq 28} 'twenty-seven support peers joined before selected-room update'
 Click $b 108;WaitFor {Enabled $b 109} 'peer room list';SelectRow $b '攻城入口測試';Snapshot $b 'lobby-selection-before-update'
 Set-Content -LiteralPath (Join-Path $root 'gui-last-peer-join') -Value 'normal last peer join' -Encoding utf8
 WaitFor {$list=C $b 205;$index=[OwnedTeleportWindow]::Send($list,0x188,0,0);$index -ge 0 -and [OwnedTeleportWindow]::Row($list,$index).Contains('攻城入口測試') -and [OwnedTeleportWindow]::Row($list,$index).Contains('29/30') -and (Authority $a).room_state.members.Count -eq 29} 'selected room retained across genuine peer arrival'
 Snapshot $b 'lobby-selection-updated';Click $b 109;WaitFor {(Enabled $b 118) -and (ParticipantRow $b 'PeerFixture2').Contains('隊伍')} 'peer join and force visible';if((ParticipantRow $b 'PeerFixture2').Contains('隊伍1')){Click $b 118}
 WaitFor {$h=C $b 205;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0);$found=$false;for($i=0;$i -lt $count;$i++){if([OwnedTeleportWindow]::Row($h,$i).Contains('PeerFixture2') -and [OwnedTeleportWindow]::Row($h,$i).Contains('隊伍2')){$found=$true}};$found} 'peer opposite force'
 WaitFor {(Authority $a).room_state.members.Count -eq 30 -and (TextOf $a 204).Contains('已收到 30 位成員')} 'original thirty-member room';Snapshot $a 'room-ready';Snapshot $b 'room-ready';Click $a 110;Click $b 110
 WaitFor {(Authority $a).phase -eq 'BATTLE' -and (Authority $b).phase -eq 'BATTLE' -and (TextOf $a 204).Contains('地圖 '+$MapId) -and (TextOf $b 204).Contains('地圖 '+$MapId)} 'normal fifteen-per-team siege battle'
 SelectReadySkill $a;SelectReadySkill $b;Snapshot $a 'battle';Snapshot $b 'battle';WriteMarkerJson 'gui-starts.json' @{map=$MapId;a=(Authority $a);b=(Authority $b)}
 WaitFor {Test-Path -LiteralPath (Join-Path $root 'gui-routes.json')} 'original navigation plans';$routes=Get-Content -LiteralPath (Join-Path $root 'gui-routes.json') -Raw|ConvertFrom-Json;$result.route_plan_sha256=(Get-FileHash -LiteralPath (Join-Path $root 'gui-routes.json')).Hash;Save
 foreach($client in $clients){$previous=$null;foreach($step in $routes.($client.Role).steps){StepWorld $client $step;$s=Authority $client;$sector=@([Math]::Floor(($s.x-32704)/64),[Math]::Floor(($s.y-32704)/64)) -join ',';if($sector -ne $previous){Snapshot $client ('sector-'+$s.x+'-'+$s.y);$previous=$sector}};WaitFor {Enabled $client 113} 'entrance frame settled';Snapshot $client 'near-entrance'}
 foreach($client in $clients){PortalRoundtrip $client;$state=Authority $client;$key=([string]$state.x)+','+([string]$state.y);$post=$routes.post_walks.$key;if(!$post){throw ('No original post-portal walk '+$key)};StepWorld $client $post;Snapshot $client 'post-portal-walk'}
 foreach($client in $clients){Set-Content -LiteralPath (Join-Path $root ($client.Role+'-report-saved')) -Value 'GUI source-only portal complete' -Encoding utf8}
 if(!$support.WaitForExit(15000) -or $support.ExitCode -ne 0){throw 'Support cleanup failed'}
 foreach($client in $clients){[void][OwnedTeleportWindow]::PostMessage($client.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero)};foreach($client in $clients){if(!$client.Process.WaitForExit(15000)){throw 'GUI cleanup timeout'}}
 if(!$server.WaitForExit(15000) -or $server.ExitCode -ne 0){throw 'Server cleanup failed'};$result.server=Get-Content -LiteralPath (Join-Path $root 'result.json') -Raw|ConvertFrom-Json
 if($result.server.actual_client_sessions -ne 60 -or !$result.server.memberships_empty -or $result.server.authority_mutations -ne 0){throw 'Sixty-session cleanup mismatch'}
 Stable;foreach($n in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $n)).Hash -ne $pins[$n]){throw 'QA source drift'}}
 $result.observed_source_before_sha256=$observedSourceBefore;$result.observed_source_after_sha256=(Get-FileHash -LiteralPath $source).Hash;$result.reference_is_current=($observedSourceBefore -eq $jarSha -and $result.observed_source_after_sha256 -eq $jarSha);$result.inputs_unchanged=$true;$result.status='verified_native_siege_portal_GUI_pending_independent_comparison';Save
}catch{$result.status='failed';$result.error=$_.Exception.Message;$result.error_stack=$_.ScriptStackTrace;Set-Content -LiteralPath (Join-Path $root 'gui-abort') -Value 'abort' -Encoding utf8;foreach($client in $clients){try{Snapshot $client 'failure'}catch{}};Save;throw}
finally {
 foreach($client in $clients){if(!$client.Process.HasExited){[void][OwnedTeleportWindow]::PostMessage($client.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero);if(!$client.Process.WaitForExit(5000)){StopOwned $client.Process ('127.0.0.1:'+$ready.port)}};$client.Process.Dispose()}
 if($support -and !$support.HasExited){Set-Content -LiteralPath (Join-Path $root 'gui-abort') -Value 'abort' -Encoding utf8;if(!$support.WaitForExit(10000)){StopOwned $support $immutableSupport}}
 if($server){if(!$server.HasExited){StopOwned $server $root};$server.Dispose()}
 foreach($n in @('gui-starts.json','gui-routes.json','result.json','ready.json','server-trace.jsonl')){if(Test-Path -LiteralPath (Join-Path $root $n)){Copy-Item -LiteralPath (Join-Path $root $n) -Destination (Join-Path $out $n)}}
 foreach($f in Get-ChildItem -LiteralPath $root -Filter '*-siege-member.json'){Copy-Item -LiteralPath $f.FullName -Destination (Join-Path $out $f.Name)}
 Save
}
Write-Output $out
