# Issue #148 — T027 批 4：技能 55、56、59、61、62、64、65、66

日期：2026-11-12。分支：`task/148`，PR #283。runner：claude-local（第 1 次嘗試）。本張只做第四批八個技能，T027 與 M2 整體都還沒完成。

## 做了什麼

| ID | 送包（未加密 body，含 opcode）與選取 | 收包與呈現 |
|---|---|---|
| 55 狂暴術 | `14 06 06 + d(actor)`，7 bytes；沒選目標就用自己，原畫面菱形；reuse 10ms | S37 3943 照常收包（凍結收包位置）；3943 是八方向，**呈現未接**，繪製時記為 omitted，不用通用特效替代 |
| 56 疾病術 | `14 06 07 + d h h`，11 bytes；其他活體，三格；reuse 500ms | S37 2230：新增精確定義（11 格、各 3 ticks、Sound839 在 source1、attr128 → Multiply） |
| 59 冰雪暴 | `14 07 02`，3 bytes，無尾段（selector-only）；reuse 1500ms | S39 action18／type0／gfx757 改為接受：保留施法動作、名單順序與 miss；25 節點呈現**未接**（omitted） |
| 61 返生術 | `14 07 04 + d(corpse)`，7 bytes；和 41 共用明確屍體意圖，原畫面菱形；reuse 10ms | S37 3944（八方向，**未接**）；成功光效 S37 230：新增精確定義（7 格、Sound124 在 source1、Screen）。光效不算復活成功，actor 的死亡狀態不因 S37 改變 |
| 62 震裂術 | `14 07 05`，3 bytes，無尾段（selector-only）；reuse 1500ms | S39 action18／type0／gfx1812 改為接受；1812／1813／1814 三層呈現**未接**（omitted） |
| 64 魔法封印 | `14 07 07 + d h h`，11 bytes；其他活體，三格；reuse 500ms | S37 2177 沿用既有精確定義（17 格、Sound808） |
| 65 雷霆風暴 | `14 08 00 + d h h`，11 bytes；其他活體，十格；reuse 2000ms | S39 action18／type8／gfx3924 改為接受；收包時凍結**第一個**目標位置（miss 也算；第一個目標不存在時依 heading 退一步，不會改取第二個）；呈現**未接**（omitted） |
| 66 沉睡之霧 | `14 08 01 + d h h`，11 bytes；其他活體，十格；reuse 1500ms | S37 760：新增精確定義（21 格、各 6 ticks、Sound136 在 source1、Screen） |

- 沿用既有的施法者守衛：已學會、存活、HP／MP 有值且大於 0、沒有移動封鎖、可見目標。MP／HP 成本、魔法寶石 40318、機率、MR、狀態期間（320／64／16／32 秒）、同盟／隊伍資格、復活 HP 與經驗都由伺服器決定，本機不預扣、也不預測成功。
- Ctrl／LFC1：既有的 `native_forced_combat::object_target` 已經涵蓋 7／11 byte 的 actor body，所以 55／56／61／64／65／66 不用另外改；59／62 是 3 bytes，不會附加 LFC1（測試有驗）。
- 選單：八個技能都有名稱，不再顯示「尚未支援」。61 走 `Target::resolve_for_skill` 的屍體路徑：活體選取在死亡後不能拿來施放 61，屍體選取在復活後也會失效。
- 新增的 2230／760／230 定義放在**獨立**的延遲載入 bank（`BATCH4_GRAPHICS`、`qualify_batch4`、`load_reviewed_batch4_finite`）。原本的 additional bank 是全部一起載入，如果新定義失敗就會連帶停掉既有 25 個有限特效；分開放就不會互相影響。這個 bank 只在世界中真的出現 2230／760／230 的 S37 時才載入，載入錯誤會出現在 receipt 的 `load_error`／`batch4_bank`。
- S39 decode 只放行這三組精確的 (gfx, type, action)；同一個 gfx 用錯的 type 或 action 仍然隔離，不會改動世界（測試有驗）。

## 來源核對
- `TASKS.md` T027、`docs/PROTOCOL-CONTRACT.md` §6.3（八技能的 row／column 與尾段形狀）、`docs/SKILLS-STATUS.md`。
- 歷史候選說明只當設計參考：`維護工具/native-E149-{berserk55-disease56,blizzard59,resurrection61,earthquake62-silence64,storm65-sleep66}.md`，以及 `prepare-native-E149-*.py` 裡的 reuse／距離值。沒有執行舊的 prepare 腳本，也沒有套用候選整包。
- 原始定義：唯讀 `D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr`，SHA256 `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127`（4,067,417 bytes，和載入器要求的正式 catalog 相同）。用 `scratch/issue-148/spec.py`（`python -I`，唯讀）從原始列產生 `Spec` 位元組。產生器先拿既有、已審過的 2228 列自我核對，產出和現行原始碼**逐字相同**，之後才產生 2230／760／230。
- 原始列（摘錄）：`#2230 1 disease m … 0.animation(0 11,0.10:3 0.0:3[839 …) 102.type(0) 104.attr(128)`、`#760 1 fog of sleeping … 0.fire(0 21,0.20:6 0.0:6[136 …) 104.attr(8)`、`#230 1 lfs orange … 0.Breath(0 7,0.0:4 0.1:2[124 …) 104.attr(8)`、`#757`（23 格，Sound123，Hit 在 source3）、`#1812`（16 格，101.shadow(1813)、105.clothes(1 1814)）、`#3924`（13 格，`<1348` 重疊音效）、`#3943`／`#3944`（`8` 個 bank）。
- 精靈檔是否存在：用 `scratch/issue-148/idx2.py` 以 28 bytes 一筆唯讀解析 `D:\lineage開發\0-客戶端\Sprite*.idx`（Sprite.idx、Sprite00–15.idx 都查了）。結果：`Sprite.idx` 有 `2230-0.spr`（75,918 bytes）、`760-0.spr`（485,556）、`2177-0.spr`（72,761）、`3943-0.spr`（154,014）、`3944-0.spr`（176,858）；`230-0.SPR`（大寫副檔名，和既有可正常載入的 `226／227／228-0.SPR` 相同）。資源沒有複製進 repo。

