2026-10-24 現行追加規格：fixture binding v3、case plan／execution v4、suite plan v3；沿已編譯43da2d的Java六class，11 Rust來源與原兩版controller相同。原八setter32筆與新setItem／setItemId八筆分開保留trace、marker與SHA，每案合計40次，兩版16案共640次均未執行。producer／reader／suite只完成來源／AST及檔案計畫；使用者測試／桌面暫緩仍有效。完整規格見 [模板／ID整合](native-E149-morph-fixture-binding.md)；以下v1／v2、舊v3與32-only說明保留階段記錄，schema與範圍以本追加規格及實作為準。

# E149 共用初始化與物品屬性觀測候選

目前完成靜態資源讀回、兩個 Java 檔修改與 javac，並接入 v2 producer／reader／準備計畫的來源契約；沒有呼叫初始化、SQL、案例或 GUI。新的 fixture source binding 分別固定 Java 編譯證據與逐 byte 相同的 11 個 Rust 輸入；舊兩版 Rust 映像及其資格原樣保留，不新增實跑資格。

`SharedResourceMain` 原入口先呼叫 `SharedClientResources.initialize()`，再進 Main。`initialize()` 本體輸出 profile；JVM clinit 先跑 initializeProfile／loadDurations：驗證 B64ED920 catalog SHA／header，讀取 JAR 內 `battle/spr-duration-tw13081901.csv`，要求 60303 個不重複有效項並 Map.copyOf。這段未發現 SQL／網路呼叫；不推論完整 Main 初始化已通過。

新 fixture 候選在正式 JAR／proof 路徑檢查後、port reservation／ServerConfig／GameHub 前，驗證固定 profile、重建目錄內的 catalog 與 initializer code-source JAR，再呼叫原 initialize；不替換 static 欄位或建造資源 snapshot。ready/result 新增 shared_client_resources_initialized 與 native_item_attribute_observer_kind，仍須實跑及獨立 reader 核對，不能只看布林值授予資格。

原物品 JSON 的 17 欄保持不變。inventory_state 新增 native_attributes sidecar，每筆包含 ID、兩次讀取穩定標記，以及以下分區：

| 分區 | 資料 | 比較範圍 |
|---|---|---|
| permanent（8 欄） | 元素種類／等級、greater、完整鑲卡陣列、魔石 ID／等級／時間、覺醒 ID 清單 | 尚未到期的同 ID staged 背包接續須保持；新租裝與自然到期的語意另驗 |
| cached_abilities | 複製原 EnumMap 後讀取已有的能力鍵／值 | 僅記錄；不呼叫重新計算或完整性修復 getter |
| diagnostics（14 欄） | actor 觀測 ID、item ID、count revision、remainingTime、owner／角色 ID、equip window、temp equipped、ability dirty、lighting／match、dark presence、魔石／equipment generation | 不當作跨 owner／copy 的不變值 |

只用限定 private field 讀取、JDK atomic.get、陣列／ArrayList 副本、EnumMap clone；Timestamp 先 clone 再讀 epoch/nanos，避免 Date getter 對原物件的日曆快取正規化。反射欄位 descriptor 放在有上限的 observer 自有 cache，只處理 local.lito381.server 類別階層，避開 JDK module 私有欄位。沒有 Field.set、遊戲方法、inventory.items()、ability audit/recompute 或 timer 回呼。

觀察仍不是原子授權快照。部分 JDK backing collection view 可有自身的懶快取，與原生 Item.view 刷新不同；不宣稱零 JVM 內部快取活動。診斷時間／generation 可正常變化；不能把這些變化判成接續遺失。dark-item payload、timer lease、非預設狀態及拒絕矩陣仍未涵蓋。

`native_E149_item_attributes.py` 的純資料解析／staged 比較初稿已接入 reader 原始碼，六個 Python 模組只做 AST／來源審查，reader 及比較函式尚未執行。它分別報告元素、鑲卡、魔石、覺醒等非零觀測覆蓋；全零／空值只能證明預設路徑，即使比較相同也不得宣告完整屬性矩陣或 owner 合格。非預設案例必須由正常遊戲流程產生，不用反射改欄位、偽造 owner 或縮短時鐘。

編譯後新唯讀路徑已審查；v2 計畫將 fixture binding、Java source／class、原客戶端映像與目前 Rust byte 對照分別固定。接續開發正常玩法的非預設狀態／效果／拒絕／期限案例與完整 owner 核對；Java-only 變更不代表重編 Rust 或通過功能。使用者恢復測試後才跑 Job／DB／JDBC／原案例、非預設狀態、自然期限與退出回收。

後續靜態追查 2554 個 server class 的 setter methodref（含以子類別為 owner 的繼承呼叫）、相關 fieldref 與欄位名稱字串，目前只找到 Item.setItem → setGreater 的明確 setter 呼叫；其餘非預設元素／鑲卡／魔石／覺醒尚無可證明的正常 GAME 請求產生路徑。這不排除通用反射、外部 caller 或執行期路由。原 StateCopy 會深複製永久資料，卻拒絕 active ScheduledFuture／EquipmentTimerLease 與 pending magic-stone expiry，因此這些必須另列為拒絕／期限案例，不能強行當成功 staged 複製。先驗可到達的既有恢復流程，同時保留非預設前提為待補；不以直接 setter／反射造值授予正常玩法資格。
