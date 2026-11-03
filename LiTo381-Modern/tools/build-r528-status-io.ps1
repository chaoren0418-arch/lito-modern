$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
$root=Join-Path $project 'target/r528-status-io-backend-consumer'
if(Test-Path -LiteralPath $root){throw 'Own new target exists; preserve initial compile/results'}
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r528-status-integration-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch;$env:R528_STATUS_INTEGRATION_ROOT=$scratch
[IO.Directory]::CreateDirectory($root)|Out-Null
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$previous=Get-Content -LiteralPath (Join-Path $logs 'r527-consumer-candidate.json') -Raw -Encoding UTF8|ConvertFrom-Json
$lib=$previous.production_library.path
if((Get-FileHash -LiteralPath $lib).Hash -ne $previous.production_library.sha256){throw 'Frozen regular library changed'}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r527-library-build.jsonl') -Encoding UTF8|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib' -and $_ -like '*i686-pc-windows-msvc*'});if($p.Count -ne 1){throw "Exact target $name rlib required"};$p[0]}
foreach($name in @('r528-status-io-consumer.rs','r528-status-io-countdown-case.rs','r528-status-io-provider.rs','r528-status-io-reader.rs','r528-status-io-interface-tests.rs')){[IO.File]::Copy((Join-Path $PSScriptRoot $name),(Join-Path $root $name),$false)}
foreach($name in @('accepted-status-io-handoff.json','accepted-status-io-launch.json','accepted-status-io-manifest.json')){[IO.File]::Copy((Join-Path $logs "r528-status-io-accepted/$name"),(Join-Path $root $name),$false)}
$source=Join-Path $root 'r528-status-io-consumer.rs'
$exe=Join-Path $root 'r528-status-io-consumer.exe'
$arguments=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','r528_status_io_consumer',$source,'--extern',"lito_modern=$lib",'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps')),'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/debug/deps')))
foreach($name in @('anyhow','serde_json','sha2','zeroize','serde','windows')){$arguments+=@('--extern',"$name=$(Artifact $name)")}
foreach($p in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$arguments+=@('-L',$p)}
& rustc @arguments -o $exe 1>(Join-Path $logs 'r528-status-io-build.stdout.log') 2>(Join-Path $logs 'r528-status-io-build.stderr.log')
$buildExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r528-status-io-build-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$buildExit;production_library_rebuilt=$false;old_consumer_rebuilt=$false;scratch=$scratch;source=Pin $source})
if($buildExit -ne 0){throw 'New wrapper compile failed; initial result retained'}
$test=Join-Path $root 'r528-status-io-interface-tests.exe'
& rustc @arguments --test -o $test 1>(Join-Path $logs 'r528-status-io-tests-build.stdout.log') 2>(Join-Path $logs 'r528-status-io-tests-build.stderr.log')
$testBuildExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r528-status-io-tests-build-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$testBuildExit;production_library_rebuilt=$false;source=Pin $source})
if($testBuildExit -ne 0){throw 'New interface harness compile failed; initial result retained'}
$filter='r528_status_io_integration'
& $test $filter --nocapture 1>(Join-Path $logs 'r528-status-io-tests.stdout.log') 2>(Join-Path $logs 'r528-status-io-tests.stderr.log')
$testExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r528-status-io-tests-accounting.json') -Value ([ordered]@{attempts=1;filter=$filter;exit_code=$testExit;binary=Pin $test;own_fixture_root=$scratch;network=0;JavaSQL=0;credentials_reads=0;PIDqueries=0;server_writes=0;old_PASS_replays=0;synthetic_Runtime_replay=$false})
if($testExit -ne 0){throw 'New interface tests failed; no automatic replay'}
& $exe --check-prepared (Join-Path $logs 'r528-status-io-provider-supply-request.json') 1>(Join-Path $logs 'r528-status-io-prepared.stdout.log') 2>(Join-Path $logs 'r528-status-io-prepared.stderr.log')
$checkExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r528-status-io-prepared-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$checkExit;binary=Pin $exe;network=0;credentials_reads=0;PIDqueries=0;JavaSQL=0;GUI=0})
if($checkExit -ne 0){throw 'New preparation check failed; initial result retained'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r528-status-io-candidate.json') -Value ([ordered]@{status='COLD_STATUS_IO_CONSUMER_READY_FRESH_PROVIDER_NOT_AUTHORIZED';executable=Pin $exe;source=Pin $source;countdown_module=Pin (Join-Path $root 'r528-status-io-countdown-case.rs');provider_module=Pin (Join-Path $root 'r528-status-io-provider.rs');status_reader=Pin (Join-Path $root 'r528-status-io-reader.rs');interface_tests_source=Pin (Join-Path $root 'r528-status-io-interface-tests.rs');production_library=Pin $lib;tests=Pin $test;scratch=$scratch;production_source_changes=0;actual_backend_runs=0;GUI=0;old_PASS_replays=0})
