# CANDIDATES — 44 檔隔離候選盤點（T003 / Issue #2）

更新：2026-11-04。狀態分「已驗證／推測／待驗證」。

本檔只盤點，**不套用**候選。`維護工具/candidates/` 列在 `.gitignore`，此 clone **沒有**候選本體；不可從 `docs/HISTORY.md` 或腳本還原 bytes。

## 結論（已驗證）

最新隔離 overlay 是 **44 檔 = 30 Runtime + 11 Rust QA + 3 Java**，372 module、原六 nested path，`production_applied=False`，**不可套用**。全體 475 案未跑；#97（共享 callback／重入繪圖／native clock／資源就緒 gate）未完整。

建議：**整包視為單一 `effective_overlay_files`，不要單檔 cherry-pick。** Claude 審閱後再定併入順序；恢復測試前不套用。T020（#97）blocked on T003。

## 來源與重建方法

| 來源 | 角色 | 結果 |
|---|---|---|
| `維護工具/candidates/**` | 任務指定唯讀本體 | **不在 repo**（`.gitignore` 第 111 行） |
| `docs/HISTORY.md` 2026-10-25 首條 | 最新 ready-gate 計數 | 44 檔 30／11／3、不可套用、475 案未跑 |
| `維護工具/develop-native-E149-static2941-queue.py` L31–32 | 44 檔契約 | `require(len(files)==44 … overlay_count=44, runtime_files=30, Rust_QA_files=11, Java_files=3)` |
| `維護工具/develop-native-E149-static2941-ready.py` 與 `-v2.py` | 最新 schema `E149-static2941-ready-source-v1` | 覆寫 connection／static2941／login／login_window／world_scene／world_view；不增檔 |
| `維護工具/prepare-native-E149-*.py` 與 `develop-native-E149-*.py` | 繼承 `effective_overlay_files` 的插入鏈 | 下列 44 個 overlay key |
| `LiTo381-Modern/logs/continuation-20261003/` | 任務備援 | **不能**當 44 檔清單（2026-10-03，早於 overlay） |

最新候選樹（本體不在此 clone；stamp 在 gitignored `驗證紀錄/`）：

```
維護工具/candidates/native-E149-static2941-ready-<stamp>/<overlay-key>
```

HISTORY 指向的 ready 證據：`驗證紀錄/current/native-E149-static2941-ready-development/20261024-211350-252315/evidence.json`（不在 git）。

**差異行數**：候選 bytes 不在 repo，無法對 `src/` 做 diff。欄位一律 `無法計算`。

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

候選路徑＝`維護工具/candidates/native-E149-static2941-ready-<stamp>/`＋「overlay key」。對應 `src/`＝同一相對路徑（`tests/` 列亦然）。

### A. Runtime（30）

