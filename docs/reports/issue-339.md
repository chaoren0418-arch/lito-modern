# Issue #339 — T058（M3 2/2）：附件、名稱與特效排序（G07.3）

執行者：claude-local。分支 `task/339`（基底 `origin/main` `e9d59ab`），第 1 次嘗試。
本張範圍只有「附件、名稱與特效排序」；地形、跨區塊、遮擋與屋頂是 #338（已合併）。

## 結論

1. **附件有兩個與原版不同的地方，已修正**。
   - 原版每個附件用自己 sprite 的屬性 104 選寫入器。現行只接受 0 與 8，其他值會讓整個 layer plan 失敗，連陰影一起不畫。本張補上 1（半透明）與 `0x80`（multiply）。
   - 原版畫陰影時不讀陰影 sprite 自己的屬性 104。現行卻要求它是 0，否則同樣整個 plan 失敗。本張拿掉這個檢查。
   - 共用 list.spr 有 3598 列帶陰影或附件，plan 接受的列從 3036 增加到 3225（多 189 列）。
2. **附件的狀態繼承與原版一致，沒有改行為**：只有「本體半透明」會傳給附件，中毒綠、變暗、本體目錄旗標 1 都不會。本張把這條規則從 `native_world_layers` 裡的一個 `if` 改成 `Mix::submitted`，並用測試對照原版 `4E7000` 的逐字模型。
3. **名稱不需要改**：原版三段名稱（自身、全體、指向）都畫在世界分片之後，而且不看本體的半透明狀態。#338 留下的問題「物件半透明時名稱是否仍顯示」答案是會顯示，現行行為相同。
4. **特效排序不需要改，但有一條界線要寫清楚**：原版同一格、同 depth 的兩個 actor 誰在上面取決於誰最後換影格，沒有固定答案。現行固定為特效在本體底下，本張用測試把這個策略固定下來，並在文件標明它不是原版時序。
5. **還有一個沒處理的缺口**：371 列的**本體**屬性 104 不是 0 或 1（多數是 8 或 128 的特效型物件）。這類物件如果以 S_PUT_OBJECT 出現，現行會用一般方式畫本體並回報 `layer_error`。這屬於本體透明，不在本張「附件、名稱與特效排序」的範圍，列在「給下一棒的提示」。
6. G07.3 仍是 partially_verified：本張新增的 6 個 lib 測試要等 PR 的 CI；實機項目（GUI 清單 W13、W14）沒有做。

## 做了什麼

### 程式

| 檔案 | 變更 |
|---|---|
| `LiTo381-Modern/src/modern_lito/native_actor_layers.rs` | `Mix` 增加 `Half`、`Multiply`；新增 `Mix::from_native_word`（0／1／8／`0x80`，其他值拒絕）、`native_word`、`submitted(actor_half)`。`Plan::from_catalog` 的附件改用 `from_native_word`，陰影不再讀自己的屬性 104。新增 3 個單元測試 |
| `LiTo381-Modern/src/modern_lito/native_world_layers.rs` | 附件的寫入器改由 `attachment.mix.submitted(style == Half)` 決定，對應到既有的 `Blend::Over`／`Actor(Half)`／`Screen`／`Multiply`。raster 沒有改 |
| `LiTo381-Modern/src/modern_lito/native_class1.rs`、`native_earth_jail.rs` | 這兩個特效沿用 actor 的 layer plan，`match` 補上新的兩個值：`Half` 對應特效的 `Shadow`（同一支半透明寫入器），`Multiply` 對應 `Multiply`。目前它們用的 1801／1805 附件字都是 8，行為不變 |
| `LiTo381-Modern/src/modern_lito/native_world_layer_order_tests.rs`（新增） | 3 個合成場景測試，見「待驗證」 |
| `LiTo381-Modern/src/modern_lito/native_world_occlusion_tests.rs` | 7 個測試輔助項目改成 `pub(super)`，讓新測試共用 #338 的合成場景。測試內容沒有改 |
| `LiTo381-Modern/src/modern_lito/mod.rs` | 註冊新的測試模組 |

