# Issue #58：T010-E gate／四映像 inventory／控制與反例執行前置

## 結論

**阻塞；未通過 T010-E，不解除 T011／M1 後續門檻。** 本輪完成逐項靜態核對，沒有符合現行範圍且具備完整隔離入口的執行組。12 gate、4 inventory、17 原控制與 6 反例均待驗證；測試啟動 0、通過 0、測試失敗 0、預期反例成立 0、runtime ignored 0、runtime 逾時 0、自動重試 0。未啟動不是通過，也不是實際測試失敗。

主要阻塞仍是 #57 所述的缺少當版 gate 映像與 runner 適配。四份歷史 library-test PE 本次已實算 SHA256／大小且符合資格檔，這僅解除「映像是否存在且 bytes 相符」的不確定性，**不代表 inventory 或案例已執行**。

## 工作基準與授權界線

- 專屬本機 Codex 對話由 dot 主控開啟；隔離工作樹 `scratch/wt/task-58`，分支 `task/58`；固定基準 `a038641565774bfde2d6dcbbff8fe3c59fd7d677`。根工作樹 main 未切換或修改。
- 開工 GitHub #58 為 open、runner:dot、task:running，只有派工留言、無 RESULT；遠端／本機均無 task/58，開啟 PR 查詢為空。此次建立的分支與工作樹屬本輪。沒有觀察到其他執行者；CIM 程序查詢受限，不能宣稱已完整排除所有本機程序。
- 前置 #57、#56 均 closed、task:done、review:pass、gate:pass；#56 的 76 項通過不等於本輪歷史矩陣的資格。#57 的審查留言明確保留 #58 執行前置。
- 已讀原根及隔離最新 AGENTS.md、TASKS.md T010、docs/TESTING.md、docs/CANDIDATES.md、#57 報告與留言。最新規則已記錄恢復測試，但不擴張本次授權。直接委派優先於 Issue：未讀寫 docs/REPORT.md、config/server.properties、正式 BIN、封存；未連正式 3306／2000、未操作既有正式遊戲程序、未開 GUI。
- 範圍只新增本報告。沒有修改 src／tests、候選、舊工作佇列或 runner；未套用 44 檔。一次性靜態稽核／產報腳本與原始 log 只放本工作樹的 `scratch/issue-58/`。
- 實際權限是 workspace-write／auto_review，**完全存取未能驗證**。預設 git fetch 發生網路失敗，其後標準 require_escalated 的唯讀 fetch／ls-remote 核准成功。`Get-CimInstance Win32_Process -Filter "Name = 'codex.exe'"` 回覆「拒絕存取」；這是 OS 查詢失敗，不是測試失敗，也不是本輪收到自動核准審查拒絕。未修改 ACL、safe.directory、防火牆或其他安全設定，未用另一執行方式繞過必要核准。

## 已驗證的靜態前置與阻塞代碼

全文 R 指原 repo 根 `D:\lineage開發\3-lito開發\lito重建版`；W 指 `R\scratch\wt\task-58`。以下行號以原始檔案為準。

| 代碼 | 靜態證據與影響 | 解除所需工作 |
|---|---|---|
| G1 | W 的 src／tests 共 899 份 Rust 檔，12 個 gate 函式定義均不存在；當版定義仍在凍結候選。#57 找不到當版已連結 PE／獨立入口，本輪未找到替代。 | 主控決定合法的隔離候選編譯／資格方案；不能拿零案 cargo exit0 或舊四 PE 代替。 |
| M1 | `run-native-E149-component-matrix.py:15–25` 仍讀舊佇列旗標；本輪唯讀確認 `test_execution_deferred_by_user=true`、active_workloads 為空。`55–86` 會更新八份進度檔與舊佇列；`123–158,207–208` 無 inventory-only／單安全組入口。 | 另派 runner 適配，移除對歷史流程的寫入依賴，保留使用者授權 gate；本輪不擅改旗標或繞過。 |
| M2 | `native_E149_component_diagnostics.py:10–27,52–121` 保存並回復原根固定 Debug／Release 診斷目錄；wire `record()` 使用編譯時 CARGO_MANIFEST_DIR，改 cwd／TEMP 無法隔離。 | 審查獨立輸出或完整交易保存／回復方案，建立互斥且無其他 writer 的證據；不得無核對覆寫或清空。 |
| M3 | 原入口要求 PID／FILETIME、原 handle、私有 Job、命令列與原始輸出回讀；`native_E149_owned_process.py:41–50` 依賴 CIM，`140–167` 包含 kill-on-close、suspended→Assign→resume。當前 CIM 查詢失敗，子程序同一查詢能力尚未實證。 | 標準權限流程驗證所需能力；不能只用一般 subprocess 裸跑 PE、用 PID 或名稱代替身分。 |
| M4 | `run-native-E149-component-matrix.py:91–95` 還需 1049 編譯輸入、正常產品與正式參考 pins 全量資格；本輪只核對指定文字與四測試 PE，未觸碰正式 BIN。 | 決定適用的新基準與資格輸入範圍，保留禁止正式資源修改的界線；不可改 SHA 冒充相同資格。 |

