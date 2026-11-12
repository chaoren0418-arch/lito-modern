# 架構（現況版）— lito-modern

更新：2026-11-12（T032，#326）。依據：`main` @ `64a3796` 的已追蹤檔案，靜態閱讀，沒有執行程式。

本檔只描述「現在的程式長什麼樣、資料怎麼流、要改某件事該去哪個檔」。它不記錄進度與每輪結果：

| 想找的東西 | 去哪裡 |
|---|---|
| 里程碑、任務、完成定義 | `PLAN.md`、`TASKS.md` |
| 每張任務做了什麼 | `docs/REPORT.md`（索引）→ `docs/reports/issue-<N>.md` |
| 封包格式與 opcode | `docs/PROTOCOL-CONTRACT.md`（唯一準則） |
| 各技能的接線與測試狀態 | `docs/SKILLS-STATUS.md` |
| G01–G12 定義原文 | `docs/G01-G12.md` |
| 可參考的外部原始碼 | `docs/REUSE-MAP.md` |
| 測試怎麼跑 | `docs/TESTING.md`、`docs/DEMO.md` |
| 單一子系統的研究細節 | `docs/NATIVE-*.zh-TW.md`（51 篇，見 §9） |
| 2026-11-04 以前的逐輪紀錄 | `docs/HISTORY.md`；本檔的舊版（流水帳，387 行）在 git `92bbda0` |

文中的標記：**已驗證**＝本次直接從原始碼或設定檔讀到；**推測**＝由檔名、註解或文件推得，沒有逐行追過；**待驗證**＝要執行程式或實機才能確認。沒有標記的敘述屬於「已驗證」。

## 1. 一張圖

```
                    啟動現代客戶端.ps1 ──► cargo build ──► LiToModern.exe
                                                              │ src/main.rs：依參數選路由
                ┌─────────────────────────────────────────────┼──────────────────────────────┐
                ▼                                             ▼                              ▼
      原生路由（預設，現行主線）                       離線／診斷路由                  歷史 profile 路由
      native_startup(_window)                    --offline-scene、--verify-resources      --profile <json>
                │ 設定檔 LiToModern.native.json    --probe-native-*（只限 loopback）        session.rs／transport.rs
                ▼                                 --version、--check-native-config         （Modern v37／v47，TLS）
      native_login_window（唯一視窗，UI 執行緒）                                            不相容正式伺服器
        │  20ms WM_TIMER 輪詢
        ├─► LoginController（native_login.rs）── 狀態機 LoginPhase、Snapshot（含 World）
        │        │ 送出 body／收 Event
        │        ▼
        │   native_connection::Controller ──► 執行緒 lito-native-network ──TCP──► LiTo 伺服器（E149）
        │        （native_wire 框架＋加密、native_version 版本交換）
        │
        ├─► native_world_view::Presentation ──► 執行緒 native-world-load
        │        （世代柵欄）                     native_world_scene::Model：讀資源、動畫、CPU 合成 → Frame
        │
        ├─► StretchDIBits 把 Frame 貼到視窗；Win32 子控件顯示清單與表單
        └─► combat_audio::CombatVoicePool ──► audio_output（waveOut）
```

## 2. 目標與邊界

- 目標：Rust 原生 Windows x64 客戶端，對接 LiTo 3.81C 伺服器的正式 JAR **E149**（`E1498871…`），走完「登入→大廳→進房→戰鬥→退出」。
- 這個程式**不啟動、不注入、不包裝** `LiTo.bin` 或 TW 客戶端（`src/lib.rs`、`modern_lito/mod.rs` 檔頭；`--version` 輸出 `launches_native_bin=false`、`launches_tw=false`）。
- 遊戲資源一律從外部客戶端目錄唯讀讀取（預設 `D:\lineage開發\0-客戶端`，可用 `LITO_CLIENT_ROOT` 或設定檔改），不複製進 repo。
- 伺服器是權威：HP／MP、傷害、成本、材料、冷卻結果、狀態期限都以收到的封包為準。客戶端只做送包、預測移動、呈現（`native_skills.rs` 檔頭「No local MP spending」）。
- 伺服器端產品範圍只有競技場與租裝城鎮，沒有 WORLD 大地圖（`PLAN.md` 事實 4）。

