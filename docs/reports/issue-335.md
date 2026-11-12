# Issue #335 — T056-2：FA 限制與 S69 戰爭狀態的陣營顏色分支（G07）

執行者：claude-local（本機）。分支 `task/335`，PR #363。
本張範圍是 T056 的 2／2：FA 限制、S69 戰爭狀態的陣營顏色分支。徽章、全體名稱、名字後綴圖屬第 1 張（#334），本張沒有動。

**本機沒有執行任何編譯產物，也沒有開遊戲、沒有跑 `cargo check`。** 下面「已驗證」分兩種：靜態讀原版程式得到的事實，以及 CI 跑過的單元測試。畫面上實際長什麼樣子全部在「待驗證」。

## 結論

- **S69 的戰爭狀態分支在 main 已經做完**（`Context::clan_changed`，2026-10-16 那一輪補的），本張逐行對過原版 4E7690，行為一致，沒有改；只補了 CI 會跑的單元測試。`NATIVE-LABEL-UPDATES` 第 19 行「未實作」的說法是更早的紀錄，已更新。
- **陣營顏色**：原版讀 1F 旗標決定顏色的地方只有一處——指向名稱（48A624）。main 缺的是其中「圖形 0xA9B 在 1F 未設時固定用 0x7C00」這個分支，已補上。其餘（陣營 byte 0／1／其他、地圖 500／560）main 原本就對。
- **FA 旗標（actor+13A）有第二個來源**：S0B 在 status bit 4 時尾端多一個 byte 和一個 dword，dword 非零就設 +13A。main 原本把這 5 個 byte 原樣保存但不解讀，而且每次 S0B 都把旗標清成 false。已接上。目前的伺服器（LS `edfb5c82` 的 `NativeActorTail`）固定送 `FF 00 00 00 00`，所以實際行為不變。
- **FA 限制**是五個聊天指令（`title`／`emblem`／`ban`／`surrender`／`cease`）先看自身的 +13A。規則已寫成 `native_fa_gate` 並有測試，但**現代客戶端沒有斜線指令輸入，這個規則目前沒有任何呼叫端**，玩家看不到任何差別。
- 敵盟（+20＝2）可以不按強制攻擊鍵就選為攻擊目標，這是盟戰玩法，沒有做。

## 做了什麼

### 1. 指向名稱顏色（`native_world_labels.rs`）

新增 `hover_color`，對應原版 48A624–48A6F1：

| 順序 | 條件 | 顏色 |
|---|---|---|
| 1 | 1F 未設，而且圖形是 0xA9B | 0x7C00（`[50344C]`） |
| 2 | 陣營 byte（4E9220 的回傳值）＝0，地圖不是 500／560 | 0x7C00 |
| 3 | 陣營 byte＝1，地圖不是 500／560 | 0x001F（`[503444]`） |
| 4 | 其他 | actor+0x28（正義值顏色） |

第 1 列是本張新增的；2–4 列是把 main 原有的判斷搬進同一個函式，結果不變。自身名稱（489EFD）不走這個函式，仍然只分「零＝0x7C00／非零＝0x001F」。

1F 旗標由 S0B（4E7DB3）、S36、S69（4E7690）在盟戰背景下設定，或由 status bit 4 設定；`native_world_scene.rs` 原本就會替指向的角色解析它，本張沒有改解析方式。

### 2. S0B 尾端的 +13A（`native_world.rs`）

原版 4A3A48：status `&4` 時先讀一個 `c`（0–14 保留，其他改成 0xFF），再讀一個 `d`；`d != 0` 當成建構子的最後一個參數，寫進 actor+13A（4E810C／4E8138／4E8143）。

`actor()` 現在在讀到 5 byte 尾端時，後 4 byte 不全為零就把 `clan_marker_active` 設成 true。`status_tail` 欄位照舊保存原始 5 byte。那個 `c` 的用途沒有追。

### 3. FA 指令限制（新檔 `native_fa_gate.rs`）

