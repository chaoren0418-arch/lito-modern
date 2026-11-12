# Issue #145 — T027 批 1：技能 5、12、20、21、22、23、25、27

日期：2026-11-06。分支：`task/145`。範圍只涵蓋 T027 第一批八個技能；T027 其餘五批與 M2 整體狀態未改為完成。

## 做了什麼

| ID | 入口與送包（包含 opcode，未加密的 body） | 收包／呈現 |
|---|---|---|
| 5 | 選擇隨機傳送或伺服器 S30 書籤；`14 00 04 + h(map) h(x) h(y)`，9 bytes | 隨機為當前 map、x=y=0；書籤綁來源、角色選取代次、連線、場景、來源地圖與書籤版本。收到 S5A 才退役舊場景，沿既有 T023／G06 C08 路徑回覆一次 |
| 12 | 背包選取物品；`14 01 03 + d(item)`，7 bytes | S37 gfx747、既有背包權威更新、物品拒絕訊息 79 |
| 20 | 指定可見活體，距離 1；`14 02 03 + d(actor) h(x) h(y)`，11 bytes | S0A 闇盲狀態與既有 gfx746 |
| 21 | 背包選取物品；`14 02 04 + d(item)`，7 bytes | S37 gfx748；不預改裝備屬性 |
| 22 | 指定可見活體，距離 6；`14 02 05 + d(actor) h(x) h(y)`，11 bytes | S39 action18/type8/gfx1804；依第一個 wire 目標凍結效果中心，保留 miss 與收包順序 |
| 23 | 明確指定畫面菱形範圍內的其他活體；`14 02 06 + d(actor)`，7 bytes | S37 gfx2166–2169；NPC 資格與弱點由伺服器決定 |
| 25 | 指定可見活體，距離 6；`14 03 00 + d(actor) h(x) h(y)`，11 bytes | S39 gfx171 飛行→各接收者 gfx218；每次延遲 callback 只走一步，重複音效按 occurrence 去重 |
| 27 | 指定可見活體，距離 4；`14 03 02 + d(actor) h(x) h(y)`，11 bytes | S37 gfx172、材料不足 299；不預扣 40318 或改耐久 |

所有請求保留已學技能、存活、場景就緒、控制限制及 MP 有效值檢查；實際 MP 成本、材料、命中、NPC 資格、附魔內容與 HP 均由伺服器決定。12／21 的選取綁整筆物品、背包版本、角色／連線與場景，移除後同 ID 重建也不能復用舊選取。

原生視窗加入八個技能標籤、背包附魔入口與技能 5 的目的地下拉選單。書籤或背包改版時取消舊選取，不會在刷新時自動重新授權。沒有建立假畫面或啟動遊戲。

## 來源核對

- `docs/PROTOCOL-CONTRACT.md` §6.3、`docs/SKILLS-STATUS.md`、`TASKS.md` T023／T027／T061，以及 `docs/HISTORY.md` 的對應技能紀錄。
- 唯讀正式處理器：原專案 `驗證紀錄/current/native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt`，SHA256 `DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25`。技能 5 的指定座標需要保存書籤；隨機為當前地圖與零座標。一般技能 5 使用直接 S5A，不把技能 69 的 S55／C4E 流程移入本批。
- 唯讀原始 `row15937-list.spr`，SHA256 `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127`。新效果逐一核對 graphics、動作、影格、tick、屬性、音效與 Hit；資源不複製進 repo。
- 隔離候選 `native-E149-static2941-ready-20261024-210441-110315` 僅作行為參照。本批重接現行 src 的 controller／World／Scene／History，沒有整包套用候選、修改封存或選入舊 PE。

## 已驗證

本輪完整套件：`powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/run-local-test-suite.ps1`，使用 Full 模式，未使用 `-Quick`。記錄根目錄：`LiTo381-Modern/target/current/verification/local-test-suite/20261106-154845-072-14212/`（不追蹤）。

已通過下列建置檢查，exit code **0**，原始輸出在 `001.log`；命令於 `LiTo381-Modern/` 執行：

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.22s
```

完整套件：**93/93 步驟 PASS，exit code 0**。`summary.txt` 保留各步驟與解析結果。library 的 ignored 案不計入通過；只有白名單明列的 opt-in 案另行執行。

```text
002.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast
test result: ok. 1933 passed; 0 failed; 575 ignored; 0 measured; 0 filtered out; finished in 7.57s

003.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast --release
test result: ok. 1933 passed; 0 failed; 575 ignored; 0 measured; 0 filtered out; finished in 1.87s

061.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_selector_only --no-fail-fast -- --include-ignored --exact counter72_malformed_capture_is_atomic_and_unknown_profiles_stay_quarantined
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.03s

081.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact all_eight_complete_wire_shapes_are_pure_and_use_learned_ids
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

082.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact every_skill_rejects_unlearned_dead_blocked_and_zero_mana_without_mutation
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.10s

083.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact actor_ranges_screen_edges_and_stale_selection_are_enforced
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.03s

084.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact menu_exposes_eight_skills_and_keeps_item_actor_and_destination_domains_separate
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

