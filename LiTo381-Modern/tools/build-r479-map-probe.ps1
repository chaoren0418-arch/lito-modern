$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot;$logs=Join-Path $project 'logs\continuation-20261003'
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$rows=@(Get-Content (Join-Path $logs 'r478a-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($p.Count -ne 1){throw 'Exact artifact required'};$p[0]}
$old=Get-Content (Join-Path $logs 'r478-player-candidate.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $old.production_library.path).Hash -ne $old.production_library.sha256){throw 'R478 fixed library drift'}
$exe=Join-Path $project 'target\r479-map-batch\r479-native-batch-probe.exe';$source=Join-Path $PSScriptRoot 'r479-native-batch-probe.rs'
$args=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','r479_native_batch_probe',$source,'--extern',"lito_modern=$($old.production_library.path)",'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\i686-pc-windows-msvc\debug\deps')),'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\debug\deps')))
foreach($name in @('anyhow','serde_json','sha2','png')){$args+=@('--extern',"$name=$(Artifact $name)")};foreach($p in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$args+=@('-L',$p)};$args+=@('-o',$exe)
& rustc @args 1> (Join-Path $logs 'r479-probe-build.stdout.log') 2> (Join-Path $logs 'r479-probe-build.stderr.log');if($LASTEXITCODE -ne 0){throw 'Inventory/probe compile failed'}
$input=Join-Path $logs 'r479-resource-inventory-input.json';$inventory=Join-Path $logs 'r479-native38-inventory.json';$output=Join-Path $project 'target\r479-map-batch\probe\batch-results.json'
& $exe $input $inventory $output 1> (Join-Path $logs 'r479-probe.stdout.log') 2> (Join-Path $logs 'r479-probe.stderr.log');if($LASTEXITCODE -ne 0){throw 'Inventory/probe execution failed'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r479-probe-candidate.json') -Value ([ordered]@{executable=Pin $exe;source=Pin $source;fixed_R478_library=$old.production_library;input=Pin $input;inventory=Pin $inventory;output=Pin $output;stdout=Pin (Join-Path $logs 'r479-probe.stdout.log');stderr=Pin (Join-Path $logs 'r479-probe.stderr.log');build_stdout=Pin (Join-Path $logs 'r479-probe-build.stdout.log');build_stderr=Pin (Join-Path $logs 'r479-probe-build.stderr.log');compile_attempts=1;executions=1;GUI=0;LOADED=$false})
Get-Content (Join-Path $logs 'r479-probe.stdout.log') -Tail 1
