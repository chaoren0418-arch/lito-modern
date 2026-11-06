# Issue #179 — T045 第 1 張：原生物品操作與物品技能

分支：`task/179`。僅修改本 worktree；原始專案、資源、伺服器來源與正式 JAR 都只讀。未修改 `docs/REPORT.md`，未啟動遊戲、正式服務或 DB，未操作既有遊戲程序。

## 結論

已實作有來源依據的物品目標／文字選項、type3 charge、C76 整疊銷毀及五個物品技能的控制器與介面接線，補上正式 E149 封包 oracle 與回歸測試。Storm_Walk、生命之泉沿用既有權威處理，新增保護與驗證；194／197 新增各自的隔離正式流程測試入口。

**本張尚不能完整結案**：尚有特殊 useType 的完整介面未接，194／197 真正 ClientSession 流程及 GUI 未執行；技能資源回歸出現失敗，詳下列結果。不能把送包／重播驗證當作正式玩法驗收。

## 已驗證

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
| 技能 12／21／73／100／107 | 新增 `native_skills::item_request`、`LoginController::cast_item`，送 `14 row col d(item_objid)`；已學會、存活、連線／物品快照與既有施法 gate 都保留。支援 BattlePending／WorldReady。先在背包選物品、關閉背包後施放；世界角色目標不能混用。 |
| unsigned ID | 以 `0xFEDCBA98`、`0xFFFFFFFF` 驗證雙目標、銷毀、五個技能及 S19／S1A／S17；既有背包模型本來就是 u32，沒有改成 signed。 |
| Storm_Walk 42501 | 沿用已有的 type5 路徑及 `native-target-item-current.json`；本次重驗封包／失效選點／portal feedback，沒有重跑 SQL 或宣稱遊戲內傳送已驗收。 |
| 生命之泉 | 保留 C14 地面請求，依 S4D、S1A／S17 更新 MP 與材料；不從物品名稱／icon 推算 40318。驗證送出不預扣、獨立權威 delta。 |
| 194／197 | 使用共用普通道具請求與既有三種藥水 effect bank；新增 `room-world-item-heal-194`／`room-world-item-heal-197`，隔離 fixture 分別提供 40011／40012，檢查正確 effect、兩次數量變更與 HP、效果退休。入口編譯通過，端到端尚未跑。 |

### 協定證據

- 唯讀正式 JAR：原始專案的 `LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar`，本次實算 SHA256 為 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。
- 新增 `tests/oracle/NativeItemOptionsOracle.java`，以該 JAR 的真正 Protocol parser／writer 執行，硬性核對 SHA；檢查 C76、鑑定／選項、燈具、返回、變形、座標物品及五個物品技能的 unsigned 向量，每個 request 的所有截斷與多餘尾段都必須拒絕。另輸出 S37 gfx194／197、S19／S1A／S17，核對生命之泉 40318×1／20MP catalog。**這個 oracle 不建立 ClientSession、socket、DB 或 Main。**
- `javac` 最終結束碼 0；oracle 執行結束碼 0。初次新增 oracle 曾使用不存在的 `changeInventoryAmount` 方法而編譯失敗，已依正式方法 `changeInventoryCount` 修正，沒有放寬 oracle 斷言。
- `NativeGameHarness.java` 使用正式 E149 JAR 編譯結束碼 0；本次沒有執行它。
- 原版靜態證據：唯讀 `LiTo.unpacked-oep-current.exe`（既有 pin `4271D92F…5C2AB6E9`）之 406FF0（signed charge／`cd`）、406F20（文字選項）、406D80／407120（座標目標）、407810（物件目標）、407760／407890（背包目標）。僅用 pefile／capstone 讀取，未執行舊 BIN。type30 另核對唯讀來源 `C_ItemUSe.java` 的 object dword 分支。

### 怎麼驗證

共同參數：`--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`；工作目錄 `LiTo381-Modern`。

檢查命令：`cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`。

測試命令：`cargo test <共同參數> --test <下表名稱>`。逐一取用 `run-local-test-suite.ps1` 原本的 `Measure-TestResult` 函式判斷，不另造寬鬆門檻；0 tests、解析失敗或 failed > 0 都不算通過。

| 檢查／測試 target | 結束碼 | 門檻結果 |
|---|---:|---|
| `check` | 0 | `PASS` |
| `native_item_actions` | 0 | `PASS` |
| `native_inventory` | 0 | `PASS` |
| `native_skills` | 101 | `FAIL: command-exit,test-failed` |
| `native_skill_selection` | 0 | `PASS` |
| `native_reagent_skills` | 0 | `PASS` |
| `native_item_effects` | 0 | `PASS` |
| `native_inventory_names` | 0 | `PASS` |

