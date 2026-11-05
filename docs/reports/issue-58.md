# Issue #58 — T010-E 第 2 次嘗試：執行前置仍阻塞

## 結論與本次計數

**阻塞（blocked）；T010-E 未通過，不解除 T011／M1 門檻。** 本次重新查核確認，#57 所需的當版 gate 映像與符合現行範圍的 runner 仍未交付。只完成靜態前置查核，未執行任何 gate、inventory 或控制／反例；不能把本報告提交、映像雜湊相符或零案 exit0 視為測試通過。

| 組別 | 本次列項數 | 已驗證通過 | 已驗證失敗 | 待驗證（未啟動） |
|---|---:|---:|---:|---:|
| 資源就緒 gate | 12 | 0 | 0 | 12 |
| 四映像 inventory | 4 | 0 | 0 | 4 |
| 原控制名稱 | 17 | 0 | 0 | 17 |
| 指定反例名稱 | 6 | 0 | 0 | 6 |
| 合計 | 39 | 0 | 0 | 39 |

測試啟動 0、預期反例成立 0、runtime ignored 0、逾時 0、測試重試 0。四案 gate 的來源標記為 ignored，不能混算成執行後的 ignored；本次四案也均未啟動。23 個控制／反例名稱 × 四映像＝92 個執行位置皆未執行；candidate 額外 10 次亦未執行，完整矩陣仍為 102 次待驗證。無首個**測試**失敗，因此沒有失敗測試名或測試 stdout／stderr；以下收據是靜態查核，不冒充失敗測試證據。

## 工作基準與範圍

- 日期：2026-11-05；執行者為本機 codex-local，第 2 次嘗試。R 指 `D:\lineage開發\3-lito開發\lito重建版`，W 指 `R\scratch\wt\task-58`。
- 開工即在 W 的 `task/58`，HEAD `56eb3256a1b38c965cf1610cd3fefc2a8c477592`；工作樹乾淨。未 checkout、重建分支或切換原根。遠端 `main` 仍為 `a038641565774bfde2d6dcbbff8fe3c59fd7d677`，遠端 `task/58` 開工仍為 `56eb3256a1b38c965cf1610cd3fefc2a8c477592`。
- 已讀 W 與 R 的 AGENTS.md、W 的 docs/REPORT.md、TASKS.md／PLAN.md 的 T010／M1、docs/TESTING.md、docs/CANDIDATES.md、#57／前次 #58 報告。R 的 AGENTS.md 仍含舊暫緩敘述；依目前使用者派工與 W 規則，安全測試已授權，無須重新取得測試同意。
- GitHub #58 現為 `stage:test`、`runner:codex-local`、`task:running`、attempts 2；前次 RESULT 後只有重排與本次派工，沒有新 runner／gate 映像交付。唯讀查詢現有 Issue 清單亦未見這兩項前置的新任務；這不是對未登記本機產物的全域證明。
- 本次僅更新 `docs/reports/issue-58.md`；一次性腳本與收據寫 W/scratch/issue-58。原專案檔案僅唯讀；未改 docs/REPORT.md、src、tests、候選或舊工作佇列，未套用 44 檔。
- 未讀機密設定、正式 BIN、封存；未連正式 DB／服務，未啟動、附加或停止遊戲／正式伺服器，未開原生 GUI。Cargo 未啟動，沒有將不含 gate 的現行 library 測試拿來充數。

## 本次已驗證的前置與阻塞

原始靜態收據 UTC 起點 `2026-11-05T08:16:26.835974+00:00`。本次重新實算四測試 PE bytes／SHA256、matrix plan 與五個工具及 ready contract 的 SHA256，皆與前次收據相同；不是沿用舊結果宣稱重跑。讀取前檢查 reparse 與單 link，未複製或啟動 PE。

| 代碼 | 本次證據與影響 | 必要前置 |
|---|---|---|
| G1 | W/src、tests 共 899 份 Rust 檔都沒有 12 個 gate 的函式定義；ready contract SHA 仍為 `A5EAFAB92F673DEEE4D1E8EC4EEE39085B3B3C87C24C565DED2F0ACFBF831A97`。#57 指定的當版連結映像／獨立入口缺件，派工未提供新的合格入口。 | 先決定隔離候選建置方案與映像資格；維持 44 檔凍結，不直接套用或以舊四 PE 替代。 |
| M1 | `run-native-E149-component-matrix.py:15–25` 仍讀舊旗標 `test_execution_deferred_by_user=true`；55–86 行寫八份進度檔及佇列，123–158／207–208 行無 inventory-only 或單安全組入口。runner SHA 仍為 `3BA9D479EDF86616C0104AAEC58E11AADFC1C7D5548D15DE891B5FD2BD740ECE`。 | 交付可選 inventory／安全組且只寫本 worktree 的入口，保留身分、Job、精確選案、原始輸出與首敗停止。舊旗標是程式相容性問題，並非撤回現行使用者授權；本任務不能改舊佇列或原 runner。 |
| M2 | `native_E149_component_diagnostics.py:10–27,52–121` 保存／回復 R 下固定 Debug／Release 診斷根；wire 原控制固定使用編譯時 `CARGO_MANIFEST_DIR`，換 cwd／TEMP 不能搬移寫入。 | 需能把寫入限制於 W 的建置／隔離方案；在本次「R 唯讀」要求下，原地保存後回復仍算寫入，不能執行。 |
| M4 | runner:91–95 仍核對 1049 編譯輸入、正常產品與正式參考 pins。本次只驗證 plan、指定工具及四測試 PE，不觸碰正式 BIN。 | 明確交付符合現行測試範圍的資格輸入及審查依據；四 PE hash 相符不等於完整資格通過。 |