## 3. Repo 佈局

| 路徑 | 內容 | 追蹤檔數 |
|---|---|---|
| `LiTo381-Modern/` | 唯一的 Cargo 工程（package `lito-modern` 0.1.0） | 2158 |
| `LiTo381-Modern/src/` | 原始碼，851 個 `.rs`、約 35.4 萬行 | 851 |
| `LiTo381-Modern/tests/` | 整合測試 138 個 `.rs`，加 `fixtures/`（92）、`oracle/`（58）、`support/` | 293 |
| `LiTo381-Modern/tools/` | `build.ps1`、`build-evidence.ps1`，其餘是歷史 r4xx／r5xx 的一次性工具 | 903 |
| `LiTo381-Modern/logs/continuation-20261003/` | 87 個 lib 測試讀的歷史輸入（JSON＋PNG），以白名單入 git | 98 |
| `LiTo381-Modern/config`、`packaging`、`windows` | profile 範例、打包說明、`app.manifest` | 6 |
| `docs/` | 文件；`docs/reports/` 每張任務一份報告、`docs/plans/` 拆單 JSON | 166 |
| `維護工具/` | 測試腳本、研究紀錄（76 篇 `.md`）、歷史 apply／audit／run 腳本（約 1950 個 `.py`） | 2387 |
| `修補提案/` | 要交給伺服器 repo 的修補證據（T040 之後移除） | 37 |
| `啟動現代客戶端.ps1` | 唯一正常入口 | 1 |
| `.github/workflows/build.yml` | CI | 1 |

不追蹤：`LiTo381-Modern/target`、`dist`、其餘 `logs`、`reports`、`provenance`、`驗證紀錄/`、`維護工具/candidates/`、`封存/`、`交付/`、`整理紀錄/`、`scratch/`。

`維護工具/` 與 `LiTo381-Modern/tools/` 的絕大多數檔案是停用流程留下的一次性腳本，**不是建置或執行的一部分**。現行還在用的只有：`tools/build.ps1`、`tools/build-evidence.ps1`、`維護工具/run-local-test-suite.ps1`、`維護工具/test-whitelist/`、`維護工具/run-teleport-failure.ps1`、`維護工具/check-issue-92-evidence.sh`（推測：依 `TASKS.md` 與 `docs/REPORT.md` 引用到的清單，沒有逐檔確認其餘腳本都沒人呼叫）。

## 4. 建置與執行

### 4.1 工程
- `Cargo.toml`：一個 lib（`src/lib.rs`）＋兩個 bin。`LiToModern`（`src/main.rs`，`default-run`）是現行主程式；`LiToModern-485F-Candidate`（`src/bin/native_backend_485f_candidate.rs`）是保留的歷史候選，不是玩家入口。
- feature `teleport-wire-test`：只給 `tests/native_teleport_controller.rs` 用的合成故障 harness，正常建置不含。
- 相依：`windows` 0.59（Win32 GDI、waveOut、IME、HiDpi、Controls）、`rustls`（只有歷史 profile 路由的 `transport.rs`、`profile_probe.rs` 使用）、`aes`、`flate2`、`png`、`roxmltree`、`sha2`、`serde`、`zeroize`、`anyhow`、`base64`、`webpki-roots`；build 相依 `winres`。
- Release profile：`opt-level="z"`、thin LTO、16 codegen units、strip symbols。Debug 只對 `sha2` 開 `opt-level=3`（資源 pin 與影格摘要是熱路徑）。
- `main.rs` 在非測試建置用 `windows_subsystem = "windows"`，沒有主控台視窗。

### 4.2 指令
| 用途 | 指令 |
|---|---|
| 建置並啟動 | `pwsh -NoProfile -File .\啟動現代客戶端.ps1`（`-BuildOnly`、`-Verify`、`-OfflineScene`、`-ConfigureNative`、`-NativeConfig`、`-NativeLoginEndpoint`、`-ProbeNativeEndpoint`、`-ResourceRoot`） |
| 只建置 | `LiTo381-Modern/tools/build.ps1`（支援 `-WhatIf`） |
| 本機測試總表 | `維護工具/run-local-test-suite.ps1` |
| CI | `cargo check --locked --all-targets`、`cargo test --locked --lib --no-fail-fast`（windows-latest，`x86_64-pc-windows-msvc`） |

