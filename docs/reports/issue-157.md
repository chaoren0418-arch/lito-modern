# Issue #157 — T022 批 7 技能 181–188

## 做了什麼

本張把 181–188 八個龍騎士技能的請求、選單與控制器回饋接上。封包形狀依 `docs/PROTOCOL-CONTRACT.md` §6.3 的 181–188 列（第 464–471 行），射程、reuse 與施法時鐘依 E149 catalog，目標規則依伺服器原始碼。分支 `task/157`，從 `c44f93f` 開始。

| 技能 | row／column | 形狀 | body | 客戶端檢查 | reuse 毫秒 | 施法時鐘毫秒 |
|---|---|---|---|---|---|---|
| 181 龍之護鎧 | 22／4 | ① | 3 bytes | 自施 | 10 | 0 |
| 182 燃燒擊砍 | 22／5 | ① | 3 bytes | 自施 | 2000 | 3600 |
| 183 護衛毀滅 | 22／6 | ⑩ | 11 bytes | 必須選取他人；三格內 | 1000 | 1800 |
| 184 岩漿噴吐 | 22／7 | ⑩ | 11 bytes | 必須選取他人；十格內且在畫面菱形內 | 1000 | 0 |
| 185 覺醒：安塔瑞斯 | 23／0 | ① | 3 bytes | 自施 | 0 | 0 |
| 186 血之渴望 | 23／1 | ① | 3 bytes | 自施 | 10 | 0 |
| 187 屠宰者 | 23／2 | ⑩ | 11 bytes | 必須選取他人；兩格內 | 2000 | 0 |
| 188 恐懼無助 | 23／3 | ⑩ | 11 bytes | 必須選取他人；三格內 | 1000 | 1200 |

- **`native_skills.rs`**
  - `selector_only` 加入 181、182、185、186。
  - 183、184、187、188 併入既有的 ⑩ 攻擊分支（射程 3／10／2／3）；184 另外檢查畫面菱形。伺服器 `DragonSkillRequests.dispatch` 對這四個技能只接受長度 11 的 body，不會替舊客戶端猜附近目標，所以客戶端沒有選取目標就不送。
  - 新增 `hp_paid`（181–188）：這八個技能 E149 的 `mpCost` 都是 0，伺服器只檢查 `playerHp <= hpCost`（回 279）。`can_cast` 原本要求 MP 大於 0，會把 MP 用光的龍騎士擋在客戶端，所以這八個技能跳過 MP 檢查。HP 是否足夠不在客戶端預判，交給伺服器回 279。
  - `reuse_millis` 與 `client_clock_millis` 補上表中的數值（181、186 的 10 是既有預設值）。
  - 新增 `Pending::observe_strikes`：187 的伺服器流程不送施法動作（S_ACTION），扣 HP 之後直接跑 3 次一般物理攻擊（與 C_ATTACK 同一條 `attack(..., DIRECT)` 路徑）。所以 187 以「送出請求之後收到自己的物理攻擊回覆」當作施法成立；拒絕碼與中斷的優先順序不變。
  - `CastContext` 多記一個送出時的物理攻擊回覆數：187 的連擊把目標打死時，不會被誤判成「施法已中止」。
- **`native_login.rs`**：新增 `skill_strike_pending`（只有 187 會設值），`observe_skill` 據此改走 `observe_strikes`；中斷連線重置時一併清掉。
- **`native_skill_selection.rs`**：選單加上 181–188 的中文標籤。
- **coverage parser**：`維護工具/audit-native-E149-skill-catalog.py` 不用改。`維護工具/test_native_skill_coverage.py` 加 1 案（合併 main 後合計 9 案），確認 181–188 全部被辨識。
- **測試**：新增 `LiTo381-Modern/tests/native_skills_181_188.rs` 共 8 案，已登錄到 `維護工具/test-whitelist/tests.tsv`（LF）。

### 同步 main

合併 `origin/main`（`b6a3a89`，含批 3／#153 的 129–138）時 `native_skills.rs` 有三處衝突，兩邊的內容都保留：
- `can_cast`：main 的 `skill == 130` 例外＋本張的 `hp_paid`。
- `selector_only`：main 的 129、130、134、137、138＋本張的 181、182、185、186。
- `reuse_millis`：main 的 129–138＋本張的 181–188。

其餘檔案（`native_login.rs`、`native_skill_selection.rs`、`tests.tsv`、coverage 測試）自動合併。

第二次同步（2026-11-12）：合併 `origin/main`（`2d7f018`，含批 5／#155 的 156–166），四個檔案有衝突，兩邊的內容都保留：
- `native_skills.rs` 的 `selector_only`（main 的 156、159、161、162、164＋本張的 181、182、185、186）與 `reuse_millis`（main 的 156–166＋本張的 185、182｜187、183｜184｜188）。
- `native_skill_selection.rs` 的選單標籤、`tests.tsv`（156_166 排在 181_188 之前）、`test_native_skill_coverage.py`（兩案都留，合計 10 案）。
- `docs/REPORT.md` 取 main 版本。

