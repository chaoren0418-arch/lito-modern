# Issue #150 — T027 批 6：技能 76、79、80、87、88、89、90、91

日期：2026-11-12。分支：`task/150`，PR #298。runner：claude-local（第 1 次嘗試）。本張只做第六批八個技能；T027 六張的送包到此都有了，但 T027 與 M2 整體還沒完成（各批都留有未接的呈現與同意流程，見各批報告）。

## 做了什麼

| ID | 送包（未加密 body，含 opcode）與選取 | 收包與呈現 |
|---|---|---|
| 76 集體緩速術 | `14 09 03 + d(actor) h(x) h(y)`，11 bytes；其他活體，十格；reuse 3000ms | S37 3932（13 格、overlap 音效 1349）：新增精確定義，Screen，固定在收包位置 |
| 79 靈魂昇華 | `14 09 06 + d(actor)`，7 bytes；沒選目標就送自己，原畫面菱形；reuse 10ms | S37 3935（19 格、overlap 音效 1356）：新增精確定義 |
| 80 冰雪颶風 | `14 09 07`，3 bytes，無尾段；reuse 3000ms | S39 action18／type0／gfx3933 改為接受：保留施法動作、名單順序與 miss；31 格呈現**未接**（omitted） |
| 87 衝擊之暈 | `14 0A 06 + d(actor) h(x) h(y)`，11 bytes；其他活體，一格；自己的步態必須是 50（雙手劍）或 54；reuse 8000ms | S37 4434（6 格、Sound1498）：新增精確定義。眩暈物件 4183 與 S25 mode 22／23 是 main 既有的，本批沒動 |
| 88 增幅防禦 | `14 0A 07`，3 bytes；reuse 3000ms | S37 4824（19 格、overlap 音效 3089）：新增精確定義 |
| 89 尖刺盔甲 | `14 0B 00`，3 bytes；reuse 5000ms | S37 4648（22 格、overlap 音效 1924）：新增精確定義 |
| 90 堅固防護 | `14 0B 01`，3 bytes；reuse 3000ms | S37 5831（14 格、overlap 音效 3973、一個 Hit）：既有的通用 received-finite 規則就能收，本批沒有改它 |
| 91 反擊屏障 | `14 0B 02`，3 bytes；自己的步態必須是 50；reuse 3000ms | S37 5832（17 格、overlap 音效 3974、一個 Hit）與觸發時的 10710（6 格、無音效、一個 Hit）：同上，走既有通用規則 |

- 沿用既有的施法者守衛：已學會、存活、HP／MP 有值且大於 0、沒有移動封鎖、不是觀戰、目標可見。
- **HP 成本不在本機判斷**：88 是 50、89 是 120、90／91 是 100。伺服器要求 HP 嚴格大於成本（相等也拒絕），不足回 S10 279；MP 不足回 278。本機在邊界兩側送的都是同樣三個 bytes，不預扣 HP／MP，也不預測拒絕；279（「HP 不足」）與 278（「MP 不足」）沿用既有的拒絕回饋。
- **步態條件**（87：50 或 54；91：50）是原客戶端送包前就有的條件，所以本機先擋；實際武器種類、90 的盾牌／臂甲資格（伺服器沒有合格裝備時送 S_MESSAGE「你並未裝備盾牌」）、機率、傷害、期間（192／64／1200 秒、眩暈 2–4 秒、緩速 64 秒、冰凍 32 秒）、76 的兩顆魔法寶石都由伺服器決定。
- **五個帶 `109.effect` 列的特效**（3932、4434 是 `1 -1`；3935、4824、4648 是 `0 -1`）：通用的 received-finite 規則只收 102／104 兩種屬性，原本這五個會被記為錯誤並 omitted。本批在 `native_additional_finite_effects.rs` 加了逐列釘死的原始定義（header、屬性、每格 subframe／ticks、唯一的音效事件與它的種類），`tower_assets::load_finite` 對這五個 ID 改用釘死的定義，其他 ID 的規則不變。之後一樣走既有的逐 ID 快取（各自的 bank、各自的錯誤、LRU 上限、attr8 → Screen），所以其中一個定義或 SPR 不合時只有那一個 omitted，不影響其他特效。`<n` 是原始的「可重疊音效」，不是子特效。
- 80 的 3933 只加進 S39 的接受清單（type0／action18），和 59／62／70 一樣以收包時的施法者位置為中心（`target_origin` 為空），沒有呈現。
- 選單：八個技能都有名稱，不再顯示「尚未支援」；80／88／89／90／91 加進 `selector_only`，選單會先丟掉 hover 的角色目標。
- Ctrl／LFC1 與目標政策：76／87（11 bytes）與 79（7 bytes）是角色 body，既有規則已涵蓋；五個三 bytes 的不會附加。這部分沒有改程式，只加了測試。

