# Issue #57 — M1 gate、元件與 G06 靜態資源盤點

## 結論與驗證界線

四類定義均找到來源，以下逐名列出。**本報告只驗證文字／JSON 定義及部分文字檔 SHA，執行 gate、inventory、測試、DB、Java、遊戲與 GUI 的次數均為 0。** 找到原始 runner 不代表現在可以安全執行；後續 #58／G06 的執行門檻尚未解除。

- 報告基底：遠端 main `6174e4012ceb60099facff4d2b919669b4d2ca3d`，隔離 worktree `scratch/wt/task-57`、分支 `task/57`。根工作樹及 #53 的 worktree 不切換、不修改。
- 盤點對象：總報告 2026-10-25 的正常來源清單 SHA `3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628`、正式參考 JAR SHA `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。這是歷史來源／參考身分，**不是** Git main 的 commit SHA，也不是 main 已包含候選的證明。
- 依據最新 AGENTS.md、TASKS.md、docs/TESTING.md、docs/CANDIDATES.md：測試授權已恢復，但 #57 明確只盤點；44 檔仍凍結為唯讀參考，不套用、不 cherry-pick。遵守本次禁止接觸 docs/REPORT.md 的限制。
- 最新派工讀取仍為 `runner:dot`／`task:running`；開始時只有原派工留言，無 RESULT、無 task/57 遠端分支。發佈前另行重讀確認。
- 權限實際為 `workspace-write`／`auto_review`；本機讀取與報告寫入使用 `use_default`。預設網路失敗原文：`fatal: unable to access 'https://github.com/chaoren0418-arch/lito-modern.git/': Failed to connect to github.com:443 after 29 ms: Could not connect to server`。其後 `require_escalated` 的 git ls-remote／fetch 已核准成功，未修改 ACL／安全設定。
- 全文 `R` 指 `D:\lineage開發\3-lito開發\lito重建版`；相對路徑均相對 R，行號為原始文字檔的 1-based 行號。原資源只記錄來源，不讀正式 BIN、不複製、不解碼，不開封存或 ZIP。

## 來源索引與版本

| 編號 | 原始來源 | 用途 |
|---|---|---|
| S0 | `../LiTo381-總開發報告.md:9–19,29–33,47` | 當版 gate、候選、binding 與 640 次總綱 |
| G | `維護工具/candidates/native-E149-static2941-ready-20261024-210441-110315/contract.json` | 44 檔 ready contract；8 純＋4 原資源；本次文字 SHA256 `A5EAFAB92F673DEEE4D1E8EC4EEE39085B3B3C87C24C565DED2F0ACFBF831A97` |
| C | `維護工具/candidates/native-E149-component-matrix-20261023-231736-569424/plan.json` | baseline 23／candidate 28，精確名稱與預期斷言；本次文字 SHA256 `C42E594C279B4400B89B769DDE38EF0DC48BAE1056BA05F6CF4EBBA20E62F377` |
| I | `驗證紀錄/current/native-E149-component-linked-images-development/20261024-000018-240936/evidence.json` | 四 PE 的歷史連結證據；未執行；本次文字 SHA256 `6A580ECBCA332945198BF26D01198A7B03A2CF8D6A59B4028B377B9EEDC32D74` |
| S | `維護工具/candidates/native-E149-recovery-suite-plan-20261024-024515-538911/plan.json` | suite v3 的 16 個 member 計畫；本次文字 SHA256 `54B51B533D74D604224F9235DF472009433E448256B3276DEC1CDC8A836F6C1E` |
| B | `驗證紀錄/current/native-E149-observer-stability-binding-development/20261024-024235-491711/fixture-binding.json` | fixture binding v3＋observer stability extension；本次文字 SHA256 `481C3F82A8C257F7CBF6201DE5C3BBE959110EE4EFDCA3505B391C3A903F8472` |
| Q | `驗證紀錄/current/native-E149-component-matrix-compile/20261023-234918-939903/qualified-images-20261024-000005-592914/qualified.json` | 四映像獨立資格，原 producer 的 --images 輸入 |
| T | `維護工具/candidates/native-E149-static2941-ready-20261024-210441-110315/native_connection.rs:827–850` | 12 個 gate 的實際候選來源 |
| A | `維護工具/candidates/native-E149-hide97-static-draw-bridge-20261024-195610-910039/tower_assets.rs:1374–1465` | Static2941Bank 的資源及 SHA／形狀檢查 |

G 的 SHA 與來源 evidence 記錄相同；I 的 Q、C 及工具 pins 已做文字核對。S 的 **16/16** member plan SHA 與索引相同，16 份皆為 `E149-recovery-case-plan-v4`，各 32＋8＝40 次。這些核對不讀／啟動 PE，不驗證正式資源現況；原始檔案若在後續改變，必須重新核對。

## 一、12 個資源就緒 gate

完整名稱前綴為 `modern_lito::native_connection::static2941_queue_development_tests::`。來源 T；對照模板 `維護工具/templates/native_static2941_ready_tests.rs:1–24`。G 的 `current_ready_gate_group_cases=12`，其中四案 `#[ignore]`。以下資源欄「原」表示必須設定 `LITO_SHARED_RESOURCE_ROOT`；「純」只表示不需要原資源，並不代表沒有檔案寫入。

