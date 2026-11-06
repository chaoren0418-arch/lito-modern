# Issue #173 — T041 本機測試套件：技能與 G06

**腳本擴充已交付；完整 M2 驗收尚未通過。** 最終基線 80／80、技能 41／41、既有 G06 5／5 通過；缺少的三個 G06 scenario 明列失敗，完整入口結束碼為 **1**。待 T023 第 2 張補回三案後才能取得八案全通過的結論。

## 做了什麼

工作目錄為 `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-173`（以下稱 W），沿用 `task/173`，基準 `6dbfbd9`。未 checkout、建分支、合併其他任務或修改 Rust／Java／正式服務設定。

- `run-local-test-suite.ps1` 預設包含原 T012 基線、技能、G06 三類；總表新增分類欄。`-Quick` 維持 check＋Debug lib；`-Category Baseline/Skills/G06` 可單獨診斷。
- 技能依 T026 狀態表與經核對的 `skills.tsv` 精確執行：33 個短名稱展開成 41 個入口（38 整合、3 lib）。處理重複 target、巢狀 module 與 `include!` 三個引入端；每項都使用 `--include-ignored --exact`。
- 啟動時核對技能表 181 列、ID 唯一性、名稱格式、清單重複、未對照與過期項；表新增測試而清單漏更會失敗。143 列「尚無」測試明列未驗證，不當作通過，也不改寫 T026 狀態。
- G06 固定八案，與既有 `native_E149_recovery_contract.py` 名稱一致。以當次 Cargo JSON artifact 找到本 worktree 的測試 EXE，經既有 `run-teleport-failure.ps1` 執行；保留 E149 hash、memory 模式、loopback、來源／程序所有權、lease、清理守門。再核對 receipt、精確測試名稱及 T012 `running/test result` 門檻。
- 缺工具／JAR、建置失敗會列出八案失敗。當版 owned runner 尚不支援的三案列 `unsupported-owned-scenario (T023-2)`；實跑首案失敗後停止後續支援案，不重試。未執行項的數量與結束碼使用 `N/A`，不偽造 0。
- 共用子程序執行支援含空白路徑，清除繼承的 `LITO_*`、隔離 TEMP／TMP，保存完整命令／stdout／stderr。Windows PowerShell 5.1 以 AST 唯讀取得要求 PS7 的 runner 參數，不執行 runner 來探測能力。

## 已驗證

### 工具與安全範圍

本輪使用唯讀工具路徑 `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe`（PowerShell 7.6.5），以及 `D:\dev-cache\cargo\bin\cargo.exe`。所有新產物均在 W 的 `scratch/` 或 `LiTo381-Modern/target/current/`；沒有寫原始專案目錄。

正式 JAR 從 `D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar` 唯讀驗證後複製到 W 的 `LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar`。來源與副本 SHA256 均為 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。

41 個技能入口及其 fixture helper 已靜態核對：記憶體狀態、嵌入 fixture 與選擇器／封包／時鐘斷言；fire arrow 入口另透過 helper 唯讀原客戶端資源，在記憶體 canvas 驗投射物與事件。沒有啟動 GUI、遊戲或外連／DB。未選取 `native_world_entry` 同檔的真實服務測試，也未以整個 target 取代精確選取。

### 腳本自身回歸

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/test-local-test-suite.ps1
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' -NoProfile -File 維護工具/test-local-test-suite.ps1
```

兩者結束碼均為 0，最終輸出：

```text
Runner QA: 55 assertions passed; PowerShell=5.1.19041.6456. Synthetic checks only.
Runner QA: 55 assertions passed; PowerShell=7.6.5. Synthetic checks only.
```

涵蓋正向與 CRLF 結果、0 tests、failed／ignored／measured、缺損結果、多 harness、非零結束碼、技能表漏項／重複／過期清單、實際子程序參數引號與環境隔離、啟動失敗、路徑界線，以及 G06 五案支援／三案缺失、八案支援、清理錯誤、錯誤測試身分、非零碼與首敗停止。G06 八案正向在此使用明確標示的合成 adapter，**不算實際 G06 通過證據**。回歸腳本不啟動 Cargo、Java、網路服務或遊戲。

原始 log：`scratch/issue-173/runner-qa-ps51.log`、`runner-qa-ps7.log`。開發回歸過程修正了測試載入器未綁定 `$PSScriptRoot` 的問題；也修正實際 PS5.1 無法讀 PS7 `Get-Command.Parameters` 的問題。上列數字取修正後最終執行。

### Cargo 與整套實跑

共同參數固定為 `--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`，cwd 為 W 的 `LiTo381-Modern`。完整入口在 W 根目錄：

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' -NoProfile -File 維護工具/run-local-test-suite.ps1
```

