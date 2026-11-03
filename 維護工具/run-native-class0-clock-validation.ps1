#Requires -Version 7.0
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot;$project=Join-Path $taskRoot 'LiTo381-Modern';$stage=Join-Path $taskRoot '驗證紀錄/current/native-effect-native-clock'
$tools='C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin'
$env:RUSTC=Join-Path $tools 'rustc.exe';$env:RUSTDOC=Join-Path $tools 'rustdoc.exe';$env:PATH=$tools+';'+$env:PATH;$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP
$pins=Get-Content -LiteralPath (Join-Path $stage 'final-source-inputs.json') -Raw|ConvertFrom-Json -AsHashtable
$index=@{status='running';source_files=$pins.Count;source_manifest_sha256=(Get-FileHash -LiteralPath (Join-Path $stage 'final-source-inputs.json')).Hash;steps=@();normal_mains=@{};whole_goal_complete=$false}
function Save {$index|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $stage 'validation-pipeline.json') -Encoding utf8}
function Stable {foreach($name in $pins.Keys){if((Get-FileHash -LiteralPath (Join-Path $taskRoot $name)).Hash -ne $pins[$name]){throw ('Compilation input drift: '+$name)}}}
function Tests([string]$Profile,[string[]]$Targets,[string]$Label){
 Stable
 $args=@('test','--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--message-format=json')
 if($Profile -eq 'Release'){$args+='--release'}
 foreach($target in $Targets){$args+=@('--test',$target)}
 $args+=@('--','--test-threads=1')
 $log=Join-Path $stage ($Label+'.jsonl');$start=[DateTime]::UtcNow.ToString('o')
 & (Join-Path $tools 'cargo.exe') @args *> $log
 if($LASTEXITCODE -ne 0){throw ('Scoped test batch failed: '+$Label)}
 Stable;$index.steps+=@{profile=$Profile;targets=$Targets;log=$log;log_sha256=(Get-FileHash -LiteralPath $log).Hash;started_utc=$start;finished_utc=[DateTime]::UtcNow.ToString('o')};Save
}
Push-Location $project
try {
 Save;Stable
 Tests -Profile Debug -Targets @('native_effect_lifecycle') -Label 'Debug-final-effect-tests'
 foreach($profile in @('Debug','Release')){
  Stable
  & (Join-Path $taskRoot '啟動現代客戶端.ps1') -BuildOnly -BuildProfile $profile *> (Join-Path $stage ($profile+'-entry-console.txt'))
  if($LASTEXITCODE -ne 0){throw ('Unique entry build failed: '+$profile)}
  $buildLog=Join-Path $taskRoot $(if($profile -eq 'Release'){'驗證紀錄/current/build-release.log'}else{'驗證紀錄/current/build.log'})
  Copy-Item -LiteralPath $buildLog -Destination (Join-Path $stage ($profile+'-entry-build.log'))
  $exe=Join-Path $project ('target/current/x86_64-pc-windows-msvc/'+$profile.ToLowerInvariant()+'/LiToModern.exe');$index.normal_mains[$profile]=@{path=$exe;sha256=(Get-FileHash -LiteralPath $exe).Hash;entry_build_log_sha256=(Get-FileHash -LiteralPath (Join-Path $stage ($profile+'-entry-build.log'))).Hash};Stable;Save
 }
 Tests -Profile Release -Targets @('native_effect_lifecycle','native_holy_weapon','native_poison_cure','native_travel','native_turn_undead','native_reagent_skills','native_item_effects','native_world_effects') -Label 'Release-final-eight-tests'
 foreach($profile in @('Debug','Release')){if((Get-FileHash -LiteralPath $index.normal_mains[$profile].path).Hash -ne $index.normal_mains[$profile].sha256){throw 'Test build changed normal main; unique-entry restoration required'}}
 Stable;$index.status='completed_pending_independent_qualification';Save
 Write-Output ($index|ConvertTo-Json -Depth 6)
}catch{$index.status='failed';$index.error=$_.Exception.Message;Save;throw}finally{Pop-Location}
