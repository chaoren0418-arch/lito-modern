# SKILLS-STATUS — E149 技能狀態表（T026 第 1 張，#140）

更新：2026-11-06。這份表是 M2 完成定義的第一項：「打開 `docs/SKILLS-STATUS.md`，看到 E149 目錄每個技能一列……凡是標『以 E149 為準可驗』的列，全部顯示通過」。目前各技能大多還沒實作或驗證，「測試」欄由 M2 各批任務完成後更新。

## 來源與方法
- **技能目錄**：`docs/PROTOCOL-CONTRACT.md` §6.3，共 181 項，也就是 180 個 catalog 記錄加上 TELEPORT。欄位包括 ID、常數、記錄類型、尾段形狀，讀的是 `src/main @2066774`。E149 與它的 catalog 常數 223／223 相同（§0、§8）。
- **名稱、職業、SQL 欄位**：lito-launcher-server `2066774` 的 `lito伺服器/LiTo381-Server/docs/maintenance/combat-coverage-current.json`（SHA256 `2baa7f083cb656ef736ff23b4920247b3c0f843226193a148cf036948c6c271e`）。用到的 `skill_matrix` 欄位有 name、class_by_sql_level、sql.target、target_to、ranged、area、mpConsume、hpConsume、itemConsumeId／Count、sysmsgID_happen、sysmsgID_fail。這份是伺服器端隔離候選 `efd4bc36` 的覆蓋紀錄，SQL 欄位來自共用資料庫。它的 808 列裡「已定義」180 列加上獨立處理器 1 列（skill 5），正好對應本表 181 列。
- **客戶端狀態**：main 已接的 38 個以 `LiTo381-Modern/src/modern_lito/native_skills.rs` 的 `request()` 與 `ground_request()` 為準（#92 證據 E3）。其餘依 TASKS.md `## M2` 的 T020、T021、T022、T027 分批。
- **測試**：掃描 `LiTo381-Modern/src`、`tests` 底下所有 `#[test]` 函式，找呼叫 `native_skills` 的 `request`／`ground_request`／`self_tick_request` 的測試，並取出它測的技能 ID。傳 ID 的方式都納入：
  1. 字面：`request(&w, 26, …)`。
  2. 迴圈：`for X in [26,42]` 或 `for (X, …) in [(26,…),…]`，而呼叫的參數是 X。
  3. 欄位：參數是 `c.skill` 這類欄位時，取同一函式內的 `skill: N`、`selected(&w, N)`、`select(Some(N))`；`selected(&w, id)` 的 id 來自迴圈時，取迴圈清單。
  4. 常數：參數是大寫常數時，取同檔的 `const NAME: … = N;`。
  5. `include!`：被引入的測試檔，用引入端的常數，測試掛在引入端。例如 `native_tw_presentation_tests.rs` 由 `native_call_lightning.rs`、`native_disintegrate.rs`、`native_cone_cold.rs` 引入，`SKILL` 分別是 34、77、38。

  只收目錄 181 個 ID 內的數字。執行狀態判定：沒有 `#[ignore]` 的 lib 測試算 #101 已執行通過；整合測試在 `維護工具/test-whitelist/tests.tsv` 白名單內，算 #101 已執行通過（76／76）；其餘算未執行。經過多層函式才傳到 request 的 ID 仍可能漏抓。
- **是否需要 Ctrl**：依 E149 的 0.6.67 強制目標行為（`forced-combat-0.6.67.md:5`：Ctrl 解除明確指定目標的敵友限制，一般操作遵守模式與隊伍規則）逐列填寫。判定順序：
  1. 尾段形狀不含 `d(target)`：不適用。
  2. 記錄類型是物品類（ItemWeaponEnchant、ItemArmorEnchant、MagicalWeapon、BringStone）：不適用，目標是物品 object ID。
  3. 負面或攻擊類：SQL `target=attack`，或 SQL `type` 是 1、4、64。填「依目標而定」，對敵方不需要，對同隊需要 Ctrl。36（只能對怪物）與 145（對受控召喚物）依原始碼個別判定，見該列。
  4. 其他物件目標：type 2 輔助、16 治癒、32、128 等，填「不需要（自身／友方輔助）」。其中 SQL target=none 的列，④ 的目標可能是自身 object ID，交 T061 核對。
  5. 26、42、68 另有說明。

  `type` 的語意是依列內技能歸納的（1、4 是負面，2 是輔助，16 是治癒，64 是攻擊），文件沒有明寫。伺服器隔離候選裡更嚴格的「敵方玩家需要明確意圖」政策沒有部署在 E149，所以不列入本欄。

## 分類規則
落在下列任一伺服器缺口組的技能列為**伺服器待定**。這類技能只驗送包形狀與畫面，不驗與原版同源，依 TASKS T026。其餘列為**以 E149 為準可驗**。
- **材料**：`itemConsumeId ≠ 0`，共 58 個，對應伺服器 TASKS T025「58 列材料技能」。
- **受控 NPC**：記錄類型是 ElementalSummon、Taming、CreateZombie、Summon、ReturnToNature 之一，對應伺服器 T023。
- **立方體**：Cube* 共 4 個，屬 HP 生產端，對應伺服器 T024。
- **自施政策**：26、42，伺服器 TASKS T022 把它們列為待驗。實機回報只有 26 不按 Ctrl 無法自施，成因未判定，可能是客戶端也可能是伺服器（`source-completion-work-current.zh-TW.md:41`、`combat-integration-lifecycle.zh-TW.md:282`）。正式 0.6.71 的 26／42 合成自施封包 16 個通過，但不能當成游標驗收（同檔 `:285`）。
- **伺服器停用**：131（`combat-continuation-current.zh-TW.md:84`）。
- **處理器待查**：192、202、203、207、208、213（`combat-coverage-current.md`）。208 另有暈眩 2／3 秒的差異（`combat-parity-0.6.68.json`）。

**伺服器缺口組對照**（TASKS T026 要求對到六組）：

| 伺服器組 | 內容 | 對本表的影響 |
|---|---|---|
| T021 目標／Ctrl | 807 列 × 正常／Ctrl 的政策 | 所有物件目標技能都受影響。分類不改，因為 E149 的 Ctrl 行為（0.6.67 強制目標）本身可以驗；隔離候選的新政策不在 E149。逐列影響寫在「是否需要 Ctrl」欄 |
| T022 自施 | 26、42 | 列為伺服器待定 |
| T023 受控 NPC | 召喚、馴服、造屍、回歸自然（6 個） | 列為伺服器待定 |
| T024 HP 生產端 | 立方體 4 個；proc、娃娃、毒不是單一技能 ID | 立方體列為伺服器待定；其餘歸 T047、T026 第 2 張 |
| T025 材料 | 58 個材料技能 | 列為伺服器待定 |
| T028 顯示 | HPBAR、傷害數字、奇古獸 | 不是技能 ID 層級的缺口，分類不改；顯示由 T046、T047 處理 |

## 統計（#140 基線）

#145 增補：技能 5、12、20–23、25、27 已接入，13 個本批整合案通過完整本機套件。以下保留 #140 的盤點基線；八列最新狀態及限制見表與 [#145 報告](reports/issue-145.md)，不把離線通過當作 GUI／正式施法驗收。

- 以 E149 為準可驗：**114**；伺服器待定：**67**；合計 181。
- 伺服器待定的原因分布（一個技能可以有多個原因）：材料 58、受控 6、處理器待查 6、立方體 4、自施政策 2、伺服器停用 1、暈眩 1。
- 客戶端狀態：main 已接 38、候選獨有 48＋97／98、97–100 中未接的 99／100、未接 91（#92 E3）。
- 有送包測試的技能：38 個；其中**已執行通過**的有 14 個：14、26、31、32、34、38、42、49、52、54、72、77、78、206。來源是：
  - 白名單測試：`tests/native_selector_only.rs` 的兩個測試、`tests/native_ability_buffs.rs` 的三個測試（#101 76／76）。
  - lib 測試：`native_tw_presentation_tests.rs` 經 `include!` 引入的 `unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation`（34、38、77）。這個測試只斷言 request 的前 3 bytes（`:100-103`），不是完整封包。

  有測試、但所有測試都還沒執行的技能有 24 個。
- 「資源／特效核對」欄在 #140 初版全部是「待核對」，由 M2 各批回填；#145 已回填本批八列。

## 表
目標欄是 SQL 的 target／target_to／ranged／area；訊息欄是 happen／fail 的訊息 ID。

