# 原生治癒、護盾與物件特效子集

同一 Windows x64 客戶端已接入技能 1 治癒與技能 3 護盾。世界視窗 F1 對目前選取目標施放治癒，未選取時對自己；F2 對自己施放護盾。按鍵重複不會連續送出，施法會清除按住移動／攻擊輸入。世界狀態列顯示正式 MP 與施法回覆狀態。尚未人工驗收完整 GUI 操作、DPI、IME 或音效裝置。

## 正式協定與權威

固定正式 JAR：`589733435518D92C05BAEEAE37A686D384EC3F7B9897E975AD1EF50D67A74AA7`。`NativeSkillOracle.java` 直接呼叫此 JAR 的 Protocol／BattleSpellCatalog，固定 C14 請求、S1E／S1F 技能位圖、S37 物件特效、S53 座標特效、S4D MP 及 S10 拒絕向量。治癒 C14 帶物件 ID，護盾只帶 selector；後續已接入的光箭4與火牆58另見 NATIVE-ATTACK-SKILLS／NATIVE-GROUND-SKILLS。

技能書最多 220 技能，嚴格檢查長度和保留位元；換地圖保留技能書，重新選角色則清除。正式 action19 只表示收到施法動作，不推定所有技能成功。客戶端不扣 MP、不自行補 HP，也不自行套用護盾 AC。MP 與 HP 由正式封包更新。

兩個已接入技能在固定 JAR 的 reuse 均為 10 ms；共用控制器另要求上一個請求結束及角色動作就緒。真正耗魔受伺服器修正影響，因此基本耗魔 4／8 不當作本地拒絕門檻。正在等待回覆、死亡、離場、未學會或未取得可用 MP 時不可送出。治癒目標需存在、存活且在目前限定範圍內；完整技能範圍與視線仍由伺服器判斷。

MP 不足的正式行為不同：護盾會回 S10 code278；治癒不回動作或拒絕封包。後者兩秒後顯示未收到施法回覆，必須重新按鍵，不會自動重試或標記成功。拒絕碼保留序號，避免同一批其他 S10 訊息將它覆蓋。更完整的拒絕種類與技能狀態仍待擴充。

## 原始特效

本文件的支援技能路徑接入經來源檢查的 wire gfx221／744。正式 JAR 自己將治癒模板 gfx11722 輸出成 wire744；客戶端直接讀取744，沒有自行替換圖像。原始 Text/list.spr、Sprite 成員、24 FPS 與逐幀 tick 驅動播放。現行 native_effect_timing 播放來源1..N-1、終止0並保留0的停留時間；跨過的 Sound 事件只發出一次，沿用既有有界音效準備流程。

S37 記住接收時目標座標與物件 incarnation，後者只作事件來源紀錄。原版普通221／744 class0入口複製位置後獨立播放，現行效果不因原物件死亡／移除／替換而退休；換世界仍清除。S53 使用正式座標。最多保留128個事件／64個播放中效果，未支援圖形明確記錄 omitted。原版細分座標及其他效果類別另待核實，詳見 [特效生命週期](NATIVE-EFFECT-LIFECYCLE.zh-TW.md)。

## 可重跑證據

```powershell
# 先在 LiTo381-Modern 內產生當前資源時序及離線影像證據
cargo test -j 1 --offline --locked --target x86_64-pc-windows-msvc --test native_skills --test native_effect_lifecycle
# 回到重建版根目錄後驗證正式隔離情境
pwsh -NoProfile -File .\維護工具\verify-native-combat.ps1 -SkillsOnly
python .\維護工具\verify-native-skills.py
```

Rust 測試從 `LiTo381-Modern` 執行 `cargo test -j 1 --offline --locked --target x86_64-pc-windows-msvc --test native_skills`。多個測試執行檔同時連結曾遇到 MSVC LNK1102；本輪最終使用 `-j 1`，仍維持完整除錯資訊與檢查。完整回歸請使用明列的17組現行測試，避免把歷史大型 lib suite 當成本階段範圍。

Rust 的五個技能測試涵蓋原子封包解析、技能書／換圖、請求限制、拒絕／逾時／物件替換及實際原始資源播放。資源測試保存31幀、4 PNG，並檢查畫面播放不修改權威 World。

兩個真正 ClientSession 使用隔離 MemoryAccountRepository：僅這兩個案例的角色資料設已學會1／3、最大MP10，其餘正式角色欄位保留。治癒案例先走近並承受真實反擊，再施法確認 HP 上升、MP10→6；第二個護盾請求確認明確拒絕。護盾案例 MP10→2，再請求治癒確認沉默拒絕與逾時。兩者都只出現一次施法動作及一次可見特效。沒有 SQL、玩家資料、正式部署或封包回覆注入。

當前精確 EXE／來源 SHA、測試數與影格數見 `驗證紀錄/20261012/current-build-receipt.json`；本階段日誌、正式 bytecode 摘錄、技能及既有戰鬥圖像核對在 `驗證紀錄/current/native-skills/`。歷史 schema16 保存在 `驗證紀錄/20261012/native-impact-build-receipt.json`。

## 後續

攻擊／座標技能、完整技能書與快捷列、buff 狀態／AC UI、各技能耗魔與冷卻呈現、技能中斷與特殊投射物、完整 GUI 及音效裝置驗收仍未完成。原版完整效果引擎與完整可玩客戶端都持續開發中。

本輪收尾發現外部正式JAR由9C97F868更新至58973343；12組oracle排除JAR雜湊後完全一致，治癒／護盾及選定Protocol方法排除constant pool索引後bytecode相同。已更新隔離快照與fixture，重新建置並完整重跑84 Rust／36 TCP；先前成功結果保存於native-skills/pre-drift，沒有只以向量一致宣告新版執行相容。
