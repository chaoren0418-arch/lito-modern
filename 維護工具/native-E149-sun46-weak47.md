# E149 烈炎術46／弱化術47與class1接續候選

使用者暫緩所有測試、--list、桌面、server／DB／fixture與播放；本輪只有來源、唯讀JAR／PE／raw核對及metadata編譯。

本機入口：46為C14 `14 05 05`、47為`14 05 06`，均角色u32 ID＋u16 x/y共11bytes，其他可見活體、基本三格、reuse500ms，沿既有Target／Owner／controller、滑鼠及T鍵。MP20／10、47的40318×1、機率、boss modifier、近戰damage-5／hit-1與64秒狀態全由E149決定，不預扣、不推命中或自造狀態。名字僅取唯讀SQL作標籤；實際getter仍可經CombatDatabase／ResourceDatabase／CombatRules覆寫，未呼叫getter或DB。

46走現有AttackSpell分支，S23 wire0／gfx1811。raw1811為class1、attr8、19影格，source1立即Sound709、source11 Hit，原逐幀整數毫秒基準為Hit410ms、source0於1738ms、完整停留至1904ms，晚到不追幀。原S23 constructor與44F350比較鏈對1811直達45001A，不走僅2483／2484的兩步偏移；直接固定wire to，不因後來actor移動而重查。單bank無額外shadow；Screen沿原attr8。原hurt相對gfx家族／完整guard尚未綁定，1811的Hit只診斷，不套通用Impact；45同一未解處保留。

47走NativeWeaknessMaterialAction的battle／world原包裝與ClientSession nativeMaterial bodies。兩者讀AttackSpellRequest，實際BootstrapMethods綁repair032與classic predicate，材料admit先於原action19／狀態啟用／S37。raw2228是九影格、source1 Sound843、attr102=0／104=128；接現有RGB555 Multiply，不能當Screen或blind。基本視覺1166ms與伺服器64秒弱化分開，S37和動作均不當成功ACK。原dark40 route案例只擴充2228為另一已審核Multiply例外，其餘profile斷言保留。

補上所有既有S23 class1的metadata History（252／236／1801及1811）：保留generation、admission、固定位置、Clip cursor及計數，無SPR／PCM／archive；同來源Model重建接續，來源退役清active留admission。active序號併入既有FiniteHistory／WorldView的音效去重保留集合；raw ring與book被淘汰時仍由active準備資源，同generation worker只合admission，不複製peer graphs。

修正世界效果音效生命週期：class1改由原SharedEffectBank建typed PreparedCue，保留真cast sequence／gfx／source slot。class1及class7的音效actor=0是既有世界效果audibility域，PreparedCue仍受connection／generation／publication Scope管控，沒有新network actor或cast ID；避免Model的audible_actors過濾在caster離開後消耗獨立效果音效。原class7資源案例補強已移除caster後仍有Sound128 prepared world cue。原32音效／64 active／512 fragments／4096去重上限不增加；最大共存、實際聲音、GUI與G11未驗。

七Runtime差分、總33檔（十九Runtime／十一Rust QA／三Java），365個頂層module及原六nested配對不變。兩版library／library-test／七skill整合共18metadata命令，只編譯不執行。新增17案＝15純（含合成RGB555像素）＋2原共享資源opt-in，前185案保留，共202筆已寫案例全未跑；非library inventory、GUI／pixels／音效資格。

新案例涵蓋兩請求與三格、目標與owner失效、MP與材料authority、S23不提前hurt、S37不造弱化狀態、原410／1738／1904及1166ms時序、class1重建／raw ring／book淘汰、source退役、admission merge、世代退役，以及四profile實際bank／layer與typed音效metadata、2228實際bank／Multiply memory fragments。原資源從未在本輪讀取或播放；只有已解碼list.spr的唯讀定義與PE／classfile靜態來源核對。

來源識別62／180，剩118；不當作兩技能或G06完成。正常1049編譯輸入／所有PE、正式JAR、registry及G06 binding／16案640次不變，未套用或選入正常來源。原LT初始化／相對hurt家族、action0可選倍率、uint32 rollover及真實GAME仍待；接續依下一組當版定義與G06／完整library Debug／Release、Release及完整GUI。測試須等使用者明確恢復。
