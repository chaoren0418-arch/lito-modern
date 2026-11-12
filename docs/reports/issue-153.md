# Issue #153 — T022 批 3 技能 129、130、132–134、137、138

## 做了什麼

本張把妖精系七個技能的請求、選單與控制器接上。封包形狀依 `docs/PROTOCOL-CONTRACT.md` §6.3（第 424–431 行），射程與 reuse 依 E149 catalog。分支 `task/153`，從 `df76b15` 開始。

| 技能 | row／column | 形狀 | body | 客戶端檢查 | reuse 毫秒 |
|---|---|---|---|---|---|
| 129 魔法防禦 | 16／0 | ① | 3 bytes | 自施 | 0 |
| 130 心靈轉換 | 16／1 | ① | 3 bytes | 自施；**MP 為 0 也能送**（只扣 HP） | 100 |
| 132 三重矢 | 16／3 | ⑩ | 11 bytes | 必選目標、不可對自己、存活、可見、畫面菱形內；自己須是弓姿勢（walk action 20）；背包要有箭 | 100 |
| 133 弱化屬性 | 16／4 | ⑩ | 11 bytes | 必選目標、不可對自己、存活、可見、畫面菱形內且十格內 | 100 |
| 134 鏡反射 | 16／5 | ① | 3 bytes | 自施 | 1000 |
| 137 淨化精神 | 17／0 | ① | 3 bytes | 自施 | 0 |
| 138 屬性防禦 | 17／1 | ① | 3 bytes | 自施 | 0 |

- **`native_skills.rs`**
  - `selector_only` 加入 129、130、134、137、138。
  - `request()` 新增 `132 | 133` 分支，送 `d(target) h(x) h(y)`。132 多檢查弓姿勢（新常數 `BOW_WALK_ACTION = 20`），133 多檢查十格（新常數 `ELEMENTAL_FALL_DOWN_RANGE = 10`）。
  - `can_cast` 對 130 不檢查 MP：伺服器 `useBodyToMindSpell` 只看 HP，MP 用光正是要用這招的時候。其他技能的 MP 檢查不變。
  - `reuse_millis` 依 catalog 補上七個值。
- **`native_triple_arrow.rs`**（#180 建的模組）：新增 `preflight`（只有 132 會擷取箭的快照，沒有箭就不送包）與 `receipt`（伺服器回包的扣箭數符合單次三重矢時才產生文字）。
- **`native_login.rs`**：控制器新增 `triple_arrow` 欄位與 `triple_arrow_receipt()`。送 132 前呼叫 `preflight`，送出後保存快照；之後任何施法、一般攻擊或中斷都會清掉，避免把一般射擊的箭算進三重矢。不預扣箭、不預扣 MP。
- **`native_login_window.rs`**：施法回饋文字後面附上扣箭結果（例如「收到施法動作（伺服器扣除 3 支箭）」）。
- **`native_skill_selection.rs`**：選單加上七個技能的中文標籤。
- **coverage parser**：`維護工具/audit-native-E149-skill-catalog.py` 不用改，既有規則已能辨識 `132 | 133 =>` 與 `selector_only`。`維護工具/test_native_skill_coverage.py` 加 1 案，合計 7 案。
- **測試**：新增 `tests/native_skills_129_138.rs`，共 7 案，已登錄到 `維護工具/test-whitelist/tests.tsv`（LF）。

`native_forced_combat::object_target` 對 11 bytes 的 C14 已通用處理，132／133 會自動走既有的目標政策與 LFC1 附加，沒有改動。

## 已驗證

### 來源（唯讀）

- 形狀與 row／column：`docs/PROTOCOL-CONTRACT.md` §6.3 第 424–431 行。
- 伺服器原始碼（上層 repo，唯讀，`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/ClientSession.java`）：
  - 42006 `useMagicResistanceSpell`：`selectorOnlySpellRequest`，缺材料回 299。
  - 42100 `useBodyToMindSpell`：`selectorOnlySpellRequest`，只檢查 HP（不足回 279），沒有 MP 檢查。
  - 42194 `useTripleArrowSpell`：`attackSpellRequest`；沒有弓、walk action 不是 `BOW_WALK`、目標不存在時**不回任何訊息**直接丟棄；MP 不足回 278；受理後先扣 MP，再做 `hitCount` 次一般 `attack()`。42299 `tripleArrowTargetPresent`：目標不可為 0 或自己。
  - 48600 `useElementalFallDownSpell`：`attackSpellRequest`；距離 > `maximumRange` 或不在畫面菱形內就丟棄；沉默／變身回 285，超重回 316。
  - 50113 `useCounterMirrorSpell`：`selectorOnlySpellRequest`；285、316、278、299。
  - 77255 `debitClassicWorldProjectile`：沒有合格的箭時該次射擊被拒絕。因為 MP 在射擊前就扣了，所以客戶端在沒有箭時不送包。
  - `BattleAnimationCatalog.java:174`：`BOW_WALK = 20`。
