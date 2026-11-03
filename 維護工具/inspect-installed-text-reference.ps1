#Requires -Version 7.0
[CmdletBinding()]param([Parameter(Mandatory)][string]$Output)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
$taskRoot='D:\lineage開發\3-lito開發\lito重建版'
$evidencePath=[IO.Path]::GetFullPath($Output)
if(!$evidencePath.StartsWith($taskRoot+'\驗證紀錄\current\',[StringComparison]::OrdinalIgnoreCase)){throw 'Owned evidence path required'}
if(Test-Path -LiteralPath $evidencePath){throw 'Preserve existing observation'}
New-Item -ItemType Directory -Path $evidencePath|Out-Null
$runtimePath=Join-Path $taskRoot ('LiTo381-Modern\target\current\native-text-reference-oracle\'+[Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $runtimePath|Out-Null
$referencePath='D:\lineage開發\PakViewer-main\src\PakViewer.Cli\bin\Release\net10.0'
$assemblyPins=@{}
foreach($name in @('Lin.Helper.Core.dll','BouncyCastle.Cryptography.dll')){
 $source=Join-Path $referencePath $name;$destination=Join-Path $runtimePath $name
 Copy-Item -LiteralPath $source -Destination $destination
 $hash=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
 if((Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash -ne $hash){throw 'Oracle copy mismatch'}
 $assemblyPins[$name]=$hash
}
[Reflection.Assembly]::LoadFrom((Join-Path $runtimePath 'BouncyCastle.Cryptography.dll'))|Out-Null
$assembly=[Reflection.Assembly]::LoadFrom((Join-Path $runtimePath 'Lin.Helper.Core.dll'))
$roots=@('D:\lineage開發\0-客戶端','D:\lineage開發\0-客戶端-公版260707修正門跟icon')
$results=@()
foreach($resourcePath in $roots){
 $files=@('Text.idx','Text.pak');$before=@{}
 foreach($name in $files){$before[$name]=(Get-FileHash -LiteralPath (Join-Path $resourcePath $name) -Algorithm SHA256).Hash}
 $pak=[Lin.Helper.Core.Pak.PakFile]::new((Join-Path $resourcePath 'Text.idx'))
 try{
  $groups=[Collections.Generic.Dictionary[string,Collections.Generic.List[int]]]::new([StringComparer]::OrdinalIgnoreCase)
  for($i=0;$i -lt $pak.Count;$i++){
   $name=$pak.Files[$i].FileName
   if(!$groups.ContainsKey($name)){$groups.Add($name,[Collections.Generic.List[int]]::new())}
   $groups[$name].Add($i)
  }
  if($groups['list.spr'].Count -ne 1){throw 'Sample must be unambiguous'}
  [byte[]]$sample=$pak.Extract($groups['list.spr'][0])
  $sampleHash=[Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($sample))
  $indexRows=@($pak.Files|ForEach-Object {[ordered]@{name=$_.FileName;offset=$_.Offset;size=$_.FileSize}})
  $label=if($resourcePath -eq $roots[0]){'current'}else{'public-reference'}
  $indexRows|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $evidencePath ($label+'-index.json')) -Encoding utf8NoBOM
  $ambiguous=@($groups.GetEnumerator()|Where-Object {$_.Value.Count -gt 1}).Count
  $sampleText=[Text.Encoding]::Latin1.GetString($sample)
  $row=[ordered]@{root=$resourcePath;file_hashes=$before;count=$pak.Count;unique_names=$groups.Count;ambiguous_names=$ambiguous;protection=$pak.EncryptionType;protected=$pak.IsProtected;list_spr_bytes=$sample.Length;list_spr_sha256=$sampleHash;list_spr_header_count=[regex]::Matches($sampleText,'(?m)^#').Count;corruptsoulq1_e_records=$groups['corruptsoulq1-e.html'].Count}
  foreach($name in $files){if((Get-FileHash -LiteralPath (Join-Path $resourcePath $name) -Algorithm SHA256).Hash -ne $before[$name]){throw 'Readonly reference input changed'}}
  $results+=$row
 }finally{$pak.Dispose()}
}
Copy-Item -LiteralPath $PSCommandPath -Destination (Join-Path $evidencePath 'oracle.ps1')
$record=[ordered]@{status='independent_existing_CSharp_parser_readonly_observation';assembly=$assembly.FullName;oracle_assembly_hashes=$assemblyPins;results=$results;game_or_UI_started=$false;SQL_access=$false;resources_modified=$false;product_source_modified=$false;whole_goal_complete=$false}
$record|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $evidencePath 'readback.json') -Encoding utf8NoBOM
$record|ConvertTo-Json -Depth 10 -Compress
