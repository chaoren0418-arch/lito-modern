# Issue #308 — T036（M3 3/15）跨函式 helper 閉合審查：技能／戰鬥（2／2）

執行者：claude-local（本機，靜態審查）。審查基準：本分支合併 `origin/main` `4b2d603` 之後的樹（合併 commit `6f9e667`）。
**本張沒有編譯、沒有執行任何測試；下面沒有任何一項是「測試通過」。** 本張只回答：這些測試跑起來會碰到什麼（檔案、網路、視窗、音效裝置、子程序、執行緒），所以能不能交給執行張（#311）去跑。

## 結論

- 本張審 **63 個測試**，全部可以放行，沒有不放行的項目。
  - 56 個是 #54 標「跨函式／資源 helper 安全邊界尚未封閉」的技能／戰鬥後半（#307 留給本張的 10 個 target）。其中 1 個在 #167 改過名，其餘 55 個名稱沒變。
  - 7 個是同一批測試檔在 #54 之後新增的（都在 `native_ranged`），一併審了。它們不在 1055 項總帳裡。
- **41 個是純記憶體**（A 類）：任何 Windows 機器都能跑。
- **20 個只多一件事：唯讀讀取 `D:\lineage開發\0-客戶端`**（B 類）：只能在有這個資源根的本機跑。
- **2 個除了唯讀資源根，還會寫固定檔名的證據檔**到 `LiTo381-Modern/target/current/verification/`（V 類）：和 #306 已放行的第 45 項用同一個 helper，條件相同。
- 63 個裡有 13 個早就在 `skills.tsv`（其中 3 個也在 `tests.tsv`），都是 A 類。所以**本張新放行 50 個：A 28、B 20、V 2**。
- 63 個測試的閉包裡**沒有**：網路、子程序、建立視窗、音效裝置、登錄檔、刪除檔案。
- T036 總帳 `維護工具/test-whitelist/full.tsv` 已更新：原本標 `#308 helper-pending` 的 43 列全部改成 `run`（25 列不需前置、18 列 `requires=client`）。總帳仍是 1055 列。

| 分類 | 數量 | 已在白名單 | 本張新放行 | 其中在總帳（已改 `run`） | 不在總帳（#54 之後新增） |
|---|---:|---:|---:|---:|---:|
| A 純記憶體 | 41 | 13 | 28 | 25 | 3 |
| B 唯讀資源根 | 20 | 0 | 20 | 18 | 2 |
| V 資源根＋固定檔名證據 | 2 | 0 | 2 | 0 | 2 |
| 合計 | 63 | 13 | 50 | 43 | 7 |

## 本張範圍

沿用 #307 的切法（它在本 Issue 留言公布）：`native_light_spell`（6）、`native_lightning`（3）、`native_poison_cure`（4）、`native_ranged`（9）、`native_reagent_skills`（5）、`native_skill_selection`（7）、`native_skills`（9）、`native_turn_undead`（6）、`native_vampiric_touch`（4）、`native_wind_cutter`（3），共 56 項。

- 這 10 個檔現在共有 75 個 `#[test]`：56 個 helper 項、12 個 #54 標寫檔的（#306 已審，本張沒有重審）、7 個新增。
- `native_ranged` 的 `malformed_ranged_packets_are_atomic_and_other_effect_branches_remain_unknown` 在 #167（`bd2a423`）改名為 `malformed_ranged_packets_are_atomic_for_every_flight_type`。舊名已不存在；總帳那一列改用現行名稱。
- 7 個新增測試（#167 加的，`native_ranged.rs`）：`e149_resource_mapping_preserves_arrow_and_throwing_vectors`、`physical_class3_types_launch_once_and_release_hurt_only_at_arrival`、`throwing_flights_retire_on_identity_change_and_do_not_hurt_misses_or_dead_targets`、`throwing_stance_preloads_its_own_resources_before_any_attack_cue`、`interleaved_arrow_and_sting_keep_separate_resources_and_arrivals`、`original_throwing_action63_launches_sting2989_through_world_model`、`original_stone_and_energy_resources_reach_model_arrival_and_retirement`。

## 怎麼審的

`docs/REUSE-MAP.md` 已查：本張是本 repo 測試的副作用審查，沒有需要參考 tw-bin-engine 或 lito-launcher-server 的地方，兩個 repo 都沒有讀。本張沒有宣稱缺資源，所以沒有做 pak 查詢。

