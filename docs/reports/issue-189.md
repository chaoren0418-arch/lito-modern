# Issue #189 — T054-2 房間模式、隊伍、準備與觀戰整合

## 結論

本張完成可由客戶端獨立修正的房間流程與離線回歸，**整張尚未完成**。觀戰單獨主動離場仍有服務端依賴，原 native 初始鏡頭偏移仍缺完整來源契約。T054 與 G06.2 的整體驗收需要繼續追蹤。

基底為 `e83403140e814e45fc7c53ed0cb8929424f5a079`，工作分支為既有 `task/189`。

## 做了什麼

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

### 原 native 初始鏡頭偏移

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
- `docs/reports/issue-189.md`
- 驗證日誌位於 `LiTo381-Modern/target/current/verification/issue-189/`，一次性腳本位於 `scratch/`，均不入 git。

## 給下一棒的提示

- Claude 審查 `task/189` 的客戶端變更與離線測試。調度器保留 T054-2 未完成狀態，分別交服務端執行者處理觀戰獨立離場、交資源／逆向執行者提供原 native 初始鏡頭契約。
- 服務端合約完成後，本機測試者補跑觀戰者主動離場時其他兩名參戰者持續戰鬥的隔離案例。正式驗收須包含最後成員離房、正常結算返回及重入。
- 全員準備、真人數、假人資格與測試房身分由伺服器裁定。客戶端名冊缺少每位成員的 observer／virtual 身分與 SQL 規則，因此前端須以封包與中文訊息呈現結果。
- 使用者接手最終 GUI 驗收，調度器在合併後補 `docs/REPORT.md` 索引。
