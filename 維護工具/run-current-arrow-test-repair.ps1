#Requires -Version 7.0
$ErrorActionPreference='Stop'
$taskRoot='D:\lineage開發\3-lito開發\lito重建版'
$stage=Join-Path $taskRoot '驗證紀錄/current/native-resource-optin/arrow-frame-comparison-repair'
$installed='C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin'
$env:RUSTC=Join-Path $installed 'rustc.exe';$env:RUSTDOC=Join-Path $installed 'rustdoc.exe';$env:PATH=$installed+';'+$env:PATH
$env:TEMP=Join-Path $taskRoot 'LiTo381-Modern/target/current/scratch';$env:TMP=$env:TEMP
$memory=Get-CimInstance Win32_OperatingSystem
if($memory.FreeVirtualMemory -lt 8*1024*1024){throw 'Meaningful commit availability required before measured lib compilation'}
@{status='admitted_single_cfg_test_repair';available_commit_kib=$memory.FreeVirtualMemory;required_commit_kib=8*1024*1024;utc=[DateTime]::UtcNow.ToString('o');external_processes_stopped=$false}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $stage 'admission.json') -Encoding utf8
foreach($profile in @('Debug','Release')){
 & (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $profile *> (Join-Path $stage ($profile+'-unique-entry.log'))
 if($LASTEXITCODE -ne 0){throw 'Unique entry build failed'}
}
$output=& (Join-Path $taskRoot '維護工具/measure-native-library-build.ps1') -SourceManifest '驗證紀錄/current/native-resource-optin/arrow-frame-comparison-repair/source-inputs.json' -ExpectedSourceFiles 968
$output|Set-Content -LiteralPath (Join-Path $stage 'compile-result.json') -Encoding utf8
if($LASTEXITCODE -ne 0){throw 'Measured library build failed'}
$run=($output|ConvertFrom-Json).output
foreach($tool in @('qualify-native-library-build.py','inventory-pinned-library-tests.py','audit-ignored-library-sources.py')){
 & python -X utf8 (Join-Path $taskRoot ('維護工具/'+$tool)) $run
 if($LASTEXITCODE -ne 0){throw ('Build/inventory qualification failed: '+$tool)}
}
& (Join-Path $taskRoot '維護工具/run-pinned-library-tests.ps1') -BuildRun $run -RunLabel 'full-current' -PetFixtures
if($LASTEXITCODE -ne 0){throw 'Full current library default runtime failed'}
foreach($tool in @('qualify-current-full-library-default.py','qualify-current-library-wire-faults.py','run-current-resource-optin.py')){
 & python -X utf8 (Join-Path $taskRoot ('維護工具/'+$tool)) $run
 if($LASTEXITCODE -ne 0){throw ('Runtime qualification failed: '+$tool)}
}
@{status='completed_cfg_test_repair_and_full_default_plus_five_optin';build_run=$run;whole_goal_complete=$false}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $stage 'pipeline-completed.json') -Encoding utf8
