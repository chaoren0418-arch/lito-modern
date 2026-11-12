# Issue #158 — T022 批 8 技能 189–195

## 做了什麼

本張把 189–195 七個技能的 `C_USE_SPELL` 請求、選單標籤、reuse 與施法時鐘接上。封包形狀依 `docs/PROTOCOL-CONTRACT.md` §6.3 第 472–478 行；目標規則、射程、reuse 依伺服器原始碼與 E149 catalog。分支 `task/158`，從 `2d7f018` 開始。

| 技能 | row／column | 形狀 | body | 客戶端檢查 | reuse 毫秒 | 施法時鐘毫秒 |
|---|---|---|---|---|---|---|
| 189 岩漿之箭 `SHOCK_SKIN` | 23／4 | ① | 3 bytes | 不送目標 | 10 | 0 |
| 190 覺醒：法利昂 | 23／5 | ① | 3 bytes | 不送目標 | 0 | 0 |
| 191 致命身軀 | 23／6 | ① | 3 bytes | 不送目標 | 1000 | 30 |
| 192 奪命之雷 | 23／7 | ⑩ | 11 bytes | 必須選目標；不可是自己；存活、可見；Chebyshev 八格內 | 4000 | 1800 |
| 193 驚悚死神 | 24／0 | ⑩ | 11 bytes | 同上，三格內 | 1000 | 0 |
| 194 暴龍之眼 | 24／1 | ⑩ | 11 bytes | 同上，十格內，且在畫面菱形內 | 500 | 1800 |
| 195 覺醒：巴拉卡斯 | 24／2 | ① | 3 bytes | 不送目標 | 0 | 0 |

- **`native_skills.rs`**
  - `selector_only` 加入 189、190、191、195。
  - 192、193、194 併入既有的「⑩ 目標＋目前座標」分支（和 152、153 等相同）。射程表加上 192＝8、193＝3，194 用預設的 10 並列入畫面菱形檢查。
  - 新增 `hp_cost_only`（189–195）：這七個技能的 E149 `mpCost` 都是 0，伺服器只檢查 `playerHp <= hpCost`。`can_cast` 原本要求 MP 大於 0，現在對這七個技能略過，MP 是 0 或尚未取得都能送出。HP 是否足夠不在客戶端預測，由伺服器回 279。
  - `reuse_millis` 與 `client_clock_millis` 依 catalog 填入（見上表）。
- **`native_skill_selection.rs`**：選單加上 189–195 的中文標籤（名稱取自既有的 `native_skill_catalog.rs`）。
- **coverage parser 測試**（`維護工具/test_native_skill_coverage.py`）：新增 `test_current_source_covers_eighth_batch`，共 10 案。parser 本身（`audit-native-E149-skill-catalog.py`）不用改，新增的寫法它都認得。
- **測試**：新增 `tests/native_skills_189_195.rs`，共 7 案，已登錄到 `維護工具/test-whitelist/tests.tsv`（LF）。

沒有改 `native_login.rs`：拒絕碼 279／299／280 已在既有的 `last_skill_rejection` 名單內，一般的 `Pending::observe` 就能處理。成功回覆的情況分兩種：

- 190、191、193、195 成功時伺服器送 `S_ACTION`（動作 19），既有收包會算成施法回覆。
- **189 與 194 成功時，客戶端目前收不到施法回覆**（見「待驗證」第一項）。請求本身會送出，伺服器也會照常結算，但畫面會在 2 秒後顯示「未收到施法回覆，可重新施放」。192 的情況未確認。

## 已驗證

### 來源（唯讀）

