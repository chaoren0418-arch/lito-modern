# Issue #147 — T027 批 3：技能 44–48、50、51、53

日期：2026-11-06。分支：`task/147`。本張只處理第三批八個技能；未將 T027 或 M2 整體標為完成。

## 做了什麼

| ID | 請求與選取 | 收包與呈現 |
|---|---|---|
| 44 魔法相消術 | `14 05 03 + d(actor)`，7 bytes；無目標用自己，三格；reuse 500ms | S37 gfx870 明確使用 bank232、Sound140；特效不自行清除狀態 |
| 45 地裂術 | `14 05 04 + d(actor) h(x) h(y)`，11 bytes；其他活體，八格；reuse 100ms | S23 wire type0/gfx129，class7 逐段展開→class6 gfx123；130／124 陰影、128／129／3080 音效；保留結尾停留及最後提交 |
| 46 烈炎術 | `14 05 05 + d(actor) h(x) h(y)`，11 bytes；其他活體，三格；reuse 500ms | S23 gfx1811，固定 wire to；19 frames、Sound709、source11 Hit 診斷 |
| 47 弱化術 | `14 05 06 + d(actor) h(x) h(y)`，11 bytes；其他活體，三格；reuse 500ms | S37 gfx2228，9 frames、Sound843、RGB555 Multiply；狀態由伺服器更新 |
| 48 祝福魔法武器 | `14 05 07 + d(actor)`，7 bytes；無目標用自己，原畫面範圍 | S37 gfx2176／Sound800；控制器接裝備拒絕訊息79，不把特效當附魔成功 |
| 50 冰矛圍籬 | `14 06 01 + d(actor) h(x) h(y)`，11 bytes；其他活體，六格；reuse 500ms、client clock 1000ms | S23 gfx756→219；明確空音效群組，不借用光箭音效；冰凍與解除依權威封包 |
| 51 召喚術 | `14 06 02 + d(selection)`，7 bytes；獨立種類下拉選單，31 個正式選項；reuse 1000ms | S37 2510／755與施法動作都不是同伴成功 ACK；兩秒內保留等待，接受晚到353／743／319／299拒絕 |
| 53 龍捲風 | `14 06 04`，3 bytes，無尾段；reuse 1500ms | S39 action18/type0/gfx758；四個固定偏移成一組，單一時鐘與Sound125；Hit列表保留 miss、目標 incarnation 與去重 |

沿用已學技能、存活、MP 已知且大於零、控制狀態、可見目標、目標版本與場景守衛。成本、材料、機率、受控 NPC 資格、召喚模板、生命週期與狀態期間均由伺服器決定，不在客戶端預扣或預測成功。

召喚選項另外綁定連線、角色背包 owner／revision、InputStamp、地圖與角色 incarnation；切換技能、背包、場景或失去連線後須重新選取。一般 actor 請求仍拒絕技能51，避免將召喚值誤解成物件 ID。拒絕訊息獨立保存序號，後續一般訊息不會覆蓋先前的召喚拒絕。

class1、投射物與地裂術加入 metadata History，保留時鐘、固定位置、進度與入場記錄；來源退役清除活動效果，晚到 History 只合併入場記錄。活動序號納入既有音效保留範圍；資源不隨 History 複製。地裂術使用有界路徑與同時限 FIFO，原始世界起點固定、終點只在指定入口依捕捉身分解析，陰影與主體先檢查整組再提交。

## 來源核對

- `TASKS.md` T027、`docs/PROTOCOL-CONTRACT.md` §6.3 的八個 row/column／尾段，以及 `docs/SKILLS-STATUS.md`。
- 原專案唯讀 `驗證紀錄/current/native-E149-summon51-inspection/20261024-105000/inspection.json`，SHA256 `E01AF24E7CE4FFD3FF2628100762CD29088E3C679669F07F403B5C84FCBA3E29`。本輪以其31個解析選項作選單基礎；未執行伺服器類別或查 DB。
- 唯讀 `row15937-list.spr`，SHA256 `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127`；核對870、2228、756、758、129／130、123／124的原始定義。原資源測試另實際讀取客戶端 Text／Sprite／Sound，解圖與準備聲音，不播放音效。
- `維護工具/native-E149-{object374448,sun46-weak47,freeze50,summon51,tornado53-group,eruption45-core,eruption45-scene}.md`、既有 prepare／template 與地裂核心只作設計參照。未執行舊 prepare、套用候選整包、修改原專案、封存或硬連結。

## 已驗證

完整執行 `powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/run-local-test-suite.ps1`，Full 模式，未使用 `-Quick`。**114／114 步驟 PASS，套件 exit code 0**。0 tests、失敗與解析錯誤均依原門檻拒絕；577 個 ignored 不算通過，白名單 opt-in 案另行執行。

本次結果使用整合上游 `683b603`（#186 復活生命週期、#192 Ctrl／LFC1）後的來源。`native_login.rs` 衝突保留兩者行為，召喚仍使用獨立選項參數；既有加密 loopback 另加入本批八技能，確認六個 actor 技能的 Ctrl 尾段、53 selector 無尾段、51 召喚值無 LFC1，實際經控制器、加密傳輸及自行建立的 localhost fixture 比對。

