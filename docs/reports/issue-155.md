# Issue #155 — T022 批 5 技能 156–166

## 做了什麼

本張把 156–166 十一個技能（妖精屬性魔法）的 `C_USE_SPELL` 請求、選單標籤與 reuse 接上。封包形狀依 `docs/PROTOCOL-CONTRACT.md` §6.3 的 156–166 列；目標規則、射程與 reuse 依伺服器原始碼與 E149 catalog。分支 `task/155`，從 `3318f0b` 開始。

| 技能 | row／column | 形狀 | body | 客戶端檢查 | reuse 毫秒 |
|---|---|---|---|---|---|
| 156 暴風之眼 | 19／3 | ① | 3 bytes | 不送目標 | 0 |
| 157 大地屏障 | 19／4 | ⑩ | 11 bytes | 必須選目標；不可是自己；存活、可見、在畫面菱形內；送目標目前座標 | 600 |
| 158 生命之泉 | 19／5 | ④ | 7 bytes | 選取目標或自己（未選時對自己）；存活、可見、畫面內 | 0 |
| 159 大地的祝福 | 19／6 | ① | 3 bytes | 不送目標 | 0 |
| 160 水之防護 | 19／7 | ④ | 7 bytes | 同 158 | 0 |
| 161 封印禁地 | 20／0 | ① | 3 bytes | 不送目標 | 9000 |
| 162 召喚強力屬性精靈 | 20／1 | ① | 3 bytes | 不送目標 | 0 |
| 163 烈炎武器 | 20／2 | ④ | 7 bytes | 一律送自己的物件 ID；hover 目標會被清掉 | 0 |
| 164 生命的祝福 | 20／3 | ① | 3 bytes | 不送目標 | 300 |
| 165 生命呼喚 | 20／4 | ④ | 7 bytes | 必須選屍體；不可是自己；可見；Chebyshev 十格內 | 0 |
| 166 暴風神射 | 20／5 | ④ | 7 bytes | 同 163 | 0 |

- **`native_skills.rs`**
  - `selector_only` 加入 156、159、161、162、164。
  - 158、160 併入既有的「④ 目標或自己」分支（和 8、9、37 等相同），並列入畫面菱形檢查。
  - 新增三個分支：
    - `163 | 166`：伺服器只接受施法者自己的物件 ID（其他 ID 回 280），所以固定送自己的 ID，傳入其他目標時直接拒絕。
    - `157`：⑩，目標必選且不可是自己。
    - `165`：④，目標必須是屍體，射程用新常數 `CALL_OF_NATURE_RANGE`（10）。
  - 新增分類 helper：`self_bound`（163／166）、`corpse_targeted`（41／165）、`summon_feedback`（51／162）。
  - `reuse_millis` 依 catalog：157＝600、161＝9000、164＝300，其餘 0。
- **`native_skill_selection.rs`**
  - `choice` 對 `self_bound` 技能清掉 hover 目標（和 selector 技能相同），避免殘留的 hover 擋住自施。
  - `Target::resolve_for_skill` 的屍體分支由「只有 41」改成 `corpse_targeted`，165 沿用同一套守衛：活著時的選取不會變成屍體授權，復活、替換、動作序號改變都會失效。
  - 選單加上 156–166 的中文標籤。
- **`native_login.rs`**：`skill_summon_pending` 改用 `summon_feedback(skill)`，讓 162 也走 `observe_summon`。伺服器在扣完 MP 與精靈玉之後才可能回 353（地圖不允許召喚），這和 51 的情況相同。
- **`native_login_window.rs`**：T 鍵循環目標時，選到 165 也改成只挑屍體（原本只認 41）。
- **coverage parser 測試**（`維護工具/test_native_skill_coverage.py`）：新增 `test_current_source_covers_fifth_batch`，共 7 案。parser 本身（`audit-native-E149-skill-catalog.py`）不用改，新分支都是它認得的寫法。
- **測試**：新增 `tests/native_skills_156_166.rs`，共 9 案，已登錄到 `維護工具/test-whitelist/tests.tsv`（LF）。

## 已驗證

### 來源（唯讀）

