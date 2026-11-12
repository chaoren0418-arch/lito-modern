# Issue #337 — T057（M3 2/2）：缺件與未知格式的失敗診斷、地圖域、LTzone、map_page（G07.4）

runner：claude-local。分支：`task/337`。本張做 G07.4 與地圖相關四項；G07.1 的矩陣在 #336 已完成。

## 結論

- 世界畫面的資源失敗（缺附件、缺陰影、本體或附加層解碼失敗、音效載入失敗、三個 TW 特效資格不符、區域表讀不到）現在會收成同一種紀錄，**HUD 會多一行摘要**，影格 receipt 有完整清單，stderr 每個失敗記一次。原本這些只散在 receipt 的各個欄位，玩家看不到。
- 地圖載入失敗時，HUD 會說明這張地圖該讀哪個域、兩個域各有沒有資料、為什麼不改讀另一個域。
- 新增 LTzone 區域表解析；HUD 的地圖編號後面會顯示區域名稱（目前只有 500 LODUN、560 LENOA）。表裡沒有安全區欄位，程式不由顏色推導任何規則。
- 地圖頁：原版圖片表只有第 1、2 頁。開著地圖頁時 HUD 會說明這一頁為什麼看不到圖。
- 新增可重跑的盤點工具與 `docs/MAP-DOMAINS.md`（雙域 35 張逐張比較、11 張經典地圖、三張區域表、兩張地圖頁圖片）。
- 沒有放寬任何解碼器，也沒有開始繪製地圖頁圖片。
- 本機只做了編譯檢查與 Python 工具；Rust 測試沒有在本機執行。CI 只在 PR 上跑，PR 由調度器在審查後開啟，所以寫報告時還沒有 CI 結果。

## 做了什麼

### 1. `src/modern_lito/resource_diagnostics.rs`（新增）

- `Diagnostic`：誰要用（`consumer`）、哪一類（`kind`）、來源（`source`）、成員（`member`）、原始原因（`reason`）。
- `Kind`：缺件、格式不支援、內容損毀、超過上限、同名多筆、不可用、其他失敗。
- `classify(&anyhow::Error)`：依錯誤鏈裡的型別分類（`ResourceError`、`SpriteError`、`TileError`、`WaveError`、`MapError`、`io::Error`）。不靠比對錯誤文字。
- `collect(&receipt)`：從一張影格 receipt 取出所有資源失敗。隱藏的角色不算失敗。
- `Ledger`：跨影格去重，上限 128 筆，超過的計數。`hud()` 給一行摘要，`receipt()` 給完整清單。
- `Domain`、`TerrainPresence`、`explain_terrain`、`terrain_diagnostic`：地圖域的規則與載入失敗的來源說明。
- 常數：`CLASSIC_DOMAIN_MAPS = [4]`、`CLASSIC_REFRESHED_MAPS`（11 張）、`DUAL_DOMAIN_REFERENCE_MAP = 504`。

### 2. `src/modern_lito/native_world_scene.rs`（接線）

- 角色本體失敗、附加層失敗、音效載入失敗的 receipt 多帶 `error_kind`／`layer_error_kind`／`sound_id`。
- `Model` 多兩個欄位：`zones`（LTzone 表或讀取失敗的原因）、`diagnostics`（場景的診斷清單）。
- 每張影格的 receipt 多兩欄：
  - `zone`：來源、區域數、目前位置的名稱與顯示色、`combat_rule_from_color:false`；表讀不到時是 `table_error`。
  - `resource_diagnostics`：`count`、`dropped`、`limit`、`rows`、`hud`。
- 區域表讀不到不會讓世界載入失敗，只會變成一筆診斷；取消仍照常中止。

### 3. `src/modern_lito/native_zone.rs`（新增）

- `Table::parse`：總列數、色彩列數、色彩列、區域列；`;` 註解；名稱保留原始位元組。
- 明確拒絕：列數不符、色彩索引未定義、欄位數不對、非數字、NUL、超過 64 KiB、同一張地圖兩筆整張地圖的區域。
- `Table::load`：讀 `LiTo/LTtext` 的 `LTzone-c.tbl`。
- `Table::label(map, x, y)`：只回名稱與顯示色。

