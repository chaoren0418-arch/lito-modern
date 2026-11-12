//! Standalone LiTo engine, extracted into its own independently built client.
//!
//! This module must not depend on TW memory addresses, injection, or native LiTo
//! resource packs implicitly. Explicit pinned Native resource inspection is
//! separately opt-in; it does not grant session/world admission.
//! Decoders and reference presentation are not a playable game
//! engine. TW launch, injection and LHZ automation are not part of this crate.

pub mod actor_assets;
pub mod ammo_protocol;
#[cfg(test)]
#[path = "ammo_resource_reference_tests.rs"]
mod ammo_resource_reference_tests;
pub mod animation;
pub mod application_protocol;
#[cfg(test)]
mod armor_protection_reference_tests;
pub mod audio;
mod audio_gain;
#[cfg(windows)]
pub mod audio_output;
pub(crate) mod battle_action;
pub(crate) mod battle_audio;
pub(crate) mod battle_body_audio;
pub(crate) mod battle_bows;
pub mod battle_control;
pub mod battle_details;
pub(crate) mod battle_equipment;
pub(crate) mod battle_vip_equipment;
pub(crate) mod battle_vip_world;
pub mod battle_follow;
pub mod battle_greater_haste;
pub(crate) mod battle_haste;
pub(crate) mod battle_healing;
pub(crate) mod battle_input;
pub(crate) mod battle_inventory;
pub(crate) mod battle_items;
pub(crate) mod battle_minimap;
pub(crate) mod battle_hud;
pub(crate) mod battle_motion;
pub(crate) mod battle_operation;
pub mod battle_payload;
pub mod battle_playback;
pub mod battle_player;
pub mod battle_player_healing;
pub(crate) mod battle_protection;
pub mod battle_read;
pub(crate) mod battle_revival;
pub(crate) mod battle_route;
pub mod battle_runtime;
pub mod battle_scene;
mod battle_vip_scene_preview;
pub mod battle_session;
pub(crate) mod battle_skillbook;
pub mod battle_slow;
pub mod battle_slow_visual;
pub mod battle_space;
pub(crate) mod battle_spells;
pub(crate) mod battle_state;
pub mod battle_target_haste;
mod battle_target_haste_visual;
pub(crate) mod battle_teleport;
pub mod battle_window;
pub(crate) mod battle_world;
pub mod character_data;
pub mod chat_protocol;
pub mod chat_state;
#[cfg(windows)]
pub mod chat_window;
pub mod combat_animation;
pub mod combat_audio;
pub mod combat_protocol;
pub mod compositor;
pub mod connection;
#[cfg(test)]
mod deathmatch_resource_tests;
pub mod descriptions;
pub mod direct_chat_protocol;
pub mod direct_chat_state;
pub mod direct_chat_window;
pub mod disintegrate_assets;
pub mod disintegrate_cast_protocol;
pub mod disintegrate_effect;
pub(crate) mod disintegrate_presentation;
pub mod disintegrate_protocol;
pub mod equipment;
pub mod effect_catalog;
pub mod vip_status;
pub(crate) mod vip_status_preview;
pub(crate) mod vip_inventory_preview;
pub mod equipment_history;
pub mod equipment_history_wire;
pub mod equipment_transition;
pub mod filtered_chat_protocol;
pub mod fire_storm_effect;
pub(crate) mod fire_storm_presentation;
pub mod fire_storm_protocol;
pub mod fixed_walk;
pub mod gm_protocol;
pub mod greater_haste_effect;
pub mod haste_effect;
pub mod healing_effect;
pub mod honor_healing_effect;
pub mod honor_protocol;
pub mod immune_to_harm_effect;
mod battle_armor_effects;
pub mod immune_to_harm_icon;
pub mod inventory_icons;
pub mod native_skill_graphics;
pub mod item_protocol;
pub mod light_arrow_effect;
#[cfg(windows)]
pub mod login_window;
pub mod map;
#[cfg(test)]
mod map509_reference_tests;
#[cfg(test)]
mod map532_reference_tests;
pub mod matchmaking_protocol;
#[cfg(all(test, windows))]
pub mod mode_window;
pub mod npc_assets;
pub mod npc_disintegrate_protocol;
pub mod npc_disintegrate_wire;
pub(crate) mod npc_fire_storm_frame;
pub(crate) mod npc_fire_storm_protocol;
pub mod npc_fire_storm_wire;
pub mod npc_presentation;
pub mod npc_protocol;
pub mod npc_spell_protocol;
mod paced_animation;
pub mod participant;
pub mod pet_animation;
#[cfg(test)]
mod pet_race_resource_probe_tests;
pub mod player_healing;
pub mod player_healing_history;
pub mod player_appearance;
pub mod polymorph_assets;
pub mod preferences;
pub mod presentation;
#[cfg(any(test, debug_assertions))]
pub mod preview;
pub mod profile;
#[cfg(windows)]
mod profile_picker;
pub mod protocol;
pub mod quest_life;
pub mod quest_presentation;
pub mod quest_protocol;
#[cfg(test)]
mod quest_resource_probe_tests;
pub mod race_protocol;
pub(crate) mod race_space;
pub mod ranged_effect;
pub mod record_protocol;
pub mod regen_protocol;
pub(crate) mod resource_cipher;
pub mod resources;
pub mod rebuild_verification;
pub mod native_wire;
pub mod native_opcodes;
pub mod native_integrity;
pub mod native_version;
pub mod native_connection;
pub(crate) mod native_clock;
pub(crate) mod native_static2941;
pub(crate) mod native_dark_elf_effects;
pub(crate) mod native_player_callbacks;
pub mod native_login;
pub mod native_game;
#[cfg(windows)]
pub mod native_login_window;
pub mod native_resources;
pub mod native_pet_race_component;
pub mod native_pet_race_terrain;
pub mod native_pet_race_offline;
pub mod native_pet_race_session;
pub mod native_pet_race_quit_session;
pub mod native_pet_race_return_profile;
pub mod native_pet_race_wire_boundary;
pub mod native_pet_race_start_component;
// Unfrozen local codec/owner experiments. No live transport/profile route.
#[allow(dead_code)]
pub(crate) mod native_canonical_draft;
#[allow(dead_code)]
pub(crate) mod native_action_inspection;
#[allow(dead_code)]
pub(crate) mod native_actor_inspection;
#[allow(dead_code)]
pub(crate) mod native_catalog_draft;
#[allow(dead_code)]
pub(crate) mod native_actor_dependency_draft;
#[allow(dead_code)]
pub(crate) mod native_event_scheduler;
#[allow(dead_code)]
pub(crate) mod native_scene_composition;
#[cfg(test)]
mod native_player_loadout_evidence_tests;
#[cfg(test)]
mod native_player_navigation_evidence_tests;
#[allow(dead_code)]
pub(crate) mod native_navigation_runner;
#[cfg(test)]
mod native_navigation_runner_tests;
#[allow(dead_code)]
pub(crate) mod native_external_fixture;
#[allow(dead_code)]
pub(crate) mod native_appearance_inspection;
#[cfg(test)]
mod native_appearance_inspection_tests;
#[allow(dead_code)]
pub(crate) mod native_appearance_clock_study;
mod native_appearance_continuation_study;
#[cfg(test)]
mod native_appearance_continuation_study_tests;
mod native_appearance_layer_study;
#[cfg(test)]
mod native_appearance_layer_study_tests;
mod native_appearance_replay_study;
mod native_locomotion_study;
#[cfg(test)]
mod native_appearance_replay_study_tests;
mod native_appearance_occlusion_study;
#[cfg(test)]
mod native_appearance_occlusion_study_tests;
mod native_appearance_terrain_study;
mod native_appearance_idle_study;
mod native_roof_callback_study;
mod native_layer8_callback_study;
#[cfg(test)]
mod native_layer8_callback_study_tests;
#[cfg(test)]
mod native_layer8_sequence_study_tests;
#[cfg(test)]
mod native_roof_callback_study_tests;
#[cfg(test)]
mod native_appearance_idle_study_tests;
#[cfg(test)]
mod native_appearance_terrain_study_tests;
#[cfg(test)]
mod native_appearance_clock_study_tests;
#[cfg(windows)]
pub mod room_window;
pub mod rooms;
pub mod scene;
pub mod scene_model;
#[cfg(windows)]
pub mod scene_preview;
#[cfg(windows)]
pub mod session;
#[cfg(windows)]
pub mod settings_window;
pub(crate) mod shared_action_catalog;
pub mod shop_icons;
pub mod shop_protocol;
pub mod shop_window;
pub mod siege_presentation;
pub mod siege_protocol;
pub mod siege_space;
pub mod skill_spending;
pub mod slow_effect;
pub mod spell_protocol;
pub mod sprite;
pub mod sprite_actions;
pub mod target_history;
pub mod teleport_effect;
pub mod tile;
mod tile_pixels;
mod tile_reader;
pub mod time_attack_presentation;
pub mod time_attack_protocol;
pub mod timed_effect;
pub mod timed_protocol;
pub mod tower_assets;
pub mod transport;
pub mod tw_action_catalog;
pub mod use_history;
pub mod vital_balance;
pub mod weight_protocol;
pub mod world_audio;
pub mod world_depth;
#[cfg(all(test, windows))]
mod world_integration_tests;
pub mod world_map;
pub mod world_navigation;
pub mod world_path;
pub mod world_protocol;
mod world_raster;
mod world_roofs;
pub mod world_runtime;
pub mod world_scene;
#[cfg(windows)]
pub mod world_window;

