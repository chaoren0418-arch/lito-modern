# Issue #314 — T036-9：Java peer／TLS 盤點（M3 9／15）

執行者：claude-local。分支 `task/314`。依據：lito-modern `main` @ `e33e62e`；lito-launcher-server 本機 working tree `HEAD` `6bb4a27b` 與 `origin/main` `edfb5c82`（2026-11-12）。
**本張沒有執行任何測試，也沒有啟動 Java。** 只做靜態閱讀、`javac` 編譯（編譯器，不執行產物）與本機路徑存在性檢查。下面所有「可以跑」的判斷都還不是「通過」。

## 結論

1. #54 裡屬於本張的是 **169 個 ignored lib 測試**（理由文字四類：Java peer／fixture 125、TLS fixtures 38、server root 3、外部 Java peer 3）。逐項做了呼叫閉包分析，分成 6 個批次：

   | 批次 | 數量 | 內容 | 需要 | 建議 |
   |---|---:|---|---|---|
   | A0 檔案 fixture | 3 | 讀 `<server root>/target/modern-{equipment,revival}-test-fixtures/*.request／.reply`，測試當下不啟動 Java | Java 產生的檔案 fixture | T036-10 |
   | A1 stdio peer | 42 | `java -cp … Modern*WirePeer`，只走 stdin／stdout，沒有 socket | JDK17＋classpath | T036-10 |
   | A2 TLS peer | 94 | `transport.rs` 的 `JavaPeer`：loopback、port 0、私有 CA | 再加 TLS fixtures | T036-10 |
   | B1 stdio＋共享資源 | 9 | 同 A1，另讀 `LITO_SHARED_RESOURCE_ROOT` | 再加共享資源根 | T036-11 |
   | B2 TLS＋共享資源 | 11 | TLS peer，Rust 或 Java 端讀共享資源根 | 同上 | T036-11 |
   | B3 TLS＋共享資源＋job 閘 | 3 | `canonical-equipment` 模式：Rust 把 Java 放進 kill-on-close job 並送 `GO`，但 Java 原始碼在此模式不開 MySQL | 再加 server root（含 `db/modern_match_settlement.sql`） | T036-11 |
   | D 自有隔離 DB | 7 | 模式在 Rust 的 process-job 清單內，Java 會啟動自有 mysqld（地圖輪替 4、寵物賽跑 3） | `LITO_MODERN_MYSQL_HOME` | **移到自有隔離 DB 那兩張，不在 T036-10／11 跑** |
   | 合計 | 169 | | | |

2. **目前本機一項都跑不了**，原因都在前置：伺服器 repo 沒有 `target/`（沒有已編譯的 peer、沒有 TLS fixtures、沒有檔案 fixture）；六個相關環境變數都沒設。
3. 伺服器 **本機 working tree（`6bb4a27b`）的 `src/main/java` 自己編不過**（`javac` 35 個錯誤，缺 `SharedDatabase` 等 6 個類別）。**`origin/main`（`edfb5c82`）用 `javac` 可以編過**：main＋compile-stubs 1179 檔 → 4449 個 class，test 1452 檔 → 2096 個 class，本張盤到的 64 個 peer class 全部產出。編譯腳本是 `維護工具/build-java-test-peers.ps1`，產物在本專案 `LiTo381-Modern/target/java-peer/`，不寫伺服器 repo。
4. 除了這 169 項，還有 **41 個被 #54 歸在別張的「缺前置條件」測試同樣要 Java peer**（共享資源根 39，其中 4 個可能還要 SQL；雜項環境變數 2）。那幾張如果只設共享資源根，這些會全部在 `expect("…JAVA…")` 失敗。清單見 TSV 的 `scope54` 欄。
5. 靜態上看不到會連正式服務、正式 DB 或既有遊戲程序的路徑：A／B 批用到的 36 個 peer class，自己的檔案裡 listen 都是 `127.0.0.1`／port 0 或沒有 listen。但 Java 端只查到「peer 自己的 `.java`」這一層，跨類別的行為沒有追完，所以執行時要用 fail-closed 的環境（見「執行環境」）。