| # | 精確名稱後綴 | T 行號 | 資源／行為 |
|---|---|---:|---|
| 1 | `distinct_decode_dispatch_constructor_enqueue_samples_and_zero_deadline_are_bound` | 835 | 原；bank 載入／解碼 |
| 2 | `copied_world_observer_is_revoked_by_controller_drain_without_mutating_the_copy` | 837 | 原；bank 載入／解碼 |
| 3 | `equal_or_earlier_deadlines_keep_admission_order_in_the_callback_receipt` | 839 | 原；bank 載入／解碼 |
| 4 | `ring_eviction_revokes_only_removed_observers_and_keeps_queue_bounded` | 840 | 純；記憶體狀態 |
| 5 | `owner_change_and_queue_drop_revoke_old_snapshots` | 841 | 純；記憶體狀態 |
| 6 | `retired_or_already_scheduled_receipt_cannot_create_another_lifetime` | 842 | 純；記憶體狀態 |
| 7 | `wrong_connection_is_rejected_before_revoking_the_current_owner` | 843 | 純；記憶體狀態 |
| 8 | `untimed_world_accept_does_not_create_a_controller_clock_or_deadline` | 844 | 純；記憶體狀態 |
| 9 | `pending_resource_never_acquires_a_deadline_from_poll_or_paint` | 845 | 純；記憶體狀態 |
| 10 | `source_history_cannot_reset_a_retired_token` | 846 | 純；記憶體狀態 |
| 11 | `wrong_root_resource_proof_does_not_start_the_callback_clock` | 848 | 原；bank 載入／解碼 |
| 12 | `missing_spr_source_keeps_pending_and_does_not_create_a_fake_timer` | 849 | 純；會建立及移除自己的空目錄 |

四個原資源案共同使用 `SharedActionCatalog::load` 與 `Static2941Bank::load`（T:832、A:1397–1465）：

- `LITO_SHARED_RESOURCE_ROOT=D:\lineage開發\0-客戶端`，原 Text.idx／Text.pak 內 `list.spr` 及 Sprite 索引／封裝；catalog 必須符合程式 `LIST_SHA` 與 2941、type9、無 action 的定義。`shared_action_catalog.rs:62–100` 說明 Text/list.spr 取得方式。
- 主圖 `2941-0.spr`，865 bytes，SHA256 `E1A842865E4CC1B61B1A6A9B43CD451DB97AEF515CBE6F5FAF9A74CE922CA080`；只有 MissingEntry 可讀後備 `1120-0.spr`，211 bytes，SHA256 `1302F1D0C6E1636C75210726F302B8B86F18F7D365A1F8D1C4A52DB3C19129C6`。IO、損壞、取消不能以後備掩蓋。
- 第 11 案先取得真正 bank，再故意傳入不同 root，不能用假的 bank 取代原資源。第 12 案會寫入編譯時 `CARGO_MANIFEST_DIR/target/current/verification/static2941-empty-source`；單改 cwd 不會改變它。
- 來源未見這 12 案啟動 DB、遊戲／Java 程序或 GUI；四案會解碼原 SPR，後續須獨立授權執行環境。它們不證明 #97 畫面／時序等價。