### 文件

- `docs/NATIVE-ACTOR-LAYERS.zh-TW.md`：檔尾新增「附件與陰影的繪製字」；第 15 行原本寫「0x40／0x100 繼承仍未映射」，改成指向新段落（行數不變，`TASKS.md` 引用的 `NATIVE-ACTOR-LAYERS:11` 不受影響）。
- `docs/NATIVE-WORLD-LABELS.zh-TW.md`：檔尾新增「名稱與繪製狀態」（加在檔尾，`TASKS.md` 引用的 `:13`、`:15`、`:25` 不受影響）。
- `docs/NATIVE-WORLD-EFFECTS.zh-TW.md`：新增一段說明同 depth 排序的界線。
- `docs/reports/issue-339/`：`calls.py`（列出相對呼叫點）、`list_layers.py`（盤點 list.spr 的 101／104／105）、`list-spr-layers.json`（盤點結果）。

### 查過 `docs/REUSE-MAP.md`

REUSE-MAP 沒有 T058 的列。與本張相關的只有第 67、156 列（特效的 104 混色、list.spr 標記語意兩邊共用，TW 可照做）。本張的規則全部來自 LiTo unpacked 映像的靜態反組譯，沒有引用 tw-bin-engine 或 lito-launcher-server 的程式碼，也沒有讀它們的檔案。

## 怎麼驗證

依本機規定，**沒有在本機執行任何編譯出來的程式**（含 `cargo test`）。本機只做：靜態反組譯（Python＋pefile＋Capstone，唯讀）、PakViewer CLI 匯出 list.spr（唯讀）、`cargo check`。

### 已驗證

**A. 原版規則（靜態反組譯，映像未執行）**

