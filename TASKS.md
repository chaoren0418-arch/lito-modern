# TASKS — lito-modern

格式：`ID | 狀態 | 負責 | 範圍 | 驗收條件`。狀態：todo / doing / review / done / blocked。
Cursor 接任務時只改「範圍」列出的路徑；PR 標題用 `ID: 摘要`。所有 cargo 指令在 `LiTo381-Modern/` 下執行。

## 範圍凍結規則（2026-11-06，#92）
- **完成定義凍結**：每個里程碑的「驗收：使用者打開會看到什麼」寫完後不再改寫；要改需開新 Issue 說明理由並標 `needs:user`。
- **主線封頂**：M0、M1、V1 已全部完成（11 項）。2026-11-06 使用者同意開放 M2（#132），M2 的 23 項（57 張）原樣升級成主線。主線分母以項計由 11 變成 34，以張計是 11＋57＝68，兩者是同一份範圍的兩種計數，封頂。
- **分母只因「不確定項」兌現而變動**：U-M1-1 已兌現為 T011-F；U-V1-1、U-V1-2 沒有兌現，已關閉。M2 的不確定項 U-M2-1～3 兌現時，最多 +8 張（見 PLAN），兌現時 PR 要引用編號與估算。
- **新發現的工作一律進 `## Backlog`**，不算進度。要升級成主線，必須同時砍掉一項主線，或經使用者同意（`needs:user`）。
- **M2 已開放（2026-11-06，#132）；M3 尚未開放**：完成定義都已凍結。M3（44 張）放在 Backlog，等 M2 完成後再開 Issue 經使用者同意，原樣升級；不得在升級時加項目。

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

## M1 驗證（使用者已於 2026-11-05 同意恢復測試，Issue #43；由本機 dot／codex-local 執行）— 主線 0 項（完成）
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

### T011-H | done | 本機 codex-local | Java harness、`維護工具/run-teleport-failure.ps1`、Rust 守門（#91，PR #95；修正 #122）
#118 首案在 Rust 啟動前阻塞：`run-teleport-failure.ps1:114` 在 Java 剛啟動時讀 `MainModule.FileName` 失敗，身分沒綁定，清理守門就拒絕 kill。修正 #122：改用 `QueryFullProcessImageNameW` 在期限內重試；未綁定時用原始 handle 清理；保存日誌。安全契約不放寬。
Java harness 的 ready 契約補上 town／pid，並做 fresh root 檢查；runner：白名單、repo 根、隔離的 temp／JVM options、來源 SHA 綁定、三個 listener 的所有權、java 與 Rust 分開清理、原子 lease；Rust 端拒絕 2000／3306。範圍與驗收見 `docs/plans/issue-82.json`。

### T011-F | done | 本機 codex-local（PR #115） | `native_disintegrate.rs`、`native_cone_cold.rs`、`native_call_lightning.rs`（#111）
U-M1-1 兌現：#81 首跑 timeout 失敗，原因是角色 A 進世界時預載 77 Disintegrate 的特效資源，資格檢查不符（`disintegrate_assets.rs:207`），錯誤一路往上丟，中止了整個連線。修正方向：特效資源資格不符時，改標「呈現不可用」並記錄原因，不中止連線或流程；不放寬資格檢查，也不替換資源。範圍與驗收見 #111。

### T011-B | done | 本機 codex-local | 執行當版 G06.3d 5 個 scenario（#118，PR #129：5／5 通過，每案 `running 1 test`、1 passed；首跑 #81 失敗，由 T011-F 修正；#118 第 1 次被 runner 阻塞，由 #122 修正）
**可排程條件**：T011-H 與 T011-F 都已合併並通過審查（PR #95、PR #115），重跑 #118 已排入佇列。**每案啟動 Rust 前的檢查**：lease、目錄契約、javac、來源綁定、三個 listener 的所有權，任何一項失敗都中止，回報為阻塞。**實跑後的驗收**：`running 1 test` 而且 1 passed；result.json 的 sessions／memberships 檢查通過；退出鏈完整；E149 資源在 memory 模式下的初始化有實際證據。沒有實跑，不能宣稱通過。首個失敗就停止，不自動重試。

## M1-V1 打開就看得到（V1；2026-11-06，#86；兩張平行）— 主線 0 項（完成）
驗收：使用者打開會看到什麼（凍結）：
- 打開 `docs/DEMO.md`，GitHub 上直接看到三張實際畫面的截圖（設定畫面、登入畫面不連線、離線場景），每張下方有重現指令與預期畫面。
- 在本機執行 `pwsh -File 維護工具/run-local-test-suite.ps1`，看到 check／lib Debug／lib Release／白名單 ignored／白名單整合測試的總表，結束碼 0。

### T012 | done | 本機 codex-local（#101） | `維護工具/run-local-test-suite.ps1`（新增）、`維護工具/test-whitelist/`（新增）
一個指令跑完本機測試套件：check、lib Debug／Release、白名單 ignored、白名單整合測試，最後印出總表。前置：記錄 pwsh 路徑與版本（7.0 以上），找不到就 blocked；子程序的 TEMP／TMP 指向 `target/current/scratch`，並清除繼承的 `LITO_*`。
驗收：附上完整指令與實際輸出（總表），而且腳本最終結束碼必須是 0：check 結束碼 0；Debug 與 Release 都有有效結果且 failed=0；白名單每一項都是 `running 1 test` 而且通過。出現 0 tests、結果缺失、failed>0 或命令非 0 都算失敗。數字和 #53、#55、#56 比較，合理變動可以說明，但不能免除上述門檻。

### T013 | done | 本機 dot（#102，PR #105） | `LiTo381-Modern/src/modern_lito/scene_preview.rs`（只加第二次 ShowWindow）、`docs/screenshots/`（新增）、`docs/DEMO.md`（新增）
先修 `scene_preview.rs`：比照登入與設定視窗，在第一次 ShowWindow 之後補 `IsWindowVisible` 檢查與第二次 ShowWindow，讓腳本用 Hidden 啟動 `-OfflineScene` 時視窗仍可見。然後截圖客戶端的三個不連線畫面：`-ConfigureNative` 設定畫面、`-NativeLoginEndpoint` 登入畫面（不按連線）、`-OfflineScene` 離線場景，並寫 `docs/DEMO.md`。
驗收：修正通過 cargo check；照 DEMO.md 的重現指令啟動時三個視窗都確實可見；repo 內有三張對應實際畫面的 PNG，DEMO.md 可以直接顯示圖片，並寫明每張的重現指令與預期畫面。缺任何一張就是未完成（blocked），要如實記錄錯誤與恢復方式，但不能用錯誤紀錄代替截圖。