## 來源核對
- `TASKS.md` T027、`docs/PROTOCOL-CONTRACT.md` §6.3、`docs/SKILLS-STATUS.md`、`docs/REUSE-MAP.md`。
- 歷史候選說明只當設計參考（沒有執行舊的 prepare 腳本，也沒有套用候選整包）：`維護工具/native-E149-{greater75-massslow76,advance79-development,freezing80-shock87,selector80,reduction88-bounce89,solid90-counter91}.md`。
- 唯讀參考相鄰 repo lito-launcher-server（`D:\lineage開發\3-lito開發`，讀工作目錄現況，沒有改動、沒有 checkout）：`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/BattleSpellCatalog.java` 的八筆定義——76（row9／col3、gfx3932、MP20、reuse3000、射程 10、範圍 3、64 秒、寶石×2、失敗 280）、79（9／6、3935、MP20、reuse10、1200 秒、280）、80（9／7、action18、3933、MP60、reuse3000、client clock 0、範圍 10、32 秒）、87（10／6、4434、物件 4183、MP15、reuse8000、射程 1、步態 50／54、2+0..2 秒、280）、88（10／7、4824、MP5／HP50、reuse3000、192 秒）、89（11／0、4648、MP5／HP120、reuse5000、64 秒）、90（11／1、5831、MP10／HP100、reuse3000、64 秒）、91（11／2、5832、MP10／HP100、reuse3000、64 秒、武器 type3、觸發 10710）。
- 原始特效定義：`D:\lineage開發\3-lito開發\lito脫殼資源\resources\decoded\Text\row15937-list.spr`（4,067,417 bytes，SHA-256 `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127`，和 `tower_assets` 釘住的 shared catalog 大小相同）。用唯讀 python 腳本（`scratch/issue-150/defs.py`、`lit.py`）取出 3932／3935／4434／4824／4648／5831／5832／10710 八列的原始 bytes（含 CRLF 與縮排），逐字放進 lib 測試當輸入。本批沒有宣稱缺資源，所以沒有做 pak 清查。
- TW 3.81 與 E149 的差異以 `docs/PROTOCOL-CONTRACT.md` 為準；本批沒有發現需要改契約的地方。SQL 表的 76「射程 -1」保留原值，本機依伺服器定義採十格。

## 已驗證
- 本機 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`：**exit 0**，原文 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 3m 55s`（`scratch/issue-150/check1.log`；補上 `tests/native_t027_batch6.rs` 後再跑一次也是 exit 0，`check2.log`）。新增與修改的檔案沒有 warning。
- 本機 `python 維護工具/test_native_skill_coverage.py`（只解析原始碼，不是編譯產物）：`Ran 9 tests … OK`。
- CI：**PR #298 的 run 43189145172 通過**，lib 測試 `test result: ok. 2319 passed; 0 failed; 597 ignored`，本批 9 案全部 ok；細節見下方「CI」一節（本機不能執行測試，以 PR 的 CI 為準）。

### 新增的 lib 測試（都不需要資源，CI 的 `cargo test --lib` 會跑）
`native_skills::t027_batch6_tests`（9 案）：
1. `eight_wire_shapes_are_exact_and_costs_are_not_predicted`：八個 body 逐 byte 比對、長度 3／7／11、HP／MP 為 1 時仍送包、狀態不被改動、哪些 body 算角色目標、目標政策在戰鬥階段放行、79 自用／失效 ID 不退回自用、76／87 不能沒有目標或指自己、五個 selector 帶目標會被拒絕、reuse 與 client clock。
2. `caster_gates_reject_without_partial_mutation`：7 種施法者狀態加觀戰 × 8 技能，失敗時不改動狀態。
3. `target_kind_range_and_weapon_stance_follow_the_formal_record`：活體／屍體／不在場、十格與一格（含對角）、79 原畫面菱形 18／20、十種步態下 87（50／54）與 91（50）的放行，以及其他六個不看步態。
4. `menu_labels_selectors_and_stale_targets_cannot_be_mixed`：選單名稱、selector 丟掉 hover 目標、屍體不能當活體目標、目標死亡／incarnation／連線變更後失效、79 無目標等於自己、76／87 沒有預設目標、步態條件在選單路徑同樣成立。
5. `hp_cost_boundaries_stay_with_the_server_and_279_differs_from_278`：四個 HP 技能在 HP＝1／成本−1／成本／成本＋1 都送出同樣三 bytes 且不改狀態；278／279／280／299 拒絕碼與文字；自己的 S20 action19 才算施法動作，別人的不算；兩秒沒回覆是 NoResponse。
6. `object_effects_are_atomic_and_never_grant_status_or_spend_points`：S37 3932／3935／4434／4824／4648／5831／5832／10710 截斷時原子拒絕、收包位置凍結（角色之後移動也不跟）、不改 HP／MP／角色／移動封鎖，不算施法 ACK。
7. `freezing_blizzard_s39_keeps_order_misses_and_the_formal_profile_only`：3933 截斷原子性、名單與 miss、`CastAction` 回饋、以施法者為中心、錯 type／action 仍隔離、空名單、繪製時記為 omitted（沒有畫面、音效或本機 Impact）。
8. `original_definitions_qualify_exactly_with_their_cue_and_end_times`：把八列原始定義交給真正的 `SpriteActionCatalog` 解析——五個帶 109 的列被通用規則拒絕、被釘死定義接受；5831／5832／10710 被通用規則接受。逐列核對格數、音效格與種類、Hit 格，以及用整數 callback（每格 `floor(ticks×1000/24)`）算出的時間：音效 166／0／41／82／123／0／83ms，Hit 1494／1993／374ms，畫面結束 2451／2076／246／2366／2281／2324／2491／665ms。這些數字和歷史候選說明裡的數字一致（166／2451、41／246、82／2366、123／2281、1494／2324、83／1993／2491、374／665）。
9. `changed_original_definitions_are_refused_not_approximated`：14 種改動（109 的值、attr、header、音效 ID 或種類、ticks、事件位置）只讓被改到的那一列被拒絕，其他列照常；缺列時是錯誤而不是改用別的列。

