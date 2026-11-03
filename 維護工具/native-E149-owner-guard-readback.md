2026-10-24 現行追加規格：fixture binding v3、case plan／execution v4、suite plan v3；沿已編譯43da2d的Java六class，11 Rust來源與原兩版controller相同。原八setter32筆與新setItem／setItemId八筆分開保留trace、marker與SHA，每案合計40次，兩版16案共640次均未執行。producer／reader／suite只完成來源／AST及檔案計畫；使用者測試／桌面暫緩仍有效。完整規格見 [模板／ID整合](native-E149-morph-fixture-binding.md)；以下v1／v2、舊v3與32-only說明保留階段記錄，schema與範圍以本追加規格及實作為準。

# E149 owner guard 與結果讀回整合

新增 Java guard 候選接入 producer／reader 的來源契約，兩版 v2 計畫分開固定原 Rust 映像與新 Java 編譯資格。這是未來案例使用的開發準備；尚未呼叫 guard、執行 reader／純解析函式、Job、DB、SQL、案例或 GUI。原候選開發紀錄與舊計畫保持原範圍。

producer 要求 ready 的 owner_guard_probe_kind 為 G06_E149_owner_guard_v1，將同一值寫入 execution，並固定 owner-guard-trace.jsonl 的 SHA。reader 同時核對 ready、result、plan／execution、原 Java source／class 與原始案例的所有終止證據。

純讀回模組 native_E149_owner_guards.py 檢查總共 7–8 筆 receipt：兩個有效接續 owner 的正向控制、一或兩個外來 inventory 拒絕、四個實際選角 GAME owner 的退役拒絕。每個 owner／session 必須與兩角色舊／新場景觀測的四組實際 admission 一致。每筆 source_observation 須能在原始 server trace 中找到，並核對來源限定的角色／帳號識別。

正向控制必須對應原 continuation-observed 的 LOBBY JSON，以及同一實際 native inventory 的 observer ID。外來拒絕須發生於該 owner 的正向控制之後、所有退役記錄之前；其 inventory ID 必須等於另一帳號 admission 最後退役 receipt 保留的實際參考。四筆退役 receipt 必須精確記錄 inactive／matches=false 與原 stale 例外，且原 session 的最後 trace 已 disconnected／retired。所有 receipt 都須宣告未嘗試物品 mutation，不得授予完整 owner 資格。

foreign matches 拒絕、拒絕後仍接受自身 inventory，以及原 Future 完成後／fixture close 前的執行順序，結合固定 Java 控制流程與實際 receipt 判斷；raw receipt 沒有額外的 live lifetime mode 欄位，不把這項順序推論擴張為完整 lifetime 觀測。取樣仍不是原子快照；若跨取樣的原參考／ID／phase 無法吻合，reader 拒絕授予本案例資格，須保留原始結果再修正同步點。

本輪只做七個工具模組的 AST／來源核對，以及既有 6 class、11 個相同 Rust 輸入與兩版原映像的檔案核對。非預設屬性正常產生路徑、完整 mutation／來源與帳號分支／timer／expiry 矩陣仍待開發或實跑；完整 Main、G06、library／Release／GUI 及整體目標未完成。