兩個腳本的 Cargo 參數一致：`--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`。`CARGO_HOME`、`RUSTUP_HOME` 在 `D:\dev-cache`。Linux 無法建置（依賴 `windows` crate 與 WinRes）。

### 4.3 `main.rs` 的路由
`route()` 對參數做**完全比對**（個數與形式不符就報錯，沒有模糊後備）。

| 參數 | 行為 | 建置 |
|---|---|---|
| （無） | 原生路由：讀 exe 旁的 `LiToModern.native.json`；沒有或無效就開設定視窗（`native_startup_window`），然後進 `native_login_window` | 兩版 |
| `--configure-native` | 強制重新設定 | 兩版 |
| `--native-config <絕對路徑>` | 用指定設定檔進原生路由 | 兩版 |
| `--check-native-config <絕對路徑>` | 不連線檢查設定，輸出 JSON | 兩版 |
| `--native-login <IP:port> [資源根]` | 直接開原生登入視窗 | 兩版 |
| `--offline-scene <資源根>` | 離線場景預覽（`scene_preview`） | 兩版 |
| `--verify-resources <資源根> [--output <目錄>]` | 無視窗資源驗證（`rebuild_verification`） | 兩版 |
| `--probe-native-loopback／-login／-game／-lobby <127.0.0.1:port>` | 無視窗診斷，**只接受 loopback**；帳密從 stdin JSON 讀入 | 兩版 |
| `--version`、`--help` | 不連線 | 兩版 |
| `--profile`／`--lito <絕對路徑>`、`--probe-profile` | 歷史 profile 路由（§8） | 兩版 |
| `--lito-preview`、`--lito-scene`、`--canonical-profile`、`--target-haste-profile`、`--cooperative-haste-profile`、`--slow-profile` | 開發用 | 只有 Debug |

設定檔 `LiToModern.native.json`（`native_startup.rs`）只有登入端點與資源根目錄兩項，上限 16 KB。

## 5. 原始碼的兩個世代

`src/modern_lito/` 是**單層目錄**，847 個檔案全部平放，沒有子目錄。模組用檔名前綴分組，並大量用 `#[path = "..."]` 把子模組與測試掛在父檔底下（例如 `native_login.rs` 掛了 `native_login_callbacks.rs` 與十幾個 `*_tests.rs`）。所以「一個檔案屬於誰」要看 `mod.rs` 與父檔的 `#[path]`，不能只看目錄。

| 世代 | 檔案 | 行數 | 定位 |
|---|---|---|---|
| **原生（`native_*`）** | 365 檔（其中 `*_tests.rs` 109、`*_study*` 60、`*_probe*` 20） | 約 7.5 萬 | **現行主線**。對接 LiTo 3.81C／E149 的正式協定 |
| **共用解碼與呈現** | `resources`、`sprite`、`sprite_actions`、`tile*`、`compositor*`、`world_map`、`world_raster`、`world_depth`、`tower_assets`、`combat_audio`、`audio*`、`animation`、`scene*` 等 | — | 兩個世代共用。原生路由直接引用（例如 `compositor` 被 23 個、`world_map` 被 24 個 `native_*` 檔引用） |
| **歷史 profile（Modern v37／v47）** | `session*`、`transport*`、`login_window`、`world_window`、`world_runtime`、`world_scene`、`world_protocol`、`battle_*`、`canonical_*`、`quest_*`、`siege_*`、`time_attack_*`、`race_*`、`shop_*`、`chat_*`、`*_protocol` 等 | 非 `native_*` 檔合計 482 檔、約 27.9 萬行（含上一列的共用部分） | 早期自訂協定（TLS）的客戶端。**與正式伺服器不相容**（`--version` 輸出 `formal_server_compatible=false`；`--help` 寫明「相容性尚待驗證」）。保留但不是目標 |

另有 `src/shared_v1/morph_auth.rs`：從別的專案固定複製的一份來源，不是路徑相依（`lib.rs` 註解）。

