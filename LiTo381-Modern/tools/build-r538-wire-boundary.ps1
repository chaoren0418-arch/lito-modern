$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
Set-Location -LiteralPath $project
if(Test-Path -LiteralPath (Join-Path $logs 'r538-library-build.jsonl')){throw 'Initial R538 build already exists; preserve it'}
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r538-wire-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
$env:LITO_R538_INPUT_ROOT=Join-Path $project 'target/r538-native-pet-race-wire-inputs'
$env:LITO_R538_LOOPBACK_ROOT=Join-Path $project 'target/r538-native-pet-race-loopback-wire-inputs'
$env:LITO_R538_OUTER_ROOT=Join-Path $project 'target/r535-native-pet-race-outer-inputs'
$env:LITO_R538_RETURN_ROOT=Join-Path $project 'target/r537-native-pet-race-return-inputs'
$env:LITO_R538_TERRAIN_ROOT=Join-Path $project 'target/r534-native-pet-race-route-inputs'
$env:LITO_R538_OUTPUT_ROOT=$logs
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
& cargo build --offline --locked --lib --target i686-pc-windows-msvc --target-dir target/r409-dev-ready --message-format=json 1>(Join-Path $logs 'r538-library-build.jsonl') 2>(Join-Path $logs 'r538-library-build.stderr.log')
$code=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r538-library-build-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$code;scratch_retained=$scratch;old_PASS_replays=0;GUI=0;JavaSQL=0;new_regular_library_build=$true;R532_rebuilt=$false})
if($code -ne 0){throw 'R538 initial library compile failed; preserve logs'}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r538-library-build.jsonl') -Encoding UTF8|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
$libraries=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib' -and $_ -like '*i686-pc-windows-msvc*'})
if($libraries.Count -ne 1){throw 'Exact new target library required'}
$destination=Join-Path $project 'target/r538-native-pet-race-wire-boundary/library'
[IO.Directory]::CreateDirectory($destination)|Out-Null
[IO.File]::Copy($libraries[0],(Join-Path $destination 'liblito_modern.rlib'),$false)
& cargo test --offline --locked --lib --target i686-pc-windows-msvc --target-dir target/r409-dev-ready r538_native_wire -- --nocapture 1>(Join-Path $logs 'r538-targeted-tests.stdout.log') 2>(Join-Path $logs 'r538-targeted-tests.stderr.log')
$code=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r538-targeted-tests-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$code;filter='r538_native_wire';old_tests_executed=0;new_tests_expected=3;actual_Java_calls=0;network=0;GUI=0;old_PASS_replays=0;actual_keyed_source_wire=$true;Modern_trace_delivery_hypothetical=$true;source_loopback_received_separately_proven=$true;live_World_admission=$false;new_qualified_full_bindings=0})
if($code -ne 0){throw 'R538 new focused test failure; preserve initial outcome'}
Write-Output 'R538 new existing-receiver wire boundary and three focused checks complete; no new runtime/socket.'
