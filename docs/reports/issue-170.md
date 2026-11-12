# Issue #170 — T023 第 2／4 張：3 個舊 scenario、更多取消／舊世代、G06 通用 finite History 四 profile

## 做了什麼

在 worktree `task/170`（基底 `b3ccf01`）上改了下面這些。沒有套用隔離候選，沒有改 `docs/REPORT.md`，也沒有改原專案目錄。

### 1. 補回 3 個舊 scenario（`tests/native_teleport_failure.rs`）

依 `維護工具/native_E149_recovery_contract.py` 與候選 `native-E149-morph-denial-20261024-010728-014199` 的舊測試名稱補回正式（ignored）入口：

| scenario | 測試入口 | 行為 |
|---|---|---|
| `cancel-repeat` | `repeated_cancellation_preserves_only_one_authenticated_recovery` | 目的端 TCP 建立中連續 cancel 4 次；回收（quiescent）後再 cancel 2 次，每次都檢查 phase／error／epoch 不變；之後一次明確 LOGIN 接續原角色 |
| `refused-cancel-repeat` | `failed_handoff_then_repeated_cancel_preserves_the_authenticated_recovery` | 目的端拒絕連線後有界失敗（沿用 refused 的檢查），回收後再 cancel 2 次，失敗訊息與 epoch 保持不變；之後一次明確 LOGIN 接續 |
| `cancel-expiry-source` | `expired_stage_and_source_handoff_require_the_requested_source_directory` | #169 的 `source_handoff_expiry_confirms_c8_before_manual_selection` **改回舊名**，內容不變（自然 33＋31 秒） |

- 結果 JSON 新增 `cancellation_calls`；cancel-repeat 至少 6 次，refused-cancel-repeat 至少 2 次。
- runner `維護工具/run-teleport-failure.ps1`：ValidateSet 加入兩個新 scenario、測試名稱對應表更新（含改名）、`$refusedDestination` 讓 refused-cancel-repeat 沿用「目的端不得有 listener」的檢查、新增重複取消的證據檢查。session 數量與 lease 都不變（伺服器端看到的 session 和 cancel-retry／refused 相同）。
- Java fixture `NativeTeleportFailureHarness.java`：接受這兩個 scenario；`refused-cancel-repeat` 走同一個已關閉的 loopback 目的端。
- registry（`native_teleport_registry.rs`）只更新兩列的說明文字；13／14 個 key、scope、分組都沒變。

### 2. 更多取消／舊世代、畸形與跨世代 wire 回覆（`native_teleport_controller_tests.rs`，離線加密 loopback）

新增 4 個案例，CI 的 `cargo test --lib` 會執行；integration 檔的 `recovery_wire` 也同步開放：

1. `repeated_cancellation_before_and_after_quiescence_keeps_one_recovery`：目的端 epoch 2 時連續 cancel，回收後再 cancel 3 次。檢查只留下一份關聯（同端點／帳號／來源／角色）、舊端只收到 1 次 C_D9、背景不會開新 epoch，之後一次明確 LOGIN 接續成功，不送 C8／C9。
2. `failed_handoff_then_repeated_cancel_keeps_the_authenticated_recovery`：endpoint 指向已關閉的 loopback port，等待有界失敗後 cancel 3 次。檢查 phase 仍是 Failed、錯誤訊息不變、關聯保留，之後明確 LOGIN 接續成功。
3. `recovery_game_rejects_previous_generation_endpoint_reply`：恢復用的 GAME epoch 收到上一代傳送的 F4 endpoint。檢查會失敗、不換目標、不開新 epoch、不送 C8／C9／C_D9；已取用的關聯不會因為 cancel 復活。（對應 HISTORY L1155／L1172／L1227 的「跨世代 wire 回覆」；L1172 本身只是舊 GUI 通知重播。）
4. `recovery_game_malformed_directory_reply_fails_without_retry`：恢復用的 GAME 收到畸形 S03（缺 count、多一個 byte、負數 count）。檢查會失敗、不重試 C8、不送 C9、不留恢復。

另外把 `authenticated_recovery_redirect` 拆成 `explicit_recovery_redirect`＋`complete_explicit_recovery`，原有案例的斷言沒變。

### 3. G06 通用 finite History 四 profile（HISTORY L45）

先前的缺口：`FiniteHistory::for_source(false)` 讓 Skills／Items／Reagent／Travel 四個 profile 整個不還原。來源一換，generation／seen／計數就全部歸零，同一個 S37 ring 會重新啟動 graph 和音效。另外 `finite_visual_history` 要等到下一次發請求時才判斷來源；A→B→A 時，如果 B 的 worker 還沒完成，A 的 graph 會被帶回來。

修正如下：

