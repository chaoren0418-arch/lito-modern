param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
$candidate=Get-Content -LiteralPath (Join-Path $project 'logs\continuation-20261003\r427-candidate.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $candidate.path).Hash -ne $candidate.sha256){throw 'R427 fixed binary drift'}
$out=Join-Path $project ('logs\r427-local-recheck-'+[Guid]::NewGuid().ToString('N'))
$null=New-Item -ItemType Directory -Path $out
foreach($filter in @('native_canonical_draft','modern_lito::battle_world::tests::','modern_lito::protocol::extension_tests::')){
    $label=$filter.Replace('::','-')
    & $candidate.path $filter --nocapture --test-threads=1 1> (Join-Path $out ($label+'.stdout.log')) 2> (Join-Path $out ($label+'.stderr.log'))
    if($LASTEXITCODE -ne 0){throw ('Failed local regression '+$filter)}
}
Write-Output $out
