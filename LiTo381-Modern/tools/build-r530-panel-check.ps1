$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs/continuation-20261003'
$root=Join-Path $project 'target/r530-player-entry-candidate'
if(Test-Path -LiteralPath (Join-Path $logs 'r530-panel-build.stderr.log')){throw 'First compile log exists; preserve attempts'}
$scratch=Join-Path 'D:\lito-dev-runtime\20261003-supervision\modern' ('r530-panel-check-'+[guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($scratch)|Out-Null
$env:TEMP=$scratch;$env:TMP=$scratch
$rows=@(Get-Content -LiteralPath (Join-Path $logs 'r530-library-build.jsonl') -Encoding UTF8|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
function Artifact($name){$p=@($rows|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq $name}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib' -and $_ -like '*i686-pc-windows-msvc*'});if($p.Count -ne 1){throw "Exact target $name required"};$p[0]}
$arguments=@('--edition=2021','--target','i686-pc-windows-msvc','--crate-name','r530_panel_check',(Join-Path $PSScriptRoot 'r530-panel-callback-check.rs'),'--extern',('lito_modern='+ (Join-Path $root 'library/liblito_modern.rlib')),'--extern',('anyhow='+ (Artifact 'anyhow')),'--extern',('serde_json='+ (Artifact 'serde_json')),'--extern',('sha2='+ (Artifact 'sha2')),'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps')),'-L',('dependency='+ (Join-Path $project 'target/r409-dev-ready/debug/deps')))
foreach($p in @($rows|Where-Object{$_.reason -eq 'build-script-executed'}|ForEach-Object{$_.linked_paths}|Select-Object -Unique)){$arguments+=@('-L',$p)}
$exe=Join-Path $root 'r530-panel-check.exe'
& rustc @arguments -o $exe 1>(Join-Path $logs 'r530-panel-build.stdout.log') 2>(Join-Path $logs 'r530-panel-build.stderr.log')
if($LASTEXITCODE -ne 0){throw 'New Panel check compile failed; first failure retained'}
& $exe $project 1>(Join-Path $logs 'r530-panel-tests.stdout.log') 2>(Join-Path $logs 'r530-panel-tests.stderr.log')
if($LASTEXITCODE -ne 0){throw 'New Panel check failed; first result retained, no automatic replay'}
Write-Output 'R530 new finite Panel callback check passed; no Java/SQL/provider/GUI run'
