param([string]$OutputDirectory=(Join-Path $PSScriptRoot '..\logs\continuation-20261003\player-asset-verification'))
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$project=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$workspace=Split-Path (Split-Path (Split-Path $project))
$shared=Join-Path $workspace '0-客戶端'
$candidate=Get-Content -LiteralPath (Join-Path $project 'logs\continuation-20261003\r410-candidate.json') -Raw|ConvertFrom-Json
if((Get-FileHash -LiteralPath $candidate.path).Hash -ne $candidate.sha256){throw 'Immutable R410 host changed'}
if(Test-Path -LiteralPath (Join-Path $OutputDirectory 'results.json')){throw 'Do not overwrite completed asset verification; select a new output directory'}
$null=New-Item -ItemType Directory -Path $OutputDirectory -Force
$tests=@(
    'real_shared_hud_members_qualify_and_zero_mp_draws',
    'real_shared_f5_f12_frame_is_qualified',
    'real_shared_skill_ids_resolve_to_decodable_icons',
    'real_worker_skill_icon_and_proof_are_bound_to_current_skill_model',
    'real_worker_bound_supply_is_visible_and_only_current_server_proof_acts',
    'real_shared_inventory_icons_match_independent_pakviewer_load_l1_image',
    'shared_tw_pack_has_real_ultimate_definitions_without_base_catalog_fallback'
)
$saved=[Environment]::GetEnvironmentVariable('LITO_SHARED_RESOURCE_ROOT','Process')
$results=[Collections.Generic.List[object]]::new()
Push-Location $project
try {
    $env:LITO_SHARED_RESOURCE_ROOT=$shared
    foreach($test in $tests){
        $stdout=Join-Path $OutputDirectory ($test+'.stdout.log')
        $stderr=Join-Path $OutputDirectory ($test+'.stderr.log')
        & $candidate.path $test --ignored --nocapture --test-threads=1 1> $stdout 2> $stderr
        $exitCode=$LASTEXITCODE
        $summary=[regex]::Match((Get-Content -LiteralPath $stdout -Raw),'test result: ok\. 1 passed; 0 failed; 0 ignored;.*finished in ([0-9.]+)s')
        $results.Add([ordered]@{test=$test;exit_code=$exitCode;passed=$exitCode -eq 0 -and $summary.Success;elapsed_seconds=if($summary.Success){[double]::Parse($summary.Groups[1].Value,[Globalization.CultureInfo]::InvariantCulture)}else{$null};stdout=$stdout;stderr=$stderr;stdout_sha256=(Get-FileHash -LiteralPath $stdout).Hash})
        Write-Output ($test+': exit='+$exitCode)
    }
} finally {
    Pop-Location
    [Environment]::SetEnvironmentVariable('LITO_SHARED_RESOURCE_ROOT',$saved,'Process')
    Write-ModernEvidenceJson -Path (Join-Path $OutputDirectory 'results.json') -Value ([ordered]@{verified_at=[DateTimeOffset]::Now.ToString('o');test_host=$candidate.path;test_host_sha256=$candidate.sha256;shared_root=$shared;results=$results.ToArray();passed=@($results|Where-Object{$_.passed}).Count;scope='Read-only actual shared HUD/IMG/TBT/TW action pack and worker proofs. No HWND/DPI/click acceptance or native process; no Java/SQL/VIP full-scene claim.'})
}
if(@($results|Where-Object{-not $_.passed}).Count){throw 'One or more explicit asset verifications failed'}
