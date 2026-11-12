# Issue #309 — T036（M3 4/15）跨函式 helper 閉合審查：世界／渲染

執行者：claude-local（本機，靜態審查）。審查基準：本分支合併 `origin/main` `1a86a86` 之後的樹（合併 commit `0572a5c`）。
**本張沒有編譯、沒有執行任何測試；下面沒有任何一項是「測試通過」。** 本張只回答：這些測試跑起來會碰到什麼（檔案、網路、子程序、視窗、執行緒），所以能不能交給執行張（#311）去跑。

## 結論

- 本張審 **175 個測試**：#54 標「跨函式／資源 helper 安全邊界尚未封閉」的世界／渲染 152 項、沒有人認領的 21 項，加上 #307 轉來複核的 2 項。175 個在現行 main 都還在。
- **107 個可以放行**（不需要客戶端資源，任何 Windows 機器都能跑）：
  - 100 個純記憶體（P）；
  - 2 個只連自己綁定的 loopback（L）；
  - 5 個只多寫一個自有獨占目錄（W，比照 #306 的 U 類）。
- **67 個可以放行，但只能在有 `D:\lineage開發\0-客戶端` 的本機跑**（R）：唯讀讀取資源根，沒有寫檔、對外網路、子程序、視窗。#307 轉來的 2 項都在這一類，場景鏈已閉合。
- **1 個不放行**（N）：啟動 `native_city_runtime` 並讀原機 snapshot manifest，連線路徑沒有審結，建議交給「原機 snapshot」那一張（T036-12）。
- 107＋67 裡有 4 個早就在 `維護工具/test-whitelist/skills.tsv`。所以**本張新放行 170 個：不需資源 104、需資源根 66**。
- 放行的 174 個測試，閉包裡**沒有**：子程序啟動、建立視窗（HWND）、音效裝置、登錄檔、對外連線、刪除或覆寫既有檔案。

| 分類 | 數量 | 已在 skills.tsv | 本張新放行 | 去向 |
|---|---:|---:|---:|---|
| P 純記憶體 | 100 | 3 | 97 | #311 執行 |
| L 自有 loopback | 2 | 0 | 2 | #311 執行（其中 1 個要跑 5 分鐘以上，見下） |
| W 寫自有獨占目錄 | 5 | 0 | 5 | #311 執行 |
| R 唯讀資源根 | 67 | 1 | 66 | #311 執行（限本機） |
| N 未審結 | 1 | 0 | 0 | 建議交 T036-12 |
| 合計 | 175 | 4 | 170 | |

## 產出

| 路徑 | 內容 |
|---|---|
| `docs/reports/issue-309/approved-candidates.tsv` | 107 項（P＋L＋W），格式同 `維護工具/test-whitelist/tests.tsv`（target、測試名、ignored、features） |
| `docs/reports/issue-309/resource-root-readonly.tsv` | 67 項（R），同格式；`lib` 那 1 列是 lib 單元測試全名 |
| `docs/reports/issue-309/still-excluded.tsv` | 1 項（N） |
| `docs/reports/issue-309/closure-summary.tsv` | 175 項逐項：分類、旗標、閉包大小、非純記憶體位置的裁定統計、測試來源行 |
| `docs/reports/issue-309/sites.tsv` | 閉包內每個副作用原語位置：種類、檔案、函式、裁定、有幾個測試走到 |
| `docs/reports/issue-309/site-adjudication.json` | 逐位置的人工裁定與理由（工具的輸入） |
| `docs/reports/issue-309/scope.tsv` | 本張的 175 列範圍 |
| `維護工具/audit-test-helper-closure.py` | 靜態閉包工具（唯讀，不編譯、不執行專案程式） |

本張沒有改 `維護工具/test-whitelist/*.tsv`，也沒有改任何 `src/`、`tests/`。把清單併進白名單是 #311 的事。

## 本張範圍怎麼定的

Issue 只寫「世界／渲染」，沒有列檔案，所以本張自己切。

