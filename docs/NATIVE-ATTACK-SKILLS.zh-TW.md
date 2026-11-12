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
## 2026-11-12 T028-4（#166）：S23／class1／火球的原生回呼時序

以下取代上文「跨影格只發出一次 Sound97」「一次性飛行」的舊 rational 時間軸敘述；舊 `start_effect`／`effect_frame` 只保留給 inspection 與 battle presentation。

- **S23 class4（167→219、1583→218、1799→1800、756→219）與冰箭 class5（1797→1798）** 改由 `NativeSpellClock` 驅動（`light_arrow_effect.rs`、`native_spell_projectiles.rs`）。依據同一份 4271D92F 反組譯（`ida-client-oep-v2/full-disassembly.asm`）：
  - `sub_424780+AD`／`sub_42F570+9A` 直接呼叫移動函式，首步立即；未抵達時以 `mov ecx,3Ch; call sub_4D65F0` 重排 60ms 回呼。延遲的 dispatch 只前進一格，下一個期限從該次 dispatch 起算，不追趕 wall time。
  - 每次移動都呼叫 `sub_4E5B50`：來源影格 +1、到影格數歸 0，再進 `sub_4E5C00`。飛行影格是依移動循環的 1..N-1,0，不依 catalog 的 24Hz ticks，也不停在 source0；source1 的音效 cue 每 N 次移動重入一次（實際是否發聲由 same-ID 抑制政策決定）。
  - class4 先 `4E5B50`（4246CD）再建立次級（…42473A）；class5 先在目標迴圈建立次級（…42F4CB）再 `4E5B50`（42F500）。相鄰目標的 cue 順序因此是 1583：97→99、1799：719→711、冰箭：713→719。
  - 抵達以實際到達終點的那次移動回呼為準，不用 `path.impact_at()`。class4 爆炸固定在飛行終點；冰箭在抵達當下解析同 incarnation 目標的座標後固定，無目標不建立次級。次級沿用 native finite `Clip`，保留 source0 的獨立停留（218／219 為 41ms、1800 為 166ms、1798 為 125ms）。
  - 音效改為 `PreparedCue::from_callback_event`（真實 graphics／source slot／cast sequence＋回呼 occurrence），由 `FiniteDeliveryHistory` 依 (event, occurrence) 去重；仍在飛行的 S23 sequence 併入 `active_finite_sequences`，raw ring 淘汰後不提早釋放去重鍵。
- **全域整數毫秒起點**：`native_effect_timing::native_elapsed_ns` 先對 now 與 start 各自 floor 到毫秒再相減。S23 飛行與 S39 火球 171 的移動步進原本是先相減再 floor，已改用同一函式；class1、S39 有限特效沿用同一函式。
- **class1 啟動位置**：`sub_44F350` 只有 gfx 0x9B3／0x9B4（2483／2484）進 `loc_44FFFB`（`push 2` 經 `sub_4B9390` 走兩格）；252／236／1801／1811 依序比過 0x2F6、0x71B、0x2F5、0x8D5 後到 `loc_45001A` 直接啟動。`native_class1::position` 因此回傳 S23 wire destination，不再沿 heading 加兩格。2483／2484 沒有加入支援清單。
- **共享 latch**：`FiniteDeliveryHistory` 以 `Weak` 記住每個 key 的 canonical payload；重複項若與 canonical 是同一個 `Arc`，只移除重複排程，不消耗 canonical 的 latch；不同 payload 同 key 仍 discard。

驗證範圍與未驗證項目見 `docs/reports/issue-166.md`；本段只記來源與行為，不代表 E149 實際施法、GUI 或實體音效已驗收。
