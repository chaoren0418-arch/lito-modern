$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r512-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($rows|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Production library incomplete'}
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($p.Count -ne 1){throw "Exact $name rlib required"};$p[0]}
$root=Join-Path $project 'target/r512-fixed-consumer'
if(Test-Path -LiteralPath $root){throw 'Already frozen'}
[IO.Directory]::CreateDirectory($root)|Out-Null
$lib=Join-Path $root 'liblito_modern.rlib'
[IO.File]::Copy((Artifact 'lito_modern'),$lib,$false)
$source=Join-Path $PSScriptRoot 'r512-return-consumer.rs';$fixed=Join-Path $root 'r512-return-consumer.rs'
[IO.File]::Copy($source,$fixed,$false)
$exe=Join-Path $root 'r512-return-consumer.exe'
$args=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','r508_bootstrap_consumer',$fixed,'--extern',"lito_modern=$lib",'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps')),'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/debug/deps')))
foreach($name in @('anyhow','serde_json','sha2','png')){$args+=@('--extern',"$name=$(Artifact $name)")}
foreach($p in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$args+=@('-L',$p)}
$args+=@('-o',$exe)
& rustc @args 1>(Join-Path $logs 'r512-consumer-build.stdout.log') 2>(Join-Path $logs 'r512-consumer-build.stderr.log')
if($LASTEXITCODE -ne 0){throw 'New consumer compile failed; logs retained'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r512-consumer-candidate.json') -Value ([ordered]@{executable=Pin $exe;production_library=Pin $lib;source=Pin $fixed;production_snapshot=Pin (Join-Path $logs 'r512-production-source-snapshot.json');GUI=0;historical_PASS_reruns=0})