映像：`D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe`，SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`（與 #338 同一個）。
重現：`python -I docs/reports/issue-338/dis.py <exe> <位址>:<長度>`；呼叫點用 `python -I docs/reports/issue-339/calls.py <exe> <位址>`。

| 位址 | 讀到的內容 |
|---|---|
| `4E0200`–`4E025D` | 本體：`4CD9D0(影格, x, y, word [actor+1A])`，bank 是 `actor+84`。`actor+8C` 非 0 時呼叫 `4E7000`。`actor+88` 非 0 時再提交陰影，字是 `(word [actor+1A] & 0x0FFF) | 5`（`and dh, 0xf`／`or edx, 5`） |
| `4E7000`–`4E70A1` | 逐一處理 `actor+8C` 個附件，bank 在 `actor+90+4i`，字在 `actor+B0+2i`。本體字 `test al, 2` 成立就跳過這個附件；`test al, 0x40` 成立則 `or al, 0x40`；否則 `test ah, 1`（`0x100`）成立則 `or ecx, 1`；都不成立就用附件自己的字 |
| `4E6EA0`–`4E6F6B` | 附件數量與 sprite 清單來自表 `647E48`；bank 從 `647E84` 取，缺的時候呼叫 `4406C0` 載入；字從 `647E68[附件 sprite]` 複製到 `actor+B0+2i`。任何一個 bank 指標是 0 就把 `actor+8C` 清成 0（既有的「缺一個附件就整組不畫」） |
| `4E7EDD`–`4E7F25` | S_PUT_OBJECT 建構時的第一次提交，順序與字的算法和 `4E0200` 相同（本體 → `4E7000` → 陰影 `and ah, 0xf`／`or al, 5`） |
| `4E7C57`–`4E7E6C` | 建構時 `actor+1A` 先取 `647E68[sprite]`（本體的屬性 104），再依封包狀態加上 `0x20`、`0x200`、`2`、`0x100` 或 `0x40`（各自的條件分支沒有在這裡逐一列出；`0x100`／`0x40` 的「自己／別人」區分見下一列） |
| `4E04D3`–`4E0500` | 換 sprite 時同樣重設 `actor+1A`：表值，中毒加 `0x20`，隱身時自己 `0x100`、別人 `0x40` |
| `483770`–`483882` | 插入每格清單：`4837B4` 起找到第一個 depth 不小於新項目的位置（`cmp edx, ecx`／`jle`），把後面的項目往後搬，新項目放在那個位置。同 depth 時新項目在前 |
| `488D50`–`488E74` | 繪製分派：`0x1000`／`0x2000`／`0x4000`／`0x8000` 各自對照一個選項，選項關閉就不畫；`0x40`；`0xC00`；`2` 不畫；`8` → `[8300E0]`；`0x101` → `[768894]`；`0x80` → `[768898]`；`0x10`；`0x20`；`0x200`；其餘一般寫入。與現行 `native_object_dispatch::select` 的順序一致 |
| `4E6B90`／`4E6BF0`／`4E70F0` | 屬性 100 開關成立／解除時，對本體、每個附件、陰影已提交的片段呼叫 `4CDAF0`，並把 `word [actor+1A]` 直接設成 1／0 |
| `4E01A0` | 重新提交前先撤掉本體、附件（`4E70B0`）、陰影的舊片段 |
| `4E7140` | 名稱用的矩形：`actor+84`（本體 bank）目前影格的四個邊界加上 `4B8FC0(actor+2C)` 的座標。不讀附件與陰影 |
| `488C34`–`48A7C9` | 格迴圈、自身名稱（`489E53`）、全體名稱（`489F30`）、指向名稱（`48A2A1`）在同一個函式內，`488F3A`–`489E4A` 之間沒有 `ret`。`489E40`–`48A780` 之間 `actor+1A` 只有 `48A3F5` 一個讀取點，檢查 `0x1000`／`0x2000`／`0x4000`／`0x8000` 與四個選項 |
| 呼叫點 | `4E7000`：2 個（`4E022E`、`4E7EFE`）。`4E6EA0`：1 個（`4E5F10`）。`4E0200`：16 個。`4E70F0`：2 個（`4E6BC0`、`4E6C1B`） |

**B. 資源（PakViewer CLI，唯讀）**

- 查詢的 pak：`D:\lineage開發\0-客戶端\Text.idx`／`Text.pak`，檔名 `list.spr`。指令：`PakViewer.exe -cli export <Text.idx> list.spr <scratch 路徑>`。匯出 4,067,417 bytes，SHA256 `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127`（與 #338 相同）。本張沒有宣稱缺任何資源，所以沒有查其他 pak。
- `list_layers.py` 的盤點（結果在 `list-spr-layers.json`）：
  - 3161 列有陰影。陰影 sprite 自己的屬性 104：0 有 3147 個，256 有 10 個，1 有 2 個，8 與 128 各 1 個。
  - 1608 列有附件（1 個 1308 列、2 個 199 列、3 個 46 列、4 個 51 列、5 個 3 列、6 個 1 列）。附件 sprite 的屬性 104（以引用次數計）：8 有 1251、128 有 450、0 有 357、1 有 6、12 有 2、4104 有 2、8193 有 1。
  - 帶陰影或附件的 3598 列裡，plan 接受的列從 3036 增加到 3225。仍拒絕的 373 列：371 列是本體屬性 104 不是 0 或 1，2 列是附件字不支援（`#3908` 的 12、`#6284` 的 4104）。
  - 同時有屬性 100（遮擋開關）與陰影或附件的有 18 列，都不是競技場的門（`#2083` 沒有陰影也沒有附件）。
- 這份統計沒有把 `native_cube` 對本體 128 的例外算進去，所以「接受的列」是下限。

**C. 編譯**

- `cargo check --locked --offline -j1 --all-targets --target-dir target/current`（在 `LiTo381-Modern/`）：`Finished dev profile`，exit 0。本張新增與修改的 7 個 `.rs` 檔沒有任何警告（輸出裡的警告都在其他既有檔案）。這一步包含 lib 測試與 `tests/` 的型別檢查，不包含執行。

