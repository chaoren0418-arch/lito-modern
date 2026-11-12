# 稱號、血盟 metadata 與原圖標記（限定驗證）

schema34：同一AMD64 EXE通過188 Rust／37組、54隔離TCP及93獨立像素比對。新增11階段metadata擷取，七個標記phase皆經TCP畫面出現。完整GUI／音效裝置與玩家玩法未驗收。

## 封包與原版欄位

正式 JAR 在本階段由 `589733435518D92C05BAEEAE37A686D384EC3F7B9897E975AD1EF50D67A74AA7` 更新為 `DB2EF1343A85A7DC830CBC42311A43AB66E2521BF623F115894F90FD36344511`。Protocol.class 完全相同；19 組 oracle 重新產生後，內容只有 JAR 身份改變。之後又更新為 `C3C0FECD67510397E1436FFDAF9AD5BB5D314FF518887DB13E1031683453B9F0`，Protocol.class 及19組oracle再次確認相同；DB2EF134的188測試／54TCP完成結果另存，最終驗收改對C3C0FECD重跑。原版行為另由固定 PE 獨立解碼。

| 封包 | 正式產生器 | 接收欄位與效果 |
| --- | --- | --- |
| 0x2F | `Protocol.title` | object D、title S；更新稱號 |
| 0x69 | `Protocol.pledgeMetadata` | object D、clan S、title S、emblem D；更新三個 metadata 欄位 |
| 0xFA | `Protocol.battleClanMetadata` | object D、emblem D、text S、active D；text 寫入稱號欄位，非 clan 欄位；非零 active 啟用標記 |

對應原版 handler 4A73F0／4AFDC0／4AC1C0，稱號 setter 4E73A0 更新 actor+5C，血盟 setter 4E7690 更新 +60。FA 更新 +48、+5C、+13A。稱號更新保留已捕捉的 Hover／Target 身份，只重算文字位置；不改 HP、MP、動作、移動與目標退休 revision。未知 object 不建立新角色，也不排隊重播。

原版共用字串解析器對 FA 使用 17-byte destination，最多保留 16 bytes，仍消耗整個 wire 字串；現代端重現有效 CP950 邊界上的截斷。若截斷切開雙位元組字元，整包拒絕，避免引入無效 Unicode。這是明示的現代錯誤處理政策。全包長度、尾端、編碼與 object ID 驗證通過後才原子更新。S0B 新 incarnation 清除舊 marker flag。

S69 在原版戰爭全域旗標 834F14 啟用時另有陣營顏色與狀態分支；FA 旗標另有輸入限制。現況（2026-11-13，#335，細節見 `docs/reports/issue-335.md`）：

- S69 的狀態分支（4E7690：敵盟 +20＝2、己盟 +20＝1，兩者都設 +1F；空血盟只清 +20）由 `native_actor_relations::Context::clan_changed` 處理，已有單元測試。
- 指向名稱的顏色（48A624）：圖形 0xA9B 在 +1F 未設時固定用 0x7C00；其餘依 S0B 尾端的陣營 byte（0＝0x7C00、1＝0x001F、其他＝正義值顏色），地圖 500／560 一律用正義值顏色。`native_world_labels::hover_color`。
- actor+13A 除了 FA 以外，S0B 在 status bit 4 時尾端的 dword 非零也會設它；現在兩個來源都接上。
- FA 輸入限制是聊天指令 `title`／`emblem`／`ban`／`surrender`／`cease` 讀自身的 +13A：規則在 `native_fa_gate`，但現代客戶端沒有斜線指令輸入，還沒有呼叫端。
- 敵盟（+20＝2）不按強制攻擊鍵也能選為攻擊目標、游標跟著變，這屬於盟戰玩法，沒有做。

## 原始標記與時計

原版 486240 在名稱之後呼叫 4BA220 色鍵 IMG copy。global 834BD4 的 image table 取 7550..7556；image loader 4D4D90 經 archive route 42EA50，BASE 分支優先查 global 6479A0。初始化 42EEEE 的固定字串是 `LiTo\LTSurf`。