每組均獨立評估：gate 因 G1 未啟動；inventory 因 M1／M3／M4 未啟動（原路徑也會建立 DiagnosticStore）；控制與反例因同一矩陣前置未啟動，會寫診斷的六個 wire 原控制另有 M2。沒有因一組阻塞而略過其他組的靜態審查。由於沒有啟動任何測試，無首個測試失敗收據；首敗停止規則未被觸發，亦未自動重試。

## 12 gate 逐項

完整名稱前綴 `modern_lito::native_connection::static2941_queue_development_tests::`；來源 `維護工具/candidates/native-E149-static2941-ready-20261024-210441-110315/native_connection.rs`。以下每項皆未啟動。

| # | 名稱後綴 | 行 | 資源／副作用靜態核對 | 結果與原因 |
|---|---|---:|---|---|
| 1 | `distinct_decode_dispatch_constructor_enqueue_samples_and_zero_deadline_are_bound` | 835 | 原 Text/list.spr 與 Sprite bank；讀取／解碼；來源標 ignored | 待驗證：G1 |
| 2 | `copied_world_observer_is_revoked_by_controller_drain_without_mutating_the_copy` | 837 | 原 Text/list.spr 與 Sprite bank；讀取／解碼；來源標 ignored | 待驗證：G1 |
| 3 | `equal_or_earlier_deadlines_keep_admission_order_in_the_callback_receipt` | 839 | 原 Text/list.spr 與 Sprite bank；讀取／解碼；來源標 ignored | 待驗證：G1 |
| 4 | `ring_eviction_revokes_only_removed_observers_and_keeps_queue_bounded` | 840 | 合成 World／Queue／History 記憶體狀態 | 待驗證：G1 |
| 5 | `owner_change_and_queue_drop_revoke_old_snapshots` | 841 | 合成 World／Queue／History 記憶體狀態 | 待驗證：G1 |
| 6 | `retired_or_already_scheduled_receipt_cannot_create_another_lifetime` | 842 | 合成 World／Queue／History 記憶體狀態 | 待驗證：G1 |
| 7 | `wrong_connection_is_rejected_before_revoking_the_current_owner` | 843 | 合成 World／Queue／History 記憶體狀態 | 待驗證：G1 |
| 8 | `untimed_world_accept_does_not_create_a_controller_clock_or_deadline` | 844 | 合成 World／Queue／History 記憶體狀態 | 待驗證：G1 |
| 9 | `pending_resource_never_acquires_a_deadline_from_poll_or_paint` | 845 | 合成 World／Queue／History 記憶體狀態 | 待驗證：G1 |
| 10 | `source_history_cannot_reset_a_retired_token` | 846 | 合成 World／Queue／History 記憶體狀態 | 待驗證：G1 |
| 11 | `wrong_root_resource_proof_does_not_start_the_callback_clock` | 848 | 原 Text/list.spr 與 Sprite bank；讀取／解碼；來源標 ignored | 待驗證：G1 |
| 12 | `missing_spr_source_keeps_pending_and_does_not_create_a_fake_timer` | 849 | 建立／移除編譯根下 static2941-empty-source 的專屬空目錄 | 待驗證：G1 |