原版指令分派（403029 起）裡，下面五個分支會讀自身 actor+13A：

| 指令 | 位址 | 旗標成立 | 旗標不成立 |
|---|---|---|---|
| `title` | 404291 | 進 4044A0 | 印字串表第 253 筆 |
| `emblem` | 40402C | 進 404C80 | 印第 253 筆 |
| `ban` | 403F71 | 進 404520 | 印第 253 筆 |
| `surrender` | 403FE4 | 送出 | 直接返回，沒有任何輸出 |
| `cease` | 403FB4 | 送出 | 直接返回，沒有任何輸出 |

`war`（404015）不讀這個旗標。模組提供 `Command`、`gate(own_flag, command)`、`own_flag(world)`，以及每個指令的在地化別名在字串表 834BC4 的索引（364／379／384／381／382）。**沒有組任何送出封包，也沒有接任何輸入。**

### 4. S69 單元測試（`native_actor_relations.rs`）

行為沒有改，只加 `tests` 模組，直接對 `clan_changed` 驗 4E7690 的每個分支。原本只有 `tests/native_actor_relations.rs` 整合測試，CI 只編譯不執行。

### 5. 收據與工具

- `native_world_scene.rs`：`world_labels` 收據多 `hover_flag_1f`（指向者的 1F）與 `own_fa_flag`（自身 +13A）。
- `維護工具/t056-native-names/scan.py`：找出對某個結構偏移的 byte 存取（本張用來找 +0x20、+0x13A 的讀寫點）。`cmds.py`：把一段指令分派碼摘要成「字串／跳躍目標」。兩個都只讀原版 EXE。

## 怎麼驗證

1. **靜態反組譯**：Capstone 讀 `lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`（和 #334 同一個檔），用 `維護工具/t056-native-names/` 的 `dis.py`、`scan.py`、`cmds.py`。
2. **伺服器端對照**（唯讀，LS `origin/main` `edfb5c82`）：`NativeActorTail.java`（S0B 尾端 5 byte 的內容）；另外只用 `git grep` 看了 `Protocol.battleClanMetadata` 有哪些呼叫行，沒有讀呼叫端的條件。沒有抄任何格式。
3. **單元測試（交 CI）**：新增 4 案 lib 測試，純記憶體加既有 fixture `tests/fixtures/native-world-current.json`：
   - `native_world_labels::tests::hover_colour_follows_side_byte_war_flag_and_fixed_body`
   - `native_fa_gate::tests::refused_commands_split_into_notice_and_silent`
   - `native_fa_gate::tests::put_object_status_tail_dword_and_fa_drive_the_own_flag`
   - `native_actor_relations::tests::s69_clan_update_sets_side_and_flag_only_while_war_is_active`

沒有查任何 pak：本張沒有用到新資源，也沒有宣稱缺資源。REUSE-MAP 已查，沒有 T056 的列；TW（tw-bin-engine）沒有讀。

## 已驗證

**CI（PR #363）**：見本檔最後一節「CI 結果」。

**靜態讀碼（原版位址）**：

