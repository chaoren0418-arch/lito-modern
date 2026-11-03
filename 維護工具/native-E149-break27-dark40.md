# #27 壞物術與 #40 黑闇之影候選來源

正式 E1498871 的 reader、WORLD／BATTLE 實際方法與原資源已唯讀核對。候選未套用或選入 registry；兩版 metadata 編譯不是玩法或像素驗收。

|技能|C14；11 bytes|本機範圍／reuse|正式基值及伺服器權威|
|---|---|---|---|
|27 壞物術|14 03 02＋actor u32 LE＋x/y u16 LE|指定其他存活可見actor、四格／3000ms|MP32、40318×1、命中機率；PC武器damage／durability及原inventory delta＋S6F延伸描述、NPC／WORLD companion的stack均由E149決定|
|40 黑闇之影|14 04 07＋同上payload|指定其他存活可見actor、三格／500ms|MP15、40318×1、機率、32秒；PC用S0A=1設定blind，清除與到期依伺服器；不由2175特效推定blind|

兩者caster19、目標S37沿原接收與incarnation／target_revision／owner／connection／correction／generation守衛。消耗、目標武器／怪物資格及狀態由伺服器決定；不預扣、不直接改耐久、不計本機32秒，不把動作或影像當命中ACK。#20致盲原746／一格／16秒來源保留，不因共用BlindSpell family改成#40。

172原15frame、subframe14／0..13、ticks5/1/1/1/3/3/3/3/3/3/4/4/4/4/4，Sound100在source2；2175原8frame、subframe7／0..6、4ticks、Sound802在source1，102=0且104=128，無Hit。本輪保留原header／attrs及嚴格銀行資格，沒有替代圖。

2175按原attr128接既有RGB555 Multiply，不沿一般attr8的Screen。原PE parser將104寫647E68、共用action保留到effect+1A、main draw經4CD9D0／483770提交原flags，488DE5選flag80；4C0F40／4C0F60走80D89C／80E09C／80E89C，initializer用非負channel product除31。既有Multiply處理covered black與透明孔洞，非任意opacity。2510仍Over，其餘既有finite維持Screen；未改其他特效／原compositor或旗標選擇器。分析PE不是已證明可執行原版；圖庫初始化／LT基準仍未解，本輪未執行原指令，也未擴張舊像素資格。

新增12個待驗案例（11純資料／接續／合成像素＋1原資源opt-in），連同前129案全部未執行。原資源案會讀archive、解SPR／準備音效，核對172 source2及2175 Multiply實際fragment、退役，不播放裝置；合成像素不是GUI像素。141只表示已寫案例，非完整library inventory或通過數。使用者目前使用電腦，測試／--list／GUI／fixture／server／DB／播放暫緩。

當版180項請求來源識別預期58／剩122，須以發布時實際集合重新核對，不當玩法資格。下一群#41／#45已確認存在正式catalog且仍缺入口：先追reader、死體／目標型別、原畫面／資源／機率及狀態分支再實作，不用靜態Map索引代替資料流。G06、完整library Debug／Release、Release回歸、完整GUI及其餘G01–G12仍待，自行驗證後才交人工。
