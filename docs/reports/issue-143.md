# Issue #143：T020-2 暗隱術（第三次嘗試）

本次在既有 `task/143`、基底 `77f30e1` 上補正式玩家 callback 與 S37/169 巢狀 drain 接線，完成本張本機驗證，提交 Claude 審查。沒有切換／另建分支，沒有人工編輯 `docs/REPORT.md` 或修改 T020 的整體狀態；#142 與完整 M2 驗收另行裁定。

首次提交 `ac9ea21` 後，PR #252 顯示與主線 `683b603` 衝突，已在同一分支整合並重跑下列所有檢查。四處衝突均在 `native_login.rs`：保留主線 `CombatInput`／LFC1 append，再經本張 `send_player_action`，並將移動中的 attack gate 放在新 `attack_with_input` 入口。其他主線內容原樣保留；`docs/REPORT.md` 與該主線版本完全一致，沒有人工編輯索引。

## 做了什麼

- `native_static2941::Queue` 的同一 due queue 同時保存玩家 source-frame callback、camera callback 識別及 static retirement。保留 unsigned deadline 排序、同期限 FIFO、remove-before-call、固定外層 cutoff、fresh enqueue sample 與重入退役身分守衛。正式 `drain_with` 接受借用的繪圖 host；舊測試 helper 委派相同的巢狀 drain 核心。
- 新 `native_player_callbacks` 擁有 source frame、held／pending／cancel／continuation 狀態。成功 `move_presented` 後從 frame 1 起步，按真實 catalog 的每幀 duration 逐段整數運算預排 callback，各 enqueue 重新讀 native clock。callback 推進 frame 至零；只有 held input 存在且沒有新收到的角色動作，才排續步。續步重新檢查真實導航，成功送出 move 後才 predict 並建立下一步。
- `4E2150` 兩個 suffix 直接接正式繪圖 host：非零 source frame 走一般 suffix；frame 零先處理 cancel，再處理待送動作，最後才處理 continuation draw。控制狀態來自 Windows held input／前景與視窗狀態、已驗證的施法請求、成功 transport send，以及角色動作佇列，不從 heading 或 elapsed pose 推造。
- `4E8750` duration 使用 catalog 原 ticks／FPS，speed、brave 分段截斷；特殊 7/16 分支由選角 mode／等級與 actor status 控制。normal-tail 使用真正完成的新 composition、fresh clock、原始 unsigned delta；frame count=12 且 delta>40 才抑制 suffix draw，之後才 cap 至500。重複畫同一 Frame 不重算。
- 新 `native_login_callbacks::DrawHost` 在 callback 返回前同步完成既有 CPU renderer／SPR 解碼，接收 `Prepared`、enqueue 靜態 retirement，再合成可發佈的 Frame。不是只送出視窗重畫要求，也不是只排下一張 worker。`World.player_walk` 是 source-frame 權威；renderer 用它選圖與位置，完成後交回既有 Animator，避免同一步從 elapsed=0 再播。
- 正式 `WM_TIMER → poll_presented → accept_game_received` 在成功接受 World 封包後檢查成功動作送出旗標、graphics=169、target=目前 own actor。每個接收事件後先完成巢狀 pump，才處理下一個事件。入口取消 Camera 識別的 callback、重設 camera 狀態；至少一次 drain→draw→fresh relative sample，以 signed wrapping 累加至500；只有正常完成才寫完成旗標。World／queue 退役不誤寫完成。
- 可安全延後的已驗證自體施法在步伐 frame 零送出，回覆 timeout 與施法冷卻以實際送出時刻調整；有指定目標、地面、物品與傳送請求仍保留既有等待條件。`action_sent` 只在 transport 接受送出後設立；message 0x49 與場景控制換代會清除，不能以 `skill_pending` 代替。
- 保留前兩次已移植的12 ready gates、原 PE／ASM 控制流核對、方向6 flags=0／alias=-1／shadow=-1 與固定2941 SPR fragments／lease／History 接線；本次把它們和正式 producer 一起重跑。

## 已驗證

### 行為與整合

