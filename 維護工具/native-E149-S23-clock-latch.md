# S23 class4／冰箭原生回呼時序來源

使用者仍暫緩測試與桌面。只有來源開發、唯讀原指令／資源核對及metadata編譯；沒有測試、--list、GUI、server／DB／fixture、播放或新PE。

現行S23的167→219、1583→218、1799→1800及1797→1798，改接獨立NativeSpellClock。原424540／42F300均在首步立即移動，之後60ms回呼一次；延遲只前進一格，從實際dispatch排下一次。每次movement透過4E5B50遞增一個source frame並循環，不按catalog的24Hz ticks補跑或停在source0。class4折成四heading bank，冰箭class5保留八方向。

抵達時間取實際移動回呼，不用path.impact_at的理想wall time提前release。class4先進飛行幀再start secondary（4246CD→42473A）；class5順序相反（42F4CB→42F500）。相鄰#15為97→99、#7為719→711、冰箭為713→719；#4 secondary沒有音效。冰箭空target不造secondary。secondary使用既有native finite Clip，保留source0的獨立停留，218／219為41ms、1800為166ms、1798為125ms。Hit只作既有presentation receipt，不新增HP／MP權威回呼，也不在secondary Hit再hurt一次。

所有取圖走exact source-index API，draw不推動時鐘。class4 explosion凍結在flight endpoint，冰箭在arrival解析同incarnation target當前座標後凍結；既有target／caster／generation／history及來源退役admission守衛保留。原public rational API保留供既有inspection用途；本輪不移記它的舊runtime資格到新callback來源。

S23音效改用PreparedCue原生群組與callback occurrence，保持真實graphics/source slot/cast sequence；同source1的不同movement可重入，同occurrence由history及shared latch去重。沿scope及same-ID suppress政策，沒有播放器／裝置呼叫。FiniteHistory提供仍active的S23 sequences給WorldView及PendingRender，ring／book hint淘汰後不提早清除去重鍵；保留4096原上限，不提高預算或降低64 active／256格呈現上限來湊通過。

共享latch修補：原FiniteDeliveryHistory對重複事件一律discard，若duplicate與尚待播放的canonical是同一Arc，會一起消耗canonical。改用4096上限內的Weak canonical identity：相同payload只移除重複排程，不消耗原latch；不同payload同key仍discard；Weak不保有bank／PCM，expired identity仍以原key拒絕重播。新增三個純metadata／silent sink案例，沒有音效裝置。初版18metadata成功僅留來源／cohort與日誌，不移記修正版。

五Runtime差分（assets、projectiles、scene、view、audio），沿28檔成29檔：十五Runtime／十一Rust QA／三Java。新增七個純時序、三個pending latch／弱引用案例及一個原資源四profile／重建／PCM metadata／memory draw案例，全部未跑；前97個書面案例保留。Debug／Release各library、library-test與七skill integration共18metadata，只記編譯範圍。

正常1049輸入／所有PE、registry、G06 binding及16案640次不變；請求識別仍52／180、缺128。原圖庫base初始化矛盾及LT相對hurt家族仍未綁定；新callback僅依已有函式片段／資源契約開發，不能代表分析PE整體可執行或當版施法／GUI通過。

待驗：恢復測試後先依總報告元件／G06，再當版完整library雙版及必要資源；明確執行新opt-in並核對late／adjacent／bank／無target／死人／incarnation／重建／source ABA／raw ring淘汰與duplicate audio；補實際E149及正常GUI／pixels／physical audio、效能與長時間故障。64個最長路徑與其他finite事件共存的4096 key預算及G11尚未資格化，不能用編譯或記憶體draw替代。
