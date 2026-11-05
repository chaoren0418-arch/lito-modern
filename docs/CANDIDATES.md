# CANDIDATES — 44 檔隔離候選盤點（T003 / Issue #2）

更新：2026-11-04。狀態分「已驗證／推測／待驗證」。

本檔只盤點，**不套用**候選。`維護工具/candidates/` 列在 `.gitignore`，此 clone **沒有**候選本體；不可從 `docs/HISTORY.md` 或腳本還原 bytes。

## 結論（已驗證）

最新隔離 overlay 是 **44 檔 = 30 Runtime + 11 Rust QA + 3 Java**，372 module、原六 nested path，`production_applied=False`，**不可套用**。全體 475 案未跑；#97（共享 callback／重入繪圖／native clock／資源就緒 gate）未完整。

建議：**整包視為單一 `effective_overlay_files`，不要單檔 cherry-pick。** Claude 已裁定（見「裁定」一節）：**現在不套用，凍結為參考**；T020（#97）改為 blocked on T010。

## 來源與重建方法

| 來源 | 角色 | 結果 |
|---|---|---|
| `維護工具/candidates/**` | 任務指定唯讀本體 | **不在 repo**（`.gitignore` 第 111 行） |
| `docs/HISTORY.md` 2026-10-25 首條 | 最新 ready-gate 計數 | 44 檔 30／11／3、不可套用、475 案未跑 |
| `維護工具/develop-native-E149-static2941-queue.py` L31–32 | 44 檔契約 | `require(len(files)==44 … overlay_count=44, runtime_files=30, Rust_QA_files=11, Java_files=3)` |
| `維護工具/develop-native-E149-static2941-ready.py` 與 `-v2.py` | 最新 schema `E149-static2941-ready-source-v1` | 覆寫 connection／static2941／login／login_window／world_scene／world_view；不增檔 |
| `維護工具/prepare-native-E149-*.py` 與 `develop-native-E149-*.py` | 繼承 `effective_overlay_files` 的插入鏈 | 下列 44 個 overlay key |
| `LiTo381-Modern/logs/continuation-20261003/` | 任務備援 | **不能**當 44 檔清單（2026-10-03，早於 overlay） |

候選本體不在此 clone。**取檔方式**（#41 已在使用者電腦上實測，見 `docs/reports/issue-41.md`）：**不能**用「ready 目錄＋overlay key」拼路徑，這樣 44 檔一個都找不到。正確作法是讀 contract `維護工具/candidates/native-E149-static2941-ready-20261024-210441-110315/contract.json`（SHA256 `A5EAFAB92F673DEEE4D1E8EC4EEE39085B3B3C87C24C565DED2F0ACFBF831A97`），逐 key 查 `effective_overlay_paths`。44 個實體檔分散在 25 個歷代候選目錄，只有 6 個在 ready 目錄裡。

HISTORY 指向的 ready 證據：`驗證紀錄/current/native-E149-static2941-ready-development/20261024-211350-252315/evidence.json`（不在 git）。

**差異行數**（#41，2026-11-05，使用者電腦上實算）：44 檔的候選 SHA256 都和 contract 的 `effective_overlay_files` 一致（44／44，0 缺檔）。
- 25 檔在 `src/` 或 `tests/` 有同名 predecessor：一覽表填的是 `git diff --no-index --numstat` 的 `+增／−刪`，合計 **+8,050／−3,250**。
- 19 檔沒有同名檔：填「新檔 N 行」，合計 **3,343 行**。
- 第 3、13、35、42 列的 predecessor 是 CRLF、候選是 LF，numstat 顯示整檔替換，數字**不代表**語意修改量。

**是否已併入**：所有契約 `production_applied=False`。正常來源若有同名檔，那是 **predecessor**，不是候選本體。`mod.rs` 目前只有 `native_class1` 在清單內的候選獨有模組中出現，且是正常來源既有模組，不是 overlay 套用。

## Overlay 成長（已驗證計數）

| 檔數 | Runtime | QA | Java | 插入點 | 腳本 |
|---:|---:|---:|---:|---|---|
| 13 | 6 | 5 | 2 | isolated `keyboard`＋`blind20`＋`cancel-recovery` | `check-native-development-types.py` |
| 14 | 6 | 5 | 3 | recovery bootstrap 加第三份 Java | `native_E149_recovery_contract.py`（11 Rust＋3 Java＝14） |
| 22 | 8 | 11 | 3 | 11 QA 固定；之後新 QA 改 append 進 runtime `#[cfg(test)]` | `prepare-native-E149-object374448.py` |
| 30 | 16 | 11 | 3 | zombie41／break27 | `prepare-native-E149-zombie41.py` `require(len(files)==30)` |
| 33 | 19 | 11 | 3 | eruption45-scene ＋3 | `prepare-native-E149-eruption45-scene.py` |
| 42 | 28 | 11 | 3 | mass69-fire70 ＋2；其後到 hide97 只覆寫 | qualify／HISTORY |
| 43 | 29 | 11 | 3 | ingress ＋`native_connection.rs` | HISTORY |
| **44** | **30** | **11** | **3** | queue ＋`native_static2941.rs` | `develop-native-E149-static2941-queue.py` |

