$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
$root=Join-Path $project 'target/r528-final-backend-consumer'
if(Test-Path -LiteralPath $root){throw 'Final cold consumer build target exists; preserve first attempt and do not overwrite'}
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r528-final-build-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
[IO.Directory]::CreateDirectory($root)|Out-Null
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$previous=Get-Content -LiteralPath (Join-Path $logs 'r527-consumer-candidate.json') -Raw -Encoding UTF8|ConvertFrom-Json
$lib=$previous.production_library.path
if((Get-FileHash -LiteralPath $lib).Hash -ne $previous.production_library.sha256){throw 'Frozen R527 regular library changed'}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r527-library-build.jsonl') -Encoding UTF8|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib' -and $_ -like '*i686-pc-windows-msvc*'});if($p.Count -ne 1){throw "Exact target $name rlib required"};$p[0]}
foreach($name in @('r528-backend-consumer.rs','r528-final-countdown-case.rs','r528-provider.rs')){[IO.File]::Copy((Join-Path $PSScriptRoot $name),(Join-Path $root $name),$false)}
$source=Join-Path $root 'r528-backend-consumer.rs'
$exe=Join-Path $root 'r528-backend-consumer.exe'
$arguments=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','r528_countdown_consumer',$source,'--extern',"lito_modern=$lib",'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps')),'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/debug/deps')))
foreach($name in @('anyhow','serde_json','sha2','zeroize','serde','windows')){$arguments+=@('--extern',"$name=$(Artifact $name)")}
foreach($p in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$arguments+=@('-L',$p)}
& rustc @arguments -o $exe 1>(Join-Path $logs 'r528-final-build.stdout.log') 2>(Join-Path $logs 'r528-final-build.stderr.log')
if($LASTEXITCODE -ne 0){throw 'Final cold consumer compile failed; first attempt retained'}
$test=Join-Path $root 'r528-countdown-new-boundaries.exe'
& rustc @arguments --test -o $test 1>(Join-Path $logs 'r528-final-interface-boundaries-build.stdout.log') 2>(Join-Path $logs 'r528-final-interface-boundaries-build.stderr.log')
if($LASTEXITCODE -ne 0){throw 'New consumer test compile failed; first attempt retained'}
$filter='r528_final_provider_boundaries'
& $test $filter --nocapture 1>(Join-Path $logs 'r528-final-interface-boundaries.stdout.log') 2>(Join-Path $logs 'r528-final-interface-boundaries.stderr.log')
$testExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r528-final-interface-boundaries-accounting.json') -Value ([ordered]@{attempts=1;filter=$filter;exit_code=$testExit;binary=Pin $test;network_cases=0;old_tests_executed=0;production_library_rebuilt=$false;synthetic_Runtime_replay=$false})
if($testExit -ne 0){throw 'New targeted boundary test failed; no automatic test replay'}
& $exe --check-prepared (Join-Path $logs 'r528-final-provider-supply-request.json') 1>(Join-Path $logs 'r528-final-preparation-check.stdout.log') 2>(Join-Path $logs 'r528-final-preparation-check.stderr.log')
$checkExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r528-final-preparation-check-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$checkExit;binary=Pin $exe;network_cases=0;credentials_reads=0;PIDqueries=0;JavaSQL=0;GUI=0})
if($checkExit -ne 0){throw 'Final cold consumer preparation check failed; first execution retained'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r528-final-candidate.json') -Value ([ordered]@{status='COLD_CONSUMER_READY_FRESH_PROVIDER_EXECUTION_NOT_AUTHORIZED';executable=Pin $exe;source=Pin $source;countdown_module=Pin (Join-Path $root 'r528-final-countdown-case.rs');production_library=Pin $lib;tests=Pin $test;scratch=$scratch;final_provider_interface_accepted=$true;actual_backend_runs=0;provider_module=Pin (Join-Path $root 'r528-provider.rs');GUI=0;old_PASS_replays=0})
