# Issue #195 — T063 寵物與召喚物控制介面

## 做了什麼

在 `task/195` worktree 實作正式 GAME 控制器的同伴面板，支援競技場與一般世界。世界畫面右上角的「寵物／召喚物」開啟面板；同伴來自已收到的主人欄位，不以外觀、NPC 模板或項圈名稱猜測所有權。

PR #251 建立後，為解決同期間合入 #192 造成的衝突，在原分支同步 `main@683b603`。僅人工合併 `native_login.rs` 的相鄰模組宣告，保留同伴控制器與強制目標測試兩者；相對最新 main 仍只有本張 9 檔差異，沒有自行修改 #192 的功能或 `docs/REPORT.md` 索引。

| 範圍 | 本張實作與權威界線 |
| --- | --- |
| 控制 | `aggressive`、`defensive`、`stay`、`extend`、`alert`、`attackchr`、`getitem`、`changename`、`dismiss` 九種已知靜態動作，透過 `C_HACTION 0x27 cdss` 送出；不在客戶端模擬 AI 或群組姿態。 |
| 同伴資料 | `C_DIALOG 0x29 cd` → `S_HYPERTEXT 0x2A`，顯示姿態、HP／MP、等級、名稱、攜帶標籤、EXP%、正義值；非同伴對話仍交既有商店流程。 |
| 指定攻擊 | 必須先收到 `S_SLAVE_CONTROL 0x57 dc`，才開放 `C_SLAVE_CONTROL 0x4F cdcd`；原樣回送 mode，檢查目前目標生命與可見範圍。 |
| 改名 | `S_ASK 0x24 h(325)` 後送 `C_ANSWER 0x23 chcs`；驗證 MS950 精確往返、2–27 bytes 與原生命名範圍；新問題取代舊改名回呼。 |
| 項圈與用品 | 選取已收取的背包實例，沿用原生 `C_USE_ITEM 0x1C cds`；補上 `WorldReady` 的一般物品使用與待決操作退役檢查。不能把任意名稱辨識為項圈，也不自行產生召回成功。 |
| 交付／餵食 | `C_GIVE 0x11 cdhhdd`，重查同伴、背包實例、持有數量及距離小於 3 格，使用當下同伴座標；餵食、可交易性、裝備限制由伺服器裁定。 |
| CHA／容量 | 顯示 `S_STATUS`／能力更新已收取的 CHA。來源職業加成與容量規則不複製到客戶端；未收到剩餘容量時明示未知，顯示容量／材料等伺服器拒絕訊息。 |
| 期限／解放 | 封包未提供到期時間，明示未知；只依伺服器 `S_REMOVE_OBJECT 0x15` 移除。解放有明確確認，不先刪除同伴、項圈或背包。 |

請求保留連線、場景世代、角色生命、同伴 incarnation、選取修訂與主人欄位；死亡、控制中斷、同 ID 替換、換場、斷線後拒絕舊選取。選單等待 8 秒、目標提示 15 秒、改名提示 30 秒只限制介面操作，不是同伴生存期限。送包失敗不消耗提示；送出成功只代表已排入傳輸，沒有宣稱伺服器接受。

## 已驗證

