#Requires -Version 7.0
param(
 [Parameter(Mandatory)][ValidateSet('Debug','Release')][string]$BuildProfile,
 [Parameter(Mandatory)][string]$NormalQualification,
 [Parameter(Mandatory)][string]$RustEvidence,
 [Parameter(Mandatory)][string]$CohortDirectory,
 [ValidateSet('cross-endpoint','timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry')][string[]]$Scenarios=@('cross-endpoint','timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry'),
 [ValidateSet('lito-legacy','tw13081901-v1')][string]$ResourceProfile='lito-legacy',
 [string]$CommonCatalogPath='',
 [switch]$ExtendedItemDetails,
 [ValidatePattern('^[a-z0-9-]{1,40}$')][string]$ReceiptLabel='recovery-batch'
)
$ErrorActionPreference='Stop';$taskRoot=Split-Path -Parent $PSScriptRoot
$commonCatalogSHA=$null;if($CommonCatalogPath){$commonCatalogSHA=(Get-FileHash -LiteralPath $CommonCatalogPath).Hash}
if($ResourceProfile -eq 'tw13081901-v1' -and (!$ExtendedItemDetails -or $commonCatalogSHA -ne 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01')){throw 'Current formal TW GUI cohort requires extended details and pinned owned catalog'}
$qualifierProfileArgs=@('--resource-profile',$ResourceProfile);if($ExtendedItemDetails){$qualifierProfileArgs+='--extended-item-details'}
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$stage=[IO.Path]::GetFullPath($CohortDirectory)
$ownedEvidence=[IO.Path]::GetFullPath((Join-Path $taskRoot '驗證紀錄/current'))+'\'
if(!$stage.StartsWith($ownedEvidence,[StringComparison]::OrdinalIgnoreCase)){throw 'GUI cohort must stay in owned evidence'}
New-Item -ItemType Directory -Path $stage -Force|Out-Null
$manifest=Get-Content -LiteralPath $NormalQualification -Raw|ConvertFrom-Json
$rust=Get-Content -LiteralPath $RustEvidence -Raw|ConvertFrom-Json
$main=$manifest.profiles.$BuildProfile
if($main.features.Count -ne 0 -or !$main.unique_entry -or $rust.profile -ne $BuildProfile -or $rust.failed -ne 0 -or $rust.integration_targets -ne 93 -or $rust.products_retained -ne 94 -or $rust.normal_mains.$BuildProfile -ne $main.sha256){throw 'Current GUI requires the exact normal/default Rust evidence'}
$expected=Get-Content -LiteralPath (Join-Path (Split-Path -Parent $NormalQualification) 'source-inputs.json') -Raw|ConvertFrom-Json -AsHashtable
if($expected.Count -ne $manifest.source_files){throw 'GUI source manifest count mismatch'}
$installedRust='C:/Users/CHAOREN/.rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
$previousRustc=$env:RUSTC;$previousRustdoc=$env:RUSTDOC;$previousPath=$env:PATH
$env:RUSTC=Join-Path $installedRust 'rustc.exe';$env:RUSTDOC=Join-Path $installedRust 'rustdoc.exe';$env:PATH=$installedRust+';'+$env:PATH
$caseRoot=Join-Path $taskRoot '驗證紀錄/current/native-teleport-gui'
$receiptPath=Join-Path $stage ($BuildProfile+'-'+$ReceiptLabel+'.json')
if(Test-Path -LiteralPath $receiptPath){throw 'Existing batch retained; use a distinct receipt label for unfinished scenarios'}
$batch=[ordered]@{status='running';profile=$BuildProfile;resource_profile=$ResourceProfile;extended_item_details=$ExtendedItemDetails.IsPresent;common_catalog_sha256=$commonCatalogSHA;started_utc=[DateTime]::UtcNow.ToString('o');main_sha256=$main.sha256;normal_qualification_sha256=(Get-FileHash -LiteralPath $NormalQualification).Hash;Rust_evidence_sha256=(Get-FileHash -LiteralPath $RustEvidence).Hash;driver_sha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash;active_scenario=$null;cases=@();whole_goal_complete=$false}
function Save {$batch|ConvertTo-Json -Depth 14|Set-Content -LiteralPath $receiptPath -Encoding utf8}
function VerifySources {if((Get-FileHash -LiteralPath $main.path).Hash -ne $main.sha256){throw 'GUI normal main drift'};foreach($name in $expected.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $expected[$name]){throw ('Source drift: '+$name)}}}
Save
try {
 foreach($scenario in $Scenarios){
  VerifySources
  $batch.active_scenario=$scenario;Save
  $before=@(Get-ChildItem -LiteralPath $caseRoot -Directory|Select-Object -ExpandProperty Name)
  $log=Join-Path $stage ($BuildProfile+'-'+$ReceiptLabel+'-'+$scenario+'.log')
  & (Join-Path $PSScriptRoot 'run-isolated-native-teleport-window-test.ps1') -BuildProfile $BuildProfile -Scenario $scenario -ReplayStaleCommands:($scenario -eq 'cancel-retry') -ResourceProfile $ResourceProfile -CommonCatalogPath $CommonCatalogPath -ExtendedItemDetails:$ExtendedItemDetails *> $log
  $matches=@(Get-ChildItem -LiteralPath $caseRoot -Directory|Where-Object {$_.Name -notin $before}|ForEach-Object {
   $p=Join-Path $_.FullName 'receipt.json'
   if(Test-Path -LiteralPath $p){$d=Get-Content -LiteralPath $p -Raw|ConvertFrom-Json;if($d.exe_sha256 -eq $main.sha256 -and $d.scenario -eq $scenario){$_.FullName}}
  })
  if($matches.Count -ne 1){throw 'Cannot uniquely bind profile/scenario case'}
  $case=$matches[0]
  foreach($validator in @($(if($scenario -eq 'cross-endpoint'){'verify-native-teleport-gui-authority.py'}else{'verify-native-teleport-recovery-gui.py'}),'verify-teleport-glyph-pixels.py','verify-current-hud-teleport-scenes.py')){
   & python -X utf8 (Join-Path $PSScriptRoot $validator) $case >> $log
   if($LASTEXITCODE -ne 0){throw ('Independent comparison failed: '+$validator+' '+$scenario)}
  }
  if($ResourceProfile -eq 'tw13081901-v1' -and $scenario -eq 'cross-endpoint'){
   & python -X utf8 (Join-Path $PSScriptRoot 'verify-TW-teleport-inventory-continuity.py') $case >> $log
   if($LASTEXITCODE -ne 0){throw 'TW full inventory field/detail continuity comparison failed'}
  }
  if($ResourceProfile -eq 'tw13081901-v1' -and $scenario -ne 'cross-endpoint'){
   & python -X utf8 (Join-Path $PSScriptRoot 'verify-teleport-recovery-inventory-continuity.py') $case --expected-resource-profile tw13081901-v1 >> $log
   if($LASTEXITCODE -ne 0){throw 'TW recovery complete staged/reentered item fields and wire details comparison failed'}
  }
  if($scenario -eq 'cancel-retry'){
   & python -X utf8 (Join-Path $PSScriptRoot 'verify-native-teleport-stale-commands.py') $case >> $log
   if($LASTEXITCODE -ne 0){throw 'Stale GUI notification comparison failed'}
  }
  & python -X utf8 (Join-Path $PSScriptRoot 'qualify-native-game-timer-teleport-current-case.py') $case --profile $BuildProfile --normal-qualification $NormalQualification --rust-evidence $RustEvidence --cohort $stage @qualifierProfileArgs >> $log
  if($LASTEXITCODE -ne 0){throw 'Default source/build binding or exact movement comparison failed'}
  VerifySources
  $q=Get-Content -LiteralPath (Join-Path $case 'current-default-source-qualification.json') -Raw|ConvertFrom-Json
  $batch.cases+=@($q);Save
 }
 $batch.status='verified_current_default_main_GUI_selected_teleport_scenarios';$batch.active_scenario=$null;$batch.completed_utc=[DateTime]::UtcNow.ToString('o');Save
}catch{$batch.status='failed';$batch.error=$_.Exception.Message;$batch.error_stack=$_.ScriptStackTrace;Save;throw}

finally{$env:RUSTC=$previousRustc;$env:RUSTDOC=$previousRustdoc;$env:PATH=$previousPath}
