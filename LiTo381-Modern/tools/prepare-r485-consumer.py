from pathlib import Path
P=Path(__file__).resolve().parent.parent
old=(P/'tools/r484-batch-consumer.rs').read_text(encoding='utf-8');prefix=old[:old.index('fn study527(')];body=(P/'tools/r485-consumer-body.rs').read_text(encoding='utf-8')
(P/'tools/r485-batch-consumer.rs').write_text(prefix+body,encoding='utf-8')
build=(P/'tools/build-r484-batch-consumer.ps1').read_text(encoding='utf-8').replace('r484','r485').replace('R484','R485')
build=build.replace("'r483-candidate.json'","'r484-candidate.json'").replace("# Expected","# Expected").replace("$pins.Count -ne 157","$pins.Count -ne 160").replace('Expected157','Expected160')
oldpaths="$paths=@($old.source_pins.path)+@((Join-Path $project 'target\\r485-player-entry-candidate\\LiToModern-R485.rs'),(Join-Path $PSScriptRoot 'r485-batch-consumer.rs'))"
newpaths="$paths=@($old.source_pins.path)+@((Join-Path $project 'target\\r485-player-entry-candidate\\LiToModern-R485.rs'),(Join-Path $PSScriptRoot 'r485-batch-consumer.rs'),(Join-Path $PSScriptRoot 'r485-complete-map-reference.rs'))"
assert oldpaths in build;build=build.replace(oldpaths,newpaths)
(P/'tools/build-r485-batch-consumer.ps1').write_text(build,encoding='utf-8')
manual=(P/'tools/start-r484-player-candidate.ps1').read_text(encoding='utf-8').replace('r484','r485').replace('R483','R485');(P/'tools/start-r485-player-candidate.ps1').write_text(manual,encoding='utf-8')
print('R485 finite new format/2map/563negative consumer prepared;160 source pairs planned')
