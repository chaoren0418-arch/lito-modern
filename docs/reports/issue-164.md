# Issue #164 — T028-2 狀態與自身類當版補驗（技能 2、13、60，含 SELF_TICK 時鐘）

範圍：日光術 2、偵測術 13、隱身術 60，以及隱身術 SELF_TICK 的原生呼叫者時鐘。工作分支 `task/164`。未套用隔離候選，也沒有改 `docs/REPORT.md`。沒有啟動遊戲、伺服器或任何編譯出來的程式，沒有連正式服務或 DB。

## 做了什麼

### 1. SELF_TICK 原生呼叫者時鐘：已用靜態分析確認，並修正產品

對 recovered PE `LiTo.unpacked-oep-current.exe`（SHA256 `4271D92F…5C2AB6E9`，`lito登入器/舊版分析與驗收/runtime/`，唯讀）和 IDA 全反組譯 `ida-client-oep-v2/full-disassembly.asm` 做了唯讀靜態分析：

- `sub_4CB5F0` 是施法送包的分派函式，`esi` 是從 0 起算的技能索引。佐證：case 0／18／34／56 是同一組治癒分支，對應技能 1／19／35／57。
- 跳躍表：`4CB668 movzx dl, byte_4CBD00[esi]`，接著 `4CB66E jmp jpt[edx*4]`，表在 `0x4CBC80`。`byte_4CBD00[59] = 19`，`jpt[19] = 0x4CBAAD`。
- `0x4CBAAD`：`call 4D69F0; push eax; push 3; push 7; push 14h; push "cccd"(0x502CE0); call 4C6AE0`。也就是說，隱身術的 dword 是 **4D69F0 原生計時器的取樣值**：QPC×1000／頻率取低 32 位元，查詢頻率失敗時改用毫秒備援。它不是另外直接讀一次 OS uptime。
- 技能 2（索引 1）和技能 13（索引 12）都跳到 `0x4CBBD4`，送 `push col; push row; push 14h; push "ccc"(0x5028CC); call 4C6AE0`，**沒有任何尾段**。這與目前 `[0x14,0,1]`、`[0x14,1,4]` 一致。

修正前，`ClientTick::now()` 直接呼叫 `GetTickCount`，和 `native_clock`（4D69F0 的重建）是兩個不同的時鐘來源，epoch 和解析度都不同。修正後：

- `LiTo381-Modern/src/modern_lito/native_skills.rs`：`ClientTick::now()` 改用 `native_clock::sample()`，也就是 controller 建立時初始化的那一個 process clock。時鐘還沒初始化或初始化失敗時，請求直接被拒（「隱身術原生時鐘不可用」），不會改用其他時鐘。
- 同檔新增 `#[cfg(test)]`：
  - `invisibility_tick_is_the_shared_native_clock_sample`：送出的 tick 必須落在前後兩次 native clock 取樣之間（以 wrapping 方式比較）。
  - `original_invisibility_caller_sends_native_4d69f0_sample_as_cccd`：`#[ignore]`，需要 `LITO_NATIVE_CLOCK_PE`。它逐 byte 核對上面的跳躍表、`call 4D69F0`、`cccd`／`ccc` 格式字串和 `call 4C6AE0`。同一組斷言我已經用 Python 腳本 `scratch/issue-164/pe_probe2.py` 對 PE 實際比對，結果 `ok`。
- `native_peer_probe.rs`：收據欄位 `real_os_uptime_provider` 改名為 `native_4D69F0_clock_provider`，避免收據寫錯來源。
- `tests/native_skills.rs` 的既有測試 `invisibility_tick_envelope_guards_clock_and_menu_preserve_authority`：取 tick 之前先建立 `LoginController::default()`（原版 480A60 啟動時就初始化時鐘）。

### 2. 新增離線補驗 `LiTo381-Modern/tests/native_t028_status.rs`（4 個測試，不需要共享資源）

伺服器的事實以 lito-launcher-server `origin/main`（`git show` 唯讀）為準：

- `BattleSpellCatalog.java:305`：`LIGHT = LightSpell(..., 2510, mp 4, reuse 10, 3_600 秒, lightRange 0x0E, sound 145)`。
- `BattleTimedStatus.activateLight/expireLight`：伺服器端 3600 秒的 lease。
- `ClientSession.java:75794`：到期時呼叫 `broadcastLightState`，發 `S_CHANGE_LIGHT(0x1B)`，range 是 `max(物品光源, 0)`（`battleLightRange()`，`ClientSession.java:57239`）。

| 測試 | 內容 |
|---|---|
| `light_has_no_client_duration_and_only_server_change_light_moves_range` | 2：請求只有 `14 00 01`，不帶期限，排入請求時不改 MP 和光源。光源範圍只由 S1B 改變：自己與遠端施法者設成 0x0E；到期後伺服器重送 0 或物品光源 8。客戶端沒有 3600 秒計時。未知 actor 的 S1B 只計入 orphan，不會建立光源。 |
| `light_range_feeds_night_source_selection_with_original_signed_rules` | 2 的夜間亮度輸入：原 4D02F0 曲線在午夜是 6，正午是 15，有天氣時降 1。一般地圖 mask 是 15，走 full-bright bypass；暗地圖（例如 307）是 2。`source_range`：0x0E 選 range 14，遠端施法者同樣適用；到期後的 0 沒有 stencil；物品光源 8；超過 15 截成 15；signed 0x80 視為關閉；致盲時只算自己，固定 5。 |
| `detection_reveal_restores_hidden_actor_and_retires_pre_hide_selection` | 13：請求是 `14 01 04`，帶目標會被拒，選單選擇會丟掉目標，客戶端不會自己揭露。伺服器回 749 效果和 S34 invisible=0 之後，actor 重新可見；但隱身前就選取的 Target 仍然失效，必須重新選取才能對它施放治癒。格式錯誤、截斷或多 byte 的 S34 都不會部分改動狀態。 |
| `invisibility_self_state_is_server_applied_and_cast_stays_self_tick` | 60：`14 07 03 + tick`，tick 落在 native clock 的取樣區間內，排入請求時不會自己隱身。S34 套到自己身上後，自己仍會畫出來（半透明），自己的選取不失效，再施放仍是 self/tick 形狀、不接受目標。遠端 actor 隱身時則完全不顯示。 |

