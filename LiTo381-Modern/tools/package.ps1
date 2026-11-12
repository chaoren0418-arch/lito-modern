#Requires -Version 7.0
# Builds the portable player candidate under dist/<Candidate> from the current
# Release build (tools/build.ps1 -Mode Release). The package holds one player
# EXE, a double-click entry that needs only cmd.exe, the README and the version
# lists for source, executable and external resources. No DLL, native BIN or
# resource archive is ever copied.
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[a-zA-Z0-9-]{1,64}$')][string]$Candidate,
    # External client resources this candidate is prepared against: hashed read-only, never copied.
    [string]$ResourceRoot=$env:LITO_CLIENT_ROOT,
    # Optional effectlist2.xml recorded next to the resource list.
    [string]$EffectCatalog,
    # Also write dist/<Candidate>.zip with the candidate as its single top-level folder.
    [switch]$Zip,
    # Record, instead of refuse, tracked source changes that are not committed.
    [switch]$AllowUncommitted
)
$ErrorActionPreference='Stop'
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$target=Join-Path $project 'target'
. (Join-Path $PSScriptRoot 'build-evidence.ps1')
. (Join-Path $PSScriptRoot 'portable-evidence.ps1')
if($Candidate -match '^(con|prn|aux|nul|com[1-9]|lpt[1-9])$'){throw 'Reserved candidate directory name'}
if([string]::IsNullOrWhiteSpace($ResourceRoot)){throw 'Pass -ResourceRoot (or set LITO_CLIENT_ROOT): the package must list the resource version it was prepared against'}
if(![IO.Path]::IsPathFullyQualified($ResourceRoot) -or !(Test-Path -LiteralPath $ResourceRoot -PathType Container)){throw "Resource root must be an existing absolute directory: $ResourceRoot"}
if($EffectCatalog -and (![IO.Path]::IsPathFullyQualified($EffectCatalog) -or !(Test-Path -LiteralPath $EffectCatalog -PathType Leaf))){throw "Effect catalog must be an existing absolute file: $EffectCatalog"}

function Get-SourceRevision {
    $commit=@(& git -C $project rev-parse HEAD 2>$null)
    if($LASTEXITCODE -ne 0 -or $commit.Count -ne 1 -or $commit[0] -notmatch '^[0-9a-f]{40}$'){throw 'Cannot read the source commit (git rev-parse HEAD)'}
    $changes=@(& git -C $project status --porcelain --untracked-files=no)
    if($LASTEXITCODE -ne 0){throw 'Cannot read the source state (git status)'}
    if($changes.Count -and !$AllowUncommitted){throw "Tracked source changes are not committed ($($changes.Count)); commit them or pass -AllowUncommitted"}
    [pscustomobject]@{Commit=$commit[0];UncommittedTrackedChanges=$changes.Count}
}
function Get-Toolchain {
    $rustc=@(& rustc -Vv)
    if($LASTEXITCODE -ne 0){throw 'rustc -Vv failed'}
    $cargo=@(& cargo -V)
    if($LASTEXITCODE -ne 0){throw 'cargo -V failed'}
    [pscustomobject]@{Rustc=$rustc;Cargo=$cargo[0]}
}
function Write-Json($Value,[string]$Path){
    $Value | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $Path -Encoding utf8
}

