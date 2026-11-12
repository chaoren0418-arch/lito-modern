# Issue #307 — T036（M3 2/15）跨函式 helper 閉合審查：技能／戰鬥（1／2）

執行者：claude-local（本機，靜態審查）。審查基準：`origin/main` `f8ed20ec`（2026-11-12）。
**本張沒有執行任何測試，也沒有編譯；下面沒有任何一項是「測試通過」。** 結論只回答一個問題：這些測試跑起來會碰到什麼（檔案、網路、視窗、音效裝置、子程序），所以能不能放行給執行張（#310）去跑。

## 結論

- 本張審 56 個測試（#54 標「跨函式／資源 helper 安全邊界尚未封閉」的技能／戰鬥前半）。56 個在現行 main 都還在，名稱沒變。
- **32 個是純記憶體**（A 類）：只用內嵌 fixture 和記憶體狀態，可以放行，任何機器都能跑。
- **22 個只多一件事：唯讀讀取 `D:\lineage開發\0-客戶端`**（B 類）：沒有寫檔、網路、視窗、音效裝置、子程序、執行緒。可以放行，但只能在有這個資源根的本機跑。
- **2 個沒有閉合**（B′ 類），不放行，轉給世界／渲染那一張（#309）：一個走完整場景（`native_world_scene`），一個走 `world_raster`／`native_world_layers`。
- 56 個裡有 16 個早就在 `維護工具/test-whitelist/skills.tsv`（T041 放行）。所以**本張新放行 38 個：A 類 17、B 類 21**。清單在「新放行清單」一節，格式與既有白名單相同。
- 順帶確認一項歷史失敗的現況：C72D「499 過 1 敗」那 1 敗是神聖武器的效果清單（HISTORY L71）。現行測試的期待清單是 12 項，和 `qualify_skills` 的回傳順序逐項對得上（靜態比對）。有沒有真的轉綠要等 #310 實跑。

| 分類 | 數量 | 已在 skills.tsv | 本張新放行 | 去向 |
|---|---:|---:|---:|---|
| A 純記憶體 | 32 | 15 | 17 | #310 執行 |
| B 唯讀資源根 | 22 | 1 | 21 | #310 執行（限本機） |
| B′ 未閉合 | 2 | 0 | 0 | #309 複核 |
| 合計 | 56 | 16 | 38 | |

## 本張範圍怎麼定的

Issue 只寫「技能／戰鬥（1／2）」，沒有列檔案，所以本張自己切，並已在 #308 留言告知另一半。

- #54 的「不明副作用」共 458 項：375 項是「helper 未封閉」，83 項是「同檔寫檔」（歸 #306）。
- 375 項裡屬於技能／戰鬥的有 19 個 target、112 項。依 target 名稱字母序對半切，剛好 56／56：
  - **本張（56）**：lib `native_disintegrate_tests`（10）、`native_attack_skills`（15）、`native_chill_touch`（3）、`native_combat`（11）、`native_cone_cold`（3）、`native_fire_arrow`（2）、`native_ground_skills`（6）、`native_holy_weapon`（3）、`native_ice_arrow`（3）。
  - **#308（56）**：`native_light_spell`（6）、`native_lightning`（3）、`native_poison_cure`（4）、`native_ranged`（9）、`native_reagent_skills`（5）、`native_skill_selection`（7）、`native_skills`（9）、`native_turn_undead`（6）、`native_vampiric_touch`（4）、`native_wind_cutter`（3）。
- 這 9 個檔裡另有 5 個「同檔寫檔」的測試（寫 `target/current/verification/*.png／*.json`），屬於 #306，本張沒有審：`native_attack_skills::original_class4_flight_impact_audio_and_hurt_are_once`、`native_chill_touch::common_finite_clock_quantizes_birth_and_dispatches_late_hit_once`、`native_fire_arrow::original_class4_flight_impact_audio_and_hurt_are_once`、`native_ground_skills::original_resource_eight_segments_terminal_dwell_and_same_id_sound`、`native_ice_arrow::original_class5_flight_impact_audio_and_hurt_are_once`。

