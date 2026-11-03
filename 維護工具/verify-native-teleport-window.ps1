#Requires -Version 7.0
param([ValidateSet('Debug','Release')][string]$BuildProfile='Release',[ValidateSet('cross-endpoint','timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry')][string]$Scenario='cross-endpoint',[switch]$ReplayStaleCommands,[ValidateSet('lito-legacy','tw13081901-v1')][string]$ResourceProfile='lito-legacy',[string]$CommonCatalogPath='',[switch]$ExtendedItemDetails)
# Only owned native windows on the unswitched test desktop receive messages.
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot;$project=Join-Path $taskRoot 'LiTo381-Modern'
$commonCatalogSHA=$null
if($ResourceProfile -eq 'tw13081901-v1'){
 if(!$CommonCatalogPath){throw 'Shared GUI diagnostic requires an explicit owned common catalog'}
 $CommonCatalogPath=(Resolve-Path -LiteralPath $CommonCatalogPath).Path
 if(!$CommonCatalogPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase) -or ((Get-Item -LiteralPath $CommonCatalogPath).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Shared GUI catalog outside owned rebuild files'}
 $commonCatalogSHA=(Get-FileHash -LiteralPath $CommonCatalogPath).Hash
 if($commonCatalogSHA -ne 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'){throw 'Shared GUI catalog differs from reviewed current source'}
}elseif($CommonCatalogPath){throw 'GUI common catalog requires explicit shared profile'}
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $project ('target/current/native-teleport-gui/'+$stamp)
$out=Join-Path $taskRoot ('驗證紀錄/current/native-teleport-gui/'+$stamp)
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
$result=[ordered]@{status='running';resource_profile=$ResourceProfile;extended_item_details=$ExtendedItemDetails.IsPresent;common_catalog_sha256=$commonCatalogSHA;replay_stale_commands=$ReplayStaleCommands.IsPresent;stale_command_replays=@();scenario=$Scenario;runtime=$root;output=$out;jar_sha256=$jarSha;exe_sha256=(Get-FileHash -LiteralPath $exe).Hash;steps=@();physical_input_verified=$false;private_desktop=$true;product_complete=$false}
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
 $worldShot=$name -in @('town','new-battle','reentered','recovery-moved');$settle=[Diagnostics.Stopwatch]::StartNew()
 Start-Sleep -Milliseconds 180
 [void][OwnedTeleportWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)
 $rect=[OwnedTeleportWindow]::PhysicalRect($client.Window);$until=[DateTime]::UtcNow.AddSeconds(3);$probe=@()
 do {
  $bitmap=[System.Drawing.Bitmap]::new($rect.Right-$rect.Left,$rect.Bottom-$rect.Top);$graphics=[System.Drawing.Graphics]::FromImage($bitmap);$dc=$graphics.GetHdc()
  try{if(![OwnedTeleportWindow]::PrintWindow($client.Window,$dc,2)){throw 'PrintWindow failed'}}finally{$graphics.ReleaseHdc($dc);$graphics.Dispose()}
  $colored=0
  if($worldShot){for($gy=0;$gy -lt 8;$gy++){for($gx=0;$gx -lt 16;$gx++){$c=$bitmap.GetPixel([int][Math]::Round($bitmap.Width*(0.25+0.5*$gx/15)),[int][Math]::Round($bitmap.Height*(0.25+0.25*$gy/7)));$maximum=[Math]::Max($c.R,[Math]::Max($c.G,$c.B));$minimum=[Math]::Min($c.R,[Math]::Min($c.G,$c.B));if($maximum -gt 20 -and $maximum-$minimum -gt 15){$colored++}}};$probe+=@($colored)}
  if(!$worldShot -or $colored -ge 10 -or [DateTime]::UtcNow -ge $until){break}
  $bitmap.Dispose();Start-Sleep -Milliseconds 60;[void][OwnedTeleportWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)
 }while($true)
 $path=Join-Path $out ($client.Role+'-'+$name+'.png');try{$bitmap.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)}finally{$bitmap.Dispose()};$settle.Stop()
 $script:steps+=@{role=$client.Role;step=$name;status=(TextOf $client 204);rental_money=$(if($true){TextOf $client 223}else{''});rental_body=$(if($true){TextOf $client 220}else{''});inventory_button=(TextOf $client 114);controls=@{connect=(Enabled $client 101);cancel=(Enabled $client 104);inventory=(Enabled $client 114);npc_visible=[OwnedTeleportWindow]::IsWindowVisible((C $client 220))};terrain_paint_probe=$probe;scene_settle_ms=$settle.ElapsedMilliseconds;png=$path;png_sha256=(Get-FileHash -LiteralPath $path).Hash};Save
 if($worldShot -and $colored -lt 10){throw 'Actual world terrain was not painted within bounded capture wait'}
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
 if(!$row){throw 'Current owned authority absent'};return ($row|ConvertFrom-Json)
}
function Bootstrap($client) {
 Click $client 101;WaitFor {Enabled $client 102} 'login ready';SetText $client 201 $client.Account;SetText $client 202 'TestOnlyPassword';Click $client 102
 WaitFor {Enabled $client 103} 'source ready';SelectCombo $client 0;Click $client 103;WaitFor {Enabled $client 105} 'GAME ready';Click $client 105
 WaitFor {(Enabled $client 106) -or (Enabled $client 103)} 'directory notice';if(Enabled $client 106){Click $client 106}
 WaitFor {Enabled $client 103} 'directory';SelectCombo $client 0;Click $client 103;WaitFor {Enabled $client 127} 'rental set ready';Snapshot $client 'rental-selection'
 [void][OwnedTeleportWindow]::Send((C $client 217),0x14e,1,0);Click $client 127;WaitFor {Enabled $client 128} 'rental accepted';Click $client 128
 WaitFor {(Enabled $client 129) -and (TextOf $client 204).Contains('地圖 500')} 'original town rendered';Snapshot $client 'town'
}
function StepWorld($client,$step) {
 $before=Authority $client;if($before.x -ne $step.before[0] -or $before.y -ne $step.before[1]){throw 'GUI route authority differs'}
 [void][OwnedTeleportWindow]::Send($client.Window,0x201,0,(20 -bor (20 -shl 16)));Start-Sleep -Milliseconds 850
 [void][OwnedTeleportWindow]::PostMessage($client.Window,0x100,[IntPtr]$step.key,[IntPtr]::Zero)
 [void][OwnedTeleportWindow]::PostMessage($client.Window,0x101,[IntPtr]$step.key,[IntPtr]::Zero)
 WaitFor {$state=Authority $client;$state.x -eq $step.after[0] -and $state.y -eq $step.after[1]} 'single tapped GUI exact authoritative pair'
 $state=Authority $client;if($state.x -ne $step.after[0] -or $state.y -ne $step.after[1]){throw 'GUI route step corrected'}
 $script:steps+=@{role=$client.Role;step='GUI-key-walk';before=$step.before;after=$step.after;key=$step.key};Save
}
function PrepareTeleport($client) {
 foreach($step in $route.steps){StepWorld $client $step}
 Click $client 129;WaitFor {Enabled $client 130} 'NPC controls';SelectComboMatch $client 219 '傳送師-尤依琳';Click $client 130
 WaitFor {Enabled $client 134} 'original teleport links';Snapshot $client 'teleport-dialog'
 $combo=C $client 224;$count=[OwnedTeleportWindow]::Send($combo,0x146,0,0);$labels=@();for($i=0;$i -lt $count;$i++){$labels+=@([OwnedTeleportWindow]::ComboRow($combo,$i))}
 $result[($client.Role+'-teleport-labels')]=$labels;Save
 foreach($choice in @(@{label='祝福之地';id=7311;step='original-B-image'},@{label='勇氣之地';id=7312;step='original-C-image'})){
  SelectComboMatch $client 224 $choice.label;Snapshot $client $choice.step
  $window=[OwnedTeleportWindow]::PhysicalRect($client.Window);$control=[OwnedTeleportWindow]::PhysicalRect((C $client 224))
  $script:steps[-1].glyph=@{id=$choice.id;dpi=[OwnedTeleportWindow]::GetDpiForWindow($client.Window);control_rect=@(($control.Left-$window.Left),($control.Top-$window.Top),($control.Right-$window.Left),($control.Bottom-$window.Top))};Save
 }
 $needle=if($client.Role -eq 'a'){'祝福之地'}else{'勇氣之地'};SelectComboMatch $client 224 $needle
 $before=Authority $client;$result[($client.Role+'-before')]=$before;Save
}
function Transfer($client) {
 PrepareTeleport $client;Click $client 134
 WaitFor {if(Enabled $client 106){Click $client 106};(TextOf $client 304) -eq '頻道' -and (Enabled $client 103) -and !([OwnedTeleportWindow]::IsWindowVisible((C $client 127)))} 'destination continuation'
 if([OwnedTeleportWindow]::IsWindowVisible((C $client 220))){throw 'Old dialog remained visible'}
 Snapshot $client 'destination';$result[($client.Role+'-destination')]=Authority $client;Save
 SelectCombo $client 0;Click $client 103;WaitFor {Enabled $client 119} 'destination lobby';Snapshot $client 'destination-lobby'
}

