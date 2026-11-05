# TASKS — lito-modern

格式：`ID | 狀態 | 負責 | 範圍 | 驗收條件`。狀態：todo / doing / review / done / blocked。
Cursor 接任務時只改「範圍」列出的路徑；PR 標題用 `ID: 摘要`。所有 cargo 指令在 `LiTo381-Modern/` 下執行。

## 範圍凍結規則（2026-11-06，#92）
- **完成定義凍結**：每個里程碑的「驗收：使用者打開會看到什麼」寫完後不再改寫；要改需開新 Issue 說明理由並標 `needs:user`。
- **主線封頂**：主線＝最近未完成的里程碑 M1 與 V1，**主線剩 4 項，封頂**（T011-H、T011-B、T012、T013）。進度條只算 `## M*` 區段（V1 的標題是 `## M1-V1`）。
- **分母只因「不確定項」兌現而變動**：主線的不確定項見 `PLAN.md`「盤點：已知範圍」的 U-M1-1、U-V1-1、U-V1-2，最多 +3 張。兌現時 PR 說明要引用當初的編號與估算。
- **新發現的工作一律進 `## Backlog`**，不算進度。要升級成主線，必須同時砍掉一項主線，或經使用者同意（`needs:user`）。
- **M2、M3 尚未開放**：它們的完成定義已凍結，任務清單已在盤點中一次列完（M2 24 張、M3 24 張），目前放在 Backlog。等 M1 與 V1 完成後，開新 Issue（`needs:user`），把盤點列好的那一批原樣升級成主線；不得在升級時另外加項目。

## M0 可建置（完成）
驗收：使用者打開會看到什麼（凍結）：GitHub Actions 的 `build.yml` 在 main 最新一次是綠色；`docs/reports/issue-59.md` 有 clone 到其他路徑後 `啟動現代客戶端.ps1 -BuildOnly` 結束碼 0 的實際輸出。
主線 0 項（全部完成）。

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
現況（2026-11-05）：已完成（Issue #3，PR #5）；Windows 原機實際建置與 clone 到其他路徑的腳本行為仍待驗證，交給 T010-F：只有在 clone 位置實際 `-BuildOnly` 成功才算驗證，T010-A 不算 T005 驗收。`build.ps1` 仍是 i686／`target/`，修正交給 T010-G。

## M1 驗證（使用者已於 2026-11-05 同意恢復測試，Issue #43；由本機 dot／codex-local 執行）— 主線剩 2 項，封頂
驗收：使用者打開會看到什麼（凍結）：
- `docs/reports/issue-53.md`、`issue-55.md`、`issue-56.md` 有當版 main 的 check、lib Debug／Release、白名單的實際數字（已有）。
- T011-B 的報告有 G06 當版 5 個 scenario（timeout、refused、wrong-service、cancel-retry、cancel-expiry）各自的實際輸出：`running 1 test`、`1 passed`，以及 result.json 的 sessions／memberships 檢查結果。
- M1 不包含白名單以外的 1055 個待驗測試（574＋481）；它們在 Backlog（M3 廣域回歸），報告中照舊列為待驗證。

### T010 | done | 本機 dot／codex-local＋Claude | `docs/TESTING.md`
當版 main 驗證（2026-11-05，#76 調整範圍）：
- 已驗證：`cargo check --all-targets`，以及 `cargo test --lib` 的 Debug 與 Release，都是 2464 個測試、1889 通過、0 失敗（#53）；T010-S 靜態安全分類（#54）；白名單 ignored 單元測試 1／1（#55）；白名單整合測試 76／76（#56）；T005 建置腳本（#59），包括 clone 到其他路徑的實際 `-BuildOnly`。
- 待驗證：白名單以外的 574 個 ignored 單元測試（GUI 5、DB 119、不明副作用 20、缺前置條件 430），以及 481 個整合測試（GUI 1、DB 1、不明副作用 438、缺前置條件 41），分類見 #54。「不明副作用」共 458 項，要先完成副作用與安全邊界核對才能排執行。另外 `build.ps1` 的實際執行也待驗證。
- 延後（候選資格，不計入 M1）：12 個 gate 案、四元件 inventory、17 原控制＋6 反例（#58 以 not planned 關閉：全是凍結候選的資格檢查）。

**T011 G06（當版）**（已拆為下面的 T011-H、T011-B，本身不計項）：對象是 `tests/native_teleport_failure.rs` 的 5 個 scenario。
舊版的「兩版 16 案 640 次」依賴候選建置，已延後（#61 以 not planned 關閉）。改以當版的 5 個 scenario 為對象：timeout、refused、wrong-service、cancel-retry、cancel-expiry。T011-R（#80）已完成盤點；#82 裁定採用 memory 模式（不需要 DB）。

