# Issue #56：T010-C 白名單整合測試

## 結論

**已驗證通過 76／76；實際測試失敗 0；執行錯誤／未實際執行 0；白名單尚未啟動 0。** 76 項各單獨執行一次，全部符合 `running 1 test`、1 passed／0 failed／0 ignored。
白名單以外 481 項維持排除待驗證；此結果不是全部 557 項整合測試、JVM harness、M1、G06 或 GUI 通過。

## 做了什麼／來源與隔離

- 開工核對 #56：OPEN、runner:dot、task:running、attempts:1，只有調度留言，未有 RESULT、task/56 分支或同任務 PR。本機對話清單中的「執行安全白名單測試」實為已完成 #55；本輪使用獨立本機 Codex 對話。
- 前置 #55 已 CLOSED、task:done、review:pass、gate:pass；最新 main 已包含其報告。本次不重跑 #55。
- 固定基準 `85cc83d9eefabbc39179c963a62102f6c7fb5fdf`；與 #54 分類基準 `bb96171d2938437dd224af73da466ab8bbab8f73` 的整個 LiTo381-Modern git diff 為空。
- 專屬 worktree：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-56`；分支 `task/56`。原始工作區 main 未切換或修改。
- 已讀原工作區及固定基準 AGENTS.md、TASKS.md T010、Cargo.toml／build.rs。依直接指示未讀寫 docs/REPORT.md、config/server.properties、正式 BIN、封存；不操作既有遊戲程序、不啟動正式伺服器、不連 2000／3306，不建立資料庫。
- 追蹤變更只有本報告；一次性腳本、白名單、逐項原始 log 與 CSV 在專屬 scratch，建置在 LiTo381-Modern/target/current。
- 實際工具權限為 workspace-write／auto_review；工具無法設定或證明完全存取。一般 git 網路讀取失敗後，以標準 require_escalated 核准的唯讀 fetch／ls-remote 取得最新來源；未繞過核准、改 ACL、safe.directory 或防火牆。測試由一般沙箱執行。

## JAR 與環境

正式 JAR 只做 SHA256 唯讀核對：

- 路徑：`D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar`
- 實算：`E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`，符合指定身分。
- `LITO_MODERN_TEST_JAVA=C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe`；該 JDK release 檔為 Microsoft Java 17.0.17，且 executable 存在。未啟動 Java，因此沒有宣稱 JVM runtime 實測。
- `LITO_MODERN_TEST_JAVA_CLASSPATH` 為上述單一 JAR；`LITO_MODERN_TEST_SERVER_ROOT` 為其父目錄。76 項白名單不讀這三個變數，也不啟動 JAR／JVM／server。設定變數不代表允許讀取 server.properties 或其他正式資源。
- Cargo 子程序 TEMP／TMP 指向本 worktree 的 scratch/issue-56/tmp；只在該程序清除繼承的 LITO_* 再設定上述三項，未改使用者或系統環境。CARGO_HOME／RUSTUP_HOME 沿用 D:\dev-cache 設定；未複製其他任務的 Cargo 設定或建置產物。

## 逐項資源與連線核對

重新閱讀 22 個 target 的白名單測試本體與 helper，沿用來源完全相同的 #54 安全分類；另核對 Controller.start／Session.connect／probe_loopback、Schedule、TerrainCache、Source.parse、文字解碼等重要邊界。72 項沒有實際 socket；4 項僅當次持有的 loopback listener。
以下 fixture 均位於本 repo 的 `LiTo381-Modern/tests/fixtures/`，以 include_bytes! 在編譯時讀取，執行時使用內嵌 bytes。表中為該 target 白名單可能涉及的 fixture 聯集；每項確切入口與結果見下一表。不讀取 fixture 中描述的外部來源路徑；檔名含 .bin 的內嵌捕捉不是正式遊戲 BIN。所有白名單測試均無檔案寫入；Cargo／收據寫入範圍另如上列。

| target | 白名單數 | 資源（fixture） | 連線／修改邊界 |
|---|---:|---|---|
| `native_ability_buffs` | 5 | `native-ability-codecs.json`、`native-world-current.json` | World／技能請求／選取只解析與修改記憶體；request 回傳 bytes，沒有送出封包。 |
| `native_ability_world` | 2 | `native-world-current.json` | 內嵌能力封包由 World.accept 解析；更新／退役狀態只在記憶體。 |
| `native_attributes` | 3 | `native-attributes-current.json`、`native-world-current.json` | 能力／經驗／護甲／魔法欄位的記憶體解析與原子拒絕。 |
| `native_doors` | 4 | `native-doors-current.json`、`native-world-current.json` | 門封包、Choice／Pending／Instant 判斷；不建立遊戲或送出門操作。 |
| `native_early_roster` | 3 | `native-early-roster.json` | EarlyRoster Vec、256 筆容量與 epoch 判斷；無 socket。 |
| `native_game_timers` | 2 | 無；人工資料／記憶體 | 僅兩個 Schedule 狀態測試；人工 Instant 加減，沒有 300 秒等待、登入或 Save 寫入。其他同檔測試未執行。 |
| `native_green_clock` | 8 | `native-green-clock-original.json`、`native-world-current.json` | Cell 注入時鐘、Core／Owned 排程狀態、渲染綁定；內嵌 SHA fixture 與記憶體斷言。 |
| `native_light_color` | 5 | `native-light-color-original.bin` | 內嵌色彩捕捉、Mask／Channels／Remap 與像素切片；不開視窗、不讀正式 BIN。 |
| `native_light_lifecycle` | 1 | 無；人工資料／記憶體 | 人工 put/action/resurrect bytes → World；不接觸角色服務。 |
| `native_light_sources` | 2 | `native-light-source.lmp`、`native-light-sources-original.json` | Source::parse 解析內嵌 LMP（不是 Source::load）；Mask／Remap 只寫記憶體。 |
| `native_network` | 8 | `formal-cipher-0.6.42.json`、`native-version-current.json` | 四例持有當次 TcpListener(127.0.0.1:0) 並以 local_addr 連入；三例純 codec；一例 invalid limits／非 loopback 在 worker 前拒絕。 |
| `native_points` | 1 | `native-points-current.json` | S_DB 字面是封包 opcode，解析內嵌租借點數；沒有 SQL／DB 呼叫。 |
| `native_portal` | 3 | `native-portal-current.json`、`native-world-current.json` | World 記憶體轉場／取消，沒有 endpoint handoff 或連線。 |
| `native_render_environment` | 4 | 無；人工資料／記憶體 | World／State／Bound／Key 純狀態與排程 callback；沒有 renderer 視窗或資源 loader。 |
| `native_room_departures` | 2 | `native-room-current.json` | 內嵌 S_D0 與 Room／RetiredDepartures；沒有實際進退房。 |
| `native_room_mode` | 2 | `native-room-current.json`、`native-room-mode-current.json` | Room mode／Info 封包解析；記憶體中更新欄位。 |
| `native_selector_only` | 4 | `native-selector-caster3-s39-current.json` | 內嵌選技與 S39 bytes；skills.request 回傳 bytes，沒有傳輸、角色或資源實寫。 |
| `native_services` | 2 | `native-services-current.json` | SocketAddr 僅資料解析／比對；127.0.0.2:65535、127.0.0.1:1 沒有被連線。 |
| `native_terrain_cache` | 3 | 無；人工資料／記憶體 | render_with callback 明確是測試的 pattern／錯誤閉包；Canvas 僅 Vec 像素與 AtomicBool。 |
| `native_water_clock` | 7 | `native-water-clock-original.bin` | 內嵌捕捉、Cell 時鐘、Core／History／Owned；只操作記憶體排程與相位。 |
| `native_world_metadata` | 3 | `native-world-current.json`、`native-world-metadata-current.json` | master／bounds bytes → World，包含 CP950 文字轉換，無外部連線或物件載入。 |
| `rebuild_input` | 2 | 無；人工資料／記憶體 | Viewport 幾何運算，不建立 HWND。 |

四個 socket 案例均先 bind `127.0.0.1:0`，listener 被移入 peer thread 並持有至 accept；client 使用該 listener.local_addr，Session.connect 核對實際 peer 等於指定地址。取消／shutdown 只關閉測試自身 socket，join 只回收自身 thread。`invalid_limits_and_non_loopback_cli_probe_do_not_start_worker` 的 127.0.0.1:1 在 Limits.validate（max_frame=31）先拒絕；192.0.2.1:1 在 probe_loopback 的 loopback 檢查先拒絕，未連線。
文字解析使用 Windows MultiByteToWideChar／CP950，是記憶體文字轉換，未開 HWND。Source.parse 與 TerrainCache 的受控 callback 不會呼叫正式資源 loader。沒有以未命中關鍵字取代 #54 的呼叫鏈審查。

## 執行與逐項結果

白名單由合併後 #54 報告原文重建（UTF-8、LF、實際 Tab），SHA256 為 `3E95BBE0A3654DA26405FBC9AAFF82C57D4D8E7829859E17FB98424EAA3761B1`。76 項 ignored 欄皆 no、features 欄皆 -，因此沒有啟用額外 feature；仍依 Issue 使用 --include-ignored。

工作目錄是本 worktree 的 LiTo381-Modern，每項完整命令在 execution-summary.csv：

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test <target> --no-fail-fast -- --include-ignored --exact <全名>
```

