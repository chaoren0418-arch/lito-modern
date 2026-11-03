2026-10-24 現行追加規格：fixture binding v3、case plan／execution v4、suite plan v3；沿已編譯43da2d的Java六class，11 Rust來源與原兩版controller相同。原八setter32筆與新setItem／setItemId八筆分開保留trace、marker與SHA，每案合計40次，兩版16案共640次均未執行。producer／reader／suite只完成來源／AST及檔案計畫；使用者測試／桌面暫緩仍有效。完整規格見 [模板／ID整合](native-E149-morph-fixture-binding.md)；以下v1／v2、舊v3與32-only說明保留階段記錄，schema與範圍以本追加規格及實作為準。

# E149 G06 自動驗證：目前是開發階段

使用者已要求暫緩測試。本組已有準備工具、獨立結果 reader、驗證專用觀測確認點及 Job／handle 執行工具初稿，尚未實跑。Debug／Release 隔離 rlib／正式案例 PE 已實際連結並獨立核對；沒有執行資料庫、SQL、Java factory、客戶端或 GUI，也未套用正常來源。型別／語法與來源審查均不代表案例通過。

`prepare-native-E149-recovery-case.py --profile Debug|Release --scenario <case>` 只寫入唯讀依賴的雜湊與 v3 準備計畫。原 13 檔候選與新整合 4 檔有 3 檔重疊，去重為 **14 檔**。新計畫分開固定客戶端 Rust 連結資格與 Java fixture 編譯資格；原 v1／v2 計畫保留歷史證據，不能用於本輪執行。source-ready 不是執行授權，producer 第一個動作仍檢查使用者暫緩旗標，且明確要求 `--execute-owned-tests`。

`current_E149_fixture_binding` 指向只供未來案例使用的 `E149-fixture-source-binding-v1`。11 個 Rust 檔須與原 Debug／Release 連結時逐 byte 相同；3 個 Java context 中僅允許指定 Java 檔的差異，另核對新 javac 原始輸入／命令／exit0 與 6 class SHA。原連結資格的 14 檔 map、原來源路徑及 cohort 原樣保留；同內容複製的 Rust 路徑可以不同，內容 SHA 必須相同。新增 binding 不重編或重標舊客戶端映像，也不證明新 Java 與客戶端實際相容。producer 只接受目前登記的 fixture binding，execution 與 reader 逐項核對同一 binding／Java source／class／client-source context。

八個案例各使用單一精確 Rust test：timeout、refused、wrong-service、cancel-retry、cancel-expiry、cancel-repeat、refused-cancel-repeat、cancel-expiry-source。GUI 或 cancel-late 另行開發；不混入這個控制器範圍。

## 新增觀測確認點

客戶端在真正收到接續選角與背包後，以測試驅動寫 `a/b-continuation-observe-request`，內容是帳號、角色與測試乙。伺服器沿原只讀 observer 取樣，確認實際 **LOBBY**、相符身分／來源、active owner 與穩定觀察，把原 JSON 行同時寫入 trace 及 `a/b-continuation-observed.json`，客戶端才加入下一個租裝房。這只同步驗證程式，不改伺服器場景、封包、owner、時鐘或租期；八秒僅是驅動等待上限。客戶端 `character_selected` 與伺服器 `LOBBY` 是不同狀態名稱，已用固定 E149 enum 位元碼核對。

## 尚未實跑的 producer 初稿

`run-native-E149-recovery-case.py` 與 `native_E149_owned_process.py` 已具備來源初稿；仍須完成靜態審查、重新產生計畫，之後在使用者恢復測試時驗證真實執行。每次啟動前及等待期間都讀取暫緩旗標；原使用者授權是準繩，不能因自動續行自行開測。producer 不自動執行 reader，不自動重試失敗案例。

每批使用無名稱、不可繼承的 Windows Job，設定 KILL_ON_JOB_CLOSE，禁止 breakaway。工作子程序經 CreateProcess(CREATE_SUSPENDED) 建立，先核對映像／FILETIME、加入 Job 並保存原始身分，才 ResumeThread；不使用先跑再掛入 Job 的時序。DB 子程序的紀錄還須核對 datadir、映像與完整命令，採用活體 handle 時另驗證其確為該 Job 成員。正常退出先要求私有 DB shutdown，再查 Job ActiveProcesses=0；失敗才終止自己持有的程序或 Job 成員。assign/resume 失敗即回收尚未放行的子程序，保留原始錯誤，不放寬限制。Job 與 handle 流程尚未實跑。

