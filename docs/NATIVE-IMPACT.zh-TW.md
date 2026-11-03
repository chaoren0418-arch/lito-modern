# 普通箭矢抵達與受傷呈現

本階段只接入現行原生客戶端的 #66 普通箭矢受傷回呼；完整技能、特殊變身／投射物、原版動作中斷語意及 GUI 仍未完成。

## 靜態依據與來源限制

本輪重新核對 recovered PE `4271D92F…`、IDA `2A3A6E10…` 與現行 packed BIN `CE6EE68A…`。直接對 recovered PE 重新解碼機器指令，確認 `40E420` 比較固定終點、damage marker、物件存在及 death/action44/45，再於普通分支呼叫 `4E45F0(2)`；未抵達時排 30ms 回呼。發射仍由 `40E8F0` 固定當時目標座標。

IDA header 來源為另一份 recovered image `AEB05C78…`，因此不能只靠 IDA 檔案名稱聲稱等同目前 PE；本輪另存 raw bytes、7 個關鍵指令斷言及完整相關函式解碼。既有 recovery metadata 描述 mapped image 重建，不等同本輪重新脫殼現行 packed BIN；本輪沒有執行原版程序。原始來源及範圍見 `驗證紀錄/current/native-impact/native-callback-evidence.json`。

## 現行接線

- S23 仍立即更新權威攻擊回覆、動作及投射物 cue；#66 不再於接收時將目標受傷排入 World。HP／比例只套用原有伺服器封包。
- 原攻擊 Hit entry 發射，沿既有 16.16 固定路徑前進；抵達時產生一次 presentation impact。沒有 marker 的命中、Miss、未到終點而結束、死亡及失效物件都不觸發。
- 同幀所有角色先取樣，釋放到期 impact 後，只以同一時間重取受影響角色，保留已產生的影格事件。新受傷不能消耗抵達前的時間，也不複製音效。
- 每次 S_PUT_OBJECT 取得 world 內獨立 incarnation；投射物保存攻擊者／目標 incarnation，發射前或途中重用 ID 都會退休。動畫追蹤也以 incarnation 重設，避免新物件高 action sequence 延續舊受傷或死亡。
- 現行受傷沿既有有界動作佇列播放；death、已知 action44/45、佇列滿額不接受 impact。特殊圖形轉換與原版完整動作中斷策略仍待補齐。未知投射物維持先前呈現後備，不能視為已支援。

## 驗證方式

`native_ranged` 在固定 JAR 向量與原資源上驗證 330ms 抵達邊界、相鄰立即抵達、跳幀後只觸發一次、零傷害／Miss／死亡／刪除／重用 ID／換世界。逐幀 renderer 測試驗證目標受傷只在抵達後出現，World 完全不變。

`native_combat` 驗證同時間重取不重播事件，以及 S_PUT_OBJECT 後立即收到更高序號新攻擊仍清除舊呈現。真正 ClientSession 遠程案例另斷言一次抵達回呼及受傷畫面順序，保持正式箭量 20→19。最終實際執行數量與執行檔識別以 `驗證紀錄/current/native-impact/summary.json` 及 current-build-receipt 為準。

所有新產物限重建版；未碰正式 SQL、玩家資料或穩定程序。PNG 是 CPU renderer 輸出，不是 GUI／音效装置驗收。

## 本輪最終收據

79 Rust／16 組、34 TCP、19 像素比對通過；六戰鬥模式／七場次 918 幀、164 PNG、274 不重複影格事件、9 WAV。EXE 25,987,072 bytes，SHA256 `F050BDD7C2CA21E690A319079D5C8D8A3D566484860CA36A2222ED02227218AA`，schema16 固定 743 份來源。Release 只編譯檢查，沒有 Release 執行或玩家 GUI 驗收。

完成 session 後可執行 `python 維護工具/verify-native-impact.py` 重做唯讀圖像、事件及抵達順序稽核；需要 Pillow。
