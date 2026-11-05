# Issue #101：T012 本機測試套件

## 結論

**已驗證：實作完成；同步 T013 後，PowerShell 7.6.5 與 Windows PowerShell 5.1.19041.6456 的完整版、`-Quick` 四次最終實跑結束碼全部為 0。** 完整版各 80 個步驟全部符合成功門檻：check、lib Debug／Release、1 項 ignored lib、76 項白名單整合測試。此報告取代先前 PowerShell 前置阻擋紀錄。

## 做了什麼

- 沿用 worktree `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-101` 與 `task/101`；起始 SHA `e6323498504cc7578fb91b7909141d9dd4aadabd`，工作樹原先乾淨。沒有 checkout／建立分支。原始專案檔案只讀；提交／同步所需共用 Git metadata 由 Git 正常管理。
- 已讀 AGENTS.md、docs/REPORT.md、TASKS.md T012／T013、#53／#54／#56／#91 與先前 #101 報告。依本次明確指示，以專案內可攜 pwsh 解除舊版「找不到就停止」的前置阻擋。
- 從 #54「白名單原文」兩個 code block 原樣擷取 UTF-8 無 BOM、LF 檔案；tests.tsv 為四欄真正 Tab、沒有標題列。lib 1 行、tests 76 行，逐字節比對通過，沒有擴充白名單。
- 新增 PowerShell 5.1／7 相容的 `維護工具/run-local-test-suite.ps1`：路徑相對腳本定位，依序執行指定 Cargo 命令；`-Quick` 只執行 check 與 Debug lib。PATH 沒有 cargo 時使用既有 `CARGO_HOME/bin/cargo.exe`，不改 PATH。
- 每個 Cargo 都使用獨立 ProcessStartInfo 環境：TEMP／TMP 指向此 worktree 的 `LiTo381-Modern/target/current/scratch`，移除繼承的全部 `LITO_*`。不變更呼叫端環境，因此結束後不需復原；雙版本實測呼叫端值保持原樣。
- check 只判結束碼，測試數為 N/A。測試步驟要求唯一有效的 running／test result 記錄及一致計數；0 tests、缺失／格式錯誤、failed > 0 或命令非 0 都失敗。白名單另要求 running 1、passed 1、failed／ignored／measured 0。失敗不短路，最後彙整為非零結束碼。
- 總表同時印出並存成 UTF-8 `summary.txt`；每一步另存命令與 stdout／stderr 到同目錄的編號 `.log`。長步驟每 30 秒印進度；雙輸出管線非同步讀取，避免編譯診斷塞滿管線。
- 起始基準沒有 DEMO；驗收期間查得 T013 已經 PR #105 合併，因此在第一輪四次驗收全數通過後，同步 `origin/main` 的 `fde15d48bb77ba4f1784edb8e1eddee377cdcb41`，未發生衝突，並更新 `docs/DEMO.md`「怎麼跑測試」的實際連結、日常命令及總表說明。
- 同步帶入的場景可見性修正、截圖、#102 報告與索引均是上游既有內容，本任務未編輯。最終測試時 `git diff fde15d48bb77ba4f1784edb8e1eddee377cdcb41 -- LiTo381-Modern` 為空；本任務相對該合併基準的交付差異只有指定五檔，沒有 src、tests 或 docs/REPORT.md。

## PowerShell 前置與工具來源

原執行環境 `(Get-Command pwsh).Source` 找不到命令；`$PSVersionTable.PSVersion` 為 **5.1.19041.6456**。

重用 #91 已下載的官方 ZIP（唯讀來源 `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\scratch\tools\powershell.zip`），先核對 [PowerShell 官方 v7.6.5 發布頁](https://github.com/PowerShell/PowerShell/releases/tag/v7.6.5) 的 Windows x64 ZIP SHA256，再解壓到本 worktree `scratch/tools/pwsh/`。未安裝系統工具、未改 PATH 或永久設定。

