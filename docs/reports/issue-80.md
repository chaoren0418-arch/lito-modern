# Issue #80 — T011-R 當版 owned teleport harness 靜態盤點

## 結論與範圍

**已驗證（靜態）：五案盤點完成；五案皆判定「不可執行」，T011-B 保持阻塞。** 這是啟動鏈／隔離條件尚未封閉，不是五案測試失敗或通過。僅補上 `LITO_TELEPORT_SCENARIO` 不能解除 #54 的排除。

- 基準：2026-11-05 讀取 GitHub Issue #80 本文及全部留言，仍為 open、`runner:dot`、`task:running`；開始時只有派工留言，沒有 RESULT、task/80 遠端分支或同分支 PR。
- `origin/main`：`e3e5f00d1970d03bf40d505299293aff0b5f9677`。隔離 worktree：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80`，分支 `task/80`。主工作目錄原為 `92bbda01c30488cea855f84c6ba7d1e5b23c7c98`；未切換或套用其來源。
- 已讀原工作目錄與基準版 AGENTS.md、基準版 TASKS.md 的 T011，以及 `docs/reports/issue-54.md` 五列。基準規則記載恢復測試，不改變本次明確「只讀、不執行」範圍。
- 唯一追蹤產出：本檔。只執行文字／JSON／檔案雜湊／JAR class 結構讀取及 Git 操作；沒有 cargo、javac、Java harness、SQL、MySQL、遊戲、GUI 或 socket 探測。未讀寫 `config/server.properties`、`docs/REPORT.md`、封存或 BIN 資源；未改安全設定。
- 完全存取：**未設定、無法驗證為全域完全存取**。環境是 workspace-write；一般網路 Git 查詢失敗後，僅 Git 遠端操作使用逐項核准的提升權限。這不等於整體沙箱解除，亦不是盤點阻塞。

## 五案對照

以下共通前置 P 與共通副作用 S 是每列的一部分。

**P：**當版測試二進位；當版兩個 Java 來源及可證明對應的 class；固定 E149 JAR；新的專屬 root 位於該測試編譯時 manifest 的 `target/current`；真實 owned peer 產生的 ready；隔離及清理證明；可讀的原客戶端地圖／圖形資源。若改走 E149 原 resource factories，還需要 JDBC、MySQL bundle、靜態 SQL、owner/import proofs、當版 bootstrap 及 source/class 綁定。資源檔存在不等於上述啟動鏈成立。

**S：**Rust 本體不啟動外部程序、不直接開 DB，但會建立實際 TCP LOGIN／GAME／傳送連線，兩個帳號 `PeerFixture`／`PeerFixture2` 登入、進房、移動、NPC 操作及恢復；讀 `D:\lineage開發\0-客戶端`；在 root 寫每角色回報及完成標記。Java peer 會開 loopback listeners、工作執行緒、寫測試設定與 capture/trace。原 runner 還會開 javac/java/test.exe；E149 runner 另開 Python、mysqld/mysql/mysqladmin 及 Java 內的 PowerShell listener 身分查詢。

| scenario／精確測試入口 | #54 原分類與原因 | 本次前置與副作用邊界 | 最終判定 |
|---|---|---|---|
| `timeout` — `destination_timeout_retires_old_authority_and_allows_fresh_login`（104） | 缺前置條件：未設定 LITO_TELEPORT_SCENARIO；前置未成立不代表其餘副作用安全 | P、S；第三埠綁 `127.0.0.3:0` 後不 accept，實際 TCP 連入但無 S_KEY。兩角色依序驗證失敗 5–15 秒內，8 個正式協定 session；Rust 入口只排除 cancel-expiry，沒有強制 scenario=timeout | **不可執行**：舊 runner 不接受 E149；E149 runner 只綁候選，當版 peer/resource 初始化與所有權鏈未成立 |
| `refused` — `refused_destination_retires_authority_and_allows_fresh_login`（121） | 同上：缺前置條件，未設定 LITO_TELEPORT_SCENARIO；不代表其餘副作用安全 | P、S；第三埠先由 `127.0.0.1:0` 取得，關閉後 Java 做最多 5 秒 connect probe，必須真的 ConnectException；Rust 檢查 native TCP connect error 且少於 5 秒。釋放到 client 連線之間仍有埠被其他程序重占的競態 | **不可執行**：共通啟動鏈阻塞，且未實證拒絕端口／碰撞排除 |
| `wrong-service` — `wrong_service_destination_retires_authority_and_allows_fresh_login`（126） | 同上 | P、S；第三個 `127.0.0.3` listener 實際執行 LOGIN ClientSession，作為錯誤 GAME 目的端；共 10 sessions，失敗需少於 15 秒 | **不可執行**：共通啟動鏈阻塞；不能使用既有 LOGIN 服務替代 |
| `cancel-retry` — `cancellation_then_immediate_retry_consumes_staged_character_once`（116） | 同上 | P、S；進入 GameConnecting 且 epoch+1 才 cancel；最多 3 秒等待 worker 回收，取消耗時小於 5 秒；自 requested 起自然等到至少 600ms，再做 fresh LOGIN/GAME 與 staged character 單次消耗；8 sessions | **不可執行**：共通啟動鏈阻塞；當版 class／E149 stage 行為待驗證 |
| `cancel-expiry` — `cancellation_retires_destination_and_expired_handoff_requires_fresh_selection`（110） | 同上 | P、S；同取消條件，自 requested 起每角色自然等待至少 33 秒，寫 expiry marker；恢復必須重新選角。角色 b 等 a 完成才做其流程，故兩個 expiry 段合計至少約 66 秒，另加登入、資源載入、移動時間；8 sessions | **不可執行**：共通啟動鏈阻塞；33 秒等待不可快轉或換假時鐘 |

行號皆相對 `LiTo381-Modern/tests/native_teleport_failure.rs`。本次 process environment 中三個 LITO_TELEPORT 變數皆未設定。注意 timeout 在 Rust helper 內有預設值；#54 的「SCENARIO 缺少」是歷史分類，不是說其入口必定因該變數而失敗；它仍先缺 ROOT/ready/peer。

## 當版 Rust 與 ready 契約

`run_owned_harness`（129–134）讀取：

| 變數 | 當版要求 |
|---|---|
| `LITO_TELEPORT_SCENARIO` | 上表五值；後四案入口 assert exact value。timeout 須操作員明確指定，避免名稱與實際行為錯配 |
| `LITO_TELEPORT_ROOT` | 必填。root 與 `env!("CARGO_MANIFEST_DIR")/target/current` 都必須已存在且可 canonicalize；root canonical path 必須 starts_with 後者 |
| `LITO_TELEPORT_REFERENCE_SHA` | ready.jar_sha256 必須等於此字串；未設時預設舊 `bbc59308c43df89ee5029d3b3c802be8ad55fee65c41faca99a76912dd5dce43`。E149 必須明確設小寫 `e1498871c7be6b287821c57ef5d95933db1740d9b4d4a80f6c3465f21d99d498` |

當版 Java（76）產出的 `$root/ready.json` 格式如下（佔位值不是可用 ready，不應人工建立）：

```json
{
  "port": "實際 LOGIN listener 的整數埠號",
  "destination_port": "實際失敗目的端的整數埠號",
  "destination_refused": false,
  "destination_wrong_service": false,
  "jar_sha256": "實際 ClientSession 來源 JAR 的小寫 SHA256"
}
```

Rust 只消費 `port` 的 u64 值及 `jar_sha256`，組成 `127.0.0.1:<port>`；不驗證 scenario、PID、埠所有權、ready 新鮮度、destination 欄位或 DB proof。也沒有拒絕 2000／3306 的檢查。root 的 canonical 檢查不代表 root 內個別既有檔案沒有 reparse／硬連結，也未禁止 stale marker 覆用。Java current main 只 normalize root，沒有同等 target/current／全新目錄限制。必須由可信任的全新 owned producer 封閉這些邊界。

`native_login.rs:212–218` 的來源 redirect 只檢查 index／port 非零；`native_connection.rs:133–137` 直接 connect peer。**初始 peer 是 loopback 不足以證明後續 redirect 全部是 owned loopback**；要以固定 fixture 來源及埠所有權證據約束，不能任給 ready 指向既有服務。

預期寫入（非本次實際寫入）：

- Rust root：`a/b-failure.json`、`a/b-failure-complete`、錯誤時 `a/b-recovery-error.json` 或 `a/b-recovery-frame-error.json`；expiry 多 `a/b-expiry-observed`。這些以 write 覆寫，b 等 a marker，舊 marker 可能影響隔離。
- 當版 Java root：`server.properties`（**測試 root 內的新檔，不是正式 config**）、`ready.json`、`server-trace.jsonl`、`staged-trace.jsonl`、`result.json`、`captures/`；設定的 `maps/`、`state.properties`、`unused-sql.properties` 均指向 root。state persistence=false；其餘 JAR constructor 的實際 I/O 仍不能僅由設定宣稱全無副作用。
- 三個 listener 全用 port=0；LOGIN=127.0.0.1、GAME=127.0.0.2、失敗端=127.0.0.3（refused 改 .1 並先關閉）。它們不是 DB；DB 的「只綁 .1」條件不應誤套到兩個 GAME loopback alias。
- Java fixture 本身上限 200 秒；expiry 320 秒；accept timeout 90／180 秒；每 acceptor/session join 3 秒。舊 runner lease 220／340 秒，ready 等 15 秒、退出等 8 秒。這些是上限而不是測試通過時間。

## 產生工具與當版適用性

| 工具 | 查到的行為 | 當版判定 |
|---|---|---|
| `維護工具/run-teleport-failure.ps1` | 4–14 硬編主 repo、只收主 repo target/current 的 TestBinary/JAR；生成 runtime/oracle 與 `驗證紀錄/current/native-teleport-failure/<stamp>`；45 編譯兩個當版 Java；47 啟 Java、49 等 ready；51 設三變數後執行單一測試二進位；finally 只在 PID/出生時間/命令匹配後回收 | 8 的白名單只有 BBC59308／9719F45D／6928A147／76EBE06D／C02AAAE8，**沒有 E149**。不是可從隔離 worktree 直接使用的 E149 runner。未隔離 Java temp／繼承 JVM options；37 還 pin `.cargo/config.toml`（新 worktree 找不到）。不得改白名單繞過 |
| `維護工具/run-teleport-failure-75DA.ps1` | 同型 runner，8 強制 75DA69B0… | 不是 E149，不能改用舊 JAR 冒充當版 |
| `維護工具/verify-native-teleport-window.ps1` | 原生 GUI 專用，編譯 harness 並啟動 owned native windows | 不屬本任務授權，不能當 headless 五案入口 |
| `維護工具/run-native-E149-recovery-case.py` | 需 v4 plan、binding、Java classes、linked image、producer review；198–243 會啟自有 DB、Java E149 fixture、候選測試 image；Java 需 8 參數及 actual owner/import proofs；另加 JDBC、profile/catalog JVM properties；驗證擴充 ready | 不使用當版單純兩參數 Java，且不執行 cargo 當版來源。`native_E149_recovery_contract.py:180–242` 由候選 overlay 產生 `維護工具/candidates/native-E149-recovery-case-plan-*/plan.json`。不可將候選 class/image 轉作當版證據 |
| `維護工具/native-E149-owned-mysql.py` | plan/serve/status/stop/recover 具私有 datadir、臨時高埠、靜態 SQL、shutdown 與程序身分檢查 | 是候選 E149 鏈的 DB producer；原始碼具隔離措施，未實跑。沒有使當版 Java 自動取得原 resource factories |

E149 runner 還會讀舊 `工作佇列.json`，由 `checkpoint()` 修改八個舊進度檔、工作狀態並產生 timestamp audit/archive；此流程不符合本次只新增報告以及現行停止舊工作佇列流程的界線。未呼叫它，未改舊旗標。其失敗分支可能終止自己持有的 controller/fixture/job；DB 正常用 mysqladmin，失敗可強制回收，**強制回收不能算符合「shutdown 成功並確認退出」**。

實讀的已登記 binding：`驗證紀錄/current/native-E149-observer-stability-binding-development/20261024-024235-491711/fixture-binding.json`，status=`source_binding_only_no_runtime`。它指向 `維護工具/candidates/native-E149-observer-stability-20261024-023412-802678/...`；候選 harness SHA=`235C0BB9…`，當版=`F67D61B8…`，明確不是同一來源。候選另有 Bootstrap/ReadonlySnapshot、owner guard、retired mutation/morph probes；這些超過當版五案，不得附帶執行。

**已驗證（靜態）**：候選 bootstrap `NativeRecoveryProductionBootstrap.java:70–152` 驗證 own DB PID/出生時間/image/port/config proof，JDBC URL 為 `127.0.0.1:<owner port>/<schema>`，呼叫原靜態 item resource factories。當版 harness 沒有這段 bootstrap 或 proofs/JDBC 介面，仍採 memory accounts。memory mode 不等於已證明 E149 resource 初始化完整，也不能將候選 `static_sql_access=true` 套到當版。以 ZIP/class 結構讀取確認 E149 仍有當版所用 GameStateStore(Path,bool)、GameHub(config,state,accounts) constructor；**沒有據此宣稱編譯不相容或可運作**，完整 API/初始化待驗證。

## 自有隔離 DB 條件逐項判定

當版 memory harness 沒有自行啟動 DB，也沒有此 DB proof 鏈。以下僅評估 E149 DB producer 的來源；全部都未啟動。

| 條件 | 靜態證據／可滿足性 | 本次資格 |
|---|---|---|
| 本任務自行啟動 mysqld，第一參數 `--no-defaults` | common command 同供 bootstrap/daemon；保留 Popen identity，排除 MYSQL inherited vars | 原始碼具備；本次禁止啟動，待驗證 |
| datadir/tmpdir 只在 target/current 或 scratch | `CASES=P/target/current/verification/native-E149-owned-mysql`；每案 root/data、root/tmp，檢查 owned/reparse | 原始碼具備；隔離 worktree 沒有實際案，待驗證 |
| 只綁 127.0.0.1，OS 配埠 | reservation bind(.1,0)、SO_EXCLUSIVEADDRUSE、限定49152–65535；release 後交 mysqld；verify_live 查真正 listener owner | 原始碼具備；釋放競態必須由 listener 檢查拒絕，未實證 |
| 只匯入固定靜態表、無玩家資料 | 實讀固定 SQL SHA 相符；19 CREATE＋6277 INSERT，表名與 manifest 一致。producer 另建新 DB 的 vendor 系統表與隨機合成帳號，沒有匯入舊玩家 DB；匯入後查表名／列數／SELECT 權限 | 檔案與選表已靜態核對；實際匯入／權限／JDBC 待驗證 |
| mysqladmin shutdown 並確認退出 | finally verify_live→專用 defaults-file mysqladmin shutdown→wait15；錯誤記錄並 fallback terminate；runner 另持 handle/job 終止確認 | 正常路徑具備；未發生 shutdown；fallback 不算通過 |
| 不連正式 2000／3306 或正式資料庫 | DB 高埠及新 schema、固定 loopback；未讀正式帳密。當版 Rust 對 ready/redirect 沒有同等拒絕檢查，須 producer 保證 | DB 路徑靜態未指向正式埠；整體五案仍未封閉 |

固定 SQL 表為 armor、etcitem、etcitem_arrow、weapon 與 15 張裝備屬性／祝福／附魔／卡片表；沒有 account、character 或玩家 inventory 表。本次未讀來源全庫 dump，未匯入任何 SQL。

## 前置資源與尚缺項

已實讀 SHA 的完整清單附後。JDK 只 hash 可執行檔，沒有 `java -version` 或編譯；MySQL bundle 的 33 檔皆與 manifest 相符；JDBC 與正式／快取 E149 JAR 一致性已核對。資料夾沒有單一 SHA，以下用 bundle manifest 加逐檔 SHA 表示，避免把歷史宣告當本次實讀。

| 前置 | 本次結論 |
|---|---|
| JDK 17 java/javac、當版兩個 Java、E149 JAR、JDBC、MySQL、靜態 SQL | 存在，SHA 附後；不代表執行相容性 |
| 當版五案可用的 class 綁定／E149 owned producer | **找不到可證明當版可用的完整鏈**；已登記六個 class 是候選，逐檔存在但不得替換 |
| 隔離 worktree 的 target/current、fresh root、ready、owner/import proofs | **找不到**：新 worktree 沒有 target/current；沒有為本次建立 runtime 或執行 producer |
| 當版測試 image | 未建置、未執行；候選 linked image 不採用 |
| client 資源與 E149 catalog BIN | 受「不碰 BIN」界線限制，未讀取／雜湊；Rust 的 Model::load 與 E149 `-Dlito.commonCatalog` 仍是依賴；不可宣稱已齊備 |
| 安全、直接可執行的 harness 啟動指令 | **找不到**。現有兩套鏈各有上述阻塞；不能自行編造 plan、owner/import proof、class 身分或 ready |

## 指令交接（未執行；不是已可執行 recipe）

本 Issue 要求可直接執行的完整指令，但已發現真實阻塞；本報告不能以補佔位值假裝完成該項驗收。以下只精確交接已知介面與五個 Cargo 入口，**仍缺安全且當版對應的 harness 產生／清理命令**，T011-B 不應開始。每案必須獨立 fresh root，不能一次跑五案或共用 stale marker。

舊當版 Java 介面（從 runner 靜態抄錄；非可執行資格）：

```text
javac.exe -J-Dfile.encoding=UTF-8 -encoding UTF-8 -cp <固定JAR> -d <本案classes> <當版NativePeerHarness.java> <當版NativeTeleportFailureHarness.java>
java.exe -Dfile.encoding=UTF-8 -Xverify:all -cp "<本案classes>;<固定JAR>" local.lito381.server.NativeTeleportFailureHarness <本案root> <scenario>
```

E149 候選介面還需 `-Djava.io.tmpdir=<root/tmp> -XX:-UsePerfData -Dlito.resourceProfile=tw13081901-v1 -Dlito.extendedItemDetails=true -Dlito.commonCatalog=<catalog>`，classpath 加 JDBC，main 參數為 `root scenario owner-file owner-sha import-file import-sha SQL --execute-owned-tests`。**當版 main 不消費這些新增 proof 參數；把命令換上當版 class 也不會產生候選 bootstrap。** 不提供可繞過檢查的替代啟動命令。

後續修復並經審查的 producer 必須交付 root、classes SHA、實際 listener/PID、來源 pins、退出程序，並限制 temp／繼承環境。下列每一段的 ROOT 都必須由該 producer 提供，不能自行指定既有 ready。前面加明確停止，避免在鏈尚未補齊時當成可跑腳本：

```powershell
throw 'T011-B blocked：當版 E149 owned producer／清理鏈尚未通過審查；以下僅交接精確入口'
Set-Location 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80\LiTo381-Modern'
$env:LITO_TELEPORT_REFERENCE_SHA='e1498871c7be6b287821c57ef5d95933db1740d9b4d4a80f6c3465f21d99d498'

