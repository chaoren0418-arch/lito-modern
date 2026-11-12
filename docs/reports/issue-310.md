# Issue #310：T036-5 跨函式 helper 閉合審查——背包／登入／傳送

## 結論

#54 標成「跨函式／資源 helper 安全邊界尚未封閉」的 375 項裡，本張認領 28 個 target 共 **118 項**，逐項做完靜態閉合；同一批測試檔在 #54 之後新增的 12 個測試一併審了。**本輪執行的測試數是 0，也沒有編譯**；以下全部是讀原始碼得到的裁定，回答的是「這個測試跑起來會碰到什麼」，不是測試結果。

| 裁定 | #54 helper 項 | #54 之後新增 | 其中已在白名單 | 本張新放行 |
|---|---:|---:|---:|---:|
| A 純記憶體／CPU | 90 | 12 | 2 | 100 |
| B 唯讀資源根（限有 `D:\lineage開發\0-客戶端` 的本機） | 20 | 0 | 0 | 20 |
| C 自有 loopback | 2 | 0 | 0 | 2 |
| D 自有新建目錄寫入 | 5 | 0 | 0 | 5 |
| 不放行：轉「原機 snapshot」類（T036-12） | 1 | 0 | 0 | 0 |
| 合計 | 118 | 12 | 2 | **127** |

- A：閉包內沒有檔案、網路、子程序、環境變數、視窗、音效裝置。
- B：只多一件事，唯讀載入寫死在測試檔裡的 `D:\lineage開發\0-客戶端`，其餘在記憶體。分類方式和 #307 的 B 類相同。
- C：測試自己 bind `127.0.0.1:0`，只連自己持有的 listener。
- D：只寫 `target/current/scratch/native-backend-485f-evidence/run-<pid>-<n>/`，`create_dir`＋`create_new`，不覆寫、不刪除。
- 127 項新放行清單在文末，格式和 `維護工具/test-whitelist/tests.tsv` 相同，交給執行張 **#311（T036-6）**。#307 報告裡寫的「#310（執行）」是筆誤，#310 是本張。

審查基準：先在 `origin/main` `f8ed20ec` 審完，push 前合併到 `3a81fc2`（含 #306、#307）重跑工具，124 項的原語統計沒有變化（只有 4 項的可達函式數多 1）。#54 的分類基準是 `bb96171d`，之後 `tests/`、`src/` 都改過很多，所以這次是對現行原始碼重新審，不是沿用 #54 的行號。

## 範圍怎麼切

TASKS.md 只寫「背包／登入／傳送 1 張」，沒有列 target。#307 已公布技能／戰鬥的切法（19 個 target、112 項，#307 與 #308 各 56）。本張認領的是「不屬於技能／戰鬥、也不屬於世界／渲染」的 target：

- 背包／物品：`native_inventory`（7）、`native_inventory_icons`（5）、`native_inventory_names`（4）、`native_item_actions`（15）、`native_merchant`（5）、`native_ground_pickup`（5）
- 登入／大廳／房間／NPC 對話：`native_login`（3）、`native_lobby`（3）、`native_startup`（3）、`native_lifecycle`（3）、`native_classic_entry`（2）、`native_room`（4）、`native_room_admin`（4）、`native_room_force`（4）、`native_room_ui`（4）、`native_game`（4）、`native_game_timers`（2）、`native_world_entry`（2）、`rebuild_wire`（1）、`native_npc_document`（6）、`native_npc_images`（3）、`native_npc_input`（2）、`native_painted_input`（2）、`native_library_regression`（6，都是 `backend_evidence`）
- 傳送／入口：`native_teleport`（14）、`native_arena_portal`（2）、`native_map_portal`（1）、`native_travel`（2）

375 = 本張 118 ＋ 技能／戰鬥 112（#307、#308）＋ 其餘 145。本張沒有認領的 51 個 target 如下；扣掉 #307／#308 的 19 個之後，剩下 32 個 target、145 項應該都是 #309（世界／渲染）的，已在 #309 留言列出：

`lib:native_disintegrate_tests`（10）、`native_actor_layers`（2）、`native_actor_relations`（10）、`native_actor_states`（6）、`native_animation`（4）、`native_attack_skills`（15）、`native_background`（2）、`native_background_geometry`（3）、`native_background_pixels`（4）、`native_chill_touch`（3）、`native_clan_markers`（1）、`native_combat`（11）、`native_cone_cold`（3）、`native_effect_lifecycle`（4）、`native_emblems`（4）、`native_environment`（9）、`native_fire_arrow`（2）、`native_ground_skills`（6）、`native_holy_weapon`（3）、`native_ice_arrow`（3）、`native_label_updates`（2）、`native_light_mask`（4）、`native_light_spell`（6）、`native_lightning`（3）、`native_minimap`（4）、`native_motion`（6）、`native_navigation`（8）、`native_object_dispatch`（1）、`native_observer_camera`（3）、`native_obstacles`（4）、`native_poison_cure`（4）、`native_ranged`（9）、`native_reagent_skills`（5）、`native_resource_loading`（3）、`native_skill_selection`（7）、`native_skills`（9）、`native_stalac_layers`（4）、`native_terrain_pixels`（3）、`native_terrain_residence`（1）、`native_turn_undead`（6）、`native_vampiric_touch`（4）、`native_water_light`（3）、`native_wind_cutter`（3）、`native_world`（5）、`native_world_effects`（3）、`native_world_labels`（2）、`native_world_layers`（4）、`native_world_lighting`（11）、`native_world_lighting_scene`（20）、`native_world_roofs`（4）、`native_world_text`（1）。共 51 個 target、257 項。

## 做了什麼