本機在 `LiTo381-Modern/` 執行，使用既有 MSVC x64 環境、離線 Cargo 快取，建置與暫存均留在 worktree。沒有啟動遊戲程序或伺服器。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼：0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 37s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_t063
結束碼：0
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 2551 filtered out; finished in 0.01s
```

16 案涵蓋九種動作的確切封包、選單欄位、截短／多餘位元組、非本同伴回覆、提示與一次性回覆、逾時、改名編碼、角色／連線／同伴世代失效、失敗傳輸不消耗提示、容量拒絕不修改背包、項圈使用、`WorldReady`，以及只有伺服器移除才退役同伴。這些是離線合成封包與現有 fixture 測試，不是正式伺服器行為驗收。

既有物品操作與商店回歸結果：

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_item_actions
結束碼：0
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_merchant
結束碼：0
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

從 `維護工具/run-local-test-suite.ps1` 以 PowerShell AST 讀出原本的 `Measure-TestResult` 函式，對四份完整紀錄與結束碼套用同一門檻；`check`、`test`、`item-tests`、`merchant-tests` 均為 **PASS**，解析程式結束碼 0。三個測試 harness 共 36 passed、0 failed、0 ignored、0 個 running-0-tests；沒有把 filtered out 計入通過。未執行整套測試或真機驗收。差異空白檢查通過，保留既有檔案的 CRLF。

## 推測與來源界線

- TASKS 引用的「69 項來源寵物控制」是伺服器續作報告的彙總，沒有對應 69 個客戶端 opcode。這次採用來源中可辨識的九種玩家動作，以及 CHA／容量／項圈的既有送包與顯示入口；**不宣稱重跑或驗收伺服器的 69 項控制**。
- 唯讀核對 `D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`：`Protocol.java` 的 `hypertextActionRequest`、`dialogObjectId`、`hypertext`、`slaveControlRequest`、`slaveControl`、`petRenameAsk`、`askAnswer`、`giveRequest`；`BattleCompanion.java` 的 `anicomArguments`／`apply`；`ClientSession.java` 的同伴選單、目標、改名與交付分派。
- 讀取時 SHA256：`Protocol.java` = `9A0A1E67FF94BD08C3FA5403582807F1A5D0CB74E28AED2ABCB4E7788107F213`；`BattleCompanion.java` = `A8B26B216B4048DC7B6F8AD5D22F6D3CBF9AACB90FE0B27A8D584D70C401D472`。來源不是正式 E149 執行結果；未將來源或歷史候選的測試通過轉述為本張正式驗收。
- 依 `docs/SKILLS-STATUS.md` 受控 NPC／材料「伺服器待定」界線，不新增 CHA 容量公式、召喚存續排程、群組 AI、材料扣除或主動解放計算。

## 待驗證／未驗證什麼

- E149 正式隔離環境中的同伴選單與指令端到端、九種控制對各種 PET／SUMMON／TAMED 的結果、69 項來源控制的完整矩陣。
- 實際項圈召回／解放後刪除、材料／負重／容量拒絕、同伴收取與餵食效果、召喚物自然到期，以及各伺服器待定 AI／群組語義。
- 原生 GUI 實際操作、DPI 與螢幕配置驗收。本張未啟動遊戲，沒有截圖，也未連正式服務或 DB。
- 未新增伺服器未提供的容量／到期欄位。`C_NPC_ITEM_CONTROL` 的同伴裝備列不捏造：目前來源分派明示未提供可裝備模板與配套列表更新，待伺服器契約及已收取列表具備後才能綁定。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_companions.rs`：命令、權威選取、選單／提示解析、請求與退役。
- `LiTo381-Modern/src/modern_lito/native_companion_controller.rs`：GAME 傳輸與收包接線。
- `LiTo381-Modern/src/modern_lito/native_companion_window.rs`：玩家可操作的 Win32 同伴面板。
- `LiTo381-Modern/src/modern_lito/native_companion_tests.rs`：T063 離線測試。
- `native_login.rs`、`native_login_window.rs`、`native_item_actions.rs`、`mod.rs`：必要的註冊與接線。
- `docs/reports/issue-195.md`：本報告；未修改 `docs/REPORT.md` 或任務完成狀態。
- 未追蹤建置／測試紀錄：本 worktree 的 `scratch/check.log`、`scratch/test.log`、`scratch/item-tests.log`、`scratch/merchant-tests.log` 及同名 `.exit`；`scratch/measure.ps1` 重用套件門檻。產物位於 `LiTo381-Modern/target/current`。

## 給下一棒的提示

Claude 審查封包與生命週期邊界；本機測試執行器在另行核准的隔離伺服器環境驗證上述待驗項，GUI 交使用者依 M2／T030 排程驗收。不要從 `anicom` 的攜帶文字推算數值容量，也不要把本機提示倒數當成召喚期限。若伺服器要提供剩餘容量或期限，先訂收包契約再補顯示。
