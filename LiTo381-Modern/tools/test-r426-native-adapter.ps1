# Explicit reproduction only. Do not rerun completed regression without a new concern.
param([switch]$IncludeActualNative564)
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$logs=Join-Path $project 'logs\continuation-20261003'
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$candidate=Get-Content -LiteralPath (Join-Path $logs 'r426-candidate.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $candidate.path).Hash -ne $candidate.sha256){throw 'Fixed R426 candidate drift'}
$folder=Join-Path $logs ('r426-replay-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $folder
foreach($filter in @('modern_lito::native_resources::tests::','modern_lito::tile::tests::','modern_lito::tile_reader::tests::')){
    $name=$filter.Split('::')[1]
    & $candidate.path $filter --nocapture --test-threads=1 1> (Join-Path $folder ($name+'.stdout.log')) 2> (Join-Path $folder ($name+'.stderr.log'))
    if($LASTEXITCODE -ne 0){throw ('Regression failed: '+$filter)}
}
if($IncludeActualNative564){
    $fixture=Get-Content -LiteralPath (Join-Path $logs 'native-dm564-fixture-snapshot.json') -Raw|ConvertFrom-Json
    if((Get-FileHash -LiteralPath $fixture.consumer_input).Hash -ne $fixture.consumer_input_sha256){throw 'Fixed Native input drift'}
    $env:LITO_MODERN_NATIVE_PINNED_INPUT=$fixture.consumer_input
    $env:LITO_MODERN_NATIVE_OUTPUT=Join-Path $folder 'roof-CPU'
    $null=New-Item -ItemType Directory -Path $env:LITO_MODERN_NATIVE_OUTPUT
    & $candidate.path 'modern_lito::native_resources::tests::real_native564_adapter_static_cancel_drift_and_roof_compositor' --exact --ignored --nocapture --test-threads=1 1> (Join-Path $folder 'actual.stdout.log') 2> (Join-Path $folder 'actual.stderr.log')
    if($LASTEXITCODE -ne 0){throw 'Actual Native regression failed'}
}
Write-ModernEvidenceJson -Path (Join-Path $folder 'completion.json') -Value ([ordered]@{completed_at=[DateTimeOffset]::Now.ToString('o');binary=$candidate.path;binary_sha256=$candidate.sha256;actual_native_included=[bool]$IncludeActualNative564;mode_additions=0;GUI=0})
Write-Output $folder
