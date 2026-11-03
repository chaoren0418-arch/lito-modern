# 原生光箭與 S23 技能分支

同一 Windows x64 客戶端已接入技能4光箭。世界視窗點選目標後按 F3；F1 治癒、F2 護盾維持原操作。必須已學會、有可用 MP、目標存在且存活，並符合目前十格範圍限制。請求排入後禁止重複施法與攻擊，等正式回覆及角色動作完成才可再施放。完整技能書／快捷列、其他技能及 GUI 人工驗收仍待完成。

## 正式封包與權威

`NativeAttackSkillOracle.java` 直接呼叫固定正式 JAR 的 BattleSpellCatalog／Protocol。光箭為 skill4、row0/column3、action18、effect167、基本耗魔3、maximumRange10、reuse10ms。C14 帶目標物件 ID 及目前座標；客戶端不猜 HP／MP，也不將基本耗魔當成完整伺服器耗魔公式。

S23 的 29-byte envelope 現在區分 type0 技能與 type0x7F 普通遠程，嚴格檢查長度、尾端、ID、heading 與無目標不得帶傷害的限制。其他 type 仍明列未支援。type0 計入 own_cast_packets，普通攻擊維持 own_attack_packets；技能不會錯誤解鎖尚未回覆的普通攻擊。支援的 gfx167 不會在收到 S23 時提前排入受傷；HP／比例更新仍立即依正式封包處理。

MP 不足時固定正式伺服器回 S10 code278，回覆不產生施法動作、飛行或耗魔。目標、自身死亡、範圍、視線、安全區及其他伺服器條件仍由正式核心裁定。

## 原始 class4 動作與效果

共用 LightArrowAssets 只載入原始167／219效果，角色 action18 由現行世界 animator 負責。以原 Hit entry 發射，在那一刻凍結伺服器目標座標；Miss 使用封包終點。167 依原版固定點計算、原始四方向圖庫及60ms步進播放，抵達後建立219效果。跨影格或直接跳過完整播放區間仍只發出一次 Sound97。普通箭66仍使用自己的 type0x7F／30ms路徑。

本輪直接從 recovered PE 解碼424540／424780／4B92D0，核對固定點 rounding、四方向折疊、60ms排程、立即第一步及抵達時對4E8EC0的呼叫；來源列於 `native-class4-evidence.json`。IDA header 指向另一份 recovered image，本輪沒有宣告重新脫殼或執行原 BIN。

抵達受傷只排一次 presentation action，不修改 World；零傷害、Miss、死亡目標不排入受傷。事件保存 attacker／target incarnation，物件刪除／替換與換世界會退休。每個世界最多128個 cue，播放中上限64；未支援圖形或超出資源限制明確記錄 omitted。完整原版特殊圖形／中斷佇列和遮擋語義仍未宣告等價。

## 驗證

```powershell
pwsh -NoProfile -File .\維護工具\verify-native-combat.ps1 -AttackSkillsOnly
python .\維護工具\verify-native-attack-skills.py
```

五個 Rust 測試涵蓋正式請求／邊界、原子解析、施法權威與抵達受傷、目標凍結／替換／換世界、零距離／零傷害／Miss，以及原始效果、音效只播放一次。資源測試留下61個時間點與9個PNG，包含飛行和抵達中的代表影格。

兩個真正 ClientSession 皆使用隔離 MemoryAccountRepository，僅測試角色學會光箭，最大MP分別10／2。正常案例沿原地圖實走兩步進入十格範圍，再依序施放兩次；確認 MP10→7→4、正式目標比例下降、兩次167飛行及219抵達。拒絕案例確認278、MP2不變、沒有施法動作與飛行。最後的 MP 也與隔離伺服器實際欄位比對。沒有 SQL、玩家實表、正式部署或封包回覆注入。

最終實際測試數、EXE／JAR與来源SHA見 `驗證紀錄/20261012/current-build-receipt.json`；本輪資料位於 `驗證紀錄/current/native-attack-skills/`。schema17保存在 `驗證紀錄/20261012/native-skills-build-receipt.json`。GUI鍵盤工作流、音效裝置、完整技能系統與完整可玩客戶端仍待驗收。

下一步延伸其他攻擊與座標技能、持續地面效果與角色狀態，並把目前快捷鍵整合成完整技能書／快捷列；保留普通攻擊、光箭及既有技能的共同控制器與正式回歸。

PNG已檢視；正式十格目標與抵達效果位於右側視野邊緣，完整目標與效果另以較近距離的離線圖像核對。正式到達時序、HP／MP與事件次數由逐幀及伺服器收據驗證，不能將離線圖像冒充正式GUI操作。

## 2026-10-14 冰箭6 class5

FD1D504C／BBC59308新增C14 row0/column5及S23 gfx1797，MP4與十格範圍由正式來源核對。原Text/list.spr SHA523C61AD：1797八方向、109.effect(5 1798)，719／713與次級Hit；不能套用class4折疊四方向。唯讀4271D92F recovered PE獨立解碼42F1F0／42F300／42F570等6個範圍，證明發射時凍結路徑、立即第一步與60ms步進，抵達後在當前目標位置建立次級，Miss無次級。

36相關Rust通過，1個既有全caster golden測試仍ignored；兩正式ClientSession情境共8連線完成：成功兩次施法、雙端一次性飛行／命中與聲音，MP2正式278拒絕。PNG已檢視，但不當真人GUI／音效裝置／Release驗收。資料見ice-arrow-qualified.json。首輪metadata用全量嚴格parser遇既有614行錯誤，改用既有inspection入口並僅嚴格核對1797／1798；另修正測試loader名與過時『6未支援』前提，未降低其他技能拒絕門檻。
