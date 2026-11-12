# Issue #149 — T027 批 5：技能 67、68、69、70、71、73、74、75

日期：2026-11-12。分支：`task/149`，PR #291。runner：claude-local（第 1 次嘗試）。本張只做第五批八個技能，T027 與 M2 整體都還沒完成。

## 做了什麼

| ID | 送包（未加密 body，含 opcode）與選取 | 收包與呈現 |
|---|---|---|
| 67 變形術 | `14 08 02 + d(actor)`，7 bytes；沒選目標就用自己，原畫面菱形；reuse 10ms | S37 230 沿用既有精確定義。有控制戒指時伺服器送給被施法者的 S2A `monlist`：**只消化、不回覆**（見下） |
| 68 聖結界 | `14 08 03 + d(actor)`，7 bytes；自己或畫面上同一個非空血盟；reuse 1500ms（main 既有，本批補測試） | S37 228／Sound132 沿用既有精確定義 |
| 69 集體傳送術 | `14 08 04 + h(map) h(x) h(y)`，9 bytes；必須明確選「隨機」（目前地圖、座標 0／0）或自己收到的書籤；reuse 10ms | 施法方：S55 `55 00 40 + 自己 ID` → 回一次 C4E `4E 00` → S5A → 既有 C08。S37 2236／Sound836 沿用既有定義。**受邀方 S24/748 同意未接** |
| 70 火風暴 | `14 08 05`，3 bytes，無尾段（selector-only）；reuse 2000ms | S39 action18／type0／gfx1819 改為接受：保留施法動作、名單順序與 miss；四節點呈現**未接**（omitted） |
| 71 藥水霜化術 | `14 08 06 + d h h`，11 bytes；其他活體，三格；reuse 與 client clock 各 500ms | S37 2232／Sound838 沿用既有精確定義 |
| 73 創造魔法武器 | `14 09 00 + d(背包物件)`，7 bytes；走既有 ItemTarget（main 既有）；reuse 0 | S37 763／Sound138 沿用既有精確定義；S10/79 走既有物品拒絕路徑 |
| 74 流星雨 | `14 09 01 + d h h`，11 bytes；其他活體，十格；reuse 3000ms | S39 action18／type8／gfx762 改為接受；收包時凍結**第一個**目標位置（miss 也算；第一個目標不存在時依 heading 退一步，不取第二個）；呈現**未接**（omitted） |
| 75 終極返生術 | `14 09 02 + d(corpse)`，7 bytes；和 41／61 共用明確屍體意圖，原畫面菱形；reuse 10ms | S37 3944（八方向，**未接**，和 61 相同）；**S24/322 同意未接** |

- 沿用既有的施法者守衛：已學會、存活、HP／MP 有值且大於 0、沒有移動封鎖、可見目標。MP／HP 成本、魔法寶石 40318、機率、狀態期間（7200／32／16 秒）、血盟／隊伍資格、復活 HP 與經驗、附魔資格都由伺服器決定，本機不預扣、也不預測成功。
- **69 施法方交握**（`native_login.rs`）：送出 69 之後，只有「這條連線自己的、仍然有效的請求」會回應 S55，而且只回一次；S55 必須在送出後 2 秒內到、物件 ID 必須是自己、InputStamp（連線、世代、生命、控制中斷）沒變。回了 C4E 之後才把後續的 S5A 當成本次請求的回覆，等待上限 8 秒，地圖必須是請求的地圖、座標以伺服器為準。2236 特效或施法動作都不會觸發回覆。伺服器拒絕時的 `25 07`（mode 7）沿用既有解鎖路徑，不建立 portal。等待期間沿用技能 5 的「等待傳送回覆」閘門（不能再施法、攻擊、用物品）。
  - 為什麼一定要做這段：伺服器在送 S55 之前就已經保留 portal（`reserveMassTeleportPortal`），客戶端如果不回 C4E，這個保留會一直卡在伺服器端。只做送包、不做交握會讓施法者卡住，所以兩者一起做。
  - 其他來源的 S55（116／118 援護盟友、受邀方）**維持原狀**（仍是未處理 opcode），本批沒有改。
