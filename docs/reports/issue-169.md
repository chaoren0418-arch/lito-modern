# Issue #169 — T023 第 1／4 張：取消、來源確認與來源綁定

## 做了什麼

本輪在既有 `task/169` worktree 接續 `d21c2eb882fb395a4e7a711c2b2266f12195e1f8`，處理審查指出的兩項缺口。未切分支、套用隔離候選、修改原專案或 `docs/REPORT.md`。

前次完成的一次性 LOGIN 恢復保留：第二次取消保留尚未取用的端點／帳號／來源／角色關聯；明確選來源時取用一次，不符亦消耗。取消仍清掉密碼及世界／背包／服務權威。逾時或首份完整清單缺少原角色時，只送一次 C8，等待新的完整 S03／SCB 清單才開放手動選角；不自動登入、不重送 C9，部分接續不能重試。

本次補上：

1. `source_handoff_expiry_confirms_c8_before_manual_selection` 是可執行的 ignored 正式入口，對應 runner／Java 的 `cancel-expiry-source`。先自然等待 staged 33 秒，重新 LOGIN 選來源後再自然等待 31 秒。原 `cancel-expiry` 保持 stage-only；未加入其他兩個 repeated-cancel 正式 scenario。
2. 共用 `observe_source_expiry` 檢查等待時的 RedirectReady、來源、epoch、空角色／背包／世界／房間／商人／服務權威。新增加密 loopback 案例實際執行同一函式 31 秒，再檢查一次 C8、部分清單拒絕選角、完整清單發布、明確選角才送一次 C9。
3. Java memory fixture 預設「測試甲」，客戶端按收到的名稱選「測試乙」。ready 增加 scenario／requested_source／default_source，Rust 與 runner 都核對。Rust 固定 E149 SHA，環境變數不能改成其他 JAR。
4. 雙重到期 fixture watchdog／runner lease 採 440／460 秒，listener accept 上限 420 秒；這是測試整體預算。8 秒控制器回覆、30 秒伺服器 TTL 契約、普通 peer 協調 60 秒、到期 peer 協調 120 秒均未變。
5. runner 在 javac／Java 前執行映像內的來源 registry 檢查，要求精確一案通過及唯一綁定；將綁定存入 receipt，所有來源 SHA 納入結尾 drift check。新增的檢查程序也沿原 handle／身分核對 cleanup 回收。

## 13 檔 registry／14 檔 fixture 的現行重整

產出為 `LiTo381-Modern/src/modern_lito/native_teleport_registry.rs`，由指定整合測試以 `#[path]` 編譯、owned runner 實際消費。它取代「外部候選路徑＋歷史映像資格」的選取方式，不再讀 `工作佇列.json` 或候選目錄。

`REGISTRY` 保留原來 13 個唯一遷移 key（keyboard 4、blind20 3、cancel-recovery 6）；fixture 在相同 13 個 key 後加入 bootstrap，成為 14 個。每列有現行來源、實作／驗證位置及處理方式。依賴另列 `DEPENDENCIES`，不拿新增 helper 湊檔數。來源去重後以編譯時內嵌 bytes 和工作樹逐 byte 比對，產生當版 SHA256。

**13／14 是遷移條目數，不是宣稱 14 份歷史功能均已重建或驗收。** 本張依傳送恢復範圍重整可執行綁定：保留現行鍵盤／技能基線、明確排除不存在的 blind QA，以當版 memory fixture 取代本張不使用的 production bootstrap／完整物品 observer。綁定固定輸出 `full_historical_14_file_behavior_verified=false`，排除項不算通過。若驗收要求是重建所有歷史功能而非本張來源重整，該部分仍需任務負責人明訂範圍，不能由本報告改記完成。

下表路徑相對 `LiTo381-Modern/`。機器可讀對應由 `recovery_registry::source_binding_matches_compiled_checkout --exact --nocapture` 輸出；owned runner 將其存入 `receipt.source_registry`。