085.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact item_target_rejects_inventory_revision_aba_epoch_and_scene_changes
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

086.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact item_replies_and_weapon_break_material_rejection_only_change_received_authority
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

087.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact blind_and_object_effect_replies_are_authoritative_and_truncation_atomic
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.01s

088.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact area_replies_preserve_order_misses_origin_and_reject_malformed_packets_atomically
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.01s

089.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact teleport_keeps_authoritative_position_until_portal_and_retires_old_scene
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

090.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact no_portal_rejection_timeout_and_retired_owner_never_invent_teleport_success
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

091.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact original_batch1_effect_assets_render_and_retire_without_authority_writes
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 15.18s

092.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact teleport_bookmarks_are_atomic_bounded_and_encode_map_before_coordinates
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

093.log | exit=0 | cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch1 --no-fail-fast -- --include-ignored --exact teleport_choices_retire_on_bookmark_identity_epoch_or_scene_changes
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

```

新增 `tests/native_t027_batch1.rs`：13 案，全部列入本機測試白名單，含一個原資源 opt-in 案。涵蓋八個完整送包向量、權威狀態不變、邊界距離、選取失效、S30 截斷與 50 筆上限、跨地圖目的地、S37／S39 截斷原子性、回覆狀態、傳送失敗與效果實際資源繪製／退場。

另加 library 測試：火球延遲 callback／邊界／來源、音效 occurrence 去重，以及正式 controller 在合成加密 loopback 上的 S5A→C08 唯一回覆、取消、EOF、拒絕與逾時。loopback 測試直接建立已登入的合成 Snapshot 及 pending，以隔離驗證接收與退役路徑；不是正式登入或完整 UI→C14→伺服器施法測試。

開發中曾執行 13 案整合測試，exit code 101，12 過、1 失敗（來源 map 改變仍可使用舊選取）。已新增來源 map 綁定，保留失敗紀錄於 `scratch/batch1-final.log`；不以此輪作完成證據。

```text
test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.71s
```

第一輪 Full 套件 `20261106-152500-930-24348` 為 92／93 步驟通過、exit code 1；本批 13 案均通過。唯一失敗是 `native_selector_only::counter72_malformed_capture_is_atomic_and_unknown_profiles_stay_quarantined`，舊案仍要求本批新支援的 action18/type8/gfx171 必須被隔離。已更新為檢查未接的 gfx1805、171／1804 的錯誤 action／type 仍隔離，並正向檢查 170／171／1804；保留所有截斷／畸形原子性斷言，未刪除失敗測試或降低執行門檻。該輪 `061.log` 原文：

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.03s
```

## 推測

正式 javap、原 list 定義及隔離候選行為與本批實作一致的部分，是靜態契約與離線模型證據。它們不能替代正式服務實際施法、原生 callback 全域排程或逐像素對照。客戶端未推算傷害、屬性、材料或附魔成功。

## 待驗證／未驗證什麼

- 未啟動客戶端或既有遊戲，未做 GUI、滑鼠／鍵盤／DPI 驗收、實際音效播放或與原版並排比較。
- 未連正式服務或 DB；未驗正式消耗、命中、附魔成功、書籤資料來源與傳送地圖資格。技能 27 的 40318 材料生產端仍依狀態表列「伺服器待定」。
- Ctrl／LFC1 與完整敵友政策另屬 T061。本批可選取明確目標，但不新增強制攻擊擴充。
- 未重跑 G06 歷史 640 次矩陣或宣稱 T023 整體完成；本批只新增技能 5 所需的同連線傳送驗證。T030 完整 GUI 仍待 M2 完成後安排。

## 產出路徑與給下一棒的提示

- `LiTo381-Modern/src/modern_lito/native_skills.rs`、`native_skill_selection.rs`、`native_login.rs`、`native_login_window.rs`：請求與操作入口。
- `native_teleport_spell.rs`、`native_portal.rs`、`native_teleport_controller_tests.rs`：S30、技能 5 請求綁定、S5A admission 與 T023 接線。
- `native_batch1_area.rs`、`native_attack_many.rs`、`native_world.rs`、`light_arrow_effect.rs`、`native_audio_delivery.rs`、`native_additional_finite_effects.rs`：精確資源與範圍效果。`mod.rs` 登記新模組。
- `LiTo381-Modern/tests/native_t027_batch1.rs`、`native_selector_only.rs`、`維護工具/test-whitelist/tests.tsv`：自動驗證與本批新增支援的隔離規則回歸。
- 本報告：`docs/reports/issue-145.md`；不修改 `docs/REPORT.md`。
- `docs/SKILLS-STATUS.md`：僅回填本批八列與增補說明，保留 #140 盤點基線及伺服器待定分類。
- 雲端 Claude 審查本批範圍與證據，使用者依既有流程合併。後續批次沿自己的 Issue 實作；物品技能可復用 ItemTarget，重複 callback 音效使用 occurrence，不可把普通 finite 去重移除。技能 69 獨立處理 S55／C4E／八秒失敗，不能套用本批技能 5 的直接 S5A 規則。