- #54 的「不明副作用」458 項：83 項是寫檔（#306 已審），375 項是「helper 未封閉」，分給 4 張。
- #307 已把技能／戰鬥切成 19 個 target、112 項（#307 56、#308 56）。
- 背包／登入／傳送照 target 名稱看是 90 項（`native_ground_pickup`、`native_inventory*`、`native_item_actions`、`native_lobby`、`native_login`、`native_merchant`、`native_npc_*`、`native_room*`、`native_teleport`、`native_travel`），留給 #310。
- **本張取世界／渲染 36 個 target、152 項**，另外把三邊都不像的 **7 個 target、21 項**一起審掉，避免沒人認領：`native_game`（4）、`native_game_timers`（2）、`native_label_updates`（2）、`native_library_regression`（6）、`native_lifecycle`（3）、`native_startup`（3）、`rebuild_wire`（1）。112＋90＋152＋21＝375。
- #307 轉來 2 項（它標 B′，場景鏈沒往下閉合）：`native_ground_skills::original_wave_duration_gates_repeated_ground_audio_and_world_reset`、lib `modern_lito::native_disintegrate_tests::actual_fragment_submission_uses_screen_multiply_and_remains_click_through`。

這 43 個 target 在現行 main 另有 23 個 #54 之後才加的測試（`native_library_regression` 20、`native_observer_camera` 2、`native_game` 1）。它們不在 #54 的 1055 項裡，本張沒有分類。

### 各 target 的結果

| target | P | L | R | W | N |
|---|---:|---:|---:|---:|---:|
| `native_actor_layers` |  |  | 2 |  |  |
| `native_actor_relations` | 10 |  |  |  |  |
| `native_actor_states` | 4 |  | 2 |  |  |
| `native_animation` | 4 |  |  |  |  |
| `native_arena_portal` |  |  | 2 |  |  |
| `native_background` |  |  | 2 |  |  |
| `native_background_geometry` | 3 |  |  |  |  |
| `native_background_pixels` | 4 |  |  |  |  |
| `native_clan_markers` | 1 |  |  |  |  |
| `native_classic_entry` | 1 |  | 1 |  |  |
| `native_effect_lifecycle` | 3 |  | 1 |  |  |
| `native_emblems` | 4 |  |  |  |  |
| `native_environment` | 9 |  |  |  |  |
| `native_game` | 4 |  |  |  |  |
| `native_game_timers` |  | 2 |  |  |  |
| `native_label_updates` | 2 |  |  |  |  |
| `native_library_regression` |  |  |  | 5 | 1 |
| `native_lifecycle` | 2 |  | 1 |  |  |
| `native_light_mask` | 4 |  |  |  |  |
| `native_map_portal` |  |  | 1 |  |  |
| `native_minimap` | 4 |  |  |  |  |
| `native_motion` | 6 |  |  |  |  |
| `native_navigation` | 6 |  | 2 |  |  |
| `native_object_dispatch` | 1 |  |  |  |  |
| `native_observer_camera` | 3 |  |  |  |  |
| `native_obstacles` | 3 |  | 1 |  |  |
| `native_painted_input` |  |  | 2 |  |  |
| `native_resource_loading` | 1 |  | 2 |  |  |
| `native_stalac_layers` | 1 |  | 3 |  |  |
| `native_startup` | 3 |  |  |  |  |
| `native_terrain_pixels` | 3 |  |  |  |  |
| `native_terrain_residence` |  |  | 1 |  |  |
| `native_water_light` | 2 |  | 1 |  |  |
| `native_world` | 5 |  |  |  |  |
| `native_world_effects` |  |  | 3 |  |  |
| `native_world_entry` | 1 |  | 1 |  |  |
| `native_world_labels` | 1 |  | 1 |  |  |
| `native_world_layers` |  |  | 4 |  |  |
| `native_world_lighting` | 1 |  | 10 |  |  |
| `native_world_lighting_scene` |  |  | 20 |  |  |
| `native_world_roofs` | 2 |  | 2 |  |  |
| `native_world_text` | 1 |  |  |  |  |
| `rebuild_wire` | 1 |  |  |  |  |
| #307 轉來：`native_ground_skills`、lib |  |  | 2 |  |  |
| 合計 | 100 | 2 | 67 | 5 | 1 |

同一批 target 裡還有 #54 標為寫檔的測試（例如 `native_world_layers::actual_terrain_body_and_ground_fragments_match_independent_pixels_and_owners`），那些歸 #306，本張沒有審。

## 怎麼審的

`docs/REUSE-MAP.md` 已查：本張是本 repo 測試的副作用審查，表內沒有對應列，沒有引用 tw-bin-engine 或 lito-launcher-server 的檔案。本張沒有宣稱缺資源，所以沒有做 pak 查詢。

