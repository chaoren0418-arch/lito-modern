# #80 冰雪颶風／#87 衝擊之暈：請求與有限特效來源

正式 E149 完整方法／classfile、原Text與3933原start靜態路徑已唯讀核對。來源證據為 `驗證紀錄/current/native-E149-freezing80-shock87-review/20261024-172623-343534/review.json`；45完整方法、17class資料與補充physical wire均保留，不執行遊戲class或PE。

- #80：C14 `[20,9,7]` 三bytes，無指定目標，自己周圍十格、reuse3000ms。MP60、傷害／機率／32秒冰凍及目標名單由server判定。正式S39 action18／type0／3933，以wire caster位置固定，原start走單節點預設路徑。31原幀、Screen、source1 `<1353`可重疊音效，無Hit；模型0ms音效／5440ms視覺，不作冰凍期限。S25／freeze176物件仍依server資料與既有生命週期，無本機狀態或傷害預測。
- #87：C14 `[20,10,6] + u32 actor + u16 x/y` 十一bytes，其他可見活體、一格，authoritative walk family須50／54，reuse8000ms。MP15、傷害／機率／2–4秒眩暈由server判定。S23 physicalDamageAttack與S37/4434先於狀態成功判定，不能當眩暈成功ACK。4434六原幀、Screen、source2 Sound1498；模型41ms音效／246ms視覺。
- 3933與4434已接typed world音效、真cast key、接收定點、History與原來源退役守衛。重建不重放已發布音效；3933重備失敗只退役一次。原64 casts／32 audio／512 fragments限制保留，沒有G11資格聲明。

**87尚未完成持續狀態物件4183呈現。** 正式S0B生成獨立物件（gfx4183、walk0），S25控制與S08移除仍由server授權；原4183只有六幀action3、source1 `<1453`。現有stationary actor可解析action3，但其`animation`標籤會被一般idle/breath守衛拒絕，且沒有可核對的重複overlap1453 callback／History路徑。不得把有限4434當持續狀態，也不得本機到2–4秒自行刪物件。下一步優先綁原S0B構造／action3循環時序，再接每個incarnation的typed音效／History／S08退役，之後才續88／89。

接續可先讀 `維護工具/prepare-native-actor-green-clock-original.py` 與 `audit-native-actor-green-clock.py` 指向的舊 water-clock／actor-green-clock 原片段封存，核對 phase／FPS／keep callback 與本輪原PE身分。舊工具綁1019輸入，不能直接重跑或把歷史向量資格移記4183；目前僅讀取其來源定位證據，原時序仍待綁定。

第一次唯讀review工具錯把stance0解析視為無條件，守衛於候選寫出前退出1；原來源與失敗已封存於 `驗證紀錄/current/native-E149-freezing80-shock87-review/20261024-172550-874458/`。v2依實際stationary分支修正資料判讀，沒有執行測試或重啟遊戲。

新增13案例（11純／2原資源opt-in）僅寫成來源，全未執行。metadata編譯不連結或執行測試，並非完整library／玩法／GUI通過。使用者目前使用電腦，測試本文／--list、GUI、server／DB／fixture與播放繼續暫緩。

恢復測試後，須核對成本／機率／拒絕／低MP／兩種walk family／原資源像素與音效、實際HP／冰凍及自然32秒／眩暈自然期限、4183移除及傳送退役、同來源History／重備取消與ring去重，再依總報告完成當版完整library Debug／Release、Release與完整GUI。整體G01–G12及G06仍未完成。