非 overlay key（break27 只 pin 正常來源，不計入 44）：`compositor.rs`、`native_environment.rs`、`native_inventory.rs`、`native_object_dispatch.rs`。六 nested `#[path]` 是 overlay 檔內部 rewrite，不是獨立候選檔。

## 共用阻擋

每一列都成立：

1. **依賴** — overlay 是單一繼承 dict；#97 共享 callback／重入繪圖／native clock／資源就緒 gate 未資格。
2. **測試未跑** — 475 案；使用者暫緩測試。
3. **行為未驗** — 未套用、未連 PE、未載 SPR、未 GUI、未播放。

建議順序：**0＝整包或不套用**（預設）。1–44 僅在 Claude 決定分批時用，依歷史插入／依賴；#97 兩檔最後。

---

## 一覽表（44 列，每列可追溯）

候選實體路徑以 contract 的 `effective_overlay_paths[<overlay key>]` 為準，見上方「取檔方式」；每列的完整實體路徑與 SHA256 見 `docs/reports/issue-41.md`「44 列檔案結果」。對應 `src/`＝同一相對路徑（`tests/` 列亦然）。

### A. Runtime（30）

| 建議順序 | overlay key（候選路徑後綴） | 對應 src/ | 正常來源有同名檔 | 已併入 | 差異行數 | 阻擋原因 | 插入來源 |
|---:|---|---|---|---|---|---|---|
| 1 | `LiTo381-Modern/src/modern_lito/world_raster.rs` | 同左 | 有 | 否 | +64／−0 | 依賴；測試未跑；行為未驗 | isolated `keyboard`：`prepare-native-keyboard-target-candidate.py` |
| 2 | `LiTo381-Modern/src/modern_lito/native_world_scene.rs` | 同左 | 有 | 否 | +230／−19 | 依賴；測試未跑；行為未驗；#97 Scene／History 未接完 | isolated `keyboard` |
| 3 | `LiTo381-Modern/src/modern_lito/native_login_window.rs` | 同左 | 有 | 否 | +2840／−2512（CRLF→LF 整檔替換，非語意修改量） | 依賴；測試未跑；行為未驗；含 nested `native_rental_window.rs` | isolated `keyboard` |
| 4 | `LiTo381-Modern/src/modern_lito/native_skills.rs` | 同左 | 有 | 否 | +995／−7 | 依賴；測試未跑；行為未驗；技能 request 長尾 | isolated `blind20`：`prepare-native-blind20-candidate.py` |
| 5 | `LiTo381-Modern/src/modern_lito/native_skill_selection.rs` | 同左 | 有 | 否 | +93／−3 | 依賴；測試未跑；行為未驗 | isolated `blind20` |
| 6 | `LiTo381-Modern/src/modern_lito/native_login.rs` | 同左 | 有 | 否 | +290／−46 | 依賴；測試未跑；行為未驗；G06／#97 controller | isolated `cancel-recovery`：`prepare-native-teleport-cancel-idempotence.py` |
| 7 | `LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs` | 同左 | 有 | 否 | +229／−9 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-slow29-candidate.py` |
| 8 | `LiTo381-Modern/src/modern_lito/native_selector_effects.rs` | 同左 | 有 | 否 | +23／−2 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-advance79-candidate.py` |
| 9 | `LiTo381-Modern/src/modern_lito/light_arrow_effect.rs` | 同左 | 有 | 否 | +387／−15 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-area-fireball.py`（22→23） |
| 10 | `LiTo381-Modern/src/modern_lito/native_attack_many.rs` | 同左 | 有 | 否 | +585／−19 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-tornado53.py`（23→24） |
| 11 | `LiTo381-Modern/src/modern_lito/native_class1.rs` | 同左 | 有（`mod.rs` 已宣告正常模組） | 否 | +212／−13 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-class1-origin.py`（24→25） |
| 12 | `LiTo381-Modern/src/modern_lito/native_world.rs` | 同左 | 有 | 否 | +85／−4 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-area2230.py`（25→26） |
| 13 | `LiTo381-Modern/src/modern_lito/native_audio_delivery.rs` | 同左 | 有 | 否 | +315／−256（CRLF→LF 整檔替換，非語意修改量） | 依賴；測試未跑；行為未驗 | `prepare-native-E149-fireball25.py`（26→27） |
| 14 | `LiTo381-Modern/src/modern_lito/native_spell_projectiles.rs` | 同左 | 有 | 否 | +157／−95 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-S23-history.py`（27→28） |
| 15 | `LiTo381-Modern/src/modern_lito/native_world_view.rs` | 同左 | 有 | 否 | +63／−4 | 依賴；測試未跑；行為未驗；#97 view callback | `prepare-native-E149-S23-clock.py`（28→29） |
| 16 | `LiTo381-Modern/src/modern_lito/native_skill_effects.rs` | 同左 | 有 | 否 | +226／−10 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-break27-dark40.py`（29→30） |
| 17 | `LiTo381-Modern/src/modern_lito/native_eruption45.rs` | 同左（**src 無檔**；併入需新 `mod`） | 無 | 否 | 新檔 380 行 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-eruption45-scene.py`（30→33） |
| 18 | `LiTo381-Modern/src/modern_lito/native_eruption45_assets.rs` | 同左（**src 無檔**） | 無 | 否 | 新檔 393 行 | 依賴；測試未跑；行為未驗；候選獨有模組 | eruption45-scene |
| 19 | `LiTo381-Modern/src/modern_lito/native_eruption45_scene.rs` | 同左（**src 無檔**） | 無 | 否 | 新檔 373 行 | 依賴；測試未跑；行為未驗；候選獨有模組 | eruption45-scene |
| 20 | `LiTo381-Modern/src/modern_lito/tower_assets.rs` | 同左 | 有 | 否 | +136／−1 | 依賴；測試未跑；行為未驗；#97 Static2941Bank 掛這裡 | `prepare-native-E149-berserk55-disease56.py`；後 `develop-native-E149-hide97-static-assets.py` 覆寫 |
| 21 | `LiTo381-Modern/src/modern_lito/native_blizzard59.rs` | 同左（**src 無檔**） | 無 | 否 | 新檔 111 行 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-blizzard59.py`（34→35） |
| 22 | `LiTo381-Modern/src/modern_lito/native_resurrection61.rs` | 同左（**src 無檔**） | 無 | 否 | 新檔 76 行 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-resurrection61.py`（35→36） |
| 23 | `LiTo381-Modern/src/modern_lito/native_appearance27.rs` | 同左（**src 無檔**） | 無 | 否 | 新檔 184 行 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-appearance27-queue.py`（36→37） |
| 24 | `LiTo381-Modern/src/modern_lito/native_world_animation.rs` | 同左 | 有 | 否 | +81／−5 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-appearance27-binding.py`（37→39） |
| 25 | `LiTo381-Modern/src/modern_lito/native_combat_animation.rs` | 同左 | 有 | 否 | +6／−0 | 依賴；測試未跑；行為未驗 | appearance27-binding |
| 26 | `LiTo381-Modern/src/modern_lito/native_shape67.rs` | 同左（**src 無檔**） | 無 | 否 | 新檔 271 行 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-shape67-protection68.py`（39→40） |
| 27 | `LiTo381-Modern/src/modern_lito/native_mass69.rs` | 同左（**src 無檔**） | 無 | 否 | 新檔 236 行 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-mass69-fire70.py`（40→42） |
| 28 | `LiTo381-Modern/src/modern_lito/native_fire70.rs` | 同左（**src 無檔**） | 無 | 否 | 新檔 111 行 | 依賴；測試未跑；行為未驗；候選獨有模組 | mass69-fire70 |
| 29 | `LiTo381-Modern/src/modern_lito/native_connection.rs` | 同左 | 有 | 否 | +129／−0 | **#97 ingress clock**；依賴；測試未跑；行為未驗。分批時最後才碰 | `develop-native-E149-static-effect-ingress.py`（42→43） |
| 30 | `LiTo381-Modern/src/modern_lito/native_static2941.rs` | 同左（**src 無檔**；`mod.rs` 無此 mod） | 無 | 否 | 新檔 153 行 | **#97 專用 queue／ready-gate**；依賴；測試未跑；行為未驗。分批時最後 | `develop-native-E149-static2941-queue.py` L30（43→44） |

### B. Rust QA（11；object374448 之後不再增加 overlay key）

| 建議順序 | overlay key（候選路徑後綴） | 對應 src/ 或 tests/ | 正常來源有同名檔 | 已併入 | 差異行數 | 阻擋原因 | 插入來源 |
|---:|---|---|---|---|---|---|---|
| 31 | `LiTo381-Modern/tests/native_skill_selection.rs` | 同左 | 有 | 否 | +61／−0 | 依賴；測試未跑；行為未驗 | isolated `keyboard` |
| 32 | `LiTo381-Modern/tests/native_blind.rs` | 同左（**tests 無檔**） | 無 | 否 | 新檔 113 行 | 依賴；測試未跑；行為未驗；候選獨有 QA | isolated `blind20` |
| 33 | `LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs` | 同左 | 有 | 否 | +124／−0 | 依賴；測試未跑；行為未驗；G06 租裝返回 | isolated `cancel-recovery` |
| 34 | `LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs` | 同左 | 有 | 否 | +62／−0 | 依賴；測試未跑；行為未驗；G06 wire | `prepare-native-recovery-default-source.py`／component-matrix |
| 35 | `LiTo381-Modern/tests/native_teleport_failure.rs` | 同左 | 有 | 否 | +198／−134（CRLF→LF 整檔替換，非語意修改量） | 依賴；測試未跑；行為未驗；G06 失敗恢復 | `prepare-native-E149-recovery-bootstrap.py` |
| 36 | `LiTo381-Modern/tests/native_slow29.rs` | 同左（**tests 無檔**） | 無 | 否 | 新檔 89 行 | 依賴；測試未跑；行為未驗；候選獨有 QA | `prepare-native-E149-slow29-candidate.py` |
| 37 | `LiTo381-Modern/tests/native_advance79.rs` | 同左（**tests 無檔**） | 無 | 否 | 新檔 68 行 | 依賴；測試未跑；行為未驗；候選獨有 QA | `prepare-native-E149-advance79-candidate.py` |
| 38 | `LiTo381-Modern/tests/native_item_spells.rs` | 同左（**tests 無檔**） | 無 | 否 | 新檔 64 行 | 依賴；測試未跑；行為未驗；候選獨有 QA | `prepare-native-E149-item-spells-candidate.py` |
| 39 | `LiTo381-Modern/tests/native_magical73.rs` | 同左（**tests 無檔**） | 無 | 否 | 新檔 66 行 | 依賴；測試未跑；行為未驗；候選獨有 QA | `develop-native-E149-magical73.py`（19→20） |
| 40 | `LiTo381-Modern/tests/native_shadow107.rs` | 同左（**tests 無檔**） | 無 | 否 | 新檔 95 行 | 依賴；測試未跑；行為未驗；候選獨有 QA | `prepare-native-E149-shadow107.py`（20→21） |
| 41 | `LiTo381-Modern/tests/native_object374448.rs` | 同左（**tests 無檔**） | 無 | 否 | 新檔 105 行 | 依賴；測試未跑；行為未驗；候選獨有 QA；11 QA 於此固定 | `prepare-native-E149-object374448.py`（21→22） |

### C. Java（3；fixture／G06 oracle，不是客戶端 runtime）

| 建議順序 | overlay key（候選路徑後綴） | 對應 tests/oracle | 正常來源有同名檔 | 已併入 | 差異行數 | 阻擋原因 | 插入來源 |
|---:|---|---|---|---|---|---|---|
| 42 | `LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java` | 同左 | 有 | 否 | +459／−96（CRLF→LF 整檔替換，非語意修改量） | 依賴；測試未跑；行為未驗；Java fixture；不進 Rust 客戶端 | isolated／recovery overlay |
| 43 | `LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java` | 同左（**oracle 無檔**） | 無 | 否 | 新檔 247 行 | 依賴；測試未跑；行為未驗；候選獨有 Java | recovery overlay 13 檔內 |
| 44 | `LiTo381-Modern/tests/oracle/NativeRecoveryProductionBootstrap.java` | 同左（**oracle 無檔**） | 無 | 否 | 新檔 208 行 | 依賴；測試未跑；行為未驗；候選獨有 Java；14 檔 overlay 的 +1 | `prepare-native-E149-recovery-bootstrap.py` |

列數：30＋11＋3＝**44**。

## 建議併入策略（推測；交 Claude）

1. **預設不套用。** HISTORY／所有 contract：`production_applied=False`、`whole_goal_complete=False`。
2. 若分批：先 1–16（正常來源已有同名 runtime），再 17–28（候選獨有技能模組＋`mod.rs`），**29–30（#97）最後**，且等 T020 與測試恢復。QA／Java 隨對應 runtime，不要先合測試。
3. 不要把 `compositor.rs` 等四個 pin 當成第 45–48 檔。
4. 使用者未同意前不跑 475 案、不連正式服務／DB、不啟動遊戲。

## 裁定（Claude 主控，2026-11-05，Issue #33）

**裁定：現在不套用，凍結為參考。不整包套用，不分批套用，也不單檔 cherry-pick。** 候選目錄維持在使用者電腦上原樣唯讀保存，不刪除、不移動。這個裁定隨時可以重新評估，沒有不可逆的成分。

### 理由

1. **違反 PLAN M1「停止疊加，先驗證」**：44 檔建立在 475 個沒跑過的測試案、640 次沒跑過的 G06 上。PLAN 明定 M1 之前「不接新技能」，而 44 檔裡有 9 個是候選獨有的技能模組（第 17–19、21–23、26–28 列）。
2. **沒有任何驗證手段**：使用者指示測試暫緩，套用後只能靠 `cargo check`，475 案與 GUI 都不能跑；這等於把未驗證的程式碼搬進 `main`，違反 CLAUDE.md 規則 5（不以未驗證冒充完成）。
3. **#97 本身沒有完成**：第 29、30 列（`native_connection.rs` ingress clock、`native_static2941.rs` queue／ready-gate）就是 #97 未完成的部分，其他列又透過繼承鏈依賴它，所以不存在「先套安全的那批」的乾淨切點。
4. **repo 內看不到候選本體**：沒有 bytes、SHA、diff，無法審查實際內容。在看不到 diff 的情況下批准套用，不是審查。

### 什麼時候重新評估

- **條件**：使用者已於 2026-11-05 同意恢復測試（#43）。還要等 M1 的 12 個 gate 案與 G06 有結果（T010／T011，本機執行中）。
- **靜態準備已完成**（T003-R1，#41，報告 `docs/reports/issue-41.md`）：
  1. 取檔方式已更正，見上方「來源與重建方法」。
  2. 44 檔的 SHA256 與 diff 統計已補進一覽表。
  3. 依賴分析：44 檔之間共有 **497 條引用、171 組不同的檔案依賴**。引用涵蓋 `super::`、`crate::`、群組 use、`mod`、`#[path]`，以及 cfg(test)。
