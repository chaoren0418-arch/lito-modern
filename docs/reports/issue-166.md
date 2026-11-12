# Issue #166 — T028-4 候選修正在 `src` 重寫（S23／冰箭時鐘、全域毫秒起點、class1 啟動位置、火球 class5 時序）

範圍：T028 第 4 張。工作分支 `task/166`（基底 origin/main `0137ced`）。隔離候選**沒有套用**，也沒有複製候選檔；只把 `維護工具/native-E149-S23-clock*.md`、`native-E149-class1-origin.md`、`native-E149-fireball-*.md` 記載的修正，依現行 `src` 的結構重寫，候選本體只當設計參考（唯讀）。

依使用者 2026-11-09 規定，本機**沒有執行任何編譯產物或測試二進位**：只做 `cargo check`、唯讀靜態逆向（既有 IDA 反組譯文字檔）與 git。沒有連線正式服務或 DB，沒有啟停遊戲程序，沒有改 `docs/REPORT.md`。

## 做了什麼

### 1. S23 class4／冰箭 class5 改用 `NativeSpellClock`

原本 `native_spell_projectiles.rs` 用舊的 rational 時間軸：飛行影格依 catalog 的 24Hz ticks 播一次後停在 source0，抵達時間取 `path.impact_at()` 的理想 wall time，次級在 1..N-1 播完就移除，音效走 legacy 路徑且整次施法只響一次。

改成原生移動回呼時鐘（`light_arrow_effect.rs` 新增 `NativeSpellClock`／`NativeSpellSample`／`NativeSpellCue`，以及 `start_native_s23`、`native_s23_frame`、`native_s23_events`）：

- 首步立即，之後每 60ms 一次移動回呼；延遲的 dispatch 只前進一格，下一個期限從該次 dispatch 起算，不追趕 wall time。
- 每次移動讓飛行來源影格 +1 並循環（1..N-1,0,1…），不依 24Hz ticks，也不停在 source0。source1 的音效 cue 每 N 次移動重入一次。
- 抵達是「實際走到凍結終點的那次移動回呼」。class4 爆炸固定在飛行終點；冰箭在抵達當下解析同 incarnation 目標的座標後固定；冰箭無目標不建立次級。
- cue 順序依原指令：class4 先飛行 cue 再次級（1583 相鄰＝97→99、1799＝719→711）；class5 先次級再飛行 cue（冰箭相鄰＝713→719）。
- 次級沿用既有 native finite `Clip`，保留 source0 的獨立停留（218／219 為 41ms、1800 為 166ms、1798 為 125ms）。次級的 Hit 只記成 `non_audio_cues`（`authority_callback:false`），不再觸發第二次受傷；受傷仍只在抵達時釋放一次。
- 音效改成 `PreparedCue::from_callback_event`（真實 graphics／source slot／cast sequence＋回呼 occurrence）。同一 source1 的不同移動是不同 occurrence，可以重入；同一 occurrence 由 `FiniteDeliveryHistory` 去重。
- `update` 擁有時鐘；`draw_to` 只讀最後一次 sample，不推動時鐘。`update` 產生的 cue 暫存到下一次 `draw_to` 才交出，而且只交出一次（上限 256 筆，超過時該飛行以 `clock_errors` 結束，不會無限累積）。
- 756（技能 50，無聲 class4）走同一個時鐘；129（eruption）不變。
- 既有 History（`retire`／`merge`／`oldest`）保留，新增 `active_sequences()`；`native_world_scene.rs` 的 `active_finite_sequences` 併入仍在飛行的 S23 sequence，raw ring 淘汰後不會提早釋放重入音效的去重鍵。
- 舊的 `start_effect`／`effect_frame`（rational）保留給 inspection 與 battle presentation（`battle_spells.rs` 仍在用），沒有改它的資格。

### 2. 全域整數毫秒起點