- 形狀與 row／column：`docs/PROTOCOL-CONTRACT.md` §6.3 第 472–478 行。
- 伺服器原始碼（上層 repo `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`，唯讀，行號為本機工作目錄現況，比契約文件記的少 12 行）：
  - `DragonSkillRequests.java`：`TARGETED` 含 192、193、194（第 9 行）；長度不是 11 直接丟棄（28）；`targetObjectId <= 0` 靜默忽略（33）。
  - `ClientSession.java`：
    - 189 `useCenteredAreaAttackSpell`（61233）：`selectorOnlySpellRequest`；`playerHp <= hpCost` 回 279（61378）。
    - 190／195 `useAwakenSpell`（51984）：`selectorOnlySpellRequest`；HP 不足 279、材料不足 299（52037–52042）；成功送 `S_ACTION`（52089）；再施放一次是解除（`awakenStatus.toggle`）。
    - 191 `useMortalBodySpell`（36721）：`selectorOnlySpellRequest`；HP 不足 279（36771）；成功送 `S_ACTION` 與 `S_EFFECT`。
    - 192 `useSelectorOnlyTargetedAttackSpell`（60398）→ `useAttackSpell`：目標是自己就丟棄（63885）；畫面菱形與視線（63920）；`max(|dx|,|dy|) <= maximumRange`（63923）；HP 不足 279（63998）。
    - 193 `useGuardBrakeSpell`（35307）：目標由 `DragonSkillRequests.guardTarget` 以請求的 ID 加 Chebyshev 射程決定；HP 不足 279（35435）。
    - 194 `useSelectorOnlyTargetedFreezeAttackSpell`（60377）：BATTLE 走 `useLineAttackSpell`，只檢查 Chebyshev 射程（59864），自己為目標時丟棄（59834），HP 不足 279（59961）；WORLD 走 `useClassicWorldLineFreezeAttackSpell`，另外檢查畫面菱形（65225）。
- E149 catalog：原專案唯讀 `驗證紀錄/current/native-E149-skill-catalog-audit/20261024-040044-690336/catalog-facts.json`，重算 SHA256 為 `1EE2338D209CE41154B42B9B51B2A564261AB07FF1340EC7BCCBB188E216B996`（與 #151–#155 相同）。189–195 的 row／column、`mpCost`（全部 0）、`hpCost`（40／1／50／45／50／50／1）、`reuseDelayMillis`、`clientCastClockMillis`、`maximumRange`（192＝8、193＝3、194＝10）都依此表。

### 命令與結果

依 2026-11-09 的本機限制，本機只編譯，不執行測試執行檔。Cargo 命令在 `LiTo381-Modern/` 執行。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0（Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 45s；只有既存警告）

沒有另外跑 cargo test --no-run：--all-targets 的 check 已涵蓋新測試檔的型別檢查。測試執行檔未執行，沒有 `test result:` 可附，等 CI。

