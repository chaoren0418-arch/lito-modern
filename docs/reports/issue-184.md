# Issue #184 — T047 第 1 張（第 2 次嘗試）

## 結論

本輪處理 codex-review 退回的三點：

1. **動畫先於 HP（P1）**：已修正。HUD 的 HP／MP／目標血量改由呈現端 gate 提交。每個值都記錄收到當下該角色已收到的動作序號，要等動畫播放端開始播那個動作，HUD 才換成新值。走路中或前一個動作還沒播完時，HUD 保留舊值。封包接收端照舊立即保存權威值。
2. **有限特效快取（P2）**：已修正。錯誤紀錄不再占用特效庫名額。特效庫改成 LRU 淘汰，正在播放或還沒開始播的特效庫不會被淘汰。名額或位元組暫時不夠時，記為 `deferred`，cue 不消耗，保護集合改變後再重試。
3. **GFX 6983（P1，驗收缺口）**：**仍受阻擋，需要主控裁定驗收範圍**。查過客戶端全部 .idx/.pak（清單見下），6983 只有一組 SPR，每個方向 9 幀。共享與 TW 動作表首幀都引用 subframe 9。`LiTo.bin` 是 Themida 加殼，無法用靜態分析確認原生引擎怎麼處理越界 subframe。沒有來源依據時，我不能自訂「越界＝空白」的規則。

本輪也已合併 `origin/main`（落後 35 個 commit），6 個檔案的衝突已解。

## 已驗證

### 做了什麼

- `native_world.rs`：新增 `hp_barrier`、`mp_barrier`、`hp_ratio_barriers`。收到 S_HP（0x0d）、S_MP（0x4d）、S_STATUS（0x0c）、S_HPMETER（0x68）時，記下該角色當下的 `action_sequence`。`hp_ratio_barriers` 以 incarnation 綁定，最多 1024 筆，超過時清掉已不存在的物件。
- `native_combat_animation.rs`：新增 `Playback::presented(actor, barrier)`。條件是序號不超過 barrier 的 cue 都已被播放端收下（`seen` 已涵蓋），而且不在排隊中，也就是已開始播、已播完或已被合併。`hold_for_motion` 不更新 `seen`，所以走路期間一律回傳 false。
- `native_combat.rs`：新增 `VitalsGate`，負責保留呈現值。每個 world generation 或本機物件換人時重置。目標標籤改用呈現中的 ratio。`held()` 統計被擋下的次數。
- `native_world_scene.rs`：`Model` 持有 `vitals_gate`，每張 Frame 在所有姿勢取樣（含投射物命中後重取樣）之後才提交。姿勢取樣失敗的角色沒有動畫可等，直接提交，避免 HUD 卡住。
- `native_skill_effects.rs`：`FINITE_BANK_LIMIT=32`、`FINITE_ERROR_LIMIT=192`、`FINITE_BYTE_LIMIT=64 MiB`，錯誤與特效庫分開計算。被引用的 GFX 最多 128（S37 環）＋64（播放中），低於 192，所以被引用的錯誤不會被擠掉後一再重試。LRU 由 `finite_victim` 選出，保護集合是播放中與尚未開始的 cue。容量不足時寫入 `finite_deferred`，`draw_to` 對 deferred 的 cue 撤回 `seen`，等之後重試。單一特效庫超過 64 MiB 時給具名錯誤 `finite effect <gfx> alone exceeds ...`。receipt 新增 `deferred`、`evicted`、`retained_bytes`、`error_limit`。
- 合併 main 時，`tower_assets.rs` 保留兩邊的 `ReviewedEarthJail` 與 `ReceivedFinite`。`native_skill_effects.rs` 的混色保留 main 的 2175 Multiply，以及本張的 finite Over。`native_login_window.rs` 使用 main 的 `resolve_for_skill` 並搭配 Frame 的 vitals 標籤。

### 新增測試（`native_t047_tests.rs`）

- `hp_waits_for_action_held_behind_running_walk`：同一批封包先收到受傷動作、HP 與 HPMETER，接著是移動。走路期間畫出 walk，HUD 的 HP 與目標標籤維持舊值，`held()>=2`。走路結束那一幀姿勢變成 action2 `damage`，HP 與標籤才換成新值。這就是審查要求的「走路中先收到動作及 HP，再 draw」情境。
- `hp_and_mp_wait_for_queued_action_behind_active_action`：攻擊播放中又收到受傷動作，加上 HP 與 MP。攻擊還沒播完時，HP／MP 都維持舊值。受傷動作出列開始播後，兩者同時提交。之後沒有待播動作時，新值在下一幀就提交。
- `finite_cache_evicts_least_recent_unprotected_bank_only`：驗證 LRU 選擇、保護集合、全部受保護時回傳 None，以及錯誤上限不小於 128＋64。
- `original_unsupported_flood_does_not_block_valid_finite_effect`（`#[ignore]`，需原資源）：先送 40 個不存在的 GFX，之後送 7049，7049 仍載入並畫出，`started=1`。

