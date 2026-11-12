# Issue #179 — T045 第 1 張：原生物品操作與物品技能

分支：`task/179`。僅修改本 worktree；原始專案、資源、伺服器來源與正式 JAR 都只讀。未修改 `docs/REPORT.md`，未啟動遊戲、正式服務或 DB，未操作既有遊戲程序。

## 結論

已實作有來源依據的物品目標／文字選項、type3 charge、C76 整疊銷毀及五個物品技能的控制器與介面接線，補上正式 E149 封包 oracle 與回歸測試。Storm_Walk、生命之泉沿用既有權威處理，新增保護與驗證；本輪補上信件 12／13、哨子 15、逐項伺服器阻塞依據，以及可重跑的 E149 ClientSession 測試執行器。

**本張尚不能完整結案**：TW 治療資源資格不符仍需處理，不能放寬 `tower_assets.rs` 門檻；特殊 useType 6／29／28／31–36 的伺服器阻塞見下表。本輪實際測試結果及未修改基準分支對照列於下方。

## 第 3 次（2026-11-12，claude-local）：與 main 同步

本輪接手卡在合併佇列的舊分支，只做同步與合併後修正，**沒有新增本張範圍外功能**。

### 做了什麼
- `git merge origin/main`（885c5c0，之後再同步到 main 最新版；#151 已把 107 加進 `item_targeted`，本分支再補 73）。衝突 5 檔：`native_item_actions.rs`（3 處，兩邊內容相同，取 main）、`native_login.rs`（4 處）、`native_login_window.rs`（6 處）、`native_skill_selection.rs`、`native_skills.rs`。
- **物品技能改用 main 的設計**：main 由 #145（T027 批一）已加入 `native_skills::ItemTarget`（含連線 epoch、InputStamp、背包 revision、地圖快照）與 `item_targeted`／`cast_item(&ItemTarget)`，背包開著時選物品施放。本分支舊的 `inventory_target`／`item_request(s, skill, &Choice)`／「關閉背包後施放」流程已移除，改將 main 的 `item_targeted` 由 `12|21|100` 擴充為本張要求的 `12|21|73|100|107`。`native_login.rs` 衝突全取 main（含 `cast_with_input`、傳送書籤、`CombatInput`）。
- `native_login_window.rs`：保留本分支的 C76 銷毀、選項送出／取消、信件欄位；移除殘留的 `skill_item_choice`，技能說明文字改用 `skill_item_target`。
- **修正合併後的控制 ID 衝突**：本分支 `ITEM_MAIL` 與 main 的 `SKILL_LABEL` 同為 318（`cargo check` 出現 unreachable pattern 警告，會讓信件按鈕與技能標籤共用同一 HWND ID）。`ITEM_MAIL` 改為未使用的 321。
- `native_skill_selection.rs`：保留 73／107 的「選取背包物品」標籤，12／21／100 用 main 既有標籤，預設分支用 main 的 `catalog::name`。
- `tests/native_item_actions.rs`：兩個物品技能測試改用 `ItemTarget::capture(&s, 2, u32::MAX)` 與新簽名 `item_request(&s, 2, skill, &target)`；斷言內容（`14 row col FF FF FF FF`、未學會拒絕、charge 變動／物品移除拒絕、E149 fixture 向量）不變。

### 怎麼驗證（本輪）
| 指令 | 結束碼 |
|---|---:|
| `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` | 0（無 error；本輪造成的 unreachable pattern 警告已清除） |
| `cargo test <同參數> --no-run --test native_item_actions --test native_skills --test native_skill_selection --test native_t021 --test native_t027_batch1 --test native_item_sessions --test native_inventory` | 0（只編譯） |

依使用者 2026-11-09 規定，本機**不執行**編譯出的測試程式；本輪沒有 `test result:` 原文。測試交 GitHub Actions（`cargo test --locked --lib`）；整合測試 target 未在 CI 執行，列為待驗證。下方「已驗證」各節是**第 2 次（合併前程式碼）**的本機結果，合併後未重跑，不能當成目前分支的通過證據。

