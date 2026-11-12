# 本機測試套件清單

在 repo 根目錄執行：

```powershell
pwsh -NoProfile -File 維護工具/run-local-test-suite.ps1
```

預設依序執行 T012 基線、技能、G06，任一失敗都回傳 1。總表與逐步 log 存在本 worktree 的 `LiTo381-Modern/target/current/verification/local-test-suite/`。技能與 G06 每項必須只有一個完整 Rust harness，`running 1 test`、`1 passed`、`failed/ignored/measured=0`、程序結束碼 0；0 tests、結果缺失或解析失敗均不通過。

可選參數：

| 參數 | 範圍 |
|---|---|
| `-Quick` | 保留 T012 的 check 與 Debug lib 兩步；不是 M2 完整驗收 |
| `-Category Baseline` | 原 T012 完整 80 步 |
| `-Category Skills` | 只執行技能表列名的精確測試 |
| `-Category G06` | 只執行八個 G06 scenario，含 no-run 建置與 owned runner |
| `-ReferenceJar <絕對路徑>` | 本 worktree `target/current` 內的 E149 JAR；預設 `target/current/server-reference/formal-server-E1498871.jar`（相對 Cargo 專案） |
| `-PowerShellPath <pwsh.exe>` | 從 Windows PowerShell 5.1 呼叫 G06 時，可指定 PowerShell 7+；否則從 PATH 尋找 |

G06 使用既有 `run-teleport-failure.ps1` 的 JDK、memory 模式、loopback listener、來源綁定、原子 lease、程序身分與清理守門。JAR 必須先由本機執行者驗證為 E149（SHA256 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`），再準備於本 worktree 的上述路徑；套件不掃描或修改外部伺服器目錄。測試 EXE 從本次 `cargo test --no-run --message-format=json` 的單一 artifact 取得，不以目錄中「最新 EXE」猜測。Cargo／pwsh 子程序清除繼承的 `LITO_*`，TEMP／TMP 使用本 worktree 的 `target/current/scratch`。

G06 必跑順序與契約 `native_E149_recovery_contract.py` 一致：`timeout`、`refused`、`wrong-service`、`cancel-retry`、`cancel-expiry`、`cancel-repeat`、`refused-cancel-repeat`、`cancel-expiry-source`。後三案由 T023 第 2 張（#170，已合併）補回，現行 runner 與 `native_teleport_failure.rs` 已有全部八案。套件先核對 owned runner 是否支援全部八案（以 UTF-8 讀取 runner 原始碼後解析 `ValidateSet`，PowerShell 5.1 也可讀）；缺案時八列都顯示 `FAIL: precondition: incomplete-owned-scenarios (T023-2)` 並列出缺項，結束碼為 1，不建置或啟動 G06 子程序。支援齊全後才建置，並以測試 EXE 的 `--list --ignored --format terse` 檢查八個精確測試名稱各出現一次；列舉失敗、缺項或重複都不啟動 Java。此列舉不是測試通過證據。開始實跑後若任一案失敗，後續案均列為未執行的失敗，不重試。前置條件缺失同樣會列出八個失敗項；只有八案全部實跑通過才可稱 G06 通過。

## 技能精確對照

`lib.txt` 與 `tests.tsv` 保留 T012 原白名單；`skills.tsv` 使用相同四欄 TSV：target、完整 Rust 測試名稱、ignored 註記（yes/no）、features（無則 `-`）。保留 target `__lib` 表示 `cargo test --lib`。執行時一律 `--include-ignored --exact`，不會因 ignored 而誤報通過。

`skills.tsv` 對應 T026 `docs/SKILLS-STATUS.md` 的「送包測試」欄。#173 第 4 次（合併 main 後）基準有 181 技能列、36 個去重參照，精確展開為 66 項（63 整合、3 lib）；58 列有測試、123 列「尚無」。原 41 項之外新增：

- 整檔參照 `native_t027_batch1`（13 案，#145）與 `native_t027_batch2`（11 案，#146）：清單必須列出該檔所有 `#[test]` 函式，腳本以靜態原始碼掃描（不編譯、不執行）核對，缺一個、多一個、「（N 案」與原始碼數量不符、檔內出現 `mod`／`include!` 或無法解析的 `#[test]`，技能類都失敗。其中 3 項是 `#[ignore]` 的唯讀原資源記憶體繪製測試（`ignored=yes`），以 `--include-ignored --exact` 照跑，需要 `D:\lineage開發\0-客戶端`。
- 限定參照 `native_t021::wire_shapes_and_menu_keep_selectors_separate_from_inventory_objects`：以 target 加名稱對照。

原 41 項的細節：

- `formal_attack_spell_layout_and_request_guards` 對應 attack／fire arrow／ice arrow／wind cutter 四個 target。
- attack skills 中三個同名函式分別存在 `disintegrate_request_contract` 與 `call_lightning_request_contract`，各自執行。
- `unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation` 經 `include!` 引入三個 `availability_tests` 模組，各自執行。
- 其餘測試使用清單中的精確 target 與函式名；共享多個技能的同一測試只列一次。與基線重複的測試仍在技能類獨立回報，不能累加成不同案例。

已檢查這 41 個選定入口及所用 fixture helper：使用記憶體狀態、嵌入 fixture、選擇器／封包／時鐘斷言；fire arrow 的入口另呼叫 `class4_zero_distance_zero_damage_and_miss_keep_authority`，會從 `D:\lineage開發\0-客戶端` 唯讀載入原資源，在記憶體 canvas 驗證投射物與事件。這些入口沒有外連、DB、GUI 或遊戲程序操作。`native_world_entry` 同檔的真實服務測試及其他未列入的資源／GUI 測試不會被選到。

每次執行均核對 181 列、唯一 ID、非空測試名稱、清單重複、未對照名稱及過期清單項。狀態表新增測試但清單未更新時，技能類會失敗，不能靜默漏跑；更新清單前須核對精確名稱與副作用。同一短名稱若新增 target 或 include 模組，也須同步補上精確對照。目前 123 列「尚無」測試只回報未驗證，不代表那些技能已通過；本腳本通過不會自動更新狀態表，也不代表技能的資源／畫面／伺服器同源性已驗證。

腳本自身回歸（合成輸出不計入 Rust 通過數）：

```powershell
pwsh -NoProfile -File 維護工具/test-local-test-suite.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/test-local-test-suite.ps1
```
