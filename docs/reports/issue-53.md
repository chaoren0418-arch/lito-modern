# Issue #53：T010-A 本機單元測試基線

## 做了什麼

- 核對 Issue #53 為 `open`、`runner:dot`、`task:running`；執行前只有調度器派工留言，未見既有結果，且本機沒有 #53 分支或報告。
- 讀取專案規則、T010 與測試規劃。Issue #53 指定與 2026-11-04 CI 統計比較，沒有指定來源 commit。2026-11-05 透過 GitHub compare API 確認本機已有的 `c6f7b12b2421e6c25b53d885238dc34575ce1140` 與遠端 `main` 完全相同，因此固定該 commit 執行。
- 原工作目錄的 `main` 為初始化提交 `92bbda0`，追蹤檔案乾淨，但有未追蹤且被忽略的歷史證據檔。`git checkout --no-overwrite-ignore` 以 `The following untracked working tree files would be overwritten by checkout` 拒絕切換，未覆寫原資料。
- 改在專案內建立獨立 worktree，分支遵循 `AGENTS.md` 的 `feat/<任務編號>-<短名>` 規則：`feat/53-local-test-baseline`。未在原目錄強制 checkout、reset 或 clean。
- 依序安排 `cargo check --all-targets`、Debug 與 Release 的非 ignored `cargo test --lib`，每步只執行一次；首個非零結束碼即停止後續步驟。

## 執行環境與路徑差異

| 項目 | 實際值 |
|---|---|
| 作業系統／目標 | 本機 Windows／`x86_64-pc-windows-msvc` |
| 來源 SHA | `c6f7b12b2421e6c25b53d885238dc34575ce1140` |
| 原指定工作目錄 | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern` |
| 實際 Cargo 工作目錄 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-53\LiTo381-Modern` |
| `cargo --version` | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| `rustc --version` | `rustc 1.95.0 (59807616e 2026-04-14)` |
| `CARGO_HOME` | `D:\dev-cache\cargo` |
| `RUSTUP_HOME` | `D:\dev-cache\rustup` |
| 執行權限 | 此本機對話為 `workspace-write`／`auto_review`，不是已驗證的完全存取 |
| 建置目錄 | 實際工作目錄下 `target/current`，執行前不存在 |
| 暫存目錄 | worktree 的 `scratch/issue-53/tmp`；僅對執行子程序設定 `TEMP`／`TMP` |

原目錄未追蹤的 `LiTo381-Modern/.cargo/config.toml` 未複製進 worktree；其中本機 `dev/test` 的 `debug=1`、`incremental=false` 並非本次設定。本次採固定 commit 的 Cargo 設定與指定命令列參數，因此這份紀錄不能宣稱已驗證原工作目錄的所有本機設定與絕對路徑。

優先嘗試了本機 Codex CLI `0.166.0-alpha.5.1`，呼叫參數為 `--approve-for-me --ephemeral`，但其原始啟動標頭實際為 `approval: on-request`、`sandbox: read-only`，因此不能宣稱該 CLI 已符合自動核准或完全存取。CLI 在模型連線階段出現原文 `嘗試存取通訊端被拒絕，因為存取權限不足。 (os error 10013)`，未開始執行測試。已中止該次 CLI（結束碼 1），沒有重新啟動或調低安全限制；CLI 內建連線重試不代表 Cargo 重試。後續由目前本機 Codex 對話直接執行離線 Cargo。最初的本機 shell 唯讀探針成功，未遇到實際的 `setup refresh had errors`。

另外，普通權限的 `git fetch origin main` 因 `Failed to connect to github.com:443` 失敗；來源確認改用已連線的 GitHub connector，沒有更改網路或安全設定。

## 指令與結果

