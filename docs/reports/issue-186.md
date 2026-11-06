# Issue #186 — T048 復活分支與榮譽延遲重生

## 做了什麼

- 以 `e83403140e814e45fc7c53ed0cb8929424f5a079` 為基底，在既有 `task/186` worktree 修改；沒有切換或建立分支。範圍為原生封包接收、死亡確認、復活生命週期與同範圍測試。
- 保留既有 `S_RESURRECT 0x11` 格式：`c d(target) c(action) d(source) h(graphics)`，恰好 12 bytes；target、source 非零，保留 unsigned ID／action／graphics，不要求 source 是自己或仍在畫面內。
- 死亡確認重新檢查目前 HP=0、死亡角色、觀戰／結算狀態與尚未確認的動作；只有 CE0 成功排入 transport 才消耗待決要求。送出失敗與計數溢位不部分更新。
- CE0 不再安裝一般登入操作的 8 秒回覆期限。復活／結算由伺服器決定，不以本機 5 秒計時器復活、恢復 HP 或重送 CE0；傳輸錯誤與使用者退出仍由原流程處理。獨立退出請求的期限不會被復活封包清除。
- 自己復活時清除舊生命的攻擊、技能、物品、拾取、門與入口待決，以及技能重用等待和傳送選擇；同 poll 死亡／復活也適用。遠端復活只中止以該角色為目標的待決攻擊，不清除自己死亡要求。
- 支援非榮譽分支的 `S11 → S12 → HP/ratio` 出生點重新定位：該次位置更新直接定位，不播放遠端復活角色從來源格走到出生點。HP／ratio 關閉此短暫狀態，榮譽原地重生後下一次正常行走仍播放。物件移除、替換、換圖和結算會清除狀態。
- 結算後忽略格式合法的遲到 S11，不重新恢復角色；缺席目標保留 orphan 診斷且不建立角色。HP、MP、血量比例、位置與控制解除仍只採各自的權威封包。

## 已驗證

### 正式 JAR 靜態核對

唯讀來源：`D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar`。

SHA256：`E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。

本輪執行 `javap -p -c`，只反組譯 `ClientSession`、`GameHub`、`Protocol`，沒有執行遊戲方法、啟動伺服器或連 DB。輸出在本 worktree 的 `scratch/issue-186/ClientSession.txt`、`GameHub.txt`、`Protocol.txt`。

- `ClientSession.confirmDeath`：offset 5 消耗死亡確認；39–51 呼叫正式擊敗判定；94–207 區分 respawn、HONOR_MATCH 排程與其他模式 `respawnAtBattleSpawn`；259–320 是 ended 的結算分支。未 ended 且未 respawn 可直接 return，不保證 CE0 一定立即有回覆。
- `GameHub.scheduleHonorRespawn`：offset 109–115 將任務以 **5000 MILLISECONDS** 排程；完成時再次核對房間、場次與 token。此時間不是客戶端權威，也不是畫面倒數來源。
- `ClientSession.serverHonorRespawnDue`：offset 100–176 廣播 `Protocol.resurrect(self, 0, self, battleGfx)`，182–224 送自己 HP 和廣播 ratio；保留死亡位置，沒有 S12。
- `ClientSession.respawnAtBattleSpawn`：offset 92–151 建立 S11 與經 `NativeMovement.legacyDestinationPacket` 轉換的 S12，193–236 對每位成員先送 S11 再送 S12，後續送 minimap／HP／ratio。客戶端繼續使用既有 source-cell S12 解碼規則。
- `Protocol.resurrect`：offset 0–44 核對 DWORD 身分、BYTE action 與 WORD graphics；52–74 依序寫入 `17, target, action, source, graphics`。source 與 target 無須相等。

### 建置與測試

| 項目 | 結束碼 | 原套件門檻 | 日誌（worktree 根目錄相對路徑） |
|---|---:|---|---|
| 首次 `cargo check --all-targets` | 0 | PASS，3m 06s | `scratch/issue-186/check.log` |
| 新增復活測試（最終） | 0 | PASS，12 passed | `scratch/issue-186/resurrection-tests.log` |
| 既有移動測試 | 0 | PASS，10 passed | `scratch/issue-186/movement-tests.log` |
| `native_lifecycle` | 0 | PASS，3 passed | `scratch/issue-186/lifecycle-tests.log` |
| `native_combat` | 0 | PASS，11 passed | `scratch/issue-186/combat-tests.log` |
| `native_light_lifecycle` | 0 | PASS，1 passed | `scratch/issue-186/light-tests.log` |
| 最終 `cargo check --all-targets`（含所有修改） | 0 | PASS | `scratch/issue-186/check-final.log` |

最終 37 個唯一測試實跑通過，0 failed、0 ignored；五組 `test result:` 原文依上表順序如下：

```text
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 2527 filtered out; finished in 8.64s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 2529 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

