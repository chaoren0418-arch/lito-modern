#Requires -Version 7.0
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$project=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$output=Join-Path $project 'target/modern-bin-consumer-20261007'
function Save([string]$Path,$Value){
    $stream=[IO.File]::Open($Path,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read)
    try{$raw=[Text.Encoding]::UTF8.GetBytes((ConvertTo-Json -InputObject $Value -Depth 12)+"`n");$stream.Write($raw,0,$raw.Length)}finally{$stream.Dispose()}
}
function Pin([string]$Path){[ordered]@{path=[IO.Path]::GetFullPath($Path);bytes=(Get-Item -LiteralPath $Path).Length;sha256=(Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash}}
if(Test-Path -LiteralPath (Join-Path $output 'source')){throw 'Preserve previous source and build logs; do not replay'}
. (Join-Path $PSScriptRoot 'build-evidence.ps1')
$inputs=@(Get-ModernBuildInputs $project)
$digest=Get-ModernInputDigest $inputs
$pins=@()
foreach($row in $inputs){
    $dest=Join-Path $output ('source/'+$row.Path)
    $null=[IO.Directory]::CreateDirectory((Split-Path -Parent $dest))
    [IO.File]::Copy((Join-Path $project $row.Path),$dest,$false)
    $pins+=([ordered]@{relative=$row.Path;source=Pin (Join-Path $project $row.Path);snapshot=Pin $dest})
}
foreach($relative in @('tools/native-resource-adapter.rs','tools/build-modern-bin-consumer-20261007.ps1')){
    $dest=Join-Path $output ('source/'+$relative)
    $null=[IO.Directory]::CreateDirectory((Split-Path -Parent $dest))
    [IO.File]::Copy((Join-Path $project $relative),$dest,$false)
    $pins+=([ordered]@{relative=$relative;source=Pin (Join-Path $project $relative);snapshot=Pin $dest})
}
Save (Join-Path $output 'source-snapshot.json') ([ordered]@{input_digest=$digest;inputs=$pins})
$scratch=Join-Path (Join-Path $project 'target/build-scratch') ('bin-consumer-20261007-'+[guid]::NewGuid().ToString('N'))
$null=[IO.Directory]::CreateDirectory($scratch)
$previousTemp=$env:TEMP;$previousTmp=$env:TMP
$env:TEMP=$scratch;$env:TMP=$scratch
$lease=[IO.File]::Open((Join-Path $project 'target/project-lease.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
try{
    $arguments=@('--offline','--locked','--manifest-path',(Join-Path $project 'Cargo.toml'),'--target','i686-pc-windows-msvc','--target-dir',(Join-Path $project 'target/r409-dev-ready'),'--lib')
    & cargo build @arguments --message-format=json 1>(Join-Path $output 'build.jsonl') 2>(Join-Path $output 'build.stderr.log')
    $code=$LASTEXITCODE
    Save (Join-Path $output 'build-accounting.json') ([ordered]@{attempts=1;exit_code=$code;scratch=$scratch;command=@('cargo','build')+$arguments})
    if($code -ne 0){throw 'Library build failed; logs retained'}
    $rows=@(Get-Content -LiteralPath (Join-Path $output 'build.jsonl') -Encoding UTF8|ForEach-Object{$_|ConvertFrom-Json})
    $libs=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'})
    if($libs.Count -ne 1){throw 'Exact new library artifact required'}
    $files=@($libs[0].filenames|Where-Object{$_ -like '*.rlib'})
    if($files.Count -ne 1){throw 'Exact rlib required'}
    $library=Join-Path $output 'liblito_modern.rlib';[IO.File]::Copy($files[0],$library,$false)
    & cargo test @arguments --no-run --message-format=json 1>(Join-Path $output 'test-build.jsonl') 2>(Join-Path $output 'test-build.stderr.log')
    $code=$LASTEXITCODE
    Save (Join-Path $output 'test-build-accounting.json') ([ordered]@{attempts=1;exit_code=$code;command=@('cargo','test')+$arguments+@('--no-run')})
    if($code -ne 0){throw 'New test compilation failed; logs retained'}
    $rows=@(Get-Content -LiteralPath (Join-Path $output 'test-build.jsonl') -Encoding UTF8|ForEach-Object{$_|ConvertFrom-Json})
    $bins=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern' -and $_.profile.test})
    if($bins.Count -ne 1){throw 'Exact new test artifact required'}
    $test=Join-Path $output 'new-background-tests.exe';[IO.File]::Copy($bins[0].executable,$test,$false)
    & $test modern_bin_20261007_ --nocapture 1>(Join-Path $output 'tests.stdout.log') 2>(Join-Path $output 'tests.stderr.log')
    $code=$LASTEXITCODE
    Save (Join-Path $output 'test-accounting.json') ([ordered]@{attempts=1;exit_code=$code;filter='modern_bin_20261007_';expected_new_tests=3;old_PASS_replays=0;GUI=0;network=0;Java_SQL=0;READY=0;claims=0;command=@($test,'modern_bin_20261007_','--nocapture')})
    if($code -ne 0){throw 'New background checks failed; logs retained'}
    if($digest -cne (Get-ModernInputDigest @(Get-ModernBuildInputs $project))){throw 'Build source changed during validation'}
    Save (Join-Path $output 'artifacts.json') ([ordered]@{library=Pin $library;test=Pin $test;input_digest=$digest;library_fresh=$libs[0].fresh})
    Write-Output 'New offline background consumer: library compiled and three focused CPU checks completed.'
}finally{$lease.Dispose();$env:TEMP=$previousTemp;$env:TMP=$previousTmp}