- **69 目的地選擇**：`native_teleport_spell::Choice` 現在綁定它是為哪個技能選的（5 或 69）。為 5 選的目的地不能拿來施放 69，反過來也不行（含快捷列）；視窗的目的地下拉在選 5 或 69 時都會出現，切換技能時會清掉選取。
- **67 的 `monlist`**：目前 main 上，物件 ID 是自己的 S2A 會掉進商人對話路徑，因為沒有對應的 NPC 互動而報錯、中斷連線（被別人施放 67 也會遇到）。本批把「自己的、沒有參數的 `monlist`」先攔下來：只計數（`shape_menu_offers`），不進同伴或商人狀態，**不送 C_HACTION**，所以不會套用任何外觀。96 個選項的選單沒有做。
- **73 的目標政策修正**：`native_forced_combat::object_target` 原本沒有把 73 排除，7 bytes 的 73 會被當成「角色目標」，背包物件 ID 進到 `native_target_policy::validate` 後查不到角色而被拒絕（「目標已離開」），也就是 73 實際上送不出去（靜態閱讀得出，沒有實機重現）。現在 73 和 12／21／100／107 一樣不算角色目標，也不會附加 LFC1。
- **T 鍵循環屍體**：原本只有選 41 時才循環屍體，現在 41／61／75 都是（`native_skills::corpse_skill`）；61 在批 4 漏了這一處。
- Ctrl／LFC1：67／68／71／74／75 是 7／11 bytes 的角色 body，既有規則已涵蓋；69（9 bytes）、70（3 bytes）、73（物件 ID）都不會附加。
- 選單：八個技能都有名稱，不再顯示「尚未支援」。

## 來源核對
- `TASKS.md` T027、`docs/PROTOCOL-CONTRACT.md` §6.3（八技能的 row／column 與尾段形狀）、`docs/SKILLS-STATUS.md`、`docs/HISTORY.md` L27–L35。
- 歷史候選說明只當設計參考：`維護工具/native-E149-{shape67-protection68,mass69-fire70,mass69-integration,decay71-meteor74,greater75-massslow76,magical73}.md`，以及 `prepare-native-E149-*.py` 裡的 reuse／距離值。沒有執行舊的 prepare 腳本，也沒有套用候選整包或 `維護工具/templates/`。
- 唯讀參考相鄰 repo lito-launcher-server（`D:\lineage開發\3-lito開發`，沒有改動、沒有 checkout；讀的是工作目錄現況）：
  - `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/ClientSession.java`：`useMassTeleportSpell`（約 40863–41031：拒絕一律 `S_PARALYSE` mode 7＋可選訊息；成功送 S_ACTION、S_EFFECT 2236、背包、MP，最後 `sendMassTeleportPortalRequest`）、`answerMassTeleport`（748 一次性）、`returnSummon`（約 28348：收 C4E 後送 S_PORTAL）、變形術送 `monlist` 的位置（約 34250）。
  - `Protocol.java`：`requestSummon`＝`c(0x55) c(0) c(0x40) d(objectId)`、`returnSummon`＝`cc`（狀態 byte 只讀不信任）、`massTeleportSpellRequest`（`ccc hhh`）、`massTeleportAsk`（`S_ASK h(748) s("")`）、`S_ASK 0x24`／`C_ANSWER 0x23`／`C_HACTION 0x27`／`S_HYPERTEXT 0x2A`。
  - `BattleSpellCatalog.java`：67（gfx230、reuse10）、68（gfx228、reuse1500）、69（action19、gfx2236、reuse10、question748、失敗訊息 276）、70（1819、reuse2000）、71（2232、reuse500、clock500、射程 3）、73（763、reuse0、訊息 685／79／161）、74（762、reuse3000、射程 10）、75（3944、reuse10、question322）。
  - `lito登入器/LiTo381-Launcher/native/forced_combat.h`：角色目標排除清單（5、51、58、60、63、69、116、118）；Rust 端另外排除物品目標 12／21／100／107，本批加上 73。
- TW 3.81 與 E149 的差異以 `docs/PROTOCOL-CONTRACT.md` 為準；本批沒有發現需要改契約的地方。
- 既有有限特效定義（沒有新增定義）：228（7 格、Sound132）、763（23 格、Sound138）、2232（21 格、Sound838）、2236（7 格、Sound836），都在 `native_additional_finite_effects.rs` 的既有 bank；230 在批 4 的 bank。畫面結束時間依整數 callback（每格 `floor(ticks×1000/24)`）算出 830／1909／1952／455ms；同一算法對既有的 2177 得 2822ms，和批 4 報告一致。本批沒有宣稱缺資源，所以沒有做 pak 清查。