1. **取範圍**：從 #54 報告的表格機械取出 56 列，和現行 `tests/*.rs` 的 `#[test]` 對照，找出改名的 1 項與新增的 7 項。結果在 `docs/reports/issue-308/scope.tsv`。
2. **讀完 10 個測試檔全文**（2,597 行），逐個測試記下它直接呼叫什麼。
3. **用 #309 的閉包工具算每個測試的呼叫閉包**（`維護工具/audit-test-helper-closure.py`，唯讀 Python，不編譯、不執行專案程式），列出閉包內每個副作用原語的位置。逐位置裁定沿用 #309 的 `site-adjudication.json`，只新增兩條（下表）。63 個測試走到的位置全部有裁定，沒有 unknown。
4. **人工修正工具的分類**。工具是保守上界：觸及一個型別，就把那個型別上同名的方法全算進來；函式內的 `use` 也可能連到同名的別的型別。所以工具標成「唯讀資源根」的 27 項裡，有 7 項實際上不讀檔。我回原始碼讀了被呼叫的函式，確認後改成 A（見「人工下修的 7 項」）。反方向（工具標純記憶體、實際有副作用）沒有發現：工具標 P 的 34 項，本體都只有 `World::accept`、`native_skills::request`、選單與封包解碼，和 #307 A 類走的是同一批函式。
5. **合併 main 後重跑**：push 前 main 前進 9 個 commit，改了閉包內的 `native_world.rs`、`native_world_scene.rs`、`native_inventory*.rs` 等。合併後重跑工具：63 項的分類一項都沒變，副作用位置清單（`sites.tsv`）一行都沒變；只有 5 個走完整場景的測試可達項目數多了 1 到 8 個。10 個測試檔沒有變動。

重跑指令（repo 根目錄，不到一分鐘）：

```powershell
$t = 'native_light_spell,native_lightning,native_poison_cure,native_ranged,native_reagent_skills,native_skill_selection,native_skills,native_turn_undead,native_vampiric_touch,native_wind_cutter'
python -I 維護工具/audit-test-helper-closure.py --targets $t `
  --out scratch/issue-308/closure.json `
  --scope docs/reports/issue-308/scope.tsv `
  --adjudication docs/reports/issue-308/site-adjudication.json `
  --summary-dir docs/reports/issue-308
python -I -B 維護工具/t036-helper-closure/issue308-finalize.py   # 併入人工裁定、重產清單、檢查總帳（不加 --ledger 不寫總帳）
```

第一步會把 `approved-candidates.tsv` 等三份清單寫成工具的原始分類，第二步再覆寫成最終版，兩步要一起跑。

## 已驗證（靜態讀碼與工具輸出，不是測試結果）

### 新增的兩條位置裁定

| 位置 | 內容 | 裁定 |
|---|---|---|
| `tests/native_light_spell.rs:92` | `std::fs::read(root.join("Sound/145.wav"))`，root 寫死為 `D:\lineage開發\0-客戶端`。只讀進記憶體算 SHA-256 與長度 | 唯讀資源根 |
| `tests/native_ranged.rs:160–231` `render_projectile` | `create_dir_all(<CARGO_MANIFEST_DIR>/target/current/verification)`，再寫 `native-projectile<gfx>-<ms>.png`、`native-projectile<gfx>.json`。只有建立與覆寫自己上一次的輸出，沒有刪除、改名、讀回 | 固定檔名證據（同 #306 V 類） |

### 沿用 #309 裁定的位置（本張重讀過的部分）

