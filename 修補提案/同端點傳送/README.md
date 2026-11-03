# 同端點傳送：隔離參考核心行為對照

本資料只保存在重建目錄，供重建版開發時比較既有核心的行為。重建版與登入器是不同專案、產物獨立；先前詢問替換登入器 JAR 的部署提案已撤回。

以下 6928A147／B33F9FA1 記錄僅為舊版本隔離歷史對照。當版重建來源44FEA738／唯讀參考75DA69B0的Debug同端點實測亦未通過：新GAME接續後，頻道請求收到租裝選擇回覆，缺後續頻道人數資料而逾時。見[當版失敗邊界](../../驗證紀錄/current/native44FE-same-endpoint/20261018-171204-385221-debug/independent-boundary/boundary.json)。舊C02邊界仍保留原版本資格；本資料不提出登入器替換或部署要求。

當時正式參考 6928A147 的同位址新 GAME 已自動接續角色／租裝，但 `rentalItemSelectionContext()` 在暫存被消耗后重新攔截 C_CA，誤作租裝重選，無法进入正常頻道。強迫重租會取代已接續裝備，不能當傳送成功。

候選在該 helper 入口補 `if (rentalBattleLoadoutActive) return false`，其餘原有條件保留。僅 ClientSession.class 中一個方法變更，其他 2149 方法的實際指令／exception regions 一致。來源 SHA：6928A147DF1E784304B9080632C56121B173BF60B172023EBD263E82FD448628；候選 SHA：B33F9FA1D3173337A7BA8AC18872ADF4C037033D68F3266442C05C86A74896CC。

已在 same-endpoint 與 cross-endpoint 各兩名玩家、6條實際 session（共12）驗證角色／租裝物件14與11不變、舊畫面輸入失效、新 epoch／socket、共同戰鬥與完整回收。其設定、captures、被動權威及原始來源 pins 見 [候選收據](../../驗證紀錄/current/native-world-integration/same-endpoint-server-candidate.json)。JVM -Xverify:all 通過；Debug／Release各7向量通過；Release候選同端點再6sessions與正式6928跨端點再6sessions、角色／租裝／世代／共享戰鬥及獨立capture比對通過。主Release D1B27339由唯一入口建置。這不是正式來源的同端點驗收，也不涵蓋GUI、取消、過期、其他來源或全產品。

候選檔位於 `LiTo381-Modern/target/current/server-reference/same-endpoint-candidate-6928.jar`。相鄰登入器維持唯讀。此候選不是重建版交付產物，也不以登入器更新作開發或完成依賴。後續傳送取消／逾期／重入實作與驗證均在重建版進行。
