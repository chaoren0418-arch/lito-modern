#Requires -Version 7.0
# One candidate EXE/reference, immutable receipt copies and explicit test targets.
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$taskRoot=Split-Path -Parent $PSScriptRoot;$project=Join-Path $taskRoot 'LiTo381-Modern'
$out=Join-Path $taskRoot ('驗證紀錄/current/candidate-regression/'+[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss-fff'))
New-Item -ItemType Directory -Path $out -Force|Out-Null
$pin=Get-Content -Raw (Join-Path $taskRoot '驗證紀錄/current/reference-snapshot.json')|ConvertFrom-Json
$exe=Join-Path $project 'target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe'
$exeSha=(Get-FileHash -LiteralPath $exe).Hash
$inputFiles=@()
foreach($folder in @('LiTo381-Modern/src','LiTo381-Modern/tests','維護工具')){
    $inputFiles+=Get-ChildItem -LiteralPath (Join-Path $taskRoot $folder) -Recurse -File | Where-Object {$_.Extension -in @('.rs','.java','.json','.py','.ps1','.html')}
}
foreach($name in @('LiTo381-Modern/Cargo.toml','LiTo381-Modern/Cargo.lock','LiTo381-Modern/build.rs','啟動現代客戶端.ps1')){
    $path=Join-Path $taskRoot $name;if(Test-Path -LiteralPath $path){$inputFiles+=Get-Item -LiteralPath $path}
}
$inputPins=[ordered]@{}
foreach($file in $inputFiles|Sort-Object FullName -Unique){$inputPins[[IO.Path]::GetRelativePath($taskRoot,$file.FullName).Replace('\','/')]=(Get-FileHash -LiteralPath $file.FullName).Hash}
@{exe_sha256=$exeSha;files=$inputPins}|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $out 'development-input-pins.json') -Encoding utf8
$index=[ordered]@{status='running';exe_sha256=$exeSha;jar_sha256=$pin.sha256;output=$out;single_cases=@();peer_profiles=@();recovery_modes=@();boundary_observations=@()}
function SaveIndex {$index|ConvertTo-Json -Depth 20|Set-Content (Join-Path $out 'index.json') -Encoding utf8}
function Stable {
    if((Get-FileHash -LiteralPath $exe).Hash -ne $exeSha -or (Get-FileHash -LiteralPath $pin.source).Hash -ne $pin.sha256 -or (Get-FileHash -LiteralPath $pin.snapshot).Hash -ne $pin.sha256){throw 'Candidate/reference changed'}
}
function RunScript([string]$Name,[string[]]$Options,[string]$Log){
    Stable
    & pwsh -NoProfile -File (Join-Path $PSScriptRoot $Name) -SkipBuild @Options *> (Join-Path $out $Log)
    if($LASTEXITCODE -ne 0){throw ($Name+' failed: '+(Join-Path $out $Log))}
    Stable
}
function Latest([string]$Folder,[string]$Key,[string]$Value,[switch]$Compare){
    $found=$null
    foreach($file in Get-ChildItem -LiteralPath (Join-Path $taskRoot ('驗證紀錄/current/'+$Folder)) -Filter receipt.json -Recurse | Sort-Object FullName){
        $r=Get-Content -Raw -LiteralPath $file.FullName|ConvertFrom-Json
        $markerPath=Join-Path $file.DirectoryName 'validation-completed.json'
        if(Test-Path -LiteralPath $markerPath){
            $binding=Get-Content -LiteralPath $markerPath -Raw|ConvertFrom-Json
            if($binding.status -ne 'verified_owned_peer_comparison_bindings' -or $binding.receipt_sha256 -ne (Get-FileHash -LiteralPath $file.FullName).Hash){continue}
            $bindingValid=$true
            foreach($entry in $binding.comparison_pins.PSObject.Properties){
                $comparisonPath=Join-Path $file.DirectoryName $entry.Name
                if(!(Test-Path -LiteralPath $comparisonPath) -or (Get-FileHash -LiteralPath $comparisonPath).Hash -ne $entry.Value){$bindingValid=$false;break}
            }
            if(!$bindingValid){continue}
        }elseif(!$r.validation_completed){continue}
        $v=if($Key -eq 'action_profile' -and !$r.action_profile){'melee'}else{$r.$Key}
        if($r.exe_sha256 -ne $exeSha -or $r.jar_sha256 -ne $pin.sha256 -or $v -ne $Value){continue}
        if($Compare){
            $comparisonName=switch($Value){'room-admin'{'room-admin-comparison.json'} 'classic-entry'{'classic-entry-comparison.json'} 'rental-entry'{'rental-entry-comparison.json'} 'rental-shop'{'rental-entry-comparison.json'} 'rental-citizen'{'rental-citizen-comparison.json'} 'rental-hotel'{'rental-hotel-comparison.json'} 'rental-hotel-entry'{'rental-hotel-entry-comparison.json'} 'rental-hotel-middle-entry'{'rental-hotel-entry-comparison.json'} 'rental-hotel-share'{'rental-hotel-share-comparison.json'} 'rental-merchants'{'rental-merchants-comparison.json'} 'rental-weight'{'rental-weight-comparison.json'} 'rental-return'{'rental-return-comparison.json'} default{'authority-comparison.json'}}
            $comparison=Join-Path $file.DirectoryName $comparisonName
            if(!(Test-Path -LiteralPath $comparison)){continue}
            $a=Get-Content -Raw -LiteralPath $comparison|ConvertFrom-Json
            if($a.receipt_sha256 -ne (Get-FileHash -LiteralPath $file.FullName).Hash){continue}
            if($Value -like 'rental-*'){$attributes=Join-Path $file.DirectoryName 'attributes-comparison.json';if(!(Test-Path -LiteralPath $attributes)){continue};$ar=Get-Content -Raw -LiteralPath $attributes|ConvertFrom-Json;if($ar.receipt_sha256 -ne (Get-FileHash -LiteralPath $file.FullName).Hash){continue}}
        }
        if($Key -eq 'action_profile' -and $Value -in @('movement','rental-shop')){
            $miniPath=Join-Path $file.DirectoryName 'minimap-comparison.json'
            if(!(Test-Path -LiteralPath $miniPath)){continue}
            $mini=Get-Content -LiteralPath $miniPath -Raw|ConvertFrom-Json
            if($mini.receipt_sha256 -ne (Get-FileHash -LiteralPath $file.FullName).Hash){continue}
        }
        $found=$file.FullName
    }
    return $found
}
$oldTemp=$env:TEMP;$oldTmp=$env:TMP;$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP
SaveIndex
Push-Location $project
try{
    Stable
    $suites=@('native_animation','native_animation_resources','native_combat','native_game','native_lobby','native_login','native_motion','native_navigation','native_network','native_ranged','native_room','native_room_force','native_room_departures','native_room_mode','native_doors','native_obstacles','native_world_metadata','native_lifecycle','native_terrain_cache','native_world','rebuild_input','rebuild_wire','native_skills','native_attack_skills','native_fire_arrow','native_wind_cutter','native_skill_selection','native_ground_skills','native_effect_lifecycle','native_inventory','native_inventory_names','native_inventory_icons','native_item_actions','native_item_effects','native_reagent_skills','native_ground_pickup','native_world_layers','native_actor_layers','native_world_roofs','native_world_effects','native_actor_states','native_world_text','native_world_labels','native_label_updates','native_clan_markers','native_resource_loading','native_emblems','native_room_admin','native_classic_entry','native_services','native_points','native_city_resources','native_merchant','native_attributes','native_room_ui','native_npc_document','native_rental_routes','native_npc_input','native_portal','native_minimap')
    $cargoArgs=@('test','-j','1','--offline','--locked','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--bin','LiToModern')
    foreach($suite in $suites){$cargoArgs+=@('--test',$suite)}
    & cargo @cargoArgs *> (Join-Path $out 'rust.log')
    if($LASTEXITCODE -ne 0){throw 'Candidate Rust suites failed'}
    $text=Get-Content -Raw (Join-Path $out 'rust.log')
    $matches=[regex]::Matches($text,'test result: ok\. (\d+) passed; (\d+) failed;')
    $passed=0;foreach($m in $matches){$passed+=[int]$m.Groups[1].Value;if([int]$m.Groups[2].Value -ne 0){throw 'Rust failure'}}
    if($matches.Count -ne 61 -or $passed -ne 269){throw ('Unexpected Rust scope: '+$matches.Count+' suites / '+$passed)}
    $index.rust=@{passed=$passed;failed=0;suites=$matches.Count;log_sha256=(Get-FileHash (Join-Path $out 'rust.log')).Hash};SaveIndex
    Write-Output ('Verified Rust '+$passed+' tests')
    $stages=@()
    foreach($name in @('room','motion','emblems','label-updates','actor-states','navigation','lobby','game','login','handshake','inventory')){
        $stages+=@{Name=$name;Script="verify-native-$name.ps1";Options=@()}
    }
    $stages+=@(
        @{Name='item-actions-inventory';Script='verify-native-item-actions.ps1';Options=@('-InventoryOnly')},
        @{Name='combat-self';Script='verify-native-combat.ps1';Options=@('-SelfOnly')},
        @{Name='item-actions-ground';Script='verify-native-item-actions.ps1';Options=@('-GroundOnly')},
        @{Name='reagent-skills';Script='verify-native-reagent-skills.ps1';Options=@()},
        @{Name='ground-pickup';Script='verify-native-ground-pickup.ps1';Options=@()}
    )
    foreach($stage in $stages){
        RunScript $stage.Script $stage.Options ($stage.Name+'.log')
        $source=Join-Path $taskRoot ('驗證紀錄/current/native-'+$stage.Name+'/receipt.json')
        $r=Get-Content -Raw -LiteralPath $source|ConvertFrom-Json
        if($r.exe_sha256 -ne $exeSha -or $r.jar_sha256 -ne $pin.sha256 -or $r.cases.Count -lt 1){throw 'Scenario receipt mismatch'}
        $dest=Join-Path $out ($stage.Name+'-receipt.json');Copy-Item -LiteralPath $source -Destination $dest
        $index.single_cases+=@{group=$stage.Name;count=$r.cases.Count;receipt=$dest;sha256=(Get-FileHash $dest).Hash};SaveIndex
        Write-Output ('Verified '+$stage.Name+' '+$r.cases.Count+' cases')
    }
    foreach($profile in @('melee','heal','bolt','bolt-refused','fire-arrow','fire-arrow-refused','wind-cutter','wind-cutter-refused','wall','wall-refused','ranged','item-heal','doors','movement','honor-respawn','room-admin','rental-entry','rental-shop','rental-citizen','rental-hotel','rental-hotel-entry','rental-hotel-middle-entry','rental-hotel-share','rental-merchants','rental-weight')){
        $receipt=Latest 'native-real-peers' 'action_profile' $profile -Compare
        if(!$receipt){RunScript 'verify-native-real-peers.ps1' @('-ActionProfile',$profile) ('peer-'+$profile+'.log');$receipt=Latest 'native-real-peers' 'action_profile' $profile -Compare}
        if(!$receipt){throw ('Peer receipt missing '+$profile)}
        $index.peer_profiles+=@{profile=$profile;receipt=$receipt;sha256=(Get-FileHash -LiteralPath $receipt).Hash};SaveIndex
        Write-Output ('Verified peer '+$profile)
    }
    # Negative/partial boundary observations are counted separately from successful gameplay scenarios.
    foreach($profile in @('classic-entry','rental-return')){
        $receipt=Latest 'native-real-peers' 'action_profile' $profile -Compare
        if(!$receipt){RunScript 'verify-native-real-peers.ps1' @('-ActionProfile',$profile) ('boundary-'+$profile+'.log');$receipt=Latest 'native-real-peers' 'action_profile' $profile -Compare}
        if(!$receipt){throw ('Boundary observation receipt missing '+$profile)}
        $index.boundary_observations+=@{profile=$profile;receipt=$receipt;sha256=(Get-FileHash -LiteralPath $receipt).Hash};SaveIndex
        Write-Output ('Recorded boundary '+$profile+'; not a successful transition claim')
    }
    foreach($mode in @('room','world')){
        $receipt=Latest 'native-recovery' 'mode' $mode
        if(!$receipt){RunScript 'verify-native-recovery.ps1' @('-Mode',$mode) ('recovery-'+$mode+'.log');$receipt=Latest 'native-recovery' 'mode' $mode}
        if(!$receipt){throw ('Recovery receipt missing '+$mode)}
        $index.recovery_modes+=@{mode=$mode;receipt=$receipt;sha256=(Get-FileHash -LiteralPath $receipt).Hash};SaveIndex
        Write-Output ('Verified recovery '+$mode)
    }
    Stable
    foreach($entry in $inputPins.GetEnumerator()){
        if((Get-FileHash -LiteralPath (Join-Path $taskRoot $entry.Key)).Hash -ne $entry.Value){throw ('Development input changed: '+$entry.Key)}
    }
    $index.development_input_files_verified=$inputPins.Count
    $index.development_input_pins_sha256=(Get-FileHash -LiteralPath (Join-Path $out 'development-input-pins.json')).Hash
    $count=0;foreach($row in $index.single_cases){$count+=$row.count}
    if($count -ne 43 -or $index.peer_profiles.Count -ne 25 -or $index.recovery_modes.Count -ne 2 -or $index.boundary_observations.Count -ne 2){throw 'Incomplete candidate scope'}
    $index.status='verified_current_candidate_subset';$index.tcp_scenarios=$count+$index.peer_profiles.Count+$index.recovery_modes.Count;$index.observed_boundary_cases=$index.boundary_observations.Count;$index.gui_verified=$false;$index.product_complete=$false;SaveIndex
    Write-Output ('Verified one candidate: '+$out)
}catch{$index.status='failed';$index.error=$_.Exception.Message;SaveIndex;throw}
finally{Pop-Location;$env:TEMP=$oldTemp;$env:TMP=$oldTmp}
