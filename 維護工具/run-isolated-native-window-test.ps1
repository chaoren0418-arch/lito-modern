#Requires -Version 7.0
param([ValidateSet('room-admin','rental-shop','rental-hotel','rental-merchants')][string]$ActionProfile='rental-shop',[switch]$HotelMiddle,[switch]$NpcNavigation,[switch]$BattleWithdrawal,[switch]$BattleCooldown,[ValidateSet('Debug','Release')][string]$BuildProfile='Debug',[string]$ReferenceSnapshot)
# Native messages target the owned GUI thread; physical input is a separate acceptance scope.
# An unswitched private desktop owns only this test process tree. The user's input
# desktop is never selected, captured, or used to keep the test in foreground.
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
Add-Type -TypeDefinition @'
using System;using System.Text;using System.Runtime.InteropServices;
public static class LitoTestDesktop {
 [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] public struct Startup {
  public int size;public string reserved;public string desktop;public string title;
  public int x,y,cx,cy,charsX,charsY,fill,flags;public short show,reservedSize;public IntPtr reservedPtr,input,output,error;
 }
 [StructLayout(LayoutKind.Sequential)]public struct Info{public IntPtr process,thread;public uint pid,tid;}
 [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)]public static extern IntPtr CreateDesktop(string name,IntPtr device,IntPtr mode,uint flags,uint access,IntPtr security);
 [DllImport("user32.dll",SetLastError=true)]public static extern bool CloseDesktop(IntPtr desktop);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)]static extern bool CreateProcess(string app,StringBuilder command,IntPtr pa,IntPtr ta,bool inherit,uint flags,IntPtr env,string cwd,ref Startup startup,out Info info);
 [DllImport("kernel32.dll")]public static extern uint WaitForSingleObject(IntPtr h,uint ms);
 [DllImport("kernel32.dll")]public static extern bool GetExitCodeProcess(IntPtr h,out uint code);
 [DllImport("kernel32.dll")]public static extern bool CloseHandle(IntPtr h);
 public static Info Start(string app,string command,string cwd,string desktop){var s=new Startup{size=Marshal.SizeOf<Startup>(),desktop="winsta0\\"+desktop,flags=1,show=0};Info i;
  if(!CreateProcess(app,new StringBuilder(command),IntPtr.Zero,IntPtr.Zero,false,0x08000000,IntPtr.Zero,cwd,ref s,out i))throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());return i;
 }
}
'@
$name='LitoGui'+[Guid]::NewGuid().ToString('N')
$desktop=[LitoTestDesktop]::CreateDesktop($name,[IntPtr]::Zero,[IntPtr]::Zero,0,0x01ff,[IntPtr]::Zero)
if($desktop -eq [IntPtr]::Zero){throw ('Cannot create owned test desktop: '+[Runtime.InteropServices.Marshal]::GetLastWin32Error())}
$child=$null;$identity=$null
try{
 $pwsh=(Get-Process -Id $PID).Path;$script=Join-Path $PSScriptRoot 'verify-native-room-window.ps1'
 $command='"'+$pwsh+'" -NoLogo -NoProfile -NonInteractive -File "'+$script+'" -ActionProfile '+$ActionProfile+' -BuildProfile '+$BuildProfile+$(if($HotelMiddle){' -HotelMiddle'}else{''})+$(if($NpcNavigation){' -NpcNavigation'}else{''})+$(if($BattleWithdrawal){' -BattleWithdrawal'}else{''})+$(if($BattleCooldown){' -BattleCooldown'}else{''})
 if($ReferenceSnapshot){if(![IO.Path]::IsPathFullyQualified($ReferenceSnapshot) -or $ReferenceSnapshot.Contains('"')){throw 'Reference snapshot path must be absolute and unquoted'};$command+=' -ReferenceSnapshot "'+$ReferenceSnapshot+'"'}
 $child=[LitoTestDesktop]::Start($pwsh,$command,$taskRoot,$name);$identity=Get-Process -Id $child.pid
 $deadline=[DateTime]::UtcNow.AddMinutes(7)
 while([LitoTestDesktop]::WaitForSingleObject($child.process,250) -eq 258){if([DateTime]::UtcNow -gt $deadline){throw 'Isolated GUI test timed out'}}
 [uint32]$code=0;if(![LitoTestDesktop]::GetExitCodeProcess($child.process,[ref]$code)){throw 'No test exit code'}
 if($code -ne 0){throw ('Isolated GUI test failed: exit '+$code+'; inspect latest native-room-gui receipt')}
 Write-Output 'Private-desktop native widget test passed; user desktop was not switched.'
}finally{
 if($child){
  if([LitoTestDesktop]::WaitForSingleObject($child.process,0) -eq 258){$live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$child.pid);if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$identity.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$live.CommandLine.Contains($script)){throw 'Owned desktop child identity mismatch'};$identity.Kill($true);$identity.WaitForExit()}
  [void][LitoTestDesktop]::CloseHandle($child.thread);[void][LitoTestDesktop]::CloseHandle($child.process)
 }
 if(![LitoTestDesktop]::CloseDesktop($desktop)){Write-Warning 'Owned desktop handle could not be closed'}
}