### 怎麼驗證

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

結束碼 **0**（合併後與最終各跑一次），完整輸出在 `scratch/check-r2.log`。變更的檔案沒有新的 warning。

**本輪沒有在本機執行 `cargo test`**：使用者 2026-11-09 規定，本機不得執行編譯出來的程式，測試二進位也算在內。新測試交給 GitHub Actions 的 `cargo test --locked --lib --no-fail-fast` 跑，結果以 PR 的 CI 為準。因此本輪沒有 `test result:` 原文。前一輪的 45 passed 是舊程式碼的結果，不能代表本輪。

### 第 3 次同步 origin/main（2026-11-12）

PR #271 在閘門通過後落後 main 62 個 commit，合併時有 6 個檔案衝突。解法如下：

- `native_login_window.rs`：main 把 HUD 的 MP 移進 `native_status_icons::summary`。這裡新增 `summary_with_mp`，傳入 `frame.vitals.mp`，讓 MP 仍然等對應動作開始播放才更新。`summary` 改成呼叫它，傳入 `world.mp`，行為不變。
- `native_world.rs`：S_MANA_POINT 改用 main 的 `set_mp`（會計數 `mp_notifications`），同時保留 `mp_barrier`。結構欄位兩邊都保留。
- `native_world_scene.rs`、`native_movement_tests.rs`：兩邊新增的欄位都保留（`vitals`／`vitals_gate` 與 `status4183_leases`／`local_impacts`）。
- `native_skill_effects.rs`：同時保留有限特效 LRU 快取與 main 的暗黑妖精 bank。選 bank 時先查 `finite_banks`，再查暗黑妖精。有限特效載入器排除 `native_dark_elf_effects::GRAPHICS`，避免同一個 GFX 被兩個 bank 重複載入。receipt 同時輸出 `finite_banks`、`dark_elf_bank`，`load_error` 也包含暗黑妖精的錯誤。
- `tower_assets.rs`：enum 同時有 `ReceivedFinite`、`ReviewedDarkElf`。缺幀時保留本分支的 `MissingFrame` 具名診斷，並加上 main 的暗黑妖精空白幀判定。
- `native_t021_render_tests.rs`（main 新增）：`Active` 初始化補上 `heading: 0`。

合併後執行 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`，結束碼 **0**，輸出在 `scratch/check184.log`。`cargo test` 同樣交給 CI。

### 資源核對（6983）

全部只讀，腳本與輸出在 `scratch/`：

- `scan_all_idx.py` → `scan_all_idx.log`：遞迴掃描 `D:\lineage開發\0-客戶端` 底下全部 .idx，共 24 個：`LiTo/LTicon`、`LTsprite`、`LTsurf`、`LTtext`、`LTtile`、`Sprite`、`Sprite00`～`Sprite15`、`Text`、`Tile`。名稱以 `6983-` 或 `6983.` 開頭的只有 `Sprite10..15`、`Sprite00`、`Sprite01` 的 `6983-0..7.spr`，以及 `Sprite02` 的 `6983.tbt`（圖示）。沒有任何沒有 .idx 的 .pak，`launcher/TW13081901.pak` 除外，它是動作表文字。也沒有散檔 `6983*`。
- `spr_tail.py` → `spr_tail.log`：八個方向的 6983 SPR 都是 9 幀。幀 0、1 是空白 sentinel（192,120，無碎片），幀 2～8 有圖。7049 是 8 幀，幀 0 為空白 sentinel。
- `tw_def.py` → `tw_def.log`（TW13081901.pak，文字 SHA256 `28c4fc60…`）：
  - `#6983 8 illusionist azel att one eff`：`0.ani(1 10,0.9:3 0.0:3<3297 0.1:3 … 0.8:3) 104.attr(8)`
  - `#7049 8 illusionist kiringku effect`：`0.eff(1 8,0.7:3 0.0:0 … 0.5:2! 0.6:3) 102.type(0) 104.attr(8)`
  - 7049 的首幀引用最後一幀 7（幀數 8），6983 的首幀引用 9，超出 0～8 的範圍。
