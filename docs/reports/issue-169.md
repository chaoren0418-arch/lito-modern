# Issue #169 — T023 第 1／4 張：取消與 LOGIN 恢復

## 做了什麼

在 `task/169` 工作樹重寫現行控制器的恢復路徑，未套用歷史隔離候選。

- 第二次及後續 `cancel()` 保留尚未取用的傳送恢復識別；失敗後再取消亦同。密碼與認證 credentials 清空，世界、背包、服務與操作權威仍退役；只保留比對所需的端點／帳號／來源／角色識別，不排程自動登入。
- 明確選來源時一次性取用識別，重新核對 LOGIN IP／port、帳號與來源；不符亦消耗，後續取消不可復活已消耗的識別。
- 未使用的接續等滿原 8 秒，或第一份完整 GAME 清單缺少原角色時，送一次所選來源的 C8。清掉未確認清單，等新清單完整才發布 `CharactersReady`；不預送 C9。
- 已有選角結果、部分背包、未完成清單或錯誤 GAME／來源狀態不得重新要求接續。C8 enqueue 失敗走完整退役，C8 回覆亦有 8 秒期限，不重試、不新開連線。
- `native_teleport_failure.rs` 新增 feature 控制的離線 loopback 入口；既有自然到期案例改以清單中的「測試乙」名稱選來源，保留 33 秒 staged 觀察，另準備 31 秒來源交接觀察與收據欄位供第 2 張接入 dual-expiry scenario。既有 `cancel-expiry` 仍只驗 staged 到期。原普通協調上限 60 秒、到期案例協調上限 120 秒、原 15 秒外層回復與 8 秒畫面／步行界限不放寬。

## 13 檔 registry／14 檔 fixture 對應

唯讀核對原專案 `工作佇列.json` 所指三個 registry manifest 的 SHA，以及 `current_E149_fixture_binding` 的 SHA；13 個 key 無重複，fixture 是 11 Rust＋3 Java＝14 檔。fixture binding SHA256：`481C3F82A8C257F7CBF6201DE5C3BBE959110EE4EFDCA3505B391C3A903F8472`。這是歷史來源盤點，不代表這批候選已執行或已併入。

下表路徑相對 `LiTo381-Modern/`。依本張範圍把舊 registry 對應至現行實作與下一棒，不把另外兩組功能帶入本張。

| 舊組別 | 路徑 | 本張處理 |
|---|---|---|
| keyboard | `src/modern_lito/world_raster.rs` | 未改；鍵盤／呈現另張 |
| keyboard | `src/modern_lito/native_world_scene.rs` | 未改；鍵盤／呈現另張 |
| keyboard | `src/modern_lito/native_login_window.rs` | 未改；鍵盤入口另張 |
| keyboard | `tests/native_skill_selection.rs` | 未改；選技另張 |
| blind20 | `src/modern_lito/native_skills.rs` | 未改；技能 20 另張 |
| blind20 | `src/modern_lito/native_skill_selection.rs` | 未改；技能 20 另張 |
| blind20 | `tests/native_blind.rs` | 歷史候選獨有；本張不新增技能測試 |
| cancel-recovery | `src/modern_lito/native_login.rs` | 現行 src 重寫一次性恢復與 C8 gate |
| cancel-recovery | `src/modern_lito/native_rental_return_tests.rs` | 原測試保留；新恢復狀態案例集中到 `native_teleport_recovery_tests.rs` |
| cancel-recovery | `src/modern_lito/native_teleport_controller_tests.rs` | 沿現行加密 peer 加入恢復測試 |
| cancel-recovery | `tests/native_teleport_failure.rs` | 新增離線入口、33／31 秒觀察程式及按名稱選來源 |
| cancel-recovery | `tests/oracle/NativeTeleportFailureHarness.java` | 現行 owned harness 保留；未搬入舊 observer／mutation overlay |
| cancel-recovery | `tests/oracle/NativeRecoveryReadonlySnapshot.java` | 歷史候選獨有；正式 observer／物品驗收後續另張 |
| fixture 新增第 14 檔 | `tests/oracle/NativeRecoveryProductionBootstrap.java` | 歷史候選獨有；正式 production item bootstrap 後續另張 |

舊 13 檔是 keyboard 4＋blind20 3＋cancel-recovery 6；第 14 檔來自 Java bootstrap。它不是「13 個 Rust runtime 檔」。本張來源以目前 git 版本為基礎，沒有把舊 linked image／metadata 資格移記到新版本，也不重寫原專案 registry 或 fixture binding。

## 已驗證

