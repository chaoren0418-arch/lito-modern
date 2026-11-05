# Issue #111 — T011-F：TW 特效資源資格不符時不中止流程

**已驗證：三個特效以可觀測的不可用狀態處理資源載入失敗；新增 12 案全通過，完整 Debug lib 為 1901 passed／0 failed／575 ignored，最終 cargo check 與 lib test 結束碼均為 0。未重跑 G06。**

## 已驗證：本機資源盤點

- 日期：2026-11-06（Asia/Taipei）。工作目錄 `W = D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-111`，沿用 `task/111`，未 checkout／建立分支。
- 已讀 `AGENTS.md`、`docs/REPORT.md`、`TASKS.md` 的 T011 與 `docs/reports/issue-81.md`。原始專案、客戶端資源均未寫入；未讀改機密設定、封存或硬連結。
- 三個 `load_current_tw` 均呼叫 `SharedTwActionCatalog::load_selected(&root.join("launcher"), ...)`。實際 pack 路徑均為 **`D:\lineage開發\0-客戶端\launcher\TW13081901.pak`**，檔案存在，大小 **1,406,534 bytes**。
- T011-B 的 `tests/native_teleport_failure.rs:39` 直接以 `D:\lineage開發\0-客戶端` 呼叫 `Model::load`，與本輪盤點根目錄一致。
- `CURRENT_TEXT` 指 pack 解密／解壓、移除 `S` 前綴後的原始 text bytes，**不是另一個 TXT 檔案**；loader 不讀 loose TXT。text 的來源路徑即上述 pack，僅在記憶體計算，沒有另存／複製資源。
- `D:\lineage開發\0-客戶端\TW13081901.pak` 仍不存在；這是歷史 root namespace 的缺檔，當前三個 launcher loader 的失敗並非讀取此路徑造成。

先以本 worktree 未修改的來源建置 lib（`cargo build --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib`，結束碼 0），再於 `scratch/issue-111/audit.rs` 寫一次性 Rust 程式，連結該 lib，分別呼叫實際 `load_selected`。三次均成功完成解碼、內容綁定與 selected catalog parsing；盤點程式結束碼 0。另用 `Get-FileHash -Algorithm SHA256` 獨立核對 pack SHA，相符。盤點完成後才修改呈現層。

三個 loader 的 `CURRENT_PACK`／`CURRENT_TEXT` 與 decoded 條件完全相同，逐項比對如下：

| 條件 | 三者原始碼要求 | 三者實際讀得 | 判定 |
|---|---|---|---|
| pack SHA256 | `B3788DF4C7F077EF3FF15B070A5D020CCB224E48DEC3757C4CD34528FCA4B218` | `92B512A32349D0775E1036F1832E21F1141AE66E64821D6978F0879246E59CA7` | 不符 |
| 解碼 text SHA256 | `7EBE2B464C8B1EA9C58AC985DD8FF4C823784D8231894262EDDCD7691CA94008` | `28C4FC600833FB8C609AB195A67FF926E8FFA571100D8D4B389A23BE8C2C8DB8` | 不符 |
| decoded bytes | 5,974,928 | 5,990,880 | 不符（多 15,952 bytes） |

| loader／技能 | 選取 definitions IDs | 要求數量 | 實際數量 | 判定 |
|---|---|---:|---:|---|
| Disintegrate／77 | 11746、11747、11748、11749 | 4 | 4 | 符合 |
| cone cold／38 | 11738、11739、11740、11741 | 4 | 4 | 符合 |
| call lightning／34 | 11736 | 1 | 1 | 符合 |

因此三者現有 `require_current_profile` 都應拒絕此 pack；不能因 definitions 數量相同而放行。這些結果不證明檔案損毀，也未判定資源內容變更的來源。

## 修正與 diff 摘要

- `native_disintegrate.rs`、`native_cone_cold.rs`、`native_call_lightning.rs`：一致新增 `PresentationStatus::{Pending, Ready, Unavailable { reason }}`。既有 snapshot／場景 receipt 的 `presentation.state` 可讀出 `pending`／`ready`／`unavailable`；不可用時另有 `presentation.reason` 完整錯誤鏈，`assets_ready=false`。
- `prepare` 仍使用原本嚴格的 `load_current_tw`。資源 loader 回傳錯誤時，只標記本機呈現不可用並回傳 `Ok(())`；以 stderr 記錄一次原因與 generation。同來源、同世代後續 prepare 不再呼叫 loader，也不重複記錄。
- 不可用紀錄隨既有 `History` 保存／合併，依 canonical resource root 與 generation 還原，涵蓋同世代 owner／場景替換。新世代清除失敗狀態並按原本「已學技能或有對應投射物」條件重試；不同資源根不繼承失敗紀錄。已成功載入資源的跨世代沿用規則維持原樣。
- 取消載入（含 loader 執行期間取消）、舊 owner、同 owner 改用其他 root、時鐘與 cue budget 等生命週期防護保留；不會被轉成資源不可用。
- 不可用時沒有 assets／clips，不提交特效圖像、音效或特效時序回呼；沒有替代圖或空白圖。特效仍只借用 `&World`，未更動技能請求或伺服器權威狀態處理。
- 新增共用 `native_tw_presentation_tests.rs`，由三個呈現模組各自納入測試，注入各自資源型別的錯誤。未修改 `native_world_scene.rs`：原呼叫層會收到 `Ok`，原有 snapshot 已能帶出新狀態。
- **未修改**任何 `*_assets.rs`、資格檢查、SHA 常數、資源、runner、harness 或 `docs/REPORT.md`。

## 檢查與測試