`native_*` 裡還有三種非產品檔：
- `*_study.rs`（60 個）：對原版行為的研究模型，多數標 `#[allow(dead_code)]`，沒有接進執行路徑。
- `*_probe.rs`（20 個）：供診斷與整合測試驅動的探針。
- `*_draft.rs`、`native_canonical_*`、`native_pet_race_*`、`native_offline_*`、`native_multi*`、`native_mode_menu*`、`native_city_*`、`native_backend_485f_*`：早期「離線預覽／模式選單／多區塊」實驗，各自有獨立視窗；`mod.rs` 註明「No live transport/profile route」的那幾個沒有接進連線流程（推測：其餘幾組也不在預設原生路由上，依 `main.rs` 沒有路由指向它們判斷）。

**要動功能時，預設只改 `native_*` 與共用解碼層；不要改歷史 profile 世代。**

## 6. 原生路由的分層

由下往上。每層只依賴下面的層。

### 6.1 線路層：框架、加密、版本
| 檔案 | 責任 |
|---|---|
| `native_wire.rs` | TCP 框架（u16 長度，body 上限 65533）、`CipherState`（64-bit 狀態、鏈式 XOR）、`FrameDecoder` |
| `native_version.rs` | `C_VERSION` 請求與 `S_VERSION_CHECK` 回覆 |
| `native_opcodes.rs` | `C_*`／`S_*` 常數表。同一個數值在兩個方向可以是不同封包，常數依方向分開 |
| `native_integrity.rs` | `S_EVENT` subtype 33 完整性挑戰與 `C_ADDR` 回覆。只回答已部署的固定錨點（`0x004ED840`、5 bytes），其他位址一律不回 |

格式以 `docs/PROTOCOL-CONTRACT.md` 為準。文字欄位是 MS950（`native_login::encode_text`／`decode_text`）。

### 6.2 連線層
`native_connection.rs`：`Controller` 擁有一條 TCP。**一個工作執行緒 `lito-native-network`** 負責所有讀寫與取消；UI 端只輪詢有界的 `Event`：

- `Negotiated`：版本交換完成。
- `Packet { epoch, body, rx_counter, receipt }`：一個已解密的完整 body。`receipt`（`IngressReceipt`）是解密完成當下取的時間，給需要「收包時刻」的呈現用。
- `Sent { epoch, tx_counter }`：某個 body 已寫出。

`epoch` 每次連線加 1；用舊 epoch 送出會被拒絕，舊 epoch 的事件會被略過。**所有上層狀態都用 epoch 過濾遲到的封包與回覆**。預設限制：連線逾時 2 秒、握手逾時 5 秒、IO 片 50ms、讀寫塊 4096 bytes。

### 6.3 控制層：`LoginController`
`native_login.rs`（3709 行）是整個原生流程的**唯一狀態機**。它擁有 `Controller`，把 `Event::Packet` 交給各領域的 `parse()`，更新 `Snapshot`，並決定下一個要送的 body。名字叫 Login，實際涵蓋登入到結算的全部階段。

`LoginPhase`（48 個狀態）依序分成這幾段：

| 段 | 狀態 | 領域模組 |
|---|---|---|
| LOGIN 端點 | `Disconnected` → `Connecting` → `ReadyForLogin` → `LoginPending` → `AwaitingSources` → `SourcesReady` → `RedirectPending` → `RedirectReady` → `HandoffClosing` | `native_login`（帳密、來源選服 `S_LITO_SERVER_LIST`） |
| GAME 端點 | `GameConnecting` → `GameLoginPending` → `LoadingCharacters` → `CharactersReady` → `CharacterPending` → `CharacterSelected`（另有 `ClassicEntryPending`、`ClassicCharacterSelected`） | `native_game`（8 列角色目錄；分頁在 `native_character_pages`） |
| 租裝來源 | `RentalSetPending`、`RentalReady`、`RentalArmPending`、`RentalAwaitPermission`、`RentalRestartPending` | `native_rental_source`、`native_services`、`native_merchant` |
| 頻道與大廳 | `ChannelPending` → `AwaitingChannelInfo` → `LoadingRooms` → `LobbyReady`；`ReturningCharacters` | `native_lobby`（4 個頻道、房間列） |
| 房間 | `RoomJoining`、`RoomCreating`、`RoomEditing`、`KickPending`、`ObserverPending`、`ModePending`、`MapPending`、`RoomAwaitSelf`、`RoomReady`、`ReadyPending`、`ForcePending`、`RoomLeaving` | `native_room`、`native_room_ui` |
| 世界與戰鬥 | `BattlePending` → `WorldEntering` → `WorldReady` | `native_world` 與 §6.4 |
| 結算 | `BattleResult` → `ResultReturning` →（`RoomLeaving` → 大廳） | `native_combat`、`native_points` |
| 其他 | `TeleportQuitPending`、`Closing`、`Closed`、`Failed` | `native_teleport`、`native_portal` |

