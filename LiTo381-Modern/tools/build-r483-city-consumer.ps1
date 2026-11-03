$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r483b-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($rows|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Final production library incomplete'}
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($p.Count -ne 1){throw "Exact $name artifact required"};$p[0]}
$player=Get-Content -LiteralPath (Join-Path $logs 'r483-player-candidate.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $player.production_library.path).Hash -ne $player.production_library.sha256){throw 'Fixed production library drift'}
$root=Join-Path $project 'target\r483c-fixed-city-consumer'
if(Test-Path -LiteralPath $root){throw 'Consumer already frozen'}
[IO.Directory]::CreateDirectory((Join-Path $root 'source'))|Out-Null
$source=Join-Path $PSScriptRoot 'r483-city-consumer.rs'
$exe=Join-Path $root 'r483-city-consumer.exe'
$args=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','r483_city_consumer',$source,'--extern',"lito_modern=$($player.production_library.path)",'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\i686-pc-windows-msvc\debug\deps')),'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\debug\deps')))
foreach($name in @('anyhow','serde_json','sha2','png')){$args+=@('--extern',"$name=$(Artifact $name)")}
foreach($p in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$args+=@('-L',$p)}
$args+=@('-o',$exe)
& rustc @args 1> (Join-Path $logs 'r483c-consumer-build.stdout.log') 2> (Join-Path $logs 'r483c-consumer-build.stderr.log')
if($LASTEXITCODE -ne 0){throw 'Consumer compile failed; source/build logs retained'}
$old=Get-Content -LiteralPath (Join-Path $logs 'r482-candidate.json') -Raw|ConvertFrom-Json
foreach($s in $old.source_pins){if((Get-FileHash -LiteralPath $s.snapshot).Hash -ne $s.sha256){throw 'R482 source snapshot drift'}}
$paths=@($old.source_pins.path)+@(foreach($n in @('src\modern_lito\native_city_resources.rs','src\modern_lito\native_city_flow.rs','src\modern_lito\native_city_runtime.rs','src\modern_lito\native_city_window.rs','target\r483-player-entry-candidate\LiToModern-R483.rs','tools\r483-city-consumer.rs')){Join-Path $project $n})
$pins=@(foreach($p in $paths|Select-Object -Unique){$dst=Join-Path $root ('source\'+(Split-Path $p -Leaf));[IO.File]::Copy($p,$dst,$false);$pin=Pin $p;if((Get-FileHash -LiteralPath $dst).Hash -ne $pin.sha256){throw 'Source copy drift'};$pin.snapshot=$dst;$pin})
if($pins.Count -ne 155){throw 'Expected155 fixed source pairs'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r483c-consumer-candidate.json') -Value ([ordered]@{path=$exe;bytes=(Get-Item -LiteralPath $exe).Length;sha256=(Get-FileHash -LiteralPath $exe).Hash;source_pins=$pins;production_library=$player.production_library;validation='PENDING one finite new city API/worker consumer';GUI=0;LOADED=$false})
& $exe $project 1> (Join-Path $logs 'r483c-consumer.stdout.log') 2> (Join-Path $logs 'r483c-consumer.stderr.log')
$exit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r483c-consumer-exit.json') -Value ([ordered]@{exit_code=$exit;stdout=Pin (Join-Path $logs 'r483c-consumer.stdout.log');stderr=Pin (Join-Path $logs 'r483c-consumer.stderr.log');executions=1;historical_PASS_reruns=0})
if($exit -ne 0){throw 'City consumer failed; exact source/exe/logs/partial evidence retained'}
foreach($s in $pins){if((Get-FileHash -LiteralPath $s.path).Hash -ne $s.sha256 -or (Get-FileHash -LiteralPath $s.snapshot).Hash -ne $s.sha256){throw 'Source drift during execution'}}
Get-Content -LiteralPath (Join-Path $logs 'r483c-consumer.stdout.log') -Tail 3