- `resources::open_readonly`（`resources.rs:864–878`）：`OpenOptions` 只設 `read(true)`；Windows 上 `share_mode(FILE_SHARE_READ)`、`OPEN_REPARSE_POINT`。`reject_reparse_chain`、`verify_open_path`、`final_handle_path`（`:852–939`）只查屬性與最終路徑。
- `audio_output.rs`（waveOut 裝置）出現在 63 項中 17 項的可達檔案裡，但走到的只有 `AudioOutputError`、`DriverError` 兩個錯誤型別和它們的 `fmt`／`source`。`PcmVoice`、`NativeWaveOut`、`Voice::drop` 都不在閉包內，所以不會開音效裝置。這是用工具列出 `native_vampiric_touch::all_eight_banks…` 在該檔的可達項目確認的。
- `native_connection.rs` 出現在 58 項的可達檔案裡，走到的是 `IngressReceipt` 型別（`Instant::now`）。這 10 個檔的 75 個測試，閉包裡都沒有任何 `net` 種類的命中；`fs-write` 命中只有 14 個測試，就是 #306 的 12 項加本張的 V 類 2 項。
- 其餘位置（`native_resources.rs`、`native_world_scene.rs`、`shared_action_catalog.rs`、`native_water_light.rs`、三個 `prepare_with_loader`、`native_world_text::raster` 的 GDI 記憶體 DC、CP950 轉碼、時鐘）本張沒有重讀，直接引用 #309 的裁定。完整清單在 `docs/reports/issue-308/sites.tsv`。

### 人工下修的 7 項（工具標 R，實際是 A）

| 測試 | 依據 |
|---|---|
| `native_ranged::arrival_is_once_at_the_original_30ms_terminal_callback_even_when_frames_skip` | 只呼叫 `Projectiles::default()` 與 `update()`。`update`（`native_projectiles.rs:181–318`）全文讀過，不呼叫 `load`；讀檔只發生在 `prepare`（`:147`）與 `draw_to`（`:336`）經 `load`（`:168`）。`update` 用到的 `RangedPath::new／sample` 是座標運算，五個 `supports_graphics` 是常數比對 |
| `native_ranged::no_hurt_for_miss_zero_damage_death_removed_or_replaced_objects` | 同上 |
| `native_ranged::same_id_replacement_before_launch_cannot_bind_an_old_cue` | 同上 |
| `native_ranged::physical_class3_types_launch_once_and_release_hurt_only_at_arrival`（新增） | 同上 |
| `native_ranged::throwing_flights_retire_on_identity_change_and_do_not_hurt_misses_or_dead_targets`（新增） | 同上 |
| `native_light_spell::light_uses_self_selector_without_predicting_authority_and_scopes_UI_choice`（已在白名單） | 函式內 `use native_skill_selection::{Model,Target}`，工具把 `Model` 連到 `native_world_scene::Model`。本體（`:15–30`）只有 `native_skills::request`、選單與 `Target::capture` |
| `native_skills::invisibility_tick_envelope_guards_clock_and_menu_preserve_authority`（已在白名單） | 本體（`:597–629`）只有 `self_tick_request`、`request`、`ClientTick::now`、`LoginController::default()`、選單，沒有任何路徑參數 |

### 各類走到的東西

- **A（41）**：內嵌 fixture（`include_bytes!`）餵 `World::accept`，再呼叫 `native_skills::request／ground_request／self_tick_request`、`native_skill_selection`、`native_combat::decode_attack／attack_request`、`native_attack_many::decode`、`native_wire_audio::Cursor::take`、`Projectiles::update`。另有三個不經 `World` 的：`native_poison_cure::verified_numeric_comma…`（記憶體內解析 `list.spr` 片段）、`native_reagent_skills::completion_waits…`（`native_combat_probe::reagent_completion_ready`，`native_combat_probe.rs:14–42`，只檢查 JSON 欄位與經過時間；同檔的寫檔函式不在閉包內）、`native_vampiric_touch::server_reuse…`（兩個查表）。命中的原語只有 `Instant::now`、CP950 轉碼。
- **B（20）**：
  - 16 項經 `resources`／`shared_action_catalog` 唯讀載入原資源，在記憶體 canvas 或 fragment 佇列上畫：`Effects::prepare`（`native_skill_effects`、`native_attack_many`、`native_class1`）、`native_spell_projectiles::Projectiles::prepare`、`native_projectiles::Projectiles::prepare／draw`、`LightArrowAssets::load_class4_effect_only`、`SharedArchive::open(...).read_entry("list.spr")`。音效只載成 `PreparedSound`（記憶體）並回傳事件，不播放。
  - 4 項走完整場景 `native_world_scene::Model::load`：`native_light_spell::scene_prepares…`、`native_skill_selection::actual_render_camera…`（這兩項另呼叫 `draw_at`），`native_turn_undead::actual_quest554…`、`actual_quest_wall_rejection…`（載入 map 554 後只用 `navigation()` 算路徑，結果用 `println!` 印到 stdout）。場景鏈的閉合直接引用 #309：命中只有唯讀資源根、GDI 記憶體 DC、行程內載入執行緒。