| 舊 key | 現行實作／處理 | 驗證位置與界限 |
|---|---|---|
| `src/modern_lito/world_raster.rs` | 綁定現行基線 | all-targets check；不授鍵盤候選資格 |
| `src/modern_lito/native_world_scene.rs` | 綁定現行基線 | all-targets check；不授鍵盤候選資格 |
| `src/modern_lito/native_login_window.rs` | 綁定現行基線 | all-targets check；GUI 未驗 |
| `tests/native_skill_selection.rs` | 保留現行測試 | `cargo test --test native_skill_selection` |
| `src/modern_lito/native_skills.rs` | 綁定現行基線 | all-targets check；不授 blind20 候選資格 |
| `src/modern_lito/native_skill_selection.rs` | 綁定現行基線 | `native_skill_selection` 測試；不授 blind20 資格 |
| `tests/native_blind.rs` | `excluded-candidate-only`，無編譯來源 | `candidate_only_qa_is_not_admitted`；非致盲功能測試 |
| `src/modern_lito/native_login.rs` | `cancel`／`take_teleport_recovery`／`refresh_recovery_source` | `native_login` 單元與 `recovery_wire` |
| `src/modern_lito/native_rental_return_tests.rs` | 租裝測試保留；取消案例拆到 `native_teleport_recovery_tests.rs` | `native_login` 單元 |
| `src/modern_lito/native_teleport_controller_tests.rs` | 現行 FaultPeer 與恢復案例 | `recovery_wire`，含自然 8／31 秒 |
| `tests/native_teleport_failure.rs` | 六個 owned 入口與離線入口 | loopback 已跑；正式入口可達但未執行 |
| `tests/oracle/NativeTeleportFailureHarness.java` | 非預設來源、雙重到期、memory fixture | E149 javac；runtime 未跑 |
| `tests/oracle/NativeRecoveryReadonlySnapshot.java` | 本張改用 `NativeTeleportFailureHarness.stagedSnapshot`／`NativePeerHarness.observe` | E149 javac；不授完整物品 observer 資格 |
| `tests/oracle/NativeRecoveryProductionBootstrap.java` | 本張改用 `NativeTeleportFailureHarness.main` 啟動前建立的 memory repository | E149 javac；production／SQL 未納入 |

負面測試令 controller、Java 或 runner 缺檔／內容漂移，要求拒絕且錯誤指出該項；正向測試核對當版 bytes、13／14 唯一 key 與分組數量。此 gate 防止舊映像搭配新版 fixture，不能取代正式行為驗收。

## 已驗證

2026-11-06，最終來源的 `cargo check --all-targets` 及三項相關 Cargo 測試皆通過，命令結束碼均為 **0**。結果如下：

| 命令 | 結束碼／gate | 原始日誌 |
|---|---|---|
| `cargo check --all-targets` | 0／PASS | `001.log`、`check.json` |
| `native_teleport_failure -- recovery_` | 0／PASS；11 passed，6 filtered out | `002.log`、`wire.json` |
| `native_login` 單元 | 0／PASS；36 passed，1 ignored | `003.log`、`lib.json` |
| `native_skill_selection` | 0／PASS；7 passed | `004.log`、`selection.json` |

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 48.67s
test result: ok. 36 passed; 0 failed; 1 ignored; 0 measured; 2453 filtered out; finished in 19.68s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.65s
```

另以 runner 的精確測試入口實際匯出綁定並用相同 marker／schema 解析，核對 26 份現行來源 SHA，結束碼 **0**；日誌 `binding-export.log`、綁定 `source-binding.json`（皆不入 git）。

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.01s
```

這一案已包含在上述 11 案；部分 loopback 亦由 library 重複執行，不能把各命令數字相加當作不同新增案例數。

沿 `維護工具/run-local-test-suite.ps1` 實際 `Invoke-SuiteStep`／`Measure-TestResult` 執行與解析，另要求 passed > 0。0 tests、非零退出碼、failed > 0、解析失敗均不計通過。不是整套 whitelist 全跑。

