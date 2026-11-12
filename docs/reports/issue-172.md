# Issue #172 — T023 第 4／4 張：G06 物品接續拒絕與期限、T 鍵入口與 cancel-late、畸形與跨世代 wire 回覆

## 做了什麼

在 worktree `task/172`（基底 `c048862`）上改了下面這些。沒有套用隔離候選，沒有改 `docs/REPORT.md`，沒有改原專案目錄，也沒有改相鄰 repo。

### 1. G06 物品接續可到達的拒絕與期限案例（客戶端可觀察的部分）

依據 `維護工具/native-E149-extended-item-observation.md:15-27`、`native-E149-state-copy-preconditions.md`、`native-E149-item-timer-routes.md`：

- StateCopy 拒絕 active ScheduledFuture／EquipmentTimerLease／pending magic-stone expiry 是**伺服器內部**的判斷，wire 上沒有專屬欄位；而且 `item-timer-routes` 的靜態掃描記錄這幾種計時物品在正常 GAME 的建立入口**還沒有證明可達**（`normal_GAME_creation_proven=false`）。所以客戶端不能、也不該去「造」這些狀態。
- 客戶端真正看得到的只有兩種結果：目的端**拒絕接續**（`S_LITO_SELECT_CHAR_RESULT` 非 0），或目的端**一直不回**（伺服器端複製失敗）。這張把這兩種結果固定成測試：都要有界失敗、不送 C8／C9、保留**剛好一份**未使用的關聯（同端點／帳號／來源／角色），之後一次明確 LOGIN 可以接回原角色。
- 租裝自然到期：未到期的列保留原 ID，到期的列消失，重新租的列使用新 ID。客戶端只發布目的端給的背包；來源端的列與舊 ID 都不會帶過去，之後才到的舊 ID 刪除／數量更新只算 orphan，不會把舊 ID 重新授予。

### 2. T 鍵入口

原生視窗之前**沒有 T 鍵**（`docs/reports/issue-146.md` 記錄「未新增歷史候選的 T 鍵循環功能」）。依 `native-E149-zombie41.md:7` 等描述的原行為（從實際已呈現的像素 owner 循環目標；#41 選中時循環屍體）補上：

- `native_skill_selection::cycle_target`（純函式）：只從呼叫端傳入的「已畫出的 id」裡選；跳過自己、不存在的 id、死亡角色（#41 屍體模式則只選屍體）；照呈現順序前進並循環；回傳新抓的 `Target`，不重用舊選取。世界不 ready（傳送交接、取消、失敗後場景已退役）或 own id 不符就回 None。
- `InputSurface::cycle_target`：先檢查 `input_ready`（同 generation／revision／移動 stamp），再用該畫面的 visibility 清單，排除地面物品。
- `native_login_window`：世界有焦點、非 repeat 的 `WM_KEYDOWN 0x54` 呼叫 `cycle_target`；畫面 epoch 必須等於目前連線 epoch。只改選取，不送任何封包；施法／攻擊仍走既有的 gate。

### 3. cancel-late

依 HISTORY L479（自然等待約 3500ms 再取消）。`native-E149-recovery-case-contract.md:11` 寫明 cancel-late 不混入八個 owned controller 案例，所以這張用離線加密 loopback 的控制器測試處理，沒有加新的 owned scenario：

- 目的端 GAME 已連上但不回（`GameLoginPending`），以及目的端已發布角色、等待接續（`CharacterPending`）兩個階段，各自然等 3500ms 再取消：取消在 3 秒內完成，來源端只收到 1 次 C_D9，目的端只連 1 次、沒有 C8／C9，背景不會開新 epoch，保留一份關聯，之後一次明確 LOGIN 接回。
- 接續已完成後才取消：等同一般退出，不產生任何恢復關聯。

### 4. 畸形與跨世代 wire 回覆（目的端接續階段）

#170 做的是**恢復用 GAME** 的舊世代 F4 與畸形 S03。這張補**直接傳送的目的端 GAME**：

- 選角結果之前（接續還沒被消費，要保留關聯）：`c8`（截斷）、`c8 00 00`（多一個 byte）、`c8 05`（未知結果碼）。
- 選角結果之後（伺服器已消費接續，不能復活關聯）：重複的 `c8 00`、截斷的 S7C、S7C 後面多一個 byte。
- 跨世代：目錄還沒發布就收到選角結果；目的端重播來源端的 F3／F4 傳送對。兩種都不能選角、不能換目標、不能送 C9 或對目的端送 C_D9。

### 檔案

- 新增 `LiTo381-Modern/src/modern_lito/native_teleport_continuation_tests.rs`（掛在 `native_login::teleport_wire_tests::continuation`，`cargo test --lib` 會跑）：8 案
  - `destination_refused_continuation_keeps_one_recovery_without_c9`（3 種非 0 結果碼）
  - `destination_silent_continuation_is_bounded_and_keeps_one_recovery`
  - `expired_and_rerented_items_use_only_destination_ids`
  - `malformed_continuation_replies_fail_closed_on_the_destination`（6 種）
  - `cross_generation_continuation_replies_do_not_select_or_retarget`（2 種）
  - `late_cancel_keeps_one_recovery_for_an_explicit_login`（2 個階段，各自然等 3.5 秒）
  - `cancel_after_completed_continuation_creates_no_recovery`
  - `t_key_cycle_is_scoped_to_the_presented_live_world`