- **V（2）**：`native_ranged::original_throwing_action63_launches_sting2989_through_world_model`、`original_stone_and_energy_resources_reach_model_arrival_and_retirement`。走完整場景（同上），另寫證據檔。三組檔名以圖形編號區分（2989；1156、2349），和 #306 第 45 項（66）互不覆寫。目錄由 helper 自己建。

### 逐項結果

「工具」欄是工具的保守分類（P 純記憶體、R 唯讀資源根、W 寫檔）；「最終」欄是人工裁定後的分類。完整資料（含可達項目數、來源行號、依據）在 `docs/reports/issue-308/verdicts.tsv`。「白」欄打勾表示已在 `skills.tsv`。

| target | 測試 | 工具 | 最終 | 白 |
|---|---|:-:|:-:|:-:|
| native_light_spell | `direct_sound_cursor_consumes_once_bounds_bursts_and_does_not_replay_across_worlds` | P | A | |
| native_light_spell | `formal_change_light_is_atomic_unsigned_actor_scoped_and_does_not_change_motion_owner` | P | A | |
| native_light_spell | `formal_direct_sound_is_bounded_atomic_and_cleared_with_the_world` | P | A | |
| native_light_spell | `light_uses_self_selector_without_predicting_authority_and_scopes_UI_choice` | R | A | ✓ |
| native_light_spell | `original_light_single_bitmap_retires_on_166ms_callback_without_embedded_sound` | R | B | |
| native_light_spell | `scene_prepares_the_original_direct_145_WAV_once_without_inventing_light_duration` | R | B（場景） | |
| native_lightning | `many_hits_reject_miss_replacement_removal_death_and_new_world` | R | B | |
| native_lightning | `many_packet_is_atomic_bounded_and_preserves_each_unsigned_target_marker` | P | A | |
| native_lightning | `request_clock_reuse_range_and_target_guard_preserve_authority` | P | A | ✓ |
| native_poison_cure | `cure_uses_its_own_timing_and_sound_with_direct_192_graphics_bank` | R | B | |
| native_poison_cure | `object_and_coordinate_envelopes_preserve_authority_and_gate_live_targets` | P | A | ✓ |
| native_poison_cure | `reagent_and_probability_refusals_do_not_invent_a_poison_or_expire_one_locally` | P | A | |
| native_poison_cure | `verified_numeric_comma_does_not_relax_other_delimiters` | P | A | |
| native_ranged | `arrival_is_once_at_the_original_30ms_terminal_callback_even_when_frames_skip` | R | A | |
| native_ranged | `current_jar_far_attack_vectors_preserve_native_unsigned_fields` | P | A | |
| native_ranged | `malformed_ranged_packets_are_atomic_for_every_flight_type`（改名） | P | A | |
| native_ranged | `no_hurt_for_miss_zero_damage_death_removed_or_replaced_objects` | R | A | |
| native_ranged | `projectile_launch_is_once_and_retired_on_actor_removal_or_new_world` | R | B | |
| native_ranged | `ranged_authority_acknowledges_and_queues_launch_without_inventing_hp` | P | A | |
| native_ranged | `ranged_cues_have_a_visible_budget_and_orphan_attackers_do_not_spawn_effects` | P | A | |
| native_ranged | `same_id_replacement_before_launch_cannot_bind_an_old_cue` | R | A | |
| native_ranged | `zero_damage_hit_freezes_live_target_at_launch_and_never_retargets` | R | B | |
| native_ranged | `e149_resource_mapping_preserves_arrow_and_throwing_vectors`（新增） | P | A | |
| native_ranged | `interleaved_arrow_and_sting_keep_separate_resources_and_arrivals`（新增） | R | B | |
| native_ranged | `original_stone_and_energy_resources_reach_model_arrival_and_retirement`（新增） | W | V | |
| native_ranged | `original_throwing_action63_launches_sting2989_through_world_model`（新增） | W | V | |
| native_ranged | `physical_class3_types_launch_once_and_release_hurt_only_at_arrival`（新增） | R | A | |
| native_ranged | `throwing_flights_retire_on_identity_change_and_do_not_hurt_misses_or_dead_targets`（新增） | R | A | |
| native_ranged | `throwing_stance_preloads_its_own_resources_before_any_attack_cue`（新增） | R | B | |
| native_reagent_skills | `completion_waits_for_sampled_effect_and_caster_with_bounded_diagnostics` | P | A | |
| native_reagent_skills | `formal_action19_effect_and_mana_keep_authority_atomic` | P | A | |
| native_reagent_skills | `formal_reagent_request_self_screen_boundary_and_no_prediction` | P | A | ✓ |
| native_reagent_skills | `life_stream_choice_is_owned_ground_and_invalidated_by_correction` | P | A | |
| native_reagent_skills | `reagent_effect_is_fixed_bounded_reset_and_not_an_actor_effect` | R | B | |
| native_skill_selection | `actual_render_camera_ground_cells_and_object_hits_reject_stale_worlds` | R | B（場景） | |
| native_skill_selection | `all_learned_rows_visible_but_unsupported_and_invalid_choices_cannot_cast` | P | A | |
| native_skill_selection | `learned_insert_remove_preserves_id_never_retargets_combo_row` | P | A | |
| native_skill_selection | `owner_change_fences_selection_before_and_after_refresh` | P | A | |
| native_skill_selection | `request_rechecks_learned_mp_hp_and_target_range_without_refresh` | P | A | |
| native_skill_selection | `selection_uses_formal_packets_and_preserves_authority` | P | A | ✓ |
| native_skill_selection | `stale_target_never_becomes_another_actor_or_implicit_self_heal` | P | A | |
| native_skills | `current_book_is_atomic_add_remove_and_survives_map_but_not_new_world_owner` | P | A | |
| native_skills | `detection_original749_frames_sound_once_and_generation_retirement` | R | B | |
| native_skills | `detection_selector_menu_guards_and_cooldown_leave_authority_unchanged` | P | A | ✓ |
| native_skills | `effects_preserve_formal_unsigned_ids_and_are_bounded_atomic_and_map_scoped` | P | A | |
| native_skills | `invisibility_tick_envelope_guards_clock_and_menu_preserve_authority` | R | A | ✓ |
| native_skills | `skill_feedback_requires_authority_and_expires_without_guessing_success` | P | A | |
| native_skills | `spell_requests_use_formal_selectors_and_never_predict_mana` | P | A | ✓ |
| native_skills | `tw_higher_full_heal_group_budget_rejects_both_layers_atomically` | R | B | |
| native_skills | `tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority` | P | A | ✓ |
| native_turn_undead | `actual_quest554_source_spawn_can_route_to_original_ranged_target` | R | B（場景） | |
| native_turn_undead | `actual_quest_wall_rejection_replans_to_a_reachable_casting_line` | R | B（場景） | |
| native_turn_undead | `low_mana_turn_undead_times_out_without_predicted_mana_death_or_rejection` | P | A | ✓ |
| native_turn_undead | `original_754_zero_frames_sound_and_skipped_subframe_are_exact` | R | B | |
| native_turn_undead | `original_boss_summon_circle_definition_and_terminal_are_finite` | R | B | |
| native_turn_undead | `selector_clock_range_and_dead_target_guards_never_predict_death_or_mana` | P | A | ✓ |
| native_vampiric_touch | `all_eight_banks_frozen_heading_offset_once_sound_and_effect_hit` | R | B | |
| native_vampiric_touch | `miss_replacement_and_map_change_cannot_hurt_a_new_target` | R | B | |
| native_vampiric_touch | `server_reuse_and_recovered_client_clock_are_separate` | P | A | |
| native_vampiric_touch | `vampiric_envelope_range_and_authority_do_not_predict_healing_or_hurt` | P | A | ✓ |
| native_wind_cutter | `class4_launch_freezes_target_and_retires_reused_identity` | R | B | |
| native_wind_cutter | `class4_zero_distance_zero_damage_and_miss_keep_authority` | R | B | |
| native_wind_cutter | `formal_attack_spell_layout_and_request_guards` | P | A | ✓ |

