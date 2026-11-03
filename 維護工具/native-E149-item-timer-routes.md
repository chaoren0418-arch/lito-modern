# 正式 E149 計時物品入口：實際指令掃描

只解析固定 JAR bytes 與 javap；沒有載入遊戲類別、啟動 scheduler、呼叫遊戲方法或測試。

新 classfile reader 解析 Code operand width、switch padding／table、wide及branch boundary，不把常數池出現名字當呼叫，也不在原 bytes 裡搜尋可能是 operand 的 opcode。掃描涵蓋2554個server類別、31461個有Code的方法、929761條指令；實際欄位寫入另對選出類別的javap完整清單核對。

七個StateCopy條件欄共有13處顯式寫入，均位於 `NativeDeWorldKinds.Item`：

- `_magicStoneExpireFuture`、`_equipmentTimerFuture`、`_activeEquipmentTimer` 的直接寫入均緊接aconst_null，即清空；沒有藉這些顯式指令建立非空Future／active lease。
- 兩個pending boolean的直接寫入均緊接iconst_0。
- pending generation有constructor／mutation重設為-1，以及 `applyMagicStoneExpiryIfNeeded` 的計算值寫入。
- pending equipment lease有 `terminalizeEquipmentLocked` 傳入參考寫入。

原JAR含74處其他排程呼叫，但Item內沒有直接schedule呼叫；沒有解碼到EquipmentTimerLease constructor呼叫，也沒有 `applyMagicStoneExpiryIfNeeded`、`terminalizeEquipmentLocked` 或 `stopEquipmentTimerSerialized` 的直接caller，所匹配method handle亦為零。這些是固定JAR的顯式指令事實，**不是全域不可達證明**。

通用反射欄位寫入、動態載入／外部呼叫仍未排除。原 `StateCopy.copyFields` 自己使用反射複製，不能當成正常GAME正向建立計時物品的證據；Field.setAccessible只改metadata，不列為遊戲欄位寫入。立即前一指令的常數不是一般性stack/dataflow分析，pending generation／pending lease的資料來源仍需原方法與正常入口證據。

本輪產出逐方法／offset／descriptor／Code SHA與前五指令context、class hash完整清單、明確反射寫入／排程／method handle清單及原javap。靜態前提表七項都維持 `normal_GAME_creation_proven=false`；不能翻旗標、造Future／lease、反射改遊戲欄位或注入時鐘，來把缺入口的案例算通過。

目前StateCopy唯讀候選仍未選入canonical；現行16案仍是原32＋新8，共640次全未執行，不因本次掃描重備或重編。下一步需要正常GAME入口、實際物品／模板來源及原owner授權的正向證據，才補timer拒絕／自然到期驗證；可繼續獨立的剩餘玩法開發。完整owner／Main／G06、library／GUI與G01–G12仍未完成。
