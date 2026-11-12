# Issue #156 — T022 批 6 技能 167–176

## 做了什麼

本張把 167–176 十個妖精技能的請求、選單標籤與重用時間接上。封包形狀依 `docs/PROTOCOL-CONTRACT.md` §6.3 的 167–176 列（第 454–463 行），並用原版 PE 的 dispatch 表逐一核對；射程、reuse 依 E149 catalog；目標規則依伺服器原始碼。分支 `task/156`，從 `b105ab0` 開始。

| 技能 | row／column | 形狀 | body | 客戶端檢查 | reuse／原版施法時鐘（毫秒） |
|---|---|---|---|---|---|
| 167 風之枷鎖 | 20／6 | ⑩ | 11 bytes | 必須選取他人；存活、可見、八格內 | 100／600 |
| 168 鋼鐵防護 | 20／7 | ④ | 7 bytes | 一律送自己的物件 ID；指定他人會被拒絕 | 0／0 |
| 169 體能激發 | 21／0 | ① | 3 bytes | 自施 | 100／100 |
| 170 水之元氣 | 21／1 | ④ | 7 bytes | 沒選目標就送自己；選了目標就送該目標（存活、可見、畫面菱形內） | 100／400 |
| 171 屬性之火 | 21／2 | ① | 3 bytes | 自施 | 100／300 |
| 172 暴風疾走 | 21／3 | ① | 3 bytes | 自施 | 0／0 |
| 173 污濁之水 | 21／4 | ⑩ | 11 bytes | 必須選取他人；三格內 | 1000／1000 |
| 174 精準射擊 | 21／5 | ⑩ | 11 bytes | 必須選取他人；十格內且在畫面菱形內 | 3000／3000 |
| 175 烈焰之魂 | 21／6 | ① | 3 bytes | 自施 | 300／5000 |
| 176 能量激發 | 21／7 | ① | 3 bytes | 自施 | 100／100 |

實際的下次可施法時間沿用控制器既有算式：`max(reuse_millis, client_clock_millis)`，所以 175 送出後客戶端會等 5 秒。

- **`native_skills.rs`**
  - `selector_only` 加入 169、171、172、175、176。
  - `self_object` 加入 168，`request` 的自己物件分支變成 `148 | 149 | 151 | 168`。伺服器 `useIronSkinSpell` 對不是自己的 ID 回 280。
  - 170 併入既有的「目標或自己」④ 分支（與 1、8、9 等同一段），範圍用畫面菱形。
  - 167、173、174 併入既有的 ⑩ 攻擊分支：167 八格、173 三格、174 十格再加畫面菱形。
  - `reuse_millis` 與 `client_clock_millis` 補上十個技能的值。
- **`native_skill_selection.rs`**：選單加上 167–176 的中文標籤。172 的標籤直接寫明「伺服器只扣 MP，沒有動作與效果」（原因見下一節）。
- **`tests/native_skills_145_155.rs`**：`self_object` 的全域檢查原本寫死只有 148／149／151，加入 168（只改這一行的過濾條件與一行註解）。
- **coverage parser**：`維護工具/audit-native-E149-skill-catalog.py` 不用改（新技能都落在它認得的明確 arm 或 `selector_only`）。`維護工具/test_native_skill_coverage.py` 加 1 案（合併批 5 後合計 10 案），確認 167–176 全部被辨識。
- **測試**：新增 `LiTo381-Modern/tests/native_skills_167_176.rs` 共 8 案，已登錄到 `維護工具/test-whitelist/tests.tsv`（LF、無 BOM）。

### 172 暴風疾走的行為

伺服器 `useOtherNoEffectSpell` 受理後只扣 MP 並送 `S_MANA_POINT`，不送施法動作、特效或訊息。客戶端沒有任何可以當成「施法成功」的封包，所以送出兩秒後回饋會顯示「未收到施法回覆，可重新施放」。本張沒有把 MP 變動當成受理依據（自然回魔也會讓 MP 變動），只在選單標籤上說明這個技能沒有效果。

### 同步 main

合併 `origin/main`（`2d7f018`，含批 5／#155）時有四個檔案衝突，兩邊的內容都保留：
- `native_skills.rs`：`selector_only`（main 的 156、159、161、162、164＋本張的 169、171、172、175、176）、④「目標或自己」分支（main 的 158、160＋本張的 170）、`reuse_millis`（兩邊的值都在）。批 5 另外新增了 `self_bound`（163／166），與本張沿用的 `self_object`（148／149／151／168）是兩個功能相同的分類，本張沒有合併它們。
- `native_skill_selection.rs`：選單標籤 156–166 在前、167–176 在後。
- `維護工具/test-whitelist/tests.tsv`：`native_skills_156_166` 9 列在前、`native_skills_167_176` 8 列在後，維持 LF。
- `維護工具/test_native_skill_coverage.py`：批 5 的測試原本斷言「167 尚未接上」，本張接上 167 後這個斷言不再成立，已刪除那兩行（註解與 `assertNotIn(167, …)`），其餘保留。

