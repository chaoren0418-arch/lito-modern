# Issue #161 — T024 第 1／2 張：4183 時序與 S0B 重送（第 2 次嘗試）

本張只處理技能 #87 的持續物件 4183；不包含 T024 第二張的 LT hurt／base、速度倍率及 uint32 rollover。工作分支為 `task/161`，已合併最新 `origin/main`；未修改 `docs/REPORT.md`（合併時取 main 版本）或任務總表。

## 本輪修正（回應 codex-review 退回）

審查指出：callback 超限後，失效 live 仍留在 `Runtime.live`，`leases()` 把失效租約交給之後每一張 Frame，`Frame::status4183_current` 因而永遠不成立，目前世界畫面無法發布；且 `Runtime.error` 讓 `poll` 直接回 `Ok`，同世界所有 4183 靜默停擺，移除再加入也無法恢復。

修正方式為「物件隔離」，把物件失效與整張 Frame 的舊快照守衛分開：

- `native_status4183.rs` 移除全域 `Runtime.error`，改為每物件 `faults: BTreeMap<u32, Fault>`（incarnation、發生時刻、錯誤字串）與 `recovered` 計數。
- `Runtime::isolate`：物件在 S0B refresh 或 poll callback 失敗時，從 live 集合移除、退役其租約（舊快照、cached frame、deferred audio 仍照舊被拒），並移除該物件的待處理音效。因為它不再出現在 `leases()`，之後新畫出的 Frame 不會帶入失效租約，可以正常發布。
- `poll` 不再因單一物件失敗而中止或停擺；callback 失敗只隔離該物件並遞增 `visual_revision`，觸發重繪。其他 4183 繼續依自己的時鐘派送。全 runtime 錯誤（時鐘倒退、4096 派送上限、revision 溢位）仍回 `Err`，由 `native_login` 的 `fail("4183 visual clock: …")` 送到控制器錯誤通道。
- 錯誤可見性：Frame 收據 `status4183.object_faults` 與 `status4183.recovered` 取代原本的 `clock_error`；Frame 不再因錯誤而被拒，所以收據會被發布。
- 恢復：同 ID 的新 S0B 若符合 canonical 條件，會建立全新 live（新 584 ms 佇列、新租約），清除 fault 並把 `recovered` 加 1。物件被 `S_REMOVE_OBJECT 0x15` 移除或換世界時，fault 一併清除（不算恢復）。被隔離但仍存在的物件維持 fail closed：不走一般 Animator，也不畫 4183。

### 合併 main 的接線調整

main 新增 `IngressReceipt`（worker 解碼完成的 `Instant`）及 `static2941`／player callback。本輪：

- `native_connection.rs` 完全採 main 版本；移除本張原先自加的 `received_ms` 欄位。
- `World::accept_received` 改呼叫 `accept_at(body, ms_at(ingress.decoded_at()), now_ms())`，4183 的 wire 收包時刻改取自 main 的 `IngressReceipt`，派送時刻仍是實際 dispatch。`native_status4183::ms_at` 與 `now_ms` 共用同一個程序內原點。
- `native_login.rs` 採 main 版本，只在 main 的 native callback drain 之後加入 `poll_status4183`。
- `native_world_scene.rs`／`native_world_view.rs`／`native_movement_tests.rs`／`native_audio_delivery.rs`：兩邊欄位與條件並存（`static_active(world) && status4183_current()`、`occurrence` 與 `status_lease` 並存）。

## 先前已完成（第 1 次嘗試，維持不變）

- 4183 的 584 ms 起始等待以派送時刻為準，收包與派送是不同 sample；控制器在一批收包後以固定 sample 派送到期 callback，Model 只讀快照。source frame `1→2→3→4→5→0`、一般 168 ms、末幀 166 ms、晚到 remaining 最低 1 ms 再加 2 ms；無取模追幀、無本機到期刪除、不改 HP／MP／S25。
- S0B 重送：保留排程、重設動作與影格；idle 已排隊時 1 ms Queue callback；動作中重送另排 584 ms；同 deadline 依插入序號。
- 專用路徑嚴格驗證原六幀 action3、type0、Screen attr8、source1 overlap1453；一般 Animator 的 `breath`／`idle` 標籤守衛不放寬。
- 每物件最多 64 個 callback、每次 poll 最多 4096 次派送、音效 ring 32 筆。
- 靜態來源核對（`4E7A40`、`4E02C0`、`4E9300`、`4E45F0`、`4E34C0`、`4D6570` 等）與證據 SHA256 見本檔 git 歷史（commit `c466355`）的同名章節，未變更。

