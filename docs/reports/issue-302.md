# Issue #302 — T035（M3 2/2）版面與文字：長提示被裁切、名稱數量重複、失焦後持續發招（claude-local）

## 結論

三項都改了程式，並把判斷規則抽成不碰視窗的純函式加上單元測試。**本機只做了編譯檢查；單元測試等 CI，畫面與實際按鍵的效果都還沒有實機證據。**

1. **長提示被裁切**：世界畫面的狀態區換文字時不再跳回第一行，預設會跟到最後一行；另外修掉兩處「狀態區蓋住下方選項列／技能書」的重疊。
2. **名稱數量重複顯示**：背包、歸還清單、同伴物品、附近物品清單，名稱結尾和權威數量相同的 ` (數量)` 不再重複顯示。
3. **失焦後持續發招**：原本已處理切走與失去焦點；本輪補上「拖曳／縮放視窗」「系統選單」「最小化」「視窗被停用」這幾種收不到放開按鍵的情況。

依本機限制，本輪沒有執行任何編譯出的程式，也沒有開任何視窗。

## 原因分析

### 1. 長提示被裁切
- 已確認（讀紀錄）：原始觀測在 `驗證紀錄/current/native-room-selection/HUD-layout-followup.json`——192 DPI 下，一段很長的提示在 HUD 換行後蓋到下方的操作提示。之後狀態區已改成唯讀、可捲動的多行文字框（`NATIVE-WORLD-INTEGRATION:159`），文字不再畫到框外。
- 已確認（讀程式）：改成文字框之後還留著三個問題。
  - 每次換文字都用 `SetWindowTextW`，文字框會回到第一行。世界畫面的狀態文字隨狀態、冷卻時間一直在變，所以換行後多出來的最後幾行（操作提示、入口提示）預設看不到，往下捲也會馬上被拉回去。
  - 狀態區只有選了傳送技能（5、69）才會上移讓出選項列。選召喚（51）或要輸入文字的技能（113、116、118）時，選項列一樣出現在同一位置，狀態區卻不動，兩者上下重疊約 24 px。
  - 技能書只在技能 5 時上移，狀態區卻在 5 和 69 都上移；選 69 且技能書開著時，技能清單底部和狀態區頂部重疊 35 px。

### 2. 名稱數量重複顯示
- 已確認（讀程式）：清單文字是「名稱 × 數量」，名稱直接用伺服器字串。伺服器的名稱在數量大於 1 時結尾已經帶 ` (數量)`，所以會出現「治癒藥水 (5) × 5」。
- 伺服器的名稱格式引用自 lito-launcher-server `origin/main`（`291474f2`）`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/RuntimeInventory.java` 的 `nativeViewName()`：`名稱[ ($9)][ (充能次數)][ (數量)]`，數量只在大於 1 時附加。這是 `src/main`，不等於正式 JAR E149（見 `docs/REUSE-MAP.md` §0.1 第 3 點），E149 是否相同屬於推測；所以程式只在「結尾數字和權威數量完全相同」時才去掉，格式不同時照原樣顯示。
- `docs/GUI-ACCEPTANCE.md` 的 L10 把這一項寫成「來源伺服器清單」。來源清單是直接顯示伺服器給的名稱，沒有另外加數量，不會重複；這個缺陷實際在物品清單（`NATIVE-INVENTORY:44`）。本張沒有改 GUI-ACCEPTANCE，見「給下一棒的提示」。

### 3. 失焦後持續發招
- 已確認（讀程式）：按住 Space 的連打與按住方向鍵的移動，是由 20 ms 計時器持續推進，直到收到放開按鍵。原本在 `WM_ACTIVATEAPP`（切到別的程式）、`WM_KILLFOCUS`、`WM_CANCELMODE` 會清掉按住狀態；重新取得焦點後，鍵盤自動重複的按下訊息不會重新啟動（要求是第一次按下）。
- 找到的缺口：拖曳標題列、拉邊框、開系統選單（例如 Alt+Space）時，Windows 會進入自己的訊息迴圈。這段期間計時器照常觸發，但放開按鍵的訊息不會經過本程式的處理，焦點也沒有離開，所以連打和移動會一直繼續。最小化、視窗被模態對話框停用也沒有明確處理。
- 另一個缺口：這些訊息若在介面物件正被使用時送達（例如動作中跳出確認對話框），原本會被直接略過。