第二次合併 `origin/main`（`a573a02`，含 T027 批 5／#149）時只有 `native_skills.rs` 衝突，同樣兩邊保留：`selector_only`（main 的 70）、④ 分支（main 的 67）、⑩ 攻擊分支與射程（main 的 71、74＋本張的 167、173、174）、`reuse_millis` 與 `client_clock_millis`（main 的 70／71／73／74＋本張的 167–176）。

## 已驗證

### 來源（唯讀）

- `docs/PROTOCOL-CONTRACT.md` §6.3 第 454–463 行：row／column 與形狀。
- 原版 PE（`D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`，重算 SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`）。只用 Python 讀檔與 Capstone 反組譯，沒有執行。腳本在 `scratch/issue-156/pe_probe.py`、`pe_disasm.py`（scratch 不進 git）：

  | 技能 | jump table case → handler | 內容 |
  |---|---|---|
  | 167 | 29 → `4CB78B` | `mov edx, 8`；`call 4CA6E0`（點選射程 8，⑩） |
  | 168、170 | 4 → `4CB71D` | `call 4C9880`（物件形式 ④；148／149／151 也是這個 handler） |
  | 169、171 | 1 → `4CBBD4` | `ccc`（與技能 2、13 相同） |
  | 172 | 31 → `4CBC27` | `push "ccc"`；`call 4C6AE0` |
  | 173、174 | 2 → `4CB755` | `mov edx, 0Ah`；`call 4CA6E0`（點選射程 10，⑩；152／153 也是這個 handler） |
  | 175、176 | 30 → `4CBACB` | `push (esi & 7)`；`jmp 4CBBDA`（跳進 `ccc` 的後半段） |

  施法時鐘 `word_635774[skill-1]`（167–176）：600、0、100、400、300、0、1000、3000、5000、100。
- 伺服器原始碼（上層 repo lito-launcher-server，commit `6bb4a27b`，`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`）：
  - `ClientSession.java` 53665 起 `useWindShackleSpell`：`selectedRange > spell.clientClickRange()`（8）或找不到目標回 280；施法動作送出後，若沒有任何對象中招再送一次 280（54002–54006）。
  - 51749 起 `useIronSkinSpell`：`request.targetObjectId() != playerObjectId()` 回 280（51791）。
  - 52990 起 `useWaterLifeSpell`：目標可以是自己或同場存活玩家，要求 `isInNativeScreen`；不符回 280，缺材料回 299。
  - 54070 起 `useTargetNegativeStatusSpell`（173／174）：目標排除自己（`candidate != this`），`range > spell.maximumRange()` 或不在 `isInNativeScreen` 就只記 log、不回覆。
  - 41347 `usePhysicalProcBuffSpell`（171／175）、44215 `useExoticVitalizeSpell`（169）、44346 `useAdditionalFireSpell`（176）、41520 `useOtherNoEffectSpell`（172）：都用 `selectorOnlySpellRequest`。
  - `BattleSpellCatalog.java` 897 起（167）、952（168）、1155（169）、1173（176）、1194（170）、1401／1415（171／175）、1432（172）、2311（173，`maximumRange` 3）、2329（174，`clientClickRange` 10、`maximumRange()` 回傳 19）。
- E149 catalog：原專案唯讀 `驗證紀錄/current/native-E149-skill-catalog-audit/20261024-040044-690336/catalog-facts.json`，重算 SHA256 `1EE2338D209CE41154B42B9B51B2A564261AB07FF1340EC7BCCBB188E216B996`（與 #154 相同）。reuseDelayMillis（167–176）：100、0、100、100、100、0、1000、3000、300、100。`clientCastClockMillis` 有記錄的 167／169／170／173／174／176 與 PE 的值一致。

### 命令與結果（合併 main `a573a02` 之後重跑）

依 2026-11-09 的本機限制，本機只編譯，不執行測試執行檔。Cargo 命令都在 `LiTo381-Modern/` 執行。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0（Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 08s；只有既存警告，本張改動的檔案沒有警告）

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_skills_167_176 --test native_skills_156_166 --test native_skills_145_155 --test native_selector_only --test native_skill_selection --no-run
結束碼 0（五個測試執行檔都編譯成功，未執行；沒有 test result: 行）

python -B 維護工具/test_native_skill_coverage.py
結束碼 0；Ran 10 tests in 0.041s；OK
```

