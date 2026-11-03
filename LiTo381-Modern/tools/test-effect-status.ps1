# Frozen R381 asset verification. CatalogOnly needs no mutable Java fixtures.
param([switch]$CatalogOnly)
$ErrorActionPreference='Stop'
$stageRoot=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$litoRoot=Split-Path (Split-Path $stageRoot)
$workspace=Split-Path $litoRoot
$server=Join-Path $litoRoot 'lito伺服器\LiTo381-Server'
$testExe=Join-Path $stageRoot 'target\r381-effect-status\i686-pc-windows-msvc\debug\deps\lito_modern-e61d19571b185afc.exe'
if((Get-FileHash -LiteralPath $testExe).Hash -ne '11BC1A31DE6659B04E00859FD3C9AD1262222AF07AAA822B1DFDCD83247C79DF'){throw 'Frozen test executable differs'}
$plain=Join-Path $workspace '客戶端xml\effectlist2.xml'
$encrypted=Join-Path $workspace '客戶端xml\effectlist2.xml_e.xml'
$catalogChecks=@{
    $plain='07EFD481DD89B63DA0BE8AF0F1643416A3E875B69F4B7FFFB8BE0683EC05C130'
    $encrypted='6DE9938C07D28EF8BFC6F8AE61A1AAC786D64604574E4E44AA71C84F9395BF55'
}
foreach($path in $catalogChecks.Keys){if((Get-FileHash -LiteralPath $path).Hash -ne $catalogChecks[$path]){throw ('Reference changed: '+$path)}}
if(-not $CatalogOnly){
    $classes=@(Get-Content -LiteralPath (Join-Path $stageRoot 'logs\R380-selected-classes-compiled.log') -Raw|ConvertFrom-Json)
    if($classes.Count -ne 1181){throw 'Unexpected class manifest'}
    $legacyPrefix='D:\lineage開發\LiTo381-Server\'
    foreach($entry in $classes){
        $path=if($entry.Path.StartsWith($legacyPrefix,[StringComparison]::OrdinalIgnoreCase)){Join-Path $server $entry.Path.Substring($legacyPrefix.Length)}else{$entry.Path}
        if((Get-FileHash -LiteralPath $path).Hash -ne $entry.Hash){throw ('Frozen Java input changed: '+$path)}
    }
    $r309Inputs=@(Get-Content -LiteralPath (Join-Path $stageRoot 'logs\modern-preflight-20260927T071602Z-c91478138da64c8aa97edcecbe0b4dd0\inputs.json') -Raw|ConvertFrom-Json)
    $r309Jars=@($r309Inputs|Where-Object{$_.Path.StartsWith('C:\Users\CHAOREN\.m2\repository\') -and $_.Path.EndsWith('.jar')}|Select-Object -ExpandProperty Path)
}
$keys=@('LITO_MODERN_TEST_JAVA','LITO_MODERN_TEST_JAVA_CLASSPATH','LITO_SHARED_RESOURCE_ROOT','LITO_MODERN_TEST_SERVER_ROOT','LITO_EFFECTLIST_REFERENCE','LITO_EFFECTLIST_ENCRYPTED_REFERENCE','LITO_VIP_STATUS_EVIDENCE')
$saved=@{}
foreach($key in $keys){$saved[$key]=[Environment]::GetEnvironmentVariable($key,'Process')}
Push-Location $stageRoot
try {
    $env:LITO_SHARED_RESOURCE_ROOT=Join-Path $workspace '0-客戶端'
    $env:LITO_EFFECTLIST_REFERENCE=$plain
    $env:LITO_EFFECTLIST_ENCRYPTED_REFERENCE=$encrypted
    $env:LITO_VIP_STATUS_EVIDENCE=$null
    & $testExe effect_catalog --include-ignored --nocapture --test-threads=1
    if($LASTEXITCODE -ne 0){throw 'Effect catalog verification failed'}
    if(-not $CatalogOnly){
        $env:LITO_MODERN_TEST_JAVA='C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe'
        $env:LITO_MODERN_TEST_JAVA_CLASSPATH="$server\target\test-classes;$server\target\classes;$server\target\lib\*;"+($r309Jars -join ';')
        $env:LITO_MODERN_TEST_SERVER_ROOT=$server
        & $testExe java_vip_status_icons_follow_source_owned_equipment_without_invented_xml_timers --ignored --nocapture --test-threads=1
        if($LASTEXITCODE -ne 0){throw 'VIP status verification failed'}
    }
} finally {
    Pop-Location
    foreach($key in $keys){[Environment]::SetEnvironmentVariable($key,$saved[$key],'Process')}
}