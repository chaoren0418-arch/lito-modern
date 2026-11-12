# Issue #159 — T022 批 9 技能 201–205、207–210

## 做了什麼

本張把 201–205、207–210 九個幻術師技能的 `C_USE_SPELL` 請求、選單標籤、reuse 與施法時鐘接上（206 專注已在 #101 接好，沒有動）。封包形狀依 `docs/PROTOCOL-CONTRACT.md` §6.3 第 479–488 行；目標規則、射程依伺服器原始碼；reuse 與施法時鐘依 E149 catalog。分支 `task/159`，從 `3e761f2` 開始。

| 技能 | row／column | 形狀 | body | 客戶端檢查 | reuse 毫秒 | 施法時鐘毫秒 |
|---|---|---|---|---|---|---|
| 201 鏡像 | 25／0 | ① | 3 bytes | 不送目標 | 10 | 1800 |
| 202 混亂 | 25／1 | ⑩ | 11 bytes | 必須選目標；不可是自己；存活、可見；三格內 | 3000 | 0 |
| 203 粉碎能量 | 25／2 | ⑩ | 11 bytes | 同上，一格內 | 600 | 0 |
| 204 幻覺：歐吉 | 25／3 | ⑩ | 11 bytes | 沒選目標就送自己（含自己的座標）；選了目標就送該目標，須存活、可見、三格內 | 2000 | 0 |
| 205 立方：燃燒 | 25／4 | ① | 3 bytes | 不送目標 | 5000 | 0 |
| 207 心靈破壞 | 25／6 | ⑩ | 11 bytes | 同 202，三格內 | 3000 | 0 |
| 208 骷髏毀壞 | 25／7 | ⑩ | 11 bytes | 同 202，三格內 | 2000 | 0 |
| 209 幻覺：巫妖 | 26／0 | ⑩ | 11 bytes | 同 204 | 2000 | 64 |
| 210 立方：地裂 | 26／1 | ① | 3 bytes | 不送目標 | 5000 | 0 |

202、203、204、207、208、209 是 TASKS.md 說的 wrapper 技能：伺服器 `IllusionSkillRequests.dispatch` 只收 11 bytes，其他長度直接丟棄，所以這六個一律送 ⑩ 形狀。

- **`native_skills.rs`**
  - `selector_only` 加入 201、205、210。
  - `request` 新增兩個分支：`204 | 209`（目標或自己，三格內）與 `202 | 203 | 207 | 208`（必須選他人；203 一格、其餘三格）。沒有併進既有的 ⑩ 攻擊大分支，因為批 7、批 8 的分支也在改那一行，分開寫可以少一處衝突。
  - 新增 `illusion_targeted`（202、203、204、207、208、209），標示哪些技能走 wrapper；目前只有測試用它確認這六個都送 11 bytes。批 10 的 211–214、216–219 可以直接加進去。
  - `reuse_millis`、`client_clock_millis` 補上表中的值。
  - `Feedback::label` 加上 1412：「此處無法設置立方，或附近已有立方」。
- **`native_world.rs`**：`last_skill_rejection` 認得的訊息碼加上 1412。伺服器只有四個立方 handler（205、210 與批 10 的另外兩個）在放置失敗時會送這個碼；沒加的話，立方放不下去時客戶端只會顯示「未收到施法回覆」。
- **`native_skill_selection.rs`**：選單加上九個技能的中文標籤（名稱取自既有的 `native_skill_catalog.rs`）。
- **coverage parser**：`維護工具/audit-native-E149-skill-catalog.py` 不用改（新分支都是它認得的明確 arm 或 `selector_only`）。`維護工具/test_native_skill_coverage.py` 加 1 案 `test_current_source_covers_ninth_batch`，合計 11 案。
- **測試**：新增 `LiTo381-Modern/tests/native_skills_201_210.rs` 共 8 案，已登錄到 `維護工具/test-whitelist/tests.tsv`（LF、無 BOM）。

沒有改 `native_login.rs`：成功時伺服器送的回覆既有收包都會算成施法回覆（201／204／205／209／210 是 `S_ACTION` 動作 19；202／203／207／208 是 `S_ATTACK` 動作 18 帶特效）。

## 已驗證

### 來源（唯讀）