**D. 現行程式的靜態核對（讀碼）**

- 提交順序：`native_world_layers::compose_with_presentations` 依序呼叫本體、附件（依 `bindings.attachments` 的順序，也就是屬性 105 的順序）、陰影；`world_raster` 的 key 是「depth 遞增、同 depth 時後插入的先畫」。與 `4E0200`＋`483770` 相同。
- 隱藏的 actor（`Style::Hidden`）整個不提交，對應 `4E7000` 的 bit 2 與陰影字 `2|5` 在分派時被跳過。
- 別人的隱身（`0x40`）在現行是 `Hidden`（沒有 see-invisible），附件不會被畫，與原版在沒有 focus 選項時相同。
- 名稱在分片合成、照明、HP 條之後才畫（`native_world_scene.rs` 的 `layout_with`／`draw`），不進深度佇列。

### 推測

- `[768894]` 這支寫入器對附件字 1 與陰影字 5 是同一個指標，呼叫時沒有把字傳進去（`push ebx`／`mov edx, esi`），所以推測兩者算法相同（既有 `compositor.rs` 的註解記載是 5 位元平均）。沒有另外反組譯寫入器本體來確認它不讀 bit 4。因為同樣的理由，附件字 12（8 加 bit 4）沒有當成 screen 接受。
- 屬性 100 開關成立後，原版的本體字被設成 1。下一次換影格重新提交時，附件拿到的是自己的字（1 不是 `0x100`），也就是附件會變回原本的寫入器，只有本體保持半透明。現行 #338 的做法是開關成立期間附件一直半透明。兩者只在「有附件、而且開關成立期間會換影格」的物件上不同（list.spr 有 8 列同時帶屬性 100 與附件，例如 `#975` spin door）；競技場的門沒有附件，不受影響。沒有改。
- 屬性 100 開關解除時，原版把陰影片段的字從 5 改成 4。依分派順序，4 會走一般寫入，也就是陰影在下一次重新提交之前可能變成不透明。這是從位元運算推出來的，沒有實機確認；12 列帶陰影的 switch 物件會遇到，競技場的門沒有陰影。沒有改。
- 特效 actor 的本體字同樣來自 `647E68`（`4E04D3`），所以特效本體與附件的寫入器規則與一般 actor 相同。`44F170` 這個位址沒有直接的相對呼叫點（推測是經由函式表呼叫），沒有另外追。
- 選項位元 `0x1000`–`0x8000` 對照的四個選項是 `447990` 以 `0x3B6`、`0x3E9`、`0x3E9`、`0x527` 查出來的值，意義沒有查。

### 待驗證

**交給 PR 的 GitHub Actions CI（`cargo test --locked --lib --no-fail-fast`）**——本機沒有執行，以 CI 的 `test result:` 為準，沒跑或失敗都不算通過：

- `modern_lito::native_actor_layers::tests::`
  - `attachment_words_select_the_writer_of_the_native_dispatch`
  - `only_the_half_state_reaches_an_attachment_as_in_4e7000_and_4cdaf0`（用照抄 `4E7000` 的獨立模型，對六種本體字與四種附件字逐一比對 `Mix::submitted` 選到的分派路徑）
  - `plans_read_the_attachment_word_and_never_the_shadow_sprite_word`（list.spr 的 `#902`、`#1049`、`#3387`、`#3600`、`#3908` 原列）
- `modern_lito::native_world_layer_order_tests::`
  - `equal_depth_layers_paint_shadow_then_attachments_in_reverse_then_the_body`
  - `attachment_writers_follow_the_sprite_word_and_only_inherit_the_half_state`
  - `effects_share_the_depth_queue_and_stay_click_through`
- 既有 lib 測試不能退步，特別是 #338 的 `native_world_occlusion_tests`（輔助項目改了可見性）。

**交給本機 Codex（需要 `D:\lineage開發\0-客戶端`，CI 不跑）**

