# 火球171／218原生時序開發

本次只補可供S39整合使用的來源核心，未接通#25請求／選單或世界派送。使用者正在使用電腦，全部測試與桌面操作暫緩。

- 只讀原PE4271D92F之42F100、42F570、42F300、42F550與S39 class5分支；完整bytes／反組譯及SHA保存在native-E149-fireball25-contract。沒有執行原PE。
- NativeAreaFireballFlight保留不可變路徑、八方向及metadata cursor，首步立即、之後60ms一格；延遲從實際dispatch重新排程，不依wall time補跑；抵達事件只發一次，回退時鐘拒絕。飛行有限影格走1..N-1再0，末幀保持到實際抵達。
- 回傳arrived_now與flight_cues_after_arrival，要求呼叫端先為本次解析的接收者啟動218，再派送171 cue。相鄰距離原順序為99再98。這是API次序契約，尚未接入native_attack_many。
- 171/218獨立exact-source-frame API沿原SPR與WAV bank；218 native finite Clip保留terminal0的41ms停留，沒有虛構Hit。既有class4與S23冰箭的rational API未改資格，冰箭class5時序仍需另行稽核。
- class5專用路徑入口允許從u16起點走到[-1,65536]的一步fallback邊界；原public路徑入口仍限定u16，路徑長度仍受256格presentation上限限制。

新增七個有意義的延後QA（一項需明示原資源根）；僅編譯，不執行。兩版library／library-test／七項既有skill integration共18metadata命令，只有實際終止並核對來源／日誌後才發布編譯結果。無新PE或新施法資格。

下一步接#25角色ID+x/y請求、六格／500ms與選單；S39 action18/type8/171接收保存首目標fallback，launch只重查同incarnation的首目標再固定flight，arrival按原完整列表逐一解析有效incarnation。miss／dead也有secondary visual，hurt只給有效hit；未知／替換角色不補投，duplicate不自動去重。預檢全部anchor及fragment budget後整組提交，保留history／generation退役與原音效抑制政策。

完整施法、當版技能書／接受／低MP／range／LOS／多目標／到達次序／像素／音效及GUI均未驗。不要把本次編譯、舊S23冰箭或歷史75結果移記為#25完整通過；正常1049輸入、所有PE、registry及G06 binding／16案640次不變。