- E149 catalog：原專案唯讀 `驗證紀錄/current/native-E149-skill-catalog-audit/20261024-040044-690336/catalog-facts.json`，重算 SHA256 為 `1EE2338D209CE41154B42B9B51B2A564261AB07FF1340EC7BCCBB188E216B996`（與 #151、#152 相同）。七個技能的 row／column、reuseDelayMillis、maximumRange 依此表：132 是 -1（畫面內），133 是 10。

`docs/SKILLS-STATUS.md` 第 175 行把 133 的射程寫成 -1，catalog 是 10，伺服器程式也用 `spell.maximumRange()` 比距離。本張依 catalog 做十格檢查。

### 命令與結果

依 2026-11-09 的本機限制，本機只編譯，不執行測試執行檔。Cargo 命令在 `LiTo381-Modern/` 執行。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0（Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 08s；只有既存警告）

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_skills_129_138 --test native_skills_110_118 --test native_skills_101_109 --test native_selector_only --test native_skill_selection --no-run
結束碼 0（五個測試執行檔都編譯成功，未執行）

python -B 維護工具/test_native_skill_coverage.py
結束碼 0；Ran 7 tests in 0.060s；OK
```

`cargo test` 的 `test result:` 原文本機沒有，以 PR 的 CI 為準（見下方「待驗證」）。

## 推測

- 原版客戶端在非弓姿勢時不送 132：依伺服器註解「The selected LiTo.bin producer and retained SprTable normally suppress this request」，沒有反組譯 `LiTo.bin` 確認。
- 沒有箭時不送 132 是沿用 #180 的設計。伺服器有 `projectile.consumeOnAttack()` 這個旗標，表示可能存在不耗箭的弓；本張沒有查哪些武器屬於這類。如果有，這個檢查會擋掉合法施法。
- 選單標籤把 129、134、137、138 的材料寫成「精靈玉」，依據是 catalog 的 `itemConsumeId = 40319`，名稱沒有對照客戶端文字表。`docs/SKILLS-STATUS.md` 把這四個的材料標「伺服器待定」。
- 132 的 `secondaryReuseDelayMillis = 400` 與 130 的同名欄位沒有放進 `client_clock_millis`，不確定原版客戶端是否另有施法節拍。

## 待驗證與未驗證什麼

- 新的 7 案 `native_skills_129_138` 與既有回歸的 `test result:`，以 PR 最新 commit 的 CI 為準。本機沒有執行，在 CI 結果出來前不算通過。
- 控制器的 `triple_arrow` 接線（送包前 `preflight`、攻擊後清除、`triple_arrow_receipt`）只在函式層測過，沒有控制器整合測試，因為需要 `Frame`。
- 扣箭文字在視窗上的顯示沒有開 GUI 確認，交給 Codex／使用者驗收。
- 沒有跑正式伺服器施法：三重矢實際扣箭數、三次命中的動作與特效（gfx 4394）、弱化屬性的成功率與 280、心靈轉換的 HP／MP 變化與訊息 702、四個增益的持續時間與材料消耗。
- 本張沒有接七個技能的特效／音效資源與渲染（catalog effectGfx 2186、2179、4394、4396、4395、2180、2184），也沒有用 PakViewer 查資源，所以不宣稱資源有缺或齊全。
- `Feedback::Rejected(285)`／`Rejected(316)` 目前顯示通用的「伺服器限制施法」，沒有加專屬文案。
- `docs/SKILLS-STATUS.md` 的「客戶端」欄沒有更新（批 1、批 2 也沒改），留給 T026 統一同步；133 射程的 -1 也一併留給 T026 更正。
- 沒有跑完整 Full 套件、Release 建置與 T030 GUI 驗收。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`
- `LiTo381-Modern/src/modern_lito/native_triple_arrow.rs`
- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`
- `LiTo381-Modern/tests/native_skills_129_138.rs`
- `維護工具/test_native_skill_coverage.py`
- `維護工具/test-whitelist/tests.tsv`
- 本報告 `docs/reports/issue-153.md`

## 給下一棒的提示

- Claude 審查重點：
  - 130 在 MP 為 0 時放行（`can_cast` 的唯一例外）是否可以接受。
  - 132 沒有箭就不送包，會不會擋到不耗箭的弓（見「推測」）。
  - 133 用十格而不是 `SKILLS-STATUS.md` 寫的 -1。
- CI 綠燈後由使用者合併。
- 後續批次：
  - 純 selector 技能直接加進 `selector_only`；`d h h` 目標技能可以比照 `132 | 133` 分支。
  - 只扣 HP、不看 MP 的技能要像 130 一樣在 `can_cast` 放行，先查伺服器 handler 有沒有 `playerMp < mpCost`。
  - helper 有變動時，要同步跑 `python -B 維護工具/test_native_skill_coverage.py`。
- T026 同步 `SKILLS-STATUS.md` 時，請把 133 的射程改成 10。
