# Issue #193 — T061-2 目標選取政策

工作分支 `task/193`（PR #257）。第 1 輪（codex-local，commit `b5bf27e`，基底 `683b603`）完成客戶端送出前目標政策、26／42／68 自施分派與技能 68 選取生命週期；審查以「N1 驗收未完成」退回。第 2 輪（claude-local，本輪）合併 `origin/main`（至 `120c7be`）、解衝突，並補上 main 新增的「延後送出」路徑上的 68 目標退役防護。

**結論：本張的客戶端範圍已完成；#120 N1 的伺服器端（扣料、安裝、延後輸出 receiver 世代）不在本 repo 能完成的範圍，列在「未完成」，不標完成。**

## 做了什麼

### 第 1 輪（沿用，已審查抽查無異常）
- 新增 `native_target_policy`：送出前檢查請求的目標物件，依目前房間名冊（`S_CD` 名稱→force）判斷敵我。mode 1／3／5 對敵方玩家要「前景視窗＋按住 Ctrl」才送出；自身、友方、名冊外物件（NPC、召喚物）不需 Ctrl。
- 26／42／68 自施一律用自身 object ID 走 7-byte 物件格式 `14 row column id32-le`（26=`3,1`、42=`5,1`、68=`8,3`），不需 Ctrl；68 reuse 1500 ms。
- Ctrl 不能繞過：旁觀（`state.observer`、`world.observer_mode`）、自身死亡／HP 0、目標不存在／死亡／超出距離／隱形、血盟限制、撤退中、移動鎖；任何一項失敗都不改狀態、不建 pending、不入佇列。
- `cast_selected_with_input`：以 `Target` token（incarnation、target_revision、world generation、屍體動作）重新解析，目標退役或被替換時拒絕，不會改綁到同 ID 的新物件，也不會退回自施。
- 68 送出後保留目標 token；`poll` 時目標失效即回 `Interrupted`，清掉 pending，不動背包與 MP（材料與 MP 以伺服器為準）。

### 第 2 輪（本輪新增）
- **合併 `origin/main`**：唯一衝突在 `native_login.rs` 的 `poll` 技能 pending 區塊。保留兩邊條件：外層仍先要求 `skill_target_pending` 能解析（否則 `Interrupted`），內層加入 main 的 `player_callbacks.pending()`（延後送出期間維持 `Pending`）。失效時沿用 main 的 `player_callbacks.cancel_pending()`，延後的 body 一併丟棄。
- **延後送出路徑的 68 目標生命週期**（對應 N1「延後輸出時 receiver 已換世代要丟棄」的客戶端那一半）：main 新增 `send_player_action` → `Control::defer`，走路中的動作會延到 frame-zero suffix 才送出。原本只靠 `InputStamp`（世代、自身 id／incarnation／dead）清掉延後 body，但目標的 `target_revision`、隱形、目標本身的 incarnation 不在 stamp 內。新增 `LoginController::discard_stale_deferred_target()`，在 `run_player_callbacks` 進入 host 前檢查：68 的目標 token 已失效就取消延後 body，不會送到 transport。
- 新增 `Control::defer_for_test`（僅 `#[cfg(test)]`）讓離線 fixture 能放入延後 body。
- 新增測試 `target_policy_68_deferred_body_is_dropped_when_receiver_retires_before_frame_zero`：對「他人目標」與「自施」各 6 種情形（target_revision、incarnation、死亡、隱形／替換、世代變更、未變更對照組）檢查：失效時延後 body 被丟棄、transport 沒有排入、`poll` 回 `Interrupted` 並清掉 pending、背包與 MP 不變；未變更時 body 保留。

## 怎麼驗證

### 本輪（2026-11-12，claude-local）
依使用者 2026-11-09 規定，本機**不執行**編譯出來的程式（含 `cargo test` 的測試執行檔），測試交 GitHub Actions CI（`.github/workflows/build.yml`：`cargo check --locked --all-targets`、`cargo test --locked --lib --no-fail-fast`）。新測試在 `native_login.rs` 的 `#[cfg(test)]` 模組內，屬 lib 測試，CI 會跑到。

- `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`：合併後與修改後各跑一次，結束碼皆 `0`（`Finished dev profile ... in 52.32s`）。觸及的三個檔案沒有新增 warning。
- `cargo test`：**本機未執行**，以 PR #257 最新 commit 的 CI 結果為準（見 PR checks）。

### 第 1 輪（2026-11-06，codex-local，當時規定允許本機測試；僅供參考）
- `cargo test ... --lib --no-fail-fast` 結束碼 `0`：`test result: ok. 1982 passed; 0 failed; 577 ignored; 0 measured; 0 filtered out; finished in 9.63s`
- 回歸 4 組（`native_skills`、`native_skill_selection`、`native_attack_skills`、`native_t027_batch2`）結束碼皆 `0`，共 17 案通過。
- 這些數字是合併 main 之前的結果，不能代表本輪程式碼。