## 做了什麼

- `LiTo381-Modern/src/modern_lito/native_hud_policy.rs`（新增，純邏輯，不呼叫 Win32）
  - `focus_release`：哪些視窗訊息要放開按住的輸入。除了原有三種，新增 `WM_ACTIVATE`（變成非作用中，或在最小化狀態被啟用）、`WM_ENTERSIZEMOVE`、`WM_ENTERMENULOOP`、`WM_SIZE`（最小化）、`WM_ENABLE`（被停用）。
  - `strip` 常數與 `world_status`、`option_row`、`option_lift`：世界畫面下方各列的位置。三種選項列（傳送、召喚、技能文字）任一出現，狀態區和技能書一起上移 40。
  - `status_first_line`：狀態區換文字或改變大小之後，第一個可見行是哪一行。原本就在最後（或全部看得到）就跟到最後一行；使用者往回捲過就留在那一行。
  - 7 個單元測試。
- `native_login_window.rs`
  - `text()` 對狀態區：換文字前記下捲動位置，換完依規則捲回去。`layout()` 結尾做同樣的事，視窗變窄造成重新換行時也會跟到最後一行。
  - 狀態區、選項列位置改用上面的規則。
  - 放開輸入改成一個 `release_input`，由 `focus_release` 決定何時呼叫；介面物件忙碌時先記下來，下一個訊息補做。
  - 附近物品清單改用 `display_name`。
- `native_skill_window.rs`：技能書上移量改用同一個規則；快捷列位置改用同一組常數（數值不變）。
- `native_inventory.rs`：新增 `display_name(name, count)`；`Item::label_with_name` 改用它。模型裡保存的伺服器名稱不變，`native_inventory_names::Label.name` 也仍是完整名稱。3 個單元測試。
- `tests/native_inventory_names.rs`：預期文字由 `Sword (2) × 2` 改為 `Sword × 2`，並補一條「數字和數量不同時保留」。
- `docs/NATIVE-INVENTORY.zh-TW.md`：第 44 行改成現在的行為。

`docs/REUSE-MAP.md` 已查：版面與焦點是本 repo 自己的 Win32 視窗，表內沒有對應列；物品名稱格式引用了上面那一個 lito-launcher-server 檔案（唯讀，沒有改對方 repo）。沒有引用 tw-bin-engine。

## 已驗證

- `cargo check --locked --offline -j1 --all-targets --target x86_64-pc-windows-msvc --target-dir LiTo381-Modern/target/current`：結果見文末「本機編譯結果」。
- 新增與修改的檔案沒有新的編譯警告（同一段）。

## 推測

- 正式 JAR E149 的物品名稱也是「數量大於 1 時結尾附 ` (數量)`」。依據是 `NATIVE-INVENTORY:44` 的舊描述與 lito-launcher-server `src/main`；沒有用 `javap` 對 E149 核對。格式不同時，新程式不會去掉任何字，只是回到修改前的顯示。
- 多行文字框在 `SetWindowTextW` 之後會回到第一行、`EM_LINESCROLL` 可以在重畫前把它捲回去，所以畫面不會閃到第一行再跳走。這是依 Windows 文件的行為寫的，沒有實機看過。
- 可見行數用「文字框的排版區高度 ÷ 字型高度」算。若實際行距比字型高度大，算出來的行數可能多一行，最後一行會被切到一半。

## 待驗證