## 已驗證

| 項目 | 命令 | 結果 |
|---|---|---|
| 修正後編譯（合併前） | `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`（於 `LiTo381-Modern`） | 結束碼 0；`native_status4183*`、`native_world_scene`、`native_world_view` 無新警告 |
| 合併 `origin/main` 後編譯 | 同上命令 | 結束碼 0，`Finished dev profile`；所有測試目標（含新回歸案例）皆可編譯 |

新增／修改的回歸案例（`native_status4183_tests.rs`），均已編譯通過：

- `callback_budget_fails_closed_and_clock_regression_is_rejected`（改寫）：超限物件被隔離、租約退役、`leases()` 全部有效（目前 Frame 可發布）、fault 含 `4183 callback budget`；時鐘倒退仍回錯誤。
- `budget_fault_does_not_reject_current_frames_or_freeze_other_objects`：兩個 4183，一個超限；先前快照被退役，目前租約全有效，另一個物件在 584／752 ms 照常推進影格並產生音效。
- `faulted_object_is_rebuilt_by_a_new_put_or_cleared_by_removal`：隔離後新 S0B 重建（fault 清除、`recovered==1`、新佇列 604 ms）；再次失效後以 `0x15` 移除則 fault 清除且不算恢復。
- `callback_fault_during_poll_isolates_only_that_object`：poll 期間單一物件的 deadline 溢位只隔離該物件，`poll` 回 `Ok` 並遞增 revision，另一物件繼續推進，殘留音效皆為有效租約。

## 推測

- 物件隔離（fault 後該物件不畫、等待新 S0B 重建）是現代端的 fail-closed 策略，原 PE 在 callback 佇列滿時的行為未逆向確認；canonical 流程下 poll callback 先 pop 再排，不會觸及 64 上限，只有異常重送風暴會觸發。
- action3 啟動前的靜態 subframe0 仍是來源推論（收據 `initial_static_inferred`）。

## 待驗證／未驗證什麼

- **本輪依使用者 2026-11-09 規定，未在本機執行任何 `cargo test`**（測試二進位屬於編譯出來的程式）。所有 `native_status4183::tests`、`movement_tests`、`native_animation`、`native_network` 的實跑結果交由 PR 的 GitHub Actions CI；CI 的 `test result:` 為準，0 tests／failed>0／解析失敗都不算通過。
- 第 1 次嘗試的 39 個通過案例紀錄只適用於舊 commit，不代表本輪程式碼。
- 唯讀原資源案例 `original_bank_screen_fragments_audio_and_refresh_lease`（`#[ignore]`、需 `LITO_SHARED_RESOURCE_ROOT`）待 Codex 本機以 `--exact --ignored` 執行。
- 合併後的 `IngressReceipt` 時刻與 4183 `received_ms` 接線、與 main 的 static2941／player callback drain 的先後順序，只有編譯驗證，未實跑。
- 沒有啟動遊戲、連正式服務或 DB；無 GUI、自然眩暈、完整技能 #87 驗收。T024 第二張及 M2 未因本張完成。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_status4183.rs`、`native_status4183_tests.rs`
- 接線：`mod.rs`、`native_login.rs`、`native_world.rs`、`native_world_scene.rs`、`native_world_view.rs`、`native_audio_delivery.rs`、`native_movement_tests.rs`
- 本報告：`docs/reports/issue-161.md`

## 給下一棒的提示

- 審查者：請核對 `Runtime::isolate` 是否同時滿足「舊快照退役」與「目前 Frame 不帶失效租約」，以及 `poll` 中 `continue` 後不再使用已移除的 live。
- CI／Codex：確認 `native_status4183::tests` 的 `test result:`（預期 18 passed、1 ignored），並以 exact 執行 ignored 原資源案例。
- T024 第二張：勿把本張的 u64 程序時鐘視為 uint32 rollover 已解決；main 的 `native_clock`（u32）是之後統一時鐘的候選。
