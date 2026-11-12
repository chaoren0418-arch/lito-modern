# Issue #190 — T055-1：G06.3c 租裝返回生命週期

## 做了什麼

本張僅整合租裝正常返回、重新入村、服務物品與 NPC 互動清單生命週期。沒有修改 T055-2 的來源目錄／入口交接，也沒有套用隔離候選或修改伺服器。

- `return_from_rental` 送出 C10 後立即撤銷 NPC 清單與待執行的世界操作；地圖入口尚在等待回覆時拒絕並行返回。
- 返回期間到重新選角前，過期世界、NPC、傳送回覆不能重新啟動戰鬥、恢復互動清單或延長期限。S65／SF0／SD0／SF5 仍走原有嚴格順序驗證；沒有配裝標記的 mode6 角色也必須先取得自身離房確認。
- 收到自己的 SD0 時退役場景並隱藏背包。保留同一連線的權威背包暫存，持續接受服務物品刪除；重新選擇同角色時可恢復未被伺服器移除的原有物品。S7C 是增量批次，不能自行把它當成完整替換，亦不能在客戶端猜測服務租約已取消。
- `Snapshot.rental_return_outcome` 明確區分 `Normal` 與 `Reconnected`；正常返回須等 SF5 後完整角色目錄到齊。既有探針不得把重連恢復算成正常返回。
- 共用既有合成加密 loopback 測試工具，增加租裝／旅館服務生命週期測試；沒有新增產品中的模擬伺服器或測試開關。

## 已驗證

在本 worktree 執行 Debug／Windows x64 檢查及離線測試。驗證包裝器直接取用 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult`，核對結束碼、單一測試 harness、非零 running 數、計數一致與 failed=0；沒有把略過測試算成通過。

新增 5 案涵蓋：map500／501／503 返回及重新入村、買／賣清單與旅館數量輸入失效、物品刪除前後的背包保留、相同 NPC 的新互動、舊世界／NPC 封包重播不改狀態或期限、無配裝標記仍須自身離房、缺少離房確認逾時，以及重連恢復結果分流。正常流程使用合成加密 TCP；其餘錯序／重播案例也直接呼叫產品 dispatcher 檢查原子性。

以下命令均於 `LiTo381-Modern/` 執行；完整 stdout／stderr 與結束碼留在產物目錄。

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_login::rental_lifecycle_tests
ExitCode=0; Measure-TestResult=PASS
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2490 filtered out; finished in 0.53s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_login::rental_return_tests
ExitCode=0; Measure-TestResult=PASS
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 2484 filtered out; finished in 0.06s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_login::teleport_wire_tests
ExitCode=0; Measure-TestResult=PASS
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2489 filtered out; finished in 1.83s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_merchant
ExitCode=0; Measure-TestResult=PASS
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_inventory
ExitCode=0; Measure-TestResult=PASS
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
```

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
ExitCode=0; Measure-TestResult=PASS
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 21s
```

最終版本合計 **34 passed／0 failed／0 ignored**；沒有執行全部 library、Release 或需正式 harness 的 ignored 測試。編譯仍有專案既存警告。差異檢查 `git -c core.whitespace=cr-at-eol diff --cached --check` 通過（既有兩檔採 CRLF，保留原格式）。

首跑新增測試的失敗紀錄（`lifecycle-first-failed.log`）如下，**未計為通過**：

```text
ExitCode=101; Measure-TestResult=FAIL: command-exit,test-failed
test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 2490 filtered out; finished in 0.55s
```

原因：有活躍世界時，過早 SF5 先經 `World::accept` 寫入 `unhandled_opcodes`，才由返回守門拒絕，違反「拒絕時狀態不變」。已將返回期間的 SD0／SF5 直接送往控制封包驗證，再完成上述 5 案重跑。

## 第 2 次嘗試（CI 退回修正，2026-11-12）

- PR #218 CI（run 42174575421）失敗原文：`test result: FAILED. 1919 passed; 1 failed; 575 ignored`，失敗案為 `native_login::teleport_wire_tests::malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner`，訊息 `native TCP write: An existing connection was forcibly closed by the remote host. (os error 10054)`，合成 peer 同時回報 `Overlapped I/O operation is in progress. (os error 997)`。
- 判斷（推測）：非本張返回邏輯錯誤，而是測試用 `FaultPeer` 以 `SO_RCVTIMEO` 10 ms 輪詢讀取；Windows 在接收逾時後 socket 狀態不定，之後 I/O 可能以 997 失敗並重設連線。本分支新增的 lifecycle 測試共用同一 peer，增加觸發機率。
- 修正：`native_teleport_controller_tests.rs` 的 `FaultPeer` 交握後改為非阻塞模式輪詢（WouldBlock 時 sleep 1 ms），寫入改為帶 2 秒期限的非阻塞寫入迴圈；不再使用 read／write timeout。只改測試輔助，不改產品程式。
- 合併 `origin/main`（9a5b887）：`native_login.rs` 衝突，main 新增 `accept_game_received`（ingress epoch 檢查）；解法為保留 main 的 ingress 檢查在前，本張租裝返回 fence 接在其後。
- 已驗證：合併後 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` 結束碼 0（`Finished dev profile`）。
- 依 2026-11-09 本機限制，本次未在本機執行 `cargo test`；測試交給 CI。
- 已驗證（CI run 43145011407，commit 62c383b，結論 pass）：`test result: ok. 2030 passed; 0 failed; 585 ignored; 0 measured; 0 filtered out; finished in 10.86s`；本張 5 個 `rental_lifecycle_tests::*` 與先前失敗的 `teleport_wire_tests::malformed_and_misordered_teleport_replies_over_tcp_retire_pending_owner` 均為 `ok`。單次通過不能證明 flake 已根除（推測已根除，待後續 CI 持續觀察）。
- 再次同步（分支落後 main 55 個 commit）：合併 `origin/main`，`native_teleport_controller_tests.rs` 衝突——main 把 `FaultPeer::new` 拆成 `at`／`waiting_at`（可指定位址與 accept 期限）；解法為採用 main 的建構函式，保留本分支 `pub(super)` 可見性（`rental_lifecycle_tests` 需要）與交握後非阻塞輪詢。合併後 `cargo check ... --all-targets` 結束碼 0。測試結果以此次 push 後的 PR CI 為準（待驗證）。