## 已驗證
- 本機 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`：**exit 0**，原文 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2m 48s`（`scratch/issue-148/check1.log`）；加上測試修改後再跑一次：**exit 0**，`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 4.68s`（`check2.log`）。變更的檔案沒有新的 warning。
- 新增的 lib 測試 `native_skills::t027_batch4_tests`（9 案，全部不需要資源，CI 的 `cargo test --lib` 會跑）：
  1. `eight_wire_shapes_are_exact_and_costs_are_not_predicted`：八個 body 逐 byte 比對、長度 3／7／11、HP／MP 為 1 時仍送包（不預扣）、世界不被改動、LFC1 只適用 actor body、55 自用／失效 ID 不會退回自用、reuse 值。
  2. `caster_gates_reject_without_partial_mutation`：7 種施法者狀態 × 8 技能，失敗時不改動世界。
  3. `target_kind_and_range_follow_the_formal_record`：活體／屍體／不在場、三格與十格（含對角）、55／61 原畫面菱形 18／20。
  4. `menu_labels_and_corpse_intent_cannot_be_upgraded`：選單名稱、61 屍體意圖、59／62 丟掉 hover 目標、incarnation／連線失效。
  5. `object_effects_are_atomic_and_never_grant_status_cost_or_life`：S37 3943／2230／2177／760／3944／230 截斷時原子拒絕、收包位置凍結、不改 HP／MP／封鎖／死亡、不算施法 ACK。
  6. `centered_and_area_s39_keep_order_misses_and_the_formal_profile_only`：757／1812／3924 截斷原子性、名單與 miss、`CastAction` 回饋、3924 首目標位置與 heading 退一步、錯 type／action 仍隔離。
  7. `unqualified_presentations_are_omitted_not_substituted`：3943／3944 與 757／1812／3924 繪製時記為 omitted，沒有畫面、音效或本機 Impact。
  8. `rejections_keep_their_codes_and_silence_times_out`：278／279／280／299 拒絕碼，兩秒沒回覆就是 NoResponse。
  9. `batch4_finite_definitions_are_exact_and_visual_only`：用原始三列文字 qualify；整數 callback 的畫面結束時間 2230=1375ms、760=5250ms、230=830ms；改掉音效、attr、ticks 或 bank 數都會被拒絕；三個 gfx 不在舊 bank 的清單裡。
- CI 結果：見下方「CI」一節。

### CI
- 第 1 輪（commit `task #148: T027 批 4…`，run 43170045183）：`cargo check` 通過；`cargo test --lib` **失敗**，原文 `test result: FAILED. 2237 passed; 1 failed; 591 ignored; 0 measured; 0 filtered out; finished in 22.61s`。本批另外 8 案都是 ok；失敗的是 `batch4_finite_definitions_are_exact_and_visual_only`，panic 在測試輔助函式 `qualify` 的 `parse_for_inspection(...).unwrap()`（native_t027_batch4_tests.rs:337）。也就是某一筆故意改壞的定義列在解析階段就被拒絕，沒有走到 qualify。修正方式：解析失敗也算「不合格」（回傳 Err），原始三列如果解析失敗會另外 panic 並附上訊息。這不是放寬檢查：改壞的列不論在哪一層被拒絕都不能通過。修正後本機 `cargo check …--all-targets` exit 0（`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 49.36s`）。
- 第 2 輪：推送後 PR 變成 CONFLICTING（main 前進到 `ed9a9d8`，含 #184 新增的通用「收到的 S37 有限特效」逐 gfx bank），GitHub 不會對衝突中的 PR 跑 CI。依常設授權 `git merge origin/main`，兩個檔案有衝突：
  - `native_skill_effects.rs`：採用 main 的 `finite_banks` 通用路徑，再把本批的 `batch4_bank` 接回去；`BATCH4_GRAPHICS` 從通用路徑的候選中**排除**。原因：通用 qualifier 只接受 attr 104 為 0／8，所以 2230（attr128）會被它拒絕；760／230 雖然會被它接受，但本批保留逐字比對的精確定義。2230 的混色維持 Multiply（main 的 mix 判斷順序是 finite_over → 2175|2228|2230 Multiply → Screen；2230 不會進 finite_over）。另外確認 3943／3944 有 `109.effect` 屬性，通用路徑也會拒絕，所以它們仍然是「呈現未接」。
  - `tower_assets.rs`：兩邊的新函式都保留（`load_reviewed_batch4_finite` 與 main 的 `load_finite`）。
  - 合併後本機 `cargo check …--all-targets`：**exit 0**（`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 3m 55s`）。