## 已驗證
- 本機 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`：**exit 0**，原文 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 3m 27s`（`scratch/issue-149/check1.log`）。新增與修改的檔案沒有新的 warning（`native_login.rs` 的 `cast_request_at`／`accept_game` never used 是既有的）。
- CI：**PR #291 的 run 43181326594 通過**，lib 測試 `test result: ok. 2310 passed; 0 failed; 597 ignored`，本批 13 案全部 ok；細節見下方「CI」一節（本機不能執行測試，以 PR 的 CI 為準）。

### 新增的 lib 測試（都不需要資源，CI 的 `cargo test --lib` 會跑）
`native_skills::t027_batch5_tests`（9 案）：
1. `eight_wire_shapes_are_exact_and_costs_are_not_predicted`：八個 body 逐 byte 比對、長度 3／7／9／11、HP／MP 為 1 時仍送包、狀態不被改動、哪些 body 算角色目標（73、69、70 不算）、目標政策在戰鬥階段放行、67 自用／失效 ID 不退回自用、69／73 沒有角色或地面形式、reuse 與 client clock。
2. `caster_gates_reject_without_partial_mutation`：7 種施法者狀態 × 8 技能，失敗時不改動狀態；觀戰不能送 69。
3. `target_kind_range_and_clan_follow_the_formal_record`：活體／屍體／不在場、三格與十格（含對角）、67／75 原畫面菱形 18／20、68 的五種血盟組合。
4. `menu_labels_corpse_intent_and_destination_domain_cannot_be_mixed`：選單名稱、75 屍體意圖不能由活體升級、復活後失效、T 鍵屍體循環、70 丟掉 hover 目標、69／73 不走角色路徑、incarnation／連線失效。
5. `mass_teleport_destinations_are_explicit_skill_bound_and_retire_with_their_owner`：隨機＝目前地圖＋0／0、書籤是 map 在前、5 與 69 的選擇互不通用、只學 5 不能送 69、角色／場景／書籤 revision 變更後失效（含 A→B→A）。
6. `object_effects_are_atomic_and_never_grant_form_status_item_or_life`：S37 230／228／2236／2232／763／3944 截斷時原子拒絕、收包位置凍結、不改 HP／MP／角色（外觀、死亡、位置）／地圖／背包，不算施法 ACK。
7. `centered_and_area_s39_keep_order_misses_and_the_formal_profile_only`：1819／762 截斷原子性、名單與 miss、`CastAction` 回饋、762 首目標位置與 heading 退一步、錯 type／action 仍隔離、1819 空名單。
8. `unqualified_presentations_are_omitted_not_substituted`：3944 與 1819／762 繪製時記為 omitted，沒有畫面、音效或本機 Impact。
9. `rejections_keep_their_codes_and_silence_times_out`：276／278／279／280／299 拒絕碼；73 只有物品路徑讀 79，685／161 不算回覆；兩秒沒回覆是 NoResponse。

`native_login::teleport_wire_tests::t027_batch5`（4 案，沿用既有的合成加密 loopback `FaultPeer`，不連任何服務）：
1. `mass69_answers_only_its_own_s55_once_and_then_admits_the_server_portal`：別人的 S55 與 2236 特效都不觸發；自己的 S55 只回一次 `4E 00`；重複的 S55 不再回；S5A 用伺服器座標、C08 只送一次。
2. `mass69_portal_before_the_return_reply_is_not_this_requests_reply`：沒回過 C4E 時的 S5A 不算本次請求；已回覆的請求只接受所請求的地圖，而且要學會 69（只學 5 不行）。
3. `mass69_unlock_expiry_and_stale_owner_never_send_the_return_reply`：`25 07` 之後、超過兩秒、控制中斷之後都不回 C4E；回覆後等待上限是 8 秒而不是 2 秒，過期後不接受 portal。
4. `shape67_own_monlist_offer_is_consumed_without_a_reply_or_dialog_state`：自己的 `monlist` 被消化兩次、狀態 JSON 不變、沒有送出任何封包（含 C_HACTION）；別人的 `monlist`、帶參數的 `monlist`、`summonlist` 維持原本的拒絕。