### T011-H | doing | 本機 codex-local | Java harness、`維護工具/run-teleport-failure.ps1`、Rust 守門（#91）
Java harness 的 ready 契約補上 town／pid，並做 fresh root 檢查；runner：白名單、repo 根、隔離的 temp／JVM options、來源 SHA 綁定、三個 listener 的所有權、java 與 Rust 分開清理、原子 lease；Rust 端拒絕 2000／3306。範圍與驗收見 `docs/plans/issue-82.json`。

### T011-B | blocked(T011-H) | 本機 dot／codex-local | 執行當版 G06 5 個 scenario（#81）
**可排程條件**：T011-H 合併並通過審查，T011-B 就改回 queued。**每案啟動 Rust 前的檢查**：lease、目錄契約、javac、來源綁定、三個 listener 的所有權，任何一項失敗都中止，回報為阻塞。**實跑後的驗收**：`running 1 test` 而且 1 passed；result.json 的 sessions／memberships 檢查通過；退出鏈完整；E149 資源在 memory 模式下的初始化有實際證據。沒有實跑，不能宣稱通過。首個失敗就停止，不自動重試。

## M1-V1 打開就看得到（V1；2026-11-06，#86；兩張平行）— 主線剩 2 項，封頂
驗收：使用者打開會看到什麼（凍結）：
- 打開 `docs/DEMO.md`，GitHub 上直接看到三張實際畫面的截圖（設定畫面、登入畫面不連線、離線場景），每張下方有重現指令與預期畫面。
- 在本機執行 `pwsh -File 維護工具/run-local-test-suite.ps1`，看到 check／lib Debug／lib Release／白名單 ignored／白名單整合測試的總表，結束碼 0。

### T012 | todo | 本機 codex-local | `維護工具/run-local-test-suite.ps1`（新增）、`維護工具/test-whitelist/`（新增）
一個指令跑完本機測試套件：check、lib Debug／Release、白名單 ignored、白名單整合測試，最後印出總表。前置：記錄 pwsh 路徑與版本（7.0 以上），找不到就 blocked；子程序的 TEMP／TMP 指向 `target/current/scratch`，並清除繼承的 `LITO_*`。
驗收：附上完整指令與實際輸出（總表），而且腳本最終結束碼必須是 0：check 結束碼 0；Debug 與 Release 都有有效結果且 failed=0；白名單每一項都是 `running 1 test` 而且通過。出現 0 tests、結果缺失、failed>0 或命令非 0 都算失敗。數字和 #53、#55、#56 比較，合理變動可以說明，但不能免除上述門檻。

### T013 | todo | 本機 dot | `LiTo381-Modern/src/modern_lito/scene_preview.rs`（只加第二次 ShowWindow）、`docs/screenshots/`（新增）、`docs/DEMO.md`（新增）
先修 `scene_preview.rs`：比照登入與設定視窗，在第一次 ShowWindow 之後補 `IsWindowVisible` 檢查與第二次 ShowWindow，讓腳本用 Hidden 啟動 `-OfflineScene` 時視窗仍可見。然後截圖客戶端的三個不連線畫面：`-ConfigureNative` 設定畫面、`-NativeLoginEndpoint` 登入畫面（不按連線）、`-OfflineScene` 離線場景，並寫 `docs/DEMO.md`。
驗收：修正通過 cargo check；照 DEMO.md 的重現指令啟動時三個視窗都確實可見；repo 內有三張對應實際畫面的 PNG，DEMO.md 可以直接顯示圖片，並寫明每張的重現指令與預期畫面。缺任何一張就是未完成（blocked），要如實記錄錯誤與恢復方式，但不能用錯誤紀錄代替截圖。

## M2 技能與 G06 收尾（尚未開放）— 主線 0 項
驗收：使用者打開會看到什麼（凍結）：
- 打開 `docs/SKILLS-STATUS.md`（M2 新增），看到 E149 目錄每個技能一列：送包形狀測試（名稱＋通過與否）、資源／特效核對、「以 E149 為準可驗」或「伺服器待定」。凡是標可驗的列，全部顯示通過。
- 執行 `run-local-test-suite.ps1`，總表多出「技能」與「G06」兩類，結束碼 0。G06 包含當版 5 個 scenario，以及補回的 3 個舊 scenario。
- #97 暗隱術的 12 個 ready-gate 案在 `src/` 的版本全部通過（8 純案＋4 原資源案）。
任務清單：見 Backlog「預定 M2」（24 張，開放時原樣升級）。

