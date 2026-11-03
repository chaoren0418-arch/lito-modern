#Requires -Version 7.0
[CmdletBinding()]param(
    [string]$ResourceRoot='D:\lineage開發\0-客戶端',
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
$project=Join-Path $PSScriptRoot 'LiTo381-Modern'
$evidence=Join-Path $PSScriptRoot '驗證紀錄\current'
New-Item -ItemType Directory -Path $evidence -Force|Out-Null
$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$scratch=Join-Path $project 'target\current\scratch'
New-Item -ItemType Directory -Path $scratch -Force|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
Push-Location $project
try {
    $buildArgs=@('build','-j','1','--offline','--locked','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--bin','LiToModern')
    if($BuildProfile -eq 'Release'){$buildArgs+='--release'}
    $buildLog=Join-Path $evidence $(if($BuildProfile -eq 'Release'){'build-release.log'}else{'build.log'})
    & cargo @buildArgs *> $buildLog
    if($LASTEXITCODE -ne 0){Get-Content -LiteralPath $buildLog -Tail 30;throw 'x64 build failed'}
    $profileDir=$BuildProfile.ToLowerInvariant()
    $exe=Join-Path $project ('target\current\x86_64-pc-windows-msvc\'+$profileDir+'\LiToModern.exe')
    if($BuildOnly){Write-Output $exe;return}
    if($NativeLoginEndpoint){
        if($ResourceRoot.Contains('"')){throw 'Invalid resource path'}
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