### CI
- 第 1 次推送（`5fcb438`）時 main 已前進到 `3e761f2`（#156 技能 167–176），所以開 PR 前先依常設授權 `git merge origin/main`。只有 `native_skills.rs` 衝突，六處都是同一行的清單，兩邊的項目都保留：`selector_only`（本批 80／88／89／90／91 加上 main 的 169／171／172／175／176）、自己或角色分支與畫面菱形清單（本批 79 加上 main 的 170）、角色＋座標分支（本批 76／87 加上 main 的 167／173／174）、距離表（本批 76=10、87=1 加上 main 的 167=8、173=3）、`reuse_millis`（本批 76／80／88／90／91=3000、89=5000、87=8000 加上 main 的 167–176）。合併後本機 `cargo check …--all-targets`：**exit 0**，`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1m 49s`（`scratch/issue-150/check3.log`）；`python 維護工具/test_native_skill_coverage.py`：`Ran 10 tests … OK`。
- **PR #298 的 run 43189145172（`9c4ded5`）：通過**。`cargo check --locked --all-targets` 原文 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1m 10s`；`cargo test --locked --lib --no-fail-fast` 原文 `test result: ok. 2319 passed; 0 failed; 597 ignored; 0 measured; 0 filtered out; finished in 23.33s`。本批 9 案（`modern_lito::native_skills::t027_batch6_tests::*`）逐案都是 `ok`（log 存在 `scratch/issue-150/ci-43189145172.log`）。597 個 ignored 不算通過，原資源 opt-in 案仍待 Codex。PR 狀態 MERGEABLE／CLEAN。
- 本報告與狀態表補上 CI 結果的 commit 另外會再跑一次 CI；那一輪的數字寫在調度器摘要（`=== RESULT ===`）裡，不再為了補數字多推一個 commit。

## 推測
- 送包形狀、範圍、步態條件與 reuse 是依正式 contract、候選說明與相鄰伺服器原始碼做的靜態對照，不代表正式 E149 服務一定接受。相鄰伺服器原始碼是工作目錄現況，不等於正式唯讀 JAR `E1498871…`；歷史候選說明（讀 E149 JAR 得出）在八個技能的 row／column、形狀、reuse、成本與特效 ID 上和它一致。
- 帶 `109.effect(1 -1)` 的 3932／4434 在 S37 下當成「固定在收包位置的有限特效」處理，是沿用歷史候選的核對結論（候選當時讀原 PE 得出）以及 main 上 2942／2943（同樣 `1 -1`、同樣走 S37 有限特效）的既有作法；本批沒有重新讀原 PE。
- 五個釘死定義與三個通用定義的音效時間、結束時間是由定義算出來的無延遲模型；實際解圖、混色、音效檔是否存在、SPR 有沒有空白格，要等原資源 opt-in 案跑過才算驗證。SPR 若有既有規則不認得的空白格，該特效會載入失敗並 omitted（只影響那一個 ID）。
- 87 成功時伺服器回的是武器的一般攻擊動作而不是施法動作 18／19（依相鄰伺服器的註解與候選說明），所以施法回饋會在兩秒後顯示「未收到施法回覆，可重新施放」；送包、冷卻 8 秒與之後的封包處理不受影響。這一點沒有實機確認，所以沒有為它改回饋邏輯。