### CI
- 第 1 次推送（`2caa94c`）：PR #291 一開就是 CONFLICTING（main 已前進到 `a09f6cf`，含 #154 技能 145–155），GitHub 不會對衝突中的 PR 跑 CI。依常設授權 `git merge origin/main`，只有 `native_skills.rs` 衝突，四處都是同一行的清單，兩邊的項目都保留：`selector_only`（本批 70 加上 main 的 146／147／150／154／155）、角色＋座標分支（本批 71／74 加上 main 的 152／153）、距離表（本批 71=3、74=10，保留 main 對 152／153 的畫面菱形條件）、`reuse_millis`（本批 70=2000、71=500、73=0、74=3000，加上 main 的 145／146／147..=152／153／154／155）。合併後本機 `cargo check …--all-targets`：**exit 0**，`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1m 07s`（`scratch/issue-149/check2.log`）；另外跑了靜態的 `python 維護工具/test_native_skill_coverage.py`（只解析原始碼，不是編譯產物）：`Ran 7 tests … OK`。
- **PR #291 的 run 43181326594（`5eb0807`）：通過**。`cargo check --locked --all-targets` 原文 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1m 19s`；`cargo test --locked --lib --no-fail-fast` 原文 `test result: ok. 2310 passed; 0 failed; 597 ignored; 0 measured; 0 filtered out; finished in 23.33s`。本批 13 案（`native_skills::t027_batch5_tests` 9 案、`native_login::teleport_wire_tests::t027_batch5` 4 案）逐案都是 `ok`（log 存在 `scratch/issue-149/ci-43181326594.log`）。597 個 ignored 不算通過，原資源 opt-in 案仍待 Codex。
- 報告更新的 commit `9cc7542`：run 43182045535 **通過**，`test result: ok. 2310 passed; 0 failed; 597 ignored; 0 measured; 0 filtered out; finished in 23.56s`。
- 第 2 次同步：main 又前進到 `b105ab0`（#153 技能 129、130、132–134、137、138），PR 再度 CONFLICTING，再合併一次。同樣只有 `native_skills.rs` 兩處同一行的清單衝突，兩邊都保留：`selector_only`（本批 70 加上 main 的 129／130／134／137／138）、`reuse_millis`（本批 70／71／73／74 加上 main 的 129／137／138=0、130／132／133=100、134=1000）。合併後本機 `cargo check …--all-targets`：**exit 0**，`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1m 44s`（`scratch/issue-149/check3.log`）；`python 維護工具/test_native_skill_coverage.py`：`Ran 8 tests … OK`。
- **第 2 次同步後的 CI（run 43183053494，`56e9e3e`）：通過**，`cargo test --locked --lib --no-fail-fast` 原文 `test result: ok. 2310 passed; 0 failed; 597 ignored; 0 measured; 0 filtered out; finished in 24.60s`，本批 13 案全部 `ok`；PR 狀態 MERGEABLE／CLEAN。之後只再推了這段報告文字。

## 推測
- 送包形狀、範圍與 reuse 是依正式 contract、候選說明與相鄰伺服器原始碼做的靜態對照，不代表正式 E149 服務一定接受（血盟資格、材料、機率等）。相鄰伺服器原始碼是工作目錄現況，不等於正式唯讀 JAR `E1498871…`；兩者在 69 的 S55／C4E 流程上和歷史候選說明（讀 E149 JAR 得出）一致。
- 69 回覆 C4E 的時間點（收到 S55 就立刻回）與狀態 byte 0：伺服器端只讀不信任這個 byte；原客戶端實際回覆的時間點與 byte 來源沒有另外核定（HISTORY L33 已註明）。
- 228／763／2232／2236 的畫面結束時間是由既有定義算出來的；實際解圖、混色與音效要等原資源 opt-in 案跑過才算驗證。

## 待驗證／未完成
- **本機沒有執行任何 `cargo test` 或 `run-local-test-suite.ps1`**（2026-11-09 規定不得執行編譯產物）。lib 測試以 PR CI 為準；`run-local-test-suite.ps1` 與原資源 opt-in 案 `native_t027_batch5::original_batch5_object_effects_use_exact_banks_blends_and_sounds`（228／763／2232／2236 實際解圖、Screen、各一次 Sound132／138／838／836、結束時間）需要交給 Codex 執行。
- **`run-local-test-suite.ps1` 的技能類目前在 main 上應該就會失敗**（靜態閱讀腳本得出，沒有執行）：`docs/SKILLS-STATUS.md` 批 3 與批 4 的「送包測試」欄寫的是 `native_t027_batch3::…` 與 `native_skills::t027_batch4_tests::…`，但 `維護工具/test-whitelist/skills.tsv` 沒有對應列，`Read-SkillEntries` 會丟 `Unmapped T026 test`。本批自己的列已經對好（狀態表寫完整名稱 `modern_lito::native_skills::t027_batch5_tests::eight_wire_shapes_are_exact_and_costs_are_not_predicted`，`skills.tsv` 加了同名的 `__lib` 列）；批 3／批 4 的列不在本張範圍，沒有動。
- 67：96 選項的變身選單、C_HACTION 一次性回覆、S27 外觀佇列的接線都沒有做。目前有控制戒指的角色被施放 67 時不會出現選單，也不會變身。
- 69 受邀方：S24/748 的非模態同意／拒絕、C23 回覆、同意後的 S55 都沒有做（收到 748 目前被忽略，等於沒有人同意；伺服器 30 秒後過期）。等待 S5A 期間沒有額外擋移動，也沒有「取消」按鈕；逾時只清掉等待，不關連線（歷史候選是關連線）。
- 61／75 被復活方的 S24/321／322 同意流程沒有做；3944 八方向呈現沒有做。
- 70 的 1819 四節點（方向 3／7／5 再移根到方向 1）與 74 的 762（Screen 20 格、Sound130、Hit 在 source4）呈現沒有做，收包後記為 omitted。
- 116／118 的 S55 也還沒有處理（不在本張範圍）。
- 沒有啟動遊戲或客戶端、沒有連正式服務或 DB、沒有做 GUI、DPI、滑鼠鍵盤、音效裝置或和原版並排的驗收；HISTORY 列的 69 書籤／隨機／748 同意拒絕／自然 30 秒／重施／舊 owner／EOF 取消與八秒失敗的實機矩陣、G06 與 T030 都不在本張範圍。

