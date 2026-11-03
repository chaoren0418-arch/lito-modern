#Requires -Version 7.0
# Record current layered composition timings and explicit changes from flat history.
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
function Read-Json([string]$path){Get-Content -LiteralPath (Join-Path $root $path) -Raw|ConvertFrom-Json}
function Median($values){$v=@($values|Sort-Object);if(!$v.Count){throw 'Missing timing samples'};if($v.Count%2){return $v[[int][Math]::Floor($v.Count/2)]};return ($v[$v.Count/2-1]+$v[$v.Count/2])/2}
$sha=(Get-FileHash -LiteralPath (Join-Path $root 'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe')).Hash
$pin=Read-Json '驗證紀錄/current/reference-snapshot.json'
$prior=Read-Json '驗證紀錄/20261012/native-ground-pickup-build-receipt.json'
$layerPrior=Read-Json '驗證紀錄/20261012/native-world-layers-build-receipt.json'
$nav=Read-Json '驗證紀錄/current/native-navigation/receipt.json'
$animation=Read-Json '驗證紀錄/current/native-animation/receipt.json'
foreach($r in @($nav,$animation)){if($r.exe_sha256 -ne $sha -or $r.jar_sha256 -ne $pin.sha256){throw 'Stale performance evidence'}}
$poses=@{}
foreach($case in $prior.formal_native_animation.cases){foreach($pose in $case.poses){$poses[$case.mode+':'+$pose.elapsed_ns]=$pose.rgba_sha256}}
$changes=@()
foreach($case in $animation.cases){foreach($pose in $case.poses){$key=$case.mode+':'+$pose.elapsed_ns;if(!$poses.ContainsKey($key)){throw 'Missing historical pose'};$changes+=@{sample=$key;before=$poses[$key];after=$pose.rgba_sha256;changed=$poses[$key] -ne $pose.rgba_sha256}}}
if($changes.Count -ne 18){throw 'Missing historical animation comparisons'}
$c=$nav.cases[0].client;$old=$prior.formal_native_navigation.cases[0].client
$changes+=@{sample='route-final';before=$old.world_render.frame.rgba_sha256;after=$c.world_render.frame.rgba_sha256;changed=$old.world_render.frame.rgba_sha256 -ne $c.world_render.frame.rgba_sha256}
if(!($c.world_render.frame.fragment_composition.terrain_and_bodies) -or !($c.world_render.frame.fragment_composition.pixel_ownership)){throw 'Layered composition not active'}
$samples=@($c.navigation_render_samples)
if($samples.Count -ne $c.navigation_frames -or $samples.Count -lt 2 -or $samples.Count -gt 2048){throw 'Incomplete timings'}
if(@($samples|Where-Object{$_.prepared_terrain_bytes -le 0 -or $_.prepared_terrain_bytes -gt 67108864 -or $_.composition_frames -lt 1 -or $_.terrain_fragments -le 0 -or $_.hit_surface_bytes -ne 960000}).Count -ne 0){throw 'Fragment composition evidence invalid'}
$times=@($c.navigation_trace|Where-Object{!$_.blocked}|ForEach-Object{[double]$_.elapsed_ms});$intervals=@()
for($i=1;$i -lt $times.Count;$i++){$intervals+=($times[$i]-$times[$i-1])}
$report=[ordered]@{schema=2;status='verified_layered_composition_timings';exe_sha256=$sha;jar_sha256=$pin.sha256;animation_pixel_comparisons=0;route_pixel_comparisons=0;historical_pixel_comparisons=$changes;historical_pixel_changes=@($changes|Where-Object changed).Count;historical_pixel_identity_required=$false;reason='Terrain/body half-cell interleaving and native fragment origin replace flattened terrain then whole-sprite overlay. Historical hashes are preserved, not rebaselined.';prepared_terrain_bytes=$samples[-1].prepared_terrain_bytes;hit_surface_bytes=960000;frames=$samples.Count;frame_median_ms=(Median @($samples|ForEach-Object{$_.total/1000}));composition_median_ms=(Median @($samples|ForEach-Object{$_.terrain/1000}));hash_median_ms=(Median @($samples|ForEach-Object{$_.rgba_sha256/1000}));route_load_ms=$c.navigation_load_ms;route_step_intervals_ms=$intervals;prior_flat_exe=$prior.exe.sha256;prior_frame_median_ms=$prior.native_performance.frame_median_ms;debug_only=$true;release_exe_tested=$false;gui_workflow_tested=$false;performance_full_acceptance=$false;playable_game=$false;utc=[DateTime]::UtcNow.ToString('o')}
$report['prior_body_only_layered_exe']=$layerPrior.exe.sha256
$report['prior_body_only_frame_median_ms']=$layerPrior.native_performance.frame_median_ms
$report['prior_body_only_composition_median_ms']=$layerPrior.native_performance.composition_median_ms
$report['comparison_scope']='Current run includes actor shadows/attachments and removes redundant terrain presort; prior body-only run has different visual work. Timings are observations, not isolated optimization benchmarks.'
$actorPrior=Read-Json '驗證紀錄/20261012/native-actor-layers-build-receipt.json'
$report['prior_actor_layer_exe']=$actorPrior.exe.sha256
$report['prior_actor_layer_frame_median_ms']=$actorPrior.native_performance.frame_median_ms
$report['prior_actor_layer_composition_median_ms']=$actorPrior.native_performance.composition_median_ms
$report['comparison_scope']='Current run additionally samples source Layer5 roof groups and RGB555 transitions. Timings are observed Debug costs; no isolated optimization or full performance acceptance.'
$report|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $root '驗證紀錄/current/native-performance/receipt.json') -Encoding utf8
Write-Output 'Layered composition timings and historical pixel changes recorded'
