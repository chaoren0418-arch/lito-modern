# Issue #182 — T046 第 2／3 張：buff／AC／冷卻／MP 呈現與狀態圖示 lease

日期：2026-11-12（第 1 次嘗試）。分支：`task/182`（起點 `3c569d2`，origin/main）。

**結論：新增 `native_status_icons.rs`，把伺服器送來的狀態時間（ICONS1 36 種、ICON_I2H、娃娃、S_MAGE_SHIELD 的龍之護鎧 3080 與 9 個幻術、自己的 S_SPEED writeH）統一做成「只由伺服器決定時長」的 lease，接進 `World` 的解碼與輪詢。世界畫面的狀態列改成顯示 AC、MP、技能冷卻和 buff 剩餘秒數。MP 只在數值真的改變時才累計通知。`cargo check --all-targets` 結束碼 0；新增 11 個 `--lib` 單元測試，依規定本機不執行，交給 PR CI。**

## 做了什麼

### 1. 新模組 `LiTo381-Modern/src/modern_lito/native_status_icons.rs`

每一項格式都有來源，全部是唯讀參考 lito-launcher-server：

| 封包 | 格式 | 來源 | 客戶端行為 |
|---|---|---|---|
| ICONS1 | `7B c(20) d(204) 204×c`，共 210 bytes | `lito伺服器/LiTo381-Server/tools/combat-continuation/integration-lifecycle/NativeSourceIcons1.java` 的 `RULES`、`WEAPONS`、`duration()` | 嚴格解碼。31 條 slot 規則加上共用武器 slot 10／11（`skill-1`），去重後共 **36 種**。slot 25 是 slot 24 的標記（值 3），72..76 是 epoch。Source 版面以外的 byte 不為 0 就拒絕。每次收到都是完整快照，會取代前一次的 ICONS1 lease |
| ICON_I2H | `7B c(40) h(seconds)` | `NativeSourceProc68Icon.java` | 技能 68（聖結界）的 lease；0 表示清除 |
| 娃娃 | `7B c(56) h(seconds)` | `Protocol.magicDollIcon`、登入器 `native/shared_packetbox.inc` | lease；0 表示清除；大於 32767 時忽略 |
| S_MAGE_SHIELD | `6D h(seconds) c(type)` | `Protocol.mageShield`（receiver sub_4AFF30）、登入器 `native/dragon_status.inc` | type 181 是龍之護鎧，圖示 **3080**（effectlist2 effect107；3079 是技能圖示，不用）。9 個幻術 type `201,204,206,209,211,214,216,217,219` 依序對應圖示 `3123,3117,3097,3115,3127,3113,3119,3125,3111`。秒數大於 32767 時，跟原生一樣拒絕，不動既有 lease，只累計 `refused`。0 表示移除。同一 type 重送只會刷新，不會多出第二筆 |
| S_SPEED（自己） | `29 d(id) c(mode) h(seconds)` | `Protocol.speed`、`NativeSourceDollSlowWire.java`、`NativeSourceDollSlow.java` | 只有自己這個 actor 會把完整的 **writeH** 秒數（0..65535，不截成 byte，娃娃 29 緩速也一樣）當作本機計時。mode≠0 且秒數 0 表示永久。時間到時，自己的 actor 恢復 speed 0。其他人的 S_SPEED 不建立計時 |

- ICONS1 的秒數只能還原出**下限**（`seconds_floor`）。mode 0／4 是 `raw*4`，mode 1／3 是 `raw*16`；mode 2 時 raw 1 是 1 秒、raw 小於 16 是 `raw*2`、其餘是 `raw*4`。文件註解寫明 mode 2 的 raw 8..15 有兩種解讀。
- `Leases::views(now)` 輸出標籤（名稱取自 `native_skill_selection::catalog`；1017 顯示「果實勇敢」）、圖示（只有 3080 和 9 個幻術有確定圖示，其他不猜）以及剩餘秒數（無條件進位）。
- `summary(world, now, cooldown)` 組出一行 HUD 文字，例如 `AC -5　MP 30/50　冷卻 1.4 秒　狀態：龍之護鎧〔圖示 3080〕 1200 秒、…`。最多列 6 項，超過時加上「等 N 項」。

### 2. 接線

