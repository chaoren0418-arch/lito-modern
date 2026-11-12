# Issue #336 — T057（M3 1/2）：資源命名空間與格式矩陣（G07.1）

runner：claude-local。分支：`task/336`。本張只做 G07.1；G07.4（失敗診斷）與地圖／LTzone／map_page 屬於 #337。

## 做了什麼

1. **新工具 `維護工具/resource-matrix/resource_matrix.py`**（唯讀、可重跑、綁資源 SHA）
   - 列舉客戶端的 22 個封存（共用 `Sprite`、`Sprite00–15`、`Tile`、`Text`；LiTo 的 `LTicon`、`LTsprite`、`LTsurf`、`LTtext`、`LTtile`）的**每一筆索引記錄**，以及 `map/`、`Sound/`、`Font/`、`LiTo/LTmap/`、`LiTo/LTsound/`、`LiTo/LTSurf/`、`LiTo/` 散檔與 4 個指名的資料檔。
   - 每個來源（idx、pak、散檔樹）都算 SHA-256，再合成一個資源集合雜湊。
   - 每個成員歸到「命名空間 × 容器 × 格式 × 狀態 × 判定」的某一列。工具結尾核對「列加總 == 列舉總數」，不相等就失敗，所以沒有未分類的餘數。
   - 判定來自把 Rust 解碼器在解像素前的檢查照抄成 Python 後逐成員執行：`sprite.rs`（SPR）、`tile.rs`（TIL，含每個區塊的型別）、`inventory_icons.rs`（TBT／LBT，完整走訪）、`native_mode_menu_skin.rs`（IMG）、`audio.rs`（WAV）、`map.rs`（S32，共用與 Native 兩種 portal 規則）。`Text`／`LTtext` 的索引與成員表頭用移植的 L1 解碼（`resource_cipher.rs`）。
   - 三個模式：預設重產、`--check`（重算後與入庫輸出比對）、`--self-test`（不讀客戶端的純邏輯自測）。
2. **產出**
   - `docs/RESOURCE-MATRIX.md`：矩陣本體（來源 SHA、格式矩陣、拒絕原因碼、格式支援表、命名空間事實）。
   - `docs/resource-matrix/matrix.json`：機器可讀版（逐分片、逐地圖目錄、256 種未列空影格數值）。
   - `docs/resource-matrix/exceptions.tsv`：22,844 筆例外成員的名稱與原因（2.4 MB）。
   - 逐成員完整清單（322,822 列、含每個成員的 SHA-256、48 MB）不入庫，重跑後在 `scratch/resource-matrix/members.tsv`，其 SHA-256 記在矩陣裡。
3. **`LiTo381-Modern/tests/resource_matrix_exhaustive.rs`**（`#[ignore]`）：讀 `members.tsv`，對每個屬於解碼器格式的成員實際呼叫 Rust 解碼器，把與矩陣判定不一致的成員寫到 `scratch/resource-matrix/rust-decode.tsv`，有任何不一致就失敗。
4. **CI**：`.github/workflows/build.yml` 加一步 `resource_matrix.py --self-test`。自測除了關卡邏輯，也檢查格式支援表引用的 Rust 模組存在、`tile.rs` 支援的型別集合與 `sprite.rs` 的上限和空影格標記數量沒有變；Rust 端改了這些，CI 會提醒要重新對照。

## 結果摘要（資源集合雜湊 `74DCB59C…C9F951`）

| 狀態 | 成員數 |
|---|---:|
| 有解碼器、容器層通過 | 269,070 |
| 有解碼器、會被拒絕 | 12,964 |
| 只有指名成員有消費端（html／tbl／list.spr／list.ico） | 37,699 |
| 沒有解碼器（png、mp3、seg、Tile 內 xml、bmp、fnt 等） | 2,942 |
| 非遊戲資源 | 147 |
| 合計 | 322,822 |

值得 #337（G07.4）與後續任務注意的發現：

- **SPR 空影格標記**：12,221 個 SPR 含有「邊界反向、零區塊」的空影格，但數值不在 `sprite.rs` `empty_sentinel` 認得的 5 組內，會被當成 `reversed declared frame bounds` 拒絕。未列數值有 256 種，最多的是 `(192,120,-128,-120,0000,0000)`。其中 LiTo 專屬的 `LTsprite` 有 26 個、`LTsurf` 有 3 個。
- **SPR 其他拒絕**：45 個邊界反向但帶區塊；27 個區塊數超過預設上限 16,384；3 個截斷；2 個影格數為 0。
- **WAV**：`Sound/` 有 351 個是 IMA ADPCM（encoding 17）、266 個 fmt chunk 長度不是 16／18，`audio.rs` 都會拒絕；163 個 MP3、1 個 MIDI 沒有解碼器。`LTsound` 的 31 個 WAV 全部通過，18 個 MP3 沒有解碼器。
- **IMG**：9 個超過 `decode_img` 的 1,048,576 像素上限（例如 `LTsurf` 的 `6700.img`、`6701.img`）。
- **TIL**：型別全部落在支援集合內；32 個有非結尾的空槽，28 個只有 empty-tail 入口會接受。
- **TBT**：6 個截斷（`8219`、`8221`、`8254`、`8268`、`8273`、`8279`）。LBT 226 個全部通過。
- **LTmap**：62 個地圖目錄的 495 個 S32 全部以 Native portal 規則解析到 Layer8。共用 `map/` 有 512 個 S32 在 Layer8 後還有未知尾段，`map/18` 只有 SEG。
- **命名空間**：Sprite00–15 的 169,141 個成員全部符合「檔名 bytes 加總取低 4 bits」的分片路由。共用 Sprite 全家有 2,817 個 `.spr` 名稱出現在一個以上封存且內容不同。`LTtile` 的 1,205 個 TIL 與 `Tile` 全部同名、內容全部不同；`LTsprite` 有 295 個與共用 Sprite 同名、內容全部不同。`Text` 有 110 個名稱因 20 bytes 截斷而重複（共 9,246 筆記錄），查這些名稱會得到 `AmbiguousEntry`。