```text
ZIP SHA256: 32EB8F6CDCE08F86E987D625A2733E54AC3E289AE7E1621B14C0B5BCEC2434EA
(Get-Command pwsh).Source（由該 pwsh 自身執行）:
D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-101\scratch\tools\pwsh\pwsh.exe
$PSVersionTable.PSVersion: 7.6.5
Authenticode: Valid
Signer: CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
rustc 1.95.0 (59807616e 2026-04-14)
CARGO_HOME=D:\dev-cache\cargo
RUSTUP_HOME=D:\dev-cache\rustup
```

白名單 SHA256：

| 檔案 | SHA256 |
|---|---|
| 維護工具/test-whitelist/lib.txt | `4D2A60D6D94D9AA222DDA6E3EEEE481284D4BB0D71351AF70F7F56CDC3031065` |
| 維護工具/test-whitelist/tests.tsv | `3E95BBE0A3654DA26405FBC9AAFF82C57D4D8E7829859E17FB98424EAA3761B1` |

TSV 雜湊與 #56 完全相同。最終程式碼基準為 `fde15d48bb77ba4f1784edb8e1eddee377cdcb41`；相對 #54 的 `bb96171d2938437dd224af73da466ab8bbab8f73`，只有 #91 的 native_teleport_failure.rs／Java harness，以及 #102 的 scene_preview.rs 可見性修正；沒有新增 lib 測試，白名單測試本體未變更。

## 怎麼驗證

先在起始基準跑完四次真實驗收，均為 0；同步 T013 後，因程式碼基準改變，再完整重跑四次，下表及原樣輸出皆取**同步後的最終四次**。全部從本 worktree 根目錄執行，依序且未同時爭用 target。共同 Cargo 參數為 `--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`。check 用 `--all-targets`；lib 用 `--lib --no-fail-fast`（Release 另加 `--release`）；白名單 lib 用 `-- --ignored --exact <名>`；白名單整合用 `--test <target> [--features ...] --no-fail-fast -- --include-ignored --exact <名>`。

| 實跑 | 步驟 | 腳本最終結束碼 | 總表證據（相對 worktree） |
|---|---:|---:|---|
| Windows PowerShell 5.1 Quick | 2 | 0 | `LiTo381-Modern/target/current/verification/local-test-suite/20261106-063509-432-17880/summary.txt` |
| PowerShell 7.6.5 Quick | 2 | 0 | `LiTo381-Modern/target/current/verification/local-test-suite/20261106-063807-104-10292/summary.txt` |
| PowerShell 7.6.5 完整版 | 80 | 0 | `LiTo381-Modern/target/current/verification/local-test-suite/20261106-063812-343-20128/summary.txt` |
| Windows PowerShell 5.1 完整版 | 80 | 0 | `LiTo381-Modern/target/current/verification/local-test-suite/20261106-065224-128-18452/summary.txt` |

任務指定的完整版命令為：

```powershell
pwsh -NoProfile -File 維護工具/run-local-test-suite.ps1
```

