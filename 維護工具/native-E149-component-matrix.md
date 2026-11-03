# G06 原版／候選元件矩陣

範圍是 native_login 下兩個現有測試模組，與正式 Java／DB／原 owner 案例分開。原版每版 23 案＝17 個原有控制案＋6 個預期指定斷言失敗案；候選每版 28 案＝17 個原有控制案＋11 個新元件案。11 個元件案加另行處理的 3 個新正式案才是原提案的 14 案，不能把 14 全算成元件案。

prepare-native-E149-component-matrix.py 從固定候選與當時實際 metadata compiler cohort 產生計畫、原版隔離 wrapper、完整 362 模組圖及逐名選擇。候選沿用原 metadata 的全圖；原版僅把 native_login Runtime 換回正常 3E，並使用配對的原版 QA（避免引用候選才有的私有方法）。T鍵／#20等其他候選模組在兩種模式相同。只重寫 path/include 字串，保留函式、cfg、斷言、fixture 及時限，不套用正常來源。

案例清單目前依原始碼中的 test／cfg_attr(test,test) 屬性推得，尚非實際 --list inventory。恢復測試後還需依兩版真正 libtest inventory 核對精確名字、忽略狀態、過濾數與全部選擇。純 controller 狀態與加密 loopback 案有合成前提，不授予正式帳號／30秒自然期限／GUI資格。

compile-native-E149-component-matrix.py 支援 --metadata-only（只編譯原版兩版 library-test metadata）及 --link-only（兩版×原版／候選四份 library-test PE），都不執行輸出。命令沿原 compiler 設定與 13 個直接依賴，x64、Release z／thin／16／strip 維持；連結模式補齊原已核對 native link paths。編譯前後核對 1049 正常輸入、工具鏈、候選與正常兩EXE；建立隱藏低優先序原 compiler，記錄 PID／FILETIME／實際 wait／exit 及原始日誌。遇到活動計時案例或來源凍結不編譯。

本輪 metadata 編譯的實際結果以 qualified-development.json 為準。library-test 的 rmeta 可以是空 marker，exit0只代表編譯／型別通過，不代表產生 PE、執行 test、原版已重現或候選已修好。現有候選全圖 metadata 資格保留原範圍，不把新原版 wrapper 資格移記成正常來源／完整library。

後續需要四份 PE 的獨立連結資格及精確執行／讀回工具：17 個原有控制案必須先通過；6 個預期反例除單案 Rust exit101／普通 panic 外，還須符合固定測試名與指定斷言證據。編譯錯誤、存取違規、連線前提失敗、其他 panic 或 wrapper 非零不能當反例成功。候選28案及所有原案須真正通過。第一個非預期結果保留原輸出並停止，不自動重跑。

使用者仍要求暫緩測試；目前沒有啟動案例、--list、DB、Java 或GUI。完成元件與正式案例後，仍須獨立核對才套用，再做最終正常來源完整library Debug／Release、Release回歸及完整GUI，整體G01–G12未完成。