各組獨立檢查：gate 因 G1 待驗證；inventory 因 M1／M4 待驗證；17 控制與 6 反例因同一矩陣前置待驗證，6 個 wire 原控制另受 M2 限制。純記憶體與無診斷寫入案例未被誤標為會連 DB／開遊戲或 GUI；它們缺的是合規入口及資格。本次沒有因一組阻塞而跳過其他組的前置評估。

**更正前次 M3：** `native_E149_owned_process.py` 的 `command_line()`／CIM 用於 `HeldProcess.adopt()`；矩陣的新子程序走 `OwnedJob.spawn_suspended()`，以原 handle、PID／FILETIME、映像 SHA、新建 Job 綁定後 resume，這條路徑未呼叫 CIM。因此前次全域 CIM 查詢失敗不能單獨證明矩陣無法啟動；本次移除 M3 作為逐案阻塞原因。原 handle／Job 能力仍未實跑，亦未聲稱已通過。G1／M1／M2／M4 不因本項更正而解除。

本次預設命令工具在建立程序前回覆 `helper_unknown_error: setup refresh had errors`，指定 PowerShell 與 Node 工具也遇到相同沙箱初始化錯誤。之後 `require_escalated` 成功完成唯讀與工作樹內操作，未收到自動核准審查拒絕；未改 ACL。Git 因擁有者差異拒絕預設讀取，改用單次 `-c safe.directory=D:/lineage開發/3-lito開發/lito重建版/scratch/wt/task-58`，未改全域 Git 設定。這些工具前置錯誤不是測試失敗，也不是測試重試。

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

資格檔：`驗證紀錄/current/native-E149-component-matrix-compile/20261023-234918-939903/qualified-images-20261024-000005-592914/qualified.json`；PE 父目錄為 `R/LiTo381-Modern/target/current/verification/native-E149-component-matrix/20261023-234918-939903`。第 2 次嘗試重新檢查路徑祖先無 reparse、檔案單 link，再唯讀 hash；沒有啟動／複製 PE。

| 映像 | bytes | 本次實算 SHA256（符合原資格） | 計畫所選模組案數 | runtime 結果 |
|---|---:|---|---:|---|
| Debug-baseline | 71746048 | `8D0449896FD86C0E20E0477D058E84AB958B8610914D8EA176FC188B2114BA72` | 23 | 待驗證：M1／M4 |
| Debug-candidate | 71760896 | `E89FE0229038DAC8E1D15DF47E6964FE34A98817F112C014D77F1CDCDA5F6A18` | 28 | 待驗證：M1／M4 |
| Release-baseline | 30469632 | `D75572046ED6C7F4874F36CA405F5CA1D652919E77467121212B4B420ADCF22B` | 23 | 待驗證：M1／M4 |
| Release-candidate | 30511104 | `1BE4B5A5E6D7BAC0287DEDB76CA0C4F1385D7291CD86D489F143968111931292` | 28 | 待驗證：M1／M4 |

`--list` 本身不執行測試本文，預期無 DB／GUI／遊戲；但會啟動測試 PE，仍須符合原身分、Job、輸出保存及 inventory 解析保障。不能因為只列清單就跳過這些前置。未取得當次名稱、總數、重複與 ignored 清單，23／28 僅是計畫值。

## 17 原控制與 6 反例逐項

來源 C：`維護工具/candidates/native-E149-component-matrix-20261023-231736-569424/plan.json` 的 cases.baseline；本次 SHA256 `C42E594C279B4400B89B769DDE38EF0DC48BAE1056BA05F6CF4EBBA20E62F377`，與資格檔一致。對應 `baseline/native_rental_return_tests.rs` 簡稱 RTest，`baseline/native_teleport_controller_tests.rs` 簡稱 WTest。各列名稱完整；所有列在 Debug／Release × baseline／candidate 的本次結果都為**待驗證（未啟動）**，不是通過。

