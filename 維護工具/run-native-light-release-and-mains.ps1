#Requires -Version 7.0
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot
$project=Join-Path $taskRoot 'LiTo381-Modern'
$stage=Join-Path $taskRoot '驗證紀錄/current/native-light-spell'
$tools='C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin'
$env:RUSTC=Join-Path $tools 'rustc.exe';$env:RUSTDOC=Join-Path $tools 'rustdoc.exe'
$env:PATH=$tools+';'+$env:PATH
$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP
$env:PYTHONDONTWRITEBYTECODE='1';$env:PYTHONIOENCODING='utf-8'
$pins=Get-Content -LiteralPath (Join-Path $stage 'repaired-source-inputs.json') -Raw|ConvertFrom-Json -AsHashtable
$state=@{status='running';phase='Release-scoped';source_files=$pins.Count;source_manifest_sha256=(Get-FileHash -LiteralPath (Join-Path $stage 'repaired-source-inputs.json')).Hash;normal_mains=@{};whole_goal_complete=$false}
function Save {$state|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $stage 'release-and-mains.json') -Encoding utf8}
function Stable {foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw ('Compilation input drift: '+$name)}}}
Push-Location $project
try {
 Stable
 $debug=Get-Content -LiteralPath (Join-Path $stage 'Debug-scoped-qualified.json') -Raw|ConvertFrom-Json
 if($debug.passed -ne 42 -or $debug.failed -ne 0 -or $debug.source_manifest_sha256 -ne $state.source_manifest_sha256){throw 'Current Debug qualification required'}
 $busy=@(Get-CimInstance Win32_Process|Where-Object { $_.Name -match '^(cargo|rustc|native_.*|LiToModern.*)\.exe$' -and $_.CommandLine -and $_.CommandLine.Replace('/','\').Contains($project) })
 if($busy.Count){throw 'Wait for owned compilation/tests/UI to finish'}
 Save
 & (Join-Path $tools 'cargo.exe') test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --release --message-format=json --test native_light_spell --test native_skills -- --test-threads=1 *> (Join-Path $stage 'Release-scoped.jsonl')
 if($LASTEXITCODE -ne 0){throw 'Release scoped tests failed'}
 Stable
 & python (Join-Path $PSScriptRoot 'qualify-native-light-scoped.py') --profile Release *> (Join-Path $stage 'Release-qualification-console.txt')
 if($LASTEXITCODE -ne 0){throw 'Release independent qualification failed'}
 foreach($profile in @('Debug','Release')) {
  $state.phase=$profile+'-unique-entry-build';Save;Stable
  & (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $profile *> (Join-Path $stage ($profile+'-entry-console.txt'))
  if($LASTEXITCODE -ne 0){throw ('Unique entry failed: '+$profile)}
  $log=Join-Path $taskRoot $(if($profile -eq 'Release'){'驗證紀錄/current/build-release.log'}else{'驗證紀錄/current/build.log'})
  Copy-Item -LiteralPath $log -Destination (Join-Path $stage ($profile+'-entry-build.log'))
  $exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$profile.ToLowerInvariant()+'/LiToModern.exe')
  $state.normal_mains[$profile]=@{path=$exe;sha256=(Get-FileHash -LiteralPath $exe).Hash;entry_build_log_sha256=(Get-FileHash -LiteralPath (Join-Path $stage ($profile+'-entry-build.log'))).Hash}
  Stable;Save
 }
 $state.phase=$null;$state.status='completed_pending_normal_archive_qualification';Save
} catch {$state.status='failed';$state.error=$_.Exception.Message;Save;throw} finally {Pop-Location}