## M2 技能與 G06 收尾（2026-11-06 使用者同意開放，#132）— 主線 23 項（57 張），封頂
驗收：使用者打開會看到什麼（凍結）：
- 打開 `docs/SKILLS-STATUS.md`（M2 新增），看到 E149 目錄每個技能一列：送包形狀測試（名稱＋通過與否）、資源／特效核對、「以 E149 為準可驗」或「伺服器待定」。凡是標可驗的列，全部顯示通過。
- 執行 `run-local-test-suite.ps1`，總表多出「技能」與「G06」兩類，結束碼 0。G06 包含當版 5 個 scenario，以及補回的 3 個舊 scenario。
- #97 暗隱術的 12 個 ready-gate 案在 `src/` 的版本全部通過（8 純案＋4 原資源案）。
任務清單：下列 23 項共 57 張，由 #132 原樣從 Backlog 升級，依 `docs/plans/issue-132.json` 建立 Issue。只有 U-M2-1～3 兌現時可以增加張數，上限 +8。
技能集合以 `docs/PROTOCOL-CONTRACT.md` §6.3 為準：git `2066774`，共 181 項，也就是 180 個 catalog 記錄加上 dispatcher 特例 5 TELEPORT。E149 與 `2066774` 的 `BattleSpellCatalog` 常數 223／223 完全相同，`new *Spell(` 也同為 186 處。逐 ID 去重後的分配如下，合計 38＋48＋4＋91＝181，沒有重複：

| 集合 | 數量 | ID |
|---|---|---|
| main 已接 | 38 | 1–4、6–11、13–19、26、28、31、32、34、35、38、42、43、49、52、54、57、58、60、63、72、77、78、131、206 |
| 候選獨有 | 48 | 見 T027 |
| #97–#100 | 4 | 97–100 |
| 未接 | 91 | 見 T022 |

### T025 | todo | Claude | `docs/PROTOCOL-CONTRACT.md`（1 張）
：版本綁定。寫明 M2／M3 都以 E149（e1498871）為準；對齊第 8 節「待對 e1498871 核對」與 T004-1 已完成之間的矛盾。另外把 180／181 的來源寫進契約：180 個 catalog 記錄，加上 TELEPORT 是 dispatcher 特例（`維護工具/generate-spell-request-shapes.py:192`）。再寫一份「伺服器換版時客戶端要重核」的清單：JAR SHA 與血統分支、`Protocol$*` 變動與 opcode／欄位、ClientSession／GameHub 變動 class、技能結果與狀態圖示 ID、房間與結算、移動與傳送、裝備與租賃、GM 指令、登入流程、相容 DLL 與 hook 位址（見報告附錄 D）。另外依伺服器 maintenance 全讀結果，重核清單還要加上：候選協定變更（ICONS1、ICON_I2H、OwnCharStatus、1016／1017、娃娃 29 的 buffDuration `writeH`、S_HPMeter 104 的 hide255、施法時間窗與 SprTable 截斷、裝備只送一份最終描述、舊角色封包抑制、173／174 的 opcode 修正、portal 在落點前改 showId）；Maven 產物和 E149 的差異（`NativeDeVisibilitySwitch` 缺失、1320 個 Modern* 類、49 個非 Modern 類，`docs/server-source-gap.md:37-1454`）；來源伺服器清單（8 格原生限制，目前只啟用 380_ac_1／2）；`TW13081901.txt` 的 pin。注意：maintenance 目錄最新的正式 JAR 紀錄是 0.6.68 `c02aaae8`，0.6.69–0.6.71 要從登入器 repo 的 release 紀錄取得。依 #120 新 N3：同一個 builder SHA 可能對應不同的輸入 Java 與不同的候選 JAR，所以重核清單要用 branch／manifest pin 追溯血統，不能只比 builder SHA；`df9464f0` 底下的兩條 magical-weapon 後繼不能混為同一條鏈。（1 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 版本綁定、§8 對齊、180／181 來源、伺服器換版重核清單（全部）

### T026 | todo | Claude | `docs/SKILLS-STATUS.md`（新增）（2 張）
：技能分類。把 181 個技能逐一對到伺服器缺口組（伺服器 T021 目標／Ctrl、T022 自施、T023 受控 NPC、T024 HP 生產端、T025 材料、T028 顯示），分成「以 E149 為準可驗」與「伺服器待定」兩類。後者只驗送包形狀與畫面，不驗與原版同源。另外依 G05 的 work，把範圍擴大成**玩法支援矩陣**：所有玩家可達的操作（技能、物品使用／裝卸／丟棄／拾取、遠程與彈藥、耗材／MP、buff／debuff、復活），每一列標明封包、權威狀態、呈現、失敗路徑，以及缺失／拒絕／不適用。G05.4 商人附加屬性要寫明「依伺服器權威值顯示」。另外要納入：每個技能的目標類型與是否需要 Ctrl（與 T061 對齊）；訊息 ID 278、279、299、1339、1412；處理器待查的 192、202、203、207、208、213；131 停用、39 個保留槽（`combat-coverage-current.json` 的 explicit_reserved_slots：SQL 保留 38＋catalog 保留 1；舊文件 `resource-database-0.6.62.md:27` 記 34 個）、208 暈眩 2／3 秒、4016、49011–49017 別名；技能 90 的裝備模式、BUFF 3／151 衝突、武器 Proc 未接、材料 25／58；卡瑞／莎爾 NPC 與 map1005 的範圍判定。資料來源：伺服器 `combat-coverage-current.json`（808 列，其中 180＋1 列對應本目錄 181）。依 #120 補：技能 68 的目標生命週期（新 N1，與 T061 對齊）；隱身重施時，指定隱身裝備 20077／120077／20062 的 toggle 例外（新 N2，與 T045、T047 對齊；先由 T025 確認正式版是否承襲）。（2 張：技能 1、物品與其他操作 1）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 技能狀態表 `docs/SKILLS-STATUS.md`：181 技能逐列（目標類型、是否需要 Ctrl、可驗／伺服器待定、訊息 ID、保留槽、別名等技能欄位）
  2. 玩法支援矩陣：技能以外的玩家可達操作（物品使用／裝卸／丟棄／拾取、遠程與彈藥、耗材／MP、buff／debuff、復活），以及 #120 新 N2 的隱身裝備例外

### T020 | todo | Cursor | `src/`：#97 暗隱術（2 張）
：在 `src/` 重寫 ingress clock、專用 queue、Static2941Bank 與資源就緒 gate（在 enqueue 前，原 4E5C00 先 SPR 再 4D65F0）。範圍包括：
  - 共享 callback、重入繪圖 producer 與 4D6570 固定 sample 的交錯；4E2150 兩個 suffix 的上游 CFG；4857B0 巢狀 drain；方向 6 alias／shadow defaults；
  - native clock 的 import 初始化與 rollover；
  - 把 12 個 ready-gate 案移植到 main；重編 library-test 與七個整合 metadata（HISTORY L1–L13）。

  候選第 29–30 列只當設計參考。（2 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. ingress clock、專用 queue、Static2941Bank、資源就緒 gate，以及 native clock 的 import 初始化與 rollover
  2. 共享 callback、重入繪圖 producer 與 4D6570 固定 sample 的交錯、4E2150 suffix、4857B0 巢狀 drain、方向 6 alias／shadow；12 個 ready-gate 案移植；library-test 與七個整合 metadata 重編