最終完整入口結束碼 **1**。實際總表共 129 步；按類別統計如下（check 是建置檢查，不是 Rust 測試數）：

| 類別 | 步驟 | PASS | FAIL／未執行 |
|---|---:|---:|---:|
| Baseline | 80 | 80 | 0 |
| 技能 (Skills) | 41 | 41 | 0 |
| G06 | 8 | 5 | 3 |

最終證據：`LiTo381-Modern/target/current/verification/local-test-suite/20261106-150351-657-18988/summary.txt` 與同目錄編號 log；外層 console／結束碼為 `scratch/issue-173/final-suite.log`、`final-suite.exit`。

指定建置命令如下，最終執行結束碼 **0**（`001.log`；check 不產生 `test result:`）：

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

Debug／Release lib 命令為 `cargo test` 加共同參數與 `--lib --no-fail-fast`（Release 再加 `--release`），兩者結束碼 **0**；原始結果：

```text
log=002.log; exit=0
test result: ok. 1915 passed; 0 failed; 575 ignored; 0 measured; 0 filtered out; finished in 5.05s
log=003.log; exit=0
test result: ok. 1915 passed; 0 failed; 575 ignored; 0 measured; 0 filtered out; finished in 1.61s
```

當版 lib 為 1915 passed／575 ignored；相較 #101 的 1889 passed，增加 26 個通過項。本輪 Rust 原始碼沒有變更，差異屬起始基準版本差異，不將重跑或不同 profile 當成不同測試案例。

最終總表節錄（保留所有技能與 G06 列；原 T012 的 76 個白名單整合列見完整 summary，皆按原門檻逐項執行）：