### 本輪待驗證
1. 合併後 `native_item_actions`、`native_skills`、`native_t021`、`native_t027_batch1` 等整合測試的實跑結果（交 Codex／CI）。
2. 73／107 改走 main 的 `ItemTarget` 路徑後，背包開著時施放的 UI 流程（原生 GUI 驗收）。
3. `formal_item_session` 五情境合併後重跑。

---

## 已驗證（第 2 次，合併前程式碼）

### 做了什麼

| 範圍 | 實作及界線 |
|---|---|
| type3 | 僅原版 signed charge 為正（1–127）時送 `1C d(objid)`，沒有字串尾零；0、128–255 拒絕。UI 顯示可用次數。收到 S19 才改 charge，不預扣。原版動作 17 的本地 callback 時序未重現。 |
| type4／16 | 先保存來源快照，再由文字欄送 `1C d(objid) s(MS950)`；拒絕空白長度、NUL／控制字元、不可無損編碼字元及超過 255 bytes。type4 額外檢查 charge。 |
| type5／17 | 世界選點送 `1C d(item) d(target) h(x) h(y)`；保留 owner、場景、控制狀態、角色 incarnation、目標座標／可見性與 frame 檢查。 |
| type8／30 | 選世界物件送 `1C d(item) d(target)`，不追加座標；type8 允許已死亡的可見物件作為目標，仍由伺服器判定復活資格。 |
| type7／14／26／27 | 先按使用，再選背包目標、確認；來源與目標皆重驗完整物品快照與 owner，送兩個實際 objid，不使用目錄 ID。 |
| type9／10 | 返回道具送目前地圖及兩個零 word；燈具送 Windows uptime dword。兩者不混用普通空字串格式。 |
| 其他耗材 | 補已核對的 signed useType 預設分支 `F7–FA、FC–FE`；`FF` 因船票與其他物品共用同一值，維持拒絕，沒有用名稱／圖示猜測格式。 |
| C76 | 新增「銷毀整疊」與獨立確認／取消，顯示來源物品；`76 d(objid)` 沒有數量欄位。改選物品後不能確認舊物品，S17 前不移除任何資料。 |
| 選項生命週期 | 來源更新、換角色／連線、換場景或控制中斷後拒絕；Esc／取消／關閉背包會取消選項。單一物品 pending gate、逾時不重送。 |
| 技能 12／21／73／100／107 | 送 `14 row col d(item_objid)`；已學會、存活、連線／物品快照與既有施法 gate 都保留。支援 BattlePending／WorldReady。（第 3 次起改用 main 的 `ItemTarget`／`cast_item`，背包開著時選物品施放；本列原「關閉背包後施放」流程已移除。） |
| unsigned ID | 以 `0xFEDCBA98`、`0xFFFFFFFF` 驗證雙目標、銷毀、五個技能及 S19／S1A／S17；既有背包模型本來就是 u32，沒有改成 signed。 |
| Storm_Walk 42501 | 沿用已有的 type5 路徑及 `native-target-item-current.json`；本次重驗封包／失效選點／portal feedback，沒有重跑 SQL 或宣稱遊戲內傳送已驗收。 |
| 生命之泉 | 保留 C14 地面請求，依 S4D、S1A／S17 更新 MP 與材料；不從物品名稱／icon 推算 40318。驗證送出不預扣、獨立權威 delta。 |
| 194／197 | 使用共用普通道具請求與既有三種藥水 effect bank；新增 `room-world-item-heal-194`／`room-world-item-heal-197`，隔離 fixture 分別提供 40011／40012，檢查正確 effect、兩次數量變更與 HP、效果退休。本輪已使用兩個真實玩家 Session 執行，證據見下方；沒有用舊單人 bot 探針代替。 |

### 協定證據