function ReplayOldCommands($client,[string]$stage,[int[]]$ids) {
 if(!$ReplayStaleCommands){return}
 $before=@{status=(TextOf $client 204);inventory=(TextOf $client 114);connect=(Enabled $client 101);npc_visible=[OwnedTeleportWindow]::IsWindowVisible((C $client 220))}
 $attempts=@()
 foreach($id in $ids){
  $control=C $client $id
  $enabled=[OwnedTeleportWindow]::IsWindowEnabled($control);$visible=[OwnedTeleportWindow]::IsWindowVisible($control)
  # Replay a queued own-control notification, including hidden/disabled controls.
  # Each HWND is checked against this client's held PID by C before delivery.
  [void][OwnedTeleportWindow]::Send($client.Window,0x111,$id,$control.ToInt64())
  $attempts+=@{id=$id;control_enabled=$enabled;control_visible=$visible;feedback=(TextOf $client 204)}
 }
 $after=@{inventory=(TextOf $client 114);connect=(Enabled $client 101);npc_visible=[OwnedTeleportWindow]::IsWindowVisible((C $client 220))}
 if($stage -ne 'immediate-repeat' -and ($before.inventory -ne $after.inventory -or $before.connect -ne $after.connect -or $before.npc_visible -ne $after.npc_visible)){throw 'Stale command changed retired/recovered UI authority'}
 $result.stale_command_replays+=@{role=$client.Role;stage=$stage;before=$before;after=$after;attempts=$attempts};Save
}
function RecoverTeleport($client) {
 PrepareTeleport $client;$timer=[Diagnostics.Stopwatch]::StartNew();Click $client 134
 ReplayOldCommands $client 'immediate-repeat' @(134,134,134,134,134,134,134,134)
 if($Scenario -notin @('timeout','refused','wrong-service')) {
  WaitFor {(Get-NetTCPConnection -OwningProcess $client.Process.Id -RemotePort $ready.destination_port -State Established -ErrorAction SilentlyContinue|Measure-Object).Count -gt 0} 'owned destination socket established'
  if($timer.ElapsedMilliseconds -ge 4500){throw 'Cancel missed the destination handshake window'}
  Snapshot $client 'destination-pending'
  ReplayOldCommands $client 'destination-connecting' @(134,135,131,132,133,127,128,130)
  if($Scenario -eq 'cancel-late') {
   $samples=0
   while($timer.ElapsedMilliseconds -lt 3500){if(!(Enabled $client 104) -or (Enabled $client 101) -or (TextOf $client 114) -ne '背包（0）'){throw 'Late-cancel pending state changed early'};$samples++;Start-Sleep -Milliseconds 25}
   $result[($client.Role+'-pre-cancel-ms')]=$timer.ElapsedMilliseconds;$result[($client.Role+'-pending-samples')]=$samples;Save
  }
  Click $client 104
  WaitFor {(Enabled $client 101) -and !(Enabled $client 104) -and (TextOf $client 204) -eq '尚未連線。'} 'cancelled and transport reclaimed'
 } else {
  WaitFor {(Enabled $client 101) -and (TextOf $client 204).StartsWith('連線失敗：')} 'bounded no-response failure'
 }
 if((TextOf $client 114) -ne '背包（0）' -or (Enabled $client 114) -or [OwnedTeleportWindow]::IsWindowVisible((C $client 220))){throw 'Old inventory/dialog survived cancellation or failure'}
 $result[($client.Role+'-stop-ms')]=$timer.ElapsedMilliseconds;Snapshot $client 'stopped';Save
 ReplayOldCommands $client 'after-cancel' @(134,135,131,132,133,127,128,130)
 if($Scenario -eq 'cancel-expiry') {
  $samples=0
  while($timer.ElapsedMilliseconds -lt 33000){if(!(Enabled $client 101) -or (Enabled $client 104) -or (TextOf $client 114) -ne '背包（0）'){throw 'Cancelled GUI resumed in background'};$samples++;Start-Sleep -Milliseconds 40}
  $result[($client.Role+'-closed-samples')]=$samples;$result[($client.Role+'-natural-wait-ms')]=$timer.ElapsedMilliseconds;Snapshot $client 'expired';Save
 }
 Click $client 101;WaitFor {Enabled $client 102} 'fresh LOGIN';SetText $client 201 $client.Account;SetText $client 202 'TestOnlyPassword';Click $client 102
 WaitFor {Enabled $client 103} 'fresh source';SelectCombo $client 0;Click $client 103;WaitFor {Enabled $client 105} 'fresh GAME';Click $client 105
 if($Scenario -eq 'cancel-expiry') {
  WaitFor {if(Enabled $client 106){Click $client 106};(TextOf $client 304) -eq '角色' -and (Enabled $client 103)} 'expired stage requires manual selection'
  Snapshot $client 'fresh-directory';SelectCombo $client 0;Click $client 103
 }
 WaitFor {if(Enabled $client 106){Click $client 106};(Enabled $client 127) -and [OwnedTeleportWindow]::IsWindowVisible((C $client 127))} 'recovered character and rental controls'
 Snapshot $client 'continued';$result[($client.Role+'-continued')]=Authority $client;Save
 [void][OwnedTeleportWindow]::Send((C $client 217),0x14e,1,0);Click $client 127;WaitFor {Enabled $client 128} 'new rental scene permitted';Click $client 128
 WaitFor {(Enabled $client 129) -and (TextOf $client 204).StartsWith('地圖 500')} 'recovered original town rendered'
 Snapshot $client 'reentered';$result[($client.Role+'-reentered')]=Authority $client;Save
 ReplayOldCommands $client 'new-world-old-controls' @(134,135,131,132,133,127,128,130)
 $key=if($client.Role -eq 'a'){0x68}else{0x26};$after=if($client.Role -eq 'a'){@(32770,32817)}else{@(32771,32817)}
 StepWorld $client ([pscustomobject]@{before=@(32770,32818);after=$after;key=$key});Snapshot $client 'recovery-moved'
 $result[($client.Role+'-after-move')]=Authority $client;Save
}