| ID | 名稱 | 常數 | 職業 | 記錄類型 | 尾段形狀 | 目標 | 是否需要 Ctrl（E149） | 成本 | 訊息 | 客戶端 | 送包測試（狀態） | 資源／特效核對 | 分類 | 備註 |
|---:|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 初級治癒術 | `BASIC_HEAL` | 共通魔法／法師 | HealingSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 4 | 77／280 | main 已接 | `destroyed_guardian_retires_attacks_spells_and_selection_even_if_repaired_between_polls`（未執行（白名單外））；`selection_uses_formal_packets_and_preserves_authority`（未執行（白名單外））；`spell_requests_use_formal_selectors_and_never_predict_mana`（未執行（白名單外））；`tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 2 | 日光術 | `LIGHT` | 共通魔法／法師 | LightSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 4 | 0／0 | main 已接 | `light_uses_self_selector_without_predicting_authority_and_scopes_UI_choice`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 3 | 保護罩 | `SHIELD` | 共通魔法／法師 | ShieldSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 8 | 0／0 | main 已接 | `selection_uses_formal_packets_and_preserves_authority`（未執行（白名單外））；`spell_requests_use_formal_selectors_and_never_predict_mana`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 | BUFF selector 有語意衝突，伺服器已停用 SQL 覆寫、保留既有行為，以 E149 既有行為驗 |
| 4 | 光箭 | `LIGHT_ARROW` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 3 | 0／0 | main 已接 | `hidden_targets_are_rejected_until_visible_and_identity_replacement_invalidates_selection`（未執行（白名單外））；`formal_attack_spell_layout_and_request_guards`（未執行（白名單外））；`selection_uses_formal_packets_and_preserves_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 5 | 指定傳送 | `TELEPORT` | 共通魔法／法師 | （dispatcher 特例） | ⑤ h(map) h(x) h(y) | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 已接（#145） | `native_t027_batch1`（13 案通過，詳 #145） | S5A／C08 離線通過；169 沿既有；GUI 待驗 | 以 E149 為準可驗 |  |
| 6 | 冰箭 | `ICE_ARROW` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 4 | 0／0 | main 已接 | `formal_attack_spell_layout_and_request_guards`（未執行（白名單外））；`destroyed_guardian_retires_attacks_spells_and_selection_even_if_repaired_between_polls`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 7 | 風刃 | `WIND_CUTTER` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 4 | 0／0 | main 已接 | `formal_attack_spell_layout_and_request_guards`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 8 | 神聖武器 | `HOLY_WEAPON` | 共通魔法／法師 | WeaponBuffSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 10 | 0／0 | main 已接 | `object_selector_screen_and_owner_guards_preserve_authority`（未執行（白名單外））；`spell_requests_use_formal_selectors_and_never_predict_mana`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 9 | 解毒術 | `CURE_POISON` | 共通魔法／法師 | CurseRemovalSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 211／280 | main 已接 | `object_and_coordinate_envelopes_preserve_authority_and_gate_live_targets`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 10 | 寒冷戰慄 | `CHILL_TOUCH` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 8 | 0／0 | main 已接 | `request_layout_guards_and_server_authority_do_not_predict_healing_or_hurt`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 11 | 毒咒 | `CURSE_POISON` | 共通魔法／法師 | PoisonSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 10 | 0／280 | main 已接 | `object_and_coordinate_envelopes_preserve_authority_and_gate_live_targets`（未執行（白名單外）） | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 12 | 擬似魔法武器 | `ENCHANT_WEAPON` | 共通魔法／法師 | ItemWeaponEnchantSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不適用（目標是物品 object ID，不是角色） | MP 20 | 0／0 | 已接（#145） | `native_t027_batch1`（13 案通過，詳 #145） | 原資源離線通過；GUI 待驗 | 以 E149 為準可驗 |  |
| 13 | 無所遁形術 | `DETECTION` | 共通魔法／法師 | DetectionSpell | ① 無尾段 | none／3／射程 0／範圍 -1 | 不適用（非物件目標） | MP 15 | 0／0 | main 已接 | `detection_selector_menu_guards_and_cooldown_leave_authority_unchanged`（未執行（白名單外））；`invisibility_tick_envelope_guards_clock_and_menu_preserve_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 14 | 負重強化 | `DECREASE_WEIGHT` | 共通魔法／法師 | WeightSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10、HP 50 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 15 | 火箭 | `FIRE_ARROW` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 6 | 0／0 | main 已接 | `formal_attack_spell_layout_and_request_guards`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 16 | 地獄之牙 | `STALAC` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 6 | 0／0 | main 已接 | `stalac_packet_range_clock_and_no_predicted_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 17 | 極光雷電 | `LIGHTNING` | 共通魔法／法師 | LineAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | main 已接 | `request_clock_reuse_range_and_target_guard_preserve_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 18 | 起死回生術 | `TURN_UNDEAD` | 共通魔法／法師 | TurnUndeadSpell | ⑩ d(target) h(x) h(y) | attack／2／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／280 | main 已接 | `low_mana_turn_undead_times_out_without_predicted_mana_death_or_rejection`（未執行（白名單外））；`selector_clock_range_and_dead_target_guards_never_predict_death_or_mana`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 19 | 中級治癒術 | `EXTRA_HEAL` | 共通魔法／法師 | HealingSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 77／280 | main 已接 | `tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 20 | 闇盲咒術 | `CURSE_BLIND` | 共通魔法／法師 | BlindSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／280 | 已接（#145） | `native_t027_batch1`（13 案通過，詳 #145） | 原資源離線通過；GUI 待驗 | 以 E149 為準可驗 |  |
| 21 | 鎧甲護持 | `BLESSED_ARMOR` | 共通魔法／法師 | ItemArmorEnchantSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不適用（目標是物品 object ID，不是角色） | MP 20 | 0／0 | 已接（#145） | `native_t027_batch1`（13 案通過，詳 #145） | 原資源離線通過；GUI 待驗 | 以 E149 為準可驗 |  |
| 22 | 寒冰氣息 | `FROZEN_CLOUD` | 共通魔法／法師 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | 已接（#145） | `native_t027_batch1`（13 案通過，詳 #145） | 原資源離線通過；GUI 待驗 | 以 E149 為準可驗 |  |
| 23 | 能量感測 | `REVEAL_WEAKNESS` | 共通魔法／法師 | RevealWeaknessSpell | ④ d(target) | buff／2／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 8 | 0／280 | 已接（#145） | `native_t027_batch1`（13 案通過，詳 #145） | 原資源離線通過；GUI 待驗 | 以 E149 為準可驗 |  |
| 25 | 燃燒的火球 | `BURNING_FIREBALL` | 共通魔法／法師 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | 已接（#145） | `native_t027_batch1`（13 案通過，詳 #145） | 原資源離線通過；GUI 待驗 | 以 E149 為準可驗 |  |
| 26 | 通暢氣脈術 | `PHYSICAL_ENCHANT_DEX` | 共通魔法／法師 | AbilityBuffSpell | ④ d(target) | buff／4／射程 -1／範圍 0 | 政策上不需要（自身用自身 object ID 走 7-byte 分派）；實機回報不按 Ctrl 無法自施，成因未判定（`source-completion-work-current.zh-TW.md:41`、`combat-integration-lifecycle.zh-TW.md:282`、`:285`） | MP 50 | 294／280 | main 已接 | `ability_buffs_keep_native_screen_asymmetric_edges`（已執行通過（白名單，#101））；`ability_buffs_reject_unlearned_dead_missing_and_outside_screen_targets`（已執行通過（白名單，#101））；`both_ability_buffs_use_explicit_self_or_selected_object_without_authority_writes`（已執行通過（白名單，#101）） | 待核對 | 伺服器待定：自施政策（伺服器 TASKS T022 待驗） |  |
| 27 | 壞物術 | `WEAPON_BREAK` | 共通魔法／法師 | WeaponBreakSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 4／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 32 | 0／280 | 已接（#145） | `native_t027_batch1`（13 案通過，詳 #145） | 原資源離線通過；GUI 待驗 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 28 | 吸血鬼之吻 | `VAMPIRIC_TOUCH` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 4／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 12 | 0／0 | main 已接 | `vampiric_envelope_range_and_authority_do_not_predict_healing_or_hurt`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 29 | 緩速術 | `SLOW` | 共通魔法／法師 | EntangleSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 277／280 | 已接（#146） | `native_t027_batch2`（11 案通過，詳 #146） | S29／752 原影格通過；GUI 待驗 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 30 | 巖牢 | `EARTH_JAIL` | 共通魔法／法師 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 8／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 24 | 0／0 | 已接（#146） | `native_t027_batch2`（11 案通過，詳 #146） | S39／1805＋1807＋1806、兩次 Hit、煙霧尾段通過；GUI 待驗 | 以 E149 為準可驗 |  |
| 31 | 魔法屏障 | `COUNTER_MAGIC` | 共通魔法／法師 | CounterMagicSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 32 | 冥想術 | `MEDITATION` | 共通魔法／法師 | RegenSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 5、HP 30 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 33 | 木乃伊的詛咒 | `CURSE_PARALYZE` | 共通魔法／法師 | ParalysisSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 2／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 150 | 0／280 | 已接（#146） | `native_t027_batch2`（11 案通過，詳 #146） | S25／746 收包與原資源通過；GUI 待驗 | 伺服器待定：材料 40318×2（伺服器 T025） |  |
| 34 | 極道落雷 | `CALL_LIGHTNING` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 18 | 0／0 | main 已接 | `unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation（include! native_tw_presentation_tests.rs）`（已執行通過（lib，#101））；`current_c02_exact_object_coordinates_and_readonly_authority`（未執行（白名單外））；`invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests`（未執行（白名單外））；`mp_and_lawful_are_server_permission_inputs_and_never_locally_spent`（未執行（白名單外））；`range_six_boundary_and_target_changes_use_current_received_coordinates`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 35 | 高級治癒術 | `GREATER_HEAL` | 共通魔法／法師 | HealingSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 77／280 | main 已接 | `tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 36 | 迷魅術 | `TAMING_MONSTER` | 共通魔法／法師 | TamingSpell | ⑩ d(target) h(x) h(y) | buff／2／射程 -1／範圍 0 | 不需要：目標只能是 WORLD／Quest／Time Attack 的怪物，找不到就拒絕非怪物目標（`ClientSession.java:72350-72368`）；Quest／Time Attack 用 canAttackQuestMonster／canAttackTimeAttackActor 判定，不經 ForcedCombat。讀的是 src/main @2066774，E149 待核對 | MP 30、HP 20 | 0／280 | 已接（#146） | `native_t027_batch2`（11 案通過，詳 #146） | 228 原影格／Sound132／容量319通過；正式同伴待驗 | 伺服器待定：材料 40318×1（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 37 | 聖潔之光 | `REMOVE_CURSE` | 共通魔法／法師 | CurseRemovalSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 155／280 | 已接（#146） | `native_t027_batch2`（11 案通過，詳 #146） | S25／227 原影格／Sound122通過；GUI 待驗 | 以 E149 為準可驗 |  |
| 38 | 冰錐 | `CONE_OF_COLD` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 18 | 0／0 | main 已接 | `unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation（include! native_tw_presentation_tests.rs）`（已執行通過（lib，#101））；`cold_request_uses_observed_C14_and_six_cell_range_without_spending_authority`（未執行（白名單外））；`cold_selector_rechecks_target_life_generation_owner_and_book`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 39 | 魔力奪取 | `MANA_DRAIN` | 共通魔法／法師 | ManaDrainSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 4／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 1、HP 50 | 690／280 | 已接（#146） | `native_t027_batch2`（11 案通過，詳 #146） | 2171 原影格／Sound806、HP／MP權威更新通過；GUI 待驗 | 以 E149 為準可驗 |  |
| 40 | 黑闇之影 | `DARKNESS` | 共通魔法／法師 | BlindSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 15 | 0／280 | 已接（#146） | `native_t027_batch2`（11 案通過，詳 #146） | S0A／2175 原影格／Multiply通過；GUI 待驗 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 41 | 造屍術 | `CREATE_ZOMBIE` | 共通魔法／法師 | CreateZombieSpell | ④ d(target) | buff／2／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 5、HP 80 | 0／280 | 已接（#146） | `native_t027_batch2`（11 案通過，詳 #146） | 226 原影格／Sound139、屍體移除後固定位置通過；正式同伴待驗 | 伺服器待定：材料 40318×1（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 42 | 體魄強健術 | `PHYSICAL_ENCHANT_STR` | 共通魔法／法師 | AbilityBuffSpell | ④ d(target) | buff／4／射程 -1／範圍 0 | 不需要（自身用自身 object ID 走 7-byte 分派） | MP 50 | 292／280 | main 已接 | `ability_buffs_keep_native_screen_asymmetric_edges`（已執行通過（白名單，#101））；`ability_buffs_reject_unlearned_dead_missing_and_outside_screen_targets`（已執行通過（白名單，#101））；`both_ability_buffs_use_explicit_self_or_selected_object_without_authority_writes`（已執行通過（白名單，#101）） | 待核對 | 伺服器待定：自施政策（伺服器 TASKS T022 待驗） |  |
| 43 | 加速術 | `HASTE` | 共通魔法／法師 | HasteSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 40 | 184／280 | main 已接 | `selection_uses_formal_packets_and_preserves_authority`（未執行（白名單外））；`spell_requests_use_formal_selectors_and_never_predict_mana`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 44 | 魔法相消術 | `CANCELLATION` | 共通魔法／法師 | CancellationSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 40 | 0／280 | 已接（T027 批 3／#147） | `native_t027_batch3::eight_wire_shapes_are_exact_and_costs_are_not_predicted` 通過 | 870→232／Sound140；三格，7-byte actor | 伺服器待定：材料 40318×2（伺服器 T025） |  |
| 45 | 地裂術 | `ERUPTION` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 8／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | 已接（T027 批 3／#147） | `native_t027_batch3::eight_wire_shapes_are_exact_and_costs_are_not_predicted` 通過 | 129→123／130、124 陰影；原資源離線通過，原 hurt 待核 | 以 E149 為準可驗 |  |
| 46 | 烈炎術 | `SUNBURST` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | 已接（T027 批 3／#147） | `native_t027_batch3::eight_wire_shapes_are_exact_and_costs_are_not_predicted` 通過 | 1811／Sound709／source11 Hit 診斷；原資源離線通過，原 hurt 待核 | 以 E149 為準可驗 |  |
| 47 | 弱化術 | `WEAKNESS` | 共通魔法／法師 | WeaknessSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 10 | 692／280 | 已接（T027 批 3／#147） | `native_t027_batch3::eight_wire_shapes_are_exact_and_costs_are_not_predicted` 通過 | 2228／Sound843／Multiply；原資源離線通過 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 48 | 祝福魔法武器 | `BLESS_WEAPON` | 共通魔法／法師 | WeaponBuffSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 0／0 | 已接（T027 批 3／#147） | `native_t027_batch3::eight_wire_shapes_are_exact_and_costs_are_not_predicted` 通過 | 2176／Sound800；原資源離線通過 | 以 E149 為準可驗 |  |
| 49 | 體力回復術 | `HEAL_ALL` | 共通魔法／法師 | GroupHealingSpell | ① 無尾段 | none／20／射程 0／範圍 -1 | 不適用（非物件目標） | MP 50、HP 30 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 50 | 冰矛圍籬 | `ICE_LANCE` | 共通魔法／法師 | FreezeAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 100 | 0／0 | 已接（T027 批 3／#147） | `native_t027_batch3::eight_wire_shapes_are_exact_and_costs_are_not_predicted` 通過 | 756→219／無音效；原資源離線通過，原 hurt 待核 | 以 E149 為準可驗 |  |
| 51 | 召喚術 | `SUMMON_MONSTER` | 共通魔法／法師 | SummonSpell | ⑨ d(selection) | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30、HP 30 | 0／0 | 已接（T027 批 3／#147） | `native_t027_batch3::eight_wire_shapes_are_exact_and_costs_are_not_predicted` 通過 | 31 個明確召喚選項；2510、755 不作成功 ACK；原資源離線通過 | 伺服器待定：材料 40318×3（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 52 | 神聖疾走 | `HOLY_WALK` | 共通魔法／法師 | MovementBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 15 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 53 | 龍捲風 | `TORNADO` | 共通魔法／法師 | AreaAttackSpell | ① 無尾段 | none／3／射程 0／範圍 4 | 不適用（非物件目標） | MP 40 | 0／0 | 已接（T027 批 3／#147） | `native_t027_batch3::eight_wire_shapes_are_exact_and_costs_are_not_predicted` 通過 | 758／Sound125／四圖一組；原資源離線通過 | 以 E149 為準可驗 |  |
| 54 | 強力加速術 | `GREATER_HASTE` | 共通魔法／法師 | SelfHasteSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 60 | 184／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 55 | 狂暴術 | `BERSERKERS` | 共通魔法／法師 | CombatBuffSpell | ④ d(target) | buff／12／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 40 | 0／280 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 56 | 疾病術 | `DISEASE` | 共通魔法／法師 | WeaknessSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 10 | 0／280 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 57 | 全部治癒術 | `FULL_HEAL` | 共通魔法／法師 | HealingSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 48 | 77／280 | main 已接 | `tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 58 | 火牢 | `FIRE_WALL` | 共通魔法／法師 | GroundSpell | ③ h(x) h(y) | attack／3／射程 8／範圍 0 | 不適用（非物件目標） | MP 60 | 0／0 | main 已接 | `formal_ground_request_unsigned_boundaries_range_and_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 59 | 冰雪暴 | `BLIZZARD` | 共通魔法／法師 | AreaAttackSpell | ① 無尾段 | none／3／射程 0／範圍 10 | 不適用（非物件目標） | MP 60、HP 10 | 0／0 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 60 | 隱身術 | `INVISIBILITY` | 共通魔法／法師 | InvisibilitySpell | ② d(clientTick) | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 15 | 0／0 | main 已接 | `invisibility_tick_envelope_guards_clock_and_menu_preserve_authority`（未執行（白名單外）） | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 61 | 返生術 | `RESURRECTION` | 共通魔法／法師 | ResurrectionSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 50 | 0／280 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 62 | 震裂術 | `EARTHQUAKE` | 共通魔法／法師 | AreaAttackSpell | ① 無尾段 | none／3／射程 0／範圍 3 | 不適用（非物件目標） | MP 40 | 0／0 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 63 | 治癒能量風暴 | `LIFE_STREAM` | 共通魔法／法師 | GroundSpell | ③ h(x) h(y) | buff／32／射程 -1／範圍 3 | 不適用（非物件目標） | MP 20 | 0／0 | main 已接 | `formal_ground_request_unsigned_boundaries_range_and_authority`（未執行（白名單外））；`formal_reagent_request_self_screen_boundary_and_no_prediction`（未執行（白名單外）） | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 64 | 魔法封印 | `SILENCE` | 共通魔法／法師 | SilenceSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 50 | 0／280 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 65 | 雷霆風暴 | `LIGHTNING_STORM` | 共通魔法／法師 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 48 | 0／0 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 66 | 沉睡之霧 | `FOG_OF_SLEEPING` | 共通魔法／法師 | SleepSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 10／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 50 | 0／280 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 67 | 變形術 | `SHAPE_CHANGE` | 共通魔法／法師 | PolymorphSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 80 | 0／280 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 68 | 聖結界 | `IMMUNE_TO_HARM` | 共通魔法／法師 | DamageProtectionSpell | ④ d(target) | buff／4／射程 -1／範圍 0 | 不需要（自身／血盟；Ctrl 成功封包同普通施放，不能繞過血盟限制） | MP 30 | 314／280 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×2（伺服器 T025） | 目標生命週期：扣料或安裝前目標退役、receiver 換世代時要拒絕或丟棄（#120 新 N1，T061） |
| 69 | 集體傳送術 | `MASS_TELEPORT` | 共通魔法／法師 | MassTeleportSpell | ⑥ h(map) h(x) h(y) | none／4／射程 0／範圍 2 | 不適用（非物件目標） | MP 60 | 0／0 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 70 | 火風暴 | `FIRE_STORM` | 共通魔法／法師 | AreaAttackSpell | ① 無尾段 | none／3／射程 0／範圍 3 | 不適用（非物件目標） | MP 48 | 0／0 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 71 | 藥水霜化術 | `DECAY_POTION` | 共通魔法／法師 | DecayPotionSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 80 | 0／280 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×2（伺服器 T025） |  |
| 72 | 強力無所遁形術 | `COUNTER_DETECTION` | 共通魔法／法師 | CounterDetectionSpell | ① 無尾段 | none／3／射程 0／範圍 -1 | 不適用（非物件目標） | MP 30 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 73 | 創造魔法武器 | `CREATE_MAGICAL_WEAPON` | 共通魔法／法師 | MagicalWeaponSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不適用（目標是物品 object ID，不是角色） | MP 200 | 685／0 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×10（伺服器 T025） |  |
| 74 | 流星雨 | `METEOR_STRIKE` | 共通魔法／法師 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 60 | 0／0 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 75 | 終極返生術 | `GREATER_RESURRECTION` | 共通魔法／法師 | ResurrectionSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 100 | 0／280 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 76 | 集體緩速術 | `MASS_SLOW` | 共通魔法／法師 | MassSlowSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／280 | 候選獨有（T027 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40318×2（伺服器 T025） |  |
| 77 | 究極光裂術 | `DISINTEGRATE` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 70 | 0／0 | main 已接 | `unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation（include! native_tw_presentation_tests.rs）`（已執行通過（lib，#101））；`current_c02_exact_object_coordinates_and_readonly_authority`（未執行（白名單外））；`invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests`（未執行（白名單外））；`mp_and_lawful_are_server_permission_inputs_and_never_locally_spent`（未執行（白名單外））；`range_ten_boundary_and_target_changes_use_current_received_coordinates`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 78 | 絕對屏障 | `ABSOLUTE_BARRIER` | 共通魔法／法師 | AbsoluteBarrierSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 伺服器待定：材料 40318×2（伺服器 T025） |  |
| 79 | 靈魂昇華 | `ADVANCE_SPIRIT` | 共通魔法／法師 | AdvanceSpiritSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 0／280 | 候選獨有（T027 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 80 | 冰雪颶風 | `FREEZING_BLIZZARD` | 共通魔法／法師 | FreezeAttackSpell | ① 無尾段 | none／3／射程 0／範圍 10 | 不適用（非物件目標） | MP 60 | 0／0 | 候選獨有（T027 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 87 | 衝擊之暈 | `SHOCK_STUN` | 騎士 | ShockStunSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 15 | 0／280 | 候選獨有（T027 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 88 | 增幅防禦 | `REDUCTION_ARMOR` | 騎士 | ReductionArmorSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 5、HP 50 | 0／0 | 候選獨有（T027 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 89 | 尖刺盔甲 | `BOUNCE_ATTACK` | 騎士 | BounceAttackSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 5、HP 120 | 0／0 | 候選獨有（T027 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 90 | 堅固防護 | `SOLID_CARRIAGE` | 騎士 | SolidCarriageSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10、HP 100 | 0／0 | 候選獨有（T027 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 | 裝備模式（盾 type7；臂甲 type13 僅 mode=1）只在伺服器隔離候選，E149 照自身行為驗 |
| 91 | 反擊屏障 | `COUNTER_BARRIER` | 騎士 | CounterBarrierSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10、HP 100 | 0／0 | 候選獨有（T027 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 97 | 暗隱術 | `BLIND_HIDING` | 黑暗妖精 | InvisibilitySpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 候選獨有（T020／T021） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 98 | 附加劇毒 | `ENCHANT_VENOM` | 黑暗妖精 | EnchantVenomSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10、HP 1 | 814／0 | 候選獨有（T021） | 尚無 | 待核對 | 伺服器待定：材料 40321×1（伺服器 T025） |  |
| 99 | 影之防護 | `SHADOW_ARMOR` | 黑暗妖精 | ShieldSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10、HP 5 | 0／0 | 未接（T021） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 100 | 提煉魔石 | `BRING_STONE` | 黑暗妖精 | BringStoneSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不適用（目標是物品 object ID，不是角色） | MP 5 | 0／0 | 未接（T021） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 101 | 行走加速 | `MOVING_ACCELERATION` | 黑暗妖精 | MovementBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 102 | 燃燒鬥志 | `BURNING_SPIRIT` | 黑暗妖精 | PhysicalProcBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 103 | 暗黑盲咒 | `DARK_BLIND` | 黑暗妖精 | BlindSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 0／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 147／280 | 未接（T022 批 1） | 尚無 | 待核對 | 伺服器待定：材料 40321×1（伺服器 T025） |  |
| 104 | 毒性抵抗 | `VENOM_RESIST` | 黑暗妖精 | PoisonResistanceSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 105 | 雙重破壞 | `DOUBLE_BRAKE` | 黑暗妖精 | PhysicalProcBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 823／0 | 未接（T022 批 1） | 尚無 | 待核對 | 伺服器待定：材料 40321×1（伺服器 T025） |  |
| 106 | 暗影閃避 | `UNCANNY_DODGE` | 黑暗妖精 | DodgeBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 107 | 暗影之牙 | `SHADOW_FANG` | 黑暗妖精 | ItemWeaponEnchantSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不適用（目標是物品 object ID，不是角色） | MP 30 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 伺服器待定：材料 40321×1（伺服器 T025） |  |
| 108 | 會心一擊 | `FINAL_BURN` | 黑暗妖精 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 1、HP 1 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 109 | 力量提升 | `DRESS_MIGHTY` | 黑暗妖精 | AbilityBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 292／0 | 未接（T022 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 110 | 敏捷提升 | `DRESS_DEXTERITY` | 黑暗妖精 | AbilityBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 294／0 | 未接（T022 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 111 | 閃避提升 | `DRESS_EVASION` | 黑暗妖精 | RangedEvasionBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 112 | 破壞盔甲 | `ARMOR_BREAK` | 黑暗妖精 | ArmorBreakSpell | ① 無尾段 | buff／1／射程 1／範圍 0 | 不適用（非物件目標） | MP 40、HP 30 | 0／280 | 未接（T022 批 2） | 尚無 | 待核對 | 伺服器待定：材料 40321×2（伺服器 T025） |  |
| 113 | 精準目標 | `TRUE_TARGET` | 王族 | TrueTargetSpell | ⑪ d(target) h(x) h(y) s(text) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 1 | 0／0 | 未接（T022 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 114 | 激勵士氣 | `GLOWING_AURA` | 王族 | RoyalAuraSpell | ① 無尾段 | none／8／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 115 | 鋼鐵士氣 | `SHINING_AURA` | 王族 | RoyalAuraSpell | ① 無尾段 | none／12／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 116 | 呼喚盟友 | `CALL_CLAN` | 王族 | CallClanSpell | ⑦ s(name) | buff／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 0／280 | 未接（T022 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 117 | 衝擊士氣 | `BRAVE_AURA` | 王族 | RoyalAuraSpell | ① 無尾段 | none／8／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 118 | 援護盟友 | `RUN_CLAN` | 王族 | RunClanSpell | ⑧ s(name) | buff／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 0／280 | 未接（T022 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 129 | 魔法防禦 | `RESIST_MAGIC` | 妖精 | MagicResistanceSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 5 | 0／0 | 未接（T022 批 3） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 130 | 心靈轉換 | `BODY_TO_MIND` | 妖精 | BodyToMindSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 8 | 702／0 | 未接（T022 批 3） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 131 | 世界樹的呼喚 | `WORLD_TREE_CALL` | 妖精 | WorldTreeCallSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | main 已接 | `learned_world_tree_is_selector_only_and_cross_map_admission_stays_owned`（未執行（白名單外）） | 待核對 | 伺服器待定：伺服器停用 |  |
| 132 | 三重矢 | `TRIPLE_ARROW` | 妖精 | TripleArrowSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 -1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 11 | 0／0 | 未接（T022 批 3） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 133 | 弱化屬性 | `ELEMENTAL_FALL_DOWN` | 妖精 | ElementalFallDownSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 10 | 0／280 | 未接（T022 批 3） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 134 | 鏡反射 | `COUNTER_MIRROR` | 妖精 | CounterMirrorSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／280 | 未接（T022 批 3） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 137 | 淨化精神 | `CLEAR_MIND` | 妖精 | ClearMindSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 未接（T022 批 3） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 138 | 屬性防禦 | `RESIST_ELEMENTAL` | 妖精 | ResistElementalSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 未接（T022 批 3） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 145 | 釋放元素 | `RETURN_TO_NATURE` | 妖精 | ReturnToNatureSpell | ⑩ d(target) h(x) h(y) | buff／16／射程 -1／範圍 0 | 待核對：目標是任一玩家的受控召喚物（只有 SUMMON／TAMED 可以釋放），要求同房間或同地圖（`ClientSession.java:49742` 起）。讀到的範圍內沒有看到敵友或 ForcedCombat 判定；ForcedCombat 本身對玩家的受控召喚物也會放寬敵友（`ForcedCombat.java:78-80`）。E149 普通與 Ctrl 是否有差異待 T061 核對 | MP 30 | 0／280 | 未接（T022 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40319×2（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 146 | 魂體轉換 | `BLOODY_SOUL` | 妖精 | BodyToMindSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 40 | 0／0 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 147 | 單屬性防禦 | `ELEMENTAL_PROTECTION` | 妖精 | ElementalProtectionSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 6 | 0／0 | 未接（T022 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 148 | 火焰武器 | `FIRE_WEAPON` | 妖精 | FireWeaponSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 0／280 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 149 | 風之神射 | `WIND_SHOT` | 妖精 | WindShotSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 0／280 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 150 | 風之疾走 | `WIND_WALK` | 妖精 | MovementBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 15 | 0／0 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 151 | 大地防護 | `EARTH_SKIN` | 妖精 | ShieldSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 0／280 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 | BUFF selector 有語意衝突，伺服器已停用 SQL 覆寫、保留既有行為，以 E149 既有行為驗 |
| 152 | 地面障礙 | `ENTANGLE` | 妖精 | EntangleSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 711／280 | 未接（T022 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 153 | 魔法消除 | `ERASE_MAGIC` | 妖精 | EraseMagicSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 10 | 0／280 | 未接（T022 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 154 | 召喚屬性精靈 | `LESSER_ELEMENTAL` | 妖精 | ElementalSummonSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40319×2（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 155 | 烈炎氣息 | `FIRE_BLESS` | 妖精 | FireBlessSpell | ① 無尾段 | none／8／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 156 | 暴風之眼 | `STORM_EYE` | 妖精 | StormEyeSpell | ① 無尾段 | none／8／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 157 | 大地屏障 | `EARTH_BIND` | 妖精 | BindSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 50 | 0／280 | 未接（T022 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40319×2（伺服器 T025） |  |
| 158 | 生命之泉 | `NATURES_TOUCH` | 妖精 | NaturesTouchSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 0／280 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 159 | 大地的祝福 | `EARTH_BLESS` | 妖精 | EarthBlessSpell | ① 無尾段 | none／8／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 160 | 水之防護 | `AQUA_PROTECTER` | 妖精 | AquaProtectSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 30 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 161 | 封印禁地 | `AREA_OF_SILENCE` | 妖精 | AreaSilenceSpell | ① 無尾段 | none／3／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40319×8（伺服器 T025） |  |
| 162 | 召喚強力屬性精靈 | `GREATER_ELEMENTAL` | 妖精 | ElementalSummonSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40319×4（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 163 | 烈炎武器 | `BURNING_WEAPON` | 妖精 | FireWeaponSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助）；SQL target=none，④ 的目標可能是自身 object ID，T061 核對 | MP 30 | 0／280 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 164 | 生命的祝福 | `NATURES_BLESSING` | 妖精 | NaturesBlessingSpell | ① 無尾段 | none／24／射程 0／範圍 -1 | 不適用（非物件目標） | MP 30 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 165 | 生命呼喚 | `CALL_OF_NATURE` | 妖精 | ResurrectionSpell | ④ d(target) | buff／3／射程 10／範圍 0 | 不需要（自身／友方輔助） | MP 50 | 0／280 | 未接（T022 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 166 | 暴風神射 | `STORM_SHOT` | 妖精 | StormShotSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助）；SQL target=none，④ 的目標可能是自身 object ID，T061 核對 | MP 30 | 0／280 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 167 | 風之枷鎖 | `WIND_SHACKLE` | 妖精 | WindShackleSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 50 | 1001／280 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 168 | 鋼鐵防護 | `IRON_SKIN` | 妖精 | IronSkinSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助）；SQL target=none，④ 的目標可能是自身 object ID，T061 核對 | MP 30 | 714／280 | 未接（T022 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 169 | 體能激發 | `EXOTIC_VITALIZE` | 妖精 | ExoticVitalizeSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 0／0 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 170 | 水之元氣 | `WATER_LIFE` | 妖精 | WaterLifeSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 1 | 0／280 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 171 | 屬性之火 | `ELEMENTAL_FIRE` | 妖精 | PhysicalProcBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 172 | 暴風疾走 | `STORM_WALK` | 妖精 | OtherNoEffectSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 未接（T022 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 173 | 污濁之水 | `POLLUTE_WATER` | 妖精 | PolluteWaterSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 50 | 0／280 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 174 | 精準射擊 | `STRIKER_GALE` | 妖精 | StrikerGaleSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 10 | 0／280 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×5（伺服器 T025） |  |
| 175 | 烈焰之魂 | `SOUL_OF_FLAME` | 妖精 | PhysicalProcBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 0／0 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×3（伺服器 T025） |  |
| 176 | 能量激發 | `ADDITIONAL_FIRE` | 妖精 | AdditionalFireSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 0／0 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 181 | 龍之護鎧 | `DRAGON_SKIN` | 龍騎士 | DragonSkinSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 15 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 182 | 燃燒擊砍 | `BURNING_SLASH` | 龍騎士 | BurningSlashSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 10 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 183 | 護衛毀滅 | `GUARD_BRAKE` | 龍騎士 | GuardBrakeSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 0、HP 25 | 0／280 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 184 | 岩漿噴吐 | `MAGMA_BREATH` | 龍騎士 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 0、HP 35 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 185 | 覺醒：安塔瑞斯 | `AWAKEN_ANTHARAS` | 龍騎士 | AwakenSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 1 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 伺服器待定：材料 49157×1（伺服器 T025） |  |
| 186 | 血之渴望 | `BLOODLUST` | 龍騎士 | BloodlustSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 30 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 伺服器待定：材料 49157×1（伺服器 T025） |  |
| 187 | 屠宰者 | `FOE_SLAYER` | 龍騎士 | FoeSlayerSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 2／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 0、HP 20 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 188 | 恐懼無助 | `RESIST_FEAR` | 龍騎士 | ResistFearSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 0、HP 30 | 0／280 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 189 | 岩漿之箭 | `SHOCK_SKIN` | 龍騎士 | AreaAttackSpell | ① 無尾段 | none／3／射程 0／範圍 2 | 不適用（非物件目標） | MP 0、HP 40 | 0／0 | 未接（T022 批 8） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 190 | 覺醒：法利昂 | `AWAKEN_FAFURION` | 龍騎士 | AwakenSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 1 | 0／0 | 未接（T022 批 8） | 尚無 | 待核對 | 伺服器待定：材料 49157×1（伺服器 T025） |  |
| 191 | 致命身軀 | `MORTAL_BODY` | 龍騎士 | MortalBodySpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 50 | 0／0 | 未接（T022 批 8） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 192 | 奪命之雷 | `THUNDER_GRAB` | 龍騎士 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 8／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 0、HP 45 | 0／0 | 未接（T022 批 8） | 尚無 | 待核對 | 伺服器待定：處理器待查 |  |
| 193 | 驚悚死神 | `HORROR_OF_DEATH` | 幻術師 | GuardBrakeSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 0、HP 50 | 0／280 | 未接（T022 批 8） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 194 | 暴龍之眼 | `FREEZING_BREATH` | 幻術師 | FreezeAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 0、HP 50 | 0／0 | 未接（T022 批 8） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 195 | 覺醒：巴拉卡斯 | `AWAKEN_VALAKAS` | 幻術師 | AwakenSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 1 | 0／0 | 未接（T022 批 8） | 尚無 | 待核對 | 伺服器待定：材料 49157×2（伺服器 T025） |  |
| 201 | 鏡像 | `MIRROR_IMAGE` | 幻術師 | DodgeBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 未接（T022 批 9） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 202 | 混亂 | `CONFUSION` | 幻術師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 15、HP 10 | 0／0 | 未接（T022 批 9） | 尚無 | 待核對 | 伺服器待定：處理器待查 |  |
| 203 | 粉碎能量 | `SMASH` | 幻術師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 1／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 7 | 0／0 | 未接（T022 批 9） | 尚無 | 待核對 | 伺服器待定：處理器待查 |  |
| 204 | 幻覺：歐吉 | `ILLUSION_OGRE` | 幻術師 | IllusionOgreSpell | ⑩ d(target) h(x) h(y) | buff／1／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 0／0 | 未接（T022 批 9） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 205 | 立方：燃燒 | `CUBE_IGNITION` | 幻術師 | CubeIgnitionSpell | ① 無尾段 | none／32／射程 0／範圍 3 | 不適用（非物件目標） | MP 30 | 0／0 | 未接（T022 批 9） | 尚無 | 待核對 | 伺服器待定：材料 49156×1（伺服器 T025）、立方體 HP 生產端（伺服器 T024） |  |
| 206 | 專注 | `CONCENTRATION` | 幻術師 | RegenSpell | ① 無尾段 | buff／1／射程 3／範圍 0 | 不適用（非物件目標） | MP 20 | 0／280 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 207 | 心靈破壞 | `MIND_BREAK` | 幻術師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20、HP 15 | 0／0 | 未接（T022 批 9） | 尚無 | 待核對 | 伺服器待定：處理器待查 |  |
| 208 | 骷髏毀壞 | `BONE_BREAK` | 幻術師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | 未接（T022 批 9） | 尚無 | 待核對 | 伺服器待定：處理器待查、暈眩 2／3 秒差異 |  |
| 209 | 幻覺：巫妖 | `ILLUSION_LICH` | 幻術師 | IllusionLichSpell | ⑩ d(target) h(x) h(y) | buff／1／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 0／280 | 未接（T022 批 9） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 210 | 立方：地裂 | `CUBE_QUAKE` | 幻術師 | CubeQuakeSpell | ① 無尾段 | none／32／射程 0／範圍 3 | 不適用（非物件目標） | MP 35 | 0／0 | 未接（T022 批 9） | 尚無 | 待核對 | 伺服器待定：材料 49156×1（伺服器 T025）、立方體 HP 生產端（伺服器 T024） |  |
| 211 | 耐力 | `PATIENCE` | 幻術師 | PatienceSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 25 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 212 | 幻想 | `PHANTASM` | 幻術師 | PhantasmSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 30、HP 25 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 213 | 武器破壞者 | `ARM_BREAKER` | 幻術師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 25 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 伺服器待定：處理器待查 |  |
| 214 | 幻覺：鑽石高崙 | `ILLUSION_DIAMOND_GOLEM` | 幻術師 | DiamondGolemSpell | ⑩ d(target) h(x) h(y) | buff／1／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 25 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 215 | 立方：衝擊 | `CUBE_SHOCK` | 幻術師 | CubeShockSpell | ① 無尾段 | none／32／射程 0／範圍 3 | 不適用（非物件目標） | MP 55 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 伺服器待定：材料 49156×1（伺服器 T025）、立方體 HP 生產端（伺服器 T024） |  |
| 216 | 洞察 | `INSIGHT` | 幻術師 | InsightSpell | ⑩ d(target) h(x) h(y) | buff／1／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 40 | 0／280 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 217 | 恐慌 | `PANIC` | 幻術師 | PanicSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 30、HP 30 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 218 | 疼痛的歡愉 | `JOY_OF_PAIN` | 幻術師 | JoyOfPainSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 40 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 219 | 幻覺：化身 | `ILLUSION_AVATAR` | 幻術師 | IllusionAvatarSpell | ⑩ d(target) h(x) h(y) | buff／1／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 35 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 220 | 立方：和諧 | `CUBE_HARMONY` | 幻術師 | CubeHarmonySpell | ① 無尾段 | none／32／射程 0／範圍 3 | 不適用（非物件目標） | MP 55 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 伺服器待定：材料 49156×2（伺服器 T025）、立方體 HP 生產端（伺服器 T024） |  |

