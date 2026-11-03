# #75 終極返生術／#76 集體緩速術：來源開發界線

正式 E149 classfile、實際 material callback、完整方法與原 Text 定義已唯讀核對。主證據位於 `驗證紀錄/current/native-E149-greater75-massslow76-review/20261024-171220-650978/review.json`；初次37方法與補充6完整方法分別保留，不載入遊戲 class、不執行原PE。

- #75：C14 `[20,9,2] + u32 corpse` 七bytes，其他可見屍體、native screen，reuse10ms。正式名稱終極返生術；MP100、一寶石、玩家HP全回／NPC四分之一、經驗恢復與施術資格由server判定。S24/322為 `[36,66,1,0]` 四bytes（空字串NUL），普通61的321仍是三bytes。答案C23必須回原question ID與空text；322答案為 `[35,66,1,yes,0]`。
- 321／322共用既有dead-life Session與controller：token保存實際question ID、owner及不回繞命令序號；替換詢問會使舊按鈕失效。同意先consume再送，不能重送；同意後放棄只退役本機等待。recipient不需已學75或MP，仍須原dead life／connection／source／inventory owner。GUI來源依322顯示正式名稱，沿既有非模態同意／拒絕入口；未操作視窗。
- 原3944／1362八方向cast與230／124結果視覺沿既有bank。3944不是成功ACK；新生命、HP與經驗必須等權威封包，不能由音效／特效推定。
- #76：C14 `[20,9,3] + u32 actor + u16 x/y` 十一bytes，其他可見活體、十格，reuse3000ms。MP20／兩寶石、機率與64秒緩速皆server權威；移速沿既有server speed封包。正式body僅成功數大於零時在原主目標送S37/3932，不能改成S39名單或以本機範圍推算命中。
- 3932原13frame、class1、Screen，source2的`<1349`是可重疊音效，無Hit；沿既有typed audio／finite History。模型166ms進音效、2451ms視覺完成，與64秒狀態期限分開。接收位置固定，目標離場不截掉已接受效果；world actor0保留真cast key。同來源重建續播，不重送音效，source退役沿既有admission守衛。

新增16案例（15純／1原資源opt-in）僅寫成來源，全未執行。允許的metadata編譯不連結測試程式，也不代表完整library、GUI或玩法驗證。使用者目前使用電腦，所有測試本文／--list、GUI、server／DB／fixture及播放繼續暫緩。

恢復測試後須核對321／322精確封包、相互替換／舊命令、接受／拒絕／放棄等待、新生命／owner退役、成本／材料／玩家NPC資格／經驗權威；76範圍／機率／反制／緩速抵消與自然64秒、原資源像素／音效／History，以及當版完整library Debug／Release、Release回歸與GUI。整體G01–G12及G06未完成，未交人工。
