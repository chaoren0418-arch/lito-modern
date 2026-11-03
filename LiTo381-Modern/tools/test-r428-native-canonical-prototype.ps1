param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$candidate=Get-Content -LiteralPath (Join-Path $project 'logs\continuation-20261003\r428-candidate.json') -Raw|ConvertFrom-Json
$snapshot=Get-Content -LiteralPath (Join-Path $project 'logs\continuation-20261003\r428-frozen-prototype-snapshot.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $candidate.path).Hash -ne $candidate.sha256 -or (Get-FileHash -LiteralPath $snapshot.input).Hash -ne $snapshot.input_sha256){throw 'Fixed binary/input drift'}
$out=Join-Path $project ('logs\r428-prototype-recheck-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $out
$savedInput=$env:LITO_NATIVE_CANONICAL_PROTOTYPE_INPUT;$savedPin=$env:LITO_NATIVE_CANONICAL_PROTOTYPE_INPUT_SHA256
try {
    $env:LITO_NATIVE_CANONICAL_PROTOTYPE_INPUT=$snapshot.input
    $env:LITO_NATIVE_CANONICAL_PROTOTYPE_INPUT_SHA256=$snapshot.input_sha256
    & $candidate.path native_canonical_draft --nocapture --test-threads=1 1> (Join-Path $out 'local.stdout.log') 2> (Join-Path $out 'local.stderr.log')
    if($LASTEXITCODE -ne 0){throw 'Local Native tests failed'}
    & $candidate.path frozen_java_six_requests_eight_replies_proofs_and_43_negative_equivalents --ignored --nocapture --test-threads=1 1> (Join-Path $out 'prototype.stdout.log') 2> (Join-Path $out 'prototype.stderr.log')
    if($LASTEXITCODE -ne 0){throw 'Java frozen prototype consumer failed'}
} finally {$env:LITO_NATIVE_CANONICAL_PROTOTYPE_INPUT=$savedInput;$env:LITO_NATIVE_CANONICAL_PROTOTYPE_INPUT_SHA256=$savedPin}
Write-Output $out