1. 讀 `CLAUDE.md`、`docs/REPORT.md`、`docs/REUSE-MAP.md`、`docs/TESTING.md`、#54 報告與 #296 計畫。本張是靜態審查，沒有需要參考 tw-bin-engine 或 lito-launcher-server 的地方。
2. 寫了一支靜態工具 `維護工具/t036-helper-closure/testclosure.py`（Python，只讀 `.rs` 文字，不編譯、不執行專案程式）。它對每個 `#[test]` 算出函式層級的呼叫閉包，再列出閉包內每個帶副作用原語的函式和一條呼叫鏈。#307 的 `fnclosure.py` 是「同名函式一律納入」，從 `World::accept` 會擴到 1400 多個函式；這支多做了檔案作用域的名稱解析、型別觸及、方法呼叫的 self／參數個數過濾，同一類測試的閉包縮到幾十到兩三百個函式，可以直接逐項看。解析規則與已知限制寫在檔頭。
3. 對 27 個 `tests/*.rs` 共 140 個測試跑工具（約 10 秒），和 #54 的表依 `(target, 測試名)` 對上：#54 有的列在現行原始碼全部找得到，另有 12 個測試是 #54 之後新增。140 個裡 124 個屬於本張（112 個 helper 項＋12 個新增），其餘 16 個是別類（寫檔 7、缺前置 5、GUI 1、DB 1、原本就在白名單 2），沒有裁定。
4. 工具列出的每一個原語命中都回原始碼讀該行，分成真副作用和誤報；工具看不出來的動態條件（cancel 旗標）讀測試本體和被呼叫函式補上。
5. `native_library_regression` 用 `#[path]` 把 `src/` 的 `#[cfg(test)]` 模組拉進測試 target，工具不支援，6 項是人工讀 `native_backend_485f_evidence.rs` 全檔裁定的。
6. 靜態核對了一項歷史缺口（runner 漏 pin `native_room.rs`），見下文。

沒有改任何 `src/`、`tests/`、`Cargo.*`、白名單；沒有編譯，沒有執行任何測試或編譯產物。

## 已驗證（靜態，讀原始碼）

**A 類的依據。** 102 個測試的閉包內，工具列出的原語只剩下面幾種，逐一讀過：

| 函式 | 內容 | 判斷 |
|---|---|---|
| `native_login::decode_message`（`native_login.rs:58`）、`encode_text`（`:76`）、`native_inventory::Reader::string`（`native_inventory.rs:116`）、`descriptions::decode_cp950`（`descriptions.rs:379`）、`native_npc_document::parse_template`（`native_npc_document.rs:98`） | `unsafe` 呼叫 `MultiByteToWideChar`／`WideCharToMultiByte`（CP950） | 記憶體內轉碼 |
| `native_clock::initialize`（`native_clock.rs:94`）、`sample`（`:107`） | `QueryPerformanceFrequency`／`QueryPerformanceCounter`／`GetTickCount`，結果存行程內 `OnceLock` | 只讀時鐘 |
| `native_status4183::now_ms`、`native_environment::State::begin`、`LoginController` 各動作設定 deadline | `Instant::now()` | 只讀時鐘 |
| `native_world_text::Resources::drop`（`native_world_text.rs:104`） | GDI 物件釋放 | 誤報：`Resources` 只在 `raster()` 內建立，這 102 個測試的閉包不含 `raster()`；是同名型別的保守連結 |

`LoginController::default()` 會建立 `native_connection::Controller::default()`，其 `worker`／`events`／`commands` 都是 `None`（`native_connection.rs:407`）；`send` 要求 `Phase::VersionAccepted`（`:482`）；`Drop` 只在有 worker 時 join（`:685`）。這些測試的閉包都不含 `Controller::start`，所以不會開 socket 或執行緒。

兩個由人工補判的 A 項：
- `native_inventory_icons::malformed_native_images_and_cancelled_loads_fail_closed`：工具因為 `Reader::open` 把整條開檔鏈算進來。測試傳的是 `cancel=true`，`native_inventory_icons.rs:193` 第一行 `active(cancel)?` 就返回。
- `native_npc_images::cancelled_image_worker_never_returns_a_publishable_bank`：`Bank::load` 第一行 `ensure!(!cancel.load(…))` 返回。

**B 類的依據。** 20 個測試的本體或同檔 helper 把 `D:\lineage開發\0-客戶端` 寫死（不讀 `LITO_SHARED_RESOURCE_ROOT`）。閉包內的檔案操作全部走這幾個函式，讀過都是唯讀：

- `resources::open_readonly`（`resources.rs:864`）：`OpenOptions::new().read(true)`，Windows 上 `share_mode(FILE_SHARE_READ)`、`OPEN_REPARSE_POINT`。
- `resources::reject_reparse_chain`／`is_link`／`verify_open_path`／`final_handle_path`：`symlink_metadata`、`metadata`、`canonicalize`、`GetFinalPathNameByHandleW`。
- `native_world_scene::read_pin`、`native_resources::PinnedFile::open_checked`、`SharedArchive::open_index`、`resources::read_shared_loose_file`：以上述函式開檔後讀取、算 SHA-256。
- `native_arena_portal` 兩項在測試本體用 `std::fs::read_dir`／`read` 讀 `LiTo\LTmap\<id>\*.s32`。

工具在這 20 項另外標出的 `FS_WRITE`（`OpenOptions`、`symlink_metadata` 字樣）、`NET`（`Plan::bind(0)`、`water_history.bind(owner)`）、`STDIO`（載入失敗時 `eprintln!`）、`ENV`（根目錄是相對路徑時才 `current_dir()`，這些測試都給絕對路徑）都是誤報或不會走到。執行緒只有 `Service`／`Job` 自己開的 worker（`native_inventory_icons.rs:275`、`native_inventory_names.rs:192`、`native_npc_document.rs:176`）。畫面是畫到記憶體 `Canvas`；`native_world_text::raster` 用 GDI 記憶體 DC 算字形，不建立視窗。聲音只產生事件資料，閉包內沒有音效裝置呼叫。

其中 7 項走完整場景 `native_world_scene::Model::load／draw_at`（`native_ground_pickup` 4、`native_painted_input` 2、`native_map_portal` 1），閉包有 765 到 1562 個函式；`native_travel` 的 bitmap 測試（658）和 `native_lifecycle` 的復活測試（482）也很大。這 9 項靠的是工具把閉包內所有原語命中列完、再逐個讀過命中的那幾行，不是把每個函式都讀過。#307 轉給 #309 的兩個 B′ 項目走的是同一條場景鏈，上面的原語清單可以直接用。

**C 類的依據。** `native_game_timers` 的兩個測試在本體 `TcpListener::bind("127.0.0.1:0")`，listener 移進測試自己的 server 執行緒持有到結束；`Controller::start(peer)` 的 worker 只對該位址 `connect_timeout`，並核對 `peer_addr() == peer`（`native_connection.rs:136`、`:143`）。閉包內沒有檔案或其他位址。其中 `natural_callbacks_continue_with_a_full_inbound_event_queue_and_no_UI_polling` 是 `#[ignore]`，照原版 60 秒／300 秒間隔跑，單項約 5 分多鐘。