## 怎麼審的

1. **讀完 9 個測試檔的本體與同檔 helper**（`tests/native_*.rs` 8 檔全文；`native_disintegrate_tests.rs` 的 helper、4 個測試本體，其餘 6 個以全檔的模組引用與路徑字串判定）。測試只做三類事：
   - 用 `include_bytes!`／`include_str!` 內嵌 fixture，餵給 `World::accept`，再呼叫 `native_skills::request`、`native_skill_selection`、`native_combat`。
   - 以固定路徑 `D:\lineage開發\0-客戶端` 載入原資源，在記憶體 canvas 上畫（`Effects::prepare／update／draw`、`Projectiles`、`LightArrowAssets`、`SharedArchive`）。
   - 兩個測試另外走 `native_world_scene::Model` 或 `world_raster`。
2. **逐函式閉合 `World::accept`**：讀完 `native_world.rs:342–1379`，列出它分派到的每個函式，再確認那些函式所在模組沒有副作用原語。
3. **模組集合封閉檢查**：把 1、2 走到的模組列成一個集合（73 個），用腳本確認兩件事：集合內每個模組自己有哪些副作用原語；集合內引用到集合外的每一條路徑是什麼。集合外的引用共 12 個模組，逐條人工確認（見下表）。

工具是四支唯讀的 Python 靜態腳本（不編譯、不執行專案程式），放在 `維護工具/t036-helper-closure/`，#308、#309 可以直接用：

```powershell
$s = 'LiTo381-Modern\src\modern_lito'
$set = Get-Content 維護工具\t036-helper-closure\set-skills-combat-1.txt
python -I 維護工具\t036-helper-closure\setcheck.py $s $set          # 集合封閉檢查
python -I 維護工具\t036-helper-closure\showfn.py  $s "native_skills:*"  # 逐函式列副作用行與跨模組引用
python -I 維護工具\t036-helper-closure\modcats.py $s "resources,combat_audio"  # 模組的副作用分類
```

腳本的判定是保守的：去掉註解、字串與行內 `#[cfg(test)] mod … { }` 後，用關鍵字找副作用原語；模組引用看 `super::X`、`super::super::X`、`modern_lito::X`、`use super::{X,…}`。

## 已驗證（靜態讀碼與腳本輸出，不是測試結果）

### 73 個模組自己的副作用

| 分類 | 模組 | 內容 |
|---|---|---|
| 檔案寫入、網路、子程序、建立執行緒、建立視窗 | 無 | 73 個模組都沒有 |
| 音效裝置 | `combat_audio` | 只在子模組 `output`（`:388` 起），經 `PcmVoice` 呼叫 `audio_output` |
| 檔案讀取 | `resources`、`shared_action_catalog`、`native_water_light` | `resources::open_readonly`（`:864`）只設 `read(true)`；其餘是 `canonicalize`、`symlink_metadata`、`read_exact` |
| 環境 | `resources` | `current_dir()`，只在傳入相對路徑時用；測試傳的是絕對路徑 |
| 時鐘 | `native_status4183`、`native_static2941`、`native_clock` | `Instant::now`、`QueryPerformanceCounter`，只讀 |
| 其他 Win32 | `resources`、`shared_action_catalog`、`native_clock` | 查檔案屬性與最終路徑、讀時鐘 |

### 集合外的 12 條引用

