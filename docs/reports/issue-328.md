# Issue #328 — T049（M3 1／2）：背包與介面文字

範圍：祝福光暈、詛咒色盤、物品名稱 runtime 模板與格式顏色（含遞迴引用的結論）、地面與世界物品名稱。
本張不含：數字通知的 Text 資源映射、LTstring 原版 UI、S0F／S10 系統訊息 GUI（T049 第 2 張）。

分支 `task/328`。本機只做靜態分析與 `cargo check`，**沒有在本機執行任何測試或程式**；測試結果以 PR 的 CI 為準。

## 做了什麼

### 1. 靜態逆向（recovered PE，SHA256 `4271D92F…5C2AB6E9`，Capstone 解碼，未執行）

| 位址 | 結論 |
|---|---|
| 442A30 | 背包圖示繪製。+0x78 identified、+0x70 blessing；未鑑定→一般；已鑑定 0→光暈、2→詛咒、1／3→恆等調色盤。LT 圖示（≥20000）分支相同 |
| 4B9BF0、4B9E80 | 光暈遮罩（36×36，兩輪各 4 個位移）與遮罩內目的像素的查表轉換 |
| 41A2B0、4EC290、4EC1C0／4EC1F0 | 調色盤 83533C 的建表方式：往白色靠近三分之一 |
| 4B9A90、4C1170、4C1A40 附近、41A240 | 詛咒：顏色 `dword_50352C`（紅 7），混色表內容為 `顏色 + 像素/2` |
| 442980、4EC338 | 背包名稱顏色表 `word_834F34`＝調色盤第 24、14、17、7 色；索引為「已鑑定 ? blessing : 3」 |
| 4437F0、44A680、4436C0 | 名稱結尾 `(數字)` 加千分位；超過 23 bytes 換成兩行（後者未實作） |
| 4D44A0 | `$n` 展開：查不到輸出 `???`、字表項原樣複製不再掃描、第一個 `%i` 換成 75F71C 的字串、`$` 後非數字只丟掉 `$` |
| 492F1E、444A23 | 75F71C 的另外兩個引用：492F1E 在進入角色時把角色紀錄開頭字串複製進去，444A23 只讀取 |
| 4318E0（431030、431E50 同構） | 文字繪製的 `\fX`、`\bX`、`\iN`、`\k`、`\j`、`\c`、`\\` 處理；`\fX` 取 `dword_503440[X-'0']` |
| 503440–503524 | 57 色 RGB555 調色盤（已抄入 `native_text_format::PALETTE`） |
| 489E40–48A720 | 世界名稱（全部名稱模式與指向模式）。沒有地面物品專用分支；`+`／`-` 前綴省略到第一個空白，就是地面物品的強化值前綴 |

字表實況（PakViewer CLI 唯讀匯出 `Text.pak/desc-c.tbl`，SHA256 與既有文件相同；LTdesc-c 用既有解碼檔）：desc-c 16945 列中 67 列含 `%i`、38 列含 `\f`（`\fT` 30、`\f2` 7、`\fY` 1）、0 列含 `$`；LTdesc-c 171 列都沒有。

### 2. 程式

- 新增 `src/modern_lito/native_text_format.rs`：57 色調色盤、`runs`／`plain`（控制碼拆段）、`name_color`、`group_digits`／`inventory_view_name`、`rgb`。
- `native_inventory_names.rs`：`Catalog::expand(token, character)` 依 4D44A0 重寫（`resolve` 等於不帶角色名稱的 `expand`）；新增 `label_as`（帶角色名稱）、`plain`；`Label` 多一個 `color`。
- `native_inventory.rs`：`label_with_name` 在去掉重複數量後套用千分位規則。
- `native_inventory_icons.rs`：`IconStyle`、`halo_blend`、`curse_tint`、`Icon::halo`（每格被幾輪光暈蓋到）、`Icon::bgra_styled`；`bgra` 行為不變。
- `native_login_window.rs`：背包清單依物品 identified／blessing 畫光暈或詛咒色、名稱用原版顏色；清單列與空白區改黑底（選取列仍用系統反白）；背包名稱帶入目前角色名稱；「附近物品」清單的名稱去掉控制碼。
- `native_world_labels.rs`：名稱與稱號改用 `expand`（帶自己的角色名稱），並把 `\fX` 拆成多段顏色繪製（第一段維持原 kind，其餘為 `run`）。地面物品走同一條指向名稱路徑。
- `tests/native_inventory_names.rs`：依新行為更新（此檔 CI 只編譯不執行）。
- 文件：`docs/NATIVE-INVENTORY-ICONS.zh-TW.md`、`docs/NATIVE-INVENTORY-NAMES.zh-TW.md` 補上本次結論與剩餘項。