**D 類的依據。** `backend_evidence` 的 5 個測試只用 `Recorder` 的記憶體方法，最後呼叫 `save_test_report`（`native_backend_485f_evidence.rs:95`）：在 `target/current/scratch/native-backend-485f-evidence/` 下以 `fs::create_dir` 搶一個 `run-<pid>-<序號>` 新目錄（已存在就換下一個序號），檔案用 `create_new(true)` 開。不會覆寫或刪除既有檔案，路徑在 `target/` 內。執行後會留下 JSON，不會自己清。

**不放行的 1 項。** `backend_evidence::r531_evidence_checks::actual_runtime_rejection_close_and_join_remain_distinct_from_world_admission` 以 `logs/continuation-20261003/r486-map-binding.json` 啟動 `Runtime`；該 JSON 記的是 `…\LiTo381-Modern\target\r473-campaign-resource-snapshot\…` 這類原機絕對路徑。`Runtime::start` 之後的資源載入鏈沒有在本張閉合，轉給原機 snapshot 那一張（T036-12）。

**歷史缺口：runner 漏 pin `native_room.rs`（NATIVE-WORLD-INTEGRATION:13）。** 現行 `維護工具/run-native-arena-portal.ps1:43` 與 `維護工具/run-native-map-portal.ps1:43` 的 pin 清單都含 `LiTo381-Modern/src/modern_lito/native_room.rs`。這只確認腳本內容，兩支 runner 沒有重跑。

## 推測

- 工具是文字層級的過度近似，不是編譯器。它會列出解析不到的名稱；落在 A 類閉包內的 16 個逐一看過，是區域 closure／closure 參數（`counter`、`contains`、`append`、`word`、`name`、`scope`、`replace`、`pixel`、`read`）、泛型參數（`R`）、或已用「依名稱全部觸及」保守處理的型別（`battle_payload_values` 的 6 個）。`battle_payload_*` 的 `record!`／`tags!`／`scalar!` 與 `native_external_fixture` 的 `dto!` 巨集產生的是記憶體編解碼 impl 和純資料 struct。仍可能有工具和人工都沒看到的間接呼叫，所以 #311 第一次執行時應逐項 `--exact`，不要整個 target 一起跑。
- `open_readonly` 的 share mode 只有 `FILE_SHARE_READ`。B 類測試持有 pak／idx handle 期間，別的行程不能以寫入或刪除方式開同一個檔；反過來，如果已有行程以寫入方式開著，測試會開檔失敗。一般唯讀開檔的客戶端不受影響，但這點沒有實測。
- B 類 20 項沒有 `#[ignore]` 也沒有環境變數開關，在沒有 `D:\lineage開發\0-客戶端` 的機器上會直接失敗（CI 只跑 `cargo check` 和 `cargo test --lib`，不受影響）。

## 待驗證

- 127 項新放行測試的實際結果：全部未執行，由 #311（T036-6）在本機逐項跑。
- 不放行的 1 項：由 T036-12（原機 snapshot）決定。
- 和本張領域有關、需要實際執行才能重驗的歷史失敗：房間切換 90 秒逾時（NATIVE-SCENE-LOADING:9）是需要 Java peer 的 live 探針，屬 T036-10／11；傳送 F4 退役順序（NATIVE-WORLD-INTEGRATION:9、:177）對應 `native_teleport_controller`（需 `teleport-wire-test` feature，#54 列為固定檔名寫入，屬 #306）。本張沒有重驗這兩項。
- 整庫 lib 編譯記憶體失敗（NATIVE-WORLD-INTEGRATION:5、:9、:177）不在本張範圍，沒有處理。

## 逐項裁定（27 個整合測試檔，124 項）

「來源」欄：helper＝#54 的 helper 未封閉項；新增＝#54 之後才有的測試。「工具列出的原語」是工具的原始輸出（命中的函式個數），含誤報；裁定以上面的人工閱讀為準。行號與函式數是 `3a81fc2` 的。

