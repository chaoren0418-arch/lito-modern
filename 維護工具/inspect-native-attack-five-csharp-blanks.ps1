param([Parameter(Mandatory)][string]$OutputDirectory,[Parameter(Mandatory)][string]$RawExportReceipt)
$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath((Split-Path $PSScriptRoot -Parent))
$output=[IO.Path]::GetFullPath($OutputDirectory)
if(-not $output.StartsWith(($root+[IO.Path]::DirectorySeparatorChar),[StringComparison]::OrdinalIgnoreCase)){throw 'Output must remain inside rebuild root'}
if(Test-Path -LiteralPath $output){throw 'Preserve existing output; choose a new owned directory'}
New-Item -ItemType Directory -Path $output|Out-Null
$env:TEMP=Join-Path $root 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$library='D:/lineage開發/3-lito開發/lito登入器/LiTo381-Launcher/tools/class_compat/resource_audit/bin/Debug/net10.0'
$dll=[IO.Path]::GetFullPath((Join-Path $library 'Lin.Helper.Core.dll'))
if((Get-FileHash -LiteralPath $dll -Algorithm SHA256).Hash -ne '8F31AEA698513D8606432784DB6047EEDBC9431355A1E0797DC4C4006D03DE6F'){throw 'Reference DLL drift'}
$inventoryPath=Join-Path $root '驗證紀錄/current/native-attack-five-resource-inspection/20261017-144953-005315/asset-inventory.json'
$inventory=Get-Content -LiteralPath $inventoryPath -Raw|ConvertFrom-Json
$rawReceipt=[IO.Path]::GetFullPath($RawExportReceipt)
if(-not $rawReceipt.StartsWith(($root+[IO.Path]::DirectorySeparatorChar),[StringComparison]::OrdinalIgnoreCase)){throw 'Raw receipt must remain inside rebuild root'}
$export=Get-Content -LiteralPath $rawReceipt -Raw|ConvertFrom-Json
if($export.status -ne 'exported_three_exact_pinned_raw_samples_no_decoder' -or $export.members.Count -ne 3){throw 'Exact raw export is not complete'}
if($export.inventory_sha256 -ne (Get-FileHash -LiteralPath $inventoryPath -Algorithm SHA256).Hash){throw 'Raw export inventory mismatch'}
$sources=@{};$members=@();$status='reading_independent_CSharp_blank_samples';$failure=$null
Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $output 'inspector.ps1')
try{
 foreach($name in @('BouncyCastle.Cryptography.dll','ICSharpCode.SharpZipLib.dll','SixLabors.ImageSharp.dll','ZstdSharp.dll','Lin.Helper.Core.dll')){
  $p=Join-Path $library $name;$sources[$p]=(Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash
  [Reflection.Assembly]::LoadFrom($p)|Out-Null
 }
 foreach($name in @('1815-0.spr','1816-0.spr','1818-0.spr')){
  $source=@($inventory.members|Where-Object name -EQ $name);if($source.Count -ne 1){throw 'Inventory membership mismatch'};$source=$source[0]
  $entry=@($export.members|Where-Object name -EQ $name);if($entry.Count -ne 1){throw 'Raw export membership mismatch'}
  $sample=[IO.Path]::GetFullPath($entry[0].path)
  if(-not $sample.StartsWith(($root+[IO.Path]::DirectorySeparatorChar),[StringComparison]::OrdinalIgnoreCase)){throw 'Sample escaped rebuild root'}
  $raw=[IO.File]::ReadAllBytes($sample)
  $rawHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($raw))
  if($rawHash -ne $source.raw_sha256 -or $raw.Length -ne $source.raw_bytes){throw ('Selected raw bytes drift '+$name)}
  $frames=[Lin.Helper.Core.Sprite.SprReader]::LoadRaw([byte[]]$raw)
  if($frames.Count -ne $source.declared_headers.frame_count){throw 'CSharp/raw frame count mismatch'}
  $frameRows=@();$blanks=@()
  for($i=0;$i -lt $frames.Count;$i++){
   $f=$frames[$i];$h=$source.declared_headers.frames[$i];$blank=($null -eq $f.Pixels)
   $frameRows+=@{index=$i;XOffset=$f.XOffset;YOffset=$f.YOffset;Width=$f.Width;Height=$f.Height;unknown1=$f.Unknown1;unknown2=$f.Unknown2;type=$f.Type;pixels_null=$blank;pixel_bytes=if($blank){0}else{$f.Pixels.Length};pixel_sha256=if($blank){$null}else{[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($f.Pixels))};source_header=$h}
   if(($h.bounds -join ',') -eq '544,384,-480,-384'){
    if($h.references -ne 0 -or ($h.unknown -join ',') -ne '65535,0' -or -not $blank){throw '544 marker not independently null in CSharp'};$blanks+=$i
   }
  }
  if(-not $blanks.Count){throw 'Expected reviewed marker absent'}
  if((Get-FileHash -LiteralPath $sample -Algorithm SHA256).Hash -ne $rawHash){throw 'Owned raw sample drift'}
  $members+=@{name=$name;raw_sample=$sample;raw_bytes=$raw.Length;raw_sha256=$rawHash;frames=$frameRows;reviewed_544_null_slots=$blanks}
 }
 foreach($p in $sources.Keys){if((Get-FileHash -LiteralPath $p -Algorithm SHA256).Hash -ne $sources[$p]){throw 'Reference library drift'}}
 $status='executed_readonly_independent_CSharp_544_blank_samples'
}catch{$status='failed_preserved';$failure=$_.ToString();throw}
finally{
 @{status=$status;failure=$failure;reference_DLL_sha256=(Get-FileHash -LiteralPath $dll -Algorithm SHA256).Hash;reference_libraries=$sources;inventory=$inventoryPath;inventory_sha256=(Get-FileHash -LiteralPath $inventoryPath -Algorithm SHA256).Hash;raw_export_receipt=$rawReceipt;raw_export_receipt_sha256=(Get-FileHash -LiteralPath $rawReceipt -Algorithm SHA256).Hash;members=$members;original_access='read_only_by_owned_native_exporter';independent_archive_reader_claimed=$false;new_sessions=0;new_casts=0;product_sources_modified=$false;GUI_verified=$false;whole_goal_complete=$false}|ConvertTo-Json -Depth 30|Set-Content -LiteralPath (Join-Path $output 'report.json') -Encoding utf8
}
Write-Output ('Independent CSharp blank samples: '+$members.Count)
