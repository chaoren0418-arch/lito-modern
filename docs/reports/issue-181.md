# Issue #181 — T046 第 1／3 張：原生快捷列、技能書與施法中斷

日期：2026-11-12（第 2 次嘗試，回應 codex-review 退回）。分支：`task/181`。起點：`59f0c2bcd7dc4435ba43186f7767130ab7869af1`；本輪已合併 `origin/main` `120c7be`。

**結論：已補上退回的 P1／P2。技能圖示改用原版 LiTo 客戶端實際使用的 S1E 圖示表，181 筆目錄技能都有確定的原版圖形（145 筆來自 LiTo 圖示表，36 筆走原版「unknown spell」圖示 91），靜態檢查 181／181 可解碼。測試改成逐筆精確斷言。P3（3 個 TW 治癒資源資格失敗）與本分支無關，但依 2026-11-09 規定本機不再執行測試，無法補跑基線，需主控裁定（見「待驗證」）。`cargo check` 結束碼 0；`cargo test` 交給 PR CI。**

## 本輪做了什麼（回應退回）

### P1：53 筆缺圖 → 改用原版 LiTo S1E 圖示路徑

- 查證結果：**原版 LiTo 客戶端根本不讀 `SpellIcon.idx`**。在脫殼 PE（`lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`，SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`）中，`spellicon` 字串一次也沒出現。S1E 處理函式 `sub_4AEE70`（`004AEEB5..004AEEF3`）的圖示選擇如下：
  - `index = group*8+bit`（＝技能 ID − 1）。
  - 條件為 `dword_647E24`（LTSpellDesc 字串表）不為 0、`index < dword_647E20`（宣告筆數），而且該列字串不為 0。條件成立時，圖示是 `word_508648[index]`；否則走 `004AEEEC`，`push 5Bh`（圖示 91）＋`"unknown spell"`。
  - `sub_47C8A0` 會用 `"LTSpellDesc"` 載入 `dword_647E20`／`dword_647E24`。
- `word_508648` 位於 file offset `0x108648`（raw-backed），共 176 個 WORD。資料已嵌入 `native_inventory_icons.rs` 的 `LITO_SPELL_ICONS`，並附來源註記。這張表和共用 `SpellIcon.idx`（336 bytes，168 筆）**有多處不同**：79→1610（不是 741）、111→1611（不是 430）、118→1751、119→881、120→888、167→1621（不是 828）、172→828，並延伸到 169–176。所以前一版不只缺圖，還有 5 筆顯示了錯誤的圖示。
- `LiTo/LTtext` 的 `LTSpellDesc-c.tbl` 宣告 176 列，實際有 147 列（以數字開頭；與 S1E 合約記載的「Metadata147rows」一致）。有列的範圍是 `0..=22, 24..=79, 86..=90, 96..=110, 112..=119, 128..=133, 136..=137, 144..=175`，已嵌入為 `LTSPELLDESC_ROWS`。
- 新的 `spell_icon_route(skill)` 純函式照原版邏輯，回傳 `Lito(graphic)` 或 `UnknownSpell`（圖形 91）。`Reader::load_spell` 只透過這條路徑載入共用 `%d.tbt`，不再讀 `SpellIcon.idx`，也不使用 buff 圖示。
- 原本缺圖的 53 筆，現在的結果：
  - 87–91 → 1624／1887／1881／2349／2343
  - 132–134 → 1755／1744／1731
  - 160 → 2340
  - 169–176 → 1737／1756／1734／828／2346／2355／2352／2337
  - **112、181–195、201–220（共 36 筆）** → 原版圖示 91。原版 3.81C 的 LTSpellDesc 沒有這些列（112 破壞盔甲那列不存在；181 以後超出宣告的 176 筆），原客戶端同樣顯示 91。這是原版行為，不是猜圖。
- 技能書遇到走 91 的技能時，會在名稱後加註「〔原版未知技能圖示〕」，讓使用者知道這是原版的未知技能圖示，不是本專案的替代圖。資源讀取失敗時仍顯示「〔圖示未提供〕」。

### P2：測試改為精確覆蓋

- 新增 CI 會執行的 `catalog_icon_route_is_complete_and_exact`（`--lib`，不需要資源），逐一檢查 181 筆：
  - 145 筆為 `Lito(g)`，且 g 不為 0、小於 20000。
  - `UnknownSpell` 必須**完全等於** 36 筆的集合。
  - LTSpellDesc 列數必須是 147。
  - 28 筆的 ID→圖形對照逐一比對，包含所有與 SpellIcon.idx 不同的項目和原本缺圖的項目。
  - #206 必須是 91。
  - 不存在的列（24、81、92、121、135）、超出範圍的 ID（177、221、65535）都必須走 91；ID 0 必須報錯。
  
  只要任何一筆退回缺圖或換圖，這個測試就會失敗。舊的 `index_is_bounded_one_based_and_never_fabricates_an_icon` 已移除（它驗證的是不再使用的 SpellIcon.idx 解碼）。
- 實際資源測試 `real_skill_icons_and_async_service_preserve_mapping_and_missing_assets`（`#[ignore]`，需設定 `LITO_CLIENT_ROOT`）改為：
  - 從實際 `LiTo/LTtext` 讀出 `ltspelldesc-c.tbl`，斷言宣告筆數 176，並斷言有列的集合**等於**嵌入的 `LTSPELLDESC_ROWS`。
  - 對 181 筆**每一筆**呼叫 `load_spell`，任何一筆失敗就 panic。
  - 斷言 `icon.id` 等於路由後的圖形、`covered_pixels > 0`、走 91 的剛好 36 筆。
  - 非同步服務中 #206 必須成功載入圖形 91，而且不能有 error。原本「圖示或錯誤擇一即可」的寬鬆斷言已移除。