**完整可執行入口：找不到可直接用於當前 main 的已連結 12-gate 映像或獨立 runner。** 最新 main 的 native_connection.rs 沒有此測試模組；G／對應 evidence 只記錄 metadata 編譯，正常 PE 與舊四元件 PE 均不能替代它。`*_broad_gate.py` 是舊版編譯／活動批次互斥檢查，不是這 12 案 runner（例如 `native1030_broad_gate.py:11–41`）。因此不捏造 `cargo test` 指令，否則可能選到零案而 exit0。後續先由 Claude 決定如何在獨立候選編譯環境驗證，取得當版 PE／inventory 後才能提供精確指令；不得為此直接套用 44 檔。

耗時：來源沒有實測值，**找不到可靠執行耗時**。純狀態案預期比四個原資源讀取／解碼案短，僅為推測；不能把 metadata 的編譯時間當測試耗時。

## 二、四元件 inventory

四元件是 Debug／Release × baseline／candidate 四份 **library-test PE**，不是四個遊戲元件，也不是全專案四個 target。I 記錄已連結、`test_inventory_executed=false`。baseline 只把 native_login Runtime 換回正常 3E，配對 baseline QA；candidate 是當時 G06 來源，不等於最新 44 檔 ready 候選。完整定義：`維護工具/native-E149-component-matrix.md:3–13`、`native-E149-component-readback-contract.md:5–19`。

| 映像 | I 記錄的 SHA256 | 選定模組的計畫案數 |
|---|---|---:|
| `Debug-baseline.exe` | `8D0449896FD86C0E20E0477D058E84AB958B8610914D8EA176FC188B2114BA72` | 23 |
| `Debug-candidate.exe` | `E89FE0229038DAC8E1D15DF47E6964FE34A98817F112C014D77F1CDCDA5F6A18` | 28 |
| `Release-baseline.exe` | `D75572046ED6C7F4874F36CA405F5CA1D652919E77467121212B4B420ADCF22B` | 23 |
| `Release-candidate.exe` | `1BE4B5A5E6D7BAC0287DEDB76CA0C4F1385D7291CD86D489F143968111931292` | 28 |

四份映像完整父目錄：`D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903`。此處是歷史 evidence 記錄的身分，沒有重新讀取二進位或聲稱映像目前可用。

原 producer 完整命令（**僅文件；目前不可直接執行**）：

```powershell
Set-Location 'D:\lineage開發\3-lito開發\lito重建版'
& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-component-matrix.py' --images 'D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-component-matrix-compile\20261023-234918-939903\qualified-images-20261024-000005-592914\qualified.json' --execute-owned-tests
```

這個入口會先執行四次 inventory，再執行完整 102 個單案，**沒有 inventory-only 開關**（runner:137–158、207–208）。每個 inventory 的底層完整命令如下；不得直接繞過 Job、原 handle 與診斷保存機制裸跑：

```powershell

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' --list

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-candidate.exe' --list

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Release-baseline.exe' --list

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Release-candidate.exe' --list

```

需求：Windows x64、原四 PE／plan／來源與工具 pins、Python、PowerShell CIM、私有 Job，以及原兩個診斷目錄。PE 的 `--list` 不執行測試本文，但仍啟動程序。沒有 DB／Java／原生遊戲主程式／GUI；元件 wire 本文另會建立 127.0.0.1 加密 loopback。

inventory 必須解析實際名稱、總數、重複、ignored；兩個選定模組應恰為 baseline 23／candidate 28，其他 library 名稱應相同。不可沿用 2463 或其他歷史固定總數。`native_E149_libtest_readback.py:7–19` 是解析器；`inventory-pinned-library-tests.py:9–20` 是另一種 product-retention 單映像工具，不能代替四映像資格；`inventory.py` 是會寫 JSON／封裝的檔案盤點工具，也不是四元件 inventory。