```text
LOCAL TEST SUITE | Mode=Full | Category=All | PowerShell=7.6.5
Skills scope: named T026 tests only; catalog rows without tests are NOT verified.
Category | Step | Exit | passed | failed | ignored | running-0-tests | Seconds | Result
Baseline | check | 0 | N/A | N/A | N/A | N/A | 4.096 | PASS
Baseline | lib-debug | 0 | 1915 | 0 | 575 | 0 | 5.599 | PASS
Baseline | lib-release | 0 | 1915 | 0 | 575 | 0 | 6.019 | PASS
Baseline | ignored-lib:modern_lito::compositor::tests::measure_full_frame_clear_and_dib_conversion | 0 | 1 | 0 | 0 | 0 | 2.797 | PASS
技能 (Skills) | native_ability_buffs::ability_buffs_keep_native_screen_asymmetric_edges | 0 | 1 | 0 | 0 | 0 | 0.503 | PASS
技能 (Skills) | native_ability_buffs::ability_buffs_reject_unlearned_dead_missing_and_outside_screen_targets | 0 | 1 | 0 | 0 | 0 | 0.437 | PASS
技能 (Skills) | native_ability_buffs::both_ability_buffs_use_explicit_self_or_selected_object_without_authority_writes | 0 | 1 | 0 | 0 | 0 | 0.255 | PASS
技能 (Skills) | native_cone_cold::cold_request_uses_observed_C14_and_six_cell_range_without_spending_authority | 0 | 1 | 0 | 0 | 0 | 0.541 | PASS
技能 (Skills) | native_cone_cold::cold_selector_rechecks_target_life_generation_owner_and_book | 0 | 1 | 0 | 0 | 0 | 0.246 | PASS
技能 (Skills) | native_attack_skills::disintegrate_request_contract::current_c02_exact_object_coordinates_and_readonly_authority | 0 | 1 | 0 | 0 | 0 | 0.986 | PASS
技能 (Skills) | native_attack_skills::call_lightning_request_contract::current_c02_exact_object_coordinates_and_readonly_authority | 0 | 1 | 0 | 0 | 0 | 0.271 | PASS
技能 (Skills) | native_obstacles::destroyed_guardian_retires_attacks_spells_and_selection_even_if_repaired_between_polls | 0 | 1 | 0 | 0 | 0 | 0.653 | PASS
技能 (Skills) | native_skills::detection_selector_menu_guards_and_cooldown_leave_authority_unchanged | 0 | 1 | 0 | 0 | 0 | 1.056 | PASS
技能 (Skills) | native_attack_skills::formal_attack_spell_layout_and_request_guards | 0 | 1 | 0 | 0 | 0 | 0.312 | PASS
技能 (Skills) | native_fire_arrow::formal_attack_spell_layout_and_request_guards | 0 | 1 | 0 | 0 | 0 | 2.921 | PASS
技能 (Skills) | native_ice_arrow::formal_attack_spell_layout_and_request_guards | 0 | 1 | 0 | 0 | 0 | 1.079 | PASS
技能 (Skills) | native_wind_cutter::formal_attack_spell_layout_and_request_guards | 0 | 1 | 0 | 0 | 0 | 1.060 | PASS
技能 (Skills) | native_ground_skills::formal_ground_request_unsigned_boundaries_range_and_authority | 0 | 1 | 0 | 0 | 0 | 0.871 | PASS
技能 (Skills) | native_reagent_skills::formal_reagent_request_self_screen_boundary_and_no_prediction | 0 | 1 | 0 | 0 | 0 | 0.916 | PASS
技能 (Skills) | native_actor_states::hidden_targets_are_rejected_until_visible_and_identity_replacement_invalidates_selection | 0 | 1 | 0 | 0 | 0 | 0.894 | PASS
技能 (Skills) | native_attack_skills::disintegrate_request_contract::invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests | 0 | 1 | 0 | 0 | 0 | 0.262 | PASS
技能 (Skills) | native_attack_skills::call_lightning_request_contract::invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests | 0 | 1 | 0 | 0 | 0 | 0.233 | PASS
技能 (Skills) | native_skills::invisibility_tick_envelope_guards_clock_and_menu_preserve_authority | 0 | 1 | 0 | 0 | 0 | 0.276 | PASS
技能 (Skills) | native_world_entry::learned_world_tree_is_selector_only_and_cross_map_admission_stays_owned | 0 | 1 | 0 | 0 | 0 | 0.954 | PASS
技能 (Skills) | native_light_spell::light_uses_self_selector_without_predicting_authority_and_scopes_UI_choice | 0 | 1 | 0 | 0 | 0 | 0.965 | PASS
技能 (Skills) | native_turn_undead::low_mana_turn_undead_times_out_without_predicted_mana_death_or_rejection | 0 | 1 | 0 | 0 | 0 | 0.717 | PASS
技能 (Skills) | native_attack_skills::disintegrate_request_contract::mp_and_lawful_are_server_permission_inputs_and_never_locally_spent | 0 | 1 | 0 | 0 | 0 | 0.234 | PASS
技能 (Skills) | native_attack_skills::call_lightning_request_contract::mp_and_lawful_are_server_permission_inputs_and_never_locally_spent | 0 | 1 | 0 | 0 | 0 | 0.235 | PASS
技能 (Skills) | native_selector_only::nine_server_selectors_ignore_hover_and_never_spend_local_authority | 0 | 1 | 0 | 0 | 0 | 0.260 | PASS
技能 (Skills) | native_poison_cure::object_and_coordinate_envelopes_preserve_authority_and_gate_live_targets | 0 | 1 | 0 | 0 | 0 | 0.602 | PASS
技能 (Skills) | native_holy_weapon::object_selector_screen_and_owner_guards_preserve_authority | 0 | 1 | 0 | 0 | 0 | 0.610 | PASS
技能 (Skills) | native_attack_skills::call_lightning_request_contract::range_six_boundary_and_target_changes_use_current_received_coordinates | 0 | 1 | 0 | 0 | 0 | 0.222 | PASS
技能 (Skills) | native_attack_skills::disintegrate_request_contract::range_ten_boundary_and_target_changes_use_current_received_coordinates | 0 | 1 | 0 | 0 | 0 | 0.255 | PASS
技能 (Skills) | native_lightning::request_clock_reuse_range_and_target_guard_preserve_authority | 0 | 1 | 0 | 0 | 0 | 0.650 | PASS
技能 (Skills) | native_chill_touch::request_layout_guards_and_server_authority_do_not_predict_healing_or_hurt | 0 | 1 | 0 | 0 | 0 | 0.547 | PASS
技能 (Skills) | native_skill_selection::selection_uses_formal_packets_and_preserves_authority | 0 | 1 | 0 | 0 | 0 | 0.760 | PASS
技能 (Skills) | native_turn_undead::selector_clock_range_and_dead_target_guards_never_predict_death_or_mana | 0 | 1 | 0 | 0 | 0 | 0.214 | PASS
技能 (Skills) | native_selector_only::selector_learning_death_and_mana_guards_do_not_forge_requests | 0 | 1 | 0 | 0 | 0 | 0.215 | PASS
技能 (Skills) | native_skills::spell_requests_use_formal_selectors_and_never_predict_mana | 0 | 1 | 0 | 0 | 0 | 0.224 | PASS
技能 (Skills) | native_stalac_layers::stalac_packet_range_clock_and_no_predicted_authority | 0 | 1 | 0 | 0 | 0 | 0.635 | PASS
技能 (Skills) | native_skills::tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority | 0 | 1 | 0 | 0 | 0 | 0.237 | PASS
技能 (Skills) | __lib::modern_lito::native_call_lightning::availability_tests::unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation | 0 | 1 | 0 | 0 | 0 | 0.670 | PASS
技能 (Skills) | __lib::modern_lito::native_cone_cold::availability_tests::unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation | 0 | 1 | 0 | 0 | 0 | 0.218 | PASS
技能 (Skills) | __lib::modern_lito::native_disintegrate::availability_tests::unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation | 0 | 1 | 0 | 0 | 0 | 0.219 | PASS
技能 (Skills) | native_vampiric_touch::vampiric_envelope_range_and_authority_do_not_predict_healing_or_hurt | 0 | 1 | 0 | 0 | 0 | 0.530 | PASS
G06 | timeout | 0 | 1 | 0 | 0 | 0 | 92.195 | PASS
G06 | refused | 0 | 1 | 0 | 0 | 0 | 76.851 | PASS
G06 | wrong-service | 0 | 1 | 0 | 0 | 0 | 89.230 | PASS
G06 | cancel-retry | 0 | 1 | 0 | 0 | 0 | 73.924 | PASS
G06 | cancel-expiry | 0 | 1 | 0 | 0 | 0 | 175.983 | PASS
G06 | cancel-repeat | N/A | N/A | N/A | N/A | N/A | 0.000 | FAIL: unsupported-owned-scenario (T023-2)
G06 | refused-cancel-repeat | N/A | N/A | N/A | N/A | N/A | 0.000 | FAIL: unsupported-owned-scenario (T023-2)
G06 | cancel-expiry-source | N/A | N/A | N/A | N/A | N/A | 0.000 | FAIL: unsupported-owned-scenario (T023-2)
Steps=129 | Final exit code=1
```

