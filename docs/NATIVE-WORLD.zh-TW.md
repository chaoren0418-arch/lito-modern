# 正式世界啟動與原資源首幀

本文件記錄首輪世界啟動；現行連續動畫與看板處理見 [原生動畫驗證](NATIVE-ANIMATION.zh-TW.md)。

原生控制器已接收正式戰鬥的 S_WORLD → S_D2 → 狀態／物件，將伺服器的地圖與座標送入新引擎。登入視窗在戰鬥啟動後切換為場景預覽，使用有界背景工作載入資源，同一 TCP owner 持續輪詢；中斷或關閉會取消並回收載入工作。

```powershell
pwsh -NoProfile -File .\維護工具\verify-native-world.ps1
pwsh -NoProfile -File .\啟動現代客戶端.ps1 -NativeLoginEndpoint 127.0.0.1:<隔離LOGIN端口> -ResourceRoot D:\lineage開發\0-客戶端
```

## 接入資料

- S_WORLD（0x28）保留 u16 map 與環境 byte，開始新 world generation，清除舊物件、狀態與完整性挑戰。缺少地圖的 S_D2 拒絕接受。
- S_STATUS（0x0C）完整保留正式 schema0 的 37 bytes；自己的物件 ID 取自此封包，與 S_PUT_OBJECT 對上才具備可呈現的本地角色。
- S_PUT_OBJECT（0x0B）保留完整位置、ID、gfx、動作、方向、速度、lawful、名稱、稱號、公會與尾段，依 u32 ID 更新，最多 2048 筆。門可以沒有名稱；沒有把空名稱當成壞角色。
- S_TIME、S_E9、HP／MP 更新及 S_EVENT subtype33 挑戰有界解析。未知 world opcode 留在 snapshot 的 unhandled_opcodes，未宣告其行為已完成。
- 挑戰（#187）：只對正式範圍（0x004ED840、5 bytes）以相容設定檔回覆 C_ADDR——對伺服器原始碼所述 relocation-free `call $+5`（`E8 00 00 00 00`）以正式 seed／CRC 演算法計算，protectedState=0；不讀記憶體、不讀 LiTo.bin。其他範圍只記錄不回覆，同一挑戰只回一次，換世界／退役清除。這不是原版程式記憶體實測；伺服器不回成功封包，伺服器端接受與長時間戰鬥相容仍待真機驗證。

## 原資源渲染

地圖 564 使用原始 `LiTo/LTmap/564/7fff8000.s32`、LTtile。依伺服器指定 map 選擇該數字目錄，最多 64 個合法 S32 sector／32 MiB；唯讀 handle、reparse 檢查與 SHA256 固定本次讀取的檔案，沿用有界 LT 解碼器。沒有借用離線 map4 或複製整份資源。

角色圖形依收到的 logical gfx，透過共用 Text/list.spr 的直接 graphics alias、動作和方向取得 Sprite 的明確 member。此輪只取伺服器動作的第一個影格；可解析的影格事件原樣列在 unexecuted_events，不啟動聲音、特效或未實作的時間語意。語法隔離或不存在的動作不猜替代。自己的圖像載入失敗會報錯；其他缺圖會逐物件記錄，UI 顯示未呈現數量。

鏡頭以 S_STATUS 指定物件的真實座標為中心，固定 800×600 基準再依視窗大小等比顯示。靜態地形與角色尚未完整交錯排序，屋頂遮挡、特效與連續動畫未完成。後續已接入世代過濾、快取重用和世界更新，詳見 [原生移動驗證](NATIVE-MOTION.zh-TW.md)；完整 GUI 跨場景操作仍待驗收。

## 實際證據

固定正式 JAR SHA256 `205DC1BAF5598523E67899E6C92A4B9BB07494D42DFA995CD741E67330D9B542`。`NativeWorldOracle` 呼叫真正 Protocol builders 產生地圖、啟動、時間、完整狀態、中文物件、無名稱門、HP／MP 與挑戰。四個 Rust 測試核對截斷、多餘尾端、ID 更新、無號範圍與新世界清除舊資料。

room-world 使用真正 LOGIN／GAME ClientSession：登入測試乙、選角、進頻道 0、加入房間、準備、等正式倒數啟動。核對伺服器最終 BATTLE、自己的物件 ID／座標／gfx、同一 GAME epoch，以及斷線寬限期後房間與頻道人口回收。使用 MemoryAccountRepository 與非持久化 GameStateStore，無 SQL 或玩家實表。

實際收到 map564 與 12 個世界物件，首輪首幀呈現 11 個；gfx1544 的 action0 在現行共用 catalog 沒有定義。後續已由靜態看板路徑補上（見 [原生動畫驗證](NATIVE-ANIMATION.zh-TW.md)「靜態看板」）：2026-10-12 的 schema3 收據 `objects_rendered=12`、`missing_graphics` 為空，1544 以 `static_prop` 呈現；#185 另加單元測試固定這條路徑。自己的 ID=1、座標 (32729,32797)、logical gfx0 → graphics3225，畫面中心 (400,300)。PNG 是同一客戶端從真實收包狀態經 GUI 共用 renderer 產生，並已查看；不是 GUI 連線全程操作證據。

PNG：`LiTo381-Modern/target/current/verification/native-world.png`。本次 RGBA SHA256 `b71fb1cd4f1c46da545378ea69db0ad9eb3fd42a9a1a54d402130b5f00c62ae4`。收據、session logs 與 oracle 在 `驗證紀錄/current/native-world`。相关 Rust 共 45 項；TCP 共 25 情境（世界 1，加原 24）。Release 只做編譯檢查。

後續單步移動與鏡頭更新見上述文件；看板映射和連續動畫已在後續接入，下一步补移動速度／狀態限制與碰撞／尋路，並接入完整性協定、戰鬥與返回。GUI 全流程、跨 DPI、多人與長時間運行均未驗收；完整可玩目標仍未完成。
