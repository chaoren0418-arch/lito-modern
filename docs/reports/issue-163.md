# Issue #163 — T028-1 攻擊與目標類當版補驗

範圍：技能 6、8、9、10、11、18、19、28、35、57，以及全 caster golden。工作分支 `task/163`；全部修改及產物位於本 worktree。未套用隔離候選、未修改 `docs/REPORT.md`，未操作遊戲程序、正式服務或 DB。

## 做了什麼

- 新增 `LiTo381-Modern/tests/native_t028_targets.rs`：對 10 個技能核對其他目標的請求 bytes、座標、範圍邊界、自己／無目標規則；使用既有 NPC fixture 與另一個 player-shaped actor（含高位元 object ID）。驗證失效 incarnation、死亡、隱身、離開與未學技能均拒絕，不修改權威資料。
- 核對 2 秒 Pending 邊界，以及施法 action 後收到拒絕訊息的優先序；低非零 MP 不由客戶端預判伺服器結果。
- 8／9／11／18、19／35／57 的 TW 與 legacy 特效分開核對其他目標、音效一次、凍結位置、每一個整數毫秒 deadline 前 1ns／當下、末幀停留、自然退役與不重播。延遲回呼只前進一個正時長影格，換圖退役；不將有限特效時間當成毒、治癒或死亡的伺服器期限。
- 10／28 對兩種目標核對 class1 的 41／124／249／415／456／581ms 邊界，Hit 僅一次且綁定正確目標，不預測 HP／MP。
- 在 `light_arrow_effect.rs` 增加冰箭全 caster 測試：先執行既有獨立 C# RGBA golden，再核對冰箭 profile 的 14 外觀 × 8 方向、所有毫秒取樣與精確 Hit／結束邊界。保留需要真資源的 `ignored` 守門，執行時明確使用 `--ignored --exact`，不得把一般測試的 ignored 當成通過。
- 全 caster 首跑揭露冰箭載入上限問題：原 class4 上限 229 僅含四個飛行圖庫；冰箭需要 224 caster 圖庫＋8 飛行圖庫＋1 次級圖庫＝233。僅對 1797 profile 增加 4 個項目額度，其他 profile 的 229 上限與所有 encoded／RGBA bytes 上限保持不變。
- 修正補驗發現的 TW 治癒資源資格缺口：`tower_assets.rs` 原先只承認舊版 launcher pack，導致當版本機 19／35／57 的 TW 特效被拒絕。新增一組精確的 pack SHA／text SHA／解碼長度配對，舊版配對仍保留；所有既有動作、衣層、SPR 雜湊與空影格檢查不變。新增世代配對反例與目前五筆治癒定義原始 bytes SHA 測試，bank 的 catalog 診斷改為實際獲准文字 SHA。

目前唯讀資源身分：`D:\lineage開發\0-客戶端\launcher\TW13081901.pak`，1,406,534 bytes，SHA256 `92b512a32349d0775e1036f1832e21f1141ae66e64821d6978f0879246e59ca7`；解碼文字 5,990,880 bytes，SHA256 `28c4fc600833fb8c609ab195a67ff926e8ffa571100d8d4b389a23be8c2c8db8`。僅資格化 11722–11726 五筆，並未授權整份新版 pack 的其他技能。11722 是同一既有治癒 bank 的必需相依，沒有新增技能 1 的行為。

## 第 2 次嘗試（2026-11-12，claude-local）