LOGIN 與 GAME 是**兩條不同的連線**：來源選服拿到轉址後關掉 LOGIN（`HandoffClosing`），用新的 epoch 連 GAME 並重新認證。

`Snapshot` 是 UI 唯一讀取的狀態（帳密不進 Snapshot）。重要欄位：`phase`、`sources`、`characters`、`channel`、`rooms`、`joined_room`、`world`、`environment`、`inventory`、`companions`、各種 `*_feedback`。

控制層的共通做法：每個「送出後要等回覆」的動作都有一個 `*_pending` 欄位（`attack_pending`、`skill_pending`、`item_pending`、`pickup_pending`、`door_pending`、`portal_entry`、`teleport`…），綁定 epoch、世界世代與當時的輸入戳記；逾時或被新狀態取代就清掉，遲到的回覆不會復活舊請求。

### 6.4 世界模型
`native_world.rs`：`World` 是目前場景的權威狀態，身分來自 `S_STATUS`／`S_PUT_OBJECT`。內容包括 `Actor` 表、自己的座標與預測、`MovementLock`、`Resurrection`、`Score`、觀戰範圍、待播的線路音效與效果 cue。World 只收封包與本機預測，**不被呈現層修改**（HP 不由動畫改）。

圍繞 World 的領域模組（都是「封包解析＋請求組包＋小型狀態」）：

| 主題 | 模組 |
|---|---|
| 移動、通行、速度 | `native_navigation`、`native_speed`、`native_obstacles`、`native_doors`、`native_terrain_cache` |
| 近戰、遠程、受傷 | `native_combat`、`native_combat_ops`、`native_attack_many`、`native_projectiles`、`native_triple_arrow`、`native_hurt` |
| 目標政策、Ctrl 強制目標 | `native_target_policy`、`native_forced_combat` |
| 技能送包與目錄 | `native_skills`（`request`／`ground_request`／`text_request`／`item_request`／`summon_request`…）、`native_skill_catalog`、`native_skill_selection`、`native_teleport_spell` |
| 技能呈現 | `native_skill_effects`、`native_skill_graphics`、`native_skill_layers`、`native_spell_projectiles`、`native_selector_effects`、`native_ground_effects`、`native_effect_timing`、`native_class1`、`native_ability_buffs`、`native_additional_finite_effects`、`native_dark_elf_effects`、`native_cube`、個別技能（`native_call_lightning`、`native_cone_cold`、`native_disintegrate`、`native_eruption`、`native_tornado`、`native_earth_jail`、`native_static2941`、`native_status4183`） |
| 背包與物品 | `native_inventory`、`native_inventory_names`、`native_inventory_icons`、`native_item_actions`、`native_ground_pickup`、`native_map_item`、`native_tournament_shop` |
| 角色狀態與關係 | `native_actor_states`、`native_actor_relations`、`native_attributes`、`native_hp_meter`、`native_status_icons`、`native_status_family`、`native_status_receipt` |
| 同伴、NPC | `native_companions`、`native_npc_document`、`native_npc_images` |
| 環境與計時 | `native_environment`、`native_game_timers`、`native_clock`、`native_water_clock` |

各技能目前接到哪裡、有沒有測試，以 `docs/SKILLS-STATUS.md` 為準，本檔不重複。