/// Standalone entry uses the existing read-only picker, never a TW fallback.
#[cfg(windows)]
pub fn choose_profile_path() -> anyhow::Result<Option<std::path::PathBuf>> {
    unsafe { profile_picker::choose(windows::Win32::Foundation::HWND::default()) }
}

pub(crate) mod native_campaign_resource_study;
pub mod native_offline_map_flow;
pub mod native_offline_preview_resources;
pub mod native_offline_preview_runtime;
#[cfg(windows)]
pub mod native_offline_preview_window;

pub mod native_offline_menu_skin;
pub mod native_offline_menu_window;

pub mod native_multi_map_resources;
pub mod native_multi_map_runtime;
pub mod native_multi_map_menu;
pub mod native_multi_map_window;

pub mod native_mode_menu;
pub mod native_mode_menu_skin;
pub mod native_mode_menu_runtime;
pub mod native_mode_menu_window;

pub mod native_map_batch_window;
#[cfg(test)]
mod native_map_batch_tests;

pub mod native_multisector_resources;
pub mod native_multisector_flow;
pub mod native_multisector_scene;
pub mod native_multisector_runtime;
pub mod native_multisector_window;
#[cfg(test)]
mod native_multisector_tests;

pub mod native_multisector_batch_window;
#[cfg(test)]
mod native_multisector_batch_tests;