### 伺服器 N1 靜態閱讀（唯讀，未執行任何伺服器程式）
在 `D:\lineage開發\3-lito開發\lito伺服器\LiTo381-Server`：
- `tools/resource-database/build/de-native-pair-material-development/build-current.json`：狀態 `passed-pair-material-build-native-qualification-open`，產物是 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`（sha256 `175022207c3c…`）——**未完成資格驗證、未部署**。
- 同目錄 `builds/20261020T080606934231Z/inputs/build-native-pair-material.py`：
  - 第 35 行 `PairOutput.run`：延後輸出時檢查 `pair.valid() && binding.matches(receiver) && allowPublication && !DISCONNECTED` 才送出（receiver 換世代即丟棄）。
  - 第 43–47 行：`NativeCastMaterial` 在檢查材料、跳過 GM／零材料、`NativeDeItemApplication.replace` 安裝前都先 `validTargets(pc)`；目標退役時丟 `IOException("material pair target retired before install")`。
- `inputs/NativePairCastAction.java`：`Pair.valid()` 比對施法者與目標的 `Binding`、同一 hub、application 未關閉、雙方未斷線；`supported(skill)` 只有 68。
- 正式 `src/` 內**沒有** `NativePairCastAction*.java`，表示這組防護尚未併入正式伺服器原始碼。

## 已驗證
- 合併 main 後與本輪修改後，`cargo check --all-targets` 結束碼 `0`。
- 程式碼閱讀：客戶端所有技能／攻擊送出點（`cast_with_input`、`attack_with_input`）都經過 `native_target_policy::validate`；68 的目標 token 在「送出前」「延後送出前」「回覆等待中」三處都會重新解析。

## 推測
- 新測試與第 1 輪測試在合併後仍會通過（本機未執行，待 CI）。
- 伺服器候選 JAR 的 N1 邏輯（上述第 35、43–47 行）能做到「扣料／安裝前目標退役就拒絕」「延後輸出 receiver 換世代就丟棄」「不扣錯世代材料」——只是靜態閱讀，未執行任何 harness。
- `S_CD` 名稱與 actor 名稱一致即視為同一玩家；名稱不一致或重名時拒絕，尚未遇到跨伺服器例外。

## 待驗證／未完成
1. **CI**：PR #257 最新 commit 的 `cargo check` 與 `cargo test --lib`（含新測試 `target_policy_68_deferred_body_is_dropped_when_receiver_retires_before_frame_zero`）。
2. **#120 N1 伺服器端（本 repo 無法完成）**：扣料或安裝前目標退役、延後輸出時 receiver 換世代、不扣錯世代材料——權威在伺服器。目前只有 lito-launcher-server 的**未資格驗證候選**，正式 `src/` 未併入。需要在 lito-launcher-server repo 開任務：完成 `de-native-pair-material-development` 的 qualification、併入正式原始碼，並跑伺服器 harness（交本機 Codex，依規定本機 Claude 不執行）。建議主控把 T061-2 的驗收範圍明確限縮為「客戶端那一半」，伺服器那一半另開 Issue。
3. LOS、免疫、血盟權限的最終判定在伺服器；客戶端只擋已知的狀態（隱形、距離、血盟欄位），沒有 LOS 資料。
4. 實機：前景視窗＋實體 Ctrl、Pet Race 中的行為、GUI 畫面；未連正式服務或 DB、未執行 Release／ignored 測試。

## 產出路徑
- `LiTo381-Modern/src/modern_lito/native_target_policy.rs`（第 1 輪新增）
- `LiTo381-Modern/src/modern_lito/native_target_policy_tests.rs`（第 1 輪新增；本輪加 1 個測試）
- `LiTo381-Modern/src/modern_lito/native_login_callbacks.rs`（本輪：`discard_stale_deferred_target`）
- `LiTo381-Modern/src/modern_lito/native_player_callbacks.rs`（本輪：`defer_for_test`，僅測試）
- `LiTo381-Modern/src/modern_lito/native_login.rs`（解合併衝突）
- 第 1 輪其他接線：`mod.rs`、`native_forced_combat.rs`、`native_login_window.rs`、`native_skill_selection.rs`、`native_skills.rs`
- 本報告 `docs/reports/issue-193.md`

## 給下一棒的提示
- 審查者：先看 PR #257 CI；本張只能交付客戶端的 N1 那一半。若要求伺服器證據，請主控在 lito-launcher-server 開 N1 任務（qualification＋併入正式 `src/`＋harness），不要再退回本張。
- 之後若再加新的技能送出入口，必須經過 `native_target_policy::validate`，68 的目標要用 `native_forced_combat::object_target` 取出並存成 `skill_target_pending`。
- main 的延後送出（`Control::defer`）只靠 `InputStamp` 清除；其他「綁目標」的延後動作若日後出現，比照 `discard_stale_deferred_target` 處理。
