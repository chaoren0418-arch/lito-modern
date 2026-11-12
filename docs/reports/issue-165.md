# Issue #165 — T028-3 地面、特效與資源類當版補驗

範圍：技能 63（生命之泉 320 秒）、72（強力無所遁形術 3934 空影格）、77（究極光裂術 Disintegrate 舊來源路徑），以及 list.spr 全量嚴格 parser 第 614 行錯誤。工作分支 `task/165`（基底 origin/main `ce85b82`）。依使用者 2026-11-09 規定，本機**沒有執行任何編譯產物或測試二進位**；只做 `cargo check`、唯讀資源擷取（PakViewer CLI）、靜態逆向（IDA 反組譯文字檔、`javap`）與 Python 唯讀分析。沒有連線正式服務或 DB，也沒有啟停遊戲程序。

## 做了什麼

### 1. list.spr 第 614 行嚴格 parser 錯誤（`sprite_actions.rs`）

- 用 PakViewer CLI 唯讀擷取 `Text.idx/Text.pak` 的 `list.spr`：4,067,417 bytes，SHA256 `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127`（和既有 `LIST_SHA` 一致）。
- 第 614 行是 `#266 40=183 orim` 的 `2.damage(1 2,0:0:4 24.0:6[79)`：影像和子影格之間用了 `:`，不是 `.`。
- 原生證據：`lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm`（SHA `2A3A6E10…`，即 HISTORY 記載的 4271D92F 原 ASM）。
  - `sub_43DCC0` 先略過所有非數字字元（只用 `-` 記錄正負號），再讀十進位數字。
  - `sub_43DE10` 的影格迴圈 `loc_43E84D` 連續呼叫 3 次 `sub_43DCC0`，依序讀 image、subframe、ticks。
  - 所以原客戶端把 `0:0:4` 讀成 image 0／subframe 0／ticks 4，和 `0.0:4` 相同。
- 修正：只在 image／subframe 的位置多接受 `:`，和既有的 `,` 例外並列。ticks 前的分隔符仍然只接受 `:`；`-1.0:4`、`0:3.2`、`40.10.:4`、`0::0:4`、`0;0:4` 仍然拒絕（inspection 模式下維持隔離）。
- 這個放寬只影響原本會失敗的輸入，原本能解析的定義結果不變。
- 用 Python 唯讀移植 parser（`scratch/issue-165/spr_port.py`，不追蹤）盤點整個檔案：
  - 修正前：嚴格模式停在第 614 行，inspection 隔離 133 個定義，另有 1 個 header 問題。移植版重現了同一行同一錯誤，可以佐證移植正確。
  - 修正後：9 個定義解除隔離（#266／#901 的 damage，#1245／#1590／#1697／#2001 等的 death），隔離數降到 124。嚴格模式的下一個阻擋點是**第 2419 行** `#1760 Lord British 15.Get(1 3,-1.0:4 …)`，屬於有號影像 ID，語意還沒驗證，本張不放寬。
- 測試：
  - 新增 `native_43dcc0_colon_image_subframe_separator_matches_dot_and_stays_narrow`（lib，CI 可跑）。
  - 既有 `inspection_retains_unverified_body_and_ref_cannot_bypass_quarantine` 原本拿 `0:0:4` 當隔離範例，改用仍未驗證的 `-1.0:4`，並加上 `0:0:4` 經由 reference 解析成功的斷言。
  - ignored 實資源測試 `real_shared_list_actions_are_bounded_and_resolvable` 的期望從 614 改成 2419，並核對 #266 action2 的影格是 `(0,0,4)`、`(24,0,6)`，事件是 Sound(79)。

### 2. 技能 77 Disintegrate 舊來源路徑（`battle_spells.rs`、`disintegrate_presentation.rs`）

- 現行客戶端 `D:\lineage開發\0-客戶端` 的根目錄**沒有** `TW13081901.pak`，只在 `launcher\TW13081901.pak`（`Test-Path` 已確認）。
- 原生路徑 `native_disintegrate.rs` 早已改用 `DisintegrateAssets::load_current_tw`，但戰鬥 HUD（`battle_spells.rs:206`）和世界 presentation（`disintegrate_presentation.rs:387`）仍呼叫舊的根目錄 `DisintegrateAssets::load(root)`。這就是 HISTORY L1425「缺 root TW catalog」失敗的原因。
- 修正：兩個正式呼叫端都改走 `load_current_tw`。這個載入器使用精確的 pack／text SHA、解碼長度與定義數 profile，不加舊根目錄 fallback。`load` 本身保留，只供既有的舊 profile 測試使用。