- `native_world.rs`：
  - 新增 `status_icons`、`mp_notifications`，以及收包時鐘 `packet_ms`（serde skip，由 `accept_at` 的 `received_ms` 設定）。
  - `0x7B` 子碼 20／40／56 走 lease。子碼 33（完整性挑戰）和其他子碼維持原本的處理。`0x6D` 原本是 unhandled，現在走 lease。
  - `0x29` 的 S_SPEED，如果是自己的 object id，就建立 Speed lease。
  - `0x4D` 和 `0x0C` 改用 `set_mp`，數值不同時才 `mp_notifications += 1`。
  - 新增 `poll_status_icons(now)`：清掉到期的 lease，Speed 到期時把自己 actor 的 speed 歸 0，並且增加 `visual_revision`。
  - 換地圖（0x28）、portal（0x5a）、`clear_preserving_generation` 都會重建 World，lease 會跟著清空，與原生「場景清理」一致。
- `native_login.rs`：在 `poll_status4183` 之後呼叫 `poll_status_icons`。新增 `skill_cooldown_ms(now)`，從既有的 `skill_ready` 期限計算剩餘毫秒。
- `native_login_window.rs`：世界狀態列第一行原本是 `MP x/y`，改為 `summary(...)`，內容包含 AC、MP、冷卻和 buff。
- `mod.rs`：註冊 `pub mod native_status_icons;`。

### 3. 測試（`native_status_icons_tests.rs`，`--lib`，CI 會執行，不需要資源）

測試內有一份 Source 編碼器 `NativeSourceIcons1.packet` 的移植，用來產生測試封包。

1. `icons1_layout_is_the_source_36_state_table`：檢查剛好 36 種、slot 不重複、不佔用共用 slot、只有 155 同時屬於規則表和武器表。
2. `every_icons1_state_round_trips_with_an_exact_floor`：36 種狀態各用 8 種秒數，解碼出的值不能超過實際秒數，誤差必須小於該 mode 的步長。
3. `icons1_shared_weapon_marker_and_full_snapshot`：31 種非武器狀態同時存在；163 的 `skill-1`；133 的標記 3；155 只算一筆。
4. `icons1_rejects_anything_outside_the_source_layout`：未指派 slot、標記錯、少了標記、未知武器、宣告長度錯、總長度錯，都必須拒絕。
5. `world_icons1_replaces_the_previous_snapshot_and_scene_clears`：新快照取代舊快照；非狀態子碼會報錯；換地圖後清空。
6. `dragon_skin_uses_shared_icon_3080_and_native_refresh_remove_refuse`：圖示是 3080 而不是 3079；剩餘 1200／1199 秒；刷新後只有一筆；32768 被拒且不影響既有 lease；送 0 會移除；長度錯誤要拒絕。
7. `nine_illusion_leases_follow_server_duration_and_cancel`：9 個幻術的圖示與名稱；伺服器取消一個，其餘到 64.1 秒時自然到期。
8. `icon_i2h_and_doll_are_word_leases`：建立、清除和長度錯誤。
9. `own_speed_keeps_the_full_write_h_duration_and_expires_locally`：300 秒和 65535 秒都不截斷；到期恢復 speed 0；秒數 0 是永久；mode 0 清除；別人的 S_SPEED 不建立計時。
10. `mp_notifies_only_on_real_change`：同一個值重送不通知，值改變才通知。
11. `summary_shows_ac_mp_cooldown_and_leases`：檢查 HUD 文字的格式，以及超過 6 項時的省略。

## 已驗證

| 檢查 | 結束碼 | 結果 |
|---|---:|---|
| `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 5.55s`（增量；首次完整檢查 3m 47s，同樣是 0）；`native_status_icons*.rs`、`native_world.rs`、`native_login_window.rs` 沒有警告。`native_login.rs:1691 accept_game` 的 dead-code 警告是 main 原本就有的 |

資源靜態查證（唯讀 Python `scratch/probe_status_icons.py`，用 old28 格式讀 `D:\lineage開發\0-客戶端\Sprite.idx`、`Sprite00–15.idx`，輸出 `scratch/status-icons-182.json`，不進 git）：

- `3080.tbt` 在 Sprite03（1 筆，2140 B）；`3079.tbt` 在 Sprite11。
- 9 個幻術圖示各有 1 筆：3123→Sprite01、3117→Sprite04、3097→Sprite11、3115→Sprite02、3127→Sprite05、3113→Sprite00、3119→Sprite06、3125→Sprite03、3111→Sprite14。
- 本張沒有宣稱任何資源缺失。

