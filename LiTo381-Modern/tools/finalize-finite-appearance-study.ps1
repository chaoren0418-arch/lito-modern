param([Parameter(Mandatory)][ValidateSet('r447','r448')][string]$Stage,[Parameter(Mandatory)][string]$Revision)
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function Pin($p){[ordered]@{path=$p;sha256=(Get-FileHash -LiteralPath $p).Hash;bytes=(Get-Item -LiteralPath $p).Length}}
function Check($p){if((Get-FileHash -LiteralPath $p.path).Hash -ne $p.sha256){throw "Drift $($p.path)"};if($p.bytes -and (Get-Item -LiteralPath $p.path).Length -ne $p.bytes){throw 'Byte drift'}}
$cp=Join-Path $logs "$Stage-candidate.json";$c=Get-Content $cp -Raw|ConvertFrom-Json;Check $c
if($Stage -eq 'r448'){
 $baseline=Get-Content (Join-Path $project 'logs\relocation-baseline-20261003\source-manifest.json') -Raw|ConvertFrom-Json
 foreach($name in @('compositor.rs','world_depth.rs','world_map.rs','world_raster.rs')){
  $path=Join-Path $project "src\modern_lito\$name";$entry=$baseline.entries|Where-Object{(Split-Path $_.path -Leaf) -eq $name};if($entry.Count -ne 1 -or (Get-FileHash $path).Hash -ne $entry.sha256){throw 'Unchanged raster dependency baseline mismatch'}
  if(@($c.source_pins|Where-Object path -eq $path).Count -eq 0){$dst=Join-Path (Split-Path $c.path -Parent) "source\$name";[IO.File]::Copy($path,$dst,$false);$pin=Pin $path;$pin.snapshot=$dst;$c.source_pins=@($c.source_pins)+@($pin)}
 }
}
foreach($p in $c.source_pins){Check $p;if((Get-FileHash $p.snapshot).Hash -ne $p.sha256){throw 'Snapshot drift'}}
$build=@(Get-Content (Join-Path $logs "$Revision-library-build.jsonl")|ForEach-Object{try{$_|ConvertFrom-Json}catch{}})
if(-not @($build|Where-Object{$_.reason -eq 'build-finished' -and $_.success}).Count){throw 'Library build incomplete'}
$libs=@($build|Where-Object{$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object{$_.filenames}|Where-Object{$_ -like '*.rlib'});if($libs.Count -ne 1){throw 'Library count'}
$lib=Join-Path (Split-Path $c.path -Parent) "$Stage-production.rlib";if(!(Test-Path $lib)){[IO.File]::Copy($libs[0],$lib,$false)};if((Get-FileHash $lib).Hash -ne (Get-FileHash $libs[0]).Hash){throw 'Library copy drift'}
$c|Add-Member production_library (Pin $lib) -Force
$test=Join-Path $logs "$Revision-consumer.stdout.log";if([IO.File]::ReadAllText($test) -notmatch '1 passed; 0 failed; 0 ignored;'){throw 'Focused test failed'}
$detail=Join-Path $logs $(if($Stage -eq 'r447'){'r447-continuation-details.json'}else{'r448-layer-details.json'})
$d=Get-Content $detail -Raw|ConvertFrom-Json
$images=if($Stage -eq 'r447'){@($d.image)}else{@($d.images)}
foreach($p in $images){Check $p}
$asmPath=Join-Path $logs "$Stage-original-source-pins.json";$asm=Get-Content $asmPath -Raw|ConvertFrom-Json;Check $asm.original_disassembly
$raw=[IO.File]::ReadAllText($asm.original_disassembly.path);foreach($p in $asm.files){Check $p;if(!$raw.Contains([IO.File]::ReadAllText($p.path))){throw 'Original fragment differs'}}
& (Join-Path $PSScriptRoot 'audit-continuation-source-scope.ps1')
$scope=Get-Content (Join-Path $logs 'source-change-scope.json') -Raw|ConvertFrom-Json
$claims=if($Stage -eq 'r447'){
 if($d.rows.Count -ne 48 -or $d.late_callback_cases -ne 2 -or $d.queued_idle_callback_executed -or $d.renderer_LOADED){throw 'Continuation scope'}
 @('Eight headings,48 continuing/end steps of complete historical Type3 weapon2 attack47; frames2..6 then reset0.','4E34C0 in-range computes actual callback sampledNow+max(1,duration-lateness)+2 and separately expected+=duration+2. Two lateness cases include delay3/due2339; clocks are not merged.','End resetsframe0 and returns83ms,then 4E02C0 under explicit queue0/nonrenderplayer/counter0/base46/idle49 table present/queue allocation success queues opcode32/action49 with delay584. Theoretical expected clock is not advanced on this end branch.','No idle49 callback is executed; no loop or visible-residency claim. Missing/wrong queue/pointer/counter/base/allocation conditions refuse.','CPU strip selected frames1,2,3,4,5,6,0 uses two actual pinned SPR members. All7 samples viewed; source offsets recorded, contact-sheet centering is inspection placement.')
}else{
 if($d.neutral_CPU_samples -ne 48 -or $d.source_rows.Count -ne 48 -or $d.bit2_skip_submission_cases -ne 48 -or $d.images.Count -ne 2 -or $d.renderer_LOADED -or $d.native_framebuffer_parity_qualified -or $d.runtime_inputs_observed){throw 'Layer scope'}
 @('Finite original4E0200 body→105 list order→shadow submission, same directed image bank and actor+19 subframe.','4E6EA0 layer pointer missing disables entire105 count; 4E7000 explicit conditional actorflags0/layerflags0 preserve layer flags, bit2 skips105 submission. Unknown selected flags/pointer/source/shadow route are refused; neither flag0 is claimed as original runtime observation.','4CD9D0 actual signed half-cell references/depth feed483770 ascending depth/newest-first equal-depth;488860 paints scan rows/columns. Original call order is not a whole-sprite paint order.','48neutral CPU samples+48bit2 non-submission cases. 24actual SPR resources/4catalog-binding inputs verified against immutable manifest; all selected105 frames have zero fragments. Blank105 panel is observed resource data, not a synthesized substitute.','Each heading first sample compares all pixels to an independent literal native scan/depth/newest-first reference compositor; first heading17fragment ledger retained.','CPU uses actual world-decoded body/shadow/105 members and source fragment coordinates/depth, explicit fixed anchor and RGB555 reference shadow shader. Dynamic world/terrain/insertion history/framebuffer565/player events remain unqualified.')
}
$c.validation='PASSED one new focused finite consumer; immutable source/host/library/ASM/resource/output evidence; default-off conditional study only'
Write-ModernEvidenceJson -Path $cp -Value $c
$preservedNames=@('r443','r444','r445');if($Stage -eq 'r448'){$preservedNames+=@('r447')}
$historical=@(foreach($name in $preservedNames){$p=Join-Path $logs "$name-candidate.json";$h=Get-Content $p -Raw|ConvertFrom-Json;Check $h;foreach($s in $h.source_pins){if((Get-FileHash $s.snapshot).Hash -ne $s.sha256){throw 'Historical snapshot drift'}};if($h.production_library){Check $h.production_library};[ordered]@{record=Pin $p;source_snapshots=$h.source_pins.Count;preserved=$true}})
$result=Join-Path $logs "$Stage-finite-study-result.json"
Write-ModernEvidenceJson -Path $result -Value ([ordered]@{state='PASSED_FINITE_CONDITIONAL_SOURCE_AND_ACTUAL_RESOURCE_CPU_STUDY';updated_at=[DateTimeOffset]::Now.ToString('o');stage=$Stage;candidate=$c;test=Pin $test;focused_passed=1;focused_failed=0;details=Pin $detail;original_source_contract=Pin $asmPath;claims=$claims;images=$images;visual_reviewed=$true;historical_preserved=$historical;source_files=$scope.source_files;default_enabled=$false;runtime_inputs_observed=$false;live_appearance_qualified=$false;events_dispatched=$false;natural_loop_qualified=$false;renderer_LOADED=$false;GUI=0;mode_coverage='16/40';R408_changed=$false;R441_R442_R445_live_rerun=$false})
$handoff=Join-Path $logs $(if($Stage -eq 'r447'){'Native-appearance-R447原版有限續步與結束交接.txt'}else{'Native-appearance-R448原版shadow105有限CPU交接.txt'})
[IO.File]::WriteAllText($handoff,($claims -join "`r`n")+"`r`nResult: $result`r`nSources: $($c.source_pins.Count); original fragments: $($asm.files.Count). LOADEDfalse/GUI0/16of40. Live Class/override owner qualification pending; historical Class is offline only. All original binaries untouched; failed/compile-only attempts retained.",[Text.UTF8Encoding]::new($false))
Write-ModernEvidenceJson -Path (Join-Path $logs "$Stage-final-verification.json") -Value ([ordered]@{result=Pin $result;handoff=Pin $handoff;candidate=Pin $cp;details=Pin $detail;host=Pin $c.path;library=Pin $lib;images=$images;visual_reviewed=$true;source_snapshots=$c.source_pins.Count;original_fragments=$asm.files.Count;source_files=$scope.source_files;renderer_LOADED=$false;GUI=0})
$statusPath=Join-Path (Split-Path $project -Parent) '監製進度.json';$status=Get-Content $statusPath -Raw|ConvertFrom-Json
$status.updated_at=[DateTimeOffset]::Now.ToString('o');$status.phase="$($Stage)_finite_conditional_appearance_CPU_passed_live_qualification_pending"
$status.current_task="$Stage new finite focused test passed; $($c.source_pins.Count) source snapshots, host/lib, $($asm.files.Count) complete original ASM fragments and CPU outputs pinned/visually reviewed. R446 minimum requirements recorded;R443/444/445 preserved; no old live suite rerun. $($claims -join ' ') Source files:$($scope.source_files). R408 unchanged;LOADEDfalse/GUI0/16of40."
Write-ModernEvidenceJson -Path $statusPath -Value $status
Write-Output "$Stage complete: source/host/library/ASM/actual resources/CPU images pinned; live qualification remains false"