### 3. 技能 72 的 3934 空影格（`tower_assets_selector_blank_tests.rs`，新檔）

- main 已有專屬政策 `REVIEWED_SELECTOR_EMPTY_SPRS`（3934-0.spr，1,168,552 bytes，SHA，left 352，只限 slot 0／12），也就是 HISTORY L1584 要求的「專屬 SHA／slots policy」，但之前沒有任何直接測試。
- 用 PakViewer 唯讀擷取 `3934-0.spr`：Sprite03（依名稱 shard）找不到，改從 `Sprite.idx` 取得。1,168,552 bytes，SHA256 `97183EBE2307D1A7F5D244459AE7B8434072544DC800C82D94B4B1C441F03753`，和政策常數一致。
- list.spr #3934 `0.effect(0 13,0.12:4 0.0:4 0.1:4<1355 …0.11:7)`：source0 是 subframe 12，source1 是 subframe 0，兩者都是空槽；合計 61 ticks，<1355 在 source2。
- 新增 3 個 lib 測試（CI 可跑，只用合成影格）：
  - 常數 pin。
  - 只有 slot 0／12 且 metadata 完全相同時才算空槽；其他檔名、3936 的 left、8 種欄位變造（offset、尺寸、unknown1／2、rgba、fragment reference）都拒絕。
  - 來源資格會拒絕錯誤大小、錯誤 SHA、未知檔名與錯誤影格數。
- 另外新增 1 個 ignored 實資源測試：`actual_counter3934_bank_keeps_blank_dwell_slots_and_one_audio_cue`。

### 4. 技能 63 的 320 秒（`tests/native_t028_ground.rs`，新檔）

- E149 來源：以 `javap -c` 唯讀讀取 `formal-server-E1498871.jar`（SHA `E1498871C7BE…D498`）的 `BattleSpellCatalog.<clinit>`，得到 `GroundSpell(63,7,6,19,2231,-1,20,10,320,0,3,1,40318L,1,"life-stream")`。逐項和既有 BBC59308 fixture 相同：row7／col6、action19、gfx2231、MP20、lease 320 秒、pulse 0。所以 captured wire 仍然可以當現版 oracle。
- 新增 3 個整合測試：
  - E149 參數 ↔ captured 請求、施法、特效、MP 封包的對照（`no`，不需要共享資源）。
  - 兩個重疊的 S53 是兩個獨立的伺服器事件：客戶端沒有 lease 狀態，不推算 HP／MP，也不在本地阻擋再施放（`no`）。
  - 兩次播放各自在 76 ticks 退休、各響一次 841；在 319.999 秒、320 秒、322 秒都不會重播、移除或產生 pulse（`yes`，需要共享資源）。
- 這 3 個測試已加入 `維護工具/test-whitelist/tests.tsv`。

## 已驗證

- `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`（在 `LiTo381-Modern/` 執行）**結束碼 0**，原文結尾 ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 27s``。變更檔案沒有新增 warning。日誌在 `scratch/issue-165/logs/check.log`（不追蹤）。
  - 第一次 check 因新測試的 `forged[forged.len()/2]` 借用錯誤而結束碼 101，修正後重跑。
- 靜態證據：上述 SHA、反組譯片段（`scratch/issue-165/sub_43DE10.asm`）、javap 輸出（`scratch/issue-165/BattleSpellCatalog.javap.txt`）、Python 盤點結果。
- `cargo check` 不產生 `test result:`。本機沒有跑測試，所以本節**沒有任何 `test result:`**。

## 推測

- 9 個解除隔離的 damage／death 動作（orim 等怪物）現在會依原定義播放，和原客戶端一致；這個推論來自 43DCC0 的靜態語意，畫面尚未目視。
- 戰鬥與世界 presentation 的 77 改走 `launcher/` 後，在現行客戶端應該能載入成功（原生路徑的同一個載入器已通過既有 opt-in）。

## 待驗證／未驗證什麼

