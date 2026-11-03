#Requires -Version 7.0
$ErrorActionPreference='Stop';$PSNativeCommandUseErrorActionPreference=$false
$root=Split-Path -Parent $PSScriptRoot
$out=Join-Path $root '驗證紀錄/current/native-real-peers'
$project=Join-Path $root 'LiTo381-Modern'
$oldTemp=$env:TEMP;$oldTmp=$env:TMP
$env:TEMP=Join-Path $project 'target/current/scratch';$env:TMP=$env:TEMP
Push-Location $project
try {
    $suites=@('native_animation','native_animation_resources','native_combat','native_game','native_lobby','native_login','native_motion','native_navigation','native_network','native_ranged','native_room','native_room_force','native_terrain_cache','native_world','rebuild_input','rebuild_wire','native_skills','native_attack_skills','native_skill_selection','native_ground_skills','native_effect_lifecycle','native_inventory','native_inventory_names','native_inventory_icons','native_item_actions','native_item_effects','native_reagent_skills','native_ground_pickup','native_world_layers','native_actor_layers','native_world_roofs','native_world_effects','native_actor_states','native_world_text','native_world_labels','native_label_updates','native_clan_markers','native_resource_loading','native_emblems')
    $args=@('test','-j','1','--offline','--locked','--target','x86_64-pc-windows-msvc','--target-dir','target/current','--bin','LiToModern')
    foreach($suite in $suites){$args+=@('--test',$suite)}
    & cargo @args *> (Join-Path $out 'rust-current.log')
    if($LASTEXITCODE -ne 0){throw 'Current Rust regression failed'}
    Write-Output 'Current Rust suites passed'
    & (Join-Path $root '啟動現代客戶端.ps1') -BuildOnly *> (Join-Path $out 'build-current.log')
    if($LASTEXITCODE -ne 0){throw 'Current Debug build failed'}
    Write-Output 'Current Debug build passed'
    foreach($name in @('room','motion','emblems','label-updates','actor-states','navigation','lobby','game','login','handshake','inventory')){
        & pwsh -NoProfile -File (Join-Path $PSScriptRoot "verify-native-$name.ps1") -SkipBuild *> (Join-Path $out "$name-current.log")
        if($LASTEXITCODE -ne 0){throw "$name regression failed"}
        Write-Output "$name current reference verified"
    }
    & pwsh -NoProfile -File (Join-Path $PSScriptRoot 'verify-native-real-peers.ps1') -SkipBuild *> (Join-Path $out 'peers-current.log')
    if($LASTEXITCODE -ne 0){throw 'Current two-peer regression failed'}
    Write-Output 'Current two-peer authority verified'
} finally {Pop-Location;$env:TEMP=$oldTemp;$env:TMP=$oldTmp}
