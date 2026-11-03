#Requires -Version 7.0
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$OutputEncoding=[Text.UTF8Encoding]::new($false)
$taskRoot=Split-Path -Parent $PSScriptRoot
$run=Join-Path $taskRoot '驗證紀錄/current/native-game-timers/20261016-004010-201637'
$stage=Join-Path $run 'current-regression'
$normalPath=Join-Path $run 'normal/qualified.json'
$env:PYTHONDONTWRITEBYTECODE='1';$env:PYTHONIOENCODING='utf-8'
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$tools='C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin'
$env:RUSTC=Join-Path $tools 'rustc.exe';$env:RUSTDOC=Join-Path $tools 'rustdoc.exe';$env:PATH=$tools+';'+$env:PATH
$normal=Get-Content -LiteralPath $normalPath -Raw|ConvertFrom-Json
$state=@{status='initializing';phase='checking_completed_natural_cases';normal_qualification=$normalPath;source_files=976;source_manifest_sha256=$normal.source_manifest_sha256;pipeline_pid=$PID;pipeline_creation_utc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o');whole_goal_complete=$false;source_and_GUI_QA_must_remain_frozen=$true;profiles=@()}
function Save {$state|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $stage 'current-followup.json') -Encoding utf8}
Save
try {
    foreach($profile in @('Debug','Release')){
        $path=Join-Path $run ('natural/'+$profile+'/independent-qualified.json')
        $q=Get-Content -LiteralPath $path -Raw|ConvertFrom-Json
        if($q.profile -ne $profile -or $q.source_manifest_sha256 -ne $normal.source_manifest_sha256 -or $q.actual_formal_sessions -ne 4 -or $q.C_ALIVE -ne 8 -or $q.C_SAVE -ne 1 -or $q.S_TIME -ne 8 -or !$q.archive_readback_verified -or !$q.original_interval_backpressure_no_UI_poll_test_passed){throw ('Exact completed natural interval evidence required: '+$profile)}
        $main=$normal.profiles.$profile
        if((Get-FileHash -LiteralPath $main.path).Hash -ne $main.sha256){throw ('Normal main changed: '+$profile)}
        $state.profiles+=$profile
    }
    $state.phase='default93_both_profiles';$state.status='running';Save
    & (Join-Path $PSScriptRoot 'run-native-game-timer-default93-followup.ps1') *> (Join-Path $stage 'default93-pipeline-console.txt')
    if($LASTEXITCODE -ne 0){throw 'Current default93 followup failed'}
    foreach($profile in @('Debug','Release')){
        & python -X utf8 (Join-Path $PSScriptRoot 'seal-native-game-timer-default-run.py') --profile $profile *> (Join-Path $stage ($profile+'-default93-seal-console.txt'))
        if($LASTEXITCODE -ne 0){throw ('Current source/product/raw sealing failed: '+$profile)}
    }
    $state.phase='TW14_both_profiles';Save
    & (Join-Path $PSScriptRoot 'run-native-game-timer-TW14-followup.ps1') *> (Join-Path $stage 'TW14-pipeline-console.txt')
    if($LASTEXITCODE -ne 0){throw 'Current TW14 followup failed'}
    $state.status='completed_current_default93_and_TW14_defined_scopes';$state.phase=$null;$state.source_and_GUI_QA_must_remain_frozen=$false;Save
    $state|ConvertTo-Json -Depth 8
} catch {$state.status='failed';$state.error=$_.Exception.Message;Save;throw}