### 3. 行為改變（審查請留意）

- 查不到的 `$n` 以前整串保留原文（世界名稱則不畫），現在顯示 `???`，與原版相同。
- 含 `\f` 的字表項以前整串保留原文，現在會解析：背包顯示去掉控制碼的名稱；世界名稱依控制碼換色。「試煉的…」系列怪物名稱（`\fT`、`\f2`）以前不會顯示，現在會。
- 背包清單變成黑底。這是為了讓白／淡黃名稱與光暈看得見的版面選擇，不是原版背包框。

## 已驗證

- 上表各位址的指令與常數：直接解碼 PE bytes 得到（工具 `scratch/t328/dis.py`，不入 git）。
- 兩張字表的 `%i`／`\f`／`$` 統計：對實際字表掃描。
- `cargo check --locked --offline -j1 --all-targets`：見文末「建置結果」。

## 推測

- 75F71C 是「目前角色名稱」：依 492F1E 的複製來源判斷，沒有追到該紀錄的建構處。
- 多段顏色之間的 x 前進量為每 CP950 byte 6px：依定位程式推得，4315B0 本身沒有解碼。
- 世界物件名稱與背包名稱走同一個 4D44A0：沿用既有文件的判讀，本次只確認了呼叫點清單（41 個呼叫點），沒有逐一對應封包。
- blessing 大於 3（封印 128–131）時原版會讀到表外；這裡圖示退回一般畫法、名稱用白色。原版 0E 更新封包是否先遮罩這個值沒有查。

## 待驗證

- 新增的 lib 單元測試（`native_text_format` 4 個、`native_inventory_names::expand_tests` 4 個、`native_inventory_icons::style_tests` 5 個、`native_world_labels::run_tests` 1 個）**本機未執行**，等 CI。
- 光暈與詛咒色沒有和原版畫面逐像素比對；565 顯示模式的綠色表沒有實作（公式等效，未驗）。
- 黑底背包清單、名稱顏色、光暈在實機上的外觀（需開遊戲，交 Codex／使用者）。
- 地面物品指向名稱在實機上的顯示（顏色走一般物件分支 4E9220，沒有為物品另外判定）。
- `tests/native_inventory_names.rs` 等整合測試需本機資源，CI 不跑。

## 未做

- 名稱超過 23 bytes 的兩行換行（4436C0／414080）。
- 442A30 開頭的圖示底圖（`dword_648EC0[圖示]+0x14` → 4B9A40）。
- `\k`／`\j`／`\c` 的字碼頁切換（目前只略過）。
- 「全部名稱」模式（`byte_64BC13`／`64BC14`）仍未接線，地面物品名稱只在指向時顯示。

## 查過的資源

- `D:\lineage開發\0-客戶端\Text.idx`／`Text.pak`：`desc-c.tbl`（PakViewer `-cli export`，輸出到 `scratch/t328/res/`）。
- `LiTo/LTtext.pak`：`LTdesc-c.tbl`（既有解碼檔 `lito脫殼資源/resources/decoded/LTtext/row00176-LTdesc-c.tbl`）。
- 本張沒有宣稱缺資源。光暈與詛咒色都是程式算出來的，不需要額外圖檔。

## 參考來源（REUSE-MAP）

- tw-bin-engine `origin/main`：`engine/source/PlayerChat.cs`（`\f` 調色盤索引 `c-0x30`）、`PlayerHudChat.cs`（96 色調色盤，前 53 色與 LiTo 相同，之後不同，所以用 LiTo.bin 的 57 色）、`NativeUi/ItemIconSource.cs`、`ObjectRendering.cs`。TW 沒有光暈、詛咒色與依祝福狀態的名稱顏色，這三項只來自 LiTo.bin。沒有複製任何資源或程式碼。
- lito-launcher-server：只用了 recovered PE（`lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`，唯讀）。

## 給下一棒的提示

- T049 第 2 張的系統訊息可以直接用 `native_text_format::runs` 處理 `\fX`；string-c 的 `%0`／`%s` 模板是另一個函式，本張沒有看。
- 要做原版背包框時，`Icon::bgra_styled` 的背景參數換成框的底色即可，光暈會跟著底色算。
- 若要確認 75F71C，從 492F1E 往上找 `ebx` 的來源。

## 建置結果

本機 `cargo check --locked --offline -j1 --all-targets --target-dir target/current`：通過（exit 0，3 分 16 秒；只有既有的命名風格警告）。這只代表可編譯，測試結果看 PR 的 CI。