## 產出路徑
- `LiTo381-Modern/src/modern_lito/native_skills.rs`：67／70／71／74／75 送包、`mass_teleport_request`、S55／C4E 常數、`destination_skill`／`corpse_skill`、reuse 與 client clock；測試模組註冊。
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`：屍體意圖（41／61／75）、六個選單名稱。
- `LiTo381-Modern/src/modern_lito/native_teleport_spell.rs`、`native_portal.rs`：目的地選擇綁定技能、`admit_mass_teleport`。
- `LiTo381-Modern/src/modern_lito/native_login.rs`：69 施法方交握與等待視窗、`monlist` 消化、`teleport_choice_for`。
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`：目的地下拉適用 5／69、T 鍵屍體循環適用 41／61／75。
- `LiTo381-Modern/src/modern_lito/native_attack_many.rs`、`native_batch1_area.rs`：S39 兩個 profile、762 首目標位置。
- `LiTo381-Modern/src/modern_lito/native_forced_combat.rs`：73 不算角色目標。
- `LiTo381-Modern/src/modern_lito/native_t027_batch5_tests.rs`（9 案）、`native_t027_batch5_controller_tests.rs`（4 案，掛在 `native_teleport_controller_tests.rs` 底下）。
- `LiTo381-Modern/tests/native_t027_batch5.rs`、`維護工具/test-whitelist/tests.tsv`、`skills.tsv`：1 個原資源 opt-in 案與白名單。
- `docs/SKILLS-STATUS.md`：只改本批八列並加「T027 批 5 回填」一段。沒有改 `docs/REPORT.md`、`TASKS.md`、`PLAN.md`。

## 給下一棒的提示
- 雲端 Claude 審查 PR；Codex 跑 `run-local-test-suite.ps1` 與上面的原資源 opt-in 案，把結束碼和 `test result:` 原文補進本報告。跑之前要先補 `skills.tsv` 的批 3／批 4 對照（見「待驗證」），否則技能類在讀清單時就會失敗。
- S24 問答（321／322／748）建議另開一張一起做：三者共用一個「一次性問題」session（綁連線、世代、生命、問題 ID，命令 ID 不回繞，同意先 consume 再送 C23，不可預設同意）。`native_companions.rs` 目前把非 0x145 的 S24 視為「新問題取代改名」，新流程要接在它後面。748 同意之後的 S55 可以沿用本批的回覆路徑：把 `mass_return` 的條件擴成「施法方請求」或「已同意的 748 grant」。
- 67 的選單可以參考 `維護工具/templates/native_shape67*.rs`（解析與 owner 守衛），但 96 個選項要從伺服器的正式 CSV 重新取，模板裡只有佔位符；本批的 `monlist` 攔截點在 `native_login.rs` 的 `receive_companion` 之前。
- 116／118 的 S55 需要和 69 一樣回 C4E（伺服器 `returnSummon` 是同一個入口），可以直接重用 `native_skills::return_request_object`／`RETURN_REPLY`。
- 70／74 補呈現時：762 的位置已經存在 `Cue.target_origin`；1819 的四個位移是 `[1,1]、[-1,-1]、[-1,1]、[1,-1]`（不含中心），只有最後的根保留名單，可以參考 `native_tornado.rs`（固定位置群組）。