- 形狀與 row／column：`docs/PROTOCOL-CONTRACT.md` §6.3 第 443–453 行。
- 伺服器原始碼（上層 repo `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`，唯讀，行號為本機工作目錄現況）：
  - `Protocol.java`：`selectorOnlySpellRequest`（8963，3 bytes、不得有剩餘）、`objectSpellRequest`（9081，7 bytes、目標不得為 0）、`attackSpellRequest`（9345，11 bytes）。
  - `ClientSession.java`：
    - 157 `useBindSpell`（57924）：`target == this` 直接丟棄；目標用伺服器自己的位置，client 座標只比對後記 log；畫面菱形檢查在 57953。
    - 158 `useNaturesTouchSpell`（52813）／160 `useAquaProtectSpell`（53179）：目標 ID 等於自己時回退到施法者（52867／53233）；其他存活、可見的玩家也接受，沒有隊伍或血盟檢查；畫面菱形。
    - 163 `useFireWeaponSpell`（50702）／166 `useStormShotSpell`（51415）：目標 ID 不是自己就回 280（50751／51454）。
    - 165：battle（34004–34007）與 world（33253–33256）兩個分派都走 `useCallOfNatureSpell`（71834），不走 `useResurrectionSpell`。目標必須死亡且不是自己，射程是 `max(|dx|,|dy|) <= maximumRange`（71933）。
    - 162 `useElementalSummonSpell`（70748）：無尾段；353 在 70840，發生在扣 MP 與材料之後。
    - 156／159／161／164：都用 `selectorOnlySpellRequest`。
- E149 catalog：原專案唯讀 `驗證紀錄/current/native-E149-skill-catalog-audit/20261024-040044-690336/catalog-facts.json`，重算 SHA256 為 `1EE2338D209CE41154B42B9B51B2A564261AB07FF1340EC7BCCBB188E216B996`（與 #151、#152 相同）。156–166 的 row／column、`reuseDelayMillis`（157＝600、161＝9000、164＝300、其餘 0）、`maximumRange`（165＝10；157／158／160＝-1）都依此表。

### 命令與結果

依 2026-11-09 的本機限制，本機只編譯，不執行測試執行檔。Cargo 命令都在 `LiTo381-Modern/` 執行。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0（Finished `dev` profile；只有既存警告）

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_skills_156_166 --test native_skills_110_118 --test native_skills_101_109 --test native_t027_batch2 --test native_skill_selection --no-run
結束碼 0（五個測試執行檔都編譯成功，未執行；沒有 `test result:` 可附，等 CI）

python -B 維護工具/test_native_skill_coverage.py
結束碼 0；Ran 7 tests in 0.054s；OK
```

### 同步 main（2026-11-12，第二次 push）

分支落後 main 23 個 commit（批 3 #153、批 4 #154、T027 批 4 等已合併），`git merge origin/main` 有五個檔案衝突，兩邊內容都保留：

- `native_skills.rs`
  - `selector_only`、`reuse_millis`、「④ 目標或自己」分支：兩邊的技能編號合併。
  - main 的 `self_object`（148／149／151）與本張的 `self_bound`（163／166）並存，沒有合成一個，因為 `tests/native_skills_145_155.rs` 斷言 `self_object` 只含那三個技能。
  - `corpse_targeted` 改為 41／61／165（main 把 61 加進屍體選取）。
  - `summon_feedback` 改為 51／154／162（main 原本寫 `skill == 154`）。
  - 157 與 main 的 `132 | 133` 分成兩個獨立分支。
- `native_skill_selection.rs`：`choice` 同時清掉 `self_object` 與 `self_bound` 的 hover；`resolve_for_skill` 用 `corpse_targeted`；選單標籤 129–155 與 156–166 都保留。
- `native_login.rs`：`skill_summon_pending` 用 `summon_feedback(skill)`，保留 main 的 `skill_release_pending`。
- `tests.tsv`：`native_skills_156_166` 排在 `native_skills_145_155` 之後、`native_t027_batch4` 之前。
- `test_native_skill_coverage.py`：第三、四、五批的測試都保留，共 9 案。

合併後的副作用：`native_login_window.rs` 的 T 鍵循環用 `corpse_targeted`，所以選到 61 時也只挑屍體（main 原本只認 41）。這和 main 對 61 的屍體選取規則一致，但沒有專屬測試。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0（Finished `dev` profile；只有既存警告）

python -B 維護工具/test_native_skill_coverage.py
結束碼 0；Ran 9 tests in 0.050s；OK
```

合併後沒有重跑 `cargo test --no-run`；`--all-targets` 的 check 已涵蓋所有測試目標的型別檢查。上面第一次 push 的 `--no-run` 與「7 案」是合併前的結果。

