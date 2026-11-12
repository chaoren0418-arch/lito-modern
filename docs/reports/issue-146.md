# Issue #146 — T027 批 2：技能 29、30、33、36、37、39、40、41

日期：2026-11-06。分支：`task/146`。本張只處理 T027 第二批八個技能，未將 T027 或 M2 整體標為完成。

## 做了什麼

| ID | 送包（未加密 body，含 opcode）與選取 | 收包／呈現 |
|---|---|---|
| 29 緩速術 | `14 03 04 + d(actor) h(x) h(y)`，11 bytes，其他活體、八格 | S37 gfx752，原 16 影格／3 ticks／無音效；緩速與解除僅依 S29 |
| 30 巖牢 | `14 03 05 + d(actor) h(x) h(y)`，11 bytes，其他活體、八格 | S39 action18/type8/gfx1805，第一個 wire 目標座標在收包時固定；主體1805／衣層1807／陰影1806 整組提交，source2 Sound718、source4／7 Hit |
| 33 木乃伊的詛咒 | `14 04 00 + d(actor) h(x) h(y)`，11 bytes，其他活體、兩格；reuse／client clock 5000ms | S37 gfx746，麻痺與解除僅依 S25；不由動作或特效推定命中 |
| 36 迷魅術 | `14 04 03 + d(actor) h(x) h(y)`，11 bytes，其他活體、原畫面菱形內 | S37 gfx228／Sound132；容量319是拒絕，不自行建立同伴 |
| 37 聖潔之光 | `14 04 04 + d(actor)`，7 bytes，未選目標時用自己；原畫面內 | S37 gfx227／Sound122；不由特效自行解除麻痺 |
| 39 魔力奪取 | `14 04 06 + d(actor) h(x) h(y)`，11 bytes，未選目標時用自己，四格 | S37 gfx2171／Sound806；HP／MP 只依權威封包，無回覆沿既有兩秒 NoResponse |
| 40 黑闇之影 | `14 04 07 + d(actor) h(x) h(y)`，11 bytes，其他活體、三格 | S37 gfx2175／Sound802，attr128 使用既有 RGB555 Multiply；致盲與解除僅依 S0A |
| 41 造屍術 | `14 05 00 + d(corpse)`，7 bytes，明確屍體、原畫面內 | S37 gfx226／Sound139；容量319與特效可以並存，均不表示同伴已建立；屍體移除後特效仍在收包位置播放完 |

沿用存活、已學技能、MP 有效值、場景、角色版本、連線與控制限制。MP 折扣、HP 成本、材料、機率、NPC 模板與領域、同伴容量及生命週期由伺服器決定，不在本機預扣或推測成功。39 的無目標自施與錯誤明確目標分開處理，不會將失效選取偷偷回退成自施。

選單新增八個名稱。41 沿現行滑鼠可見像素選取路徑，另以死亡動作序號綁定屍體意圖；活體選取在死亡後不能取得造屍授權，屍體復活也不能轉成一般攻擊授權。原視窗的目標文字使用相同技能解析。未新增歷史候選的 T 鍵循環功能。

巖牢接現行 World／Scene／History，保留 wire 順序、miss、目標 incarnation、固定中心、重複 Hit 入口去重與場景退役。僅新增精確 action18/type8/gfx1805，其他未知 profile 繼續隔離；相應更新既有隔離回歸測試。

原資源實測發現 1805 主體與 1806 陰影的 subframe15／16 是合法 no-block 空白，1807 煙霧在這兩格仍可見。新增專用載入策略，只接受原始 SHA256、17 格數量與指定空白欄位；其他 profile 維持原限制。尾段依各層實際像素提交，保留僅煙霧與 terminal source0 停留，不因主體空白跳過整組。

## 來源核對

- `TASKS.md` T027、`docs/PROTOCOL-CONTRACT.md` §6.3、`docs/SKILLS-STATUS.md`。
- 原專案唯讀 `驗證紀錄/current/native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt`，SHA256 `DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25`。核對 entangle、area、taming、curse removal、mana drain、blind、zombie 處理器與目標／HP／容量分支。
- 唯讀 `row15937-list.spr`，SHA256 `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127`。五個新增 finite 定義直接核對原始 header、attrs、subframe、ticks、sound；巖牢另嚴格核對兩個附層與兩次 Hit。
- 原始 `1805-0.spr`：29959 bytes，SHA256 `CDDBEC0CA946D10273A3926788FFE376C06C9476C987831734395DF04925E507`；`1806-0.spr`：11361 bytes，`44831BE42AC75C74FF60F3A338DC26BF3E4250E3CC9EDF3200066BEFD7D10A5A`；`1807-0.spr`：51321 bytes，`58D9826A4C3734444A29FA330D70C3E341DF7756BA09994F4FC8069671A86906`。空白格為 offset192／120、0×0、type0、unknown65535／0、空 rgba 與空 references。
- `維護工具/native-E149-{slow29-development,curse33-drain39,reveal23-tame36,break27-dark40,zombie41,area2230}.md` 與相關 prepare 腳本僅作歷史行為參照。未執行舊 prepare 流程、套用候選整包、修改封存或使用舊 PE。

## 已驗證

