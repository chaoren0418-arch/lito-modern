# Issue #329 — T049（M3 2／2）：背包與介面文字

範圍：數字通知的 Text 資源映射、LTstring 原版 UI、S0F／S10 系統訊息 GUI。
分支 `task/329`。本機只做靜態分析與 `cargo check`，**沒有在本機執行任何測試或編譯出來的程式**；測試結果以 PR 的 CI 為準。

## 做了什麼

### 1. 靜態逆向（recovered PE，SHA256 `4271D92F…5C2AB6E9`，Capstone 解碼，未執行）

| 位址 | 結論 |
|---|---|
| 4B2A0C／4B3156 | 封包分派：兩張 byte 索引表（4B3470 遊戲中、4B36B4 其他狀態）加跳躍表。0x0F→4A4190、0x10→4A4670、0x4B→48F600 |
| 48F600 | S4B 通知（`h`）：訊息框顯示 `string-c[id]`；273–275 用 `sprintf` 帶入一列網址、307／437 帶入剩餘小時與分、438 與 317 直接顯示 |
| 4A4190、4A4140、417560／417860 | S0F（`cs`）：依 kind 選調色盤顏色，文字在換行處切行；kind 9 先把 `\f>` 改成 `\f1` |
| 4A4670、4A44D0、表 4A4BAC | S10（`hc`＋字串）：代碼小於 string-c 列數用 4A44D0 展開；`代碼−2000` 落在 LTstring 用 `sprintf`，引數形狀依列而定 |
| 4A76D0、4A7850、4A8BA0 | S_C8、S_C9、S_CE 的拒絕結果對應哪一列 LTstring |

規則全文與 LTstring 各列在原版的引用位置寫在 `docs/NATIVE-MESSAGES.zh-TW.md`。

### 2. 程式

- 新增 `src/modern_lito/native_messages.rs`：
  - `Table`／`Strings`：`string-c.tbl`、`LTstring-c.tbl` 的解析（容許 Tab、列數不足）。
  - `fill`（4A44D0）、`sprintf`（字表用到的 `%s`、`%d`、旗標、寬度、`%%`）、`code_text`（S10）、`notice_text`（S4B）、`system_color`／`split_lines`／`sanitize_kind9`（S0F）。
  - `Log`（最近 8 則 S0F／S10）、`lines`／`hud_block`（轉成畫面文字，最多 3 行）。
  - `character_refusal_row`、`channel_refusal_row`、`room_refusal_row`、`lito_line`。
- `native_inventory_names.rs`：`Catalog::load` 同一個背景執行緒多載入兩張字串表（`strings`、`strings_error`、`string_sources`）；失敗不影響物品名稱。新增 `expand_argument`。
- `native_world.rs`：`World::message_log`，S0F 與 S10 收到時各記一筆（`#[serde(skip)]`，不改變既有序列化輸出）。
- `native_login_window.rs`：
  - 數字通知顯示 string-c 文字。
  - 世界 HUD 狀態區最後附上最近 3 行 S0F／S10 訊息。
  - 選角、進頻道、進房被拒時顯示 LTstring 原文；字表未載入或沒有對應列時維持原本的文字。
- `descriptions.rs`：`decode_cp950` 改成 `pub(crate)`（只改可見性）。
- 文件：新增 `docs/NATIVE-MESSAGES.zh-TW.md`；更新 `NATIVE-GAME`（第 21、40 行）、`NATIVE-LOBBY`（第 46 行）、`NATIVE-WORLD-INTEGRATION`（第 84 行）、`GUI-ACCEPTANCE`（L12、W05 與前置表）。都在原行修改，行號沒有移動。

### 3. 行為改變（審查請留意）

- 通知只有編號時的文字從「伺服器通知 #n（文字資源對應尚未完成）」改成 string-c 原文；查不到時是「伺服器通知 #n（Text 資源沒有可顯示的內容）」或「…（文字表尚未載入）」。
- 進頻道結果 1／10、進房非零結果、選角結果 7／8 的提示文字改成 LTstring 原文（例如進房 11 從「房間密碼不正確，請重試。」變成「密碼錯誤。」）。
- HUD 狀態文字多了最多 3 行訊息。進世界前收到的 S0F（例如房間規則）也在記錄裡，進世界後會出現在這 3 行。

