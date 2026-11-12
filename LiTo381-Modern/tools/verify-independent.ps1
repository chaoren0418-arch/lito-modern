#Requires -Version 7.0
[CmdletBinding()]
param()
$ErrorActionPreference='Stop'
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$manifest=Join-Path $project 'Cargo.toml'
# .cargo is a local, untracked directory: a fresh clone has none.
$roots=@('src','.cargo','tools','windows','packaging')|ForEach-Object{Join-Path $project $_}|Where-Object{Test-Path -LiteralPath $_}
$sources=@(Get-ChildItem -LiteralPath $roots -Recurse -Force)
if(@($sources|Where-Object{ $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count -gt 0){throw 'Reparse point inside standalone source tree'}
if((Get-Item -LiteralPath $project).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Project root is a reparse point'}
$metadataText=@(& cargo metadata --offline --locked --manifest-path $manifest --filter-platform x86_64-pc-windows-msvc --format-version 1)
if($LASTEXITCODE -ne 0){throw 'Cargo metadata failed'}
$metadata=($metadataText -join "`n")|ConvertFrom-Json
$local=@($metadata.packages|Where-Object{ $null -eq $_.source })
if($local.Count -ne 1 -or [IO.Path]::GetFullPath($local[0].manifest_path) -ine $manifest){throw 'External local/path crate dependency'}
# target or target/current, depending on whether the local .cargo/config.toml applies.
$targetRoot=Join-Path $project 'target'
$targetDirectory=[IO.Path]::GetFullPath($metadata.target_directory)
if($targetDirectory -ine $targetRoot -and !$targetDirectory.StartsWith($targetRoot+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Cargo target escapes this standalone project'}
$references=@($sources|Where-Object{ -not $_.PSIsContainer -and $_.FullName -ine $PSCommandPath -and $_.Extension -in '.rs','.toml','.ps1','.cmd' }|Select-String -SimpleMatch '2-登入器')
if($references.Count -ne 0){throw 'Active source/tool path points back to the LHZ project'}
$dependencyFiles=@(Get-ChildItem -LiteralPath $targetRoot -Recurse -File -Filter '*.d' -ErrorAction SilentlyContinue)
if(@($dependencyFiles|Select-String -SimpleMatch '2-登入器').Count -ne 0){throw 'Compiled dependency file contains LHZ source paths'}
# The package ships exactly this binary; other [[bin]] targets are development candidates.
$bins=@($local[0].targets|Where-Object{ $_.kind -contains 'bin' })
if(@($bins|Where-Object name -CEQ 'LiToModern').Count -ne 1 -or $local[0].default_run -cne 'LiToModern'){throw 'Expected LiToModern as the one default player binary'}
[pscustomobject]@{Project=$project;LocalCrates=$local.Count;PlayerBinary='LiToModern';OtherBinaries=@($bins|Where-Object name -CNE 'LiToModern'|ForEach-Object name);RegistryPackages=$metadata.packages.Count-1;CheckedDependencyFiles=$dependencyFiles.Count;ExternalLocalSource=0;ReparsePoints=0}
