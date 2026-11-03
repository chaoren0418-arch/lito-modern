# GAME 斷線、重新登入與畫面世代

`NativeRecoveryHarness` 在同一正式 BBC59308 GameHub 上依序接受兩組 LOGIN／GAME ClientSession，於房間或已載入戰鬥中關閉第一組自有 GAME socket。測試保留原 LoginController，等待失敗和 worker 回收，再重新登入、選來源、選角、入房／開戰。四個 session 均實際執行；不改 HP、角色座標、戰鬥結果或玩家資料表。

戰鬥中的離線清理由正式核心延遲處理，harness 有界等待房間／頻道／來源交接清空後才容許重連。第二次進入也以伺服器實際觀察的身份、座標、HP／MP 交叉核對；完成後再次驗證清空。

檢查發現 `LoginController::start` 與重新選角原先會把 World 整體設為預設值，重用世界世代 1。Frame 的輸入／移動保護包含世界世代；相同角色與座標回到同一地圖時，舊畫面可能與新世界相符。現在清空權威資料時保留既有世代，由下一張正式地圖封包遞增；角色、背包、選擇、錯誤及動作狀態仍清空。

`tests/native_world.rs` 增加清空後重新載入不重用舊世代的驗證。`20261013-071732-260` 戰鬥故障情境已通過：連線 epoch 2→4、世界 generation 1→2，重連前 Frame 的 movement/input/death 授權均失效；新 Frame 可操作。該次 EXE 為 `AD02BC2ABC79822F3DA831865145A815D81B385F1BDF69FA5273708666825CE1`。

首次房間斷線驗證 `20261013-071101-710` 通過。首次戰鬥驗證 `20261013-071119-184` 失敗是 probe 在 RoomReady 重複送準備，阻止倒數完成；已改成只對尚未準備的自己送出一次，失敗證據保留。後續同版回歸已完成，兩個情境均列入 schema39 的 `candidate-regression/20261013-072017-855/index.json`；後續 G04 工作樹仍須另外驗證。

```powershell
pwsh -NoProfile -File 維護工具/verify-native-recovery.ps1 -Mode room
pwsh -NoProfile -File 維護工具/verify-native-recovery.ps1 -Mode world
```

這兩個情境驗證共用控制器及共用渲染器，GUI 操作、所有故障時點與長時間重連壓力仍由 G08／G11 分別驗收。