技能的每筆 Cargo 命令均見對應 log 首行：`cargo test`＋共同參數＋`--test <target>`（`__lib` 改 `--lib`）＋`--no-fail-fast -- --include-ignored --exact <完整名稱>`。以下為 41 筆 `test result:` 原文與結束碼，未將 0 tests／ignored 算成通過：

```text
081.log | native_ability_buffs::ability_buffs_keep_native_screen_asymmetric_edges | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
082.log | native_ability_buffs::ability_buffs_reject_unlearned_dead_missing_and_outside_screen_targets | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
083.log | native_ability_buffs::both_ability_buffs_use_explicit_self_or_selected_object_without_authority_writes | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
084.log | native_cone_cold::cold_request_uses_observed_C14_and_six_cell_range_without_spending_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
085.log | native_cone_cold::cold_selector_rechecks_target_life_generation_owner_and_book | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
086.log | native_attack_skills::disintegrate_request_contract::current_c02_exact_object_coordinates_and_readonly_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
087.log | native_attack_skills::call_lightning_request_contract::current_c02_exact_object_coordinates_and_readonly_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
088.log | native_obstacles::destroyed_guardian_retires_attacks_spells_and_selection_even_if_repaired_between_polls | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
089.log | native_skills::detection_selector_menu_guards_and_cooldown_leave_authority_unchanged | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
090.log | native_attack_skills::formal_attack_spell_layout_and_request_guards | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
091.log | native_fire_arrow::formal_attack_spell_layout_and_request_guards | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 1.68s
092.log | native_ice_arrow::formal_attack_spell_layout_and_request_guards | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.01s
093.log | native_wind_cutter::formal_attack_spell_layout_and_request_guards | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.01s
094.log | native_ground_skills::formal_ground_request_unsigned_boundaries_range_and_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
095.log | native_reagent_skills::formal_reagent_request_self_screen_boundary_and_no_prediction | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
096.log | native_actor_states::hidden_targets_are_rejected_until_visible_and_identity_replacement_invalidates_selection | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
097.log | native_attack_skills::disintegrate_request_contract::invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.01s
098.log | native_attack_skills::call_lightning_request_contract::invalid_unlearned_dead_invisible_blocked_and_missing_states_cannot_make_requests | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
099.log | native_skills::invisibility_tick_envelope_guards_clock_and_menu_preserve_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
100.log | native_world_entry::learned_world_tree_is_selector_only_and_cross_map_admission_stays_owned | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
101.log | native_light_spell::light_uses_self_selector_without_predicting_authority_and_scopes_UI_choice | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
102.log | native_turn_undead::low_mana_turn_undead_times_out_without_predicted_mana_death_or_rejection | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
103.log | native_attack_skills::disintegrate_request_contract::mp_and_lawful_are_server_permission_inputs_and_never_locally_spent | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
104.log | native_attack_skills::call_lightning_request_contract::mp_and_lawful_are_server_permission_inputs_and_never_locally_spent | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
105.log | native_selector_only::nine_server_selectors_ignore_hover_and_never_spend_local_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.01s
106.log | native_poison_cure::object_and_coordinate_envelopes_preserve_authority_and_gate_live_targets | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
107.log | native_holy_weapon::object_selector_screen_and_owner_guards_preserve_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
108.log | native_attack_skills::call_lightning_request_contract::range_six_boundary_and_target_changes_use_current_received_coordinates | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
109.log | native_attack_skills::disintegrate_request_contract::range_ten_boundary_and_target_changes_use_current_received_coordinates | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
110.log | native_lightning::request_clock_reuse_range_and_target_guard_preserve_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
111.log | native_chill_touch::request_layout_guards_and_server_authority_do_not_predict_healing_or_hurt | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
112.log | native_skill_selection::selection_uses_formal_packets_and_preserves_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
113.log | native_turn_undead::selector_clock_range_and_dead_target_guards_never_predict_death_or_mana | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
114.log | native_selector_only::selector_learning_death_and_mana_guards_do_not_forge_requests | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.01s
115.log | native_skills::spell_requests_use_formal_selectors_and_never_predict_mana | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
116.log | native_stalac_layers::stalac_packet_range_clock_and_no_predicted_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
117.log | native_skills::tw_higher_heal_requests_use_learned_ids_original_clocks_and_never_predict_authority | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
118.log | __lib::modern_lito::native_call_lightning::availability_tests::unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2489 filtered out; finished in 0.00s
119.log | __lib::modern_lito::native_cone_cold::availability_tests::unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2489 filtered out; finished in 0.00s
120.log | __lib::modern_lito::native_disintegrate::availability_tests::unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2489 filtered out; finished in 0.00s
121.log | native_vampiric_touch::vampiric_envelope_range_and_authority_do_not_predict_healing_or_hurt | exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

G06 的本次建置為 `cargo test`＋共同參數＋`--test native_teleport_failure --no-run --message-format=json`，結束碼 **0**（`123.log`）；no-run 不計為測試通過。之後由 owned runner 以 `--exact --ignored --nocapture` 呼叫該 artifact。實際執行結果：

```text
124.log | timeout | owned-runner exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 75.26s
125.log | refused | owned-runner exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 67.71s
126.log | wrong-service | owned-runner exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 80.21s
127.log | cancel-retry | owned-runner exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 64.43s
128.log | cancel-expiry | owned-runner exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 163.12s
```

G06 最終 owned receipt 核對（路徑相對 `LiTo381-Modern/target/current/verification/native-teleport-failure/`）：

| Scenario | sessions | memberships_empty | cleanup／drift errors | SQL access | receipt |
|---|---:|---|---|---|---|
| timeout | 8 | True | 0／0 | False | 20261106T070512130Z-3f0c5f2f0ad945dbae852fc3a4144c35/receipt.json |
| refused | 8 | True | 0／0 | False | 20261106T070644538Z-2d02d7bb7cd94748832a80edfd8356b3/receipt.json |
| wrong-service | 10 | True | 0／0 | False | 20261106T070801570Z-78e4ef002f064ead90eca7411c3e0166/receipt.json |
| cancel-retry | 8 | True | 0／0 | False | 20261106T070930656Z-0ad57632814641bc91a81817094c3cfa/receipt.json |
| cancel-expiry | 8 | True | 0／0 | False | 20261106T071045253Z-3f9a3438e4c94d2a8f78f83169bf7660/receipt.json |

### 首輪阻塞與環境修正

首輪完整入口 `20261106-144135-325-23884`：基線 80／80、技能 41／41 通過；G06 timeout 在 Rust 啟動前因 `Get-NetTCPConnection` 回報「拒絕存取」（`HRESULT 0x80041003`）停止，後四案未執行，另三案不支援，整體結束碼 1。receipt 位於 `target/current/native-teleport-failure/20261106T070026313Z-5d55d652e7334cc0bcfa92a945c01def/receipt.json`：`cleanup_errors=[]`、`drift_errors=[]`、`inputs_unchanged=true`，自建 Java 已回收、lease 已釋放；沒有 Rust `test result:`，不能算通過。

其後唯讀確認：沙箱內同一查詢確實失敗，沙箱外查詢成功。經工具自動審核允許，最終整套在可執行該所有權查詢的本機環境重跑；未改 ACL、未刪別人的 lease、未放寬 owned runner 守門，也未修改既有遊戲程序。另將子 pwsh 輸出明確設為 UTF-8，避免原先中文錯誤被 ANSI 編碼破壞；正確原始錯誤也保存在首輪 UTF-8 receipt 中。這是確認原因後的環境修正驗證，腳本自身沒有自動重試。

開發前 Quick 基線亦為 check＋Debug 成功（`20261106-143024-696-16408`；exit 0），不與最終通過數重複累加。

### 補充：PowerShell 5.1 真實技能入口與清理核對（已驗證）

最終版另執行 `powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/run-local-test-suite.ps1 -Category Skills`，結束碼 **0**，41／41 每項均為 `running 1 test`、1 passed／0 failed／0 ignored。總表位於 `LiTo381-Modern/target/current/verification/local-test-suite/20261106-151340-714-18352/summary.txt`，原始 Cargo 結果在同目錄 `001.log`–`041.log`；console／結束碼為 `scratch/issue-173/skills-ps51.log`、`skills-ps51.exit`。這是相同技能測試的跨 shell 重跑，不累加獨立案例數。

五案最終 receipt 的 javac／Java／Rust 共 15 個自建程序均 `wait_for_exit=true`、`exit_code=0`；全部 `authority_mutations=0`、`staged_loadouts_after_reentry=0`、`full_game_reentry_verified=true`、`state_persistent=false`，最終 `runner.lease` 不存在。沒有停止既有遊戲程序。

## 推測

沒有以推測補填通過項。T023 第 2 張補齊既定名稱的 Rust 測試與 owned runner 支援後，本腳本已具備八案調度；仍須實跑確認，不能由合成 adapter 推論通過。

## 待驗證／未完成事項

- G06 後三案尚不在當版 `native_teleport_failure.rs`／owned runner 中：`cancel-repeat`（repeated_cancellation）、`refused-cancel-repeat`（failed_handoff_then_repeated_cancel）、`cancel-expiry-source`（expired_stage_and_source_handoff）。T023 第 2 張是它們的實作範圍，本張不移入隔離候選或新增假測試。
- 技能表 143 列尚無測試，以及資源／特效完整核對、伺服器同源性、GUI、正式服務、DB 均未因本輪而取得通過結論。未執行白名單以外的廣域回歸；略過的 575 個 lib 測試不算通過，基線另精確執行其中白名單 1 項。
- 不宣稱 M2 完成；完整套件仍須八個 G06 scenario 全部實跑通過且總碼 0。

## 產出路徑

- `維護工具/run-local-test-suite.ps1`
- `維護工具/test-local-test-suite.ps1`
- `維護工具/test-whitelist/skills.tsv`
- `維護工具/test-whitelist/README.md`
- `docs/reports/issue-173.md`

逐步 log／summary／owned harness receipt 在 `LiTo381-Modern/target/current/`；驗收輔助資料在 `scratch/issue-173/`，均不進 git。原 T012 的 `lib.txt`、`tests.tsv`、`docs/REPORT.md`、`TASKS.md` 與其他任務檔案未改。

## 給下一棒的提示

交 Claude 審查技能對照完整性、失敗門檻、G06 owned runner 接線與報告。T023 第 2 張執行者須補齊上述三案及 runner 的 `Scenario` ValidateSet／精確測試對照，沿用既有所有權與清理契約；完成後由本機 codex-local 重跑預設完整套件。後續 T026／技能實作更新測試名稱時，同步核對 `skills.tsv`，特別是同名 target 與 include 模組。調度器合併後自行更新 REPORT 索引。