- `docs/PROTOCOL-CONTRACT.md` §6.3 第 479–488 行：row／column 與形狀。
- 伺服器原始碼（上層 repo lito-launcher-server，commit `6bb4a27b`，`src` 下工作目錄乾淨；`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`）：
  - `IllusionSkillRequests.java`：`ATTACK` 含 202、203、207、208，`OTHER` 含 204、209（第 5–6 行）；長度不是 11 就丟棄（12）；`targetObjectId <= 0` 靜默忽略（14）；`selectPc`（28）目標 ID 等於施法者就回傳施法者，否則必須在合格名單內，不在就回 null（不回覆）。
  - `ClientSession.java`：
    - 201 `useDodgeBuffSpell`（41620）：`selectorOnlySpellRequest`；MP 不足 278；成功送 `S_ACTION` 與 `S_EFFECT`。
    - 202／203／207／208 `useSelectorOnlyTargetedAttackSpell`（60398）→ `resolveSelectorOnlyPrimary` 直接採用明確目標（60436）→ `useAttackSpell`：目標是自己就丟棄（63885）；畫面菱形與視線（63920）；`max(|dx|,|dy|) > maximumRange` 丟棄（63923）；HP 不足 279、MP 不足 278（63998 起）；成功送 `Protocol.attackSpell`（64319，動作 18）。
    - 204 `useIllusionOgreSpell`（38677）、209 `useIllusionLichSpell`（39226）：合格名單是同場存活、未隱身、沒有絕對屏障、`maximumRange` 內的其他玩家（38630 起）；MP 不足 278（38755／39304）；成功送 `S_ACTION` 與 `S_EFFECT`。
    - 205 `useCubeIgnitionSpell`（36908）、210 `useCubeQuakeSpell`（37030）：`selectorOnlySpellRequest`；MP 不足 278；缺材料只記 log、不回覆（36955 起）；放置失敗送 1412（36971／37090）。全檔只有四個立方 handler 送 1412（36971、37090、37209、37328；後兩個是 215 `useCubeShockSpell` 與 `useCubeHarmonySpell`）。
  - `ForcedCombat.java`：按住 Ctrl 時客戶端會在請求後面加 8 bytes 的 LFC1（既有 `native_forced_combat::append`）。`ForcedCombat.dispatch`（49）先用 `unwrap`（37）把這段去掉再交給 `handle`，所以 wrapper 看到的仍是 11 bytes。
  - `BattleSpellCatalog.java` 1468（201）、1697（202）、1712（203）、1730（207）、1750（208）、1770（204）、1785（209）、1819（205）、1838（210）。
- E149 catalog：原專案唯讀 `驗證紀錄/current/native-E149-skill-catalog-audit/20261024-040044-690336/catalog-facts.json`，重算 SHA256 `1EE2338D209CE41154B42B9B51B2A564261AB07FF1340EC7BCCBB188E216B996`（與 #154–#158 相同）。201–210 的 row／column、`reuseDelayMillis`（10、3000、600、2000、5000、1000、3000、2000、2000、5000）、`clientCastClockMillis`（1800、0、0、0、0、1200、0、0、64、0）、`maximumRange`（202＝3、203＝1、204＝3、207＝3、208＝3、209＝3）都依此表。
- 原版 PE（`D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`，重算 SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`），只用 Python 讀檔、沒有執行（腳本 `scratch/issue-159/probe.py`，scratch 不進 git）：施法時鐘表 `word_635774[skill-1]` 的 201–210 是 1800、0、0、0、0、1200、0、0、64、0，與 catalog 一致。

### 命令與結果

依 2026-11-09 的本機限制，本機只編譯，不執行測試執行檔。Cargo 命令在 `LiTo381-Modern/` 執行。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0（Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 57s；只有既存警告，本張改動與新增的檔案沒有警告）

沒有另外跑 cargo test：--all-targets 的 check 已涵蓋新測試檔的型別檢查。測試執行檔未執行，沒有 `test result:` 可附。

python -B 維護工具/test_native_skill_coverage.py
結束碼 0；Ran 11 tests in 0.032s；OK
```

截至本報告寫成時，`task/159` 還沒有 PR；PR 由調度器開啟後才會有 CI 結果。

## 推測