## 推測

- 正式端若依契約提供完整 C10 → S65 → CD9 → SF0 → CD0 → SD0 → SF5 → CC8 → S03／SCB 回覆，客戶端應可在同一 GAME 連線完成正常返回及重選入村；合成測試不能證明現行正式伺服器已提供此路徑。
- 本機唯讀歷史紀錄 `驗證紀錄/current/native-world-integration/rental-return-boundary.json` 記載 BBC59308 的 `quitFight` 缺口；`rental-return-services-candidate-qualified.json` 的服務物品／房卡清理屬 6887A337 候選，且 `formal_deployed=false`。這些只是範圍與協定參考，不是本次正式端驗證。

## 待驗證／未完成事項

- 現行正式伺服器正常返回、真實服務租約撤銷、旅館共享房客逐出，以及完整 GUI／實體輸入仍未驗證。伺服器 `quitFight` 缺口維持 U-M3-1，不以斷線重連或 Tournament 測試宣告解決。
- 合成測試直接建立租裝初始狀態，使用歷史固定封包向量及自行構造的物品／角色資料；不是正式認證 session，也不是伺服器租約計時或 DB 清理測試。
- T055 第 2 張（C_CA／S_FB 目錄、租裝村與商店交接、map500 其他入口種類）由各自 Issue 接續。TASKS.md 的 T055 狀態未改。
- 未啟動遊戲、未操作既有遊戲程序、未連正式服務或 DB；未修改原始工作目錄的專案檔案、封存或機密設定（worktree 提交必要的共用 `.git` 中繼資料除外）。`docs/REPORT.md` 未修改。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_peer_rental_probe.rs`
- `LiTo381-Modern/src/modern_lito/native_rental_lifecycle_tests.rs`
- `LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs`（共用測試輔助函式）
- `docs/reports/issue-190.md`
- 未追蹤的本機驗證日誌：`LiTo381-Modern/target/current/verification/issue-190/`。

## 給下一棒的提示

- Claude 審查本張客戶端返回邊界與測試，再由使用者合併 `task/190`。不要把這張結果標成整體 G06.3c 正式端驗收通過。
- T055-2 執行者接續來源／入口整合時，保留返回的完整控制封包順序、背包 retained 語意與商店 owner／revision 守門。
- U-M3-1 與後續本機測試排程需分開核對正常返回、重連恢復、服務物品與共享房客；驗收讀取 `rental_return_outcome`，僅 `Normal` 可列為正常返回。
