# Issue #319：T036-14 自有隔離 DB 盤點（靜態，未執行任何測試）

## 結論

- #54 歸為「DB」的 120 項（119 個 ignored lib＋1 個整合測試）已逐項讀過 Rust 啟動端與 Java peer 端。**119 個 lib 項全部是「測試自己啟動一個全新的 MySQL 5.5 行程」，沒有任何一項連到既有資料庫**；1 個整合測試（`native_travel`）不自己啟動 peer，要靠外部腳本先備好環境。
- 本張把範圍內的測試重新分成五類，清單在 `docs/reports/issue-319/owned-db-tests.tsv`（131 列）：

  | 類別 | 數量 | 說明 |
  |---|---:|---|
  | A 可排入 #320 執行 | 122 | 119 項裡的 115 項，加上 #54 漏列的 7 項 |
  | B 互動視窗，需要操作員 | 4 | 在 #54 的 DB 類裡，但會開真視窗等人操作；建議移入 T030 清單 |
  | C 需要外部 harness | 1 | `native_travel::actual_sql_wand_portal_retires_owner_and_preserves_peer_scene` |
  | D 已在 GUI 排除 | 2 | `r383…:104`、`:226`，同時也會啟自有 mysqld，僅供對照 |
  | E 不是 DB | 2 | `battle_equipment_controller_tests.rs:570`、`:575`，有 Job／GO 閘但 Java 端不開 SQL，僅供對照 |

- **#320 現在還不能開跑**，有五個前置阻擋（見「#320 的前置阻擋」）。其中「暫存根必須是純 ASCII 路徑，但專案資料夾含中文」需要裁定。
- 本輪**沒有執行任何測試、沒有啟動 mysqld 或 Java、沒有編譯**；所有結論都是讀原始碼得到的。

## 做了什麼

1. 讀 `CLAUDE.md`、`docs/REPORT.md`、`docs/REUSE-MAP.md`、`TASKS.md` T036、#54／#92 報告。
2. 用靜態腳本掃 `LiTo381-Modern/src`、`tests` 內所有會啟動 Java peer 的測試（254 個，全部 ignored），取出每個測試用到的 peer class 與 mode 字面值，再與 #54 的分類表逐列比對。
3. 讀 Rust 啟動端兩份 `JavaPeer`：`world_integration_tests.rs:119–722`、`transport.rs:4061–4217`。
4. 唯讀參考 lito-launcher-server 工作目錄（`D:\lineage開發\3-lito開發`，HEAD `6bb4a27`，`src`／`db` 無未提交變更）的 Java 測試原始碼，讀完 `ModernSettlementMysqlFixture.java` 全檔、`ModernQuestSqlFixture.java` 的開啟與設定段、`ModernCombatTestPeer.java:40–189`、`ModernVipMainSqlTestPeer.java` 全檔，其餘 peer 用腳本掃描。
5. 唯讀查本機前置條件是否存在（MySQL 安裝目錄、JDK、伺服器編譯產物、TEMP 路徑、3306 監聽）。
6. 同步 `origin/main`（fast-forward 到 `3a81fc2`），確認清單涉及的 50 個 Rust 檔自 #54 基準 `bb96171` 以來沒有任何變更。

## 已驗證（靜態讀碼，非執行結果）

### Rust 啟動端

- lib 測試用的 `JavaPeer`（`world_integration_tests.rs:314`）對 79 個 SQL mode 建立 Windows Job Object，旗標 `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`（`:121–137`）。先把 Java 子行程指派進 Job，才從 stdin 寫入 `GO\n`（`:600–607`）。測試 panic、READY 逾時或正常結束時關閉 Job，Java 與其 mysqld 子行程一起結束（`:708–716`）；不依行程名或 PID 搜尋。
- `transport.rs` 的 `JavaPeer` 只對 `ModernVipMainSqlTestPeer` 走同一套 Job／GO 流程（`:4084–4141`）。
- SQL mode 的工作目錄設為 `LITO_MODERN_TEST_SERVER_ROOT`，且只檢查 `db/modern_match_settlement.sql` 存在（`:407–411`）。
- Rust 端會從子行程環境移除 `LITO_MODERN_PROBE_ACCOUNT`、`LITO_MODERN_PROBE_PASSWORD`（`:576–577`）。

### Java peer 端