- `native_skill_effects::History`：新增 `retired()`（清掉 active graph，保留 generation／seen／started／omitted）、`merge_admission()`（同一代的晚到 worker 只併 seen 與計數，不帶入它的 graph；不同代不併；空的 history 只採用晚到的 admission）、`active_sequences()`。
- `native_world_scene::FiniteHistory`：`for_source(false)` 改成讓四個 profile 走 `retired()`；新增 `retire_source()`（`restore_other_effects` 只能由 true 變 false，所以 ABA 也不會復活）；`merge_disintegrate_admission` 也合併四個 profile 的 admission；新增 `active_finite_sequences()`。`restore_finite_history` 一律還原四個 profile（來源不同時，graph 在前一步已經清掉）；`many` 仍照原本的 `restore_other_effects` 處理。
- `native_world_view`：`root_changed` 發生時立刻 `retire_source()`，不等 B 的 worker；音效 history 的保留序號加入四個 profile 仍在播放的序號，所以 raw ring 淘汰後不會提早釋放 key。來源退役後 active 已清空，音效也不再保留。
- 單元測試 `native_finite_history_tests.rs` 共 6 案：退役清 graph 但保留 admission、root ABA 不復活、同代晚到 worker 只併 admission、他代晚到 worker 不貢獻任何東西、四 profile 的 `Effects` 還原後沒有 active、active 序號在 ring 淘汰後仍保留音效 key（退役後才釋放）。

## 怎麼驗證

**本機規定（2026-11-09）不得執行編譯出來的程式，所以本機只做編譯與靜態檢查，`cargo test` 交給 PR 的 GitHub Actions CI。**

工作目錄 `LiTo381-Modern/`：

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
# 結束碼 0；Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 06s
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --features teleport-wire-test --test native_teleport_failure
# 結束碼 0；Finished `dev` profile [unoptimized + debuginfo] target(s) in 32.50s
```

- 兩次 check 的警告都只出現在既有檔案；本張改動的檔案沒有新警告。
- javac（JDK 17.0.17，UTF-8）以唯讀 `formal-server-E1498871.jar`（實算 SHA256 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`）編譯 `NativePeerHarness.java`、`NativeTeleportFailureHarness.java`：結束碼 **0**，產出 3 個 class（輸出在 `scratch/javac170/`，不入 git）。沒有啟動 Java。
- runner 以 PowerShell AST parser 解析：0 個語法錯誤。這只是語法檢查，不是 runner 實跑（runner 需要 PowerShell 7）。

## 已驗證

- 上面的 `cargo check`（all-targets，以及開 feature 的 integration 測試檔）結束碼 0、javac 結束碼 0、runner 語法解析 0 錯。

## 推測

- 新 wire 案例的預期行為是讀 `native_login.rs` 推出來的：F4 在 `BattlePending` 以外一律 `unsolicited teleport endpoint`；S03 的長度與正負號在 `native_source_identity::decode` 檢查；`fail()`／`cancel()` 對關聯的保留與取用規則。實際結果要看 CI。
- refused 案例在 Windows loopback 上可能約 1～2 秒才回報拒絕，所以給 10 秒上限（`pump_for`）。

## 待驗證／未驗證什麼

- **CI 的 `cargo test --lib` 結果（含新的 4 個 wire 案例與 6 個 History 單元測試）**：push 後看 PR 的 CI。本報告不宣稱任何測試通過。
- `--features teleport-wire-test --test native_teleport_failure -- recovery_`（CI 沒跑這個 feature）：交給本機測試執行者。
- 3 個正式 ignored scenario（cancel-repeat、refused-cancel-repeat、cancel-expiry-source）的 E149 owned runtime、Java fixture runtime、runner 全流程都沒跑；ignored／filtered 都不算通過。
- 四 profile History 只有純狀態測試，沒有用原資源實際繪製或播放音效來驗證，GUI 也沒驗。
- 沒有連正式服務或 DB，沒有啟停遊戲程序或 GUI。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skill_effects.rs`、`native_world_scene.rs`、`native_world_view.rs`、新增 `native_finite_history_tests.rs`
- `LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs`、`native_teleport_registry.rs`
- `LiTo381-Modern/tests/native_teleport_failure.rs`、`LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java`
- `維護工具/run-teleport-failure.ps1`
- 報告：`docs/reports/issue-170.md`；本機日誌 `scratch/check1.log`、`scratch/check2.log`（不入 git）

## 給下一棒的提示

1. Claude 審查重點：`for_source(false)` 的語意從「不還原四 profile」改成「還原 admission、清 graph」；`root_changed` 時立即退役；同代晚到 worker 的 `merge_admission`。
2. 本機測試執行者：
   - `cargo test ... --features teleport-wire-test --test native_teleport_failure -- recovery_ --test-threads=1`
   - `cargo test ... --lib -- modern_lito::native_skill_effects::finite_history_tests modern_lito::native_login::teleport_wire_tests --test-threads=1`
   - 正式入口：`run-teleport-failure.ps1 -Scenario cancel-repeat`、`-Scenario refused-cancel-repeat`、`-Scenario cancel-expiry-source`（測試名稱已改回舊名）
3. 第 3／4 張：TeleportMe 原圖示、無 S_PORTAL 的解鎖、T 鍵／cancel-late、物品拒絕矩陣都沒動。若要補 item observer，應更新 registry，不要借用 memory fixture 的資格。