Save
try {
 $compilationInputs=@{}
 foreach($folder in @('src','tests')){foreach($file in Get-ChildItem -LiteralPath (Join-Path $project $folder) -Recurse -File){$relative=[IO.Path]::GetRelativePath($taskRoot,$file.FullName).Replace('\','/');$compilationInputs[$relative]=(Get-FileHash -LiteralPath $file.FullName).Hash}}
 foreach($name in @('Cargo.toml','Cargo.lock','build.rs','.cargo/config.toml')){$relative='LiTo381-Modern/'+$name;$compilationInputs[$relative]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $relative)).Hash}
 & (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile *> (Join-Path $out 'build-before-server.log')
 $result.exe_sha256=(Get-FileHash -LiteralPath $exe).Hash
 foreach($name in $compilationInputs.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $compilationInputs[$name]){throw ('Compilation input changed during build: '+$name)}}
 @{status='completed_build';target='x86_64-pc-windows-msvc';profile=$BuildProfile;files=$compilationInputs;main_sha256=$result.exe_sha256}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $out 'compilation-inputs.json') -Encoding utf8
 $result.compilation_inputs_sha256=(Get-FileHash -LiteralPath (Join-Path $out 'compilation-inputs.json')).Hash;$result.complete_build_inputs_verified=$true;Save
 $pins=@{};foreach($name in @('維護工具/verify-native-teleport-window.ps1','維護工具/run-isolated-native-teleport-window-test.ps1','LiTo381-Modern/tests/oracle/NativeTeleportHarness.java','LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java','LiTo381-Modern/tests/oracle/NativePeerHarness.java','LiTo381-Modern/tests/fixtures/native-gui/teleport-route.json','LiTo381-Modern/src/modern_lito/native_room.rs','LiTo381-Modern/src/modern_lito/native_login.rs','LiTo381-Modern/src/modern_lito/native_login_window.rs','LiTo381-Modern/src/modern_lito/native_rental_window.rs','LiTo381-Modern/src/modern_lito/native_npc_document.rs','LiTo381-Modern/src/modern_lito/native_npc_images.rs','LiTo381-Modern/tests/native_npc_images.rs','LiTo381-Modern/Cargo.toml','LiTo381-Modern/.cargo/config.toml')){$pins[$name]=(Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash}
 $result.source_pins=$pins;Save
 $archive=Join-Path $out 'source-snapshot';foreach($name in $pins.Keys){$target=Join-Path $archive $name;New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force|Out-Null;Copy-Item -LiteralPath (Join-Path $taskRoot $name) -Destination $target}
 $route=Get-Content -LiteralPath (Join-Path $project 'tests/fixtures/native-gui/teleport-route.json') -Raw|ConvertFrom-Json;if($jarSha -notin @($route.compatible_jar_sha256)){throw 'Route version differs'}
 & (Join-Path $jdk 'javac.exe') '-J-Dfile.encoding=UTF-8' -encoding UTF-8 -cp $jar -d $classes (Join-Path $project 'tests/oracle/NativePeerHarness.java') (Join-Path $project 'tests/oracle/NativeTeleportHarness.java') (Join-Path $project 'tests/oracle/NativeTeleportFailureHarness.java') *> (Join-Path $out 'javac.log');if($LASTEXITCODE -ne 0){throw 'Harness compile failed'}
 $harness=if($Scenario -eq 'cross-endpoint'){'local.lito381.server.NativeTeleportHarness'}else{'local.lito381.server.NativeTeleportFailureHarness'}
 $harnessScenario=if($Scenario -eq 'cross-endpoint'){'gui-cross-endpoint'}else{'gui-'+$Scenario}
 $serverArgs=@('-Dfile.encoding=UTF-8','-Xverify:all',('-Dlito.resourceProfile='+$ResourceProfile),('-Dlito.extendedItemDetails='+$ExtendedItemDetails.IsPresent.ToString().ToLowerInvariant()))
 if($CommonCatalogPath){$serverArgs+=@('-Dlito.commonCatalog="'+$CommonCatalogPath+'"')}
 $serverArgs+=@('-cp',('"'+$classes+';'+$jar+'"'),$harness,('"'+$root+'"'),$harnessScenario)
 $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList $serverArgs -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $out 'server.log') -RedirectStandardError (Join-Path $out 'server.stderr.txt')
 $serverIdentity=Get-CimInstance Win32_Process -Filter ('ProcessId='+$server.Id)
 if(!$serverIdentity -or [Math]::Abs(($serverIdentity.CreationDate.ToUniversalTime()-$server.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$serverIdentity.CommandLine.Contains('-Dlito.resourceProfile='+$ResourceProfile) -or !$serverIdentity.CommandLine.Contains($root)){throw 'Owned GUI server PID/creation/profile mismatch'}
 @{pid=$server.Id;creation_utc=$server.StartTime.ToUniversalTime().ToString('o');resource_profile=$ResourceProfile;extended_item_details=$ExtendedItemDetails.IsPresent;common_catalog_sha256=$commonCatalogSHA;exe_sha256=$result.exe_sha256;jar_sha256=$jarSha;command_line=$serverIdentity.CommandLine}|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $out 'server-launch-identity.json') -Encoding utf8
 WaitFor {Test-Path -LiteralPath (Join-Path $root 'ready.json')} 'server ready';$ready=Get-Content -LiteralPath (Join-Path $root 'ready.json') -Raw|ConvertFrom-Json
 if($Scenario -eq 'refused' -and !$ready.destination_refused){throw 'Refused fixture was not a closed owned loopback port'}
 if($Scenario -eq 'wrong-service' -and !$ready.destination_wrong_service){throw 'Wrong-service fixture is not a formal LOGIN endpoint'}
 $result.destination_wrong_service=$ready.destination_wrong_service;$result.destination_refused=$ready.destination_refused;Save
 $a=Launch 'a';$b=Launch 'b';Bootstrap $a;Bootstrap $b
 if($Scenario -eq 'cross-endpoint'){Transfer $a;Transfer $b
 Click $a 119;SetText $a 213 '傳送GUI接續';SetText $a 214 '2';Click $a 121;WaitFor {Enabled $a 112} 'created room'
 Click $b 108;WaitFor {Enabled $b 109} 'room refresh';SelectRow $b '傳送GUI接續';Click $b 109;WaitFor {Enabled $b 112} 'peer join';Click $b 118
 WaitFor {$h=C $b 205;$count=[OwnedTeleportWindow]::Send($h,0x18b,0,0);$found=$false;for($n=0;$n -lt $count;$n++){$row=[OwnedTeleportWindow]::Row($h,$n);if($row.Contains('PeerFixture2') -and $row.Contains('隊伍2')){$found=$true}};$found -and (Enabled $b 110)} 'opposing force'
 if(!(Enabled $a 136) -or !(Enabled $a 137) -or (Enabled $b 136) -or (Enabled $b 137)){throw 'Map controls do not follow current room owner'}
 foreach($client in @($a,$b)){foreach($id in @(136,137)){if(![OwnedTeleportWindow]::IsWindowVisible((C $client $id))){throw 'Map control hidden or shadowed'}}}
 Click $a 137;WaitFor {(TextOf $a 204).Contains('地圖 514') -and (TextOf $b 204).Contains('地圖 514') -and (Enabled $a 136)} 'authoritative map 514 on both native windows'
 Snapshot $a 'map-next';Snapshot $b 'map-next'
 Click $a 136;WaitFor {(TextOf $a 204).Contains('地圖 564') -and (TextOf $b 204).Contains('地圖 564') -and (Enabled $a 137)} 'authoritative map 564 restored'
 Snapshot $a 'map-restored';Snapshot $b 'map-restored'
 $result.room_map_selection=@{owner='a';next=514;restored=564;previous_control=136;next_control=137;peer_controls_disabled=(!(Enabled $b 136) -and !(Enabled $b 137));visible=$true};Save
 Click $a 110;Click $b 110;WaitFor {(Authority $a).phase -eq 'BATTLE' -and (Authority $b).phase -eq 'BATTLE' -and (TextOf $a 204).StartsWith('地圖 564') -and (TextOf $b 204).StartsWith('地圖 564')} 'new shared original world'
 Snapshot $a 'new-battle';Snapshot $b 'new-battle';$result.battle=@{a=(Authority $a);b=(Authority $b)};Save
 } else {RecoverTeleport $a;RecoverTeleport $b}
 foreach($client in $clients){[void][OwnedTeleportWindow]::PostMessage($client.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero)}
 foreach($client in $clients){if(!$client.Process.WaitForExit(15000)){throw 'GUI shutdown exceeded'}}
 if(!$server.WaitForExit(15000) -or $server.ExitCode -ne 0){throw 'Server cleanup failed'}
 $result.server=Get-Content -LiteralPath (Join-Path $root 'result.json') -Raw|ConvertFrom-Json
 if($result.server.actual_client_sessions -ne $(if($Scenario -eq 'cross-endpoint'){6}elseif($Scenario -eq 'wrong-service'){10}else{8}) -or !$result.server.memberships_empty -or $(if($Scenario -eq 'cross-endpoint'){!$result.server.staged_loadouts_empty}else{$result.server.staged_loadouts_after_reentry -ne 0}) -or $result.server.authority_mutations -ne 0){throw 'Cleanup mismatch'}
 foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw 'Source drift'}}
 if((Get-FileHash -LiteralPath $exe).Hash -ne $result.exe_sha256 -or (Get-FileHash -LiteralPath $jar).Hash -ne $jarSha){throw 'Owned binary/reference drift'}
 $result.observed_source_before_sha256=$observedSourceBefore;$result.observed_source_after_sha256=(Get-FileHash -LiteralPath $source).Hash;$result.reference_is_current=($observedSourceBefore -eq $jarSha -and $result.observed_source_after_sha256 -eq $jarSha)
 foreach($name in $compilationInputs.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $compilationInputs[$name]){throw ('Compilation input changed during GUI run: '+$name)}}
 if($CommonCatalogPath -and (Get-FileHash -LiteralPath $CommonCatalogPath).Hash -ne $commonCatalogSHA){throw 'Shared GUI catalog changed during runtime'}
 $result.inputs_unchanged=$true;$result.status=if($Scenario -eq 'cross-endpoint'){'verified_native_teleport_GUI_cross_endpoint_shared_battle'}else{'verified_native_teleport_GUI_'+$Scenario+'_and_fresh_reentry'};Save
} catch {$result.status='failed';$result.error=$_.Exception.Message;$result.error_stack=$_.ScriptStackTrace;foreach($client in $clients){try{Snapshot $client 'failure'}catch{}};Save;throw}
finally {
 foreach($client in $clients){if(!$client.Process.HasExited){[void][OwnedTeleportWindow]::PostMessage($client.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero);if(!$client.Process.WaitForExit(5000)){StopOwned $client.Process ('127.0.0.1:'+$ready.port)}};$client.Process.Dispose()}
 if($server){if(!$server.HasExited){StopOwned $server $root};$server.Dispose()}
 foreach($name in @('ready.json','result.json','server-trace.jsonl','staged-trace.jsonl')){if(Test-Path -LiteralPath (Join-Path $root $name)){Copy-Item -LiteralPath (Join-Path $root $name) -Destination (Join-Path $out $name)}}
 Save
}
Write-Output $out