新增 `native_effect_timing::native_elapsed_ns(now, start)`：先對兩個端點各自 floor 到毫秒再相減。盤點現行 `src`：class1、S39 有限特效、call lightning、disintegrate、四 profile 特效早就是這個算法；**還在「先相減再 floor」的只有兩處**——S23 飛行（舊 rational 路徑）與 S39 火球 171 的移動步進（`native_batch1_area.rs` 的 `flight.step(now - start)`）。兩處都改用同一個函式，class1 與 `native_attack_many` 也改呼叫它（算式不變，只是集中）。

### 3. class1 啟動位置

`native_class1::position` 原本沿 heading 加兩格。直接讀 4271D92F 的 `sub_44F350`：只有 gfx 0x9B3／0x9B4（2483／2484）進 `loc_44FFFB`（`push 2` 後呼叫 `sub_4B9390`，把結果寫回物件座標）；其餘 gfx 依序比過 0x2F6、0x71B、0x2F5、0x8D5 後到 `loc_45001A` 直接啟動。所以 252（寒冷戰慄）、236（吸血鬼之吻）、1801（STALAC）都從 S23 wire destination 啟動，不加偏移。

- `position()` 改為回傳 wire destination；無效 heading 仍然拒絕（它決定方向圖庫）。1811 原本就直接用 wire destination，不變。
- 沒有把 2483／2484 加進支援清單。
- 八方向選圖、shadow／clothes 圖層、音效、Hit、native callback clock、末幀停留、target incarnation 守衛都沒動。

### 4. 火球 class5（171→218）時序

main 在 T027 已經有原生時序的 `batch1::Flight`（首步立即、60ms 一格、延遲只走一步、1/2/3/4/0 循環、source1 音效每五步重入、抵達時先各接收者的 99 再 171 的 98）。逐項對照 `native-E149-fireball-clock／cycle／fireball25*.md` 與 `sub_42F300` 後，**唯一的差異是上面第 2 點的毫秒起點**，已修正並加測試。其餘行為沒有改。

### 5. 共享 latch（`native-E149-S23-clock-latch.md`）

`FiniteDeliveryHistory` 原本對重複事件一律 `discard()`。如果重複項和還在等待播放的 canonical 是同一個 `Arc`（例如同一個 payload 被重新發布），會把 canonical 的 latch 一起消耗掉。改成每個 key 記一個 `Weak` canonical identity：

- 同一個 payload：只移除重複排程，不消耗 latch。
- 不同 payload、同一個 key：仍然 discard。
- `Weak` 不持有 bank／PCM；identity 過期後，同一個 key 仍然拒絕重播。4096 的上限不變。

### 6. 整合測試配合新時序的更新（`tests/`）

舊測試的期望值是舊 rational 時間軸，和新時鐘不相容，逐項依新行為重寫（**這些測試本機與 CI 都還沒跑過，見「待驗證」**）：

| 檔案 | 更新 |
|---|---|
| `native_attack_skills.rs`、`native_fire_arrow.rs`、`native_wind_cutter.rs`、`native_ice_arrow.rs` 的 `*_zero_distance_zero_damage_and_miss_keep_authority` | 原本「update(0) 後一次跳到 3 秒」假設可以追趕。改成每 1ms 驅動一次、從每次 draw 收 cue。`miss`（無目標，飛 6 格）的發射音效期望從 1 改成 2（循環重入）；`hit`／`zero`（零距離）維持 1。 |
| 同四檔的 Model 測試 `original_class*_flight_impact_audio_and_hurt_are_once` | 6 次移動回呼會再進一次 source1：`sound97`／`sound719` 期望從 1 改成 2。受傷回呼 1 次、次級音效 1 次、次級 Hit 1 次、最終 `active==0` 不變。 |
| `native_ice_arrow.rs` 的 `class5_launch_freezes_target…` | 抵達前補上 120／180／240ms 三次移動回呼（原本直接跳到 300ms 並假設已抵達）。 |
| `native_chill_touch.rs`、`native_vampiric_touch.rs` | 位置期望原本拿 `position()` 自己的回傳值當答案，無法獨立證明。改成絕對 wire 座標 `[32730,32797]`。 |

