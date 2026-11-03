#Requires -Version 7.0
[CmdletBinding()]
param([ValidateSet('Test','TestRelease','Debug','Release')][string]$Mode='Test')
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$target=Join-Path $project 'target'
$logs=Join-Path $project 'logs'
. (Join-Path $PSScriptRoot 'build-evidence.ps1')
$null=New-Item -ItemType Directory -Path $target,$logs -Force
# Held by this project's build/test/packaging tools only. LHZ has no dependency
# on this lease, directory or Cargo artifact.
$lease=[IO.File]::Open((Join-Path $target 'project-lease.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
try {
    $inputs=if($Mode -in @('Debug','Release')){@(Get-ModernBuildInputs $project)}else{@()}
    if($Mode -in @('Debug','Release')){$null=Get-ModernPlayerPath $project $Mode}
    $cargoArgs=@('--offline','--locked','--manifest-path',(Join-Path $project 'Cargo.toml'),'--target-dir',$target,'--target','i686-pc-windows-msvc')
    $verb=if($Mode -in @('Test','TestRelease')){'test'}else{'build'}
    if($Mode -in @('Release','TestRelease')){$cargoArgs+='--release'}
    $log=Join-Path $logs ("build-$($Mode.ToLower())-"+[DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffZ')+'.log')
    $output=@(& cargo $verb @cargoArgs 2>&1 | ForEach-Object { "$_" })
    $code=$LASTEXITCODE
    $output | Set-Content -LiteralPath $log -Encoding utf8
    $output | Select-Object -Last 18
    if($code -ne 0){throw "LiTo $Mode failed. See $log"}
    $receipt=if($Mode -in @('Debug','Release')){Write-ModernBuildReceipt $project $Mode $inputs $log}else{$null}
    [pscustomobject]@{Mode=$Mode;Log=$log;Target=$target;Receipt=$receipt}
} finally { $lease.Dispose() }
