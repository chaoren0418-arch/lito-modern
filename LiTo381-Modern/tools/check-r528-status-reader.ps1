$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
$root=Join-Path $project 'target/r528-status-reader-check'
if(Test-Path -LiteralPath $root){throw 'One reader check build only; preserve existing attempt'}
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r528-status-io-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch;$env:R528_READER_CHECK_ROOT=$scratch
[IO.Directory]::CreateDirectory($root)|Out-Null
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r527-library-build.jsonl') -Encoding UTF8|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib' -and $_ -like '*i686-pc-windows-msvc*'});if($p.Count -ne 1){throw "Exact target $name rlib required"};$p[0]}
$source=Join-Path $root 'r528-status-reader.rs';[IO.File]::Copy((Join-Path $PSScriptRoot 'r528-status-reader.rs'),$source,$false)
$test=Join-Path $root 'r528-status-reader-new-io-tests.exe'
$arguments=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','r528_status_reader_check',$source,'--test','-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps')),'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/debug/deps')))
foreach($name in @('anyhow','serde_json')){$arguments+=@('--extern',"$name=$(Artifact $name)")}
& rustc @arguments -o $test 1>(Join-Path $logs 'r528-status-reader-check-build.stdout.log') 2>(Join-Path $logs 'r528-status-reader-check-build.stderr.log')
$buildExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r528-status-reader-check-build-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$buildExit;source=Pin $source;consumer_recompiled=$false;production_library_rebuilt=$false;network=0;JavaSQL=0})
if($buildExit -ne 0){throw 'Reader-only test build failed; first attempt preserved'}
& $test 'r528_new_status_io' --nocapture --test-threads=1 1>(Join-Path $logs 'r528-status-reader-check.stdout.log') 2>(Join-Path $logs 'r528-status-reader-check.stderr.log')
$testExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r528-status-reader-check-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$testExit;binary=Pin $test;source=Pin $source;own_io_root=$scratch;network=0;credentials_reads=0;PIDqueries=0;JavaSQL=0;server_writes=0;old_tests_replayed=0;old_consumer_recompiled=$false;fixture_files_retained=$true})
if($testExit -ne 0){throw 'New reader IO check failed; no automatic test replay'}