### 6.5 呈現層
- `native_world_view.rs`：`Presentation` 是 UI 與渲染工作之間的唯一介面。它用 `Key { connection, generation, map }` 當**世代柵欄**：連線、世界世代或地圖一換，舊工作的結果就丟棄。它也持有延後交付的音效、受傷回呼與有限特效的歷史（重建場景後不重播）。
- `native_world_scene.rs`：`Job` 在**工作執行緒 `native-world-load`** 上跑 `Model`。`Model` 持有載入好的地形、sprite、文字快取、照明與導航資料，每次產生一個 `Frame`：像素、命中矩形（`hits`）、可見性、這一幀要播的音效、導航快照。`Completion` 把 `Model` 連同 `Frame` 還給 UI，下一幀重用同一個 Model。
- 動畫：`native_world_animation.rs` 的 `Animator`／`Clip`／`Pose`，用資源裡的 `duration_ticks`／FPS 推進；戰鬥動作在 `native_combat_animation.rs`。動畫時鐘與網路座標分開。
- 分層與遮擋：`native_world_layers`、`native_actor_layers`、`native_world_roofs`、`native_object_dispatch`、`native_world_effects`。
- 文字與標記：`native_world_text`（GDI DIB 繪字）、`native_world_labels`、`native_clan_markers`、`native_emblems`、`native_minimap`。
- 照明與水：`native_world_lighting`、`native_light_mask`、`native_light_color`、`native_light_sources`、`native_render_environment`、`native_water_light`。
- 背景：`native_background*`、`native_terrain_pixels`。
- 合成：共用的 `compositor.rs`（CPU RGBA 合成、`Viewport`）與 `world_raster.rs`。**沒有 GPU 路徑**；畫面是 CPU 合成後用 GDI `StretchDIBits` 貼上。

### 6.6 資源層
- `native_resources.rs`：原生命名空間（LTmap／LTtile／LTsprite／Text…）的唯讀存取。每個檔以 `BytePin`／`ArchivePin`（長度＋SHA-256）固定身分，IO 分塊且可取消，不從共用資源猜測後備。`SharedNativeArchives` 讓多個工作共用已開啟的 archive。
- 共用解碼器：`resources.rs`（IDX／PAK）、`resource_cipher.rs`、`sprite.rs`（SPR）、`sprite_actions.rs`（list.spr 動作、影格事件、音效標記）、`tile*.rs`（TIL）、`world_map.rs`（地圖區塊）、`tower_assets.rs`。
- 快取：`native_terrain_cache.rs` 只保留有界的靜態地形視窗，角色與特效不放進去。
- 另一個外部唯讀來源：`D:\lineage開發\客戶端xml\effectlist2.xml`（狀態圖示）。

### 6.7 音效
`combat_audio.rs` 準備 PCM（`PreparedSound`）並管理 `CombatVoicePool`；`audio_output.rs` 用 waveOut 輸出。`native_audio_delivery.rs` 保證同一個事件只交付一次（`Scope` 綁連線與世代、`FiniteDeliveryHistory` 去重）。`native_wire_audio.rs` 處理封包直接指定的音效，`native_ambient_bridge`／`native_ambient_override` 處理環境音。播放在 UI 執行緒發起。

### 6.8 視窗與輸入
`native_login_window.rs`（2990 行）是原生路由**唯一的頂層視窗**，從登入一路用到結算，靠顯示／隱藏不同的子控件切換畫面：

- `Ui` 結構持有 `LoginController`、`Presentation`、音效池、各種選取狀態；包在 `RefCell` 裡，只在 UI 執行緒存取。
- `SetTimer(hwnd, 1, 20ms)`：每個 tick 輪詢控制器與 `Presentation`，把新的 `Frame` 貼上、交付音效、更新控件。
- 表單與清單用 **Win32 標準控件**（EDIT／LISTBOX／BUTTON，Unicode）。原生視窗沒有自己處理 IME 訊息，中文輸入靠標準 EDIT 控件的系統行為（組字、取消、焦點轉移的實機行為**待驗證**）；世界畫面是自繪 DIB。HUD 目前是文字區與清單，不是原版外觀（外觀差距沒有量化過，**待驗證**，見 T034／T030）。
- 子視圖以 `#[path]` 掛入：`native_rental_window`（租裝）、`native_skill_window`（技能書）、`native_companion_window`（寵物與召喚物）。
- 輸入：`native_navigation::HeldInput`（按住移動）、`native_combat::HeldAttack`（按住攻擊）；滑鼠命中用上一幀的 `InputSurface`／`hits`，所以點擊對的是「畫面上實際畫出來的東西」。
- DPI：`windows/app.manifest` 宣告 `dpiAware=true`，版面用 `GetDpiForWindow` 換算。多螢幕不同 DPI、跨螢幕拖曳的實機行為**待驗證**。