因 PATH 沒有 pwsh，本機實際以完整路徑執行同一命令（Quick 也已實跑）：

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-101\scratch\tools\pwsh\pwsh.exe' -NoProfile -File 維護工具/run-local-test-suite.ps1
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-101\scratch\tools\pwsh\pwsh.exe' -NoProfile -File 維護工具/run-local-test-suite.ps1 -Quick
```

### PowerShell 7 完整版總表原樣輸出

以下直接嵌入 summary.txt，未改動列名、計數、耗時或結束碼。`running-0-tests` 是「未實際執行（running 0 tests）」的出現次數，`Seconds` 為該步耗時秒數；`N/A` 僅表示不適用，不能算測試通過。

```text
LOCAL TEST SUITE | Mode=Full | PowerShell=7.6.5
Step | Exit | passed | failed | ignored | running-0-tests | Seconds | Result
check | 0 | N/A | N/A | N/A | N/A | 0.569 | PASS
lib-debug | 0 | 1889 | 0 | 575 | 0 | 3.209 | PASS
lib-release | 0 | 1889 | 0 | 575 | 0 | 608.766 | PASS
ignored-lib:modern_lito::compositor::tests::measure_full_frame_clear_and_dib_conversion | 0 | 1 | 0 | 0 | 0 | 2.318 | PASS
native_ability_buffs::ability_buffs_keep_native_screen_asymmetric_edges | 0 | 1 | 0 | 0 | 0 | 105.781 | PASS
native_ability_buffs::ability_buffs_reject_unlearned_dead_missing_and_outside_screen_targets | 0 | 1 | 0 | 0 | 0 | 0.712 | PASS
native_ability_buffs::ability_choice_preserves_object_and_fences_reused_actor_and_world | 0 | 1 | 0 | 0 | 0 | 0.204 | PASS
native_ability_buffs::both_ability_buffs_use_explicit_self_or_selected_object_without_authority_writes | 0 | 1 | 0 | 0 | 0 | 0.284 | PASS
native_ability_buffs::current_formal_ability_codec_vectors_and_short_or_extended_packets | 0 | 1 | 0 | 0 | 0 | 0.210 | PASS
native_ability_world::ability_receipts_retire_on_same_map_new_world_and_new_status_owner | 0 | 1 | 0 | 0 | 0 | 5.491 | PASS
native_ability_world::original_ability_packets_update_only_authoritative_fields_and_reject_partial_input_atomically | 0 | 1 | 0 | 0 | 0 | 0.214 | PASS
native_attributes::formal_schema_zero_preserves_attribute_order_signed_modifiers_and_unsigned_experience | 0 | 1 | 0 | 0 | 0 | 5.299 | PASS
native_attributes::identity_and_scene_changes_retire_modifiers_and_unowned_updates_never_create_players | 0 | 1 | 0 | 0 | 0 | 0.212 | PASS
native_attributes::malformed_or_extended_attributes_do_not_partially_change_current_authority | 0 | 1 | 0 | 0 | 0 | 0.219 | PASS
native_doors::authoritative_toggle_requires_both_packets_and_holds_the_new_pose | 0 | 1 | 0 | 0 | 0 | 4.656 | PASS
native_doors::deployed_attribute_and_unsigned_open_vectors_match_native_codec | 0 | 1 | 0 | 0 | 0 | 0.214 | PASS
native_doors::door_identity_removal_replacement_range_and_world_are_retired | 0 | 1 | 0 | 0 | 0 | 0.194 | PASS
native_doors::malformed_attribute_never_partially_changes_world | 0 | 1 | 0 | 0 | 0 | 0.197 | PASS
native_early_roster::captured_early_members_wait_for_matching_acceptance_and_latest_snapshot_wins | 0 | 1 | 0 | 0 | 0 | 4.649 | PASS
native_early_roster::pending_roster_cannot_cross_connection_room_rejection_or_new_request | 0 | 1 | 0 | 0 | 0 | 0.298 | PASS
native_early_roster::pending_roster_preserves_wire_order_and_existing_256_member_bound | 0 | 1 | 0 | 0 | 0 | 0.213 | PASS
native_game_timers::delayed_dispatch_never_catches_up_and_stop_cannot_resurrect_the_owner | 0 | 1 | 0 | 0 | 0 | 5.812 | PASS
native_game_timers::original_immediate_alive_and_300s_save_are_rearmed_from_actual_completion | 0 | 1 | 0 | 0 | 0 | 0.324 | PASS
native_green_clock::actor_green_style_consumes_delivered_native_counter_not_elapsed_nanoseconds | 0 | 1 | 0 | 0 | 0 | 5.194 | PASS
native_green_clock::all_16_original_queue_snapshots_match_late_duplicate_cancel_map_and_scene_retirement | 0 | 1 | 0 | 0 | 0 | 0.304 | PASS
native_green_clock::all_532_original_counter_wrap_fresh_deadline_and_shader_steps_match | 0 | 1 | 0 | 0 | 0 | 0.203 | PASS
native_green_clock::host_budget_refusal_and_unsigned_wrap_storm_do_not_publish_partial_counter | 0 | 1 | 0 | 0 | 0 | 0.201 | PASS
native_green_clock::immutable_render_binding_captures_counter_without_turning_phase_into_a_resource_key | 0 | 1 | 0 | 0 | 0 | 0.189 | PASS
native_green_clock::older_or_retired_generation_cannot_restart_callbacks_or_consume_a_schedule_clock | 0 | 1 | 0 | 0 | 0 | 0.190 | PASS
native_green_clock::owner_and_scene_fences_preserve_counter_but_retire_old_callbacks | 0 | 1 | 0 | 0 | 0 | 0.191 | PASS
native_green_clock::water_and_green_share_stable_dispatch_order_and_distinct_fresh_deadlines | 0 | 1 | 0 | 0 | 0 | 0.252 | PASS
native_light_color::all_original_selector_calls_preserve_level_only_cache_semantics | 0 | 1 | 0 | 0 | 0 | 4.830 | PASS
native_light_color::all_retained_native_kernel_samples_match_with_pitch_guards_and_byte_gates | 0 | 1 | 0 | 0 | 0 | 0.352 | PASS
native_light_color::every_generated_channel_entry_matches_captured_original_writer_output | 0 | 1 | 0 | 0 | 0 | 0.201 | PASS
native_light_color::malformed_inputs_fail_before_any_surface_mutation | 0 | 1 | 0 | 0 | 0 | 0.199 | PASS
native_light_color::rgba_adapter_uses_native_555_entries_preserves_alpha_and_identity_bytes | 0 | 1 | 0 | 0 | 0 | 0.187 | PASS
native_light_lifecycle::existing_put_preserves_native_death_marker_while_resurrection_and_new_identity_clear_it | 0 | 1 | 0 | 0 | 0 | 7.561 | PASS
native_light_sources::all_original_graph_masks_queries_remap_cache_and_blit_order_match | 0 | 1 | 0 | 0 | 0 | 8.378 | PASS
native_light_sources::owner_and_extent_failures_are_atomic_and_native_phase_bounds_are_explicit | 0 | 1 | 0 | 0 | 0 | 0.512 | PASS
native_network::accepted_connection_keeps_socket_until_explicit_cancel_then_reuses_epoch | 0 | 1 | 0 | 0 | 0 | 6.927 | PASS
native_network::coalesced_post_version_packet_survives_peer_eof_and_owner_retirement | 0 | 1 | 0 | 0 | 0 | 0.321 | PASS
native_network::command_batch_is_admitted_atomically_and_written_as_two_ordered_frames | 0 | 1 | 0 | 0 | 0 | 0.335 | PASS
native_network::deployed_jar_version_request_and_replies_match_typed_native_contract | 0 | 1 | 0 | 0 | 0 | 0.372 | PASS
native_network::fragmented_and_coalesced_frames_preserve_full_u16_payload_and_eof_state | 0 | 1 | 0 | 0 | 0 | 0.284 | PASS
native_network::full_event_queue_preserves_burst_and_allows_cancel_without_consumer | 0 | 1 | 0 | 0 | 0 | 0.485 | PASS
native_network::invalid_limits_and_non_loopback_cli_probe_do_not_start_worker | 0 | 1 | 0 | 0 | 0 | 0.231 | PASS
native_network::shared_cipher_retains_unsigned_counter_carry_for_large_packets | 0 | 1 | 0 | 0 | 0 | 0.251 | PASS
native_points::captured_rental_points_are_not_a_battle_result | 0 | 1 | 0 | 0 | 0 | 5.254 | PASS
native_portal::different_map_portal_rejects_wrong_rebuild_and_cancellation_retires_pending | 0 | 1 | 0 | 0 | 0 | 5.482 | PASS
native_portal::formal_portal_payload_is_exact_and_malformed_packets_do_not_retire_world | 0 | 1 | 0 | 0 | 0 | 0.289 | PASS
native_portal::same_map_portal_retires_old_scene_and_waits_for_matching_received_self | 0 | 1 | 0 | 0 | 0 | 0.243 | PASS
native_render_environment::portal_water_retention_and_invalid_visual_domains_are_explicit | 0 | 1 | 0 | 0 | 0 | 5.294 | PASS
native_render_environment::repeated_time_receipts_keep_pixels_current_while_blind_weather_changes_do_not | 0 | 1 | 0 | 0 | 0 | 0.397 | PASS
native_render_environment::retired_owners_and_different_maps_cannot_bind_or_reuse_a_render | 0 | 1 | 0 | 0 | 0 | 0.276 | PASS
native_render_environment::water_phase_is_a_frame_animation_sample_and_does_not_starve_environment_publication | 0 | 1 | 0 | 0 | 0 | 0.313 | PASS
native_room_departures::recorded_simultaneous_departures_accept_both_network_orderings | 0 | 1 | 0 | 0 | 0 | 5.330 | PASS
native_room_departures::retired_notifications_cannot_authorize_unknown_or_rejoined_room_changes | 0 | 1 | 0 | 0 | 0 | 0.206 | PASS
native_room_mode::formal_room_modes_maps_summary_and_configuration_reject_truncation_and_tails | 0 | 1 | 0 | 0 | 0 | 5.632 | PASS
native_room_mode::updates_preserve_members_and_reject_other_rooms_without_retaining_passwords | 0 | 1 | 0 | 0 | 0 | 0.279 | PASS
native_selector_only::counter72_actual_s39_acks_cast_without_s37_or_local_resource_damage | 0 | 1 | 0 | 0 | 0 | 5.431 | PASS
native_selector_only::counter72_malformed_capture_is_atomic_and_unknown_profiles_stay_quarantined | 0 | 1 | 0 | 0 | 0 | 0.286 | PASS
native_selector_only::nine_server_selectors_ignore_hover_and_never_spend_local_authority | 0 | 1 | 0 | 0 | 0 | 0.203 | PASS
native_selector_only::selector_learning_death_and_mana_guards_do_not_forge_requests | 0 | 1 | 0 | 0 | 0 | 0.209 | PASS
native_services::direct_room_and_vs_directory_preserve_separate_unsigned_fields | 0 | 1 | 0 | 0 | 0 | 4.777 | PASS
native_services::direct_room_join_keeps_mode_six_and_map_without_inventing_channel_membership | 0 | 1 | 0 | 0 | 0 | 0.218 | PASS
native_terrain_cache::cache_matches_direct_view_across_hits_edges_rebuilds_and_clear | 0 | 1 | 0 | 0 | 0 | 4.444 | PASS
native_terrain_cache::cancelled_or_failed_rebuild_never_publishes_partial_terrain | 0 | 1 | 0 | 0 | 0 | 0.275 | PASS
native_terrain_cache::crop_preserves_stride_edges_and_rejects_overflow | 0 | 1 | 0 | 0 | 0 | 0.237 | PASS
native_water_clock::all_15_original_queue_scenarios_match_stable_order_cancel_map_filter_and_late_dispatch | 0 | 1 | 0 | 0 | 0 | 5.475 | PASS
native_water_clock::all_8656_original_phase_rows_match_native_counter_wrap_and_fresh_deadline | 0 | 1 | 0 | 0 | 0 | 0.340 | PASS
native_water_clock::all_9668_original_scroll_rows_match_even_while_water_is_disabled | 0 | 1 | 0 | 0 | 0 | 0.234 | PASS
native_water_clock::explicit_scene_retirement_uses_first_seven_keep_entries_and_cancels_water_and_aux | 0 | 1 | 0 | 0 | 0 | 0.221 | PASS
native_water_clock::game_retirement_cancels_old_callbacks_preserves_visual_values_and_rejects_old_epochs | 0 | 1 | 0 | 0 | 0 | 0.219 | PASS
native_water_clock::invalid_domains_and_wrap_storm_fail_without_partially_committing_state | 0 | 1 | 0 | 0 | 0 | 0.274 | PASS
native_water_clock::shared_camera_history_accumulates_dry_motion_once_and_ignores_map_or_correction_jumps | 0 | 1 | 0 | 0 | 0 | 0.230 | PASS
native_world_metadata::deployed_master_updates_only_existing_identity_and_retires_on_replacement | 0 | 1 | 0 | 0 | 0 | 5.044 | PASS
native_world_metadata::malformed_master_and_bounds_are_rejected_atomically | 0 | 1 | 0 | 0 | 0 | 0.295 | PASS
native_world_metadata::observer_bounds_are_half_open_map_scoped_and_do_not_restrict_players | 0 | 1 | 0 | 0 | 0 | 0.198 | PASS
rebuild_input::input_matches_letterboxed_rendering_at_varied_scales | 0 | 1 | 0 | 0 | 0 | 4.700 | PASS
rebuild_input::minimized_and_extreme_signed_coordinates_cannot_wrap_into_a_hit | 0 | 1 | 0 | 0 | 0 | 0.302 | PASS
Steps=80 | Final exit code=0
```

### PowerShell 7 Quick 總表原樣輸出

```text
LOCAL TEST SUITE | Mode=Quick | PowerShell=7.6.5
Step | Exit | passed | failed | ignored | running-0-tests | Seconds | Result
check | 0 | N/A | N/A | N/A | N/A | 0.748 | PASS
lib-debug | 0 | 1889 | 0 | 575 | 0 | 3.252 | PASS
Steps=2 | Final exit code=0
```

### 腳本失敗門檻與隔離驗證

使用 scratch 一次性驗收程式，兩種 PowerShell 都通過 15 個解析案例（含 0 tests、結果缺失／損壞、非零碼、failed、ignored-only、缺 running、多 harness、計數不符），並確認程序啟動失敗能記錄、stderr 可排空、子程序 LITO_* 清除及 TEMP／TMP 隔離、呼叫端環境不變。

另外在獨立 scratch fixture，以合成 Cargo 輸出實跑完整 runner：check 回 17、Debug 0 tests、Release 格式錯誤、ignored lib failed、最後整合項成功；兩種 shell 都跑完 5 步、標出 4 步失敗、最後結束碼 1。這些只驗證 runner，**不是 Rust 測試通過數**；實際專案驗收以以上四次真實 Cargo 結果為準。

驗收輔助腳本初次由 5.1 讀 UTF-8 無 BOM 的中文路徑時發生解析錯誤，已將 scratch 輔助腳本改為 UTF-8 BOM 後重跑通過；正式 runner 本身全 ASCII、無 PowerShell 7 專屬語法，雙版本均已實跑。此初次錯誤不算專案測試失敗或通過。

## 與 #53／#55／#56 比較

| 項目 | 歷史基準 | 本輪每次對應實跑 | 結論 |
|---|---|---|---|
| Debug lib（#53） | 1889 passed／0 failed／575 ignored | 1889／0／575 | 一致 |
| Release lib（#53） | 1889 passed／0 failed／575 ignored | 1889／0／575 | 一致 |
| ignored lib 白名單（#55） | 1／1 | 1／1，每項 running 1、passed 1、exit 0 | 一致 |
| 整合白名單（#56） | 76／76 | 76／76，每項 running 1、passed 1、exit 0 | 一致 |

ignored 575 是每次基線 lib 測試略過的數量，不能算通過；完整版另精確執行其中 1 項，剩餘 574 項 ignored lib 仍待驗證。不同 profile／shell 的重複執行也不當作不同測試案例累加。

## 使用者日常直接複製

在本專案根目錄執行下列兩行即可，兩行均已在上述 worktree 實際跑過，最終結束碼均為 0，不依賴 pwsh 的 PATH：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具\run-local-test-suite.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具\run-local-test-suite.ps1 -Quick
```

