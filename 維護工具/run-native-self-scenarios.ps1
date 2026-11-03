#Requires -Version 7.0
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$root=Split-Path -Parent $PSScriptRoot
$out=Join-Path $root '驗證紀錄/current/native-real-peers'
$archive=Join-Path $out 'before-self-scenarios'
New-Item -ItemType Directory -Path $archive -Force|Out-Null
foreach($name in @('native-reagent-skills','native-ground-pickup')){
    $source=Join-Path $root ("驗證紀錄/current/$name/receipt.json")
    $dest=Join-Path $archive ($name+'-receipt.json')
    if((Test-Path -LiteralPath $source) -and !(Test-Path -LiteralPath $dest)){Copy-Item -LiteralPath $source -Destination $dest}
}
$stages=@(
    @{Name='self-and-refusal';Script='verify-native-combat.ps1';Options=@('-SelfOnly')},
    @{Name='ground-item-actions';Script='verify-native-item-actions.ps1';Options=@('-GroundOnly')},
    @{Name='reagent-skills';Script='verify-native-reagent-skills.ps1';Options=@()},
    @{Name='ground-pickup';Script='verify-native-ground-pickup.ps1';Options=@()}
)
foreach($stage in $stages){
    $parameters=$stage.Options
    & pwsh -NoProfile -File (Join-Path $PSScriptRoot $stage.Script) -SkipBuild @parameters *> (Join-Path $out ($stage.Name+'-migrated.log'))
    if($LASTEXITCODE -ne 0){throw ($stage.Name+' failed; see '+$out)}
    Write-Output ($stage.Name+' current authority and presentation verified')
}
