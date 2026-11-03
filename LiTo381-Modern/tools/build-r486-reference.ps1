$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot;$logs=Join-Path $project 'logs\continuation-20261003'
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r485a-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($rows|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'R485 library incomplete'}
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($p.Count -ne 1){throw 'Exact artifact required'};$p[0]}
$fixed=Get-Content -LiteralPath (Join-Path $logs 'r485-player-candidate.json') -Raw|ConvertFrom-Json;$lib=$fixed.production_library.path;if((Get-FileHash -LiteralPath $lib).Hash -ne $fixed.production_library.sha256){throw 'Fixed R485 library drift'}
$root=Join-Path $project 'target\r486-paged-city';$exe=Join-Path $root 'r486-map563-reference.exe';$source=Join-Path $PSScriptRoot 'r486-map563-reference.rs'
$args=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','r486_map563_reference',$source,'--extern',"lito_modern=$lib",'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\i686-pc-windows-msvc\debug\deps')),'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\debug\deps')))
foreach($name in @('anyhow','serde_json','sha2','png')){$args+=@('--extern',"$name=$(Artifact $name)")};foreach($p in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$args+=@('-L',$p)};$args+=@('-o',$exe)
& rustc @args 1> (Join-Path $logs 'r486-reference-build.stdout.log') 2> (Join-Path $logs 'r486-reference-build.stderr.log');if($LASTEXITCODE -ne 0){throw 'New reference compile failed; source/logs retained'}
$input=Join-Path $logs 'r486-resource-input.json';$output=Join-Path $root 'reference\reference.json'
& $exe $input $output 1> (Join-Path $logs 'r486-reference.stdout.log') 2> (Join-Path $logs 'r486-reference.stderr.log');$exit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r486-reference-candidate.json') -Value ([ordered]@{executable=Pin $exe;source=Pin $source;fixed_production_library=Pin $lib;input=Pin $input;output=Pin $output;stdout=Pin (Join-Path $logs 'r486-reference.stdout.log');stderr=Pin (Join-Path $logs 'r486-reference.stderr.log');compile_attempts=1;executions=1;exit_code=$exit;GUI=0;LOADED=$false})
if($exit -ne 0){throw 'Reference execution failed; exact outputs retained'}
Get-Content -LiteralPath (Join-Path $logs 'r486-reference.stdout.log')
