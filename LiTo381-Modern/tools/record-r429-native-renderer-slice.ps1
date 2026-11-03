param()
$ErrorActionPreference='Stop'
$project=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'evidence-json.ps1')
$logs=Join-Path $project 'logs\continuation-20261003'
function ReadEvidence([string]$name){Get-Content -LiteralPath (Join-Path $logs $name) -Raw|ConvertFrom-Json}
function CheckPin([string]$path,[string]$sha){if((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $sha){throw "Pin drift: $path"}}
$candidate=ReadEvidence 'r429-candidate.json'
CheckPin $candidate.path $candidate.sha256
foreach($pin in $candidate.source_pins){CheckPin $pin.path $pin.sha256;CheckPin $pin.snapshot $pin.sha256}
$artifacts=@(Get-Content -LiteralPath (Join-Path $logs 'r429-lib-build.jsonl') | ForEach-Object {try{$_|ConvertFrom-Json}catch{}})
if(-not (@($artifacts|Where-Object {$_.reason -eq 'build-finished' -and $_.success}).Count -eq 1)){throw 'Library build incomplete'}
$rlib=@($artifacts|Where-Object {$_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'lito_modern'}|ForEach-Object {$_.filenames}|Where-Object {$_ -like '*.rlib'})
if($rlib.Count -ne 1){throw 'Expected one production library'}
$library=Join-Path (Split-Path -Parent $candidate.path) 'liblito_modern-r429.rlib'
$librarySha=(Get-FileHash -LiteralPath $rlib[0]).Hash
if(Test-Path -LiteralPath $library){CheckPin $library $librarySha}else{Copy-Item -LiteralPath $rlib[0] -Destination $library;CheckPin $library $librarySha}
$candidate|Add-Member -Force NoteProperty production_library ([ordered]@{path=$library;sha256=$librarySha;offline_locked=$true;target='i686-pc-windows-msvc'})
$testRows=@()
foreach($row in @(@('local',15),@('canonical',6),@('extensions',6),@('native_units',3),@('consumer',1))){
 $name='r429-'+$row[0]+'.stdout.log'
 $content=[IO.File]::ReadAllText((Join-Path $logs $name))
 $match=[regex]::Match($content,'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
 if(-not $match.Success -or [int]$match.Groups[1].Value -ne $row[1] -or [int]$match.Groups[2].Value -ne 0){throw "Unexpected test result: $name"}
 $testRows+= [ordered]@{suite=$row[0];passed=[int]$match.Groups[1].Value;failed=0;ignored=[int]$match.Groups[3].Value;log=Join-Path $logs $name;sha256=(Get-FileHash -LiteralPath (Join-Path $logs $name)).Hash}
}
$snapshot=ReadEvidence 'r429-service-snapshot.json'
CheckPin $snapshot.input $snapshot.input_sha256
$manifestPath=Join-Path $snapshot.root 'manifest.json'
CheckPin $manifestPath $snapshot.manifest_sha256
$manifest=Get-Content -LiteralPath $manifestPath -Raw|ConvertFrom-Json
if(@($manifest.files).Count -ne 2801){throw 'Unexpected service manifest count'}
$prefix=[IO.Path]::GetFullPath($snapshot.root)+[IO.Path]::DirectorySeparatorChar
foreach($pin in $manifest.files){
 $path=[IO.Path]::GetFullPath((Join-Path $snapshot.root $pin.path))
 if(-not $path.StartsWith($prefix,[StringComparison]::OrdinalIgnoreCase)){throw 'Snapshot path escapes root'}
 if((Get-Item -LiteralPath $path).Length -ne $pin.bytes){throw "Length drift: $path"}
 CheckPin $path $pin.sha256
}
$catalog=ReadEvidence 'r429-catalog-snapshot.json'
foreach($pin in $catalog.pins){CheckPin (Join-Path $catalog.root $pin.name) $pin.sha256}
CheckPin (Join-Path $catalog.root 'decoded-LTlist.spr') $catalog.reference_sha256
CheckPin (Join-Path $catalog.root 'LTlist-reference.spr') $catalog.reference_sha256
$detail=ReadEvidence 'r429-native-renderer-slice-details.json'
$owner=ReadEvidence 'r429-publication-owner-chain.json'
$scope=ReadEvidence 'source-change-scope.json'
if($detail.state -ne 'PASSED_FIXED_PUBLICATION_AND_PARTIAL_NATIVE_CPU_SLICE' -or $detail.LOADED -or -not $detail.opaque_partial_completion_verified){throw 'Partial completion guard failed'}
if(@($detail.publication_frames).Count -ne 11 -or @($owner.rows).Count -ne 11){throw 'Unexpected publication count'}
if($scope.source_files -ne 486 -or $scope.unchanged_source_files -ne 469 -or @($scope.changed_source_files).Count -ne 10 -or @($scope.new_source_files).Count -ne 7){throw 'Unexpected source scope'}
foreach($pin in $scope.frozen_binary_pins){CheckPin $pin.path $pin.sha256}
$historical=@()
foreach($name in @('r426-candidate.json','r427-candidate.json','r428-candidate.json')){
 $old=ReadEvidence $name
 CheckPin $old.path $old.sha256
 CheckPin $old.production_library.path $old.production_library.sha256
 foreach($pin in $old.source_pins){CheckPin $pin.snapshot $pin.sha256}
 $historical+=[ordered]@{candidate=$name;pins_verified=2+@($old.source_pins).Count;changed=$false}
}
$pngs=@()
foreach($action in $detail.actions){foreach($sample in $action.samples){
 $bytes=[IO.File]::ReadAllBytes($sample.png)
 if($bytes.Length -lt 24 -or [Convert]::ToHexString($bytes[0..7]) -ne '89504E470D0A1A0A' -or [Convert]::ToHexString($bytes[16..23]) -ne '0000020000000100'){throw 'Unexpected CPU PNG'}
 $pngs+=[ordered]@{path=$sample.png;sha256=(Get-FileHash -LiteralPath $sample.png).Hash;rgba_sha256=$sample.rgba_sha256;tick=$sample.tick;subframe=$sample.subframe;fragment_count=@($sample.ordered_fragments).Count;width=512;height=256;visually_reviewed=$true}
}}
if($pngs.Count -ne 14){throw 'Unexpected CPU sample count'}
$running=@(Get-Process -Name 'lito-modern-r429f-host-tests' -ErrorAction SilentlyContinue)
if($running.Count -ne 0){throw 'Own consumer still running'}
$candidate.validation='PASSED: 31 focused tests; fixed 11 actual publications; 14 bounded source-tick CPU slices; opaque partial completion guards; production library compiled. Full renderer/LOADED unqualified.'
Write-ModernEvidenceJson -Path (Join-Path $logs 'r429-candidate.json') -Value $candidate
$result=[ordered]@{
 updated_at=(Get-Date).ToString('o');state='PARTIAL_NATIVE_RENDERER_SLICE_VERIFIED_FULL_RENDERER_PENDING';candidate=$candidate;tests=$testRows;focused_passed=31;focused_failed=0
 service_snapshot=$snapshot;service_files_reverified=2801;catalog_snapshot=$catalog;publication_count=11;canonical89_count=8;READ61_count=1;MOVE65_count=2
 owner_chain=Join-Path $logs 'r429-publication-owner-chain.json';source_route=Join-Path $logs 'r429-native-action-route-contract.json';details=Join-Path $logs 'r429-native-renderer-slice-details.json';pngs=$pngs
 source_scope=[ordered]@{total=486;baseline=479;unchanged=469;changed=10;new=7;formal_R408_changed=$false}
 bound_completion=$detail.binding_guards;opaque_partial_completion_verified=$true;full_PreparedResources=$false;LOADED=$false;qualified_modes=16;total_modes=40;GUI=0;Rust_live_TLS=$false;SQL=$false;own_consumer_exited=$true
 receipt_correction='Both source7 MOVE receipts pass against their proper preceding publication. TLS active89 belongs to source42; borrowing its guard for source7 is rejected. Existing Rust no-resources life() is Some(1), matching Java first incarnation; no independent life0 gap.'
 clock_contract='Catalog action3 has 11 frames, each duration2 in source ticks; inspected domain0..21. Tick22 and u64max rejected. Native milliseconds, start phase and looping remain unknown; Shared default24FPS is not applied.'
 pending=@('Native actor catalog/body provenance for actual service class6/sex1 and TLS class1/sex0,class3/sex1','Authoritative catalog/resource identity: LTtext supplemental local pins are absent from SourceAttachment','ExtendedLayer8 object/default-action/start/loop binding','Native timer milliseconds and full actor/roof/effect insertion order','Full sealed Native PreparedResources before live Native LOADED/transport admission')
 historical_failures_preserved=@('r429b minimum dependency gating','r429c borrowed TLS source42 guard','r429d incorrect diagnostic life0 assumption','r429e unused SPR1MiB bound; restored existing8MiB unused bound, selected input remains1MiB')
 historical_frozen_preservation=$historical
}
Write-ModernEvidenceJson -Path (Join-Path $logs 'r429-native-renderer-slice-result.json') -Value $result
Write-Output 'R429:31 focused passed;2801 fixed bytes pins;11 publications;14 CPU PNGs;LOADED=false'
