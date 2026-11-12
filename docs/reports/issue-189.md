# Issue #189 — T054-2 房間模式、隊伍、準備與觀戰整合

## 結論

- **原 native 初始鏡頭偏移：本輪已逆向並實作**（見下方「第 2 次嘗試」）。
- **觀戰單獨主動離場：客戶端做不到，伺服器要先改**。現有伺服器把觀戰者的主動退出當成整場結算，伺服器 repo 在本專案是唯讀。本輪已開伺服器 Issue chaoren0418-arch/lito-launcher-server#366（`stage:code`、`needs:claude`），寫明要改的地方、回覆契約與驗收方式。客戶端繼續拒絕觀戰中的主動退出，也不送出封包，避免觀戰者讓其他參戰者被結算。
- T054-2 還有這一項依賴沒解決，**整張還不能算完成**。

基底：第 1 次為 `e83403140e814e45fc7c53ed0cb8929424f5a079`；第 2 次已合併 `origin/main`，衝突只在 `native_login.rs` 的模組宣告，雙方的模組都保留。

## 第 2 次嘗試（2026-11-12）

### 做了什麼

1. **靜態逆向原 native 觀戰鏡頭**。來源是唯讀的 `lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm`（SHA256 `2A3A6E10…7DEA9A`，同第 1 次）。
   - 相機錨點是全域 `dword_75AA50/54`。整個檔案只有 `sub_485580`（L283598–L283613）會寫它。
   - `sub_485580` 的呼叫者只有兩個：`sub_4855A0`（L283622）與自己角色的逐步移動 `sub_4E2150+D73`（L475295–L475307，依 heading 和自己角色同步移動錨點）。
   - `sub_4855A0` 的呼叫者：
     - `sub_485800`。它的呼叫者是 `sub_4C51B0`／`sub_4C5680`（依時間觸發的場景移動，傳 `x+0x8006`、`y+0x7FEE`）與 `sub_4E7A40`。
     - `sub_48BC80`。只有傳送封包 `sub_4AF4B0`（dispatcher case 90）會呼叫，而且自己角色為空時直接返回（L365825–L365829）。
     - `sub_4E7A40`。這是 S_PUT_OBJECT 處理，只有在 `id == dword_75AA48`（自己）時才設錨點。錨點設為**自己座標加 (+6, −18)**（L484790–L484809：`lea edx,[ecx+6]`、`add eax,0FFFFFFEEh`）。
   - **S_WORLD（`sub_4A7130`）、S_E2（`sub_4AC5E0`，只把矩形加進 `byte_75A8B0` 向量）、S_E5（`sub_4AAAB0`，只設 `byte_64AFA1` 並切換 UI 視窗）都不寫錨點。**
   - 移動檢查 `sub_48B640`／`sub_48B7B0` 都會讀自己角色指標 `dword_834EE4`。沒有自己角色時，native 沒有可以只移動鏡頭的路徑。
   - 結論：伺服器不會送觀戰者自己的物件（`ClientSession.startBattle` L12067–L12098 只送參戰者），所以 native 觀戰者的初始鏡頭就是**前一個場景留下的錨點**。native 不會依觀戰範圍置中，也沒有觀戰專用的偏移。偏移 (+6, −18) 只存在於「自己座標 → 錨點」這一步。
2. **客戶端實作**（`LiTo381-Modern/src/modern_lito/native_world.rs`）
   - 新增 `NATIVE_CAMERA_ANCHOR_OFFSET = [6, -18]`。本地鏡頭存的是「等同自己角色的格子」，所以 (+6, −18) 由渲染端原本的投影吸收；這個常數只用來記錄逆向依據。
   - 新增 `World::native_view_anchor`。換地圖（S_WORLD 0x28）與 `clear_preserving_generation` 時，從 `native_anchor_cell()`（有自己角色就用它的座標，否則沿用上一個錨點）帶過去。這對應 native 全域錨點跨場景保留的行為。
   - `refresh_observer_camera`：觀戰鏡頭不存在，或已不在半開範圍內時，**優先沿用保留的錨點**（和 native 相同）。只有沒有錨點，或錨點落在範圍外時（native 這時看不到競技場），才退回範圍中心。這個退回點是刻意保留的差異，寫在程式註解裡。
   - 觀戰平移和 native 一樣不寫錨點。觀戰期間沒有自己角色，錨點維持不變。