四個原資源案需要 `LITO_SHARED_RESOURCE_ROOT`、`SharedActionCatalog::load`、`Static2941Bank::load`（同來源 832 行）；2941 主圖／1120 fallback 的身份與限制見 #57。第 11 案仍先載入真 bank；第 12 案寫 `CARGO_MANIFEST_DIR/target/current/verification/static2941-empty-source`，不是系統 TEMP。來源未見這 12 案開 DB、遊戲或 GUI；本輪未讀取／解碼原資源，沒有假 bank 或 stub。

沒有可驗證的當版可執行入口，因此不新增會選到零案的 cargo 指令，也不重跑 #53／#55 的現行 main 同名或相近案例充數。

## 四映像 inventory 逐項

資格檔：`驗證紀錄/current/native-E149-component-matrix-compile/20261023-234918-939903/qualified-images-20261024-000005-592914/qualified.json`；PE 父目錄為 `R/LiTo381-Modern/target/current/verification/native-E149-component-matrix/20261023-234918-939903`。本次先檢查路徑祖先無 reparse、檔案單 link，再唯讀 hash；沒有啟動／複製 PE。

| 映像 | bytes | 本次實算 SHA256（符合原資格） | 計畫所選模組案數 | runtime 結果 |
|---|---:|---|---:|---|
| Debug-baseline | 71746048 | `8D0449896FD86C0E20E0477D058E84AB958B8610914D8EA176FC188B2114BA72` | 23 | 待驗證：M1／M3／M4 |
| Debug-candidate | 71760896 | `E89FE0229038DAC8E1D15DF47E6964FE34A98817F112C014D77F1CDCDA5F6A18` | 28 | 待驗證：M1／M3／M4 |
| Release-baseline | 30469632 | `D75572046ED6C7F4874F36CA405F5CA1D652919E77467121212B4B420ADCF22B` | 23 | 待驗證：M1／M3／M4 |
| Release-candidate | 30511104 | `1BE4B5A5E6D7BAC0287DEDB76CA0C4F1385D7291CD86D489F143968111931292` | 28 | 待驗證：M1／M3／M4 |

`--list` 本身不執行測試本文，預期無 DB／GUI／遊戲；但會啟動測試 PE，仍須符合原身分、Job、輸出保存及 inventory 解析保障。不能因為只列清單就跳過這些前置。未取得當次名稱、總數、重複與 ignored 清單，23／28 僅是計畫值。

## 17 原控制與 6 反例逐項

來源 C：`維護工具/candidates/native-E149-component-matrix-20261023-231736-569424/plan.json` 的 cases.baseline；本次 SHA256 `C42E594C279B4400B89B769DDE38EF0DC48BAE1056BA05F6CF4EBBA20E62F377`，與資格檔一致。對應 `baseline/native_rental_return_tests.rs` 簡稱 RTest，`baseline/native_teleport_controller_tests.rs` 簡稱 WTest。各列名稱完整；所有列在 Debug／Release × baseline／candidate 的本次結果都為**待驗證（未啟動）**，不是通過。

注意：C 的 `module=pure` 是模組分類，不能一概解讀為無 socket。RTest:102–105 的 owned_listener 綁定 127.0.0.1:0；原控制 7–11 都呼叫它，第 10 案實際觸發一次 loopback transport。WTest:26–93 建立 owned loopback peer／執行緒，135–144 固定寫診斷 JSON。反例的三個 wire 案本體未呼叫 record，仍需共同矩陣前置。

