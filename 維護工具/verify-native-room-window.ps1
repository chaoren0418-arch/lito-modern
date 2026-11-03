#Requires -Version 7.0
param([ValidateSet("room-admin","rental-shop","rental-hotel","rental-merchants")][string]$ActionProfile="room-admin",[switch]$NpcNavigation,[switch]$BattleWithdrawal,[switch]$BattleCooldown,[switch]$HotelMiddle,[ValidateSet("Debug","Release")][string]$BuildProfile="Debug",[string]$ReferenceSnapshot)
# Only windows belonging to the two processes launched by this test are controlled.
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
if($BattleCooldown -and !$BattleWithdrawal){throw 'BattleCooldown requires BattleWithdrawal'}
if($BattleWithdrawal -and $ActionProfile -ne 'room-admin'){throw 'BattleWithdrawal requires room-admin'}
if($NpcNavigation -and $ActionProfile -ne 'rental-shop'){throw 'NpcNavigation requires rental-shop'}
if($HotelMiddle -and $ActionProfile -ne 'rental-hotel'){throw 'HotelMiddle requires rental-hotel'}
$taskRoot=Split-Path -Parent $PSScriptRoot;$project=Join-Path $taskRoot 'LiTo381-Modern'
$stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')
$root=Join-Path $project ('target/current/native-room-gui/'+$stamp)
$out=Join-Path $taskRoot ('驗證紀錄/current/native-room-gui/'+$stamp)
New-Item -ItemType Directory -Path $root,$out -Force|Out-Null
$previousTemp=$env:TEMP;$previousTmp=$env:TMP;$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP
$referencePath=if($ReferenceSnapshot){$ReferenceSnapshot}else{Join-Path $taskRoot '驗證紀錄/current/reference-snapshot.json'}
$pin=Get-Content -Raw -LiteralPath $referencePath|ConvertFrom-Json
$jar=$pin.snapshot
$exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
$jdk='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin';$classes=Join-Path $root 'oracle'
New-Item -ItemType Directory -Path $classes -Force|Out-Null
if((Get-FileHash -LiteralPath $jar).Hash -ne $pin.sha256 -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $pin.sha256){throw 'Reference drift'}
# Cold compilation must finish before owned server accept/fixture deadlines start.
& (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $BuildProfile *> (Join-Path $out 'build-before-server.log')
if($LASTEXITCODE -ne 0){throw 'Native widget prebuild failed'}
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System;using System.Text;using System.Runtime.InteropServices;
public static class OwnedRoomWindow {
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
 [DllImport("user32.dll",CharSet=CharSet.Unicode,EntryPoint="GetPropW")]public static extern IntPtr GetProp(IntPtr h,string key);
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
$clients=@();$server=$null;$steps=@();$result=[ordered]@{status='running';action_profile=$ActionProfile;output=$out;runtime=$root;jar_sha256=$pin.sha256;exe_sha256=(Get-FileHash -LiteralPath $exe).Hash;steps=@();physical_input_verified=$false;ime_verified=$false;multi_dpi_verified=$false;product_complete=$false}
function Save {$result.steps=$script:steps;$result|ConvertTo-Json -Depth 15|Set-Content -LiteralPath (Join-Path $out 'receipt.json') -Encoding utf8}
function AssertRouteReference([string]$File,[string]$Declared) {
 if($Declared -eq $pin.sha256){return}
 if(!$pin.route_geometry_compatibility -or (Get-FileHash -LiteralPath $pin.route_geometry_compatibility).Hash -ne $pin.route_geometry_compatibility_sha256){throw 'Route geometry bridge absent or changed'}
 $bridge=Get-Content -LiteralPath $pin.route_geometry_compatibility -Raw|ConvertFrom-Json
 $relative=[IO.Path]::GetRelativePath($taskRoot,$File).Replace('\','/')
 $fixture=$bridge.fixtures.PSObject.Properties[$relative]
 if($bridge.status -ne 'verified_route_geometry_catalogues_and_Npc_except_name' -or $Declared -ne $bridge.old_sha256 -or $bridge.current_sha256 -ne $pin.sha256 -or !$fixture -or (Get-FileHash -LiteralPath $File).Hash -ne $fixture.Value -or (Get-FileHash -LiteralPath $bridge.old_jar).Hash -ne $bridge.old_sha256 -or (Get-FileHash -LiteralPath $bridge.current_jar).Hash -ne $bridge.current_sha256){throw 'Route geometry proof mismatch'}
 $result.route_geometry_bridge=@{path=$pin.route_geometry_compatibility;sha256=$pin.route_geometry_compatibility_sha256;old_fixture_jar_sha256=$Declared;name_change_excluded=$true}
 Save
}
function StopOwned([Diagnostics.Process]$Process,[string]$Marker) {
 $Process.Refresh();if($Process.HasExited){return}
 $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$Process.Id)
 if(!$live){$Process.Refresh();if($Process.HasExited){return};throw 'Owned process disappeared without exit observation'}
 $command=$live.CommandLine.Replace('/','\').Replace('\\','\');$mark=$Marker.Replace('/','\').Replace('\\','\')
 if([Math]::Abs(($live.CreationDate.ToUniversalTime()-$Process.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$command.Contains($mark)){throw 'Owned process identity mismatch'}
 $Process.Kill();$Process.WaitForExit()
}
function WaitFor([scriptblock]$Condition,[string]$Reason){$until=[DateTime]::UtcNow.AddSeconds(18);while(!(& $Condition)){if([DateTime]::UtcNow -gt $until){throw ('Timeout: '+$Reason)};Start-Sleep -Milliseconds 50}}
function C($client,[int]$id){$h=[OwnedRoomWindow]::GetDlgItem($client.Window,$id);if($h -eq [IntPtr]::Zero){throw ('Missing control '+$id)};[uint32]$owner=0;[void][OwnedRoomWindow]::GetWindowThreadProcessId($h,[ref]$owner);if($owner -ne $client.Process.Id){throw 'Foreign control'};return $h}
function Enabled($client,[int]$id){return [OwnedRoomWindow]::IsWindowEnabled((C $client $id))}
function TextOf($client,[int]$id){return [OwnedRoomWindow]::Text((C $client $id))}
function SetText($client,[int]$id,[string]$text){[OwnedRoomWindow]::SetText((C $client $id),$text)}
function Click($client,[int]$id){$h=C $client $id;if(![OwnedRoomWindow]::IsWindowVisible($h) -or ![OwnedRoomWindow]::IsWindowEnabled($h)){throw ('Control not available '+$id)};[void][OwnedRoomWindow]::Send($h,0xf5,0,0)}
function SelectCombo($client,[int]$index){[void][OwnedRoomWindow]::Send((C $client 203),0x14e,$index,0)}
function SelectRow($client,[string]$match){$h=C $client 205;$count=[OwnedRoomWindow]::Send($h,0x18b,0,0);$found=-1;for($n=0;$n -lt $count;$n++){if([OwnedRoomWindow]::Row($h,$n).Contains($match)){$found=$n;break}};if($found -lt 0){throw ('List row absent '+$match)};[void][OwnedRoomWindow]::Send($h,0x186,$found,0);[void][OwnedRoomWindow]::Send($client.Window,0x111,(205 -bor (1 -shl 16)),$h.ToInt64())}
function SelectComboMatch($client,[int]$id,[string]$match){$h=C $client $id;$count=[OwnedRoomWindow]::Send($h,0x146,0,0);$labels=@();for($i=0;$i -lt $count;$i++){$label=[OwnedRoomWindow]::ComboRow($h,$i);$labels+=@($label);if($label.Contains($match)){[void][OwnedRoomWindow]::Send($h,0x14e,$i,0);return}};throw ('Combo match absent: '+$match+' in '+($labels -join ', '))}
function SelectMerchantRow($client,[string]$match){$h=C $client 221;$count=[OwnedRoomWindow]::Send($h,0x18b,0,0);for($n=0;$n -lt $count;$n++){if([OwnedRoomWindow]::Row($h,$n).Contains($match)){[void][OwnedRoomWindow]::Send($h,0x186,$n,0);[void][OwnedRoomWindow]::Send($client.Window,0x111,(221 -bor (1 -shl 16)),$h.ToInt64());return}};throw ('Merchant row absent '+$match)}
function Snapshot($client,[string]$name){
 # Match the qualified teleport capture: let native child paints/selection transitions settle.
 Start-Sleep -Milliseconds 250
 [void][OwnedRoomWindow]::RedrawWindow($client.Window,[IntPtr]::Zero,[IntPtr]::Zero,0x185)
 $rect=[OwnedRoomWindow]::PhysicalRect($client.Window)
 $bitmap=[System.Drawing.Bitmap]::new($rect.Right-$rect.Left,$rect.Bottom-$rect.Top);$graphics=[System.Drawing.Graphics]::FromImage($bitmap);$dc=$graphics.GetHdc()
 try{if(![OwnedRoomWindow]::PrintWindow($client.Window,$dc,2)){throw 'PrintWindow failed'}}finally{$graphics.ReleaseHdc($dc);$graphics.Dispose()}
 $path=Join-Path $out ($client.Role+'-'+$name+'.png');try{$bitmap.Save($path,[System.Drawing.Imaging.ImageFormat]::Png)}finally{$bitmap.Dispose()}
 $choice=$null
 if($ActionProfile -ne 'room-admin'){
  $h=C $client 224;$bounds=[OwnedRoomWindow]::PhysicalRect($h);$index=[OwnedRoomWindow]::Send($h,0x147,0,0)
  $choice=@{visible=[OwnedRoomWindow]::IsWindowVisible($h);enabled=[OwnedRoomWindow]::IsWindowEnabled($h);index=$index;label=$(if($index -ge 0){[OwnedRoomWindow]::ComboRow($h,$index)}else{''});dpi=[OwnedRoomWindow]::GetDpiForWindow($client.Window);rect=@(($bounds.Left-$rect.Left),($bounds.Top-$rect.Top),($bounds.Right-$rect.Left),($bounds.Bottom-$rect.Top))}
  $choice.paint=@{};foreach($key in @('Stage','Flags','Item','Foreground','Length','Dropped','Height','OldROP','OldBackground')){$choice.paint[$key]=[OwnedRoomWindow]::GetProp($h,('LiTo.NpcChoice.'+$key)).ToInt64()-1}

 }
 $script:steps+=@{capture_settle_ms=250;choice=$choice;role=$client.Role;step=$name;status=(TextOf $client 204);rental_money=$(if($ActionProfile -ne 'room-admin'){TextOf $client 223}else{''});rental_body=$(if($ActionProfile -ne 'room-admin'){TextOf $client 220}else{''});png=$path;png_sha256=(Get-FileHash -LiteralPath $path).Hash};Save
}
function LaunchClient([string]$role){
 $before=@(Get-Process -Name LiToModern -ErrorAction SilentlyContinue|Select-Object -ExpandProperty Id)
 & (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildProfile $BuildProfile -NativeLoginEndpoint ('127.0.0.1:'+$ready.port) *> (Join-Path $out ('launch-'+$role+'.log'))
 $limit=[DateTime]::UtcNow.AddSeconds(5);$owned=@()
 do{
  $owned=@(Get-CimInstance Win32_Process -Filter "Name = 'LiToModern.exe'"|Where-Object {$_.ProcessId -notin $before -and $_.ExecutablePath -and [IO.Path]::GetFullPath($_.ExecutablePath) -eq [IO.Path]::GetFullPath($exe) -and $_.CommandLine -and $_.CommandLine.Contains('127.0.0.1:'+$ready.port)})
  if($owned.Count -gt 1){throw 'Multiple owned GUI processes for unique endpoint'}
  if($owned.Count -eq 0){Start-Sleep -Milliseconds 50}
 }while($owned.Count -eq 0 -and [DateTime]::UtcNow -lt $limit)
 if($owned.Count -ne 1){throw 'Cannot identify uniquely owned GUI process'}
 $p=Get-Process -Id $owned[0].ProcessId;$client=[pscustomobject]@{Role=$role;Process=$p;Window=[IntPtr]::Zero};$script:clients+=@($client)
 WaitFor {$client.Window=[OwnedRoomWindow]::Find($p.Id);$client.Window -ne [IntPtr]::Zero} 'native window creation'
 if(![OwnedRoomWindow]::IsWindowVisible($client.Window)){throw 'Launcher created a hidden native window'}
 return $client
}
function SelectCharacter($client,[string]$account){
 Click $client 101;WaitFor {Enabled $client 102} 'login ready';SetText $client 201 $account;SetText $client 202 'TestOnlyPassword';Click $client 102
 WaitFor {Enabled $client 103} 'sources';SelectCombo $client 1;Click $client 103
 WaitFor {Enabled $client 105} 'game handoff';Click $client 105
 WaitFor {(Enabled $client 106) -or (Enabled $client 103)} 'character directory'
 if(Enabled $client 106){Click $client 106};WaitFor {Enabled $client 103} 'character selection';SelectCombo $client 0;Click $client 103
 WaitFor {(TextOf $client 304) -eq '頻道' -and (Enabled $client 103)} 'channel selection'
}
function EnterLobby($client,[string]$account){
 SelectCharacter $client $account;SelectCombo $client 0;Click $client 103
 WaitFor {Enabled $client 119} 'lobby room tools';Snapshot $client 'lobby'
}
function Authority($client){
 $objectId=if($client.Role -eq 'a'){1}else{3};$trace=Join-Path $root 'server-trace.jsonl';$needle='"object_id":'+$objectId+','
 $last=Get-Content -LiteralPath $trace -Tail 120|Where-Object {$_.Contains($needle)}|Select-Object -Last 1
 if(!$last){$last=Get-Content -LiteralPath $trace|Where-Object {$_.Contains($needle)}|Select-Object -Last 1}
 if(!$last){throw ('No formal state for owned player '+$objectId)}
 $state=($last|ConvertFrom-Json).state;if($state.object_id -ne $objectId){throw 'Formal state identity mismatch'};return $state
}

function StepWorld($client,[int]$key,[int]$dx,[int]$dy){
 [void][OwnedRoomWindow]::Send($client.Window,0x201,0,(20 -bor (20 -shl 16)))
 WaitFor {[OwnedRoomWindow]::FocusFor($client.Window) -eq $client.Window} 'owned thread world focus'
 Start-Sleep -Milliseconds 850
 $before=Authority $client
 [void][OwnedRoomWindow]::PostMessage($client.Window,0x100,[IntPtr]$key,[IntPtr]::Zero)
 try{WaitFor {$s=Authority $client; $s.x -ne $before.x -or $s.y -ne $before.y} 'authoritative GUI movement'}
 catch{$result.failed_movement=@{before=@($before.x,$before.y);key=$key;status=(TextOf $client 204);thread_focus_owned=([OwnedRoomWindow]::FocusFor($client.Window) -eq $client.Window)};Snapshot $client 'movement-failure';throw}
 finally{[void][OwnedRoomWindow]::PostMessage($client.Window,0x101,[IntPtr]$key,[IntPtr]::Zero)}
 $after=Authority $client
 if($after.x -ne $before.x+$dx -or $after.y -ne $before.y+$dy){throw 'Movement did not perform the requested single step'}
 $script:steps+=@{role=$client.Role;step='native-key-message-movement';before=@($before.x,$before.y);after=@($after.x,$after.y);key=$key};Save
}
function ExerciseBattleCooldown($attacker,$victim){
 $routeFile=Join-Path $project 'tests/fixtures/native-gui/battle-withdrawal-route.json';$route=Get-Content -LiteralPath $routeFile -Raw|ConvertFrom-Json
 if($route.formal_jar_sha256 -ne $pin.sha256){throw 'Combat movement route source mismatch'}
 $position=Authority $victim;if($position.x -ne $route.target[0] -or $position.y -ne $route.target[1]){throw 'Combat target start differs'}
 foreach($step in $route.steps){$position=Authority $attacker;if($position.x -ne $step.before[0] -or $position.y -ne $step.before[1]){throw 'Combat route start differs'};StepWorld $attacker $step.key ($step.after[0]-$step.before[0]) ($step.after[1]-$step.before[1])}
 Start-Sleep -Milliseconds 850
 $own=Authority $attacker;$peer=Authority $victim;$dx=$peer.x-$own.x;$dy=$peer.y-$own.y
 if([Math]::Max([Math]::Abs($dx),[Math]::Abs($dy)) -ne 1){throw 'Combat target is not adjacent'}
 $rect=[OwnedRoomWindow]::PhysicalClientRect($attacker.Window)
 $dpi=[Math]::Max(96,[OwnedRoomWindow]::GetDpiForWindow($attacker.Window));$cw=$rect.Right;$ch=$rect.Bottom-[Math]::Floor(185*$dpi/96)
 if($cw*600 -le $ch*800){$vw=$cw;$vh=[Math]::Floor($cw*600/800)}else{$vw=[Math]::Floor($ch*800/600);$vh=$ch}
 # Same projection and integer viewport as the native renderer; pick the peer's torso.
 $cx=400+24*($dx+$dy);$cy=300+12*(-$dx+$dy)-35
 $px=[int]([Math]::Floor(($cw-$vw)/2)+[Math]::Floor(($cx+0.5)*$vw/800));$py=[int]([Math]::Floor(($ch-$vh)/2)+[Math]::Floor(($cy+0.5)*$vh/600))
 $result.cooldown_pick=@{canvas=@($cx,$cy);client=@($px,$py);own=@($own.x,$own.y);peer=@($peer.x,$peer.y);viewport=@($vw,$vh);client_size=@($cw,$ch);dpi=$dpi};Snapshot $attacker 'cooldown-target-before';Save
 [void][OwnedRoomWindow]::SendPhysical($attacker.Window,0x201,0,($px -bor ($py -shl 16)))
 [void][OwnedRoomWindow]::SendPhysical($attacker.Window,0x202,0,($px -bor ($py -shl 16)))
 try {WaitFor {(TextOf $attacker 204).Contains('目標：PeerFixture2')} 'actual peer target selection'}catch{Snapshot $attacker 'cooldown-target-missed';throw}
 Snapshot $attacker 'cooldown-target-selected';$result.cooldown_before_hit=@{a=(Authority $attacker);b=(Authority $victim)};Save
 [void][OwnedRoomWindow]::PostMessage($attacker.Window,0x100,[IntPtr]32,[IntPtr]::Zero)
 try{WaitFor {(Authority $victim).hp -lt $peer.hp} 'one authoritative GUI attack'}finally{[void][OwnedRoomWindow]::PostMessage($attacker.Window,0x101,[IntPtr]32,[IntPtr]::Zero)}
 $hit=Authority $victim;if($hit.hp -le 0){throw 'Cooldown fixture attack was lethal'}
 $result.cooldown_after_hit=@{a=(Authority $attacker);b=$hit};Save
 Click $attacker 107
 WaitFor {(TextOf $attacker 204).Contains('戰鬥中無法退出，請停止攻擊並等待 ') -and (Enabled $attacker 107)} 'visible authoritative cooldown refusal'
 Snapshot $attacker 'cooldown-refused';$result.cooldown_refusal_text=TextOf $attacker 204;$timer=[Diagnostics.Stopwatch]::StartNew()
 WaitFor {if((Authority $attacker).phase -ne 'BATTLE' -or (Authority $victim).phase -ne 'BATTLE'){throw 'Battle retired during refused withdrawal'};$timer.ElapsedMilliseconds -ge 11000} 'real cooldown wait without deadline disconnect'
 $result.cooldown_wait_ms=$timer.ElapsedMilliseconds;$timer.Stop()
 $result.cooldown_after_wait=@{a=(Authority $attacker);b=(Authority $victim)};Save
 # The formal server regenerates HP while waiting. The independent packet
 # validator requires exactly one attack and verifies the HP timeline.
 if($result.cooldown_after_wait.b.hp -lt $hit.hp -or !(Enabled $attacker 107)){throw 'Unexpected additional damage or withdrawal did not recover'}
 Snapshot $attacker 'cooldown-after-wait';$result.battle_cooldown_widgets_verified=$true;$result.cooldown_route_sha256=(Get-FileHash -LiteralPath $routeFile).Hash;Save
}
function CitizenDialog($client,[string]$name,[int]$count){
 Click $client 129;SelectComboMatch $client 219 $name;Click $client 130
 WaitFor {Enabled $client 134} 'NPC local pages';SelectComboMatch $client 224 '成為正式市民';Click $client 134
 WaitFor {([OwnedRoomWindow]::Send((C $client 224),0x146,0,0)) -gt 0} 'citizen action page'
 [void][OwnedRoomWindow]::Send((C $client 224),0x14e,0,0)
 Snapshot $client ($name+'-citizen-page')
 Click $client 134
 WaitFor {(@((Authority $client).rental_emblems).Count -eq $count) -and (Enabled $client 130)} 'authoritative emblem grant'
 Snapshot $client ($name+'-emblem');Click $client 129
}
Save
try{
 & (Join-Path $jdk 'javac.exe') -encoding UTF-8 -cp $jar -d $classes (Join-Path $project 'tests/oracle/NativePeerHarness.java') *> (Join-Path $out 'javac.log');if($LASTEXITCODE -ne 0){throw 'Harness compile failed'}
 $server=Start-Process -FilePath (Join-Path $jdk 'java.exe') -ArgumentList @('-Dfile.encoding=UTF-8','-cp',('"'+$classes+';'+$jar+'"'),'local.lito381.server.NativePeerHarness',('"'+$root+'"'),$ActionProfile) -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $out 'server.log') -RedirectStandardError (Join-Path $out 'server.stderr.txt')
 WaitFor {Test-Path -LiteralPath (Join-Path $root 'ready.json')} 'server ready';$ready=Get-Content -Raw -LiteralPath (Join-Path $root 'ready.json')|ConvertFrom-Json
 $a=LaunchClient 'a';$b=LaunchClient 'b'
 if($ActionProfile -eq 'room-admin'){
 EnterLobby $a 'PeerFixture';EnterLobby $b 'PeerFixture2'
 Click $a 119;SetText $a 213 '原生操作房';SetText $a 214 '8';Click $a 216;SetText $a 215 '12';Snapshot $a 'create-form';Click $a 121
 WaitFor {Enabled $a 112} 'created room';Snapshot $a 'created'
 Click $b 108;WaitFor {Enabled $b 109} 'room refresh';SelectRow $b '原生操作房';SetText $b 206 '00';Click $b 109
 WaitFor {(Enabled $b 109) -and (TextOf $b 204).Contains('密碼不正確')} 'wrong password refusal';Snapshot $b 'wrong-password'
 SetText $b 206 '12';Click $b 109;WaitFor {Enabled $b 112} 'peer join'
 if((Enabled $b 120) -or (Enabled $b 124)){throw 'Non-owner admin enabled'}
 Click $a 120;SetText $a 213 '原生設定房';SetText $a 214 '6';SetText $a 215 '12';Snapshot $a 'edit-form';Click $a 121
 WaitFor {(Enabled $a 120) -and (TextOf $a 204).Contains('原生設定房') -and (TextOf $b 204).Contains('原生設定房')} 'edit acknowledgement'
 Click $a 126;WaitFor {(Enabled $a 126) -and (TextOf $a 204).Contains('榮譽戰')} 'next mode';Snapshot $a 'honor-mode'
 Click $a 125;WaitFor {(Enabled $a 125) -and (TextOf $a 204).Contains('死鬥')} 'previous mode'
 Click $b 123;WaitFor {(Enabled $b 123) -and (TextOf $b 123) -eq '關閉觀戰'} 'observer on';Snapshot $b 'observer'
 Click $b 123;WaitFor {(Enabled $b 123) -and (TextOf $b 123) -eq '開啟觀戰'} 'observer off'
 if($BattleWithdrawal){
 Click $b 118
 WaitFor {$h=C $b 205;$count=[OwnedRoomWindow]::Send($h,0x18b,0,0);$found=$false;for($n=0;$n -lt $count;$n++){$row=[OwnedRoomWindow]::Row($h,$n);if($row.Contains('PeerFixture2') -and $row.Contains('隊伍2')){$found=$true}};$found -and (Enabled $b 110)} 'opposing team acknowledgement'
 Click $a 110;Click $b 110
 WaitFor {(Authority $a).phase -eq 'BATTLE' -and (Authority $b).phase -eq 'BATTLE' -and (Enabled $a 107) -and (TextOf $a 107) -eq '退出戰鬥'} 'live battle withdrawal button'
 WaitFor {(TextOf $a 204).Contains('地圖 564') -and (TextOf $b 204).Contains('地圖 564')} 'both original world frames rendered before withdrawal'
 Snapshot $a 'withdrawal-active';Snapshot $b 'withdrawal-peer-active'
 if($BattleCooldown){ExerciseBattleCooldown $a $b}
 $result.withdrawal_before=@{a=(Authority $a);b=(Authority $b)};Save
 Click $a 107
 WaitFor {(TextOf $a 204).Contains('戰鬥結束') -and (TextOf $b 204).Contains('戰鬥結束') -and (TextOf $a 107) -eq '返回大廳'} 'authoritative withdrawal settlement'
 Snapshot $a 'withdrawal-result';Snapshot $b 'withdrawal-peer-result'
 $result.withdrawal_result=@{a=(Authority $a);b=(Authority $b)};Save
 Click $a 107;Click $b 107
 WaitFor {(Enabled $a 119) -and (Enabled $b 119)} 'both returned to lobby'
 Snapshot $a 'withdrawal-lobby';Snapshot $b 'withdrawal-peer-lobby'
 $result.battle_withdrawal_widgets_verified=$true;Save
 }else{
 SelectRow $a 'PeerFixture2';WaitFor {Enabled $a 124} 'kick selection';Click $a 124
 WaitFor {(Enabled $a 120) -and (Enabled $b 119)} 'kick acknowledgement';Snapshot $a 'member-removed';Snapshot $b 'removed-to-lobby'
 Click $a 112;WaitFor {Enabled $a 119} 'owner leave'
 }
 }else{
  SelectCharacter $a 'PeerFixture';SelectCharacter $b 'PeerFixture2'
  foreach($c in $clients){
   WaitFor {Enabled $c 127} 'rental selection';Snapshot $c 'rental-selection'
   [void][OwnedRoomWindow]::Send((C $c 217),0x14e,3,0);Click $c 127
   WaitFor {(Enabled $c 127) -and !(Enabled $c 128)} 'unsupported loadout refused'
   [void][OwnedRoomWindow]::Send((C $c 217),0x14e,0,0);Click $c 127;WaitFor {Enabled $c 128} 'loadout accepted';Click $c 128
   WaitFor {(Enabled $c 129) -and (TextOf $c 204).Contains('地圖 500')} 'rental world render';Snapshot $c 'rental-world'
  }
  if($ActionProfile -eq 'rental-hotel'){
  $routeFile=Join-Path $project 'tests/fixtures/native-gui/hotel-route.json';$route=Get-Content -LiteralPath $routeFile -Raw|ConvertFrom-Json
  AssertRouteReference $routeFile $route.jar_sha256
  Copy-Item -LiteralPath $routeFile -Destination (Join-Path $out 'hotel-route.json')
  $result.hotel_route_sha256=(Get-FileHash -LiteralPath $routeFile).Hash
  foreach($c in $clients){
   foreach($step in $route.steps){
    $position=Authority $c
    if($position.x -ne $step.before[0] -or $position.y -ne $step.before[1]){throw 'Hotel GUI route start mismatch'}
    StepWorld $c $step.key ($step.after[0]-$step.before[0]) ($step.after[1]-$step.before[1])
   }
   Click $c 129;SelectComboMatch $c 219 '席巴';Click $c 130
   WaitFor {Enabled $c 134} 'hotel original dialog';Snapshot $c 'hotel-original'
   [void][OwnedRoomWindow]::Send((C $c 224),0x14e,0,0);Click $c 134
   WaitFor {Enabled $c 133} 'hotel numeric request';Snapshot $c 'hotel-key-count'
   foreach($invalid in @('0','9')){
    SetText $c 222 $invalid;Click $c 133
    WaitFor {(TextOf $c 204).Contains('請輸入 1～8 的數量')} 'local hotel range rejection'
    if(!(Authority $c).hotel_input_pending -or (Authority $c).rental_hotel){throw 'Invalid numeric input changed formal lease'}
    Snapshot $c ('hotel-invalid-'+$invalid)
   }
   SetText $c 222 '1';Click $c 133
   WaitFor {(Authority $c).rental_hotel -and !(Enabled $c 133)} 'first hotel lease';Snapshot $c 'hotel-first-lease'
   Click $c 130;WaitFor {Enabled $c 134} 'hotel return menu'
   [void][OwnedRoomWindow]::Send((C $c 224),0x14e,1,0);Click $c 134
   WaitFor {!(Authority $c).rental_hotel -and (TextOf $c 220).Contains('席巴')} 'hotel lease return';Snapshot $c 'hotel-returned'
   Click $c 130;WaitFor {Enabled $c 134} 'hotel new menu'
   [void][OwnedRoomWindow]::Send((C $c 224),0x14e,0,0);Click $c 134;WaitFor {Enabled $c 133} 'new quantity request'
   $quantity=if($HotelMiddle){if($c.Role -eq 'a'){4}else{2}}else{if($c.Role -eq 'a'){8}else{1}}
   SetText $c 222 $quantity;Click $c 133
   WaitFor {(Authority $c).rental_hotel -and !(Enabled $c 133)} 'entry lease';Snapshot $c 'hotel-rented'
   Click $c 130;WaitFor {Enabled $c 134} 'hotel entry menu'
   [void][OwnedRoomWindow]::Send((C $c 224),0x14e,2,0);Click $c 134
   $map=if($quantity -eq 1){501}elseif($quantity -le 4){502}else{503}
   WaitFor {(Authority $c).rental_map -eq $map -and (TextOf $c 204).Contains(('地圖 '+$map))} 'hotel new scene'
   if([OwnedRoomWindow]::IsWindowVisible((C $c 220))){throw 'Old NPC panel survived hotel portal'}
   Snapshot $c 'hotel-entered'
   $script:steps+=@{role=$c.Role;step='hotel-entry-authority';quantity=$quantity;map=$map;authority=(Authority $c)};Save
  }
  $result.hotel_widget_flow_verified=$true;$result.private_desktop_world_focus_verified=$true

  }elseif($ActionProfile -eq 'rental-merchants'){
  $planFile=Join-Path $project 'tests/fixtures/native-gui/merchant-plan.json';$plan=Get-Content -LiteralPath $planFile -Raw|ConvertFrom-Json
  AssertRouteReference $planFile $plan.jar_sha256
  Copy-Item -LiteralPath $planFile -Destination (Join-Path $out 'merchant-gui-plan.json');$result.plan_sha256=(Get-FileHash -LiteralPath $planFile).Hash
  $result.merchant_transactions=@()
  foreach($c in $clients){
   foreach($shop in $plan.shops){
    foreach($step in $shop.steps){
     $position=Authority $c;if($position.x -ne $step.before[0] -or $position.y -ne $step.before[1]){throw 'Merchant GUI route start mismatch'}
     StepWorld $c $step.key ($step.after[0]-$step.before[0]) ($step.after[1]-$step.before[1])
    }
    Click $c 129;SelectComboMatch $c 219 $shop.name;Click $c 130
    WaitFor {(Enabled $c 131) -or (Enabled $c 134)} 'merchant original dialog'
    if(!(Enabled $c 131)){SelectComboMatch $c 224 '想要租用裝備';Click $c 134;WaitFor {Enabled $c 131} 'Maya local trade page'}
    Snapshot $c ('merchant-'+$shop.npc+'-original')
    $rowNumber=0
    foreach($item in $shop.items){
     $before=Authority $c;$beforeIds=@($before.rental_inventory|ForEach-Object {$_.id})
     Click $c 131;WaitFor {Enabled $c 222} 'merchant buy list'
     $listCount=[OwnedRoomWindow]::Send((C $c 221),0x18b,0,0);if($listCount -ne $shop.items.Count){throw 'Merchant GUI row count differs'}
     SelectMerchantRow $c $item.name;SetText $c 222 '1';Click $c 133
     $balance=20-$item.gems
     WaitFor {(Authority $c).rental_gems -eq $balance -and (TextOf $c 223).Contains(('光之寶石：'+$balance))} 'merchant rent authority'
     $rented=Authority $c;$added=@($rented.rental_inventory|Where-Object {$_.id -notin $beforeIds})
     if($added.Count -ne 1 -or $added[0].catalog_id -ne $item.catalog_id){throw 'Wrong merchant object granted'}
     if($rowNumber -eq 0){Snapshot $c ('merchant-'+$shop.npc+'-rented')}
     Click $c 132;WaitFor {Enabled $c 222} 'merchant return list'
     SelectMerchantRow $c $item.name;SetText $c 222 '1';Click $c 133
     WaitFor {(Authority $c).rental_gems -eq 20 -and (TextOf $c 223).Contains('光之寶石：20')} 'merchant return authority'
     $returned=Authority $c
     if(@($returned.rental_inventory|Where-Object {$_.id -eq $added[0].id}).Count -ne 0){throw 'Returned merchant item remains'}
     $result.merchant_transactions+=@{role=$c.Role;npc=$shop.npc;row=$rowNumber;catalog_id=$item.catalog_id;name=$item.name;price=$item.gems;item=$added[0].id;before=$before;rented=$rented;returned=$returned};Save
     $rowNumber++
    }
    Snapshot $c ('merchant-'+$shop.npc+'-all-returned');Click $c 129
   }
  }
  $result.all_78_merchant_widget_rows_verified=$true;$result.private_desktop_world_focus_verified=$true

  }else{
  foreach($c in $clients){
   Click $c 129;WaitFor {Enabled $c 130} 'nearby targets';SelectComboMatch $c 219 '勒夫';Click $c 130
   WaitFor {Enabled $c 131} 'original NPC dialog';Snapshot $c 'npc-original'
   Click $c 131;WaitFor {Enabled $c 222} 'rent list';Snapshot $c 'rent-list';SelectMerchantRow $c '火蜥蝪' ;SetText $c 222 '7';Click $c 133
   WaitFor {(TextOf $c 204).Contains('光之寶石不足')} 'transaction refusal';Snapshot $c 'rent-refused'
   Click $c 131;WaitFor {Enabled $c 222} 'capacity rent list';SelectMerchantRow $c '火蜥蝪';SetText $c 222 '257';Click $c 133
   WaitFor {(TextOf $c 204).Contains('背包空間不足')} 'capacity refusal';Snapshot $c 'rent-capacity-refused'
   Click $c 131;WaitFor {Enabled $c 222} 'fresh rent list';SelectMerchantRow $c '火蜥蝪';SetText $c 222 '1';Click $c 133
   WaitFor {(TextOf $c 223).Contains('光之寶石：17')} 'rent inventory';Snapshot $c 'rented'
   Click $c 132;WaitFor {Enabled $c 222} 'return list';SelectMerchantRow $c '火蜥蝪';SetText $c 222 '1';Click $c 133
   WaitFor {(TextOf $c 223).Contains('光之寶石：20')} 'return inventory';Snapshot $c 'returned'
   Click $c 129;Snapshot $c 'back-to-scene'
  }
  if($NpcNavigation){
   $routeFile=Join-Path $project 'tests/fixtures/native-gui/citizen-routes.json'
   $routePlan=Get-Content -Raw -LiteralPath $routeFile|ConvertFrom-Json
   AssertRouteReference $routeFile $routePlan.jar_sha256
   $result.route_plan_sha256=(Get-FileHash -LiteralPath $routeFile).Hash
   Copy-Item -LiteralPath $routeFile -Destination (Join-Path $out 'rental-npc-routes.json')
   $count=0
   foreach($route in $routePlan.routes|Select-Object -First 5){
    foreach($step in $route.steps){
     $position=Authority $a
     if($position.x -ne $step.before[0] -or $position.y -ne $step.before[1]){throw 'Route start no longer matches authority'}
     StepWorld $a $step.key ($step.after[0]-$step.before[0]) ($step.after[1]-$step.before[1])
    }
    if($route.resource -ne 0){$count++;CitizenDialog $a $route.npc.name $count}
   }
   Click $a 129;SelectComboMatch $a 219 '潘塔休';Click $a 130;WaitFor {Enabled $a 134} 'guide links'
   SelectComboMatch $a 224 '成為正式市民';Click $a 134
   [void][OwnedRoomWindow]::Send((C $a 224),0x14e,0,0)
   WaitFor {([OwnedRoomWindow]::Send((C $a 224),0x147,0,0)) -eq 0} 'actual guide choice selected'
   Snapshot $a 'guide-citizen-page'
   Click $a 134
   WaitFor {(Authority $a).rental_citizen -and (TextOf $a 223).Contains('光之寶石：21') -and (Enabled $a 130)} 'citizenship authoritative grant'
   Snapshot $a 'citizenship-complete';$result.npc_navigation_widget_verified=$true
  }
  }
 }
 foreach($c in $clients){[void][OwnedRoomWindow]::PostMessage($c.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero)}
 foreach($c in $clients){if(!$c.Process.WaitForExit(15000)){throw 'GUI worker shutdown did not finish'}}
 if(!$server.WaitForExit(15000) -or $server.ExitCode -ne 0){throw 'Formal sessions failed cleanup'}
 $formal=Get-Content -Raw -LiteralPath (Join-Path $root 'result.json')|ConvertFrom-Json
 if($formal.actual_client_sessions -ne 4 -or !$formal.memberships_empty -or $formal.authority_mutations -ne 0 -or $formal.sql_access -or $formal.state_persistent){throw 'Isolation/cleanup mismatch'}
 if((Get-FileHash -LiteralPath $exe).Hash -ne $result.exe_sha256 -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $pin.sha256){throw 'Candidate/reference drift'}
 $result.reference_snapshot_sha256=(Get-FileHash -LiteralPath $referencePath).Hash;$result.inputs_reference_unchanged=$true
 $result.status=if($BattleWithdrawal){'verified_native_battle_withdrawal_widget_flow'}elseif($ActionProfile -eq 'room-admin'){'verified_native_room_widget_flow'}elseif($ActionProfile -eq 'rental-hotel'){'verified_native_hotel_widget_flow'}elseif($ActionProfile -eq 'rental-merchants'){'verified_native_merchant_widget_flow'}else{'verified_native_rental_widget_flow'};$result.server=$formal;$result.gui_widget_events_verified=$true;Save;Write-Output ('Verified native room widget flow: '+$out)
}catch{$result.status='failed';$result.error=$_.Exception.Message;Save;throw}
finally{
 foreach($c in $clients){if(!$c.Process.HasExited){[void][OwnedRoomWindow]::PostMessage($c.Window,0x10,[IntPtr]::Zero,[IntPtr]::Zero);if(!$c.Process.WaitForExit(5000)){StopOwned $c.Process ('127.0.0.1:'+$ready.port)}};$c.Process.Dispose()}
 if($server){if(!$server.HasExited){StopOwned $server $root};$server.Dispose()}
 foreach($name in @('result.json','server-trace.jsonl','ready.json')){$p=Join-Path $root $name;if(Test-Path -LiteralPath $p){Copy-Item -LiteralPath $p -Destination (Join-Path $out $name)}}
 $env:TEMP=$previousTemp;$env:TMP=$previousTmp
}
