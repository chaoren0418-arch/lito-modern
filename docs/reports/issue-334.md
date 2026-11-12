# Issue #334 — T056-1：徽章位置、全體名稱權限、名字後綴圖（G07）

執行者：claude-local（本機）。分支 `task/334`，PR #362。
本張範圍是 T056 的 1／2：emblem 請求、快取與繪製；S2F 稱號、全體名稱權限、特殊 NPC 徽記。FA 限制與 S69 陣營顏色分支屬第 2 張，本張沒有做。

**本機沒有執行任何編譯產物，也沒有開遊戲。** 下面「已驗證」分兩種：靜態讀原版程式得到的事實，以及 CI 跑過的單元測試。畫面上實際長什麼樣子全部在「待驗證」。

## 結論

- emblem 的請求與快取在 main 已經做完，而且和原版一致（快取 512 筆、未回覆 32 筆、同 ID 不重送），本張沒有改。缺的是**繪製位置**：main 是「自己和指向者的名字左邊 18 px」的現代配置，本張改成原版的兩條規則。
- S2F 稱號的收包在 main 已經做完。本張只補一個原版有、main 沒有的顯示分支（地圖 500／560 指向自己時也畫稱號）。
- 全體名稱：原版的權限與開關條件、逐角色的排版都已實作並有單元測試，但**開關沒有任何輸入來源**，所以玩家現在還叫不出這個模式。
- 特殊 NPC 徽記：原版是名字後面接一張 13×12 的小圖（`773.img`）。條件與位置已實作，圖從 BASE `Sprite.pak` 讀。
- 有一個行為改變要注意：**自己的徽章不再固定顯示在自己名字旁**，只有滑鼠指向自己、或盟戰背景啟用時才看得到。這是原版行為。

## 做了什麼

### 1. 徽章繪製位置（`native_emblems.rs`）

原版有兩個地方畫 16×12 徽章，本張都照做：

| 原版位置 | 條件 | 位置 |
|---|---|---|
| 489D15（名字旁） | 只有滑鼠指向的角色（75A98C）；emblem ID 非零；隱身且沒有偵測時不畫 | 名字列左 20 px、上 1 px |
| 485F30（頭頂） | 盟戰旗標 834F14 啟用；actor+0x20 非零；不是指向者；不是隱身；emblem ID 非零；y > 0 | 角色位置 +17／−52 |

- 名字旁：有 emblem ID 時名字本身右移 10 px（main 已有），徽章 x 是「未右移的置中位置 −10」，等於名字 x −20；y 是名稱矩形頂 −4，名字是頂 −3。
- 頭頂：`World.relations.active` 對應 834F14，`relations.side_20` 對應 actor+0x20，這兩個狀態在 main 已經由 S36／S0B／S69 維護。錨點沿用血盟標記（`native_clan_markers`）的做法，用已插值的角色位置，不另加原版移動中的子格表。
- `draw` 多兩個參數：指向者 ID 與角色位置表。收據的 `placement` 改為 `native_hover_name_left_20_up_1`，每列多 `placement`（`hover_name`／`war_head`）。

### 2. 全體名稱 pass（`native_world_labels.rs`）

原版 489F30 的入口條件是兩個 byte 都非零：

- **權限 64BC13**：自身角色建構（4E7AB5）時，S0B status bit 0x80 成立就寫 2，否則寫 0。另有一條寫 1 的分支，來源是客戶端自己的受保護變數，沒有封包會設它，本張不重建。`name_privilege()` 回傳這個值。
- **開關 64BC14**：預設 0，原版由指令字 `c` 切換（452D87，和 `forcewin`／`prepare`／`load`／`item`／`kick`／`teleport` 在同一個指令表）。對應 `Policy.all_names` 與 `Model::set_all_names`。

啟用時原版跳過整段指向名稱，改為對名單上每個非自己的角色做下面的事（`all_name_lines`）：

1. 名稱矩形沿用 4E7200（和指向名稱同一套：依稱號決定 14／27 高、有 emblem 加寬 18）。
2. 有主人（actor+0x64）時先畫**主人列**：主人字串取 `(` 之前的部分（46EA30），後面接稱號的最後 2 個 CP950 byte（48A00C）。然後 y +13。
3. **名字只畫給 1F 旗標成立、或有主人的角色**（48A186）。也就是一般玩家在這個模式下不顯示名字。
4. 1F 不成立、有主人、圖形不是 0／1／61／48／138／37／734／1186 時，名字後面接後綴圖。

新增 `layout_with(…, &Policy)`；原本的 `layout` 保留，等於預設政策。`Line` 多一個 `badge` 欄位（要量寬度的完整名字）。

### 3. 名字後綴圖（新檔 `native_name_badge.rs`）

