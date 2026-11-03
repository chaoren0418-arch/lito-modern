# #23 能量感測與 #36 迷魅術候選來源

依 E1498871 正式 catalog、ClientSession／NativePetMaterialAction／NativePetWorldLifetime、Protocol reader 及原 list.spr 核對；未執行遊戲類別。候選未套用、未選入 registry；metadata 編譯不等於玩法驗收。

|技能|正式請求|本機目標|伺服器權威|
|---|---|---|---|
|23 能量感測|14 02 06＋actor u32 LE；7 bytes，無座標|明確其他存活可見actor、原生畫面內|NPC領域、弱點bitmask、MP8基值、reuse10ms；caster19、target2510及所有weaknessEffects S37按wire接收|
|36 迷魅術|14 04 03＋actor u32 LE＋x/y u16 LE；11 bytes|同上；range=-1不可當無限制|WORLD／quest／timeAttack怪物領域、可迷魅／容量／HP20／MP30／40318×1基值／reuse10ms／3600秒／機率全由E149決定|

客戶端不由name／master／graphics推測NPC模板、弱點或可迷魅性；已知活體與範圍驗證後，正式伺服器判定最終資格。不預扣HP／MP／材料、不造companion或到期，不把caster19／228圖當成功。正式容量拒絕319補入接收記錄及文案；舊訊息不拒絕新intent，來源退役優先。收到動作只記CastAction，可能先於後續容量訊息，並非成功ACK。

成功迷魅沿正式remove原怪物、送新companion object、對companion ID送228；不把原怪物effect重綁到未來相同ID。WORLD與兩種BATTLE分支各自保留正式發布。通用actor接收／incarnation與選取fence沿原來源；完整寵物指令／owner與自然3600秒、傳送退役仍待當版GAME／GUI驗證。

新增嚴格finite profiles：228原7frame、ticks4/2/2/4/4/2/2與Sound132；2166／2167／2168／2169各11frame、source0為subframe10、source1..10為0..9、各4ticks、Sound809／810／811／812在source1，保留原102／104 attrs、沒有Hit。原2510已在既有skill bank按action0讀alias149、單frame／4ticks／無聲、Over混合；action3原兩frame不拿來造施法duration。本輪保留其既有來源。正式earth/fire/water/wind定義映射與SPR英文註解不同，按正式wire圖號顯示，不靠註解改元素或假造疊圖順序。

新增10個待驗案例：9個純資料／生命週期案例與1個明確忽略的原資源案例；後者實際讀原archive、核對所有finite及2510、解SPR與音效準備、不播放裝置。前119案全部未跑，129只表示已寫案數，非完整library inventory。使用者目前使用電腦；測試／--list／GUI／fixture／server／DB／音效播放仍暫緩。

下一步依剩餘124個請求缺口核對正式reader與分支，先查#24／#40的目標／原資源／拒絕路徑；不得沿歷史75估算。G06元件／16案640次、原gfx初始化與LT綁定、當版完整library Debug／Release、Release回歸、完整GUI／乾淨主機交付及G01–G12仍待；自行完成後才交人工。