1. **取範圍**：從 #54 報告的表格機械取出 173 列，加 #307 的 2 列，寫成 `scope.tsv`。
2. **算每個測試的呼叫閉包**（`維護工具/audit-test-helper-closure.py`）。工具自己做詞法與項目解析，索引 `src/` 與 `tests/` 的 1000 個 `.rs`、20,903 個項目，再從每個 `#[test]` 往下走：
   - `Q::name`：Q 是 crate 內型別就走那個型別 impl 裡的 `name`；Q 是模組就走那個模組的項目。型別與模組依該檔的 `use`、glob、`super` 解析，不是只比名字。
   - `.name(`：所有「已觸及型別」上叫 `name` 的方法都算（做到固定點）。
   - 觸及一個型別時，把它的定義（欄位型別遞移）、它所有的 trait impl（含 `Drop`、`Default`、`From`）和那些 trait 的預設方法都算進來。
   - 整合測試的範圍是 lib 的非測試程式碼加上該 target 自己的檔案；`#[cfg(test)]` 的項目不算（整合測試連不到）。`cfg(any(test, feature = …))` 保守算進來。
3. **在閉包內找副作用原語**：檔案讀、檔案寫、路徑探測、網路、子程序、環境變數、執行緒、時鐘、Win32／`unsafe`／FFI、`static mut`、stdio、panic hook。每個命中都記下一條見證路徑。
4. **逐位置人工裁定**：175 個測試一共走到 186 個位置（種類×函式，見 `sites.tsv`），裁定寫進 `site-adjudication.json`。工具沒有對到裁定的位置一律當 unknown，該測試不放行；最後結果裡 unknown 是 0。
   - 時鐘 51 列：只影響計時，不另外讀。
   - 資源根 39、CP950 16、loopback 10、寫檔 2、GDI 2、純記憶體 2、誤報 14：逐一讀過所在函式。
   - 執行緒 29 列：依命中的呼叫本身裁定（都是 `std::thread::sleep`、`thread::scope`、`Builder::spawn`、`JoinHandle`），沒有逐函式讀。
   - 未審結 21 列：全部只被 N 類那 1 個測試走到，沒有審。
5. **測試的分類**取它走到的位置裡最嚴重的那一個：N ＞ W ＞ L ＞ R ＞ P。

重跑指令（在 repo 根目錄，不到一分鐘）：

```powershell
$t = 'native_actor_layers,native_actor_relations,native_actor_states,native_animation,native_animation_resources,native_arena_portal,native_background,native_background_geometry,native_background_pixels,native_city_resources,native_clan_markers,native_classic_entry,native_effect_lifecycle,native_emblems,native_environment,native_light_mask,native_map_portal,native_minimap,native_motion,native_navigation,native_object_dispatch,native_observer_camera,native_obstacles,native_painted_input,native_resource_loading,native_stalac_layers,native_terrain_pixels,native_terrain_residence,native_type12_prop,native_water_light,native_world,native_world_effects,native_world_entry,native_world_labels,native_world_layers,native_world_lighting,native_world_lighting_scene,native_world_roofs,native_world_text,native_game,native_game_timers,native_lifecycle,native_library_regression,native_startup,rebuild_wire,native_label_updates,native_ground_skills'
python -I 維護工具/audit-test-helper-closure.py --targets $t `
  --lib-tests modern_lito::native_disintegrate_tests::actual_fragment_submission_uses_screen_multiply_and_remains_click_through `
  --out scratch/issue-309/closure.json `
  --scope docs/reports/issue-309/scope.tsv `
  --adjudication docs/reports/issue-309/site-adjudication.json `
  --summary-dir docs/reports/issue-309
