# G06 模板／ID 拒絕分支整合

本輪將已編譯的真實模板／ID候選選為**未來案例**的 fixture。只做來源／AST與純檔案準備，沒有啟動 Job、DB、SQL、server、controller，也沒有執行 guard、setter、reader 或 GUI。測試與桌面暫緩仍有效。

現行 schema：fixture binding v3、case plan v4、execution v4、suite plan v3。舊 binding／plan 不由新版工具靜默轉換或補欄位；舊來源及資格已先封存。測試恢復時須用新計畫，執行入口仍先核對目前使用者授權與登記的 binding。

原 Java 43da2d actual exit0／六 class 沿原資格；本輪 Java／Rust 未改、不重編。新 binding 同時綁新 morph 來源／classes、原八 setter 編譯資格及原 owner guard 資格。原兩版 controller 的十一 Rust 輸入逐 byte 相同，僅兩個 Java context 不同；不重標原映像的功能／相容性資格。

每個案例固定四個實際 owner admission。原 `item-mutation-denial-trace.jsonl` 仍需八方法×四物件＝三十二筆；新增 `item-morph-denial-trace.jsonl` 需兩方法×四物件＝八筆。每案四十次，兩版八分支共十六案規劃六百四十次，實際零次。

producer 檢查新 ready 的 probe kind、兩個方法、四個原 owner，再保留兩條 trace 的完整檔案與 SHA。outer reader 同時檢查 plan／execution／ready／result 的相同 scope、來源及 classes；兩條 trace 各維持 64MiB讀取上限，檔案缺失或範圍不符即失敗，不補造或重試。

outer reader 呼叫新純 reader；內部先完整核對原三十二筆，再核對新增八筆。每對 morph 連回同一原 actor／entry／store／owner／session、原 source-qualified 身分、原三十二筆最後觀測、同一正向 capture 內兩個真實物件及不同模板／ID，且要求精確 stale-owner 拒絕與前後完整觀測不變。參數驗證、detached／staged、同值、錯誤型別與意外成功均不合格。

suite 仍依 Debug／Release ×八個原恢復分支串行、首個失敗即停、零自動重試。原 handle／FILETIME／Job／DB 退出屏障與自然 33／31秒等待不變。計畫產生與 AST 不會啟動這些程序或等待。

此整合不證明四個 inventory 確有兩個模板，也不證明 reader 或真實拒絕可運作。正常 GAME 非預設擴充屬性、source／account 個別分支、active timer lease／自然到期、Main、完整 owner／G06、library／GUI仍待。後續使用者恢復測試後，先依元件反例／G06計畫核對原行為，再驗技能候選與合併新正常來源，完成新當版完整 library Debug／Release、Release 回歸及 GUI，才能交人工。