| 被引用模組 | 引用的項目 | 判定 |
|---|---|---|
| `world_scene` | `BaseAppearance` | 兩個欄位的資料結構加一個查表方法（`world_scene.rs:58–75`） |
| `audio_output` | `PcmVoice` 等 | 只有建立 `CombatVoicePool` 才到得了。建立點只有 4 處，都不在集合內：`battle_playback.rs:258`、`native_login_window.rs:2728`、`world_audio.rs:393`、`scene_preview.rs:205`。測試沒有建立它；lib 測試用自寫的 `Sink` 代替播放器 |
| `native_city_runtime` | `status_receipt` | 實際是 `native_status_receipt.rs`，已在集合內，純解析 |
| `native_connection` | `IngressReceipt` | 只出現在 `World::accept_received`（`pub(crate)`）與 2941 收據；測試呼叫的是 `accept` |
| `native_login` | `decode_text`、`encode_text`、`Snapshot`、`LoginPhase` | 前兩個只做 MS950 轉碼（`MultiByteToWideChar`／`WideCharToMultiByte`）；後兩個是資料型別，測試走的 `request`／`ground_request` 沒用到 |
| `native_player_callbacks` | `WalkPose`、`Event`、`Kind` | 資料型別 |
| `native_inventory`、`native_item_actions` | `Inventory`、`Item`、`Choice` | 只在 `native_skills` 的道具施法函式；`request`（`:287–480`）沒用到 |
| `native_world_labels`、`native_world_text` | `Line`、`Cache` | 只是 `native_emblems::draw`、`native_minimap::draw` 的參數；A、B 類測試不呼叫 |
| `native_object_dispatch`、`native_terrain_pixels`、`tile`、`tile_pixels`、`native_multisector_scene`、`native_world_roofs` | 多項 | 從 `world_raster`、`native_world_layers` 引出，**本張沒有往下閉合**；只有 1 個 B′ 測試會走到 |

### A 類（32）走到的東西

- `World::accept`（`native_world.rs:610`）→ `accept_inner`（`:656`）分派到的函式全部是位元組解析與記憶體更新。涉及模組：`native_portal`、`native_emblems`、`native_actor_relations`、`native_skills`、`native_minimap`、`native_ability_buffs`、`native_attributes`、`native_doors`、`native_attack_many`、`native_combat`、`native_obstacles`、`native_hp_meter`、`native_speed`、`native_status_icons`、`native_status_receipt`、`native_integrity`、`native_status4183`。這些模組加上 `native_skill_selection`、`native_skill_catalog`、`native_combat_ops`、`native_effect_timing` 共 282 個函式，命中的只有 `Instant::now` 與 `OnceLock`。
- 字串欄位經 `native_login::decode_message`（`:58`）做 MS950 轉碼。
- 技能名稱（「冰錐術」「究極光裂術」等）來自 `native_skill_catalog` 的程式內表格，不讀檔。
- `World` 的 `Default` 與 `Serialize` 都是 derive；欄位型別所在模組都在上面清單裡。

### B 類（22）多走到的東西

- 讀檔一律經 `resources`（`SharedArchive::open`、`read_entry`、`read_shared_sound_with_limit`）與 `shared_action_catalog`，唯讀。
- 解碼與繪圖（`tower_assets`、`light_arrow_effect`、`teleport_effect`、`sprite`、`sprite_actions`、`native_actor_layers`、`compositor`、`native_world_effects`、`disintegrate_assets`、`cone_cold_assets`）沒有任何副作用原語。
- 音效只載入成 `PreparedSound`／`SoundBank`（記憶體），回傳事件，不播放。
- `native_disintegrate::prepare_with_loader`（`:316`）失敗時會 `eprintln!`，只寫 stderr。
- lib 的 3 個測試會故意傳不存在的路徑：`Z:\missing-after-cancel`（`actual_stale_generation…`）、`D:/lineage開發/3-lito開發/lito重建版/LiTo381-Modern/target/current/absent-native77-root`（`actual_prepare_generation…`、`actual_cancel_and_failed_source…`）。只會嘗試讀取；第二個路徑寫死在原始專案目錄下，測試的前提是它不存在。

### 逐項判定

「skills」欄打勾表示已在 `skills.tsv`。

**lib `modern_lito::native_disintegrate_tests::native77_probe::`（10；全部 `#[ignore]`）**

