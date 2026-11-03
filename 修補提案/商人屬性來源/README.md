# 商人屬性來源：歷史隔離行為對照

重建版與登入器／伺服器為獨立專案。本目錄保存舊BBC59308的脫離連線計算對照，不提出替換／部署外部JAR的要求，也不以它作重建版開發或完成依賴。客戶端仍遵循實際權威封包，不補加未收到的角色屬性。

現行C02原始JAR的78項來源缺口已另以原類別確認，僅脫離連線計算、0遊戲sessions，未驗實際TW／ClientSession或屬性GUI。見[當前唯讀邊界](../../驗證紀錄/current/native-current-merchant-bonus-audit/qualified.json)。以下段落保留舊候選的歷史驗證與當時尚待範圍，不是目前部署提案。


此目錄是待審查候選，未修改或部署正式伺服器。正式 BBC59308 JAR 的缺口仍存在；客戶端不預測或補加屬性。

實測租用並穿戴 29047 火蜥蝪紋樣戒指時，正式 S52 火抗仍是 0。正式 JAR 內的 `retained-lito-rental-merchant-2010.tsv` 明列火抗 10，但 `RentalEquipmentBonuses` 只查 `RentalItemDefinitionCatalog`，78 個商人商品 ID 均找不到來源。

候選只改 `RentalEquipmentBonuses.java`。保留原有 3.80 配裝來源優先次序，缺項才查已通過雜湊檢查的商人 catalog，從其完整 `NativeArmorStatus` 讀取額外屬性。未穿戴與來源綁定裝備仍排除；原生 AC 與武器命中／傷害維持原有計算路徑。保留資料沒有提供的效果明確為零，未知欄位拒絕處理，不將保留的 2010 來源冒稱為官方原始資料。

`merchant-bonuses.patch` 是單檔差異，`source-provenance.json` 固定唯讀原始檔與正式 JAR 身份。`MerchantBonusRepairOracle.java` 只建立脫離連線的不可變測試物件，沒有資料庫、玩家 session 或時間修改。

`review-validation.json` 記錄已執行的獨立比對：57 個原有配裝結果不變；78 個商人商品逐欄對照 JAR 內 TSV；270 個未穿戴／來源綁定案例為零；共 405 個 catalog 狀態與 35 個集合欄位核對。候選的戒指火抗為 10，原正式結果為 0。

尚待修補後正式 ClientSession 的穿戴／卸除屬性封包、自然到期、跨服務交接及伺服器完整回歸。這些純計算測試不能替代上述驗收，亦不構成新正式來源的相容認證。相鄰伺服器目錄依專案規則保持唯讀，本目錄沒有自動部署指令。