隔離限制：wire 案固定寫原編譯時 `target/current/verification/native-teleport-wire/Debug`、`Release`，換 cwd 無效。原 `native_E149_component_diagnostics.py` 設計逐案封存、核對 bytes／mtime、回復，拒絕 reparse／多 hardlink／未知檔案及並行變更；這套回復未實跑，不能假稱已安全。runner 還會寫工作佇列與八份進度檔。因此目前只保留入口，後續需先改成符合現行報告範圍的獨立 runner，並與 #53 序列化，不能邊跑 #53 邊使用共用診斷根。

耗時：`run-native-E149-component-matrix.py:112` 每次 inventory timeout 30 秒，四次合計等待額度 120 秒；不是實測值，也不含雜湊／保存／回復及啟動成本。

## 三、17 原控制與 6 指定反例

精確來源：C 的 `cases.baseline`（每項 JSON 的 `test_name`／`expected`／`existing_control`／`panic_contains`）。以下完整名稱可直接對照，不以模糊 substring 選案。

| # | 17 個原控制（baseline 與 candidate 都應 pass） |
|---|---|
| 1 | `modern_lito::native_login::rental_return_tests::quit_before_restart_is_rejected` |
| 2 | `modern_lito::native_login::rental_return_tests::duplicate_and_malformed_quit_are_rejected` |
| 3 | `modern_lito::native_login::rental_return_tests::repeated_restart_is_rejected` |
| 4 | `modern_lito::native_login::rental_return_tests::departure_before_quit_cannot_retire_room` |
| 5 | `modern_lito::native_login::rental_return_tests::source_refresh_requires_quit_and_own_departure` |
| 6 | `modern_lito::native_login::rental_return_tests::own_departure_after_quit_retires_only_once` |
| 7 | `modern_lito::native_login::rental_return_tests::unacknowledged_return_timeout_cannot_open_login_or_keep_credentials` |
| 8 | `modern_lito::native_login::rental_return_tests::missing_source_return_timeout_cannot_connect_an_owned_login_peer` |
| 9 | `modern_lito::native_login::rental_return_tests::cancellation_after_return_retirement_prevents_login_and_drops_credentials` |
| 10 | `modern_lito::native_login::rental_return_tests::acknowledged_return_timeout_retires_world_before_one_owned_login_connection` |
| 11 | `modern_lito::native_login::rental_return_tests::cancel_live_world_retires_snapshot_without_losing_generation` |
| 12 | `modern_lito::native_login::teleport_wire_tests::retired_endpoint_duplicate_and_old_world_packets_do_not_change_quit_barrier` |
| 13 | `modern_lito::native_login::teleport_wire_tests::coalesced_old_socket_burst_cannot_retarget_or_repeat_quit` |
| 14 | `modern_lito::native_login::teleport_wire_tests::malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner` |
| 15 | `modern_lito::native_login::teleport_wire_tests::new_destination_does_not_accept_previous_endpoint_as_a_new_handoff` |
| 16 | `modern_lito::native_login::teleport_wire_tests::cancel_discards_old_inbound_burst_before_fresh_login` |
| 17 | `modern_lito::native_login::teleport_wire_tests::invalid_transport_envelopes_during_teleport_retire_scope` |
| # | baseline 指定反例（candidate 同名案應 pass） | 指定訊息 | 原斷言位置 |
|---|---|---|---|
| 1 | `modern_lito::native_login::rental_return_tests::repeated_cancel_keeps_only_the_unused_recovery_correlation` | `unused correlation survived cancel` | C 同目錄 `baseline/native_rental_return_tests.rs:203` |
| 2 | `modern_lito::native_login::rental_return_tests::failure_then_cancel_does_not_erase_or_restore_retired_authority` | `assertion failed: c.teleport_recovery.is_some()` | C 同目錄 `baseline/native_rental_return_tests.rs:214` |
| 3 | `modern_lito::native_login::rental_return_tests::expired_recovery_send_failure_never_publishes_the_unconfirmed_directory` | `expired recovery source refresh` | C 同目錄 `baseline/native_rental_return_tests.rs:234` |
| 4 | `modern_lito::native_login::teleport_wire_tests::expired_recovery_confirms_source_before_publishing_a_new_directory` | `left: Failed`<br>`right: LoadingCharacters` | C 同目錄 `baseline/native_teleport_controller_tests.rs:256` |
| 5 | `modern_lito::native_login::teleport_wire_tests::cancelling_the_expired_recovery_refresh_cannot_start_another_transport` | `left: Failed`<br>`right: LoadingCharacters` | C 同目錄 `baseline/native_teleport_controller_tests.rs:275` |
| 6 | `modern_lito::native_login::teleport_wire_tests::recovery_character_absent_from_first_directory_confirms_source_before_manual_choice` | `controller Failed:`<br>`teleport character absent from destination directory` | C 同目錄 `baseline/native_teleport_controller_tests.rs:99` |