# timeout：ROOT=審查後 producer 為本案產生且仍存活的全新 owned root
$env:LITO_TELEPORT_ROOT='<timeout owned root：找不到可用 producer>'
$env:LITO_TELEPORT_SCENARIO='timeout'
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport_failure --no-fail-fast -- --ignored --exact destination_timeout_retires_old_authority_and_allows_fresh_login

# refused：不同 fresh root；Java 必須獨立確認 ConnectException
$env:LITO_TELEPORT_ROOT='<refused owned root：找不到可用 producer>'
$env:LITO_TELEPORT_SCENARIO='refused'
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport_failure --no-fail-fast -- --ignored --exact refused_destination_retires_authority_and_allows_fresh_login

# wrong-service：不同 fresh root；錯誤端由本案 owned LOGIN peer 提供
$env:LITO_TELEPORT_ROOT='<wrong-service owned root：找不到可用 producer>'
$env:LITO_TELEPORT_SCENARIO='wrong-service'
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport_failure --no-fail-fast -- --ignored --exact wrong_service_destination_retires_authority_and_allows_fresh_login

# cancel-retry：每角色 requested 起至少自然 600ms
$env:LITO_TELEPORT_ROOT='<cancel-retry owned root：找不到可用 producer>'
$env:LITO_TELEPORT_SCENARIO='cancel-retry'
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport_failure --no-fail-fast -- --ignored --exact cancellation_then_immediate_retry_consumes_staged_character_once