### 4. `src/modern_lito/native_map_item.rs`

- `NATIVE_PAGE_IMAGES = [6700, 6701]`、`page_image(page)`。
- `View::diagnostic()`：第 1、2 頁是「圖片在資源裡，但本版還沒有繪製」；第 3–18 頁是「原版圖片表沒有這一頁」。
- `View::status()`：頁碼加上診斷，給 HUD 用。原有的 `label()` 不變。

### 5. `src/modern_lito/native_login_window.rs`（HUD）

- 世界載入失敗：原訊息後面多一行地圖域說明。
- 世界畫面：地圖編號後面顯示區域名稱；最後多「資源診斷…」與地圖頁狀態（有才顯示）。
- 既有 GUI 驗收腳本用的 `地圖 500`、`世界資源載入失敗` 字樣都還在。

### 6. `src/modern_lito/native_actor_layers.rs`

- 分片規則抽成 `shard_of(name)`，`Bank::ensure` 與診斷的來源路徑共用同一個函式。行為不變。

### 7. 工具與文件

- `維護工具/resource-matrix/map_domains.py`：重產、`--check`、`--self-test`、`--pe <脫殼 PE>`。
- `docs/MAP-DOMAINS.md`、`docs/resource-matrix/map-domains.json`：工具產物。
- `.github/workflows/build.yml`：加一步 `map_domains.py --self-test`。自測也檢查 Rust 端的地圖頁圖片表、經典域清單、11 張清單、選域條件與 `decode_img` 上限字樣還在；Rust 改了，CI 會提醒重新對照。

## HUD 會看到什麼（範例，依程式的格式字串）

```text
地圖 500（LODUN）　HP 120/120　…
…
方向鍵移動；Space攻擊；O門；P入口；Esc清除。未顯示 1 個物件。
資源診斷 3 項（缺件 2、不可用 1）；最新：技能特效不可用：技能 77 究極光裂術（launcher/TW13081901.pak）—…
地圖第 5 頁（共 18 頁）；地圖頁缺件：第 5 頁（原版圖片表 dword_62F7BC）—原版圖片表只有第 1、2 頁（6700、6701），這一頁沒有對應圖片
```

```text
世界資源載入失敗：no Native S32 sectors
地圖 77 讀競技場域（LiTo/LTmap/77），但這個資料夾不存在。同編號只有經典域（map/77）的資料；兩個域的地形與通行不同，不會拿來代替。這是伺服器更新過通行資料的 11 張經典地圖之一，客戶端沒有競技場域的版本。
```

這兩段是照格式字串寫的範例，不是實機截圖。

## 已驗證

**編譯**
- `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`：結束碼 0（含測試目標的編譯）。
- 合併 `origin/main`（落後 23 個 commit，`native_world_scene.rs` 一處衝突：取 main 的那一列再補回 `layer_error_kind`）後重跑同一個指令，結束碼 0；兩個 Python 自測也重跑通過。

**工具**
- `map_domains.py --self-test`、`resource_matrix.py --self-test`：都印出 `self-test ok`。
- `map_domains.py` 重產後立刻 `--check`：`check ok`。
- `map_domains.py --pe <LiTo.unpacked-oep-current.exe>`：`pe check ok`。PE SHA-256 `4271D92F…5C2AB6E9`；VA `0x0062F7BC`（檔案位移 `0x22F7BC`）是 `2C 1A 00 00 2D 1A 00 00`（6700、6701），後面緊接 `%d.alp\0`。同一個位址在檔案裡出現 14 次，都是以 `[物件+0xC]` 為索引乘 4 取值的指令。

**地圖域（`docs/MAP-DOMAINS.md`）**
- 競技場域 `LiTo/LTmap` 有 62 張地圖；共用 `map/` 有 717 個資料夾。
- 兩邊都有的編號共 35 個（含 500、504）。35 張的同名 S32 沒有任何一個內容相同。
- 504：競技場域 7 個 S32（780,293 bytes），經典域 31 個（4,389,601 bytes）；同名 6 個，全部不同。
- 11 張經典地圖（77–82、104、105、108–110）在共用 `map/` 都有（77–82 各 4 個 S32，其餘各 9 個），在 `LiTo/LTmap` 都沒有資料夾。
- 客戶端的選域規則在 `native_world_scene.rs` `load_terrain`：只有地圖 4 走經典域，其餘只讀競技場域，沒有後備。