反例必須是單一精確測試、Rust exit101、1 failed、普通且唯一同名 panic、指定原檔／斷言行與訊息；wrong panic、crash、前提連線錯、編譯錯都不是預期成功。定位演算法見 `verify-native-E149-component-matrix.py:8–30`；第三個 wire 反例用第 99 行 pump 的三秒上限，而非正式 33／31 秒期限。

執行順序及完整入口同上一節：每個 profile 先 baseline 17 控制、再 6 指定反例，再 candidate 17 控制＋11 新元件，總計 `(23+28)×2=102`；期望 90 正向通過＋12 指定反例。**17＋6 並不是整個候選矩陣**；11 新元件＋另外 3 個正式 G06 新案才是歷史「14 案」。

每一個單案底層參數為 `test_name --exact --nocapture --test-threads=1`，不加 `--ignored`；環境固定 `RUST_BACKTRACE=0`、`RUST_LIB_BACKTRACE=0`、`RUST_TEST_THREADS=1`，移除繼承測試選項。以下為全部原控制／反例在 Debug baseline 的完整命令展開（僅文件，不能繞過上節 runner 直接執行；其餘三份由同 runner 與 C 精確展開）：

```powershell

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::quit_before_restart_is_rejected' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::duplicate_and_malformed_quit_are_rejected' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::repeated_restart_is_rejected' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::departure_before_quit_cannot_retire_room' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::source_refresh_requires_quit_and_own_departure' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::own_departure_after_quit_retires_only_once' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::unacknowledged_return_timeout_cannot_open_login_or_keep_credentials' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::missing_source_return_timeout_cannot_connect_an_owned_login_peer' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::cancellation_after_return_retirement_prevents_login_and_drops_credentials' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::acknowledged_return_timeout_retires_world_before_one_owned_login_connection' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::cancel_live_world_retires_snapshot_without_losing_generation' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::repeated_cancel_keeps_only_the_unused_recovery_correlation' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::failure_then_cancel_does_not_erase_or_restore_retired_authority' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::rental_return_tests::expired_recovery_send_failure_never_publishes_the_unconfirmed_directory' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::teleport_wire_tests::retired_endpoint_duplicate_and_old_world_packets_do_not_change_quit_barrier' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::teleport_wire_tests::coalesced_old_socket_burst_cannot_retarget_or_repeat_quit' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::teleport_wire_tests::malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::teleport_wire_tests::new_destination_does_not_accept_previous_endpoint_as_a_new_handoff' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::teleport_wire_tests::cancel_discards_old_inbound_burst_before_fresh_login' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::teleport_wire_tests::invalid_transport_envelopes_during_teleport_retire_scope' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::teleport_wire_tests::expired_recovery_confirms_source_before_publishing_a_new_directory' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::teleport_wire_tests::cancelling_the_expired_recovery_refresh_cannot_start_another_transport' --exact --nocapture --test-threads=1

& 'D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\verification\native-E149-component-matrix\20261023-234918-939903\Debug-baseline.exe' 'modern_lito::native_login::teleport_wire_tests::recovery_character_absent_from_first_directory_confirms_source_before_manual_choice' --exact --nocapture --test-threads=1

```

candidate 額外五個、不在六反例中的新案如下；另外六案就是上表同名反例轉為應 pass：