## 做了什麼

- 讀 `CLAUDE.md`、`docs/REPORT.md`、`docs/REUSE-MAP.md`（§0：LS working tree 落後，要讀 `origin/main`；§0.3：LS 原始碼不等於 E149）、`docs/TESTING.md`、`docs/reports/issue-54.md`、`TASKS.md` 的 T036／T010／T011。
- 寫了靜態分析工具 `維護工具/t036-java-peer-inventory/`（5 個 Python 檔＋README），對 #54 的 1132 列逐一做呼叫閉包，記錄 `Command::new`、環境變數、peer class、模式字串、檔案寫入、HWND 標記。
- 寫了 `維護工具/build-java-test-peers.ps1`：只做 `javac`，把伺服器的 main＋test 原始碼編到本專案 `target/java-peer/<label>/`，輸出 `classpath.txt` 與 `build.json`（來源 commit、檔數、jar SHA256）。`-ServerRef origin/main` 用 `git archive` 把該 commit 的原始碼解到輸出目錄再編，不 checkout、不改伺服器 repo。
- 讀了伺服器端相關 Java：`ModernTlsTestMaterial`、`ModernTlsTestPeer`、`ModernBattleWirePeer`、`ModernCombatTestPeer`、`ModernSettlementMysqlFixture`、`ModernQuestSqlFixture`、`ModernPetRaceTestPeer`、`ModernNpcServiceTestPeer`、`ModernHasteTestPeer`、`ModernEquipmentTestPeer`、`ModernMatchmakingTestPeer`、`ModernServerTestPeer`、`ModernBattleEquipmentProtocolTest`、`ModernBattleRevivalProtocolTest`，以及 `pom.xml`（兩個版本）。
- 產出兩份 TSV：`docs/reports/issue-314/java-peer-tls-inventory.tsv`（333 列）、`java-peer-classes.tsv`（64 列）。

沒有改 `src/`、`tests/`、`TASKS.md`、`PLAN.md`、`docs/REPORT.md`。

## 四種 peer 的啟動方式

| 家族 | Rust 啟動點 | Java 端 | 測試怎麼找到 peer |
|---|---|---|---|
| WIRE／STDIO | `battle_payload_interop_tests.rs:62`（`Peer::start_named`）、`equipment_history_wire_interop_tests.rs:22`、`npc_*_tests.rs`、`protocol.rs:1783` | `ModernBattleWirePeer`（47 個測試共用，依第一個參數分 30 多種模式）、`Modern*WirePeer`、`ModernNpcFireStorm*Peer`、`ModernSessionStdioTestPeer` | 子程序的 stdin／stdout；沒有 socket |
| TLS | `transport.rs:4083`（`JavaPeer::start_class`） | `ModernTlsTestPeer`、`ModernApplicationTestPeer`、`ModernChat*`、`ModernNavigation*`、`ModernNpcService*` 等 | Java 印 `READY <port>`，Rust 連 `127.0.0.1:<port>`，以 `root.der` 驗證 |
| WORLD | `world_integration_tests.rs:314`（`JavaPeer::start_mode_lifetime`） | `ModernWorldTestPeer`、`ModernCombatTestPeer`、`Modern*SqlTestPeer` 等 | 同上；SQL 模式另外建 kill-on-close job、送 `GO`，並把工作目錄切到 `LITO_MODERN_TEST_SERVER_ROOT` |
| FILEFIX | `battle_equipment_tests.rs:137`、`battle_revival_tests.rs:123` | JUnit `ModernBattleEquipmentProtocolTest`、`ModernBattleRevivalProtocolTest` 事先寫出的檔案 | 讀檔，不啟動 Java |

