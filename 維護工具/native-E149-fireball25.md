# #25火球術：請求與S39 class5整合來源

使用者仍在使用電腦；本輪只有來源開發、唯讀原協定／PE核對及metadata編譯，沒有測試、--list、GUI、server／DB／fixture、音效播放或新PE。

E149原180項目錄的25定義為171、range6、reuse500；請求沿AreaAttackSpell的OBJECT_TARGET reader，C14 `14 03 00`接unsigned actor ID、x/y。選單沿既有learned book／owner／角色選取世代與revision守衛。MP20基值及實際成本、傷害、命中、範圍／遮擋與radius全由E149決定，MP1不冒充本機成本資格。

S39只新增action18/type8/gfx171。接收時保存首target當時位置或wire origin的一步fallback；launch只重查同incarnation的第一target，不因miss改選第二人，不接納接收時未知／後來替換的ID。其後路徑固定，採前輪原生60ms一格及1/2/3/4/0循環時序，延遲不補跑；source1音效cue每五步可重入。

arrival按原完整wire名單逐一解析當前同incarnation角色，每個有效名單項各建一個218，包括miss／dead與duplicate；未知／替換項不補投。爆炸使用各角色arrival時位置後固定。一組相同起點時序的218 cursor繪製所有獨立anchor，保留terminal0的41ms停留，沒有虛構Hit。hurt候選只給hit且有效存活、非已知禁止gfx的角色，再由共用animator保護death／transform／guardian及隊列；不改HP／MP或任何權威。

每個secondary start均有自己的99音效payload，再派送同callback的171 cue，所以相鄰或第六步arrival可見99×N→98。新PreparedCue callback occurrence保留真實source slot與cast sequence；history按(event,occurrence)去重，重入幀不會被整次施法的finite key吞掉，重播同occurrence仍拒絕。原finite API用None occurrence，語義保持；全部payload沿現有scope、shared latch、same-ID播放抑制與4096 history上限，沒有音效裝置呼叫。

draw前先預檢全部anchor／offset及完整fragment queue容量，最多128名單／64 active／512 queue沿原上限。History只保留cursor／Animation metadata與frozen anchors，不帶SPR或PCM；換世代清空、caster incarnation退役停止，重建資源不重發已進入的callback。

四Runtime差分（skills、selection、many、audio），沿前26檔來源成27檔：十三Runtime／十一Rust QA／三Java。新增九個many案例（一個原資源opt-in）、三個audio identity案例；前82個書面案例保留。這些全部未執行。Debug／Release各library、library-test與七項skill integration共18metadata；僅在實際程序退出、來源／日誌與產品hash讀回後記編譯結果。

限制：原PE 4DF7C0還有以動態graphics base相加的九項禁止hurt家族，該base尚未綁至現代catalog，不用記憶體快照的零值猜絕對gfx。現有已知字面gfx與共用animator守衛已接；完整原hurt資格／移動佇列行為仍須補原base及實際資源／當版施法驗證。class4／冰箭舊rational時序亦須另查其原分支，本次不移記其資格。

候選請求識別增加為52／180，缺128個入口；這是來源識別，非技能通過率。未套用正常來源、未選入registry，正常1049輸入／所有PE與G06 binding／16案640次不變。下一步補上述原graphics base資格與class4／冰箭循環稽核，再續當版剩餘玩法；使用者明確恢復測試後，依總報告先完成元件／G06、當版完整library雙版、Release與完整GUI，全部自行確認後才交人工。
