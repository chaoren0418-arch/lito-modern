$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$rows=@(Get-Content (Join-Path $logs 'r474a-library-build.jsonl') | ForEach-Object {try{$_|ConvertFrom-Json}catch{}})
if(-not @($rows|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'New library build incomplete'}
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($p.Count -ne 1){throw "Exact $name artifact selection"};$p[0]}
$lib=Join-Path $project 'target\r474c-player-library\liblito_modern.rlib';if(Test-Path $lib){if((Get-FileHash $lib).Hash -ne (Get-FileHash (Artifact 'lito_modern')).Hash){throw 'Fixed library drift'}}else{[IO.File]::Copy((Artifact 'lito_modern'),$lib,$false)}
$exe=Join-Path $project 'target\r474-offline-map-flow\native-offline-map-menu.exe'
$env:R474_MODERN_ROOT=$project
$compileArgs=@('--edition=2021','--target','i686-pc-windows-msvc',(Join-Path $PSScriptRoot 'native-offline-map-menu.rs'),'--extern',"lito_modern=$lib",'--extern',"serde_json=$(Artifact 'serde_json')",'--extern',"sha2=$(Artifact 'sha2')",'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\i686-pc-windows-msvc\debug\deps')),'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\debug\deps')))
foreach($linkPath in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$compileArgs+=@('-L',$linkPath)}
$compileArgs+=@('-o',$exe)
& rustc @compileArgs 1> (Join-Path $logs 'r474h-player-build.stdout.log') 2> (Join-Path $logs 'r474h-player-build.stderr.log')
if($LASTEXITCODE -ne 0){throw 'New player tool compile failed'}
$manifest=Join-Path $logs 'r474-mode-map-binding.json'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r474-player-tool.json') -Value ([ordered]@{executable=Pin $exe;library=Pin $lib;binding=Pin $manifest;source=Pin (Join-Path $PSScriptRoot 'native-offline-map-menu.rs');inspection_only=$true;default_main_rebuilt=$false;original_BIN_executed=$false;GUI=0})
$commands=@('list','select 1 526','select 1 564','enter','pan 240 120','back','enter','back','quit')
$commandPath=Join-Path $project 'target\r474-offline-map-flow\player-commands.txt';[IO.File]::WriteAllLines($commandPath,$commands,[Text.UTF8Encoding]::new($false))
$commands | & $exe $manifest (Join-Path $project 'target\r474-offline-map-flow\player-session') 1> (Join-Path $logs 'r474-player-session.stdout.log') 2> (Join-Path $logs 'r474-player-session.stderr.log')
if($LASTEXITCODE -ne 0){throw 'New player flow run failed'}
Get-Content (Join-Path $logs 'r474-player-session.stdout.log') -Tail 9