設定視窗是另一個小視窗：`native_startup_window.rs`。

## 7. 貫穿各層的不變條件

改任何一層都要守住這幾條，測試也大多在驗這些：

1. **伺服器權威**。客戶端不扣 MP、不算傷害、不自己結束狀態。呈現層不寫 World。
2. **三層世代**。連線 `epoch`（每次重連）→ 世界 `generation`（每次進場／換圖）→ 角色 `incarnation`（同 ID 重新 `S_PUT_OBJECT`）。任何延後的東西（封包回覆、渲染結果、動畫回呼、音效）都要帶著它出生時的世代，交付前比對；不符就丟棄。
3. **一次性**。影格事件、受傷回呼、音效、有限特效各自有去重歷史；場景重建、資源取消、同 ID 替換都不能讓它們重播或復活。
4. **有界**。框架長度、事件佇列、歷史鍵數（例如音效 4096）、地形駐留、設定檔大小都有上限；超過就拒絕或淘汰，不無限成長。
5. **不猜**。未知的壓縮、sprite 型別、opcode、位址一律明確拒絕並附診斷；不用共用資源頂替原生資源，不用別的技能的時序頂替。
6. **時鐘分開**。網路收包時刻（`IngressReceipt`）、原版計時器算術（`native_clock`，無號 32-bit 比較）、資源動畫時鐘（`native_world_animation`）、繪製時刻是四件事，不互相代替。
7. **機密不落地**。帳密用 `Zeroizing`，不進 `Snapshot`、不進診斷輸出。
8. **診斷路由只連 loopback**。`--probe-native-*` 拒絕非 loopback 端點。

## 8. 歷史 profile 路由

`--profile <json>` 走 `session::SessionController` ＋ `login_window`，底層是 `transport.rs`（rustls TLS）與一整套自訂協定（`protocol.rs`、`application_protocol.rs`、各 `*_protocol.rs`），世界畫面是 `world_window`／`world_runtime`／`world_scene`，戰鬥是 `battle_*`。profile schema1 對 v37、schema2 對 v47。

這是早期「先定自己的協定」的產物，正式伺服器不講這個協定。現況：程式碼與它的 lib 測試仍在建置範圍內（佔 lib 測試的多數，**推測**：依檔案數與行數比例），但不在任何里程碑的完成定義裡。是否刪除是不可逆決策，沒有排進任務。

## 9. 測試

| 層 | 位置 | 數量（靜態計數） | 誰跑 |
|---|---|---|---|
| lib 單元測試 | `src/modern_lito/*_tests.rs` 與各檔內 `mod tests` | 約 2875 個 `#[test]`，其中約 583 個 `#[ignore]` | CI（`cargo test --lib`） |
| 整合測試 | `tests/*.rs`（138 檔，多數 `native_*`） | 約 700 個 `#[test]` | 本機 `run-local-test-suite.ps1` 的白名單；多數需要 JDK 與正式 JAR |
| oracle／fixture | `tests/oracle`（正式 JAR 的 cipher／版本／登入向量）、`tests/fixtures`（真正 `ClientSession` 的隔離測試端） | — | 同上 |
| G06.3d | `tests/native_teleport_failure.rs`＋`維護工具/run-teleport-failure.ps1` | — | 本機 |
| 原生 GUI | 完整驗收只由使用者做（T030）；不連線畫面截圖見 `docs/DEMO.md` | — | 使用者／本機 |

數量是對原始碼做字串計數得到的，**不是執行結果**；通過與失敗數請看 PR 的 CI 與 `docs/TESTING.md`（2026-11-04 基線：lib 2463 個，1888 通過、0 失敗、575 ignored）。當版 CI 數字**待驗證**（本次沒有重跑）。