- `cargo test --test native_actor_layers`、`--test native_actor_states`、`--test native_stalac_layers`、`--test native_type12_prop`：這些既有測試會走 `Plan::from_catalog`，本機只通過編譯。預期不變（它們用的 sprite 附件字都是 0 或 8）。
- 新接受的 189 列沒有任何一列用真實 SPR 畫過。建議抽 `#902`（附件字 1）、`#3387`（8 與 128）、`#3600`（陰影 sprite 字 256）各合成一張，確認 SPR 存在、有 fragments、畫面合理。

**需要實機（GUI 清單 W13、W14，使用者或 T030）**

- 帶 multiply 或半透明附件的 NPC／怪物實際畫面是否與原版相同。LiTo 競技場與租裝城鎮會出現哪些這類 sprite 沒有對伺服器資料查過。
- 自己隱身時身上附件的樣子（一般附件半透明、screen 附件不變）。
- 特效與角色重疊時的前後順序觀感（見結論 4）。
- 站在競技場門格時門的名稱（指向時）是否照常顯示。

## 未驗證／沒做的

- 沒有處理本體屬性 104 不是 0／1 的 371 列（見結論 5）。
- 沒有接 `0x1000`–`0x8000` 的選項條件（繪製與指向名稱都沒有）。
- 沒有還原原版 timer 的插入歷史，所以不同 actor 之間同 depth 的順序仍是新引擎策略。
- 沒有改 #338 的遮擋開關行為（見「推測」第 2、3 點）。
- `4E6F70`（對 actor 動態追加一個附件，唯一呼叫點 `4E5FA0`）只看到它存在，沒有追是誰在什麼情況下呼叫。現行沒有動態附件。
- 沒有改 `docs/REPORT.md`、`PLAN.md`、`TASKS.md`、`docs/G01-G12*.md`、`docs/GUI-ACCEPTANCE.md`。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_actor_layers.rs`
- `LiTo381-Modern/src/modern_lito/native_world_layers.rs`
- `LiTo381-Modern/src/modern_lito/native_world_layer_order_tests.rs`
- `LiTo381-Modern/src/modern_lito/{mod.rs,native_class1.rs,native_earth_jail.rs,native_world_occlusion_tests.rs}`
- `docs/NATIVE-ACTOR-LAYERS.zh-TW.md`、`docs/NATIVE-WORLD-LABELS.zh-TW.md`、`docs/NATIVE-WORLD-EFFECTS.zh-TW.md`
- `docs/reports/issue-339.md`、`docs/reports/issue-339/`

## 給下一棒的提示

- **審查者**：先看 PR CI 的 `test result:` 與上面 6 個測試名稱。行為改變只有兩處：`Plan::from_catalog` 多接受附件字 1 與 `0x80`、不再檢查陰影 sprite 的字。`native_world_layers.rs` 對原本支援的組合（附件字 0／8）結果不變。
- **本體屬性 104（建議另開一張）**：371 列本體字不是 0／1。規則已經在本報告的表裡（`4E7C57` 取表值、`488D50` 分派），做法是讓 `Presentation` 帶本體字並對照 `native_object_dispatch::select`；要一併決定 screen／multiply 本體的點選 owner。帶 `0x1000`–`0x8000` 的列要先查出四個選項的意義。
- **本機 Codex**：請跑上面「交給本機 Codex」的四個整合測試，並回報 `#902`、`#3387`、`#3600` 三張合成圖的結果。若 `Bank::prepare` 回報 `actor attachment lacks qualified fragments`，那是 SPR 內容的問題，不是本張的規則。
- **GUI 清單**：W13／W14 可以補「帶半透明或 multiply 附件的 NPC」與「自己隱身時的附件」兩項觀察；更新清單屬於 T034／主控範圍。
- **畫面等價研究**：遮擋開關對附件與陰影的兩個原版細節（重新提交後附件還原、解除後陰影字變 4）寫在「推測」，要還原需要先有實機畫面佐證。