| Target | 測試（行號） | 來源 | 可達函式 | 工具列出的原語 | 裁定 | 依據 |
|---|---|---|---:|---|---|---|
| native_inventory | `formal_records_and_all_deltas_keep_unsigned_fields_and_opaque_details`（18） | helper | 23 | WIN=3 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory | `every_truncation_and_trailing_field_is_rejected_atomically`（63） | helper | 22 | WIN=3 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory | `batches_add_to_existing_objects_and_replacement_does_not_duplicate_rows`（110） | helper | 23 | WIN=3 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory | `selected_connection_and_reselection_own_inventory_not_world_generation`（126） | helper | 21 | WIN=3 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory | `resource_limits_allow_maximum_wire_fields_but_bound_inventory_storage`（155） | helper | 22 | WIN=3 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory | `labels_preserve_authoritative_names_and_do_not_expose_control_characters`（187） | helper | 24 | WIN=3 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory | `same_character_resume_without_replay_keeps_authority_and_rejected_switch_rolls_back`（197） | helper | 25 | WIN=3 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory_icons | `covered_black_is_opaque_and_uncovered_pixels_use_background`（23） | helper | 25 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory_icons | `signed_offsets_clip_without_wrapping_or_losing_coverage_count`（40） | helper | 24 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory_icons | `malformed_native_images_and_cancelled_loads_fail_closed`（57） | helper | 316 | ENV=1, FS_READ=10, FS_WRITE=3, WIN=4 | A 純記憶體 | 唯一碰檔案的路徑是 `Reader::open("never-opened", cancel=true)`；`native_inventory_icons.rs:193` 先 `active(cancel)?` 返回，未開檔 |
| native_inventory_icons | `viewport_changes_publish_only_current_intrinsic_ids_and_close_reclaims_worker`（123） | helper | 338 | ENV=1, FS_READ=10, FS_WRITE=3, THREAD=2, TIME=1, WIN=4 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_inventory_icons | `visible_budget_hidden_rows_and_failures_are_bounded_without_retry_loops`（139） | helper | 339 | ENV=1, FS_READ=10, FS_WRITE=3, THREAD=2, TIME=1, WIN=4 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_inventory_names | `references_use_actual_base_slots_then_lito_offset_and_preserve_literals`（39） | helper | 24 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory_names | `invalid_or_unsupported_references_never_publish_partial_names_or_change_wire_item`（53） | helper | 27 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory_names | `labels_resolve_only_presentation_and_count_stays_authoritative`（86） | helper | 27 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_inventory_names | `worker_closure_reclaims_loading_ready_and_failed_jobs`（147） | helper | 321 | ENV=1, FS_READ=10, FS_WRITE=3, THREAD=2, TIME=1, WIN=5 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_item_actions | `formal_letters_use_mixed_ms950_utf16_and_fence_stale_choices`（8） | 新增 | 259 | TIME=3, WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `read_mail_is_atomic_unsigned_and_cannot_cross_inventory_owner`（41） | 新增 | 223 | TIME=3, WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `formal_read_and_instrument_envelopes_and_server_blockers_are_explicit`（66） | 新增 | 258 | TIME=3, WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `request_envelopes_match_actual_jar_unsigned_vectors`（123） | helper | 5 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `requests_do_not_predict_inventory_count_name_or_equipment`（139） | helper | 139 | WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `specialized_or_unknown_use_types_never_fall_through_to_default_packet`（156） | helper | 139 | WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `stale_character_connection_reselection_and_changed_instance_are_rejected`（170） | helper | 139 | WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `transition_dead_and_blocked_worlds_cannot_issue_item_requests`（190） | helper | 245 | TIME=2, WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `drop_uses_current_world_position_and_authoritative_quantity_bounds`（212） | helper | 245 | TIME=2, WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `pending_tracks_only_selected_item_changes_and_does_not_claim_success`（226） | helper | 117 | TIME=1, WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `pending_expires_without_replay_and_interrupts_on_owner_or_world_change`（242） | helper | 223 | TIME=3, WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `formal_ground_items_preserve_quantity_identity_and_strict_envelope`（269） | helper | 17 | WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `target_location_envelope_matches_actual_sql_loaded_formal_parser`（296） | helper | 2 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `type_five_requires_explicit_world_target_without_predicting_authority`（313） | helper | 255 | TIME=2, WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `target_cursor_rejects_retired_world_even_with_unchanged_inventory`（329） | helper | 241 | TIME=2, WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `actor_target_requires_current_live_coordinates_and_presence`（356） | helper | 241 | TIME=2, WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `target_pending_requires_exact_new_rejection_or_authoritative_portal`（372） | helper | 225 | TIME=3, WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `ordinary_portal_admission_requires_live_room_and_same_map`（395） | helper | 222 | TIME=2, WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `charged_type_three_is_cd_and_waits_for_authoritative_charge_delta`（425） | 新增 | 252 | TIME=3, WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `destroy_is_whole_stack_cd_and_never_predicts_removal`（448） | 新增 | 139 | WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `inventory_options_use_both_unsigned_objects_and_reject_stale_choices`（470） | 新增 | 243 | TIME=2, WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `text_options_are_bounded_exact_ms950_and_never_default_empty`（499） | 新增 | 134 | WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `home_and_light_have_distinct_tails_and_actor_target_is_not_a_coordinate_packet`（516） | 新增 | 253 | TIME=2, WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `five_item_spells_never_send_actor_or_catalog_id_or_spend_locally`（539） | 新增 | 263 | TIME=2, WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `healing_effects_and_reagent_deltas_are_independent_authority`（565） | 新增 | 257 | TIME=3, WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_item_actions | `formal_e149_options_and_item_skill_vectors_match_production_builders`（602） | 新增 | 267 | TIME=2, WIN=6 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_merchant | `formal_merchant_rows_and_dialog_are_exact_bounded_and_unsigned`（5） | helper | 11 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_merchant | `received_zero_buy_order_is_not_a_zero_inventory_object`（13） | helper | 9 | WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_merchant | `shop_list_is_single_use_and_expires_with_character_world_and_time`（21） | helper | 19 | TIME=1, WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_merchant | `captured_system_refusal_is_not_a_template_message_and_invalid_text_is_atomic`（34） | helper | 137 | TIME=2, WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_merchant | `formal_inventory_rejections_are_separate_from_text_messages`（47） | helper | 137 | TIME=2, WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_ground_pickup | `formal_all_unsigned_wire_x_and_quantity_vectors`（57） | helper | 1525 | ENV=2, FS_READ=24, FS_WRITE=4, NET=4, STDIO=3, THREAD=1, TIME=9, WIN=12 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_ground_pickup | `capture_requires_actual_type9_resource_and_fresh_render`（88） | helper | 1525 | ENV=2, FS_READ=24, FS_WRITE=4, NET=4, STDIO=3, THREAD=1, TIME=9, WIN=12 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_ground_pickup | `stale_pickup_choice_never_rebinds_same_id_or_new_owner`（108） | helper | 1525 | ENV=2, FS_READ=24, FS_WRITE=4, NET=4, STDIO=3, THREAD=1, TIME=9, WIN=12 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_ground_pickup | `split_ground_and_inventory_updates_are_observations_not_success_ack`（139） | helper | 1562 | ENV=2, FS_READ=24, FS_WRITE=4, NET=4, STDIO=3, THREAD=1, TIME=10, WIN=13 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_ground_pickup | `formal_ground_replacement_and_removal_are_atomic`（171） | helper | 180 | TIME=2, WIN=3 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_login | `chinese_credentials_and_source_match_deployed_java_ms950_parser`（13） | helper | 8 | WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_login | `deployed_login_replies_retain_full_failure_body_and_unsigned_endpoints`（39） | helper | 16 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_login | `credentials_and_selection_require_current_live_state`（97） | helper | 104 | TIME=4, WIN=4 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_lobby | `deployed_channel_results_are_exact_and_keep_full_unsigned_tails`（16） | helper | 13 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_lobby | `deployed_chinese_room_row_is_bounded_and_updates_by_server_id`（61） | helper | 11 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_lobby | `lobby_actions_do_not_run_without_live_character_and_channel`（96） | helper | 74 | TIME=3, WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_startup | `endpoints_roots_and_schema_are_explicit_without_legacy_fields`（12） | helper | 20 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_startup | `unicode_bom_duplicate_keys_and_input_budgets_are_checked`（50） | helper | 19 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_startup | `settings_are_adjacent_to_executable_independent_of_working_directory`（65） | helper | 4 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_lifecycle | `deployed_resurrection_vectors_preserve_unsigned_identity_and_never_infer_health`（7） | helper | 138 | TIME=2, WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_lifecycle | `resurrected_life_retires_death_playback_selection_and_ack_without_requiring_a_new_map`（17） | helper | 482 | ENV=1, FS_READ=6, FS_WRITE=3, TIME=2, WIN=5 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_lifecycle | `malformed_or_orphan_resurrection_never_creates_or_partially_mutates_an_actor`（32） | helper | 139 | TIME=2, WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_classic_entry | `classic_entry_preserves_received_identity_and_formal_check_domain`（4） | helper | 25 | WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_classic_entry | `classic_entry_cannot_start_without_current_character_directory`（14） | helper | 87 | TIME=1, WIN=4 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room | `room_envelopes_match_deployed_jar_and_reject_partial_or_trailing_fields`（16） | helper | 14 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room | `joined_room_preserves_settings_but_never_serializes_password`（34） | helper | 16 | WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room | `participant_ready_two_and_signed_selector_are_authoritative_incremental_fields`（59） | helper | 15 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room | `room_actions_require_current_server_lobby_and_membership`（102） | helper | 77 | TIME=3, WIN=4 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_admin | `settings_match_formal_vectors_and_wire_capacity_domain`（8） | helper | 8 | WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_admin | `settings_reject_unrepresentable_fields_and_invalid_option_combinations`（24） | helper | 6 | WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_admin | `observer_and_kick_replies_match_formal_and_reject_partial_envelopes`（35） | helper | 17 | WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_admin | `admin_actions_require_live_phase_and_do_not_mutate_idle_snapshot`（48） | helper | 81 | TIME=4, WIN=4 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_force | `deployed_force_contract_is_bounded_and_preserves_authoritative_name`（3） | helper | 13 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_force | `disconnected_controller_cannot_queue_a_force_change`（20） | helper | 70 | TIME=1, WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_force | `captured_force_before_roster_does_not_create_a_member_or_override_full_snapshot`（34） | helper | 19 | TIME=1, WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_force | `incomplete_roster_force_is_bounded_and_cannot_cross_room_or_connection`（53） | helper | 18 | TIME=1, WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_ui | `draft_cannot_outlive_connection_character_channel_owner_or_received_configuration`（13） | helper | 105 | WIN=4 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_ui | `edits_preserve_opaque_received_options_and_match_formal_bytes_without_retaining_password`（33） | helper | 106 | WIN=4 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_ui | `kick_choice_keeps_named_member_across_reordering_and_rejects_changed_identity`（54） | helper | 80 | WIN=3 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_room_ui | `lobby_room_choice_follows_id_through_live_counts_and_order_without_crossing_scope`（71） | helper | 81 | WIN=3 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_game | `deployed_character_row_keeps_signed_fields_rank_order_and_original_name`（12） | helper | 29 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_game | `deployed_game_envelopes_reject_truncation_trailing_bytes_and_invalid_projection`（64） | helper | 23 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_game | `complete_directory_is_atomic_bounded_and_rejects_duplicate_names`（114） | helper | 29 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_game | `notice_acknowledgements_match_deployed_parsers`（154） | helper | 26 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_game | `clan_capacity_is_sixteen_ms950_bytes_plus_nul_without_field_shift`（173） | 新增 | 28 | WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語；已在 `tests.tsv` |
| native_game_timers | `admitted_timer_bytes_share_the_ordered_counter_and_final_quit_barrier`（112） | helper | 289 | NET=7, THREAD=5, TIME=10, WIN=1 | C 自有 loopback | 測試自己 bind `127.0.0.1:0` 並持有 listener；`Controller::start` 只連該位址 |
| native_game_timers | `natural_callbacks_continue_with_a_full_inbound_event_queue_and_no_UI_polling`（193） | helper／ignored | 284 | NET=7, THREAD=5, TIME=10, WIN=1 | C 自有 loopback | 測試自己 bind `127.0.0.1:0` 並持有 listener；`Controller::start` 只連該位址 |
| native_world_entry | `learned_world_tree_is_selector_only_and_cross_map_admission_stays_owned`（70） | helper | 214 | TIME=2, WIN=5 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語；已在 `skills.tsv` |
| native_world_entry | `formal_map_rejection_is_visible_and_does_not_change_world`（86） | helper | 137 | TIME=3, WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| rebuild_wire | `current_formal_jar_cipher_matches_native_x64_rx_tx_and_partial_io`（2） | helper | 25 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_npc_document | `original_chinese_documents_preserve_text_and_received_actions`（3） | helper | 8 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_npc_document | `truncated_unqualified_and_oversized_markup_never_produces_action_buttons`（14） | helper | 8 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_npc_document | `original_archive_loader_matches_fixture_hashes_and_obeys_cancellation`（22） | helper | 258 | FS_READ=9, FS_WRITE=3, WIN=5 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_npc_document | `local_page_links_are_distinct_from_wire_requests_and_stay_in_the_catalog`（32） | helper | 8 | WIN=1 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_npc_document | `original_hotel_variables_are_plain_text_and_missing_values_fail_closed`（43） | helper | 261 | FS_READ=9, FS_WRITE=3, WIN=5 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_npc_document | `original_teleport_page_keeps_only_source_actions_and_image_references`（61） | helper | 259 | FS_READ=9, FS_WRITE=3, WIN=5 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_npc_images | `original_page_images_match_independent_rgb555_reference_and_page_bindings`（6） | helper | 259 | FS_READ=9, FS_WRITE=3, WIN=5 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_npc_images | `cancelled_image_worker_never_returns_a_publishable_bank`（29） | helper | 228 | FS_READ=7, FS_WRITE=3, WIN=4 | A 純記憶體 | `Bank::load(root, cancel=true)` 第一行 `ensure!(!cancel…)` 返回，未開檔 |
| native_npc_images | `cancelling_the_real_document_worker_does_not_publish_cached_images`（53） | helper | 264 | FS_READ=9, FS_WRITE=3, THREAD=2, TIME=1, WIN=5 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_npc_input | `formal_numeric_request_bounds_and_reply_match_pinned_protocol`（6） | helper | 13 | WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_npc_input | `quantity_request_is_owned_expiring_and_cannot_be_reused_after_submission`（15） | helper | 21 | TIME=1, WIN=2 | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_painted_input | `painted_masks_and_camera_survive_unpainted_idle_ticks_without_copying_canvas`（11） | helper | 1471 | ENV=2, FS_READ=24, FS_WRITE=4, NET=4, STDIO=3, THREAD=1, TIME=9, WIN=10 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_painted_input | `painted_surface_rejects_portals_updates_actor_life_and_control_interruptions`（30） | helper | 1471 | ENV=2, FS_READ=24, FS_WRITE=4, NET=4, STDIO=3, THREAD=1, TIME=9, WIN=10 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_teleport | `only_reachable_page_choices_authorize_teleport`（6） | helper | 3 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `teleport_requires_matching_status_then_endpoint_once`（11） | helper | 10 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `stale_owner_and_unavailable_slots_cannot_redirect`（21） | helper | 10 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `every_owner_dimension_rejects_stale_replies_without_consuming_current_pending`（34） | helper | 9 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `formal_status_vectors_preserve_slots_and_missing_slots_are_unavailable`（59） | helper | 4 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `formal_endpoint_vectors_keep_little_endian_ip_and_port`（68） | helper | 3 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `availability_rejects_every_truncation_trailing_data_and_unknown_state`（74） | helper | 3 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `endpoint_rejects_every_truncation_trailing_data_and_invalid_index_or_port`（84） | helper | 3 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `same_endpoint_catalog_rebinds_only_matching_routes_once_after_fresh_game_character`（105） | helper | 13 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `cross_endpoint_or_port_change_never_carries_source_rental_routes`（114） | helper | 13 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `old_or_skipped_connection_and_wrong_character_consume_the_catalog`（123） | helper | 13 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `absent_directory_and_routes_for_another_peer_do_not_invent_rental_service`（132） | helper | 13 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `changing_the_pending_owner_or_destination_cannot_retarget_a_captured_catalog`（140） | helper | 13 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_teleport | `remembered_directory_is_an_owned_snapshot_and_connection_overflow_cannot_wrap`（149） | helper | 13 | NONE | A 純記憶體 | 閉包內無檔案／網路／子程序／環境變數／執行緒原語 |
| native_arena_portal | `original_101_and_102_sources_are_available_without_client_destinations`（19） | helper | 238 | FS_READ=1, TIME=3, WIN=3 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_arena_portal | `close_gui_spawn_crosses_real_original_sector_before_source_entrance`（129） | helper | 67 | FS_READ=1 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_map_portal | `original_native_entrances_encode_source_only_and_guard_every_owner_change`（13） | helper | 765 | ENV=1, FS_READ=18, FS_WRITE=3, THREAD=1, TIME=7, WIN=6 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_travel | `original_169_frames_sound_and_controls_are_exact`（24） | helper | 330 | ENV=1, FS_READ=6, FS_WRITE=3, WIN=4 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |
| native_travel | `original_169_bitmap_sound_once_terminal_dwell_and_portal_retirement`（38） | helper | 658 | ENV=2, FS_READ=8, FS_WRITE=4, TIME=2, WIN=6 | B 唯讀資源根 | 唯讀載入寫死的 `D:\lineage開發\0-客戶端`，其餘在記憶體 |