整合測試的環境變數見 `docs/TESTING.md`。隔離測試端一律是 loopback 上自己啟動的程序，不連正式服務或 DB。

## 10. 外部相依與版本 pin

| 對象 | 關係 |
|---|---|
| 正式 JAR E149（`E1498871…`，對應伺服器 0.6.71） | 協定與行為的準則。唯讀。伺服器換版時要重核的清單在 `docs/PROTOCOL-CONTRACT.md`（T025） |
| lito-launcher-server（上層目錄，獨立 repo） | 唯讀參考。它的 `src/main` 比 E149 新，寫進客戶端的格式要先對 E149 核對 |
| tw-bin-engine（`D:\BinEngine`） | 唯讀參考做法；opcode 與部分欄位長度不同，不能照抄 |
| `D:\lineage開發\0-客戶端` | 遊戲資源，唯讀 |
| `LiTo.bin` | 只當靜態逆向依據；不執行、不讀它的記憶體、不包裝 |

## 11. 常見修改要去哪裡

| 要做的事 | 先看 |
|---|---|
| 新增或修正一個封包 | `docs/PROTOCOL-CONTRACT.md` → `native_opcodes.rs` → 對應領域模組的 `parse()`／請求函式 → `native_login.rs` 的分派 |
| 新增一個技能的送包 | `native_skills.rs`（`request` 系列與分類函式）、`native_skill_catalog.rs`；UI 選取在 `native_skill_selection.rs`、`native_skill_window.rs` |
| 新增一個技能的畫面／音效 | `native_skill_effects.rs`、`native_skill_graphics.rs`、`native_effect_timing.rs`；特殊型態參考既有的個別技能模組 |
| 改流程狀態（登入、大廳、房間、結算） | `native_login.rs` 的 `LoginPhase` 與對應 `*_pending`；畫面在 `native_login_window.rs` |
| 改世界畫面的繪製順序 | `native_world_scene.rs`、`native_world_layers.rs`、`native_actor_layers.rs`、`native_world_roofs.rs` |
| 改資源讀取 | `native_resources.rs`；格式解碼在 `sprite.rs`、`tile.rs`、`resources.rs` |
| 改移動或碰撞 | `native_navigation.rs`、`native_obstacles.rs`、`native_doors.rs`、`native_world.rs` 的 `MovementLock` |
| 改啟動參數或設定檔 | `src/main.rs` 的 `route()`、`native_startup.rs` |
| 改建置參數 | `Cargo.toml`、`tools/build.ps1`、`啟動現代客戶端.ps1`、`.github/workflows/build.yml`（四處要一致） |

## 12. 已知的結構性問題

這些是現況，不是待辦清單；有對應任務的已註明。

1. **`native_login.rs` 與 `native_login_window.rs` 過大**（3709／2990 行），而且名稱與職責不符：它們其實是整個客戶端的控制器與主視窗。新功能幾乎都要改這兩個檔，平行開發容易衝突。沒有排定拆分。
2. **單層目錄 847 檔**，產品、研究（`*_study`）、探針、歷史實驗混在一起，靠 `mod.rs` 與 `#[path]` 才分得清。
3. **歷史 profile 世代約佔原始碼行數的七成以上**（含共用層），仍參與編譯與 lib 測試，拉長建置時間；整庫 lib 編譯曾有記憶體失敗的紀錄（`TASKS.md` T036 註記）。
4. **UI 是 Win32 標準控件**，與原版外觀有差距；`docs/REUSE-MAP.md` §4 有後續選項的評估。
5. **`維護工具/`、`LiTo381-Modern/tools/` 有數千個一次性腳本**，內含寫死的絕對路徑；現行流程不用它們。整理歸 T037。
6. **`LiTo381-Modern/README.md` 的描述已過時**（仍寫「預設啟動離線共享資源場景」「`native_*` 是歷史候選」）；現況以本檔為準。修正它不在本張範圍。
7. **87 個 lib 測試讀 `logs/continuation-20261003/`** 的歷史快照（內含原機器的絕對路徑），已標 `#[ignore]`；長期應改用 `tests/fixtures/` 或刪除（`docs/TESTING.md`）。