注意：C 的 `module=pure` 是模組分類，不能一概解讀為無 socket。RTest:102–105 的 owned_listener 綁定 127.0.0.1:0；原控制 7–11 都呼叫它，第 10 案實際觸發一次 loopback transport。WTest:26–93 建立 owned loopback peer／執行緒，135–144 固定寫診斷 JSON。反例的三個 wire 案本體未呼叫 record，仍需共同矩陣前置。

| # | 原控制精確名稱 | 本體來源行 | 資源與副作用 | 阻塞原因 |
|---|---|---:|---|---|
| 1 | `modern_lito::native_login::rental_return_tests::quit_before_restart_is_rejected` | RTest:41 | 合成狀態／封包；無 listener | M1／M4 |
| 2 | `modern_lito::native_login::rental_return_tests::duplicate_and_malformed_quit_are_rejected` | RTest:49 | 合成狀態／封包；無 listener | M1／M4 |
| 3 | `modern_lito::native_login::rental_return_tests::repeated_restart_is_rejected` | RTest:57 | 合成狀態／封包；無 listener | M1／M4 |
| 4 | `modern_lito::native_login::rental_return_tests::departure_before_quit_cannot_retire_room` | RTest:64 | 合成狀態／封包；無 listener | M1／M4 |
| 5 | `modern_lito::native_login::rental_return_tests::source_refresh_requires_quit_and_own_departure` | RTest:70 | 合成狀態／封包；無 listener | M1／M4 |
| 6 | `modern_lito::native_login::rental_return_tests::own_departure_after_quit_retires_only_once` | RTest:76 | 合成狀態／封包；無 listener | M1／M4 |
| 7 | `modern_lito::native_login::rental_return_tests::unacknowledged_return_timeout_cannot_open_login_or_keep_credentials` | RTest:104 | owned loopback listener | M1／M4 |
| 8 | `modern_lito::native_login::rental_return_tests::missing_source_return_timeout_cannot_connect_an_owned_login_peer` | RTest:116 | owned loopback listener | M1／M4 |
| 9 | `modern_lito::native_login::rental_return_tests::cancellation_after_return_retirement_prevents_login_and_drops_credentials` | RTest:128 | owned loopback listener | M1／M4 |
| 10 | `modern_lito::native_login::rental_return_tests::acknowledged_return_timeout_retires_world_before_one_owned_login_connection` | RTest:139 | owned loopback listener；實際連接及退役 transport | M1／M4 |
| 11 | `modern_lito::native_login::rental_return_tests::cancel_live_world_retires_snapshot_without_losing_generation` | RTest:155 | owned loopback listener | M1／M4 |
| 12 | `modern_lito::native_login::teleport_wire_tests::retired_endpoint_duplicate_and_old_world_packets_do_not_change_quit_barrier` | WTest:139 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M4；另 M2 |
| 13 | `modern_lito::native_login::teleport_wire_tests::coalesced_old_socket_burst_cannot_retarget_or_repeat_quit` | WTest:158 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M4；另 M2 |
| 14 | `modern_lito::native_login::teleport_wire_tests::malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner` | WTest:176 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M4；另 M2 |
| 15 | `modern_lito::native_login::teleport_wire_tests::new_destination_does_not_accept_previous_endpoint_as_a_new_handoff` | WTest:206 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M4；另 M2 |
| 16 | `modern_lito::native_login::teleport_wire_tests::cancel_discards_old_inbound_burst_before_fresh_login` | WTest:218 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M4；另 M2 |
| 17 | `modern_lito::native_login::teleport_wire_tests::invalid_transport_envelopes_during_teleport_retire_scope` | WTest:229 | 內嵌 fixture／加密 loopback／threads；固定診斷 JSON | M1／M4；另 M2 |

| # | 反例精確名稱 | 本體來源行 | 資源與副作用／預期斷言 | 阻塞原因 |
|---|---|---:|---|---|
| 1 | `modern_lito::native_login::rental_return_tests::repeated_cancel_keeps_only_the_unused_recovery_correlation` | RTest:199 | 合成 controller 狀態；未開 listener；指定訊息 `unused correlation survived cancel` | M1／M4 |
| 2 | `modern_lito::native_login::rental_return_tests::failure_then_cancel_does_not_erase_or_restore_retired_authority` | RTest:212 | 合成 controller 狀態；未開 listener；指定訊息 `assertion failed: c.teleport_recovery.is_some()` | M1／M4 |
| 3 | `modern_lito::native_login::rental_return_tests::expired_recovery_send_failure_never_publishes_the_unconfirmed_directory` | RTest:231 | 合成 controller 狀態；未開 listener；指定訊息 `expired recovery source refresh` | M1／M4 |
| 4 | `modern_lito::native_login::teleport_wire_tests::expired_recovery_confirms_source_before_publishing_a_new_directory` | WTest:254 | 內嵌 fixture／加密 loopback／threads；本體未呼叫 record；指定訊息 `left: Failed；right: LoadingCharacters` | M1／M4 |
| 5 | `modern_lito::native_login::teleport_wire_tests::cancelling_the_expired_recovery_refresh_cannot_start_another_transport` | WTest:273 | 內嵌 fixture／加密 loopback／threads；本體未呼叫 record；指定訊息 `left: Failed；right: LoadingCharacters` | M1／M4 |
| 6 | `modern_lito::native_login::teleport_wire_tests::recovery_character_absent_from_first_directory_confirms_source_before_manual_choice` | WTest:284 | 內嵌 fixture／加密 loopback／threads；本體未呼叫 record；指定訊息 `controller Failed:；teleport character absent from destination directory` | M1／M4 |

