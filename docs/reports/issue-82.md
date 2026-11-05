# Issue #82 / T011-H：當版 G06 owned harness 的最小開發範圍

## 裁定

**採用 memory 模式的 `維護工具/run-teleport-failure.ps1`，只做最小適配，不走 E149 自有 DB 鏈。** 這項工作是驗證基礎設施，不是新功能，所以留在 **M1**。PLAN M2 的 G06 開發（T023 等）不受影響。

理由（依 #80 盤點，加上雲端讀 `維護工具/run-teleport-failure.ps1` 全文 82 行）：

- **當版的 5 個 scenario 本來就是 memory 模式**。當版 `tests/oracle/NativeTeleportFailureHarness.java`（2 個參數：root、scenario）用記憶體帳號；#80 也確認「當版 memory harness 沒有自行啟動 DB」。所以**不需要自有 MySQL**，自有隔離 DB 的條件也不適用。
- **run-teleport-failure.ps1 的大部分機制已經符合需求**：
  - 每次在 `target/current/native-teleport-failure/<stamp>` 建立新的 root（第 11–12 行）；
  - 測試執行檔與 JAR 必須位於 `target/current`（第 5–6 行）；
  - 以當版的兩個 Java 來源對指定 JAR 執行 javac（第 44–45 行）；
  - 等待 ready、設定三個環境變數、執行單一測試並帶 `--exact --ignored`（第 47–56 行）；
  - 清理時比對 PID、啟動時間與命令列（第 16–23 行）；
  - 檢查 `result.json` 的 sessions 數、memberships 與 authority_mutations，並做 source／binary 漂移檢查（第 64–72 行）。
- **E149 runner（`run-native-E149-recovery-case.py`）不適合**：它綁定候選的 class 與 image，需要 v4 plan，還會改舊的工作佇列與進度檔（#80）。不採用。

## 最小開發範圍（T011-H1，見 `docs/plans/issue-82.json`）

| # | 缺口（#80 或雲端讀碼） | 修正 |
|---|---|---|
| 1 | 第 8 行的 JAR 白名單只有 BBC59308、9719F45D、6928A147、76EBE06D 等舊版，不接受 E149 | 加入 `E1498871…D99D498` |
| 2 | 第 4 行硬編 `$taskRoot`，隔離 worktree 不能用 | 改由 `$PSScriptRoot` 推得 |
| 3 | 第 13 行把輸出寫到 `驗證紀錄/current/…`（舊流程） | 改寫到 `target/current/verification/native-teleport-failure/<stamp>` |
| 4 | Rust 與 runner 都沒有驗證 ready 埠的所有權，也沒有拒絕 2000／3306（#80「當版 Rust 與 ready 契約」） | runner 用 `Get-NetTCPConnection` 確認 `port`、`destination_port` 的 listener 屬於 `$server.Id`（`refused` 則確認沒有 listener），並拒絕 2000／3306 與 1024 以下的埠；Rust 的 `run_owned_harness` 也拒絕 2000／3306 |
| 5 | 第 9、19 行用 `Get-CimInstance` 檢查與清理；本機曾發生 CIM 存取被拒（#58） | 改用持有中的 Process handle 比對 Id、StartTime 與映像路徑；改用 lease 檔偵測重複執行 |
| 6 | 第 37 行的 pin 清單包含可能不存在的 `.cargo/config.toml` | 不存在時記為 `absent`，不讓腳本失敗 |
| 7 | 沒有 dry-run | 新增 `-WhatIf`，在建立目錄、編譯或啟動之前返回 |
| 8 | 第 105 行的 `timeout` 只用 `assert_ne!(…, "cancel-expiry")`，沒有要求明確指定 scenario | 改成必須明確等於 `timeout` |

T011-H1 的驗收只有語法檢查、5 次 `-WhatIf` 與 `cargo check --tests`；**不執行任何 scenario**。實際執行交給 T011-B（#81）。

## T011-B（#81）的前置

- T011-H1 合併；
- 用 `cargo test … --test native_teleport_failure --no-run` 建出當版測試執行檔；
- 把正式 JAR 複製到 `target/current`，並核對 SHA256；
- 每個 scenario 執行一次 `pwsh -NoProfile -File 維護工具/run-teleport-failure.ps1 -TestBinary <exe> -ReferenceJar <jar> -Scenario <s>`。

H1 合併後，雲端主控會更新 #81 的本文，並把它改回 `task:queued`。

## 怎麼驗證

- 雲端閱讀 `維護工具/run-teleport-failure.ps1` 全文（main，82 行），以及 `tests/native_teleport_failure.rs` 的第 26、105–132 行，上表引用的行號都來自這次閱讀。
- `python3 -m json.tool docs/plans/issue-82.json > /dev/null` 結束碼 0。

## 未驗證什麼

- 當版 Java harness 能否對 E149 JAR 編譯成功、memory 模式下 E149 的資源能否正確初始化：交給 T011-H1 的 javac（`-WhatIf` 不編譯）與 T011-B 的實跑。
- `Get-NetTCPConnection` 在使用者電腦的權限下能不能用，由 T011-H1 在本機確認。
- 沒有執行任何測試或程序。

## 產出路徑

- `docs/plans/issue-82.json`
- `TASKS.md`（T011 改為 `blocked(T011-H1)`，並寫明路線）
- `docs/reports/issue-82.md`（本檔）