| 測試 | 類 |
|---|---|
| `actual_budget_and_late_timer_do_not_catch_up` | B |
| `actual_cancel_and_failed_source_old_publication_cannot_relaunch` | B |
| `actual_canvas_screen_then_multiply_and_projection_preflight` | B |
| `actual_fragment_submission_uses_screen_multiply_and_remains_click_through` | **B′**（`world_raster`、`native_world_layers`） |
| `actual_frozen_position_target_life_and_caster_retirement` | B |
| `actual_marker_timing_once_normal_and_critical` | B |
| `actual_model_rebuild_restores_only_admission_not_pixels_or_callbacks` | B |
| `actual_prepare_generation_and_failed_source_retire_before_draw` | B |
| `actual_prepared_payload_publication_and_history_once` | B |
| `actual_stale_generation_owner_and_life_stamps_never_rebind` | B |

**`native_attack_skills`（15）**

| 測試 | 類 | skills |
|---|---|:-:|
| `call_lightning_request_contract::current_c02_exact_object_coordinates_and_readonly_authority` | A | ✓ |
| `call_lightning_request_contract::invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests` | A | ✓ |
| `call_lightning_request_contract::mp_and_lawful_are_server_permission_inputs_and_never_locally_spent` | A | ✓ |
| `call_lightning_request_contract::range_six_boundary_and_target_changes_use_current_received_coordinates` | A | ✓ |
| `call_lightning_request_contract::selected_learned_skill_has_native_label_and_rejects_stale_life_world_or_connection` | A | |
| `disintegrate_request_contract::current_c02_exact_object_coordinates_and_readonly_authority` | A | ✓ |
| `disintegrate_request_contract::invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests` | A | ✓ |
| `disintegrate_request_contract::mp_and_lawful_are_server_permission_inputs_and_never_locally_spent` | A | ✓ |
| `disintegrate_request_contract::range_ten_boundary_and_target_changes_use_current_received_coordinates` | A | ✓ |
| `disintegrate_request_contract::selected_learned_skill_has_native_label_and_rejects_stale_life_world_or_connection` | A | |
| `captured_c02_disintegrate_lawful352_is_a_rejection_without_authority_or_cast` | A | |
| `formal_attack_spell_layout_and_request_guards` | A | ✓ |
| `spell_authority_is_atomic_and_does_not_preempt_arrival_hurt` | A | |
| `class4_launch_freezes_target_and_retires_reused_identity` | B | |
| `class4_zero_distance_zero_damage_and_miss_keep_authority` | B | |

**`native_chill_touch`（3）**

| 測試 | 類 | skills |
|---|---|:-:|
| `request_layout_guards_and_server_authority_do_not_predict_healing_or_hurt` | A | ✓ |
| `all_eight_banks_frozen_heading_offset_once_sound_and_effect_hit` | B | |
| `miss_replacement_and_map_change_cannot_hurt_a_new_target` | B | |

**`native_combat`（11）**

| 測試 | 類 |
|---|---|
| `attack_feedback_distinguishes_reply_timeout_and_interrupted_authority` | A |
| `death_confirmation_requires_authoritative_zero_hp_and_death` | A |
| `formal_melee_vectors_and_atomic_rejection_never_infer_hp` | A |
| `formal_movement_release_does_not_cure_controls_or_predict_actor_state` | A |
| `held_attack_requires_a_new_press_after_even_a_brief_control_interruption` | A |
| `held_attack_retires_world_owner_correction_and_invalid_targets` | A |
| `held_attack_waits_for_authority_and_animation_and_stops_on_release_timeout` | A |
| `orphan_and_malformed_updates_do_not_create_actors` | A |
| `target_range_death_result_and_scoreboard_guards` | A |
| `npc_damage_bursts_preserve_spell_events_authority_and_death_preemption` | B（只讀 Text 的 `list.spr`） |
| `original_resource_attack_events_fire_once_and_death_holds` | B（只讀 Text 的 `list.spr`） |

**`native_cone_cold`（3）**