# cancel-expiry：每角色 requested 起至少自然 33 秒；兩角色依序
$env:LITO_TELEPORT_ROOT='<cancel-expiry owned root：找不到可用 producer>'
$env:LITO_TELEPORT_SCENARIO='cancel-expiry'
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport_failure --no-fail-fast -- --ignored --exact cancellation_retires_destination_and_expired_handoff_requires_fresh_selection
```

## 怎麼核對／未驗證什麼／給下一棒

已驗證僅指：上述來源條件、檔案存在、實讀雜湊、五個入口與時間界線、候選／當版身分差異、SQL 選表與文字計數。方法為直接讀取 tracked source、target/current 固定快取、resources 固定 SQL、已登記 binding 的精確檔案；不匯入或執行任何 repo Python 模組。JAR class 僅用 Python zipfile/struct 解析 constructor descriptor。靜態暫存工具在 worktree `scratch/inventory80.py`、`scratch/jar80.py`；不是測試或 harness。

推測：當版 memory fixture 缺少 E149 原 resource 初始化可能影響登入／世界／裝備恢復；須由來源接線核對及後續授權驗證判定，未直接宣稱必然失敗。

待驗證：完整當版 Java/E149 編譯與 runtime 相容、資源載入、JDBC/MySQL 5.5.22 相容、DB bootstrap/import/grant、埠碰撞、退出回收、五案所有功能斷言。未跑任何測試，沒有首個測試失敗輸出，不能報略過／ignored／零測試為通過。

建議下一棒由 Claude 審查裁定最小「當版 E149 owned harness」開發範圍，再交本機開發者補足：移除舊硬編根／進度寫入耦合、當版 source/class/JAR 綁定、fresh root 與 ready 所有權、redirect／DB 埠驗證、temp 界線與可證明退出。不得套用凍結候選或改安全旗標替代修復。補齊後另交 T011-B 執行，第一個失敗停止並留原始輸出、不自動重試。本次沒有開新任務或修改 src/tests/scripts。

## 實讀資源 SHA256 明細

以下所有「pin 相符」都代表本次重新計算比對；不代表 runtime 通過。

| 項目 | 絕對路徑 | 狀態 | 實讀 SHA256 |
|---|---|---|---|
| JDK 17 java.exe | `C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe` | 存在 | `0D3815053960E0B7695A09E8C26406C53D92B13C71E946CCB6EB84E6A8028EDE` |
| JDK 17 javac.exe | `C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javac.exe` | 存在 | `5A4D18EF2035F89807252BF313652044DC8BFB698FDC210D457A73574362AC33` |
| 當版來源 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80\LiTo381-Modern\tests\native_teleport_failure.rs` | 存在 | `B47D11DFBEFCD1523E8E0AE607C448039C24D441405E26EE8A11130776EBBCFA` |
| 當版來源 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80\LiTo381-Modern\tests\oracle\NativeTeleportFailureHarness.java` | 存在 | `F67D61B8F6170E2E48C3FBF4445E8754E9132600824101D2A9C2A732DF0F69B3` |
| 當版來源 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80\LiTo381-Modern\tests\oracle\NativePeerHarness.java` | 存在 | `E6108F193F70EE219FD342695C95E5499668BE4FBA8E9A5B2B3D99875751863D` |
| 正式 JAR（唯讀） | `D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar` | 存在；pin 相符 | `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498` |
| 正式 JAR 快取 | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar` | 存在；pin 相符 | `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498` |
| JDBC 快取 | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\server-reference\lib\mysql-connector-j-8.0.33.jar` | 存在；pin 相符 | `E2A3B2FC726A1AC64E998585DB86B30FA8BF3F706195B78BB77C5F99BF877BD9` |
| MySQL bundle manifest | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\bundle.json` | 存在；pin 相符 | `279356805C4C35ADD6E16C98545A369599F1F226F28AC2909823E2BB641FC3DC` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\bin\mysql.exe` | 存在；pin 相符 | `D44920ADC09266E5F49B1E53EC5B78CED28E85B4A3D532DC4B50EF1D64DDB0FD` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\bin\mysqladmin.exe` | 存在；pin 相符 | `2414F6776DD8F666114F74A2D07168BBAEEB216C86AC0CEE7FC51F4197706B84` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\bin\mysqld.exe` | 存在；pin 相符 | `2D6C58CA657ADE987036BF8614E607BE3F67DDEF6527B419B61FC56C5C705B5B` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\Index.xml` | 存在；pin 相符 | `20031D3B9296CF4C93174E9BC5457D705A4D7A6D93E4FA4FD018E4A5C32C90ED` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\README` | 存在；pin 相符 | `0AF62CE1AE651E980006F2E9D29BD2F24E1CD18C6A0B1914E9C437CC80E990AB` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\armscii8.xml` | 存在；pin 相符 | `E01038504EF8AA5A0446BEE9A9E145ECB7B36D55629636D34347653AB6CFA752` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\ascii.xml` | 存在；pin 相符 | `9F84003651A656972E5331309669639815EDF0BAA1FE8CA941D461838608021C` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\cp1250.xml` | 存在；pin 相符 | `D06BC18DA702F1B8FE08C75C27579962580B81647E63651D354AF40551066A6A` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\cp1251.xml` | 存在；pin 相符 | `C3573CEC09DD0F4F0CE59559C1AD99666D4560641EBFFA63F2D7D1683B35DFD5` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\cp1256.xml` | 存在；pin 相符 | `6C27DF1E06304C8BBBB923D803E0B9374B80D24F07176DF89C2156F803993E10` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\cp1257.xml` | 存在；pin 相符 | `99D9A977A9C6A1BECD43B00EFDDBBB33D1F27714D9FC7A9C4C22146F939FFF6B` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\cp850.xml` | 存在；pin 相符 | `B4C100EBC6872C373D95BCBDE2DF2FCA0DD2716B706AFED5BBBE456448FA0CA0` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\cp852.xml` | 存在；pin 相符 | `39EB33C14B41BD906FA2561DED53B85C37055AA8B6F699ECF894E1E414C63E4F` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\cp866.xml` | 存在；pin 相符 | `AC6484FA54D3C8CB31DBF2F2F8CDD21357BBAD9AAA956A4CB1BC0C9D17B0941C` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\dec8.xml` | 存在；pin 相符 | `FFC3A988E48EFEE0B9478701AD852CC929B0752C4EB11E96B5D8B0705582456A` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\geostd8.xml` | 存在；pin 相符 | `7EBD0EDFD484F34B6C7346D8E39B4AB6FE74968764AF12CE7C9A3552904CE6D2` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\greek.xml` | 存在；pin 相符 | `16A97D6C557685B699596244AA506F711FFF3F3A345AFEBDDCE770484FA6B203` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\hebrew.xml` | 存在；pin 相符 | `65ED41EF4DEFB996CF40CB11919F2F8E3B0641E65FA17DC0E1D67D9DC258F593` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\hp8.xml` | 存在；pin 相符 | `C6B508F62643994C53BC7A5BD2DD605286BD8CDA057788D8EAA2DA609F01EC8D` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\keybcs2.xml` | 存在；pin 相符 | `51BAF840255E4CF4CB4700BAD344CF4B592E24A7B23B9C459C3FA0E29961C528` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\koi8r.xml` | 存在；pin 相符 | `645686F2202EBCCB8CCF31762CCDCC3DF106CE557B6313B0D8607DBA6AD9E61F` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\koi8u.xml` | 存在；pin 相符 | `4BD9E49CA61BB3774869BD85D7C9EBECC6FC20E591912FCFE0354044909C9748` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\latin1.xml` | 存在；pin 相符 | `3C46FC7008255C9FCB4AD7AF18FA1995BCFFF596D3F5BDC330334E09BACECFB6` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\latin2.xml` | 存在；pin 相符 | `D5328B3BA3B8B2BED5636A39EDACE5D006AD75CDE36DA59114902F4D86DD9B19` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\latin5.xml` | 存在；pin 相符 | `48E0C713A184122D15A38307DD82DA126BCBC288ABC638BF1BA936A4857E29A0` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\latin7.xml` | 存在；pin 相符 | `C6B0FFFB1D061A71A25DEBD6AAD0EC0FA5AD8BFE8BA3FE2E529619ED5C8FD8A9` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\macce.xml` | 存在；pin 相符 | `AEED0E5DAE008FE51C09B413AB9DC179A088E230C9B4C21EFBE7A8C74259FB22` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\macroman.xml` | 存在；pin 相符 | `7067C80322E1D8E053479E2C7F1FBAE381D1E3C3A8697479876873977B3F597F` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\charsets\swe7.xml` | 存在；pin 相符 | `836207BE2D832BA797BC0E232F96D70B56CDDB049F9995E6B8B5A9E4D061C76E` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\english\errmsg.sys` | 存在；pin 相符 | `F06CB3D1DBC6AE2F55A0F96ED1DA507997FE5DEEAE8BE794FBB5B19555F8AC18` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\fill_help_tables.sql` | 存在；pin 相符 | `E2D5C30C900B2A0686231CD78753F65D6411800BCEA305E27CF53E1135815144` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\mysql_system_tables.sql` | 存在；pin 相符 | `E837765728BDE300A3273CBBCD911BA09703BD8CD725C804D8F4FC0C0F66C29E` |
| MySQL bundle | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\share\mysql_system_tables_data.sql` | 存在；pin 相符 | `20CA7E859293B397F5BDDCC4E690494E76F20FE146B0778A77953A71EF658F7B` |
| 靜態 SQL manifest | `D:\lineage開發\3-lito開發\lito重建版\維護工具\resources\E149-static-item-tables-D4111D8AEFF5\manifest.json` | 存在；pin 相符 | `91CE4FB2317AC2AED0701AA35B4D6AAA1A6B0CE214A9CEA27C8F2CDF35647921` |
| 靜態 SQL | `D:\lineage開發\3-lito開發\lito重建版\維護工具\resources\E149-static-item-tables-D4111D8AEFF5\static-tables.sql` | 存在；pin 相符 | `C1E29B0479FE40476AD53B4E52498451C3A1E469A27483EABE67FA880EBCA7DD` |
| 候選 fixture binding | `D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-observer-stability-binding-development\20261024-024235-491711\fixture-binding.json` | 存在；pin 相符 | `481C3F82A8C257F7CBF6201DE5C3BBE959110EE4EFDCA3505B391C3A903F8472` |
| 候選 fixture 綁定來源 | `D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-observer-stability-20261024-023412-802678\LiTo381-Modern\tests\oracle\NativeTeleportFailureHarness.java` | 存在；pin 相符 | `235C0BB96D18ACB79326104FA14BA29723BE560079F1CB02323B4F2FF89204D0` |
| 候選 fixture 綁定來源 | `D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-observer-stability-20261024-023412-802678\LiTo381-Modern\tests\oracle\NativeRecoveryReadonlySnapshot.java` | 存在；pin 相符 | `A12CC6AC2E136FDE482F255DFBB66B71F112EB236036DE1ADA97A1E263727E74` |
| 候選 fixture 綁定來源 | `D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-observer-stability-20261024-023412-802678\LiTo381-Modern\tests\oracle\NativeRecoveryProductionBootstrap.java` | 存在；pin 相符 | `793064EFA911F12E536D217456E86360CD52B4CAECEC612807AF019DB707C6A1` |
| 候選 fixture 綁定來源 | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\oracle\NativePeerHarness.java` | 存在；pin 相符 | `E6108F193F70EE219FD342695C95E5499668BE4FBA8E9A5B2B3D99875751863D` |
| 候選 fixture class（不是當版證據） | `D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-recovery-bootstrap-compile\20261024-023424-550188\classes\local\lito381\server\NativePeerHarness$Session.class` | 存在；pin 相符 | `AF277F5127AD1CD4F1677D412E00DF5D4F3C0559A801CCCB59618E4DF735B989` |
| 候選 fixture class（不是當版證據） | `D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-recovery-bootstrap-compile\20261024-023424-550188\classes\local\lito381\server\NativePeerHarness.class` | 存在；pin 相符 | `47A0DB1F3223DB036BEB6B1682CBA4FB8BE40CB3960F1AE39290470242A475FE` |
| 候選 fixture class（不是當版證據） | `D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-recovery-bootstrap-compile\20261024-023424-550188\classes\local\lito381\server\NativeRecoveryProductionBootstrap$Session.class` | 存在；pin 相符 | `E9EF4ED16D33138D9F7729E26C6618604D913BB25ECEFD1D6A13CF2B9BED0716` |
| 候選 fixture class（不是當版證據） | `D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-recovery-bootstrap-compile\20261024-023424-550188\classes\local\lito381\server\NativeRecoveryProductionBootstrap.class` | 存在；pin 相符 | `CA5EB691DB1A549DB6B947143DA3C6E3A32FC6AEC8126CCF1BA0FDDDDCF2DD5D` |
| 候選 fixture class（不是當版證據） | `D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-recovery-bootstrap-compile\20261024-023424-550188\classes\local\lito381\server\NativeRecoveryReadonlySnapshot.class` | 存在；pin 相符 | `FD6CB62D9B8C9A168B9960FC765DD8D32A84AF175E701E2FC6191CF20C11DACA` |
| 候選 fixture class（不是當版證據） | `D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-recovery-bootstrap-compile\20261024-023424-550188\classes\local\lito381\server\NativeTeleportFailureHarness.class` | 存在；pin 相符 | `7D57DBF57DD201985B74F43A4313F22CAAB55E4741A2E6F07CB66F2B0B71EAB0` |
| 盤點來源 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80\AGENTS.md` | 存在 | `30DB838971E0C2BE6ECA629E452D1506A30A77FEC46B3CF8636893EF3520FBA1` |
| 盤點來源 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80\docs\reports\issue-54.md` | 存在 | `23E9630FA6C42F3A921E59E13BBBC7A457E790A4156761D61CBF363071E9EDB7` |
| 盤點來源 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80\維護工具\run-teleport-failure.ps1` | 存在 | `798059F7CBC993F066B0BAC431DBA18566B81E2368425F7DA0FFCF459969E5E3` |
| 盤點來源 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80\維護工具\run-native-E149-recovery-case.py` | 存在 | `76386094552FB64058E71D165CC68D6512AE9FF0793CA598193B09F23DCC1EFB` |
| 盤點來源 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80\維護工具\native_E149_recovery_contract.py` | 存在 | `B8CCC6E69A26AB2069D3F81C6CE388A11BB5B1484B11A7EA29C01C718C471026` |
| 盤點來源 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-80\維護工具\native-E149-owned-mysql.py` | 存在 | `1B4C52A2A808633ECCDD461D1483D4CAD579DC4C274945E81FDF57C0D6E68AA1` |
