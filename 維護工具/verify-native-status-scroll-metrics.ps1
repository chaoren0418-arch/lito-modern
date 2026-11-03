#Requires -Version 7.0
# Native-control prototype only. Font scaling here does not qualify actual multi-DPI product GUI.
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
Add-Type -TypeDefinition @'
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
'@
Add-Type -TypeDefinition @"
using System;using System.Text;using System.Runtime.InteropServices;
public static class StatusScrollProbe {
 [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)]public static extern IntPtr CreateWindowEx(uint ex,string cls,string text,uint style,int x,int y,int w,int h,IntPtr parent,IntPtr menu,IntPtr instance,IntPtr param);
 [DllImport("user32.dll")]public static extern bool DestroyWindow(IntPtr h);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]public static extern IntPtr SendMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]public static extern int GetWindowText(IntPtr h,StringBuilder s,int size);
 [DllImport("gdi32.dll",CharSet=CharSet.Unicode)]public static extern IntPtr CreateFont(int height,int width,int escape,int orient,int weight,uint italic,uint underline,uint strike,uint charset,uint outp,uint clip,uint quality,uint pitch,string face);
 [DllImport("gdi32.dll")]public static extern bool DeleteObject(IntPtr h);
 public static long Send(IntPtr h,uint m,long w,long l){return SendMessage(h,m,(IntPtr)w,(IntPtr)l).ToInt64();}
 public static string Text(IntPtr h){var s=new StringBuilder(8192);GetWindowText(h,s,s.Capacity);return s.ToString();}
}
"@
$evidence=Join-Path $taskRoot '驗證紀錄/current/native-room-selection'
$case=Join-Path $taskRoot '驗證紀錄/current/native-siege-portal-gui/20261015-050632-657/receipt.json'
$receipt=Get-Content -LiteralPath $case -Raw|ConvertFrom-Json
$text=($receipt.steps|Where-Object {$_.role -eq 'b' -and $_.step -eq 'post-portal-walk'}).status
if(!$text.Contains('入口：沒有回覆') -or !$text.Contains('方向鍵移動')){throw 'Missing original long GUI status'}
$results=@()
foreach($scale in @(1,2)){
 foreach($logicalWidth in @(456,528,568)){
  $owner=[IntPtr]::Zero;$edit=[IntPtr]::Zero;$font=[IntPtr]::Zero
  try{
   $owner=[StatusScrollProbe]::CreateWindowEx(0,'STATIC','owned status prototype',([uint32]2147483648),0,0,600*$scale,400*$scale,[IntPtr]::Zero,[IntPtr]::Zero,[IntPtr]::Zero,[IntPtr]::Zero)
   if($owner -eq [IntPtr]::Zero){throw 'Owned prototype owner creation failed'}
   $style=[uint32](0x40000000 -bor 0x10000000 -bor 0x00200000 -bor 0x0004 -bor 0x0800 -bor 0x0040)
   $edit=[StatusScrollProbe]::CreateWindowEx(0,'EDIT',$text,$style,0,0,$logicalWidth*$scale,90*$scale,$owner,[IntPtr]::Zero,[IntPtr]::Zero,[IntPtr]::Zero)
   if($edit -eq [IntPtr]::Zero){throw 'Owned readonly edit creation failed'}
   $font=[StatusScrollProbe]::CreateFont(-16*$scale,0,0,0,400,0,0,0,1,0,0,5,0,'Microsoft JhengHei UI')
   if($font -eq [IntPtr]::Zero){throw 'Owned font creation failed'}
   [void][StatusScrollProbe]::Send($edit,0x30,$font.ToInt64(),1)
   $initial=[StatusScrollProbe]::Text($edit)
   [void][StatusScrollProbe]::Send($edit,0x102,88,0)
   if([StatusScrollProbe]::Text($edit) -ne $initial -or $initial -ne $text){throw 'Readonly input changed original status'}
   $lines=[StatusScrollProbe]::Send($edit,0xba,0,0)
   [void][StatusScrollProbe]::Send($edit,0x115,6,0)
   $first=[StatusScrollProbe]::Send($edit,0xce,0,0)
   [void][StatusScrollProbe]::Send($edit,0x115,7,0)
   $last=[StatusScrollProbe]::Send($edit,0xce,0,0)
   if($lines -lt 5 -or $first -ne 0 -or $last -le $first){throw 'Overflow was not accessible by native scrolling'}
   $class=[OwnedTeleportWindow]::ClassName($edit);$actualStyle=[OwnedTeleportWindow]::Style($edit)
   $lastY=[OwnedTeleportWindow]::CharacterY($edit,($text.Length-1));$glyphHeight=[OwnedTeleportWindow]::LinePitch($edit);$rect=[OwnedTeleportWindow]::PhysicalClientRect($edit)
   if($class -ne 'Edit' -or ($actualStyle -band 0x800) -eq 0 -or ($actualStyle -band 0x200000) -eq 0){throw 'GUI helper class/style checks failed'}
   if($lastY -lt 0 -or $lastY+$glyphHeight -gt $rect.Bottom){throw 'Last status glyph does not fit after native scroll'}
   [void][StatusScrollProbe]::Send($edit,0x115,6,0)
   $restored=[StatusScrollProbe]::Send($edit,0xce,0,0)
   if($restored -ne 0 -or [StatusScrollProbe]::Text($edit) -ne $text){throw 'Scrolling changed content or failed to restore top'}
   $results+=@{font_scale=$scale;logical_width=$logicalWidth;logical_height=90;last_character_y=$lastY;glyph_height=$glyphHeight;client_height=$rect.Bottom;wrapped_lines=$lines;first_visible_before=$first;first_visible_at_bottom=$last;readonly_preserved=$true;all_text_retained=$true;top_restored=$true}
  }finally{if($owner -ne [IntPtr]::Zero){[void][StatusScrollProbe]::DestroyWindow($owner)};if($font -ne [IntPtr]::Zero){[void][StatusScrollProbe]::DeleteObject($font)}}
 }
}
$result=@{status='verified_native_scroll_prototype_and_GUI_text_metrics_helpers_only';source_receipt=$case;source_receipt_sha256=(Get-FileHash -LiteralPath $case).Hash;source_status=$text;checks=$results;product_modified=$false;actual_product_focus_movement_verified=$false;actual_multi_DPI_GUI_verified=$false;verifier_sha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash}
$result|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $evidence 'status-scroll-line-pitch-prototype.json') -Encoding utf8
Write-Output ('Native status prototype passed '+$results.Count+' font/width cases; product GUI still pending.')