### 總帳更新

`維護工具/test-whitelist/full.tsv` 改了 43 列（原本都是 `excluded`／`#308`／`helper-pending`）：verdict 改 `run`、source 維持 `#308`、reason 改 `-`；A 類 25 列 requires 寫 `-`，B 類 18 列寫 `client`。其中 `native_ranged` 改名那一列的名稱欄一併改成現行名稱。

改完後用 `issue308-finalize.py` 重做 `run-local-test-suite.ps1` 的 `Read-FullManifest`／`Resolve-FullPlan` 同一套檢查（1055 列、八欄格式、不重複、`excluded` 有理由、`run` 以外不寫 requires、`excluded` 的不在其他白名單、`run` 的不在 `tests.tsv`），全部通過。這是我用 Python 重寫的檢查，不是執行那支 PowerShell 腳本。

| | 改前（#327） | 改後 |
|---|---:|---:|
| `run` | 111 | 154 |
| 其中 `requires=client` | 76 | 94 |
| `covered` | 38 | 38 |
| `excluded` | 906 | 863 |
| 其中 `helper-pending` | 302 | 259 |

### 歷史失敗：C3 材料技能首輪失敗（NATIVE-LABEL-UPDATES:50）

文件記的是：C3 版本第一輪材料技能的即時 TCP 驗證，在固定等待時間後檢查特效結束沒有過；同一個 EXE 重跑三種情境全過；失敗當下沒有輸出特效收據，根因未確認。

