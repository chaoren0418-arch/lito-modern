# E149 隔離靜態資料庫工具

目前只完成開發與靜態檢查。使用者已要求暫緩測試；不可啟動資料庫、執行 SQL、原資源 factory 或 GUI。工具尚未實跑，不代表 MySQL 5.5／JDBC／中文路徑相容，亦不是完整 Main 或遊戲驗收。

`prepare-native-E149-mysql-toolchain.py` 僅將現有 x64 MySQL 5.5.22 的三個程式、vendor 原始初始化 SQL、charset／errmsg 複製到重建版固定快取，並補齊原固定 JAR 的 Connector/J 8.0.33。沒有執行程式、複製既有資料目錄或 my.ini；原始 vendor 檔頭保留。

`native-E149-owned-mysql.py plan --bundle <bundle.json> --static <manifest.json>` 只產生計畫、核對檔案雜湊。計畫不是啟動／匯入／READY 證據，沒有密碼、datadir 或預占埠。原始碼或依賴改變後須建立新計畫。

日後使用者恢復測試後，才可使用 `serve --plan <plan.json> --execute`。入口先檢查工作佇列的暫緩旗標與正式 E149 雜湊；旗標不能取代使用者授權。這是前景的有期限工具程序；由未占用的 CLI 啟動並保留原始程序身分，不需視窗或真人登入。

- 僅用重建目錄的新 datadir 與 tmpdir，`--no-defaults` 第一參數，vendor bootstrap 加隨機合成 root 密碼。排除繼承的 MYSQL 環境變數；不連正式資料庫、不讀正式帳密或玩家表。
- OS 配發 127.0.0.1 高埠；啟動前才釋放預占 socket。每次 mysql／mysqladmin 命令前核對原 PID、FILETIME 出生時間、執行檔 SHA、完整命令／datadir／bind 與真正 listener owner。不憑連線成功或逾時推定身分。
- SQL 經 stdin 傳入；client 專用 defaults-file 第一參數且固定 TCP。保留原 19 表匯入 SQL；匯入必須 exit0，再查實際表名與各表列數，核對 19 表／6277 列。陳述式 6296 是固定輸入的 CREATE／INSERT 數，不是獨立伺服器稽核計數。
- 僅授予 `lg06_<nonce8>` 對該次 schema 的 SELECT，schema grant 逸出底線萬用字元並讀回權限。成功匯入後才寫實際 `static-import.properties`／`database-owner.properties`；其中 epoch millis 用整數 FILETIME 換算，與 Java adapter 契約一致。密碼只供本次隔離實例，不輸出到終端。
- 正常停止、期限到達或暫緩旗標恢復時，用核對過的 mysqladmin shutdown。失敗才終止原 Popen handle；不按名稱或單獨 PID 殺程序。所有原始錯誤與日誌保留，沒有遞迴刪除。

`status --plan ...` 只讀實際存活身分；`stop --plan ...` 僅建立相符 nonce／PID／出生時間的停止請求，不當作已終止。若 producer 意外消失，日後明確執行 `recover --plan ... --execute`，再次核對身分並持有原 Windows process handle，才嘗試 shutdown／必要的同 handle 回收。無法確認身分就停止，不操作其他程序；原結果與復原紀錄分開保存。

後續仍需把 Java bootstrap adapter 接入 G06 fixture，使用其原 lifetime accept／submit／Future／cleanup。只能在實際 JDBC 原 factory 啟動後記錄 static SQL 使用；player SQL 始終禁止。接著才實跑原版 6 反例、候選 14 案與原既有案、33／31 秒自然到期、最終當版 Debug／Release library 與完整 GUI。任何步驟失敗都保留原始證據，未完成前不交人工。
