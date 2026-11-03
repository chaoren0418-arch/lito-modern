function Save-OwnedFixedProbeFiles([string]$Runtime,[string]$Destination) {
 $taskRoot=Split-Path -Parent $PSScriptRoot
 $scope=[IO.Path]::GetFullPath($Runtime)
 $allowed=[IO.Path]::GetFullPath((Join-Path $taskRoot 'LiTo381-Modern/target/current'))+'\'
 if(!$scope.StartsWith($allowed,[StringComparison]::OrdinalIgnoreCase)){throw 'Probe snapshot outside owned target'}
 New-Item -ItemType Directory -Path $Destination -Force|Out-Null
 $pins=[ordered]@{}
 foreach($file in Get-ChildItem -LiteralPath $scope -File){
  if($file.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Probe snapshot refuses reparse file'}
  $before=(Get-FileHash -LiteralPath $file.FullName).Hash
  $copy=Join-Path $Destination $file.Name;Copy-Item -LiteralPath $file.FullName -Destination $copy
  if((Get-FileHash -LiteralPath $copy).Hash -ne $before -or (Get-FileHash -LiteralPath $file.FullName).Hash -ne $before){throw 'Probe snapshot readback mismatch'}
  $pins[$file.Name]=$before
 }
 $pins|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $Destination 'file-pins.json') -Encoding utf8
 return $pins
}