## 待驗證／未完成
- **本機沒有執行任何 `cargo test` 或 `run-local-test-suite.ps1`**（2026-11-09 規定不得執行編譯產物）。lib 測試以 PR CI 為準；`run-local-test-suite.ps1` 與原資源 opt-in 案 `native_t027_batch6::original_batch6_object_effects_use_exact_definitions_screen_and_sounds`（八個特效實際解圖、Screen、音效各一次（10710 無音效）、結束時間）需要交給 Codex 執行。
- `run-local-test-suite.ps1` 的技能類：本批自己的列已經對好（狀態表寫完整名稱，`skills.tsv` 加了同名的 `__lib` 列，`tests.tsv` 加了 opt-in 案）。#149 報告提到批 3／批 4 在 `skills.tsv` 缺對照、技能類會在讀清單時失敗，這件事不在本張範圍，沒有動，也沒有重新確認現況。
- 80 的 3933 呈現（31 格、Screen、overlap 音效 1353、無 Hit、以施法者位置為中心）沒有做，收包後記為 omitted。
- 87：施法回饋沒有接「自己的攻擊動作」當成回覆（見「推測」）；4183 眩暈物件的呈現、S25 22／23 的鎖定沿用 main 既有實作，本批沒有另外測。
- 90：原客戶端在送包前會先看背包有沒有 use type 25 的裝備，本機沒有做這個前置檢查（歷史候選說明要求不要用單一路徑代替伺服器的裝備規則），沒有合格裝備時由伺服器回訊息。
- 91 觸發反擊時攻擊方的 S20 action2、權威傷害，88／89／90／91 的狀態圖示與自然到期，76 的緩速與 79 的最大 HP／MP 變化，都只收伺服器封包，本批沒有新增對應的呈現或測試。
- 5831／5832／10710 的 Hit 只產生診斷事件（既有行為），不會造成本機傷害或受擊動畫。
- 沒有啟動遊戲或客戶端、沒有連正式服務或 DB、沒有做 GUI、DPI、滑鼠鍵盤、音效裝置或和原版並排的驗收；HISTORY 列的 88／89 HP 成本邊界 279／278 實機矩陣、90／91 換裝移除與實際反擊、自然 192／64 秒、G06 與 T030 都不在本張範圍。

## 產出路徑
- `LiTo381-Modern/src/modern_lito/native_skills.rs`：76／79／87 送包分支、五個 selector、步態條件與常數、reuse；測試模組註冊。
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`：八個選單名稱。
- `LiTo381-Modern/src/modern_lito/native_attack_many.rs`：S39 接受 3933／type0／action18。
- `LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs`：五個釘死定義與 `qualify_batch6`。
- `LiTo381-Modern/src/modern_lito/tower_assets.rs`：`load_finite` 對這五個 ID 用釘死定義。
- `LiTo381-Modern/src/modern_lito/native_t027_batch6_tests.rs`（9 案）。
- `LiTo381-Modern/tests/native_t027_batch6.rs`、`維護工具/test-whitelist/tests.tsv`、`skills.tsv`：1 個原資源 opt-in 案與白名單。
- `docs/SKILLS-STATUS.md`：只改本批八列並加「T027 批 6 回填」一段。沒有改 `docs/REPORT.md`、`TASKS.md`、`PLAN.md`。

## 給下一棒的提示
- 雲端 Claude 審查 PR；Codex 跑 `run-local-test-suite.ps1` 與上面的原資源 opt-in 案，把結束碼和 `test result:` 原文補進本報告。opt-in 案若有某個 ID 失敗，先看 receipt 的 `finite_banks.errors`：是定義不合（`batch 6 finite effect … changed`）還是 SPR／音效檔的問題。
- T027 六張都交了之後，`TASKS.md` 的 T027 狀態與 `docs/SKILLS-STATUS.md` 的統計表由主控同步（本張沒有改）。
- 80 補呈現時可以和 59 的 757、62 的 1812、70 的 1819 一起做：都是 type0、以收包時的施法者位置為中心。3933 的定義是 `0.effect(0 31,…)`、102 type0、104 attr8、沒有 109，音效 `<1353` 在 source1。
- 87 的回饋若要修：伺服器回的是自己的 S23 攻擊動作（`own_attack_packets`），可以比照 `observe_release` 加一個只給 87 用的觀察函式；先請 Codex 在實機確認回的封包再動。
- 之後還有帶 109 列的 S37 特效要收時，照本批的作法在 `BATCH6_SPECS` 旁邊加一列釘死定義即可，不需要再開新的 bank。
