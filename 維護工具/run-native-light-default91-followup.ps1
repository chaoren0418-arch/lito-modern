#Requires -Version 7.0
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot;$stage=Join-Path $taskRoot '驗證紀錄/current/native-light-spell/current-regression';$normal=Join-Path $taskRoot '驗證紀錄/current/native-light-spell/normal/qualified.json'
$env:PYTHONDONTWRITEBYTECODE='1';$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$state=@{status='running';profiles=@{};normal_qualification=$normal;whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $stage 'default91-followup.json') -Encoding utf8}
Save
try {
 foreach($profile in @('Debug','Release')){
  $state.active_profile=$profile;Save
  $log=Join-Path $stage ($profile+'-default91-console.txt')
  & python -X utf8 (Join-Path $PSScriptRoot 'run-native-light-default-regression.py') --profile $profile --normal-qualification $normal *> $log
  if($LASTEXITCODE -ne 0){throw ('Current default91 failed: '+$profile+'; '+$log)}
  $json=Get-Content -LiteralPath $log|Where-Object {$_.StartsWith('{')}|Select-Object -Last 1
  if(!$json){throw 'Missing terminal default91 run path'}
  $result=$json|ConvertFrom-Json;$run=$result.output
  $output=Join-Path $stage ($profile+'-default91-qualified.json')
  & python -X utf8 (Join-Path $PSScriptRoot 'qualify-native-light-default-regression.py') $run --output $output --normal-qualification $normal >> $log
  if($LASTEXITCODE -ne 0){throw ('Independent current default91 qualification failed: '+$profile)}
  $state.profiles[$profile]=@{run=$run;qualification=$output;qualification_sha256=(Get-FileHash -LiteralPath $output).Hash};Save
 }
 $state.status='both_profiles_default91_qualified';$state.active_profile=$null;Save
 Write-Output ($state|ConvertTo-Json -Depth 8)
}catch{$state.status='failed';$state.error=$_.Exception.Message;Save;throw}