SQL 的判定來源：`world_integration_tests.rs:416–498` 的第二個 `matches!(mode, …)`（79 個模式，與前一個「工作目錄切到 server root」的清單相同）＝會建 process job 的模式；`transport.rs:4084` 只有 `ModernVipMainSqlTestPeer` 會建 job。`canonical-equipment` 在這份清單裡，但 `ModernCombatTestPeer.java` 的 `sql = records ? ModernSettlementMysqlFixture.open(…) : null`，而 `canonical-equipment` 不會讓 `records` 為真（它只因 `actionVersion==41` 需要 `GO`），所以另立 B3。

## 本機前置條件現況（2026-11-12，`Test-Path` 檢查）

| 項目 | 現況 |
|---|---|
| JDK 17 `java.exe`／`javac.exe`／`keytool.exe`（`C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin`） | 存在；`javac 17.0.17` |
| `PATH` 上的 `java` | `C:\ProgramData\Oracle\Java\javapath\java.exe`（不是 JDK 17；測試要求明確指定，不能靠 PATH） |
| 伺服器 `LiTo381-Server/target/`（`test-classes`、`classes`、`lib`、`modern-tls-test-fixtures`） | **不存在** |
| 伺服器 `db/modern_match_settlement.sql`、`pom.xml`、`lib/`（4 個 jar） | 存在 |
| Maven 本機倉庫內 pom 指定的 jar（mysql-connector-j 8.0.33、HikariCP 5.1.0、junit-jupiter 5.10.3 及其相依，共 11 個） | 都在 `D:\dev-cache\maven\repository` |
| 舊的已編譯快照 `…\lito重建版\LiTo381-Modern\target\relocation-baseline-20261003\fixture\java\{test-classes,classes,lib}` | 存在（2026-10-03，不在 git）；64 個 peer class 缺 1 個（`ModernOrdinaryMapRotationTestPeer`，只有 D 批用到）；裡面沒有 TLS fixtures 也沒有檔案 fixture |
| `LITO_MODERN_TEST_JAVA`、`…_JAVA_CLASSPATH`、`LITO_MODERN_TLS_TEST_FIXTURES`、`LITO_MODERN_TEST_SERVER_ROOT`、`LITO_SHARED_RESOURCE_ROOT`、`LITO_MODERN_MYSQL_HOME` | 全部未設定 |
| 共享資源根 `D:\lineage開發\0-客戶端` | 存在 |
| `C:\Program Files\MySQL\MySQL Server 5.5` | 存在（只有 D 批要用，本張不碰） |

## Java 端邊界（讀原始碼所得）

- **TLS fixtures 可以產在任何目錄**：`ModernTlsTestPeer prepare <dir>` 呼叫 `ModernTlsTestMaterial.prepare`，用 `java.home\bin\keytool.exe` 在 `<dir>` 產生 `root.p12／root.der／root.pem`、`server.p12`、`expired.p12` 等，寫完放 `complete-v1`；已有 `complete-v1` 就直接返回。不碰系統信任庫。所以 fixtures 可以放本專案 `target/`，不必放伺服器 repo。
- **listen**：A／B 批用到的 36 個 peer class 中，26 個自己的檔案裡是 `127.0.0.1`／port 0；10 個沒有 listen（7 個 stdio、`ModernNpcFireStormFramePeer`，以及 `ModernPlayerHealingRuntimeTestPeer`、`ModernVipWorldRuntimeTestPeer` 兩個轉呼叫別的類別）。逐類別見 `java-peer-classes.tsv`。
- **暫存目錄**：11 個 A／B 批的 peer class 用 `Files.createTempDirectory("modern-…")`，位置是 `java.io.tmpdir`。不改的話會寫到 `%TEMP%`，違反專案規則第 1 條；Rust 的啟動程式碼沒有傳 JVM 參數，只能靠環境變數改（見下）。
- **自有 MySQL**：`ModernSettlementMysqlFixture.open` 需要 `LITO_MODERN_MYSQL_HOME`；資料目錄建在 `java.io.tmpdir\lito-modern-mysql-fixtures\mysql-5.5-*`；`--bind-address=127.0.0.1`、port 取自 `new ServerSocket(0, 1, 127.0.0.1)`。**它要求 `java.io.tmpdir` 是純 ASCII 路徑**（`checkedTemporaryRoot`），而本專案路徑 `D:\lineage開發\…` 不是 ASCII——D 批要先解決「只寫專案資料夾」與這個限制的衝突（歷史上用的是 `D:\lito-dev-runtime`）。這是自有隔離 DB 那兩張的事。
- **`ModernPetRaceTestPeer`**：沒有 `LITO_MODERN_MYSQL_HOME` 就直接丟例外，確實會開 SQL，所以寵物賽跑 3 項歸 D。
- **檔案 fixture（A0）**：兩個 JUnit 類別寫到相對路徑 `target/modern-equipment-test-fixtures`、`target/modern-revival-test-fixtures`（相對於 JVM 工作目錄）。它們是 `@Test`，沒有 `main`，要靠 Maven surefire 或 JUnit launcher 跑。