- **依賴圖的結論**（#41「依賴圖與研究分組」一節；來源層級分析，未經編譯器驗證）：
  - **跨組雙向依賴**：以「不同檔案對」計算——G1→G2 26 組、G1→G3 8 組、G2→G1 50 組、G3→G1 4 組、G3→G2 2 組。
  - **23 檔強連通分量**：第 2、4、5、6、10、11、12、13、14、16、19、21–30、33、34 列互相引用、構成循環，其中包含 #97 的第 29、30 列。另有一個 3 檔強連通分量（第 7、8、20 列）。
  - **直接依賴 #97（G3）的 G1／G2 檔**：第 2、3、6、12、15、35 列。
- **因此裁定補充**：在不改寫的前提下，**不存在可以逐批套用的切法**。23 檔強連通分量只能整塊進來，而它又包含不會照原樣套用的 #97 兩檔。未來若要利用這些候選，可行做法只有兩種：
  - 以 `src/` 上新寫的 #97 為基礎，**把候選逐檔改寫**成依賴新實作的版本，每檔一個 PR，各自通過 cargo check／test；
  - 或者只把候選當設計參考，在 `src/` 上重新實作。

  這兩種做法都要等 M1 結果與 T020（#97）完成後再決定。原本的 G1／G2／G3 只是研究用的編號，不代表順序。

### 對 T020（#97）的影響

T020 不再卡在 T003，改為卡在 **T010**：使用者已同意恢復測試（#43），M1 由本機執行中。PLAN 把 #97 放在 M2，M2 在 M1 之後。到時候 #97 要在 `src/` 上直接開發，不以候選第 29–30 列為基底；那兩檔只當作設計參考。

## 未驗證

- ~~候選 bytes、SHA、與 `src/` 的 diff~~：已由 #41 在使用者電腦上實算（44／44 SHA256 一致，numstat 見一覽表）。候選內容本身仍不在 repo。
- ~~最新 `<stamp>` 目錄名~~：已確認 ready 目錄為 `native-E149-static2941-ready-20261024-210441-110315`（#41）。
- 依賴圖是來源層級分析：沒有展開巨集，也沒有驗證 cfg 可達性與名稱解析，非 overlay 模組的遞迴依賴也未分析（#41「推測／分析界線」）。
- 六 nested path 的完整 resolved 清單（只驗證 rewrite 次數＝6）。
- overlay 與當版 `src/` 行為是否等價（推測：否，且未跑測試）。
- `continuation-20261003` 與 44 檔的對應（已排除為來源）。