## 表外的訊息 ID
SQL 的 happen／fail 欄只記技能本身的訊息。以下是伺服器在拒絕或特殊情況送出、不在表內的訊息 ID。客戶端要能顯示它們，顯示由 T046、T049 處理。出處都在 lito-launcher-server `2066774` 的 `lito伺服器/LiTo381-Server/docs/maintenance/`：

| 訊息 ID | 情境 | 出處 |
|---:|---|---|
| 278 | 自施技能 MP 不足（Light 在 MP 不足時不送任何訊息） | `native-reuse-lease-current.zh-TW.md:19` |
| 279 | HP 不足（HP 成本技能） | `native-zero-mp-current.zh-TW.md:13` |
| 299 | 缺材料，或保存前失敗 | `native-cast-material-progress.zh-TW.md:18-19`、`native-magical-weapon-material-progress.zh-TW.md:3` |
| 1412 | 立方體重複或無效放置 | `native-cube-material-progress.zh-TW.md:7` |
| 1339 | 別名 49014 在 PC 模式寫入 SILENCE64 時送出 | `combat-closeout-current.zh-TW.md:15` |

上表的情境大多出自伺服器隔離候選的紀錄。E149 是否送同樣的 ID，要在各批驗證時核對。

## 其他判定
- **材料 25／58**：伺服器端隔離候選的材料邊界進度是 25／58，其餘 33 個列在伺服器的機器可驗證清單（`native-pair-material-progress.zh-TW.md:13`）。那份清單在未追蹤的建置產物裡，而且 E149 不含這個候選，所以本表不區分 25 與 33，58 個都列伺服器待定。
- **4016（7 列）**：屬於 NPC 用的 SkillMode 邊界，來源啟用者沒有證明（`combat-coverage-current.md:253`）。它不在玩家可施放的 181 列內，客戶端不需處理。
- **武器 Proc**：屬性武器 Proc 及其呈現外控還沒補（`combat-coverage-current.md:251`）。這是物品層級，不是技能 ID，歸 T026 第 2 張的玩法支援矩陣與 T047。
- **卡瑞／莎爾 NPC 與 map1005**：伺服器已接來源 NPC 出生點與原生 C_DIALOG，並唯讀載入卡瑞的 map1005（`combat-coverage-current.md:253`）。判定為 BATTLE 模式中的 NPC 對話，在產品範圍內；它不是玩家技能，所以不在本表，NPC 對話 GUI 歸 T034。
- **技能 68 目標生命週期**：#120 新 N1，已寫在 68 列的備註，實作歸 T061。