- `LiTo.bin` 用 pefile 讀區段：程式區段熵 7.99，另有 `Themida` 區段，屬加殼檔案。`lito脫殼資源` 內只有零散的 VA 片段，沒有完整的脫殼 PE，因此無法靜態確認原生處理越界 subframe 的分支。
- 程式維持具名診斷：`finite_banks.errors["6983"]` 為 `SPR 6983-0.spr requires subframe 9, decoded frame count is 9`。

## 推測

- 6983 的動作表可能是依 10 幀版本 SPR 撰寫的，而客戶端附的是 9 幀版本。原生引擎可能把越界幀當成空白跳過（類似 7049 首幀停留 3 tick），但目前沒有證據。
- `VitalsGate` 以「動作開始播」作為提交點，不是 Hit 影格。原生端的 HP 是在動作開始時更新還是在 Hit 時更新，仍待原生對照。

## 待驗證

- **需要主控裁定**：6983 要（a）從本張驗收範圍拆出，成為資源阻擋項，或（b）提供原生越界 subframe 行為的動態證據，例如脫殼 PE 或實機觀測，再依證據實作。未裁定前，6983 只有負例診斷，八方向完整播放還沒完成。
- **proc_spr 所需 GFX 清單**：伺服器端 `nativeSourceShowProcSpr` 的 GFX 來自 `NativeProcSkillCatalog`，在 DB 技能表的 `castGraphics` 欄位，見 `lito伺服器/LiTo381-Server/tools/combat-continuation/integration-lifecycle/NativeSourceProc68Graphics.java`、`NativeSourceProcScalarSkills.java`。本機規則禁止連 DB，所以本輪沒列出完整清單。客戶端目前的支援結果：單一 action0、一向或八方向、class0、已支援混色與事件的 GFX 可播，例如 7049。其他 GFX 會給具名錯誤。完整清單需要伺服器端提供 castGraphics 匯出，或由能讀 DB dump 的執行者核對。
- CI 上新測試與既有回歸的結果，push 後看 PR。
- 原資源測試（`#[ignore]`）與離線 PNG：本輪依規定沒有在本機執行，交給 Codex 執行 `--include-ignored`。
- 真機：HUD 扣血時機、走路中受擊、光術完整封包鏈、音效裝置。

## 產出路徑

- 程式：`LiTo381-Modern/src/modern_lito/{native_world,native_combat,native_combat_animation,native_world_animation,native_world_scene,native_skill_effects,native_t047_tests}.rs`，以及合併時解衝突的檔案。
- 報告：`docs/reports/issue-184.md`。
- 本機證據：`scratch/check-r2.log`、`scratch/scan_all_idx.{py,log}`、`scratch/spr_tail.{py,log}`、`scratch/tw_def.{py,log}`，不進 git。

## 給下一棒的提示

1. 主控：先裁定 6983 的範圍（見待驗證第一項）。若拆出，本張其餘項目可依 CI 結果送審。
2. Codex（本機測試）：執行 `cargo test --lib modern_lito::native_t047_tests -- --include-ignored`，以及 `native_combat`、`native_ranged`、`native_actor_states`、`native_effect_lifecycle` 的回歸。產出的 PNG 在 `target/current/issue-184/`。
3. 伺服器端：匯出 proc_spr 使用的 castGraphics 清單，客戶端再逐一跑 `qualify_finite`。

## 第 4 次同步 origin/main（CI 失敗修正）

- 已驗證：PR #271 的 CI（run 43162779710）失敗原因是 `error[E0063]: missing field 'heading' in initializer of 'native_skill_effects::Active'`，位置在 main 新增的 `native_finite_history_tests.rs:19`。CI 跑的是 PR 與最新 main 的合併結果，本機上一輪還沒有這個檔案，所以本機檢查會通過。
- 已驗證：已再次合併 origin/main（ea6c7ef）。`native_skill_effects.rs` 有一處衝突，已解掉：
  - 本分支的 `finite_over` 判斷保留，mix 用 Over。
  - main 新增的 2228 Multiply 也保留。
- 已驗證：`native_finite_history_tests.rs` 已補上 `heading: 0`。
- 已驗證：`cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` 的結束碼是 0，最後一行輸出是 `Finished dev profile ... in 3m 07s`。
- 待驗證：`cargo test` 依規定不在本機執行，結果看 PR #271 的 CI。