- `modern_lito::native_login::rental_return_tests::recovery_binding_mismatches_consume_the_one_shot_without_revival`

- `modern_lito::native_login::rental_return_tests::matching_recovery_is_taken_once_and_later_cancel_cannot_revive_it`

- `modern_lito::native_login::rental_return_tests::ordinary_cancel_without_an_authorized_destination_cannot_create_recovery`

- `modern_lito::native_login::rental_return_tests::a_new_authorized_handoff_replaces_only_the_old_correlation`

- `modern_lito::native_login::rental_return_tests::partial_or_unbound_recovery_cannot_request_another_source_directory`

需要資源／隔離／DB／視窗：與第二節相同。pure 是合成 controller 狀態，wire 是合成 encrypted loopback，沒有正式帳號／正式 DB／遊戲主程式／GUI。診斷 JSON 寫入必須隔離或保持原交易保存及回復；不能以這些元件案證明正式 owner 或自然期限。

耗時：每單案 timeout 120 秒；102 案＋四 inventory 的等待額度為 `102×120+4×30=12,360 秒（206 分鐘）`，僅保守 timeout 加總，不是實測或端到端上限。多數純案可能很短，但無當版實跑資料可精估。首個非預期結果停該矩陣，不自動重試。

## 四、G06 兩版 16 案、640 次

S 固定 Debug／Release 各八案，B 是目前 observer-stability binding。不可改用舊 v1／v2、512 次、六分支 96 sessions 的方案。`continue-native-startup-G06-controller.py:18` 仍是六分支舊 controller；`checkpoint-native-current-G06-complete.py:28` 的 12 案／96 sessions 也不是本計畫。

| scenario | 每個 profile 的精確 test_name | 自然等待 |
|---|---|---|
| `timeout` | `destination_timeout_retires_old_authority_and_allows_fresh_login` | 無此兩項額外等待 |
| `refused` | `refused_destination_retires_authority_and_allows_fresh_login` | 無此兩項額外等待 |
| `wrong-service` | `wrong_service_destination_retires_authority_and_allows_fresh_login` | 無此兩項額外等待 |
| `cancel-retry` | `cancellation_then_immediate_retry_consumes_staged_character_once` | 無此兩項額外等待 |
| `cancel-expiry` | `cancellation_retires_destination_and_expired_handoff_requires_fresh_selection` | 33 秒 |
| `cancel-repeat` | `repeated_cancellation_preserves_only_one_authenticated_recovery` | 無此兩項額外等待 |
| `refused-cancel-repeat` | `failed_handoff_then_repeated_cancel_preserves_the_authenticated_recovery` | 無此兩項額外等待 |
| `cancel-expiry-source` | `expired_stage_and_source_handoff_require_the_requested_source_directory` | 33 秒＋另 31 秒 |

每案四個真正退役 owner，各 8 setter：`setCount`、`setEquipped`、`setIdentified`、`setSealed`、`setEnchantLevel`、`set_durability`、`setBless`、`setChargeCount`，共 32 筆；另四 owner × `setItem`／`setItemId`＝8 筆。每案 40，16 案合計 640。兩類 trace 分開保存 SHA、marker、原 actor／owner／store／session 與拒絕前後狀態。必須精確 stale-owner 拒絕，不能把同值、參數錯、detached／staged、錯誤型別或意外成功當通過。來源：`native_E149_recovery_contract.py:12–24`、`native-E149-morph-fixture-binding.md:5–15`。

原 suite 完整命令（**僅文件；目前不可直接執行**）：

```powershell

Set-Location 'D:\lineage開發\3-lito開發\lito重建版'
& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\native_E149_recovery_suite.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-suite-plan-20261024-024515-538911\plan.json' --execute-owned-tests

```

原 single-case producer 的 16 個完整命令如下（suite 已依此順序執行並自動串接獨立 reader；不要另跑一次）：

