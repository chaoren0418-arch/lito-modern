# Issue #338 — T058（M3 1/2）：地形、跨區塊、遮擋與屋頂（含屋頂進出）（G07.3）

執行者：claude-local。分支 `task/338`（基底 `origin/main` `b8aa9db`）。程式在第 1 次嘗試完成（commit `53fb769`）；第 2 次嘗試只補交本報告、合併當時的 `origin/main`（落後 40 個 commit，自動合併無衝突）並重跑 `cargo check`，沒有再改程式。
本張範圍只有「地形、跨區塊、遮擋與屋頂（含屋頂進出）」；附件、名稱與特效排序是 #339。

## 結論

1. **找到並補上一個原版有、現行沒有的遮擋規則**：list.spr 屬性 100（`100.switch`）。原版在本地角色站到物件列出的格子時，把該物件（本體與附屬 sprite）改用半透明寫入器繪製。LiTo 競技場的門就是這種物件（`#2083`，`100.switch(1 -2,0,2,0)`）。現行 `main` 把屬性 100 當成不解讀的原始位元組，沒有任何程式使用它。本張新增 `native_actor_switch` 並接到 `native_world_scene`。
2. **屋頂進出與跨區塊第一次有 CI 可重跑的證據**：以前的證據都需要本機客戶端資源（`tests/native_world_roofs.rs`，CI 不跑），而且 TCP 路線沒有走進觸發格（`NATIVE-WORLD-ROOFS:35`）。本張新增不需要資源的合成場景測試，走現行正式路徑：錄製的世界封包加預測步 → 屋頂 runtime → 分片 raster。
3. **對照原版時發現一個未決的差異**（沒有改）：觀戰模式下現行用觀戰相機所在格觸發屋頂；原版 `48AFC0` 只有本地角色步進與 S_PUT_OBJECT 三個呼叫點，觀戰時是否會觸發沒有證據。
4. G07.3 仍是 partially_verified：實機的屋頂進出、門半透明（GUI 清單 W13、W14）沒有做，本張新增的 lib 測試結果要等 PR 的 CI。

## 做了什麼

### 程式

| 檔案 | 變更 |
|---|---|
| `LiTo381-Modern/src/modern_lito/native_actor_switch.rs`（新增） | 解析屬性 100（`count` 與 `count` 組 `x0,y0,x1,y1`）；`covers(物件格, 本地格)`；`present()` 把成立的物件改成 `Style::Half`；`Cache` 依 logical graphics 快取，格式不合時回報錯誤、不套用。5 個單元測試 |
| `LiTo381-Modern/src/modern_lito/native_world_scene.rs` | `Model` 多一個 `actor_switches` 快取；畫每個 actor 時用「actor 邏輯格」與「本地角色邏輯格」判斷，觀戰者傳 `None`；actor receipt 多 `occlusion_switch { listed, switched, error }`。屋頂的身份與觸發格改呼叫 `native_world_roofs::scope`（行為不變，只是抽出來讓測試能走同一段） |
| `LiTo381-Modern/src/modern_lito/native_world_roofs.rs` | 新增 `scope(&World)`：玩家用本地角色的邏輯格與 incarnation，觀戰者用相機格與 world generation（原邏輯原樣搬出） |
| `LiTo381-Modern/src/modern_lito/native_resources.rs` | 把載入器裡「常駐視窗範圍」與「各區塊的靜態參照＋Layer4 群組序號」兩段原樣抽成 `resident_window`、`collect_static_draws`（載入器改呼叫它們）；多一個 `ensure!(tiles.len() == groups.len())`，避免 `zip` 靜默截斷造成後面的屋頂群組全部錯位。新增只在測試編譯的 `NativeStaticAssets::from_owned_sources_for_test`，用呼叫端提供的 S32／TIL 位元組走同一段收集與 TIL 解碼 |
| `LiTo381-Modern/src/modern_lito/native_world_occlusion_tests.rs`（新增） | 3 個合成場景測試，見下 |
| `LiTo381-Modern/src/modern_lito/mod.rs` | 註冊上面兩個模組 |
| `LiTo381-Modern/tests/native_world_roofs.rs` | 新增 `arena_door_switch_follows_the_local_logical_cell_on_the_formal_scene`（需要本機客戶端資源，CI 只編譯不執行） |

### 文件
- `docs/NATIVE-WORLD-ROOFS.zh-TW.md`：在檔尾新增三節（`100.switch`、CI 合成場景、#338 之後仍未完成）。第 1–35 行沒有動，所以 `TASKS.md`／`GUI-ACCEPTANCE.md` 引用的 `NATIVE-WORLD-ROOFS:35` 仍指向同一句。
- `docs/reports/issue-338/`：靜態分析腳本（`dis.py`、`roof_callers.py`、`xref.py`、`list_switch.py`）與 196 筆屬性 100 清單 `list-spr-switch.tsv`。

