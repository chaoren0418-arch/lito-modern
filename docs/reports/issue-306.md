# Issue #306 — T036（M3 1/15）檔案寫入所有權審查（83 項）

## 結論

83 項全部逐項讀過現行原始碼（基準 `f8ed20e`）。**73 項的寫入所有權已閉合，可以列入放行候選；10 項不放行。本輪沒有執行任何測試，「可放行」只代表寫入目標安全，不代表測試會通過。**

| 類 | 項數 | 寫到哪裡 | 裁定 |
|:-:|---:|---|---|
| V | 50 | `LiTo381-Modern/target/current/verification/` 下的固定檔名證據（JSON／PNG） | 可放行 |
| F | 11 | `target/current/verification/` 下固定名稱的自有合成 fixture 目錄 | 可放行 |
| U | 5 | 每次執行獨占的目錄（名稱含 PID、計數器或時間） | 可放行 |
| T | 6 | `target/current/verification/native-teleport-wire/<Debug\|Release>/<case>.json`，另連自己綁定的 loopback | 可放行 |
| N | 1 | 不寫檔；連向剛釋放的 loopback port | 可放行（殘餘風險見下） |
| L | 10 | **覆寫 git 追蹤的 `logs/continuation-20261003/` 證據檔** | 不放行 |

73 項放行候選裡，55 項會唯讀讀取寫死的 `D:\lineage開發\0-客戶端`，只能在本機跑；18 項不需要客戶端資源。

審查中發現 4 個測試用相對路徑寫檔（寫到哪裡取決於執行時的工作目錄），本張已改成以 `CARGO_MANIFEST_DIR` 為根，改完後才列入 V 類。

## 範圍怎麼來的

`docs/reports/issue-54.md` 的「不明副作用」共 458 項，其中 375 項的理由是「跨函式 helper 未封閉」（屬 T036 的另外 4 張）。剩下 83 項就是本張：

- 76 項：本體或同檔 helper 含 `fs::write`／`File::create`／`create_dir_all`（ignored lib 10、整合 66）；
- 6 項：`native_teleport_controller`（需 `teleport-wire-test`），固定檔名 JSON 覆寫；
- 1 項：`native_network::stalled_seed_can_cancel_and_reconnect_without_stale_owner_or_thread`，釋放 port 後再連線。

#54 的分類基準是 `bb96171`，之後 `tests/` 有變動。83 個函式在現行 main 都還在；兩個函式的行號移動（`native_chill_touch` 61→64、`native_ranged` 132→145），下表一律用現行行號。

## 怎麼審

1. 從 #54 報告的表格機械取出 83 列（`scratch/issue-306/parse54.py`、`sel.py`）。
2. 對每個測試函式取「本體＋同檔可達 helper」的閉包，列出其中所有檔案系統呼叫與路徑字面值（`pertest.py`）。涵蓋的呼叫：`fs::write`、`File::create`、`create_dir(_all)`、`remove_file`、`remove_dir(_all)`、`rename`、`copy`、`OpenOptions`、`set_permissions`、`Command::new`、`temp_dir`、`env::var`、`set_var`、`hard_link`、`symlink`、`set_len`。
3. 另外掃描不是由 `CARGO_MANIFEST_DIR` 接出來的路徑字面值（`rel.py`），找出相對路徑寫入。
4. 掃描 `src/` 非測試程式碼的寫檔函式（`libwriters.py`），確認這些測試會呼叫到的只有 `native_startup::Settings::save`（寫到測試傳入的路徑）。其餘寫檔函式都在 `*_probe.rs`、`native_peer_*`、`preferences.rs`、`rebuild_verification.rs`、`native_backend_485f_entry.rs`，這 83 項沒有呼叫（`native_reagent_skills`、`native_resource_loading` 只引用 `native_combat_probe` 的純判斷函式，而且不在這 83 項的閉包內）。
5. 全庫比對輸出檔名（`dups.py`）：V／F／T 類的輸出檔名沒有兩個測試共用同一個檔。
6. 個案人工讀碼：U 類 5 項、F 類的 `fixture_root`、T 類的 `record`、N 類的連線流程、L 類的輸出位置。

`docs/REUSE-MAP.md` 已查：本張是本 repo 測試的寫檔位置審查，表內沒有對應列，沒有引用 tw-bin-engine 或 lito-launcher-server 的檔案。本張沒有宣稱缺資源，所以沒有做 pak 查詢。

## 各類說明

### V 類（50 項）：固定檔名證據輸出

- 目標都在 `<CARGO_MANIFEST_DIR>/target/current/verification/`（另有 `native_ice_arrow_source` 寫 `target/current/ice-arrow-draft/resource-chain.json`）。整個 `LiTo381-Modern/target/` 不進 git。
- 只有建立與覆寫，沒有刪除、改名、複製到專案外。寫完之後沒有別的測試把這些檔讀回來當輸入。
- 檔名固定，所以同一個 worktree 重跑會覆寫上一次的證據，Debug 與 Release 也互相覆寫。這是證據保存問題，不是所有權問題。
- **37 項自己不建目錄**（表中「自建目錄＝否」）：`target/current/verification/` 不存在時會在寫檔那一行失敗。執行前要先建好這個目錄。
- 與 `維護工具/` 腳本在同一目錄下用的名稱（`local-test-suite/`、`native-E149-*`、`native-teleport-wire/`）沒有重疊。

