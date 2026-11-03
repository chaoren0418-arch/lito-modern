# #61 復活術隔離來源範圍

只做開發、靜態核對與編譯；測試本文／GUI／server／DB／播放全部暫緩。候選未選入registry或套用正常來源。來源入口數不等於完整玩法驗收。

C14 `14 07 04`＋屍體u32 ID共七bytes，不附座標。延用原畫面非無限range、顯式death action意圖、滑鼠／可見像素T鍵、技能書／Owner／controller互斥。MP50／reuse10、玩家最大HP除2／NPC除4、模板資格／材料／失敗280及經驗處理由正式端決定，不預扣／復活或猜NPC類型。

S24/321恰為`24 41 01`三bytes，沒有caster ID／文字；不得從3944或名單猜施法者。只在有完整inventory owner與已選來源／角色的active GAME死者上建prompt，綁connection、selection、來源、map／room／phase、generation、原actor incarnation與death action。替換詢問、其他question、換圖、斷線、observer、結果或新life使舊token失效。

C23同意為`23 41 01 01 00`，拒絕為`23 41 01 00 00`；尾巴是空字串。原生非modal兩按鈕明確操作，沒有預設同意／Enter自動接受。每個詢問有不同原生command ID，舊queued click不授權新詢問；ID不回繞，每controller最多30000次，耗盡明確失敗並要求重開視窗，不重用舊ID。送出前先取用token，傳送失敗關閉該controller，不自動重送；同意後保留等待，暫擋既有自動C_E0。使用者可明確「放棄等待」，只解除本機等待，不送第二個C23，原死亡流程才可接續。無自訂server TTL或注入時鐘。

3944是施法呈現，不是復活ACK；正式成功先S37 gfx230再S11，之後才HP／ratio。兩者在接收時凍結位置，3944另保存heading，轉為world有限視覺，避免同poll S11換incarnation把已收到光效丟掉；typed音效actor0、真sequence／Scope保留。3944原八bank／24frame／Sound1362 source2，230單bank／7frame／Sound124 source1，均Screen；整數callback視覺3371／830ms，不代替狀態或成功。原S11繼續獨立更新life，HP等自己的權威封包。

本輪15案例來源（14純＋1原資源opt-in）與前252共267案全未跑，非完整library inventory。原準備器poll插入點有三處而拒絕，原script／部分候選保留，沒有contract或compiler；修正版改唯一poll邊界。正式GAME的接受／拒絕／caster離開、NPC資格、同意／拒絕／放棄、S37→S11順序與資源／八方向／音效／GUI全流程仍待測試恢復。
