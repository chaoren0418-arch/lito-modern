param([Parameter(Mandatory)][ValidatePattern('^r4(47|48|49|50|51|52|53|54|55|56|57|58|59|60|61|62|63|64|65|66|67|68|69|70|71|72|73|74|75|76|77|78|79|80)[a-z]$')][string]$Revision,[Parameter(Mandatory)][string]$Previous,[Parameter(Mandatory)][string[]]$NewSources,[Parameter(Mandatory)][string]$Test,[switch]$NoExecute)
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
$build=@(Get-Content (Join-Path $logs "$Revision-host-build.jsonl")|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Host build incomplete'}
$exe=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern' -and $_.executable}|ForEach-Object{$_.executable});if($exe.Count -ne 1){throw 'Host count'}
$old=Get-Content (Join-Path $logs "$Previous-candidate.json") -Raw|ConvertFrom-Json
$paths=@($old.source_pins.path)+@($NewSources|ForEach-Object{Join-Path $project $_})
$root=Join-Path $project "target\$Revision-fixed-finite-appearance-host";if(Test-Path $root){throw 'Already frozen'}
[IO.Directory]::CreateDirectory((Join-Path $root 'source'))|Out-Null
$hostPath=Join-Path $root "$Revision-tests.exe";[IO.File]::Copy($exe[0],$hostPath,$false)
$pins=@(foreach($p in $paths|Select-Object -Unique){$dst=Join-Path $root ('source\'+(Split-Path $p -Leaf));[IO.File]::Copy($p,$dst,$false);$pin=Pin $p;if((Get-FileHash $dst).Hash -ne $pin.sha256){throw 'Source copy drift'};$pin.snapshot=$dst;$pin})
$candidate=[ordered]@{path=$hostPath;sha256=(Get-FileHash $hostPath).Hash;bytes=(Get-Item $hostPath).Length;source_pins=$pins;validation='PENDING finite focused test';LOADED=$false}
Write-ModernEvidenceJson -Path (Join-Path $logs "$Revision-candidate.json") -Value $candidate
if($NoExecute){Write-Output "Fixed source/host preserved without execution";return}
& $hostPath $Test '--exact' '--nocapture' 1> (Join-Path $logs "$Revision-consumer.stdout.log") 2> (Join-Path $logs "$Revision-consumer.stderr.log")
if($LASTEXITCODE -ne 0){Get-Content (Join-Path $logs "$Revision-consumer.stdout.log");throw 'Focused test failed'}
if([IO.File]::ReadAllText((Join-Path $logs "$Revision-consumer.stdout.log")) -notmatch '1 passed; 0 failed; 0 ignored;'){throw 'Exactly one focused test must pass; zero selected is not verification'}
foreach($p in $pins){if((Get-FileHash $p.path).Hash -ne $p.sha256 -or (Get-FileHash $p.snapshot).Hash -ne $p.sha256){throw 'Source drift during test'}}
Get-Content (Join-Path $logs "$Revision-consumer.stdout.log") -Tail 4