- 原版兩處（48A1C8、48A6F6）都取 IMG 表 834BD4 的第 773 格，用色鍵複製（4BA220）畫在「名字 x ＋ 432330(名字) ＋ 7」，y 與名字同列。
- 指向時的條件（48A6F6）：1F 不成立、稱號非空、圖形不是上面 8 個再加 2786／2796。
- `773.img` 在 BASE `Sprite.pak`，320 bytes（8-byte header＋13×12 RGB555，色鍵 0），SHA256 `A3C6D21A…8B88185F`。
- 432330 在 CP950 且字型物件存在時用實際字型量寬度，所以這裡用 `native_world_text` 的 GDI 寬度（`advance`），不是「byte 數 ×6」。
- 第一次需要時才開檔；載入失敗只記在收據，不影響名字。

### 4. 指向稱號（S2F）

原版 48A309 在地圖 500／560 允許指向自己；這時稱號也會畫，x 用自身名稱的錨點（48A36C）。main 原本這個情況不畫稱號，已補上。其餘 S2F 行為沒有改。

### 5. 接線與既有驗證腳本

- `native_world_scene.rs`：計算指向者與（全體名稱時）所有角色的 1F、呼叫 `layout_with`、把位置表提前給徽章用、畫後綴圖；收據多 `name_badges`，`world_labels` 多 `all_names_switch`／`name_privilege`／`flag_1f_unresolved`。全體名稱開關在 Model 因世界換代重載時保留（原版是行程全域變數）。
- `native_emblem_probe.rs` 與 `維護工具/verify-native-emblems.ps1`：這組隔離 TCP 驗證原本預期「自己＋bot 各一張」。依原版規則只有指向者（bot）會畫，六步的預期張數由 1／2／2／1／1／0 改成 1／1／1／0／0／0，不透明像素總數由 1176 改成 504。**這組腳本本張沒有重跑**（見待驗證）。

## 怎麼驗證

1. **靜態反組譯**：用 Capstone 讀 `lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`（SHA256 `4271D92F…5C2AB6E9`，與 `驗證紀錄/current/native-world-labels/native-label-raw.json` 記錄的是同一個檔）。工具放在 `維護工具/t056-native-names/dis.py`，只讀檔。
2. **資源查詢**：`維護工具/t056-native-names/find_img.py` 唯讀掃過 `LiTo/LTsurf.idx`、`LiTo/LTicon.idx`、`LiTo/LTsprite.idx`、`Sprite.idx`、`Sprite00..15.idx`，找 `773.img` 與語系變體 `773c／h／j／e.img`。只有 `Sprite.idx` 有 `773.img`，其餘都沒有。沒有用到 PakViewer，因為這些 idx 是明文的 28-byte 記錄；本張也沒有宣稱缺任何資源。
3. **單元測試（交 CI）**：新增 7 案 lib 測試，全部是純記憶體加既有 fixture `tests/fixtures/native-world-current.json`：
   - `native_world_labels::tests`（4 案）：權限與開關的組合；主人列文字（含 CP950 雙位元組結尾）；全體名稱只畫 1F／有主人的角色、座標、後綴條件；指向後綴的三個否決條件。
   - `native_emblems::tests`（1 案）：沒指向不畫、指向者位置與像素、盟戰時頭頂位置與指向者排除、頂邊略過、未快取 ID、取消。
   - `native_name_badge::tests`（2 案）：13×12 形狀與色鍵、位置公式。
   - `tests/native_emblems.rs` 的既有整合測試同步改了呼叫方式（CI 只編譯它，不執行）。

## 已驗證

**CI（PR #362）**：結果見本檔最後一節「CI 結果」。

**靜態讀碼（原版位址）**：

| 項目 | 位址 | 內容 |
|---|---|---|
| 全體名稱入口 | 489F30–489F44 | `byte_64BC13` 與 `byte_64BC14` 都非零才進入，否則跳到指向區段 48A2A1 |
| 權限寫入 | 4E7AB5–4E7AF3 | 自身 S0B status `&0x80` → 2；受保護變數 ==1 → 1；否則 0 |
| 開關切換 | 452D72–452D92 | 指令字與 `"c"`（5025A8）相同時把 64BC14 反相 |
| 逐角色排版 | 489F60–48A281 | 略過自己；矩形 4E7200；主人列 46EA30＋稱號尾 2 byte；名字條件 1F 或有主人；後綴圖 |
| 指向徽章 | 489D15–489E0E | 指向者、emblem 非零；x＝置中 −10（自己用自身錨點 −0x22）；y＝矩形頂 −4（有稱號且開關時 +13） |
| 盟戰頭頂徽章 | 485F30–486090 | 由 489D03 在 834F14 非零時呼叫；+0x20 非零、非隱身、y>0、非指向者、emblem 非零；位置 +0x11／−0x34 |
| 指向後綴圖 | 48A6F6–48A7C6 | 1F==0、稱號長度非零、圖形不在 10 個值內；`[834BD4]+0xC14`；x＝名字 x＋432330＋7 |
| 文字寬度 | 432330 | codepage 950 取字型物件 647D08，有物件時量實際寬度，否則長度 ×6 |
| emblem 快取 | 424130–4242A5 | 已排序表上限 0x200、等待清單上限 0x20、同 ID 不重複請求（與 main 現行實作一致） |