### T021 | todo | Cursor | `src/`：#98 附加劇毒、#99 影之防護、#100 提煉魔石（1 張）
（技能目錄 ID，不是 Issue 編號）。97／98 要驗權威 S34／S08 現身、材料／低 MP／HP 邊界、原資源／音效／History、自然期限（L13）。（1 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 98、99、100 全部，加上 97／98 的權威驗證

### T027 | todo | Cursor | `src/`：候選獨有的 48 個技能重做進 `src`（6 張）
，逐 ID 分 6 批，每批 8 個：
  - 批 1：5、12、20–23、25、27
  - 批 2：29、30、33、36、37、39–41
  - 批 3：44–48、50、51、53
  - 批 4：55、56、59、61、62、64–66
  - 批 5：67–71、73–75
  - 批 6：76、79、80、87–91

  5 TELEPORT 和 G06 有關，要跟 T023 對齊。各技能的歷史待測條件見 HISTORY L17–L65，例如 #69 的 S55-C4E-S5A-C08、EOF 取消與八秒失敗、#88／#89 的 HP 成本邊界 279／278。每批都要有送包形狀與收包測試，並通過 `run-local-test-suite.ps1`。分批可以依候選依賴邊在 6 批之間調換，但總數不變。（6 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 批 1：技能 5、12、20–23、25、27
  2. 批 2：技能 29、30、33、36、37、39–41
  3. 批 3：技能 44–48、50、51、53
  4. 批 4：技能 55、56、59、61、62、64–66
  5. 批 5：技能 67–71、73–75
  6. 批 6：技能 76、79、80、87–91

### T022 | todo | Cursor | `src/`：未接的 91 個技能（10 張）
，分 10 批：
  - 批 1：101–109；批 2：110–118（107 在候選有參考實作 `tests/native_shadow107.rs`，見 #106 與報告附錄 E2；只當設計參考）
  - 批 3：129、130、132–134、137、138
  - 批 4：145–155；批 5：156–166；批 6：167–176
  - 批 7：181–188；批 8：189–195
  - 批 9：201–205、207–210；批 10：211–220

  21 個 wrapper 技能一律走 11-byte 的 ⑩ 形狀；113 TRUE_TARGET 帶 MS950 文字。共用 `native_skills::selector_only` helper，coverage parser 也要涵蓋（L11、L13、L17）。（10 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 批 1：技能 101–109
  2. 批 2：技能 110–118
  3. 批 3：技能 129、130、132–134、137、138
  4. 批 4：技能 145–155
  5. 批 5：技能 156–166
  6. 批 6：技能 167–176
  7. 批 7：技能 181–188
  8. 批 8：技能 189–195
  9. 批 9：技能 201–205、207–210
  10. 批 10：技能 211–220

### T024 | todo | Cursor | `src/`（2 張）
：時序與 hurt，分 2 張：
  - #87 持續 4183：wire→model 時間、S0B 重送交錯、idle 標籤守衛（L1–L25）。
  - LT hurt／base：原 hurt guard、LT 相對 gfx 與初始化（不得猜 base）、action0 速度倍率、uint32 長 uptime rollover、#45 class7 child hurt 排除（L49–L65）；hurt queue threshold（L1109）；特殊變身與動作中斷分支（L713、L751）。

  （2 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. #87 持續 4183：wire→model 時間、S0B 重送交錯、idle 標籤守衛
  2. LT hurt／base、action0 速度倍率、uint32 rollover、#45 class7 child hurt、hurt queue threshold、特殊變身與中斷分支

### T028 | todo | Cursor＋本機 | `src/`、`tests/`：main 已接 38 技能的當版補驗（4 張）
。這 38 個只有舊來源的限定資格（HISTORY L93–L135「只保留原版本資格」）。已知缺口：
  - 13 偵測術、60 隱身術：Release 真實施法揭露；SELF_TICK 原生呼叫者時鐘（L93–L149）。
  - 19／35／57 較高治癒：其他目標（L327、L1265）。
  - 2 日光術：夜間亮度、3600 秒自然到期（L1282–L1304）。
  - 72：3934 空影格的專屬 SHA／slots policy（L1580、L1584）。
  - 77 Disintegrate：舊來源路徑（L1425）。
  - 6 冰箭：全 caster golden（L1026）。
  - 8、9、11、10、28、18：各自的其他目標、clock 邊界、自然到期（L1070–L1131）。
  - 63：320 秒（L635）。

  另外要在 `src` 重寫候選裡對 main 既有技能的修正：S23 class4 與冰箭的 NativeSpellClock 時序、全域整數毫秒起點、class1 啟動位置、火球 class5 時序（`維護工具/native-E149-S23-clock*.md`、`native-E149-class1-origin.md`、`native-E149-fireball-*.md`）；處理 ignored 的全 caster golden 測試，以及 list.spr 全量嚴格 parser 的 614 行錯誤（NATIVE-ATTACK-SKILLS:42）。
  分 4 張：攻擊與目標類、狀態與自身類、地面、特效與資源類、候選修正重寫。（4 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 攻擊與目標類：6、8、9、10、11、18、19、28、35、57，以及全 caster golden
  2. 狀態與自身類：2、13、60（含 SELF_TICK 時鐘）
  3. 地面、特效與資源類：63、72（3934 空影格）、77（Disintegrate 舊路徑），以及 list.spr 614 行 parser 錯誤
  4. 候選修正在 `src` 重寫：S23 class4／冰箭 NativeSpellClock、全域毫秒起點、class1 啟動位置、火球 class5 時序

### T029 | todo | Cursor | `src/`：非技能的戰鬥基礎（2 張）
，分 2 張：
  - 遠程：D722 起新增的 ResourceDatabase.arrow／flightType 映射，舊弓箭收據不能當現版驗收（L1106、L1109）；弓／投擲 stance20／62；S23 其他 type；特殊投射物（NATIVE-COMBAT:20、:22、NATIVE-RANGED:21）。
  - 操作：alpha 命中、自動追擊、目標圈、拒絕提示、攻擊專用速度、延遲回覆與重試交錯（L753、NATIVE-COMBAT:36、:38、NATIVE-NAVIGATION:23、NATIVE-OUTCOMES:19、:34）。

  另外包括空揮封包 `17 00 00 00 00 DE 7F 1D 80`（target 0、damage 0），以及它的拒絕條件：死亡、觀戰、隱身、麻痺、超重、座標不合法（`empty-melee-0.6.49.json` L34–42）。（2 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 遠程：arrow／flightType 映射、弓／投擲 stance20／62、S23 其他 type、特殊投射物
  2. 操作：alpha 命中、自動追擊、目標圈、拒絕提示、攻擊專用速度、延遲回覆與重試交錯、空揮封包與拒絕條件