API 設計依據：[Microsoft Job Object 指派與繼承規則](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject)、[limit flags](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information)、[extended limit 結構](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information)。直接 `_winapi.CreateProcess` 參數順序另以本機 CPython 3.12 subprocess.py 核對；保留 primary thread handle 供加入 Job 後放行。

producer 必須先完成／核對獨立連結映像，才啟動有期限的 DB producer，等待真正匯入完成，帶實際 owner／import proof 與固定 JDBC 啟動 fixture，最後執行單一精確 Rust test。所有子程序需記錄原 PID、FILETIME ticks、整數 epoch millis、映像 SHA、完整命令與實際 exit／wait；持有原 daemon handle 至回收完畢。失敗保留原始輸出，依序回收自己建立的 controller、fixture、DB；不得按名稱、單獨 PID 或逾時猜測身分。

native_E149_recovery_suite.py 可在使用者恢復測試後序列化八案×兩版：每案沿原 producer 執行並回收，再由獨立 reader 核對，首個失敗就停止、不重試。single-case producer 回傳原 execution 路徑，例外附帶原 output 路徑；suite 不另建立外層 Job。`--prepare` 僅寫16份計畫索引，執行仍需現行暫緩旗標為false與明確執行參數；細節見 native-E149-recovery-suite.md。此16案不代替完整library、原版反例／候選元件案或GUI。

Java 命令由 reader 固定：UTF-8、個案 tmpdir、停用 PerfData、tw13081901-v1／extendedItemDetails／B64ED920 catalog、Xverify:all、三段 classpath（6 個已編譯 fixture class 的目錄、E149 JAR、Connector/J），再帶 root、scenario、兩份實際 proof 路徑及 SHA、固定 SQL 和 `--execute-owned-tests`。移除繼承的 JAVA_TOOL_OPTIONS／JDK_JAVA_OPTIONS／_JAVA_OPTIONS。controller 僅可用 testName、--exact、--ignored、--nocapture，固定三個 LITO_TELEPORT 環境變數，不要求真人登入。

## 連結映像證據格式

reader 要求獨立資格檔 `status=qualified_linked_isolated_E149_recovery_controller`，包含 profile、原來源 SHA、原 14 檔有效覆蓋、product／SHA、`full_source_and_overlay_pins_verified=true`、兩個實際 compiler exit0 與原編譯身分終止核對，以及 compile_cohort／SHA。cohort 使用 `schema=E149-recovery-linked-build-v1`、`status=linked_compilers_exited`，包含 1049 筆基底輸入清單路徑、原 14 檔覆蓋及全部 input_pins、依序 library／formal_test 兩份實際 rustc --emit=link 命令、exit、原始 stdout／stderr SHA。正式案例命令仍固定當時的來源路徑，另與現行計畫核對相同 Rust bytes。兩版已有此限定連結資格；建置時的可變工具／文件版本另存不可變 snapshot，原觀測 cohort 原樣保存。Java-only context 另經 fixture binding 綁定；這不授予任何執行結果。

## 原始案例 execution 格式

未來 producer 寫入 `target/current/verification/native-E149-recovery-cases/<case>/execution.json`，只可用 `schema=E149-recovery-execution-v3`、`status=owned_processes_exited_results_unreviewed`。必要欄位由 reader 原始碼定義，包含 plan／SHA、profile／scenario、runtime、linked_image_qualification／SHA、client_source_binding、fixture_source_binding／SHA、fixture_qualification／SHA、Java_source_pins／Java_classes、controller_product_sha256、inputs_unchanged、forced_cleanup_used、runtime_producer_pins、runtime_producer_source_reviewed、完整 evidence_pins、controller_stdout 與 database_original_handle_signalled。

processes 是攤平的原始程序紀錄，必須含 database_producer、database_bootstrap、database_daemon、fixture、controller，及每個 mysql_client。每列包含 kind、pid、creation_filetime_ticks、start_epoch_millis、command、executable_sha256、exit_code、wait_confirmed；直接建立的三種主工作程序另存 created_suspended／job_assigned_before_resume。controller 存確實使用的 environment，fixture 存 inherited_java_options_removed。execution 必須存 owned_job_kill_on_close 與最後真正的 Job accounting（active_processes=0）；不得以 state JSON 本身取代活體或原 handle 終止觀測。

evidence_pins 必須覆蓋所有實際被 reader 消費的結果、ready、DB 最終 state、兩份 proof、trace、staged trace、兩名玩家結果與觀測確認點、原始 capture、controller stdout、連結資格及 cohort，以及新 fixture binding／編譯資格。proof 內合成密碼不輸出到終端；原始失敗、退出及回收結果不可被 reader 改寫。