## 執行環境（給 T036-10／11；本張沒有實跑，全部待驗證）

共同原則：一次一個測試（`--exact`、`--test-threads=1`），名稱照 TSV 的 `test` 欄；首個失敗就停，不自動重試。

```powershell
# 都在本專案內；<run> 例如 LiTo381-Modern\target\t036-java\<時間戳>
$jp = 'LiTo381-Modern\target\java-peer\<label>'                 # build-java-test-peers.ps1 的輸出
$env:LITO_MODERN_TEST_JAVA           = 'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe'
$env:LITO_MODERN_TEST_JAVA_CLASSPATH = Get-Content "$jp\classpath.txt" -Raw
$env:JAVA_TOOL_OPTIONS = '-Djava.io.tmpdir=<run>\jtmp -Dfile.encoding=UTF-8'   # peer 的暫存目錄留在專案內
$env:TEMP = $env:TMP   = '<run>\tmp'
$env:LOCALAPPDATA      = '<run>\localappdata'                   # preferences.rs:150 的正式路徑是 %LOCALAPPDATA%
# A2／B2／B3：先產 TLS fixtures（預期 stdout 印 FIXTURES_READY）
& $env:LITO_MODERN_TEST_JAVA -cp $env:LITO_MODERN_TEST_JAVA_CLASSPATH local.lito381.server.ModernTlsTestPeer prepare '<run>\tls'
$env:LITO_MODERN_TLS_TEST_FIXTURES = '<run>\tls'
# B 批才設：
$env:LITO_SHARED_RESOURCE_ROOT = 'D:\lineage開發\0-客戶端'
# B3 才設（目錄內要有 db\modern_match_settlement.sql；可指向 -ServerRef 解出的副本或伺服器 repo，只讀）：
$env:LITO_MODERN_TEST_SERVER_ROOT = '<server root>'
```

**fail-closed 的部分（A／B 批一律不設）**：`LITO_MODERN_MYSQL_HOME`、`LITO_MODERN_PROBE_ACCOUNT`、`LITO_MODERN_PROBE_PASSWORD`、`LITO_MODERN_REAL_DB_*`、`LITO_MODERN_TEST_SERVER_JAR`。沒有 `LITO_MODERN_MYSQL_HOME` 時，任何走到 `ModernSettlementMysqlFixture.open(Path.of(getenv(…)))` 的 peer 會先丟例外，不會起 mysqld。A 批另外不設 `LITO_SHARED_RESOURCE_ROOT`、`LITO_MODERN_TEST_SERVER_ROOT`：靜態分錯批的測試會在 `expect` 失敗，而不是多做事。