## M3 GUI 與交付（尚未開放）— 主線 0 項
驗收：使用者打開會看到什麼（凍結）：
- 在一台乾淨的 Windows x64 主機（沒有 VS 預覽 runtime）上解壓可攜 ZIP，雙擊 `啟動現代客戶端.ps1`，連本機自架的 E149 伺服器，走完競技場流程「登入→大廳→進房→戰鬥→退出」。
- `docs/GUI-ACCEPTANCE.md`（M3 新增）列出 T030 清單的每一項，各附截圖或錄影，以及「通過／失敗／待伺服器」。
- `docs/G01-G12.md`（M3 新增）有 G01–G12 每一項的定義、證據連結與狀態。
- 執行 `run-local-test-suite.ps1 -Full`，Debug 與 Release 的總表結束碼 0；不能執行的項目逐項寫明排除理由。
任務清單：見 Backlog「預定 M3」（24 張，開放時原樣升級）。

## Backlog（不算進度）
本區有兩種項目：一種是 M2／M3 已排定的任務，開放時原樣升級；另一種是排除或延後的工作。2026-11-06 依 #92，原本在 M2／M3 的 T020–T024、T030–T032，以及修補提案 T040，都移到這裡。它們和「打開就看得到」（V1）沒有直接因果關係。

### 預定 M2（24 張；開放時原樣升級，總數不變）
- **T025 | backlog(M2) | Claude | `docs/PROTOCOL-CONTRACT.md`**：版本綁定。寫明 M2／M3 都以 E149（e1498871）為準；對齊第 8 節「待對 e1498871 核對」與 T004-1 已完成之間的矛盾；查清目錄 180 項（E149）與 181 項（2066774）差的是哪一項。（1 張）
- **T026 | backlog(M2) | Claude | `docs/SKILLS-STATUS.md`（新增）**：技能分類。把 181 個技能逐一對到伺服器缺口組（伺服器 T021 目標／Ctrl、T022 自施、T023 受控 NPC、T024 HP 生產端、T025 材料、T028 顯示），分成「以 E149 為準可驗」與「伺服器待定」兩類。後者只驗送包形狀與畫面，不驗與原版同源。（1 張）
- **T020 | backlog(M2) | Cursor | `src/`：#97 暗隱術**：在 `src/` 重寫 ingress clock、專用 queue、Static2941Bank 與資源就緒 gate；共享 callback、重入繪圖 producer 與固定 sample、巢狀 drain 的交錯；把 12 個 ready-gate 案移植到 main。候選第 29–30 列只當設計參考。（2 張）
- **T021 | backlog(M2) | Cursor | `src/`：#98 附加劇毒、#99 影之防護、#100 提煉魔石**（技能目錄 ID，不是 Issue 編號）。（1 張）
- **T027 | backlog(M2) | Cursor | `src/`：候選獨有的 49 個技能重做進 `src`**：12、20–23、25、27、29、30、33、36、37、39–41、44–48、50、51、53、55、56、59、61、62、64–71、73–76、79、80、87–91（97、98 已含在 T020、T021）。依候選檔的依賴邊分 6 批，每批約 8 個；每批都要有送包形狀與收包測試，並通過 `run-local-test-suite.ps1`。（6 張）
- **T022 | backlog(M2) | Cursor | `src/`：剩下的技能（目錄扣掉 main 39 個、候選 49 個與 97–100）**，依目錄分 10 批：101–118（2 張）、129–138（1 張）、精靈 145–176（3 張）、龍騎士 181–195（2 張）、幻術師 201–220 扣掉 206（2 張；21 個 wrapper 技能一律走 11-byte 形狀）。（10 張）
- **T024 | backlog(M2) | Cursor | `src/`**：#87 持續 4183（wire→model 時間、S0B 重送交錯），加上 LT hurt/base（45／46 hurt 相對 gfx、倍率、u32 rollover）。（1 張）
- **T023 | backlog(M2) | Cursor | `src/`、`tests/native_teleport_failure.rs`**：G06 客戶端可獨立完成的部分。第二次 cancel 的一次性 LOGIN 恢復（同端點、同帳號、同來源）；C8 清單確認；8／30／60 秒與 33／31 秒；在 `src` 重整 13 檔 registry 與 fixture 14 檔；補回 3 個舊 scenario（repeated_cancellation、failed_handoff_then_repeated_cancel、expired_stage_and_source_handoff）。（2 張）