靜態能確認的只有兩點：

- 現行的完成判定是 `native_combat_probe::reagent_completion_ready`（`native_combat_probe.rs:14–42`）：不再是固定等待，而是每次取樣檢查 `active == 0` 且施法者就緒，最多等 10 秒；逾時的錯誤訊息帶經過毫秒數、`caster_ready` 與整份特效收據。文件說的「失敗時沒有收據」在現行程式碼裡已經補上。
- 對應的單元測試是 `native_reagent_skills::completion_waits_for_sampled_effect_and_caster_with_bounded_diagnostics`（A 類，本張放行）。它只測這個判定函式的邏輯。

首輪失敗本身是即時連線探針的行為，要實跑才能重驗，見待驗證。

## 推測

- **工具的 P 類沒有漏報**：這是由「P 類 34 項的本體我都讀過，呼叫的函式和 #307 A 類相同」推出來的。`World::accept` 往下的 280 多個函式我沒有逐一重讀，引用的是 #307 的封閉檢查和這次工具的輸出。
- **B／V 類執行時會擋住別的程序改資源檔**：`open_readonly` 用 `FILE_SHARE_READ`，持有 pak／idx 期間別的程序不能以寫入或刪除權限開同一個檔；反過來對方先以寫入開著，測試會開檔失敗。建議跑的時候不要同時啟動或更新客戶端。沒有實測。
- **耗時較長的項目**（沒有量過）：`native_wind_cutter::class4_zero_distance…` 以 1 ms 步進跑 3 組各 3001 輪、畫 800×600；`native_ranged::physical_class3…` 與 `malformed_ranged…` 各跑上千輪 `serde_json::to_value(&World)`；走完整場景的 6 項（B 4、V 2）要載入整張地圖。
- **呼叫 `draw_at` 的場景項（B 2、V 2）結果應該不取決於主機字型**：它們斷言的是收據欄位、座標、命中框與飛行物事件，不比對整張畫面；但 `Model::draw_at` 內會經過 GDI 文字。沒有驗過。
- C3 首輪失敗的根因仍是文件裡的假設（首次載入與取樣時鐘的差異）。現行的輪詢判定對這種延遲比較寬容，但我沒有證據說它就是原因。

## 待驗證

- **63 項全部沒有實跑。** 放行只代表副作用邊界已閉合，會不會通過要等 #311 在本機逐項執行。B、V 類需要 `D:\lineage開發\0-客戶端`；V 類會在 `LiTo381-Modern/target/current/verification/` 留下 PNG 與 JSON。
- **C3 材料技能首輪失敗**：要以即時連線的材料技能探針重跑（需要 Java peer，屬 T036-10／11 的範圍），確認首輪就通過，或失敗時有收據可查。本張沒有重驗。
- **總帳改動後 `run-local-test-suite.ps1 -Full -Category T036` 與 `test-local-test-suite.ps1` 的實際行為**：我只用 Python 重做了它的清單檢查，兩支腳本都沒有執行。
- **工具的限制**（同 #309）：呼叫解析是自寫的近似，不是編譯器；巨集產生的程式碼只看 `macro_rules!` 本體；外部 crate（`serde`、`png`、`sha2`）當黑箱；`build.rs` 與連結期行為不在範圍。工具回報 1 個解析不到的路徑 `modern_lito::resources`（`resources::read_shared_sound` 的完整路徑寫法），出現在 `native_turn_undead.rs:62`，那是 #306 的寫檔測試，不在本張的 63 項內。
- 本張審到 `6f9e667`。之後這 10 個測試檔或閉包內的模組有改動，要重跑上面的指令看分類有沒有變。