完整版預期輸出樣子如下（本輪 5.1 實際總表節錄；耗時每次不同；所有 80 列皆須 PASS，最後碼才為 0）：

```text
LOCAL TEST SUITE | Mode=Full | PowerShell=5.1.19041.6456
Step | Exit | passed | failed | ignored | running-0-tests | Seconds | Result
check | 0 | N/A | N/A | N/A | N/A | 4.622 | PASS
lib-debug | 0 | 1889 | 0 | 575 | 0 | 6.457 | PASS
lib-release | 0 | 1889 | 0 | 575 | 0 | 6.342 | PASS
...（此處節錄；完整版有 80 個步驟）
Steps=80 | Final exit code=0
```

Quick 預期只有兩列；以下是本輪 5.1 實際原樣總表，耗時可能不同：

```text
LOCAL TEST SUITE | Mode=Quick | PowerShell=5.1.19041.6456
Step | Exit | passed | failed | ignored | running-0-tests | Seconds | Result
check | 0 | N/A | N/A | N/A | N/A | 99.968 | PASS
lib-debug | 0 | 1889 | 0 | 575 | 0 | 75.008 | PASS
Steps=2 | Final exit code=0
```

執行時也會印出每步完整 Cargo 命令及 log 路徑；最後印 `Summary: <本次 summary.txt 絕對路徑>`。可讀取 `$LASTEXITCODE` 判斷整體成功／失敗；任一步失敗時應查同目錄的編號 log，不得把其他 PASS 列解讀為全套通過。這兩行驗證的是此 worktree 的當版追蹤設定；沒有複製或驗證原始目錄未追蹤的本機 Cargo 設定。