- 唯讀正式 JAR：原始專案的 `LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar`，本次實算 SHA256 為 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。
- 新增 `tests/oracle/NativeItemOptionsOracle.java`，以該 JAR 的真正 Protocol parser／writer 執行，硬性核對 SHA；檢查 C76、鑑定／選項、燈具、返回、變形、座標物品及五個物品技能的 unsigned 向量，每個 request 的所有截斷與多餘尾段都必須拒絕。另輸出 S37 gfx194／197、S19／S1A／S17，核對生命之泉 40318×1／20MP catalog。**這個 oracle 不建立 ClientSession、socket、DB 或 Main。**
- `javac` 最終結束碼 0；oracle 執行結束碼 0。初次新增 oracle 曾使用不存在的 `changeInventoryAmount` 方法而編譯失敗，已依正式方法 `changeInventoryCount` 修正，沒有放寬 oracle 斷言。
- `NativeGameHarness.java` 使用正式 E149 JAR 編譯結束碼 0；本次沒有執行它。
- 原版靜態證據：唯讀 `LiTo.unpacked-oep-current.exe`（既有 pin `4271D92F…5C2AB6E9`）之 406FF0（signed charge／`cd`）、406F20（文字選項）、406D80／407120（座標目標）、407810（物件目標）、407760／407890（背包目標）。僅用 pefile／capstone 讀取，未執行舊 BIN。type30 另核對唯讀來源 `C_ItemUSe.java` 的 object dword 分支。

### 第 2 次修正：信件與特殊 useType

- type12：背包內輸入收件人、標題、內文；使用正式 `1C d(objid) h(0) s(MS950 收件人) S(UTF16LE 標題) S(UTF16LE 內文)`。普通紙的 code=0；收件人上限 27 bytes、標題 20 個 UTF16 units、內文 256 units。拒絕不可編碼收件人、控制碼、超長輸入與過期快照。依 E149 `useLetterPaper` 只在 BattlePending 送出，取消不送。
- type13：送普通空字串 envelope；原子解析正式 S5E 的 unsigned objid、icon/code、MS950 名稱、UTF16LE 內容與日期。只有當前 owner 的現存 type13 物品才能收信，更新伺服器 icon；背包內顯示唯讀內容。換角色、連線、移除物品後不沿用內容，新的同物品回覆才完成 `MailReceived`。
- type15：正式 catalog 40315=WHISTLE，送普通空字串 envelope。E149 回覆 `S4A c(0) h(437)`，沿用既有 `native_world`／`native_wire_audio` 播放；音效沒有物品 objid，因此不將任意音效當成物品消耗 ACK，背包不預扣。
- 新增測試涵蓋封包成功向量、取消、場景／角色／物品／phase 過期、拒絕輸入、所有截斷與多餘尾段、S5E 未知物品與跨 owner、unsigned objid。沒有把封包測試當作寄信真流程或 GUI 驗收。

下列類型有具體伺服器阻塞，不能用客戶端虛構選單或回覆補齊：

| useType | E149 的可定位證據 | 明確處置／接手 |
|---|---|---|
| 6 | 40100 的 effect=NONE；`sourcePoisonBody$useInventoryItemBeforeTravel` 沒有書籤位置 handler，末段 bytecode 6181–6202 僅記錄無效果後返回；外層 GmTravel 僅處理自己的 stormItems 等分支。 | 客戶端顯示「伺服器待定」，拒絕送出。伺服器 Claude 必須提供書籤清單回覆、選擇請求與場景更新契約後解除。 |
| 29 | 正式 catalog 掃描 1..600000 無此 useType；相同 dispatch／GmTravel 無群體書籤處理器。 | 伺服器 Claude 補物品定義、書籤群組資格與正式 handler；本張保持阻塞，不排成未定客戶端 code。 |
| 28 | 40090–40094 均 type28、effect=NONE；dispatch 無空白卷軸寫入技能分支，非空 argument 亦不符合普通使用分支。 | 伺服器 Claude 補可寫技能來源、選擇格式、扣料與成品更新；本張明確拒絕。 |
| 31／32／33／34／35／36 | 同一掃描範圍各為 0 筆；已有普通紙 40310/type12、閉信 49016/type13、開信 49017/type13，不能據此推定卡片 code／範本。 | 伺服器 Claude 分別給出六種類型的 catalog、letterCode/template 與 handler；未有權威資料前各自阻塞。 |

