# 生命之泉耗材與原始特效

技能63已接入原生已學會技能選单及地面施放控制器。選點可為自己所在格，使用正式C14 row7／column6與u16座標；共同owner、世界世代、權威修正及物件incarnation防止沿用失效選點。實際GUI與音效裝置仍待人工驗收。

正式伺服器決定材料40318×1與20MP消耗。背包wire只含物件ID，客戶端不把名稱、圖示或固定ID視為材料目錄，也不預扣背包／MP。一次待回覆，收到施法動作或兩秒無回覆後才可手動重試。選點範圍以現行JAR的1,681個座標答案比對，包含不對稱畫面邊界與unsigned極值。

隔離記憶體正式ClientSession驗證三種情境：

- 兩個材料、80MP：連續兩次手動施放，材料2→1→移除、MP80→60→40，再次施放沒有施法回覆。
- 沒有材料、80MP：兩次手動請求均無施法回覆，無消耗或特效。
- 兩個材料、2MP：兩次手動請求均無施法回覆，無消耗或特效。

所有情境核對無普通攻擊、HP50保持權威、無移動修正、無預測物品變更、不相關藥水維持原狀、關閉後worker與背包owner清除。材料catalog ID只存在伺服器的測試資料中；客戶端以收到的物件ID追蹤delta。

## 原始資源與新引擎選擇

原始list.spr的gfx2231只有action3，沒有action0：19個source frames，24FPS，每格4ticks；source0是subframe18，source1..18是subframe0..17，sound841只在source1。

唯讀解碼固定原生PE顯示S53進入class0/action0。4E5C00缺action0分支走4E62A1，載入第一張SPR、subframe0並返回false；fallback表只提供46..53對4..7，沒有0←3。原版這條路徑不會自動播放action3。原始證據保存在`驗證紀錄/current/native-reagent-skills/frame-selection-raw.json`與前階段`native-item-actions/native-action-fallback-recon.json`。

新引擎明確選擇播放原始2231/action3一次：沿用有界播放元件，source1..18後停留terminal0共4ticks，在76ticks（約3.167秒）後退休；841聲音一次。收據標明`modern_explicit_action3_once`，**不宣稱與舊版S53/action0動畫相同**。獨立延遲資源bank只接受此已限定定義與S53位置事件；不作泛用缺動作替代，也不接收S37 actor效果。

正式JAR對此技能建立320秒地面lease，這份catalog不產生地面傷害pulse。這是伺服器狀態的行為；客戶端不據此推算動畫長度、回血、傷害、buff或320秒循環。離線oracle核對自己格、同位置兩個lease、319.999秒未過期／320秒清除且沒有damage pulse。

## 驗證與重跑

`維護工具/verify-native-reagent-skills.ps1 -SkipBuild`使用固定正式JAR與現行x64 EXE執行三情境。`維護工具/verify-native-reagent-skills.py`獨立讀取原始2231-0.spr與Sound/841.wav核對bytes／SHA256、每次聲音與terminal時序、PNG與RGBA收據。新增截圖不是獨立像素golden；不增加既有25張像素比對數。

Rust新增6項測試涵蓋正式選點oracle、權威封包原子性、失效owner與選點、原始19影格及音效、固定位置／64效果上限／世代清除與S37拒絕、場景音效準備。完整回歸的最終數量與EXE身份以`驗證紀錄/current/native-reagent-skills/summary.json`及`驗證紀錄/20261012/current-build-receipt.json`為準；未通過完整pipeline前不發布schema26。

所有測試資料與輸出只在重建版，不啟動舊BIN、不碰玩家SQL或正式程序。其他技能、拾取、地面排序／名稱、特殊物品選項、快捷列／buff、GUI／IME／DPI與音效裝置，以及完整可玩客戶端仍進行中。

後續 schema27 已接入原生附近物品清單與指定數量拾取，並以實際 inventory 封包重播驗證；本頁上述拾取待辦已由此子集接續。地面排序／遮擋、直接滑鼠拾取與完整 GUI 仍未完成，詳見 [地面拾取](NATIVE-GROUND-PICKUP.zh-TW.md)。
