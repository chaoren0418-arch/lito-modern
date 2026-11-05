# REPORT — lito-modern 本輪紀錄

本檔給調度器／下一棒看「這一輪改了什麼、下一步誰接」。歷史細節在 `docs/HISTORY.md` 與 git。不取代 `PLAN.md`／`TASKS.md`。

## 現況（截至 Issue #2）

- 正常來源 SHA `3E58268A…`；正式 JAR `E1498871…`。
- 最新隔離候選 44 檔（30 Runtime／11 Rust QA／3 Java）**不可套用**；盤點見 `docs/CANDIDATES.md`。
- 475 案全未跑；G06 640 次未跑；#97 未完整。
- 使用者指示測試與桌面暫緩。

## 本輪變更

- Issue #2 / T003：新建 `docs/CANDIDATES.md`，盤點 44 檔隔離候選（候選路徑／對應 `src/`／是否已併入／差異行數／阻擋原因／建議順序）。候選目錄不在 repo，來源為 `docs/HISTORY.md` 與 `維護工具/` 的 prepare／develop 腳本；`continuation-20261003` 已排除。詳 `docs/reports/issue-2.md`。
- Issue #3（T005）：`啟動現代客戶端.ps1` 與 `LiTo381-Modern/tools/build.ps1` 不再把客戶端資源寫死成唯一路徑；改讀 `LITO_CLIENT_ROOT`（未設時預設仍為 `D:\lineage開發\0-客戶端`），專案／Cargo 路徑改相對 repo 根。Cargo 參數維持原樣（啟動腳本 `--locked --offline -j1` + `target/current`；`build.ps1` 仍 `--offline --locked` + `target` + `i686-pc-windows-msvc`）。兩腳本支援 `-WhatIf`，必要路徑缺失時丟明確錯誤。詳見 `docs/reports/issue-3.md`。

## 下一步