上述 catalog 範圍由可重跑的 `tests/oracle/NativeItemSupportOracle.java` 硬 pin E149 後查核，`javac`／執行結束碼均 0；輸出 `target/current/issue-179/revision2/support-oracle.log`。不存在的結論僅限明列掃描範圍；handler 結論另依正式 bytecode，不只依目錄掃描。dispatch 摘錄在 `target/current/issue-179/revision2/E149-item-dispatch.txt`；完整唯讀反組譯暫存在 `scratch/E149-ClientSession.txt`、`scratch/E149-GmTravel.txt`。這些類型的拒絕路徑有測試，不宣稱不存在的成功路徑已通過。

### 正式 E149 ClientSession 隔離測試

執行入口：`LiTo381-Modern/tools/run-item-sessions.ps1 -ReferenceJar <正式 E149 絕對路徑> -Mode <情境>`。

執行器先編譯，再啟動自有 loopback listener；真實 Rust transport／LoginController 的兩個玩家各經 LOGIN→GAME→大廳→房間→戰鬥。每輪是 4 個實際 ClientSession、獨立 GameStateStore(memory)、不持久化、不讀 SQL；完成後檢查 membership 清空、worker_quiescent、shutdown=closed。這是無 GUI 的 Cargo integration test，不啟動遊戲 EXE，也不啟停既有程序。

角色與庫存的初始資料透過 memory repository/store 準備：194／197 各兩瓶；生命之泉設定已學 63、MP80（低 MP 為 2）、寶石兩顆（缺料為 0）。另在任何 socket／Session 建立前，將正式 `BattleSpellCatalog.groundSpell(63)` 的 MP、reuse、duration、reagent/count 初始化成隔離 JVM 的單一 `CombatDatabase.Snapshot` 記憶體列；沒有呼叫 SQL loader。E149 新 `NativeCastMaterial` 的預設來源為 0，未準備該列會錯把材料視為無消耗，故以實際扣料斷言攔下。這是明列的初始 fixture，並非完整 SQL 資料庫驗收。正式 E149 對裝飾過的 repository 要求 `NativeCastMaterialPolicy.Source`，測試來源驗證 account/object/name 後提供普通角色 AccessLevel=0；沒有 GM 免料、固定亂數、改伺服器時鐘或執行中反射改權威狀態。首次 JVM 真實 SecureRandom 於 ready 前初始化，保留原握手期限。

第一次生命之泉跑出 `material character policy provider unavailable`；這是測試前置不完整，已補上上述正式介面。執行器拒絕把 policy provider 錯誤當缺料測試通過。另一輪缺少靜態技能列時 MP 已扣但寶石未扣，測試亦失敗；補齊資料後才重跑。缺料／用盡的正式回覆為 `NativeCastMaterial.reject` 的 S_MESSAGE_CODE=299，低 MP 則沿用正式 NoResponse。首次 JVM 未預熱時的握手逾時亦不算通過。藥水探針現在在原有 12 秒期限內等待實際退休，逐次要求繪製，不再於固定 3.5 秒直接裁定；最後 197 重跑通過。

| Mode | Cargo 結束碼 | Java 結束碼 | Gate |
|---|---:|---:|---|
| `item-heal-194` | 0 | 0 | `PASS` |
| `item-heal-197` | 0 | 0 | `PASS` |
| `item-stream` | 0 | 0 | `PASS` |
| `item-stream-no-reagent` | 0 | 0 | `PASS` |
| `item-stream-no-mana` | 0 | 0 | `PASS` |

```text
item-heal-194
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.08s
item-heal-197
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.61s
item-stream
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 36.81s
item-stream-no-reagent
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.41s
item-stream-no-mana
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.92s
```

