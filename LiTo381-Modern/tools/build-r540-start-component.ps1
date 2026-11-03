$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
Set-Location -LiteralPath $project
if(Test-Path -LiteralPath (Join-Path $logs 'r540-library-build.jsonl')){throw 'Preserve existing initial R540 build'}
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r540-start-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
$env:LITO_R540_INPUT_ROOT=Join-Path $project 'target/r540-native-pet-race-start-inputs'
$env:LITO_R540_COMPONENT_ROOT=Join-Path $project 'target/r534-native-pet-race-route-inputs'
$env:LITO_R540_OUTPUT_ROOT=$logs
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
& cargo build --offline --locked --lib --target i686-pc-windows-msvc --target-dir target/r409-dev-ready --message-format=json 1>(Join-Path $logs 'r540-library-build.jsonl') 2>(Join-Path $logs 'r540-library-build.stderr.log')
$code=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r540-library-build-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$code;scratch_retained=$scratch;old_PASS_replays=0;GUI=0;JavaSQL=0;new_regular_library_build=$true;R532_rebuilt=$false})
if($code -ne 0){throw 'R540 initial library compile failed; preserve logs'}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r540-library-build.jsonl') -Encoding UTF8|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
$libraries=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib' -and $_ -like '*i686-pc-windows-msvc*'})
if($libraries.Count -ne 1){throw 'Exact new target library required'}
$destination=Join-Path $project 'target/r540-native-pet-race-start-component/library'
[IO.Directory]::CreateDirectory($destination)|Out-Null
[IO.File]::Copy($libraries[0],(Join-Path $destination 'liblito_modern.rlib'),$false)
& cargo test --offline --locked --lib --target i686-pc-windows-msvc --target-dir target/r409-dev-ready r540_start_component -- --nocapture 1>(Join-Path $logs 'r540-targeted-tests.stdout.log') 2>(Join-Path $logs 'r540-targeted-tests.stderr.log')
$code=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r540-targeted-tests-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$code;filter='r540_start_component';old_tests_executed=0;new_tests_expected=3;actual_Java_calls=0;network=0;GUI=0;old_PASS_replays=0;own_key_source_wire=$true;Modern_trace_delivery_hypothetical=$true;full_World_admission=$false;new_qualified_full_bindings=0})
if($code -ne 0){throw 'R540 focused test failure; preserve initial outcome'}
Write-Output 'R540 three new component checks complete; no full WORLD/runtime/socket.'
