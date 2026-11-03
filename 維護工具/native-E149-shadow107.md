# #107 暗影之牙：物品施法與原2951效果候選

正式 E149 的 ItemWeaponEnchantSpell 與 NativeItemWeaponCastAction 明確支援 skill107。封包是 row13／column2，即 `14 0D 02` 後接 unsigned 32-bit 背包物品 ID，透過原 objectSpellRequest 處理。原 commitSelf／admitSelf 指施法者提交，不把目標改成自身 actor。

候選沿 #12／#21／#73 的 typed ItemTarget、背包明確選取與 controller.cast_item，補 #107 選單及 reuse0。MP30 是原 metadata 基值，材料40321×1的原靜態物品名稱為「二級黑魔石」；192秒、melee／ranged各+5、sourceType2／nativeId4、無效79、成功訊息0與到期308，都由正式伺服器決定。不在客戶端預扣 MP／材料、推算武器資格、改能力值或自行計時到期；原 Pending／身分及動作 gate 保留。

2951 嚴格定義取自原 Text/list.spr，SHA523C61AD，header `#2951 1 shadow fang`，13影格、24fps、image0，subframe依12、0至11，ticks為4／2／2／2／2／3／6／5／4／4／4／4／4，只有第二個影格帶 Sound1510；attrs102.type(0)及104.attr(8)。完整原header／attrs／影格／聲音事件逐項比對後才接 finite bank；沿既有 SuppressWhileSameIdPlaying 及世代退役路徑。這是原資源定義接線，尚未播放或驗證實體音效。

五檔差分：三個 Runtime、擴充原八案的 item QA、新增六案 Shadow Fang QA。以 #73 的二十檔投影為基底，換入目前已選定的三份 G06 Java 來源，再新增 #107 QA，合二十一檔來源投影。原六份 Java class 沿各自來源資格，本輪沒有重編／執行 Java。G06 十六案／640筆計畫與正常來源、所有 PE 保持原狀。

編譯範圍是兩版各 library／library-test／六個 skill 整合，共16個 metadata 命令。新增六案與原31案都未執行；沒有連結含 #107 的新正常入口 PE，不移記十九檔 GUI PE 或舊 runtime 資格。

恢復測試後仍需正式已學技能書、正常 GUI 明確選物品、成功／低MP／材料不足／無效武器／更新與ABA／傳送及取消／重施放與原192秒期限、2951圖像與Sound1510音效，以及當版完整 library Debug／Release、Release回歸與完整GUI。測試與桌面暫緩維持，整體未完成。