每種 Mode 的可定位證據是 `target/current/native-real-peers/issue-179-<Mode>/test.log`、`summary.json`、`result.json`、`server.log`、`server-trace.jsonl`、`a-client.json`／`b-client.json`。

194／197 都由另一真實玩家正常攻擊後補血，驗證兩瓶的數量 2→1→移除、首次 HP 實增、滿血第二次不越界、對應 gfx 正確、各次效果繪製並退休、送出不預測／重複 pending 拒絕。生命之泉正常情境兩次成功後第三次缺料拒絕，MP80→60→40，寶石2→1→移除；缺料與低 MP 各連續兩次拒絕且不改 MP／庫存、不生成效果。各狀態數值另摘錄於 `revision2/<Mode>-compact.json`，完整資料保留於 client JSON。

### Cargo 檢查與回歸

共同參數：`--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`。

完整檢查：`cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`。

相關測試：`cargo test <共同參數> --test <target>`；正式流程使用 `--test native_item_sessions formal_item_session -- --ignored --exact --nocapture`。該測試預設 ignored，普通全套未執行時不算通過；本報告只計明確以 owned harness 執行的結果。

兩種 runner 都取用 `維護工具/run-local-test-suite.ps1` 原 `Measure-TestResult` 函式，0 tests、failed>0、解析失敗、非零結束碼均不通過。

| 檢查／target | 結束碼 | Gate |
|---|---:|---|
| `check` | 0 | `PASS` |
| `native_item_actions` | 0 | `PASS` |
| `native_inventory` | 0 | `PASS` |
| `native_skills` | 101 | `FAIL: command-exit,test-failed` |
| `native_skill_selection` | 0 | `PASS` |
| `native_reagent_skills` | 0 | `PASS` |
| `native_item_effects` | 0 | `PASS` |
| `native_inventory_names` | 0 | `PASS` |

`test result:` 原文：

```text
native_item_actions
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
native_inventory
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
native_skills
test result: FAILED. 10 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.43s
native_skill_selection
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.43s
native_reagent_skills
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.12s
native_item_effects
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.59s
native_inventory_names
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
```

原始 log 在 `target/current/issue-179/revision2/final/`；最後 all-targets 與物品操作重跑在 `revision2/retry/`。完整相關測試 runner 因 TW 三案結束碼 1，不能稱整輪通過；最後針對性重跑結束碼 0。以上都是本機結果，未宣稱 GitHub CI 通過。

### TW 治療資源與基準對照

三個失敗案例：`original_tw_basic_heal_frames_sound_clock_and_retirement_preserve_authority`、`tw_higher_heal_original_slots_clothes_sound_and_retirement_preserve_authority`、`tw_higher_full_heal_group_budget_rejects_both_layers_atomically`。

`tower_assets.rs` 要求的 TW pack SHA 為 `B3788DF4C7F077EF3FF15B070A5D020CCB224E48DEC3757C4CD34528FCA4B218`、text SHA `7EBE2B464C8B1EA9C58AC985DD8FF4C823784D8231894262EDDCD7691CA94008`、decoded bytes=5974928、sprites=5。實際唯讀 `D:/lineage開發/0-客戶端/launcher/TW13081901.pak` 為 `92B512A32349D0775E1036F1832E21F1141AE66E64821D6978F0879246E59CA7`，大小 1406534；既有 #111 報告已記錄其 text/decoded generation 不同。未修改 pin 或任何資源資格條件。

在允許讀取的客戶端、登入器來源及原專案（排除封存、scratch、歷史暫存證據與連結）查找，未找到符合 pin 的檔案（原專案查找紀錄：`revision2/resource-search.json`）；未讀寫封存目錄或操作硬連結。不能將未知來源複製改名後宣稱合格。

