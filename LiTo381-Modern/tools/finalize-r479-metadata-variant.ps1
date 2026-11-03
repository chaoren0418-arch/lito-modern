$ErrorActionPreference='Stop';$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;bytes=(Get-Item -LiteralPath $p).Length;sha256=(Get-FileHash -LiteralPath $p).Hash}}
$old=Get-Content -LiteralPath (Join-Path $logs 'r479a-before-metadata-refinement-host.json') -Raw|ConvertFrom-Json
if($old.source_pins.Count -ne 135){throw 'Original135 source pairs required'}
$entry=Get-Content -LiteralPath (Join-Path $logs 'r479b-entry-source-contract.json') -Raw|ConvertFrom-Json
$root=Join-Path $project 'target\r479b-final-source-snapshot';if(Test-Path -LiteralPath $root){throw 'Final variant snapshots already exist'}
[IO.Directory]::CreateDirectory($root)|Out-Null
$pins=@(foreach($v in $old.source_pins){if((Get-FileHash -LiteralPath $v.path).Hash -ne $v.sha256 -or (Get-FileHash -LiteralPath $v.snapshot).Hash -ne $v.sha256){throw 'Actual focused sources changed'};$dst=Join-Path $root (Split-Path $v.path -Leaf);[IO.File]::Copy($v.snapshot,$dst,$false);$n=Pin $v.path;$n.snapshot=$dst;$n})
$dst=Join-Path $root (Split-Path $entry.candidate_main.path -Leaf);[IO.File]::Copy($entry.candidate_main.path,$dst,$false);$n=Pin $entry.candidate_main.path;$n.snapshot=$dst;$pins+=@($n)
$old.source_pins=$pins;$old.validation='PASS original focused host/library sources byte-unchanged;136 fixed pairs include new separately compiled R479b main. Descriptive-only binding correction proven, no focused rerun.'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r479-candidate.json') -Value $old
$proof=Get-Content -LiteralPath (Join-Path $logs 'r479-metadata-refinement.json') -Raw|ConvertFrom-Json
$oldPlayer=Get-Content -LiteralPath (Join-Path $logs 'r479a-before-metadata-refinement-player.json') -Raw|ConvertFrom-Json
$player=Get-Content -LiteralPath (Join-Path $logs 'r479-player-candidate.json') -Raw|ConvertFrom-Json
if($oldPlayer.production_library.sha256 -ne $player.production_library.sha256){throw 'Library changed for metadata-only correction'}
$proof|Add-Member -NotePropertyName production_library_byte_identical -NotePropertyValue $true -Force
$proof|Add-Member -NotePropertyName new_player -NotePropertyValue (Pin (Join-Path $logs 'r479-player-candidate.json')) -Force
$proof|Add-Member -NotePropertyName final_source_pairs -NotePropertyValue 136 -Force
Write-ModernEvidenceJson -Path (Join-Path $logs 'r479-metadata-refinement.json') -Value $proof
Write-Output 'R479b136 fixed source pairs; actual focused source and library unchanged'