第二次同步後重跑：`cargo check … --all-targets` 結束碼 0；`cargo test … --test native_skills_181_188 --test native_skills_156_166 --test native_skills_145_155 --test native_skills_129_138 --test native_skill_selection --test native_skills --no-run` 結束碼 0（六個測試執行檔編譯成功，未執行）；`python -B 維護工具/test_native_skill_coverage.py` 結束碼 0（`Ran 10 tests`、`OK`）。

第三次同步（2026-11-12）：合併 `origin/main`（`77c4eed`，含 T027 批 5／#149 的 67–71、73–75），只有 `native_skills.rs` 有四處衝突，兩邊的內容都保留：
- `selector_only`：main 的 70＋本張的 181、182、185、186。
- `request` 的 ⑩ 目標分支：main 的 71、74（射程 3／10）＋本張的 183、184、187、188（射程與 184 的畫面菱形檢查）。
- `reuse_millis`：main 的 70、71、73、74＋本張的 181–188。
- `client_clock_millis`：main 的 71＋本張的 182、183、188。

其餘檔案（`native_login.rs`、`native_skill_selection.rs`、`tests.tsv`、`docs/REPORT.md` 等）自動合併，`docs/REPORT.md` 即 main 版本。

第三次同步後重跑：`cargo check … --all-targets` 結束碼 0；`cargo test … --test native_skills_181_188 --test native_t027_batch5 --test native_skills_156_166 --test native_skill_selection --test native_skills --lib --no-run` 結束碼 0（六個測試執行檔編譯成功，未執行）；`python -B 維護工具/test_native_skill_coverage.py` 結束碼 0（`Ran 10 tests`、`OK`）。

## 已驗證

### 來源（唯讀）

- `docs/PROTOCOL-CONTRACT.md` §6.3 第 464–471 行：row／column 與形狀。
- 伺服器原始碼（上層 repo lito-launcher-server，commit `6bb4a27b`）`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`：
  - `DragonSkillRequests.java` 9、22–37：`TARGETED` 含 183／184／187／188；長度不是 11 直接拒絕；目標 ID ≤ 0 不處理；之後截成 3 bytes 轉交舊分派。`guardTarget`／`fearTarget`（49–62）只接受請求指定的 ID，且切比雪夫距離不超過 `range`。
  - `ForcedCombat.dispatch`（`ClientSession.java` 1518）先拆掉 LFC1 尾段，才進到 `useSpell`（32149）→ `DragonSkillRequests.dispatch`，所以按 Ctrl 附加 LFC1 不會影響長度 11 的檢查。
  - `ClientSession.java`：
    - 35307 起 `useGuardBrakeSpell`、35883 起 `useResistFearSpell`：候選排除自己（35633、35686 `candidate != this`）；`playerHp <= hpCost` 回 279（35435）；先送施法動作（35502–35508），機率失敗才送 `failureMessage` 280（35572）。
    - 51877 起 `useDragonSkinSpell`、52705 起 `useBurningSlashSpell`：selector-only；只檢查 HP，回 279（51927、52755）。
    - 51984 起 `useAwakenSpell`、52191 起 `useBloodlustSpell`：selector-only；279／278／299（材料 49157 不足）之後才送動作。
    - 52344 起 `useFoeSlayerSpell`：279／278 之後不送施法動作（52440 的註解），直接 `attack(physical, AttackEnvelope.DIRECT)` 3 次（52464–52466），再送特效 7020／6509。
    - 60350 起 `useSelectorOnlyTargetedAreaAttackSpell` → `resolveSelectorOnlyPrimary`（60436 先取 `DragonSkillRequests.current()`）→ `useAreaAttackSpell`（60732）：目標是自己就拒絕（60782）；`range > maximumRange`、不在畫面菱形或沒有視線就拒絕（60822–60828）。
- E149 catalog：原專案唯讀 `驗證紀錄/current/native-E149-skill-catalog-audit/20261024-040044-690336/catalog-facts.json`，重算 SHA256 為 `1EE2338D209CE41154B42B9B51B2A564261AB07FF1340EC7BCCBB188E216B996`（與 #151／#152／#154 相同）。181–188 的 `mpCost` 全部是 0；`hpCost` 依序 15／10／25／35／1／30／20／30；`reuseDelayMillis` 10／2000／1000／1000／0／10／2000／1000；`clientCastClockMillis` 182＝3600、183＝1800、188＝1200，其餘 0（184 的紀錄型別沒有這個欄位）；`maximumRange` 183＝3、184＝10、187＝2、188＝3。

### 命令與結果（合併 main 之後重跑）

依 2026-11-09 的本機限制，本機只編譯，不執行測試執行檔。Cargo 命令都在 `LiTo381-Modern/` 執行。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0（Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 43s；只有既存警告）

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_skills_181_188 --test native_skills_145_155 --test native_skills_129_138 --test native_skills_110_118 --test native_skills_101_109 --test native_skill_selection --test native_skills --no-run
結束碼 0（七個測試執行檔都編譯成功，未執行；沒有 test result: 行）

