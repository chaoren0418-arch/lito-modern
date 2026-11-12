# Issue #331 — T052-2（M3 2／2）：地面物品直接滑鼠拾取與排序、特殊物品選項

日期：2026-11-13。分支：`task/331`（起點 `origin/main` `5331d47`）。執行者：claude-local。

## 結論

- **直接滑鼠拾取**：點畫面上的地面物品，現在照原版 `sub_4062E0` 處理——一格內直接送拾取（堆疊送整疊），一格外先走過去再撿；986 ms 內不重送。按住 Shift 點堆疊會改開數量面板。
- **排序**：「附近物品」清單改成先近後遠（同距離依 Y、X、物件 ID）。**畫面上的繪製先後沒有改**，原因見「未完成事項」。
- **特殊物品選項**：目標、文字、信件這些選項在 #179 已經接好，這張查到一個實際缺口並補上：腰帶（useType 37）等 E149 視為防具的型別，客戶端原本會拒絕使用，現在走普通使用（裝卸）。6／28／29／31–36 仍然等伺服器，沒有動。

`cargo check --all-targets` 的結果見最後一節。單元測試本機沒跑（2026-11-09 規定），交 PR 的 CI。**真機畫面沒看過**，列在「待驗證」。

## 做了什麼

### `native_ground_pickup.rs`（純邏輯，新增）

| 項目 | 內容 |
|---|---|
| `click(own, item, count, pick_all)` | 點到地面物品後要做的事：數量 0 → 不處理；一格外 → `Approach`；一格內單件或 `pick_all` → `Send(整疊)`；其餘 → `Prompt(整疊)` |
| `unobstructed` | 物品不在腳下時，自己往物品那一步走不過去就不撿 |
| `prompt_quantity` | 數量輸入：留白等於整疊；0、超過整疊、不是數字都不送 |
| `Throttle` | 距上次送出不足 986 ms 就不送 |
| `Approach` | 走向一格外的物品：每個 tick 給下一步朝向；物品、場景或控制狀態一變就結束；最多 64 步、20 秒 |
| `order` | 清單排序：距離、Y、X、物件 ID |

原有的 `Choice`、`request`（C0B 14 bytes）、`Pending` 沒有改。

### `native_login_window.rs`（接線）

- 左鍵點到地面物品 → `ground_click`。一格內 → `ground_pick`（重新 `Choice::capture`、查通行、查節流後呼叫既有的 `controller.pickup`）；一格外 → 記下 `Approach`，由 `input_tick` 裡的 `pickup_tick` 用既有的 `move_presented` 一步一步走。
- 取消走向物品的時機：Esc、鍵盤移動、開始攻擊、任何新的左鍵點選、開背包／租裝／寵物視窗、失去焦點、死亡確認、離開世界。
- 面板的「拾取」按鈕：數量改用 `prompt_quantity`（留白等於整疊），並套用同一個 986 ms 節流。
- 「附近物品」清單用 `order` 排序。
- HUD 提示列顯示「正在走向地面物品」、拾取狀態或拒絕原因（約 5 秒）。沒有新增控制項。

### `native_item_actions.rs`

- 新增 `armor_use_type`（2、18–25、37、40、43–45、47–54、70），`default_use_type` 把它們納入普通使用。實際多出來的是 37、40、43–45、47–54、70。

### 文件

- `docs/NATIVE-GROUND-PICKUP.zh-TW.md`：新增「直接滑鼠拾取與清單排序」一節（原版依據表、新引擎做法、與原版不同之處）。
- `docs/NATIVE-WORLD-LAYERS.zh-TW.md`：點選地面物品那一句改成現在的行為。
- `docs/NATIVE-REAGENT-SKILLS.zh-TW.md`：文末加一段指向本張（第 31、33 行的行號不變）。
- `docs/SKILLS-STATUS.md`：「使用物品（其他分支）」「裝備與卸下」「地面拾取」三列改成現況。第一列原本寫「尚未實作」，其實 #179 已經做完，這次一起更正。

## 已驗證（唯讀靜態證據；沒有執行任何編譯出來的程式）

`docs/REUSE-MAP.md` 已查：T052 沒有專屬列；用到 T045 列（use_type 要對 E149 核對）與 T029 列（`TW:src/ApproachWalk.cs` 的走近做法）。TW 的 `docs/reference/world-objects.md` 第 1–2 節（`origin/main`，TW16 BIN 的拾取流程）只當閱讀入口，位址屬於 TW16，沒有拿來用；下面的位址都是 LiTo.bin 自己的。