### F 類（11 項）：固定名稱的自有合成 fixture

- `native_resource_loading` 8 項用 `fixture_root(<名稱>)` 建 `target/current/verification/resource-loading/<名稱>/`，11 個名稱互不相同；`native_world_lighting_scene` 3 項各建一個空目錄（`resource-loading/presentation-other-root`、`resource-loading/job-other-root`、`ordinary-admission-v2/source-B-invalid`）。
- 寫入的是測試自己合成的小型 `.idx`／`.pak`／`.s32`。測試裡「以寫入模式開檔應該失敗」的斷言，對象也是這些自己建的檔。
- 不清理。`retained_native_geometry_keeps_sector_identity_locks_and_rejects_new_sources` 中途失敗時可能留下 `80007fff.s32`，下次重跑會受影響；重跑前請清掉該測試自己的 fixture 目錄。
- `native_world_lighting_scene` 3 項另外會唯讀載入客戶端資源。

### U 類（5 項）：每次執行獨占的目錄

| 測試 | 目錄 | 清理 | 備註 |
|---|---|---|---|
| `native_inventory_icons::corrupt_primary_archive_cannot_be_treated_as_base_archive_miss` | `target/current/scratch/icon-corrupt-<PID>/` | 自己刪（逐檔、非遞迴） | 用 `create_dir`：父目錄 `target/current/scratch/` 要先存在；同名目錄已存在會直接失敗，不會覆寫。從客戶端複製 `LiTo/LTicon.pak`（唯讀） |
| `native_inventory_names::corrupt_primary_archive_is_an_error_and_cannot_be_treated_as_a_base_miss` | `target/current/scratch/name-corrupt-<PID>/` | 自己刪 | 同上，複製 `LiTo/LTtext.pak` |
| `native_npc_images::absent_image_archive_preserves_original_text_actions_and_reports_the_asset_error` | `target/current/npc-images-missing-<PID>-<奈秒>/` | 不清理 | 每跑一次留下最多 2 MB 的 `LTtext.idx/.pak` 複本 |
| `native_light_mask::loader_uses_only_the_named_base_resource_and_checks_hash_and_cancellation` | `target/current/scratch/native-light-mask-loader-<PID>/` | 自己刪 | 不讀客戶端 |
| `native_startup::bounded_file_save_replaces_atomically_and_read_is_unchanged` | `target/current/native-startup-tests/<PID>-<計數>/` | 不清理 | `Settings::save` 只寫測試給的路徑；測試還會斷言工作目錄下的設定檔沒有被動到 |

### T 類（6 項）：`native_teleport_controller`

- 六個 wrapper 呼叫 `src/modern_lito/native_teleport_controller_tests.rs` 的同名函式。寫檔只有 `record()`（543–552 行）：`create_dir_all` 後寫 `target/current/verification/native-teleport-wire/<Debug|Release>/<case>.json`，case 名稱互不相同。
- 網路只用 `FaultPeer::new()` 綁定的 `127.0.0.1:0`，listener 由測試持有。
- 同一批函式在 `cargo test --lib` 下也是測試（`#[cfg_attr(test, test)]`），所以 CI 每次都在跑同樣的本體與同樣的寫檔。main 的 CI run 43195895110（`3496f98`）六案都是 `ok`。這只證明 lib 版本在 CI 通過；以 `--features teleport-wire-test --test native_teleport_controller` 跑的整合版本沒有執行過。
- `維護工具/native_E149_component_diagnostics.py` 會讀這個目錄，是既有的消費端。

### N 類（1 項）：`native_network::stalled_seed_…`

- 沒有任何檔案寫入。
- #54 排除的理由是第二段（162–165 行）：綁 `127.0.0.1:0`、取得 port、釋放，再連那個 port，預期連線失敗。
- 讀了 `native_connection.rs:133–169`：客戶端連上後先等對方送 seed frame，收到合法 seed 之前不送任何資料；之後也只會送一個版本請求，沒有帳號或密碼。
- 所以最壞情況是：釋放與重連之間，本機另一個程序剛好拿到同一個 port 並接受連線。這時對方收不到任何資料，測試會因為階段不是 `Failed` 而失敗。對象限定 127.0.0.1。我判斷可以放行；審查者若不接受這個殘餘風險，把這一項留在排除即可。

### L 類（10 項）：不放行

這 10 個 ignored lib 測試結束時會覆寫 `LiTo381-Modern/logs/continuation-20261003/` 下 **git 追蹤中**的檔案：

- 10 個 `*-details.json`（r429、r430×2、r431×2、r432×2、r433、r434、r435）；
- `r429-native-action-slices/` 與 `r430-native-prince-slices/` 的 PNG（追蹤中共 24 張）。

