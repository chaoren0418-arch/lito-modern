# Issue #55：T010-B 白名單 ignored 單元測試

## 結論

**已驗證：已審查白名單的唯一 ignored lib 測試單次實際執行並通過。** 結束碼 0，`running 1 test`，1 passed／0 failed／0 ignored，沒有零測試、重試、逾時或未實際執行的白名單項目。其餘 574 個 ignored lib 仍排除待驗證；本結果不代表完整 M1 或全部單元測試通過。

## 做了什麼／來源與授權邊界

- 開工核對 Issue #55 為 OPEN、`runner:dot`、`task:running`；只有調度派工留言，沒有既有 RESULT、task/55 本機／遠端分支或 worktree。查閱現有 Codex 工作階段，未發現另一個 #55 執行工作。
- 前置 #54 已 CLOSED、`task:done`、`review:pass`、`gate:pass`；PR #71 已合併。依已審查提交 `c69eca12825f975e12ed907fb2af7d9689d08172` 的 `docs/reports/issue-54.md` 白名單處理，沒有自行擴充。
- 固定執行來源 `04e0ad87f04ce10065e134a900bc05a6171d4426`。以 `git diff --exit-code bb96171d2938437dd224af73da466ab8bbab8f73 HEAD -- LiTo381-Modern` 核對整個 Cargo 專案與 #54 分類基準無差異。
- 專屬本機 Codex 工作階段直接使用 Windows PowerShell／git／Cargo，未啟動巢狀模型。隔離 worktree 為 `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-55`，分支依本次派工使用 `task/55`；原工作區維持原分支。
- 已讀原目錄與隔離 worktree 的 AGENTS.md、TASKS.md T010、Cargo.toml、build.rs。依本次直接指示，不讀寫 `docs/REPORT.md`、`config/server.properties`、正式 BIN 與封存內容；不連正式服務／DB，不操作既有遊戲程序，不做 GUI 驗收，不修改 src／tests。
- 本輪只新增本報告為追蹤變更；一次性腳本、白名單、log 與 CSV 留在隔離 worktree 的 scratch，不進 git。沒有更動 ACL、安全設定、safe.directory 或防火牆。

## 逐項安全核對

白名單只有以下一個 ignored lib 測試，已重新閱讀本體及其所有專案內呼叫：

| 測試全名 | 安全理由與來源 | 裁定 |
|---|---|---|
| `modern_lito::compositor::tests::measure_full_frame_clear_and_dib_conversion` | `compositor.rs:643–669`：800×600 記憶體 Canvas，clear 與轉換各 100 次，逐像素比較，stderr 計時。`Canvas::new`（:59）、`clear`（:130）、`rgba`（:85）、`copy_bgra`（:554）只配置／存取 Vec；`compositor_dib.rs:4` 的 copy、scalar 與 SSE2 分支只存取輸入輸出切片。 | 安全，允許單次 exact 執行 |

`src/lib.rs` 匯出 modern_lito，`src/modern_lito/mod.rs:73` 宣告 compositor，測試位於 `#[cfg(test)] mod tests`；完整名稱與 #54 實際列名一致。此項沒有資源檔、JAR、BIN、socket、DB、HWND、子程序或檔案寫入呼叫，無其他測試 helper 或 fixture 需求。CPU 指令偵測與計時不構成遊戲程序操作。

Cargo 使用標準 lib 測試 harness，沒有 `harness=false`；來源未見 ctor／link_section／no_main 自訂啟動掛勾。build.rs 僅透過 winres 編譯 Windows manifest。Cargo 會執行離線編譯所需工具與相依套件 build script，故不宣稱整個 Cargo 命令僅包含測試函式本身。

## 怎麼執行／實際權限

- 實際環境為 `workspace-write`／`auto_review`，**未設定或驗證完全存取**；工具未提供可切換及證明完全存取的介面。一般 gh 網路讀取遇到 socket access denied；GitHub 讀取先由連接器完成，git fetch 依標準 `require_escalated` 核准後成功。沒有繞過必要核准。
- `CARGO_HOME=D:\dev-cache\cargo`、`RUSTUP_HOME=D:\dev-cache\rustup`；產物限定本 worktree 的 `LiTo381-Modern/target/current`。
- 執行子程序設定 `LITO_SHARED_RESOURCE_ROOT=D:\lineage開發\0-客戶端` 與 `LITO_EFFECTLIST_REFERENCE=D:\lineage開發\客戶端xml\effectlist2.xml`；本項不讀取這兩個位置，因此沒有宣稱已驗證資源身分。TEMP／TMP 指向 `scratch/issue-55/tmp`，未複製其他工作區未追蹤的 Cargo 設定。
- 從合併後 #54 報告重建 `scratch/t010-whitelist-lib.txt`，UTF-8／LF，SHA256 為 `4D2A60D6D94D9AA222DDA6E3EEEE481284D4BB0D71351AF70F7F56CDC3031065`，與已審查白名單相同。執行腳本檢查名單恰好一行且全名一致；既有 log／CSV 會拒絕執行，以避免重試。