| 建議順序 | overlay key（候選路徑後綴） | 對應 src/ | 正常來源有同名檔 | 已併入 | 差異行數 | 阻擋原因 | 插入來源 |
|---:|---|---|---|---|---|---|---|
| 1 | `LiTo381-Modern/src/modern_lito/world_raster.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | isolated `keyboard`：`prepare-native-keyboard-target-candidate.py` |
| 2 | `LiTo381-Modern/src/modern_lito/native_world_scene.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；#97 Scene／History 未接完 | isolated `keyboard` |
| 3 | `LiTo381-Modern/src/modern_lito/native_login_window.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；含 nested `native_rental_window.rs` | isolated `keyboard` |
| 4 | `LiTo381-Modern/src/modern_lito/native_skills.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；技能 request 長尾 | isolated `blind20`：`prepare-native-blind20-candidate.py` |
| 5 | `LiTo381-Modern/src/modern_lito/native_skill_selection.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | isolated `blind20` |
| 6 | `LiTo381-Modern/src/modern_lito/native_login.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；G06／#97 controller | isolated `cancel-recovery`：`prepare-native-teleport-cancel-idempotence.py` |
| 7 | `LiTo381-Modern/src/modern_lito/native_additional_finite_effects.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-slow29-candidate.py` |
| 8 | `LiTo381-Modern/src/modern_lito/native_selector_effects.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-advance79-candidate.py` |
| 9 | `LiTo381-Modern/src/modern_lito/light_arrow_effect.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-area-fireball.py`（22→23） |
| 10 | `LiTo381-Modern/src/modern_lito/native_attack_many.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-tornado53.py`（23→24） |
| 11 | `LiTo381-Modern/src/modern_lito/native_class1.rs` | 同左 | 有（`mod.rs` 已宣告正常模組） | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-class1-origin.py`（24→25） |
| 12 | `LiTo381-Modern/src/modern_lito/native_world.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-area2230.py`（25→26） |
| 13 | `LiTo381-Modern/src/modern_lito/native_audio_delivery.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-fireball25.py`（26→27） |
| 14 | `LiTo381-Modern/src/modern_lito/native_spell_projectiles.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-S23-history.py`（27→28） |
| 15 | `LiTo381-Modern/src/modern_lito/native_world_view.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；#97 view callback | `prepare-native-E149-S23-clock.py`（28→29） |
| 16 | `LiTo381-Modern/src/modern_lito/native_skill_effects.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-break27-dark40.py`（29→30） |
| 17 | `LiTo381-Modern/src/modern_lito/native_eruption45.rs` | 同左（**src 無檔**；併入需新 `mod`） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-eruption45-scene.py`（30→33） |
| 18 | `LiTo381-Modern/src/modern_lito/native_eruption45_assets.rs` | 同左（**src 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有模組 | eruption45-scene |
| 19 | `LiTo381-Modern/src/modern_lito/native_eruption45_scene.rs` | 同左（**src 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有模組 | eruption45-scene |
| 20 | `LiTo381-Modern/src/modern_lito/tower_assets.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；#97 Static2941Bank 掛這裡 | `prepare-native-E149-berserk55-disease56.py`；後 `develop-native-E149-hide97-static-assets.py` 覆寫 |
| 21 | `LiTo381-Modern/src/modern_lito/native_blizzard59.rs` | 同左（**src 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-blizzard59.py`（34→35） |
| 22 | `LiTo381-Modern/src/modern_lito/native_resurrection61.rs` | 同左（**src 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-resurrection61.py`（35→36） |
| 23 | `LiTo381-Modern/src/modern_lito/native_appearance27.rs` | 同左（**src 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-appearance27-queue.py`（36→37） |
| 24 | `LiTo381-Modern/src/modern_lito/native_world_animation.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | `prepare-native-E149-appearance27-binding.py`（37→39） |
| 25 | `LiTo381-Modern/src/modern_lito/native_combat_animation.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | appearance27-binding |
| 26 | `LiTo381-Modern/src/modern_lito/native_shape67.rs` | 同左（**src 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-shape67-protection68.py`（39→40） |
| 27 | `LiTo381-Modern/src/modern_lito/native_mass69.rs` | 同左（**src 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有模組 | `prepare-native-E149-mass69-fire70.py`（40→42） |
| 28 | `LiTo381-Modern/src/modern_lito/native_fire70.rs` | 同左（**src 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有模組 | mass69-fire70 |
| 29 | `LiTo381-Modern/src/modern_lito/native_connection.rs` | 同左 | 有 | 否 | 無法計算 | **#97 ingress clock**；依賴；測試未跑；行為未驗。分批時最後才碰 | `develop-native-E149-static-effect-ingress.py`（42→43） |
| 30 | `LiTo381-Modern/src/modern_lito/native_static2941.rs` | 同左（**src 無檔**；`mod.rs` 無此 mod） | 無 | 否 | 無法計算 | **#97 專用 queue／ready-gate**；依賴；測試未跑；行為未驗。分批時最後 | `develop-native-E149-static2941-queue.py` L30（43→44） |

