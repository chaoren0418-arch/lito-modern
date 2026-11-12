# T036-9（#314）Java peer／TLS 靜態盤點工具

只讀原始碼的靜態分析；**不建置、不執行任何測試或 Java 程式**。產出就是 `docs/reports/issue-314/` 下的兩份 TSV。
來源（Rust 或伺服器 Java）有變更時要重跑，不要手改 TSV。

## 檔案

| 檔案 | 作用 |
|---|---|
| `rows.py` | 解析 `docs/reports/issue-54.md` 的 1132 列分類與兩份白名單 |
| `analyze.py` | 逐測試做呼叫閉包（同檔、`include!`、`#[path] mod`、`super::`、限定路徑），記錄 `Command::new`、環境變數、peer class、字串常值等標記；輸出 `scratch/issue-314/analysis.json` |
| `summ_lib.py` | 載入 `analysis.json`，把 #54 理由文字對應到本張的四個桶（J125／T38／S3／X3） |
| `inventory.py` | 產出 `java-peer-tls-inventory.tsv`（每個測試一列：批次、peer class、SQL 模式、需要的環境）並印統計與「#54 之後新增的 ignored 測試」 |
| `java_direct.py` | 產出 `java-peer-classes.tsv`（每個 peer class 自身檔案的 listen／tmpdir／MySQL fixture／getenv 標記） |

## 重跑

在 repo 根目錄，用 `python -I`（3.12）：

```powershell
# 1. 取得伺服器 Java 測試來源（唯讀）。建議用 origin/main 的解出副本：
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具\build-java-test-peers.ps1 -ServerRef origin/main -Label <新標籤>
$src = 'LiTo381-Modern\target\java-peer\<新標籤>\server-src\src\test\java\local\lito381\server'

# 2. 分析並產出
python -I 維護工具\t036-java-peer-inventory\analyze.py
python -I 維護工具\t036-java-peer-inventory\inventory.py   docs\reports\issue-314 $src
python -I 維護工具\t036-java-peer-inventory\java_direct.py docs\reports\issue-314 $src
```

## 已知限制

- 呼叫解析以名稱為準，刻意偏寬：同一個 `mod tests` 內不同 inline module 的同名函式會互相命中，所以某些測試會被多標一項環境需求。反方向（漏標 Java／SQL 依賴）靜態上不能排除，執行時靠 fail-closed 的環境設定擋住（見 `docs/reports/issue-314.md`）。
- `sql_modes` 是「閉包內出現過、且在 Rust `JavaPeer::start_mode_lifetime` 的 process-job 清單裡的字串」；以參數選模式的 helper 會把所有候選模式都列出來。
- `java_direct.py` 只看 peer class 自己的 `.java`，不追 Java 端的跨類別呼叫；`SREPO` 與 2026-10-03 快照路徑是本機絕對路徑常數。
- macro 產生的測試（`rotation!` 等 16 個）以 macro 呼叫列加 `macro_rules!` 本體代替函式本體。