**資源**：`Sprite.pak` 的 `773.img`：offset 831742805、320 bytes、13×12、flag 1、色鍵 0。

## 推測

- **status bit 0x80 是觀戰／幽靈狀態**：依 L1J 系伺服器的慣例（STATUS_GHOST＝128）推測。LiTo E149 什麼情況下會對自己送出這個 bit，本張沒有用 `javap` 核對。
- **`773.img` 的用途**：圖是一個 13×12 的小圖示，原版把它接在「有稱號的非玩家外觀角色」與「別人的寵物／召喚物」名字後面。任務說的「特殊 NPC 徽記」我判定就是這張圖，依據是 NATIVE-WORLD-LABELS 第 13 行把它和 emblem、全體名稱並列，而同一個函式裡只有這一個圖像呼叫符合。
- **主人列接「稱號最後 2 byte」**：反組譯確實是這樣寫的，但用意不明（可能是所有格助詞）。稱號少於 2 byte 時原版會讀到緩衝區前面的記憶體，這裡改成不接尾巴。
- **字型物件 647D08 在 LiTo 是存在的**：依 NATIVE-WORLD-LABELS 記錄的「CP950 走 GDI」推測。若它不存在，後綴圖的 x 應改用 byte 數 ×6。
- **1F 旗標查不到圖形類別時當成成立**：盟戰結束後 1F 的清除要查 SPR 類別；查不到時沿用最後一次權威狀態，並在收據記 `flag_1f_unresolved`。這是本張自訂的處理。

## 待驗證

需要實機或本機執行，交本機 Codex：

1. **畫面**（GUI-ACCEPTANCE W19）：指向有血盟徽章的玩家，徽章在名字左邊、和名字同一列；指向自己時自己的徽章出現；沒指向時自己的徽章不出現。
2. **後綴圖位置**：指向有稱號的 NPC，名字後 7 px 出現小圖；不同字型（新細明體以外）下位置是否仍貼著名字。
3. **`維護工具/verify-native-emblems.ps1`** 隔離 TCP 六步重跑，確認新的預期張數（1／1／1／0／0／0）與像素總數 504 成立。
4. **盟戰頭頂徽章**：需要 S36 宣戰背景加上兩個血盟的角色；目前沒有現成 fixture 走完整封包，只有單元測試直接設狀態。移動中的位置和原版差多少（原版另加子格表）沒有量過。
5. **全體名稱**：開關沒有輸入來源，無法從畫面驗。接線後要驗：status 0x80 的自己加開關時，寵物／召喚物與 1F 角色顯示名字，一般玩家不顯示。
6. `tests/native_emblems.rs`、`tests/native_world_labels.rs` 等整合測試在 CI 只編譯不執行；`native_world_labels` 第 3 案需要 `D:\lineage開發\0-客戶端`。

本張沒有做、也不算通過的項目：

