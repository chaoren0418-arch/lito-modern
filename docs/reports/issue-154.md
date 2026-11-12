# Issue #154 — T022 批 4 技能 145–155

## 做了什麼

本張把 145–155 十一個妖精技能的請求、選單與控制器回饋接上。封包形狀依 `docs/PROTOCOL-CONTRACT.md` §6.3 的 145–155 列（第 432–442 行），射程與 reuse 依 E149 catalog，目標規則依伺服器原始碼。分支 `task/154`，從 `3318f0b` 開始。

| 技能 | row／column | 形狀 | body | 客戶端檢查 | reuse 毫秒 |
|---|---|---|---|---|---|
| 145 釋放元素 | 18／0 | ⑩ | 11 bytes | 必須明確選取他人物件；存活、可見、在畫面菱形內；不會退回自己 | 0 |
| 146 魂體轉換 | 18／1 | ① | 3 bytes | 自施 | 1200 |
| 147 單屬性防禦 | 18／2 | ① | 3 bytes | 自施 | 0 |
| 148 火焰武器 | 18／3 | ④ | 7 bytes | 一律送自己的物件 ID；指定他人會被拒絕 | 0 |
| 149 風之神射 | 18／4 | ④ | 7 bytes | 同 148 | 0 |
| 150 風之疾走 | 18／5 | ① | 3 bytes | 自施 | 0 |
| 151 大地防護 | 18／6 | ④ | 7 bytes | 同 148 | 0 |
| 152 地面障礙 | 18／7 | ⑩ | 11 bytes | 選取他人；十格內且在畫面菱形內 | 0 |
| 153 魔法消除 | 19／0 | ⑩ | 11 bytes | 同 152（伺服器 `selectionRange()` 固定 10） | 600 |
| 154 召喚屬性精靈 | 19／1 | ① | 3 bytes | 自施 | 0 |
| 155 烈炎氣息 | 19／2 | ① | 3 bytes | 自施 | 0 |

- **`native_skills.rs`**
  - `selector_only` 加入 146、147、150、154、155。
  - 新增 `self_object`（148／149／151）與 `request` 的 `148 | 149 | 151` 分支：伺服器 `useFireWeaponSpell`／`useWindShotSpell`／`useShieldSpell` 只接受 `targetObjectId == playerObjectId()`，其他 ID 回 280，所以客戶端固定送自己的 ID，不送 hover 到的角色。
  - `request` 新增 `145` 分支（⑩，畫面菱形內）；152、153 併入既有的 ⑩ 攻擊分支（預設十格），並額外檢查畫面菱形（伺服器兩者都查）。
  - `reuse_millis`：146＝1200、153＝600，其餘 0。
  - 新增 `Pending::observe_release`：145 的伺服器流程是先扣 MP／材料並送施法動作，之後才送 S10 79（對象不是 SUMMON／TAMED）或 280（機率失敗），所以施法動作之後收到的 79 也算拒絕。
  - `Feedback::Rejected(79)` 的文案由「伺服器拒絕此物品附魔」改成「對象不適用，沒有任何事情發生」，因為現在 145 也會用到。
- **`native_login.rs`**
  - 新增 `skill_release_pending`（只有 145 為真）→ `observe_skill` 走 `observe_release`，並沿用 `hold_summon` 在施法動作後最多等 2 秒的晚到拒絕。
  - 154 設 `skill_summon_pending`：伺服器在施法動作之後才送 353（地圖不能召喚），與 51 相同，直接沿用 `observe_summon`。
- **`native_skill_selection.rs`**：`self_object` 技能清掉 hover 目標；選單加上 145–155 的中文標籤。
- **`native_world.rs`**：只改 `last_item_skill_rejection` 的註解。
- **coverage parser**：`維護工具/audit-native-E149-skill-catalog.py` 不用改（新分支都是它認得的明確 arm 或 `selector_only`）。`維護工具/test_native_skill_coverage.py` 加 1 案（合計 7 案），確認 145–155 全部被辨識。
- **測試**：新增 `LiTo381-Modern/tests/native_skills_145_155.rs` 共 8 案，已登錄到 `維護工具/test-whitelist/tests.tsv`（LF）。

## 已驗證

### 來源（唯讀）

- `docs/PROTOCOL-CONTRACT.md` §6.3 第 432–442 行：row／column 與形狀。
- 伺服器原始碼（上層 repo lito-launcher-server，commit `6bb4a27b`，檔案無未提交變更）`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/ClientSession.java`：
  - 49730 起 `useReturnToNatureSpell`：目標從各玩家的受控同伴裡找；要求畫面內與視線；先送動作，再送 79／280（49947–49961）。
  - 50702 起 `useFireWeaponSpell`、50887 起 `useWindShotSpell`、34337 起 `useShieldSpell`：非自己的 ID 回 `failureMessage`（280）。
  - 48945 起 `useEntangleSpell`、49363 起 `useEraseMagicSpell`：目標排除自己（`candidate != this`），`distance > range` 或不在 `isInNativeScreen` 就拒絕。
  - 70748 起 `useElementalSummonSpell`：selector-only；送出動作後才可能送 353。
  - `BattleSpellCatalog.java` 8234：`EraseMagicSpell.selectionRange()` 回傳 10。