## 怎麼驗證

**已驗證**
- `resource_matrix.py --self-test` 通過（本機）。
- 完整重產一次後立刻跑 `--check`，結果 `check ok`、exit 0：同一份資源重跑得到相同位元組。單次約 4 分 20 秒。
- 逐成員覆蓋：工具內的加總核對通過（322,822 == 322,822）。
- `cargo check --locked --offline -j1 --test resource_matrix_exhaustive` 通過（只編譯，沒有執行）。
- 上面各項數字都是 Python 關卡逐成員執行的結果，可用 `members.tsv`／`exceptions.tsv` 逐筆回查。

**推測**
- 「Python 關卡拒絕 ⇒ Rust 解碼器也拒絕」：關卡是逐行對照原始碼抄的，但沒有實際跑 Rust。
- 「容器層通過 ⇒ Rust 能完整解碼」：SPR／TIL 的 RLE 本體、影格尺寸與總像素上限沒有在 Python 端檢查，所以通過數是上限。
- LBT 與 TBT 同格式：依據是 `native_inventory_icons.rs` 用同一個 `decode_tbt_cancellable`，而且 226 個 LBT 全部通過走訪。
- 12,221 個未列空影格在原版客戶端是「跳過不畫」：依據是 `sprite.rs` 對已列 5 組的註解（Native 4CD9D0 跳過區塊迴圈），沒有對這 256 種逐一查原版。

**待驗證**
- 執行 `resource_matrix_exhaustive`（需要本機客戶端資源；本機執行限制不允許我跑編譯出來的程式，CI 沒有資源）。它會給出 Rust 解碼器對 28 萬個成員的實際結果與不一致清單。
- CI 結果（push 後看 PR）。
- 任何畫面層面的確認（哪些被拒絕的 SPR 實際會在遊戲裡出現）需要實機。

## 未驗證／沒有做的事

- 沒有改 `src/`：這張只建立矩陣。空影格標記、ADPCM、IMG 上限等要不要放寬，是 G07.4（#337）或各自任務的決定。
- 沒有逐成員解 `Text` 的內容（只解索引與每個成員前 16 bytes 判斷編碼開頭）；html／tbl 的語法沒有驗。
- 沒有用 PakViewer 交叉比對索引。本報告沒有宣稱任何資源「缺件」；所有數字都來自直接讀 idx／pak。
- 沒有更新 `docs/G01-G12.md`、`docs/G01-G12-AUDIT.md` 的 G07.1 狀態字樣（那兩份由各自的流程維護）。建議的新狀態見下。
- `LiTo.bin`、執行檔、設定檔、紀錄與 `工具包/` 不納入資源集合，只列數量。

## 產出路徑

- `維護工具/resource-matrix/resource_matrix.py`
- `docs/RESOURCE-MATRIX.md`
- `docs/resource-matrix/matrix.json`
- `docs/resource-matrix/exceptions.tsv`
- `LiTo381-Modern/tests/resource_matrix_exhaustive.rs`
- `.github/workflows/build.yml`（加一步自測）
- `docs/reports/issue-336.md`（本檔）
- 不入庫：`scratch/resource-matrix/members.tsv`

## 給下一棒的提示

- **#337（G07.4）**：`exceptions.tsv` 的 `verdict` 欄就是診斷要顯示的來源原因；`matrix.json` 的 `spr_unlisted_empty_frame_markers` 是 256 種未列數值與各自的成員數。先決定空影格要「列舉放行」還是「零區塊反向一律視為空影格」，這一項就涵蓋 12,221 個成員。
- **能在本機跑程式的執行者（Codex）**：先跑 `python -X utf8 維護工具/resource-matrix/resource_matrix.py` 產生 `members.tsv`，再跑
  `cargo test --locked --offline -j1 --target-dir target/current --test resource_matrix_exhaustive -- --ignored --nocapture`。
  不一致清單在 `scratch/resource-matrix/rust-decode.tsv`。如果不一致集中在「Python 通過、Rust 拒絕」，多半是 RLE 本體或尺寸上限，請把原因碼補進工具的關卡再重產。
- **客戶端資源會變**：`Sprite10–14` 的 idx／pak 修改時間是 2026-11-09。資源一變，`--check` 就會失敗，這是預期行為；重產後在 PR 說明差異。
- **`src/` 改動解碼器時**：若改了 `tile.rs` 的型別集合、`sprite.rs` 的上限或 `empty_sentinel`，CI 的自測會失敗，請同步改 `resource_matrix.py` 的對應關卡並重產矩陣。
- G07.1 的狀態建議由 `…exhaustive_member_coverage_pending` 改為「逐成員矩陣已完成（Python 關卡），Rust 逐成員實跑待驗證」；等 `resource_matrix_exhaustive` 跑過再標完成。