### 查過 `docs/REUSE-MAP.md`
- REUSE-MAP 沒有 T058 的列。唯讀查了兩個來源：
  - lito-launcher-server `origin/main` `edfb5c82`：`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/BattleDoor.java:7` `static final int GFX = 2083;`（競技場門的 gfx）。
  - tw-bin-engine `origin/main` `41b0fe2c`：在 `engine/source/*.cs` 與 `docs/reference/*.md` 搜尋 `.switch`／`switch(`／屋頂／roof／Layer5，沒有找到屬性 100 的處理（推測：只做了關鍵字搜尋，沒有逐檔讀）。沒有引用它的程式碼。
- 規則來源是 LiTo 的 unpacked 映像靜態反組譯（下節），不是 TW16。

## 怎麼驗證

依本機規定，**沒有在本機執行任何編譯出來的程式**（含 `cargo test`）。本機只做：靜態反組譯（Python＋pefile＋Capstone，唯讀）、PakViewer CLI 匯出 list.spr（唯讀）、`cargo check`。

### 已驗證

**A. 原版規則（靜態反組譯，映像未執行）**
映像：`D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe`，7,094,272 bytes，SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`（與既有 `驗證紀錄/current/native-world-roofs/native-roof-raw.json` 用的是同一個映像）。重現方式：`python -I docs/reports/issue-338/dis.py <exe> <位址>:<長度>`。

| 位址 | 讀到的內容 |
|---|---|
| `43E377`–`43E43F` | list.spr 屬性編號 `0x64`（100）：第一個數字是 count，寫進 `[647E80 + sprite*8]`；配置 `count<<4` bytes；之後每組依序讀 4 個數字到 `+0`、`+4`、`+8`、`+C` |
| `4E6C50`–`4E6D98` | 以 `word [actor+14]` 查上表。每個矩形：y 從 `actor+30 + [+4]` 到 `actor+30 + [+C]`；x 從 `actor+2C + [+0]` 起 `add esi, 2` 到 `actor+2C + [+8]`；每格 `push 2`、`call 483890`（kind 2）。若本地角色（`[834EE4]`）的 `+2C`／`+30` 等於該格，回傳 1 |
| `62F3AC` 表 | 前四個 dword 是 `1, 2, 5, 0x800000`（引用既有 `native-roof-raw.json` 的 `kind_flag_table`）。kind 2 → 旗標 5（含 bit 4） |
| `48AFC0`–`48B0CA` | 參數是舊格；另取本地角色 `+2C`／`+30` 的格。舊格的登記：`test byte, 4` 成立 → `call 4E6BF0`；否則清 `[+6]`。新格的登記：bit 4 成立 → `push 1; call 4E6B90`；否則走 `433210`（屋頂目標）。也就是 bit 4 的登記不進 50 ms 漸變 |
| `48AFC0` 的呼叫點 | 全映像搜尋 `E8` 相對呼叫只有 3 個：`4E2F2F`（本地角色步進，更新 `+2C`／`+30` 之後、傳入舊座標）、`4E7F5D`（S_PUT_OBJECT 後 `4E6C50` 回傳 1 時）、`4E7F9A`（放入的就是本地角色時） |
| `4E6B90`／`4E6BF0` | 對 `actor+84`、`actor+88`、`actor+8C` 的陣列（`4E70F0`）呼叫 `4CDAF0(方向, x, y, 值)`，並把 `word [actor+1A]` 設成 1／0 |
| `4CDAF0` | 逐片段找到格內的登記，`and edx, 0xFFFFFFFC; or edx, 值`，也就是 `(flags & !3) | 值` |
| `488DAD`–`488DE0` | 繪製分派順序：`0xC00` → `2`（不畫）→ `8` → `test eax, 0x101`（半透明寫入器 `[768894]`）→ `0x80` → `0x10` → `0x20`。與現行 `native_object_dispatch::select` 的順序一致 |

**B. 資源（PakViewer CLI，唯讀）**
- 查詢的 pak：`D:\lineage開發\0-客戶端\Text.idx`／`Text.pak`，檔名 `list.spr`。指令：`PakViewer.exe -cli export <Text.idx> list.spr <scratch 路徑>`。匯出 4,067,417 bytes，SHA256 `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127`。本張沒有宣稱缺任何資源，所以沒有查其他 pak。
- `list_switch.py` 盤點：196 列帶 `100.switch(...)`，全部符合「count 後面剛好 4×count 個數字」；count 最大 7，偏移範圍 −18～58；49 列含奇數 x 偏移。清單在 `list-spr-switch.tsv`。
- `#2083 riderhouse a r door`：`100.switch(1 -2,0,2,0)`。錄製的世界 fixture（`tests/fixtures/native-world-current.json` 的 `door` 封包）gfx 是 `0x0823`＝2083，伺服器 `BattleDoor.GFX = 2083`。

