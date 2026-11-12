# Issue #183 — T046 第 3／3 張：HPBAR（只給任務／限時怪物，0xFF 隱藏）

日期：2026-11-12（第 1 次嘗試）。分支：`task/183`（起點 `7bbca43`，origin/main）。

**結論：新增 `native_hp_meter.rs`。`S_HIT_RATIO`（0x68＝104）改為接受 0..=100 與 0xFF（hide255），其他值照舊整包拒絕。世界畫面依伺服器給的比例在 actor 頭上畫 HP 條：0..=100 顯示，0xFF 隱藏。「只給任務／限時怪物」由伺服器決定誰拿到比例，客戶端不依怪物種類、死亡或距離自行推算。`cargo check --all-targets` 結束碼 0；新增 9 個 `--lib` 單元測試，依規定本機不執行，交給 PR CI。**

## 做了什麼

### 1. 伺服器來源（唯讀參考 lito-launcher-server，`D:\lineage開發\3-lito開發\lito伺服器\LiTo381-Server`）

| 來源 | 內容 | 客戶端結論 |
|---|---|---|
| `src/main/java/local/lito381/server/Protocol.java` `questMonster`（約 L15735）、`timeAttackActor`（約 L15760） | S_PUT_OBJECT 的 hp 欄位帶 `healthRatio(hp,maxHp)`（0..100） | 任務怪、限時怪一出現就有比例 → 顯示 |
| 同檔 `guardianTower` | 守護塔也帶比例 | 依值顯示（伺服器給的） |
| 同檔 `classicWorldMonster`（約 L15347） | 普通世界怪物帶 `c(0xFF)` | 隱藏，直到伺服器送 0x68 |
| 同檔 `classicPet` | 非主人視角的寵物帶 `0xFF` | 隱藏 |
| `ClientSession.java` L11356、L12085 | 自己 `actor == this ? 0xFF` | 自己隱藏；fixture 自己的 hp 位元組也是 0xFF（已用測試鎖定） |
| `Protocol.hitRatio`（約 L16049） | `68 d(objectId) c(ratio)`，E149 只送 0..100 | 0..100 照收 |
| `tools/combat-continuation/integration-lifecycle/NativeSourceMonsterHp.java` | 候選版：只對 kind 1／2（BattleQuestEncounter／BattleTimeAttackEncounter）的施法者送比例；同一 viewer 比例沒變就不重送；怪物死亡時送 `c(255)` 隱藏；由 `settings().hpbar()` 開關 | 0xFF＝隱藏；之後伺服器若再送比例，條會再出現 |

### 2. 程式

- `LiTo381-Modern/src/modern_lito/native_hp_meter.rs`（新增）
  - `receive(ratio)`：0..=100 或 0xFF 通過，101..=254 回錯（沒有任何伺服器版本會送這些值，維持原本的嚴格拒絕）。
  - `visible_ratio`、`fill_width`（比例 > 0 時至少 1px）。
  - `layout(world, hits)`：只處理本幀真的畫出、`actor_visible` 的 actor。條放在身體上緣上方；上緣超出畫布時改放下緣下方（與名稱標籤同規則）；左右夾在 800px 畫布內。
  - `draw(canvas, bars)`：40×5（含 1px 外框）的 RGBA 條，回傳 receipt（`bars`、`hidden_value`、`native_geometry_complete: false`）。
- `native_world.rs` 0x68：原本 `ratio <= 100` 才收，現在改用 `native_hp_meter::receive`，0xFF 存進 `Actor::hp_ratio`。失敗時整包不套用，`visual_revision` 不變。
- `native_world_scene.rs`：在 lighting 之後、名稱標籤之前畫 HP 條（名稱蓋在條上面）；frame receipt 新增 `hp_meter`。
- `mod.rs`：註冊 `pub mod native_hp_meter;`。
- `native_combat::target_label` 不用改：hp > 100（含 0xFF）原本就顯示「HP 未知」。

### 3. 測試（`native_hp_meter_tests.rs`，`--lib`，CI 會執行，不需要資源）

