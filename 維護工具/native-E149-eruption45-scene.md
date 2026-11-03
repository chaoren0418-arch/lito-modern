# E149 #45 地裂術接收呈現候選

只做開發，測試、--list、桌面、server／DB與音效播放仍暫緩。

前輪核心／共享資源兩個module保留，新增scene adapter並修改native_spell_projectiles。原30檔加三新module共33檔：十九Runtime、十一Rust QA、三Java；完整graph365個頂層module，原六nested配對不變。新兩版library／library-test四個metadata命令僅編譯，不沿用或擴張前輪七技能integration的編譯範圍。

正式靜態分支：45在ATTACK_SPELLS，attackSpell factory首先查此表，非空即於21返回；六個selector-only特殊表都不含45。原ClientSession dispatch於4755／6993走classic／battle兩個AttackSpell overload，個別保存。C14字面前綴14 05 04、actor ID＋x/y為11bytes；S23 opcode35、wire flight_type固定0，依共享129的attribute109選class7，非type7 wire。SharedClientResources兩種namespace的129均不改映射；AttackSpell getter仍經ResourceDatabase，20MP／100ms／range8及gfx129是catalog基值，不宣稱當版GAME已觀測，成本／資格／覆寫全由伺服器決定。本輪不登記本機#45請求，識別仍59／缺121。

原排程4D65F0反向掃描至舊due<=new due，再插其後；4D6570移除front後回呼，同deadline為FIFO。核心的registration順序因此有原指令證據。原queue是unsigned32，長uptime rollover未驗；原43E888 cadence parser另有647EA4可選action0倍率，實際runtime設定未綁定，不能把基本24FPS當完整速度設定資格。分析PE外部patch仍非可執行原版證據。

scene adapter借Model shared Text一次準備129／130／123／124與3音效。原S23 caster Hit啟動：核對caster incarnation與存活，來源固定wire from，root entry只重查捕捉的同incarnation target，缺席／ID重用保留wire endpoint。之後C7射線固定，caster／target消失不取消；C6建立時再查同捕捉身分一次，否則固定原endpoint，之後不追蹤。generation及source退役清理graph，同來源metadata History接續clock／node／occurrence；來源退役保留admission，raw ring／book淘汰不提前刪active audio sequences。共同class4／5與class7合計維持原64 cast預算，fragment512及audio4096上限不增加；G11最大共存未驗。

音效PreparedCue沿真正cast sequence／graphics／source slot；callback occurrence以(node+1)<<32|callback注入本地節點身分，不造network actor或cast sequence。source1的128可在多節點各觸發，3080保留可重疊群組；同payload publication latch及Weak canonical dedup沿既有來源。沒有播放器呼叫。當前draw只取clock snapshot：每節點main用Over，對應shadow用Shadow，完整cast先檢查所有anchor／subframe／fragment總預算再提交；C6 final source1提交成功才精確commit，投影失敗保留該final draw。

**尚未派發child hurt**：source4 Hit保存精確時間／目標／damage marker診斷，無HP或通用Impact替代。原相對gfx家族、base+26與狀態／動作guard綁定未完成，不能猜base=1／5894／11297；完整技能與受擊呈現仍未完成。正式入口／目標資格與ResourceDatabase設定須繼續核對後接請求與選單。

新增十案均待跑：八個純來源／metadata案例，包含caster存活與incarnation、wire0≠class7、target ABA fallback、wire source固定、source退役、ring／book淘汰、世代admission及節點occurrence；兩個原共享資源opt-in驗三節點傳播／rebuild／target ABA、雙層memory fragments及C6最後提交在投影失敗後仍可提交。前165案保留，新合計175筆已寫案例全未跑；非library inventory、GUI或原資源／音效資格。

接續先完成原hurt guard／LT與正式45目標／資源覆寫分支，再接本機入口；其餘技能、G06、完整library雙版、Release及完整GUI沿總報告。測試只在使用者明確恢復後進行，確認前不交人工。
