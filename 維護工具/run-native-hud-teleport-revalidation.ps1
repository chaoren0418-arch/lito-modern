#Requires -Version 7.0
param([ValidateSet('Debug','Release')][string]$BuildProfile,[ValidateSet('timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry')][string[]]$Scenarios=@('timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry'),[ValidateSet('recovery-batch','recovery-supplement','visual-recheck')][string]$ReceiptLabel='recovery-batch')
$ErrorActionPreference='Stop';$taskRoot=Split-Path -Parent $PSScriptRoot
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$stage=Join-Path $taskRoot '驗證紀錄/current/native-status-scroll/teleport-revalidation';New-Item -ItemType Directory -Path $stage -Force|Out-Null
$caseRoot=Join-Path $taskRoot '驗證紀錄/current/native-teleport-gui'
$manifest=Get-Content -LiteralPath (Join-Path $taskRoot ('驗證紀錄/current/native-status-scroll/'+$BuildProfile.ToLowerInvariant()+'-build-inputs-qualified.json')) -Raw|ConvertFrom-Json
$batch=[ordered]@{status='running';profile=$BuildProfile;started_utc=[DateTime]::UtcNow.ToString('o');main_sha256=$manifest.main.sha256;active_scenario=$null;cases=@();full_game_complete=$false}
$receiptPath=Join-Path $stage ($BuildProfile.ToLowerInvariant()+'-'+$ReceiptLabel+'.json')
function Save {$batch|ConvertTo-Json -Depth 12|Set-Content -LiteralPath $receiptPath -Encoding utf8}
Save
try {
 foreach($scenario in $Scenarios){
  $batch.active_scenario=$scenario;Save
  $before=@(Get-ChildItem -LiteralPath $caseRoot -Directory|Select-Object -ExpandProperty Name)
  $log=Join-Path $stage ($BuildProfile.ToLowerInvariant()+'-'+$ReceiptLabel+'-'+$scenario+'.log')
  & (Join-Path $PSScriptRoot 'run-isolated-native-teleport-window-test.ps1') -BuildProfile $BuildProfile -Scenario $scenario -ReplayStaleCommands:($scenario -eq 'cancel-retry') *> $log
  $matches=@(Get-ChildItem -LiteralPath $caseRoot -Directory|Where-Object {$_.Name -notin $before}|ForEach-Object {$p=Join-Path $_.FullName 'receipt.json';if(Test-Path -LiteralPath $p){$d=Get-Content -LiteralPath $p -Raw|ConvertFrom-Json;if($d.exe_sha256 -eq $manifest.main.sha256 -and $d.scenario -eq $scenario){$_.FullName}}})
  if($matches.Count -ne 1){throw 'Cannot uniquely bind completed case to profile/scenario'}
  $case=$matches[0]
  & python -X utf8 (Join-Path $PSScriptRoot 'verify-native-teleport-recovery-gui.py') $case >> $log
  if($LASTEXITCODE -ne 0){throw ('Independent recovery comparison failed '+$scenario)}
  & python -X utf8 (Join-Path $PSScriptRoot 'verify-teleport-glyph-pixels.py') $case >> $log
  if($LASTEXITCODE -ne 0){throw ('Original glyph comparison failed '+$scenario)}
  & python -X utf8 (Join-Path $PSScriptRoot 'qualify-current-hud-teleport-case.py') $case --profile $BuildProfile >> $log
  if($LASTEXITCODE -ne 0){throw ('Current-source binding failed '+$scenario)}
  $q=Get-Content -LiteralPath (Join-Path $case 'HUD-source-qualification.json') -Raw|ConvertFrom-Json
  $batch.cases+=@($q);Save
 }
 $batch.status='verified_current_source_GUI_recovery_scenarios';$batch.active_scenario=$null;$batch.completed_utc=[DateTime]::UtcNow.ToString('o');Save
 Write-Output ($BuildProfile+' recovery batch passed '+$Scenarios.Count+' scenarios; exact per-case receipts retained.')
}catch{$batch.status='failed';$batch.error=$_.Exception.Message;$batch.error_stack=$_.ScriptStackTrace;Save;throw}