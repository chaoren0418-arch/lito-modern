#Requires -Version 7.0
param(
 [Parameter(Mandatory)][ValidateSet('Debug','Release')][string]$BuildProfile,
 [ValidateSet('timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry')][string[]]$Scenarios=@('timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry'),
 [ValidatePattern('^[a-z0-9-]{1,40}$')][string]$ReceiptLabel='recovery-batch'
)
$ErrorActionPreference='Stop';$taskRoot=Split-Path -Parent $PSScriptRoot
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$stage=Join-Path $taskRoot '驗證紀錄/current/native-teleport-wire'
$manifest=Get-Content -LiteralPath (Join-Path $stage ($BuildProfile+'-default-qualified.json')) -Raw|ConvertFrom-Json
$expected=Get-Content -LiteralPath (Join-Path $stage 'green-inputs-before.json') -Raw|ConvertFrom-Json -AsHashtable
$caseRoot=Join-Path $taskRoot '驗證紀錄/current/native-teleport-gui'
$receiptPath=Join-Path $stage ($BuildProfile+'-'+$ReceiptLabel+'.json')
if(Test-Path -LiteralPath $receiptPath){throw 'Existing batch retained; use a distinct receipt label for unfinished scenarios'}
$batch=[ordered]@{status='running';profile=$BuildProfile;started_utc=[DateTime]::UtcNow.ToString('o');main_sha256=$manifest.normal_main.sha256;active_scenario=$null;cases=@();whole_goal_complete=$false}
function Save {$batch|ConvertTo-Json -Depth 14|Set-Content -LiteralPath $receiptPath -Encoding utf8}
function VerifySources {foreach($name in $expected.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $expected[$name]){throw ('Source drift: '+$name)}}}
Save
try {
 foreach($scenario in $Scenarios){
  VerifySources
  $batch.active_scenario=$scenario;Save
  $before=@(Get-ChildItem -LiteralPath $caseRoot -Directory|Select-Object -ExpandProperty Name)
  $log=Join-Path $stage ($BuildProfile+'-'+$ReceiptLabel+'-'+$scenario+'.log')
  & (Join-Path $PSScriptRoot 'run-isolated-native-teleport-window-test.ps1') -BuildProfile $BuildProfile -Scenario $scenario -ReplayStaleCommands:($scenario -eq 'cancel-retry') *> $log
  $matches=@(Get-ChildItem -LiteralPath $caseRoot -Directory|Where-Object {$_.Name -notin $before}|ForEach-Object {
   $p=Join-Path $_.FullName 'receipt.json'
   if(Test-Path -LiteralPath $p){$d=Get-Content -LiteralPath $p -Raw|ConvertFrom-Json;if($d.exe_sha256 -eq $manifest.normal_main.sha256 -and $d.scenario -eq $scenario){$_.FullName}}
  })
  if($matches.Count -ne 1){throw 'Cannot uniquely bind profile/scenario case'}
  $case=$matches[0]
  foreach($validator in @('verify-native-teleport-recovery-gui.py','verify-teleport-glyph-pixels.py','verify-current-hud-teleport-scenes.py')){
   & python -X utf8 (Join-Path $PSScriptRoot $validator) $case >> $log
   if($LASTEXITCODE -ne 0){throw ('Independent comparison failed: '+$validator+' '+$scenario)}
  }
  if($scenario -eq 'cancel-retry'){
   & python -X utf8 (Join-Path $PSScriptRoot 'verify-native-teleport-stale-commands.py') $case >> $log
   if($LASTEXITCODE -ne 0){throw 'Stale GUI notification comparison failed'}
  }
  & python -X utf8 (Join-Path $PSScriptRoot 'qualify-native-teleport-wire-real-case.py') $case --profile $BuildProfile >> $log
  if($LASTEXITCODE -ne 0){throw 'Default source/build binding or exact movement comparison failed'}
  VerifySources
  $q=Get-Content -LiteralPath (Join-Path $case 'wire-fix-default-source-qualification.json') -Raw|ConvertFrom-Json
  $batch.cases+=@($q);Save
 }
 $batch.status='verified_current_default_wire_fix_GUI_recovery_scenarios';$batch.active_scenario=$null;$batch.completed_utc=[DateTime]::UtcNow.ToString('o');Save
}catch{$batch.status='failed';$batch.error=$_.Exception.Message;$batch.error_stack=$_.ScriptStackTrace;Save;throw}
