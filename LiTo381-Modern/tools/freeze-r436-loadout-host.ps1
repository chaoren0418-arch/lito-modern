param([Parameter(Mandatory)][string]$Revision)
$ErrorActionPreference='Stop'
if($Revision -notmatch '^r436[a-z]$'){throw 'Revision'}
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
$a=@(Get-Content (Join-Path $logs "$Revision-host-build.jsonl")|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($a|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Build incomplete'}
$exe=@($a|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern' -and $_.executable}|ForEach-Object{$_.executable});if($exe.Count -ne 1){throw 'Host count'}
$root=Join-Path $project "target\$Revision-fixed-native-loadout-host";if(Test-Path $root){throw 'Already frozen'};[IO.Directory]::CreateDirectory((Join-Path $root 'source'))|Out-Null
$target=Join-Path $root "lito-modern-$Revision-host-tests.exe";[IO.File]::Copy($exe[0],$target,$false)
$pins=@(foreach($name in @('native_actor_dependency_draft.rs','native_actor_dependency_draft_tests.rs','native_wire_capture_tests.rs','native_event_scheduler.rs','native_event_scheduler_tests.rs','native_scene_composition.rs','native_scene_composition_tests.rs','native_player_loadout_evidence_tests.rs','native_catalog_draft.rs','native_actor_inspection.rs','native_action_inspection.rs','native_resources.rs','native_canonical_draft.rs','native_renderer_slice_tests.rs','mod.rs')){
 $p=Join-Path $project "src\modern_lito\$name";$dest=Join-Path $root "source\$name";[IO.File]::Copy($p,$dest,$false);$sha=(Get-FileHash $p).Hash;if((Get-FileHash $dest).Hash -ne $sha){throw 'Copy drift'};[ordered]@{path=$p;snapshot=$dest;sha256=$sha}
})
Write-ModernEvidenceJson -Path (Join-Path $logs 'r436-candidate.json') -Value ([ordered]@{path=$target;sha256=(Get-FileHash $target).Hash;source_pins=$pins;validation='Pending4 new offline loadout tests';LOADED=$false;formal_R408_changed=$false})
& $target 'modern_lito::native_player_loadout_evidence_tests::' '--nocapture' 1> (Join-Path $logs "$Revision-consumer.stdout.log") 2> (Join-Path $logs "$Revision-consumer.stderr.log")
if($LASTEXITCODE -ne 0){Get-Content (Join-Path $logs "$Revision-consumer.stdout.log");Get-Content (Join-Path $logs "$Revision-consumer.stderr.log");throw 'Consumer failed'}
Get-Content (Join-Path $logs "$Revision-consumer.stdout.log") -Tail 4