### T023 | todo | Cursor | `src/`、`tests/native_teleport_failure.rs`：G06.3d 傳送與失敗恢復中，客戶端可以獨立完成的部分（4 張）
，分 4 張（下面 4 段各 1 張）：
  - 第二次 cancel 的一次性 LOGIN 恢復（同端點、同帳號、同來源）；C8 清單確認；8／30／60 秒與 33／31 秒；在 `src` 重整 13 檔 registry 與 fixture 14 檔。
  - 補回 3 個舊 scenario（repeated_cancellation、failed_handoff_then_repeated_cancel、expired_stage_and_source_handoff）；更多取消／舊世代、畸形或跨世代 wire 回覆（L1155、L1172、L1227）；G06 通用 finite History 四 profile（L45）。
  - G06.3d 旅館、傳送與跨端點服務：TeleportMe 原圖示排版（L1150）、同端點以外的 Presentation／音效／其他服務（`docs/G01-G12.md` G06.3d 的 state）；觀戰移到 T054。

  - G06 物品接續中可以到達的拒絕與期限案例：StateCopy 拒絕 active ScheduledFuture／EquipmentTimerLease／pending magic-stone expiry、租裝自然到期（`維護工具/native-E149-extended-item-observation.md:15-27`）；T 鍵入口與 cancel-late 案例（`native-E149-integrated-client.md:15`、`native-E149-recovery-case-contract.md:11`）；畸形與跨世代 wire 回覆（NATIVE-WORLD-INTEGRATION:126、:164）。

  同端點 C02（selection=0 的 rental-item-set，L1230）依賴伺服器，列在 U-M3-1。沒有正常 GAME 產生路徑的非預設值（元素、鑲卡、魔石、覺醒、dark-item payload）歸「G06 正向前提」排除。另外要涵蓋沒有 S_PORTAL 的解鎖與恢復：奪命之雷開啟時技能 5／131 不建立 portal（`native-thunder-teleport-policy-progress.zh-TW.md:5`）、技能 69 失敗時補送 4-byte 解鎖（`source-mass-teleport-exit.zh-TW.md:15`）；以伺服器權威座標與 showId 為準（`combat-integration-lifecycle.zh-TW.md:86`）。（4 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 第二次 cancel 的一次性 LOGIN 恢復、C8 清單、8／30／60 與 33／31 秒、13 檔 registry 與 fixture 14 檔
  2. 補回 3 個舊 scenario、更多取消／舊世代、G06 通用 finite History 四 profile
  3. G06.3d 旅館、傳送與跨端點服務：TeleportMe 原圖示排版、同端點以外的 Presentation／音效／其他服務；沒有 S_PORTAL 的解鎖（奪命之雷、69 失敗 4-byte 解鎖）、權威座標與 showId
  4. G06 物品接續可到達的拒絕與期限案例、T 鍵入口與 cancel-late、畸形與跨世代 wire 回覆

### T041 | todo | 本機 codex-local | `維護工具/run-local-test-suite.ps1`（1 張）
：擴充 T012 的腳本，加入「技能」與「G06」兩類。技能類跑 T026 狀態表列出的測試，G06 類跑 8 個 scenario；成功門檻和 T012 相同。這是 M2 完成定義要用的腳本。（1 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 測試腳本加入「技能」與「G06」兩類（全部）

### T043 | todo | Cursor＋本機 | `tests/`、fixture（2 張）
：雙真實玩家戰鬥 fixture。NATIVE-EMBLEMS:29–31 記載：正式 JAR `EBF5EE66` 起，startBattle 會強制關閉舊訓練 bot。EBF5EE66 是 E149 的祖先（0.6.59，見報告附錄 D），E149 是否仍是如此，要在本任務第一步用 javap 核對；所以 schema37 的 54 個情境不能標成新版通過。要用兩個真實玩家連線，重新核對攻擊、HP／MP、死亡、結算、重入（NATIVE-EMBLEMS:29–31）。T028、T029、T048 的戰鬥驗證以它為前提。（2 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 先用 javap 核對 E149 的 startBattle 是否關閉訓練 bot；建立雙真實玩家連線 fixture 本身
  2. 用 fixture 重新核對攻擊、HP／MP、死亡、結算、重入

### T044 | todo | Cursor | `src/`：移動正確性與 G04 剩餘項（3 張）
。範圍包括：
  - 已在途移動、靜默拒絕（Cube: Quake 等）、本地碰撞預判；
  - 動態門與動態物件占位、場地邊界；
  - 城門、全部碰撞模式；
  - 移動中斷、其他模式物件、持續輸入中斷；
  - G04.4（in_progress）：加入／離開、死亡復活、換圖及同 ID 替換的同步；
  - G04.7（pending）：其他模式的城門、任務怪物、限時模式物件與進出場。

  出處：NATIVE-ANIMATION:27、NATIVE-MOTION:17、NATIVE-NAVIGATION:15、NATIVE-WORLD-INTEGRATION:16、:40；`docs/G01-G12.md` G04、G04.4、G04.7。另外要實作移動鎖握手：伺服器拒絕移動時送 S_BLINK，客戶端回 C_BLINK，伺服器再送 S_PARALYSE 模式 17（`25 11`）解鎖；重複、未請求或格式錯誤的確認不會解鎖（`movement-lock-0.6.66.md:5`、`:11`）。（3 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 已在途移動、靜默拒絕、本地碰撞預判；C_BLINK 與 S_PARALYSE 模式 17 解鎖握手
  2. 動態門與動態物件占位、場地邊界、城門與全部碰撞模式、移動中斷、其他模式物件、持續輸入中斷
  3. G04.4 加入／離開、死亡復活、換圖、同 ID 替換的同步；G04.7 其他模式的城門、任務怪物、限時模式物件與進出場

### T045 | todo | Cursor | `src/`：物品系統缺口（2 張）
。需要目標或選項的 use_type、type3 charge、194／197 正式流程、C76 銷毀、生命之泉等耗材（NATIVE-ITEM-ACTIONS:9、:39、NATIVE-GROUND-SKILLS:33）。玩家之間的交易不在範圍內（理由見「排除或延後」）。另外包括：擂台 NPC 商店 16 列買價與賣價（`data-sources-0.6.60.json` L402–439）、S_SHOW_MAP 地圖道具 18 筆、Storm_Walk 42501 的 useType 5、三重矢扣箭、12／107 依已裝備武器決定目標、5 個技能（73、100、21、12、107）要送物品 objid、物件 ID 以 unsigned dword 處理（`native-de-inventory-root-current.zh-TW.md:7`、`:13`）。伺服器沒有地面物品與陷阱鏈，那部分標「伺服器待定」。（2 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 需要目標或選項的 use_type、type3 charge、194／197 正式流程、C76 銷毀、生命之泉等耗材、Storm_Walk useType 5、5 個技能送物品 objid、unsigned 物件 ID
  2. 擂台 NPC 商店 16 列、S_SHOW_MAP 18 筆、三重矢扣箭、12／107 依已裝備武器決定目標；地面物品與陷阱標「伺服器待定」