- **CI**：push 後由 PR 的 GitHub Actions 跑 `cargo check --all-targets` 與 `cargo test --lib`。這會涵蓋 `native_43dcc0_colon…`、`inspection_retains…`、3 個 `counter3934_*`；結果以 PR CI 為準。
- **交給 Codex 本機**（需要共享資源，CI 沒有）：
  - `LITO_SHARED_RESOURCE_ROOT=D:\lineage開發\0-客戶端` 執行 `維護工具/run-local-test-suite.ps1`，其中包含本張 3 個 `native_t028_ground` whitelist 測試。
  - 用 `--lib … -- --ignored --exact` 個別執行：
    - `modern_lito::sprite_actions::tests::real_shared_list_actions_are_bounded_and_resolvable`
    - `modern_lito::tower_assets::selector_blank_tests::actual_counter3934_bank_keeps_blank_dwell_slots_and_one_audio_cue`
    - `modern_lito::disintegrate_assets::tests::all_four_shared_sprites_match_independent_csharp_pixels_current_launcher_tw_profile`
    - `real_disintegrate_layers_share_world_depth_and_survive_target_removal_current_launcher_tw_profile`（`world_scene` 模組 tests 內）
    - `shared_one_shots_match_csharp_composites_and_release_owned_assets_current_launcher_tw_profile`（`disintegrate_effect` 模組 tests 內）
    - 77 的戰鬥／世界 presentation 改用現版 profile 後，需要另由 Codex 以現行客戶端資源執行一次實際載入（例如上述 world_scene 測試），才能確認 HISTORY L1425 的「缺 root TW catalog」失敗已消失
  - 執行既有的 `native_reagent_skills`、`native_selector_only`。
  - `parse_for_inspection` 的結果改變後，請 Codex 跑完整的既有實資源回歸，確認沒有任何測試依賴「orim 等 9 個動作被隔離」。本機用 grep 沒有找到這類依賴。
- list.spr 全量嚴格解析**仍然失敗**，下一個阻擋點在第 2419 行。剩餘 124 個隔離定義的分布（Python 盤點）：
  - 45 個 `-1.` 有號影像
  - 25 個未知影格修飾字
  - 17 個巢狀 `(`
  - 11 個缺逗號
  - 8＋6＋6 個括號異常
  - 5 個其他分隔符
  - 1 個命令分隔
  - 另有 1 個 header（#5288，line 19766）

  每一類都需要原生語意證據才能放寬，本張沒有處理。
- E149 真實施法、GUI、實體音效、伺服器 lease 到期與回血都沒有驗證；本張沒有啟動客戶端或伺服器。
- 不將 T028 或 M2 標為完成。

## 產出路徑

- 追蹤：
  - `LiTo381-Modern/src/modern_lito/sprite_actions.rs`
  - `LiTo381-Modern/src/modern_lito/battle_spells.rs`
  - `LiTo381-Modern/src/modern_lito/disintegrate_presentation.rs`
  - `LiTo381-Modern/src/modern_lito/tower_assets.rs`（只加測試 mod 宣告）
  - `LiTo381-Modern/src/modern_lito/tower_assets_selector_blank_tests.rs`
  - `LiTo381-Modern/tests/native_t028_ground.rs`
  - `維護工具/test-whitelist/tests.tsv`
  - 本報告
- 不追蹤（`scratch/issue-165/`）：`list.spr`、`3934-0.spr`（唯讀擷取）、`spr_port.py`、`sub_43DE10.asm`、`BattleSpellCatalog.javap.txt`、`logs/check.log`。

### PakViewer 查詢清單（唯讀，`publish\win-x64\PakViewer.exe -cli export`）

| idx | 檔名 | 結果 |
|---|---|---|
| `Text.idx` | `list.spr` | 找到，4,067,417 bytes，L1，SHA `523C61AD…` |
| `Sprite03.idx` | `3934-0.spr` | 不在此 shard（File not found） |
| `Sprite.idx` | `3934-0.spr` | 找到，1,168,552 bytes，SHA `97183EBE…` |

另以 `Test-Path` 確認 `0-客戶端\TW13081901.pak` 不存在、`0-客戶端\launcher\TW13081901.pak` 存在。

## 給下一棒的提示

- Claude 審查時請注意：`:` 的放寬是**全域** parser 變更（list.spr 與 TW 目錄共用）。依據是同一個原生 43DCC0 讀取器，而且只放寬 image／subframe 這一個位置。TW 文字第 1370 行也有同一筆 orim，但 TW 目錄只解析選定 ID，不受影響。
- 第 4 張（S23／冰箭時鐘、class1 起點）修改時鐘後，請重跑本張的 63 時序測試。
- 若要繼續推進全量嚴格 parser，下一步是 `-1.`（45 處）：需要先從 43DE10 追出 image `-1`（存成 `ax ^ 0x1951` 的 word）在繪製端的語意，再決定 `ActionFrame.image_id` 的型別。