## reader 的限定驗證範圍

`verify-native-E149-recovery-case.py <execution.json>` 僅讀取證據與 OS 程序身分，成功後另寫 `qualified-selected-case.json`，不啟動任何遊戲或測試。它要求真正的 19 表／6277 列匯入、原資源 factory 成功、僅靜態 SQL／禁止玩家 SQL、所有原程序實際退出，並逐角色核對連線 epoch、來源／角色、舊世界退役、owner 身分／背包 owner、實際移動與退出後 owner 退役。

owner 的 `source` 實際是來源限定的角色識別，`account` 是來源限定的帳號識別，不能與 root.source 的「測試乙」或裸帳號直接比較。依固定 E149 `Owner` constructor → ClientSession.sourceCharacterIdentity／sourceAccountIdentity → GameStateStore.sourceCharacter 的原始位元碼：兩個成分先做 Java String.trim（僅 UTF-16 code unit <= U+0020），拒絕空字串或內含 U+001F，然後以 U+001F 串接；owner 建構時不轉小寫。reader 對同一 root 來源／選角／帳號重建期望值；原觀察欄仍保留原樣。新計畫固定 `owner_identity_contract`／SHA 及原方法 excerpt；先前誤比裸值的 reader／計畫已保留歷史範圍，尚未執行，不能當通過或產品失敗。

新計畫／execution／ready／result 同時固定 `owner_guard_probe_kind=G06_E149_owner_guard_v1`。producer 固定 owner-guard-trace.jsonl 的 SHA；reader 以純解析模組核對兩筆接續正向控制、一或兩筆外來 inventory 拒絕與四筆實際 GAME owner 退役，逐一連回原 server trace、continuation ACK 與實際 owner／session／inventory observer ID。四個原 Future 退出後、fixture close 前才檢查退役；此順序另由固定 Java 控制流程約束，不推論完整 mutation／來源／帳號／timer 拒絕矩陣。細節見 native-E149-owner-guard-readback.md；所有新檢查仍未實跑。

本輪 fixture binding v2／case plan與execution v3另要求 `retired_item_mutation_probe_kind=G06_E149_retired_bound_item_mutation_v1`、八個固定方法、四個owner target及每案32次attempt。producer會核對ready marker並保存item-mutation-denial-trace.jsonl／SHA；reader確認ready／result／plan／execution一致，先固定全部raw，再呼叫純reader逐案核對原trace角色／實際actor／不同值／精確stale-owner例外與前後觀測。trace限64MiB。少案、空檔、錯誤原因或未保存raw不能授此範圍資格。九個Python模組僅來源／AST核對，沒有執行runtime／parser；舊版本與原映像資格仍保留，不沿用舊計畫繞過新探測。這32次是每個fixture案例的計畫數；完整非預設屬性／來源帳號／timer與Main仍待。

背包逐欄比較既有 **17 個觀察欄位**，包含 RuntimeInventory 投影及所選原始屬性；S7C 另解碼每個物品的 ID、use type、charge、icon、blessing、count、identified、MS950 name、完整 opaque detail bytes。未到期的 staged 背包須原 ID／欄位／封包保持；自然到期後不得再授予舊 ID，重新租裝使用新 ID 並保持其餘觀察欄位與 wire metadata。33 秒與另外 31 秒仍須實際等待，不能改時鐘。

ready／result 均須記錄原 `SharedClientResources.initialize()` 已呼叫，以及 `G06_E149_native_item_fields_v1`。reader 另讀每個物品的 native_attributes sidecar，驗證 ID／schema／穩定觀察／型別；未到期同 ID staged 接續逐一比較 8 項永久欄（元素、鑲卡、魔石、覺醒）。cached ability 與 14 項診斷只觀察、不要求 timer／generation／owner 診斷不變、不觸發重算。自然到期案例改驗舊 ID 不再授予，不能把新租裝的時間戳強行當不變值。各階段分別報告非預設覆蓋，完整屬性矩陣旗標始終 false；正常玩法建立非預設狀態、效果、拒絕與期限矩陣仍待。

CaptureLog 沒有每 session 的 wire ID，因此 C8 數量／測試乙值是整體封包證據，另結合逐帳號的 client／server 狀態；不假稱每 TCP session 的完整歸屬。穩定讀取不是原子授權快照，dark-item payload／timer lease 及完整物品 UI 仍未涵蓋。此單案 reader 不授予完整 Main、完整 owner 生命週期、全部 G06／library／Release／GUI 或整體目標資格。