### 原版 LiTo.bin

來源：`lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`（SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`，本輪重算）與同目錄 `ida-client-oep-v2/full-disassembly.asm`。只讀檔，沒有執行。

| 事實 | 位置 | 對應的實作 |
|---|---|---|
| 互動距離是 `abs(dx2) ≤ 2` 且 `abs(dy) ≤ 1`（X 是兩倍座標）＝一格 | `sub_4062E0` `loc_40630B` | `DIRECT_REACH = 1` |
| 不在腳下時查「往物品那一步」能不能走，不能走就不撿 | `loc_4063B6`，0x4063D4 呼叫 `sub_48B640`；`sub_48B640` 讀地格旗標 bit 0 並檢查斜角兩側，移動函式 `sub_4E1DB0` 也呼叫它 | `unobstructed`＋`Navigation::can_step` |
| 數量 > 1 且 `byte_502278 == 0` 才問數量，否則送整疊 | `loc_40644D` | `click` 的 `Prompt`／`Send` |
| 數量 > 0 且距 `dword_63D5F0` 滿 0x3DA ms 才送 `chhddc`（opcode 0x0B）；沒滿不送也不排隊 | `loc_40646B` | `Throttle`（986 ms） |
| 數量回呼：取消不送；空白＝整疊（`dword_636D3C`）；`%u` 解析；0 或大於整疊不送 | `sub_406620` | `prompt_quantity` |
| 互動模式 3 走向目標，走完且目標還在記下的位置時再呼叫 `sub_4062E0` | `sub_4E2150`（`loc_4E2583`、0x4E27BD） | `Approach`（步進方式是新引擎的，見「推測」） |
| `byte_502278` 在 OEP 映像的初值是 1 | pefile 讀 VA 0x502278 | 預設送整疊 |
| useType 分派：`movsx eax, byte [esi+6Ch]; dec eax; cmp eax, 23h; ja default`，1..36 以外都走普通使用 | `sub_445C10` `loc_445D96` | 37 等型別送 `1C d s("")` |

### 正式 JAR E149

`lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar`，SHA256 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`（本輪重算）。工具：JDK 17 `javap -p -c -constants`、Python `zipfile`。

| 事實 | 位置 |
|---|---|
| `armor()` 直接回傳 `armorUseType(useType)` | `InventoryCatalog$Definition.armor` |
| `armorUseType` 的 tableswitch（2..70）回傳 true 的是 2、18–25、37、40、43–45、47–54、70 | `InventoryCatalog$Definition.armorUseType` |
| 型別名稱對應：belt → 37、food → 38、spell_long2 → 39、earring → 40、fishing_rod → 42、aid_left → 43、aid_right → 44、enc → 46、talisman／shoulder → 49、choice_doll → 55、pants → 70 | `ClassicWorldDropCatalog.nativeUseType`（位移 2250–2300） |
| 租裝目錄有 6 件 belt | JAR 內 `rental/rental-item-definitions-380.tsv` 的 `[armor]` 區段 |

所以修改前，租裝拿到的腰帶在背包裡按「使用／裝卸」會被客戶端擋下（訊息是「請選取此物品需要的目標或選項」），封包不會送出。

### 資源

本張沒有宣稱缺資源，沒有做 pak 查詢。

## 推測

- `byte_502278` 是「全部拾取」選項：依據是它在 `sub_4062E0` 裡的用法，以及 TW16 對應位置的說明；LiTo.bin 的設定畫面與設定檔沒有查，啟動後會不會被改成 0 不知道。
- `byte_64BC13`、`byte_64AFBF`（`sub_4062E0` 開頭與類別 9 分支都會看）推測是幽靈／觀戰一類的狀態旗標，沒有追。客戶端用既有的 `available`（存活、非觀戰、沒有移動限制）把關。
- 原版模式 3 的走路是它自己的尋路；新引擎用的是滑鼠追擊同一套貪婪步進（每步讓距離變小，直線被擋時試相鄰兩個方向）。兩者在空曠地會走到同一格旁，有障礙時路線可能不同。
- E149 對 37 等型別的裝卸流程與 18–25 相同：依據是 `armor()` 共用同一個判斷；沒有逐一追 `ClientSession` 的裝備處理。40、43–45、47–54、70 在目前的租裝目錄裡沒有物品，只有 37 有。

