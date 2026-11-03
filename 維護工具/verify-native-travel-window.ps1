#Requires -Version 7.0
param([string]$Runtime,[string]$Output,[ValidateSet('Debug','Release')][string]$BuildProfile='Release')
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot;$project=Join-Path $taskRoot 'LiTo381-Modern'
$root=[IO.Path]::GetFullPath($Runtime);$out=[IO.Path]::GetFullPath($Output)
$allowed=[IO.Path]::GetFullPath((Join-Path $project 'target/current'))+'\'
if(!$root.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw 'Owned travel GUI runtime required'}
$ready=Get-Content -LiteralPath (Join-Path $root 'travel-ready.json') -Raw|ConvertFrom-Json
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
$clients=@();$steps=@();$result=[ordered]@{status='running';runtime=$root;private_desktop=$true;physical_input_verified=$false;exe_sha256=(Get-FileHash -LiteralPath $exe).Hash;steps=@();product_complete=$false}
function Save {$result.steps=$script:steps;$result|ConvertTo-Json -Depth 30|Set-Content -LiteralPath (Join-Path $out 'travel-gui.json') -Encoding utf8}
function WaitFor([scriptblock]$Condition,[string]$Reason){$until=[DateTime]::UtcNow.AddSeconds(40);while(!(& $Condition)){if([DateTime]::UtcNow -gt $until){throw ('Timeout: '+$Reason)};Start-Sleep -Milliseconds 30}}
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System;using System.Text;using System.Runtime.InteropServices;
public static class OwnedTeleportWindow {
 [DllImport("kernel32.dll")]static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")]static extern IntPtr GetThreadDesktop(uint thread);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]static extern bool GetUserObjectInformation(IntPtr h,int index,StringBuilder value,uint bytes,out uint needed);
 public static string DesktopName(){var name=new StringBuilder(260);uint needed;if(!GetUserObjectInformation(GetThreadDesktop(GetCurrentThreadId()),2,name,520,out needed))throw new Exception("Thread desktop query failed");return name.ToString();}
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
$desktopName=[OwnedTeleportWindow]::DesktopName();if(!$desktopName.StartsWith('LitoTravelGui')){throw 'Owned unswitched travel test desktop required'};$result.desktop_name=$desktopName
function C($client,[int]$id){$h=[OwnedTeleportWindow]::GetDlgItem($client.Window,$id);if($h -eq [IntPtr]::Zero){throw ('Missing control '+$id)};[uint32]$owner=0;[void][OwnedTeleportWindow]::GetWindowThreadProcessId($h,[ref]$owner);if($owner -ne $client.Process.Id){throw 'Foreign control'};return $h}
function Enabled($client,[int]$id){return [OwnedTeleportWindow]::IsWindowEnabled((C $client $id))}
function TextOf($client,[int]$id){return [OwnedTeleportWindow]::Text((C $client $id))}
function SetText($client,[int]$id,[string]$text){[OwnedTeleportWindow]::SetText((C $client $id),$text)}
function Click($client,[int]$id){$h=C $client $id;if(![OwnedTeleportWindow]::IsWindowVisible($h) -or ![OwnedTeleportWindow]::IsWindowEnabled($h)){throw ('Control not available '+$id)};[void][OwnedTeleportWindow]::Send($h,0xf5,0,0)}
function SelectCombo($client,[int]$index){[void][OwnedTeleportWindow]::Send((C $client 203),0x14e,$index,0)}
function SelectRow($client,[string]$match){$h=C $client 205;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0);$found=-1;for($n=0;$n -lt $count;$n++){if([OwnedTeleportWindow]::Row($h,$n).Contains($match)){$found=$n;break}};if($found -lt 0){throw ('List row absent '+$match)};[void][OwnedTeleportWindow]::Send($h,0x186,$found,0);[void][OwnedTeleportWindow]::Send($client.Window,0x111,(205 -bor (1 -shl 16)),$h.ToInt64())}