`test result:` 原文（同一 target 採最後一次結果）：

```text
native_item_actions
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
native_inventory
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
native_skills
test result: FAILED. 10 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.34s
native_skill_selection
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.20s
native_reagent_skills
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.20s
native_item_effects
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.72s
native_inventory_names
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
```

最終 all-targets 檢查及物品操作重跑結束碼皆為 0；`native_skills` 仍失敗，因此整輪 runner 結束碼為 1。全部執行結果都已解析，沒有 0 tests，也沒有把略過當作通過。

`native_skills` 失敗案例：

- `original_tw_basic_heal_frames_sound_clock_and_retirement_preserve_authority`
- `tw_higher_heal_original_slots_clothes_sound_and_retirement_preserve_authority`
- `tw_higher_full_heal_group_budget_rejects_both_layers_atomically`

第一輪第一案先在寫入 `target/current/verification/native-TW11722-original-frame-metadata.json` 時遇到目錄不存在；其他兩案為 `unreviewed TW healing catalog generation`。資源資格檢查位於本次未修改的 `tower_assets.rs:282–283`，同時核對 pack SHA、text SHA、decoded bytes 及 sprites 數量。本次不更動 pin／不放寬該檢查；最終重跑結果見上表與 final log。這些是實際失敗，不是略過後宣稱通過，也沒有執行未修改分支的對照測試來冒稱已證明 baseline 相同失敗。

## 推測

- 194／197 與生命之泉沿用已有的 request／權威 delta／效果呈現後，預期可完成遊戲內使用；本次沒有用真正 ClientSession 執行這些 gameplay 情境，不能以離線向量、重播或編譯結果提升為已驗證。
- 新增目標／選項的伺服器受理資格、材料足不足及技能結果仍由伺服器決定；客戶端沒有自行判定附魔成功、補血量或裝備結果。

## 待驗證／未完成什麼

1. 特殊 useType 6／29（書籤傳送）、12／13／31–36（信件／卡片）、15（樂器）、28（空白魔法卷軸）與其他未明列類型尚未提供完整選單／回應流程。本張沒有假設所有特殊物品都可用同一尾段。需要接上各自的權威資料及介面後才能稱「全部特殊 useType 已完成」。
2. 真正正式 E149 ClientSession 的 194／197、生命之泉缺料／低 MP／連續消耗、目標道具、銷毀拒絕及五個物品技能，需由本機隔離測試接續。不得連正式服務或 DB。
3. 原生 GUI、鍵盤焦點、Esc、DPI、銷毀確認與背包關閉後施法選取需驗收；本次遵守「不啟動遊戲」。
4. 12／107 依已裝備武器決定目標、三重矢扣箭、NPC 商店 16 列、地圖道具 18 筆屬第 2 張，未修改。地面物品／陷阱鏈未新增。
5. 完整測試結果中的失敗不算通過；未執行的隔離正式流程也不算通過。

## 產出路徑

- `src/modern_lito/native_item_actions.rs`、`native_inventory.rs`：目標／選項／charge／C76 與權威守衛。
- `src/modern_lito/native_skills.rs`、`native_skill_selection.rs`、`native_login.rs`、`native_login_window.rs`：物品技能及原生控制器／介面。
- `src/modern_lito/native_item_probe.rs`、`native_combat_probe.rs`、`tests/oracle/NativeGameHarness.java`：194／197 正式流程探針入口。
- `tests/native_item_actions.rs`、`tests/oracle/NativeItemOptionsOracle.java`、`tests/fixtures/native-item-options-e149.json`：測試與正式協定向量。
- `LiTo381-Modern/target/current/issue-179/`：不入 git 的完整檢查／測試 log 與 summary；`target/current/issue-179-oracle/`：不入 git 的 Java class。
- 本報告：`docs/reports/issue-179.md`。

## 給下一棒的提示

請 Claude 審查本分支的封包形狀、選項取消／過期守衛與已知限制，保持 T045 未完成狀態。本機測試執行器接續 194／197 正式流程與失敗資源測試前置；特殊 useType 的剩餘選單由後續 code 接續，不能將本報告當作全部物品系統已驗收。

194／197 新模式可沿用既有隔離 harness／probe 協定，但執行前需按正式 E149 與獨立 memory store 的既有規則準備；不要直接套用舊腳本指向不同 SHA 的 reference-snapshot。本次只做 compile／offline parser 測試。第 2 張只需延伸物品技能的目標選擇政策，保留這裡 `cast_item` 的 u32 objid 與快照驗證。
