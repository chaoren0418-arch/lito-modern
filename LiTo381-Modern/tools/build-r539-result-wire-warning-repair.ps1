$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
Set-Location -LiteralPath $project
if(Test-Path -LiteralPath (Join-Path $logs 'r539-final-test-compile.jsonl')){throw 'Preserve existing final compile'}
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r539-test-name-repair-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
& cargo test --offline --locked --lib --no-run --target i686-pc-windows-msvc --target-dir target/r409-dev-ready --message-format=json 1>(Join-Path $logs 'r539-final-test-compile.jsonl') 2>(Join-Path $logs 'r539-final-test-compile.stderr.log')
$code=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r539-final-test-compile-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$code;no_run=$true;tests_executed=0;test_name_only_change=$true;regular_library_rebuilt=$false;old_PASS_replays=0;scratch_retained=$scratch;network=0;GUI=0;JavaSQL=0})
if($code -ne 0){throw 'Final test compile failed; preserve outcome'}
Write-Output 'R539 test-name-only warning repair compiled --no-run; previous PASS not replayed.'