### T046 | todo | Cursor | `src/`：快捷列、技能書與狀態 UI（3 張）
。CLAUDE.md 目標明列「快捷列」。範圍包括技能圖示與快捷列綁定、完整技能書、buff／AC／冷卻／MP 呈現，以及技能中斷（NATIVE-SKILLS:41、NATIVE-SKILL-SELECTION:27、NATIVE-REAGENT-SKILLS:31、NATIVE-ATTACK-SKILLS:3、:32）。另外包括：ICONS1 36 種狀態、ICON_I2H、OwnCharStatus、HPBAR（只給任務／限時怪物，0xFF 隱藏）、S_SPEED 與娃娃緩速的 `writeH` 期限、龍之護鎧圖示 3080（不是 3079）與 `6D h c(181)`、9 個幻術 buff 圖示 lease、1480ms 動畫窗內的第二次施法會被拒絕（快捷列連按）、MP 只在真的改變時才通知。（3 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 快捷列與技能圖示綁定、完整技能書、技能中斷、1480ms 動畫窗內的連按拒絕
  2. buff／AC／冷卻／MP 呈現、ICONS1 36 種狀態、ICON_I2H、OwnCharStatus、龍之護鎧 3080、9 個幻術 lease、S_SPEED 與娃娃緩速的 writeH 期限、MP 只在改變時通知
  3. HPBAR（只給任務／限時怪物，0xFF 隱藏）

### T047 | todo | Cursor | `src/`（2 張）
：影格事件 Effect／Hit 映射到特效（NATIVE-ANIMATION:11、NATIVE-COMBAT:30），以及 gfx1544 action0 在 catalog 缺定義、首幀只畫出 11／12 個物件（NATIVE-WORLD:24、:34）。另外依 G07.2（pending）：全部玩法可見角色、NPC、狀態型物件的原始動作。另外包括：Source169 移除特效、奇古獸 SPR、proc_spr、凍結 palette、立方體 GFX、傷害數字與 miss／critical、動畫先於 HP、光術的封包順序（隱身解除→motion→effect→sound→light→MP）、隱身重施不顯形、變身 7200 秒到期還原、武器族退回 spear24。驗收要看攻擊影格，不能只看封包（0.6.49 的前例：封包通過，但玩家只看到轉向）。（2 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 影格事件 Effect／Hit 映射到特效、Source169 移除特效、奇古獸 SPR、proc_spr、凍結 palette、立方體 GFX、光術封包順序、動畫先於 HP
  2. gfx1544 action0 在 catalog 缺定義、首幀 12 個物件只畫出 11 個（NATIVE-WORLD:24、:34；驗收要 12／12 都畫出，或對缺的那個給出明確的資源診斷）、G07.2 可見角色／NPC／狀態物件的原始動作、傷害數字與 miss／critical、隱身重施不顯形、變身 7200 秒還原、武器族退回 spear24；驗收要看攻擊影格

### T048 | todo | Cursor | `src/`（1 張）
：復活的其他分支，包括 S_RESURRECT 與榮譽模式延遲重生（NATIVE-OUTCOMES:13、NATIVE-CONTINUOUS:22）。（1 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. S_RESURRECT 與榮譽模式延遲重生等其他復活分支（全部）

### T050 | todo | Cursor | `src/`（1 張）
：S_EVENT subtype33 完整性挑戰目前只記錄，沒有送 C_ADDR；要調查並接線，否則不能宣稱長時間戰鬥相容（NATIVE-WORLD:18）。（1 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. subtype33 完整性挑戰與 C_ADDR（全部）

### T054 | todo | Cursor＋本機 | `src/`：房間分支正式整合（G06.1、G06.2）（2 張）
。G06 的 work 是「核實建房、密碼房、設定、踢人、觀戰、隊伍及可達模式的正式契約」。
  - G06.1 目前只到 verified_controller_live_subset：要完成建房、設定、密碼、踢人與房主轉移的正式契約。
  - G06.2 是 candidate_verified_formal_integration_pending：要把模式切換、隊伍、準備、觀戰整合進 main，並納入觀戰單獨主動離場與原 native 初始鏡頭偏移（L1052–L1062）。

  另外依伺服器規則補驗：大廳人數會算入 ArenaBot、測試房不保存勝敗但仍顯示結算、隊伍模式人數規則、全員準備才能開始、拒絕時的中文系統訊息（`room-rules-0.6.61.md:21-25`、`bot-room-count-0.6.64.md:5-12`）；跨來源同頻道建房、預設房重用與改名（`room-directory-0.6.54.json` L21–29）。（2 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. G06.1：建房、設定、密碼、踢人、房主轉移的正式契約；跨來源同頻道建房、預設房重用與改名
  2. G06.2：模式切換、隊伍、準備、觀戰（含單獨主動離場與原 native 初始鏡頭偏移）整合進 main；房間規則（ArenaBot 人數、測試房結算、隊伍人數、全員準備、拒絕訊息）

### T055 | todo | Cursor＋本機 | `src/`：租裝來源整合（G06.3）（2 張）
。
  - G06.3c 是 candidate_defined_return_scope_verified_formal_integration_pending：要完成正常返回、重新入村、服務物品與互動清單生命週期的正式整合。
  - rental-source 模式：C_CA、S_FB 目錄、租裝村與商店交接，以及 map500 其他入口種類（NATIVE-LOBBY:44、NATIVE-WORLD-INTEGRATION:129）。
  - G06 的 acceptance 要求「不得以 Tournament 通過代替租裝或其他模式」。伺服器端 quitFight 的缺口列在 U-M3-1。

  （2 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. G06.3c：正常返回、重新入村、服務物品與互動清單生命週期的正式整合
  2. rental-source：C_CA、S_FB 目錄、租裝村與商店交接，以及 map500 其他入口種類

### T061 | todo | Cursor | `src/`：Ctrl 強制目標與目標選取政策（2 張）
。Rust 客戶端目前沒有 LFC1 擴充（`grep LFC1 src` 為 0 筆）；原本是登入器 DLL 在原生送出點附加。依伺服器文件，範圍包括：
  - 前景視窗、戰鬥狀態、按住 Ctrl 時，在攻擊與技能封包後面附加 `4C 46 43 31 <opcode> 01 67 A5`，並更新長度。無目標、座標型或非物件格式的技能不附加；Pet Race 不套用。出處：`forced-combat-0.6.67.md:5`、`:11-13`，`source-material-control-ingress.zh-TW.md:5`、`:10`。
  - 目標政策：自身與友方的合法輔助不需要 Ctrl；對敵方玩家要有明確意圖；Ctrl 不能繞過死亡、旁觀、距離、LOS、免疫與血盟限制。26／42／68 用自身 object ID 走 7-byte 物件分派。出處：`combat-continuation-current.zh-TW.md:21`、`:52`、`:76-77`，`combat-integration-lifecycle.zh-TW.md:282-302`、`:757-763`，`native-pair-material-progress.zh-TW.md:5-7`。伺服器端記錄「26 不按 Ctrl 無法自施」仍未修（`source-completion-work-current.zh-TW.md:41`）。

  以上路徑都在 lito-launcher-server `lito伺服器/LiTo381-Server/docs/maintenance/`。另外依 #120 補一個目標生命週期案例：技能 68 在扣料或安裝前目標已退役、或延後輸出時 receiver 已換世代，要拒絕或丟棄，而且不能扣到錯的世代的材料（#120 新 N1；`build-native-pair-material.py:35`）。（2 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. LFC1 Ctrl 強制目標擴充：附加條件、長度更新、不附加的情形（無目標、座標型、非物件格式、Pet Race）
  2. 目標選取政策：自身／友方不需 Ctrl、敵方需要明確意圖、不能繞過的限制、26／42／68 自身 object ID 7-byte 分派，以及 #120 新 N1 的技能 68 目標生命週期

