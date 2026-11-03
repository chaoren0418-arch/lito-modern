param([Parameter(Mandatory)][string]$Revision)
$ErrorActionPreference='Stop'
if($Revision -notmatch '^r435[a-z]$'){throw 'Invalid revision'}
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$a=@(Get-Content -LiteralPath (Join-Path $logs "$Revision-host-build.jsonl")|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($a|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'No successful build'}
$exe=@($a|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern' -and $_.executable}|ForEach-Object{$_.executable})
if($exe.Count -ne 1){throw 'Host count'}
$root=Join-Path $project "target\$Revision-fixed-native-actor-host"
if(Test-Path -LiteralPath $root){throw 'Already frozen'}
[IO.Directory]::CreateDirectory((Join-Path $root 'source'))|Out-Null
$target=Join-Path $root "lito-modern-$Revision-host-tests.exe"
Copy-Item -LiteralPath $exe[0] -Destination $target
$pins=@(foreach($name in @('native_actor_dependency_draft.rs','native_actor_dependency_draft_tests.rs','native_wire_capture_tests.rs','native_event_scheduler.rs','native_event_scheduler_tests.rs','native_scene_composition.rs','native_scene_composition_tests.rs','native_catalog_draft.rs','native_actor_inspection.rs','native_action_inspection.rs','native_resources.rs','native_canonical_draft.rs','native_renderer_slice_tests.rs','mod.rs')){
 $p=Join-Path $project "src\modern_lito\$name";$dest=Join-Path $root "source\$name";Copy-Item -LiteralPath $p -Destination $dest
 $sha=(Get-FileHash -LiteralPath $p).Hash;if((Get-FileHash -LiteralPath $dest).Hash -ne $sha){throw 'Copy mismatch'}
 [ordered]@{path=$p;snapshot=$dest;sha256=$sha}
})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r435-candidate.json') -Value ([ordered]@{path=$target;sha256=(Get-FileHash -LiteralPath $target).Hash;source_pins=$pins;validation='Pending fixed consumer';LOADED=$false;formal_R408_changed=$false})
& $target 'modern_lito::native_scene_composition::tests::' '--nocapture' 1> (Join-Path $logs "$Revision-local.stdout.log") 2> (Join-Path $logs "$Revision-local.stderr.log")
if($LASTEXITCODE -ne 0){Get-Content -LiteralPath (Join-Path $logs "$Revision-local.stdout.log");Get-Content -LiteralPath (Join-Path $logs "$Revision-local.stderr.log");throw 'Local tests failed'}
& $target 'modern_lito::native_scene_composition::tests::real_native_scene_interleaving_and_owner_bound_partial' '--exact' '--ignored' '--nocapture' 1> (Join-Path $logs "$Revision-consumer.stdout.log") 2> (Join-Path $logs "$Revision-consumer.stderr.log")
if($LASTEXITCODE -ne 0){Get-Content -LiteralPath (Join-Path $logs "$Revision-consumer.stdout.log");Get-Content -LiteralPath (Join-Path $logs "$Revision-consumer.stderr.log");throw 'Consumer failed'}
Get-Content -LiteralPath (Join-Path $logs "$Revision-local.stdout.log") -Tail 3
Get-Content -LiteralPath (Join-Path $logs "$Revision-consumer.stdout.log") -Tail 3
