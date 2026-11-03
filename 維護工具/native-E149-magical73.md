# #73 創造魔法武器：E149 物品目標候選

更正先前依 `commitSelf`／`admitSelf` 名稱將 #73 說成自用生成技能的判讀。正式 E149 的 `ClientSession.nativeMaterialMagicalWeaponBody` 實際解析 `Protocol.objectSpellRequest`，把 `targetObjectId` 交給 `NativeMagicalWeaponAction.admit`；其 effect 呼叫 `RuntimeInventory.createMagicalWeapon(long)`。提交者是施法者，封包目標仍是背包物品。

原 `RuntimeInventory.createMagicalWeapon` 查找同一 ID 的既有物品，要求 weapon、存在 nativeWeaponStatus、safeEnchant>0、原 enchantLevel==0，成功時只對同一 map key 寫入 `withEnchantLevel(1)`。它沒有建立新 ID 或新增物品。這些是正式 JAR 的方法位元碼判讀，沒有啟動遊戲類別，也沒有實際驗證正式施法。

候選接入既有 typed ItemTarget／背包明確選取／controller.cast_item 路徑，C14 09/00 後接 unsigned 32-bit 物品 ID。沒有 actor／ground／無目標 fallback，不根據名稱、icon 或本機模板推算伺服器資格。沿用 owner／完整 Item／inventory revision／map／InputStamp 檢查，任何換代、傳送、控制中斷及同 bytes ABA 仍需重新選取。

原 metadata：skill73、action19、effect763、MP200、reuse0、material40318×10，訊息685／79／161。MP、材料、可附魔資格及實際 +1 全由伺服器決定，不在客戶端預扣、生成或改寫。既有物品 Pending 79 拒絕處理沿用，685／161 都不當作施法成功 ACK；原動作與 Pending gate 不變。763 嚴格 finite 定義與既有聲音路徑沒有修改；實際資源播放仍待。

四檔差分候選：兩個 Runtime、擴充原八案的 item QA、新增五案 magical73 QA。新版把原「#73 尚未支援」斷言改成不支援 #74，原歷史候選與結果保留。新候選疊到十九檔清單後為二十檔，但不改原 canonical registry、G06 binding／十六案或既有兩個正常／兩個隔離 GUI PE。

編譯只做 Debug／Release metadata，包含整庫與 library-test、#20／#29／#79／物品技能及新 #73 QA；沒有測試執行，也沒有產出新版 GUI PE。既有十九檔 GUI PE 不含這次 #73 變更；不得將它們的建置資格移記這次來源。測試與桌面暫緩維持，恢復後還需當版兩版施法／材料與低MP／無效物品／返回及舊目標／成功與拒絕、原763資源音效、完整library／Release與正常GUI驗證。