另外兩項還會寫 `target/r434e-conditional-event-cpu/`、`target/r435c-scene-cpu/`（不進 git，沒有問題）。

不放行的理由：

1. 執行後工作區會變髒，追蹤中的歷史證據被改寫（輸出含絕對路徑與雜湊，換機器幾乎一定不同）。
2. 這些測試同時依賴原機 snapshot（`r4xx-*-snapshot.json` 內記的絕對路徑）與未追蹤的 `target/r425-native-dm564-fixture`，前置條件本來就不成立。

全庫搜尋的結果是：沒有其他測試讀這 10 個 details 檔，它們只是輸出。

## 本張的程式變更

4 個測試原本用相對路徑寫檔，實際位置取決於執行時的工作目錄（`cargo test` 會把工作目錄設成套件根目錄，但直接執行測試 EXE 時不是）。改成以 `CARGO_MANIFEST_DIR` 為根，和其他測試一致：

| 檔案 | 測試 | 變更 |
|---|---|---|
| `tests/native_minimap.rs:52` | `original_geometry_overlay_filters_hidden_markers_and_blocks_world_hit_tests` | 輸出目錄改接 `CARGO_MANIFEST_DIR` |
| `tests/native_ground_pickup.rs:223` | `formal_pickup_action15_uses_original_frames_then_returns_to_idle` | 同上 |
| `tests/native_ground_skills.rs:207、222` | `original_resource_eight_segments_terminal_dwell_and_same_id_sound` | PNG 與 JSON 都改；JSON 內記錄的 `png` 欄位仍是原來的相對字串 |
| `tests/native_reagent_skills.rs:340` | `world_scene_prepares_original_sound_once_and_preserves_server_state` | 同上 |

沒有改斷言、沒有改輸出檔名、沒有改任何 `src/`。

## 逐項表（83 項）

類別代號見「結論」。「寫入／刪除目標」是閉包內的路徑字面值；只寫檔名的項目都接在同列的目錄之下。「讀客戶端」指閉包內引用寫死的 `D:\lineage開發\0-客戶端`（唯讀）。

