# 原生 #29 緩速術開發候選

候選延伸現有 #20 的兩個施法／選單來源，新增 #29 與精確 752 特效；保留原六個 #20 案例，另寫六個 #29 案例。新候選沒有選入現有13檔registry，也沒有套用正常產品。既有G06原映像、四元件PE與16份正式計畫保留原範圍；不得把它們當作本候選的runtime資格。

現行E149的dispatcher → useEntangleSpell → NativeNegativeMaterialAction.entangle → nativeMaterialEntangleBody已核對原位元碼。#29的額外int是伺服器自己的percentile fixture配置，不在C14封包內；客戶端不設定它。C14為14／03／04、目標u32 LE與目前x/y u16 LE；目標須是仍可見的另一個角色／物件，八格Chebyshev內。原server另檢查地圖／遮擋／native screen，材料、有效MP成本、機率、64秒狀態與haste抵銷均由原server決定。本地只保留500ms重用限制與既有pending／角色生命週期檢查，不預扣或預測成功。

752的原定義為24fps、16個影格、每格3ticks，依原subframe 7,0,1,2,3,4,5,6,7,0,1,2,3,4,5,6保存；原播放時序含兩輪八個子影格與terminal dwell，無音效事件。候選加入既有精確finite bank，不放寬其他sprite/header/action/attribute/blank條件，也不另造持續64秒的畫面迴圈。完整SPR解碼、實際影格／像素、播放與聲音仍待測試；新增752也要驗整個additional bank，避免破壞原效果。

Debug／Release metadata範圍：完整library、library-test、原#20整合程式及新#29整合程式；--emit=metadata不連結或執行PE，test rmeta為空只代表compiler接受。首輪路徑準備誤把已封存的生成複本當原候選路徑，未啟動rustc；核對內容與pin後修正。接著兩項Debug編譯成功，但整合因metadata檔名不符lib*規則而退出1；原來源／cohort／日誌保留。接續只複製原library為正確名稱並核對SHA，沿用兩項成功結果，另做剩餘六個compiler；不刪除或改寫原失敗。

六個新案例包含精確封包／八方向邊界、不可用目標與caster狀態、選單與舊target／connection／技能書失效、低MP／缺材料／失敗回覆、S_SPEED緩速與解除、752原影格／有限播放／退役。它們尚未執行，純狀態案例也不是正式施放或GUI證據。恢復測試後須實跑元件／原資源與現行E149技能書、自然機率成功／失敗、低MP、材料缺少／原子消耗、haste抵銷、角色／連線退役與GUI，再獨立合併到現行候選、重建及完整library／Release／GUI回歸。

#79靈魂昇華仍列未完成：已核對其object-target request與native screen，但3935原資料含Effect(1356)及attr109(0 -1)。子特效及控制欄位須追溯原繪製語意後才接入，不能丟棄或改成一般純音效finite。這不縮減完整G01–G12目標；正常來源仍38個已識別ID，#20與#29的未套用候選最多提供40個請求分支，不等於40技能完成。
