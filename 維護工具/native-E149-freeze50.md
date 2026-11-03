# E149 #50 冰矛圍籬開發候選

僅來源與metadata編譯；測試、--list、PE、遊戲類別、GUI與音效播放全部未執行。未套用正常來源或選入registry。

- 正式E149 FreezeAttackSpell字面定義：row6/column1、action18、gfx756、MP100、reuse500ms、client clock1000ms、range6、基本16秒。正式getter可能覆寫，沒有執行GAME觀測。C14 `14 06 01`＋角色u32LE＋x/y u16LE，共11bytes；使用其他可見活體、六格Chebyshev，保留Target incarnation/revision與Owner connection/generation/corrections。成本、機率、傷害、冰凍與解除皆由server權威。
- 原classfile BootstrapMethods綁定WORLD兩個原lambda及BATTLE實際repair020兩lambda。完整原方法與Protocol parser另存primary。原S23 wire type0；raw756有8圖、class4→219、attr8；class4方向折四bank。756 source3/0/1/2各2ticks，219為11frame。兩者都沒有Sound/Effect/Hit，明確不借用光箭97。
- loader與NativeSpellClock新增嚴格756分支；音效group改Option，無音效profile的SoundBank為空，沒有偽造Sound0或載入fallback WAV。舊四profile每個原音效保留；完整graph及七整合目標只編譯型別，所有既有測試本文未跑。
- S23 caster Hit啟動，立即一步、每60ms一次、晚poll不補跑；沿class4 target座標在launch固定、secondary219與source0 terminal dwell41ms。756 arrival僅診斷；原LT hurt相對gfx／base+26與完整guard仍未綁定，不走既有generic Impact作替代。16秒狀態不由視覺或arrival產生。
- 活動class4/5加入seen保留集合，raw ring被淘汰再回入時不重建第二條飛行。重建資源明確失敗時將該flight標finished、omitted一次，draw移除；錯誤不無限保留audio history。原64cast／32音效／512fragment／4096key預算不改；完整G11未驗。
- 新12案（11純＋1原資源opt-in）僅寫入並編譯，保留前202案，共214案全未執行；不是完整library inventory或通過數。新原資源案涵蓋四bank、空音效bank、memory fragments、ring eviction去重、同來源重建、退役／generation與失敗準備；沒有播放器或GUI。
- 本輪四Runtime差分仍33 overlays（19 Runtime／11 Rust QA／3 Java）、365 top-level module與原六nested保留。正常1049輸入及所有PE、正式JAR、G06 binding／16案640次、registry不改。

下一組缺口#51／#55；#51已確認正式summonSpellRequest使用selection欄位，需獨立召喚選項UI／owner守衛／controller路徑，不能當actor ID或selector-only。召喚容量、模板、材料、30HP/MP與1800秒生命週期仍以正式GAME結果為準。其餘G01–G12、當版完整library Debug／Release、Release回歸與完整GUI待使用者明確恢復測試後執行；未交人工。