- Issue #2 / T003：Claude 審閱 `docs/CANDIDATES.md`，決定整包或不套用（不要單檔 cherry-pick）。有候選本體時再補 SHA 與 diff 行數。T020（#97）仍等此審閱；恢復測試仍要使用者同意（T010）。
- Issue #3：請 Claude 審查 `task/3` PR，確認 CI `cargo check` 綠、`-WhatIf` 與缺路徑錯誤可接受後合併。測試檔與 `維護工具/` 裡其餘 `D:\lineage開發\...` 硬編不在本任務範圍。
- 2026-11-04 — #3 T005: 啟動與建置腳本去硬編路徑（LITO_CLIENT_ROOT）（code，codex-local，PR #5 已合併）→ [報告](reports/issue-3.md)
- 2026-11-04 — #4 T004: docs/PROTOCOL-CONTRACT.md 協定契約（Claude 撰寫）（design，claude-chat，分支 task/4）→ [報告](reports/issue-4.md)
- 2026-11-05 — #4 T004: docs/PROTOCOL-CONTRACT.md 協定契約（Claude 撰寫）（design，claude-chat，PR #12 已合併）→ [報告](reports/issue-4.md)
- 2026-11-05 — #13 T004-1: T004-1: 以正式 JAR e1498871 靜態核對 PROTOCOL-CONTRACT 常數（resource，codex-local，PR #19 已合併）→ [報告](reports/issue-13.md)
- 2026-11-05 — #15 T004-3: T004-3: NATIVE-PROTOCOL/LOGIN/GAME/LOBBY/ROOM 文件改指向 PROTOCOL-CONTRACT（plan，claude-chat，PR #24 已合併）→ [報告](reports/issue-15.md)
- 2026-11-05 — #16 T004-4: T004-4: C_USE_SPELL 技能 (row,column) → 尾段形狀對照表（design，claude-chat，PR #25 已合併）→ [報告](reports/issue-16.md)
- 2026-11-05 — #21 T004-5: PROTOCOL-CONTRACT 補 S_LITO_GAME_INFO 0xD5（房間摘要子型別）（design，claude-chat，PR #28 已合併）→ [報告](reports/issue-21.md)
- 2026-11-05 — #14 T004-2: T004-2: Rust 集中 opcode 常數表（native_opcodes.rs）（code，codex-local，PR #29 已合併）→ [報告](reports/issue-14.md)
- 2026-11-05 — #32 主控：同步 TASKS.md 狀態（調度台進度圖的資料來源）（plan，claude-chat，PR #38 已合併）→ [報告](reports/issue-32.md)
- 2026-11-05 — #33 T003-R: 裁定 44 檔隔離候選的併入策略（整包／分批／不套用）（design，claude-chat，PR #40 已合併）→ [報告](reports/issue-33.md)
- 2026-11-05 — #41 T003-R1: T003-R1: 44 檔隔離候選依 contract 取檔、SHA256、src/ diff 統計與依賴圖（本機靜態）（resource，codex-local，PR #45 已合併）→ [報告](reports/issue-41.md)
- 2026-11-05 — #46 T003-R2: 依 #41 結果更新 CANDIDATES.md（SHA256、diff 統計、依賴圖結論）（design，claude-chat，PR #50 已合併）→ [報告](reports/issue-46.md)
- 2026-11-05 — #43 主控：使用者同意恢復測試（解除 T010），請排 M1 測試任務（plan，claude-chat，PR #52 已合併）→ [報告](reports/issue-43.md)
- 2026-11-05 — #60 T010-G: T010-G: build.ps1／build-evidence.ps1 對齊 x86_64-pc-windows-msvc、target/current、-j1（code，codex-local，PR #62 已合併）→ [報告](reports/issue-60.md)
- 2026-11-05 — #57 T010-D: T010-D: 盤點 M1 gate 定義（12 gate 案／四元件 inventory／17 原控制＋6 反例／G06 16 案 640 次）（resource，dot，PR #66 已合併）→ [報告](reports/issue-57.md)
- 2026-11-05 — #53 T010-A: T010-A: 本機單元測試基線（cargo check＋cargo test --lib，Debug／Release）（test，dot，PR #67 已合併）→ [報告](reports/issue-53.md)
- 2026-11-05 — #59 T010-F: T010-F: 本機驗證 T005 建置腳本（-WhatIf／-BuildOnly，以及 clone 到其他路徑）（test，dot，PR #69 已合併）→ [報告](reports/issue-59.md)
- 2026-11-05 — #54 T010-S: T010-S: ignored 單元測試與整合測試的安全分類（靜態，不執行）（resource，dot，PR #71 已合併）→ [報告](reports/issue-54.md)
- 2026-11-05 — #55 T010-B: T010-B: 本機逐一執行白名單內的 ignored 單元測試（test，dot，PR #73 已合併）→ [報告](reports/issue-55.md)
- 2026-11-05 — #56 T010-C: T010-C: 本機逐一執行白名單內的整合測試（loopback 隔離 harness）（test，dot，PR #75 已合併）→ [報告](reports/issue-56.md)
- 2026-11-05 — #76 M1-R: M1 範圍改為當版 main；候選資格 gate 延後；G06 以 tests/native_teleport_failure.rs 重新設計（design，claude-chat，PR #79 已合併）→ [報告](reports/issue-76.md)
- 2026-11-06 — #82 T011-H: 設計當版 G06 owned harness 的最小開發範圍（依 #80 盤點）（design，claude-chat，PR #89 已合併）→ [報告](reports/issue-82.md)
- 2026-11-06 — #91 T011-H: T011-H: 當版 G06 owned harness 適配（Java ready 契約＋runner 隔離／綁定／所有權／清理／lease＋Rust 守門）（code，codex-local，PR 已合併）→ [報告](reports/issue-91.md)
- 2026-11-06 — #86 主控：任務切大一點＋排一個「打開就看得到」的里程碑（使用者 2026-11-06 指示）（plan，claude-chat，PR #98 已合併）→ [報告](reports/issue-86.md)
- 2026-11-06 — #102 T013: T013（V1）: 離線場景視窗在隱藏啟動下仍可見（scene_preview.rs）＋三個不連線畫面截圖＋docs/DEMO.md（code，dot，PR #105 已合併）→ [報告](reports/issue-102.md)
- 2026-11-06 — #81 T011-B: T011-B: 本機執行當版 G06 5 個 scenario（依 T011-R）（test，codex-local，PR #110 已合併）→ [報告](reports/issue-81.md)
- 2026-11-06 — #101 T012: T012（V1）: 一個指令跑完本機測試套件並印出總表（run-local-test-suite.ps1）（code，codex-local，PR 已合併）→ [報告](reports/issue-101.md)
- 2026-11-06 — #106 T092-R: #92 步驟 0 本機盤點——G01–G12 定義原文、44 候選檔 545 測試清單、伺服器未追蹤 build 計數（resource，dot，PR #107 已合併）→ [報告](reports/issue-106.md)
- 2026-11-06 — #111 T011-F: G06 首跑失敗修正——TW 特效資源資格不符時不中止連線（Disintegrate／cone cold／call lightning）（code，codex-local，PR #115 已合併）→ [報告](reports/issue-111.md)