| 測試 | 類 | skills |
|---|---|:-:|
| `cold_raw_S23_is_atomic_and_defers_hurt_until_effect_arrival` | A | |
| `cold_request_uses_observed_C14_and_six_cell_range_without_spending_authority` | A | ✓ |
| `cold_selector_rechecks_target_life_generation_owner_and_book` | A | ✓ |

**`native_fire_arrow`（2）**

| 測試 | 類 | skills |
|---|---|:-:|
| `formal_attack_spell_layout_and_request_guards` | B（內部呼叫 `class4_zero_distance…`，會載入原資源） | ✓ |
| `class4_launch_freezes_target_and_retires_reused_identity` | B | |

**`native_ground_skills`（6）**

| 測試 | 類 | skills |
|---|---|:-:|
| `action18_and_location_effect_are_atomic_without_local_mana_or_damage` | A | |
| `formal_ground_request_unsigned_boundaries_range_and_authority` | A | ✓ |
| `ground_choice_is_separate_from_actor_target_and_fenced_by_owner` | A | |
| `native_class0_enters_source1_and_dwells_terminal0_before_retirement` | A | |
| `ground_effects_are_bounded_deduplicated_and_never_follow_actor_ids` | B | |
| `original_wave_duration_gates_repeated_ground_audio_and_world_reset` | **B′**（`native_world_scene::Model::load／draw_at`） | |

**`native_holy_weapon`（3）**

| 測試 | 類 | skills |
|---|---|:-:|
| `object_selector_screen_and_owner_guards_preserve_authority` | A | ✓ |
| `holy_sound_terminal_dwell_and_map_retirement_do_not_predict_a_buff` | B | |
| `original_holy_definition_is_exact_and_changed_controls_fail_closed` | B（只讀 Text 的 `list.spr`；即 C72D 那 1 敗） | |

**`native_ice_arrow`（3）**

| 測試 | 類 | skills |
|---|---|:-:|
| `formal_attack_spell_layout_and_request_guards` | A | ✓ |
| `class5_launch_freezes_target_and_retires_reused_identity` | B | |
| `class5_zero_distance_zero_damage_and_miss_keep_authority` | B | |

## 新放行清單（38）

格式與 `維護工具/test-whitelist/tests.tsv` 相同：target、完整測試名稱、ignored、features；真正的 Tab 分隔。本張**沒有**改 `tests.tsv`／`lib.txt`／`skills.tsv`（基線套件有固定步數，改清單會改變套件行為），由 #310 決定併入哪一份。

### A 類（17）：純記憶體

```text
native_attack_skills	call_lightning_request_contract::selected_learned_skill_has_native_label_and_rejects_stale_life_world_or_connection	no	-
native_attack_skills	disintegrate_request_contract::selected_learned_skill_has_native_label_and_rejects_stale_life_world_or_connection	no	-
native_attack_skills	captured_c02_disintegrate_lawful352_is_a_rejection_without_authority_or_cast	no	-
native_attack_skills	spell_authority_is_atomic_and_does_not_preempt_arrival_hurt	no	-
native_combat	attack_feedback_distinguishes_reply_timeout_and_interrupted_authority	no	-
native_combat	death_confirmation_requires_authoritative_zero_hp_and_death	no	-
native_combat	formal_melee_vectors_and_atomic_rejection_never_infer_hp	no	-
native_combat	formal_movement_release_does_not_cure_controls_or_predict_actor_state	no	-
native_combat	held_attack_requires_a_new_press_after_even_a_brief_control_interruption	no	-
native_combat	held_attack_retires_world_owner_correction_and_invalid_targets	no	-
native_combat	held_attack_waits_for_authority_and_animation_and_stops_on_release_timeout	no	-
native_combat	orphan_and_malformed_updates_do_not_create_actors	no	-
native_combat	target_range_death_result_and_scoreboard_guards	no	-
native_cone_cold	cold_raw_S23_is_atomic_and_defers_hurt_until_effect_arrival	no	-
native_ground_skills	action18_and_location_effect_are_atomic_without_local_mana_or_damage	no	-
native_ground_skills	ground_choice_is_separate_from_actor_target_and_fenced_by_owner	no	-
native_ground_skills	native_class0_enters_source1_and_dwells_terminal0_before_retirement	no	-
```

