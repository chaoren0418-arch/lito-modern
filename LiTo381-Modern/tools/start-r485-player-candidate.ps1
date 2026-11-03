param([switch]$OfflinePreview)
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$candidate=Get-Content (Join-Path $project 'logs\continuation-20261003\r485-player-candidate.json') -Raw|ConvertFrom-Json
foreach($pin in @($candidate.executable,$candidate.production_library,$candidate.candidate_main,$candidate.binding,$candidate.skin_binding)){if((Get-FileHash -LiteralPath $pin.path).Hash -ne $pin.sha256 -or (Get-Item -LiteralPath $pin.path).Length -ne $pin.bytes){throw 'R485 fixed candidate fingerprint mismatch'}}
$flag=if($OfflinePreview){'--offline-map-preview'}else{'--candidate-menu'}
# This launcher is for the user's explicitly selected interactive player window.
$process=Start-Process -FilePath $candidate.executable.path -ArgumentList $flag -WorkingDirectory $project -WindowStyle Normal -Wait -PassThru
if($process.ExitCode -ne 0){throw 'Player candidate exited with an error'}