測試設計：11 個離線模型／控制器／畫面授權測試，加上 1 個自有 `127.0.0.1:0` 模擬 TCP 對端測試。後者使用真正的 `LoginController::confirm_death`、加密 transport 和輪詢；對端解密核對 CE0，實際等待 8.5 秒才送出 S11／HP／ratio，再核對沒有第二次 CE0。測試結束取消自己建立的連線並 join 自有執行緒；沒有啟動外部遊戲程序。8.5 秒是人工延遲交付測試，不是 E149 的排程時間。

離線測試的角色／握手底稿來自既有 `native-world-current.json`、`native-version-current.json`；既有 `native_lifecycle` 使用 `native-lifecycle-current.json`（BBC59308）。這些合成或歷史 fixture 的 PASS 不等於 E149 雙玩家 PASS。

首次新增測試執行結束碼 **101**：10 通過、1 失敗。離線控制器共用的角色 fixture 帶徽章，輪詢時觸發沒有 transport 的徽章查詢，導致 `Failed`；測試前置改為清除與本張無關的徽章，未修改產品徽章處理。之後新增真正經過 CE0 transport 的 8.5 秒延遲測試，避免只靠離線控制器測試判定期限行為。失敗日誌保留於 `scratch/issue-186/resurrection-tests-attempt1.log`，不計為通過：

```text
test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 2527 filtered out; finished in 0.04s
```

### 怎麼驗證

以 PowerShell AST 僅載入 `維護工具/run-local-test-suite.ps1` 原始 `Measure-TestResult` 函式，不執行其他套件。每次 Cargo invocation 分別判定 `running`、唯一且可解析的 `test result:`、結束碼、failed 與零測試；本次測試全部實跑，filtered out 不計通過。暫存 runner 為 `scratch/issue-186/validate.ps1`。

在本 worktree 設定 `PATH=D:\dev-cache\cargo\bin;…`、`CARGO_HOME=D:\dev-cache\cargo`、`RUSTUP_HOME=D:\dev-cache\rustup`，`TEMP`／`TMP` 指向本 worktree 的 `scratch/issue-186/tmp`。以下在 `LiTo381-Modern/` 執行：

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib '::resurrection_tests::' -- --nocapture
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_world_scene::movement_tests::
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_lifecycle
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_combat
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_light_lifecycle
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

作業系統預設不允許直接執行 `.ps1`，因此暫存驗證 runner 透過 `powershell.exe -NoProfile -ExecutionPolicy Bypass -File …` 在單一子程序執行；未修改系統執行原則。首輪與最終日誌均保留既有 compiler warnings，沒有將 warning 清理混入本張。

## 推測

- 移除 CE0 的一般 8 秒期限，可避免排程／網路延遲或「尚未結算且不重生」狀態被誤判為登入回覆逾時；本輪沒有量測正式網路延遲。
- S11 後至下一次 HP／ratio 的位置更新視為重生定位，依據 E149 上述送包順序。這是客戶端呈現判斷，不是額外的伺服器 ACK。

## 待驗證／未驗證什麼

- 本輪離線封包與控制器測試不等於 E149 真實雙玩家榮譽戰鬥通過；需使用 #174／#175 的 owned loopback fixture 驗證自然死亡、實際 CE0、約 5 秒後伺服器重生、連續兩次生命與重入。
- Quest／Siege 等其他模式的自然死亡、出生點阻擋或重生取消、網路中斷，以及正式服務和 DB 均未執行。
- 原生 GUI、完整死亡／復活動畫、聲音、跨 DPI、實際鍵鼠與長時間遊玩未驗收。
- 技能 61／75 的施法及 question 321／322 同意問答仍屬 T027 對應技能實作；本張處理共用 S11／死亡確認分支，沒有套用隔離候選或宣稱那些技能與問答已完成。技能 165 也不因本張而成為可施放。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_world.rs`
- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_world_scene.rs`
- `LiTo381-Modern/src/modern_lito/native_resurrection_tests.rs`
- 本報告：`docs/reports/issue-186.md`
- 不追蹤的本輪日誌與反組譯：`scratch/issue-186/`；建置產物：`LiTo381-Modern/target/current/`。

## 給下一棒的提示

- Claude 審查本張生命週期改動與 E149 送包順序，尤其 CE0 不再使用一般回覆期限，以及不清除獨立退出期限的行為。
- 本機測試接手真實榮譽與其他模式復活；使用雙真實玩家 fixture，不沿用 schema37 訓練 bot 舊 PASS。需要的測試前置與模式可達性先由本機處理。
- T027 的技能問答接線時，應防止等待明確同意期間自動 CE0 消耗死亡流程，並沿用本張 S11 生命更新；不要自行補 HP、重生時間或經驗恢復。
- 未修改 `docs/REPORT.md`、`TASKS.md` 或其他任務狀態；由調度器在合併後更新索引與任務裁定。