### B. Rust QA（11；object374448 之後不再增加 overlay key）

| 建議順序 | overlay key（候選路徑後綴） | 對應 src/ 或 tests/ | 正常來源有同名檔 | 已併入 | 差異行數 | 阻擋原因 | 插入來源 |
|---:|---|---|---|---|---|---|---|
| 31 | `LiTo381-Modern/tests/native_skill_selection.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗 | isolated `keyboard` |
| 32 | `LiTo381-Modern/tests/native_blind.rs` | 同左（**tests 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有 QA | isolated `blind20` |
| 33 | `LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；G06 租裝返回 | isolated `cancel-recovery` |
| 34 | `LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；G06 wire | `prepare-native-recovery-default-source.py`／component-matrix |
| 35 | `LiTo381-Modern/tests/native_teleport_failure.rs` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；G06 失敗恢復 | `prepare-native-E149-recovery-bootstrap.py` |
| 36 | `LiTo381-Modern/tests/native_slow29.rs` | 同左（**tests 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有 QA | `prepare-native-E149-slow29-candidate.py` |
| 37 | `LiTo381-Modern/tests/native_advance79.rs` | 同左（**tests 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有 QA | `prepare-native-E149-advance79-candidate.py` |
| 38 | `LiTo381-Modern/tests/native_item_spells.rs` | 同左（**tests 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有 QA | `prepare-native-E149-item-spells-candidate.py` |
| 39 | `LiTo381-Modern/tests/native_magical73.rs` | 同左（**tests 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有 QA | `develop-native-E149-magical73.py`（19→20） |
| 40 | `LiTo381-Modern/tests/native_shadow107.rs` | 同左（**tests 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有 QA | `prepare-native-E149-shadow107.py`（20→21） |
| 41 | `LiTo381-Modern/tests/native_object374448.rs` | 同左（**tests 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有 QA；11 QA 於此固定 | `prepare-native-E149-object374448.py`（21→22） |

### C. Java（3；fixture／G06 oracle，不是客戶端 runtime）

| 建議順序 | overlay key（候選路徑後綴） | 對應 tests/oracle | 正常來源有同名檔 | 已併入 | 差異行數 | 阻擋原因 | 插入來源 |
|---:|---|---|---|---|---|---|---|
| 42 | `LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java` | 同左 | 有 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；Java fixture；不進 Rust 客戶端 | isolated／recovery overlay |
| 43 | `LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java` | 同左（**oracle 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有 Java | recovery overlay 13 檔內 |
| 44 | `LiTo381-Modern/tests/oracle/NativeRecoveryProductionBootstrap.java` | 同左（**oracle 無檔**） | 無 | 否 | 無法計算 | 依賴；測試未跑；行為未驗；候選獨有 Java；14 檔 overlay 的 +1 | `prepare-native-E149-recovery-bootstrap.py` |

列數：30＋11＋3＝**44**。

## 建議併入策略（推測；交 Claude）

1. **預設不套用。** HISTORY／所有 contract：`production_applied=False`、`whole_goal_complete=False`。
2. 若分批：先 1–16（正常來源已有同名 runtime），再 17–28（候選獨有技能模組＋`mod.rs`），**29–30（#97）最後**，且等 T020 與測試恢復。QA／Java 隨對應 runtime，不要先合測試。
3. 不要把 `compositor.rs` 等四個 pin 當成第 45–48 檔。
4. 使用者未同意前不跑 475 案、不連正式服務／DB、不啟動遊戲。

## 未驗證

- 候選 bytes、SHA、與 `src/` 的實際 diff（本體不在 repo）。
- 最新 `<stamp>` 目錄名（證據在 gitignored `驗證紀錄/`）。
- 六 nested path 的完整 resolved 清單（只驗證 rewrite 次數＝6）。
- overlay 與當版 `src/` 行為是否等價（推測：否，且未跑測試）。
- `continuation-20261003` 與 44 檔的對應（已排除為來源）。