| # | 原控制精確名稱 | 本體來源行 | 資源與副作用 | 阻塞原因 |
|---|---|---:|---|---|
| 1 | `modern_lito::native_login::rental_return_tests::quit_before_restart_is_rejected` | RTest:41 | 合成狀態／封包；無 listener | M1／M3／M4 |
| 2 | `modern_lito::native_login::rental_return_tests::duplicate_and_malformed_quit_are_rejected` | RTest:49 | 合成狀態／封包；無 listener | M1／M3／M4 |
| 3 | `modern_lito::native_login::rental_return_tests::repeated_restart_is_rejected` | RTest:57 | 合成狀態／封包；無 listener | M1／M3／M4 |
| 4 | `modern_lito::native_login::rental_return_tests::departure_before_quit_cannot_retire_room` | RTest:64 | 合成狀態／封包；無 listener | M1／M3／M4 |
| 5 | `modern_lito::native_login::rental_return_tests::source_refresh_requires_quit_and_own_departure` | RTest:70 | 合成狀態／封包；無 listener | M1／M3／M4 |
| 6 | `modern_lito::native_login::rental_return_tests::own_departure_after_quit_retires_only_once` | RTest:76 | 合成狀態／封包；無 listener | M1／M3／M4 |
| 7 | `modern_lito::native_login::rental_return_tests::unacknowledged_return_timeout_cannot_open_login_or_keep_credentials` | RTest:104 | owned loopback listener；第 10 案連接及退役 transport | M1／M3／M4 |
| 8 | `modern_lito::native_login::rental_return_tests::missing_source_return_timeout_cannot_connect_an_owned_login_peer` | RTest:116 | owned loopback listener；第 10 案連接及退役 transport | M1／M3／M4 |
| 9 | `modern_lito::native_login::rental_return_tests::cancellation_after_return_retirement_prevents_login_and_drops_credentials` | RTest:128 | owned loopback listener；第 10 案連接及退役 transport | M1／M3／M4 |
| 10 | `modern_lito::native_login::rental_return_tests::acknowledged_return_timeout_retires_world_before_one_owned_login_connection` | RTest:139 | owned loopback listener；第 10 案連接及退役 transport | M1／M3／M4 |
| 11 | `modern_lito::native_login::rental_return_tests::cancel_live_world_retires_snapshot_without_losing_generation` | RTest:155 | owned loopback listener；第 10 案連接及退役 transport | M1／M3／M4 |
| 12 | `modern_lito::native_login::teleport_wire_tests::retired_endpoint_duplicate_and_old_world_packets_do_not_change_quit_barrier` | WTest:139 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M3／M4；另 M2 |
| 13 | `modern_lito::native_login::teleport_wire_tests::coalesced_old_socket_burst_cannot_retarget_or_repeat_quit` | WTest:158 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M3／M4；另 M2 |
| 14 | `modern_lito::native_login::teleport_wire_tests::malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner` | WTest:176 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M3／M4；另 M2 |
| 15 | `modern_lito::native_login::teleport_wire_tests::new_destination_does_not_accept_previous_endpoint_as_a_new_handoff` | WTest:206 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M3／M4；另 M2 |
| 16 | `modern_lito::native_login::teleport_wire_tests::cancel_discards_old_inbound_burst_before_fresh_login` | WTest:218 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M3／M4；另 M2 |
| 17 | `modern_lito::native_login::teleport_wire_tests::invalid_transport_envelopes_during_teleport_retire_scope` | WTest:229 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M3／M4；另 M2 |

| # | 反例精確名稱 | 本體來源行 | 資源與副作用／預期斷言 | 阻塞原因 |
|---|---|---:|---|---|
| 1 | `modern_lito::native_login::rental_return_tests::repeated_cancel_keeps_only_the_unused_recovery_correlation` | RTest:199 | 合成 controller 狀態；未開 listener；指定訊息 `unused correlation survived cancel` | M1／M3／M4 |
| 2 | `modern_lito::native_login::rental_return_tests::failure_then_cancel_does_not_erase_or_restore_retired_authority` | RTest:212 | 合成 controller 狀態；未開 listener；指定訊息 `assertion failed: c.teleport_recovery.is_some()` | M1／M3／M4 |
| 3 | `modern_lito::native_login::rental_return_tests::expired_recovery_send_failure_never_publishes_the_unconfirmed_directory` | RTest:231 | 合成 controller 狀態；未開 listener；指定訊息 `expired recovery source refresh` | M1／M3／M4 |
| 4 | `modern_lito::native_login::teleport_wire_tests::expired_recovery_confirms_source_before_publishing_a_new_directory` | WTest:254 | 內嵌 fixture／加密 loopback／threads；本體未呼叫 record；指定訊息 `left: Failed；right: LoadingCharacters` | M1／M3／M4 |
| 5 | `modern_lito::native_login::teleport_wire_tests::cancelling_the_expired_recovery_refresh_cannot_start_another_transport` | WTest:273 | 內嵌 fixture／加密 loopback／threads；本體未呼叫 record；指定訊息 `left: Failed；right: LoadingCharacters` | M1／M3／M4 |
| 6 | `modern_lito::native_login::teleport_wire_tests::recovery_character_absent_from_first_directory_confirms_source_before_manual_choice` | WTest:284 | 內嵌 fixture／加密 loopback／threads；本體未呼叫 record；指定訊息 `controller Failed:；teleport character absent from destination directory` | M1／M3／M4 |