- 分支落後 main 35 個 commit，`git merge origin/main`（main 頂端 `e40c68a`）自動合併、無衝突；`light_arrow_effect.rs`、`tower_assets.rs` 兩個重疊檔已逐段核對，本張的冰箭 233 項額度、TW 治癒世代配對與 main 新增的 S39 火球路徑、ReviewedEarthJail 策略並存。
- 補上 `維護工具/test-whitelist/tests.tsv`：`native_t028_targets` 7 個測試（需共享資源者第 3 欄 `yes`：治癒定義 SHA、延遲回呼、有限特效邊界、寒冷／吸血），讓 `run-local-test-suite.ps1` 會實際跑到本張新測試。
- 合併後 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` **結束碼 0**，原文結尾 ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 22s``（日誌 `scratch/issue-163/check-merged.log`，不追蹤）。
- 依使用者 2026-11-09 規定，本輪**未在本機執行任何測試二進位**；下方「已驗證」的 `test result:` 是第 1 次嘗試於 2026-11-06（基底 `59f0c2b`，規定生效前）的結果。合併後的測試結果以 PR CI（`cargo test --lib`，含 `tw_healing_generations_reject_mixed_changed_and_unknown_identity`）為準；需要共享資源的整合測試與 ignored golden 列入「待驗證」。

## 已驗證

2026-11-06，本張 10 組 Debug 測試合計 **33 passed／0 failed／0 ignored**，所有 Cargo test 結束碼 **0**，T012 解析門檻均為 PASS。全 caster 測試會先呼叫原 C# golden 前置測試，不重複灌入測試總數。來源基底為 `59f0c2bcd7dc4435ba43186f7767130ab7869af1` 加本分支變更。

最終 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` **結束碼 0**；原文結尾為 ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 22s``，日誌 `check-final.log`。`cargo check` 不產生 `test result:`；`git diff --check` 亦通過。

| 日誌 | 執行範圍 | 結束碼 | `test result:` 原文 |
|---|---|---:|---|
| 001.log | 新增 T028 目標矩陣 | 0 | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.91s` |
| 002.log | native_ice_arrow | 0 | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.28s` |
| 003.log | native_holy_weapon | 0 | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.60s` |
| 004.log | native_poison_cure | 0 | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.36s` |
| 005.log | native_chill_touch | 0 | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.58s` |
| 006.log | native_vampiric_touch | 0 | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.57s` |
| 007.log | native_skills：higher | 0 | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 5.78s` |
| 008.log | native_turn_undead：指定原圖／到期測試 | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 6.20s` |
| 009.log | TW 世代配對反例 | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2503 filtered out; finished in 0.00s` |
| 010.log | 冰箭全 caster＋原 C# golden | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2503 filtered out; finished in 6.90s` |

`filtered out` 均為範圍外測試，不算通過。JSON 總表是 `target/current/verification/issue-163/results-debug.json`。編譯既有 warnings 未視為錯誤，也未為消除它們擴大修改範圍。

### 怎麼驗證

在 `LiTo381-Modern/` 執行；共同 Cargo 參數為 `--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`。測試 runner 清除繼承的 `LITO_*`，僅明確設定 `LITO_SHARED_RESOURCE_ROOT=D:\lineage開發\0-客戶端`；不使用既有 peer、正式連線設定或音效裝置。每項分開啟動一個 harness，依 T012 門檻核對 `running` 與 `test result:`，任何 0 tests、failed > 0、解析失敗或非零結束碼均不算通過。

```powershell
$env:LITO_SHARED_RESOURCE_ROOT = 'D:\lineage開發\0-客戶端'
New-Item -ItemType Directory -Force target/current/verification | Out-Null
$common = @('--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current')
cargo check @common --all-targets
cargo test @common --test native_t028_targets
cargo test @common --test native_ice_arrow
cargo test @common --test native_holy_weapon
cargo test @common --test native_poison_cure
cargo test @common --test native_chill_touch
cargo test @common --test native_vampiric_touch
cargo test @common --test native_skills higher
cargo test @common --test native_turn_undead original_turn_undead_frames_pixels_sound_once_and_zero_terminal_retirement_after_death -- --exact
cargo test @common --lib modern_lito::tower_assets::tests::tw_healing_generations_reject_mixed_changed_and_unknown_identity -- --exact
cargo test @common --lib modern_lito::light_arrow_effect::tests::actual_shared_ice_arrow_all_casters_match_golden_and_launch_once -- --ignored --exact
```

