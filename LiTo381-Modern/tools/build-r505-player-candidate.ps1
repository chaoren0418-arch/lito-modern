$ErrorActionPreference='Stop'
$scratch='D:\lito-dev-runtime\20261003-supervision\modern\r495-native-send'
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003';$root=Join-Path $project 'target\r505-player-entry-candidate'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$rows=@(Get-Content (Join-Path $logs 'r505a-library-build.jsonl')|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($rows|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Final UI library incomplete'}
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($p.Count -ne 1){throw "Exact $name rlib required"};$p[0]}
$entryPath=Join-Path $logs 'r505-entry-source-contract.json';$entry=Get-Content $entryPath -Raw -Encoding UTF8|ConvertFrom-Json
foreach($pin in @($entry.original_main,$entry.candidate_main,$entry.binding,$entry.skin_binding)){if((Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256 -or (Get-Item -LiteralPath $pin.path).Length -ne $pin.bytes){throw 'Candidate entry source drift'}}
$libRoot=Join-Path $root 'library';[IO.Directory]::CreateDirectory($libRoot)|Out-Null;$lib=Join-Path $libRoot 'liblito_modern.rlib';[IO.File]::Copy((Artifact 'lito_modern'),$lib,$false)
$env:R505_MODERN_ROOT=$project;$env:R505_BINDING_MANIFEST=$entry.binding.path;$env:R505_BINDING_SHA256=$entry.binding.sha256;$env:R505_BINDING_BYTES=[string]$entry.binding.bytes;$env:CARGO_PKG_VERSION='0.1.0';$env:R505_SKIN_MANIFEST=$entry.skin_binding.path;$env:R505_SKIN_SHA256=$entry.skin_binding.sha256;$env:R505_SKIN_BYTES=[string]$entry.skin_binding.bytes
$exe=Join-Path $root 'LiToModern-R505.exe';$args=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','lito_modern_r505_player',$entry.candidate_main.path,'--extern',"lito_modern=$lib",'--extern',"anyhow=$(Artifact 'anyhow')",'--extern',"windows=$(Artifact 'windows')",'--extern',"serde_json=$(Artifact 'serde_json')",'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\i686-pc-windows-msvc\debug\deps')),'-L',('dependency='+ (Join-Path $project 'target\r409-dev-ready\debug\deps')))
foreach($p in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$args+=@('-L',$p)};$args+=@('-o',$exe)
& rustc @args 1> (Join-Path $logs 'r505-player-build.stdout.log') 2> (Join-Path $logs 'r505-player-build.stderr.log');if($LASTEXITCODE -ne 0){throw 'Separate player candidate failed to build'}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r505-player-candidate.json') -Value ([ordered]@{state='R505_SEPARATE_PLAYER_CANDIDATE_COMPILED_STATIC_ENTRY_ONLY_NEVER_EXECUTED';executable=Pin $exe;production_library=Pin $lib;original_main=$entry.original_main;candidate_main=$entry.candidate_main;binding=$entry.binding;skin_binding=$entry.skin_binding;entry_source_contract=Pin $entryPath;default_no_args_network_route_unchanged=$true;main_source_modified=$false;R408_rebuilt_or_overwritten=$false;candidate_executions=0;only_executed_candidate_flag=$null;native_GUI_executed=$false;GUI=0;LOADED=$false})
Write-Output 'R505 player linked and frozen;0 executions (no --version or GUI launch)'
