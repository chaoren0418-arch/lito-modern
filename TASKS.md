# TASKS — lito-modern

格式：`ID | 狀態 | 負責 | 範圍 | 驗收條件`。狀態：todo / doing / review / done / blocked。
Cursor 接任務時只改「範圍」列出的路徑；PR 標題用 `ID: 摘要`。所有 cargo 指令在 `LiTo381-Modern/` 下執行。

## M0 可建置

### T001 | done | Claude | `.github/workflows/build.yml`
CI：windows-latest `cargo check --locked --all-targets` + `cargo test --lib`（允許失敗，先看結果）。
驗收：check 綠；test 結果列在 PR／commit 說明。

### T002 | done | Claude | `LiTo381-Modern/src/**`（只改測試 cfg）, `logs/continuation-20261003/**`, `docs/TESTING.md`
把需要視窗／socket／原資源的測試標 `#[ignore]` 並加註原因（`// requires: window | socket | assets`），讓 `cargo test --lib` 與 `cargo test --tests` 在乾淨機器可跑完。
驗收：CI `cargo test` 不再 continue-on-error 也能綠；ignored 數量與原因列在 `docs/TESTING.md`。

### T003 | done | Cursor | `docs/CANDIDATES.md`（新建）, `維護工具/candidates/**`（唯讀）
盤點 44 檔隔離候選：每檔對應 `src/` 哪個檔、是否已併入、差異行數、阻擋原因。
驗收：表格完整；Claude 審閱後決定併入順序。
現況（2026-11-05）：盤點已完成（Issue #2）。Claude 已裁定（Issue #33，`docs/CANDIDATES.md`「裁定」一節）：現在不套用，凍結為參考。等 M1 完成，且 T003-R1 的依賴分析回來後，再重新裁定可行的批次（目前的 G1／G2／G3 只是研究分組，彼此有交錯依賴）。

### T004 | done | Claude | `docs/PROTOCOL-CONTRACT.md`（新建）
寫與 lito-launcher-server 0.6.71 的協定契約：登入握手、版本、加密、必用 opcode 清單，取代純 SHA pin。
驗收：Cursor 能依此文件不看 JAR 寫 client 端。
現況（2026-11-05）：已完成（Issue #4，PR #12）。後續子任務：T004-1 以正式 JAR e1498871 靜態核對（#13，done）、T004-3 NATIVE-* 文件改指向契約（#15，done）、T004-4 C_USE_SPELL 技能尾段對照（#16，done）、T004-5 補 S_LITO_GAME_INFO（#21，done）、T004-2 Rust 集中 opcode 常數表（#14，PR #29，done）。

### T005 | done | Cursor | `啟動現代客戶端.ps1`, `LiTo381-Modern/tools/build.ps1`
去硬編路徑（`D:\lineage開發\...`）改相對 repo 根或環境變數 `LITO_CLIENT_ROOT`。
驗收：clone 到任意路徑可建置。
現況（2026-11-05）：已完成（Issue #3，PR #5）；Windows 原機實際建置由 T010-A 驗證。

## M1 驗證（使用者已於 2026-11-05 同意恢復測試，Issue #43；由本機 dot／codex-local 執行）

### T010 | todo | 本機 dot／codex-local＋Claude | `docs/TESTING.md`
跑 12 個 gate 案、四元件 inventory、17 原控制／6 反例；結果與失敗收據記錄。
拆分（`docs/plans/issue-43.json`）：T010-A 單元測試基線 → T010-B 需資源的 ignored 測試 → T010-C 整合測試；T010-D 盤點 gate 定義 → T010-E 跑 gate 與 inventory。首個失敗保存原始收據，另開 bug Issue。

### T011 | blocked(T010) | 本機 dot／codex-local＋Claude | G06 兩版 16 案 640 次
核對 PID／FILETIME／Job／DB 退出、33／31 秒、32＋8 雙 trace。首個失敗停止，不自動重試。

## M2 #97–#100 與 G06

### T020 | blocked(T010) | Cursor | #97：共享 callback、重入繪圖 producer、固定 sample／巢狀 drain 交錯
在 `src/` 直接開發，不以候選第 29–30 列為基底（見 CANDIDATES.md「裁定」）；PLAN 把 #97 放在 M2，要等 M1 驗證完成。
合法時機可提交；callback 後不能再發佈；不固定一幀、不借 32 秒期限。

### T021 | blocked(T020) | Cursor | #98 附加劇毒、#99、#100
### T022 | blocked(T021) | Cursor | 剩 92 技能，依使用頻率排序（Claude 給順序）
### T023 | todo | Cursor | G06：第二次 cancel 一次性 LOGIN 恢復、C8 清單確認、8／30／60 秒與 33／31 秒；13 檔 registry + fixture 14 檔套用
### T024 | todo | Cursor | #87 持續 4183、LT hurt/base、G06 物品／source/account／timer 正向前提

## M3 GUI 與交付

### T030 | blocked(M2) | 使用者+Cursor | 完整 GUI 驗收清單（普通房、租裝、旅館返回、Enter 負向、EOF／取消、新技能、退役／傳送恢復；IME／多 DPI／聲音）
### T031 | todo | Cursor | `packaging/**`
clean host 可攜交付：不依賴 VS 預覽 runtime；最低 Windows 版本明確；動態載入驗證。
### T032 | todo | Claude | `docs/ARCHITECTURE.zh-TW.md` 重寫為現況版（現在是流水帳）

## 修補提案（`修補提案/`，屬於伺服器端，應移到 lito-launcher-server）
- 租裝返回、商人屬性來源、房間最後離開、同端點傳送：四個 Java patch 候選。T040 | todo | Claude：評估後搬到 lito-launcher-server 開 PR，這裡刪除。