工作目錄為本 worktree 的 `LiTo381-Modern`，唯一測試命令：

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast -- --ignored --exact modern_lito::compositor::tests::measure_full_frame_clear_and_dib_conversion
```

## 已驗證結果與原始收據

| 白名單測試全名 | 結果 | passed | failed | ignored | 錯誤／未實際執行 |
|---|---|---:|---:|---:|---:|
| `modern_lito::compositor::tests::measure_full_frame_clear_and_dib_conversion` | 已驗證通過（單次） | 1 | 0 | 0 | 0 |

原始輸出末段（完整原始輸出保留於 scratch）：

```text
running 1 test
test modern_lito::compositor::tests::measure_full_frame_clear_and_dib_conversion ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2463 filtered out; finished in 1.89s
```

UTC 開始 `2026-11-05T05:48:40.3455325Z`，結束 `2026-11-05T05:54:36.9590502Z`，整條命令含首次編譯共 356.6135177 秒，Cargo 結束碼 0。harness 確認執行數為 1，passed + failed = 1，ignored = 0；2,463 filtered out 包含非白名單的 ignored 與一般 lib 測試，不是全部 ignored，也不計為通過。編譯有 99 個 warning（13 重複），未執行 cargo fix 或修改程式碼。標準 harness 捕捉了成功測試的 stderr，故本輪不提供 clear／conversion 分段時間，亦未為取得時間另行重跑。

| 收據（相對 worktree） | SHA256 |
|---|---|
| `scratch/issue-55/01-lib-exact.log` | `F803B213565217D9E84B1FE5FB7BB0F4B405DE297100284E443457CE1357D392` |
| `scratch/issue-55/execution-summary.csv` | `51EB65A9B44E983110DB4A6F8BE09F85ED5BD84279D4409B738D20F046F877E9` |
| `scratch/issue-55/run-exact.ps1` | `CBF32F77BDD29603EC9BFBBA8297DE67B3DE507D8570F92DF15058B9D9B9C1CF` |
| `scratch/t010-whitelist-lib.txt` | `4D2A60D6D94D9AA222DDA6E3EEEE481284D4BB0D71351AF70F7F56CDC3031065` |

沒有失敗項目，故無首失敗重現收據；上列完整命令及 log 可供審查重現。本輪只執行一次，未使用 `--list` 追加另一輪 Cargo 命令。

## 未執行項目／待驗證

依 #54 的 ignored lib 分類保留，不把它們視為失敗或通過：

| 排除分類 | 未執行數 | 原因 |
|---|---:|---|
| GUI | 5 | 原生／官方視窗或操作員驗收不在本輪範圍 |
| DB | 119 | SQL／DB 流程不在授權安全範圍 |
| 不明副作用 | 20 | helper、外部目標或輸出所有權未完成安全證明 |
| 缺前置條件 | 430 | fixture、pins、環境或權限條件未核實 |
| 合計 | 574 | 維持白名單外排除 |

上述逐項全名、分類理由與來源行號沿用已審查的 `docs/reports/issue-54.md`。557 個 integration 入口屬 T010-C 範圍，本輪全部未執行（安全 76、GUI 1、DB 1、不明副作用 438、缺前置條件 41），不混入 575 個 ignored lib 的統計。

未驗證 GUI、正式 DB／服務、資源 pins、JAR、BIN、完整 M1、G06、實際遊戲畫面或幀率。本項是有界 CPU 轉換測試，不能推論遊戲 FPS 達標；無推測的通過數。

## 產出路徑／給下一棒的提示

- 唯一追蹤產出：`docs/reports/issue-55.md`。
- 本機證據：`scratch/issue-55/run-exact.ps1`、`scratch/issue-55/01-lib-exact.log`、`scratch/issue-55/execution-summary.csv`、`scratch/t010-whitelist-lib.txt`；均相對上述專屬 worktree。
- 請 Claude／本機審查者核對來源、白名單 hash、唯一命令與原始輸出，再由調度器處理後續。T010-C 需另按整合白名單授權執行；574 個排除項目須先釐清前置／安全性，不得直接跑整包 ignored。此報告不解除完整 M1／G06 門檻。
