# Issue #330 — T052-1（M3 1／2）：裝備與彈藥 UI

日期：2026-11-12。分支：`task/330`（起點 `origin/main` `91e916c`）。執行者：claude-local。

## 結論

裝備與彈藥 UI 已接進原生視窗：背包清單分成「裝備中／彈藥／其他」三組，世界 HUD 在弓或鐵手甲姿勢時顯示彈藥數量，伺服器的缺彈藥訊息會顯示在提示列。全部只呈現伺服器送來的背包資料，不預扣、不推測。

`cargo check --all-targets` 結束碼 0。**單元測試本機沒跑**（2026-11-09 規定），交給 PR 的 CI；**真機畫面沒看過**，列在「待驗證」。

本張只做「裝備與彈藥 UI」。T052 的另外兩項（地面物品直接滑鼠拾取與排序、特殊物品選項）屬第 2 張，沒有動。

## 做了什麼

### 新模組 `LiTo381-Modern/src/modern_lito/native_equipment.rs`（純邏輯，無 Win32）

| 項目 | 內容 |
|---|---|
| `wear(name)` | 從伺服器名稱判讀裝備狀態：` ($9)` 武器、` ($117)` 防具、` ($10)` 照明。標記後面最多接「次數」「數量」兩組 ` (數字)`；標記在其他位置不算裝備中 |
| `Ammo` | 箭＝useType 254（0xFE）、飛刀＝253（0xFD）；`for_stance`：walk action 20 → 箭、62 → 飛刀、其餘（近戰）沒有 |
| `Summary::of(inventory)` | 把背包分成裝備中（武器→防具→照明）、箭、飛刀、其他；`total()` 用 u64 合計 |
| `Summary::rows(stance)` | 清單列：`【裝備中】武器 1／防具 2／照明 1`、`【彈藥】箭 307（2 疊）／飛刀 1；目前武器使用箭`、`【其他物品】2 項`，每件物品剛好出現一次 |
| `Summary::hud(stance)` | HUD 文字「箭 N」或「飛刀 N」；近戰姿勢不顯示 |
| `MissingAmmoNotice` | 伺服器缺彈藥訊息的顯示期限：收到後顯示到背包下一次變動為止；伺服器再次拒絕時重新顯示 |

### 視窗 `native_login_window.rs`

- 背包清單改用上面的分組。標題列的 ID 是 0（物品 ID 不會是 0），選到標題列時沒有可操作的物品；裝卸仍走原本的「使用／裝卸」按鈕（C1C），沒有新增送包。
- 清單可見列的圖示改成依「實際顯示的列」取，原本是依背包原順序取；分組後兩者順序不同，不改會要求到錯的圖示。
- 標題列文字靠左（不留圖示欄位）。
- 世界 HUD 第一行：`地圖 …　HP …　箭 20　…`（只有弓／鐵手甲姿勢才有）。
- HUD 提示列：伺服器送「沒有可用的箭矢或飛針，請先在商店購買。」時顯示原文。
- **沒有新增任何控制項或版面列**，所以沒有控制項 ID 撞號與版面重疊的新風險。

### 文件

- `docs/NATIVE-RANGED.zh-TW.md`：第 13 行改指向新段落，新增「裝備與彈藥呈現」一段（加在第 37 行之後，TASKS.md 引用的 :13、:37 行號不變）。
- `docs/SKILLS-STATUS.md`：「彈藥」列的 UI 狀態更新。

## 已驗證

### 靜態證據（唯讀，沒有執行任何編譯產物）