### T062 | todo | Cursor | `src/`：選角 8 角色分頁與選服流程（1 張）
。伺服器會送 8 筆角色，並以選定的角色名稱進場；原生角色區只有 4 格，相容層的作法是每頁 4 筆，用 PgUp／PgDn／滾輪翻頁（`buff-schema-and-character-pages-20261018.zh-TW.md:15-25`、`:39`）。Rust 客戶端要先核實是否已支援，缺了就補；血盟名稱上限 16 bytes 加 NUL（同檔 `:35`）；選服流程要能驗收（`combat-closeout-current.zh-TW.md:74`、`release-verification-20261018.zh-TW.md:16`）。（1 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 選角 8 角色分頁、選服流程、血盟名稱 16 bytes（全部）

### T063 | todo | Cursor | `src/`：寵物與召喚物控制介面（1 張）
。依 G05「所有玩家可達操作」：寵物 69 項控制、CHA、容量、項圈（`combat-continuation-current.zh-TW.md:64`），召喚物期限與解放（`:65`）。T026 矩陣判定為「伺服器待定」的控制，只做送包與顯示。（1 張）

**分張**（#132 定義，每張一個 Issue，只做本張範圍）：
  1. 寵物 69 項控制、CHA、容量、項圈、召喚物期限與解放（全部）

## M3 GUI 與交付（2026-11-12 使用者同意開放，#296）— 主線 44 張
驗收：使用者打開會看到什麼（凍結）：
- 在一台乾淨的 Windows x64 主機（沒有 VS 預覽 runtime）上解壓可攜 ZIP，雙擊 ZIP 內的啟動入口（T031 提供可雙擊的 `.cmd` 或 exe，不需要另外安裝 PowerShell 7），連本機自架的 E149 伺服器，走完競技場流程「登入→大廳→進房→戰鬥→退出」。
- `docs/GUI-ACCEPTANCE.md`（M3 新增）列出 T030 清單的每一項，各附截圖或錄影，以及「通過／失敗／待伺服器」。
- `docs/G01-G12-AUDIT.md`（T037 新增）對 `docs/G01-G12.md` 的 46 項逐項列出：本次驗收結論、證據連結、已知限制、重跑指令，以及驗收時的來源版本（客戶端 commit、E149）。
- 執行 `run-local-test-suite.ps1 -Full`，Debug 與 Release 的總表結束碼 0；不能執行的項目逐項寫明排除理由。
任務清單：Backlog「預定 M3」的 44 張已原樣升級（#296，分張見 `docs/plans/issue-296.json`）；各張狀態以 GitHub Issue 為準。其中 30 張本機可做，14 張要執行測試或開遊戲，等 Codex。

## Backlog（不算進度）
本區有兩種項目：一種是 M2／M3 已排定的任務，開放時原樣升級；另一種是排除或延後的工作。2026-11-06 依 #92，原本在 M2／M3 的 T020–T024、T030–T032，以及修補提案 T040，都移到這裡。它們和「打開就看得到」（V1）沒有直接因果關係。

### 預定 M2：已於 2026-11-06 經使用者同意升級為主線（#132），見 `## M2`。

### 預定 M3（44 張；開放時原樣升級，總數不變）
- **T040 | backlog(M3) | Claude | lito-launcher-server Issue**：把修補提案四項（租裝返回、同端點傳送、房間最後離開、商人屬性來源）以 Issue 加證據路徑的形式交給伺服器 repo，不搬 ASM 工具。伺服器接收後，刪除本 repo 的 `修補提案/`。（1 張）
- **T033 | done | 本機 codex-local | `docs/G01-G12.md`**：已由 #106 提前完成（`工作佇列.json` SHA256 `0EC8ACBF…830AC`，共 46 項：12 個主項、34 個子項），不計入 M3 張數。map563 與 ARGO 的範圍判定改由 T037 做（U-M3-6、U-M3-8）。
- **T034 | backlog(M3) | Claude | `docs/GUI-ACCEPTANCE.md`（新增）**：T030 的檢查清單。
  - 流程：普通房、租裝入口與商店、旅館返回（3600／7200 秒自然租期）、商人與市民、Enter 負向、EOF／取消、新技能、owner 退役與傳送恢復、觀戰、雙玩家戰鬥退出、小地圖、NPC caption。
  - 核對面向：實體輸入、像素、IME、多 DPI、聲音（實體裝置、無裝置／拔插）。
  - 其他清單項：建房、中文密碼往返（NATIVE-ROOM:46）、TeleportMe 原圖示排版（NATIVE-TELEPORT:22）、屋頂進出（NATIVE-WORLD-ROOFS:35）、開門 O 鍵、觀戰切換與鏡頭、G09 音效事件、聊天（T060）。
  - 依 G08 子項逐條列入：
    - G08.1 房間建立／修改、踢人、觀戰、模式控件；
    - G08.2a–e 徽章與市民資格頁、瑪雅／菲佛／克里商人與容量／重量／到期、旅館租房／鑰匙／退房、四位傳送 NPC、鍵盤走訪；
    - G08.3 屬性、狀態、系統訊息與小地圖；
    - G08.4 從唯一入口走完整流程，含 IME、DPI、焦點、取消；
    - G07.5 小地圖的觀戰 GUI。
  - 數值 schema 寫明登入器的啟動方式。

  （1 張）
- **T035 | backlog(M3) | Cursor | `src/`、`維護工具/`：已知 GUI 與輸入缺陷**，分 2 張：第 1 張是截圖與輸入（第 2 項）；第 2 張是版面與文字（第 1、3 項）。
  - Release 長入口提示換行被下方操作列裁切（NATIVE-WORLD-INTEGRATION:156）。
  - FrameArrived 逾時、geometry unavailable 的恢復路徑，以及 Release 的 OS SendInput 輸入探針失敗：擷取逾時、click 缺 geometry、沒有 result.json（HISTORY L1411–L1417、NATIVE-LOGIN:14）。
  - 伺服器名稱含 `(數量)` 時，數量會重複顯示（NATIVE-INVENTORY:44）。
  - G08 的 acceptance：失焦後不能持續發招；無法操作或布局遮擋都要修好。

  （2 張）