已知不確定處：
- `JAVA_TOOL_OPTIONS` 裡的非 ASCII 路徑 JVM 能不能正確解讀，沒有驗證。T011-H 的 runner 是把 `-Djava.io.tmpdir` 直接放在 `java` 參數上，這裡做不到（啟動點在 Rust 測試裡）。跑第一個 A1 測試前先確認 `<run>\jtmp` 真的出現 `modern-*` 目錄、`%TEMP%` 沒有。
- T011-H（#91）那套 lease／listener 所有權／身分綁定守門，本批次的 Rust 啟動程式碼只有 `Drop` 時 kill 自己的 child（SQL 模式另有 job）。要不要補 runner，由 T036-10 判斷。

## 與其他張的交叉（TSV 的 `scope54` 欄）

| #54 歸類 | 也要 Java 的數量 | 說明 |
|---|---:|---|
| 共享資源根（159 項那兩張） | 35（B1 12、B2 23）＋ 4 個可能要 SQL | 例：`battle_revival_interop_tests`×6、`transport.rs`×8、`battle_read_interop_tests`×5、`battle_player_*`／`battle_vip_*`／`battle_armor_*`。4 個可能 SQL 的是 `world_integration_tests` 的 `java_combat_*`／`java_armed_combat_*`／`java_learned_teleport_*`：共用 helper 以參數選模式，閉包裡同時出現 `magic`、`records` 與非 SQL 模式，要讀呼叫點才能定 |
| 雜項環境變數 | 2 | `protocol::tests::actual_java_server_negotiates_rust_generated_hello_over_stdio`（A1）、`…::java_battle_read_tls_two_sources_shared_manifest_…`（另要 `LITO_CANONICAL_CAPTURE_DIR`） |
| REAL_DB（已排除） | 1 | `transport::tests::java_application_real_database_readonly_…`：`ModernDatabaseNavigationTestPeer`，連真實 DB，維持排除 |
| DB（自有隔離 DB 那兩張） | 119 | 全部經 `JavaPeer`；前置（classpath、TLS fixtures）與本張相同 |
| GUI（已排除） | 3 | 也會啟動 Java peer；維持排除 |

B1 裡有 5 個共享資源根項目會寫證據檔（`LITO_PLAYER_APPEARANCE_TEST_PNG`、`LITO_VIP_*_EVIDENCE*`、`LITO_MODERN_UTILITY_EVIDENCE`），輸出路徑的所有權屬於「檔案寫入所有權審查」那張。

## #54 基準（`bb96171`，2026-11-05）之後的變動

- #54 的 575 個 lib 列在現行 `main` 都還找得到；其中 559 個一般函式的 `#[ignore]` 都還在（靜態比對），16 個 macro 產生的沒有比對屬性。
- 現行 `src/` 另有約 23 個 `#[test] #[ignore]` 函式不在 #54 清單（`native_static2941_*`、`native_t047*`、`native_login_callback_tests` 等）；靜態上都不碰 Java。
- `tests/` 從基準到現在新增 25 個檔，其中 `native_two_player_battle.rs`（要 `LITO_T043_REFERENCE_JAR`＋`JAVA_HOME`，自己編譯並啟動 E149）與 `native_item_sessions.rs`（要外部 runner 提供的 `LITO_ITEM_TEST_PEER`）有 Java 依賴。這些不在 1055 之內，沒有任何一張 T036 涵蓋。
- #54 的 1 個整合列（`native_ranged::malformed_ranged_packets_are_atomic_and_other_effect_branches_remain_unknown`）在現行來源找不到同名函式。

## 怎麼驗證