## 推測

- 原版客戶端對 163／166 是否也送自己的物件 ID，本張沒有反組譯確認。依據是伺服器 parser 要求 7 bytes 且目標不得為 0，而 handler 只接受施法者自己的 ID。`docs/SKILLS-STATUS.md` 已把這點列為 T061 核對項。
- 165 只做 Chebyshev 十格檢查，沒有另外加畫面菱形，因為伺服器沒有檢查菱形。斜向十格（曼哈頓 20）的屍體是否真的會出現在客戶端物件表，未確認。
- 選單標籤裡的「精靈玉」是依 catalog `itemConsumeId` 40319 與批 3 的用語寫的，沒有對照客戶端字串表。

## 待驗證與未驗證什麼

- 新的 9 案 `native_skills_156_166` 與既有回歸的 `test result:`，以 PR 最新 commit 的 CI 為準。本機沒有執行，在 CI 結果出來前不算通過。
- 165 的 S10 592（屍體所在格站著活人）沒有接成施法拒絕回饋。目前施法會顯示「未收到施法回覆」，訊息本身仍會進 `last_message`。沒有把 592 放進共用的 `last_skill_rejection`，因為沒確認其他功能是否也會送 592。
- 165 成功後，死亡玩家會收到 `S_ASK` 322（是否接受復活）。收包與回覆不在本張範圍，未確認是否已接。
- 屬性限制沒有在客戶端檢查：163 需要火屬性，164／165 需要水屬性。客戶端目前沒有妖精屬性資料，屬性不符時伺服器靜默丟棄，畫面會顯示「未收到施法回覆」。
- 伺服器靜默拒絕的情況都只會看到「未收到施法回覆」：157 在 battle 階段 MP 不足、161 畫面內沒有對象、162 MP 不足或已有同類同伴。
- 157 對同隊目標需要 Ctrl（LFC1）的規則由既有的 `native_target_policy`／`native_forced_combat` 處理，本張沒有為 157 另外寫測試。
- 控制器層的 `skill_summon_pending`（162 → `observe_summon`）只在 `Pending` 層測過，沒有控制器整合測試，因為需要 `Frame`。
- 沒有跑正式伺服器施法；材料扣除（157／161／162／165 的 40319）、buff 效果、精靈召喚、復活流程都沒有實機驗證。`docs/SKILLS-STATUS.md` 把 157、161、162、165 標為「伺服器待定」。
- 本張沒有接 156–166 的特效／音效資源與渲染（catalog effectGfx 2288、2251、2243、2287、5829、2241、2510、2242、2244、2245、2248）。
- `docs/SKILLS-STATUS.md` 的「客戶端」欄沒有更新（前幾批也沒改），留給 T026 統一同步。
- 沒有跑完整 Full 套件、Release 建置與 T030 GUI 驗收。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`
- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`
- `LiTo381-Modern/tests/native_skills_156_166.rs`
- `維護工具/test_native_skill_coverage.py`
- `維護工具/test-whitelist/tests.tsv`
- 本報告 `docs/reports/issue-155.md`

## 給下一棒的提示

- Claude 審查重點：
  - 163／166 固定送自己的物件 ID，這個做法是否可以接受。
  - 165 沿用 41 的屍體選取守衛（`corpse_targeted`），以及只檢查 Chebyshev 十格。
  - 162 借用 51 的 `observe_summon`，施法後會保留 pending 最多 2 秒等 353。
- 和 PR #289（批 3，`task/153`）會在三個地方相鄰衝突，後合併的一方自己同步 main 即可，兩邊的內容都要保留：
  - `native_skills.rs` 的 `selector_only` 清單與 `reuse_millis`。
  - `native_skill_selection.rs` 的選單標籤。
  - `維護工具/test-whitelist/tests.tsv`（`native_skills_129_138` 排在 `native_skills_156_166` 前面）。
- 後續批次：
  - 純 selector 技能直接加進 `selector_only`。
  - 伺服器只收自己 ID 的 ④ 技能加進 `self_bound` 與 `163 | 166` 分支。
  - 需要屍體的技能加進 `corpse_targeted`。
  - helper 或分支有變動時，同步跑 `python -B 維護工具/test_native_skill_coverage.py`。
- 之後若要接 165 的 592 回饋，建議比照 #152 的 `last_clan_skill_rejection` 做成 165 專用的序號欄位。