## 推測

- **opcode 路由**：LiTo.bin 的 packet-box receiver 是 0x7B 的 `sub_4B07C0`，有 0..40 的 switch；登入器 `shared_packetbox.inc` 也寫明「0xFA 是 LiTo clan，絕不改道」。但伺服器 `Protocol.S_PACKET_BOX = 0xFA`，候選 `NativeSourceIcons1`／`Proc68Icon`，以及 E149 的 `magicDollIcon`（`FA 38 h`）都是用 0xFA 送出。本張只在 0x7B 上解碼狀態子碼，0xFA 維持 clan label 的處理。因此 E149 實際送的 `FA 38 h` 娃娃圖示，在本客戶端（和改動前一樣）會被當成 clan label 解析失敗。這是既有的協定落差，不是本張造成的，需要換版重核（PROTOCOL-CONTRACT §9 #11）時一起確認。
- ICONS1、ICON_I2H 只存在於隔離候選，E149 不會送（PROTOCOL-CONTRACT §9 #11）。本張實作的是接收端，E149 下不會被觸發。E149 實際會送的是 `6D`（DragonCombat、IllusionStatusIcons、ClientSession）和 `29`。
- ICONS1 mode 0／1 的 Source 寫法是 `(byte)(seconds/4)` 和 `(byte)(seconds/16)`，分別在 1024 秒和 4096 秒以上會回繞。這時解碼出的下限並不正確，因為線上的值本來就已經回繞。
- 標籤「護盾狀態（SHIELD）」（type 5）、「大地護膚狀態（EARTH_SKIN）」（type 6）取自 `dragon_status.inc` 的註解。原生圖示列在 `0x6328D8`，不在本張範圍，所以這兩種沒有對應圖示。ICONS1 的 36 種狀態也只有文字，沒有 buff 圖示。原版 slot→狀態列的對照需要再逆向 `sub_4B07C0` case 20，本張不猜。
- OwnCharStatus：LiTo 3.81C 的 S_STATUS 0x0C 就是「retained S_OwnCharStatus」的順序（`Protocol.characterStatus` 註解），既有的 `native_status_receipt` 已經解碼。本張只把 AC 和 MP 接到 HUD，並在 0x0C 上做 MP 變更通知。候選的 Source OwnCharStatus 生產端還沒完成（combat-continuation 文件記載），所以沒有新的格式可以接。

## 待驗證／未完成

- **本機沒有執行任何 `cargo test`**（依 2026-11-09 規定）。新的 11 個測試和既有 `--lib` 測試交給 PR 的 GitHub Actions；CI 結果以 PR 頁面為準，本報告不預先宣稱通過。之後 Codex 本機可以跑：
  ```text
  cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_status_icons -- --test-threads=1
  cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_world --test native_minimap -- --test-threads=1
  ```
  門檻以 `run-local-test-suite.ps1` 為準。
- 沒有做 GUI 驗收：狀態列文字長度、是否換行、DPI 都還沒看過。還沒把 3080 和幻術 `.tbt` 圖示畫到畫面上，目前只在文字中標出圖示編號。畫出圖示需要 buff 圖示列 UI，建議另開任務。
- 沒有連正式服務或 DB，也沒有啟動遊戲。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_status_icons.rs`、`native_status_icons_tests.rs`（新增）
- `LiTo381-Modern/src/modern_lito/native_world.rs`、`native_login.rs`、`native_login_window.rs`、`mod.rs`（修改）
- 本報告 `docs/reports/issue-182.md`
- 本機證據（不進 git）：`scratch/check-182.log`、`scratch/probe_status_icons.py`、`scratch/status-icons-182.json`

## 給下一棒的提示

1. T046-3（HPBAR）可以沿用 `World::packet_ms`／`poll_status_icons` 的寫法，HPBAR 的 0xFF 隱藏也是由伺服器決定，不在本機推算。
2. buff 圖示要畫成圖時，用 `Leases::views()` 的 `icon` 欄位，不要和 `spell_icon_route`（技能圖示）混用。3080／3079 的差別已有測試鎖定。
3. 需要主控裁定 0xFA 與 0x7B 的 packet-box 路由落差（見「推測」第 1 點），可併入伺服器換版重核或另開 resource 任務。
4. ICONS1 slot 對應原生狀態列與圖示，需要逆向 `sub_4B07C0` case 20 和 `0x6328D8` 的 77 列表。