新增15個純邏輯案例與3個 controller 案。純邏輯涵蓋兩 suffix、cancel 優先、pending 只送一次、fresh continuation 與固定 cutoff、真正續步、收到戰鬥動作後不續步、12幀 draw 抑制、整數 duration、169 正反條件、camera 取消、rollover、巢狀 drain 內執行玩家 callback、重入退役及重複畫面不讀新 clock。

兩個資源整合案使用唯讀 `D:/lineage開發/0-客戶端` 的實際地圖／SPR、正式 `Presentation`／`Model` 和本測試自己建立的 `127.0.0.1:0` 加密 loopback。沒有連外部遊戲伺服器：

1. 從載入的真實地圖選取方向6可走的邊，正式 `move_presented` 送出封包並建立 source walk。步伐中正式 `cast` 保留待送請求，frame 零才由 peer 收到。callback 內合成2941並以 cutoff+1 fresh sample enqueue，舊 cutoff 不能消費；下一 drain 使 lease 退役、舊 Frame 失效，下一次真實合成移除 fragments。
2. 正式 `cast` 成功送出後，peer 送出自己的 S37/169，正式 `poll_presented` 收包並完成真實繪圖 pump；最終 Frame 通過 input gate。另一案確認 transport 送出失敗不建立169旗標。

測試初輪曾因 fixture 的 Windows accepted socket 繼承非阻塞模式，以及原 fixture 座標位於地圖牆邊而失敗；已明確設定測試 peer 阻塞／timeout，並從真實導航選擇可走邊。未放寬正式導航、加密、輸入或資源 gate。純測試的正常速度前置亦已明確設定，未把原 fixture 的減速狀態誤當正常速度。

### 建置與測試原文

所有命令在本 worktree 的 `LiTo381-Modern/` 執行。`CARGO_HOME=D:/dev-cache/cargo`、`RUSTUP_HOME=D:/dev-cache/rustup`；TEMP／TMP 使用本 worktree `scratch/tmp`。一次性 `scratch/run143.ps1` 以 AST 讀取現行 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult` 原函數；0 tests、failed>0、解析失敗與非零結束碼均不算通過。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
exit=0
gate=PASS
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib
exit=0
gate=PASS
test result: ok. 2005 passed; 0 failed; 585 ignored; 0 measured; 0 filtered out; finished in 9.42s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_player_callbacks::tests:: -- --test-threads=1
exit=0
gate=PASS
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 2575 filtered out; finished in 0.01s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_login::player_callbacks::tests:: -- --include-ignored --test-threads=1
exit=0
gate=PASS
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2587 filtered out; finished in 12.13s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_static2941::tests:: -- --include-ignored --test-threads=1
exit=0
gate=PASS
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 2559 filtered out; finished in 7.79s
```

全 library 的 ignored 不算通過。另行 opt-in 執行的31個 static 案包含全部12 ready gates；兩個真實資源 controller 案也有獨立非零執行結果。日誌為本 worktree `scratch/third-*.log` 與 `third-*-result.txt`。既有 warnings 仍存在，沒有宣稱零警告。

### 來源與 metadata

原 PE／ASM 僅唯讀核對，未載入執行：

