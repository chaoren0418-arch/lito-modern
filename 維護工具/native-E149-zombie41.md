# #41 造屍術：屍體意圖與可見像素選取候選

E1498871 catalog、ClientSession／NativePetMaterialAction／NativePetWorldLifetime 與 Protocol 原reader已唯讀核對。`14 05 00`＋屍體actor u32 LE共7bytes，沒有座標；range=-1仍檢查原生畫面。MP5／HP80／40318×1／reuse10ms／petCost6為正式基值，屍體模板／唯一領域、可用性、消耗、容量與companion建立都由伺服器決定；本機只驗收到的dead actor與來源，不由名稱或圖號猜NPC模板。

Target新增已捕捉的屍體death action sequence；滑鼠原pick可捕捉dead actor，#41專用resolve才允許屍體。活體選取不會在死亡後自動取得造屍授權；屍體選取不能在復活後變成一般活體授權；death action／incarnation／target_revision／generation改變都失效。Model原connection／caster／correction守衛保留。一般攻擊與其他技能仍用活體resolve。

T鍵在選中且仍學會#41時，從實際已呈現的像素owner plane循環屍體；其他模式保留原活體循環。InputSurface仍核對原authority revision／movement stamp／camera／environment key、blind=0、非觀察者、own alive及實際可見像素，排除地面物品與minimap覆蓋，不從roster／距離猜被遮住的屍體。Window只傳當下有效selected skill，無新增桌面自動輸入。

正式caster19、容量319與226效果不是造出同伴的ACK。原body即使容量不足仍可對屍體送226；成功才consume對應WORLD／quest／timeAttack corpse、remove原object、publish companion object與master。原226保留7frame／ticks4/2/2/4/4/2/2／source1 Sound139／104=8，沿既有S37固定接收座標、獨立finite生命週期；屍體被消耗不應把已接收影像改綁新ID或提早裁掉。本輪不修改該既有finite實作、不預扣或本機消耗屍體／生成pet。

Window原唯一nested rental module採相同絕對路徑配對後編譯；須重建並核對sole rewrite，保留controller原五條nested測試與此一rental路徑，不能把舊wrapper的編譯結果移記新raw source。完整362模組及兩版library／library-test／七既有整合目標只做metadata。

新增14案（13純資料／接續／合成像素＋1原資源opt-in），連同前141案全部待執行。合成Raster像素只驗owner-plane選取，非正常GUI像素；原資源案核對226解圖／音效準備與屍體消耗後固定座標，沒有裝置播放。155只是已寫案數，不是完整library inventory／通過數。候選未套用／選入registry；使用者目前使用電腦，測試／--list／GUI／fixture／server／DB／播放仍暫緩。

下一步先接#45：E149原請求與S23／S39、129四bank原direction fold、shadow130與class7 callback／terminal source0／原音效及hurt時序需完整綁定。既有type7靜態分析可依相同PE與當版raw重新核對，不能直接轉用TW或舊75資格，也不能拿一般finite單圖代替。其後依剩餘121個請求缺口與G06／G01–G12逐項完成；測試恢復後先元件／G06，再當版完整library Debug／Release、Release回歸與完整GUI，自行核對後才交人工。