**LTzone**
- `LTzone-c.tbl`（`LiTo/LTtext`，解碼後 125 bytes，SHA-256 `7DA1B04C…0AD65`）：2 色、2 區，500 LODUN（45 95 209）、560 LENOA（10 173 90），都是整張地圖。內容與伺服器的 `NativeLitoZoneCatalog` 相同。
- 表的欄位只有名稱、色彩索引、地圖、四個座標。
- 其餘 60 張競技場域地圖（含 504）沒有區域列。
- `zone2-c.tbl` 宣告 350 列、實際 344 列。客戶端沒有讀這張表。

**地圖頁**
- `6700.img`、`6701.img` 在 `LiTo/LTsurf`，各 1536×768、2,359,304 bytes，超過 `decode_img` 的 1,048,576 像素上限。
- 共用 `Sprite08` 也有一個同名的 `6700.img`，只有 9,608 bytes。

**附件與陰影缺件（PakViewer 查詢）**
- 工具：`pakviewer-cli pak search <idx> <pattern>`。
- 查過的索引 24 個：`Sprite.idx`、`Sprite00.idx`–`Sprite15.idx`、`Text.idx`、`Tile.idx`、`LiTo/LTicon.idx`、`LiTo/LTsprite.idx`、`LiTo/LTsurf.idx`、`LiTo/LTtext.idx`、`LiTo/LTtile.idx`（客戶端根目錄與 `LiTo/` 下所有 `.idx`；`launcher/` 沒有 `.idx`）。
- 查詢字樣：`383-`、`727-`、`262-`、`6700.img`、`6701.img`、`zone`。
- 結果：
  - 附件 bank 383 只有 `383-4.spr`、`383-20.spr`（在 `Sprite`）。`383-8`–`383-15`、`383-24`–`383-31` 共 16 個在 24 個索引裡都沒有。
  - 本體 727 有 4、8–15、20、24–31；`727-16`–`727-19`、`727-21`–`727-23` 共 7 個都沒有。
  - 陰影 bank 262 有 0、4、8–15、20、24–31；16–19、21–23 沒有。
- 另外用 Python 直接解析同一批索引，結果與 PakViewer 相同。
- 原始輸出在 `scratch/issue-337/pakviewer-search.txt`（不入庫）。

## 推測

- 原版以 `LiTo/LTsurf` 的 `6700.img`／`6701.img` 當地圖頁。依據：尺寸、兩張成對、與兩個區域（500、560）數量相同；沒有追原版的讀檔路徑。
- 伺服器送第 3–18 頁時，原版會把表後面的字串位元組當圖片編號。依據是表的長度；沒有追後續分支。
- `LTzone` 有範圍的列優先於整張地圖的列。原版的比對順序沒有查；目前的表只有整張地圖的列，兩種順序結果相同。
- 原版讀的是 `LTzone-c.tbl`，不是 `zone2-c.tbl`。依據是伺服器 repo 的註解，沒有對 LiTo.bin 查字串引用。
- HUD 多出來的行在現有的唯讀多行文字區放得下。沒有看過畫面。

## 待驗證

- **Rust 測試**：新增 9 個測試（`resource_diagnostics` 5、`native_zone` 3、`native_map_item` 1），本機沒有執行，等 PR 開啟後 CI 的 `cargo test --lib`。新測試只編譯過；如果 CI 有測試失敗，先看這 9 個。
- **實機**（交給 Codex 或使用者）：
  1. 進地圖 500，HUD 第一行是「地圖 500（LODUN）」；進 504 沒有括號。
  2. 場上有 gfx 727 的角色時，HUD 出現「資源診斷…角色附件缺件：383-N.spr（SpriteXX→Sprite）」。
  3. 學了技能 77／34／38 之一時，HUD 出現「技能特效不可用…launcher/TW13081901.pak」。
  4. 使用地圖道具後，HUD 出現地圖頁那一行。
  5. HUD 文字區有沒有被新增的行擠掉原有內容。