- **T030 | backlog(M3) | 使用者 | 依 T034 清單做原生 GUI 實機驗收**：真機驗收，`needs:user`。（1 張）
- **T031 | backlog(M3) | Cursor＋本機 | `packaging/**`：clean host 可攜交付**。依 G10：可攜包要附來源、EXE 與資源的版本清單，不依賴舊 BIN。提供可雙擊的啟動入口（`.cmd` 或直接啟動 exe，不依賴 PowerShell 7）；改用正式版 VC runtime，不依賴 VS 預覽 runtime；移除 5 處開發診斷 hardcode（L1569）；寫明最低 Windows 版本；在乾淨主機驗證 LoadLibrary 動態載入閉包、首次設定與中文路徑。（2 張：打包 1、乾淨主機驗證 1）
- **T036 | backlog(M3) | 本機 dot／codex-local＋Claude | 廣域回歸：白名單以外的 1055 個待驗測試**（574＋481，分類見 #54）。依 #92 盤點分 15 張：
  - 檔案寫入所有權審查（83 項）1 張；
  - 跨函式 helper 閉合 4 張（技能／戰鬥 2、世界／渲染 1、背包／登入／傳送 1），審完的新放行項目由 1 張執行；
  - 共享資源根 159 項 2 張；
  - Java peer／TLS 盤點 1 張、執行 2 張；
  - 原機 snapshot 99 項 1 張；
  - 雜項環境變數 40 項 1 張；
  - 自有隔離 DB 盤點 1 張、執行 1 張。

  GUI 6 項與 REAL_DB 1 項直接排除，移入 T030 清單。整庫 lib 編譯記憶體失敗是整庫回歸的前置阻擋，要先處理（NATIVE-WORLD-INTEGRATION:5、:9、:177）。歷史上未關閉的失敗要在對應那一張重驗：
  - C72D 499 過 1 敗（L71、L1586）、66470 紅測試（L83）；
  - 房間切換 90 秒逾時（NATIVE-SCENE-LOADING:9）、schema34 首次失敗（NATIVE-RESOURCE-LATENCY:17）、C3 材料技能首輪失敗（NATIVE-LABEL-UPDATES:50）、Release573 嚴格地形（NATIVE-WORLD-INTEGRATION:150）；
  - runner 漏 pin native_room.rs（NATIVE-WORLD-INTEGRATION:13）。（15 張）
- **T038 | backlog(M3) | 本機 dot＋Claude＋Cursor | G11 效能、長時間與故障恢復**，依 G11 的 work 分 3 張：
  - 先保存雙玩家 Release 的載入、幀時間、記憶體基準與驗收預算（`acceptance-budget-v1.json`），避免事後改標準。
  - 至少 60 分鐘、20 次完整場次循環，並在登入、房間、戰鬥、返回四個階段注入斷線（HISTORY L159、L171、L1204）。
  - 量測快取、worker、handle、音效、取消與退出；修復超限或持續增長的項目。

  斷線延遲結算記一敗，重複回呼要冪等（`dragon-skills-0.6.46.json` L45）。（3 張）
- **T037 | backlog(M3) | Claude＋本機 | `docs/G01-G12-AUDIT.md`（新增）**：G12 稽核，依 G12 的 work 分 2 張：
  - 第 1 張：保存唯一原始碼、逆向與回復證據，再整理已核實的冗餘產物。只整理專案內、本專案自己產生的產物；不動封存、硬連結、正式資源；刪除前列清單。
  - 第 2 張：對 46 項逐項核對驗收、已知限制、重跑指令與現行來源漂移，寫進 `docs/G01-G12-AUDIT.md`；同時判定 map563、ARGO 是否在範圍內（U-M3-6、U-M3-8）。

  （2 張）
- **T032 | backlog(M3) | Claude | `docs/ARCHITECTURE.zh-TW.md`**：重寫為現況版（現在是流水帳）。（1 張）
- **T042 | backlog(M3) | 本機 codex-local | `維護工具/run-local-test-suite.ps1`**：加入 `-Full` 模式，涵蓋 Debug 與 Release，以及 T036 放行的全部項目；無法執行的項目逐項輸出排除理由。這是 M3 完成定義要用的腳本。（1 張）
- **T049 | backlog(M3) | Cursor | `src/`：背包與介面文字**。範圍包括：
  - 祝福光暈、詛咒色盤（NATIVE-INVENTORY-ICONS:41）；
  - 物品名稱 runtime 模板、格式顏色、遞迴引用（NATIVE-INVENTORY-NAMES:39）；
  - 數字通知的 Text 資源映射（NATIVE-GAME:21）、LTstring 原版 UI（NATIVE-LOBBY:46）；
  - S0F／S10 系統訊息 GUI（NATIVE-WORLD-INTEGRATION:84）；
  - 地面與世界物品名稱。

  （2 張）
- **T052 | backlog(M3) | Cursor | `src/`**：裝備與彈藥 UI（NATIVE-RANGED:13、:37）；地面物品直接滑鼠拾取與排序；特殊物品選項（NATIVE-REAGENT-SKILLS:31、:33）。（2 張）
- **T053 | backlog(M3) | 本機＋Claude | G03 舊 54 情境盤點與重驗**。依 G03 的 work：
  - 逐一把舊 54 情境列為保留、改寫或失效，並說明原因；
  - 補拒絕、延遲或漏回覆、過期世代、斷線與重新登入；
  - 做 S_CE／S_D4／S_CD 順序重播，驗證初始化通知的人數、期限與換房隔離。

  每個現行必備情境都要有當前 JAR／EXE 的結果，不得沿用舊版計數。（2 張）
- **T056 | backlog(M3) | Cursor | `src/`：名稱、徽章與 FA／S69 狀態呈現（G07）**。G07 的 work 明列「全體／地面名稱、徽章位置、FA／S69 状態」。範圍包括：
  - emblem 請求、快取與繪製（NATIVE-OPTIMIZED-PROFILE:18）；
  - S2F 稱號、全體名稱權限、特殊 NPC 徽記（NATIVE-WORLD-LABELS:13、:25）；
  - FA 限制與 S69 戰爭狀態的陣營顏色分支（NATIVE-LABEL-UPDATES:19、NATIVE-SCENE-LOADING:13）。

  盟戰本身的玩法不在範圍內，這裡只做狀態呈現。（2 張）
- **T057 | backlog(M3) | Cursor＋本機 | 資源支援矩陣與失敗診斷（G07.1、G07.4）**。
  - G07.1：完成可重跑、綁資源 SHA 的資源命名空間與格式矩陣，做到逐成員覆蓋（目前只到 exhaustive_member_coverage_pending）。
  - G07.4：缺件與未知格式要「明確失敗並提供來源診斷」，不可靜默缺圖。例如 727 缺 383 附件、262 陰影 bank（NATIVE-ACTOR-LAYERS:11），以及 T011-F 的特效資源資格不符，都要有玩家看得到或報告讀得到的診斷。

  另外包括：11 張經典地圖的資料差異與地圖 504 的雙域物理（`native-de-terrain-current.zh-TW.md:11`）、LTzone 區域名稱與色彩（不能由顏色推安全區）、map_page 只有頁 1、2。（2 張）