### 同步 main

合併 `origin/main`（#143 強制攻擊、#186 復活、#192 Ctrl 強制目標、#194 角色分頁、#195 同伴、傳送技能 5、物品施法等）。衝突處理如下：

- `native_login.rs`：
  - `cast_request` 採用 main 的新簽章（item／teleport／`CombatInput`），內部轉到 `cast_request_at(..., now)`。
  - 送出前的准入拆成兩部分：共用的 `skill_gates_open`（新增 `teleport_spell` 待回覆），加上 main 的「移動中 callback、無目標施法」例外。UI 用的 `skill_action_ready` 仍要求 `movement_ready`，比送出端保守。
  - 送出改用 main 的 `send_player_action`，送出後才建立 `skill_context`。
  - `observe_skill` 納入 main 的三項：callback pending、物品施法 `observe_item`、`teleport_spell` 退休。
  - 復活處理採用 main 的 `interrupt_world_actions()`，並在其中補上 `skill_context = None`。
- `native_login_window.rs`：
  - 同時保留 `skill_ui` 與 `companions` 模組，以及技能書／快捷列與 `TELEPORT_DEST` 控制項。
  - `cast_skill` 先走 main 的物品／傳送分支，其餘技能走 ID 綁定的 `choice_id_at`。
  - 施放按鈕的啟用條件是 main 的 `choice_valid` 加上 `skill_action_ready`。
  - 版面：傳送目的地下拉移到快捷列上方（bottom−190）。選了技能 5 時，狀態欄移到 bottom−290，技能書清單上移 40。
- 合併後編譯器警告 `unreachable pattern`：本分支的 `skill_ui::BOOK = 138` 與 main 新增的 `CHARACTER_PREV = 138` 撞號，會讓角色分頁按鈕失效。已把 `BOOK` 改為 399（F5–F12 為 400–407），重新檢查後警告消失。
- 測試配合新簽章更新 6 個 `cast_request_at` 呼叫點，斷言不變。

## 已驗證