- 伺服器那 11 張經典地圖更新後的通行資料，與客戶端 `map/` 的 S32 是否一致：沒有比對。伺服器的通行資料不在客戶端，客戶端目前也不載入這 11 張。
- `resource_matrix_exhaustive`（#336 留下的 Rust 逐成員實跑）仍然沒有跑。

## 沒有做的事

- 沒有繪製地圖頁圖片，也沒有調高 `decode_img` 上限。
- 沒有放寬 SPR 空影格標記（#336 提到的 12,221 個成員）。這會改變解碼結果，需要先決定「列舉放行」或「零區塊反向一律視為空影格」，本張只負責診斷。這類失敗現在會以「內容損毀」出現在診斷裡。
- 沒有改三個 TW 特效的資格檢查或 SHA 常數。
- 沒有把診斷寫成檔案；目前在 HUD、影格 receipt 與 stderr。
- 世界畫面以外的資源失敗（背包圖示、登入與大廳的 UI 圖）沒有接進這份清單。
- 沒有更新 `docs/G01-G12.md`、`docs/G01-G12-AUDIT.md` 的狀態字樣，也沒有改 `docs/REPORT.md`。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/resource_diagnostics.rs`（新增）
- `LiTo381-Modern/src/modern_lito/native_zone.rs`（新增）
- `LiTo381-Modern/src/modern_lito/native_map_item.rs`
- `LiTo381-Modern/src/modern_lito/native_world_scene.rs`
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`
- `LiTo381-Modern/src/modern_lito/native_actor_layers.rs`
- `LiTo381-Modern/src/modern_lito/mod.rs`
- `維護工具/resource-matrix/map_domains.py`（新增）
- `docs/MAP-DOMAINS.md`、`docs/resource-matrix/map-domains.json`（新增，工具產物）
- `.github/workflows/build.yml`
- `docs/reports/issue-337.md`（本檔）
- 不入庫：`scratch/issue-337/`（PakViewer 輸出與一次性腳本）

## 參考來源（唯讀）

- lito-launcher-server `origin/main`：`lito伺服器/LiTo381-Server/docs/maintenance/native-de-terrain-current.zh-TW.md` 第 11 段（11 張經典地圖、504 雙域）、`data-sources-0.6.60.json` 的 `map-zone`／`map-item`、`resource-database-0.6.63.md`、`NativeLitoZoneCatalog.java`、`db/candidates/resource-enrichment-0.6.60/enrich_stage2.py`（地圖頁圖片表的位址）。
- 脫殼 PE：`lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`（只讀 bytes，沒有執行）。
- tw-bin-engine：沒有用到。

## 給下一棒的提示

- **要加新的診斷來源**：在產生 receipt 的地方帶上 `error_kind`（`resource_diagnostics::classify(&error)`），再到 `collect` 加一段。`Ledger` 與 HUD 不用改。
- **要畫地圖頁**：`native_map_item::page_image(page)` 給圖片編號；圖在 `LiTo/LTsurf`，1536×768，要先處理 `decode_img` 的上限。畫出來之後把 `View::diagnostic()` 第 1、2 頁那一支拿掉。
- **SPR 空影格標記**：決定放寬方式後改 `sprite.rs` 的 `empty_sentinel`，同步改 `resource_matrix.py` 的 `SPR_EMPTY_SENTINELS` 並重產矩陣（CI 自測會擋）。
- **客戶端資源變動**：`map_domains.py --check` 會失敗，這是預期行為；重產後在 PR 說明差異。
- **`LTzone-c.tbl` 內容變了**：`native_zone.rs` 的測試內嵌了目前的 125 bytes，不會自動跟著變；以 `docs/MAP-DOMAINS.md` 的 SHA 為準重新核對。
- G07.4 的狀態建議由 `pending` 改為「世界畫面的診斷已接線，實機畫面待驗證」。
