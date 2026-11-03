#Requires -Version 7.0
[CmdletBinding()]
param([Parameter(Mandatory)][ValidatePattern('^[a-zA-Z0-9-]{1,64}$')][string]$Candidate)
$ErrorActionPreference='Stop'
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$target=Join-Path $project 'target'
. (Join-Path $PSScriptRoot 'build-evidence.ps1')
if($Candidate -match '^(con|prn|aux|nul|com[1-9]|lpt[1-9])$'){throw 'Reserved candidate directory name'}
$lease=[IO.File]::Open((Join-Path $target 'project-lease.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
try {
    # Validate before creating any candidate directory. A test build or old
    # executable left in target is not evidence of a current player build.
    $receipt=Assert-ModernBuildReceipt $project 'Release'
    $exe=Get-Item -LiteralPath (Get-ModernPlayerPath $project 'Release')
    $version=Get-ModernPlayerVersion $exe.FullName 'Release'
    $null=& (Join-Path $PSScriptRoot 'verify-independent.ps1')
    $example=Get-Content -LiteralPath (Join-Path $project 'packaging/profile.example.json') -Raw | ConvertFrom-Json
    if($example.schema -ne 2 -or $example.gameplay_protocol -cne 'modern_v47'){throw 'Player package requires an explicit Modern v47 example'}
    $dist=[IO.Path]::GetFullPath((Join-Path $project 'dist'))
    if(Test-Path -LiteralPath $dist){
        $item=Get-Item -LiteralPath $dist
        if(!$item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Unsafe dist directory'}
    } else {$null=New-Item -ItemType Directory -Path $dist}
    $destination=[IO.Path]::GetFullPath((Join-Path $dist $Candidate))
    if(Test-Path -LiteralPath $destination){throw 'Candidate already exists; packaging never overwrites an earlier candidate'}
    $staging=[IO.Path]::GetFullPath((Join-Path $dist ('.' + $Candidate + '-' + [Guid]::NewGuid().ToString('N') + '.partial')))
    # Both resolved move targets are explicit children of this project's dist.
    foreach($path in @($staging,$destination)){
        if([IO.Path]::GetDirectoryName($path) -ine $dist){throw 'Candidate path escapes dist'}
    }
    $null=New-Item -ItemType Directory -Path $staging,(Join-Path $staging 'config')
    Copy-Item -LiteralPath $exe.FullName -Destination (Join-Path $staging 'LiToModern.exe')
    Copy-Item -LiteralPath (Join-Path $project 'packaging/README.md') -Destination (Join-Path $staging 'README.md')
    Copy-Item -LiteralPath (Join-Path $project 'packaging/profile.example.json') -Destination (Join-Path $staging 'config/profile.example.json')
    $files=@(Get-ChildItem -LiteralPath $staging -File -Recurse)
    if(@($files|Where-Object Extension -EQ '.exe').Count -ne 1 -or @($files|Where-Object Extension -In '.dll','.bin','.pak').Count -ne 0){throw 'Candidate contains unexpected executable/resource payload'}
    $manifest=@($files|Sort-Object FullName|ForEach-Object{[pscustomobject]@{Path=[IO.Path]::GetRelativePath($staging,$_.FullName);Length=$_.Length;SHA256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}})
    $finalReceipt=Assert-ModernBuildReceipt $project 'Release'
    if($receipt.SHA256 -cne $finalReceipt.SHA256 -or $receipt.InputSHA256 -cne $finalReceipt.InputSHA256){throw 'Build changed during packaging'}
    foreach($pair in @(@('LiToModern.exe',$exe.FullName),@('README.md',(Join-Path $project 'packaging/README.md')),@('config/profile.example.json',(Join-Path $project 'packaging/profile.example.json')))){
        if((Get-FileHash -LiteralPath (Join-Path $staging $pair[0]) -Algorithm SHA256).Hash -cne (Get-FileHash -LiteralPath $pair[1] -Algorithm SHA256).Hash){throw 'Candidate copy mismatch'}
    }
    [pscustomobject]@{Schema=2;BuildInputSHA256=$receipt.InputSHA256;ExecutableSHA256=$receipt.SHA256;Version=$version;Files=$manifest} |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $staging 'SHA256.json') -Encoding utf8
    # Directory.Move fails if the destination appeared meanwhile; it never
    # treats that directory as a container and nests an unfinished candidate.
    [IO.Directory]::Move($staging,$destination)
    [pscustomobject]@{Candidate=$destination;ExecutableSHA256=$receipt.SHA256;BuildInputSHA256=$receipt.InputSHA256;Files=$manifest}
} finally { $lease.Dispose() }