- 第 3 輪（run 43172382221，`7c46bce`）：`cargo check` 通過；`cargo test --lib` **失敗**，原文 `test result: FAILED. 2246 passed; 1 failed; 595 ignored; 0 measured; 0 filtered out; finished in 21.43s`。本批 8 案 ok，失敗的仍是 `batch4_finite_definitions_are_exact_and_visual_only`，這次訊息清楚：`original rows: parse: list.spr line 1: SpriteCount { declared: 11297, actual: 3 }`。原因是測試摘錄的 list 檔頭宣告了 11297 個 sprite，但只放了三列，而解析器會檢查數量（這行檔頭是照抄歷史候選模板 `維護工具/templates/*definition_tests.rs`，那些模板從沒跑過）。修正為 `3 0 0`。
- 第 4 輪：main 又前進到 `3f6de7a`（#172 新增 `native_t028_ground` 白名單），再合併一次；只有 `tests.tsv` 衝突，兩邊的列都保留。合併後本機 `cargo check …--all-targets`：**exit 0**（`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1m 32s`）。CI 結果以 PR #283 最新 commit 為準。

## 推測
- 送包形狀、範圍與 reuse 是依正式 contract 與候選說明做的靜態對照，不代表正式 E149 服務一定接受（同盟資格、材料、機率等）。
- 2230／760／230 的定義與原始列逐字相同；實際解圖、混色與音效準備要等原資源 opt-in 案跑過才算驗證。

## 待驗證／未完成
- **本機沒有執行任何 `cargo test` 或 `run-local-test-suite.ps1`**（2026-11-09 規定不得執行編譯產物）。lib 測試以 PR CI 為準；`run-local-test-suite.ps1` 的 Full 套件與原資源 opt-in 案 `native_t027_batch4::original_batch4_object_effects_use_exact_banks_blends_and_sounds`（2230／760／230／2177 實際解圖、Multiply／Screen、各一次 Sound839／136／124／808、結束時間）需要交給 Codex 執行。
- 呈現還沒接：55 的 3943 與 61 的 3944（八方向 S37，需要在收包時保存 heading，並讓 bank 允許方向載入）、59 的 757（25 節點：heading 0..7 半徑 5／3／7 加中心）、62 的 1812（1812 Over／1813 Shadow／1814 Screen 三層，Sound716 在 source1）、65 的 3924（Screen 13 格，`<1348` 重疊音效）。目前這些在收包後都記為 omitted，不會用替代特效。
- 61 的 S24/321 復活詢問與 C23 同意／拒絕流程（被復活方的 UI）沒有做；目前只做施法方的請求與收包。
- 沒有啟動遊戲或客戶端、沒有連正式服務或 DB、沒有做 GUI、DPI、滑鼠鍵盤、音效裝置或和原版並排的驗收；G06 歷史矩陣與 T030 不在本張範圍。

## 產出路徑
- `LiTo381-Modern/src/modern_lito/native_skills.rs`：送包、selector-only、reuse；測試模組註冊。
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`：61 屍體意圖、選單名稱。
- `LiTo381-Modern/src/modern_lito/native_attack_many.rs`、`native_batch1_area.rs`：S39 三個 profile、3924 首目標位置。
- `LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs`、`tower_assets.rs`、`native_skill_effects.rs`：2230／760／230 獨立 bank 與 Multiply。
- `LiTo381-Modern/src/modern_lito/native_t027_batch4_tests.rs`：9 個 lib 測試。
- `LiTo381-Modern/tests/native_t027_batch4.rs`、`維護工具/test-whitelist/tests.tsv`：1 個原資源 opt-in 案與白名單。
- `docs/SKILLS-STATUS.md`：只改本批八列（56／64／66 標「已接」，其餘五列標「部分接」）。沒有改 `docs/REPORT.md`。

## 給下一棒的提示
- 雲端 Claude 審查 PR #283；Codex 跑 `run-local-test-suite.ps1`（Full）與上面的原資源 opt-in 案，把結束碼和 `test result:` 原文補進本報告。
- 補呈現時：3943／3944 要在收包時保存 actor heading，並開一條「只放行 EightWay＋image_count 8」的 qualifier，不要放寬既有的 Undirected 限制；757／1812／3924 可以參考 `native_earth_jail.rs`（多層、整組預檢）與 `native_tornado.rs`（固定位置群組）。3924 的位置已經存在 `Cue.target_origin`。
- 61 的復活詢問流程要另開任務：S24/321 綁定連線、世代、死亡 action 序號，C23 同意／拒絕不可預設同意。