3. **測試**（`LiTo381-Modern/tests/native_observer_camera.rs`，新增 2 個案例）
   - 先打一場、有自己角色 (32720, 32780)，然後換地圖觀戰：範圍內沿用錨點；S_E2 與 S_E5 兩種先後順序都測；平移不改錨點；`clear_preserving_generation` 保留錨點並清掉觀戰狀態。
   - 錨點在範圍外時退回中心 (33056, 33056)；新連線沒有錨點時維持原本的中心 (32736, 32800)。原有 3 個案例不受影響。
4. **觀戰單獨離場**：唯讀核對伺服器，`ClientSession.exitRoomBeforePolyMe` L14576 與 `quitFight` L14882 在 BATTLE 階段都會進 `settleRequestedBattleExit` L14850–L14879，對全房間 `serverEndBattle` 並 `hub.endBattle`。斷線路徑 `GameHub.registerBattleDisconnect` L12906–L12910 明確排除觀戰者。客戶端沒有其他封包可以只讓觀戰者離開，所以開了伺服器 Issue #366。

### 怎麼驗證（第 2 次）

依 2026-11-09 本機規定，本輪**沒有在本機執行任何測試執行檔**，只跑編譯檢查；測試交給 PR 的 GitHub Actions CI。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
exit=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 44s
```

（合併 main 之後重跑。編譯檢查不會輸出 `test result:`；專案原有的 unused 警告沒變。）

### 第 2 次的已驗證／推測／待驗證

- **已驗證**：上面列的 asm 行號與呼叫關係（靜態讀檔）；伺服器來源的退出路徑（靜態讀檔）；`cargo check` 結束碼 0。
- **推測**：渲染端把「自己角色格子」投影成畫面時，已經包含 native 的 (+6, −18) 錨點關係。這是依現有 Rust 非觀戰畫面和 native 對齊的既有驗收推得，本輪沒有重新比對。
- **待驗證**：
  - `native_observer_camera` 新舊 5 個案例與 `room_flow_tests` 要等 CI 的 `test result:`。
  - 觀戰初始畫面要和 native 實機截圖比對，交給 Codex／使用者，要用「先打一場再觀戰」與「新連線直接觀戰」兩種情境。
  - 觀戰單獨離場要等伺服器 #366。

## 第 1 次嘗試的內容（2026-11-06，保留）

### 做了什麼

- 模式切換等待同房間 `S_D5/2` 模式與後續 `S_D5/3` 地圖。只有地圖更新時繼續等待。拒絕回覆的 `S_D5/0` 同時包含模式與地圖，可完成模式或地圖請求。錯房封包在寫入等待旗標前拒絕。
- 房主模式／地圖操作交由伺服器裁定準備重設，排除客戶端「所有名冊列都必須未準備」造成的阻擋。ArenaBot 的準備狀態也可能保留在名冊中。隊伍操作可在準備中送出，接收伺服器實際隊伍與取消倒數通知。送出請求後仍等待權威回覆才更新名冊、模式及地圖。
- `S_MESSAGE 0x0F` 可在房間尚未收到 `S_WORLD` 時解析。保存目前房間的 type 9 中文系統訊息，接入房間狀態列。開始拒絕、人數／紅藍隊規則及測試房通知均可顯示。新操作送出成功、重新入房、離房、取消與失敗退役會清理相關訊息。
- `MapPending` 接入房間頁面判定。倒數零值顯示為取消後的房間狀態。觀戰提示限定在觀戰進行期間，結算階段可顯示勝方及結算人數。
- 觀戰主動退出入口回傳明確的服務端依賴訊息，維持對其他參戰者的隔離。觀戰結算後沿用 `C_D9 → S_F0 → C_D0 → S_D0` 返回路徑，退役世界、鏡頭與房間訊息。
- 新增 `native_room_flow_tests.rs`，共用 #188 的離線封包 helpers。加密 loopback 檢查正式控制器入口發出的 literal request，使用動態 `127.0.0.1:0`、2-byte read chunk 與 3-byte write chunk。

## 已驗證

### 本機靜態契約

來源根目錄為唯讀 `D:\lineage開發\3-lito開發\lito伺服器\LiTo381-Server`。

| 項目 | 核對來源與客戶端處理 |
|---|---|
| 模式／地圖變更 | `ClientSession.java:16470–16559`，成功模式依序發 `D5/2`、`D5/3`，拒絕發權威 snapshot。`GameHub.java:10026` 允許房主在準備期間修改，接受後由伺服器清除準備。 |
| 隊伍與準備 | `ClientSession.java:11626–11692`、`:16562–16598`，回傳自己的實際 state／force，拒絕可回相同值。`GameHub.java:9622–9724` 以參戰者、全員準備及人數規則控制倒數，隊伍異動會重驗。 |
| ArenaBot 人數 | `docs/maintenance/bot-room-count-0.6.64.md:5–12`。大廳照收 `S_CC` 人數，空測試房可為 1，單人加假人為 2，雙真人為 2，停用假人空房為 0。客戶端以封包值呈現，名冊另由 `S_CD`／`S_D0` 更新。 |
| 測試房結算 | `docs/maintenance/room-rules-0.6.61.md:21–25`、`DurableBattleSettlement.java:9,45,69`。伺服器依 defaultRoomId 固定本局測試房身分及保存策略。客戶端接收 `S_D3`／`S_DB` 顯示結算，房名不參與戰績保存判定。 |
| 中文拒絕訊息 | `NativeRoomRules.java:70–115`。測試涵蓋紅隊單人測試、真人不足、紅藍隊缺人、人數一致、攻城每隊 15～50 人及測試房通知的原文。 |
| 觀戰角色與返回 | `S_E5` 決定角色旗標，世界可在沒有自己 actor 時使用獨立鏡頭。正常結算返回收到自己的 `S_D0` 後才退役，鏡頭平移的發送封包數為零。 |

| 唯讀來源 | SHA256 |
|---|---|
| `ClientSession.java` | `6CE00B818E73BF509152A460DAB4831FD5EEFC29E6344E7081C4948D40046F8B` |
| `GameHub.java` | `6A123687BB65FDCE4A6B7F8FEC8317DA0B3E949C48758F43766FAC2F9FE4886D` |
| `NativeRoomRules.java` | `7E4091F4F586BD8192607DD61FF1DA5B7AB4C26C0304D8035C6EBB0C9C1BF7B5` |
| `DurableBattleSettlement.java` | `D7B01232AAF13541E5E99FCF3AB689981A7240CAD63A5A0023BE60DC4BD4964E` |

### 怎麼驗證

測試在本 worktree 執行。子程序清除 `LITO_*`，`TEMP`／`TMP` 指向工作樹 `LiTo381-Modern/target/current/scratch`。使用 `--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`。

一次性 `scratch/verify-189.ps1` 從 `維護工具/run-local-test-suite.ps1` 的 PowerShell AST 取得原始 `Measure-TestResult` 與 `Invoke-SuiteStep`，直接使用相同門檻。每次測試必須結束碼 0、只有一組可解析結果、running 數吻合且大於零、failed 為零。略過項目另列。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
exit=0; GATE=PASS

```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib room_flow_tests
exit=0; GATE=PASS
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 2527 filtered out; finished in 1.00s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib room_admin_tests
exit=0; GATE=PASS
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 2528 filtered out; finished in 0.54s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_lobby
exit=0; GATE=PASS
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room
exit=0; GATE=PASS
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room_admin
exit=0; GATE=PASS
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room_departures
exit=0; GATE=PASS
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room_force
exit=0; GATE=PASS
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room_mode
exit=0; GATE=PASS
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room_ui
exit=0; GATE=PASS
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_observer_camera
exit=0; GATE=PASS
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_world
exit=0; GATE=PASS
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib battle_withdrawal_tests
exit=0; GATE=PASS
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2537 filtered out; finished in 0.00s
```

最終 12 組測試合計 **59 passed、0 failed、0 ignored**，每組均通過原 runner 門檻。編譯檢查結束碼為 0，編譯步驟沒有 `test result:`。專案仍有既有編譯警告。

初輪新增測試結束碼 101，結果為 8 passed／3 failed。第二輪為 11 passed／2 failed，結束碼 101。原因是離線 fixture 的 WORLD／S_0C 初始化缺少 GAME 環境世代。已將純單元的場景資料交由 World parser 建立，並將加密 loopback 的環境綁定至自己的 transport epoch。最終重跑為 13 passed／0 failed。兩輪失敗保留於 `Flow/initial.log`、`Flow/second.log`，不計入通過總數。

`git -c core.whitespace=cr-at-eol diff --check` 結束碼 0。

新增測試涵蓋模式封包順序與錯房拒絕、六種戰鬥模式的實際送收、模式／地圖拒絕、隊伍與準備的精確成員名稱、回覆同值、準備中換隊取消倒數、全員準備也不在本地合成開始、ArenaBot 名冊與清單人數、中文訊息與畸形封包、一般房／測試房結算接收、觀戰輸入隔離與完整結算返回、失敗送出及逾時清理。

## 推測

- 唯讀 Java 來源及維護文件支持以上伺服器契約。本輪固定回覆的加密 loopback 驗證客戶端實際送收與狀態轉移，服務端規則的本輪動態行為仍待驗證。
- 舊有 fixtures 標示 JAR `bbc59308…`，它們提供既有封包向量。本輪對 `E1498871…` 正式 JAR 的 live 驗證仍待執行。

## 待驗證／未驗證什麼

### 觀戰單獨主動離場

`ClientSession.exitRoomBeforePolyMe:14576` 與 `quitFight:14882` 在 BATTLE 階段都呼叫 `settleRequestedBattleExit:14850`。該方法對整個房間呼叫 `serverEndBattle`，最後 `hub.endBattle(roomId)`。觀戰者會被排除於勝敗集合，發起者的觀戰身分卻沒有進入獨立退出分支。

因此現有 `C_D0`／`C_D9` 契約無法提供所要求的單獨離場。本次維持觀戰進行中退出的拒絕，新增測試確認無退出封包入列。服務端執行者需要補上只移除觀戰者的流程及其回覆契約，確認參戰者仍可移動、戰鬥、結算後，再由客戶端任務接線。

### 原 native 初始鏡頭偏移（第 1 次的紀錄；已由第 2 次補齊）

有限靜態追查使用唯讀 `lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm`，SHA256 為 `2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A`。

- `sub_4AC5E0`（L359867–L360105）解析四個 unsigned WORD 並加入矩形集合。
- `sub_48B640`（L295211–L295403）讀取矩形，使用最小值含括、最大值排除的界線。
- `sub_4A7130`（L349305–L349440）處理 WORLD 的地圖索引，`sub_4AAAB0`（L356357–L356458）處理 E5 角色選項。

上述追查仍未建立觀戰鏡頭起點與畫面偏移的完整資料流。現有 Rust 使用收到的半開矩形中心作為初始位置。本輪驗證了該本地鏡頭的界線及退役行為，原 native 起點等價性列為待驗證，需資源／逆向執行者補齊固定來源與預期座標。

### 其他驗收邊界

- 正式多人模式、人數規則、SQL 戰績與結算收據、實體 GUI／鍵盤滑鼠、完整 G06 與 Release 均待驗證。
- 本輪只使用自建 loopback 對端，正式服務、DB、遊戲程序均未啟動或連線。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_world.rs`
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`
- `LiTo381-Modern/src/modern_lito/native_room_ui.rs`
- `LiTo381-Modern/src/modern_lito/native_room_admin_tests.rs`
- `LiTo381-Modern/src/modern_lito/native_room_flow_tests.rs`
- `LiTo381-Modern/tests/native_observer_camera.rs`（第 2 次）
- `docs/reports/issue-189.md`
- 驗證日誌位於 `LiTo381-Modern/target/current/verification/issue-189/`、`target/current/check-189.log`，一次性腳本與伺服器 Issue 內文在 `scratch/`，都不進 git。

## 給下一棒的提示

- 審查者：先看 CI 的 `native_observer_camera`、`room_flow_tests` 結果。初始鏡頭的逆向依據在「第 2 次嘗試」，行號都可以直接在 asm 裡對。
- 觀戰單獨離場由伺服器 Issue chaoren0418-arch/lito-launcher-server#366 處理。它合併後，開一張客戶端 code Issue：移除 `native_login.rs` 觀戰中退出的拒絕，依 #366 的回覆契約接上 `C_D0`／`C_D9`，並補加密 loopback 測試。
- 服務端合約完成後，本機測試者補跑觀戰者主動離場時其他兩名參戰者持續戰鬥的隔離案例。正式驗收須包含最後成員離房、正常結算返回及重入。
- 全員準備、真人數、假人資格與測試房身分由伺服器裁定。客戶端名冊缺少每位成員的 observer／virtual 身分與 SQL 規則，因此前端須以封包與中文訊息呈現結果。
- 使用者接手最終 GUI 驗收，調度器在合併後補 `docs/REPORT.md` 索引。
