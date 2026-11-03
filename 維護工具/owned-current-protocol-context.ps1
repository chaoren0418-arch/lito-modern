function New-OwnedCurrentProtocolContext([string]$ToolPath,[string]$Group,[string]$BuildProfile,[string]$SourceManifest,[string[]]$ExtraInputs,[string]$NormalQualification) {
 $taskRoot=Split-Path -Parent $PSScriptRoot
 $project=Join-Path $taskRoot 'LiTo381-Modern'
 if(!$SourceManifest){throw 'Supply -SourceManifest with the reviewed current compilation input map'}
 $map=if([IO.Path]::IsPathFullyQualified($SourceManifest)){[IO.Path]::GetFullPath($SourceManifest)}else{[IO.Path]::GetFullPath((Join-Path $taskRoot $SourceManifest))}
 if(!$map.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Compilation input map outside rebuild root'}
 $inputs=Get-Content -LiteralPath $map -Raw|ConvertFrom-Json -AsHashtable
 if($inputs.Count -ne 968){throw 'Review current compilation input count before using this wrapper'}
 if(!$NormalQualification){$NormalQualification='驗證紀錄/current/native-teleport-wire/normal-after-reference-harness-fixes/qualified.json'}
 $normalPath=if([IO.Path]::IsPathFullyQualified($NormalQualification)){[IO.Path]::GetFullPath($NormalQualification)}else{[IO.Path]::GetFullPath((Join-Path $taskRoot $NormalQualification))}
 if(!$normalPath.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase) -or ((Get-Item -LiteralPath $normalPath).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Normal qualification outside owned rebuild files'}
 $normal=Get-Content -LiteralPath $normalPath -Raw|ConvertFrom-Json
 $normalMap=Join-Path (Split-Path -Parent $normalPath) 'source-inputs.json'
 $mapSHA=(Get-FileHash -LiteralPath $map).Hash
 if($normal.source_files -ne 968 -or $normal.features.Count -ne 0 -or $mapSHA -ne $normal.source_manifest_sha256 -or (Get-FileHash -LiteralPath $normalMap).Hash -ne $mapSHA){throw 'Normal qualification does not bind this exact compilation input map'}
 $main=$normal.profiles.$BuildProfile
 if(!$main.unique_entry -or $main.features.Count -ne 0 -or $main.machine -ne 'AMD64' -or $main.optional_header -ne 'PE32+'){throw 'Expected the reviewed unique-entry normal x64 main'}
 $stamp=[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff')+'-'+$BuildProfile
 $outputBase=Join-Path $taskRoot ('驗證紀錄/current/'+$Group)
 $output=Join-Path $outputBase $stamp
 $runtime=Join-Path $project ('target/current/'+$Group+'/'+$stamp)
 $classes=Join-Path $runtime 'oracle'
 if((Test-Path -LiteralPath $output) -or (Test-Path -LiteralPath $runtime)){throw 'Unique run directory already exists'}
 New-Item -ItemType Directory -Path $output,$runtime,$classes|Out-Null
 $sources=@{}
 $all=@($ToolPath,(Join-Path $PSScriptRoot 'owned-current-protocol-context.ps1'),(Join-Path $PSScriptRoot 'owned-validation-process.ps1'),(Join-Path $PSScriptRoot 'owned-protocol-fixture.ps1'),(Join-Path $PSScriptRoot 'compare-owned-protocol-fixture.py'),$map,(Join-Path $taskRoot '驗證紀錄/current/reference-snapshot.json'),$normalPath)+@($ExtraInputs|ForEach-Object {Join-Path $project $_})
 foreach($file in $all|Select-Object -Unique){
  $path=[IO.Path]::GetFullPath($file)
  if(!$path.StartsWith($taskRoot+'\',[StringComparison]::OrdinalIgnoreCase) -or ((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Unsafe source snapshot'}
  $relative=[IO.Path]::GetRelativePath($taskRoot,$path).Replace('\','/');$sources[$relative]=(Get-FileHash -LiteralPath $path).Hash
  $copy=Join-Path $output ('source/'+$relative);New-Item -ItemType Directory -Path (Split-Path -Parent $copy) -Force|Out-Null;Copy-Item -LiteralPath $path -Destination $copy
  if((Get-FileHash -LiteralPath $copy).Hash -ne $sources[$relative]){throw 'Source snapshot readback mismatch'}
 }
 $exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$BuildProfile.ToLowerInvariant()+'/LiToModern.exe')
 $expected=$main.sha256
 return [pscustomobject]@{TaskRoot=$taskRoot;Project=$project;Output=$output;OutputBase=$outputBase;Runtime=$runtime;Classes=$classes;Inputs=$inputs;SourcePins=$sources;Exe=$exe;ExpectedMainSHA=$expected;NormalQualification=$normalPath;NormalQualificationSHA=(Get-FileHash -LiteralPath $normalPath).Hash;SourceManifest=$map;SourceManifestSHA=(Get-FileHash -LiteralPath $map).Hash}
}
function Assert-OwnedCurrentProtocolContext($Context) {
 foreach($name in $Context.Inputs.Keys){if((Get-FileHash -LiteralPath (Join-Path $Context.TaskRoot $name)).Hash -ne $Context.Inputs[$name]){throw ('Compilation input drift: '+$name)}}
 foreach($name in $Context.SourcePins.Keys){if((Get-FileHash -LiteralPath (Join-Path $Context.TaskRoot $name)).Hash -ne $Context.SourcePins[$name]){throw ('Validation tool/input drift: '+$name)}}
 if((Get-FileHash -LiteralPath $Context.Exe).Hash -ne $Context.ExpectedMainSHA){throw 'Normal main identity differs from reviewed unique-entry baseline'}
}
function Save-OwnedProtocolPresentationFiles($Response,[string]$Output,[string]$Mode) {
 $taskRoot=Split-Path -Parent $PSScriptRoot
 $allowed=[IO.Path]::GetFullPath((Join-Path $taskRoot 'LiTo381-Modern/target/current'))+'\'
 $found=[Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
 function Read-ProtocolPngPaths($Value) {
  if($null -eq $Value -or $Value -is [string] -or $Value -is [ValueType]){return}
  if($Value -is [System.Collections.IDictionary]){foreach($key in $Value.Keys){if($key -eq 'png' -and $Value[$key] -is [string] -and $Value[$key]){[void]$found.Add($Value[$key])};Read-ProtocolPngPaths $Value[$key]};return}
  if($Value -is [System.Collections.IEnumerable]){foreach($entry in $Value){Read-ProtocolPngPaths $entry};return}
  foreach($property in $Value.PSObject.Properties){if($property.Name -eq 'png' -and $property.Value -is [string] -and $property.Value){[void]$found.Add($property.Value)};Read-ProtocolPngPaths $property.Value}
 }
 Read-ProtocolPngPaths $Response
 $pins=[ordered]@{};$number=0
 foreach($file in $found){
  $path=[IO.Path]::GetFullPath($file)
  if(!$path.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase) -or ((Get-Item -LiteralPath $path).Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Presentation source outside owned target'}
  $sha=(Get-FileHash -LiteralPath $path).Hash
  $relative='presentation/'+$Mode+'/'+('{0:D3}-' -f $number)+(Split-Path -Leaf $path);$number++
  $copy=Join-Path $Output $relative;New-Item -ItemType Directory -Path (Split-Path -Parent $copy) -Force|Out-Null;Copy-Item -LiteralPath $path -Destination $copy
  if((Get-FileHash -LiteralPath $copy).Hash -ne $sha -or (Get-FileHash -LiteralPath $path).Hash -ne $sha){throw 'Presentation snapshot readback mismatch'}
  $pins[$path]=@{saved_relative=$relative;sha256=$sha}
 }
 return $pins
}