## 沒做的事

- 沒有編譯、沒有執行測試或任何編譯產物、沒有連任何服務或 DB、沒有碰遊戲程序。
- 沒有改 `src/`、`tests/`、`Cargo.*`、`lib.txt`、`tests.tsv`、`skills.tsv`、`TASKS.md`、`docs/REPORT.md`。
- 7 個新增測試沒有加進任何清單（總帳固定 1055 列，不能加列）；要不要進 `tests.tsv` 或 `skills.tsv` 由 #311 決定。
- 這 10 個檔裡 #54 標寫檔的 12 個測試（#306 已放行）沒有重審。
- T036 列的其他歷史失敗不在本張的 10 個檔裡：C72D 神聖武器（#307 已靜態核對）、66470 紅測試、房間切換逾時、schema34、Release573、runner 漏 pin（#309、#310 已處理或轉交）。

## 產出路徑

| 路徑 | 內容 |
|---|---|
| `docs/reports/issue-308.md` | 本檔 |
| `docs/reports/issue-308/verdicts.tsv` | 63 項最終分類：來源（#54／改名／新增）、工具分類、最終分類、是否已在白名單、可達項目數、來源行號、依據 |
| `docs/reports/issue-308/approved-candidates.tsv` | 新放行 A 類 28 項，格式同 `tests.tsv` |
| `docs/reports/issue-308/resource-root-readonly.tsv` | 新放行 B 類 20 項＋V 類 2 項，同格式 |
| `docs/reports/issue-308/still-excluded.tsv` | 空檔（沒有不放行的項目） |
| `docs/reports/issue-308/closure-summary.tsv`、`sites.tsv` | 工具原始輸出：逐測試的保守分類、逐位置的原語與裁定 |
| `docs/reports/issue-308/site-adjudication.json` | 逐位置裁定（#309 的版本加兩條） |
| `docs/reports/issue-308/scope.tsv` | 本張的 63 列範圍 |
| `維護工具/t036-helper-closure/issue308-finalize.py` | 併入人工裁定、產生清單、更新並檢查總帳 |
| `維護工具/test-whitelist/full.tsv`、`README.md` | 總帳 43 列改 `run`；README 補上改後的數字 |

分支 `task/308`。

## 給下一棒的提示

- **#311（執行）**：
  - 本張的清單是 `approved-candidates.tsv`（28）與 `resource-root-readonly.tsv`（22）。其中 43 項已經在總帳標 `run`，`-Full -Category T036` 會直接跑到；剩下 7 項是 `native_ranged` 的新增測試，不在總帳，要另外加進 `tests.tsv` 或 `skills.tsv` 才會被執行。
  - 一律 `--exact` 逐項跑，不要跑整個 target：同檔還有 #306 的寫檔測試。
  - V 類 2 項會自己建 `target/current/verification/`；B 類不寫檔。
  - #309、#310 的放行清單還沒有併進總帳（`helper-pending` 剩 259 列，source 欄寫 `#309`）。併入方式可以照 `issue308-finalize.py` 的做法：只改 verdict、requires、source、reason 四欄，改完跑同一套檢查。
  - 工具分類是上界。#309、#310 標成「唯讀資源根」的項目裡，可能也有只呼叫 `update()`、實際不讀檔的；如果想把更多項目放進不需要客戶端的那一組，值得回頭看。
- **T036-10／11（Java peer 執行）**：C3 材料技能首輪失敗請在材料技能的即時探針重驗；現行判定函式與診斷輸出見上文。
- **之後有人改 `Projectiles::update`**：本張有 5 項是靠「`update` 不讀檔」才歸 A 類、在總帳寫成不需客戶端。如果 `update` 之後會呼叫 `load`，這 5 項要改成 `requires=client`。