function Snapshot($client,[string]$name){
 Start-Sleep -Milliseconds 180
 [void][OwnedTeleportWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)
 $rect=[OwnedTeleportWindow]::PhysicalRect($client.Window)
 $bitmap=[System.Drawing.Bitmap]::new($rect.Right-$rect.Left,$rect.Bottom-$rect.Top);$graphics=[System.Drawing.Graphics]::FromImage($bitmap);$dc=$graphics.GetHdc()
 try{if(![OwnedTeleportWindow]::PrintWindow($client.Window,$dc,2)){throw 'Owned PrintWindow failed'}}finally{$graphics.ReleaseHdc($dc);$graphics.Dispose()}
 $path=Join-Path $out ($client.Role+'-GUI-'+$script:steps.Count+'-'+$name+'.png');try{$bitmap.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)}finally{$bitmap.Dispose()}
 $script:steps+=@{role=$client.Role;step=$name;utc=[DateTime]::UtcNow.ToString('o');request_count=(RequestCount);status=(TextOf $client 204);feedback=(TextOf $client 211);inventory=(TextOf $client 114);inventory_visible=[OwnedTeleportWindow]::IsWindowVisible((C $client 209));cancel_visible=[OwnedTeleportWindow]::IsWindowVisible((C $client 104));return_visible=[OwnedTeleportWindow]::IsWindowVisible((C $client 107));focus=[OwnedTeleportWindow]::FocusFor($client.Window).ToInt64();authority=(Authority $client);png=$path;png_sha256=(Get-FileHash -LiteralPath $path).Hash};Save
}
function StopOwned($process,[string]$marker){
 if($process.HasExited){return};$live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$process.Id)
 if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$live.CommandLine.Replace('/','\').Contains($marker.Replace('/','\'))){throw 'Owned GUI identity mismatch'}
 $process.Kill();$process.WaitForExit(5000)|Out-Null
}
function Launch([string]$role){
 $before=@(Get-Process LiToModern -ErrorAction SilentlyContinue|Select-Object -ExpandProperty Id)
 & (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildProfile $BuildProfile -NativeLoginEndpoint ('127.0.0.1:'+$ready.port) *> (Join-Path $out ('gui-launch-'+$role+'.log'))
 $owned=@(Get-CimInstance Win32_Process -Filter "Name = 'LiToModern.exe'"|Where-Object {$_.ProcessId -notin $before -and $_.ExecutablePath -eq $exe -and $_.CommandLine.Contains('127.0.0.1:'+$ready.port)})
 if($owned.Count -ne 1){throw 'Owned GUI launch identity missing'}
 $process=Get-Process -Id $owned[0].ProcessId;$client=[pscustomobject]@{Role=$role;Account=$(if($role -eq 'a'){'PeerFixture'}else{'PeerFixture2'});Process=$process;Window=[IntPtr]::Zero};$script:clients+=@($client)
 $result.owned_clients=@($script:clients|ForEach-Object {@{pid=$_.Process.Id;start_utc=$_.Process.StartTime.ToUniversalTime().ToString('o');role=$_.Role}});Save
 WaitFor {$client.Window=[OwnedTeleportWindow]::Find($process.Id);$client.Window -ne [IntPtr]::Zero} 'owned window';return $client
}
function Authority($client){
 $needle='"test_account":"'+$client.Account+'"';$trace=Join-Path $root 'travel-server-trace.jsonl'
 $row=Get-Content -LiteralPath $trace -Tail 100|Where-Object {$_.Contains($needle)}|Select-Object -Last 1
 if(!$row){throw 'Owned GUI authority absent'};return ($row|ConvertFrom-Json)
}
function Bootstrap($client){
 Click $client 101;WaitFor {Enabled $client 102} 'login';SetText $client 201 $client.Account;SetText $client 202 'TestOnlyPassword';Click $client 102
 WaitFor {Enabled $client 103} 'sources';SelectCombo $client 1;Click $client 103;WaitFor {Enabled $client 105} 'GAME';Click $client 105
 WaitFor {(Enabled $client 106) -or (Enabled $client 103)} 'notice/characters';if(Enabled $client 106){Click $client 106}
 WaitFor {Enabled $client 103} 'characters';SelectCombo $client 0;Click $client 103
 WaitFor {(TextOf $client 304) -eq '頻道' -and (Enabled $client 103)} 'channel selector';SelectCombo $client 0;Click $client 103
 WaitFor {Enabled $client 119} 'lobby';Snapshot $client 'lobby'
}
function SelectItem($client,[string]$match){
 $h=C $client 209;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0)
 for($n=0;$n -lt $count;$n++){if([OwnedTeleportWindow]::Row($h,$n).Contains($match)){[void][OwnedTeleportWindow]::Send($h,0x186,$n,0);[void][OwnedTeleportWindow]::Send($client.Window,0x111,(209 -bor (1 -shl 16)),$h.ToInt64());return}}
 throw ('Inventory row absent '+$match)
}
function Arm($client){
 if(![OwnedTeleportWindow]::IsWindowVisible((C $client 209))){Click $client 114}
 WaitFor {[OwnedTeleportWindow]::IsWindowVisible((C $client 209))} 'inventory open'
 if([OwnedTeleportWindow]::IsWindowVisible((C $client 104)) -or [OwnedTeleportWindow]::IsWindowVisible((C $client 107))){throw 'World cancel/return overlays inventory operation row'}
 $qty=[OwnedTeleportWindow]::PhysicalRect((C $client 210));$drop=[OwnedTeleportWindow]::PhysicalRect((C $client 116))
 if([Math]::Min($qty.Right,$drop.Right) -gt [Math]::Max($qty.Left,$drop.Left) -and [Math]::Min($qty.Bottom,$drop.Bottom) -gt [Math]::Max($qty.Top,$drop.Top)){throw 'Inventory quantity and drop controls overlap'}
 $statusRect=[OwnedTeleportWindow]::PhysicalRect((C $client 204));$dpi=[OwnedTeleportWindow]::GetDpiForWindow($client.Window)
 if(($statusRect.Bottom-$statusRect.Top) -lt [int](80*$dpi/96)){throw ('Inventory status clips four lines: actual='+($statusRect.Bottom-$statusRect.Top)+' required='+[int](80*$dpi/96))}
 SelectItem $client 'GM瞬間移動';WaitFor {Enabled $client 115} 'target item available'
 Snapshot $client 'inventory-wand';Click $client 115;WaitFor {![OwnedTeleportWindow]::IsWindowVisible((C $client 209))} 'target mode';Snapshot $client 'armed'
}
function Pointer($client,[int]$dx,[int]$dy,[switch]$KeepPaint){
 # Same authoritative isometric transform/camera and letterbox fit as product.
 $cx=400+24*($dx+$dy);$cy=300+12*($dy-$dx)
 $rect=[OwnedTeleportWindow]::PhysicalClientRect($client.Window);$dpi=[OwnedTeleportWindow]::GetDpiForWindow($client.Window)
 $width=$rect.Right;$height=$rect.Bottom-[int](185*$dpi/96)
 if([long]$width*600 -le [long]$height*800){$vw=$width;$vh=[int][Math]::Floor($width*600/800)}else{$vh=$height;$vw=[int][Math]::Floor($height*800/600)}
 $left=[int][Math]::Floor(($width-$vw)/2);$top=[int][Math]::Floor(($height-$vh)/2)
 $x=$left+[int][Math]::Floor(($cx+0.5)*$vw/800);$y=$top+[int][Math]::Floor(($cy+0.5)*$vh/600)
 if(!$KeepPaint){[void][OwnedTeleportWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)}
 [void][OwnedTeleportWindow]::SendPhysical($client.Window,0x201,0,($x -bor ($y -shl 16)))
 $script:steps+=@{role=$client.Role;step='pointer';delta=@($dx,$dy);canvas=@($cx,$cy);physical=@($x,$y);client_size=@($width,$height);dpi=$dpi};Save
}
function TargetPointer($client,[int]$dx,[int]$dy,[int]$expectedCount){
 # Retain the currently visible image while ordinary UI timers complete newer
 # animation frames. One click only; never retry or inject any packet/state.
 $focusBefore=[OwnedTeleportWindow]::FocusFor($client.Window)
 [void][OwnedTeleportWindow]::Send($client.Window,0x0b,0,0)
 try {
  Start-Sleep -Milliseconds 700
  if([OwnedTeleportWindow]::FocusFor($client.Window) -ne $focusBefore){throw 'Paint-only suppression changed input focus'}
  Pointer $client $dx $dy -KeepPaint
  $end=[DateTime]::UtcNow.AddSeconds(3)
  while((RequestCount) -lt $expectedCount -and [DateTime]::UtcNow -lt $end){Start-Sleep -Milliseconds 25}
  $count=RequestCount
  $script:steps+=@{role=$client.Role;step='target-pointer-attempt';attempt=1;request_count=$count;paint_suppressed_ms=700;single_click=$true};Save
  if($count -ne $expectedCount){throw 'Single visible-frame target click did not issue exactly one request'}
 } finally {
  [void][OwnedTeleportWindow]::Send($client.Window,0x0b,1,0)
  [void][OwnedTeleportWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)
 }
}
function RequestCount {
 $file=Get-ChildItem -LiteralPath (Join-Path $root 'travel-captures') -Filter frames.jsonl -Recurse|Select-Object -First 1
 if(!$file){return 0};return @(Get-Content -LiteralPath $file.FullName|ForEach-Object {$_|ConvertFrom-Json}|Where-Object {$_.direction -eq 'client_to_server' -and $_.opcode -eq '0x1C'}).Count
}
Save
try {
 $a=Launch 'a';$b=Launch 'b';Bootstrap $a;Bootstrap $b
 Click $a 119;SetText $a 213 'GM傳送GUI';SetText $a 214 '2';Click $a 121;WaitFor {Enabled $a 112} 'room'
 Click $b 108;WaitFor {Enabled $b 109} 'room refresh';SelectRow $b 'GM傳送GUI';Click $b 109;WaitFor {Enabled $b 112} 'join';Click $b 118
 WaitFor {$h=C $b 205;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0);$found=$false;for($n=0;$n -lt $count;$n++){if([OwnedTeleportWindow]::Row($h,$n).Contains('隊伍2')){$found=$true}};$found -and (Enabled $b 110)} 'opposing force'
 Click $a 110;Click $b 110;WaitFor {(Authority $a).phase -eq 'BATTLE' -and (Authority $b).phase -eq 'BATTLE' -and (TextOf $a 204).StartsWith('地圖 564') -and (TextOf $b 204).StartsWith('地圖 564')} 'rendered ordinary world'
 Snapshot $a 'battle';Snapshot $b 'battle';$result.before_a=Authority $a;$result.before_b=Authority $b;Save
 Arm $a;[void][OwnedTeleportWindow]::PostMessage($a.Window,0x100,[IntPtr]0x1b,[IntPtr]::Zero);Start-Sleep -Milliseconds 250;Pointer $a 6 2;Start-Sleep -Milliseconds 400
 if((RequestCount) -ne 0){throw 'Esc-cancelled cursor issued a target request'};Snapshot $a 'esc-cancelled'
 Arm $a;Click $a 114;WaitFor {[OwnedTeleportWindow]::FocusFor($a.Window) -eq (C $a 209)} 'actual inventory focus';Snapshot $a 'focus-changed';Click $a 114;Pointer $a 6 2;Start-Sleep -Milliseconds 500
 if((RequestCount) -ne 0){throw 'Focus/panel-cancelled cursor issued a target request'};Snapshot $a 'focus-cancelled'
 Arm $a;TargetPointer $a 1 0 1;WaitFor {(TextOf $a 204).Contains('該座標已有單位或障礙物')} 'formal target rejection';Snapshot $a 'target-rejected';$result.rejected=Authority $a;Save
 Arm $a;TargetPointer $a 6 2 2;WaitFor {$s=Authority $a;$s.x -eq 32735 -and $s.y -eq 32799 -and (TextOf $a 204).StartsWith('地圖 564')} 'GUI portal complete'
 Snapshot $a 'after-portal';Snapshot $b 'peer-after-portal';$result.after_a=Authority $a;$result.after_b=Authority $b;Save
 Pointer $a 6 2;Start-Sleep -Milliseconds 400;if((RequestCount) -ne 2){throw 'Old GUI cursor replayed after portal'};Snapshot $a 'old-cursor-rejected'
 [void][OwnedTeleportWindow]::PostMessage($a.Window,0x100,[IntPtr]0x68,[IntPtr]::Zero)
 try{WaitFor {(Authority $a).y -ne 32799} 'post-portal GUI walk'}finally{[void][OwnedTeleportWindow]::PostMessage($a.Window,0x101,[IntPtr]0x68,[IntPtr]::Zero)}
 $result.continued=Authority $a;Snapshot $a 'walked'
 if((RequestCount) -ne 2){throw 'Unexpected GM item request count'}
 $result.status='native_GUI_travel_flow_passed_independent_verification_pending';Save
} catch {
 $result.status='failed';$result.error=$_.Exception.Message;$result.stack=$_.ScriptStackTrace
 foreach($client in $clients){try{Snapshot $client 'failure'}catch{}};Save;throw
} finally {
 foreach($client in $clients){if(!$client.Process.HasExited){[void][OwnedTeleportWindow]::PostMessage($client.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero);if(!$client.Process.WaitForExit(5000)){StopOwned $client.Process ('127.0.0.1:'+$ready.port)}};$client.Process.Dispose()}
 Set-Content -LiteralPath (Join-Path $root 'travel-test-done') -Value 'done';Save
}