- `native_teleport_controller_tests.rs`：加 `pub mod continuation`。
- `native_skill_selection.rs`：`cycle_target`。`native_world_scene.rs`：`InputSurface::cycle_target`。`native_login_window.rs`：T 鍵與 `Ui::cycle_target`。
- `native_teleport_registry.rs`：controller 那一列的 sources 加入新測試檔（13／14 個 key、分組數量不變）。
- `tests/native_teleport_failure.rs`：`recovery_wire`（feature `teleport-wire-test`）加 8 個對應入口。

## 怎麼驗證

**本機規定（2026-11-09）不得執行編譯出來的程式，所以本機只做編譯，`cargo test` 交給 PR 的 GitHub Actions CI。**

工作目錄 `LiTo381-Modern/`：

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
# 結束碼 0；Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 09s
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --features teleport-wire-test --test native_teleport_failure
# 結束碼 0；Finished `dev` profile [unoptimized + debuginfo] target(s) in 36.22s
```

- 兩次 check 的警告都在既有檔案；本張改動的檔案沒有新警告（日誌 `scratch/check172.log`、`scratch/check172-feature.log`，不入 git）。
- 另外以靜態方式核對了測試依賴的路由：`0xc8` 會走到 `native_game::parse`（長度必須是 2、結果碼只能是 0／1／7／8）；F4 只在 `BattlePending` 而且 merchant 等待中才接受；接續期間 `fail()` 用 `teleport_recovery_candidate()` 保留關聯，`Selected(0)` 之後 `teleport=None`，所以不會再保留。

## 已驗證

- 上面兩次 `cargo check` 結束碼 0。
- PR #285 的 GitHub Actions CI（run 43171467479，commit `4ab90d4`，windows-latest）：`cargo check` 與 `test (lib, no GUI)` 都通過，conclusion `success`。

```text
test result: ok. 2246 passed; 0 failed; 595 ignored; 0 measured; 0 filtered out; finished in 24.03s
```

本張新增的 8 案都列為 `... ok`（`modern_lito::native_login::teleport_wire_tests::continuation::` 底下的 `destination_refused_continuation_keeps_one_recovery_without_c9`、`destination_silent_continuation_is_bounded_and_keeps_one_recovery`、`expired_and_rerented_items_use_only_destination_ids`、`malformed_continuation_replies_fail_closed_on_the_destination`、`cross_generation_continuation_replies_do_not_select_or_retarget`、`late_cancel_keeps_one_recovery_for_an_explicit_login`、`cancel_after_completed_continuation_creates_no_recovery`、`t_key_cycle_is_scoped_to_the_presented_live_world`）。符合 `run-local-test-suite.ps1` 的門檻：passed > 0、failed 0、結束碼 0。595 個 ignored 不算通過。

## 推測

- 「StateCopy 拒絕」在 wire 上會表現成選角結果非 0 或一直不回，這是依伺服器流程推的，沒有實際抓封包確認；兩種情況都有測試。
- T 鍵的循環順序用「該畫面 visibility 清單的順序」。原客戶端實際用什麼順序，沒有從 PE 逆向確認。
- 租裝到期的「新 ID」是在測試裡把 fixture 的 ID 改掉來模擬，不是伺服器實際產生的 ID。

## 待驗證／未驗證什麼

- `cargo test` 本機沒跑（規定），以 CI 為準。`--features teleport-wire-test --test native_teleport_failure` 的 8 個新入口 CI 不跑，交給本機測試執行者。
- T 鍵沒有開視窗實測、沒有截圖；GUI 上選取框是否跟著移動，要等不連線截圖或完整 GUI 驗收。
- cancel-late 沒有加 owned E149 scenario／Java fixture／runner（合約寫明另行開發）；只有離線 loopback。
- StateCopy 真實拒絕、真實租期自然到期（33／31 秒）沒有接真的伺服器；非預設屬性（元素、鑲卡、魔石、覺醒、dark-item payload）依 TASKS 列為「G06 正向前提」排除。
- 同端點 C02（selection=0 的 rental-item-set）依賴伺服器，屬 U-M3-1，沒做。
- 沒有連正式服務或 DB，沒有啟停遊戲程序或 GUI。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_teleport_continuation_tests.rs`（新增）
- `LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs`、`native_teleport_registry.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`、`native_world_scene.rs`、`native_login_window.rs`
- `LiTo381-Modern/tests/native_teleport_failure.rs`
- 報告：`docs/reports/issue-172.md`

## 給下一棒的提示

1. Claude 審查重點：
   - 接續階段的關聯保留規則：選角結果前失敗要保留、選角結果後（含重複結果、畸形 S7C）不保留。
   - `cycle_target` 只用呼叫端傳入的已呈現 id，世界不 ready 或 own id 不符就回 None；T 鍵只改選取、不送封包。
2. 本機測試執行者（Codex）：
   - `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib -- teleport_wire_tests::continuation --test-threads=1`
   - `cargo test ... --features teleport-wire-test --test native_teleport_failure -- recovery_wire --test-threads=1`
   - 不連線截圖：離線場景按 T 確認選取在畫面上的目標之間循環、不會選到自己。
3. T023 四張到這張都做完了；剩下的是真伺服器的 owned 案例、cancel-late 的 owned scenario（如果要做）與 GUI 驗收。