反例成功必須有 baseline 單精確案、exit101、唯一同名 panic、1 failed、原檔案與斷言位置及指定訊息，candidate 同名案則須 pass；不能用崩潰、前置連線錯、工具拒絕或未啟動充數。原 reader 位於 `verify-native-E149-component-matrix.py:8–30`。本輪無任何反例成立證據。

17＋6 是 23 個名稱，四映像共 92 個對應執行位置；原完整矩陣還有 candidate 額外 5 個名稱 × 兩 profile＝10 次（合計 102），本輪亦未執行。額外名稱僅沿用 #57 的候選清單，不宣稱完成其本體安全審查或 runtime。G06 16 案／640 次、DB／正式自然期限／GUI 完全未執行；本輪沒有可隔離 DB runtime 的資格證據，不提前啟動下一階段。

## 指令與原始證據

本輪唯一執行的稽核程式是自建的靜態讀取腳本，沒有 import 專案 Python 模組、啟動 libtest／Cargo／Java／DB 或修改來源：

```powershell
Set-Location 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-58'
& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'scratch/issue-58/audit.py'
```

原 matrix 命令如下，**本輪未執行，不應在阻塞未解除前重試**；四個 `--list` 與 23 精確單案的完整展開請見 `docs/reports/issue-57.md` 第二、三節，原參數保持不變。後續適配後需另給審查通過的新入口，不能直接裸跑以下入口的內部命令。

```powershell
& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'D:\lineage開發\3-lito開發\lito重建版\維護工具\run-native-E149-component-matrix.py' --images 'D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current\native-E149-component-matrix-compile\20261023-234918-939903\qualified-images-20261024-000005-592914\qualified.json' --execute-owned-tests
```

原始靜態 log 為 W 的 `scratch/issue-58/static-audit.log`，起始 UTC 2026-11-05T08:03:14.690647+00:00；記錄四映像 bytes／SHA、逐 gate 查找、舊旗標、指定工具 SHA 及 plan 案數。這份是靜態收據，**不是測試失敗輸出或 inventory log**。

- `scratch/issue-58/audit.py` SHA256：`45A04E1ADD9DF0F3E34F96D4B35638FA1D55AE7A1F9F491E93DDAF2E0CD1A0A8`。
- `scratch/issue-58/static-audit.log` SHA256：`3A665ADE69EB4C2381CEA97AD0966A8A454F32A08C5D32CAF9C2A839A148F51D`。

## 推測／待驗證與給下一棒的提示

- 已驗證：定義／來源、副作用與舊 runner 的上述限制；四測試 PE 當前 bytes／SHA；main 缺 12 個 gate 定義。上述全部是靜態事實。
- 推測：完成 runner 適配與身分／互斥資格後，inventory、純狀態及 owned loopback 組可能可執行；尚未經 runtime 證明。CIM 全域查詢受限不等於已證明自有子程序一定無法查詢，該能力仍待驗證。
- 待驗證：全部 39 個列項（12 gate＋4 inventory＋17 原控制＋6 反例），以及上述矩陣展開；沒有可報為通過的測試結果，也沒有偽造失敗測試名。
- 請 Claude／專案主控將 #58 維持 blocked，先安排兩項明確前置：(1) 當版 gate 的合法隔離建置與映像資格，保持 44 檔凍結；(2) 符合目前報告範圍、可選 inventory／安全組、保留 Job／PID／FILETIME／原始輸出／首敗停止／診斷隔離的 runner。權限需求循標準核准處理，不改 ACL 或安全設定。
- 適配完成後再交本機 dot／codex-local 逐組執行；首個非預期失敗保存原始 stdout／stderr 與重現指令並停止該組，其餘已通過安全審查的獨立組繼續。不應要求使用者進行此類開發前置操作。

交付追蹤檔只有 `docs/reports/issue-58.md`；報告完成不等於測試任務完成，請勿以本提交解除下游門檻。