## 已驗證

- 上表各位址的指令、跳躍表與常數：直接解碼 PE bytes（工具 `scratch/t329/ltdis.py`，不入 git）。
- 字表實況：`string-c.tbl` 檔頭 3484、實際 3483 列、6 列含 Tab、420 列含 `%`、253 個 `\f`、沒有 `$`；`LTstring-c.tbl` 296 列，與 `lito脫殼資源` 既有解碼檔 SHA256 相同。
- lito-launcher-server `origin/main` 的 `ClientSession.java` 只用 `systemMessage(9, …)` 與 `messageCode(代碼, 字串…)`，沒有 2000 以上的代碼（文字搜尋）。
- `cargo check --locked --offline -j1 --all-targets --target-dir target/current`：通過（exit 0，3 分 12 秒）。這只代表可編譯。

## 推測

- 區域 byte `64B6DC` 在 CP950 系統是 3（480AED 的呼叫沒有確認就是取 code page）；273–275 因此用第 431 列。本機字表的 245、247、431 三列文字相同，所以 `dword_64B6E0` 小於 0x340 時結果不受這個推測影響。
- `dword_64B6E0` 小於 0x340：它在 4A34E0 由伺服器版本封包的兩個 byte 組成，E149 實際送的值沒有核對。若不小於 0x340，原版的 273–275、438 會改用香港的列（600、599）。
- 834、875、317、153／286／288 的附帶行為用途（倉庫密碼、完整性檢查、等待、禁言計時）是依字表內容與呼叫形狀判斷的。
- `\f1` 在聊天區的實際顏色：調色盤第 1 色是 0x001F，沒有確認聊天區寫入函式是否對它另外處理。目前 HUD 是單色，不受影響。
- E149 的 S_CE／S_C9／S_C8 結果碼與原版客戶端解讀一致：依「伺服器是對這個客戶端寫的」推定，沒有用 `javap` 對 E149 逐一核對。

## 待驗證

- 實際字表在 Windows `MultiByteToWideChar(950, MB_ERR_INVALID_CHARS)` 下是否每列都解得出來（Python 的 cp950 全部解得出來；解不出的列會留空並計入 `undecodable`）。
- 實機（需開遊戲，交 Codex／使用者）：
  - L12：通知 438 是否顯示整段中文，狀態區是否放得下。
  - W05：寶石不足（S0F）、施法失敗（S10 280 等）是否出現在 HUD 最後幾行。
  - 進房密碼錯誤、等級不符進頻道時是否顯示 LTstring 原文。
- 字表在通知出現前是否已載入完成（未載入時會先顯示編號，下一次畫面更新才換成文字）。

## 未做

- HUD 訊息的顏色（`Line::color` 與 `\fX` 分段已算好，狀態區是單色 EDIT，沒有畫）。
- 原版大廳／房間的版面與其餘 LTstring 文字（進房說明第 26 列、倒數第 76 列、室長變更第 75 列、勝敗第 78–81 列、開房對話框等）。引用位置已列在 `docs/NATIVE-MESSAGES.zh-TW.md`。
- 307／437 通知（需要剩餘時間）、LTstring 數字引數的 5 列（世界封包讀取器沒有保留 S10 的原始位元組）。
- S10 代碼 73 的一次性略過、81 的區域條件、153／286／288 的計時。
- 大廳／房間狀態下原版把 S0F／S10 寫到另一個視窗（458BB0）；這裡只接了世界 HUD，房間內仍沿用既有的 `room_system_message`。

## 查過的資源

