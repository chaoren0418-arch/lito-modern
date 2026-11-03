# E149 #55狂暴術／#56疾病術開發候選

僅來源與metadata編譯；測試、--list、GUI、PE、Java遊戲類別／DB及播放全部未執行。正常來源未套用，registry未選入。

- #55 CombatBuffSpell：C14 `14 06 06`＋角色u32 ID共七bytes，未選目標時明確自用；傳入失效ID不改自用。原native screen不當無限range；正式分支另檢查同非空血盟或隊伍、WORLD map／instance或BATTLE room、活體／observer／隱形及action支援。客戶端不從名字推測這些資格；solo自用亦可能被原server靜默拒絕。MP40、reuse10、320秒、AC+10／近戰命中+2／傷害+5皆server權威與基值，無本機狀態預測。
- #56 WeaknessSpell走原NativeWeaknessMaterialAction與battle/world Body、實際Bootstrap predicates；C14 `14 06 07`＋角色ID／x/y共11bytes，其他可見活體、三格／reuse500基值。MP10、40318×1、64秒、骰10／boss5、近戰傷害-6／AC+12、lawful-10與280拒絕都由server決定，不造成功ACK。
- 原raw3943為八方向／27frame／attr8／effect0；Sound1361在source2，不在source1；整數逐callback視覺2829ms。原raw2230為單方向十一frame／Sound839在source1／attr128 Multiply，視覺1375ms。視覺時長不代替320／64秒狀態。
- 原S37 4ADDA0在接收時把actor+11h heading傳入44EDE0，constructor複製到effect+11h，4E5C00讀同欄選圖。本輪World另存受ring128上限管制的3943 heading provenance；不修改公開Effect結構。Active保存heading並隨metadata History接續；後續轉身、移動、離場不改已開始效果，S53位置型明確heading0。bank額外允許方向載入，但qualifier只放行3943 EightWay／image_count8，所有既有profile仍嚴格Undirected。
- 3943／2230的typed音效使用world actor0域，保留真sequence／gfx／action／source與publication Scope；目標離場不砍已接收效果的延後Sound1361。有限效果seen加入active集合，ring淘汰再回入不重啟；原active資源準備提示與metadata History沿用。原64active／32音效／512fragment／4096key上限不增加，G11共存仍待。
- 六Runtime差分共34 overlays（20 Runtime／11 Rust QA／3 Java），365 top-level module與原六nested target保留。新14案（13純含合成RGB555像素＋1原資源opt-in）＋前226共240案只寫入並編譯，本文全未跑，非library inventory或通過數。原資源案將驗八bank、Sound1361／839、frozen heading、memory fragments、actor離場與ring／History；不開聲音裝置或GUI。
- 原指令掃描首次把frame selector範圍截在4E63AD指令內而被嚴格拒絕；原工具／failure保留，邊界改為原ret後4E63BA，再只讀重取完整範圍。原PE為analysis rebuild，runnable_claim=false；未執行、修補或提高其完整性主張。

入口來源識別66／缺114，下一組#59／#61；這不是66項玩法或G06完成。45／46／50完整hurt相對gfx／base+26、原初始化及倍率／uint32 rollover、companion命令與自然到期、G06.3d、完整library雙版、Release與GUI／G01–G12仍待。使用者明確恢復測試後才執行；正常1049輸入／所有PE／正式JAR、registry與G06 binding／16案640次不改。