**C. 編譯**
- `cargo check --locked --offline -j1 --all-targets`（工作樹內 `LiTo381-Modern/target/current`）：`Finished dev profile`，exit 0。本張新增與修改的檔案沒有新的警告（輸出裡 `native_resources.rs:551`、`:369` 的兩個警告是原本就有的位置，不在本張改動的範圍）。這一步包含 lib 測試與 `tests/` 的型別檢查，不包含執行。
- 第 2 次嘗試：合併 `origin/main`（`mod.rs`、`native_world_scene.rs` 兩邊都有改，自動合併）後再跑同一個指令，`Finished dev profile`，exit 0。合併後的 `native_world_scene.rs`（main 那邊新增了名稱／徽章相關改動）只確認能編譯，沒有逐行重新對照兩邊改動的交互作用。

**D. 現行程式的靜態核對（讀碼）**
- 屬性 100 在 `sprite_actions.rs` 走「不解讀的屬性」分支（`id != 100 && body.contains(',')` 的另一邊），`main` 上沒有任何消費者（grep `100.switch`、`key.id == 100` 等只有 parser 自己的測試）。
- Layer4 群組序號與靜態參照的對齊：`static_draw_list` 的順序是 8192 個 floor、overlay、再依群組順序的物件；載入器的 `groups` 是同樣順序，長度一定相等（本張另外加了 `ensure!`）。
- 屋頂觸發格用本地角色的 `x`／`y`；`World::predict_step` 在送出移動時就更新這兩個欄位，與原版 `4E2F2F` 在步進開始時呼叫 `48AFC0` 的時機一致（原版顯示內插不影響）。

### 推測

- `actor+84`／`+88`／`+8C` 分別是本體、陰影、附件：`+88` 是陰影有既有註解佐證（`compositor.rs` 的 `blit_native_shadow_rgb555`：「Native actor+0x88 shadow callers add draw flags 5」），`+8C` 是「數量＋指標陣列」的形狀與 `105` 附件上限 8 相符；沒有另外追建構點。
- 把 switch 成立對應到既有的 `Style::Half`（本體與一般附件半透明、旗標 8 附件維持 screen、陰影不變）：依據是旗標運算 `(flags & !3) | 1` 與分派順序推導（本體 0→1、screen 附件 8→9 仍先走 8、陰影 5→5）。沒有像素級的原版對照。
- 現行以兩個座標的純函數判斷，等同原版「登記在 S_PUT_OBJECT 當下的位置」只對之後不移動的物件成立。門不移動；船（`#104`、`#265` 等）會移動，原版的重新登記（`4E6DA0` 看起來是取消登記）時機沒有還原。LiTo 只有競技場與租裝城鎮，推測不會遇到會移動的 switch 物件。
- 原版離開 switch 格時把陰影旗標從 5 改成 4（`(5 & !3) | 0`），依分派順序會變成一般不透明寫入。有陰影的 switch 物件離開後陰影可能變不透明；競技場門沒有 `101.shadow`，所以沒有處理。
- x 偏移為奇數的矩形（49 列）登記在兩個整格之間的 half-cell，角色的 half-cell 永遠是偶數，推測原版也不會觸發；現行同樣不觸發。
- 租裝城鎮用到哪些帶屬性 100 的門 sprite 沒有逐一對伺服器資料；清單裡 196 個都已支援，所以不影響行為。

### 待驗證

**交給 PR 的 GitHub Actions CI（`cargo test --locked --lib --no-fail-fast`）**——本機沒有執行，以下 8 個新測試以 CI 的 `test result:` 為準，沒跑或失敗都不算通過：
- `modern_lito::native_actor_switch::tests::`
  - `pinned_rows_parse_to_native_rectangles_and_absent_rows_stay_unlisted`
  - `malformed_tables_are_rejected_instead_of_partially_applied`
  - `coverage_equals_the_native_registration_loop_for_every_pinned_row`（用照抄 `4E6C50` 迴圈的獨立實作逐格比對 `covers`）
  - `decision_cache_follows_the_local_cell_and_never_switches_for_observers`
  - `switched_presentation_is_half_for_body_and_linked_layers_but_never_reveals_hidden`