### native_library_regression（人工，6 項）

| 測試 | 裁定 | 依據 |
|---|---|---|
| `modern_lito::backend_evidence::r531_evidence_checks::late_d1_e100_and_complete_write_drain_retained_ledgers_once` | D 自有新建目錄寫入 | `Recorder` 記憶體方法＋`save_test_report` |
| `modern_lito::backend_evidence::r531_evidence_checks::unseen_is_unknown_without_coverage_and_scoped_not_observed_is_not_wire_absence` | D 自有新建目錄寫入 | 同上 |
| `modern_lito::backend_evidence::r531_evidence_checks::bounded_journal_retains_first_failure_without_auth_data_or_false_negatives` | D 自有新建目錄寫入 | 同上 |
| `modern_lito::backend_evidence::r532_counterexample_checks::d1_e1_scanned_scopes_and_gaps_are_independent` | D 自有新建目錄寫入 | 同上 |
| `modern_lito::backend_evidence::r532_counterexample_checks::dropped_positive_receipts_rejection_close_and_writes_never_claim_observed` | D 自有新建目錄寫入 | 同上 |
| `modern_lito::backend_evidence::r531_evidence_checks::actual_runtime_rejection_close_and_join_remain_distinct_from_world_admission` | 不放行，轉原機 snapshot | `Runtime::start` 讀 `logs/continuation-20261003/r486-map-binding.json`（原機絕對路徑） |

