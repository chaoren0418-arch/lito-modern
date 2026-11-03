# E149 範圍技能與原火球資源鏈開發

本批先補#25所需原171→218 class5載入器；未開啟四技能施法／S39接收／GUI入口，當版請求識別仍48、132項缺口不減。

正式E149的AreaAttackSpell與ClientSession確認：#22／#25／#30為attackSpellRequest，後帶角色ID與座標；#53的maximumRange0、SELECTOR_ONLY，走周圍施法。基值#22為MP20／reuse300／range6／gfx1804，#25為20／500／6／171，#30為24／500／8／1805，#53為40／1500／0／758。所有傷害、MP、角色／範圍／遮擋與對象判定仍在伺服器；ResourceDatabase可能覆寫基值，未執行其設定或遊戲方法。

回覆走Protocol.attackMany S39。原PE4271D92F靜態派送0x4A6510..0x4A6868保留全target list，取首目標座標建立一組common effect／class5；不逐受擊者重播整個特效。此靜態路徑尚須逐欄與時序核實，不代表GUI或完整S39生命周期完成。

- 新增明確load_area_fireball_effect_only；只允許原171 class5／secondary218。flight五frame依序sub4,0,1,2,3，各1tick；Sound98在source1。
- 171八方向全數載入／選圖，不沿class4折成四方向。218沿原十一frame、Sound99與terminal dwell；不添加原圖沒有的Hit，也不假造傷害回呼。
- 原冰箭1797／1798與class4三條鏈保留，sound及bank不混用；沿既有worker loading、取消、budget、reader失敗清除及音效bank所有權。
- 一Runtime檔差分，內含五個待執行案例，其中兩個明示原資源前提；疊成23檔：九Runtime／十一Rust QA／三Java。與前輪七技能整合一起做兩版18次metadata編譯，不連結／執行PE，不執行測試、Java、GUI或播放。

下一步接S39一組效果、全受擊列表及第一目標anchor：#22 class1／原Sound714與Hit；#30 class1須包含1806 shadow、1807 clothes、Sound718與兩個Hit；#53原758／Sound125／Hit。先核實start offset、目標身分／世代與受擊回呼、零目標、首目標退役／缺席、同一sequence去重、切場景／Model接續，再接請求與選單。171→218的載入完成不能代替這些步驟。後續當版技能缺口盤點須綁實際新來源；舊22／21檔PE與ZIP不含本批。

使用者仍暫緩測試與桌面；G06為優先主線，binding／16案640次與原registry不變。恢復後先元件／G06，再當版library Debug／Release、Release回歸與完整GUI，自行確認後才交人工。