- 原版客戶端對 201–210 送什麼形狀，本張沒有從 PE 確認。PE 的 dispatch case 表（`byte_4CBD00`）讀到 201 之後的位置已經不是合理的 case 編號（例如 201 讀到 139、對應的「handler」是 `90909090`），表示這張表不涵蓋這些技能；伺服器註解寫的是「post-175 default producer emits selector-only」，也就是原版對這六個 wrapper 技能只送 3 bytes。本張依契約與 TASKS.md 送 11 bytes，這是 LiTo 伺服器 wrapper 的要求，不是原版行為。
- 204／209 的客戶端沒有判斷目標是不是玩家角色。選到怪物或塔時伺服器不回覆，畫面會在 2 秒後顯示「未收到施法回覆，可重新施放」。
- 202／203／207／208 的客戶端只檢查 Chebyshev 射程，沒有檢查視線；伺服器視線不通時不回覆。三格內一定在畫面菱形內，所以沒有另外檢查菱形。
- 1412 的中文說明是依伺服器 log 名稱 `nearby-or-invalid` 寫的，沒有對照客戶端文字表的 1412 原文。
- 選單標籤對 205／210 只寫「消耗施法材料」（catalog `itemConsumeId` 49156×1），沒有查材料的中文名稱。

## 待驗證與未驗證什麼

- **新的 8 案 `native_skills_201_210` 沒有執行過，不算通過。** `.github/workflows/build.yml` 只跑 `cargo check --locked --all-targets` 與 `cargo test --locked --lib`，不會執行 `tests/` 下的整合測試；PR 的 CI 綠燈只代表這個檔案能編譯。需要本機測試執行者（Codex）用 `run-local-test-suite.ps1` 跑出 `test result:` 才算數。
- `native_world.rs` 多認一個訊息碼 1412，會影響所有施法的回饋判斷（任何施法等待中收到 1412 都會顯示立方放置失敗）。伺服器目前只有立方 handler 會送，但既有的技能回歸測試（`native_skills`、`native_selector_only`、`native_skill_selection` 與各批 `native_skills_*`）仍應重跑。
- 205／210 缺材料時伺服器不回覆，客戶端只會顯示「未收到施法回覆」，沒有「材料不足」的提示。這需要伺服器改送 299 才能改善（SKILLS-STATUS 已記為伺服器 T025）。
- 沒有跑正式伺服器施法：混亂的沉默、粉碎能量與骷髏毀壞的傷害與暈眩、心靈破壞的 MP 扣除、歐吉／巫妖的 buff、立方的放置與每秒效果、HP／MP／材料消耗都沒驗。
- 本張沒有接 201–210 的特效／音效資源與渲染（catalog effectGfx 6813、6525、6526、6578、6553、7020、6577；立方 6706／6708／6709、6712／6714／6715），也沒有接立方物件（NPC 80149／80150）的顯示。
- `docs/SKILLS-STATUS.md` 的「客戶端」欄沒有更新（先前各批也沒改），留給 T026 統一同步。
- 沒有跑完整 Full 套件、Release 建置與 T030 GUI 驗收。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`
- `LiTo381-Modern/src/modern_lito/native_world.rs`（一行加一行註解）
- `LiTo381-Modern/tests/native_skills_201_210.rs`
- `維護工具/test_native_skill_coverage.py`
- `維護工具/test-whitelist/tests.tsv`
- 本報告 `docs/reports/issue-159.md`

## 給下一棒的提示

- Claude 審查重點：
  - 204／209 沒選目標時送自己的 ID 與自己的座標（伺服器 `selectPc` 接受施法者自己）。
  - 1412 加進全域的 `last_skill_rejection` 名單，而不是另開一個立方專用的觀察函式。
  - 201 的 1800 毫秒施法時鐘會擋住所有技能 1.8 秒（PE 與 catalog 的值，和 175 的做法相同）。
- 需要 Codex 本機執行 `native_skills_201_210`（8 案），並回歸 `native_skills_167_176`、`native_selector_only`、`native_skill_selection`、`native_skills`，回報 `test result:` 原文。
- 批 7（#157）、批 8（#158）還沒合併進 main。它們和本張都改 `native_skills.rs` 的 `selector_only`、`reuse_millis`、`client_clock_millis` 三行，以及 `tests.tsv`、`test_native_skill_coverage.py`、選單標籤；後合併的一方要解衝突，兩邊內容都保留即可。
- 批 10（#160）：211–214、216–219 是同一個 wrapper（`IllusionSkillRequests`：攻擊類 213，其餘在 `OTHER`），可以加進 `illusion_targeted`。214／216／219 和本張的 204／209 一樣用 `selectPc`；211／212／217／218 用 `selectActor`（`ClientSession.java` 37524–37668 的四個 resolver），合格對象的規則本張沒有讀，要另外確認。立方的 1412 回饋本張已經接好。`tests/native_skill_selection.rs` 第 94 行斷言第 220 個技能還是「尚未支援」，接 220 時要一起改。
