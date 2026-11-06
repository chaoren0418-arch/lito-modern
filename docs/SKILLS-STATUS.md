# SKILLS-STATUS — E149 技能狀態表（T026 第 1 張，#140）

更新：2026-11-06。這份表是 M2 完成定義的第一項：「打開 `docs/SKILLS-STATUS.md`，看到 E149 目錄每個技能一列……凡是標『以 E149 為準可驗』的列，全部顯示通過」。目前各技能大多還沒實作或驗證，「測試」欄由 M2 各批任務完成後更新。

## 來源與方法
- **技能目錄**：`docs/PROTOCOL-CONTRACT.md` §6.3，共 181 項，也就是 180 個 catalog 記錄加上 TELEPORT。欄位包括 ID、常數、記錄類型、尾段形狀，讀的是 `src/main @2066774`。E149 與它的 catalog 常數 223／223 相同（§0、§8）。
- **名稱、職業、SQL 欄位**：lito-launcher-server `2066774` 的 `lito伺服器/LiTo381-Server/docs/maintenance/combat-coverage-current.json`（SHA256 `2baa7f083cb656ef736ff23b4920247b3c0f843226193a148cf036948c6c271e`）。用到的 `skill_matrix` 欄位有 name、class_by_sql_level、sql.target、target_to、ranged、area、mpConsume、hpConsume、itemConsumeId／Count、sysmsgID_happen、sysmsgID_fail。這份是伺服器端隔離候選 `efd4bc36` 的覆蓋紀錄，SQL 欄位來自共用資料庫。它的 808 列裡「已定義」180 列加上獨立處理器 1 列（skill 5），正好對應本表 181 列。
- **客戶端狀態**：main 已接的 38 個以 `LiTo381-Modern/src/modern_lito/native_skills.rs` 的 `request()` 與 `ground_request()` 為準（#92 證據 E3）。其餘依 TASKS.md `## M2` 的 T020、T021、T022、T027 分批。
- **測試**：掃描 `LiTo381-Modern/src`、`tests` 底下所有 `#[test]` 函式，找呼叫 `native_skills` 的 `request`／`ground_request`／`self_tick_request` 的測試，並取出它測的技能 ID。傳 ID 的四種方式都納入：
  1. 字面：`request(&w, 26, …)`。
  2. 迴圈：`for X in [26,42]` 或 `for (X, …) in [(26,…),…]`，而呼叫的參數是 X。
  3. 欄位：參數是 `c.skill` 這類欄位時，取同一函式內的 `skill: N`。
  4. 常數：參數是大寫常數時，取同檔的 `const NAME: … = N;`。

  只收目錄 181 個 ID 內的數字。執行狀態判定：沒有 `#[ignore]` 的 lib 測試算 #101 已執行通過；整合測試在 `維護工具/test-whitelist/tests.tsv` 白名單內，算 #101 已執行通過（76／76）；其餘算未執行。經過多層函式才傳到 request 的 ID 仍可能漏抓。
- **是否需要 Ctrl**：依 E149 的 0.6.67 強制目標行為與 TASKS T061 已寫出的政策逐列填寫：
  - 尾段形狀不含 `d(target)`：不適用。
  - SQL target 是 attack：依目標而定，對敵方不需要，對同隊需要 Ctrl。
  - 其他物件目標：不需要（自身／友方輔助）。
  - 26、42、68 另有說明。

  伺服器隔離候選裡更嚴格的「敵方玩家需要明確意圖」政策沒有部署在 E149，所以不列入本欄。

## 分類規則
落在下列任一伺服器缺口組的技能列為**伺服器待定**。這類技能只驗送包形狀與畫面，不驗與原版同源，依 TASKS T026。其餘列為**以 E149 為準可驗**。
- **材料**：`itemConsumeId ≠ 0`，共 58 個，對應伺服器 TASKS T025「58 列材料技能」。
- **受控 NPC**：記錄類型是 ElementalSummon、Taming、CreateZombie、Summon、ReturnToNature 之一，對應伺服器 T023。
- **立方體**：Cube* 共 4 個，屬 HP 生產端，對應伺服器 T024。
- **自施政策**：26、42，對應伺服器 T022；伺服器記錄「26 不按 Ctrl 無法自施」還沒修（`source-completion-work-current.zh-TW.md:41`）。
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