## 未驗證什麼

- 白名單以外的 ignored lib 574 項及整合入口 481 項維持待驗證，沿用 #54 排除範圍；沒有執行全部 ignored／全部整合測試。
- 沒有連正式服務或 DB，沒有啟動 GUI、遊戲、Java harness，未啟動／附加／停止既有遊戲程序。僅白名單中既有自有 loopback 測試使用其自建 listener。
- 完整 M1／G06、原生 GUI、資源完整性與線上功能仍待各自任務驗收；本報告不推論它們已通過。沒有未經驗證的推測通過項。

## 產出路徑

相對合併基準 `fde15d48` 的追蹤交付僅五檔（PR 使用 merge-base 比對）：

- `維護工具/run-local-test-suite.ps1`
- `維護工具/test-whitelist/lib.txt`
- `維護工具/test-whitelist/tests.tsv`
- `docs/reports/issue-101.md`
- `docs/DEMO.md`（只改「怎麼跑測試」一節）

不進 git 的證據：上表四組 `target/current/verification/local-test-suite/<stamp>/summary.txt` 及逐步 log；`scratch/issue-101/` 的環境紀錄、四次最終 console log／exit.txt、runner-qa-ps51.log、runner-qa-ps7.log、failure-suite-qa.log 與一次性輔助腳本。同步前四次的 console／exit 證據保留為 `initial-*`，原始 summary 亦保留；不與最終四次重複計算通過案例。可攜 pwsh 位於 `scratch/tools/pwsh/`。建置產物都在本 worktree `LiTo381-Modern/target/current/`。

## 給下一棒的提示

交 Claude 審查 task/101 的白名單一致性、5.1 相容性、失敗門檻、DEMO 接線及本報告，再由使用者合併；本機實作與驗收沒有剩餘阻擋。調度器合併後更新 docs/REPORT.md 索引。T013 的 DEMO 測試入口已更新。新增白名單項目仍須先做適用的安全審查，不得從未執行清單自動加入。