```powershell

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024444-901694\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024446-942535\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024448-889003\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024450-837111\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024452-786917\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024454-742315\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024456-689435\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024458-639688\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024500-607784\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024502-503928\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024504-419896\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024506-346233\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024508-294341\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024510-179829\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024512-085101\plan.json' --execute-owned-tests

& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-recovery-case.py' --plan 'D:\lineage開發\3-lito開發\lito重建版\維護工具\candidates\native-E149-recovery-case-plan-20261024-024513-996846\plan.json' --execute-owned-tests

```

需求（取自第一份 member plan 的同名欄；其他 15 份共享 binding 與 producer pins）：

| 項目 | 計畫指向 |
|---|---|
| `java` | `C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe` |
| `python` | `C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe` |
| `formal_reference` | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar` |
| `JDBC` | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\server-reference\lib\mysql-connector-j-8.0.33.jar` |
| `catalog` | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\shared-profile-inputs\common_catalog-B64ED920.bin` |
| `SQL` | `D:\lineage開發\3-lito開發\lito重建版\維護工具\resources\E149-static-item-tables-D4111D8AEFF5\static-tables.sql` |
| `database_bundle` | `D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\owned-database-toolchains\mysql-5.5.22-2D6C58CA657A\bundle.json` |
| `static_manifest` | `D:\lineage開發\3-lito開發\lito重建版\維護工具\resources\E149-static-item-tables-D4111D8AEFF5\manifest.json` |
| `fixture_classes` | `D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-recovery-bootstrap-compile\20261024-023424-550188\classes` |
| `linked_image_qualification` | `D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-recovery-linked-build\20261023-211619-689295-debug\qualified-20261023-212654-234094\qualified.json` |

這些是計畫記錄的路徑，未重新核對任何 BIN、PE、正式 JAR 或啟動它們。需要 Windows x64、JDK 17、Python 3.12、MySQL 5.5.22 隔離 bundle、Connector/J 8.0.33、原 19 表／6277 列靜態 SQL、原 factory／SharedClientResources 與兩份已釘住的 controller 映像。不是用現行 Cargo main 編譯結果取代它們。

**會連 DB、會啟動測試程序，沒有原生 GUI。** 原設計使用每案新 datadir／tmpdir、隨機 schema／合成帳密、127.0.0.1 OS 配發高埠，mysqld `--no-defaults`，只匯入固定靜態表、禁止玩家 SQL；Java 是隔離 `NativeTeleportFailureHarness`，並非啟動既有正式遊戲程序或正式 Main。仍屬需專屬 runtime 的伺服器／測試活動，不能在本盤點中執行，不能與 #53 共用計時或輸出環境。

核對要求（必須由原始收據證明，不能憑程序名、單一 PID 或 state JSON 猜測）：

| 項目 | 必要證據與定義 |
|---|---|
| PID／FILETIME | 每程序保存原 PID、`creation_filetime_ticks`、整數 epoch millis、映像 SHA、完整命令、exit／wait；ticks 換算 `(ticks-116444736000000000)//10000`。先比 birth；只有同 birth 才可沿原 handle 回收。`native_E149_owned_process.py:25–38,64–100`；reader:85–115 |
| Job | 無名稱、不可繼承，KILL_ON_JOB_CLOSE，禁止 breakaway；CREATE_SUSPENDED → 身分核對 → AssignProcessToJobObject → ResumeThread。DB 後代須另核對映像／datadir／命令及 Job 成員。最終 ActiveProcesses=0，原 handle 均 wait confirmed，另做 CIM 原身分終止觀察。`native-E149-recovery-case-contract.md:21–25,39` |
| DB 退出 | 先核對自己的 listener／PID／FILETIME／datadir，正常 mysqladmin shutdown，再原 daemon handle signalled、producer exit、Job=0。`run-native-E149-recovery-case.py:166–190,245–276`；reader:145–210。19 表／6277 列與禁止玩家 SQL 仍須真實匯入證據 |
| 自然 33 秒 | cancel-expiry 及 cancel-expiry-source：`expiry_wait_ms >= 33000` 且 `closed_samples >=100`；過期後舊 ID 不得再授予，重新租裝用新 ID。reader:254；契約:53–55 |
| 另 31 秒 | cancel-expiry-source：`source_handoff_wait_ms >=31000`，以真正時間等待，不能改時鐘、縮租期或以 loopback 三秒斷言代替。reader:255 |
| 32＋8 雙 trace | 原 32 setter 與 8 morph 分開核對，保留完整 raw／SHA、相同原 owner 身分、正向 inventory capture、不同參數與拒絕前後不變；不得以 guard-only 證據替代 |
| 停止與資格 | 每案 producer 回收完成後 reader 成功才開始下一案；第一個 producer／reader 失敗、來源變更即停，零自動重試。只授所選矩陣資格，full library／GUI／Main／完整 G06 仍 false。`native_E149_recovery_suite.py:110–159` |