## 保留槽（39 個，不是可施放技能）
來源是 `combat-coverage-current.json` 的 `skill_matrix`：classification 為「SQL 保留槽」的 38 個，加上「catalog 明確保留槽，無 SQL 定義」的 1 個（122）。舊文件 `resource-database-0.6.62.md:27` 記的是 34 個，那是較早版本的數字。保留槽不在 181 列的目錄內，客戶端不應送出這些 ID 的施法請求。

24、81、82、83、84、85、86、92、93、94、95、96、119、120、121、122、123、124、125、126、127、128、135、136、139、140、141、142、143、144、177、178、179、180、196、197、198、199、200

## 技能別名（effect_skill_id，7 個）
這些 ID 只出現在 NPC 技能（mobskill），玩家不能施放，不在 181 列內。客戶端只要能正確呈現伺服器送來的效果。其中 49014 在 PC 模式會寫入 SILENCE64 並送訊息 1339，49017 是兩階段麻痺（`combat-closeout-current.zh-TW.md:15`、`:61`）。

| 別名 ID | 實際效果 skill | 名稱 | NPC 使用數 |
|---:|---:|---|---:|
| 49011 | 4011 | 藥水侵蝕術 | 1 |
| 49012 | 4012 | 污濁的水流 | 0 |
| 49013 | 4013 | 治癒侵蝕術 | 2 |
| 49014 | 202 | 群體魔法封印 | 2 |
| 49015 | 71 | 群體藥水霜化術 | 1 |
| 49016 | 167 | 群體風之枷鎖 | 1 |
| 49017 | 33 | 群體木乃伊的詛咒 | 2 |

