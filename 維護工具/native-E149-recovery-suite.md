2026-10-24 現行追加規格：fixture binding v3、case plan／execution v4、suite plan v3；沿已編譯43da2d的Java六class，11 Rust來源與原兩版controller相同。原八setter32筆與新setItem／setItemId八筆分開保留trace、marker與SHA，每案合計40次，兩版16案共640次均未執行。producer／reader／suite只完成來源／AST及檔案計畫；使用者測試／桌面暫緩仍有效。完整規格見 [模板／ID整合](native-E149-morph-fixture-binding.md)；以下v1／v2、舊v3與32-only說明保留階段記錄，schema與範圍以本追加規格及實作為準。

# E149 精確恢復案例序列工具

native_E149_recovery_suite.py 只涵蓋現有八個精確恢復案例 × Debug／Release，共 16 個隔離 controller 案例。這不是完整 library、原版反例／候選元件矩陣、GUI、全部 G06 或完整 G01–G12。Runtime 主程式／正常 EXE 均不由本工具更新。

目前只有原始碼、AST 與純計畫準備，沒有呼叫 execute_suite、producer、reader、guard、Job、DB 或案例。使用者仍要求暫緩測試。`--prepare` 只產生固定順序的 16 份 member plan 索引，不載入 producer／reader；schema、來源、JAR、fixture binding、工具 review 與每份計畫 SHA 必須相同。

未來執行須明確提供 `--plan <suite-plan> --execute-owned-tests`，execute_suite 第一個動作讀取現行使用者暫緩旗標、活動批次與來源凍結；暫緩時不載入 runtime 模組、建立 Job 或工作程序。每一案開始前重新核對授權與來源 SHA。真正的子程序仍由原 single-case producer 暫停建立、核對身分、加入該案私有 Job 才放行；沒有另一個外層工作程序或 Job，因此沒有新增巢狀 Job 層級。

同一 Python pipeline 先呼叫原 producer，等待該案的原 handle／Job／DB 回收完畢，再呼叫獨立 reader。reader 逐一核對原始 execution、wire、owner guard、物品與退出證據，另寫該案 qualified-selected-case.json；只有該案核對成功才開始下一案。這個序列工具會接續 reader，但 single-case producer 本身仍不自動呼叫 reader。

本輪 suite plan v2 固定新的 fixture binding v2、case plan／execution v3與九模組來源。每個member都必須有八個原BoundItem setter×四個真實退役owner的32次精確拒絕及前後狀態觀測；16案計畫共512次，全部尚未執行。member不得換回舊Java或省略mutation trace／reader，也不能以guard-only紀錄代替。原Rust映像的11個輸入及兩版連結資格不改寫，Java編譯與當前來源另綁。計畫準備及AST不建立Job、不操作桌面、不執行任何案例。

首個 producer 失敗、reader 拒絕、來源變更或使用者重新暫緩，皆立即停止，不自動重試或跳過。原始 case output、失敗 traceback、suite.json、已核對前綴與尚未開始案例全部保留。producer 例外附帶原 case output 路徑，避免依目錄名稱猜測新批次；未釋放的原活動批次／來源凍結也不會被 suite checkpoint 清空。

只有原 16 案全部經 reader 核對，才可記錄 qualified_selected_case_matrix；該狀態始終帶 full_library_qualified=false、GUI_verified=false、完整 Main／owner／整體 false。terminal checkpoint 依最新八檔 SHA 先封存、再寫入及讀回；不移記舊來源／JAR 資格。此整套序列控制、失敗停止、使用者暫緩中斷與 terminal checkpoint 仍需恢復測試後的實際案例證據。
