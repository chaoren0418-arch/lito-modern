# G06 唯讀取樣的退役邊界補強

目前 canonical observer 的 `stable_observed` 比對了來源、角色、phase、owner 參照、角色清單參照及兩份 inventory 取樣。帳號、`sourceItemDisconnected`、owner 的 volatile `active` 與弱參照內容先後讀取，沒有列入前後一致性檢查；帳號及斷線旗標更是在最後輸出時另讀一次。取樣期間若剛好退役，這不足以支持「這些欄位也前後一致」的解讀。

新候選在 inventory 取樣前保存帳號、斷線旗標、owner.active、owner.session referent、hub.sourceItemApplication 及 application.closed，取樣後把六組欄位逐一比對。JSON 輸出使用同一份已保存的值，避免檢查完成後另讀帳號／斷線狀態。原來源／角色／phase／inventory 比對保留；新增條件只會讓 stable 更嚴格。

另更正 `source_item_application_active`：原式只表示 sourceItemApplication 非空；新式要求 application 非空且 closed==false，並前後核對 application 參照與 closed 值。此旗標仍不代表 `NativePlayerLifetime.admits()` 或 `Owner.requireActive()` 已通過；觀測器不呼叫這些方法。

原 E149 Owner 的 objectId／selected／source／account 及兩個 WeakReference 欄位都是 final，active 是 volatile。新增檢查聚焦可改變的 active／referent 與 session 狀態，不寫任何遊戲欄位，不 refresh inventory，不呼叫遊戲 getter／guard／setter，不建立 owner。沒有新增同步鎖或改動原 lock 順序。

這仍是前後讀值相等的診斷取樣：不是原子快照，不排除 ABA、未觀測的短暫變化或最後一次檢查後的退役，也不授權物品 copy／mutation。新增比較會否在正常切換期間正確排除不穩定樣本，必須恢復測試後核對。

候選沿目前 morph fixture，四份來源只有 NativeRecoveryReadonlySnapshot.java 改變。與 StateCopy 前提 observer 是兩個獨立未選入分支；此候選不含 StateCopy 新欄位。Java-only 編譯並比較六份 class，預期僅 observer class 改變；javap 比對原遊戲方法呼叫集合，確認沒有新增原遊戲 API 或反射欄位寫入。這些只是來源／編譯／位元碼證據，沒有執行 observer、reader、guard、setter、server、DB 或 GUI。

目前 canonical binding、16 案／640 次計畫及所有 PE 保留。下一次更新 fixture 來源綁定時先合併此補強，再重新綁定實際 Java classes／來源；不可沿舊 class SHA 宣告新 observer 已驗。測試與桌面仍暫緩，完整當版 G06／owner／library／Release／GUI 都未完成。