### 已驗證
- 169 的來源：#54 逐項表中理由為「Java peer／產生 fixture…」125、「未設定 LITO_MODERN_TLS_TEST_FIXTURES」38、「未設定 LITO_MODERN_TEST_SERVER_ROOT」3、「外部 Java peer 尚未核對」3，全部是 ignored lib、分類「缺前置條件」（`rows.py` 解析 1132 列、白名單 77）。
- 169 項的批次總和 3＋42＋94＋9＋11＋3＋7＝169；TSV 333 列＝本張 169＋共享資源根 39＋雜項 3＋DB 119＋GUI 3。
- 工具可重跑：從 `維護工具/t036-java-peer-inventory/` 重新產生的兩份 TSV 與提交的檔案 SHA256 相同。
- 伺服器 working tree `6bb4a27b`：`javac` main 階段 35 個錯誤（缺 `SharedDatabase`、`UnifiedConfiguration`、`LiToGmCommands`、`LiveBattleStats`、`NativePlayerCommit`、`NativeLightAction`；它們在 `origin/main` 的 `src/main/java`）。
- 伺服器 `origin/main` `edfb5c82`：`build-java-test-peers.ps1 -ServerRef origin/main` 結束碼 0，約 68 秒，4449＋2096 個 class，64 個 peer class 與 `ModernTlsTestMaterial`、`ModernSettlementMysqlFixture`、兩個 fixture 產生用的 JUnit 類別都有產出；`build.json` 記 `executed_java=false`。編譯前後伺服器 repo 的 `git status --porcelain` 相同（只有原本就在的 `?? tools/`）。
- 「本機前置條件現況」表的每一格。
- 「Java 端邊界」各點對應的原始碼行（以 `origin/main` 解出的副本複核過 `records`、`ENV`、`asciiPath`、`--bind-address`、`prepare`）。

### 推測
- A／B 批不會起 mysqld、不會連外：依據是 Rust 的模式清單＋peer 自己的 `.java`；Java 端跨類別呼叫（例如 `ModernCombatTestPeer.run` 之後的 `ModernServer`、各 service）沒有逐一讀。
- 批次的「共享資源」欄：Rust 端以閉包內是否讀到 `LITO_SHARED_RESOURCE_ROOT` 為準，名稱解析偏寬。7 個 `transport.rs` 的純 TLS 測試（`java_tls_*` 5 個、profile probe 1 個、vip worker fault 1 個）閉包只經同名函式誤命中才碰到共享資源根，放在 A2，TSV 的 `shared_evidence` 欄標 `rust-cross-file`。
- `origin/main` 的 peer 與現行 Rust 測試對得上：沒有驗證。4 個相關 peer 檔在 `origin/main` 與 working tree 之間有改動（`ModernBattleWirePeer`、`ModernPlayerHealingRuntimeTestPeer`、`ModernVipTransportFaultTestPeer`、`ModernVipWorldRuntimeTestPeer`）；REUSE-MAP §0.3 也提醒 LS 原始碼比 E149 新。協定版本不合的話，測試會失敗在交握或斷言，那是「版本漂移」而不是功能壞掉，T036-10 回報時要分開記。
- `javac` 產物與 Maven 產物等效：`origin/main` 的 `pom.xml` 在 `process-classes` 階段還會跑 `tools/as-record`（用 ASM 改寫 class），本張的腳本沒有做這一步（那要執行 Java 程式）。peer 會不會因此行為不同，不知道。

### 待驗證（都要實跑，交 T036-10／11）
- 169 項的實際結果。目前 0 項執行、0 項通過。
- `ModernTlsTestPeer prepare` 產出的 fixtures 能用；`JAVA_TOOL_OPTIONS` 的 tmpdir 重導有效；`LOCALAPPDATA` 重導後 `%LOCALAPPDATA%` 真的沒有被寫。
- A0 的檔案 fixture 產生方式（見下）。
- 「整庫 lib 編譯記憶體失敗」（T036 列的前置阻擋）對本批的影響：這 169 項全在 lib 測試執行檔裡，要先有 `cargo test --lib --no-run` 的產物。

## 產出路徑