反例成功必須有 baseline 單精確案、exit101、唯一同名 panic、1 failed、原檔案與斷言位置及指定訊息，candidate 同名案則須 pass；不能用崩潰、前置連線錯、工具拒絕或未啟動充數。原 reader 位於 `verify-native-E149-component-matrix.py:8–30`。本輪無任何反例成立證據。

17＋6 是 23 個名稱，四映像共 92 個對應執行位置；原完整矩陣還有 candidate 額外 5 個名稱 × 兩 profile＝10 次（合計 102），本輪亦未執行。額外名稱僅沿用 #57 的候選清單，不宣稱完成其本體安全審查或 runtime。G06 16 案／640 次、DB／正式自然期限／GUI 完全未執行；本輪沒有可隔離 DB runtime 的資格證據，不提前啟動下一階段。

## 指令、原始收據與產出

本次實際執行的靜態查核指令（exit0；不 import 專案 Python 模組，不啟動測試）：

```powershell
Set-Location 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-58'
& 'C:\Users\CHAOREN\AppData\Local\Programs\Python\Python312\python.exe' -B -X utf8 'scratch/issue-58/audit-attempt-2.py'
```

- `scratch/issue-58/audit-attempt-2.py` SHA256：`F602CFE1D2CA916A71D0C47F64842D7E4B276C0293243C1AB3AF6F840DC6E19F`。
- `scratch/issue-58/attempt-2-static-audit.log` SHA256：`DCB4440D8BEEF38423F7C08613B41BB578B134CA9445521AC907CB6049A0D0BD`。

前次 `audit.py`、`static-audit.log` 保持原樣；本次未覆蓋前次收據。新 log 逐項列出 gate 搜尋結果、四 PE hash、舊旗標、工具與 contract hash、計畫控制／反例數。兩次收據都只屬靜態查核。

#57 第二、三節保留原 producer、四個 `--list` 與精確單案指令；**這些命令本次均未執行，也不應在前置未解除前裸跑。** 本次沒有首個失敗測試可提供重現指令；重現目前阻塞可執行上方靜態查核並讀取原 runner 指定行。修改 runner 或建立候選映像屬前置開發，超出本次只更新指定報告的範圍。

## 未驗證事項與給下一棒的提示

- **已驗證：** 本次四 PE bytes／SHA、plan 與指定工具／contract hash；899 份 Rust 檔缺少 12 gate 定義；原 runner 的寫入範圍與入口限制；CIM 與新建子程序路徑的區別。這些均為靜態結果。
- **推測：** 新入口完成後，inventory、純狀態與 owned loopback 案可望在本機隔離環境執行；尚無 runtime 證明。
- **待驗證：** 39 列項及完整 102 次矩陣；當版 gate 映像資格、Job 實際能力、精確 inventory／ignored／重複名、控制與反例預期。G06 16 案／640 次與 GUI 亦未執行。
- **Claude／專案主控先接手：** 補排「當版 gate 隔離建置與映像資格」及「四映像 inventory／安全組 runner 適配」前置。後者需固定輸出在 W、保留 Job／原 handle／FILETIME／SHA、精確名單核對、首敗原始收據與停止規則，並處理硬編診斷路徑和現行資格來源。
- **調度器：** #58 維持 blocked，#61／T011／M1 不解除。不要只因 runner 從 dot 換成 codex-local 就重派同一測試；先把前置交付納入依賴，再由本機執行。這是開發前置，不應轉請使用者做 GUI 驗收或操作正式服務。
- **本機下一輪：** 取得新映像／入口與資格後逐組執行；首個非預期失敗保存 stdout／stderr、測試名及重現指令並停止該組，其他安全獨立組繼續。略過／ignored／逾時／未執行分開計數，不算通過。

唯一追蹤產出：`docs/reports/issue-58.md`。報告可供審查，測試任務尚未完成。