- 清單 A 用到的 peer 共 29 個 class；其中 17 個直接呼叫 `ModernSettlementMysqlFixture.open` 或 `ModernQuestSqlFixture.open*`，**17 個都在第一個 open 之前讀 `GO\n`**（腳本逐檔比對位置）。其餘 12 個只是轉呼叫 `ModernCombatTestPeer` 或 `ModernPetRaceTestPeer`，閘在被呼叫端（`ModernCombatTestPeer.java:85–88`、`ModernPetRaceTestPeer.java:48–49`）。
- `ModernSettlementMysqlFixture`（SHA256 前 16 碼 `5a66fdbdf432a6f9`）的隔離做法：
  - 只讀安裝目錄的 `bin/mysqld.exe`、`share/mysql_system_tables.sql`、`share/mysql_system_tables_data.sql`（`:129–138`）；註解與程式都不複製既有 `data/mysql`（`:207–210`）。
  - `mysqld` 參數含 `--no-defaults`、自有 `--datadir`／`--tmpdir`／`--pid-file`、`--bind-address=127.0.0.1`、`--skip-named-pipe`、`--skip-shared-memory`（`:159–177`）。
  - 子行程環境先清空，只放 `SystemRoot`、`WINDIR`、`PATH=System32`、`TEMP`、`TMP`（`:187–192`）。
  - 埠由 OS 配發，等於 3306 或小於 1024 就拒絕（`:242–247`）。
  - **每一條連線**建立後先查 `@@datadir`、`@@port`、`@@version`，datadir 不是自己的、埠不符或版本不是 `5.5.22` 就中止（`:316–336`）。所以即使埠被別的 MySQL 搶走，也不會把 SQL 寫進去。
  - root 密碼每次隨機產生，日誌結束時把密碼與雜湊塗掉（`:82–88`、`:514–522`）。
  - 成功才刪自己的目錄，逐檔刪、上限 2048 個、每個路徑都檢查在自有目錄內；失敗則整個目錄保留當證據（`:540–561`）。
- 資料全是合成的：schema 為 `lito_settlement_fixture`、`quest_source_7`、`quest_source_42` 與一個 catalog；帳號 `fixture-user`；角色 objid 3333（`ModernQuestSqlFixture.java:321–370`）。設定檔內所有 `mysql.host` 都是 `127.0.0.1`，埠是 fixture 的埠（`:478–486`）。
- 正式程式碼裡會開 JDBC 連線的只有 `ModernJdbcBackend.java:57` 與 `ModernSettlementConnections.java:57`，兩者都吃傳入的設定；讀 `server.properties` 的只有 `Main` 與 `DatabaseSchemaMigrationMain`，SQL peer 用的是 `ModernMain.start(config)`，不經過這兩個入口。
- `ModernCombatTestPeer.java:119` 有一行 `mysql.port=3306`。該檔沒有 `new ModernJdbcBackend`、`new ModernSettlementConnections`、`ModernMain.start`，來源資料由記憶體內的 `ModernApplicationTestBackend` 提供，唯一的 JDBC 是 `sql::openConnection`（`:277`）。這個 3306 只是設定字串，不會被拿去連線。
- 33 個會碰到 MySQL fixture 的 peer，沒有任何一個引用 `ModernDatabaseReadProbe` 或 `ModernDatabaseNavigationTestPeer`（REAL_DB 那一項用的 class）。
- 沒有 GO 閘的 peer（`ModernHasteReadTestPeer`、`ModernVipTransportFaultTestPeer`、`ModernVipWorldRuntimeTestPeer`、`ModernPlayerHealingHistoryWirePeer`）不引用任何會開 fixture 的 class。

### #54 分類的修正

- **漏列 7 項**（#54 歸「缺前置條件」，實際 mode 在 Rust 的 Job 清單內、Java 端會開 fixture）：
  - `map_rotation_interop_tests.rs:124`、`:130`、`:135`、`:140`（`rotation-shared-516-531`／`504`／`505`／`506`）；
  - `pet_race_interop_tests.rs:837`、`:842`、`:847`（`race-finish`／`race-withdraw`／`race-disconnect`）。
- **4 項要操作員**（ignore 文字寫 INTERACTIVE，且走 `login_window::run_connected_test` 或 `run_source_selection_test`）：`battle_controller_interop_tests.rs:65`、`canonical_slow_sql_tests.rs:459`、`:464`、`canonical_source_window_tests.rs:263`。
- **不是 DB**：`battle_equipment_controller_tests.rs:570` 用 `canonical-equipment`，Java 端 `records` 為 false，`sql` 是 null（`ModernCombatTestPeer.java:85–91`）。
- `world_integration_tests.rs:998–1016` 四項用 `normal`／`armed`／`settlement`／`teleport`，不開 SQL，維持 #54 原分類。

### 測試本身會寫的檔案