工作目錄 `LiTo381-Modern/`，實際命令：

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --features teleport-wire-test --test native_teleport_failure -- recovery_ --test-threads=1
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --features teleport-wire-test --lib -- modern_lito::native_login:: --test-threads=1
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_skill_selection -- --test-threads=1
```

Java：JDK 17，UTF-8、專案內 tmp／class 輸出，以唯讀正式 `formal-server-E1498871.jar` 編譯 `NativePeerHarness.java` 與 `NativeTeleportFailureHarness.java`，最終結束碼 **0**，產出 3 個 class。JAR 實算 SHA256 為 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。只啟動 javac，未啟動 Java fixture server。

runner 靜態檢查：以 UTF-8 讀取後交 PowerShell AST parser，核對六組 runner → Rust 精確測試 → Java scenario 入口、registry gate 位於 javac 前、registry 子程序列入 cleanup，結束碼 **0**，日誌 `runner-static.log`。本機未找到 PowerShell 7，沒有把 Windows PowerShell 5 的 AST 檢查當作完整 runner 執行。`git -c core.whitespace=cr-at-eol diff --check` 亦通過。

### 保留的失敗

- 本次加入 registry 後首輪退出 **101**：編譯完成後又修改整合測試固定 JAR SHA，兩個正向綁定檢查以 `test image/source drift: tests/native_teleport_failure.rs` 拒絕；8 個恢復案例及一個拒絕測試通過。這是執行順序錯誤，不計成功；保留 `002-registry-stale.log`／`wire-registry-stale.json`，固定來源後重新編譯驗證。

```text
test result: FAILED. 9 passed; 2 failed; 0 ignored; 0 measured; 6 filtered out; finished in 48.62s
```

- javac 首次呼叫未將 `-J-Dfile.encoding=UTF-8` 引號化，被 Windows PowerShell 拆成錯誤旗標，退出 **2**。修正呼叫後退出 0；`java/javac.log`／`java/javac-final.log` 均保留。
- 前次兩個無效合成 inventory owner 測試前提已在 `d21c2eb` 修正；`003-initial-failed.log`／`lib-initial-failed.json` 仍保留，不覆寫為成功。

## 推測

正式 30 秒 source handoff 到期後，GAME 可能先提供預設來源目錄；需要 C8 重新確認所選來源及完整清單，才可開放手動選角。loopback 的預設清單是合成封包，只證明客戶端等待／清單 gate，不證明 E149 TTL 或物品接續。

## 待驗證／未驗證什麼

- 六個 ignored 正式 E149 案例、實際 33＋31 秒雙重 TTL 與預設／非預設來源、Java fixture runtime、runner 全流程均未跑；filtered／ignored 不計通過。
- 未連正式服務或 DB、未啟停遊戲或 GUI；未跑 Release、完整 library、G06 640 次或 G01–G12 驗收。
- 歷史 blind20／keyboard 候選功能、production item bootstrap／完整 observer 沒有因此完成；registry 明確記錄現行處置與資格界限。
- 另外兩個 repeated-cancel 正式 scenario、通用 finite History、旅館／Presentation／音效、T 鍵、cancel-late、物品拒絕矩陣屬其他分張。

## 產出路徑

- 前次一次性恢復：`LiTo381-Modern/src/modern_lito/native_login.rs`、`native_teleport_recovery_tests.rs`。
- 本次：`LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs`、新增 `native_teleport_registry.rs`、`LiTo381-Modern/tests/native_teleport_failure.rs`、`LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java`、`維護工具/run-teleport-failure.ps1`。
- 報告：`docs/reports/issue-169.md`。
- 不入 git：`LiTo381-Modern/target/current/verification/issue-169/` 日誌／gate／class；`scratch/verify-169.ps1`、`scratch/verify-169-runner.ps1` 為一次性本機驅動。

## 給下一棒的提示

1. Claude 審查可達入口及 src registry → 當版映像 → runner 的綁定與拒絕。區分來源重整與歷史功能驗收，不可把 14 個 key 當成 14 檔功能通過。
2. 本機測試執行者用當版映像、owned E149 JAR、PowerShell 7，透過 `run-teleport-failure.ps1 -Scenario cancel-expiry-source` 驗雙重自然到期；test／Java／runner 已接好。保留 440／460 秒 fixture／lease 及產品期限。
3. T023 第 2 張沿用此入口與觀察函式，避免重複新增雙重到期測試；另兩個 repeated-cancel scenario 仍需補。第 4 張若需完整物品 observer／bootstrap，應依當版 E149 API 實作並更新 registry，不借用 memory fixture 資格。
4. GitHub CLI 的現有 token 回傳 HTTP 401；GitHub 連接器可讀取 Issue #169／PR #201，可用於 PR 資訊更新。Git 推送在沙箱內的預檢遇到 Windows 憑證存取錯誤，最終推送結果於交付摘要另列。未修改機密設定。
