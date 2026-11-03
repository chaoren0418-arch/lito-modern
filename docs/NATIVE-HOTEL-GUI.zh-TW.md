# 旅館與市民原生控件驗證

2026-10-14，D3B6A244 客戶端與 BBC59308 正式 JAR 已完成旅館三種地圖及市民任務的限定視窗操作。唯一入口啟動兩個原生 Windows 視窗，使用隔離記憶體帳號與正式 ClientSession，未修改執行中的伺服器權威。

旅館四條路徑各透過 13 次原生鍵盤訊息步行到席巴。原始 `inn`、`inn2` 文字及姓名參數可呈現，0／9 數量遭拒；先租一把鑰匙、退租使背包回復，再分別租 1／2／4／8 把鑰匙並進入 map501／502／503。換圖後舊 NPC 面板隱藏，斷線後租約全部回收。被動正式狀態與每張截圖雜湊均獨立核對。

市民任務完成 32 步走訪、四枚徽章取得及交回、市民資格成立與寶石 20→21。原始頁面中阿里亞的「我想成為正式市民」與其他 NPC 用詞不同，已修正測試選項辨識，保留失敗紀錄。

測試在未切換的私人桌面執行，只操作自行啟動的視窗。移動由實際 GUI 執行緒焦點、原生訊息迴圈、控制器及正式移動回覆共同驗證，不需要取得使用者桌面的前景。最初一步探針把方向鍵的地圖方向寫錯，失敗證據與更正保留。

這些結果不等於實體鍵鼠、中文 IME、多 DPI、旅館訪客自行離房或全玩法 GUI 驗收。自然到期屬另一組固定 EXE 的實際長時間證據，不混入本次控件驗證。

現行證據入口：

- `驗證紀錄/current/native-world-integration/rental-hotel-gui-qualified.json`
- `驗證紀錄/current/native-world-integration/rental-citizen-gui-qualified.json`
- 同目錄的 `hotel-gui-validation-snapshot`、`citizen-gui-validation-snapshot` 保存本次測試腳本，避免後續工具變更回寫歷史結果。

## 2026-10-14 可重跑的維護工具與介面可讀性

420D8952介面版已用整理後的工具重驗房間、旅館503／501、旅館502／502、市民與四商人，五組全部通過。HP／MP改為目前值／上限，0／9數量輸入顯示中文範圍，四行狀態提示可完整呈現；畫面與權威比較記錄在 `native-world-integration/ui-readability-current-qualified.json`。不將先前Debug／Release的70項TCP結果套到此新EXE。

從專案根目錄以PowerShell執行：

```powershell
pwsh -NoProfile -File ./維護工具/run-isolated-native-window-test.ps1 -ActionProfile room-admin
pwsh -NoProfile -File ./維護工具/run-isolated-native-window-test.ps1 -ActionProfile rental-hotel
pwsh -NoProfile -File ./維護工具/run-isolated-native-window-test.ps1 -ActionProfile rental-hotel -HotelMiddle
pwsh -NoProfile -File ./維護工具/run-isolated-native-window-test.ps1 -ActionProfile rental-shop -NpcNavigation
pwsh -NoProfile -File ./維護工具/run-isolated-native-window-test.ps1 -ActionProfile rental-merchants
```

工具支援 `-BuildProfile Release`。成功的視窗收據仍須搭配各自 `verify-native-*-window-authority.py` 獨立比較；腳本退出成功不代替全部證據。三類路徑與商品表保存在 `LiTo381-Modern/tests/fixtures/native-gui`，保留正式JAR身份檢查。查詢暫時無活動的角色時，工具會從完整被動追蹤尋找最後一筆該身份資料，避免只讀尾端120行漏掉另一角色。