## 新放行清單（給 #311）

格式與 `維護工具/test-whitelist/tests.tsv` 相同：target、完整測試名稱、是否 `#[ignore]`、features；真正的 Tab 分隔。已經在 `tests.tsv`／`skills.tsv` 的 2 項（`native_game::clan_capacity_is_sixteen_ms950_bytes_plus_nul_without_field_shift`、`native_world_entry::learned_world_tree_is_selector_only_and_cross_map_admission_stays_owned`）不重複列。本張**沒有**改白名單檔：本機套件會照清單逐項實跑，併入就等於排入執行，由 #311 決定併入哪一份、何時併。

### A：純記憶體（100 項）

```text
native_inventory	formal_records_and_all_deltas_keep_unsigned_fields_and_opaque_details	no	-
native_inventory	every_truncation_and_trailing_field_is_rejected_atomically	no	-
native_inventory	batches_add_to_existing_objects_and_replacement_does_not_duplicate_rows	no	-
native_inventory	selected_connection_and_reselection_own_inventory_not_world_generation	no	-
native_inventory	resource_limits_allow_maximum_wire_fields_but_bound_inventory_storage	no	-
native_inventory	labels_preserve_authoritative_names_and_do_not_expose_control_characters	no	-
native_inventory	same_character_resume_without_replay_keeps_authority_and_rejected_switch_rolls_back	no	-
native_inventory_icons	covered_black_is_opaque_and_uncovered_pixels_use_background	no	-
native_inventory_icons	signed_offsets_clip_without_wrapping_or_losing_coverage_count	no	-
native_inventory_icons	malformed_native_images_and_cancelled_loads_fail_closed	no	-
native_inventory_names	references_use_actual_base_slots_then_lito_offset_and_preserve_literals	no	-
native_inventory_names	invalid_or_unsupported_references_never_publish_partial_names_or_change_wire_item	no	-
native_inventory_names	labels_resolve_only_presentation_and_count_stays_authoritative	no	-
native_item_actions	formal_letters_use_mixed_ms950_utf16_and_fence_stale_choices	no	-
native_item_actions	read_mail_is_atomic_unsigned_and_cannot_cross_inventory_owner	no	-
native_item_actions	formal_read_and_instrument_envelopes_and_server_blockers_are_explicit	no	-
native_item_actions	request_envelopes_match_actual_jar_unsigned_vectors	no	-
native_item_actions	requests_do_not_predict_inventory_count_name_or_equipment	no	-
native_item_actions	specialized_or_unknown_use_types_never_fall_through_to_default_packet	no	-
native_item_actions	stale_character_connection_reselection_and_changed_instance_are_rejected	no	-
native_item_actions	transition_dead_and_blocked_worlds_cannot_issue_item_requests	no	-
native_item_actions	drop_uses_current_world_position_and_authoritative_quantity_bounds	no	-
native_item_actions	pending_tracks_only_selected_item_changes_and_does_not_claim_success	no	-
native_item_actions	pending_expires_without_replay_and_interrupts_on_owner_or_world_change	no	-
native_item_actions	formal_ground_items_preserve_quantity_identity_and_strict_envelope	no	-
native_item_actions	target_location_envelope_matches_actual_sql_loaded_formal_parser	no	-
native_item_actions	type_five_requires_explicit_world_target_without_predicting_authority	no	-
native_item_actions	target_cursor_rejects_retired_world_even_with_unchanged_inventory	no	-
native_item_actions	actor_target_requires_current_live_coordinates_and_presence	no	-
native_item_actions	target_pending_requires_exact_new_rejection_or_authoritative_portal	no	-
native_item_actions	ordinary_portal_admission_requires_live_room_and_same_map	no	-
native_item_actions	charged_type_three_is_cd_and_waits_for_authoritative_charge_delta	no	-
native_item_actions	destroy_is_whole_stack_cd_and_never_predicts_removal	no	-
native_item_actions	inventory_options_use_both_unsigned_objects_and_reject_stale_choices	no	-
native_item_actions	text_options_are_bounded_exact_ms950_and_never_default_empty	no	-
native_item_actions	home_and_light_have_distinct_tails_and_actor_target_is_not_a_coordinate_packet	no	-
native_item_actions	five_item_spells_never_send_actor_or_catalog_id_or_spend_locally	no	-
native_item_actions	healing_effects_and_reagent_deltas_are_independent_authority	no	-
native_item_actions	formal_e149_options_and_item_skill_vectors_match_production_builders	no	-
native_merchant	formal_merchant_rows_and_dialog_are_exact_bounded_and_unsigned	no	-
native_merchant	received_zero_buy_order_is_not_a_zero_inventory_object	no	-
native_merchant	shop_list_is_single_use_and_expires_with_character_world_and_time	no	-
native_merchant	captured_system_refusal_is_not_a_template_message_and_invalid_text_is_atomic	no	-
native_merchant	formal_inventory_rejections_are_separate_from_text_messages	no	-
native_ground_pickup	formal_ground_replacement_and_removal_are_atomic	no	-
native_login	chinese_credentials_and_source_match_deployed_java_ms950_parser	no	-
native_login	deployed_login_replies_retain_full_failure_body_and_unsigned_endpoints	no	-
native_login	credentials_and_selection_require_current_live_state	no	-
native_lobby	deployed_channel_results_are_exact_and_keep_full_unsigned_tails	no	-
native_lobby	deployed_chinese_room_row_is_bounded_and_updates_by_server_id	no	-
native_lobby	lobby_actions_do_not_run_without_live_character_and_channel	no	-
native_startup	endpoints_roots_and_schema_are_explicit_without_legacy_fields	no	-
native_startup	unicode_bom_duplicate_keys_and_input_budgets_are_checked	no	-
native_startup	settings_are_adjacent_to_executable_independent_of_working_directory	no	-
native_lifecycle	deployed_resurrection_vectors_preserve_unsigned_identity_and_never_infer_health	no	-
native_lifecycle	malformed_or_orphan_resurrection_never_creates_or_partially_mutates_an_actor	no	-
native_classic_entry	classic_entry_preserves_received_identity_and_formal_check_domain	no	-
native_classic_entry	classic_entry_cannot_start_without_current_character_directory	no	-
native_room	room_envelopes_match_deployed_jar_and_reject_partial_or_trailing_fields	no	-
native_room	joined_room_preserves_settings_but_never_serializes_password	no	-
native_room	participant_ready_two_and_signed_selector_are_authoritative_incremental_fields	no	-
native_room	room_actions_require_current_server_lobby_and_membership	no	-
native_room_admin	settings_match_formal_vectors_and_wire_capacity_domain	no	-
native_room_admin	settings_reject_unrepresentable_fields_and_invalid_option_combinations	no	-
native_room_admin	observer_and_kick_replies_match_formal_and_reject_partial_envelopes	no	-
native_room_admin	admin_actions_require_live_phase_and_do_not_mutate_idle_snapshot	no	-
native_room_force	deployed_force_contract_is_bounded_and_preserves_authoritative_name	no	-
native_room_force	disconnected_controller_cannot_queue_a_force_change	no	-
native_room_force	captured_force_before_roster_does_not_create_a_member_or_override_full_snapshot	no	-
native_room_force	incomplete_roster_force_is_bounded_and_cannot_cross_room_or_connection	no	-
native_room_ui	draft_cannot_outlive_connection_character_channel_owner_or_received_configuration	no	-
native_room_ui	edits_preserve_opaque_received_options_and_match_formal_bytes_without_retaining_password	no	-
native_room_ui	kick_choice_keeps_named_member_across_reordering_and_rejects_changed_identity	no	-
native_room_ui	lobby_room_choice_follows_id_through_live_counts_and_order_without_crossing_scope	no	-
native_game	deployed_character_row_keeps_signed_fields_rank_order_and_original_name	no	-
native_game	deployed_game_envelopes_reject_truncation_trailing_bytes_and_invalid_projection	no	-
native_game	complete_directory_is_atomic_bounded_and_rejects_duplicate_names	no	-
native_game	notice_acknowledgements_match_deployed_parsers	no	-
native_world_entry	formal_map_rejection_is_visible_and_does_not_change_world	no	-
rebuild_wire	current_formal_jar_cipher_matches_native_x64_rx_tx_and_partial_io	no	-
native_npc_document	original_chinese_documents_preserve_text_and_received_actions	no	-
native_npc_document	truncated_unqualified_and_oversized_markup_never_produces_action_buttons	no	-
native_npc_document	local_page_links_are_distinct_from_wire_requests_and_stay_in_the_catalog	no	-
native_npc_images	cancelled_image_worker_never_returns_a_publishable_bank	no	-
native_npc_input	formal_numeric_request_bounds_and_reply_match_pinned_protocol	no	-
native_npc_input	quantity_request_is_owned_expiring_and_cannot_be_reused_after_submission	no	-
native_teleport	only_reachable_page_choices_authorize_teleport	no	-
native_teleport	teleport_requires_matching_status_then_endpoint_once	no	-
native_teleport	stale_owner_and_unavailable_slots_cannot_redirect	no	-
native_teleport	every_owner_dimension_rejects_stale_replies_without_consuming_current_pending	no	-
native_teleport	formal_status_vectors_preserve_slots_and_missing_slots_are_unavailable	no	-
native_teleport	formal_endpoint_vectors_keep_little_endian_ip_and_port	no	-
native_teleport	availability_rejects_every_truncation_trailing_data_and_unknown_state	no	-
native_teleport	endpoint_rejects_every_truncation_trailing_data_and_invalid_index_or_port	no	-
native_teleport	same_endpoint_catalog_rebinds_only_matching_routes_once_after_fresh_game_character	no	-
native_teleport	cross_endpoint_or_port_change_never_carries_source_rental_routes	no	-
native_teleport	old_or_skipped_connection_and_wrong_character_consume_the_catalog	no	-
native_teleport	absent_directory_and_routes_for_another_peer_do_not_invent_rental_service	no	-
native_teleport	changing_the_pending_owner_or_destination_cannot_retarget_a_captured_catalog	no	-
native_teleport	remembered_directory_is_an_owned_snapshot_and_connection_overflow_cannot_wrap	no	-
```