## 待驗證（需要 CI、真機或本機執行測試；本張沒有做）

1. **單元測試**（交 PR 的 CI，`cargo test --locked --lib`）：`native_ground_pickup::tests` 6 個、`native_item_actions::tests` 1 個。結果見最後一節。
2. **真機畫面**（交 Codex 或使用者）：
   - 站在物品旁（一格內）左鍵點它：直接撿起，不開面板；堆疊整疊進背包。
   - 站在兩格以上左鍵點物品：角色走到旁邊後撿起；途中按 Esc、方向鍵或點別處會停下來。
   - 隔著牆或柵欄點相鄰的物品：提示「與物品之間有障礙」，不送包。
   - Shift＋左鍵點堆疊：開「附近物品」面板、選到該物品、數量預填整疊；把數量清空再按「拾取」會撿整疊。
   - 一秒內連點兩件物品：第二下提示「拾取間隔未到」。
   - 「附近物品」清單：腳下的排最前面，越遠越後面。
   - 租裝拿到的腰帶：背包裡按「使用／裝卸」可以穿上與脫下，穿上後移到「裝備中」分組。
3. 既有的整合測試（`tests/native_ground_pickup.rs`、`tests/native_item_actions.rs`、兩個隔離 TCP 情境）這次沒有改，也沒有跑；它們不在 CI 的 `--lib` 範圍內。
4. 高 DPI 下提示列文字長度。

## 未完成事項

- **畫面上的地面物品繪製順序沒有改。** 現在的順序是既有的半格掃描＋depth＋反向插入（`NATIVE-WORLD-LAYERS`），角色之間的插入順序是新引擎自己訂的。要讓地面物品與角色在同一格時的先後跟原版一致，得先重建原版的物件插入時序；這部分沒有證據，而且改動會影響 `native_world_layers` 的 8 張像素參照，本機不能跑測試確認，所以這張沒有動。建議另開一張 resource 任務先查原版類別 9 物件的插入點。
- 送出拾取前的本機轉向（`loc_4063E7`）沒有做。
- 「全部拾取」沒有設定畫面；用 Shift 代替。
- useType 6／28／29／31–36 仍是「伺服器待定」；38、39、42、46、55、0xFB、0xFF 沒有 E149 處理依據，仍不送。

## 產出路徑

- 程式：`LiTo381-Modern/src/modern_lito/native_ground_pickup.rs`、`native_item_actions.rs`、`native_login_window.rs`。
- 文件：`docs/NATIVE-GROUND-PICKUP.zh-TW.md`、`docs/NATIVE-WORLD-LAYERS.zh-TW.md`、`docs/NATIVE-REAGENT-SKILLS.zh-TW.md`、`docs/SKILLS-STATUS.md`、本報告。
- 本機證據（不進 git）：`scratch/issue-331/`（E149 的 javap 輸出、解析腳本、`check.log`）。

## 給下一棒的提示

- **Codex（本機測試）**：`cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_ground_pickup`，再照「待驗證」第 2 點看真機畫面。隔離 TCP 的 `room-world-pickup` 兩個情境走的是控制器，不經過視窗的節流，應該不受影響，但請重跑確認。
- 直接點選的節流只放在視窗（`Ui::pickup_throttle`），控制器的 `pickup` 沒有加；之後若有別的入口（例如快捷鍵拾取）要共用，請用同一個 `Throttle`。
- 原版還有鍵盤拾取（TW16 是 F4；LiTo.bin 的對應函式這次沒有找），沒有做。
- 要做「全部拾取」設定時，把 `ground_click` 的第二個參數接到設定值即可，`click` 已經支援兩種情況。
- 本張沒有改 `docs/REPORT.md` 與 `TASKS.md`。

## 建置結果

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

- 結束碼 **0**（第 1 次就過），原文 ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 15s``。`--all-targets` 包含 lib 的測試目標，所以新增的 7 個測試有編譯過，但沒有執行。
- 這次改的三個檔案沒有新的 warning；其餘是原本就有的 warnings。`Cargo.lock` 沒有變動。
- 本機沒有執行 `cargo test`、沒有執行任何編譯出來的程式、沒有啟動遊戲、沒有連正式服務或 DB。