| # | target | 測試 | 現行位置 | 類 | 寫入／刪除目標 | 讀客戶端 | 自建目錄 | 裁定 |
|---:|---|---|---|:-:|---|:-:|:-:|---|
| 1 | lib | `fixed_append_only_sql_catalog_actor_and_discovery_consumer` | `src/modern_lito/native_wire_capture_tests.rs:59` | L | `logs/continuation-20261003/r433-append-only-consumer-details.json` | 否 | 否 | 不放行 |
| 2 | lib | `fixed_actor_authoritative_body_and_opaque_partial_completion` | `src/modern_lito/native_actor_dependency_draft_tests.rs:36` | L | `logs/continuation-20261003/r432-actor-body-details.json` | 否 | 否 | 不放行 |
| 3 | lib | `fixed_actor_protocol_and_publication_consumer` | `src/modern_lito/native_actor_dependency_draft_tests.rs:20` | L | `logs/continuation-20261003/r432-actor-publication-details.json` | 否 | 否 | 不放行 |
| 4 | lib | `frozen_service_publications_native_firepot_ticks_depth_and_partial_completion` | `src/modern_lito/native_renderer_slice_tests.rs:154` | L | `logs/continuation-20261003/r429-native-action-slices`<br>`logs/continuation-20261003/r429-native-renderer-slice-details.json` | 否 | 是 | 不放行 |
| 5 | lib | `frozen_sql_prince_body_standing_moving_depth_native_integer_milliseconds` | `src/modern_lito/native_renderer_slice_tests.rs:101` | L | `logs/continuation-20261003/r430-native-prince-slice-details.json`<br>`logs/continuation-20261003/r430-native-prince-slices` | 否 | 是 | 不放行 |
| 6 | lib | `frozen_sql_publication_owner_and_actor_requirements` | `src/modern_lito/native_renderer_slice_tests.rs:46` | L | `logs/continuation-20261003/r430-sql-publication-details.json` | 否 | 否 | 不放行 |
| 7 | lib | `fixed_catalog_publications_proofs_owner_and_partial_completion` | `src/modern_lito/native_catalog_draft_tests.rs:71` | L | `logs/continuation-20261003/r431-native-catalog-details.json` | 否 | 否 | 不放行 |
| 8 | lib | `fixed_sql_catalog_owner_generation_proofs_and_equipment_publications` | `src/modern_lito/native_catalog_draft_tests.rs:130` | L | `logs/continuation-20261003/r431-native-catalog-sql-details.json` | 否 | 否 | 不放行 |
| 9 | lib | `real_assets_conditional_event_sequence_and_opaque_partial` | `src/modern_lito/native_event_scheduler_tests.rs:70` | L | `logs/continuation-20261003/r434-conditional-event-details.json`<br>`target/r434e-conditional-event-cpu` | 否 | 是 | 不放行 |
| 10 | lib | `real_native_scene_interleaving_and_owner_bound_partial` | `src/modern_lito/native_scene_composition_tests.rs:44` | L | `logs/continuation-20261003/r435-scene-consumer-details.json`<br>`target/r435c-scene-cpu` | 否 | 是 | 不放行 |
| 11 | native_actor_layers | `body_image_and_subframe_bind_real_layers_across_actions_and_directions` | `tests/native_actor_layers.rs:102` | V | `target/current/verification/native-actor-layer-coverage.json` | 是 | 否 | 可放行 |
| 12 | native_actor_layers | `inspect_actual_actor_layer_catalog` | `tests/native_actor_layers.rs:7` | V | `target/current/verification/native-actor-layer-catalog.json` | 是 | 否 | 可放行 |
| 13 | native_actor_layers | `original_body_attachment_shadow_composition_matches_independent_rgb555_and_hit_surfaces` | `tests/native_actor_layers.rs:218` | V | `native-actor-layer-reference-{index}.png`<br>`native-actor-layer-reference.json`<br>`target/current/verification` | 是 | 否 | 可放行 |
| 14 | native_actor_states | `catalogue_half_body_preserves_attachment_style_depth_and_pixel_ownership` | `tests/native_actor_states.rs:314` | V | `native-body-half-reference.json`<br>`native-body-half-{}.png`<br>`target/current/verification` | 是 | 否 | 可放行 |
| 15 | native_actor_states | `original_body_attachment_shadow_states_match_independent_pixels_and_owners` | `tests/native_actor_states.rs:183` | V | `native-actor-state-{index}.png`<br>`native-actor-states-reference.json`<br>`target/current/verification` | 是 | 否 | 可放行 |
| 16 | native_animation_resources | `inspect_actual_world_animation_definitions` | `tests/native_animation_resources.rs:6` | V | `target/current/verification/869-original.png`<br>`target/current/verification/native-animation-definitions.json` | 是 | 否 | 可放行 |
| 17 | native_attack_skills | `original_class4_flight_impact_audio_and_hurt_are_once` | `tests/native_attack_skills.rs:87` | V | `target/current/verification/native-bolt-{ms}.png`<br>`target/current/verification/native-bolt.json` | 是 | 否 | 可放行 |
| 18 | native_chill_touch | `common_finite_clock_quantizes_birth_and_dispatches_late_hit_once` | `tests/native_chill_touch.rs:64` | V | `target/current/verification/native-common-finite-class1-clock.json` | 是 | 否 | 可放行 |
| 19 | native_city_resources | `original_rental_city_has_bounded_complete_geometry` | `tests/native_city_resources.rs:4` | V | `target/current/verification/rental-city-geometry.json` | 是 | 是 | 可放行 |
| 20 | native_clan_markers | `metadata_marker_overlay_keeps_authority_and_all_pixel_targets` | `tests/native_clan_markers.rs:138` | V | `native-clan-marker-world.json`<br>`native-clan-marker-world.png`<br>`target/current/verification` | 是 | 是 | 可放行 |
| 21 | native_clan_markers | `seven_original_images_match_scalar_pixels_with_clipping_and_cancellation` | `tests/native_clan_markers.rs:91` | V | `native-clan-marker-{phase}-{placement}.png`<br>`native-clan-markers.json`<br>`target/current/verification` | 是 | 是 | 可放行 |
| 22 | native_effect_lifecycle | `native_effect_quantizes_common_clock_and_preserves_late_frames_without_world_changes` | `tests/native_effect_lifecycle.rs:358` | V | `target/current/verification/native-class0-clock-samples.json` | 是 | 否 | 可放行 |
| 23 | native_effect_lifecycle | `original_class0_resources_match_every_boundary_including_terminal_dwell` | `tests/native_effect_lifecycle.rs:137` | V | `target/current/verification/native-effect-catalog.json` | 是 | 否 | 可放行 |
| 24 | native_effect_lifecycle | `scene_publishes_original_terminal_frames_then_retires_without_changing_authority` | `tests/native_effect_lifecycle.rs:253` | V | `target/current/verification/native-effect-terminal-{now}.png`<br>`target/current/verification/native-effect-terminal.json` | 是 | 否 | 可放行 |
| 25 | native_fire_arrow | `original_class4_flight_impact_audio_and_hurt_are_once` | `tests/native_fire_arrow.rs:95` | V | `target/current/verification/native-fire-arrow-{ms}.png`<br>`target/current/verification/native-fire-arrow.json` | 是 | 否 | 可放行 |
| 26 | native_ground_pickup | `formal_pickup_action15_uses_original_frames_then_returns_to_idle` | `tests/native_ground_pickup.rs:188` | V | `target/current/verification/native-ground-pickup-action.json` | 是 | 否 | 可放行 |
| 27 | native_ground_skills | `original_resource_eight_segments_terminal_dwell_and_same_id_sound` | `tests/native_ground_skills.rs:153` | V | `target/current/verification/native-wall-{ms}.png`<br>`target/current/verification/native-wall.json` | 是 | 否 | 可放行 |
| 28 | native_ice_arrow | `original_class5_flight_impact_audio_and_hurt_are_once` | `tests/native_ice_arrow.rs:101` | V | `target/current/verification/native-ice-arrow-{ms}.png`<br>`target/current/verification/native-ice-arrow.json` | 是 | 否 | 可放行 |
| 29 | native_ice_arrow_source | `inspect_original_ice_arrow_chain` | `tests/native_ice_arrow_source.rs:5` | V | `resource-chain.json`<br>`target/current/ice-arrow-draft` | 是 | 是 | 可放行 |
| 30 | native_inventory_icons | `corrupt_primary_archive_cannot_be_treated_as_base_archive_miss` | `tests/native_inventory_icons.rs:157` | U | `target/current/scratch/icon-corrupt-{}` | 是 | 是 | 可放行 |
| 31 | native_inventory_icons | `original_samples_and_every_present_lito_icon_match_independent_reference` | `tests/native_inventory_icons.rs:76` | V | `native-inventory-icon-{id}.png`<br>`native-inventory-icons.json`<br>`target/current/verification` | 是 | 是 | 可放行 |
| 32 | native_inventory_names | `corrupt_primary_archive_is_an_error_and_cannot_be_treated_as_a_base_miss` | `tests/native_inventory_names.rs:170` | U | `target/current/scratch/name-corrupt-{}` | 是 | 是 | 可放行 |
| 33 | native_inventory_names | `original_tables_use_lito_first_base_miss_fallback_and_known_cp950_names` | `tests/native_inventory_names.rs:103` | V | `target/current/verification/native-inventory-names.json` | 是 | 否 | 可放行 |
| 34 | native_item_effects | `original_potion_effects_follow_class0_frames_terminal_dwell_and_retire` | `tests/native_item_effects.rs:11` | V | `native-item-effect-{tick}.png`<br>`native-item-effects.json`<br>`target/current/verification` | 是 | 是 | 可放行 |
| 35 | native_item_effects | `original_type9_ground_items_are_single_frame_static_sprites` | `tests/native_item_effects.rs:99` | V | `native-ground-items.json`<br>`native-ground-items.png`<br>`target/current/verification` | 是 | 否 | 可放行 |
| 36 | native_label_updates | `received_title_reflows_existing_hover_without_changing_hit_surface` | `tests/native_label_updates.rs:121` | V | `native-label-update-title.json`<br>`native-label-update-title.png`<br>`target/current/verification` | 是 | 否 | 可放行 |
| 37 | native_light_mask | `loader_uses_only_the_named_base_resource_and_checks_hash_and_cancellation` | `tests/native_light_mask.rs:163` | U | `Sprite.idx`<br>`Sprite.pak`<br>`target/current/scratch` | 否 | 是 | 可放行 |
| 38 | native_light_spell | `original_light_definition_and_149_bitmap_are_inspected_without_alias_guessing` | `tests/native_light_spell.rs:103` | V | `target/current/verification/native-light-selected-source.json` | 是 | 否 | 可放行 |
| 39 | native_lightning | `all_eight_lightning_banks_match_independent_pixels_origin_and_multi_target_hit_once` | `tests/native_lightning.rs:38` | V | `target/current/verification/lightning-sprite-sources.json` | 是 | 否 | 可放行 |
| 40 | native_lightning | `many_common_clock_quantizes_birth_and_late_dispatch_without_replaying_hits` | `tests/native_lightning.rs:70` | V | `target/current/verification/native-common-finite-many-clock.json` | 是 | 否 | 可放行 |
| 41 | native_minimap | `original_geometry_overlay_filters_hidden_markers_and_blocks_world_hit_tests` | `tests/native_minimap.rs:43` | V | `original-geometry-ally.png`<br>`target/current/verification/native-minimap` | 是 | 是 | 可放行 |
| 42 | native_network | `stalled_seed_can_cancel_and_reconnect_without_stale_owner_or_thread` | `tests/native_network.rs:137` | N | 無 | 否 | 否 | 可放行 |
| 43 | native_npc_images | `absent_image_archive_preserves_original_text_actions_and_reports_the_asset_error` | `tests/native_npc_images.rs:34` | U | `target/current/npc-images-missing-{}-{}` | 是 | 是 | 可放行 |
| 44 | native_obstacles | `original_guardian_assets_render_healthy_damaged_and_held_destroyed_states` | `tests/native_obstacles.rs:37` | V | `target/current/verification/guardian-assets.json` | 是 | 否 | 可放行 |
| 45 | native_ranged | `original_arrow_follows_actor_hit_event_and_finishes_without_hp_mutation` | `tests/native_ranged.rs:145` | V | `native-projectile{projectile_graphics}-{ms}.png`<br>`native-projectile{projectile_graphics}.json`<br>`target/current/verification` | 是 | 是 | 可放行 |
| 46 | native_reagent_skills | `explicit_modern_original_action3_once_has_nineteen_frames_and_one_sound` | `tests/native_reagent_skills.rs:204` | V | `native-reagent-effect-{tick}.png`<br>`native-reagent-effects.json`<br>`target/current/verification` | 是 | 是 | 可放行 |
| 47 | native_reagent_skills | `world_scene_prepares_original_sound_once_and_preserves_server_state` | `tests/native_reagent_skills.rs:312` | V | `target/current/verification/native-reagent-audio.json` | 是 | 否 | 可放行 |
| 48 | native_rental_routes | `formal_rental_npcs_are_reachable_on_original_geometry_without_crossing_actors` | `tests/native_rental_routes.rs:4` | V | `target/current/verification/rental-npc-routes.json` | 是 | 是 | 可放行 |
| 49 | native_resource_loading | `cancelled_shared_source_wait_does_not_retire_the_live_owner` | `tests/native_resource_loading.rs:164` | F | `target/current/verification/resource-loading`<br>`{stem}.idx`<br>`{stem}.pak` | 否 | 是 | 可放行 |
| 50 | native_resource_loading | `marker_load_profile_keeps_identical_sources_across_fresh_banks` | `tests/native_resource_loading.rs:454` | V | `native-marker-load-profile.json`<br>`target/current/verification` | 是 | 是 | 可放行 |
| 51 | native_resource_loading | `observed_archive_lifetime_denies_writes_to_all_four_sources` | `tests/native_resource_loading.rs:63` | F | `target/current/verification/resource-loading`<br>`{stem}.idx`<br>`{stem}.pak` | 否 | 是 | 可放行 |
| 52 | native_resource_loading | `observed_native_archives_keep_pak_handles_and_reject_drifted_plans` | `tests/native_resource_loading.rs:23` | F | `target/current/verification/resource-loading`<br>`{prefix}.idx`<br>`{prefix}.pak` | 否 | 是 | 可放行 |
| 53 | native_resource_loading | `observed_surface_pins_and_override_rejections_survive_sparse_lookup` | `tests/native_resource_loading.rs:382` | F | `Sprite.idx`<br>`Sprite{shard:02}.idx`<br>`target/current/verification/resource-loading`<br>`{stem}.idx`<br>`{stem}.pak` | 否 | 是 | 可放行 |
| 54 | native_resource_loading | `retained_archive_validation_and_failed_windows_keep_the_same_source_owner` | `tests/native_resource_loading.rs:84` | F | `target/current/verification/resource-loading`<br>`{stem}.idx`<br>`{stem}.pak` | 否 | 是 | 可放行 |
| 55 | native_resource_loading | `retained_native_geometry_keeps_sector_identity_locks_and_rejects_new_sources` | `tests/native_resource_loading.rs:240` | F | `moved.s32`<br>`target/current/verification/resource-loading` | 否 | 是 | 可放行 |
| 56 | native_resource_loading | `selected_index_preserves_reads_and_checks_every_unselected_record` | `tests/native_resource_loading.rs:280` | F | `Sprite.idx`<br>`Sprite.pak`<br>`target/current/verification/resource-loading` | 否 | 是 | 可放行 |
| 57 | native_resource_loading | `shared_native_sources_release_only_after_the_last_owner_and_respect_limits` | `tests/native_resource_loading.rs:126` | F | `target/current/verification/resource-loading`<br>`{stem}.idx`<br>`{stem}.pak` | 否 | 是 | 可放行 |
| 58 | native_skills | `legacy_higher_heal_aliases_keep_original744_pixels_wire_ids_and_individual_sounds` | `tests/native_skills.rs:497` | V | `native-legacy-higher-heal-frames.json`<br>`native-legacy-higher-heal-original-definitions.json`<br>`target/current/verification` | 是 | 是 | 可放行 |
| 59 | native_skills | `original_tw_basic_heal_frames_sound_clock_and_retirement_preserve_authority` | `tests/native_skills.rs:243` | V | `target/current/verification/native-TW11722-original-frame-metadata.json`<br>`target/current/verification/native-shared-basic-heal-frames.json` | 是 | 否 | 可放行 |
| 60 | native_skills | `source_heal_and_shield_effects_play_once_independently_of_actor_replacement` | `tests/native_skills.rs:161` | V | `target/current/verification/native-skills-{ms}.png`<br>`target/current/verification/native-skills.json` | 是 | 否 | 可放行 |
| 61 | native_skills | `tw_higher_heal_original_slots_clothes_sound_and_retirement_preserve_authority` | `tests/native_skills.rs:330` | V | `target/current/verification/native-shared-higher-heal-frames.json` | 是 | 否 | 可放行 |
| 62 | native_stalac_layers | `original_stalac_body_shadow_and_smoke_share_every_subframe` | `tests/native_stalac_layers.rs:6` | V | `target/current/verification/stalac-layer-preflight.json` | 是 | 否 | 可放行 |
| 63 | native_startup | `bounded_file_save_replaces_atomically_and_read_is_unchanged` | `tests/native_startup.rs:73` | U | `oversized.json`<br>`target/current/native-startup-tests` | 否 | 是 | 可放行 |
| 64 | native_terrain_residence | `original_siege563_residence_reloads_at_sector_boundary_and_retires_old_input` | `tests/native_terrain_residence.rs:35` | V | `target/current/verification/native-terrain-residence` | 是 | 是 | 可放行 |
| 65 | native_turn_undead | `original_turn_undead_frames_pixels_sound_once_and_zero_terminal_retirement_after_death` | `tests/native_turn_undead.rs:43` | V | `target/current/verification/turn-undead-original-sources.json` | 是 | 否 | 可放行 |
| 66 | native_type12_prop | `model_uses_original_catalogue_half_without_mutating_world_or_animation` | `tests/native_type12_prop.rs:22` | V | `target/current/verification/type12-869-model-half.json` | 是 | 否 | 可放行 |
| 67 | native_type12_prop | `original_type12_is_a_single_image_not_an_invented_breath_action` | `tests/native_type12_prop.rs:6` | V | `target/current/verification/type12-869-body.json` | 是 | 否 | 可放行 |
| 68 | native_wind_cutter | `original_class4_flight_impact_audio_and_hurt_are_once` | `tests/native_wind_cutter.rs:101` | V | `target/current/verification/native-wind-cutter-{ms}.png`<br>`target/current/verification/native-wind-cutter.json` | 是 | 否 | 可放行 |
| 69 | native_world_effects | `actual_effect_fragments_match_scalar_pixels_occlusion_and_click_through_owners` | `tests/native_world_effects.rs:49` | V | `native-world-effect-{index}.png`<br>`native-world-effects-reference.json`<br>`target/current/verification` | 是 | 否 | 可放行 |
| 70 | native_world_labels | `real_world_labels_are_click_through_and_do_not_mutate_authority` | `tests/native_world_labels.rs:87` | V | `native-world-labels-{name}.json`<br>`native-world-labels-{name}.png`<br>`target/current/verification` | 是 | 否 | 可放行 |
| 71 | native_world_layers | `actual_terrain_body_and_ground_fragments_match_independent_pixels_and_owners` | `tests/native_world_layers.rs:74` | V | `native-layer-reference-{index}.png`<br>`native-layer-reference.json`<br>`target/current/verification` | 是 | 否 | 可放行 |
| 72 | native_world_lighting_scene | `admission_lifecycle_invalid_source_ABA_retires_actor_and_wire_cues` | `tests/native_world_lighting_scene.rs:883` | F | `target/current/verification/ordinary-admission-v2/source-B-invalid` | 是 | 是 | 可放行 |
| 73 | native_world_lighting_scene | `presentation_job_rejects_reused_model_from_other_resource_root` | `tests/native_world_lighting_scene.rs:618` | F | `target/current/verification/resource-loading/job-other-root` | 是 | 是 | 可放行 |
| 74 | native_world_lighting_scene | `presentation_root_change_retires_old_model_and_source_leases` | `tests/native_world_lighting_scene.rs:554` | F | `target/current/verification/resource-loading/presentation-other-root` | 是 | 是 | 可放行 |
| 75 | native_world_roofs | `real_hidden_roof_groups_use_source_565_without_fixture_substitution` | `tests/native_world_roofs.rs:324` | V | `native-roof-hidden-reference-{index}.png`<br>`native-roof-hidden-reference.json`<br>`target/current/verification` | 是 | 否 | 可放行 |
| 76 | native_world_roofs | `real_roof_groups_and_source_pixels_match_scalar_composition_and_owners` | `tests/native_world_roofs.rs:140` | V | `native-roof-reference-{index}.png`<br>`native-roof-reference.json`<br>`target/current/verification` | 是 | 否 | 可放行 |
| 77 | native_world_text | `actual_cp950_font_raster_is_cached_bounded_and_cancellable` | `tests/native_world_text.rs:5` | V | `native-world-text-reference.json`<br>`native-world-text-{i}.png`<br>`target/current/verification` | 否 | 否 | 可放行 |
| 78 | native_teleport_controller | `cancel_discards_old_inbound_burst_before_fresh_login` | `src/modern_lito/native_teleport_controller_tests.rs:634` | T | `target/current/verification/native-teleport-wire`<br>`{name}.json` | 否 | 是 | 可放行 |
| 79 | native_teleport_controller | `coalesced_old_socket_burst_cannot_retarget_or_repeat_quit` | `src/modern_lito/native_teleport_controller_tests.rs:574` | T | `target/current/verification/native-teleport-wire`<br>`{name}.json` | 否 | 是 | 可放行 |
| 80 | native_teleport_controller | `invalid_transport_envelopes_during_teleport_retire_scope` | `src/modern_lito/native_teleport_controller_tests.rs:645` | T | `target/current/verification/native-teleport-wire`<br>`{name}.json` | 否 | 是 | 可放行 |
| 81 | native_teleport_controller | `malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner` | `src/modern_lito/native_teleport_controller_tests.rs:592` | T | `target/current/verification/native-teleport-wire`<br>`{name}.json` | 否 | 是 | 可放行 |
| 82 | native_teleport_controller | `new_destination_does_not_accept_previous_endpoint_as_a_new_handoff` | `src/modern_lito/native_teleport_controller_tests.rs:622` | T | `target/current/verification/native-teleport-wire`<br>`{name}.json` | 否 | 是 | 可放行 |
| 83 | native_teleport_controller | `retired_endpoint_duplicate_and_old_world_packets_do_not_change_quit_barrier` | `src/modern_lito/native_teleport_controller_tests.rs:555` | T | `target/current/verification/native-teleport-wire`<br>`{name}.json` | 否 | 是 | 可放行 |