1. `receive_accepts_ratio_and_hide255_only`：0..=100、0xFF 通過；101..=254 全部拒絕。
2. `own_fixture_actor_is_hidden_by_server_value`：fixture 自己的 hp 是 0xFF，不出條。
3. `quest_monster_meter_updates_and_hide255_hides`：PUT 帶 100 → 條滿；0x68=40 → 條 40%、`visual_revision` 增加；0x68=255 → 隱藏；0x68=0 → 再出現（0px 填色）。
4. `ordinary_monster_put_0xff_stays_hidden_until_server_sends_ratio`。
5. `out_of_domain_ratio_is_rejected_atomically`：101／200／254、id 0、多一個 byte 都拒絕，值與 revision 不變。
6. `unknown_actor_meter_is_an_orphan_not_a_bar`。
7. `layout_places_bar_above_body_or_below_at_top_edge_and_clamps`。
8. `fill_width_is_proportional_and_nonzero_for_any_life`。
9. `draw_paints_frame_fill_and_empty_pixels`：外框、填色、空白、條外像素逐點檢查，以及畫布邊緣裁切不報錯。

## 已驗證

| 檢查 | 結束碼 | 結果 |
|---|---:|---|
| `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 4m 38s`；`native_hp_meter*.rs`、`native_world.rs`、`native_world_scene.rs` 沒有警告（log：`scratch/check-183.log`，不進 git） |

- 伺服器端誰拿到比例、誰拿到 0xFF：已逐一讀原始碼確認（見上表）。
- 本張沒有宣稱任何資源缺失；HP 條是程式畫的色塊，不讀 pak／sprite，所以沒有查 pak。

## 推測

- **E149 與候選版的 0x68 hide255 有衝突**：E149 `Protocol.hitRatio` 的註解寫著「原生 receiver 會把大於 100 的值夾成 100，存進 world object +0x51；0xFF 隱藏值只屬於 S_PUT_OBJECT 尾段，經 0x68 送會變成 100」。候選版 `NativeSourceMonsterHp.hide` 卻用 0x68 送 255 來隱藏。本張依任務文字（「0xFF 隱藏」）與候選版實作：0x68 收到 0xFF 就隱藏。如果原生真的會夾值，原版客戶端收到候選 hide255 時應該顯示 100%，那就和本實作不同。這需要逆向 LiTo.bin 的 0x68 receiver 才能裁定（PROTOCOL-CONTRACT §9 #11 已列入換版重核）。
- 101..=254：原生（依上述註解）會夾成 100；本實作照舊拒絕。兩種伺服器都不會送這些值，實際影響應該是零。
- 原生的 HP 條位置、尺寸、顏色，以及「餵了比例之後名稱改成常駐顯示」（`ClientSession.broadcastHitRatio` 註解）都還沒逆向。本張的條是現代版版面，receipt 標 `native_geometry_complete: false`；名稱常駐沒有實作，名稱仍然只在 hover／自己時顯示。
- E149 的其他玩家 S_PUT_OBJECT 帶 `battleHpRatio()`（0..100），所以本實作也會在其他玩家頭上畫條。這符合「只看伺服器值」的原則，但是不是原版行為，需要實機核對。

## 待驗證／未完成

- **本機沒有執行任何 `cargo test`**（依 2026-11-09 規定）。新的 9 個測試交給 PR 的 GitHub Actions，結果以 PR 頁面為準，本報告不預先宣稱通過。之後 Codex 本機可以跑：
  ```text
  cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_hp_meter -- --test-threads=1
  ```
  門檻以 `run-local-test-suite.ps1` 為準。
- 沒有做 GUI 驗收：條和名稱標籤的重疊、DPI、在任務／限時房間的實際外觀都還沒看過。
- 沒有連正式服務或 DB，也沒有啟動遊戲。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_hp_meter.rs`、`native_hp_meter_tests.rs`（新增）
- `LiTo381-Modern/src/modern_lito/native_world.rs`、`native_world_scene.rs`、`mod.rs`（修改）
- 本報告 `docs/reports/issue-183.md`
- 本機證據（不進 git）：`scratch/check-183.log`

## 給下一棒的提示

1. 要裁定 0x68 hide255 的衝突，請逆向 LiTo.bin 的 S_HIT_RATIO receiver（寫入 world object +0x51 的地方），看大於 100 是夾成 100，還是 0xFF 另有分支。若是夾值，就把 `native_hp_meter::receive` 改成 `min(100)`，並更新測試 3、5。
2. 原生 HP 條的繪製（位置、圖素、名稱常駐）要逆向 actor overlay 繪製函式（world labels 的 `4E7200`／`489EAF` 附近）。找到之後替換 `layout`／`draw`，並把 `NATIVE_GEOMETRY_COMPLETE` 改成 true。
3. 伺服器端是否開啟 HPBAR 由候選 `NativeSourceOtherControls.settings().hpbar()` 決定；E149 沒有這個開關，只會在戰鬥中送 0..100。