- Java 端：全部寫在 `java.io.tmpdir` 底下——`lito-modern-mysql-fixtures/mysql-5.5-*`、`lito-modern-mysql-fixtures/lifecycle.log`、`modern-quest-sql-*`、`modern-haste-sql-*`、`modern-healing-quest-sql-*`、`modern-mixed-modes-sql-*`、`modern-combat-peer-*`、`modern-siege-arena-*`。
- Java 端相對於工作目錄只**讀**三個 DDL：`db/modern_match_settlement.sql`、`db/modern_pet_race_settlement.sql`、`db/modern_world_policy.sql`；`Modern*` 正式類別沒有相對路徑字面值。
- Rust 端：`r383_official_window_host_tests.rs:64–91` 寫 `LiTo381-Modern/target/r383-owned-profiles/`（專案內，結束時刪）。其餘寫檔都要另外設環境變數才會發生：`LITO_MODERN_DISINTEGRATE_CAPTURE_DIR`、`LITO_WORLD_MAP_WRITE_REFERENCE`、`LITO_MODERN_SIEGE_CAPTURE_DIR`。
- `LITO_CANONICAL_WINDOW_INSPECT` 設了會開一個可見視窗（`siege_sql_interop_tests.rs:607`、`quest_sql_interop_tests.rs:340`）。

### 本機現況（唯讀查詢，2026-11-12）

| 項目 | 結果 |
|---|---|
| `C:\Program Files\MySQL\MySQL Server 5.5\bin\mysqld.exe` | 存在，檔案版本 5.5.22.0，SHA256 `2D6C58CA657ADE987036BF8614E607BE3F67DDEF6527B419B61FC56C5C705B5B`；兩個 system tables SQL 也在 |
| 專案內既有工具鏈快取 | `LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A`（原始專案目錄，路徑含中文） |
| JDK 17 | `C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot` |
| 伺服器 `target/classes`、`target/test-classes`、`target/lib` | **都不存在** |
| 伺服器 `db/` 三個 DDL | 都在 |
| 伺服器 `lib/` | `mysql-connector-j-8.0.33.jar`、`HikariCP-5.1.0.jar`、`protobuf-java-3.21.9.jar`、`slf4j-api-1.7.36.jar` |
| Maven 本機倉庫 | 有 `junit-jupiter` 5.10.3 全套 |
| 目前行程的 `TEMP` | `D:\dev-cache\tmp`（純 ASCII），底下沒有 `lito-modern-mysql-fixtures` |
| 3306 | **有一個既有 mysqld 在監聽**，本輪沒有碰它 |

## 推測（沒有直接證據）

- `ModernServerConfig.parse` 只解析、不連線。依據是正式程式碼只有兩處開 JDBC，但沒有逐行讀 `ModernServerConfig`。
- 把 `LITO_MODERN_MYSQL_HOME` 指到專案內的工具鏈快取可能失敗：fixture 註解（`:98–100`）記載 MySQL 5.5 遇到含 Unicode 的 datadir 會回傳不相等的路徑；basedir 含中文是否也有問題沒有驗過。`C:\Program Files\...` 是純 ASCII，較穩。
- JVM 當掉時 `hs_err_pid*.log` 會寫到工作目錄。伺服器根目錄現有兩個這種檔，推測是以前這樣留下的。
- TSV 的 `mode` 欄在一個 helper 依旗標選 mode 的檔案裡，列的是該檔的候選集合，不是每個測試確定的單一 mode。所有候選都在 Rust 的 Job 清單與 Java 的閘內，不影響分類。
- Java class 引用關係是用識別字比對算的，會多算、不會少算；沒有考慮反射。

## 待驗證

- **122 項一項都還沒跑**，通過數是 0，不是 122。
- fixture 在這台機器上能不能在 30 秒內 bootstrap 並啟動（`PROCESS_SECONDS = 30`），以及 READY 預算（20 秒或 120 秒）夠不夠。
- 伺服器測試類別能不能在本專案自有目錄編譯出來（共 1411 個測試檔、610 個主程式檔）。
- 本機伺服器 HEAD `6bb4a27` 落後 `origin/main` 632 個 commit，`src` 相差 1859 個檔。`ModernSettlementMysqlFixture.java`、`ModernQuestSqlFixture.java`、`ModernCombatTestPeer.java`、`db/modern_match_settlement.sql` 四個檔兩邊相同；其餘 peer 沒有逐一比。要用哪個 commit 的 peer 來配目前的 Rust 測試，還沒決定。
- B 類 4 項與 D 類 2 項需要真視窗與操作員，屬於 T030。
- C 類的 `native_travel` 要先跑 `維護工具/check-isolated-travel-sql.ps1`（需要 PowerShell 7，JAR SHA 鎖在 `c02aaae8…`，輸出寫到 `驗證紀錄/`），沒有檢查這個 JAR 現在還在不在。

## #320 的前置阻擋

