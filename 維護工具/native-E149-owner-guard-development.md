# E149 原 owner guard：獨立開發候選

目前只編譯新增的 QA 程式與讀取位元碼，沒有呼叫 guard、啟動 fixture、DB、案例或 GUI。這個候選尚未取代 current_E149_fixture_binding；原兩版準備計畫與既有編譯／連結資格保留原範圍。

新增的驗證程式與唯讀 observer 分開：observer 原 18 個 record accessor 與不寫遊戲欄位的路徑維持；harness 的 QA probe 則明確呼叫原 `Owner.requireActive`／`Owner.matches`，其結果必須由後續 reader 消費，不能只看 ready/result 的布林值。所有 owner、session、inventory 參考均來自本案例實際 GAME admission；沒有自行建造 owner、寫 active／identity 或呼叫 adopt／retire／物品 mutation。

三種有限檢查已寫入 Java，仍未執行：

1. 客戶端接續確認點停在實際 LOBBY 時，原 owner 對自己的實際 inventory 必須通過 requireActive，再寫 continuation ACK。每個帳號各一次；不把異常拋出當成功。
2. 若已觀察到另一帳號的實際 owner／inventory，原有效 owner 必須拒絕該 inventory，且前後仍接受自己的 inventory。另驗 matches 不接受該另一實際 session。這只驗實際 session／inventory pointer fence，不等於獨立隔離了帳號或來源 guard 的每條分支。
3. 四個選角 GAME admission 的原 Future 都結束後、hub shutdown 前，實際 owner 必須已 inactive／disconnected，對保留的 inventory 呼叫 requireActive 必須精確拋出原 stale 例外。不能先關 hub 再用全域停止造成的拒絕當退役證據。

原 NativePlayerLifetime.admits 位元碼在 OPEN 時直接接受；只有 DRAINING 分支另讀 thread-local commit。正向控制與退役檢查都在 hub 正常運作的範圍內。原 requireActive 再核對 application/lifetime、session／角色／來源限定識別及 inventory 角色；ClientSession.nativeDeInventory(int) 是四個實際 inventory 欄位的 switch getter。所有檢查仍需未來案例的實際返回、例外、observer ID、原始 trace、Future 及退出證據核對。

owner-guard-trace.jsonl 將記錄 schema、kind、guard 名稱、原 observer 的 owner／session／inventory ID、來源限定角色／帳號、active／matches、精確 rejection 與原始 observation。QA 僅保留最多 16 個 owner 的實際參考，實際選角 owner 預期四個；所有來源與失敗輸出保存。取樣不是原子快照，完整修改拒絕、非預設屬性、timer／expiry 及完整 Main／owner／G06 資格都不由這三種檢查推得。

下一步把新 Java 編譯資格與原 Rust byte 資格分開接入 producer／reader，固定 guard trace 並核對各角色的正向／外來 inventory／退役記錄；先保留現行 source-ready 計畫。使用者恢復測試後才驗實際 guard 行為與取樣時序，再跑原恢復案例及完整回歸。