| 項目 | 位址 | 內容 |
|---|---|---|
| S69 handler | 4AFDC0 | 格式 `dssd`；依序呼叫血盟 setter 4E7690、稱號 setter 4E73A0，再寫 +0x48 |
| S69 血盟 setter | 4E7690–4E771C | 834F14 為零時只換字串。非零時：空血盟 → +20＝0；敵盟清單（834ED4..834ED8）有相符 → +20＝2、+1F＝1；之後再比己盟（834EB8），相符 → +20＝1、+1F＝1 |
| S0B 建構的盟戰分支 | 4E7DB3–4E7E32 | 同樣的敵盟／己盟判斷；己盟時若主人名等於自身名則不設 1F；最後 status `&4` 設 1F |
| 指向名稱顏色 | 48A624–48A6F1 | 見上面第 1 節的表 |
| 陣營 byte | 4E9220 | `[63D54C] >= 0x52E8` 時回傳 actor+0x128；否則改查兩張名單（64AB64、64AB54） |
| 自身名稱顏色 | 489EFD–489F2B | 4E9220 非零 → `[503444]`，否則 `[50344C]` |
| 顏色常數 | 503440–50344C | 0、0x1F、0x3E0、0x7C00（檔案內的初始值） |
| FA handler | 4AC1C0–4AC23F | 格式 `ddsd`；找得到角色時寫 +0x48、稱號、`+0x13A = (active != 0)` |
| S0B 尾端 | 4A3A48–4A3A84 | status `&4`：讀 `c`（0..14 以外改 0xFF）再讀 `d`；`d != 0` 是建構子 4E7A40 的最後一個參數 |
| +13A 的寫入點 | 4AC234、4DFEA4、4E810C、4E8138、4E8143 | FA、基底建構、角色建構的三個出口 |
| +13A 的讀取點 | 403F7A、403FB9、403FEA、404035、404297、486265 | 前五個是指令分派，最後一個是血盟標記繪製（main 已有） |
| 指令分派 | 403265–403B4C、403F71–4042B0 | 見上面第 3 節的表 |
| 834F14 的其餘引用 | 4A7014、4B0B43、45C93E、474BB9、41AC62 | 都是選背景音樂（盟戰中用 9 號）；47E908 是重置；401F62 在 `war` 指令裡查敵盟清單 |
| +20＝2 的讀取點 | 402496、402501、48BA80、48CE64、48CF0E、4C9566、4C9E9C、4CB4A3、4E8D86 | 都在目標選取的判斷裡，和 Ctrl 強制攻擊旗標（64B8F6、63D550）並列 |

**伺服器端**：`NativeActorTail.complete` 在 status `&4` 時附加 `FF 00 00 00 00`。

## 推測

- **`[63D54C] >= 0x52E8` 在 LiTo 成立**，所以陣營 byte 就是 S0B 的最後一個 byte。main 原本就是這樣假設的，本張沿用，沒有去追 63D54C 的寫入點。
- **圖形 0xA9B（2715）是什麼角色**不知道。只確定原版對它在 1F 未設時固定用 0x7C00。
- **+20＝2 的九個讀取點是「敵盟不用強制攻擊也能打」加上游標變化**：依周圍的條件（Ctrl 旗標、`[+0x4C] > 0`、呼叫 49A510）判斷，沒有逐一追到底。
- **指令字的比對是否分大小寫**沒有確認（比對函式是匯入表的函式，沒有查是哪一個）。`Command::from_word` 目前只接受全小寫。
- **第 253 筆字串的內容**沒有查，只記索引。
- **+13A 的意思是「血盟領袖」**：五個受限指令在一般天堂都是君主專用，依此推測。LiTo 的伺服器在什麼情況下、對哪個客戶端送哪個角色的 `active` 非零，沒有讀；所以自身的 +13A 實際上會不會被設起來也不知道。

## 待驗證

需要實機或本機執行，交本機 Codex：

1. **指向名稱顏色**：兩個陣營的玩家各指一次，確認紅（0x7C00）／藍（0x001F）；地圖 500／560 裡是正義值顏色。
2. **0xA9B**：找到用這個圖形的角色後，確認平時指向是 0x7C00，盟戰（或 status bit 4）時跟著陣營 byte。
3. **S69 在盟戰中的畫面**：需要 S36 宣戰背景加上 S69 改血盟的封包順序；目前只有單元測試直接呼叫 setter。1F 改變後水下光源分類與頭頂徽章（#334）是否跟著變，沒有從畫面看過。
4. `tests/native_actor_relations.rs`、`tests/native_label_updates.rs`、`tests/native_world.rs` 等整合測試在 CI 只編譯不執行。本張改了 S0B 的 +13A 來源，`native_label_updates` 裡「新的 S0B 清掉舊標記」那一案在尾端 dword 為零時結果應該不變，但沒有實跑。

本張沒有做、也不算通過的項目：

