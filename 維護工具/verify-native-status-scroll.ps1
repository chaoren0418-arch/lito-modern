#Requires -Version 7.0
# Native-control prototype only. Font scaling here does not qualify actual multi-DPI product GUI.
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
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
   [void][StatusScrollProbe]::Send($edit,0x115,6,0)
   $restored=[StatusScrollProbe]::Send($edit,0xce,0,0)
   if($restored -ne 0 -or [StatusScrollProbe]::Text($edit) -ne $text){throw 'Scrolling changed content or failed to restore top'}
   $results+=@{font_scale=$scale;logical_width=$logicalWidth;logical_height=90;wrapped_lines=$lines;first_visible_before=$first;first_visible_at_bottom=$last;readonly_preserved=$true;all_text_retained=$true;top_restored=$true}
  }finally{if($owner -ne [IntPtr]::Zero){[void][StatusScrollProbe]::DestroyWindow($owner)};if($font -ne [IntPtr]::Zero){[void][StatusScrollProbe]::DeleteObject($font)}}
 }
}
$result=@{status='verified_native_readonly_scroll_control_prototype_only';source_receipt=$case;source_receipt_sha256=(Get-FileHash -LiteralPath $case).Hash;source_status=$text;checks=$results;product_modified=$false;actual_product_focus_movement_verified=$false;actual_multi_DPI_GUI_verified=$false;verifier_sha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash}
$result|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $evidence 'status-scroll-prototype.json') -Encoding utf8
Write-Output ('Native status prototype passed '+$results.Count+' font/width cases; product GUI still pending.')