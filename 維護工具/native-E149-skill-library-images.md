# 最新二十一檔候選：正常入口與 library harness 隔離連結

本輪使用 #107 metadata cohort 的完整362模組與同一份二十一檔來源投影，包含 G06／T鍵與 #12／#21／#20／#29／#73／#79／#107。八個 Runtime 逐項對回原候選 bytes；兩份 controller／window 配對來源只保留原六個巢狀模組絕對路徑，world_raster 保留一處原 fixture include_bytes 路徑。正常1049輸入及候選／wrapper／依賴全部綁定 SHA。

資格另存完整候選輸入清單：以正常1049輸入覆蓋／加入二十一檔，記錄清單檔 SHA。這是邏輯來源投影；實際 Rust cfg、harness 選擇與實體 wrapper 路徑仍以原編譯命令及逐檔 SHA 為準，並非宣告清單內每個 QA／Java 都進入每個 PE。

Debug、Release 各三個命令：一般 rlib、正常 src/main.rs GUI PE，以及 src/lib.rs 的完整 library-test harness PE。後者使用現行 feature 設定及 `--test` 連結所有 library 的 cfg(test) 模組；不開 teleport-wire-test feature。這輪並未連結八個獨立 integration target；其中六個 skill target 保留先前 metadata 資格，T鍵及 formal controller 仍需按自身來源範圍處理。十份 QA 來源含兩份 library 內模組與八份獨立 integration 檔，不能把它們都算進 library harness。

所有輸出限 target/current/verification，保留正常兩 EXE 與前一批十九檔 GUI PE。使用現有離線依賴、Windows resource.lib、原 manifest／版本資源；Release 維持 z／thin／16／strip，單工、低優先序、隱藏編譯工具視窗。不啟動 Cargo build script、遊戲、server 或 DB。

只讀核對 rlib archive、AMD64／PE32+、正常 GUI subsystem2／library harness console subsystem3，並核對原 manifest bytes 及同版正常資源。沒有 LoadLibrary、執行 PE、`--help`、`--list`、測試本文、GUI、音效播放或執行授權負向測試。

四份 PE 都沒有 runtime 資格。library 測試 inventory、名稱與數量尚未從實際映像列出，不能拿歷史1975過或新版metadata案例數代替；新 GUI PE 也不能沿舊 controller／API／GUI 結果。三份 Java 只保留目前 G06 已選來源上下文，沒有在此重編／執行，G06十六案640筆計畫不變。

使用者恢復測試後先完成既定元件及 G06，再按實際來源／映像列出並執行 library Debug／Release、獨立整合及必要資源，保留忽略項及首個失敗。候選套用正常來源後還要重建並驗正式當版 library／Release／完整 GUI，全部自行確認後才交人工。測試與桌面操作目前仍暫緩，整體未完成。
