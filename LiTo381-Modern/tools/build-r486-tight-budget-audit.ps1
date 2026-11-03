$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r486b-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($rows|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Final production library incomplete'}
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($p.Count -ne 1){throw "Exact $name artifact required"};$p[0]}
$player=Get-Content -LiteralPath (Join-Path $logs 'r486-player-candidate.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $player.production_library.path).Hash -ne $player.production_library.sha256){throw 'Fixed production library drift'}
$root=Join-Path $project 'target\r486-tight-budget-audit'
if(Test-Path -LiteralPath $root){throw 'Consumer already frozen'}
[IO.Directory]::CreateDirectory((Join-Path $root 'source'))|Out-Null
$source=Join-Path $PSScriptRoot 'r486-tight-budget-audit.rs'
$exe=Join-Path $root 'r486-tight-budget-audit.exe'
$args=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','r486_tight_budget_audit',$source,'--extern',"lito_modern=$($player.production_library.path)",'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\i686-pc-windows-msvc\debug\deps')),'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\debug\deps')))
foreach($name in @('anyhow','serde_json','sha2','png')){$args+=@('--extern',"$name=$(Artifact $name)")}
foreach($p in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$args+=@('-L',$p)}
$args+=@('-o',$exe)
[IO.File]::Copy($source,(Join-Path $root 'source\r486-tight-budget-audit.rs'),$false)
& rustc @args 1> (Join-Path $logs 'r486-tight-budget-build.stdout.log') 2> (Join-Path $logs 'r486-tight-budget-build.stderr.log')
if($LASTEXITCODE -ne 0){throw 'Consumer compile failed; source/build logs retained'}
& $exe $project 1> (Join-Path $logs 'r486-tight-budget.stdout.log') 2> (Join-Path $logs 'r486-tight-budget.stderr.log')
$exit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r486-tight-budget-candidate.json') -Value ([ordered]@{exit_code=$exit;executions=1;executable=Pin $exe;source=Pin $source;snapshot=Pin (Join-Path $root 'source\r486-tight-budget-audit.rs');production_library=$player.production_library;stdout=Pin (Join-Path $logs 'r486-tight-budget.stdout.log');stderr=Pin (Join-Path $logs 'r486-tight-budget.stderr.log')})
if($exit -ne 0){throw 'Focused budget audit failed; exact evidence retained'}
Get-Content -LiteralPath (Join-Path $logs 'r486-tight-budget.stdout.log')