- `D:\lineage開發\0-客戶端\Text.idx`／`Text.pak`：`string-c.tbl`（PakViewer `-cli export`，輸出到 `scratch/t329/`，105931 bytes，SHA256 `4EBBC1A7A002A4C1DCDFA73DBF056B0E00648E0789763410C40E5079A22D43E9`）。
- `LiTo/LTtext.idx`／`LTtext.pak`（353 個檔）：PakViewer `-cli list … string` 得到 `LTstring-c.tbl`、`-h`、`-j`、`-k` 四個；匯出 `LTstring-c.tbl`（4911 bytes，SHA256 `C0A8FFC73174B61135A8F1F33A46960DEE3A3FE9831D4D917B82648092DB535C`）。
- 本張沒有宣稱缺資源。

## 參考來源（REUSE-MAP）

- lito-launcher-server `origin/main`：`docs/reference/client-protocol-needs.md`（S10 → `sub_4A4670`）、`Protocol.java`（`notice`、`systemMessage`、`messageCode` 的格式）、`ClientSession.java`（實際送出的 kind 與代碼）；recovered PE `lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`。都是唯讀，用 `git show` 讀取。
- tw-bin-engine：本張沒有用到。
- 沒有複製任何資源或程式碼。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_messages.rs`（新）
- `LiTo381-Modern/src/modern_lito/{mod,descriptions,native_inventory_names,native_world,native_login_window}.rs`
- `docs/NATIVE-MESSAGES.zh-TW.md`（新）、`docs/NATIVE-GAME.zh-TW.md`、`docs/NATIVE-LOBBY.zh-TW.md`、`docs/NATIVE-WORLD-INTEGRATION.zh-TW.md`、`docs/GUI-ACCEPTANCE.md`
- `docs/reports/issue-329.md`

## 給下一棒的提示

- 要畫顏色時：`native_messages::lines` 回傳每行的底色（RGB555），行內 `\fX` 用 `native_text_format::runs(&line.text, line.color)` 拆段。
- 要接其餘 LTstring 文字時，先查 `docs/NATIVE-MESSAGES.zh-TW.md` 的引用位置表；`sprintf` 已支援字表用到的 `%04d`、`%-25s`、`%+d`、`%%`。
- 要支援 LTstring 數字引數的列，需要 `native_world.rs` 的 0x10 分支另外保留 count 之後的原始位元組。
- 重新產生 LTstring 引用清單的方法：在 PE 映像裡找 `0x834B94` 的每個引用，往後解 6 道指令，取第一個 `[reg + disp]`，`disp / 4` 就是列號。
- 實機驗收用 `docs/GUI-ACCEPTANCE.md` 的 L12、W05。

## 建置結果

本機 `cargo check --locked --offline -j1 --all-targets --target-dir target/current`：通過（exit 0，3 分 12 秒）。本機未執行測試。

PR #370 第一個 commit 的 CI（run 43269639808）：`cargo check --all-targets` 與 `cargo test --lib` 通過，2398 passed／0 failed／597 ignored；本張新增的 8 個單元測試（含用到 Windows CP950 轉換的 `tables_are_parsed_leniently`）都在通過清單內。之後只改了本報告這一段與「待驗證」第一項。

## 與 main 同步（2026-11-13）

PR #370 與 main 衝突（衝突時 GitHub 不跑 CI，所以 commit `001e6be` 沒有 CI 紀錄），已 `git merge origin/main` 並解決兩個檔案：

- `native_login_window.rs`：HUD 狀態文字那一行，保留 main 的「找不到音效輸出裝置」提示，後面再接本張的訊息區塊。
- `docs/GUI-ACCEPTANCE.md`：前置表採用 main 的列（main 已移除 §17 那列、新增 Q04／Q09），L12／W05 那列用本張的寫法。

main 新增的 Q04／Q09 列提到「`$數字` 台詞未轉文字，字串表屬 T049」。那是聊天台詞（#342）的顯示，本張沒有處理；`native_messages::Strings` 與 `Catalog::expand_argument` 可以直接拿去用。

合併後本機 `cargo check --locked --offline -j1 --all-targets --target-dir target/current`：通過（exit 0，2 分 39 秒）。本機未執行測試；合併 commit 的 CI 見 PR。