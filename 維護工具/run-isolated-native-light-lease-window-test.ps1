#Requires -Version 7.0
param([ValidateSet('Debug','Release')][string]$BuildProfile='Release',[ValidateSet('probe','expiry')][string]$Scenario='probe',[ValidateSet('lito-legacy','tw13081901-v1')][string]$ResourceProfile='tw13081901-v1',[Parameter(Mandatory)][string]$NormalQualification)
$ErrorActionPreference='Stop';$taskRoot=Split-Path -Parent $PSScriptRoot
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

$name='LitoLightLeaseGui'+[Guid]::NewGuid().ToString('N')
$desktop=[LitoTestDesktop]::CreateDesktop($name,[IntPtr]::Zero,[IntPtr]::Zero,0,0x01ff,[IntPtr]::Zero)
if($desktop -eq [IntPtr]::Zero){throw 'Owned desktop creation failed'};$child=$null;$identity=$null
try {
 $pwsh=(Get-Process -Id $PID).Path;$script=Join-Path $PSScriptRoot 'verify-native-light-lease-window.ps1'
 $driverRoot=Join-Path $taskRoot ('LiTo381-Modern/target/current/native-light-lease-gui-launch/'+$name);New-Item -ItemType Directory -Path $driverRoot -Force|Out-Null
 $driver=Join-Path $driverRoot 'driver.ps1';$childLog=Join-Path $driverRoot 'child.log'
 $driverText='$ErrorActionPreference=''Stop'';try { & '''+$script.Replace("'","''")+''' -BuildProfile '+$BuildProfile+' -Scenario '+$Scenario+' -ResourceProfile '+$ResourceProfile+' -NormalQualification '''+$NormalQualification.Replace("'","''")+''''+' *> '''+$childLog.Replace("'","''")+''';exit 0 }catch { $_|Out-String|Add-Content -LiteralPath '''+$childLog.Replace("'","''")+''';exit 1 }'
 Set-Content -LiteralPath $driver -Value $driverText -Encoding utf8
 $command='"'+$pwsh+'" -NoLogo -NoProfile -NonInteractive -File "'+$driver+'"'

 $child=[LitoTestDesktop]::Start($pwsh,$command,$taskRoot,$name);$identity=Get-Process -Id $child.pid
 $deadline=[DateTime]::UtcNow.AddSeconds(3900)
 while([LitoTestDesktop]::WaitForSingleObject($child.process,250) -eq 258){if([DateTime]::UtcNow -gt $deadline){throw 'Owned GUI lease expired'}}
 [uint32]$code=0;if(![LitoTestDesktop]::GetExitCodeProcess($child.process,[ref]$code) -or $code -ne 0){throw ('GUI test failed: '+$code+'; inspect latest native-teleport-gui receipt')}
 Write-Output ('LIGHT_LEASE_GUI_DRIVER_LOG='+$childLog);Get-Content -LiteralPath $childLog|Where-Object {$_.StartsWith('LIGHT_LEASE_GUI_OUTPUT=')}|Write-Output
} finally {
 if($child){
  if([LitoTestDesktop]::WaitForSingleObject($child.process,0) -eq 258){
   $live=Get-CimInstance Win32_Process -Filter ('ProcessId='+$child.pid)
   if(!$live -or [Math]::Abs(($live.CreationDate.ToUniversalTime()-$identity.StartTime.ToUniversalTime()).TotalSeconds) -gt 1 -or !$live.CommandLine.Contains($driver)){throw 'Owned desktop child identity mismatch'}
   $identity.Kill($true);$identity.WaitForExit()
  }
  [void][LitoTestDesktop]::CloseHandle($child.thread);[void][LitoTestDesktop]::CloseHandle($child.process)
 }
 if(![LitoTestDesktop]::CloseDesktop($desktop)){Write-Warning 'Desktop handle close failed'}
}