- **T058 | backlog(M3) | Cursor | `src/`：遮擋、屋頂、附件、透明與特效排序（G07.3）**。G07.3 目前是 partially_verified，要對照當前正式核心核實地形、跨區塊、遮擋、屋頂、附件、名稱與特效排序，包括屋頂進出（NATIVE-WORLD-ROOFS:35）。照明、水下、map303 背景、RGB565 等價沒有列在 G07，仍歸「畫面等價研究」。（2 張）
- **T059 | backlog(M3) | Cursor＋本機 | 音效裝置生命週期（G09）**。依 G09 的 work：
  - 原始 WAV 與影格事件；
  - 無裝置、裝置變更、同時多聲、切換場景與退出回收；
  - 不能有重複事件、無限聲音，或因裝置失敗而崩潰。

  實際聽覺驗收由 T030 做。（2 張）
- **T060 | backlog(M3) | Cursor＋本機 | 聊天**。CLAUDE.md 目標明列「聊天」。要驗證聊天視窗收發、過濾視窗、私訊的當版行為；目前 `chat_filtered_window_tests.rs:22`、`direct_chat_window_tests.rs:438` 是 GUI ignored 測試（#54），GUI 部分併入 T034 清單。另外要解析 NPC 對話 opcode 45（sayCode 加座標，範圍 8 格、大喊 50 格、全服；`native-de-chat-current.zh-TW.md:9-11`）。（1 張）

### 排除或延後（不在 M0–M3 完成定義內）
- **WORLD 地圖專屬項目**：伺服器端已確認「LiTo 只有競技場，沒有 WORLD 地圖」（lito-launcher-server `docs/maintenance/combat-continuation-current.zh-TW.md:14`）。因此排除：map303 floor gate、水下 shader／phase／scroll、攻城 P 鍵、經典 C_ENTER_WORLD 分支等 WORLD 地圖才有的項目。競技場與租裝城鎮仍在範圍內；屋頂屬於 G07.3，已歸 T058，不在排除之列。
- **凍結候選資格檢查**：12 gate 案（候選版）、四元件 inventory、17 原控制＋6 反例、舊 G06 16 案 640 次、候選 overlay QA 475 案（#58、#61 以 not planned 關閉，#33 裁定凍結）。
- **G06 正向前提**：非預設物品、source／account、timer。E149 全 Code 掃描找不到建立或排程的 caller，等伺服器端提供生產者再議。
- **伺服器缺陷**：商人屬性來源（例如 29047 戒指火抗 0，應為 10）。客戶端只顯示權威封包，GUI 驗收寫「與伺服器送出的一致」。
- **效能最佳化**：超出 G11 預算之外的優化，例如首次 Debug 載入 28 秒、封存 hash 23.65 秒。
- **TW 資源** `TW13081901.pak` 缺檔：5 個 TW 項目不能替代。
- **畫面等價研究**（HISTORY L163–L305、L1298–L1578）：照明／光罩、水下時鐘與後處理、map303 背景／viewport、背景 64ms scheduler、相機非初始更新、Bank viewport、RGB565 framebuffer 等價。這些項目沒有列在 G01–G12 的 work 中；G07 明列的遮擋、屋頂、附件、透明、特效排序不在這裡，已歸 T058。這些項目不在 M0–M3 的完成定義內；如果 T037 稽核判定 G01–G12 需要，依 U-M3-5 兌現。
- **WORLD 專屬（依 HISTORY）**：G06.4 普通世界 C05 入口與後續場景交接（`docs/G01-G12.md` G06.4；伺服器端已確認沒有 WORLD 地圖，見報告 E6）、技能 131 進 map4、WORLD 主視窗、869／$748 NPC action3、毒咒與寒冷戰慄的 WORLD 分支、攻城 P 鍵（L555、L1078、L1086、L1175–L1199）。
- **建置雜項**：Debug 281／Release 282 個編譯警告（L711 等）、外部 LiTo.bin／Server_Game.jar（L1014；本 repo 不包裝 LiTo.bin）、before-rebuild zip 與 NUL 診斷映像（L919、L93）。
- **候選 overlay 測試（原稱 475 案）**：#106 已逐檔、逐函式列出 44 個候選檔共 554 個測試宣告（545 個直接 `#[test]`＋9 個條件 test），其中 61 個和 main 同名、493 個是候選新增。主控裁定：以 #106 的具名清單（493）取代歷史的 metadata 計數 475；差額 18 沒有歷史具名名冊可以核對，記為「差額未解」。這些測試隨候選一起凍結，不另外執行；各檔對應的重寫任務見報告附錄 E，重寫時只當設計參考。
- **研究紀錄中確認排除的項目**（NATIVE-*、`維護工具/*.md`）：
  - 盟戰玩法：盟戰結束清 1F、S36 公告與音樂（NATIVE-ACTOR-RELATIONS:11、:20）。FA／S69 的狀態呈現在 G07 範圍內，已歸 T056。
  - see-invisible／GM 視野（NATIVE-ACTOR-STATES:17）。
  - C0B 末 byte 的 remembered-ID 與 986ms 節流（NATIVE-GROUND-PICKUP:15）。
  - 原版等價研究：動作中斷順序、延遲補償、效果細分座標（NATIVE-COMBAT:26、NATIVE-ANIMATION:17、NATIVE-SKILLS:19）。
  - 玩家之間的交易：G01–G12 沒有這一項。G08 的「交易控件」指 G08.2 的 NPC／租裝商店交易數量與權威貨幣提示，已歸 T055、T034。
  - VS-server directory（NATIVE-LOGIN:44）。
  - 多客戶端即時競態與自然搶拾（NATIVE-REAL-PEERS:43、:45、NATIVE-ROOM:46）。
  - connect 階段 Drop 等 timeout（NATIVE-PROTOCOL:36）、舊版名單查詢分支（NATIVE-WORLD-LABELS:15）。
  - 候選資格的「四元件／原版 6 反例／候選 14 案／元件矩陣 28 案」（`維護工具/native-E149-component-*.md`、`native-E149-owned-mysql-README.md:19`），以及 action0 cadence 之外的全域 scheduler 交錯、clock import。
- **GM 工具**：`.loc`、`.move`、`.item`、`.spawn`、`.de`、`.polyme` 與權限 200（`gm-travel-0.6.68.md`、`gm-polyme-0.6.71.md`、`source-administration-validation.json`）。G01–G12 沒有 GM 工具這一項，所以排除。GM 指令經聊天送出時，只要求 T060 能正常收發並顯示回覆；變身外型的顯示由 T047 的通用路徑涵蓋。
- **伺服器 maintenance 中純伺服器內部的項目**：鎖、FIFO、completion 佇列、生命週期、外控設定鍵（`installed_consumer=false` 的 141 列）、目錄整理紀錄。這些無客戶端影響，見報告附錄 F。
