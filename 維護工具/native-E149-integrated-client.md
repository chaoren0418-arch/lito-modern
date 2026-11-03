# G06、T 鍵及技能的隔離正常入口建置

這份清單把目前十四檔 G06 effective overlay 與十檔物品技能候選去重為十九檔，仍未選入正式候選 registry，也未套用正常來源。保留五處重疊的前後路徑與 SHA；致盲 QA 內容相同，其餘四處由已核對的技能延伸來源取代。來源不整包複製，清單引用現有候選的確定版本。

十九檔分為八個 Runtime、八份 Rust QA、三份 Java fixture。Runtime 包含原 T 鍵的 world_raster／native_world_scene、最新技能／目標模型／有限效果及 G06 控制器／正常視窗。保留 G06 取消恢復與 T 鍵來源基底，物品技能與 #20／#29／#79 沿最新十二次 metadata 編譯的版本。Java 沿目前 morph binding；StateCopy observer 的未選入分支不在此清單。

`build-native-E149-integrated-client.py` 對完整 362 模組來源圖逐項核對八個 Runtime。兩份配對來源只改寫原六個巢狀模組路徑；world_raster 另有一處原 fixture include_bytes 的絕對路徑改寫，資料 SHA 必須符合原 1049 輸入清單。來源、副本及資料都綁定 SHA；不以檔名相同推定內容相同。

建置只啟動 rustc／linker，依 Debug、Release 順序各產出 rlib 與正常 `src/main.rs` 的 Windows GUI PE；不使用測試 harness、teleport-wire-test feature 或替代 main。全部輸出限 target/current/verification，未覆寫正常兩個 EXE。保留原依賴、Windows resource.lib、正常 manifest 及版本資源；Release 維持 z／thin／16／strip。低優先序、單工，無網路下載，沒有 cargo build script／遊戲／伺服器／DB 執行。

`qualify-native-E149-integrated-client.py` 只讀原編譯命令、日誌、程序身分與 PE 檔案：核對四個實際編譯退出、rlib archive、AMD64／PE32+／Windows GUI subsystem、RT_MANIFEST 原 bytes 與正常版本資源。PE 不載入、不執行，連 `--help`／`--list` 都不啟動。保留原建置準備失敗與來源，不重寫先前資格。

八份 QA／三份 Java 僅保留在合併清單，這四個命令沒有編譯它們；Java 與舊 controller 映像繼續保留各自原資格。新正常 PE 沒有任何歷史 runtime、G06、API、音效或 GUI 資格，不能替換目前十六案計畫的舊 PE。

使用者目前要求測試與桌面操作暫緩。恢復後先完成既定四元件、G06 十六案及原資格核對，再安排合併來源的 T 鍵、技能／背包／取消與正常 GUI。正式套用須更新來源身分，重新完成該版完整 library Debug／Release、必要資源、Release 回歸及完整 GUI；全部自行確認後才交人工驗收。