## 統計
- 以 E149 為準可驗：**114**；伺服器待定：**67**；合計 181。
- 伺服器待定的原因分布（一個技能可以有多個原因）：材料 58、受控 6、處理器待查 6、立方體 4、自施政策 2、伺服器停用 1、暈眩 1。
- 客戶端狀態：main 已接 38、候選獨有 48＋97／98、97–100 中未接的 99／100、未接 91（#92 E3）。
- 有送包測試的技能：38 個；其中**已執行通過**的有 11 個：14、26、31、32、42、49、52、54、72、78、206。來源是白名單測試 `tests/native_selector_only.rs` 的 `nine_server_selectors_ignore_hover_and_never_spend_local_authority`，以及 `tests/native_ability_buffs.rs` 的兩個測試，#101 中 76／76 通過。其餘技能的測試都在白名單外、還沒執行。
- 「資源／特效核對」欄目前全部是「待核對」，由 M2 各批回填。

## 表
目標欄是 SQL 的 target／target_to／ranged／area；訊息欄是 happen／fail 的訊息 ID。

| ID | 名稱 | 常數 | 職業 | 記錄類型 | 尾段形狀 | 目標 | 是否需要 Ctrl（E149） | 成本 | 訊息 | 客戶端 | 送包測試（狀態） | 資源／特效核對 | 分類 | 備註 |
|---:|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 初級治癒術 | `BASIC_HEAL` | 共通魔法／法師 | HealingSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 4 | 77／280 | main 已接 | `destroyed_guardian_retires_attacks_spells_and_selection_even_if_repaired_between_polls`（未執行（白名單外））；`spell_requests_use_formal_selectors_and_never_predict_mana`（未執行（白名單外））；`tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 2 | 日光術 | `LIGHT` | 共通魔法／法師 | LightSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 4 | 0／0 | main 已接 | `light_uses_self_selector_without_predicting_authority_and_scopes_UI_choice`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 3 | 保護罩 | `SHIELD` | 共通魔法／法師 | ShieldSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 8 | 0／0 | main 已接 | `spell_requests_use_formal_selectors_and_never_predict_mana`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 | BUFF selector 有語意衝突，伺服器已停用 SQL 覆寫、保留既有行為，以 E149 既有行為驗 |
| 4 | 光箭 | `LIGHT_ARROW` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 3 | 0／0 | main 已接 | `hidden_targets_are_rejected_until_visible_and_identity_replacement_invalidates_selection`（未執行（白名單外））；`formal_attack_spell_layout_and_request_guards`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 5 | 指定傳送 | `TELEPORT` | 共通魔法／法師 | （dispatcher 特例） | ⑤ h(map) h(x) h(y) | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 候選獨有（T027 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 6 | 冰箭 | `ICE_ARROW` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 4 | 0／0 | main 已接 | `formal_attack_spell_layout_and_request_guards`（未執行（白名單外））；`destroyed_guardian_retires_attacks_spells_and_selection_even_if_repaired_between_polls`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 7 | 風刃 | `WIND_CUTTER` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 4 | 0／0 | main 已接 | `formal_attack_spell_layout_and_request_guards`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 8 | 神聖武器 | `HOLY_WEAPON` | 共通魔法／法師 | WeaponBuffSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 10 | 0／0 | main 已接 | `object_selector_screen_and_owner_guards_preserve_authority`（未執行（白名單外））；`spell_requests_use_formal_selectors_and_never_predict_mana`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 9 | 解毒術 | `CURE_POISON` | 共通魔法／法師 | CurseRemovalSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 211／280 | main 已接 | `object_and_coordinate_envelopes_preserve_authority_and_gate_live_targets`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 10 | 寒冷戰慄 | `CHILL_TOUCH` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 8 | 0／0 | main 已接 | `request_layout_guards_and_server_authority_do_not_predict_healing_or_hurt`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 11 | 毒咒 | `CURSE_POISON` | 共通魔法／法師 | PoisonSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 1／範圍 0 | 不需要（自身／友方輔助） | MP 10 | 0／280 | main 已接 | `object_and_coordinate_envelopes_preserve_authority_and_gate_live_targets`（未執行（白名單外）） | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 12 | 擬似魔法武器 | `ENCHANT_WEAPON` | 共通魔法／法師 | ItemWeaponEnchantSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 0／0 | 候選獨有（T027 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 13 | 無所遁形術 | `DETECTION` | 共通魔法／法師 | DetectionSpell | ① 無尾段 | none／3／射程 0／範圍 -1 | 不適用（非物件目標） | MP 15 | 0／0 | main 已接 | `detection_selector_menu_guards_and_cooldown_leave_authority_unchanged`（未執行（白名單外））；`invisibility_tick_envelope_guards_clock_and_menu_preserve_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 14 | 負重強化 | `DECREASE_WEIGHT` | 共通魔法／法師 | WeightSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10、HP 50 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 15 | 火箭 | `FIRE_ARROW` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 6 | 0／0 | main 已接 | `formal_attack_spell_layout_and_request_guards`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 16 | 地獄之牙 | `STALAC` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 6 | 0／0 | main 已接 | `stalac_packet_range_clock_and_no_predicted_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 17 | 極光雷電 | `LIGHTNING` | 共通魔法／法師 | LineAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | main 已接 | `request_clock_reuse_range_and_target_guard_preserve_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 18 | 起死回生術 | `TURN_UNDEAD` | 共通魔法／法師 | TurnUndeadSpell | ⑩ d(target) h(x) h(y) | attack／2／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／280 | main 已接 | `low_mana_turn_undead_times_out_without_predicted_mana_death_or_rejection`（未執行（白名單外））；`selector_clock_range_and_dead_target_guards_never_predict_death_or_mana`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 19 | 中級治癒術 | `EXTRA_HEAL` | 共通魔法／法師 | HealingSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 77／280 | main 已接 | `tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 20 | 闇盲咒術 | `CURSE_BLIND` | 共通魔法／法師 | BlindSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 0／280 | 候選獨有（T027 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 21 | 鎧甲護持 | `BLESSED_ARMOR` | 共通魔法／法師 | ItemArmorEnchantSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 0／0 | 候選獨有（T027 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 22 | 寒冰氣息 | `FROZEN_CLOUD` | 共通魔法／法師 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | 候選獨有（T027 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 23 | 能量感測 | `REVEAL_WEAKNESS` | 共通魔法／法師 | RevealWeaknessSpell | ④ d(target) | buff／2／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 8 | 0／280 | 候選獨有（T027 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 25 | 燃燒的火球 | `BURNING_FIREBALL` | 共通魔法／法師 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | 候選獨有（T027 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 26 | 通暢氣脈術 | `PHYSICAL_ENCHANT_DEX` | 共通魔法／法師 | AbilityBuffSpell | ④ d(target) | buff／4／射程 -1／範圍 0 | 不需要（自身用自身 object ID 走 7-byte 分派；伺服器缺口：E149 不按 Ctrl 無法自施，見分類） | MP 50 | 294／280 | main 已接 | `ability_buffs_keep_native_screen_asymmetric_edges`（已執行通過（白名單，#101））；`ability_buffs_reject_unlearned_dead_missing_and_outside_screen_targets`（已執行通過（白名單，#101））；`both_ability_buffs_use_explicit_self_or_selected_object_without_authority_writes`（已執行通過（白名單，#101）） | 待核對 | 伺服器待定：自施政策（伺服器 T022） |  |
| 27 | 壞物術 | `WEAPON_BREAK` | 共通魔法／法師 | WeaponBreakSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 4／範圍 0 | 不需要（自身／友方輔助） | MP 32 | 0／280 | 候選獨有（T027 批 1） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 28 | 吸血鬼之吻 | `VAMPIRIC_TOUCH` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 4／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 12 | 0／0 | main 已接 | `vampiric_envelope_range_and_authority_do_not_predict_healing_or_hurt`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 29 | 緩速術 | `SLOW` | 共通魔法／法師 | EntangleSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 277／280 | 候選獨有（T027 批 2） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 30 | 巖牢 | `EARTH_JAIL` | 共通魔法／法師 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 8／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 24 | 0／0 | 候選獨有（T027 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 31 | 魔法屏障 | `COUNTER_MAGIC` | 共通魔法／法師 | CounterMagicSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 32 | 冥想術 | `MEDITATION` | 共通魔法／法師 | RegenSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 5、HP 30 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 33 | 木乃伊的詛咒 | `CURSE_PARALYZE` | 共通魔法／法師 | ParalysisSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 2／範圍 0 | 不需要（自身／友方輔助） | MP 150 | 0／280 | 候選獨有（T027 批 2） | 尚無 | 待核對 | 伺服器待定：材料 40318×2（伺服器 T025） |  |
| 34 | 極道落雷 | `CALL_LIGHTNING` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 18 | 0／0 | main 已接 | `current_c02_exact_object_coordinates_and_readonly_authority`（未執行（白名單外））；`invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests`（未執行（白名單外））；`mp_and_lawful_are_server_permission_inputs_and_never_locally_spent`（未執行（白名單外））；`range_six_boundary_and_target_changes_use_current_received_coordinates`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 35 | 高級治癒術 | `GREATER_HEAL` | 共通魔法／法師 | HealingSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 77／280 | main 已接 | `tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 36 | 迷魅術 | `TAMING_MONSTER` | 共通魔法／法師 | TamingSpell | ⑩ d(target) h(x) h(y) | buff／2／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 30、HP 20 | 0／280 | 候選獨有（T027 批 2） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 37 | 聖潔之光 | `REMOVE_CURSE` | 共通魔法／法師 | CurseRemovalSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 155／280 | 候選獨有（T027 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 38 | 冰錐 | `CONE_OF_COLD` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 18 | 0／0 | main 已接 | `cold_request_uses_observed_C14_and_six_cell_range_without_spending_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 39 | 魔力奪取 | `MANA_DRAIN` | 共通魔法／法師 | ManaDrainSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 4／範圍 0 | 不需要（自身／友方輔助） | MP 1、HP 50 | 690／280 | 候選獨有（T027 批 2） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 40 | 黑闇之影 | `DARKNESS` | 共通魔法／法師 | BlindSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 0／280 | 候選獨有（T027 批 2） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 41 | 造屍術 | `CREATE_ZOMBIE` | 共通魔法／法師 | CreateZombieSpell | ④ d(target) | buff／2／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 5、HP 80 | 0／280 | 候選獨有（T027 批 2） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 42 | 體魄強健術 | `PHYSICAL_ENCHANT_STR` | 共通魔法／法師 | AbilityBuffSpell | ④ d(target) | buff／4／射程 -1／範圍 0 | 不需要（自身用自身 object ID 走 7-byte 分派；伺服器缺口：E149 不按 Ctrl 無法自施，見分類） | MP 50 | 292／280 | main 已接 | `ability_buffs_keep_native_screen_asymmetric_edges`（已執行通過（白名單，#101））；`ability_buffs_reject_unlearned_dead_missing_and_outside_screen_targets`（已執行通過（白名單，#101））；`both_ability_buffs_use_explicit_self_or_selected_object_without_authority_writes`（已執行通過（白名單，#101）） | 待核對 | 伺服器待定：自施政策（伺服器 T022） |  |
| 43 | 加速術 | `HASTE` | 共通魔法／法師 | HasteSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 40 | 184／280 | main 已接 | `spell_requests_use_formal_selectors_and_never_predict_mana`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 44 | 魔法相消術 | `CANCELLATION` | 共通魔法／法師 | CancellationSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 40 | 0／280 | 候選獨有（T027 批 3） | 尚無 | 待核對 | 伺服器待定：材料 40318×2（伺服器 T025） |  |
| 45 | 地裂術 | `ERUPTION` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 8／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | 候選獨有（T027 批 3） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 46 | 烈炎術 | `SUNBURST` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 20 | 0／0 | 候選獨有（T027 批 3） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 47 | 弱化術 | `WEAKNESS` | 共通魔法／法師 | WeaknessSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 10 | 692／280 | 候選獨有（T027 批 3） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 48 | 祝福魔法武器 | `BLESS_WEAPON` | 共通魔法／法師 | WeaponBuffSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 0／0 | 候選獨有（T027 批 3） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 49 | 體力回復術 | `HEAL_ALL` | 共通魔法／法師 | GroupHealingSpell | ① 無尾段 | none／20／射程 0／範圍 -1 | 不適用（非物件目標） | MP 50、HP 30 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 50 | 冰矛圍籬 | `ICE_LANCE` | 共通魔法／法師 | FreezeAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 6／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 100 | 0／0 | 候選獨有（T027 批 3） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 51 | 召喚術 | `SUMMON_MONSTER` | 共通魔法／法師 | SummonSpell | ⑨ d(selection) | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30、HP 30 | 0／0 | 候選獨有（T027 批 3） | 尚無 | 待核對 | 伺服器待定：材料 40318×3（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 52 | 神聖疾走 | `HOLY_WALK` | 共通魔法／法師 | MovementBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 15 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 53 | 龍捲風 | `TORNADO` | 共通魔法／法師 | AreaAttackSpell | ① 無尾段 | none／3／射程 0／範圍 4 | 不適用（非物件目標） | MP 40 | 0／0 | 候選獨有（T027 批 3） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 54 | 強力加速術 | `GREATER_HASTE` | 共通魔法／法師 | SelfHasteSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 60 | 184／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 55 | 狂暴術 | `BERSERKERS` | 共通魔法／法師 | CombatBuffSpell | ④ d(target) | buff／12／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 40 | 0／280 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 56 | 疾病術 | `DISEASE` | 共通魔法／法師 | WeaknessSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 10 | 0／280 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 57 | 全部治癒術 | `FULL_HEAL` | 共通魔法／法師 | HealingSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 48 | 77／280 | main 已接 | `tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 58 | 火牢 | `FIRE_WALL` | 共通魔法／法師 | GroundSpell | ③ h(x) h(y) | attack／3／射程 8／範圍 0 | 不適用（非物件目標） | MP 60 | 0／0 | main 已接 | `formal_ground_request_unsigned_boundaries_range_and_authority`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
| 59 | 冰雪暴 | `BLIZZARD` | 共通魔法／法師 | AreaAttackSpell | ① 無尾段 | none／3／射程 0／範圍 10 | 不適用（非物件目標） | MP 60、HP 10 | 0／0 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 60 | 隱身術 | `INVISIBILITY` | 共通魔法／法師 | InvisibilitySpell | ② d(clientTick) | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 15 | 0／0 | main 已接 | `invisibility_tick_envelope_guards_clock_and_menu_preserve_authority`（未執行（白名單外）） | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 61 | 返生術 | `RESURRECTION` | 共通魔法／法師 | ResurrectionSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 50 | 0／280 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 62 | 震裂術 | `EARTHQUAKE` | 共通魔法／法師 | AreaAttackSpell | ① 無尾段 | none／3／射程 0／範圍 3 | 不適用（非物件目標） | MP 40 | 0／0 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 63 | 治癒能量風暴 | `LIFE_STREAM` | 共通魔法／法師 | GroundSpell | ③ h(x) h(y) | buff／32／射程 -1／範圍 3 | 不適用（非物件目標） | MP 20 | 0／0 | main 已接 | `formal_ground_request_unsigned_boundaries_range_and_authority`（未執行（白名單外））；`formal_reagent_request_self_screen_boundary_and_no_prediction`（未執行（白名單外）） | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 64 | 魔法封印 | `SILENCE` | 共通魔法／法師 | SilenceSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 50 | 0／280 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 65 | 雷霆風暴 | `LIGHTNING_STORM` | 共通魔法／法師 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 48 | 0／0 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 66 | 沉睡之霧 | `FOG_OF_SLEEPING` | 共通魔法／法師 | SleepSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 10／範圍 3 | 不需要（自身／友方輔助） | MP 50 | 0／280 | 候選獨有（T027 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 67 | 變形術 | `SHAPE_CHANGE` | 共通魔法／法師 | PolymorphSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 80 | 0／280 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 68 | 聖結界 | `IMMUNE_TO_HARM` | 共通魔法／法師 | DamageProtectionSpell | ④ d(target) | buff／4／射程 -1／範圍 0 | 不需要（自身／血盟；Ctrl 成功封包同普通施放，不能繞過血盟限制） | MP 30 | 314／280 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×2（伺服器 T025） | 目標生命週期：扣料或安裝前目標退役、receiver 換世代時要拒絕或丟棄（#120 新 N1，T061） |
| 69 | 集體傳送術 | `MASS_TELEPORT` | 共通魔法／法師 | MassTeleportSpell | ⑥ h(map) h(x) h(y) | none／4／射程 0／範圍 2 | 不適用（非物件目標） | MP 60 | 0／0 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 70 | 火風暴 | `FIRE_STORM` | 共通魔法／法師 | AreaAttackSpell | ① 無尾段 | none／3／射程 0／範圍 3 | 不適用（非物件目標） | MP 48 | 0／0 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 71 | 藥水霜化術 | `DECAY_POTION` | 共通魔法／法師 | DecayPotionSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 80 | 0／280 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×2（伺服器 T025） |  |
| 72 | 強力無所遁形術 | `COUNTER_DETECTION` | 共通魔法／法師 | CounterDetectionSpell | ① 無尾段 | none／3／射程 0／範圍 -1 | 不適用（非物件目標） | MP 30 | 0／0 | main 已接 | `nine_server_selectors_ignore_hover_and_never_spend_local_authority`（已執行通過（白名單，#101））；`selector_learning_death_and_mana_guards_do_not_forge_requests`（已執行通過（白名單，#101）） | 待核對 | 以 E149 為準可驗 |  |
| 73 | 創造魔法武器 | `CREATE_MAGICAL_WEAPON` | 共通魔法／法師 | MagicalWeaponSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助） | MP 200 | 685／0 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×10（伺服器 T025） |  |
| 74 | 流星雨 | `METEOR_STRIKE` | 共通魔法／法師 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 60 | 0／0 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 75 | 終極返生術 | `GREATER_RESURRECTION` | 共通魔法／法師 | ResurrectionSpell | ④ d(target) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 100 | 0／280 | 候選獨有（T027 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40318×1（伺服器 T025） |  |
| 76 | 集體緩速術 | `MASS_SLOW` | 共通魔法／法師 | MassSlowSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 3 | 不需要（自身／友方輔助） | MP 20 | 0／280 | 候選獨有（T027 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40318×2（伺服器 T025） |  |
| 77 | 究極光裂術 | `DISINTEGRATE` | 共通魔法／法師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 70 | 0／0 | main 已接 | `current_c02_exact_object_coordinates_and_readonly_authority`（未執行（白名單外））；`invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests`（未執行（白名單外））；`mp_and_lawful_are_server_permission_inputs_and_never_locally_spent`（未執行（白名單外））；`range_ten_boundary_and_target_changes_use_current_received_coordinates`（未執行（白名單外）） | 待核對 | 以 E149 為準可驗 |  |
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
| 100 | 提煉魔石 | `BRING_STONE` | 黑暗妖精 | BringStoneSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助） | MP 5 | 0／0 | 未接（T021） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 101 | 行走加速 | `MOVING_ACCELERATION` | 黑暗妖精 | MovementBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 102 | 燃燒鬥志 | `BURNING_SPIRIT` | 黑暗妖精 | PhysicalProcBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 103 | 暗黑盲咒 | `DARK_BLIND` | 黑暗妖精 | BlindSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 0／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 147／280 | 未接（T022 批 1） | 尚無 | 待核對 | 伺服器待定：材料 40321×1（伺服器 T025） |  |
| 104 | 毒性抵抗 | `VENOM_RESIST` | 黑暗妖精 | PoisonResistanceSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 105 | 雙重破壞 | `DOUBLE_BRAKE` | 黑暗妖精 | PhysicalProcBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 823／0 | 未接（T022 批 1） | 尚無 | 待核對 | 伺服器待定：材料 40321×1（伺服器 T025） |  |
| 106 | 暗影閃避 | `UNCANNY_DODGE` | 黑暗妖精 | DodgeBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 107 | 暗影之牙 | `SHADOW_FANG` | 黑暗妖精 | ItemWeaponEnchantSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助） | MP 30 | 0／0 | 未接（T022 批 1） | 尚無 | 待核對 | 伺服器待定：材料 40321×1（伺服器 T025） |  |
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
| 133 | 弱化屬性 | `ELEMENTAL_FALL_DOWN` | 妖精 | ElementalFallDownSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 10 | 0／280 | 未接（T022 批 3） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 134 | 鏡反射 | `COUNTER_MIRROR` | 妖精 | CounterMirrorSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／280 | 未接（T022 批 3） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 137 | 淨化精神 | `CLEAR_MIND` | 妖精 | ClearMindSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 未接（T022 批 3） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 138 | 屬性防禦 | `RESIST_ELEMENTAL` | 妖精 | ResistElementalSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 未接（T022 批 3） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 145 | 釋放元素 | `RETURN_TO_NATURE` | 妖精 | ReturnToNatureSpell | ⑩ d(target) h(x) h(y) | buff／16／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 30 | 0／280 | 未接（T022 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40319×2（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 146 | 魂體轉換 | `BLOODY_SOUL` | 妖精 | BodyToMindSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 40 | 0／0 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 147 | 單屬性防禦 | `ELEMENTAL_PROTECTION` | 妖精 | ElementalProtectionSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 6 | 0／0 | 未接（T022 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 148 | 火焰武器 | `FIRE_WEAPON` | 妖精 | FireWeaponSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 0／280 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 149 | 風之神射 | `WIND_SHOT` | 妖精 | WindShotSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 0／280 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 150 | 風之疾走 | `WIND_WALK` | 妖精 | MovementBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 15 | 0／0 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 151 | 大地防護 | `EARTH_SKIN` | 妖精 | ShieldSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 15 | 0／280 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 | BUFF selector 有語意衝突，伺服器已停用 SQL 覆寫、保留既有行為，以 E149 既有行為驗 |
| 152 | 地面障礙 | `ENTANGLE` | 妖精 | EntangleSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 10／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 711／280 | 未接（T022 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 153 | 魔法消除 | `ERASE_MAGIC` | 妖精 | EraseMagicSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 10 | 0／280 | 未接（T022 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 154 | 召喚屬性精靈 | `LESSER_ELEMENTAL` | 妖精 | ElementalSummonSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 4） | 尚無 | 待核對 | 伺服器待定：材料 40319×2（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 155 | 烈炎氣息 | `FIRE_BLESS` | 妖精 | FireBlessSpell | ① 無尾段 | none／8／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 4） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 156 | 暴風之眼 | `STORM_EYE` | 妖精 | StormEyeSpell | ① 無尾段 | none／8／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 157 | 大地屏障 | `EARTH_BIND` | 妖精 | BindSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 50 | 0／280 | 未接（T022 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40319×2（伺服器 T025） |  |
| 158 | 生命之泉 | `NATURES_TOUCH` | 妖精 | NaturesTouchSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 20 | 0／280 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 159 | 大地的祝福 | `EARTH_BLESS` | 妖精 | EarthBlessSpell | ① 無尾段 | none／8／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 160 | 水之防護 | `AQUA_PROTECTER` | 妖精 | AquaProtectSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 30 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 161 | 封印禁地 | `AREA_OF_SILENCE` | 妖精 | AreaSilenceSpell | ① 無尾段 | none／3／射程 0／範圍 -1 | 不適用（非物件目標） | MP 40 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40319×8（伺服器 T025） |  |
| 162 | 召喚強力屬性精靈 | `GREATER_ELEMENTAL` | 妖精 | ElementalSummonSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40319×4（伺服器 T025）、受控 NPC（伺服器 T023） |  |
| 163 | 烈炎武器 | `BURNING_WEAPON` | 妖精 | FireWeaponSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助） | MP 30 | 0／280 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 164 | 生命的祝福 | `NATURES_BLESSING` | 妖精 | NaturesBlessingSpell | ① 無尾段 | none／24／射程 0／範圍 -1 | 不適用（非物件目標） | MP 30 | 0／0 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 165 | 生命呼喚 | `CALL_OF_NATURE` | 妖精 | ResurrectionSpell | ④ d(target) | buff／3／射程 10／範圍 0 | 不需要（自身／友方輔助） | MP 50 | 0／280 | 未接（T022 批 5） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 166 | 暴風神射 | `STORM_SHOT` | 妖精 | StormShotSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助） | MP 30 | 0／280 | 未接（T022 批 5） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 167 | 風之枷鎖 | `WIND_SHACKLE` | 妖精 | WindShackleSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 50 | 1001／280 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 168 | 鋼鐵防護 | `IRON_SKIN` | 妖精 | IronSkinSpell | ④ d(target) | none／0／射程 0／範圍 0 | 不需要（自身／友方輔助） | MP 30 | 714／280 | 未接（T022 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 169 | 體能激發 | `EXOTIC_VITALIZE` | 妖精 | ExoticVitalizeSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 0／0 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 170 | 水之元氣 | `WATER_LIFE` | 妖精 | WaterLifeSpell | ④ d(target) | buff／1／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 1 | 0／280 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 171 | 屬性之火 | `ELEMENTAL_FIRE` | 妖精 | PhysicalProcBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 20 | 0／0 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 172 | 暴風疾走 | `STORM_WALK` | 妖精 | OtherNoEffectSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 10 | 0／0 | 未接（T022 批 6） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 173 | 污濁之水 | `POLLUTE_WATER` | 妖精 | PolluteWaterSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 50 | 0／280 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 174 | 精準射擊 | `STRIKER_GALE` | 妖精 | StrikerGaleSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 -1／範圍 0 | 不需要（自身／友方輔助） | MP 10 | 0／280 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×5（伺服器 T025） |  |
| 175 | 烈焰之魂 | `SOUL_OF_FLAME` | 妖精 | PhysicalProcBuffSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 0／0 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×3（伺服器 T025） |  |
| 176 | 能量激發 | `ADDITIONAL_FIRE` | 妖精 | AdditionalFireSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 30 | 0／0 | 未接（T022 批 6） | 尚無 | 待核對 | 伺服器待定：材料 40319×1（伺服器 T025） |  |
| 181 | 龍之護鎧 | `DRAGON_SKIN` | 龍騎士 | DragonSkinSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 15 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 182 | 燃燒擊砍 | `BURNING_SLASH` | 龍騎士 | BurningSlashSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 10 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 183 | 護衛毀滅 | `GUARD_BRAKE` | 龍騎士 | GuardBrakeSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 0、HP 25 | 0／280 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 184 | 岩漿噴吐 | `MAGMA_BREATH` | 龍騎士 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 10／範圍 3 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 0、HP 35 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 185 | 覺醒：安塔瑞斯 | `AWAKEN_ANTHARAS` | 龍騎士 | AwakenSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 1 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 伺服器待定：材料 49157×1（伺服器 T025） |  |
| 186 | 血之渴望 | `BLOODLUST` | 龍騎士 | BloodlustSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 30 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 伺服器待定：材料 49157×1（伺服器 T025） |  |
| 187 | 屠宰者 | `FOE_SLAYER` | 龍騎士 | FoeSlayerSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 2／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 0、HP 20 | 0／0 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 188 | 恐懼無助 | `RESIST_FEAR` | 龍騎士 | ResistFearSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 0、HP 30 | 0／280 | 未接（T022 批 7） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 189 | 岩漿之箭 | `SHOCK_SKIN` | 龍騎士 | AreaAttackSpell | ① 無尾段 | none／3／射程 0／範圍 2 | 不適用（非物件目標） | MP 0、HP 40 | 0／0 | 未接（T022 批 8） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 190 | 覺醒：法利昂 | `AWAKEN_FAFURION` | 龍騎士 | AwakenSpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 1 | 0／0 | 未接（T022 批 8） | 尚無 | 待核對 | 伺服器待定：材料 49157×1（伺服器 T025） |  |
| 191 | 致命身軀 | `MORTAL_BODY` | 龍騎士 | MortalBodySpell | ① 無尾段 | none／0／射程 0／範圍 0 | 不適用（非物件目標） | MP 0、HP 50 | 0／0 | 未接（T022 批 8） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 192 | 奪命之雷 | `THUNDER_GRAB` | 龍騎士 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 8／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 0、HP 45 | 0／0 | 未接（T022 批 8） | 尚無 | 待核對 | 伺服器待定：處理器待查 |  |
| 193 | 驚悚死神 | `HORROR_OF_DEATH` | 幻術師 | GuardBrakeSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 0、HP 50 | 0／280 | 未接（T022 批 8） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
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
| 212 | 幻想 | `PHANTASM` | 幻術師 | PhantasmSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 30、HP 25 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 213 | 武器破壞者 | `ARM_BREAKER` | 幻術師 | AttackSpell | ⑩ d(target) h(x) h(y) | attack／3／射程 3／範圍 0 | 依目標而定：對敵方不需要；對同隊需要 Ctrl（附 LFC1） | MP 25 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 伺服器待定：處理器待查 |  |
| 214 | 幻覺：鑽石高崙 | `ILLUSION_DIAMOND_GOLEM` | 幻術師 | DiamondGolemSpell | ⑩ d(target) h(x) h(y) | buff／1／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 25 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 215 | 立方：衝擊 | `CUBE_SHOCK` | 幻術師 | CubeShockSpell | ① 無尾段 | none／32／射程 0／範圍 3 | 不適用（非物件目標） | MP 55 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 伺服器待定：材料 49156×1（伺服器 T025）、立方體 HP 生產端（伺服器 T024） |  |
| 216 | 洞察 | `INSIGHT` | 幻術師 | InsightSpell | ⑩ d(target) h(x) h(y) | buff／1／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 40 | 0／280 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
| 217 | 恐慌 | `PANIC` | 幻術師 | PanicSpell | ⑩ d(target) h(x) h(y) | buff／3／射程 3／範圍 0 | 不需要（自身／友方輔助） | MP 30、HP 30 | 0／0 | 未接（T022 批 10） | 尚無 | 待核對 | 以 E149 為準可驗 |  |
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