1. **CI**：本 PR 的 `cargo test --lib`，包含 `native_hud_policy` 7 個與 `native_inventory::display_name_tests` 3 個新測試。本機沒有執行任何測試。
2. `tests/native_inventory_names.rs` 是整合測試，CI 只編譯不執行；要在本機測試套件（`run-local-test-suite.ps1`）裡跑。
3. 實機（`docs/GUI-ACCEPTANCE.md`）：
   - R24、W06：世界畫面的長提示換行後，預設看得到最後一行；往回捲之後不會被拉回去；文字變短後不留空白。100%、150%、200% 縮放都要看。
   - 選技能 51、69、113／116／118，技能書開與關：狀態區、選項列、技能清單、快捷列互不重疊。
   - 背包、歸還清單、同伴物品、附近物品：數量大於 1 的物品只顯示一次數量；有充能次數的魔杖仍看得到次數。
   - B07、F02、O13：按住 Space 或方向鍵時 Alt+Tab、點別的視窗、最小化、**拖曳標題列、拉邊框、按 Alt+Space 開系統選單**，攻擊與移動立刻停止；回來後不會自己繼續。
4. 既有的 GUI 自動化腳本若會在送鍵途中移動或縮放客戶端視窗，按住狀態現在會被清掉，需要重新按下。沒有逐支檢查腳本。

## 沒有做的事

- 沒有改視窗最小尺寸。大廳／房間畫面的控件用固定座標排到 714（96 DPI），`WM_GETMINMAXINFO` 的最小高度 800 因此不能直接調小；工作區比它矮時下半部仍會在螢幕外（#301 報告已提到）。要解決需要讓這些畫面可以捲動或重新排版，超出本張「已知缺陷」的範圍。
- 狀態區上移 40 時會蓋住世界畫面保留區最下面 40 px（技能 5 原本就是這樣，現在 51、69、113／116／118 也一樣）。沒有改世界畫面的保留高度，因為那會讓畫面在選技能時縮放。
- 沒有改 `docs/GUI-ACCEPTANCE.md`、`TASKS.md`、`docs/REPORT.md`。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_hud_policy.rs`（新增）
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`、`native_skill_window.rs`、`native_inventory.rs`、`mod.rs`
- `LiTo381-Modern/tests/native_inventory_names.rs`
- `docs/NATIVE-INVENTORY.zh-TW.md`
- `docs/reports/issue-302.md`

## 給下一棒的提示

- `docs/GUI-ACCEPTANCE.md` L10 的「名稱含 `(數量)` 時數量不重複顯示」應該移到物品清單的項目（S11、M05 一帶）；來源伺服器清單沒有這個問題。請雲端對話在下次改 GUI-ACCEPTANCE 時順手更正。
- 要調整世界畫面下方各列的位置，改 `native_hud_policy::strip` 的常數，`world_rows_never_overlap_with_or_without_the_option_row` 會檢查有沒有重疊。背包開啟時的各列（-326、-286、-240、-215…）還是寫在 `native_login_window.rs` 的 `layout()` 裡，沒有搬進去。
- 新增任何「會讓視窗收不到放開按鍵」的情況，加在 `focus_release`，不要在 `wndproc` 另外寫清除。
- 若實機發現最後一行被切一半，改 `status_lines()` 的可見行數算法（目前用 `tmHeight`，可改成加上 `tmExternalLeading`）。

## 本機編譯結果

- 指令：`cargo check --locked --offline -j1 --all-targets --manifest-path LiTo381-Modern\Cargo.toml --target-dir LiTo381-Modern\target\current --target x86_64-pc-windows-msvc`
- 結果：`Finished `dev` profile`，結束碼 0（lib、bin、全部整合測試與 lib 測試目標都編譯通過；只編譯，沒有執行）。
- 輸出中沒有任何一行指向 `native_hud_policy.rs`、`native_inventory.rs`、`native_login_window.rs`、`native_skill_window.rs`、`tests/native_inventory_names.rs`，也就是這些檔案沒有警告或錯誤。
