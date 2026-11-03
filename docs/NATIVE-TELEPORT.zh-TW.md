# 原生跨端點傳送

正式參照為 BBC59308 JAR；所有測試伺服器均使用本重建版內固定副本、MemoryAccountRepository、獨立 loopback 埠，無 SQL、正式部署或執行中權威修改。原始四位傳送師目前都使用 TeleportMe，僅 B／C 可達；其他頁面或 A／D 不當作已實作功能。

`native_teleport::Pending` 將實際 NPC 操作與 connection／world generation／actor incarnation／inventory selection 綁定。S_F3 僅接收一次頁面可用狀態；S_F4 必須匹配此次選擇索引與可用槽位，不能單憑地址封包開始交接。相同 endpoint 的轉場尚需獨立分支，不能沿用跨端點成功收據。

端點確認後禁止舊世界輸入，排入 C_D9。`native_connection` 記錄包含未完成寫入的 TX counter；等對應 Sent 事件確認整份封包已送出，才回收舊 worker。原生跨端點流程允許立即 reset 舊 socket，S_F0 是 best effort；正式 quitFight 的 BATTLE 優先分支不當作正常租裝返回已修復。

交接會清除舊場景、NPC／交易、房間／頻道／服務目錄、物品 owner、技能／動作等待與公告，保留世界世代單調性。新 epoch 重新交換正式版本並認證；完整目的地角色清單中必須找到原角色，等待正式 GAME 自動選角的 S_C8／S_7C。此時不再送 C_C9，避免重複消費已暫存租裝。新世界僅由正式後續房間／地圖封包建立。

目的地連線／握手失敗時清除權威與密碼。正式 GameHub 可能仍保留暫存租裝；同一 controller 的一次重新登入只能在相同 LOGIN endpoint、帳號及來源重新認證後等待原角色接續。不同帳號／來源、取消或再次失敗不能挪用這個關聯。暫存過期、使用者取消後另開連線、同 endpoint、完整實體 GUI／Release 均需另驗，不能由本次成功例推論。

20261014-001012-969（第一版）：測試 EXE 0188806C、BBC59308，兩玩家透過 B／C 到 127.0.0.3／.4，epoch 2→3，14／11 件租裝與被動權威逐一相符；舊畫面拒絕新世界輸入，兩端同房 map564 開戰，6 條正式 ClientSession 回收。獨立比較見 `驗證紀錄/current/native-teleport-handoff/20261014-001012-969/teleport-handoff-comparison.json`。後續失敗恢復版需另綁版本，不能沿用第一版全驗。

20261014-001536-629（第一版失敗邊界）：目的地只接 TCP 而不送 S_KEY，兩端約 5 秒逾時，舊權威清除，可重新認證 LOGIN；當時未驗 GAME 重入，伺服器仍有兩份暫存租裝。此發現促成一次、同帳號／來源的恢復接續，後續結果以新收據為準。

重跑工具：`維護工具/run-teleport-handoff.ps1`、`run-teleport-failure.ps1`；兩者只接受 target/current 內測試 EXE 與固定 JAR，保存來源 hash、自己的 PID／建立時間、實際範圍和清理結果。成功比較工具為 `verify-teleport-handoff.py`。測試沒有證明正式部署已配置傳送地址，也沒有代表 G06 或完整產品完成。

恢復接續版：主程式13867311僅完成唯一入口建置；成功測試EXE21A66983、失敗重入測試852B6118，皆使用BBC59308。20261014-002344-821成功兩端接續map564；20261014-002156-986兩端各經epoch2→3（目的地握手逾時）→4（新LOGIN）→5（新GAME），原角色／租裝接續後再次入map500並完成移動，正式暫存租裝為0。32相關Rust／6suite、14條正式ClientSession清理通過；獨立比較與來源pins整合於rental-teleport-handoff-qualified.json。這是控制器子集，完整GUI／Release、同endpoint及過期／取消／跨帳號來源的對抗案例仍另驗。


2026-10-14 revision49：原TeleportMe頁面未入Catalog使控件不可操作，已依LTtext原343頁加入B／C請求並保留7311／7312圖示參照；原B標籤製作備註保留於資料，控件僅顯示目的地。OOM后Debug limited symbols1保留斷言、Release optz ThinLTO16，兩profile各15相關Rust通過，唯一入口主69B769C1完成。固定76EB原方法2149／983未變、舊handle改名內容相同；GUI6sessions實際原頁面／14步原生鍵訊息、角色租裝14／11、無重選角、新共同map564與圖像／被動權威核對。另一固定76拒絕端點恢复8sessions過。外部来源更新C02，固定副本資格與最新来源變更分開記錄；未修改相鄰專案。先前漏頁面、隱藏控件判斷及錯把房間地圖描述當戰場的失敗保留，最後由BATTLE權威與真實世界狀態重驗。完整原圖示排版、實體輸入、GUI取消／恢復、最新C02相容與整體G01–G12仍待。

詳[GUI限定收據](../驗證紀錄/current/native-world-integration/teleport-gui-qualified.json)。


2026-10-14 revision50：主69B769C1／固定76EB的原生GUI目標5秒無回覆恢復、目標TCP建立後取消立即重試、取消後真實33秒自然逾期三案各雙端8sessions，共24sessions。停止後背包0／舊NPC消失／連線可用；立即重入不重送C_C9且原14／11租裝物件接續，逾期經被動staged expiry觀察后須手動選角、新基本背包28／29無舊租裝。兩端均重入原map500並由鍵訊息作權威步行，全部session／membership回收。固定副本與外部來源變更分開，無相鄰來源／SQL／時鐘或執行期權威寫入；此輪只改測試／harness，產品Rust未變。非必要密碼控件讀取失敗保留，移除後重跑；PrintWindow初次樣式滯後，等180ms後與控件旗標吻合。原圖示排版、更多取消／舊世代、實體輸入與最新來源全相容／整體G01–G12仍待。

詳[GUI恢復限定收據](../驗證紀錄/current/native-world-integration/teleport-gui-recovery-qualified.json)。
