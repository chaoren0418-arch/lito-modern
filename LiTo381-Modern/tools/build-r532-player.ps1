$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
$root=Join-Path $project 'target/r532-player-entry-candidate'
if(Test-Path -LiteralPath (Join-Path $logs 'r532-library-build.jsonl')){throw 'R532 initial library build log exists; preserve initial attempts'}
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r532-room-ui-build-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
& cargo build --offline --locked --lib --target i686-pc-windows-msvc --target-dir target/r409-dev-ready --message-format=json 1>(Join-Path $logs 'r532-library-build.jsonl') 2>(Join-Path $logs 'r532-library-build.stderr.log')
$buildExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r532-library-build-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$buildExit;scratch_retained=$scratch;production_source_changes=2;previous_frozen_library_rebuilt=$false;new_regular_library_build=$true;old_PASS_replays=0;JavaSQL=0;GUI=0})
if($buildExit -ne 0){throw 'R532 regular candidate compile failed; preserve first attempt'}
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r532-library-build.jsonl') -Encoding UTF8|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib' -and $_ -like '*i686-pc-windows-msvc*'});if($p.Count -ne 1){throw "Exact target $name rlib required"};$p[0]}
$libRoot=Join-Path $root 'library';[IO.Directory]::CreateDirectory($libRoot)|Out-Null
$lib=Join-Path $libRoot 'liblito_modern.rlib';[IO.File]::Copy((Artifact 'lito_modern'),$lib,$false)
$filter='r532_counterexample_checks'
& cargo test --offline --locked --lib --target i686-pc-windows-msvc --target-dir target/r409-dev-ready $filter -- --nocapture 1>(Join-Path $logs 'r532-targeted-tests.stdout.log') 2>(Join-Path $logs 'r532-targeted-tests.stderr.log')
$testExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r532-targeted-tests-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$testExit;filter=$filter;old_tests_executed=0;own_CPU_Runtime_only=$true;qualified_positive_UI_callback_or_GUI_executed=$false;network=0;JavaSQL=0;credentials_reads=0;GUI=0;old_PASS_replays=0})
if($testExit -ne 0){throw 'R532 new targeted checks failed; preserve first result, no automatic rerun'}
$entry=Get-Content -LiteralPath (Join-Path $logs 'r529-entry-source-contract.json') -Raw -Encoding UTF8|ConvertFrom-Json
$entry.candidate_main=Pin (Join-Path $root 'LiToModern-R532.rs')
foreach($pin in @($entry.original_main,$entry.candidate_main,$entry.binding,$entry.skin_binding)){if((Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256 -or (Get-Item -LiteralPath $pin.path).Length -ne $pin.bytes){throw 'R532 candidate entry source/pin drift'}}
$env:R532_MODERN_ROOT=$project;$env:R532_BINDING_MANIFEST=$entry.binding.path;$env:R532_BINDING_SHA256=$entry.binding.sha256;$env:R532_BINDING_BYTES=[string]$entry.binding.bytes;$env:CARGO_PKG_VERSION='0.1.0';$env:R532_SKIN_MANIFEST=$entry.skin_binding.path;$env:R532_SKIN_SHA256=$entry.skin_binding.sha256;$env:R532_SKIN_BYTES=[string]$entry.skin_binding.bytes
$exe=Join-Path $root 'LiToModern-R532.exe'
$arguments=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','lito_modern_r532_player',$entry.candidate_main.path,'--extern',"lito_modern=$lib",'--extern',"anyhow=$(Artifact 'anyhow')",'--extern',"windows=$(Artifact 'windows')",'--extern',"serde_json=$(Artifact 'serde_json')",'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps')),'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/debug/deps')))
foreach($p in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$arguments+=@('-L',$p)}
& rustc @arguments -o $exe 1>(Join-Path $logs 'r532-player-build.stdout.log') 2>(Join-Path $logs 'r532-player-build.stderr.log')
$playerExit=$LASTEXITCODE
Write-ModernEvidenceJson -Path (Join-Path $logs 'r532-player-build-accounting.json') -Value ([ordered]@{attempts=1;exit_code=$playerExit;GUI=0;candidate_executions=0;network=0;previous_frozen_consumers_rebuilt=$false})
if($playerExit -ne 0){throw 'New separate candidate player compile failed; first result retained'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r532-player-candidate.json') -Value ([ordered]@{state='R532_BACKEND485F_BOUNDED_RECEIPT_EVIDENCE_COMPILED_GUI_UNEXECUTED';executable=Pin $exe;production_library=Pin $lib;candidate_main=$entry.candidate_main;original_main=$entry.original_main;binding=$entry.binding;skin_binding=$entry.skin_binding;production_source_changes=2;production_source_count=644;candidate_executions=0;GUI=0;JavaSQL=0;old_PASS_replays=0})
Write-Output 'R532 candidate linked and frozen; no executable/GUI run, new targeted CPU checks only'