python -B 維護工具/test_native_skill_coverage.py
結束碼 0；Ran 10 tests in 0.058s；OK
```

## 推測

- 194 加上畫面菱形是取 BATTLE 與 WORLD 兩個 handler 的交集：BATTLE 只看十格，WORLD 還看菱形。差別只在十格內但菱形外的角落（例如 dx＝10、dy＝9），這些位置的物件是否會出現在客戶端物件表沒有確認。
- 192 沒有另外寫菱形檢查，因為 Chebyshev 八格內的曼哈頓距離最多 16，一定在菱形內。
- 原版客戶端對 192／193／194 實際送什麼，本張沒有反組譯確認。伺服器註解寫「舊客戶端無法指定這些目標」，11-byte 形狀是伺服器為重建版定的規格（契約 §6.3 判定順序第 1 步）。
- 190／195 選單標籤寫「施法材料」而不是物品名，因為 catalog 只有 `itemConsumeId` 49157，沒有對照客戶端字串表。

## 待驗證與未驗證什麼

- **189／194 的成功回覆沒有接上。** 伺服器用 `S_ATTACK_MANY`（0x39）回覆這兩個技能（`ClientSession.java` 61547 與 60179；189 的 type 是 0，194 用預設的 8），特效編號是 6532 與 6988。客戶端的 `native_attack_many::decode` 只收白名單內的特效編號（170、171、1804、1805、3924、757、758、1812、3934），其他一律隔離成未處理封包，不會計入 `own_cast_packets`。所以施法成功後畫面仍會顯示「未收到施法回覆」，而且看不到 HP 以外的結果。本張沒有放寬白名單：那是收包與特效的範圍，放寬前要先確認伺服器對這兩個技能送的 type 欄位與呈現方式。批 7 的 184（同樣是 `AreaAttackSpell`）應該有相同情況，沒有查證。
- 192 成功時伺服器送 `S_ATTACK`（0x23，`Protocol.attackSpell`，64319），特效 6512。`native_combat::decode_attack` 是否接受這個封包、是否算成施法回覆，沒有查證。
- 新的 7 案 `native_skills_189_195` 與既有回歸的 `test result:`，以 PR 最新 commit 的 CI 為準。本機沒有執行，在 CI 結果出來前不算通過。
- 沒有跑正式伺服器施法；HP 扣除、覺醒變身與解除、材料扣除、束縛／凍結／反射效果都沒有實機驗證。`docs/SKILLS-STATUS.md` 把 190、195（材料 49157）與 192（處理器待查）標為「伺服器待定」。
- 伺服器靜默丟棄的情況只會看到「未收到施法回覆」：reuse 未到、角色外形不支援該施法動作、192／194 目標是友方或沒有視線、193 目標在絕對屏障中。
- 192／193／194 對同隊目標需要 Ctrl（LFC1）的規則由既有的 `native_target_policy`／`native_forced_combat` 處理，本張沒有另外寫測試。
- 194 是直線技能，伺服器會打到直線上的其他目標；客戶端只送主目標，沒有預測直線。
- 190／195 的覺醒狀態（外形 9364／9363、每 4 秒扣 8 MP）收包與顯示不在本張範圍，未確認是否已接。
- 控制器層（`LoginController::cast`）沒有為這七個技能寫整合測試；`skill_ready` 用 `max(reuse, 施法時鐘)` 是既有邏輯。
- 本張沒有接 189–195 的特效／音效資源與渲染（catalog effectGfx 6532、6976、6519、6512、6588、6988、6922）。
- `docs/SKILLS-STATUS.md` 的「客戶端」欄沒有更新（前幾批也沒改），留給 T026 統一同步。
- 沒有跑完整 Full 套件、Release 建置與 T030 GUI 驗收。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`
- `LiTo381-Modern/tests/native_skills_189_195.rs`
- `維護工具/test_native_skill_coverage.py`
- `維護工具/test-whitelist/tests.tsv`
- 本報告 `docs/reports/issue-158.md`

## 給下一棒的提示

- **和 PR #293（批 7，`task/157`，尚未合併）會衝突**，後合併的一方自己同步 main，兩邊內容都要保留：
  - `native_skills.rs`：`can_cast` 的 MP 條件、`selector_only`、⑩ 分支的技能清單與射程表、菱形清單（152／153／184／194）、`reuse_millis`、`client_clock_millis`。
  - `native_skill_selection.rs` 的選單標籤、`tests.tsv`、`test_native_skill_coverage.py`。
  - 批 7 的 helper 叫 `hp_paid`（181–188），本張叫 `hp_cost_only`（189–195）。刻意不同名：兩邊的測試都斷言「範圍外的技能不是 HP 付費」，同名合併後會讓對方的測試失敗。合併時 `can_cast` 寫成 `hp_paid(skill) || hp_cost_only(skill)` 即可；之後要合成一個 helper 的話，兩個測試檔的範圍斷言要一起改。
- Claude 審查重點：
  - `hp_cost_only` 讓 MP 為 0 或未取得時仍可送出這七個技能，是否可以接受。
  - 194 同時檢查十格與畫面菱形。
- 批 9、批 10 的 21 個 wrapper 技能裡還有 `IllusionSkillRequests` 的 14 個，同樣是 ⑩ 11 bytes；若 catalog 的 `mpCost` 也是 0，可以沿用同一種 helper。