### B 類（21）：唯讀讀取 `D:\lineage開發\0-客戶端`

整合測試（12）：

```text
native_attack_skills	class4_launch_freezes_target_and_retires_reused_identity	no	-
native_attack_skills	class4_zero_distance_zero_damage_and_miss_keep_authority	no	-
native_chill_touch	all_eight_banks_frozen_heading_offset_once_sound_and_effect_hit	no	-
native_chill_touch	miss_replacement_and_map_change_cannot_hurt_a_new_target	no	-
native_combat	npc_damage_bursts_preserve_spell_events_authority_and_death_preemption	no	-
native_combat	original_resource_attack_events_fire_once_and_death_holds	no	-
native_fire_arrow	class4_launch_freezes_target_and_retires_reused_identity	no	-
native_ground_skills	ground_effects_are_bounded_deduplicated_and_never_follow_actor_ids	no	-
native_holy_weapon	holy_sound_terminal_dwell_and_map_retirement_do_not_predict_a_buff	no	-
native_holy_weapon	original_holy_definition_is_exact_and_changed_controls_fail_closed	no	-
native_ice_arrow	class5_launch_freezes_target_and_retires_reused_identity	no	-
native_ice_arrow	class5_zero_distance_zero_damage_and_miss_keep_authority	no	-
```

ignored lib（9；`lib.txt` 格式，一行一個全名）：

```text
modern_lito::native_disintegrate_tests::native77_probe::actual_budget_and_late_timer_do_not_catch_up
modern_lito::native_disintegrate_tests::native77_probe::actual_cancel_and_failed_source_old_publication_cannot_relaunch
modern_lito::native_disintegrate_tests::native77_probe::actual_canvas_screen_then_multiply_and_projection_preflight
modern_lito::native_disintegrate_tests::native77_probe::actual_frozen_position_target_life_and_caster_retirement
modern_lito::native_disintegrate_tests::native77_probe::actual_marker_timing_once_normal_and_critical
modern_lito::native_disintegrate_tests::native77_probe::actual_model_rebuild_restores_only_admission_not_pixels_or_callbacks
modern_lito::native_disintegrate_tests::native77_probe::actual_prepare_generation_and_failed_source_retire_before_draw
modern_lito::native_disintegrate_tests::native77_probe::actual_prepared_payload_publication_and_history_once
modern_lito::native_disintegrate_tests::native77_probe::actual_stale_generation_owner_and_life_stamps_never_rebind
```

## 推測

- **C72D 那 1 敗在原始碼層面應該已經修掉**：測試期待 `[221,744,745,749,2510,829,830,831,761,2165,871,754]`，`qualify_skills`（`native_skill_effects.rs:102–191`）的 push 順序與此一致。前提是 `qualify`、`qualify_summon_circle`、`qualify_turn_undead` 各自只回傳上面列的 ID，這三個函式我沒有逐行核對回傳數量。
- **B 類測試和正在啟動的遊戲客戶端可能互相擋檔**：`open_readonly` 的 share mode 是只允許其他程序「讀」。測試握著 pak 的期間，別的程序若要以寫入權限開同一個檔會失敗；反過來也一樣。測試不會改檔，但建議 #310 在沒有客戶端正在啟動時跑。實際會不會撞到沒有驗過。
- B 類的 `class4／class5_zero_distance…` 以 1 ms 步進跑 3001 輪、畫 800×600 canvas，共 3 組。耗時會比其他測試長，沒有量過。

## 待驗證