- E149 catalog：原專案唯讀 `驗證紀錄/current/native-E149-skill-catalog-audit/20261024-040044-690336/catalog-facts.json`，重算 SHA256 為 `1EE2338D209CE41154B42B9B51B2A564261AB07FF1340EC7BCCBB188E216B996`（與 #151／#152 相同）。reuseDelayMillis：146＝1200、153＝600、其餘 0；maximumRange：152＝10、145／153＝-1。

### 命令與結果

依 2026-11-09 的本機限制，本機只編譯，不執行測試執行檔。Cargo 命令都在 `LiTo381-Modern/` 執行。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0（Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 22s；只有既存警告）

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_skills_145_155 --test native_skills_110_118 --test native_skills_101_109 --test native_selector_only --test native_skill_selection --no-run
結束碼 0（五個測試執行檔都編譯成功，未執行；沒有 test result: 行）

python -B 維護工具/test_native_skill_coverage.py
結束碼 0；Ran 7 tests in 0.054s；OK
```

## 推測

- 145 沒有在客戶端檢查目標是否為召喚物（例如 `master` 欄位非空）。本張沒有確認 actor 封包的 `master` 對所有受控同伴都會填，所以交給伺服器判定；對一般角色施放時伺服器只記 log、不回覆，客戶端會顯示「未收到施法回覆」。
- 148／149／151 原版客戶端是否也固定送自己的 ID，本張沒有反組譯 `sub_4CB5F0` 的對應 case 確認；依伺服器只接受自己 ID 的行為推定。
- 79 的新文案是依訊息語意寫的，沒有對照客戶端訊息表原文。
- 選單標籤的材料名稱「精靈玉」是依 catalog `itemConsumeId` 40319 推定的名稱。

## 待驗證與未驗證什麼

- **新的 8 案 `native_skills_145_155` 沒有執行過，不算通過。** `.github/workflows/build.yml` 只跑 `cargo check --locked --all-targets` 與 `cargo test --locked --lib`，不會執行 `tests/` 下的整合測試，所以 PR 的 CI 綠燈只代表這個檔案能編譯。需要本機測試執行者（Codex）用 `run-local-test-suite.ps1` 跑出 `test result:` 才算數。#151／#152 的整合測試也是同樣情況。
- 控制器的 `skill_release_pending`／154 的 `skill_summon_pending` 接線只在 `Pending` 層測過，沒有控制器整合測試（需要 `Frame` 與 loopback）。
- 145 與 154 施法後會沿用召喚的 2 秒等待（等晚到的 79／280／353），這段時間不能再施法。這是為了正確顯示拒絕訊息做的取捨，請審查時確認是否可接受。
- 沒有跑正式伺服器施法：釋放同伴、精靈召喚、buff 圖示、材料與 HP／MP 消耗、屬性限制（`elfAttribute`）、155 的隊伍範圍都沒驗。
- 本張沒有接 145–155 的特效／音效資源與渲染（catalog effectGfx 2245、2178、2285、2182、2246、2247、2249、2250、10706、2510／755、2286）。
- `docs/SKILLS-STATUS.md` 的「客戶端」欄沒有更新（批 1、批 2 也沒改），留給 T026 統一同步。
- 沒有跑完整 Full 套件、Release 建置與 T030 GUI 驗收。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`
- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_world.rs`（只有註解）
- `LiTo381-Modern/tests/native_skills_145_155.rs`
- `維護工具/test_native_skill_coverage.py`
- `維護工具/test-whitelist/tests.tsv`
- 本報告 `docs/reports/issue-154.md`

## 給下一棒的提示

- Claude 審查重點：
  - 148／149／151 固定送自己的 ID、清掉 hover 的政策。
  - 145／154 施法後 2 秒等待晚到拒絕的取捨。
  - `Rejected(79)` 文案改動會影響 12／21／48／100／107 的物品施法提示。
- 需要 Codex 本機執行 `native_skills_145_155`（8 案）與回歸 `native_skills_101_109`、`native_skills_110_118`、`native_selector_only`、`native_skill_selection`，回報 `test result:` 原文。
- 後續批次：
  - 純 selector 技能直接加進 `selector_only`；只接受自己 ID 的 ④ 技能（例如 163 燃燒武器走同一個 `useFireWeaponSpell`）可加進 `self_object` 與 `request` 的同一分支。
  - 162 高等屬性精靈與 154 共用 `useElementalSummonSpell`，記得一併設 `skill_summon_pending`。
  - 批 3（#153）若先合併，`selector_only`、`reuse_millis`、選單標籤與 `tests.tsv` 會有文字衝突，兩邊都保留即可。