- **全體名稱開關的輸入**：原版是指令字 `c`。GM 工具依 G 定義排除，要不要另外給一個入口（例如觀戰模式的按鍵）需要決定。
- **S_CASTLE_OWNER 皇冠**：讀碼時發現 485D20 會在指定物件 ID 的角色頭上畫 `crown.raw`（`Sprite.pak` 內，4608 bytes＝12 張 16×12，每 100 ms 換一張，位置 +17／−52）。物件 ID 由 4AEDD0 收包（`c` 槽位 0..8、`d` 物件 ID）寫入；伺服器 `Protocol.castleOwner`（S_CASTLE_OWNER＝0x47，LS `origin/main` `edfb5c82` 的 `Protocol.java:6196`）會產生這個封包。Rust 客戶端沒有任何對應程式。它不在 T056 列出的項目裡，本張沒有做。
- **4860A0**：名字等於某個全域字串（64ABCA）的角色頭上畫同一組皇冠影格，呼叫條件（48AB87 的模式值 4）沒有追。
- **稱號開關 75A840**：指向時是否畫稱號由這個值決定，設定它的呼叫端多半是把舊值傳回去，最初來源沒有追到。現行仍是「非空就畫」。
- **64BC16**：國別碼為 1 時稱號含非 ASCII 就不畫。LiTo（CP950）不會走到，未實作。
- 指向名稱的其他原版分支沒有動：actor+0x1A 的四個隱藏旗標、圖形 0xA9B 的固定顏色、4E89F0＝10 的國別分支。
- 全體名稱時原版沒有隱身檢查；這裡仍然不顯示隱身角色的名字（沿用現有的可見性政策）。
- 全體名稱時，若指向的角色名字沒被畫出來（非 1F、無主人），原版仍會畫它的徽章；這裡不畫。
- emblem 請求時機：原版是要畫的時候才請求；main 是所有看得到、有 emblem ID 的角色都先請求。沒有改。
- `LiTo/LTsurf` 沒有 `773*.img`（已掃過 idx）；若日後 LTsurf 加入同名圖，原版會優先用它，目前的載入不會發現。Sprite shard 有同名圖時會拒絕。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_emblems.rs`（繪製改寫、單元測試）
- `LiTo381-Modern/src/modern_lito/native_world_labels.rs`（`Policy`、`layout_with`、全體名稱、後綴條件、單元測試）
- `LiTo381-Modern/src/modern_lito/native_name_badge.rs`（新檔）
- `LiTo381-Modern/src/modern_lito/native_world_scene.rs`、`mod.rs`（接線）
- `LiTo381-Modern/src/modern_lito/native_emblem_probe.rs`、`維護工具/verify-native-emblems.ps1`（預期值）
- `LiTo381-Modern/tests/native_emblems.rs`（呼叫方式）
- `docs/NATIVE-EMBLEMS.zh-TW.md`、`docs/NATIVE-WORLD-LABELS.zh-TW.md`（現況段落）
- `維護工具/t056-native-names/dis.py`、`find_img.py`（唯讀靜態工具）
- `docs/reports/issue-334.md`（本檔）

REUSE-MAP：已查，表內沒有 T056 的列。TW（tw-bin-engine）沒有讀；LS 只讀了 `Protocol.java` 的 `castleOwner`／`emblem`／`title`／`pledgeMetadata`／`battleClanMetadata` 的位置（`origin/main` `edfb5c82`），沒有照抄任何格式——本張沒有新增或修改封包解析。

## 給下一棒的提示

- **T056 第 2 張（FA 限制、S69 陣營顏色）**：
  - 直接用 `python -I -B 維護工具\t056-native-names\dis.py <VA> <長度>` 讀原版；`dis.py xref <VA>` 找引用。
  - 名字顏色的入口在指向區段 48A624–48A6F1：先看 1F 與圖形 0xA9B，再呼叫 4E9220（兩次），地圖 500／560 另有分支；全體名稱與稱號一律用 actor+0x28。
  - 834F14 的引用點：4E76AF（S69 血盟 setter）、4E7DB4（S0B 建構）、4AD17B／4AD5BB（S36）、489D04（頭頂徽章）、4A7014、4B0B43、45C93E、474BB9、47E908、41AC62、401F62。後面幾個本張沒有讀，FA 輸入限制可能在其中。
  - 頭頂徽章已經吃 `relations.active`／`side_20`，第 2 張改這兩個狀態時要一併看 `native_emblems::tests`。
- **本機測試執行者**：先重跑 `verify-native-emblems.ps1`；若第 1、2 步卡住，檢查 probe 的指向者是不是 bot（`native_combat_probe.rs` 的 `room-world-emblems` 分支）。
- **皇冠（S_CASTLE_OWNER）** 要不要做請調度器判斷；要做的話資料都在上面「待驗證」第二個清單裡，資源在 `Sprite.pak` 的 `crown.raw`。
- TASKS.md 的 T056 狀態沒有改（兩張都完成後由審查者更新）。

## CI 結果

PR #362 第一個 commit（`cargo check --locked --all-targets`＋`cargo test --locked --lib --no-fail-fast`，run 43219377680）：通過。lib 測試 2343 過、0 敗、597 ignored；本張新增的 7 案都在執行紀錄裡且為 ok：

- `native_emblems::tests::native_hover_and_war_head_placement`
- `native_name_badge::tests::qualified_shape_is_13_by_12_with_zero_colour_key`
- `native_name_badge::tests::suffix_follows_the_measured_name_by_seven_pixels`
- `native_world_labels::tests::all_name_gate_needs_own_status_bit_and_switch`
- `native_world_labels::tests::all_name_pass_prints_only_flagged_and_owned_actors`
- `native_world_labels::tests::hover_suffix_needs_title_without_flag_or_class_body`
- `native_world_labels::tests::owner_line_keeps_text_before_parenthesis_and_title_tail`

之後的 commit 只加本報告（與同步 main）；最新 commit 的 CI 以 PR 頁面為準。
