#Requires -Version 7.0
[CmdletBinding(SupportsShouldProcess)]
param([ValidateSet('Test','TestRelease','Debug','Release')][string]$Mode='Test')
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false

$defaultClientRoot='D:\lineage開發\0-客戶端'
$repoRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..'))
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
if([string]::IsNullOrWhiteSpace($env:LITO_CLIENT_ROOT)){$env:LITO_CLIENT_ROOT=$defaultClientRoot}
$clientRoot=$env:LITO_CLIENT_ROOT
if($clientRoot.Contains('"')){throw 'Invalid LITO_CLIENT_ROOT'}
if(-not [IO.Path]::IsPathFullyQualified($clientRoot) -and $clientRoot -notmatch '^[A-Za-z]:[\\/]'){
    $clientRoot=[IO.Path]::GetFullPath((Join-Path $repoRoot $clientRoot))
    $env:LITO_CLIENT_ROOT=$clientRoot
}elseif([IO.Path]::IsPathFullyQualified($clientRoot)){
    $clientRoot=[IO.Path]::GetFullPath($clientRoot)
    $env:LITO_CLIENT_ROOT=$clientRoot
}

$target=Join-Path $project 'target'
$logs=Join-Path $project 'logs'
$manifest=Join-Path $project 'Cargo.toml'
$evidenceScript=Join-Path $PSScriptRoot 'build-evidence.ps1'
if(!(Test-Path -LiteralPath $manifest -PathType Leaf)){throw "Cargo.toml not found: $manifest (repo root $repoRoot). Clone the full repository and run LiTo381-Modern/tools/build.ps1."}
if(!(Test-Path -LiteralPath $evidenceScript -PathType Leaf)){throw "Missing helper: $evidenceScript"}
if(!(Test-Path -LiteralPath $clientRoot -PathType Container)){
    Write-Warning "LITO_CLIENT_ROOT not found: $clientRoot. cargo compile does not copy pak/map/sprite; set LITO_CLIENT_ROOT to an existing client tree if a mode needs assets. Default: $defaultClientRoot."
}

. $evidenceScript
$cargoArgs=@('--offline','--locked','--manifest-path',$manifest,'--target-dir',$target,'--target','i686-pc-windows-msvc')
$verb=if($Mode -in @('Test','TestRelease')){'test'}else{'build'}
if($Mode -in @('Release','TestRelease')){$cargoArgs+='--release'}
$cargoCommand="cargo $verb "+($cargoArgs -join ' ')
if(-not $PSCmdlet.ShouldProcess($project,$cargoCommand)){
    [pscustomobject]@{Mode=$Mode;RepoRoot=$repoRoot;Project=$project;Target=$target;ClientRoot=$clientRoot;Cargo=$cargoCommand;WhatIf=$true}
    return
}

$null=New-Item -ItemType Directory -Path $target,$logs -Force
# Held by this project's build/test/packaging tools only. LHZ has no dependency
# on this lease, directory or Cargo artifact.
$lease=[IO.File]::Open((Join-Path $target 'project-lease.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
try {
    $inputs=if($Mode -in @('Debug','Release')){@(Get-ModernBuildInputs $project)}else{@()}
    if($Mode -in @('Debug','Release')){$null=Get-ModernPlayerPath $project $Mode}
    $log=Join-Path $logs ("build-$($Mode.ToLower())-"+[DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffZ')+'.log')
    $output=@(& cargo $verb @cargoArgs 2>&1 | ForEach-Object { "$_" })
    $code=$LASTEXITCODE
    $output | Set-Content -LiteralPath $log -Encoding utf8
    $output | Select-Object -Last 18
    if($code -ne 0){throw "LiTo $Mode failed. See $log"}
    $receipt=if($Mode -in @('Debug','Release')){Write-ModernBuildReceipt $project $Mode $inputs $log}else{$null}
    [pscustomobject]@{Mode=$Mode;Log=$log;Target=$target;Receipt=$receipt;ClientRoot=$clientRoot}
} finally { $lease.Dispose() }