七張實際原圖皆在唯讀 `LiTo/LTsurf`，各 872 bytes：8-byte header 與 24×18 RGB555，色鍵為 0。以有界且固定的 archive handle 讀取，只保留七張 RGBA。若出現優先 Sprite shard 或語系覆寫則明確拒絕這項尚未限定的資源組合，不用 LTsurf 圖偷偷覆蓋它。原圖的黑色鍵會透明，RGB555 的五位元色值沿用現行 bitmap 展開。

原版每個 active 遠端角色先用目前 phase 繪製，再把共用 frame delta 加入同一 accumulator；達 350ms 歸零並換到下一 phase，6 後回 0，不保留餘數、不一次追趕多格。frame delta 上限 500ms。離開邊界仍推進共用時計，停用者不推進；自身名稱比對相符則略過。

明示現代政策：

- 角色排序使用 snapshot incarnation／ID；原版 roster 的歷史插入／刪除順序未完整重建。
- anchor 是目前已插值的角色投影位置 +13、-62，不再次加原版 12／8／6／4 動作子格表。
- 800×600 邊界取代原版 744×480 限制；靠右下的圖像交由 Canvas 裁切。
- 隱身或本體無可用圖的角色不繪製標記，但 active 角色仍計入時計。原版該函式沒有隱身檢查。
- 自身採 object identity 加 ASCII 不分大小寫名称比對；原版字元表的完整初始化尚未限定。
- 世代、地圖、自身 incarnation 或時間倒退時重置；使用現代 render request 的單調時間差，未聲稱等同原版前一個畫面結尾更新的時計歷史。

標記是 click-through overlay，不改任何像素 owner、選取資格或伺服器權威。

## 驗證範圍與重跑

`tests/native_label_updates.rs` 包含正式 oracle、原子拒絕、截斷、孤立更新、稱號重排及全畫面 pick 一致性。`tests/native_clan_markers.rs` 包含共用時鐘、原圖七相位與三種裁切位置、取消、世界 overlay、隱身與 authority／pick 保留。

`維護工具/verify-native-label-updates.ps1` 使用真正部署的 ClientSession 與加密 writer，將受控 S0B 設定包及十個 metadata 包送入新客戶端。檔案只協調擷取時機；不直接改 Rust World。這是封包整合測試，非自然入盟／戰爭玩法驗收。

證據位於 `驗證紀錄/current/native-label-updates`；原始位元組與解碼在 `native-update-raw.json`。測試圖像與快取僅写入本重建版 `LiTo381-Modern/target/current`。本階段是限定子集驗證，完整客戶端仍在開發。

本次Debug隔離TCP第一次啟用標記的整幀耗時 1658.704ms，含來源驗證與圖像載入；這不是純解碼計時。後續使用七張圖像快取，首次載入延遲仍需優化。GUI與流暢度尚未驗收。

C3版本第一輪材料技能測試未通過固定等待時間後的特效結束檢查；同一EXE重跑三種情境全部通過。失敗時未輸出該影格的特效收據，根因尚未確認；首次載入／取樣時計差異是待查假設，測試計時穩定性與失敗診斷仍待補強。原始失敗封包及日誌保存在C3-reagent-first-failure。

兩輪地面技能皆漏取1000–1083.33ms結尾區間；原始失敗時間軸保存在C3-ground-first-failure與C3-ground-second-failure。驗證器改為逐張核對取樣是否符合原始時間軸，明列結尾擷取次數，不要求實時排程必定覆蓋83.33ms區間。離線固定時間樣本仍強制驗證結尾影格與持續時間。僅一份PowerShell驗證腳本變更，825個其他來源pins與EXE未變；兩種地面技能TCP與下游獨立audit重跑。即時完整影格涵蓋與Debug效能仍未驗收。