### 3. 白名單

`維護工具/test-whitelist/tests.tsv` 新增 7 列：上面 4 個測試，加上原本列為「白名單外」的 `native_skills::detection_selector_menu_guards_and_cooldown_leave_authority_unchanged`、`native_skills::invisibility_tick_envelope_guards_clock_and_menu_preserve_authority`、`native_light_spell::light_uses_self_selector_without_predicting_authority_and_scopes_UI_choice`。第 3 欄都是 `no`（不需要共享資源）。

## 已驗證

- `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`（在 `LiTo381-Modern/` 執行）：**結束碼 0**，原文結尾 ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 52s``。日誌在 `scratch/issue-164/check.log`（不追蹤）。warning 都是既有的，本張修改的檔案沒有新增 warning。`cargo check` 不會輸出 `test result:`。
- 原版 PE 的靜態事實：跳躍表 case 59 → `0x4CBAAD` → `call 4D69F0` → `cccd`／`14 07 03`；技能 2／13 → `0x4CBBD4` → `ccc`。以 Python 唯讀讀取 PE 位元組並核對 SHA，結果 `ok`。這是靜態證據，沒有執行舊程式。
- 伺服器端日光術 3600 秒、range 0x0E、到期重送 S1B 的來源行號如上（唯讀 `git show origin/main`）。

## 推測

- 4D69F0 在一般機器上會走 QPC 分支。這與 `native_clock.rs` 的既有重建一致，但原 PE 外部 thunk 的實際 import 名稱仍未還原（`native_clock.rs` 已註明）。
- 伺服器端 `clientTick` 目前只用在日誌（`ClientSession.java:58782`），換成不同時鐘來源預期不會影響伺服器判定。這是看程式碼得到的推論，沒有實際驗證。

## 待驗證／未驗證什麼

- **本輪沒有在本機執行任何測試二進位**（使用者 2026-11-09 規定），所以沒有 `test result:` 原文可附。新增與修改的測試以 PR CI 為準。CI 只跑 `cargo test --lib`，會涵蓋 `invisibility_tick_is_the_shared_native_clock_sample`。整合測試 `native_t028_status`、`native_skills`、`native_light_spell` 需要交給 Codex 在本機用 `run-local-test-suite.ps1` 跑（已列入白名單）。ignored 的 PE 靜態測試要用 `LITO_NATIVE_CLOCK_PE=<上述 PE>` 執行：`cargo test --lib modern_lito::native_skills::tests::original_invisibility_caller_sends_native_4d69f0_sample_as_cccd -- --ignored --exact`。
- **2 的夜間亮度還有實作缺口（本張不偽造）**：`native_world_lighting` 的 ambient 目前只用 `map_mask_argument`。一般地圖是 15，直接 full-bright bypass。`native_environment::State.ambient_level`（4D02F0 的日夜曲線）有計算出來，但繪圖端沒有使用，所以一般地圖在夜間施放日光術，畫面不會變亮。原版把 ambient_level 接到 mask 的消費端還沒定位（HISTORY L1298「RGB像素／空間光源合成仍待定位」），需要另開逆向任務找 4D02F0 結果的 consumer。本張只驗證輸入規則（曲線、source_range、map mask）。
- 2 的 3600 秒實際自然到期、13 Release 版的真實施法揭露、60 的真實施法、材料 40318、伺服器回覆順序，都需要隔離的本機 harness 或實機，本張沒有執行。
- `維護工具/oracle/NativeDetectionInvisibilityCandidateProbe.rs` 是舊版 probe，不屬於 crate，沒有編譯。之後如果再用，要先建立 controller 讓時鐘初始化，否則 `ClientTick::now()` 會回錯誤。
- `docs/SKILLS-STATUS.md` 第 66／77／123 列仍寫「未執行（白名單外）」，本張沒有修改（不在範圍內），等測試跑過之後再由後續任務更新。

## 產出路徑

- 追蹤：`LiTo381-Modern/src/modern_lito/native_skills.rs`、`LiTo381-Modern/src/modern_lito/native_peer_probe.rs`、`LiTo381-Modern/tests/native_skills.rs`、`LiTo381-Modern/tests/native_t028_status.rs`、`維護工具/test-whitelist/tests.tsv`、本報告。
- 不追蹤：`scratch/issue-164/`（`check.log`、`pe_probe*.py` 唯讀 PE 檢查腳本、`light.md`／`vis.md` 伺服器文件副本）。

## 給下一棒的提示

- Codex（本機）：執行 `run-local-test-suite.ps1`，確認本張 7 筆白名單和上面那個 ignored 的 PE 測試都通過；任何 0 tests 或 failed 都要回報。
- 第 4 張（S23／冰箭 NativeSpellClock、全域毫秒起點）也用 4D69F0。本張已把 60 的 tick 接到同一個 `native_clock`；第 4 張如果調整全域毫秒起點，要確認隱身術 tick 仍然是原 4D69F0 的語意（未 rebase、低 32 位元）。
- 建議另開 resource／design 任務：定位原版 ambient_level（4D02F0）到光源 mask 的消費端，補上一般地圖的夜間變暗與日光術光圈；在那之前不要在 `native_world_lighting` 加自創的夜間 shader。
