# S23彈體的Model重建接續及來源退役

使用者持續暫緩測試及桌面操作。本輪只做來源、靜態資料還原及metadata編譯；沒有測試／GUI／server／DB／fixture／播放或新PE。

G06相關來源缺口：原FiniteHistory只保存finite技能與S39等狀態，未包含S23 class4／ice彈體。Model在同一來源重建時可能重放仍留在World ring的舊施法；資源來源退役後亦缺少S23 admission tombstone。

新增純metadata History，保留generation、seen、不可變path、現有動畫cursor、arrival anchor與統計，不帶SPR／PCM／檔案。相同來源重建恢復active cursor；來源變更時捨棄active但保留seen，防止舊ring重新啟動。取消worker回收admission只合併同generation，不能帶回active；現有connection／source／world fence仍由WorldView負責。已接Model finite_history／restore／admission merge。資源prepare也納入active cue，當原ring及本地技能書hint都已移除時仍能準備既有彈體資源。目標／施法者incarnation及換世代清理沿既有守衛。

兩Runtime差分（native_spell_projectiles、native_world_scene）沿前27檔成28檔：十四Runtime／十一Rust QA／三Java。新增三案（一個原資源opt-in）均未跑；前94個書面案例保留。Debug／Release各library、library-test及七skill整合共18metadata，只記編譯範圍，不移記完整library／GUI資格。

既有S23 flight仍使用舊rational timeline；本輪沒有聲稱修好其60ms callback、循環幀或secondary末幀停留。原class4 callback先4E5B50再start secondary，class5順序相反，已有唯讀指令證據；下一輪須按各分支補時序，不把#25 class5直接套給class4。

原圖庫基準的靜態還原有重要矛盾：10塊合計1,189,423 bytes，原CRC18638一致，5894個F9標記及唯一末尾B3；但分析PE現存分隔迴圈只掃B3，按其字面指令只產生一個span。這不證明實際執行期base=1，也不授權用5894或共享表頭11297。原PE側錄標示analysis rebuild、runnable_claim=false；初始化自修補／另一份執行後映像及現代LT綁定仍未證明。沒有為通過而修改原PE或猜base。

只讀LTlist的九個相對項對應城堡／徽記／攻城裝置／obelisk，另+26為門；與同號BASE sprites是不同命名空間。來源未綁定前，#25原相對hurt家族仍缺資格，不能把靜態record count當正向驗收。沒有執行任何原生指令；CRC與分隔器只是對原檔資料的有界Python還原。

正常1049輸入／所有PE、原registry、G06 binding／16案640次不變；候選請求識別仍52／180，缺128不減。當版完整library雙版、Release、完整GUI及所有必要實際資格仍待使用者恢復測試後完成，整體G01–G12未完成。
