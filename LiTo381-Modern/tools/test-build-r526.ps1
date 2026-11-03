$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r526-library-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
& cargo build --offline --locked --lib --target i686-pc-windows-msvc --target-dir target/r409-dev-ready --message-format=json 1>(Join-Path $logs 'r526-library-build.jsonl') 2>(Join-Path $logs 'r526-library-build.stderr.log')
if($LASTEXITCODE -ne 0){throw 'R526 regular library failed; evidence retained'}
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r526-tests-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
& cargo test --offline --locked --lib --target i686-pc-windows-msvc --target-dir target/r409-dev-ready modern_lito::native_city_runtime::java_room_ready::r526_tests -- --nocapture 1>(Join-Path $logs 'r526-new-targeted-tests.stdout.log') 2>(Join-Path $logs 'r526-new-targeted-tests.stderr.log')
if($LASTEXITCODE -ne 0){throw 'R526 focused tests failed; evidence retained'}
Get-Content -LiteralPath (Join-Path $logs 'r526-new-targeted-tests.stdout.log')