日誌根目錄：`LiTo381-Modern/target/current/verification/local-test-suite/20261106-191547-306-14544/`（不追蹤），總表 `summary.txt`。必要檢查、兩版 library 與本批十案的命令、結束碼及結果原文：

```text

001.log | exit=0 | cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 45s

002.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast
test result: ok. 1978 passed; 0 failed; 577 ignored; 0 measured; 0 filtered out; finished in 11.18s

003.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast --release
test result: ok. 1978 passed; 0 failed; 577 ignored; 0 measured; 0 filtered out; finished in 9.35s

105.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch3 --no-fail-fast -- --include-ignored --exact eight_wire_shapes_are_exact_and_costs_are_not_predicted
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

106.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch3 --no-fail-fast -- --include-ignored --exact caster_and_target_gates_preserve_authority
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.03s

107.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch3 --no-fail-fast -- --include-ignored --exact ranges_menu_and_stale_target_intent_are_enforced
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

108.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch3 --no-fail-fast -- --include-ignored --exact all_summon_choices_require_explicit_current_inventory_and_world
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

109.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch3 --no-fail-fast -- --include-ignored --exact summon_late_rejection_survives_cast_action_and_unrelated_messages
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

110.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch3 --no-fail-fast -- --include-ignored --exact all_receive_profiles_are_atomic_and_do_not_grant_status_or_costs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.05s

111.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch3 --no-fail-fast -- --include-ignored --exact original_object_effects_use_exact_banks_and_blends
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 9.11s

112.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch3 --no-fail-fast -- --include-ignored --exact original_tornado_is_one_four_image_group_and_hit_list
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 1.17s

113.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch3 --no-fail-fast -- --include-ignored --exact original_sunburst_uses_wire_destination_and_original_hit_timing
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 1.25s

114.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch3 --no-fail-fast -- --include-ignored --exact original_lance_is_silent_and_eruption_propagates_with_paired_shadows
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 4.35s
```

新增十個 integration 案（六個純封包／模型、四個原資源記憶體繪製），全部納入既有白名單；另新增四個 library 案，驗證地裂路徑端點排除／預算、巢狀 callback FIFO、class6 最後提交與來源退役／晚到 History。未刪除既有案例或降低門檻。

合併前首輪 Full 套件亦為114／114 PASS（`20261106-183505-664-14880/`），另有最終 check exit0（`scratch/check-final.log`）。合併後重跑的上述 Full 套件才是本次交付依據。

開發階段保留紀錄：`scratch/check-initial.log`（cargo exit101，新增模組的引用路徑）、`scratch/check-second.log`（cargo exit101，資源錯誤型別轉換）均為編譯失敗，尚未執行測試，修正後才進入驗證。`scratch/batch3-first.log` 曾整批 exit0，原文 `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.26s`。

### 第 2 次嘗試（claude-local，2026-11-12）：同步 main

PR #250 因 main 前進（#143／#194／#195／#261／#267 等）變成 DIRTY。本輪 `git merge origin/main`，解三處衝突：

- `native_login.rs`：技能 pending 判斷改為先看主線 `player_callbacks.pending()`，再走本批召喚 `observe_summon`／道具／一般；結束條件保留本批 `hold_summon`，並加回主線 `player_callbacks.cancel_pending()`。
- `native_world_scene.rs`：`FiniteHistory` 同時保留主線 `static2941` 與本批 `spells`／`class1`（`for_source`、`merge_disintegrate_admission`、`finite_history` 三處合併兩邊欄位）。
- `維護工具/test-whitelist/tests.tsv`：兩邊新增列全保留。

合併後本機 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`：**exit 0**（`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 7m 38s`）。

依 2026-11-09 本機限制，本輪**未在本機執行 `cargo test`／`run-local-test-suite.ps1`**（不得執行編譯產物）；上方 114／114 為第 1 次嘗試（合併前來源）結果，合併後的測試改以 PR CI 為準，屬「待驗證」。

### 第 2 次嘗試：再同步 main（2026-11-12，落後 52 commit）

`git merge origin/main` 衝突 7 檔，處理方式：
- `native_login.rs`：main 把技能回覆觀察抽成 `observe_skill(now)`；本批的召喚分支（`observe_summon`、`hold_summon`）移入該函式，舊的內嵌區塊刪除。main 新增 `cast_request_at(..., now)`（測試在用，簽名不變），另加 `cast_request_full` 帶 `summon` 參數，`cast_request` 與 `cast_request_at` 都轉呼叫它。
- `native_login_window.rs`：同時保留 `SUMMON_OPTIONS` 與 main 的技能書／F5–F12 快捷列；`SUMMON_OPTIONS` 位置跟 `TELEPORT_DEST` 一起上移到快捷列上方（bottom-190）。控制項 ID 218 與 399／400+ 不衝突。
- `native_skills.rs`、`native_skill_selection.rs`：無目標技能清單取聯集（加入 main 的 97／98／99 和本批的 53）；`reuse_millis` 合併 main 的 97／98..=100 與本批 44–53 的值。
- `native_world.rs`：保留召喚拒絕（353／743），技能拒絕碼採用 main 新加 279 的版本。
- `native_spell_projectiles.rs`：採用 main（#167）改用 catalog class、不再限制 flight_type 的做法；另加 `graphics!=129`，讓 129 繼續只由 eruption 處理；上限計數保留 `flights+eruption.active_count()`。
- `tests.tsv`：兩邊新增的列都保留。
- 另外修了 `native_eruption.rs` 測試的 `Projectile` 初始化：main 新增了 `retain_on_arrival`／`name` 欄位，這裡補上 `0`／空值。

合併後本機 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`：**exit 0**（`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2m 13s`）。本機仍未執行 `cargo test`（規定不得執行編譯產物），改看 PR CI，列為「待驗證」。