腳本拒絕重用既有 CSV／log，逐項保存輸出及 exit code，核對 running 1 test 恰出現一次、passed+failed=1、ignored=0、正確測試全名。首個失敗或執行未完成即停止，不修改 src／tests、不重試。Cargo 使用標準 test harness；編譯會執行 build scripts 和工具，遊戲 bin 即使被 Cargo 編譯也沒有被啟動。

| # | target／測試全名 | 來源行／#54 審查理由 | 實際連線 | 結果（passed/failed/ignored） | 原始輸出 |
|---:|---|---|---|---|---|
| 1 | `native_ability_buffs::ability_buffs_keep_native_screen_asymmetric_edges` | `LiTo381-Modern/tests/native_ability_buffs.rs:56`；內嵌封包向量／World 與技能選擇狀態；只建立請求 bytes，未送出封包 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/01-native_ability_buffs.log` |
| 2 | `native_ability_buffs::ability_buffs_reject_unlearned_dead_missing_and_outside_screen_targets` | `LiTo381-Modern/tests/native_ability_buffs.rs:34`；內嵌封包向量／World 與技能選擇狀態；只建立請求 bytes，未送出封包 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/02-native_ability_buffs.log` |
| 3 | `native_ability_buffs::ability_choice_preserves_object_and_fences_reused_actor_and_world` | `LiTo381-Modern/tests/native_ability_buffs.rs:46`；內嵌封包向量／World 與技能選擇狀態；只建立請求 bytes，未送出封包 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/03-native_ability_buffs.log` |
| 4 | `native_ability_buffs::both_ability_buffs_use_explicit_self_or_selected_object_without_authority_writes` | `LiTo381-Modern/tests/native_ability_buffs.rs:26`；內嵌封包向量／World 與技能選擇狀態；只建立請求 bytes，未送出封包 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/04-native_ability_buffs.log` |
| 5 | `native_ability_buffs::current_formal_ability_codec_vectors_and_short_or_extended_packets` | `LiTo381-Modern/tests/native_ability_buffs.rs:16`；內嵌封包向量／World 與技能選擇狀態；只建立請求 bytes，未送出封包 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/05-native_ability_buffs.log` |
| 6 | `native_ability_world::ability_receipts_retire_on_same_map_new_world_and_new_status_owner` | `LiTo381-Modern/tests/native_ability_world.rs:19`；內嵌封包向量，僅 World 記憶體狀態變化 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/06-native_ability_world.log` |
| 7 | `native_ability_world::original_ability_packets_update_only_authoritative_fields_and_reject_partial_input_atomically` | `LiTo381-Modern/tests/native_ability_world.rs:7`；內嵌封包向量，僅 World 記憶體狀態變化 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/07-native_ability_world.log` |
| 8 | `native_attributes::formal_schema_zero_preserves_attribute_order_signed_modifiers_and_unsigned_experience` | `LiTo381-Modern/tests/native_attributes.rs:7`；內嵌 attribute/status 向量，純解析與 World 狀態斷言 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/08-native_attributes.log` |
| 9 | `native_attributes::identity_and_scene_changes_retire_modifiers_and_unowned_updates_never_create_players` | `LiTo381-Modern/tests/native_attributes.rs:25`；內嵌 attribute/status 向量，純解析與 World 狀態斷言 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/09-native_attributes.log` |
| 10 | `native_attributes::malformed_or_extended_attributes_do_not_partially_change_current_authority` | `LiTo381-Modern/tests/native_attributes.rs:18`；內嵌 attribute/status 向量，純解析與 World 狀態斷言 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/10-native_attributes.log` |
| 11 | `native_doors::authoritative_toggle_requires_both_packets_and_holds_the_new_pose` | `LiTo381-Modern/tests/native_doors.rs:38`；內嵌門封包與注入時間；請求 bytes／Pending 狀態，無傳送 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/11-native_doors.log` |
| 12 | `native_doors::deployed_attribute_and_unsigned_open_vectors_match_native_codec` | `LiTo381-Modern/tests/native_doors.rs:12`；內嵌門封包與注入時間；請求 bytes／Pending 狀態，無傳送 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/12-native_doors.log` |
| 13 | `native_doors::door_identity_removal_replacement_range_and_world_are_retired` | `LiTo381-Modern/tests/native_doors.rs:53`；內嵌門封包與注入時間；請求 bytes／Pending 狀態，無傳送 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/13-native_doors.log` |
| 14 | `native_doors::malformed_attribute_never_partially_changes_world` | `LiTo381-Modern/tests/native_doors.rs:30`；內嵌門封包與注入時間；請求 bytes／Pending 狀態，無傳送 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/14-native_doors.log` |
| 15 | `native_early_roster::captured_early_members_wait_for_matching_acceptance_and_latest_snapshot_wins` | `LiTo381-Modern/tests/native_early_roster.rs:4`；內嵌房間封包，純 EarlyRoster 集合操作 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/15-native_early_roster.log` |
| 16 | `native_early_roster::pending_roster_cannot_cross_connection_room_rejection_or_new_request` | `LiTo381-Modern/tests/native_early_roster.rs:6`；內嵌房間封包，純 EarlyRoster 集合操作 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/16-native_early_roster.log` |
| 17 | `native_early_roster::pending_roster_preserves_wire_order_and_existing_256_member_bound` | `LiTo381-Modern/tests/native_early_roster.rs:8`；內嵌房間封包，純 EarlyRoster 集合操作 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/17-native_early_roster.log` |
| 18 | `native_game_timers::delayed_dispatch_never_catches_up_and_stop_cannot_resurrect_the_owner` | `LiTo381-Modern/tests/native_game_timers.rs:45`；純 Schedule 記憶體狀態；Instant 加法模擬經過時間，無 socket 或 300 秒實際等待 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/18-native_game_timers.log` |
| 19 | `native_game_timers::original_immediate_alive_and_300s_save_are_rearmed_from_actual_completion` | `LiTo381-Modern/tests/native_game_timers.rs:16`；純 Schedule 記憶體狀態；Instant 加法模擬經過時間，無 socket 或 300 秒實際等待 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/19-native_game_timers.log` |
| 20 | `native_green_clock::actor_green_style_consumes_delivered_native_counter_not_elapsed_nanoseconds` | `LiTo381-Modern/tests/native_green_clock.rs:41`；內嵌 JSON／Cell 假時鐘；callback queue 與狀態檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/20-native_green_clock.log` |
| 21 | `native_green_clock::all_16_original_queue_snapshots_match_late_duplicate_cancel_map_and_scene_retirement` | `LiTo381-Modern/tests/native_green_clock.rs:64`；內嵌 JSON／Cell 假時鐘；callback queue 與狀態檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/21-native_green_clock.log` |
| 22 | `native_green_clock::all_532_original_counter_wrap_fresh_deadline_and_shader_steps_match` | `LiTo381-Modern/tests/native_green_clock.rs:51`；內嵌 JSON／Cell 假時鐘；callback queue 與狀態檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/22-native_green_clock.log` |
| 23 | `native_green_clock::host_budget_refusal_and_unsigned_wrap_storm_do_not_publish_partial_counter` | `LiTo381-Modern/tests/native_green_clock.rs:125`；內嵌 JSON／Cell 假時鐘；callback queue 與狀態檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/23-native_green_clock.log` |
| 24 | `native_green_clock::immutable_render_binding_captures_counter_without_turning_phase_into_a_resource_key` | `LiTo381-Modern/tests/native_green_clock.rs:134`；內嵌 JSON／Cell 假時鐘；callback queue 與狀態檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/24-native_green_clock.log` |
| 25 | `native_green_clock::older_or_retired_generation_cannot_restart_callbacks_or_consume_a_schedule_clock` | `LiTo381-Modern/tests/native_green_clock.rs:104`；內嵌 JSON／Cell 假時鐘；callback queue 與狀態檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/25-native_green_clock.log` |
| 26 | `native_green_clock::owner_and_scene_fences_preserve_counter_but_retire_old_callbacks` | `LiTo381-Modern/tests/native_green_clock.rs:90`；內嵌 JSON／Cell 假時鐘；callback queue 與狀態檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/26-native_green_clock.log` |
| 27 | `native_green_clock::water_and_green_share_stable_dispatch_order_and_distinct_fresh_deadlines` | `LiTo381-Modern/tests/native_green_clock.rs:116`；內嵌 JSON／Cell 假時鐘；callback queue 與狀態檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/27-native_green_clock.log` |
| 28 | `native_light_color::all_original_selector_calls_preserve_level_only_cache_semantics` | `LiTo381-Modern/tests/native_light_color.rs:28`；內嵌 capture 向量；Channels／Mask 記憶體像素運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/28-native_light_color.log` |
| 29 | `native_light_color::all_retained_native_kernel_samples_match_with_pitch_guards_and_byte_gates` | `LiTo381-Modern/tests/native_light_color.rs:78`；內嵌 capture 向量；Channels／Mask 記憶體像素運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/29-native_light_color.log` |
| 30 | `native_light_color::every_generated_channel_entry_matches_captured_original_writer_output` | `LiTo381-Modern/tests/native_light_color.rs:43`；內嵌 capture 向量；Channels／Mask 記憶體像素運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/30-native_light_color.log` |
| 31 | `native_light_color::malformed_inputs_fail_before_any_surface_mutation` | `LiTo381-Modern/tests/native_light_color.rs:138`；內嵌 capture 向量；Channels／Mask 記憶體像素運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/31-native_light_color.log` |
| 32 | `native_light_color::rgba_adapter_uses_native_555_entries_preserves_alpha_and_identity_bytes` | `LiTo381-Modern/tests/native_light_color.rs:109`；內嵌 capture 向量；Channels／Mask 記憶體像素運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/32-native_light_color.log` |
| 33 | `native_light_lifecycle::existing_put_preserves_native_death_marker_while_resurrection_and_new_identity_clear_it` | `LiTo381-Modern/tests/native_light_lifecycle.rs:21`；自行建立封包並修改記憶體 World，無渲染視窗 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/33-native_light_lifecycle.log` |
| 34 | `native_light_sources::all_original_graph_masks_queries_remap_cache_and_blit_order_match` | `LiTo381-Modern/tests/native_light_sources.rs:20`；內嵌 LMP／JSON，Mask／Remap 與像素緩衝記憶體運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/34-native_light_sources.log` |
| 35 | `native_light_sources::owner_and_extent_failures_are_atomic_and_native_phase_bounds_are_explicit` | `LiTo381-Modern/tests/native_light_sources.rs:64`；內嵌 LMP／JSON，Mask／Remap 與像素緩衝記憶體運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/35-native_light_sources.log` |
| 36 | `native_network::accepted_connection_keeps_socket_until_explicit_cancel_then_reuses_epoch` | `LiTo381-Modern/tests/native_network.rs:173`；只連測試自行綁定且持有的 127.0.0.1:0 listener；固定合成封包、cancel/join 自有執行緒，無 Java／正式服務 | 自有 loopback | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/36-native_network.log` |
| 37 | `native_network::coalesced_post_version_packet_survives_peer_eof_and_owner_retirement` | `LiTo381-Modern/tests/native_network.rs:250`；只連測試自行綁定且持有的 127.0.0.1:0 listener；固定合成封包、cancel/join 自有執行緒，無 Java／正式服務 | 自有 loopback | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/37-native_network.log` |
| 38 | `native_network::command_batch_is_admitted_atomically_and_written_as_two_ordered_frames` | `LiTo381-Modern/tests/native_network.rs:377`；只連測試自行綁定且持有的 127.0.0.1:0 listener；固定合成封包、cancel/join 自有執行緒，無 Java／正式服務 | 自有 loopback | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/38-native_network.log` |
| 39 | `native_network::deployed_jar_version_request_and_replies_match_typed_native_contract` | `LiTo381-Modern/tests/native_network.rs:24`；內嵌 codec/version/cipher 向量與記憶體 FrameDecoder，無 socket／檔案寫入 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/39-native_network.log` |
| 40 | `native_network::fragmented_and_coalesced_frames_preserve_full_u16_payload_and_eof_state` | `LiTo381-Modern/tests/native_network.rs:56`；內嵌 codec/version/cipher 向量與記憶體 FrameDecoder，無 socket／檔案寫入 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/40-native_network.log` |
| 41 | `native_network::full_event_queue_preserves_burst_and_allows_cancel_without_consumer` | `LiTo381-Modern/tests/native_network.rs:308`；只連測試自行綁定且持有的 127.0.0.1:0 listener；固定合成封包、cancel/join 自有執行緒，無 Java／正式服務 | 自有 loopback | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/41-native_network.log` |
| 42 | `native_network::invalid_limits_and_non_loopback_cli_probe_do_not_start_worker` | `LiTo381-Modern/tests/native_network.rs:231`；Limits.validate 在 worker 前拒絕；probe_loopback 在 start 前拒絕非 loopback（src/modern_lito/native_connection.rs:505、672），不建立連線 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/42-native_network.log` |
| 43 | `native_network::shared_cipher_retains_unsigned_counter_carry_for_large_packets` | `LiTo381-Modern/tests/native_network.rs:90`；內嵌 codec/version/cipher 向量與記憶體 FrameDecoder，無 socket／檔案寫入 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/43-native_network.log` |
| 44 | `native_points::captured_rental_points_are_not_a_battle_result` | `LiTo381-Modern/tests/native_points.rs:3`；內嵌 fixture 與 pure parse；S_DB 是 opcode，並非資料庫連線 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/44-native_points.log` |
| 45 | `native_portal::different_map_portal_rejects_wrong_rebuild_and_cancellation_retires_pending` | `LiTo381-Modern/tests/native_portal.rs:21`；內嵌封包，純 portal/World 狀態機，無 socket | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/45-native_portal.log` |
| 46 | `native_portal::formal_portal_payload_is_exact_and_malformed_packets_do_not_retire_world` | `LiTo381-Modern/tests/native_portal.rs:7`；內嵌封包，純 portal/World 狀態機，無 socket | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/46-native_portal.log` |
| 47 | `native_portal::same_map_portal_retires_old_scene_and_waits_for_matching_received_self` | `LiTo381-Modern/tests/native_portal.rs:13`；內嵌封包，純 portal/World 狀態機，無 socket | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/47-native_portal.log` |
| 48 | `native_render_environment::portal_water_retention_and_invalid_visual_domains_are_explicit` | `LiTo381-Modern/tests/native_render_environment.rs:60`；手建 World／State，注入 callback 時間，純 Bound/Key 檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/48-native_render_environment.log` |
| 49 | `native_render_environment::repeated_time_receipts_keep_pixels_current_while_blind_weather_changes_do_not` | `LiTo381-Modern/tests/native_render_environment.rs:42`；手建 World／State，注入 callback 時間，純 Bound/Key 檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/49-native_render_environment.log` |
| 50 | `native_render_environment::retired_owners_and_different_maps_cannot_bind_or_reuse_a_render` | `LiTo381-Modern/tests/native_render_environment.rs:26`；手建 World／State，注入 callback 時間，純 Bound/Key 檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/50-native_render_environment.log` |
| 51 | `native_render_environment::water_phase_is_a_frame_animation_sample_and_does_not_starve_environment_publication` | `LiTo381-Modern/tests/native_render_environment.rs:13`；手建 World／State，注入 callback 時間，純 Bound/Key 檢查 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/51-native_render_environment.log` |
| 52 | `native_room_departures::recorded_simultaneous_departures_accept_both_network_orderings` | `LiTo381-Modern/tests/native_room_departures.rs:18`；內嵌封包與固定字串，純 room/tombstone 狀態 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/52-native_room_departures.log` |
| 53 | `native_room_departures::retired_notifications_cannot_authorize_unknown_or_rejoined_room_changes` | `LiTo381-Modern/tests/native_room_departures.rs:33`；內嵌封包與固定字串，純 room/tombstone 狀態 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/53-native_room_departures.log` |
| 54 | `native_room_mode::formal_room_modes_maps_summary_and_configuration_reject_truncation_and_tails` | `LiTo381-Modern/tests/native_room_mode.rs:5`；內嵌 room 向量，parse/apply_info/mode_step 僅操作記憶體 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/54-native_room_mode.log` |
| 55 | `native_room_mode::updates_preserve_members_and_reject_other_rooms_without_retaining_passwords` | `LiTo381-Modern/tests/native_room_mode.rs:20`；內嵌 room 向量，parse/apply_info/mode_step 僅操作記憶體 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/55-native_room_mode.log` |
| 56 | `native_selector_only::counter72_actual_s39_acks_cast_without_s37_or_local_resource_damage` | `LiTo381-Modern/tests/native_selector_only.rs:24`；內嵌向量；技能 selector 與 World 狀態，無傳送 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/56-native_selector_only.log` |
| 57 | `native_selector_only::counter72_malformed_capture_is_atomic_and_unknown_profiles_stay_quarantined` | `LiTo381-Modern/tests/native_selector_only.rs:31`；內嵌向量；技能 selector 與 World 狀態，無傳送 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/57-native_selector_only.log` |
| 58 | `native_selector_only::nine_server_selectors_ignore_hover_and_never_spend_local_authority` | `LiTo381-Modern/tests/native_selector_only.rs:8`；內嵌向量；技能 selector 與 World 狀態，無傳送 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/58-native_selector_only.log` |
| 59 | `native_selector_only::selector_learning_death_and_mana_guards_do_not_forge_requests` | `LiTo381-Modern/tests/native_selector_only.rs:16`；內嵌向量；技能 selector 與 World 狀態，無傳送 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/59-native_selector_only.log` |
| 60 | `native_services::direct_room_and_vs_directory_preserve_separate_unsigned_fields` | `LiTo381-Modern/tests/native_services.rs:4`；內嵌向量；只解析 SocketAddr 與房間資料，未建立 socket | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/60-native_services.log` |
| 61 | `native_services::direct_room_join_keeps_mode_six_and_map_without_inventing_channel_membership` | `LiTo381-Modern/tests/native_services.rs:16`；內嵌向量；只解析 SocketAddr 與房間資料，未建立 socket | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/61-native_services.log` |
| 62 | `native_terrain_cache::cache_matches_direct_view_across_hits_edges_rebuilds_and_clear` | `LiTo381-Modern/tests/native_terrain_cache.rs:22`；手建 Canvas／注入 pattern callback；有界 CPU 記憶體快取 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/62-native_terrain_cache.log` |
| 63 | `native_terrain_cache::cancelled_or_failed_rebuild_never_publishes_partial_terrain` | `LiTo381-Modern/tests/native_terrain_cache.rs:33`；手建 Canvas／注入 pattern callback；有界 CPU 記憶體快取 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/63-native_terrain_cache.log` |
| 64 | `native_terrain_cache::crop_preserves_stride_edges_and_rejects_overflow` | `LiTo381-Modern/tests/native_terrain_cache.rs:13`；手建 Canvas／注入 pattern callback；有界 CPU 記憶體快取 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/64-native_terrain_cache.log` |
| 65 | `native_water_clock::all_15_original_queue_scenarios_match_stable_order_cancel_map_filter_and_late_dispatch` | `LiTo381-Modern/tests/native_water_clock.rs:65`；內嵌向量／Cell 假時鐘；queue、History 與整數運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/65-native_water_clock.log` |
| 66 | `native_water_clock::all_8656_original_phase_rows_match_native_counter_wrap_and_fresh_deadline` | `LiTo381-Modern/tests/native_water_clock.rs:18`；內嵌向量／Cell 假時鐘；queue、History 與整數運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/66-native_water_clock.log` |
| 67 | `native_water_clock::all_9668_original_scroll_rows_match_even_while_water_is_disabled` | `LiTo381-Modern/tests/native_water_clock.rs:30`；內嵌向量／Cell 假時鐘；queue、History 與整數運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/67-native_water_clock.log` |
| 68 | `native_water_clock::explicit_scene_retirement_uses_first_seven_keep_entries_and_cancels_water_and_aux` | `LiTo381-Modern/tests/native_water_clock.rs:130`；內嵌向量／Cell 假時鐘；queue、History 與整數運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/68-native_water_clock.log` |
| 69 | `native_water_clock::game_retirement_cancels_old_callbacks_preserves_visual_values_and_rejects_old_epochs` | `LiTo381-Modern/tests/native_water_clock.rs:94`；內嵌向量／Cell 假時鐘；queue、History 與整數運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/69-native_water_clock.log` |
| 70 | `native_water_clock::invalid_domains_and_wrap_storm_fail_without_partially_committing_state` | `LiTo381-Modern/tests/native_water_clock.rs:121`；內嵌向量／Cell 假時鐘；queue、History 與整數運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/70-native_water_clock.log` |
| 71 | `native_water_clock::shared_camera_history_accumulates_dry_motion_once_and_ignores_map_or_correction_jumps` | `LiTo381-Modern/tests/native_water_clock.rs:108`；內嵌向量／Cell 假時鐘；queue、History 與整數運算 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/71-native_water_clock.log` |
| 72 | `native_world_metadata::deployed_master_updates_only_existing_identity_and_retires_on_replacement` | `LiTo381-Modern/tests/native_world_metadata.rs:8`；內嵌封包；master/bounds 解析與 World 狀態 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/72-native_world_metadata.log` |
| 73 | `native_world_metadata::malformed_master_and_bounds_are_rejected_atomically` | `LiTo381-Modern/tests/native_world_metadata.rs:27`；內嵌封包；master/bounds 解析與 World 狀態 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/73-native_world_metadata.log` |
| 74 | `native_world_metadata::observer_bounds_are_half_open_map_scoped_and_do_not_restrict_players` | `LiTo381-Modern/tests/native_world_metadata.rs:19`；內嵌封包；master/bounds 解析與 World 狀態 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/74-native_world_metadata.log` |
| 75 | `rebuild_input::input_matches_letterboxed_rendering_at_varied_scales` | `LiTo381-Modern/tests/rebuild_input.rs:4`；Viewport 幾何整數運算，未建立視窗 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/75-rebuild_input.log` |
| 76 | `rebuild_input::minimized_and_extreme_signed_coordinates_cannot_wrap_into_a_hit` | `LiTo381-Modern/tests/rebuild_input.rs:18`；Viewport 幾何整數運算，未建立視窗 | 無 | 已驗證通過（1/0/0；exit 0） | `scratch/issue-56/76-rebuild_input.log` |

## 統計與未驗證項目

- 白名單：通過 76、測試失敗 0、執行錯誤／未實際執行 0、尚未啟動 0；不把略過、ignored 或未執行算成通過。
- 非白名單：GUI 1、DB 1、不明副作用 438、缺前置條件 41，合計 481；逐項名稱及理由沿用 docs/reports/issue-54.md。身份未知／需正式資源者未因 Issue #56 而放寬。
- native_teleport_controller 六例（teleport-wire-test）不在白名單；固定輸出檔所有權尚未核對，沒有啟用或執行。native_additional_finite_resources 的 ignored 例也不在白名單。
- native_network::stalled_seed_can_cancel_and_reconnect_without_stale_owner_or_thread 釋放 listener 後才連端點，維持排除；同檔其餘非白名單例未執行。
- JAR 實算 hash 是檔案身份證據，不是 JVM／真實伺服器 harness 通過。正式 DB／服務／GUI、完整 M1／G06 仍待驗證；無推測的通過數。

## 原始證據與下一棒

執行起訖 UTC：`2026-11-05T06:34:04.5105895Z` 至 `2026-11-05T06:39:42.8362654Z`；命令累計 337.528 秒（含首次編譯）。
原始 log 的逐檔 SHA256、完整命令、UTC 起訖、耗時、exit code 與測試計數在 scratch/issue-56/execution-summary.csv；上述逐項表可一一對照。原始輸出與腳本不進 git。

| 證據 | SHA256 |
|---|---|
| `scratch/issue-56/whitelist.tsv` | `3E95BBE0A3654DA26405FBC9AAFF82C57D4D8E7829859E17FB98424EAA3761B1` |
| `scratch/issue-56/jar-sha256.txt` | `93C4934ED9D8EA8551C7F593D8F27030AB232C76E0ACCDC032C6160EB6556DA0` |
| `scratch/issue-56/run-exact.ps1` | `01862074F1A4809979D00B625E29CE2FE4348CE5E872C0B8C777A9868F65E2A8` |
| `scratch/issue-56/execution-summary.csv` | `96F5CB4AD5D6A0EAD8964E51A8D264554DE7F9114CE9072E298D1BB3218EC994` |

交付追蹤檔只有 `docs/reports/issue-56.md`；分支 `task/56`，提交訊息以 task #56: 開頭。下一棒由 Claude／調度器審查本報告與原始收據，再裁定 #56 門檻；排除項須補安全／fixture 身份審查與適用授權，不得直接擴大測試。 本輪 76 項已執行完畢，未留下測試失敗阻塞。