共同參數為 `--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`；三步皆不帶 `--ignored`，不執行整合測試程式或原生 GUI。

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --release --no-fail-fast
```

### 已驗證

三步皆完成，結束碼皆為 0。命令耗時包括編譯；check 沒有執行測試，其測試數為 N/A。

| 步驟 | 結束碼 | 命令耗時（秒） | 總測試數 | passed | failed | ignored | 測試執行耗時（秒） |
|---|---:|---:|---:|---:|---:|---:|---:|
| check `--all-targets` | 0 | 234.371 | N/A | N/A | N/A | N/A | N/A |
| test `--lib` Debug | 0 | 330.816 | 2464 | 1889 | 0 | 575 | 3.82 |
| test `--lib --release` | 0 | 661.651 | 2464 | 1889 | 0 | 575 | 1.97 |

兩次測試的 `measured`、`filtered out` 均為 0。575 個 ignored 在各組皆未執行，不能算通過；Debug 與 Release 也不合併成唯一測試數。沒有失敗測試名稱或 Cargo 錯誤，編譯警告保留在原始日誌中。

| 步驟 | UTC 開始 | UTC 結束 |
|---|---|---|
| check | 2026-11-05T03:51:14.1348721Z | 2026-11-05T03:55:08.5501473Z |
| Debug | 2026-11-05T03:55:08.5848113Z | 2026-11-05T04:00:39.4022549Z |
| Release | 2026-11-05T04:00:39.4213359Z | 2026-11-05T04:11:41.0729599Z |

### 與 CI 基線比較

| 來源 | 總數 | passed | failed | ignored |
|---|---:|---:|---:|---:|
| Issue 指定的 2026-11-04 CI 基線 | 2463 | 1888 | 0 | 575 |
| 本機 Debug | 2464（+1） | 1889（+1） | 0（無差異） | 575（無差異） |
| 本機 Release | 2464（+1） | 1889（+1） | 0（無差異） | 575（無差異） |

已驗證的來源差異：記錄該 CI 數字的 `00ca14f` 之後，`ee1b196fdf6a854dffeabaf1e3c62fa3311d792e` 新增 `modern_lito::native_opcodes::tests::opcode_constants_match_protocol_contract_sections_2_to_5`。`git diff --unified=0 00ca14f HEAD -- LiTo381-Modern/src` 顯示恰好新增一個 `#[test]`、沒有刪除 `#[test]`；該測試在本次兩個 profile 都通過，與總數及通過數各增加 1 一致。Issue 沒有提供 CI run 的精確 SHA，本次沒有重新執行歷史 CI。

### 推測

沒有以推測補足任何測試結果；後續功能、ignored 測試及原指定目錄的有效性均保留為待驗證。

## 證據位置

以下相對路徑以 worktree 根目錄為準，原始輸出與一次性腳本不進 git：

- `scratch/issue-53/run-tests.ps1`：單次執行包裝，首個非零結束碼即停止。
- `scratch/issue-53/summary.csv`：每步命令、UTC 起訖、耗時、結束碼、測試統計。
- `scratch/issue-53/01-check.log`：完整 check stdout/stderr。
- `scratch/issue-53/02-test-debug.log`：完整 Debug stdout/stderr。
- `scratch/issue-53/03-test-release.log`：完整 Release stdout/stderr。
- 原 repo 的 `scratch/issue-53-cli/session.log`：CLI 連線錯誤原始輸出；不含測試結果。

原始 Cargo 輸出的 SHA256：

| 日誌 | SHA256 |
|---|---|
| `01-check.log` | `52AA1241BB8A65D13CA6F8C2B9C09BE6ECF0609F865896E8F59C5C70FC76AEBC` |
| `02-test-debug.log` | `2651AC184E38CF317B2F4DDA98778E5F60E978EED2F5553721E3D1690896D5F5` |
| `03-test-release.log` | `55F258AF048FAEDAFCC1E9250A8CAEA5C2812E5E6F6460D5BA4BB3093C31D218` |

## 待驗證與給下一棒的提示

- 本次三步已在上述 worktree 完成；交由 Claude 審查報告、固定版本及路徑差異，再決定接續 T010-S 等門檻。不自行修改任務狀態或宣稱整個 M1 通過。
- ignored 測試、整合測試執行、12 gate、inventory、G06、GUI、T005 的搬移建置驗收均不屬本次驗證。
- 沒有修改 `src`、`tests` 或其他既有文件；沒有操作正式 DB、正式服務、BIN、封存或既有遊戲程序；未處理 #57。
- 來源與原指定目錄不同，後續需保留本次固定 SHA 與路徑條件，不可把結果外推為原工作目錄及其本機設定皆已驗證。

## 產出

- `docs/reports/issue-53.md`（本報告）。
- 分支：`feat/53-local-test-baseline`。