- 推送後 main 又前進（#167／#170／#178／#190 等），再合併一次，衝突 2 檔：`native_skills.rs` 同時保留本批的召喚型別和 main 新增的 `#[cfg(test)] mod tests`；`native_world_scene.rs` 採用 main 的四類 History（`skills:[_;4]`、`retire_source`、`active_finite_sequences`），並把本批的 `spells`／`class1` 歷史接回 `for_source`（來源退役時 retire）、`merge_disintegrate_admission`、restore 和測試用的 `from_profiles`。再合併後 `cargo check …--all-targets`：**exit 0**（`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1m 49s`）。
- 第三次合併 main（#151 技能 101–109）：無目標技能改由 main 的 `native_skills::selector_only()` 判斷，53 加入該函式（`request` 與 `choice` 共用）；法術目標分支與距離表取聯集（45／46／47／50 加上 103／108）；`reuse_millis` 合併兩邊的值；`World` 同時保留 `last_summon_rejection` 和 main 的 `last_item_skill_rejection`。合併後 `cargo check …--all-targets`：**exit 0**（`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1m 57s`）。
## 推測

離線封包與原資源測試支持客戶端請求、解碼、守衛與指定呈現行為；不代表正式 E149 服務實際接受、機率／材料扣除或與原版逐像素、原生排程完全一致。

## 待驗證／未驗證什麼

- 合併 main（2026-11-12）後的 `cargo test` 與 `run-local-test-suite.ps1` 未在本機重跑，需由 CI 或 Codex 重驗（特別是 `native_t027_batch3` 十案與主線 player callback 與召喚等待的交錯）。
- 未啟動遊戲或客戶端、未連正式服務或 DB、未做 GUI、DPI、實體滑鼠鍵盤、音效裝置或原版並排驗收。
- 45／46／50 原生相對 graphics 的 hurt guard／LT 家族仍未核定；目前不產生通用受擊替代。45 保留節點影格記錄、46 保留原 Hit 診斷，50 arrival 不更動狀態。原 action0 可選倍率、長 uptime／u32 rollover、全域 native callback 交錯與最大共存壓力亦未宣告完成。
- 51 的正式控制能力、等級／地圖／容量、材料、同伴指令、自然到期與伺服器生命週期未驗；44／47／51 原有「伺服器待定」分類保留。
- Ctrl／LFC1 已沿上游 #192 接線並離線驗證；完整敵友政策與正式服務端解讀仍依 T061 範圍審查。未進行其他批次開發、G06 歷史矩陣或 T030 完整 GUI。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`、`native_skill_selection.rs`、`native_login.rs`、`native_login_window.rs`：八技能請求、召喚選單／控制器、等待與拒絕。
- `native_world.rs`：召喚拒絕序號；`native_additional_finite_effects.rs`、`native_skill_effects.rs`：870／2228及混色。
- `native_eruption.rs`、`native_tornado.rs`、`native_attack_many.rs`、`native_class1.rs`、`native_spell_projectiles.rs`、`light_arrow_effect.rs`、`native_world_scene.rs`：特殊呈現、空音效、History 與收包接線。
- `LiTo381-Modern/src/modern_lito/native_forced_combat_controller_tests.rs`：補上本批八技能的上游 Ctrl／召喚對接加密 loopback。
- `LiTo381-Modern/tests/native_t027_batch3.rs`、`維護工具/test-whitelist/tests.tsv`：本批十個離線／原資源案例；地裂核心另有四個單元案例。
- `docs/SKILLS-STATUS.md`：只回填本批八列；本報告 `docs/reports/issue-147.md`。未手動修改 `docs/REPORT.md`；合併僅帶入上游索引，相對 main 無差異。

## 給下一棒的提示

雲端 Claude 審查本批 PR，使用者依既有流程合併。後續批次沿各自 Issue 接續；召喚值須走 `SummonTarget`，不可放寬 actor 請求。45／46／50 的原 hurt 與完整 native 計時資格需由原技能／呈現驗證工作接續，不能以本批離線測試當作正式玩法或 GUI 通過。