2026-11-06 在此 Windows x64 工作樹完成下列命令，工作目錄為 `LiTo381-Modern/`，三項最終結束碼皆為 **0**。暫存與產物均在工作樹；未啟動一般遊戲程式。沿 `維護工具/run-local-test-suite.ps1` 的實際 `Measure-TestResult`／`Invoke-SuiteStep` 函式解析結果，另要求 passed > 0；三項 gate 均為 PASS，沒有 running 0 tests、解析失敗或 failed > 0。這是相關測試的 gate，並非整套 whitelist 已跑完。

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --features teleport-wire-test --test native_teleport_failure -- recovery_wire:: --test-threads=1
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --features teleport-wire-test --lib -- modern_lito::native_login:: --test-threads=1
```

| 命令 | 結束碼 | gate | 日誌 |
|---|---:|---|---|
| `cargo check --all-targets` | 0 | PASS | `001.log`／`check.json` |
| `native_teleport_failure::recovery_wire` | 0 | PASS；7 通過，5 個正式案例被篩除 | `002.log`／`wire.json` |
| `native_login` 相關單元 | 0 | PASS；36 通過，1 個正式負面案例忽略 | `003.log`／`lib.json` |

兩次最終測試的 `test result:` 原文：

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 17.31s
test result: ok. 36 passed; 0 failed; 1 ignored; 0 measured; 2453 filtered out; finished in 19.50s
```

本張新增 7 個純狀態案例與 7 個加密 loopback 案例。loopback 同時經 integration 與 library 入口執行，不能把兩份結果相加當成新增案例數。實際驗證包含：重複 cancel／失敗後 cancel、成功與不符身分的一次性消耗、沒有已授權目的地不得產生恢復、C8 正確 MS950 來源、清單部分到達不得選角、不自動重送 C9、直接傳送不得借用恢復分支、部分背包不重試、兩段自然 8 秒有界等待及取消後不重開連線。現行租裝返回、戰鬥退出與舊 socket 封包退役的相關測試亦通過。`git -c core.whitespace=cr-at-eol diff --check` 通過（保留既有 CRLF 檔案格式）。

首輪控制器單元測試曾退出 101：兩個新純狀態測試用 `inventory.begin(0, ...)` 建立前提，遭現行「connection 必須非零」守衛拒絕，尚未進入待驗行為。已改為有效的合成 owner 1，transport 仍保持未連線以驗 enqueue 失敗；產品守衛未放寬。原始結果保留於 `003-initial-failed.log`／`lib-initial-failed.json`：

```text
test result: FAILED. 34 passed; 2 failed; 1 ignored; 0 measured; 2453 filtered out; finished in 19.60s
```

## 推測

正式伺服器的 30 秒 staged／source handoff 到期可能造成 GAME 先提供預設來源清單；先要求所選來源的 C8 可避免把那份清單開放為手動選角。此推論依現行控制器及 `維護工具/prepare-native-recovery-source-refresh.py`、`prepare-native-recovery-default-source.py` 的契約線索，正式 E149 自然到期行為不能由合成 peer 測試取代。

## 待驗證／未驗證什麼

- 本輪沒有執行 Java、正式 E149、DB、原生 GUI 或遊戲程序。整合測試中原 5 個 opt-in 正式案例被篩除，library 中 1 個正式伺服器負面案例 ignored，均不計通過。
- 33／31 秒正式自然等待與 30 秒伺服器 TTL、完整物品接續、所有權／timer、正式來源預設順序仍須 owned fixture 驗收；不改伺服器時鐘、租期或權威資料。
- Release、完整 library、完整 GUI、G06 640 次計畫與 G01–G12 未因本張取得資格。
- 本張不恢復另外三個舊 scenario、不實作通用 finite History、Presentation／音效、T 鍵、cancel-late 或物品拒絕矩陣。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_teleport_recovery_tests.rs`
- `LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs`
- `LiTo381-Modern/tests/native_teleport_failure.rs`
- 本報告：`docs/reports/issue-169.md`
- 本機日誌與套件 gate 結果：`LiTo381-Modern/target/current/verification/issue-169/`（不入 git）。

## 給下一棒的提示

1. Claude 審查一次性識別消耗、C8 到清單發布的 gate，以及上述 13／14 檔分張對應。
2. T023 第 2 張沿本張現行來源補回三個正式 scenario；31 秒 `observe_source_expiry` 與 `cancel-expiry-source` 驅動分支已準備，尚未新增該正式 test／Java scenario／runner 入口，既有 `cancel-expiry` 維持 stage-only。接入時核對 owned fixture watchdog／lease（舊設計為 440／460 秒），不要改產品期限。合成 peer 的 `recovery_wait` 只建立控制器測試前提，不能作為正式認證、TTL 或租裝成功證據。
3. 本機測試執行者以當版 E149 owned harness 驗證自然 33／31 秒及清單來源；保留原 8／30／60 秒契約，不為取得綠燈延長期限。新 C8 回覆需全新完整 S03／SCB 清單；未到齊不可手動選角。
4. 正式 fixture 若需 production item bootstrap／readonly observer，依當版 Java API 在對應任務重新開發並驗證，不整包搬入歷史 14 檔。`docs/REPORT.md` 保持未改。