pub mod native_cross_mode_window;
#[cfg(test)]
mod native_cross_mode_tests;

pub mod native_city_flow;
pub mod native_city_runtime;
pub mod native_city_window;
pub mod native_backend_485f_entry;

pub mod native_lobby;
pub mod native_room;
pub mod native_room_ui;
pub mod native_minimap;
pub mod native_world;
pub mod native_wire_audio;
pub mod native_world_scene;
pub mod native_light_mask;
#[cfg(windows)]
pub mod native_light_color;
#[cfg(windows)]
pub mod native_light_sources;
#[cfg(windows)]
pub mod native_render_environment;
#[cfg(windows)]
pub mod native_world_lighting;
#[cfg(windows)]
pub mod native_water_light;
pub mod native_water_clock;
pub mod native_game_timers;
pub mod native_environment;
pub mod native_world_layers;
pub mod native_actor_layers;
pub mod native_world_view;
pub mod native_world_animation;
pub(crate) mod native_status4183;
pub mod native_navigation;
pub mod native_speed;
pub mod native_hp_meter;
pub mod native_status_icons;

pub mod native_terrain_cache;

pub mod native_combat;
pub mod native_combat_ops;
pub mod native_forced_combat;
pub mod native_target_policy;
pub mod native_combat_animation;
pub mod native_combat_probe;
pub mod native_peer_probe;
pub mod native_peer_room_probe;
pub mod native_peer_classic_probe;
pub mod native_services;
pub mod native_teleport;
pub mod native_peer_rental_probe;
pub mod native_points;
pub mod native_tournament_shop;
pub mod native_map_item;
pub mod native_triple_arrow;
pub mod native_merchant;
pub mod native_companions;
pub mod native_npc_document;
pub mod native_npc_images;
mod native_peer_citizen_probe;
mod native_peer_hotel_probe;
mod native_peer_hotel_share_probe;
mod native_peer_merchants_probe;
pub mod native_portal;
pub mod native_rental_source;
pub mod native_attributes;
pub mod native_peer_actions;
pub mod native_poison_probe;
pub mod native_class1;
pub mod native_disintegrate;
#[cfg(test)]
mod native_disintegrate_tests;
pub mod native_recovery_probe;
pub mod native_doors;
pub mod native_obstacles;
pub mod native_peer_world_probe;
pub mod native_peer_lifecycle_probe;
pub mod native_projectiles;
pub mod native_skills;
pub mod native_teleport_spell;
pub mod native_skill_selection;
pub mod native_ground_effects;
pub mod native_effect_timing;
pub mod native_inventory;
pub mod native_inventory_names;
pub mod native_inventory_icons;
pub mod native_skill_effects;
pub mod native_spell_projectiles;

pub mod native_item_actions;
pub mod native_ground_pickup;
pub mod native_ground_pickup_probe;
pub mod native_item_probe;

pub mod native_world_roofs;

pub mod native_world_effects;

pub mod native_actor_states;
pub mod native_actor_relations;

pub mod native_actor_state_probe;

pub mod native_world_text;
pub mod native_world_labels;
pub mod native_clan_markers;
pub mod native_emblems;
pub mod native_label_update_probe;
pub mod native_emblem_probe;

pub mod native_peer_equipped_expiry_probe;

pub mod native_attack_many;

pub mod native_turn_undead_probe;

pub mod native_terrain_pixels;
pub mod native_background_pixels;
pub mod native_background_geometry;
pub mod native_background;
pub mod native_object_dispatch;

pub mod native_startup;
#[cfg(windows)]
pub mod native_startup_window;

pub mod native_selector_effects;
pub mod native_audio_delivery;

pub mod call_lightning_assets;

pub mod native_call_lightning;

pub mod native_ambient_bridge;

pub mod native_ambient_override;

pub mod cone_cold_assets;
pub mod wire_spatial;
pub mod native_cone_cold;

pub(crate) mod native_additional_finite_effects;

pub mod native_ability_buffs;