- `docs/reports/issue-314.md`（本檔）
- `docs/reports/issue-314/java-peer-tls-inventory.tsv`：每個測試一列（批次、#54 歸屬、家族、peer class、SQL 模式、需要的環境、其他環境變數、Rust 端檔案寫入標記、HWND、`#[ignore]` 原文）
- `docs/reports/issue-314/java-peer-classes.tsv`：每個 peer class 一列
- `維護工具/build-java-test-peers.ps1`
- `維護工具/t036-java-peer-inventory/`（`rows.py`、`analyze.py`、`summ_lib.py`、`inventory.py`、`java_direct.py`、`README.md`）

不進 git 的：`scratch/issue-314/`（中間 JSON）、`LiTo381-Modern/target/java-peer/`（編譯產物；worktree 內留有 `current`、`origin-main`、`origin-main-2`、`origin-main-3` 四個失敗嘗試與 `origin-main-4` 成功產物，依規定沒有自行遞迴刪除）。

## 給下一棒的提示

**T036-10（執行 1／2）建議範圍：前置＋A0／A1／A2（139 項）。**
1. 在原始專案目錄（不是會被清掉的 worktree）跑 `維護工具\build-java-test-peers.ps1 -ServerRef origin/main -Label <新標籤>`，記下 `build.json` 的 `source_commit`。先 `-CheckOnly` 看一次。
2. 依「執行環境」設好變數，先跑 A1 的一個小測試（例如 `modern_lito::protocol::tests::actual_java_server_negotiates_rust_generated_hello_over_stdio` 或 `equipment_history_wire_interop_tests` 的一個），確認暫存目錄落點，再往下。
3. A1 的 `ModernBattleWirePeer` 測試若一開始就整批失敗，先懷疑版本漂移：改用 2026-10-03 快照的 classpath（`…\target\relocation-baseline-20261003\fixture\java\{test-classes;classes;lib\*}`）對照一次，兩種來源的結果分開記。快照與哪個伺服器 commit 對應沒有紀錄。
4. A0：需要先產檔案 fixture。可行做法是把 `origin/main` 的 `pom.xml`、`src`、`compile-stubs`、`tools/as-record` 用 `git archive` 解到本專案 `target/` 下，在那個副本裡跑 `mvn -o -Dtest=ModernBattleEquipmentProtocolTest,ModernBattleRevivalProtocolTest test`，再把 `LITO_MODERN_TEST_SERVER_ROOT` 指到副本。這會把 `src/*/resources`（含 1349 個 `.gz` 伺服器資料）複製到 `target/`，是否符合「不複製資源檔」需要 Claude 裁定；沒裁定前 A0 記為阻塞，不要硬跑。
5. 只照 TSV 的 `batch` 欄以 `A` 開頭且 `scope54` 以 `本張` 開頭的列執行。

**T036-11（執行 2／2）建議範圍：B1／B2／B3（23 項）。** B3 的 3 項要 `LITO_MODERN_TEST_SERVER_ROOT`，而且會建 process job；仍然不設 `LITO_MODERN_MYSQL_HOME`。如果調度上方便，共享資源根那兩張裡的 35 個 Java 項目（TSV `scope54=他張:共享資源根`、`batch` 以 `B` 開頭）併到這張一起跑最省事，因為環境完全相同。

**自有隔離 DB 那兩張**：請把本張的 7 項（`map_rotation_interop_tests`×4、`pet_race_interop_tests`×3）收進去；前置同樣是本張的 classpath 與 TLS fixtures，另要先解決 `java.io.tmpdir` 必須 ASCII 的問題。

**共享資源根那兩張**：TSV 裡 `scope54=他張:共享資源根` 的 39 列不是只設 `LITO_SHARED_RESOURCE_ROOT` 就能跑。

**需要另開的事**（不在任何一張 T036 內）：#54 基準之後新增的 ignored lib 測試約 23 個、`tests/` 新增 25 個檔，都還沒有安全分類。

來源一有變動（Rust 測試或伺服器 peer），照 `維護工具/t036-java-peer-inventory/README.md` 重跑，不要手改 TSV。
