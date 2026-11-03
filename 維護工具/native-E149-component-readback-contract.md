# G06 元件精確結果與連結讀回契約

目前已寫完 libtest 純解析、四映像連結核對、完整矩陣 reader、原handle／Job producer及診斷檔保護來源；六個模組僅做 AST／來源檢查。沒有呼叫 parser／image qualifier／matrix reader／producer，也沒有建立DiagnosticStore、啟動PE、--list、案例、Job、DB或GUI。四份元件PE尚未連結，本契約不是功能通過結果。使用者已要求測試及桌面暫緩；producer執行入口的第一個動作會讀取當前測試授權，未恢復時不得建立工作目錄或程序；本輪沒有實際呼叫這個執行閘門。

四映像資格只接受 compile-native-E149-component-matrix.py --link-only 的實際原 cohort：Debug／Release×baseline／candidate，原四個 rustc --test --emit=link 命令 exit0、全部來源／依賴／工具鏈／正常EXE雜湊一致、原compiler終止及四個 x64 PE32+ header／SHA。編譯控制檔另存不可變副本，原 cohort／plan 不修改。qualify-native-E149-component-images.py 只讀取檔案與 CIM，不執行映像；空 rmeta 不能代替此資格。

未來 producer 必須在使用者恢復測試後取得四份真正的 --list 原輸出及 exit0。native_E149_libtest_readback.py 驗證每筆 test 名稱、無重複與實際總數，不沿用2463等歷史固定數。兩個選定模組必須恰好包含原版23／候選28個計畫名稱，兩版各自其他 library test inventory 必須相同。--list不等於執行test body。

每版先跑baseline的17個原有控制，再跑6個指定反例；接著candidate的17個控制與11個新增元件。所有106個PE程序（4次inventory＋102次單案）都須先暫停建立、加入私有Job並記錄原PID／FILETIME／映像／命令，才放行；最後 Job=0、全部原handle wait及獨立CIM觀察終止，不能有未記錄的子程序。執行參數固定 `test_name --exact --nocapture --test-threads=1`，不送 --ignored；清除繼承的測試選項，固定三個Rust backtrace／thread環境值。

單案必須是 running 1 test，精確名稱結果、實際 inventory−1 過濾數，零忽略／零benchmark。正常通過需exit0／1 passed；反例需普通Rust panic exit101／1 failed、只有該測試執行緒的一個panic、固定原版來源檔與指定assert行號，以及完整指定訊息。一般失敗、wrong panic、crash、編譯錯或wrapper非零不授反例資格。第三個wire缺角色反例的預期位置是既有pump期限斷言，還須出現Failed／缺角色原因；這是合成loopback的三秒驅動上限，不是正式stage/source自然期限證據。

既有wire控制案使用固定編譯時 CARGO_MANIFEST_DIR，會寫入 target/current/verification/native-teleport-wire/Debug／Release 下的診斷JSON；改cwd不能隔離。native_E149_component_diagnostics.py先封存兩目錄原JSON／SHA／metadata，每案先確認原handle／Job終止，才封存完整after archive與所有新增／bytes或mtime改變的JSON。每目錄限512個單層、單hardlink的JSON，單檔8MiB、合計32MiB；拒絕reparse、未知檔案／子目錄／屬性或並行變更。既有檔從原記憶體快照寫入同一重建目錄的暫存檔、核對SHA及回復原mtime後逐檔原子替換；新檔保存核對後才逐檔移除，沒有遞迴清理。每案及最終原bytes／mtime須完全回復。所有路徑必須位於已核實的重建目錄。

reader獨立核對兩份before archive、每個程序的after archive／metadata／精確delta、變更JSON、restoration receipt與最終原目錄bytes／mtime；--list也要保存回復receipt且delta為空。若使用者中途再暫緩，只回收已登記原handle／私有Job及處理既有診斷交易，不啟動下一案。部分封存／回復失敗不自動第二次嘗試；原archive與輸出保留，checkpoint標記diagnostic_recovery_pending，阻止另一批次。只有原Job=0才解除來源凍結。這些保護目前都尚未實跑，不能宣稱資料回復驗證成功。

未來原始 execution 使用 E149-component-matrix-execution-v1／original_matrix_processes_terminal_results_unreviewed，固定 image qualification／plan／tool source review SHA、四次inventories、依序102筆cases、每筆原process／stdout／stderr、diagnostic_changes／diagnostic_after_snapshot／diagnostic_restoration_manifest及restore_verified、兩份diagnostics_before archive／files／metadata、job_accounting_final、全evidence_pins與未強制回收／輸入不變標記。欄位細節以 verify-native-E149-component-matrix.py 為準。source review必須標記component_producer_source_complete與diagnostic_preservation_source_complete=true；來源完成仍需真正四PE資格及使用者恢復測試後，才可開始執行。工具只產出未獨立審查的原結果，不能自行改記通過。

未來全部核對後的有限結果為90個正向通過與12個指定原版assert失敗，涵蓋兩版的控制及新增元件；它不是整個library綠燈。3個新增正式案與八分支兩版16案suite另驗；完整G06／owner／Main／library／Release／GUI及G01–G12仍待。
