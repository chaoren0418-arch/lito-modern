# Issue #332 — T053（M3 1／2）：G03 舊 54 情境盤點與補充情境定義

## 結論

舊 54 情境逐案裁定完成：**保留 42、改寫 11、失效 1**。另外定義了拒絕、延遲或漏回覆、過期世代、斷線、重新登入五類共 35 個補充情境。全部寫在 `docs/G03-SCENARIOS.md`。

本張只做盤點與定義，**沒有執行任何測試，也沒有改任何程式碼**。54 案的 probe 在 E149 一次都沒跑過；只有 5 案的改寫版在 #174／#175 的雙玩家測試裡有 E149 結果。

## 做了什麼

1. 找出「舊 54 情境」的逐案清單。repo 內的文件只有總數，清單在本機未追蹤的 `驗證紀錄/current/native-real-peers/regression-matrix.json`（2026-10-13，JAR BBC59308）。已把 54 列抄進 `docs/G03-SCENARIOS.md` §3，之後不必再靠未追蹤的檔案。
2. 逐案對照現行 main（`91e916c`）：probe mode 是否還在、Java harness 是否還在、有哪些 Rust 測試對應、有沒有 E149 的結果。
3. 依 E149 的現況重新裁定（不沿用 2026-10-13 的處置），每案寫原因。
4. 補五類情境定義（R1–R8、D1–D8、S1–S6、X1–X7、L1–L6），每案有前置與步驟、預期、現成入口、層級、E149 結果。
5. 把現行必備情境分成三組，列出第 2 張各要做什麼。

`docs/REUSE-MAP.md` 已查：T053 沒有專屬列。用到的是 T043 列（雙玩家 fixture、E149 的 bot 政策要 javap 核對）與 T054 列（房間分支）的結論；本張沒有直接讀 tw-bin-engine 或 lito-launcher-server 的檔案，封包定義都取自本 repo 的 `docs/PROTOCOL-CONTRACT.md`。本張沒有宣稱缺資源，所以沒有做 pak 查詢。

## 已驗證

以下都是讀檔核對，不是測試結果。

- 舊矩陣 `legacy_count=54`，逐列的群組與 mode 與 §3 的 54 列一致。舊處置的分布是 31 retain、5 由雙玩家雙場取代、7 改用真實對手、8 移除 bot 前置、1 改打 NPC、2 分離重驗，合計 54。
- 54 案用到的 probe mode 都還在 `src/modern_lito/native_login.rs:3061–3115`；`src/main.rs` 仍有四個 `--probe-native-*` 入口；`tests/oracle/` 仍有 `NativeHandshakeHarness`、`NativeLoginHarness`、`NativeGameHarness`、`NativePeerHarness`、`NativeRecoveryHarness`。
- 舊 runner（`維護工具/verify-native-*.ps1`）讀 `驗證紀錄/current/reference-snapshot.json`；該檔目前釘的 SHA 是 `C02AAAE8…`，不是 E149。所以舊 runner 不改就不能產生 E149 結果。
- E149 的訓練 bot 是條件式（#174 的 javap 結論），不是舊文件寫的「強制關閉」。
- 現有的 E149 動態結果只有三批：#174／#175 雙玩家（基底 `318b3ab`）、#118 G06 五案、#56 白名單 76 項（沒有啟動 JVM）。G06 後補三案沒有實跑（#170、#173）。
- §4 引用的 Rust 測試名稱都以現行 `tests/*.rs` 的函式名核對過。

## 推測

- 42 個保留案在 E149 應該大多能照原定義通過，理由是 `PROTOCOL-CONTRACT.md` 對 E149 核對過的登入、頻道、房間格式與這些案的斷言沒有衝突。這只是預期，沒有實跑。
- `NativeGameHarness` 用反射改寫伺服器內部的 record（例如把 `maxMp` 設成 2）、直接移除伺服器記憶體裡的地面物件。E149 的內部類別如果和 BBC59308 不同，這些 harness 可能要先修才能編譯或執行。
- #19（lobby／reselect）與 #15（inventory-reselect）可能受 T062（#194）選角分頁改動影響。
- #22（room-ready）等單人進場的案，在 E149 可能因為條件式 bot 而出現舊版沒有的 actor 2，看 harness 的 `testBotEnabled`／`testRoom` 而定。

## 待驗證（未驗證什麼）

- **54 案的當前 JAR／EXE 結果全部待驗證**，包含 E1 已有結果的 5 案（那是 2026-11-06 的 EXE，不是當前 main）。
- §4 的 35 個補充情境裡，只有 D8、X7、S5 的兩案、L4 的五案、L6 有 E149 動態結果；其餘待驗證。
- 沒有入口、要新寫的：#47 的第二場結算、R5 多種拒絕碼的動態案、D4 與 D5 的動態案、X4（對手戰鬥中斷線）、X5（房主斷線）、L3（伺服器未清完就重登）。
- L3 的預期行為還沒定：E149 對「舊 session 未清完時重登」是拒絕、踢舊還是排隊，本張沒有查到資料。
- D4 的「期限」：G03 要求驗初始化通知的期限，現有 `native_early_roster` 三個測試沒有以時間為條件的案例；暫存有沒有期限還沒核對。
- S_CE／S_D4／S_CD 順序重播屬於第 2 張，本張只在 S3 寫了它要驗什麼。

## 產出路徑

- `docs/G03-SCENARIOS.md`（新增）：54 案裁定表、補充情境定義、第 2 張的分組。
- `docs/reports/issue-332.md`（本檔）。
- 分支 `task/332`。沒有改 `src/`、`tests/`、`維護工具/`、`TASKS.md`、`PLAN.md`、`docs/REPORT.md`。

## 給下一棒的提示

第 2 張（重驗＋S_CE／S_D4／S_CD 重播）建議的順序：

1. **先重跑已經有 owned 入口的**（§5 的 A 組）：`native_two_player_battle` 兩個 ignored 測試、G06 八案（`run-teleport-failure.ps1`）。這些不必改 runner，可以最快拿到當前 EXE 的結果。這要執行編譯出的程式，Claude 本機不能跑，要交給 Codex 本機。
2. **再處理舊 runner**（B 組）。不要直接跑 `verify-native-*.ps1`：它們綁的是 `C02AAAE8` 的 snapshot，輸出寫到原始專案目錄的 `驗證紀錄/`。要先改成 JAR 由參數傳入並核對 E149 SHA、輸出寫到 worktree 的 `LiTo381-Modern/target/current/`、程序依 PID 清理，照 `run-teleport-failure.ps1` 的守門做。改 runner 屬於程式變更，Claude 本機可以寫，實跑交給 Codex。
3. 改 runner 前先用 `javac` 對 E149 編譯 `tests/oracle/NativeGameHarness.java`、`NativeRecoveryHarness.java`、`NativePeerHarness.java`（只編譯、不執行），確認反射用到的類別與欄位還在。
4. S3 的重播：真實封包可以從 `tests/fixtures/native-room-current.json`、`native-early-roster.json`、`native-force-current.json` 取；`native_room.rs:162` 有 S_D4 插在 S_CE 與 S_CD 之間的註解，從那裡讀起。
5. L3 先做 javap（`ClientSession` 的登入與斷線清理路徑），預期定了再寫測試。
6. 記錄結果時照 `docs/G03-SCENARIOS.md` §5 的計數規則：不同 EXE 或 JAR 的結果不能相加，#46 不單獨計數，略過與 filtered out 不算通過。
7. 重驗完成後，把每案的結果直接更新在 `docs/G03-SCENARIOS.md` 的「E149 結果」欄，不要另開一份清單。
