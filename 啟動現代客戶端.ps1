#Requires -Version 7.0
[CmdletBinding(SupportsShouldProcess)]
param(
    [string]$ResourceRoot,
    [switch]$BuildOnly,
    [switch]$Verify,
    [switch]$OfflineScene,
    [switch]$ConfigureNative,
    [string]$NativeConfig,
    [string]$VerificationOutputDirectory,
    [ValidateSet('Debug','Release')][string]$BuildProfile='Debug',
    [string]$ProbeNativeEndpoint,
    [string]$NativeLoginEndpoint
)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
if(@($BuildOnly.IsPresent,$Verify.IsPresent,$OfflineScene.IsPresent,$ConfigureNative.IsPresent,![string]::IsNullOrEmpty($NativeConfig),![string]::IsNullOrEmpty($ProbeNativeEndpoint),![string]::IsNullOrEmpty($NativeLoginEndpoint)).Where({$_}).Count -gt 1){throw 'Choose one mode'}

$defaultClientRoot='D:\lineage開發\0-客戶端'
$repoRoot=$PSScriptRoot
$project=Join-Path $repoRoot 'LiTo381-Modern'
$manifest=Join-Path $project 'Cargo.toml'
if(!(Test-Path -LiteralPath $manifest -PathType Leaf)){throw "Cargo.toml not found: $manifest. Run this script from a complete clone; paths are relative to the repo root."}

if(-not $PSBoundParameters.ContainsKey('ResourceRoot') -or [string]::IsNullOrWhiteSpace($ResourceRoot)){
    if(-not [string]::IsNullOrWhiteSpace($env:LITO_CLIENT_ROOT)){$ResourceRoot=$env:LITO_CLIENT_ROOT}else{$ResourceRoot=$defaultClientRoot}
}
if($ResourceRoot.Contains('"')){throw 'Invalid resource path'}
if(-not [string]::IsNullOrWhiteSpace($ResourceRoot) -and -not [IO.Path]::IsPathFullyQualified($ResourceRoot) -and $ResourceRoot -notmatch '^[A-Za-z]:[\\/]'){
    $ResourceRoot=[IO.Path]::GetFullPath((Join-Path $repoRoot $ResourceRoot))
}elseif([IO.Path]::IsPathFullyQualified($ResourceRoot)){
    $ResourceRoot=[IO.Path]::GetFullPath($ResourceRoot)
}

$needsResourceRoot=$Verify.IsPresent -or $OfflineScene.IsPresent -or -not [string]::IsNullOrEmpty($NativeLoginEndpoint)
if($needsResourceRoot -and !(Test-Path -LiteralPath $ResourceRoot -PathType Container)){
    throw "Resource root not found: $ResourceRoot. Set LITO_CLIENT_ROOT or pass -ResourceRoot to an existing client directory (default: $defaultClientRoot). This script does not copy pak/map/sprite."
}

$evidence=Join-Path $repoRoot '驗證紀錄\current'
$buildArgs=@('build','-j','1','--offline','--locked','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--bin','LiToModern')
if($BuildProfile -eq 'Release'){$buildArgs+='--release'}
$profileDir=$BuildProfile.ToLowerInvariant()
$exe=Join-Path $project ('target\current\x86_64-pc-windows-msvc\'+$profileDir+'\LiToModern.exe')
$cargoCommand='cargo '+($buildArgs -join ' ')
if(-not $PSCmdlet.ShouldProcess($project,$cargoCommand)){
    [pscustomobject]@{RepoRoot=$repoRoot;Project=$project;ResourceRoot=$ResourceRoot;Exe=$exe;Cargo=$cargoCommand;BuildProfile=$BuildProfile;WhatIf=$true}
    return
}

$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$scratch=Join-Path $project 'target\current\scratch'
New-Item -ItemType Directory -Path $evidence,$scratch -Force|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
Push-Location $project
try {
    $buildLog=Join-Path $evidence $(if($BuildProfile -eq 'Release'){'build-release.log'}else{'build.log'})
    & cargo @buildArgs *> $buildLog
    if($LASTEXITCODE -ne 0){Get-Content -LiteralPath $buildLog -Tail 30;throw 'x64 build failed'}
    if($BuildOnly){Write-Output $exe;return}
    if($NativeLoginEndpoint){
        Start-Process -FilePath $exe -ArgumentList @('--native-login',$NativeLoginEndpoint,('"'+$ResourceRoot+'"')) -WorkingDirectory $project -WindowStyle Hidden
    }elseif($ProbeNativeEndpoint){
        & $exe --probe-native-loopback $ProbeNativeEndpoint | Tee-Object -FilePath (Join-Path $evidence $(if($BuildProfile -eq 'Release'){'native-probe-release.json'}else{'native-probe.json'}))
        if($LASTEXITCODE -ne 0){throw 'Native loopback probe failed'}
    }elseif($Verify){
        $diagnosticOutput=if($VerificationOutputDirectory){$VerificationOutputDirectory}else{Join-Path $project 'target/current/verification'}
        if(![IO.Path]::IsPathFullyQualified($diagnosticOutput)){throw 'Diagnostic output must be absolute'}
        & $exe --verify-resources $ResourceRoot --output $diagnosticOutput | Tee-Object -FilePath (Join-Path $evidence $(if($BuildProfile -eq 'Release'){'resource-verification-release.json'}else{'resource-verification.json'}))
        if($LASTEXITCODE -ne 0){throw 'Resource verification failed'}
    }elseif($OfflineScene){
        Start-Process -FilePath $exe -ArgumentList @('--offline-scene',('"'+$ResourceRoot+'"')) -WorkingDirectory $project -WindowStyle Hidden
    }elseif($ConfigureNative){
        Start-Process -FilePath $exe -ArgumentList @('--configure-native') -WorkingDirectory $project -WindowStyle Hidden
    }elseif($NativeConfig){
        if($NativeConfig.Contains('"') -or ![IO.Path]::IsPathFullyQualified($NativeConfig)){throw 'Native config must be an absolute path'}
        Start-Process -FilePath $exe -ArgumentList @('--native-config',('"'+$NativeConfig+'"')) -WorkingDirectory $project -WindowStyle Hidden
    }else{
        Start-Process -FilePath $exe -WorkingDirectory $project -WindowStyle Hidden
    }
}finally { Pop-Location;$env:TEMP=$previousTemp;$env:TMP=$previousTmp }