## 已驗證

- 83 項的範圍與 #54 報告一致（76＋6＋1），函式在現行 main 都存在。
- 73 項放行候選的所有寫入、刪除目標都在 `LiTo381-Modern/target/` 之下；沒有寫入客戶端目錄、`logs/`、`tests/fixtures/`、專案外路徑，沒有遞迴刪除，沒有啟動子程序，沒有讀寫環境變數。依據是上面的靜態掃描與人工讀碼。
- 10 項 L 類會覆寫的檔案確實在 git 追蹤中（`git ls-files` 核對）。
- T 類六案的 lib 版本在 main 的 CI run 43195895110 為 `ok`。
- 本張 4 個檔案的修改：本機 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_minimap --test native_ground_pickup --test native_ground_skills --test native_reagent_skills` 結束碼 0（`Finished dev profile … in 1m 40s`），四個檔案沒有錯誤或新警告。只有編譯，沒有執行。

## 推測

- 55 項會讀客戶端的測試，是經由 `SharedArchive` 等載入器開檔。`resources.rs:873` 的共享模式是只允許他人讀取，所以測試持有檔案期間，其他程序無法以寫入模式開同一個 pak／idx。推測在測試進行時啟動或更新遊戲客戶端可能受影響；我沒有逐項確認哪些載入器會長時間持有檔案。這屬於「共享資源根」那兩張的範圍。
- `native_world_text::actual_cp950_font_raster_is_cached_bounded_and_cancellable` 用 GDI 取主機字型（不建視窗）。推測在沒有繁中字型的機器（例如 CI）上結果會不同。
- N 類的殘餘風險在實務上很低，但我沒有量測 Windows 重新配發同一個臨時 port 的機率。

## 待驗證

- 73 項放行候選**都沒有執行過**（T 類只有 lib 版本在 CI 跑過）。是否通過，要等執行那一張在本機以 `--exact` 逐項跑。
- 本張改的 4 個測試需要客戶端資源，CI 只會編譯、不會執行；修改後的行為要在本機執行時確認輸出檔出現在 `LiTo381-Modern/target/current/verification/`。
- L 類 10 項維持排除，沒有任何執行結果。

## 產出路徑

- `docs/reports/issue-306.md`（本檔）
- `docs/reports/issue-306/file-write-ownership.tsv`：83 列全表（裁定、類別、是否讀客戶端、是否自建目錄、現行位置、寫入目標）
- `docs/reports/issue-306/release-candidates.tsv`：73 列放行候選，四欄格式同 `維護工具/test-whitelist/tests.tsv`（target、全名、ignored、features）。**還沒有併入白名單**，由審查者決定後再併。
- `LiTo381-Modern/tests/native_minimap.rs`、`native_ground_pickup.rs`、`native_ground_skills.rs`、`native_reagent_skills.rs`
- 分析腳本與中間檔在 `scratch/issue-306/`（不進 git）

## 給下一棒的提示

- **執行那一張**：
  - 先建 `LiTo381-Modern/target/current/verification/` 與 `LiTo381-Modern/target/current/scratch/`，否則 37 項 V 類與 2 項 U 類會在寫檔時失敗，看起來像測試失敗。
  - 一律 `--exact` 逐項跑；T 類要加 `--features teleport-wire-test --test native_teleport_controller`。
  - 同一個 worktree 不要同時跑兩個測試程序（檔名固定，會互相覆寫）。
  - 55 項要 `D:\lineage開發\0-客戶端` 在場，路徑是寫死的，`LITO_SHARED_RESOURCE_ROOT` 對它們沒有作用；跑的時候不要同時啟動或更新客戶端。
  - 不需要客戶端的 18 項（T 類 6、N 類 1、`native_resource_loading` 8、`native_light_mask`、`native_startup`、`native_world_text`）可以考慮放進 CI，但 `native_world_text` 依賴主機字型，先在本機確認再決定。
- **L 類 10 項**要放行，得先把輸出改到 `target/current/verification/`（沒有其他測試讀那些 details 檔，改位置不會影響別的測試），並解決原機 snapshot 的前置條件。建議併入「原機 snapshot 99 項」那一張處理，或依 `docs/TESTING.md` 的方向改寫或刪除。
- **另外 4 張 helper 閉合審查**可以直接用 `scratch/issue-306/libwriters.py` 的結果：`src/` 非測試程式碼裡會寫檔的只有 `*_probe.rs`、`native_peer_*`、`native_startup.rs`（`Settings::save`）、`preferences.rs`、`rebuild_verification.rs`、`native_backend_485f_entry.rs`，寫入根目錄都由呼叫端傳入。
- 本張只在這 83 項所在的檔案裡找相對路徑寫入。其他測試檔若有同樣寫法，請在各自那一張順手檢查（搜尋 `fs::write(` 或 `File::create(` 後面直接接 `"target/`，注意有些是跨行的）。