```

`closure.json`（約 12 MB）含每個命中的完整見證路徑，放 `scratch/`，不進 git。

## 已驗證（靜態讀碼與工具輸出，不是測試結果）

### 工具本身的對照檢查

用 #54 與 #306 已有定論的項目當對照組，在同一棵樹上跑同一支工具：

| 對照組 | 項數 | 工具結果 |
|---|---:|---|
| #54 白名單的整合測試（安全） | 76 | 70 個沒有任何檔案／網路／子程序／環境變數命中；6 個命中網路，全部是 `native_network` 的自有 loopback，和 #54 的記載一致 |
| #54 標寫檔的整合測試（#306 範圍） | 72 | 72 個都找到寫檔位置。其中 `native_teleport_controller` 6 個的寫檔在 `src/` 的另一個檔案，要跨檔、跨 `cfg(any(test, feature))` 才走得到 |
| `native_network::stalled_seed_…`（不寫檔，連釋放後的 port） | 1 | 沒有寫檔命中，有網路命中，和 #306 的 N 類一致 |

第一版工具漏了 `native_teleport_controller` 那 6 個（把 `cfg(any(test, feature = "teleport-wire-test"))` 當成只在測試編譯），就是這個對照抓出來的，已修正。

### 逐位置裁定的重點

完整清單在 `sites.tsv` 與 `site-adjudication.json`。

| 位置 | 內容 | 裁定 |
|---|---|---|
| `resources.rs`：`open_readonly`、`SharedArchive::open_index`、`read_shared_loose_file`、`reject_reparse_chain`、`verify_open_path`、`final_handle_path` | `OpenOptions` 只設 `read(true)`；Windows 上 `share_mode(FILE_SHARE_READ)`。其餘是 `canonicalize`、`symlink_metadata`、`GetFinalPathNameByHandleW`。沒有寫入、建立、刪除 | 唯讀資源根 |
| `native_resources.rs`：`PinnedFile`、`NativeRetainedGeometry`、`NativeObservedArchives`、`SharedNativeArchives` | 用上面的 `open_readonly` 開檔、算 SHA-256，handle 持有到 owner 釋放 | 唯讀資源根 |
| `native_world_scene.rs`：`read_pin`、`Model::load_terrain`、`Job` | 唯讀開檔與雜湊；`Job` 開一條行程內的 `native-world-load` 執行緒 | 唯讀資源根＋執行緒 |
| `native_water_light.rs::Shade::load`、`shared_action_catalog.rs::reuse_for_model`、三個 `Effects::prepare_with_loader` | `File::open(root/shade.dat)`、`canonicalize`、`symlink_metadata` | 唯讀資源根 |
| 測試檔自己的 `std::fs::read_dir`／`read`（`native_arena_portal`、`native_navigation`、`native_terrain_residence`、`native_water_light`、`support/native_fragment_reference`） | 寫死讀 `D:\lineage開發\0-客戶端\LiTo\LTmap\<map>\*.s32`、`shade.dat` | 唯讀資源根 |
| `native_world_text.rs::raster` | `CreateCompatibleDC(None)`、`CreateFontW`、`CreateDIBSection`、`TextOutW`。記憶體 DC，不建立視窗，`Drop` 釋放 | 行程內 GDI |
| `native_login.rs::decode_message／encode_text`、`descriptions.rs::decode_cp950`、另 5 處 | `MultiByteToWideChar(950)`／`WideCharToMultiByte(950)` | 純記憶體 |
| `compositor_dib.rs::copy`、`native_clock.rs` | SSE2 記憶體複製；`QueryPerformanceCounter` | 純記憶體／時鐘 |
| `tests/native_game_timers.rs` 與 `native_connection.rs` | 測試 `TcpListener::bind("127.0.0.1:0")`，再把 `listener.local_addr()` 交給 `Controller::start`（`:113–146`、`:194–238`） | 自有 loopback |
| `native_backend_485f_evidence.rs::save_test_report` | 寫 `LiTo381-Modern/target/current/scratch/native-backend-485f-evidence/run-<pid>-<n>/`；目錄用 `create_dir` 搶占，檔案用 `create_new`，不覆寫既有檔 | 寫自有獨占目錄 |
| `native_startup.rs::Settings::validate` | 只解析 `IpAddr` 值，沒有 socket | 純記憶體 |

兩個工具誤報，人工排除：

- `native_city_runtime.rs` 的 `Command::Back／Enter／Pan` 是該模組自己的 enum，不是 `std::process::Command`。
- `native_world_layers::ground_pointer_uses_visible_pixels_and_never_rebinds_a_stale_item` 被標成會走到 `native_connection::Controller::start`。實際是 `native_skill_effects.rs:671` 的 `clip.start()` 被方法名對上；測試本體（`tests/native_world_layers.rs:278–325`）只用 `LoginController::default().snapshot()`，沒有呼叫 `start` 或 `connect`。這項歸 R。

### P 類 100 項的第二道檢查

對每個 P 測試，把它閉包碰到的 `src/` 檔案整檔掃一次（不管函式有沒有被走到）：

- 35 項：碰到的檔案整檔都沒有檔案、網路、子程序、環境變數原語。
- 65 項：同檔有這類原語，但所在函式不在閉包內。這些函式是 `native_login.rs` 的 `probe_flow`、`LoginController::login／poll_inner`，`native_connection.rs` 的 `Session::connect` 等，`native_startup.rs` 的 `Settings::save／load`，`native_static2941.rs`、兩個 `prepare_with_loader`，以及各 1 項的 `native_combat_probe.rs::Probe::tick`、`rebuild_verification.rs::run`。
- 抽讀確認：`native_startup` 3 項只呼叫 `Settings::new／parse`、`default_path`（同檔第 4 個會 `save` 的測試是 #306 的寫檔項）；`rebuild_wire` 只呼叫 `verify_formal_cipher(include_bytes!(…))`。

### #307 轉來的 2 項

| 測試 | 結果 |
|---|---|
| `native_ground_skills::original_wave_duration_gates_repeated_ground_audio_and_world_reset` | R。閉包 1,754 個項目、105 個檔案，走完整 `native_world_scene::Model::load／draw_at`。命中只有唯讀資源根、GDI 文字、載入執行緒；沒有走到 `audio_output.rs` 的裝置函式 |
| lib `…native_disintegrate_tests::actual_fragment_submission_uses_screen_multiply_and_remains_click_through` | R。閉包 604 個項目、64 個檔案，命中只有唯讀資源根與 CP950 轉換。它是 `#[ignore]` 的 lib 測試，要加 `--ignored` |