- `D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`，SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`。
- 同分析根目錄 `ida-client-oep-v2/full-disassembly.asm`，SHA256 `2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A`。

從原專案 `驗證紀錄/current/native-E149-static2941-queue-metadata/20261024-203626-169258/cohort.json` 的 paired_tests／input_pins 重新唯讀核對七份 QA 本文 SHA256 與單一 hardlink count，只將 QA 放入本 worktree `scratch/metadata/`。fixture include 重綁至本 worktree；object374448 五處方法名依當版 API 適配為 `observe_item`，預期值／分支不變。未套用候選 Runtime。

| QA | 原文 SHA256 |
|---|---|
| blind20 | `03DEB74A4883C28A96058AEC59ABE0E2EBE0BE9514B7364F3FE5210E7A77C5B4` |
| slow29 | `CFA9685F0B3EBB1098C83CA140D17955B42AE59E3AE90EDD8F3CE8006EDE3B44` |
| advance79 | `96DB321632AA8868E4F5230C1E22E143090B6E1F777F488F4D4E4F9E4FBFA935` |
| item_spells | `FFFA7E2FD2476715C558D146A745228D1924A4D23D30686B958FDC15D5D06556` |
| magical73 | `5F90100B56394690393168F2F4B41720E928642B0F3F9EFFDC7C839CDC6DE1DE` |
| shadow107 | `F73F627CAB38B179228F0B61F13593E3E8A4ABF805DBCF8E4CD89BB426D22919` |
| object374448 | `F170F1B1B857297EB579283C7AF897A73019CD65AA33C5AB6A7C65B4787587EE` |


Debug／Release 各執行 `cargo check --lib --tests --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --message-format=json`（Release加 `--release`），核對 library、library-test artifact 與 build-finished.success。七份 QA 各用當版直接相依與 library metadata 執行 `rustc --test --emit=metadata --edition=2021 --target x86_64-pc-windows-msvc`，Release加 `-C opt-level=z -C debug-assertions=off`。

```text
debug library-test=False exit=0 fresh=True
debug library-test=True exit=0 fresh=True
debug advance79 exit=0
debug blind20 exit=0
debug item_spells exit=0
debug magical73 exit=0
debug object374448 exit=0
debug shadow107 exit=0
debug slow29 exit=0
release library-test=False exit=0 fresh=False
release library-test=True exit=0 fresh=False
release advance79 exit=0
release blind20 exit=0
release item_spells exit=0
release magical73 exit=0
release object374448 exit=0
release shadow107 exit=0
release slow29 exit=0
```

完整命令／log 位於 `scratch/metadata/<profile>-<name>-command.txt`、`current-<profile>-<name>.log` 與 `current-results.txt`。metadata **僅算編譯成功，七個整合測試本文未執行**；無 `test result:`。產物只在 `LiTo381-Modern/target/current/issue143/metadata/`，不入 git。

## 推測

- 原控制流的 held／pending／cancel／成功送出旗標，現映射至本客戶端實際擁有的輸入與 transport 狀態；來源靜態核對與本地整合支持此映射，但沒有原程序同步 trace，不能宣稱所有原生輸入分支逐指令等價。
- 同步 CPU composition／資源 enqueue 已驗證；Windows 最終呈現的可見幀數及長耗時 callback 的手感仍需要 GUI 觀測。

## 待驗證／未驗證什麼

- 未執行完整原生 GUI、正式服務／DB、原遊戲程序、獨立原像素 oracle、Release 測試或 G06 640次。沒有啟動／附加／停止遊戲程序。
- 本張 source-walk adapter 支援真實 catalog 的普通八方向4／6／8／12幀步伐；不合條件的特殊姿勢沿用既有 Animator。不是整個 `4E2150` 全分支、原 camera renderer 或所有動畫事件 interpreter 的重寫；camera callback 的識別／取消及 pump 狀態已接，本張不宣稱新增完整 camera 追蹤功能。
- #97 的 UI 技能請求擴充與其他分張不在本次修改；2941 收包／SPR／queue 與相關 ready gates 的驗證不等於整個技能或 M2 已完成。既有一般施法14僅用於驗證成功發送／延後發送 authority 與169條件。

## 產出路徑與給下一棒的提示

- 新增：`LiTo381-Modern/src/modern_lito/native_player_callbacks.rs`、`native_player_callback_tests.rs`、`native_login_callbacks.rs`、`native_login_callback_tests.rs`。
- 接線：同目錄 `native_static2941.rs`、`native_login.rs`、`native_login_window.rs`、`native_world.rs`、`native_world_animation.rs`、`native_world_scene.rs`、`native_world_view.rs`、`native_movement_tests.rs`、`mod.rs`。
- 最新報告：`docs/reports/issue-143.md`。一次性腳本與日誌在本 worktree `scratch/`；建置產物在 `LiTo381-Modern/target/current/`，均不提交。
- Claude 接手 code review，重點核對正式呼叫鏈、兩 suffix 優先順序、169 旗標 authority、同步 composition 與外層固定 cutoff 的交錯，以及上述支援範圍。調度器依審查結果處理 PR／佇列；使用者依專案規則保留合併與後續 GUI 驗收。
- 本次沒有人工編輯 `docs/REPORT.md`，沒有修改原始專案、機密設定、封存或硬連結；修改檔案的 link count 均核對為1。沒有發送 GitHub 留言，也沒有合併 PR 到 main。
