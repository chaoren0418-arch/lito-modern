# 水下動畫與像素合成的重建版實作

本功能屬於重建版的新 Windows x64 核心。登入器只作唯讀協定與原行為參考；不替換登入器 JAR，也不依賴登入器部署。

2026-10-16，1002 份編譯輸入已由唯一入口建置正常 Debug `BF88BA78`、Release `694B7EC6`。兩版各 18 份實際測試 EXE、92 過、0 敗、0 忽略；來源、測試與正常主程式封存均獨立讀回。完整雜湊及逐項收據見 [正常主與來源資格](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/actual-tests-20261016-084451-903086/normal-qualified/qualified.json) 與 [限定測試資格](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/actual-tests-20261016-084451-903086/normal-qualified/scoped-qualified.json)。本版 TW+extended 十四案傳送 GUI 已逐案及整體獨立核對：112 sessions、220 移動零多送、80 原世界 PNG、56 glyph、128 舊通知；850 份 raw ZIP `028105FC` 逐檔讀回，原 handle57589 exit0、pipeline103656 及所有 owned 負載退場。僅192DPI私有桌面訊息操作；[當版完成觀察](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/actual-tests-20261016-084451-903086/normal-qualified/GUI-completion-observation.json)不等同 OS SendInput、水下 GUI 或完整遊戲驗收。999 輸入結果另留原版資格。

Rust 排程器移植原水下 phase、counter、有序 deadline queue 與 scroll 的數值行為。測試固定資料含 8656 組 phase、9668 組 scroll，以及 15 組原生隔離 queue 情境；產品沒有執行原 x86 程式碼。每輪 dispatch 只取一次截止時間，回呼重新讀時鐘排下一筆，避免晚到時補跑全部逾期動畫。S28 開啟可重複排程；關閉取消水下回呼。地圖載入會先清除舊水下回呼再重排，並確保 map aux 存在。詳見 [原時鐘隔離範圍](NATIVE-WATER-CLOCK.zh-TW.md)。

回呼與視覺歷史綁定 GAME epoch。舊連線不能更新新場景；離場取消 water 與 map aux，保留原數值 phase/counter/scroll，重登入另綁新 owner。相機取同一個實際內插 Pose，在乾地也累積負相機增量；跨非連續動作或地圖校正不補造移動。完成的非同步場景工作才可帶回同 GAME 的視覺歷史。phase 是繪製取樣值，沒有放進資源工作的 authority key，避免每 100ms 的動畫使載入工作持續作廢。

Model 在地形與角色合成、原光照 mask 之後套用原 RGB555 水下色表及 shade.dat；姓名、徽章與小地圖隨後繪製。實際測試核對 800×600 像素、phase 改變、alpha 保留、小地圖白色標記、乾地行走後 scroll 與新 Model 的同 GAME 歷史交接。map303 及特殊 global gate 仍未完成。

[產品 library 診斷](../驗證紀錄/current/native-water-scene-preview/20261016-084319-059886/receipt.json) 已輸出乾地及水下 phase0/1 的原 RGBA 與 PNG，並核對像素雜湊、封存讀回及目視地形／角色／小地圖／文字順序。這批直接使用已測產品 library 與手動水下狀態，0 條新 GAME sessions；不能宣稱正式伺服器的實際水下封包或正常 GUI 水下畫面已驗證。

首輪場景測試誤用固定 1000ms，與資源載入後的真實排程起點不符而失敗；已保留原失敗產物，改用真正 pending deadline，未刪除 phase 與像素差異斷言。診斷工具兩次連結失敗亦保留，再補 host procMacro 與原 Cargo native library 搜尋路徑才通過。完整 default/library/TCP、實體輸入、多 DPI、完整原 framebuffer、可攜交付、穩定性及 G01–G12 仍待。