### 歷史失敗在本張能靜態確認的部分

- **runner 漏 pin `native_room.rs`**（NATIVE-WORLD-INTEGRATION:13）：`維護工具/run-native-arena-portal.ps1:43` 與 `維護工具/run-native-map-portal.ps1:43` 的來源 pin 清單現在都含 `LiTo381-Modern/src/modern_lito/native_room.rs`。兩支 runner 自初始 commit `92bbda0` 之後沒有再改。「下輪關閉」需要實跑 runner，見待驗證。

## 推測

- **R 類執行時會鎖住客戶端資源檔**：`open_readonly` 用 `FILE_SHARE_READ` 開 `LTsprite.pak`、`LTtile.pak`、`.idx`、`.s32`、`shade.dat`，持有期間別的程序不能以寫入或刪除權限開這些檔。如果這時有遊戲客戶端或登入器要以寫入權限開同一個檔，會是對方失敗；反過來對方先佔住寫入，測試會失敗。我沒有查 `LiTo.bin` 或登入器開這些檔用的權限，所以只列推測。建議跑 R 類時不要同時啟動或更新客戶端。
- **GDI 文字結果取決於主機字型**：`CreateFontW` 指定 `CHINESEBIG5_CHARSET`、高度 12，沒有指定字體名。走到它的 32 個測試都在 R 類（本來就只能在本機跑），所以不影響 P 類進 CI。換一台字型不同的機器，像素比對可能不同。
- **CP950**：56 個 P 測試會呼叫 `MultiByteToWideChar(950)`。Windows 內建這個 code page，GitHub 的 windows-latest 應該可用；沒有實測。
- **計時敏感**：P／L 裡有 3 項用真實時鐘加 `sleep` 輪詢：`native_environment::real_monotonic_water_poll_rearms_once_per_ready_generation_and_retires_game_callbacks`、`native_game_timers` 2 項。R 類有 33 項會開載入執行緒或 `sleep` 輪詢（`native_world_lighting_scene` 20 項為主）。機器忙的時候可能逾時，失敗時先看是不是逾時。
- **schema34 首次失敗**（NATIVE-RESOURCE-LATENCY:17）對應的邊界測試，從名稱看是 `native_resource_loading::ranged_capture_floor_waits_without_a_frame_rate_assumption`（P）。我沒有讀它的斷言去確認涵蓋文件說的五種情境。

## 待驗證