- **38 個新放行項目的實際結果**：交 #310 在本機實跑。A 類不需要任何前置；B 類需要 `D:\lineage開發\0-客戶端` 存在。
- **C72D 神聖武器那 1 敗是否轉綠**：`native_holy_weapon::original_holy_definition_is_exact_and_changed_controls_fail_closed`，在 #310 一併看。
- **2 個 B′ 項目的閉合**（交 #309）：
  - `native_ground_skills::original_wave_duration_gates_repeated_ground_audio_and_world_reset`。`Model::load`（`native_world_scene.rs:750`）與 `draw_at` 直接引用約 40 個模組。我掃過其中的直接相依：沒有寫檔、網路、視窗、音效裝置；兩處建立執行緒（`native_world_scene.rs:301` 的 `with_model_at_…`、`native_inventory_names.rs:195` 的 `Service::start`）都不在 `load／draw_at` 這條路上；`native_world_text::raster` 用 GDI 在記憶體畫字，不開視窗。第二層以下沒有閉合。
  - lib `…::actual_fragment_submission_uses_screen_multiply_and_remains_click_through`。`world_raster`、`native_world_layers` 往外引用的 6 個模組沒有閉合。
- **腳本的盲點**：巨集產生的呼叫、trait object 的動態分派、relative `self::` 路徑不在掃描範圍；集合封閉檢查之外，我沒有另外找第二種方法交叉驗證。
- 本張審的是 `f8ed20ec`。之後這 9 個測試檔或上述 73 個模組有改動，要重審受影響的項目。

## 沒做的事

- 沒有執行測試、沒有編譯、沒有連任何服務或 DB、沒有碰遊戲程序。
- 沒有改 `src/`、`tests/`、白名單、`TASKS.md`、`docs/REPORT.md`。
- T036 列的其他歷史失敗不在本張的 9 個檔裡：66470 紅測試（不同目錄沿用 Model，屬世界／渲染）、C3 材料技能首輪失敗（`native_reagent_skills`，屬 #308）、房間切換逾時、schema34、Release573、runner 漏 pin。
- `docs/REUSE-MAP.md` 已查：本張是測試副作用審查，沒有需要參考 tw-bin-engine 或 lito-launcher-server 的地方，兩個 repo 都沒有讀。

## 產出路徑

- `docs/reports/issue-307.md`（本檔）
- `維護工具/t036-helper-closure/`：`fnclosure.py`（共用的解析與函式層級閉包）、`showfn.py`、`modcats.py`、`setcheck.py`、`set-skills-combat-1.txt`（本張的 73 個模組）
- 分支 `task/307`

## 給下一棒的提示

- **#308（技能／戰鬥 2／2）**：範圍是上面列的 10 個 target、56 項。本張的 73 模組集合可以直接當起點，多數測試走的是同一批函式（`World::accept`、`native_skills::request`、`Effects::prepare`）；只要把新走到的模組加進集合再跑 `setcheck.py`，看新增的 OUT 行即可。`fnclosure.py` 直接執行時的「同名函式一律納入」閉包太寬（從 `World::accept` 會擴到 1400 多個函式），不要拿它的輸出當證據，用 `setcheck.py` 加人工讀碼。
- **#309（世界／渲染）**：請接手上面 2 個 B′ 項目。場景鏈的入口是 `native_world_scene::Model::load／draw_at`；本張已掃過的直接相依列在「待驗證」。
- **#310（執行）**：
  - 一律用 `--exact` 逐項跑；lib 那 9 項要加 `--ignored`。不要跑整個 target，同檔還有寫檔的測試（歸 #306）。
  - B 類只能在有 `D:\lineage開發\0-客戶端` 的本機跑；GitHub Actions 沒有這個目錄，會在 `unwrap` 失敗。
  - A 類 17 項沒有任何外部前置，要不要加進 CI 可以另外提（目前 CI 只跑 `cargo test --lib`）。
- **#306（檔案寫入）**：上面列的 5 個寫檔測試都寫到 `target/current/verification/`，而且不會自己建目錄；`native_ground_skills` 那一個用的是相對路徑（看執行時的目前目錄），其餘用 `CARGO_MANIFEST_DIR`。
