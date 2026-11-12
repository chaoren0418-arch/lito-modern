# Issue #194 — T062：8 角色分頁、選服流程與血盟名稱

範圍：T062 全部一張。工作分支 `task/194`，起點 `683b603`。只在指定 worktree 修改；原專案與相鄰伺服器僅唯讀查核。未修改 `docs/REPORT.md`、TASKS 狀態、機密設定或封存資料。

## 做了什麼

- 核實既有 `native_game::Directory` 已接受 0～8 筆角色、拒絕第 9 筆與重複名稱，且 `C_LITO_SELECT_CHAR` 使用伺服器提供的原始 MS950 名稱加 NUL。保留這條送包路徑。
- 原生登入視窗原先用單一下拉選單顯示全部角色；現在每頁最多 4 筆，顯示頁碼與範圍，支援上一頁／下一頁按鈕、PgUp／PgDn 與滾輪。滾輪累積不足 120 的高解析度輸入，頁首／頁尾不循環；按鍵自動重複不連續翻頁。
- 頁內索引經完整清單索引轉換才送出。分頁綁定連線 epoch 與 directory generation；新清單、重連、離開選角均重設。未收齊角色、公告未確認、選角等待回覆時不能送出；拒絕後保留當頁並顯示原因，允許重選。
- 保留現有選服流程：登入取得來源清單→以來源名稱送 C8→驗證 redirect 與選定來源端點相符→交接 GAME→完成初始目錄後再次以來源名稱確認→發布角色清單。新增完整加密 loopback 回歸及錯誤 redirect 反例。
- 血盟解析原本已有 17-byte 容器；將文字上限具名為 `CLAN_NAME_MAX_BYTES = 16`，不截斷、不更改線上格式，角色列同時顯示血盟。新增空字串、ASCII、中英混合、8 個中文字、超長、缺 NUL、MS950 不完整字元及後續欄位不偏移測試。
- 新增的 3 個整合測試已加入 `維護工具/test-whitelist/tests.tsv`，無 ignored、無功能旗標；4 個分頁單元測試由既有 `cargo test --lib` 自動納入。

## 已驗證

2026-11-06 本機 Windows x64 執行：下列 5 個命令結束碼均為 **0**，4 組測試共 **14 passed、0 failed、0 ignored**。分頁單元組過濾掉的 2551 案未執行，不算通過。編譯存在未使用項目與命名等既有警告，沒有編譯錯誤。

於 `LiTo381-Modern/` 執行的完整命令：

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_game::pages::tests
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_game
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_login
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_character_flow
```

| 命令／測試群 | 結束碼 | `run-local-test-suite.ps1` 門檻 | 本機紀錄 |
| --- | --- | --- | --- |
| check --all-targets | 0 | PASS | `scratch/194-check.log` |
| --lib native_game::pages::tests | 0 | PASS；4 passed | `scratch/194-pages.log` |
| --test native_game | 0 | PASS；5 passed | `scratch/194-game.log` |
| --test native_login | 0 | PASS；3 passed | `scratch/194-login.log` |
| --test native_character_flow | 0 | PASS；2 passed | `scratch/194-flow.log` |

`test result:` 原文（依上表順序，check 無測試摘要）：

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2551 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
```

- 暫存 runner 以 PowerShell AST 取出當版 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult` 函式，直接套用其結束碼、唯一摘要、running 數量、0 tests、failed 及解析失敗門檻；未另行放寬。runner 本身結束碼 0。
- 同樣用現有 `Read-Whitelist -Integration` 解析白名單，共 103 列且無重複；3 個新增項目均可解析。分頁組由既有 lib harness 收集。
- 加密 loopback 實際驗證：第二個來源名稱的 C8、LOGIN→GAME 的 epoch 切換、丟棄初始空目錄後重新確認來源、8 筆接收、第二頁第 4 筆送出「角色8」、結果 7 拒絕後同頁第 1 筆送出「角色5」、接受後即時 Alive、返回 C_CA/100→S_F5→來源 C8→1 筆新目錄、舊頁失效與所屬連線清理。錯誤 redirect 測試另確認沒有連到錯誤 listener。
- `git -c core.whitespace=cr-at-eol diff --check` 結束碼 0（沿用原有 CRLF 檔案）；未改 `docs/REPORT.md`。

## 推測

- Win32 訊息迴圈在下拉選單的預設按鍵／滾輪處理前轉成翻頁，預期與按鈕走相同分頁模型；本次未啟動 GUI，因此不宣稱視覺、焦點及真實滑鼠輸入已驗收。
- 合成 loopback 證明當版客戶端的控制器、加密傳輸及分頁映射契約；不能代替正式 JAR、實際來源帳號與 DB 的端到端驗收。

## 待驗證／未驗證什麼

- 遵守本張「不啟動遊戲」限制，未開啟任何客戶端視窗，也未啟停既有遊戲或 Java 程序、未連正式服務／DB。
- 待 M2 完成後排原生 GUI 驗收：選第二個來源，8 角色兩頁，各以 PgUp／PgDn、滾輪、按鈕往返；由第二頁選角色，核對進場名稱；返回選角後回到第一頁。另驗證 0／1／4／5／8 筆、公告與拒絕、斷線重連、縮放與下拉展開時焦點。
- 本次沿用現代原生登入視窗的下拉選單元件，未重製舊客戶端四格職業動畫／美術，也未將合成測試畫面或歷史相容 DLL 的驗收當成 Rust GUI 通過。
- 未執行完整 M2、本機完整套件、Release 或正式服務驗收；略過項目不算通過。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_character_pages.rs`：分頁模型與單元測試。
- `LiTo381-Modern/src/modern_lito/native_game.rs`：分頁模組掛接。
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`：原生 UI 操作、索引映射、血盟及拒絕訊息。
- `LiTo381-Modern/src/modern_lito/native_source_identity.rs`：血盟位元組上限常數。
- `LiTo381-Modern/tests/native_game.rs`：血盟邊界回歸。
- `LiTo381-Modern/tests/native_character_flow.rs`：自有 loopback 的選服／選角／返回與 redirect 反例。
- `維護工具/test-whitelist/tests.tsv`：將新整合測試納入完整套件。
- `docs/reports/issue-194.md`：本報告。
- 本機暫存：`scratch/verify-194.ps1`、`scratch/194-*.log`、`scratch/194-summary.txt`，不進 git；建置產物在 `LiTo381-Modern/target/current`。

## 給下一棒的提示

- 請 Claude 審查分頁生命週期、第二頁名稱送包、通知／拒絕處理與測試界線；調度器於合併後更新索引與 TASKS 狀態。
- 本機執行者可重跑下列命令；所有測試只用記憶體 fixture 或自行建立的 `127.0.0.1:0` 埠，不需正式服務、資源目錄或 DB。GUI 最終驗收仍由使用者依 M2 安排執行。
- 依據唯讀伺服器文件 `D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/docs/maintenance/` 下 `buff-schema-and-character-pages-20261018.zh-TW.md`（15–39 行）、`combat-closeout-current.zh-TW.md`（74 行）及 `release-verification-20261018.zh-TW.md`（16 行）查核；未讀取或修改其 config、backups 或封存目錄。歷史文件僅用於規格，不當成本次測試證據。