- **175 項全部沒有實跑。** 放行只代表副作用邊界已閉合，會不會通過要等 #311。
- **N 類 1 項**：`native_library_regression` 的 `modern_lito::backend_evidence::r531_evidence_checks::actual_runtime_rejection_close_and_join_remain_distinct_from_world_admission`。它讀 `logs/continuation-20261003/r486-map-binding.json`（原機 snapshot manifest），啟動 `native_city_runtime::Runtime` 的工作執行緒，最後也寫 `save_test_report`。測試自己斷言 `workers_started == 0`，但 Runtime 內可以走到 `native_login_transport::Transport::connect`，我沒有把「實際不會連線」審到底。
- **工具的限制**（所以結論是「靜態閉合」，不是證明）：
  - 呼叫解析是自寫的近似，不是編譯器。方法呼叫靠「已觸及型別」的名字，對照組全過，但不保證沒有漏。
  - 巨集產生的程式碼只看 `macro_rules!` 本體；外部 crate（`serde`、`png`、`sha2`、`rustls`、`windows`）當成黑箱，只看呼叫點。
  - `build.rs` 與連結期行為不在範圍。#54 已記錄沒有 `ctor`／`link_section` 這類啟動掛勾，本張沒有重查。
  - lib 測試模式（#307 那 1 項）把 `#[cfg(test)]` 程式碼全部算進來，比整合測試更保守，但只用在 1 項上，沒有對照組。
- **歷史失敗要實跑才能關的**：
  - 房間切換 90 秒逾時（NATIVE-SCENE-LOADING:9）：是完整回歸的 GUI 探針，不是 cargo 測試。相關的 cargo 測試是 `native_world_lighting_scene` 的載入 Job 20 項（R），可由 #311 在本機跑；探針本身要實機。
  - Release573 嚴格地形（NATIVE-WORLD-INTEGRATION:150）：要 30 人實機場次。相關的 cargo 測試是 `native_terrain_residence`（R 1 項）與 `native_arena_portal::original_101_and_102_sources_are_available_without_client_destinations`（R，讀 map 573／574）。
  - runner pin：兩支 runner 重跑一次，確認收據裡有 `native_room.rs` 的 SHA。
  - 整庫 lib 編譯記憶體失敗（NATIVE-WORLD-INTEGRATION:5、:9、:177）：本張 175 項裡只有 1 項是 lib 測試，其餘是各自獨立的整合 target。整合 target 會不會碰到同一個記憶體問題，沒有量測。
- 23 個 #54 之後新增的測試沒有分類（見「範圍」）。

## 給下一棒的提示

- **#311（執行）**：
  - 清單是 `approved-candidates.tsv`（107）與 `resource-root-readonly.tsv`（67）。扣掉已在 `skills.tsv` 的 4 項再併入白名單：`native_actor_states::hidden_targets_are_rejected_until_visible_and_identity_replacement_invalidates_selection`、`native_obstacles::destroyed_guardian_retires_attacks_spells_and_selection_even_if_repaired_between_polls`、`native_stalac_layers::stalac_packet_range_clock_and_no_predicted_authority`、`native_world_entry::learned_world_tree_is_selector_only_and_cross_map_admission_stays_owned`。
  - 一律 `--exact` 逐項跑，不要跑整個 target：同檔還有 #306 的寫檔測試和需要 Java peer 的測試。
  - `native_game_timers::natural_callbacks_continue_with_a_full_inbound_event_queue_and_no_UI_polling` 是 `#[ignore]`，註解寫明要等原始的 60 秒／300 秒間隔，至少跑 5 分鐘，要加 `--ignored` 並放寬逾時。
  - `native_library_regression` 的 6 項，cargo 的測試名是 `modern_lito::backend_evidence::r531_evidence_checks::…`、`…::r532_counterexample_checks::…`（#54 用 `--list` 列出來的全名）。
  - R 類在 GitHub Actions 上會在 `unwrap` 失敗（沒有資源根），只能本機跑；跑的時候不要同時啟動或更新客戶端（見推測第 1 點）。
  - #307 的報告把執行張寫成 #310，實際是 #311；#310 是背包／登入／傳送的審查。
- **#310（背包／登入／傳送審查）**：未歸屬的 21 項本張已審，不用再審。工具可以直接用：換 `--targets` 與 `--scope`，把新走到的位置補進自己的裁定檔；`site-adjudication.json` 裡資源根、CP950、GDI 那幾條規則可以原樣沿用。
- **#308（技能／戰鬥 2／2）**：如果又遇到走場景鏈的測試，`Model::load／draw_at` 的閉合結果就是上面「逐位置裁定」那張表，可以直接引用。
- **T036-12（原機 snapshot）**：請接手 N 類那 1 項。
- **T036-7／8（共享資源根）**：本張的 R 類 67 項不在那 159 項裡（它們寫死路徑、沒有 `#[ignore]`，#54 歸在「不明副作用」）。資源根的鎖檔行為同樣適用於那 159 項。