### B：唯讀資源根，限本機（20 項）

```text
native_inventory_icons	viewport_changes_publish_only_current_intrinsic_ids_and_close_reclaims_worker	no	-
native_inventory_icons	visible_budget_hidden_rows_and_failures_are_bounded_without_retry_loops	no	-
native_inventory_names	worker_closure_reclaims_loading_ready_and_failed_jobs	no	-
native_ground_pickup	formal_all_unsigned_wire_x_and_quantity_vectors	no	-
native_ground_pickup	capture_requires_actual_type9_resource_and_fresh_render	no	-
native_ground_pickup	stale_pickup_choice_never_rebinds_same_id_or_new_owner	no	-
native_ground_pickup	split_ground_and_inventory_updates_are_observations_not_success_ack	no	-
native_lifecycle	resurrected_life_retires_death_playback_selection_and_ack_without_requiring_a_new_map	no	-
native_npc_document	original_archive_loader_matches_fixture_hashes_and_obeys_cancellation	no	-
native_npc_document	original_hotel_variables_are_plain_text_and_missing_values_fail_closed	no	-
native_npc_document	original_teleport_page_keeps_only_source_actions_and_image_references	no	-
native_npc_images	original_page_images_match_independent_rgb555_reference_and_page_bindings	no	-
native_npc_images	cancelling_the_real_document_worker_does_not_publish_cached_images	no	-
native_painted_input	painted_masks_and_camera_survive_unpainted_idle_ticks_without_copying_canvas	no	-
native_painted_input	painted_surface_rejects_portals_updates_actor_life_and_control_interruptions	no	-
native_arena_portal	original_101_and_102_sources_are_available_without_client_destinations	no	-
native_arena_portal	close_gui_spawn_crosses_real_original_sector_before_source_entrance	no	-
native_map_portal	original_native_entrances_encode_source_only_and_guard_every_owner_change	no	-
native_travel	original_169_frames_sound_and_controls_are_exact	no	-
native_travel	original_169_bitmap_sound_once_terminal_dwell_and_portal_retirement	no	-
```