1. **暫存根要裁定。** fixture 要求 `java.io.tmpdir` 是純 ASCII 的真實路徑（`ModernSettlementMysqlFixture.java:116–127`），並拒絕路徑上任何 symlink 或 junction（`:140–147`）。專案資料夾 `D:\lineage開發\…` 含中文，所以 mysqld 的 datadir **不可能放在專案資料夾內**，與「只寫專案資料夾內、不寫 %TEMP%」衝突。建議：請使用者同意一個專用的 ASCII 目錄（例如 `D:\dev-cache\lito-modern\owned-mysql\<run>`），只給這類測試用；執行時用 `TEMP`／`TMP` 指過去。不建議改 fixture 的 ASCII 檢查，那是對方 repo。
2. **Java 測試類別還沒編譯。** 伺服器沒有 `target/`。不能用 `Build-Server.cmd`（會寫進對方 repo）；要用 `javac -d` 輸出到本專案自有目錄，classpath 用伺服器 `lib/*.jar` 加 Maven 倉庫的 JUnit 5.10.3。
3. **整庫 lib 測試執行檔的編譯記憶體失敗**（NATIVE-WORLD-INTEGRATION:5、:9、:177）還沒解。A 類 122 項全部在 lib 測試執行檔裡。
4. **`LITO_SHARED_RESOURCE_ROOT` 與 `LITO_MODERN_TLS_TEST_FIXTURES`**（`root.der`、`server.p12`）分別由「共享資源根」與「Java peer／TLS」那幾張負責。注意 `ModernTlsTestMaterial` 預設寫 `target/modern-tls-test-fixtures`（相對工作目錄），在伺服器根目錄跑會寫進對方 repo。
5. **伺服器 commit 要先釘住**（見待驗證）。

## 未驗證什麼

沒有執行任何編譯出來的程式，沒有跑 `cargo`、`javac`、`java`、`mysqld`；沒有連任何資料庫；沒有讀 `config/server.properties`；沒有改對方 repo。本報告不含任何測試通過結果。背景中一次整個伺服器目錄的檔名搜尋逾時被系統中止，之後改為直接讀指定目錄，不影響結論。

## 產出路徑

- `docs/reports/issue-319.md`（本檔）
- `docs/reports/issue-319/owned-db-tests.tsv`：131 列，欄位 `bucket`、`target`、`test`（可直接給 `--exact`）、`source`、`java_peer`、`mode`、`extra_env`、`note`
- 一次性掃描腳本與中間 JSON 在 worktree 的 `scratch/issue-319/`，不進 git

## 給下一棒的提示（#320 執行張）

- 只跑 TSV 裡 `bucket` 以 `A` 開頭的 122 列，一次一個測試、一個行程：`cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib -- --ignored --exact <test> --test-threads=1`。不要整批 `--ignored`。
- 必要環境變數：`LITO_MODERN_TEST_JAVA`、`LITO_MODERN_TEST_JAVA_CLASSPATH`、`LITO_MODERN_TEST_SERVER_ROOT`、`LITO_MODERN_TLS_TEST_FIXTURES`、`LITO_SHARED_RESOURCE_ROOT`、`LITO_MODERN_MYSQL_HOME`。A 類裡 TSV `extra_env` 欄有值的 3 列（`canonical_slow_sql_tests.rs:454`、`r383_official_window_host_tests.rs:139`、`:211`）另外要 `LITO_EFFECT_CATALOG_PATH`（`D:\lineage開發\客戶端xml\effectlist2.xml`）。
- `LITO_MODERN_TEST_SERVER_ROOT` 建議指到自有目錄，裡面只放從伺服器 `db/` 複製來的三個 DDL。這樣工作目錄不在對方 repo 內，JVM 當掉的 `hs_err` 也不會寫過去。
- 執行前確認這些變數**沒有設**：`LITO_MODERN_TEST_SERVER_JAR`、`LITO_MODERN_TEST_SERVER_JAR_SHA256`（會另外啟一個 `java -jar`）、`LITO_MODERN_TEST_EXPIRED_VIP`、`LITO_CANONICAL_WINDOW_INSPECT`、`LITO_WORLD_MAP_WRITE_REFERENCE`、`LITO_MODERN_DISINTEGRATE_CAPTURE_DIR`、`LITO_MODERN_SIEGE_CAPTURE_DIR`、`LITO_MODERN_PROBE_ACCOUNT`、`LITO_MODERN_PROBE_PASSWORD`、`LITO_MODERN_REAL_DB_SERVER_CONFIG`。
- 每個測試前後記錄 3306 的監聽行程 PID 沒變，當作「沒碰既有資料庫」的執行期證據。
- 失敗時 `<暫存根>/lito-modern-mysql-fixtures/mysql-5.5-*` 會保留，是證據，不要遞迴刪除；`lifecycle.log` 每次都會留下，裡面有每次啟動的 PID、埠、datadir。
- 單一測試的期限從 50 秒到 590 秒不等，peer 存活上限 60–600 秒；122 項依序跑要以小時計。
- B 類 4 項請由雲端對話移入 T030 清單（`docs/GUI-ACCEPTANCE.md`）；TASKS.md 的「REAL_DB 1 項」對應 `transport.rs:7807`，不在本清單內。