完整執行 `powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/run-local-test-suite.ps1`，Full 模式，未使用 `-Quick`。**104／104 步驟 PASS，套件 exit code 0**；0 tests、失敗與解析錯誤均依原門檻拒絕。ignored 不算通過，明列白名單的 opt-in 案才另行執行。

本輪日誌根目錄：`LiTo381-Modern/target/current/verification/local-test-suite/20261106-173835-382-19020/`（不追蹤），完整結果在 `summary.txt`。以下為必要建置檢查、兩版 library 與本批 11 個白名單測試的命令、結束碼及原文：

```text
001.log | exit=0 | cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 16s

002.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast
test result: ok. 1954 passed; 0 failed; 577 ignored; 0 measured; 0 filtered out; finished in 3.28s

003.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast --release
test result: ok. 1954 passed; 0 failed; 577 ignored; 0 measured; 0 filtered out; finished in 1.83s

094.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact eight_wire_shapes_self_policy_and_costs_are_server_owned
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.04s

095.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact all_skills_enforce_caster_and_target_gates_without_partial_mutation
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.05s

096.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact ranges_include_diagonals_and_native_screen_asymmetric_edges
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.01s

097.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact menu_uses_learned_ids_and_rejects_stale_owner_target_and_book
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

098.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact corpse_intent_never_upgrades_live_selection_or_survives_revival_and_aba
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

099.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact object_replies_are_atomic_and_never_imply_status_cost_or_companion_success
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.01s

100.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact slow_paralysis_cure_and_darkness_change_only_on_authoritative_packets
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

101.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact rejection_capacity_and_silent_timeout_do_not_create_success
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

102.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact earth_jail_receipt_preserves_order_misses_frozen_origin_and_atomicity
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.01s

103.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact original_finite_effects_preserve_cadence_mix_sound_and_consumed_corpse_position
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 14.23s

104.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 --no-fail-fast -- --include-ignored --exact original_earth_jail_draws_three_layers_and_two_hits_without_replaying_on_duplicate_event
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 1.54s
```

新增四個 library 案，驗證巖牢精確圖層／Hit／音效／ticks 定義的反例、空白格欄位與位置限制、History 接續不重播，以及兩次 Hit 排除 miss／死亡／同 ID 替身。新增 `tests/native_t027_batch2.rs` 十一案（九個純封包／模型、兩個原資源記憶體繪製），全部納入本機白名單。原資源案檢查實際解圖、各層混色、固定位置、音效準備、退場及主體空白時的煙霧停留。

完整套件之前另獨立執行整批，exit code 0；`scratch/batch2-verified.log`：

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2 -- --include-ignored
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.47s
```

開發期間失敗不算通過：`scratch/batch2.log` 首次為 exit101，整合測試誤引用 private module，尚未執行測試；已改經正式 Effects.prepare 載入驗證。`scratch/batch2-second.log` 與 `scratch/batch2-final.log` 均 exit101、10 passed／1 failed，分別找到原巖牢 no-block 影格與專用載入器需啟用既有 Hit callback 路徑。`scratch/jail-diagnostic.log` 是診斷原資源的失敗重現（exit101、0 passed／1 failed）。均保留日誌，未刪除案例或降低測試門檻；完成證據只採修正後的獨立整批與 Full 套件。

```text
test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.85s
test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.84s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 1.49s
```

## 推測

正式處理器、原 list 定義與離線模型一致，是靜態協定與呈現證據；不代表正式服務施法、原生排程或逐像素對照已通過。

## 待驗證／未驗證什麼

- 未啟動遊戲或客戶端，未做原生 GUI、滑鼠／鍵盤、DPI、裝置音效播放及原版並排驗收。
- 未連正式服務或 DB，未驗材料實際消耗、命中機率、NPC 資格、迷魅／造屍後的正式同伴操作與期限。
- 29／33／36／40／41 保留伺服器待定分類；Ctrl／LFC1 與完整敵友政策仍屬 T061。
- 未執行其他五批的開發、G06 歷史矩陣或 T030 完整 GUI。

## 產出路徑與給下一棒的提示

- `LiTo381-Modern/src/modern_lito/native_skills.rs`、`native_skill_selection.rs`、`native_login_window.rs`：請求、選單與屍體意圖。
- `native_world.rs`：容量319回饋；`native_additional_finite_effects.rs`、`native_skill_effects.rs`：精確有限特效與 Multiply。
- `native_earth_jail.rs`、`native_attack_many.rs`、`native_batch1_area.rs`：巖牢層、凍結中心及 History 路徑。
- `tower_assets.rs`：巖牢專用原檔與空白格資格，未放寬通用載入策略。
- `LiTo381-Modern/tests/native_t027_batch2.rs`、`native_selector_only.rs`、`維護工具/test-whitelist/tests.tsv`：本批測試與隔離回歸。
- `docs/SKILLS-STATUS.md`：只回填本批八列；本報告為 `docs/reports/issue-146.md`，未修改 `docs/REPORT.md`。
- 下一棒由雲端 Claude 審查，使用者依既有流程合併。後續批次沿各自 Issue 接續；屍體技能須使用 `resolve_for_skill`，不能放寬一般活體 `resolve`；範圍效果不可用第二個目標替代缺失的第一個 wire 中心。