未修改基準：`de461994fc0a595f627d1e5a0a01a2e15ef0eaea`（前次提交 7a5251f 的 parent）。以 `git archive` 擷取 Cargo/build/src/tests/windows/tools 到本 worktree 的 `scratch/issue179-baseline/`，不 checkout、不改分支。僅建立未追蹤 verification 產物目錄，使用同一唯讀資源跑 `native_skills`，來源程式不修改；基準的 `--target-dir` 使用本 worktree `LiTo381-Modern/target/current` 絕對路徑，其餘 Cargo 參數相同。基準實跑結束碼 **101**，Gate `FAIL: command-exit,test-failed`；原文：

```text
test result: FAILED. 10 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 16.49s
```

`target/current/issue-179/revision2/baseline/native_skills.log` 可定位到相同三案與 `unreviewed TW healing catalog generation`。基準測試檔 blob 為 `9fb19eb43a7eab6ebe8f4f90c512f59071b6bc6e`、資源 loader blob 為 `6a4e16c80a19b6150f2b40a2b3e5a75ffc68d54f`，均與 parent commit 相同。這證明本輪環境下基準亦失敗；不等同資源已修復或本輪 gate 通過。

## 推測

封包向量、解析與取消／過期測試支持介面接線的正確性，但不能推定寄信、讀信、銷毀及全部物品技能在正式連線中均完成驗收。物品資格、施法結果、材料消耗仍由伺服器決定。

## 待驗證／未完成

1. TW 三案仍須取得合格資源並重跑；即使基準同樣失敗，也不能將目前 gate 標成 PASS。
2. useType 6／29／28／31–36 按上表由伺服器 Claude 接手權威契約；本張將逐項阻塞保留，不提供假的成功選單。
3. 信件與哨子本輪為正式 parser/writer＋客戶端回應模型測試，非完整寄送 ClientSession 驗收；完整 GUI、焦點、Esc、DPI 與背包呈現未啟動驗收。
4. 第 2 張的 NPC 商店 16 列、地圖道具 18 筆、三重矢扣箭、12／107 依已裝備武器選目標不在本輪修改範圍；地面物品／陷阱鏈未新增。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_item_actions.rs`、`native_inventory.rs`、`native_login.rs`、`native_login_window.rs`：選項、信件與控制器／介面接線。
- `LiTo381-Modern/src/modern_lito/native_peer_actions.rs`、`tests/native_item_sessions.rs`、`tests/oracle/NativePeerHarness.java`、`tools/run-item-sessions.ps1`：正式雙 Session 測試。
- `LiTo381-Modern/tests/native_item_actions.rs`、`tests/oracle/NativeItemOptionsOracle.java`、`tests/oracle/NativeItemSupportOracle.java`、`tests/fixtures/native-item-options-e149.json`：回歸及正式協定／目錄查核。
- `LiTo381-Modern/target/current/issue-179/revision2/`、`target/current/native-real-peers/issue-179-<情境>/`：本機未追蹤 logs、summary、server trace、客戶端狀態、軟體 renderer 圖片與 receipts。它們不是原生 GUI 截圖。
- 本報告：`docs/reports/issue-179.md`；沒有改 `docs/REPORT.md`。

## 給下一棒的提示

- （第 3 次）分支已與 main 885c5c0 同步；物品技能唯一入口是 `native_skills::ItemTarget` + `item_targeted`（現含 73／107）。之後不要再加第二套 `Choice` 版施法路徑。新增視窗控制 ID 前先查 `native_login_window.rs` 的 const 表，避免再撞號（本輪 318 撞號已改 321）。

Claude 審查本分支時請分別裁定：客戶端修正、正式五情境證據、逐項伺服器阻塞、TW 資源門檻。保持 T045 未完成；不要將 baseline 同樣失敗改記成通過。

請調度器將 TW 合格資源定位交本機 resource 執行器（可參照 #111／T057 的 generation 說明），取得符合現有 pin 的來源後重跑 `native_skills`；不要叫使用者自行準備或放寬 loader。上述特殊類型交伺服器 Claude 明確補契約，再由本機 code 接回。本張第 2 分張執行者繼續沿用 `cast_item` 的 u32 objid／owner 快照守衛。
