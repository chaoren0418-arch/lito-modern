#Requires -Version 7.0
# Internal pinned acceptance slice. The default is headless; an explicitly
# requested InteractiveWindow run opens only the owned canonical test HWND.
# Never builds, deploys, starts an original BIN or uses production accounts.
[CmdletBinding()]
param(
    [switch]$PlanOnly,
    [switch]$CheckOnly,
    [switch]$InteractiveWindow,
    [switch]$InteractiveEquipment,
    [switch]$ShowTestOutput,
    [string]$CaptureDirectory,
    [string[]]$Case = @(),
    [string]$TestExe,
    [string]$TestExeSha256,
    [string]$ProjectRoot,
    [string]$ServerRoot,
    [string]$SharedRoot,
    [string]$JavaExe,
    [string[]]$JavaTestJars = @(),
    [string]$PackagedServerJar,
    [string]$PackagedServerJarSha256,
    [string]$MysqlHome,
    [string]$OutputRoot
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$plan = @(
    @{Name='extended-healing-tls'; Filter='java_extended_tls_negotiates_healing_without_auth_or_legacy_downgrade'; Count=1}
    @{Name='legacy-tls-lifecycle'; Filter='modern_lito::transport::tests::java_tls_'; Count=10}
    @{Name='candidate-player-healing-command'; Filter='java_player_healing_rust_intents_execute_and_replies_commit_atomically'; Count=1}
    @{Name='candidate-player-healing-scene'; Filter='java_healing_schema7_complete_scene_atomic_follower_and_catchup'; Count=1}
    @{Name='selected-player-healing-observation'; Filter='java_healing_selected_complete_observation_and_atomic_catchup'; Count=1}
    @{Name='selected-player-healing-replies'; Filter='java_healing_selected_all_reply_families_preserve_complete_data'; Count=1}
    @{Name='selected-player-healing-runtime'; Filter='java_healing_runtime_selected_tls_preserves_full_state_through_shared_load_and_leave'; Count=1}
    @{Name='selected-player-healing-cast'; Filter='java_healing_runtime_two_owner_attack_heal_pending_and_leave'; Count=1}
    @{Name='selected-player-healing-main-sql'; Filter='java_healing_main_sql_ordinary_'; Count=4}
    @{Name='canonical-healing-presentation'; Filter='confirmed_healing_uses_shared_action19_fixed744_and_deduplicated109'; Count=1}
    @{Name='candidate-player-healing-battle'; Filter='java_complete_healing_battle_schema7_accounts_status_and_owner_round_trip'; Count=1}
    @{Name='candidate-player-healing-owner'; Filter='java_player_healing_owner_schema5_real_engine_cross_source_gear_life_and_effects_round_trip'; Count=1}
    @{Name='same-main-mixed-modes'; Filter='java_v47_main_sql_same_connections_six_modes_retire_and_reload'; Count=1}
    @{Name='same-main-mixed-results'; Filter='java_v47_main_sql_same_connections_alternating_winners_and_mode_results'; Count=1}
    @{Name='same-main-mixed-quest-results'; Filter='java_v47_main_sql_same_connections_pvp_quest_victories_and_source_reload'; Count=1}
    @{Name='same-main-mixed-objectives'; Filter='java_v47_main_sql_same_connections_pvp_honor_time_quest_results'; Count=1}
    @{Name='same-main-all-results'; Filter='java_v47_main_sql_same_connections_all_six_modes_saved_and_source_reload'; Count=1}
    @{Name='same-main-all-results-expired-vip'; Filter='java_v47_main_sql_same_connections_all_six_modes_saved_and_source_reload'; Count=1; ExpiredVip=$true}
    @{Name='ordinary-v47-modes'; Filter='java_ordinary_v47_'; Count=7}
    @{Name='canonical-map-509'; Filter='java_v42_shared_map_rotation_509_514_509'; Count=1}
    @{Name='canonical-map-552'; Filter='java_v42_shared_map_rotation_552_551_552'; Count=1}
    @{Name='canonical-map-531'; Filter='java_v42_shared_map_rotation_531_508_531'; Count=1}
    @{Name='canonical-map-534'; Filter='java_v42_shared_map_rotation_534_531_534'; Count=1}
    @{Name='canonical-map-516'; Filter='java_v42_shared_map_rotation_516_531_516'; Count=1}
    @{Name='canonical-map-504'; Filter='java_v42_shared_map_rotation_504_531_504'; Count=1}
    @{Name='canonical-map-505'; Filter='java_v42_shared_map_rotation_505_531_505'; Count=1}
    @{Name='canonical-map-506'; Filter='java_v42_shared_map_rotation_506_531_506'; Count=1}
    @{Name='legacy-map-509'; Filter='java_shared_509_514_509_three_saved_rounds_same_owners_and_renderers'; Count=1}
    @{Name='legacy-map-552'; Filter='java_shared_552_551_552_three_saved_rounds_same_owners_and_renderers'; Count=1}
    @{Name='shared-greater-haste'; Filter='real_shared_greater_haste_effect_frames_and_terminal'; Count=1}
    @{Name='shared-slow29'; Filter='real_shared_slow29_effect_frames_and_terminal'; Count=1}
    @{Name='shop-selected-icon'; Filter='actual_shop_selected_material_icon_loads_after_catalog_preload_and_retires'; Count=1}
    @{Name='canonical-speed-command-visual'; Filter='real_speed_commands_and_slow752_visuals_preserve_existing_haste'; Count=1}
    @{Name='canonical-slow-controller-sql'; Filter='java_v47_main_sql_slow_skillbook_materials_cast_move_and_source_return'; Count=1}
    @{Name='canonical-slow-quest-sql'; Filter='java_v47_main_sql_slow_npc_bot_quest_materials_saved_results_and_return'; Count=1}
    @{Name='shared-haste43'; Filter='real_shared_haste43_effect_frames_and_terminal'; Count=1}
    @{Name='canonical-speed46-wire'; Filter='java_speed46_negotiated_read_and_atomic_observation_following'; Count=1}
    @{Name='candidate-speed29-wire'; Filter='java_speed29_schema6_lossless_components_and_old_versions_closed'; Count=1}
    @{Name='candidate-haste43-wire'; Filter='java_haste43_schema5_complete_targets_owner_history_and_legacy_rejection'; Count=1}
    @{Name='canonical-haste43-time-wire'; Filter='java_haste43_time_attack_schema5_all_terminals_and_owner_transitions'; Count=1}
    @{Name='canonical-haste44-tls'; Filter='java_haste44_authenticated_tls_two_owners_catchup_replay_and_owner_rejection_are_atomic'; Count=1}
    @{Name='canonical-haste43-command'; Filter='java_target_haste_command_pc_bot_npc_echo_denial_and_atomic_history'; Count=1}
    @{Name='canonical-haste43-visual'; Filter='java_haste43_shared_target_visual_and_npc_motion'; Count=1}
    @{Name='canonical-haste43-controller'; Filter='java_v44_target_haste_controllers_scoped_input_shared_cast_move_and_return'; Count=1}
    @{Name='canonical-greater-haste-wire'; Filter='java_greater_haste_schema3_source_cost_gear_life_and_legacy_rejection'; Count=1}
    @{Name='canonical-greater-haste-command'; Filter='java_greater_haste_command_echo_success_denial_and_atomic_candidate_history'; Count=1}
    @{Name='canonical-payload-wire'; Filter='java_complete_battle_payloads_preserve_sections_owner_and_engine_revision'; Count=1}
    @{Name='candidate-npc-motion-wire'; Filter='java_npc_motion_schema4_scoped_pace_steps_bounds_and_legacy_rejection'; Count=1}
    @{Name='shared-npc-pacing'; Filter='real_shared_npc_paced_cycles_and_layers'; Count=1}
    @{Name='shared-npc-assets'; Filter='actual_shared_quest_npc_complete_asset_bank'; Count=1}
    @{Name='npc-cadence'; Filter='java_v43_coalesced_npc_walk_and_attack_audio_share_confirmed_clock'; Count=1}
    @{Name='npc-cadence-tls'; Filter='java_v43_controllers_shared_npc_cadence_render_audio_and_return'; Count=1}
    @{Name='npc-pursuit-tls'; Filter='java_v43_controllers_shared_distant_npc_pursuit_render_audio_and_return'; Count=1}
    @{Name='npc-dynamic-pursuit-tls'; Filter='java_v43_controllers_shared_moving_targets_npc_pursuit_and_return'; Count=1}
    @{Name='npc-low-quota-pursuit-tls'; Filter='java_v43_controllers_shared_low_quota_moving_targets_npc_pursuit_and_return'; Count=1}
    @{Name='canonical-sequence-wire'; Filter='java_canonical_sequences_allow_catchup_and_reject_rewrites_atomically'; Count=1}
    @{Name='shared-player-walk'; Filter='real_shared_player_walk_all_qualified_weapon_families_use_native_step_order'; Count=1}
    @{Name='shared-player-footsteps'; Filter='real_shared_player_footsteps_match_all_qualified_walk_boundaries_and_pcm'; Count=1}
    @{Name='shared-player-armed-art'; Filter='real_shared_armed_families_preload_and_knight_pixels_match_independent_csharp_goldens'; Count=1}
    @{Name='canonical-input'; Filter='java_canonical_modes_qualify_exact_navigation_without_load_permission'; Count=1}
    @{Name='canonical-revival-wire'; Filter='java_v40_revival_wire_and_atomic_follower_preserve_exact_death_and_new_life'; Count=1}
    @{Name='canonical-equipment-wire'; Filter='modern_lito::battle_equipment::tests::java_v41_equipment_'; Count=2}
    @{Name='canonical-revival-controller'; Filter='java_v40_controllers_shared_death_revival_move_abort_return_and_shutdown'; Count=1}
    @{Name='canonical-npc-motion-revival'; Filter='java_v43_controllers_npc_motion_death_revival_move_abort_return'; Count=1}
    @{Name='canonical-controller'; Filter='java_v39_controller_shared_load_move_kill_saved_navigation_and_shutdown'; Count=1}
    @{Name='v37-disintegrate'; Filter='java_v37_disintegrate_controller_guarded_kill_saved_return_with_v27_peer'; Count=1}
    @{Name='legacy-controller'; Filter='java_disintegrate_player_controller_catalog_input_kill_record_return_and_shutdown'; Count=1}
    @{Name='canonical-tls'; Filter='modern_lito::world_integration_tests::battle_read_interop_tests::'; Exclude='::live_slow_sql::'; Count=12}
    @{Name='canonical-haste-controller'; Filter='java_v42_greater_haste_controllers_scoped_input_shared_cast_move_and_return'; Count=1}
    @{Name='canonical-haste-sql'; Filter='java_v42_main_sql_greater_haste_wide_stats_live_gear_cast_move_and_source_return'; Count=1}
    @{Name='canonical-target-haste-sql'; Filter='java_v44_main_sql_target_haste_wide_stats_live_gear_cast_move_and_source_return'; Count=1}
    @{Name='canonical-slow-sql'; Filter='java_v47_slow_main_sql_materials_cast_atomic_state_and_source_return'; Count=1}
    @{Name='canonical-target-haste-quest-sql'; Filter='java_v44_main_sql_target_haste_npc_bot_quest_saved_results_and_return'; Count=1}
    @{Name='canonical-target-haste-time-sql'; Filter='java_v45_main_sql_time_attack_haste_'; Count=2}
    @{Name='canonical-equipment-controller'; Filter='java_v41_equipment_controller_inventory_atomic_render_move_and_return'; Count=1}
    @{Name='canonical-supplies-sql'; Filter='java_v41_sql_supplies_rendered_use_saved_results_shop_return_and_source_reload'; Count=1}
    @{Name='canonical-inventory-sql'; Filter='java_v42_inventory_'; Count=2}
    @{Name='canonical-inventory-render-lag'; Filter='java_v42_delayed_inventory_receiver_catches_up_before_use_and_saved_return'; Count=1}
    @{Name='canonical-inventory-render-pace'; Filter='java_v42_continuously_slow_inventory_receiver_uses_saves_and_returns'; Count=1}
    @{Name='canonical-matchmaking-sql'; Filter='java_v41_sql_matchmaking_shop_countdown_fire_saved_results_and_return'; Count=1}
    @{Name='canonical-teleport-sql'; Filter='java_v41_sql_teleport_shared_camera_input_recovery_and_abort_return'; Count=1}
    @{Name='canonical-gm-bots-sql'; Filter='java_v41_sql_gm_text_bots_single_human_capacity_authority_combat_saved_return'; Count=1}
    @{Name='canonical-siege-sql'; Filter='java_v41_sql_siege_shared_kent_route_portal_towers_saved_results_and_return'; Count=1}
    @{Name='canonical-honor-sql'; Filter='java_v41_honor_controllers_shared_heal_respawn_towers_saved_hud_and_return'; Count=1}
    @{Name='canonical-time-attack-sql'; Filter='java_v43_sql_time_attack_'; Count=2}
    @{Name='canonical-pet-race-sql'; Filter='java_v41_sql_pet_race_four_laps_ranked_saved_hud_and_return'; Count=1}
    @{Name='canonical-pet-race-withdraw'; Filter='java_v41_sql_pet_race_creator_withdraw_survivor_saved_and_return'; Count=1}
    @{Name='canonical-pet-race-disconnect'; Filter='java_v41_sql_pet_race_controller_disconnect_survivor_saved_and_return'; Count=1}
    @{Name='legacy-pet-race-sql'; Filter='java_pet_race_four_ordered_laps_real_window_saved_and_retire'; Count=1}
    @{Name='legacy-pet-race-withdraw'; Filter='java_pet_race_world_withdraw_does_not_abort_other_runner'; Count=1}
    @{Name='legacy-pet-race-disconnect'; Filter='java_pet_race_true_tls_disconnect_does_not_abort_other_runner'; Count=1}
    @{Name='legacy-time-attack77-sql'; Filter='java_npc77_time_attack_sql_shared_casters_source_costs_kill_settlement_and_return'; Count=1}
    @{Name='legacy-gm-bots-sql'; Filter='java_quest_sql_gm_bots_capacity_teams_sparse_magic_saved_records_and_return'; Count=1}
    @{Name='lobby-supplies-sql'; Filter='java_quest_sql_supplies_buy_sell_unready_heal_render_and_reset'; Count=1}
    @{Name='matchmaking-controller'; Filter='java_matchmaking_controllers_queue_cancel_match_balanced_sources_manual_room_and_reentry'; Count=1}
    @{Name='matchmaking-sql'; Filter='java_matchmaking_sql_shared_countdown_battle_saved_results_and_return'; Count=1}
    @{Name='navigation-channels'; Filter='java_application_navigation_cycles_all_four_channels_and_clears_server_presence_on_disconnect'; Count=1}
    @{Name='world-wire'; Filter='java_v39_passive_world_mixed_equipment_saved_and_departure'; Count=1}
    @{Name='operation-wire'; Filter='java_v38_operation_mixed_equipment_history_and_saved_receipt'; Count=1}
    @{Name='service-gear'; Filter='java_service_equipment_history_reaches_both_owners_and_terminal_without_legacy_frames'; Count=1}
    @{Name='durable-gear'; Filter='java_durable_equipment_changes_follow_both_owners_without_repricing_past_events'; Count=1}
    @{Name='scene-motion'; Filter='java_canonical_race_and_target_changes_animate_real_shared_resources'; Count=1}
    @{Name='body-audio'; Filter='java_canonical_body_cues_share_pixel_clocks_and_remain_atomic'; Count=1}
    @{Name='quest-npc70'; Filter='java_quest_npc70_sql_shared_sparse_waves_objective_saved_records_and_source_reload'; Count=1}
    @{Name='time-npc70'; Filter='java_npc70_time_attack_sql_shared_tower_casters_departure_saved_records_and_source_reload'; Count=1}
    @{Name='v37-fire'; Filter='java_v37_fire_storm_guarded_move_empty_cast_cooldown_pc_bot_multikill_and_saved_sql'; Count=1}
    @{Name='legacy-fire'; Filter='java_fire_storm_empty_cast_pc_bot_atomic_multikill_saved_sql'; Count=1}
    @{Name='npc4'; Filter='java_npc4_time_attack_sql_shared_kills_saved_result_departure_and_return'; Count=1}
    @{Name='npc77'; Filter='java_quest_npc77_sql_sparse_waves_death_revival_saved_records_and_return'; Count=1}
    @{Name='teleport'; Filter='java_quest_sql_teleport_sparse_cast_shared_renderer_move_and_normal_return'; Count=1}
    @{Name='v37-ammo'; Filter='java_v37_guarded_bow_supplies_ammo_damage_terminal_recovery_and_saved_sql'; Count=1}
    @{Name='siege-sql'; Filter='modern_lito::world_integration_tests::quest_sql_interop_tests::siege_sql_interop_tests::'; Count=2}
    @{Name='v37-life'; Filter='java_v37_honor_delayed_target_respawn_rejection_then_fresh_attack_heal_and_saved_sql'; Count=1}
)
if ($InteractiveWindow -and $InteractiveEquipment) { throw 'Select only one interactive scenario' }
if ($InteractiveEquipment) {
    if ($Case.Count -ne 0) { throw 'InteractiveEquipment cannot mix with headless Case selection' }
    $plan = @(@{Name='equipment-window'; Filter='java_v41_equipment_window_inventory_move_return_and_close'; Count=1})
} elseif ($InteractiveWindow) {
    if ($Case.Count -ne 0) { throw 'InteractiveWindow cannot mix with headless Case selection' }
    $plan = @(@{Name='canonical-window'; Filter='java_canonical_player_window_move_cast_saved_return_and_close'; Count=1})
}
if ($Case.Count -gt 0) {
    if (@($Case | Select-Object -Unique).Count -ne $Case.Count) { throw 'Duplicate case names' }
    foreach ($name in $Case) {
        if ($name -cnotin $plan.Name) { throw "Unknown case: $name" }
    }
    $plan = @($plan | Where-Object { $_.Name -cin $Case })
}
if ($PlanOnly) {
    $plan | ForEach-Object { [pscustomobject]$_ }
    return
}
function Require-Path([string]$Value, [bool]$Directory) {
    if ([string]::IsNullOrWhiteSpace($Value) -or -not [IO.Path]::IsPathFullyQualified($Value)) {
        throw 'All runtime paths must be explicitly absolute'
    }
    $item = Get-Item -LiteralPath $Value
    if ($item.PSIsContainer -ne $Directory) { throw 'Runtime path has the wrong type' }
    $item.FullName
}
$TestExe = Require-Path $TestExe $false
$JavaExe = Require-Path $JavaExe $false
$ProjectRoot = Require-Path $ProjectRoot $true
$ServerRoot = Require-Path $ServerRoot $true
$SharedRoot = Require-Path $SharedRoot $true
$MysqlHome = Require-Path $MysqlHome $true
$OutputRoot = Require-Path $OutputRoot $true
$ownTarget = [IO.Path]::GetFullPath((Join-Path $ProjectRoot 'target')) + [IO.Path]::DirectorySeparatorChar
if (-not [string]::IsNullOrWhiteSpace($CaptureDirectory)) {
    $CaptureDirectory = Require-Path $CaptureDirectory $true
    if (-not $CaptureDirectory.StartsWith($ownTarget, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Offscreen captures must belong to this standalone project target'
    }
}
if (-not $TestExe.StartsWith($ownTarget, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'The pinned test EXE must belong to this standalone project target'
}
if ($TestExeSha256 -cnotmatch '^[A-Fa-f0-9]{64}$' -or
    (Get-FileHash -LiteralPath $TestExe).Hash -ine $TestExeSha256) {
    throw 'An explicitly pinned, already-built test EXE is required'
}
if ($JavaTestJars.Count -eq 0) { throw 'Explicit Java test dependency JARs are required, including JUnit API and its dependencies' }
$JavaTestJars = @($JavaTestJars | ForEach-Object { Require-Path $_ $false })
$packagedInputs = @()
if (![string]::IsNullOrWhiteSpace($PackagedServerJar)) {
    if (@($plan | Where-Object {$_.Name -notin @('same-main-all-results','same-main-all-results-expired-vip')}).Count -gt 0) { throw 'Packaged JVM evidence currently requires the full same-main six-mode result scenario' }
    $PackagedServerJar = Require-Path $PackagedServerJar $false
    if ($PackagedServerJarSha256 -cnotmatch '^[A-Fa-f0-9]{64}$' -or (Get-FileHash -LiteralPath $PackagedServerJar).Hash -ine $PackagedServerJarSha256) { throw 'Explicit matching packaged server SHA256 required' }
    $packageRoot = Split-Path -Parent $PackagedServerJar
    $packagedInputs = @(Get-ChildItem -LiteralPath $packageRoot -File -Recurse | Select-Object -ExpandProperty FullName)
    $packagedInputs += Require-Path (Join-Path $ServerRoot 'target/modern-server.build.json') $false
    $packagedInputs += Require-Path (Join-Path $ServerRoot 'tools/package-modern.ps1') $false
} elseif (![string]::IsNullOrWhiteSpace($PackagedServerJarSha256)) { throw 'Packaged hash requires a JAR' }
$requiredClasses = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
foreach ($name in @('org/junit/jupiter/api/Assertions.class','org/junit/platform/commons/util/Preconditions.class',
    'org/opentest4j/AssertionFailedError.class','org/apiguardian/api/API.class')) { $null = $requiredClasses.Add($name) }
foreach ($jar in $JavaTestJars) {
    if ([IO.Path]::GetExtension($jar) -ine '.jar') { throw 'Java test dependency is not a JAR' }
    $archive = [IO.Compression.ZipFile]::OpenRead($jar)
    try {
        foreach ($name in @($requiredClasses)) { if ($null -ne $archive.GetEntry($name)) { $null = $requiredClasses.Remove($name) } }
    } finally { $archive.Dispose() }
}
if ($requiredClasses.Count -ne 0) { throw 'Java test classpath lacks required assertion support; no fixture was started' }
$fixtures = Require-Path (Join-Path $ServerRoot 'target/modern-tls-test-fixtures') $true
foreach ($path in @('root.der','server.p12','complete-v1')) {
    $null = Require-Path (Join-Path $fixtures $path) $false
}
$null = Require-Path (Join-Path $SharedRoot 'Sprite.idx') $false
$null = Require-Path (Join-Path $MysqlHome 'bin/mysqld.exe') $false
$null = Require-Path (Join-Path $ServerRoot 'db/modern_match_settlement.sql') $false
$revivalFixtures = @()
if ('legacy-tls-lifecycle' -cin $plan.Name) {
    $revivalFixtures += Require-Path (Join-Path $fixtures 'expired.p12') $false
}
if ('canonical-revival-wire' -cin $plan.Name) {
    foreach ($name in @('dead','revived','stale','not-dead','aborted','revived-retired')) {
        foreach ($suffix in @('request','reply')) {
            $revivalFixtures += Require-Path (Join-Path $ServerRoot "target/modern-revival-test-fixtures/$name.$suffix") $false
        }
    }
}
if ('canonical-equipment-wire' -cin $plan.Name) {
    foreach ($name in @('before','applied','unchanged','conflict','reequipped','stale','busy','terminal','retired-before','retired','honor-stale','honor-dead','siege-stale')) {
        foreach ($suffix in @('request','reply')) {
            $revivalFixtures += Require-Path (Join-Path $ServerRoot "target/modern-equipment-test-fixtures/$name.$suffix") $false
        }
    }
}
if ('canonical-speed-command-visual' -cin $plan.Name) {
    foreach ($name in @('slow','miss','haste','neutralized')) {
        foreach ($suffix in @('before','request','reply')) {
            $revivalFixtures += Require-Path (Join-Path $ServerRoot "target/modern-speed-command-fixtures/$name.$suffix") $false
        }
    }
}
function Assert-NoBuild {
    # This is a guard, not an interprocess lease. Coordinate the entire freeze
    # with other tasks; no runner can prevent an unrelated build starting later.
    # Explicit root paths are passed by this project's tools. An unrelated
    # LHZ build must not block this independent project's tests.
    $projectPattern = [regex]::Escape($ProjectRoot)
    $serverPattern = [regex]::Escape($ServerRoot)
    if (@(Get-CimInstance Win32_Process | Where-Object {
        $_.Name -in @('cargo.exe','rustc.exe','mvn.exe') -and
        $_.CommandLine -match $projectPattern
    }).Count -gt 0) { throw 'This LiTo project is being built' }
    # Maven's Windows wrapper runs as java.exe. Inspect only for build markers;
    # never print command lines, which may contain unrelated credentials.
    if (@(Get-CimInstance Win32_Process -Filter "Name='java.exe'" | Where-Object {
        $_.CommandLine -match $serverPattern -and
        $_.CommandLine -match 'org\.codehaus\.plexus\.classworlds|org\.apache\.maven|surefirebooter'
    }).Count -gt 0) {
        throw 'Java build/test runner is active; finish it before isolated peers'
    }
}
function Inventory([bool]$HashContent = $false) {
    $paths = @($TestExe, $JavaExe, $PSCommandPath) + $JavaTestJars + $revivalFixtures + $packagedInputs
    foreach ($name in @('root.der','server.p12','complete-v1')) { $paths += Join-Path $fixtures $name }
    $paths += Join-Path $SharedRoot 'Sprite.idx'
    if (@($plan | Where-Object { $_.Name -in @('canonical-slow-sql','canonical-slow-controller-sql','canonical-inventory-sql','canonical-inventory-render-lag','canonical-inventory-render-pace','canonical-teleport-sql') }).Count -gt 0) {
        # This scenario qualifies the real shared sector before LOAD; freeze it
        # along with the containers used for its first rendered surface.
        $paths += Require-Path (Join-Path $SharedRoot 'map/514/7fff8000.s32') $false
    }
    if (@($plan | Where-Object { $_.Name -in @('canonical-slow-quest-sql','canonical-target-haste-quest-sql') }).Count -gt 0) {
        foreach ($key in @('7fff8000','7fff8001','80008000','80008001')) {
            $paths += Require-Path (Join-Path $SharedRoot "map/554/$key.s32") $false
        }
    }
    # These two explicit banks consume Text/list.spr and Sprite/*.spr, not
    # just the sprite index. Freeze their containers even when run alone.
    if (@($plan | Where-Object { $_.Name -in @('shared-haste43','shared-greater-haste','shared-slow29','canonical-speed-command-visual') }).Count -gt 0) {
        foreach ($name in @('Text.idx','Text.pak','Sprite.pak')) { $paths += Join-Path $SharedRoot $name }
    }
    # Conservatively freeze every existing shared-only resource container,
    # WAV and terrain sector. A case-name allowlist omitted Siege/Race/Honor
    # terrain, and could silently miss a newly added consumer. Inventory is
    # not loading or qualification: each case still checks its exact manifest
    # and strict resource requirements. Minimal fixture roots remain valid;
    # absent optional directories become part of Inventory if later created.
    $paths += @(Get-ChildItem -LiteralPath $SharedRoot -File | Where-Object {
        $_.Name -match '^(Sprite(?:[0-9]{2})?|Text|Tile)\.(idx|pak)$'
    } | Select-Object -ExpandProperty FullName)
    $npcSoundRoot = Join-Path $SharedRoot 'sound'
    if (Test-Path -LiteralPath $npcSoundRoot -PathType Container) {
        $paths += @(Get-ChildItem -LiteralPath $npcSoundRoot -File -Filter '*.wav' | Select-Object -ExpandProperty FullName)
    }
    $terrainRoot = Join-Path $SharedRoot 'map'
    if (Test-Path -LiteralPath $terrainRoot -PathType Container) {
        $paths += @(Get-ChildItem -LiteralPath $terrainRoot -Recurse -File -Filter '*.s32' | Select-Object -ExpandProperty FullName)
    }
    # Newly consumed footstep WAVs must be frozen as well as the catalog.
    # Preserve legacy-only/minimal-root runs: a missing file is recorded as
    # absent; a case that needs it still fails its strict resource preflight.
    # Creation/removal during a run changes Inventory before the next case.
    foreach ($name in @('sound/478.wav','sound/479.wav')) {
        $soundPath = Join-Path $SharedRoot $name
        if (Test-Path -LiteralPath $soundPath -PathType Leaf) { $paths += $soundPath }
    }
    foreach ($base in @((Join-Path $ProjectRoot 'src'), (Join-Path $ServerRoot 'src'),
        (Join-Path $ServerRoot 'target/classes'), (Join-Path $ServerRoot 'target/test-classes'),
        (Join-Path $ServerRoot 'target/lib'))) {
        $paths += @(Get-ChildItem -LiteralPath $base -Recurse -File | Select-Object -ExpandProperty FullName)
    }
    foreach ($name in @('Cargo.toml','Cargo.lock','build.rs','.cargo/config.toml','config/profile.example.json','tools/build.ps1',
        'tools/build-evidence.ps1','tools/package.ps1','tools/verify-independent.ps1','packaging/README.md','packaging/profile.example.json')) {
        $p = Join-Path $ProjectRoot $name
        if (Test-Path -LiteralPath $p -PathType Leaf) { $paths += $p }
    }
    # Once a current player build exists, freeze it and its receipt alongside
    # the test executable. This is provenance, not actual HWND acceptance.
    $playerReceipt = Join-Path $ProjectRoot 'target/player-release.build.json'
    if (Test-Path -LiteralPath $playerReceipt -PathType Leaf) {
        $paths += $playerReceipt
        $paths += Require-Path (Join-Path $ProjectRoot 'target/i686-pc-windows-msvc/release/LiToModern.exe') $false
    }
    foreach ($name in @('pom.xml','db/modern_match_settlement.sql','db/modern_pet_race_settlement.sql')) { $paths += Join-Path $ServerRoot $name }
    $paths | Sort-Object -Unique | ForEach-Object {
        $item = Get-Item -LiteralPath $_
        [pscustomobject]@{Path=$item.FullName; Length=$item.Length; ModifiedUtcTicks=$item.LastWriteTimeUtc.Ticks; SHA256=$(if ($HashContent) { (Get-FileHash -LiteralPath $_).Hash } else { $null })}
    }
}
function Save-New([string]$Path, [string]$Text) {
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try {
        $bytes = [Text.UTF8Encoding]::new($false).GetBytes($Text)
        $stream.Write($bytes, 0, $bytes.Length)
    } finally { $stream.Dispose() }
}
function Confirm-Result([string]$Output, [int]$Code, [int]$Expected) {
    $summaries = [regex]::Matches($Output, '(?m)^test result: ok\. (\d+) passed; 0 failed; 0 ignored;.*$')
    $Code -eq 0 -and $Expected -gt 0 -and $summaries.Count -eq 1 -and
        [int]$summaries[0].Groups[1].Value -eq $Expected
}
function Set-ProcessEnvironment([string]$Name, [AllowNull()][object]$Value) {
    # On current PowerShell/.NET, binding plain $null to a string parameter can
    # create an empty value. Rust var_os would then incorrectly enable an
    # optional capture path. Distinguish absent from intentionally empty.
    if ($null -eq $Value) {
        [Environment]::SetEnvironmentVariable($Name, [NullString]::Value, 'Process')
    } else {
        [Environment]::SetEnvironmentVariable($Name, [string]$Value, 'Process')
    }
}
Assert-NoBuild
if ($CheckOnly) {
    [pscustomobject]@{PinnedExe=$TestExe; DependencyJars=$JavaTestJars.Count; PlannedGroups=$plan.Count; NativeProcessesStarted=0}
    return
}
$projectLease = [IO.File]::Open((Join-Path $ownTarget 'project-lease.lock'), [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
try {
$baseline = @(Inventory $true)
$baselineText = ConvertTo-Json -InputObject $baseline -Depth 5 -Compress
$metadataText = ConvertTo-Json -InputObject @(Inventory) -Depth 5 -Compress
$run = Join-Path $OutputRoot ('modern-preflight-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssZ') + '-' + [Guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $run
Save-New (Join-Path $run 'inputs.json') $baselineText
Save-New (Join-Path $run 'plan.json') (ConvertTo-Json -InputObject $plan -Depth 4)
$environment = @{
    LITO_MODERN_TEST_JAVA=$JavaExe
    LITO_MODERN_TEST_JAVA_CLASSPATH=((Join-Path $ServerRoot 'target/test-classes') + ';' + (Join-Path $ServerRoot 'target/classes') + ';' + (Join-Path $ServerRoot 'target/lib/*') + ';' + ($JavaTestJars -join ';'))
    LITO_MODERN_TEST_SERVER_ROOT=$ServerRoot
    LITO_MODERN_TEST_SERVER_JAR=$(if ([string]::IsNullOrWhiteSpace($PackagedServerJar)) { $null } else { $PackagedServerJar })
    LITO_MODERN_TEST_SERVER_JAR_SHA256=$(if ([string]::IsNullOrWhiteSpace($PackagedServerJar)) { $null } else { $PackagedServerJarSha256.ToUpperInvariant() })
    LITO_MODERN_TEST_EXPIRED_VIP=$null
    LITO_SHARED_RESOURCE_ROOT=$SharedRoot
    LITO_MODERN_TLS_TEST_FIXTURES=$fixtures
    LITO_MODERN_MYSQL_HOME=$MysqlHome
    LITO_MODERN_SPEED_COMMAND_FIXTURES=(Join-Path $ServerRoot 'target/modern-speed-command-fixtures')
    LITO_MODERN_PROBE_ACCOUNT=$null
    LITO_MODERN_PROBE_PASSWORD=$null
    LITO_CANONICAL_CAPTURE_DIR=$(if ([string]::IsNullOrWhiteSpace($CaptureDirectory)) { $null } else { $CaptureDirectory })
}
$prior = @{}
$results = [Collections.Generic.List[object]]::new()
$complete = $false
$locationPushed = $false
try {
    foreach ($key in $environment.Keys) {
        $prior[$key] = [Environment]::GetEnvironmentVariable($key, 'Process')
        Set-ProcessEnvironment $key $environment[$key]
    }
    Push-Location -LiteralPath $ProjectRoot
    $locationPushed = $true
    $listed = @(& $TestExe --ignored --list)
    if ($LASTEXITCODE -ne 0) { throw 'Cannot list ignored tests from the pinned EXE' }
    $allNames = @($listed | Where-Object { $_ -cmatch ': test$' } | ForEach-Object { $_.Substring(0, $_.Length - 6) })
    $chosen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($entry in $plan) {
        $names = @($allNames | Where-Object {
            $_.Contains($entry.Filter) -and (-not $entry.ContainsKey('Exclude') -or -not $_.Contains($entry.Exclude))
        })
        if ($names.Count -ne $entry.Count) { throw "Wrong test selection count: $($entry.Name)" }
        foreach ($name in $names) {
            $selectionKey = if ($entry.Name -eq 'same-main-all-results-expired-vip') { $name + '|expired-vip-fixture' } else { $name }
            if (-not $chosen.Add($selectionKey)) { throw 'Overlapping test filters' }
        }
    }
    foreach ($entry in $plan) {
        Assert-NoBuild
        if ((ConvertTo-Json -InputObject @(Inventory) -Depth 5 -Compress) -cne $metadataText -or
            (Get-FileHash -LiteralPath $TestExe).Hash -ine $TestExeSha256) { throw 'Frozen build inputs changed before test' }
        Write-Host "START $($entry.Name) expected=$($entry.Count)"
        Set-ProcessEnvironment 'LITO_MODERN_TEST_EXPIRED_VIP' $(if ($entry.ContainsKey('ExpiredVip') -and $entry.ExpiredVip) { 'true' } else { $null })
        $started = [DateTime]::UtcNow
        $testArgs = @($entry.Filter, '--ignored', '--test-threads=1')
        if ($entry.ContainsKey('Exclude')) { $testArgs += @('--skip', $entry.Exclude) }
        if ($ShowTestOutput) { $testArgs += '--show-output' }
        if ($InteractiveWindow -or $InteractiveEquipment) { $testArgs += '--nocapture' }
        $output = @(& $TestExe @testArgs 2>&1 | ForEach-Object { "$_" })
        $code = $LASTEXITCODE
        $elapsed = ([DateTime]::UtcNow - $started).TotalSeconds
        $joined = $output -join "`n"
        Save-New (Join-Path $run ($entry.Name + '.log')) $joined
        $passed = Confirm-Result $joined $code $entry.Count
        $results.Add([pscustomobject]@{Name=$entry.Name; Expected=$entry.Count; ExitCode=$code; Passed=$passed; Seconds=$elapsed; StartedUtc=$started.ToString('o')})
        Save-New (Join-Path $run ($entry.Name + '.result.json')) (ConvertTo-Json $results[$results.Count-1])
        Write-Host "END $($entry.Name) passed=$passed seconds=$([Math]::Round($elapsed,2))"
        if (-not $passed) { throw "Interop failed: $($entry.Name); evidence preserved in $run" }
        if ((ConvertTo-Json -InputObject @(Inventory) -Depth 5 -Compress) -cne $metadataText -or
            (Get-FileHash -LiteralPath $TestExe).Hash -ine $TestExeSha256) { throw 'Frozen build inputs changed during test' }
    }
    # Per-case metadata plus EXE digest are cheap enough for the live interval;
    # a full final digest also catches same-size/restored-timestamp mutations.
    Assert-NoBuild
    $finalText = ConvertTo-Json -InputObject @(Inventory $true) -Depth 5 -Compress
    Save-New (Join-Path $run 'final-inputs.json') $finalText
    if ($finalText -cne $baselineText) { throw 'Full input digests changed across the run' }
    $complete = $true
} finally {
    if ($locationPushed) { Pop-Location }
    foreach ($key in $prior.Keys) { Set-ProcessEnvironment $key $prior[$key] }
    Save-New (Join-Path $run 'summary.json') (ConvertTo-Json -Depth 6 -InputObject @{
        Complete=$complete; PlannedCases=($plan | Measure-Object Count -Sum).Sum; Results=@($results.ToArray())
        TestExecutable=$TestExe; TestExecutableSHA256=$TestExeSha256; ShowTestOutput=[bool]$ShowTestOutput
        PackagedServerJar=$PackagedServerJar; PackagedServerJarSHA256=$PackagedServerJarSha256
        Scope=$(if($InteractiveEquipment){'Interactive v41 equipment HWND/TLS/shared514/isolated gear store; no SQL or full acceptance'}elseif($InteractiveWindow){'Interactive canonical HWND/TLS/shared514/isolated SQL smoke, not all modes/buttons or complete acceptance'}elseif(![string]::IsNullOrWhiteSpace($PackagedServerJar)){'Headless Rust controllers to dedicated -jar JVM with isolated SQL/shared resources; no player EXE HWND or full acceptance'}else{'Headless canonical resource/TLS/isolated SQL regression, not complete UI/TW acceptance'})
    })
    Write-Host "Evidence: $run"
}
} finally { $projectLease.Dispose() }