$null=New-Item -ItemType Directory -Path $target -Force
$lease=[IO.File]::Open((Join-Path $target 'project-lease.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
try {
    # Validate before creating any candidate directory. A test build or old
    # executable left in target is not evidence of a current player build.
    $receipt=Assert-ModernBuildReceipt $project 'Release'
    $exe=Get-Item -LiteralPath (Get-ModernPlayerPath $project 'Release')
    $image=Assert-ModernPortableExecutable $exe.FullName @($project,$project.Replace('\','/'))
    $version=Get-ModernPlayerVersion $exe.FullName 'Release'
    $null=& (Join-Path $PSScriptRoot 'verify-independent.ps1')
    $revision=Get-SourceRevision
    $toolchain=Get-Toolchain
    $entrySource=Join-Path $project 'packaging/Start-LiToModern.cmd'
    $readmeSource=Join-Path $project 'packaging/README.md'

    $dist=[IO.Path]::GetFullPath((Join-Path $project 'dist'))
    if(Test-Path -LiteralPath $dist){
        $item=Get-Item -LiteralPath $dist
        if(!$item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)){throw 'Unsafe dist directory'}
    } else {$null=New-Item -ItemType Directory -Path $dist}
    $destination=[IO.Path]::GetFullPath((Join-Path $dist $Candidate))
    $zipPath=$destination + '.zip'
    if(Test-Path -LiteralPath $destination){throw 'Candidate already exists; packaging never overwrites an earlier candidate'}
    if($Zip -and (Test-Path -LiteralPath $zipPath)){throw 'Portable ZIP already exists; packaging never overwrites an earlier candidate'}
    $staging=[IO.Path]::GetFullPath((Join-Path $dist ('.' + $Candidate + '-' + [Guid]::NewGuid().ToString('N') + '.partial')))
    # Both resolved move targets are explicit children of this project's dist.
    foreach($path in @($staging,$destination)){
        if([IO.Path]::GetDirectoryName($path) -ine $dist){throw 'Candidate path escapes dist'}
    }

    # The slow, read-only part first: nothing is staged if the resources cannot be listed.
    $resources=Get-ModernResourceManifest $ResourceRoot
    if($resources.FileCount -eq 0){throw 'Resource root holds no files'}
    $catalog=if($EffectCatalog){
        $item=Get-Item -LiteralPath $EffectCatalog
        [pscustomobject]@{Path=$item.FullName;Length=$item.Length;SHA256=(Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash}
    }else{$null}

    $null=New-Item -ItemType Directory -Path $staging,(Join-Path $staging 'manifest')
    Copy-Item -LiteralPath $exe.FullName -Destination (Join-Path $staging 'LiToModern.exe')
    Copy-Item -LiteralPath $readmeSource -Destination (Join-Path $staging 'README.md')
    Copy-ModernPortableEntry $entrySource (Join-Path $staging 'Start-LiToModern.cmd')
    Write-Json ([pscustomobject]@{
        Schema=1;Repository='lito-modern';Commit=$revision.Commit;UncommittedTrackedChanges=$revision.UncommittedTrackedChanges
        BuildInputSHA256=$receipt.InputSHA256;BuildInputs=$receipt.Inputs
    }) (Join-Path $staging 'manifest/source.json')
    Write-Json ([pscustomobject]@{
        Schema=1;File='LiToModern.exe';Length=$exe.Length;SHA256=$receipt.SHA256;BuiltUtc=$receipt.CreatedUtc
        Target=$receipt.Target;Version=$version;Image=$image;Toolchain=$toolchain
        CRuntime='static (no Visual C++ Redistributable or VCRUNTIME140.dll needed)'
        MinimumWindows=$script:ModernMinimumWindows
    }) (Join-Path $staging 'manifest/executable.json')
    Write-Json ([pscustomobject]@{
        Schema=1;Packaged=$false
        Note='External read-only client resources this candidate was prepared against. They are not part of this package; LiTo.bin and other executables found under the root are listed by hash only and are never started by the player.'
        Root=$resources.Root;FileCount=$resources.FileCount;TotalBytes=$resources.TotalBytes;ListSHA256=$resources.ListSHA256
        SkippedReparsePoints=$resources.SkippedReparsePoints;EffectCatalog=$catalog;Files=$resources.Files
    }) (Join-Path $staging 'manifest/resources.json')

    $files=@(Get-ChildItem -LiteralPath $staging -File -Recurse)
    $names=@($files|ForEach-Object{[IO.Path]::GetRelativePath($staging,$_.FullName).Replace('\','/')}|Sort-Object)
    $expected=@('LiToModern.exe','README.md','Start-LiToModern.cmd','manifest/executable.json','manifest/resources.json','manifest/source.json')|Sort-Object
    if(($names -join '|') -cne ($expected -join '|')){throw 'Candidate contains unexpected payload'}
    $finalReceipt=Assert-ModernBuildReceipt $project 'Release'
    if($receipt.SHA256 -cne $finalReceipt.SHA256 -or $receipt.InputSHA256 -cne $finalReceipt.InputSHA256){throw 'Build changed during packaging'}
    foreach($pair in @(@('LiToModern.exe',$exe.FullName),@('README.md',$readmeSource))){
        if((Get-FileHash -LiteralPath (Join-Path $staging $pair[0]) -Algorithm SHA256).Hash -cne (Get-FileHash -LiteralPath $pair[1] -Algorithm SHA256).Hash){throw 'Candidate copy mismatch'}
    }
    $manifest=@($files|Sort-Object FullName|ForEach-Object{[pscustomobject]@{Path=[IO.Path]::GetRelativePath($staging,$_.FullName).Replace('\','/');Length=$_.Length;SHA256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}})
    Write-Json ([pscustomobject]@{
        Schema=3;Candidate=$Candidate;CreatedUtc=[DateTime]::UtcNow.ToString('o')
        SourceCommit=$revision.Commit;BuildInputSHA256=$receipt.InputSHA256;ExecutableSHA256=$receipt.SHA256
        ResourceListSHA256=$resources.ListSHA256;MinimumWindows=$script:ModernMinimumWindows.Name;Version=$version;Files=$manifest
    }) (Join-Path $staging 'SHA256.json')
    # Directory.Move fails if the destination appeared meanwhile; it never
    # treats that directory as a container and nests an unfinished candidate.
    [IO.Directory]::Move($staging,$destination)
    $archive=if($Zip){
        $item=New-ModernPortableZip $destination $zipPath
        [pscustomobject]@{Path=$item.FullName;Length=$item.Length;SHA256=(Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash}
    }else{$null}
    [pscustomobject]@{
        Candidate=$destination;Zip=$archive;SourceCommit=$revision.Commit;ExecutableSHA256=$receipt.SHA256;BuildInputSHA256=$receipt.InputSHA256
        Imports=$image.Imports;ResourceFiles=$resources.FileCount;ResourceListSHA256=$resources.ListSHA256;Files=$manifest
    }
} finally { $lease.Dispose() }
