from pathlib import Path
P=Path(__file__).resolve().parent.parent
old=(P/'tools/r484-batch-consumer.rs').read_text(encoding='utf-8');prefix=old[:old.index('fn study527(')]
(P/'tools/r486-batch-consumer.rs').write_text(prefix+(P/'tools/r486-consumer-body.rs').read_text(encoding='utf-8'),encoding='utf-8')
b=(P/'tools/build-r485c-batch-consumer.ps1').read_text(encoding='utf-8').replace('r485c','r486').replace('r485','r486').replace('R485','R486').replace('r486a-library-build','r486b-library-build').replace("'r484-candidate.json'","'r485-candidate.json'").replace('$pins.Count -ne 160','$pins.Count -ne 164').replace('Expected160','Expected164')
oldpaths="$paths=@($old.source_pins.path)+@((Join-Path $project 'target\\r486-player-entry-candidate\\LiToModern-R486.rs'),(Join-Path $PSScriptRoot 'r486-batch-consumer.rs'),(Join-Path $PSScriptRoot 'r486-complete-map-reference.rs'))"
newpaths="$paths=@($old.source_pins.path)+@((Join-Path $project 'target\\r486-player-entry-candidate\\LiToModern-R486.rs'),(Join-Path $PSScriptRoot 'r486-batch-consumer.rs'),(Join-Path $PSScriptRoot 'r486-map563-reference.rs'),(Join-Path $project 'src\\modern_lito\\native_city_tile_cache.rs'))"
assert oldpaths in b;b=b.replace(oldpaths,newpaths)
(P/'tools/build-r486-batch-consumer.ps1').write_text(b,encoding='utf-8')
print('R486 finite actual563 bounded-cache consumer prepared;164 source pairs')