| 檢查 | 結束碼 | 結果 |
|---|---:|---|
| `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`（合併 main 後、HEAD 狀態） | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1m 27s`；本分支觸及的檔案沒有新警告 |

靜態資源與逆向查證（唯讀 Python 腳本，輸出在本 worktree 的 `scratch/`，不進 git）：

- `scratch/pe_table.py`：從脫殼 PE 讀出 `0x508648` 的 176 個 WORD，與嵌入常數相同。
- `scratch/verify_spell_icons.py` → `scratch/spell-icon-route-181.json`：對 181 筆目錄技能套用原版路由，到 `Sprite.idx`／`Sprite00–15.idx` 找 `<g>.tbt`（分片規則同 Rust：名稱 byte 總和 % 16，找不到再查 base）。結果是 181／181 都只有 1 筆紀錄、都在 `list.ico`（99999）範圍內，TBT 結構解析全部成功（高度 1–127、覆蓋 words > 0、剛好讀到結尾，尾端 0 byte）；`bad = []`。
- 輸入雜湊：
  - LTSpellDesc-c.tbl（已解碼）`6C8BBA0B…6C54DFC`
  - `LiTo/LTtext.idx` `B8C0BB8C…D8AE23C8`
  - `LiTo/LTtext.pak` `517D853E…5580F342D`
- 反組譯依據：`ida-client-oep-v2/full-disassembly.asm` 的 `sub_4AEE70`、`sub_47C8A0`，以及既有的 `lito脫殼資源/S1E-initialization-gates.json`、`graphics-parity/fallback-91-namespace.json`。

宣稱缺資源前查過的 pak（以 old28 解析器讀索引，唯讀）：

| pak | 查詢 | 命中 |
|---|---|---|
| `Sprite.idx/pak` | `spell\|skill\|magic\|\.idx$` | `SpellIcon.idx`（336 B）、`SpellDesc-k.tbl` |
| `Sprite00–15.idx/pak` | `spell\|skill\|\.idx$\|\.tbl$` | 0 |
| `LiTo/LTicon.idx/pak` | 全列 | 226 個 `.lbt` 和 `LTlist.ico`，沒有技能索引 |
| `LiTo/LTtext`（已解碼索引） | `spell` | `LTSpellDesc-c.tbl`、`LTSpellDesc-k.tbl`、`LTspelldur.tbl` |
| `Text`（已解碼索引） | `spell` | `spelldesc-c/e/h/j/k`、`SpellDesc-h.html`、`spelldur.tbl`、`SpellExDesc-c.tbl` |

結論：這 53 筆並不是缺資源，而是前一版選錯了路徑（SpellIcon.idx）。

## 推測

- 原版 S1E 對有列但表值為 0 的情況會畫 `0.tbt`。本表在 147 個有列的位置中沒有 0（已確認），所以 `spell_icon_route` 對這種情況回報錯誤，不會觸發。
- 本專案的技能書沿用 `docs/SKILLS-STATUS.md` 的名稱，不改成原版的 `"unknown spell"` 文字。只有圖示照原版；文字差異已用「〔原版未知技能圖示〕」標註。
- `world_hotbar.rs`／`world_skill_panel.rs`（較早的世界場景快捷列）仍讀共用 `SpellIcon.idx`。它們不在本張改動範圍，但同樣會顯示錯圖或缺圖，建議由後續任務改用 `spell_icon_route`。

## 待驗證／未完成事項

- **本機未執行任何 `cargo test`**。依 2026-11-09 規定，本機不得執行編譯產物，`cargo test` 交給 PR 的 GitHub Actions（`cargo test --locked --lib --no-fail-fast`，會跑新的 `catalog_icon_route_is_complete_and_exact` 與既有 `skill_ui_tests`）。CI 結果以 PR 頁面為準，本報告不預先宣稱通過。
- `#[ignore]` 的實際資源測試，以及 `tests/native_skills.rs` 等整合測試，CI 不會執行，需由 Codex 本機執行者以 `LITO_CLIENT_ROOT=D:\lineage開發\0-客戶端` 跑：

  ```text
  cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib real_skill_icons_and_async_service -- --ignored --test-threads=1 --nocapture
  cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib skill_ui_tests -- --test-threads=1
  cargo test ... --test native_skills / native_skill_selection / native_attack_skills / native_ground_skills / native_reagent_skills / native_inventory_icons -- --test-threads=1
  ```

  門檻以 `run-local-test-suite.ps1` 為準。前一輪（舊 commit `53454fe`，規定生效前）的結果為：新增 14 案通過，相關回歸 54 passed／3 failed。本輪改了程式並合併 main，**那些數字不代表目前 HEAD**。