`native_s23_profiles.rs`、`native_t027_batch3.rs`（756／129）、`native_stalac_layers.rs`、`native_t028_targets.rs` 逐一讀過，期望值不依賴被改掉的行為，沒有修改。

### 7. 文件

`docs/NATIVE-ATTACK-SKILLS.zh-TW.md` 加一節記錄來源與新行為，並註明它取代上文「只發出一次 Sound97」的舊敘述。

## 已驗證

- **`cargo check`**：在 `LiTo381-Modern/` 執行 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`，**結束碼 0**，原文結尾 ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 54.93s``（日誌 `scratch/issue-166/logs/check-3.log`，不追蹤）。合併 origin/main（18 個 commit，自動合併、無衝突）後重跑，仍是**結束碼 0**，結尾 ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 54s``（`check-merged.log`）。`--all-targets` 涵蓋上面改過的 6 個整合測試檔的編譯。變更檔案沒有新增 warning（原本因本次改動變成未使用的 `LightArrowPlayback::without_secondary` 已移除）。`cargo check` 不產生 `test result:`。
- **靜態逆向**（唯讀，`lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm`，SHA256 `2A3A6E10…7DEA9A`，和 #165 用的是同一份；片段在 `scratch/issue-166/*.asm`，不追蹤）。以下是本輪自己讀指令確認的，不是只引用候選說明：
  - `sub_424540`（class4 移動）：累加 16.16 向量、四捨五入後清 x 低位元；未到終點時 `mov ecx,3Ch`＋`call sub_4D65F0`（回呼 `loc_424760` 再呼叫自己）；之後 `call sub_4E5B50`（+18D＝4246CD）；計數歸零時才以 `[esi+158h]` 的 gfx 在 `[esi+2Ch],[esi+30h]`（飛行終點）建立次級並刪除自己。
  - `sub_42F300`（class5 移動）：同樣的移動與 60ms 重排（回呼 `sub_42F550`）；抵達時走目標清單，逐一對解析得到的目標做受傷判斷，並在**該目標的座標** `[edi+2Ch],[edi+30h]` 建立次級；清單空或目標解析不到就不建立；**之後**才 `call sub_4E5B50`（+200＝42F500）。
  - `sub_4E5B50`：`inc` 物件的影格欄位，`>=` 影格數時歸 0，兩條路徑都呼叫 `sub_4E5C00`。
  - 首步立即：`sub_424540` 的呼叫者有 `sub_424780+AD`，`sub_42F300` 的呼叫者有 `sub_42F570+9A`（都是直接 call，不是排程）。
  - `sub_44F350`：比較鏈如上文第 3 點；`loc_44FFFB` 只有 +28、+32 兩個 xref（就是 0x9B3／0x9B4 那兩個 `jz`）。
- **PR #288 CI**（run 43175430275，commit `aa17212`，已含合併 main）結論 **success**；原文見文末「CI 結果」。本張 20 個新 lib 測試在 log 中全部是 `... ok`。

## 推測

- 「source1 的音效 cue 每 N 次移動重入一次」：`sub_4E5B50` 每次都進 `sub_4E5C00` 是讀到的；但 `sub_4E5C00` 內部「進入影格就派送該影格的 cue」這一步本輪沒有逐指令讀，沿用候選說明（`native-E149-fireball-cycle.md`）與 main 既有火球 171 的同一假設。實際聽起來是一聲還是兩聲，還要看 same-ID 抑制政策與音效長度。
- `sub_44F350` 是 class1 物件的啟動函式、而且建構時的座標就是 S23 wire destination：前者依 vtable 資料參照（`004F84C4`）與候選說明，後者依候選說明；本輪只確認了函式內的比較鏈與兩格分支。
- 更新後的 6 個整合測試檔的期望值是依新時鐘手算的（例如 `(32729,32797)→(32735,32800)` 是 6 步、4 影格飛行在第 1、5 次移動進 source1、5 影格的風刃在第 1、6 次），沒有執行過。

## 待驗證／未驗證什麼

- **本機完全沒有跑測試**。CI 只跑 `cargo test --lib`，所以：
  - CI 會涵蓋的新 lib 測試（都不需要共享資源）：
    - `light_arrow_effect::native_s23_clock_tests`（11 個：相鄰 cue 順序、延遲只走一格、循環影格、實際回呼抵達、次級 source0 停留、冰箭無目標、方向圖庫與 clone、零距離、未資格 profile、756 無聲、分數毫秒起點）
    - `native_spell_projectiles::native_s23_history_tests`（3 個）
    - `native_audio_delivery::canonical_latch_tests`（2 個）
    - `native_class1::origin_tests`（2 個）
    - `native_effect_timing::native_origin_tests`（1 個）
    - `native_batch1_area` 新增 `fireball_fractional_launch_origin_does_not_delay_the_sixty_ms_step`
  - CI **不會**執行 `tests/` 下的整合測試；上面第 6 點改過的 6 個檔案只確認能編譯。它們也不在 `維護工具/test-whitelist/tests.tsv`（原本就不在，路徑寫死 `D:\lineage開發\0-客戶端`），本張沒有把它們加進 whitelist。
- **請 Codex 本機執行**（需要 `D:\lineage開發\0-客戶端`）：
  - `cargo test … --test native_attack_skills`、`--test native_fire_arrow`、`--test native_wind_cutter`、`--test native_ice_arrow`、`--test native_chill_touch`、`--test native_vampiric_touch`
  - 沒改但行為相關的回歸：`--test native_s23_profiles`、`--test native_stalac_layers`、`--test native_t027_batch1`、`--test native_t027_batch3 -- --ignored`、`--test native_t028_targets`、`--test native_t028_ground`（#165 要求時鐘改動後重跑 63 的時序）
  - `--lib modern_lito::light_arrow_effect::tests::actual_shared_ice_arrow_all_casters_match_golden_and_launch_once -- --ignored --exact`（#163 要求第 4 張改時鐘後重跑；這個測試走的是舊 rational caster API，本張沒有改它，預期結果不變）
  - 若有測試因新時序失敗，請依本報告的來源判讀，不要為了讓舊期望通過而改回舊行為；手算期望值算錯的話請直接更正測試。
- 候選說明裡列的「原資源四 profile／Model 重建／ring 淘汰／PCM metadata」opt-in 測試**沒有移植**（它用絕對路徑 `include_bytes!` 且需要共享資源）。對應的行為目前只有上述整合測試與純時序 lib 測試涵蓋；Model 重建後接續飛行、source ABA、raw ring 淘汰後的音效去重，沒有專屬測試。
- S39 火球與其他 `many_effects` 的重入音效 key 沒有併入 `active_finite_sequences`（它們靠 `world.many_attacks` ring 保留）；本張只處理 S23。
- 64 個最長路徑與其他 finite 事件共存時的 4096 key 預算、G11 效能，沒有資格化。
- E149 真實施法、其他玩家視角、GUI 像素、實體音效（含 same-ID 抑制下實際聽到幾聲）、Release 建置，都沒有驗證。
- `sub_4DF7C0` 的相對 gfx 禁止受傷家族（動態 graphics base）仍未綁定，本張沒有處理。
- 不將 T028 或 M2 標為完成；`TASKS.md` 的 T028 狀態沒有改。

## 產出路徑

- 追蹤：
  - `LiTo381-Modern/src/modern_lito/light_arrow_effect.rs`
  - `LiTo381-Modern/src/modern_lito/native_spell_projectiles.rs`
  - `LiTo381-Modern/src/modern_lito/native_effect_timing.rs`
  - `LiTo381-Modern/src/modern_lito/native_class1.rs`
  - `LiTo381-Modern/src/modern_lito/native_batch1_area.rs`
  - `LiTo381-Modern/src/modern_lito/native_attack_many.rs`
  - `LiTo381-Modern/src/modern_lito/native_audio_delivery.rs`
  - `LiTo381-Modern/src/modern_lito/native_world_scene.rs`
  - `LiTo381-Modern/tests/native_attack_skills.rs`、`native_fire_arrow.rs`、`native_wind_cutter.rs`、`native_ice_arrow.rs`、`native_chill_touch.rs`、`native_vampiric_touch.rs`
  - `docs/NATIVE-ATTACK-SKILLS.zh-TW.md`
  - 本報告
- 不追蹤（`scratch/issue-166/`）：`sub_424540.asm`、`sub_42F300.asm`、`sub_4E5B50.asm`、`sub_44F350.asm`（反組譯片段）、`light_arrow.diff`（候選與現行的唯讀比對）、`logs/check-*.log`。

本張沒有宣稱缺任何資源，沒有用到 PakViewer。

## 引用的唯讀來源（規則 7）

- lito-launcher-server（上層 repo，未 checkout、未寫入）：`lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm`（SHA256 `2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A`）。
- 隔離候選（本機 `維護工具/candidates/`，不在 git，唯讀）：`native-E149-freeze50-20261024-103539` 的 `light_arrow_effect.rs`、`native_spell_projectiles.rs`，依 contract `effective_overlay_paths` 取檔。只參考設計，沒有複製檔案；名稱改為 snake_case，並依 main 現行結構（eruption、History、756、冰箭 233 項額度）重寫。

## 給下一棒的提示

- Claude 審查重點：
  1. `native_spell_projectiles.rs` 的 cue 交付改成「update 產生、下一次 draw 交出一次」。Model 每幀固定先 update 再 draw_to，行為等同；直接使用 `Projectiles` 的呼叫端若只 draw 不 update，畫面會停在最後一次 sample（這是刻意的：draw 不推動時鐘）。
  2. S23 音效從 legacy 路徑改成 native prepared cue，會進 `FiniteDeliveryHistory`。`native_world_view.rs` 沒有改；它原本就用 `world.projectiles` 的 sequence 保留 key，本張另外把飛行中的 sequence 併進 `active_finite_sequences`。
  3. `FiniteDeliveryHistory.seen` 從 `BTreeSet` 改成 `BTreeMap<key, Weak<Payload>>` 是全域行為（所有 finite 音效共用），只影響「重複項與 canonical 是同一個 Arc」的情況。
- 這張改了 6 個整合測試的期望值但沒有執行；合併前或合併後應該排一張 test 任務交給 Codex 跑上面「待驗證」的清單。
- T028 四張都有報告後，`TASKS.md` 的 T028 狀態與 `docs/SKILLS-STATUS.md` 由主控對話統一更新；本張沒有動。

## CI 結果

PR #288，run 43175430275（`pull_request`，head `aa17212`），結論 **success**：`cargo check --locked --all-targets` 與 `cargo test --locked --lib --no-fail-fast` 都通過。

`test result:` 原文：

```
test result: ok. 2288 passed; 0 failed; 597 ignored; 0 measured; 0 filtered out; finished in 24.19s
```

- 597 個 ignored 是需要共享資源的測試，**不算通過**。
- 本張的 20 個新 lib 測試在 log 中逐一為 `... ok`：`native_s23_clock_tests` 11 個、`native_s23_history_tests` 3 個、`canonical_latch_tests` 2 個、`native_class1::origin_tests` 2 個、`native_origin_tests` 1 個、`fireball_fractional_launch_origin_does_not_delay_the_sixty_ms_step` 1 個。
- CI 只跑 `--lib`：`tests/` 下改過的 6 個整合測試檔**只有編譯通過，沒有執行**。
- 本報告之後的 commit 只改這份報告；它的 CI 結果以 PR 頁面為準。