合併前在 `71dddaf`（3m 45s）與第一次合併後（1m 24s）也跑過同樣的 `cargo check` 與 `--no-run`，結束碼都是 0。

截至本報告最後一次更新，`task/156` 還沒有 PR，分支上也沒有 CI run（`gh run list --branch task/156` 為空）；PR 由調度器開啟後才會有 CI 結果。

## 推測

- 167 風之枷鎖：伺服器的候選名單包含施法者自己，所以伺服器其實會接受「選自己」。客戶端這裡比伺服器嚴格，一律要求選取他人；原版客戶端 `sub_4CA6E0` 是否允許點自己，本張沒有反組譯確認。
- 168 鋼鐵防護：原版 `sub_4C9880` 送出的物件 ID 是否固定為自己，本張沒有反組譯到函式內部；依伺服器只接受自己 ID 推定（與 #154 的 148／149／151 相同做法）。
- 170 水之元氣：客戶端沒有判斷目標是不是玩家角色。選到怪物時伺服器會回 280（法術未生效）。
- 173：原版點選射程是 10，但伺服器的有效射程是 3，超出時伺服器不回覆。客戶端採用 3，避免送出一定不會有回應的請求。
- 選單標籤的材料名稱「精靈玉」沿用 #154 的寫法（catalog `itemConsumeId` 40319），沒有對照客戶端文字表。

## 待驗證與未驗證什麼

- **新的 8 案 `native_skills_167_176` 沒有執行過，不算通過。** `.github/workflows/build.yml` 只跑 `cargo check --locked --all-targets` 與 `cargo test --locked --lib`，不會執行 `tests/` 下的整合測試，PR 的 CI 綠燈只代表這個檔案能編譯。需要本機測試執行者（Codex）用 `run-local-test-suite.ps1` 跑出 `test result:` 才算數。
- `native_skills_145_155` 被改了一行（`self_object` 清單加 168），同樣需要重跑。
- 167 的「施法動作之後才到的 280」：目前走一般的 `observe`，只有 280 與施法動作在同一輪輪詢內到達時才會顯示「法術未生效」；晚一輪到就只顯示「收到施法動作」。本張沒有像 145 那樣加兩秒等待，因為 167 的 reuse 只有 100 毫秒，加等待會拖慢連續施法。請審查時確認這個取捨。
- 沒有跑正式伺服器施法：枷鎖／污濁之水／精準射擊的命中、buff 圖示、材料與 MP 消耗、屬性限制（`elfAttribute`）、172 實際只扣 MP 的行為都沒驗。
- 本張沒有接 167–176 的特效／音效資源與渲染（catalog effectGfx 1799、2252、4400、4401、4402、5830、5826、5833、5825；172 為 0）。
- `docs/SKILLS-STATUS.md` 的「客戶端」欄沒有更新（先前批次也沒改），留給 T026 統一同步。
- 沒有跑完整 Full 套件、Release 建置與 T030 GUI 驗收。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`
- `LiTo381-Modern/tests/native_skills_167_176.rs`
- `LiTo381-Modern/tests/native_skills_145_155.rs`（一行）
- `維護工具/test_native_skill_coverage.py`
- `維護工具/test-whitelist/tests.tsv`
- 本報告 `docs/reports/issue-156.md`

## 給下一棒的提示

- Claude 審查重點：
  - 172 送出後一定顯示「未收到施法回覆」，只靠選單標籤說明；是否要另外做「只扣 MP」的回饋。
  - 167 不允許選自己、晚到的 280 不另外等待。
  - 175 的 5 秒客戶端施法時鐘（PE 值）會擋住所有技能 5 秒，與 33 木乃伊的詛咒同樣做法。
- 需要 Codex 本機執行 `native_skills_167_176`（8 案）與回歸 `native_skills_156_166`、`native_skills_145_155`、`native_selector_only`、`native_skill_selection`，回報 `test result:` 原文。
- 批 5（#155）已合併進 main 並同步到本分支。`self_bound`（批 5：163／166）與 `self_object`（批 4 與本張：148／149／151／168）功能相同，之後可以整理成一個分類；這不在本張範圍，沒有動。
- 後續批次：`self_object` 每加一個技能，都要同步改 `native_skills_145_155.rs` 的全域檢查清單。