- **P3（`native_skills` 3 案）需主控裁定**：
  - 失敗的三案：`original_tw_basic_heal_frames_sound_clock_and_retirement_preserve_authority`、`tw_higher_heal_original_slots_clothes_sound_and_retirement_preserve_authority`、`tw_higher_full_heal_group_budget_rejects_both_layers_atomically`。
  - 錯誤訊息是 `tower_assets.rs:283`，`load_reviewed_tw_healing_family` 中 pack SHA 比對失敗：釘選值 `B3788DF4…FCA4B218`，實際 `D:\lineage開發\0-客戶端\launcher\TW13081901.pak` 為 `92B512A3…246E59CA7`（本輪重新計算，與 #111 盤點一致）。
  - 靜態基線比較：`git diff origin/main...HEAD` 對 `tower_assets.rs`、`native_skill_effects.rs`、`tests/native_skills.rs` 沒有差異。失敗點在資源雜湊閘門，只由 pak 檔案決定，與本分支的程式無關。
  - 本機不能再執行測試，因此無法補「起點基線重跑」的執行證據。
  - 追蹤 Issue：#163（T028-1，含 19／35／57 較高治癒的當版補驗）、#111（TW 資源資格）。
  - 本張沒有放寬 SHA pin、忽略案例，也沒有把失敗當作通過。請主控決定：交由 #163 處理後，再把這三案排除在本張的驗收之外；或者要求先由 Codex 在 main 上跑同指令作為基線。
- 未啟動遊戲、未連正式服務或 DB，未做真人 GUI、DPI、焦點、讀屏驗收。合併後的版面（傳送下拉與快捷列並存、選技能 5 時技能書上移）需要 GUI 驗收。
- 目錄完整不代表 181 個技能都能施放；未接入的技能由 T020／T021／T022／T027 處理。

## 產出路徑與給下一棒的提示

- 報告：`docs/reports/issue-181.md`。
- 圖示路由與測試：`LiTo381-Modern/src/modern_lito/native_inventory_icons.rs`（`LITO_SPELL_ICONS`、`LTSPELLDESC_ROWS`、`spell_icon_route`、`skill_ui_tests`）。
- 原生 UI：`native_skill_window.rs`（`BOOK` 改為 399、未知技能圖示標註、傳送列上移）、`native_login_window.rs`。
- 控制器：`native_login.rs`（`skill_gates_open`／`skill_action_ready`／`cast_request_at`）、`native_skills.rs`；測試：`native_skill_controller_tests.rs`、`native_skillbook_tests.rs`。
- 本機證據（不進 git）：`scratch/spell-icon-route-181.json`、`scratch/verify_spell_icons.py`、`scratch/pe_table.py`、`scratch/pak_probe.py`、`scratch/check-181-r2.log`。
- 下一棒：
  1. Codex 本機跑上面列出的 ignored／整合測試並回報 `test result:` 原文。
  2. 主控裁定 P3。
  3. T046 第 2 張沿用 `skill_gates_open`／`skill_action_ready`，不要另建快捷列送包入口；buff 圖示屬於不同命名空間，不要和 `spell_icon_route` 混用。
  4. main 上 `native_rental_window.rs` 的 `SETS = 217` 與 `TELEPORT_DEST = 217` 撞號（main 既有問題，不是本張造成），建議另開任務修正。
  5. `world_hotbar.rs` 改用 `spell_icon_route`。