- **FA 指令限制沒有呼叫端**。現代客戶端的聊天視窗（`chat_window.rs`、`direct_chat_window.rs`）走的是現代協定，沒有原版的斜線指令。要不要做斜線指令、做哪幾個，需要另外決定；決定之後 `native_fa_gate::gate` 可以直接用。
- **五個指令本身**（改稱號、上傳徽章、驅逐、投降、停戰）沒有做，封包也沒有組。
- **S0B 更新既有角色時 +13A 保留**：原版 S0B 對「已存在、不是自己、4E6AF0 成立」的角色走 4E65E0 更新，不會改 +13A；這裡每次 S0B 都依尾端重設。目前伺服器尾端固定為零、FA 會另外送，差別只在 FA 之後又來一個 S0B 的情況。
- **敵盟目標選取與游標**（+20＝2）：盟戰玩法，T056 明列不在範圍。
- **盟戰背景音樂**（834F14 → 9 號）：TASKS.md 已歸在 backlog 的「盟戰玩法」。
- **4E9220 的舊版分支**（兩張名單 64AB64／64AB54）：只在 `[63D54C] < 0x52E8` 時使用，沒有重建。
- #334 列的其餘指向名稱分支（稱號開關 75A840、64BC16、actor+0x1A 的四個隱藏旗標、4E89F0＝10 的等級差顏色）仍然沒有動。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_world_labels.rs`（`hover_color`、單元測試）
- `LiTo381-Modern/src/modern_lito/native_world.rs`（S0B 尾端 → `clan_marker_active`）
- `LiTo381-Modern/src/modern_lito/native_fa_gate.rs`（新檔）
- `LiTo381-Modern/src/modern_lito/native_actor_relations.rs`（只加單元測試）
- `LiTo381-Modern/src/modern_lito/native_world_scene.rs`、`mod.rs`（收據兩欄、模組註冊）
- `docs/NATIVE-LABEL-UPDATES.zh-TW.md`（第 19 行改成現況）
- `維護工具/t056-native-names/scan.py`、`cmds.py`（唯讀靜態工具）
- `docs/reports/issue-335.md`（本檔）

## 給下一棒的提示

- **審查者**：T056 兩張都交了，TASKS.md 的 T056 狀態本張沒有改。FA 限制這一項是「規則有、入口沒有」，要算完成還是留一個後續任務，請裁定。
- **要做斜線指令的人**：分派表從 403029 開始，`python -I -B 維護工具\t056-native-names\cmds.py 403029 1400` 會列出全部指令字和分支。每個指令有 ASCII 字和字串表 834BC4 的在地化別名兩種寫法。
- **本機測試執行者**：收據 `world_labels.hover_flag_1f`、`world_labels.own_fa_flag` 可以直接看指向者的 1F 與自身的 +13A；指向名稱那一列的 `color` 是 RGB555。
- **找結構欄位的讀寫點**：`python -I -B 維護工具\t056-native-names\scan.py disp32 13a`（32 位元位移）或 `scan.py disp8 20`（8 位元位移的 byte 存取，結果要人工篩）。
- #334 提到的皇冠（S_CASTLE_OWNER、`crown.raw`）仍然沒有人做，也不在 T056 的列舉裡。

## CI 結果

PR #363 第一個 commit `d5d3783`（`cargo check --locked --all-targets`＋`cargo test --locked --lib --no-fail-fast`，run 43262014671）：通過。lib 測試 2353 過、0 敗、597 ignored；本張新增的 4 案都在執行紀錄裡且為 ok：

- `native_actor_relations::tests::s69_clan_update_sets_side_and_flag_only_while_war_is_active`
- `native_fa_gate::tests::put_object_status_tail_dword_and_fa_drive_the_own_flag`
- `native_fa_gate::tests::refused_commands_split_into_notice_and_silent`
- `native_world_labels::tests::hover_colour_follows_side_byte_war_flag_and_fixed_body`

之後的 commit 只加本報告；最新 commit 的 CI 以 PR 頁面為準。
