#Requires -Version 7.0
[CmdletBinding()]
param()
$ErrorActionPreference='Stop'
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$manifest=Join-Path $project 'Cargo.toml'
$sources=@(Get-ChildItem -LiteralPath (Join-Path $project 'src'),(Join-Path $project '.cargo'),(Join-Path $project 'tools'),(Join-Path $project 'windows') -Recurse -Force)
if(@($sources|Where-Object{ $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count -gt 0){throw 'Reparse point inside standalone source tree'}
if((Get-Item -LiteralPath $project).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Project root is a reparse point'}
$metadataText=@(& cargo metadata --offline --locked --manifest-path $manifest --filter-platform i686-pc-windows-msvc --format-version 1 2>&1)
if($LASTEXITCODE -ne 0){throw 'Cargo metadata failed'}
$metadata=($metadataText -join "`n")|ConvertFrom-Json
$local=@($metadata.packages|Where-Object{ $null -eq $_.source })
if($local.Count -ne 1 -or [IO.Path]::GetFullPath($local[0].manifest_path) -ine $manifest){throw 'External local/path crate dependency'}
if([IO.Path]::GetFullPath($metadata.target_directory) -ine (Join-Path $project 'target')){throw 'Cargo target escapes this standalone project'}
$references=@($sources|Where-Object{ -not $_.PSIsContainer -and $_.FullName -ine $PSCommandPath -and $_.Extension -in '.rs','.toml','.ps1' }|Select-String -SimpleMatch '2-登入器')
if($references.Count -ne 0){throw 'Active source/tool path points back to the LHZ project'}
$dependencyFiles=@(Get-ChildItem -LiteralPath (Join-Path $project 'target') -Recurse -File -Filter '*.d' -ErrorAction SilentlyContinue)
if(@($dependencyFiles|Select-String -SimpleMatch '2-登入器').Count -ne 0){throw 'Compiled dependency file contains LHZ source paths'}
$bins=@($local[0].targets|Where-Object{ $_.kind -contains 'bin' })
if($bins.Count -ne 1 -or $bins[0].name -cne 'LiToModern'){throw 'Expected one standalone player binary'}
[pscustomobject]@{Project=$project;LocalCrates=$local.Count;PlayerBinary=$bins[0].name;RegistryPackages=$metadata.packages.Count-1;CheckedDependencyFiles=$dependencyFiles.Count;ExternalLocalSource=0;ReparsePoints=0}