耗時（非實測）：每 profile 必須自然等待至少 `33+(33+31)=97 秒`，兩版至少 194 秒，另加其他案例、DB 初始化與回收。producer:217、228、244–245 依序有 DB ready 300 秒、fixture ready 180 秒、controller 260／380／500 秒、fixture 結束 30 秒的等待額度；兩版 16 案相關額度合計 `2×(6×770+890+1010)=13,040 秒（約 217 分鐘）`，還不含雜湊、daemon 觀察、DB shutdown、reader 與 checkpoint，故**不是全 suite 端到端上限或可靠預估**。原 DB lease 是每案 900 秒（producer:208）；實際效能與期限相容性未驗證。

## 阻塞與給下一棒的提示

1. **12 gate 缺可執行映像／runner**：只有未套用候選與 metadata，main 無此模組；已明確標「找不到」，不提供會選到零案的假指令。請 Claude 決定隔離候選驗證方式，保持 #33／#41 的凍結裁定。
2. **歷史執行入口與現行流程不相容**：靜態讀取舊工作佇列得到 `test_execution_deferred_by_user=true`，但最新 AGENTS／#43 已恢復授權。這是工具狀態過時，不能為了開測擅自改旗標。元件 runner:15–25、55–86 與 suite:14–19、80–109 會讀寫舊佇列／八份進度檔；須另開 runner 適配工作，保留身分、Job、首敗停止與診斷回復保障，不能繞過。
3. **隔離不是換 cwd**：元件診斷根、gate 空來源根及計畫內多個路徑為固定原根。#53 進行時本次只做靜態盤點；後續不能用主工作樹的產物或診斷根與它並行。不要修改舊映像／計畫的 SHA 來冒充新版本資格。
4. **前置未實證**：四 PE／G06 映像僅有歷史連結證據；本次未重新 hash 二進位、驗資源、連 DB、驗 Job、跑 reader 或 GUI。G06 非預設物品／來源帳號／timer 正向前提仍依 S0:33 待證，不得反射造狀態。
5. 交由 Claude 審查本盤點與版本界線，#58／G06 執行者依自身依賴與安全分類處理；目前不能把本報告視為測試通過或解除 #56／#58／T011 門檻。

## 做了什麼／怎麼驗證／未驗證什麼

- 已驗證：唯讀 GitHub 派工、最新規則與隔離基底；12 個 gate 的實際來源與 4 個 ignore；四映像歷史 metadata；C 的 17／6／11 名稱、指定反例位置；S 的 16 案與 640 計畫；文字檔雜湊與 member 引用。讀取方式為 Get-Content／Select-String／JSON 解析與 SHA，沒有 import 專案 Python 模組。
- 已核對 I 的八個工具／文件 pins 全部一致；抽查正常 native_login.rs、native_connection.rs、native_teleport_controller_tests.rs，原根及 main 6174e40 皆與 C 的基底清單一致。這只證明三檔，**不是 1049 輸入全量資格**。
- 推測：讀取資源可能主導四個 gate 耗時；各 timeout 加總僅排程參考，不是執行測量。
- 待驗證：所有 gate／inventory／單案／反例／G06／DB／GUI runtime 結果，以及原始資源和映像在後續執行時的可用性。
- 產出只有 `docs/reports/issue-57.md`；一次性產報腳本放 `scratch/issue57_report.py`，不進 git。沒有更新來源、測試、歷史佇列、正式資源、封存、config/server.properties、docs/REPORT.md 或安全設定。