- `modern_lito::native_world_occlusion_tests::`
  - `seam_roof_groups_keep_sector_ordinals_and_reject_wrapped_layer5_records`
  - `wire_route_enters_and_leaves_roofs_across_the_sector_seam`
  - `resident_terrain_window_matches_the_whole_map_for_every_camera_of_its_block`
- 另外，`native_resources.rs` 的載入器改成呼叫抽出的函式，既有 lib 測試不能退步。

**交給本機 Codex（需要 `D:\lineage開發\0-客戶端`，CI 不跑）**
- `cargo test --test native_world_roofs`：新增的 `arena_door_switch_follows_the_local_logical_cell_on_the_formal_scene`（真實 list.spr 與錄製的門封包，檢查門的 receipt 在門格與左右格是 `switched=true`／`body_draw_style=Half`，其餘位置不是），以及既有 6 個屋頂測試在 `native_resources.rs` 重構後的結果。這個新測試本機只通過編譯，沒有執行過。
- 真實地圖載入路徑（`load_static_retained_geometry_window`）在重構後沒有任何執行證據；CI 沒有客戶端資源，只有上面的整合測試會走到。

**需要實機（GUI 清單 W13、W14，使用者或 T030）**
- 連線後走進／走出屋頂：半透明與完全隱藏兩種、50 ms 漸變觀感、跨區塊的建築。
- 站在競技場門的門格與左右格時門是否半透明、離開後是否恢復；原版是不是一進格就立刻半透明（現行沒有漸變）。
- 觀戰模式下屋頂是否應該隨相機變化（見結論 3）。
- 半透明的門能不能被點選（現行沿用隱身本體的寫法，保留 owner；原版行為沒有查）。

## 未驗證／沒做的

- 沒有重做既有的 Layer5 位元組證據（`483890`、`432F70`、`4330D0`、`4C1A17`）與 495 個 S32 的盤點；本張只引用，並新增上表的位址。
- 沒有還原原版 timer queue、tile pointer alias、RGB565 framebuffer（這些原本就列在 `NATIVE-WORLD-ROOFS` 的未完成，且 RGB565 等價不在 G07）。
- 沒有改觀戰模式的屋頂行為。
- 合成場景只有一種 floor、兩種不透明 RLE 物件磚；TIL 6／7（基礎半透明）與 0x10／0x20 特效磚在屋頂下的行為仍只有 `world_raster` 既有單元測試與本機資源測試涵蓋。
- 沒有改 `docs/REPORT.md`、`PLAN.md`、`TASKS.md`、`docs/G01-G12*.md`、`docs/GUI-ACCEPTANCE.md`。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_actor_switch.rs`
- `LiTo381-Modern/src/modern_lito/native_world_occlusion_tests.rs`
- `LiTo381-Modern/src/modern_lito/{mod.rs,native_resources.rs,native_world_roofs.rs,native_world_scene.rs}`
- `LiTo381-Modern/tests/native_world_roofs.rs`
- `docs/NATIVE-WORLD-ROOFS.zh-TW.md`
- `docs/reports/issue-338.md`、`docs/reports/issue-338/`

## 給下一棒的提示

- **審查者**：先看 PR CI 的 `test result:` 與上面 8 個測試名稱。`native_resources.rs` 的改動是原樣搬移（`resident_window`、`collect_static_draws`），可以用 `git diff -w --color-moved` 對；唯一新增的檢查是 `Native draw/group alignment`。`native_world_scene.rs` 的屋頂取樣只是改呼叫 `scope`，條件與原本相同。
- **#339（T058 2/2，附件、名稱、特效排序）**：switch 成立時附件也會被原版加旗標 1；本張用 `Style::Half` 表達，附件的實際呈現屬於 #339 的核對範圍。陰影旗標 5→4 的後果也可以在那張一併確認。名稱（`native_world_labels`）在物件半透明時是否仍顯示沒有查。
- **本機 Codex**：請跑 `cargo test --test native_world_roofs`（7 個測試），並把新門測試的 receipt（`occlusion_switch`、`body_draw_style`）附在結果裡。若門 receipt 的 `rendered` 不是 true，先看 `layer_error`／pose 錯誤，那不是本張引入的條件。
- **GUI 清單**：建議在 W13／W14 補一項「站在競技場門格與左右格時門半透明」，以及「觀戰時屋頂是否變化」的觀察；更新清單屬於 T034／主控範圍。
- **畫面等價研究**：若之後要還原會移動的 switch 物件，起點是 `4E6DA0`（取消登記）與 `4E6C50` 的呼叫者；觀戰相機怎麼移動（是否經過 `4E2150`）決定結論 3。