### C：自有 loopback（2 項）

```text
native_game_timers	admitted_timer_bytes_share_the_ordered_counter_and_final_quit_barrier	no	-
native_game_timers	natural_callbacks_continue_with_a_full_inbound_event_queue_and_no_UI_polling	yes	-
```

### D：自有新建目錄寫入（5 項）

```text
native_library_regression	modern_lito::backend_evidence::r531_evidence_checks::late_d1_e100_and_complete_write_drain_retained_ledgers_once	no	-
native_library_regression	modern_lito::backend_evidence::r531_evidence_checks::unseen_is_unknown_without_coverage_and_scoped_not_observed_is_not_wire_absence	no	-
native_library_regression	modern_lito::backend_evidence::r531_evidence_checks::bounded_journal_retains_first_failure_without_auth_data_or_false_negatives	no	-
native_library_regression	modern_lito::backend_evidence::r532_counterexample_checks::d1_e1_scanned_scopes_and_gaps_are_independent	no	-
native_library_regression	modern_lito::backend_evidence::r532_counterexample_checks::dropped_positive_receipts_rejection_close_and_writes_never_claim_observed	no	-
```

### 不放行（1 項，給 T036-12）

```text
native_library_regression	modern_lito::backend_evidence::r531_evidence_checks::actual_runtime_rejection_close_and_join_remain_distinct_from_world_admission	no	-
```

## 產出路徑

- `docs/reports/issue-310.md`（本檔）
- `維護工具/t036-helper-closure/testclosure.py`（靜態閉合工具）
- 分支 `task/310`

工具的原始輸出（`chains.txt`、`closure.tsv`）和一次性的合併／彙整腳本留在本 worktree 的 `scratch/issue-310/`，不進 git；重跑下面的指令可以重建。

## 給下一棒的提示

- **#311（T036-6）執行**：在 `LiTo381-Modern/` 逐項跑，例如
  `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport -- --exact only_reachable_page_choices_authorize_teleport`。
  - 不要跑整個 target：同檔還有寫檔（#306）、需要 Java peer、GUI、DB 的測試。
  - B 類只能在有 `D:\lineage開發\0-客戶端` 的本機跑。
  - C 類的 `natural_callbacks_…` 要加 `--ignored`，單項約 5 分多鐘。
  - D 類的 target 是 `native_library_regression`，`--exact` 後面接清單裡的完整路徑；跑完會在 `target/current/scratch/native-backend-485f-evidence/` 留下一個新目錄。
  - 略過或沒跑的不要算通過。
- **原始碼一變就要重審**：裁定綁在 `3a81fc2`。`tests/` 或 `src/modern_lito/` 有變更時，在 repo 根目錄重跑
  `python -I 維護工具/t036-helper-closure/testclosure.py LiTo381-Modern <target,…> --chains --tsv scratch/closure.tsv`，
  比對每個測試的原語統計；有新增的 `FS_`／`NET`／`PROC`／`ENV`／`THREAD` 命中就要回去讀該行。
- **#308／#309** 可以直接用 `testclosure.py`，檔頭有規則和限制。它不支援 lib 內的 ignored 測試，也不支援用 `#[path]` 引入 `src/` 測試模組的 target（`native_library_regression`）。
- **B 類的路徑寫死**：20 項都把 `D:\lineage開發\0-客戶端` 寫在測試檔裡，不吃 `LITO_SHARED_RESOURCE_ROOT`。要不要改成讀環境變數、缺資源時 skip，本張沒有動，留給之後決定。
- 同一批測試檔裡 #54 標成「固定檔名寫入」的 7 項（例如 `native_inventory_icons::original_samples_…`、`native_startup::bounded_file_save_…`）已由 #306 裁定，本張沒有重審。