### 首跑失敗與修正

失敗均保留，未列入通過總數：

- 新增矩陣曾因 `id.to_le_bytes()` 型別不明而編譯退出 101（無 `test result:`），已改明確 `u32`。
- 新增矩陣曾是 `4 passed; 2 failed`：毒咒時間表誤採均勻 125ms。唯讀擷取原 `list.spr` 的 745 定義後，依 source0..7 的 ticks `[3,3,4,4,4,4,4,4]` 修正為播放順序 `[125,166,166,166,166,166,166,125]`ms；未更改產品時鐘。
- 較高治癒首跑 `test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 9 filtered out; finished in 5.98s`，退出 101。原因是舊 TW generation pin；新增精確身分配對後重跑。
- 全 caster 首跑 `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2502 filtered out; finished in 10.39s`，退出 101。原 C# golden 前置比對已過，冰箭載入在 `Bound("sprite entries")` 失敗；只修正冰箭 233 個項目額度後重跑。

對應保留檔為 `001-initial-compile-failure.log`、`001-initial-boundary-failure.log`、`007-initial-tw-generation-failure.log`、`009-initial-ice-budget-failure.log`。唯讀來源盤點在 `resource-inspection.log`。

## 推測

離線封包、原資源與當版函式測試通過時，可以證明相應客戶端處理路徑；不能推論 E149 真實施法、伺服器目標資格或原生 GUI 已驗收。

## 待驗證／未驗證什麼

- 正式 E149 互動、實際其他玩家／NPC 施法成功與拒絕、自然機率分支、伺服器狀態到期、實體音效與完整 GUI；本任務未啟動客戶端或伺服器。
- 第 2／3／4 張內容仍由各自 Issue 處理，尤其 S23／冰箭 NativeSpellClock 與 class1 啟動位置；本張有限時鐘測試只證明現行路徑，不能替代第 4 張的原生時序資格。
- 本張不將 T028 或 M2 整體標完成。
- 合併 main（`e40c68a`）後的整合測試尚未重跑：請 Codex 本機以 `LITO_SHARED_RESOURCE_ROOT=D:\lineage開發\0-客戶端` 執行 `run-local-test-suite.ps1`（含本張 7 筆 whitelist），以及 `--lib modern_lito::light_arrow_effect::tests::actual_shared_ice_arrow_all_casters_match_golden_and_launch_once -- --ignored --exact`；CI 無共享資源，不涵蓋這些項目。
- 本輪執行 Debug 檢查與測試，Release 測試尚未執行；舊 TW pack 的完整資源重播也未執行（僅保留其精確身分配對並測試配對守門）。

## 產出路徑

- 追蹤：`LiTo381-Modern/tests/native_t028_targets.rs`、`LiTo381-Modern/src/modern_lito/light_arrow_effect.rs`、`LiTo381-Modern/src/modern_lito/tower_assets.rs`、`維護工具/test-whitelist/tests.tsv`、本報告。
- 不追蹤：`LiTo381-Modern/target/current/verification/issue-163/`（編譯／測試日誌與門檻結果）；既有測試自行產生的證據留在 `target/current/verification/`。
- 一次性 runner：`scratch/run-163.ps1`。直接從 `維護工具/run-local-test-suite.ps1` AST 取得 `Measure-TestResult` 與 `Invoke-SuiteStep`，沿用其結束碼、running、結果格式與 Exact 門檻；僅為 golden 子程序明確加入唯讀 `LITO_SHARED_RESOURCE_ROOT`，TEMP／TMP 指向本 worktree 的 `scratch/`。

## 給下一棒的提示

Claude 請審查本張離線補驗；第 4 張修改時鐘後，重跑本張新測試、冰箭及 class1 既有測試，依獨立來源判讀差異，不為維持舊測試而保留已被來源否定的行為。需要實際服務互動的剩餘資格由調度器另交本機隔離 harness，不連正式服務。完整 GUI 仍留 T030。