正式 JAR：`lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar`，SHA256 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`（本輪重算）。工具：JDK 17 `javap -p -c -constants`／`-v`、Python 讀 class 常數。

| 事實 | E149 位置 | 結果 |
|---|---|---|
| 名稱標記與順序 | `RuntimeInventory$Item.nativeViewName`；BootstrapMethods #3 `\u0001 ($9)`（`weapon()`）、#4 `\u0001 ($10)`（`light()`）、#5 `\u0001 ($117)`（`armor()`）、#6 `\u0001 (\u0001)`（次數，再數量） | 與 `wear()` 的判讀順序一致 |
| 初始背包也帶標記 | `RuntimeInventory$Item.wireItem` 第 48 行呼叫 `nativeViewName` | S16／S7C 與 S18／S6F 用同一個名稱 |
| 彈藥 useType | `RuntimeInventory.qualifiedRuntimeProjectile(int)`：`weaponType == 10` → 253，其餘 → 254 | 一致 |
| 彈藥由伺服器選 | 同上：走訪伺服器背包，取第一疊「未裝備、數量 > 0、`nativeProjectileStatus` 不為 null」 | 客戶端不能指定，所以 UI 不標示「使用中的那一疊」 |
| 姿勢對應 | `RuntimeInventory.weaponTypeForWalk`：20 → 4、62 → 10；其餘 4／11／24／40／46／50／54／58／83 都是近戰 | 與 `Ammo::for_stance` 及單元測試清單一致 |
| 缺彈藥訊息 | `ClientSession.notifyMissingBattleProjectile`：`bipush 9`＋字串＋`Protocol.systemMessage` | 字串以 UTF-8 bytes 在 `ClientSession.class` 內完全相符 |

伺服器原始碼（lito-launcher-server `origin/main` `edfb5c82`，`RuntimeInventory.java:338–348`、`:1497–1514`、`ClientSession.java:13393`）只當閱讀入口；寫進客戶端的值都以上表的 E149 為準。REUSE-MAP 的 T045「箭的扣除：伺服器扣、客戶端只看回包」做法照做。

### 客戶端資源（PakViewer CLI，唯讀，輸出在 `scratch/issue-330/pak/`）

| pak | 查詢 | 結果 |
|---|---|---|
| `LiTo/LTtext.idx` | `pak list --filter desc` | `LTdesc-c.tbl`、`LTdesc-k.tbl`、`LTSpellDesc-c.tbl`、`LTSpellDesc-k.tbl`；沒有 `desc-c.tbl`（載入器會改讀 Text） |
| `Text.idx` | `pak list --filter desc-c`、`pak extract desc-c.tbl` | `desc-c.tbl` 249,300 bytes，SHA256 `F06C94F4…28ADB7DF`，宣告 17001 筆 |

`desc-c.tbl` 的第 9／10／117 筆是「揮舞」「打開」「使用中」，所以清單上裝備中的物品會顯示成 `… (揮舞)`、`… (使用中)`、`… (打開)`。本張沒有缺資源。

### 建置

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

結束碼與原文見本檔最後一節「建置結果」。

## 推測

- 伺服器「第一疊」的順序是伺服器背包的插入順序。客戶端背包用物件 ID 排序，兩者不一定相同，所以多疊箭時畫面上的第一疊不一定是伺服器先扣的那一疊。
- 客戶端合計的箭數可能比伺服器認定的多：伺服器還要求 `nativeProjectileStatus` 有值（伺服器原始碼 `InventoryCatalog.java:1597` 列的是目錄 40738–40748；這份清單沒有另外對 E149 核對），客戶端看不到目錄 ID，只能依 useType 算。出現這種情況時伺服器會送缺彈藥訊息，HUD 會照樣顯示。
- 「飛刀」是 3.81 台版對 useType 253 的慣用名稱；伺服器訊息寫的是「飛針」。兩者指同一類物品。

## 待驗證（需要真機或本機執行測試，本張沒有做）

1. **單元測試**：`native_equipment::tests` 共 6 個，本機沒有執行，等 PR 的 CI（`cargo test --locked --lib --no-fail-fast`）。
2. **真機畫面**（交 Codex 或使用者）：
   - 開背包：三個分組標題與順序、裝備中的物品在最上面、圖示對得上各列。
   - 選一件裝備按「使用／裝卸」：物品在「裝備中」與「其他物品」之間移動，選取不會跳到別的物品。
   - 拿弓進世界：HUD 第一行出現「箭 N」，每射一箭伺服器回包後減 1；換成近戰武器後這段文字消失。
   - 箭用完再攻擊：提示列出現「沒有可用的箭矢或飛針，請先在商店購買。」；買到箭之後訊息消失。
   - 鐵手甲＋飛刀：HUD 顯示「飛刀 N」。
3. 多疊箭時伺服器實際先扣哪一疊（見「推測」第一點）。
4. 高 DPI 下標題列與長名稱的水平捲動範圍。

## 未完成事項

- T052 第 2 張：地面物品直接滑鼠拾取與排序、特殊物品選項（NATIVE-REAGENT-SKILLS:31、:33）。
- 沒有做原版外觀的裝備視窗（人形欄位圖）。目前的原生視窗是 Win32 清單，本張沿用同一個清單做分組。
- `native_skills::equipped_weapon` 仍用 `contains(" ($9)")` 判斷；結果與 `native_equipment::wear` 對正式名稱相同，這次沒有改它（不在本張範圍）。

## 產出路徑

- 程式：`LiTo381-Modern/src/modern_lito/native_equipment.rs`（新增）、`native_login_window.rs`、`mod.rs`。
- 文件：`docs/NATIVE-RANGED.zh-TW.md`、`docs/SKILLS-STATUS.md`、本報告。
- 本機證據（不進 git）：`scratch/issue-330/javap/`（E149 反組譯）、`scratch/issue-330/ls/`（伺服器原始碼快照）、`scratch/issue-330/pak/`、`scratch/issue-330/check.log`。

## 給下一棒的提示

- **Codex（本機測試）**：`cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_equipment`，再照「待驗證」第 2 點看真機畫面。
- **T052 第 2 張**：背包清單的列現在是 `(物件 ID, 文字)`，ID 0 是標題列；要加新的分組或排序請改 `native_equipment::Summary::rows`，不要在視窗裡另外排序。地面物品清單（`ground_view`）沒有動。
- 之後若要做「指定用哪一種箭」，E149 沒有對應的請求（伺服器自己選第一疊），要先在伺服器加功能。
- 本張沒有改 `docs/REPORT.md` 與 `TASKS.md`。

## 建置結果

- 第 1 次：結束碼 **101**。`native_equipment.rs` 的測試模組少匯入 `Item`（E0425／E0422 兩個錯誤，只影響 lib test 目標）。補上匯入後重跑。
- 第 2 次（目前的程式）：結束碼 **0**，原文 ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 46s``。`native_equipment.rs` 與 `native_login_window.rs` 改動的區段沒有新的 warning；其餘是原本就有的 warnings。
- `Cargo.lock` 沒有變動。
- 沒有執行 `cargo test`、沒有執行任何編譯出來的程式、沒有啟動遊戲、沒有連正式服務或 DB。
