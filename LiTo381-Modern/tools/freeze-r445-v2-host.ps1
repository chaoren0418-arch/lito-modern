param([Parameter(Mandatory)][ValidatePattern('^r445[a-z]$')][string]$Revision,[switch]$SkipLocalGuards)
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$a=@(Get-Content (Join-Path $logs "$Revision-host-build.jsonl")|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($a|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Build incomplete'}
$exe=@($a|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern' -and $_.executable}|ForEach-Object{$_.executable});if($exe.Count -ne 1){throw 'Host count'}
$root=Join-Path $project "target\$Revision-fixed-external-control-v2-host";if(Test-Path $root){throw 'Already frozen'};[IO.Directory]::CreateDirectory((Join-Path $root 'source'))|Out-Null
$target=Join-Path $root "lito-modern-$Revision-host-tests.exe";[IO.File]::Copy($exe[0],$target,$false)
$old=Get-Content (Join-Path $logs 'r444-candidate.json') -Raw|ConvertFrom-Json
$paths=@($old.source_pins.path)+@((Join-Path $project 'src\modern_lito\native_external_control_v2.rs'),(Join-Path $project 'src\modern_lito\native_external_control_v2_tests.rs'))
$pins=@(foreach($p in $paths){$dst=Join-Path $root ('source\'+(Split-Path $p -Leaf));[IO.File]::Copy($p,$dst,$false);$sha=(Get-FileHash $p).Hash;if((Get-FileHash $dst).Hash -ne $sha){throw 'Source copy drift'};[ordered]@{path=$p;snapshot=$dst;sha256=$sha}})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r445-candidate.json') -Value ([ordered]@{path=$target;sha256=(Get-FileHash $target).Hash;source_pins=$pins;validation='PENDING1newv2guard+1affectedv1Readyguard+limitedactualv2live';LOADED=$false})
if($SkipLocalGuards){Write-Output 'Frozen corrected test-only host;unchanged v2guard/v1Ready guard results retained fromr445c';return}
foreach($test in @('modern_lito::native_external_fixture::v2::tests::v2_ready_and_same_Work_witness_mutants_are_strict_and_v1_stays_distinct','modern_lito::native_external_fixture::tests::ready_rejects_foreign_candidate_expired_and_wrong_endpoint_protocol_source')){
 $name=if($test -like '*::v2::*'){'v2guard'}else{'v1Ready-regression'}
 & $target $test '--exact' '--nocapture' 1> (Join-Path $logs "$Revision-$name.stdout.log") 2> (Join-Path $logs "$Revision-$name.stderr.log")
 if($LASTEXITCODE -ne 0){Get-Content (Join-Path $logs "$Revision-$name.stdout.log");Get-Content (Join-Path $logs "$Revision-$name.stderr.log");throw 'Focused test failed'}
 Get-Content (Join-Path $logs "$Revision-$name.stdout.log") -Tail 3
}
