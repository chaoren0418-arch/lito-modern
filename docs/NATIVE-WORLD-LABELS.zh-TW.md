# 原生世界名稱與目標退休

此階段 schema33 的同一 x64 EXE 完成 182 項 Rust／35 組、53 種隔離 TCP 與既有 72 組像素比對。新文字有 4 張字型樣本、2 張世界圖與 10 張 TCP 階段圖；70 幀名称收據及 817 份來源 pins。schema32 封存為前一個基準，完整可玩客戶端仍未完成。

## 來源與實作

`native_world_text` 對照 recovered PE 的 CP950 路徑：12px、weight400、charset136、預設品質、空 face name，由 Windows 選字型。先以現有嚴格 MS950 編碼器確認可往返，再以 Unicode GDI 繪製相同文字；本機實際 face 記錄於每個標籤。四次黑色外框依左、右、上、下順序，再畫前景；RGB555 以五位左移三位轉 GDI 色值。這不是 Sprite 的低位複製擴色，也不是 eng.fnt。Win32 API 身份根據呼叫參數與行為判定；恢復映像沒有可用的 import directory。

每個快取上限 256 筆／4MiB，取消可中止，所有 DC／font／bitmap 在 raster 結束回收。只把純像素帶回 Model。字型由宿主映射，尚未聲稱與舊 Windows 字型像素完全一致。

`native_world_labels` 使用來源 desc-c／LTdesc-c 的既有名稱服務，名稱不改動 Actor。原版以 MS950 byte 數乘六定位，因此不以實際字型 advance 取代。自身名稱在本體上方 17px；頂端越界移到本體下方。指向名稱有 14／27px 的名稱／稱號區域，稱號與名稱差 13px。保留原版 +/- 前綴省略與原寬度定位。未知／超界名稱不繪製；不輸出推測替代文字。

原版 48px 邊框與 640px 世界範圍改為現行 800×600 世界 canvas，此為明示版面調整。一般自身名稱與指向名稱接在分片合成後，沒有像素 owner，原有物件命中遮罩不變。原版全體名稱權限、emblem 圖形、特殊 NPC 徽記、稱號開關與 GM 模式尚未完整接線。目前非空指向稱號採顯示政策；沒有宣稱還原原版設定 UI。

顏色依 4DFB50 lawful 色階及 4E9220／48A656 分支：正式 battleActorWithHeading 尾端 c(0) 對應 +0x128，普通地圖使用紅色；值 1 使用藍色，500／560 使用 lawful 色。較舊版本的名單查詢分支尚未支援。主要 raw bytes 與解碼在 `驗證紀錄/current/native-world-labels/native-label-raw.json`。

## 目標生命週期

原版 4E8F70 隱身取消登錄會清除三個 target globals。World 新增呈現選取的 target_revision，遠端 false→true 隱身時退休原選取；即使兩次封包之間沒有輸入影格，隱身後立即現身也不能復活先前連打／技能／指向名稱。新按下或新指向可以重新捕捉。角色 incarnation 與權威 HP／MP／動作不變。名稱 Hover 允許屍體名稱，技能 Target 仍拒絕死亡對象。

## 驗證狀態

26 項定向 Rust 測試及完整 182 項回歸已通過；Debug 建置與 Release compile check 完成。正式隔離 ClientSession 的九個狀態封包形成十階段、70 幀：bot 名稱在 0–5 可見，6–8 不可見，9 明確重新捕捉 Hover 才恢復。十張 runtime 圖與六張 static 圖已檢視；實際宿主字型為「新細明體」。原有 72 組分片／圖示獨立像素比較仍通過，沒有把新 GDI 字型圖宣稱為獨立原版字型還原。GUI 實際滑鼠、字型跨機器一致性與音效裝置未驗收；完整可玩客戶端仍持續開發。

下一步核實正式 S2F 稱號更新與 FA 血盟資料、名稱設定／emblem／特殊標記及地面物品名稱，接續 pointer／quantity GUI、其他動態旗標與完整玩法。唯讀後續摘錄保存在本階段驗證目錄，不計入已驗證功能。