建置環境沿用 `CARGO_HOME=D:\dev-cache\cargo`、`RUSTUP_HOME=D:\dev-cache\rustup`，僅對執行程序補入既有 `D:\dev-cache\cargo\bin`；`TEMP`／`TMP` 指向 `W\scratch\issue-111\tmp`。Cargo cwd 為 `W\LiTo381-Modern`，產物只放 `target/current`。

實際命令（Debug，未加 `--release` 或 `--ignored`）：

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib availability_tests
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib
```

新增測試的最終輸出原文，結束碼 **0**（`availability-tests.log`）：

```text
running 12 tests
test modern_lito::native_call_lightning::availability_tests::cancellation_during_loader_remains_an_error_and_retires_owner ... ok
test modern_lito::native_cone_cold::availability_tests::cancellation_during_loader_remains_an_error_and_retires_owner ... ok
test modern_lito::native_disintegrate::availability_tests::cancellation_during_loader_remains_an_error_and_retires_owner ... ok
test modern_lito::native_call_lightning::availability_tests::no_demand_stays_pending_and_unavailable_does_not_relax_source_guard ... ok
test modern_lito::native_disintegrate::availability_tests::no_demand_stays_pending_and_unavailable_does_not_relax_source_guard ... ok
test modern_lito::native_cone_cold::availability_tests::unavailable_history_survives_owner_and_scene_replacement_but_new_generation_retries ... ok
test modern_lito::native_call_lightning::availability_tests::unavailable_history_survives_owner_and_scene_replacement_but_new_generation_retries ... ok
test modern_lito::native_cone_cold::availability_tests::no_demand_stays_pending_and_unavailable_does_not_relax_source_guard ... ok
test modern_lito::native_disintegrate::availability_tests::unavailable_history_survives_owner_and_scene_replacement_but_new_generation_retries ... ok
test modern_lito::native_cone_cold::availability_tests::unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation ... ok
test modern_lito::native_disintegrate::availability_tests::unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation ... ok
test modern_lito::native_call_lightning::availability_tests::unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 2464 filtered out; finished in 0.01s
```

每個 loader 均驗證：已學技能與僅有投射物兩種觸發、注入資格錯誤後 `prepare`／世界呈現 `update` 回傳 `Ok`、原因完整且無圖像／音效／Impact、同世代只載入一次、原 `World` 不變、技能請求仍成功、伺服器 HP／MP 封包仍更新權威值。另涵蓋 History 合併／場景與 owner 替換不重試、新世代重試、取消與 source guard 保留、無需求不載入及不同 root 不繼承錯誤。

完整 lib 測試的關鍵輸出原文，結束碼 **0**（`cargo-test-lib.log`）：

```text
running 2476 tests
test modern_lito::disintegrate_assets::tests::current_tw_namespace_requires_exact_profile_and_cancels_before_io ... ok
test result: ok. 1901 passed; 0 failed; 575 ignored; 0 measured; 0 filtered out; finished in 4.77s
```

既有 `require_current_profile` 單元測試確實通過：正確 profile、pack／text 的第 0／15／31 位元組異動、decoded 長度與 definitions 數量反例、I/O 前取消皆保留；三個 assets 檔案的 diff 為空。**575 個 ignored 未執行，不計入通過。**

首次新增測試執行為結束碼 101、11 passed／1 failed（`availability-tests.initial.log`）：cone cold 的投射物測試誤用命中動畫 graphics 11740，未觸發 loader，狀態仍為 pending；載入觸發應為 11738。已修正測試常數，未改產品的 graphics 規則，重跑取得上述 12／12 與完整 lib 通過結果。初輪輸出未刪除。

`cargo check --all-targets` 首輪結束碼 0；針對最後測試常數修正的最終重查結束碼亦為 **0**（`cargo-check.log`），結尾原文：

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 01s
```

`cargo check` 不會輸出 `test result:`。本輪保留既有編譯 warnings，未擴大範圍修整。`git diff --check` 結束碼 0。

## 推測

修正可避免 #81 已定位的特效資格錯誤從 prepare 傳至連線／session，因而應能讓 G06 繼續走到後續傳送與恢復檢查。是否還有其他失敗原因，必須由重新排程的 T011-B 驗證。

## 待驗證／未驗證事項

- 未重跑任何 G06 scenario，未啟動遊戲或 GUI，未連正式服務或 DB。
- 未宣稱登入、傳送、戰鬥或 G06 全流程通過；本輪驗證限唯讀資源盤點、單元測試與編譯檢查。
- 不符現有資格的三組本機特效仍不呈現；未驗證或放寬新 pack 的呈現資格。
- 略過／ignored 測試不算通過；沒有執行 Release 或原生畫面驗收。

## 產出路徑與給下一棒的提示

- git 交付：三個 `native_*.rs` 呈現模組、`LiTo381-Modern/src/modern_lito/native_tw_presentation_tests.rs`、本報告 `docs/reports/issue-111.md`。
- 本機盤點程式與證據（不入 git）：`W\scratch\issue-111\audit.rs`、`audit-build.log`、`resource-audit.log`；盤點 exe 在 `W\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\issue-111-audit.exe`。
- 檢查與測試原始輸出：`W\scratch\issue-111\cargo-check.log`、`availability-tests.log`、`cargo-test-lib.log`，各有同名 `.exit.txt` 結束碼檔案；首輪檢查／新增測試另以 `.initial.log`／`.initial.exit.txt` 保留。請保留 worktree 供審查。
- **Claude 審查修正與報告；合併後由 Claude／調度器重新排 T011-B，交本機執行者執行。** 使用者負責合併與後續真機驗收。本任務不自行重跑 G06。