python -B 維護工具/test_native_skill_coverage.py
結束碼 0；Ran 9 tests in 0.047s；OK
```

合併前在 `693bc2f` 也跑過同樣的 `cargo check`（結束碼 0）與 `--no-run`（結束碼 0，當時不含 `native_skills_129_138`）。

## 推測

- 187 以「自己的物理攻擊回覆」當作施法成立，是依伺服器走 C_ATTACK 同一條路徑推定的；本張沒有追完 `attack()` 內部（例如攻擊間隔限制）是否可能讓 3 次攻擊都不回覆。若都不回覆，客戶端 2 秒後會顯示「未收到施法回覆」，但 HP 已被伺服器扣除。
- 原版客戶端對 183／184／187／188 送的是 3 bytes（伺服器註解 `Old clients cannot specify these targets`）；本張依合約與伺服器現行行為送 11 bytes，沒有反組譯 `sub_4CB5F0` 的對應 case。
- 選單標籤的射程與「消耗 HP」文字依 catalog 數值撰寫，沒有對照客戶端文字表。

## 待驗證與未驗證什麼

- **新的 8 案 `native_skills_181_188` 沒有執行過，不算通過。** `.github/workflows/build.yml` 只跑 `cargo check --locked --all-targets` 與 `cargo test --locked --lib`，不會執行 `tests/` 下的整合測試，所以 PR 的 CI 綠燈只代表這個檔案能編譯。需要本機測試執行者（Codex）用 `run-local-test-suite.ps1` 跑出 `test result:` 才算數。
- 控制器的 `skill_strike_pending` 接線只在 `Pending` 層測過，沒有控制器整合測試（需要 `Frame` 與 loopback）。
- 183／188 的 280（機率失敗）是在施法動作之後才送。目前只有與施法動作同一輪收到時會顯示「法術未生效」；晚一輪到就只顯示「收到施法動作」。這與既有的機率技能（例如 103、112）行為相同，本張沒有另外加等待。
- 185／186 材料（49157）不足時伺服器回 299，客戶端沿用既有的「施法材料不足」文案；`docs/SKILLS-STATUS.md` 標註這兩個技能的材料規則「伺服器待定（T025）」。
- 沒有跑正式伺服器施法：HP 扣除、buff 圖示、覺醒變身外觀、屠宰者連擊與弱點消耗、岩漿噴吐範圍傷害都沒驗。
- 本張沒有接 181–188 的特效／音效資源與渲染（catalog effectGfx 6531、6590／6591、6508、6516、6975、6523、7020／6509、6586；覺醒 dragonGfx 9362）。因為沒有動資源，所以沒有用 PakViewer 查 pak。
- `docs/SKILLS-STATUS.md` 的「客戶端」欄沒有更新（前幾批也沒改），留給 T026 統一同步。
- 沒有跑完整 Full 套件、Release 建置與 T030 GUI 驗收。

## 範圍外發現（沒有修改）

- 146 魂體轉換 E149 的 `mpCost` 也是 0（扣 HP 40、回 MP 20），但 `can_cast` 仍要求 MP 大於 0。MP 歸零時正是需要這個技能的時候，客戶端卻會擋下。屬於批 4（#154）的範圍，本張沒有改；main 已經用同樣理由放行 130（#153），146 可比照處理。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`
- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/tests/native_skills_181_188.rs`
- `維護工具/test_native_skill_coverage.py`
- `維護工具/test-whitelist/tests.tsv`
- 本報告 `docs/reports/issue-157.md`

## 給下一棒的提示

- Claude 審查重點：
  - `hp_paid` 讓 181–188 跳過 MP 大於 0 的檢查，是否接受；146 是否比照處理。
  - 187 以物理攻擊回覆當作施法成立的做法，以及 `CastContext` 對 187 放寬「目標消失即中斷」。
  - 183／188 晚到的 280 不另外等待的取捨。
- 需要 Codex 本機執行 `native_skills_181_188`（8 案）與回歸 `native_skills_145_155`、`native_skills_129_138`、`native_skills_110_118`、`native_skills_101_109`、`native_skill_selection`、`native_skills`，回報 `test result:` 原文。
- 批 8（189–195）：
  - 192、193、194 也在 `DragonSkillRequests.TARGETED`，同樣必須送 11 bytes；192 走 `useSelectorOnlyTargetedAttackSpell`，193 與 183 共用 `useGuardBrakeSpell`，194 走 `useSelectorOnlyTargetedFreezeAttackSpell`（直線）。
  - 190、195 與 185 共用 `useAwakenSpell`，可直接加進 `selector_only`；`mpCost` 是否為 0 要逐一查 catalog 再決定要不要加進 `hp_paid`。
  - 其他批次合併時，`selector_only`、⑩ 分支的技能清單與射程、`reuse_millis`、選單標籤與 `tests.tsv` 會有文字衝突，兩邊都保留即可。