### 預定 M3（24 張；開放時原樣升級，總數不變）
- **T040 | backlog(M3) | Claude | lito-launcher-server Issue**：把修補提案四項（租裝返回、同端點傳送、房間最後離開、商人屬性來源）以 Issue 加證據路徑的形式交給伺服器 repo，不搬 ASM 工具。伺服器接收後，刪除本 repo 的 `修補提案/`。（1 張）
- **T033 | backlog(M3) | 本機 codex-local | `docs/G01-G12.md`（新增）**：從使用者電腦的 `工作佇列.json` 取回 G01–G12 的正式定義（只讀，不新增或修改該檔），寫成文件。（1 張）
- **T034 | backlog(M3) | Claude | `docs/GUI-ACCEPTANCE.md`（新增）**：T030 的檢查清單。普通房、租裝入口、旅館返回、Enter 負向、EOF／取消、新技能、owner 退役與傳送恢復；3600／7200 秒自然租期、雙玩家戰鬥退出；實體輸入、像素、IME、多 DPI、聲音。數值 schema 寫明登入器的啟動方式。（1 張）
- **T035 | backlog(M3) | Cursor | `src/`**：已知 GUI 缺陷。Release 長入口提示換行被下方操作列裁切（NATIVE-WORLD-INTEGRATION:156），以及 FrameArrived 逾時／geometry unavailable 的恢復路徑。（1 張）
- **T030 | backlog(M3) | 使用者 | 依 T034 清單做原生 GUI 實機驗收**：真機驗收，`needs:user`。（1 張）
- **T031 | backlog(M3) | Cursor＋本機 | `packaging/**`**：clean host 可攜交付。改用正式版 VC runtime，不依賴 VS 預覽 runtime；寫明最低 Windows 版本；在乾淨主機驗證 LoadLibrary 動態載入閉包、首次設定與中文路徑。（2 張：打包 1、乾淨主機驗證 1）
- **T036 | backlog(M3) | 本機 dot／codex-local＋Claude | 廣域回歸：白名單以外的 1055 個待驗測試**（574＋481，分類見 #54）。依 #92 盤點分 15 張：
  - 檔案寫入所有權審查（83 項）1 張；
  - 跨函式 helper 閉合 4 張（技能／戰鬥 2、世界／渲染 1、背包／登入／傳送 1），審完的新放行項目由 1 張執行；
  - 共享資源根 159 項 2 張；
  - Java peer／TLS 盤點 1 張、執行 2 張；
  - 原機 snapshot 99 項 1 張；
  - 雜項環境變數 40 項 1 張；
  - 自有隔離 DB 盤點 1 張、執行 1 張。
  
  GUI 6 項與 REAL_DB 1 項直接排除，移入 T030 清單。（15 張）
- **T037 | backlog(M3) | Claude | `docs/G01-G12.md`**：G01–G12 逐項稽核，每項附證據連結與狀態。（1 張）
- **T032 | backlog(M3) | Claude | `docs/ARCHITECTURE.zh-TW.md`**：重寫為現況版（現在是流水帳）。（1 張）

### 排除或延後（不在 M0–M3 完成定義內）
- **WORLD 地圖專屬項目**：伺服器端已確認「LiTo 只有競技場，沒有 WORLD 地圖」（lito-launcher-server `docs/maintenance/combat-continuation-current.zh-TW.md:14`）。因此排除：map303 floor gate、水下 shader／phase／scroll、屋頂 runtime、攻城 P 鍵、經典 C_ENTER_WORLD 分支等。競技場與租裝城鎮仍在範圍內。
- **凍結候選資格檢查**：12 gate 案（候選版）、四元件 inventory、17 原控制＋6 反例、舊 G06 16 案 640 次、候選 overlay QA 475 案（#58、#61 以 not planned 關閉，#33 裁定凍結）。
- **G06 正向前提**：非預設物品、source／account、timer。E149 全 Code 掃描找不到建立或排程的 caller，等伺服器端提供生產者再議。
- **伺服器缺陷**：商人屬性來源（例如 29047 戒指火抗 0，應為 10）。客戶端只顯示權威封包，GUI 驗收寫「與伺服器送出的一致」。
- **效能最佳化**：超出 G11 預算之外的優化，例如首次 Debug 載入 28 秒、封存 hash 23.65 秒。
- **TW 資源** `TW13081901.pak` 缺檔：5 個 TW 項目不能替代。