## 玩法支援矩陣：技能以外的玩家可達操作（T026 第 2 張，#141）
依 G05 的 work：「從正式核心建立所有玩家可達操作的支援矩陣，標明缺失／拒絕／不適用」；acceptance 要求每個必備操作都有封包、權威狀態、呈現與失敗路徑的證據（`docs/G01-G12.md` G05）。

**欄位說明**：
- **客戶端現況**：依 `LiTo381-Modern/src/modern_lito/` 的實作與對應的 `docs/NATIVE-*.zh-TW.md`。程式碼位置以 commit `0949a69` 的行號為準。
- **伺服器（E149）**：只寫有文件或原始碼依據的內容。各 NATIVE 文件當時對照的是較早的正式 JAR（例如 `9C97F868`、`58973343`），**都還沒對 E149 重驗**。
- **空欄的寫法**：沒有證據的欄位寫「待核對：要查什麼／交哪張任務」；客戶端還沒做的寫「尚未實作」；本來就沒有這一項的寫「不適用」。不用破折號。
- **狀態**：支援（客戶端已實作，E149 待重驗）、部分、缺失（客戶端尚未實作）、拒絕（客戶端刻意不送）、待核對、不適用。

| 操作 | 封包 | 伺服器權威 | 呈現 | 失敗路徑 | 客戶端現況 | 狀態 | 後續 |
|---|---|---|---|---|---|---|---|
| 使用物品（一般，use_type 0、1、2、11、18～25） | `C_USE_ITEM 0x1C` `c d(invObjId) s("")`：`use_request`（`native_item_actions.rs:138-143`）；允許的 use_type 見 `default_use_type`（`:96-98`） | 伺服器決定是否接受、效果與數量；客戶端不預扣 | 背包重播與效果封包 | 沒有請求序號，以背包 revision 觀察：兩秒沒更新就是 NoResponse；選取後物品已更新就要求重選（`:31-35`） | 已接 | 支援 | T045（E149 重驗） |
| 使用物品（座標或物件目標，use_type 5，例如 Storm_Walk 42501） | `C_USE_ITEM 0x1C` `c d(item) d(target) h(x) h(y)`，沒有字串結尾：`target_location_request`（`native_item_actions.rs:80-89`）；地面目標時 target＝0 | 伺服器決定落點與傳送。Storm_Walk 走 `S_PORTAL → C_GOTO_PORTAL`，收到確認後才更新座標並重建角色；不消耗數量、次數、HP、MP（`gm-travel-0.6.68.md:23-27`） | 目標游標（`native_login_window.rs:1886-1897`）；換景沿用既有流程 | 客戶端：不在世界、觀戰、死亡、移動受阻、目標離開或移動、位置不可站立、場景或控制狀態改變時拒絕（`native_item_actions.rs:61-71`、`native_login.rs:546-568`）。伺服器：死亡或觀戰不執行；落點無效不替換；麻痺或換景中拒絕；同一傳送未完成不重複啟動（`gm-travel-0.6.68.md:23-26`） | 已接（`native_login.rs:546-568` 建包並送出） | 支援 | T045（E149 實機待驗） |
| 使用物品（其他需要目標或選項的分支，含 type3 charge） | 待核對：各 use_type 的尾段格式，交 T045 | 待核對（T045） | 尚未實作 | 目前客戶端不送：`default_use_type` 以外、又不是 5 的 use_type 不會發包 | 尚未實作 | 拒絕 | T045 |
| 裝備與卸下 | 同一般使用的 `C_USE_ITEM` | 0.6.65 起，完成評估後才送一份最終描述與武器外型（`equipment-toggle-0.6.65.md:17`、`:24`） | 背包與外型更新 | 被拒絕的候選狀態不送客戶端（同上 `:17`） | 已接 | 支援 | T045、T052（裝備 UI） |
| 丟棄（指定數量，腳下） | `C0C` `c h(x) h(y) d(item) d(count)`：`drop_request`（`native_item_actions.rs:145` 起） | 伺服器決定 | 背包與地面物件更新 | 數量為 0 或超過持有量時客戶端不送（`:130`） | 已接 | 支援 | T045 |
| 銷毀 | `C76`：待核對格式（T045） | 待核對（T045） | 尚未實作 | 待核對（T045） | 尚未實作（`src/modern_lito` 沒有 C76 的建構） | 缺失 | T045 |
| 地面拾取 | `C0B`（`native_ground_pickup.rs:98`），末 byte 送 0 | 伺服器決定；客戶端不先加背包，也不先刪地面物件 | 地面物件與背包更新 | 兩秒沒有足夠更新就判為 NoResponse（NATIVE-GROUND-PICKUP） | 已接，限 3 格 Chebyshev 距離 | 支援 | T052（直接滑鼠拾取）；末 byte 的 remembered-ID 已排除 |
| 藥水 189／194／197 | 一般 `C_USE_ITEM` | 伺服器決定 | 189 已接正式流程；194／197 只驗了資源生命週期（NATIVE-ITEM-ACTIONS:39） | 待核對：194／197 在正式流程的拒絕情境（T045） | 部分 | 部分 | T045 |
| 物品目標技能（12、21、73、100、107） | `C_USE_SPELL` 加物品 objid（PROTOCOL-CONTRACT §6.3） | 伺服器決定；技能 12、107 依已裝備武器決定目標（`native-item-weapon-material-progress.zh-TW.md:5`，隔離候選） | 待核對：特效與背包更新（各技能批次） | 未裝備時伺服器保留錯誤封包（同上） | 尚未實作（候選或未接批次） | 缺失 | T027、T022、T045 |
| 近身攻擊 | `C_ATTACK 0x17` `c d(target≠0) h h`（`native_combat.rs`） | 伺服器決定命中與傷害 | `S_ATTACK`、HP 比例 | 不合法或死亡目標時客戶端不送（`native_world.rs:343` 前後） | 已接（NATIVE-COMBAT） | 支援 | T029、T043（雙真實玩家重驗） |
| 空揮近戰 | `C_ATTACK` target＝0（`17 00 00 00 00 DE 7F 1D 80`，`empty-melee-0.6.49.json` L34） | 0.6.49 起伺服器接受 target 0（同上 L35） | 待核對：空揮動作的呈現（T029） | 伺服器：死亡、觀戰、隱身、麻痺、超重、座標不合法時拒絕（同上 L37–42） | 待核對：客戶端是否會送 target 0（T029） | 待核對 | T029 |
| 遠程攻擊：姿態選擇與普通箭 | `C_FAR_ATTACK 0x18` `18 d h h`；回覆 `S_ATTACK` 29 bytes。stance 20／62 走遠程請求（`native_world.rs:345`） | 伺服器決定命中；target 非零代表命中 | 投射物 cue | 截斷或錯誤組合不能改動 World（NATIVE-RANGED） | 已接（NATIVE-RANGED:3） | 支援 | T029（E149 重驗） |
| 遠程攻擊：特殊箭、投擲的 projectile／effect discriminator | 同上，其他 discriminator | 待核對（T029） | 尚未實作 | 客戶端明確保留為未支援，不部分套用（NATIVE-RANGED） | 尚未實作 | 缺失 | T029 |
| 彈藥 | 伺服器扣箭；三重矢每次施法扣 3 支，之後的一般攻擊再扣 1 支（`native-triple-arrow-policy-progress.zh-TW.md:7`，隔離候選） | 伺服器決定 | 背包數量（已接）；彈藥 UI 尚未實作 | 待核對：沒有箭時的拒絕訊息（T029） | 部分 | 部分 | T052（彈藥 UI）、T045（三重矢） |
| 武器 Proc（屬性武器） | 不適用（沒有獨立請求；隨 `C_ATTACK` 觸發） | 伺服器權威：Proc 機率與傷害由伺服器決定；屬性武器 Proc 和它的呈現外控還沒補（`combat-coverage-current.md:251`） | 待核對：Proc 特效與傷害數字的封包（T047） | 待核對：沒觸發時不顯示，不由客戶端模擬（T047） | 尚未實作：不會由客戶端預測或模擬 Proc | 待核對 | T047 |
| 材料與耗材（技能） | `C_USE_SPELL` | 伺服器決定材料與 MP；缺料回訊息 299 | 背包與 MP 更新 | 客戶端不預扣；一次一個待回覆，兩秒沒回覆可手動重試（NATIVE-REAGENT-SKILLS） | 技能 63 已接；其餘 57 個材料技能見上表 | 部分 | 上表各技能；58 個都列伺服器待定 |
| MP／HP 消耗呈現 | `S_MANA_POINT 0x4D`、`S_HIT_POINT 0x0D`（schema 0／≥1，PROTOCOL-CONTRACT §6.2） | 伺服器權威 | HUD | MP 不足回 278（自施）、HP 不足回 279（見上「表外的訊息 ID」） | 已接數值更新；278／279 的顯示待核對 | 部分 | T046 |
| buff 狀態 | 能力 buff 封包（`native_ability_buffs.rs`；`native_world.rs:550-552` 處理到期移除） | 伺服器權威 lease：刷新、到期、取消、換場清除 | 圖示與剩餘時間 | 不屬於自己的更新計入 orphan，不套用（`native_world.rs:553`） | 能力 buff 26／42 已接，有白名單測試 | 部分 | T046 |
| debuff 狀態 | `S32` 中毒、`S34` 隱身（`native_world.rs:889`）、`S_PARALYSE`、S0B 狀態位 | 伺服器權威 | 綠色、暗色、半透明繪製（NATIVE-ACTOR-STATES） | 新身分覆寫、移除、換世界時清除舊狀態 | 已接 S0B、S32、S34 | 部分 | T046、T047 |
| 死亡確認 | `S_LITO_CONFIRM_DEATH 0xE3` → `C_LITO_CONFIRM_DEATH 0xE0` | 只在 BATTLE、有待確認的死亡、不是觀戰者且 hp≤0 時接受（PROTOCOL-CONTRACT §6.1） | 倒地動畫播完才送 | 重複呼叫不會再排入（NATIVE-OUTCOMES） | 已接 | 支援 | T043（雙真實玩家重驗） |
| 復活（技能 61、75 的同意問答） | 伺服器送 question 321／322，客戶端回答 | 伺服器決定；要玩家同意 | 尚未實作：原生問答視窗 | 待核對：拒絕、放棄、逾時的封包（T027、T048） | 尚未實作（候選獨有） | 缺失 | T027、T048 |
| 復活：S_RESURRECT 接收與生命週期 | `S_RESURRECT 0x11` `c d(target) c(action) d(source) h(graphics)`，檢查完整長度（`r.end()`）與非零身分（`native_world.rs:567-570`） | 伺服器權威：客戶端只在收到封包後套用；HP 與比例等各自的伺服器封包，不推測（`:573` 註解） | 換新的 incarnation，清 target、動作、移動 revision，解除死亡與暗色繪製，換 graphics 與 walk_action，移除相關投射物；目標是自己時清死亡確認待決（`native_world.rs:571-591`）。登入層中止舊攻擊與技能、門的待決，並清掉 C_CONFIRM_DEATH 的等待 deadline，註解寫明是為了 honor player 不被斷線（`native_login.rs:1632-1649`） | 長度不符或身分為 0：整包拒絕；目標不在世界：計入 orphan，不建立角色（`native_world.rs:569-570`、`:592-593`）。測試：`tests/native_lifecycle.rs` 三個（含 malformed／orphan），在白名單外，**未執行** | 已接（從 `0949a69` 起就有）；E149 與 GUI 待驗 | 部分 | T048（E149 重驗、白名單）、T043 |
| 復活：榮譽模式延遲重生的完整流程 | 待核對：延遲期間與重生時伺服器送出的封包序列（T048） | 待核對：重生時機、位置與 HP 的權威來源（T048） | 待核對：延遲期間的畫面（倒地、倒數、觀戰）（T048） | 待核對：延遲中離開、換場、逾時（T048） | 只有上一列的 S_RESURRECT 接收；NATIVE-OUTCOMES:13 寫「榮譽模式延遲重生、S_RESURRECT 仍未宣告支援」，這段文件早於上一列的實作 | 待核對 | T048、T043 |
| 隱身裝備例外（#120 新 N2） | 不適用（不是獨立封包，是隱身重施的判定） | 隔離候選 `NativeInvisibilityCastPolicy.java:9`：重施隱身時，指定隱身裝備 20077／120077／20062 有 toggle 例外；斗篷 3 件還不在 native 物品目錄（`native-invisibility-cast-policy-progress.zh-TW.md:6`）。E149 是否承襲，待 T025 換版重核確認 | 待核對：重施時是否顯形（T047） | 待核對（T025、T047） | 尚未實作 | 待核對 | T026（本表）、T045、T047 |
| 商人附加屬性（G05.4） | 不適用（沒有客戶端請求）；屬性由伺服器的狀態封包帶來，例如 S52 抗性（`native_world.rs:555` 處理 0x50／0x51／0x52／0x56） | 伺服器權威：租用裝備的附加屬性由伺服器 `RentalEquipmentBonuses` 計算。E149 查不到 78 個商人商品的來源，例如 29047 火蜥蜴紋樣戒指的火抗正式值是 0、應為 10（`修補提案/商人屬性來源/README.md:10`；G05.4 state `current_C02_detached_reference_gap_confirmed_client_authority_handling_and_real_modes_pending`） | **依伺服器權威值顯示**：客戶端不自行補算、不改寫伺服器送來的數值 | 伺服器缺陷（漏算屬性）不算客戶端失敗：GUI 驗收寫「與伺服器送出的一致」；修正由伺服器處理（T040） | 已接屬性更新封包；完整屬性 GUI 待做（G08.3） | 部分 | T049（屬性顯示）、T040（伺服器移交）、T034（驗收寫法） |
| 擂台 NPC 商店 | 待核對：買賣封包格式（T045） | 16 列買價與賣價（`data-sources-0.6.60.json` L402–439、L672–705） | 尚未實作：商店 UI | 待核對：金錢不足、背包滿、重量超過時的拒絕（T045） | 尚未實作 | 缺失 | T045、T034 |
| 寵物與召喚物控制 | 不適用（不在本張） | 見 T063 | 見 T063 | 見 T063 | 見 T063 | 不適用 | T063 |
| 玩家之間的交易 | 不適用 | G01–G12 沒有這一項 | 不適用 | 不適用 | 不適用 | 不適用 | 排除（TASKS「排除或延後」） |

**統計**：28 列。支援 8、部分 8、缺失 5、拒絕 1、待核對 4、不適用 2。「支援」的列都只有客戶端實作與舊版 JAR 的隔離驗證，還沒對 E149 重驗。

## T027 批 2 回填（#146）

只回填 29、30、33、36、37、39、40、41；離線送收包與原資源測試通過不等於正式服務或 GUI 驗收。原伺服器待定分類及 #140／#141 盤點基線保留。41 使用明確屍體意圖，39 未選目標時允許自施；八個技能的成本、材料與成功結果仍由伺服器決定。詳見 [Issue #146 報告](reports/issue-146.md)。

## T027 批 3 回填（#147）

只回填 44–48、50、51、53。送收包及原資源記憶體繪製測試通過；未連正式服務或執行 GUI。44 依正式處理器歷史核對採三格（SQL 射程欄仍保留原值）；51 的選項不與 actor ID 混用，施法動作之後仍等待晚到拒絕。45／46／50 尚未核定原生相對 graphics 的 hurt guard，僅保留事件診斷，不以通用受擊動畫替代。正式分類與伺服器待定項目保留，詳見 [Issue #147 報告](reports/issue-147.md)。
