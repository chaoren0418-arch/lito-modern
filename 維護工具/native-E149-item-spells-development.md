# #12／#21 物品施法候選

未選入、未套用正常來源。沿未選入#20／#29／#79，並保留現行T鍵視窗與G06取消／回復控制器的程式基底；六個Runtime檔、四份QA來源共十檔。獨立合併／驗證後才可選為正常來源；原十三檔registry、G06 Java binding及十六案不變。

固定E149原record及handler已讀回：#12 C14 `14 01 03 inventory-u32-LE`、#21 `14 02 04 inventory-u32-LE`，兩者base MP20、reuse10ms、duration1800s。物品資格、修改值、有效MP消耗及期限由server判定。invalidItem79、success161、expiration308是不同語意；#73是自行創造武器的另一入口，沒有把它誤接成背包目標。

`native_skills::ItemTarget` 將背包物件與角色ID分域，保存完整原Item、inventory owner／revision、map及InputStamp。任何物品更新（包含相同bytes的remove/readd）、owner／選角／連線變更、地圖／世界／角色生命週期、校正／控制中斷都拒絕舊選擇。client只要求現有已學技能與有效自身狀態，不按名稱／icon推測裝備資格或預扣MP／預改物品。

視窗只在實際背包row選擇通知時捕捉施法物品；一般refresh不重新授權，失效時清除選擇要求重選。背包世界頁顯示技能選單與施法鈕，調整清單／狀態位置避免重疊；地面拾取頁維持原布局。背包內只啟用這兩個物品技能，其他角色／地面技能仍走原流程。controller送出前再次核對所有權威前提、畫面就緒與互斥動作，Pending與原重用／動作完成gate共用；額外拒絕正在進行的物品／拾取。

物品施法的invalid79只在該domain的Pending被判拒絕；普通技能不新增79判定，success161也不假造施法或物品成功ACK。仍以真正S20／inventory／MP等回覆更新，不在client計時1800秒。

原唯讀解碼list.spr雜湊523C61AD與正常參照一致。只抽747／748原row metadata，加入嚴格additional finite bank：747共19source frames、ticks4、Sound113；748共20frames、ticks3、Sound137；播放仍按原1..n-1,0有限callback。原header、attrs104(8)、SPR frame與音效mode保留，不新增任意gfx fallback，不播放WAV。

新增八個QA案例程式，保留原#20／#29／#79十八案；涵蓋封包分域、owner／revision／ABA、場景／控制退役、server成本／期限、選單、79與非成功ACK、原資源metadata／有限影格音效。**全部未執行。** GUI可見性／布局只做來源審查，不宣告像素或實體操作通過。

先前準備失敗81340d（取樣呼叫anchor）、62d265（既有CRLF）、df6276（舊generated內部module path改寫）均封存，未啟動Rust或改正常來源。第一批metadata原b78f7e actual exit1：Debug前五命令0，新QA因誤用private module退出1，Release未啟動；原cohort／來源／日誌保留。修正QA只用既有public流程，同時補背包頁技能控制與布局後另建候選，不能沿用舊五項為新來源資格。

最新編譯為完整362模組六Runtime覆蓋，另保留控制器五個與視窗一個原nested module path；配對QA只改fixture include路徑。兩版各library／library-test／四份skill整合共十二項metadata，不連結／執行PE，不跑--list／server／DB／GUI。原6路徑及失敗來源均由獨立qualifier核對。

恢復測試後仍先依原G06順序，再跑本候選的原資源與正式E149背包選擇、武器／防具資格、低MP／invalid79、重施放／到期、item delta、舊選擇／傳送／取消與GUI。原16案不移記本客戶端候選；合併後新當版完整library Debug／Release、Release回歸及完整GUI仍待，G01–G12未完成。
