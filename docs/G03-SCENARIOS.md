# G03 情境清單——舊 54 情境盤點與補充情境定義（T053 1／2，#332）

更新：2026-11-12。基準 main `91e916c`，正式唯讀 JAR E149（`E1498871…`）。

本檔只做**盤點與定義**，沒有執行任何測試。「E149 結果」欄只記錄已經存在的報告；沒有的一律寫「無」，等 T053 第 2 張實跑。舊版（BBC59308、EBF5EE66 等）的通過紀錄不算現行結果。

## 1. 舊 54 情境是什麼

- 來源：schema37 的「Debug 54 隔離 TCP」（`docs/HISTORY.md:573`、`NATIVE-SCENE-LOADING.zh-TW.md:3`）。每一案是 `LiToModern.exe --probe-native-*` 連到本機 loopback 的 Java harness（`tests/oracle/Native*Harness.java`＋正式 JAR），由 `維護工具/verify-native-<群組>.ps1` 驅動。
- 逐案清單取自本機未追蹤的 `驗證紀錄/current/native-real-peers/regression-matrix.json`（`legacy_count=54`，2026-10-13，JAR BBC59308，EXE `AD02BC2A…`）。該檔不在 git，所以把 54 列抄進本檔 §3。
- 2026-10-13 的舊處置：31 案 retain、23 案因訓練 bot 被關而調整（5 案由「雙玩家雙場」取代、7 案改用真實對手、8 案移除不必要的 bot 前置、1 案改打真實 NPC、2 案分離重驗）。那次的 53 案結果綁定 BBC59308 與 `AD02BC2A…`，**不是 E149 結果**。

## 2. 影響裁定的現況（已確認）

1. **舊 runner 綁的不是 E149。** `verify-native-*.ps1` 都讀 `驗證紀錄/current/reference-snapshot.json`，該檔目前釘的是 `C02AAAE8…`（2026-10-15），輸出也寫到未追蹤的 `驗證紀錄/current/native-*`。所以 54 案**沒有任何一案**有 E149 的 probe 結果；要重跑，runner 必須先改成 owned 模式（JAR 以參數傳入並核對 E149 SHA、輸出寫到 worktree 的 `target/current/`），做法可照 `run-teleport-failure.ps1` 與 `tests/native_two_player_battle.rs` 的 fixture。這是所有案共通的前置，不逐案重複寫。
2. **probe 入口還在。** `src/main.rs` 仍有 `--probe-native-loopback／-login／-game／-lobby`；`native_login.rs:3061–3115` 的 lobby mode 清單包含 54 案用到的全部 mode，另有 `recovery-room／recovery-world`、`room-world-peer-a／b／c`、`room-world-emblems`、`room-world-item-heal-194／197`。
3. **E149 的訓練 bot 是條件式。** #174 以 javap 核對：`NativeRoomRules.activeBot` 要同時滿足 DEATHMATCH、`testBotEnabled`、`testRoom`、參戰者 1 人、沒有虛擬玩家。舊文件「startBattle 強制關 bot」只對 EBF5EE66／BBC59308 成立。單人進場時 bot 會不會出現，E149 沒有動態結果（#174 待驗證）。
4. **雙真實玩家已有 E149 動態結果。** `tests/native_two_player_battle.rs` 兩個 ignored 測試在 #174／#175 實跑通過（基底 `318b3ab`，2026-11-06）：登入→選來源→選角→進頻道→進房→準備→開戰→Energy Bolt→普攻至死亡→死亡確認→雙端結算→回大廳→同連線重入→再施法。下表以 **E1** 標示。
5. **其他 E149 相關結果**：
   - **E2**：#56 白名單 76 項整合測試通過（基底 `85cc83d`，2026-11-05）。只有 Rust 端的封包／狀態測試，沒有啟動 JVM。
   - **E3**：#118 G06 五案（timeout、refused、wrong-service、cancel-retry、cancel-expiry）以 E149 owned harness 實跑通過。後補的三案（cancel-repeat、refused-cancel-repeat、cancel-expiry-source）沒有實跑（#170、#173）。
   - CI 只跑 `cargo check` 與 `cargo test --lib`，不跑 `tests/` 整合測試，也不跑任何 probe。
6. E1–E3 都早於現在的 main，EXE 也不同。它們是「E149 上曾經通過」的證據，不是「當前 EXE」的結果；第 2 張仍要在當前 main 重跑。

## 3. 舊 54 情境逐案裁定

裁定的意思：
- **保留**：情境的步驟與斷言在 E149 仍然成立，換 runner 後照原定義重跑。
- **改寫**：目的保留，但前置、對手或斷言要換（多數是不能再依賴訓練 bot）。
- **失效**：不再單獨成案。

「現行對應」是 repo 內可以用來重驗的入口；`probe:<mode>` 指舊 probe mode，Rust 測試以 `檔名::函式` 表示。

### 3.1 連線與登入（保留 15）

| # | 群組／mode | 裁定 | 原因 | 現行對應 | E149 結果 |
|---:|---|---|---|---|---|
| 1 | handshake／accept | 保留 | 版本握手契約沒變（`native_network::deployed_jar_version_request_and_replies_match_typed_native_contract`） | `NativeHandshakeHarness` mode accept | probe 無；Rust 端 E2 |
| 2 | handshake／reject | 保留 | 伺服器拒絕版本，客戶端回報 `native version rejected` 後停止 | 同上 mode reject | 無 |
| 3 | handshake／malformed | 保留 | 版本回覆尾段長度錯誤要整包拒絕 | 同上 mode malformed | 無 |
| 4 | handshake／header-eof | 保留 | 只收到部分長度欄就斷線，回報 `EOF (PartialHeader)` | 同上；`native_network::fragmented_and_coalesced_frames_preserve_full_u16_payload_and_eof_state` | probe 無；Rust 端 E2 |
| 5 | handshake／body-eof | 保留 | 封包本體不完整就斷線，回報 `EOF (PartialBody)` | 同上 | probe 無；Rust 端 E2 |
| 6 | handshake／stall | 保留 | 對方不送資料，握手期限到要失敗而且能取消 | 同上 mode stall；`native_network::stalled_seed_can_cancel_and_reconnect_without_stale_owner_or_thread`（#306 N 類，未跑） | 無 |
| 7 | login／accept | 保留 | 帳密、來源選擇流程沒變 | `NativeLoginHarness`；`native_login` 3 測試 | probe 無；E1 走過同一步驟 |
| 8 | login／reject | 保留 | 登入失敗要保留完整失敗內容，不進 GAME | 同上 | 無 |
| 9 | login／retry | 保留 | 失敗後同一控制器重試成功 | 同上 | 無 |
| 10 | game／select | 保留 | LOGIN→GAME 交接與選角 | `NativeGameHarness` mode select | probe 無；E1 走過同一步驟 |
| 11 | game／notices | 保留 | 登入公告與確認回覆（`native_game::notice_acknowledgements_match_deployed_parsers`） | mode notices | 無 |
| 12 | game／empty | 保留 | 帳號沒有角色時目錄為空，不能選角 | mode empty | 無 |
| 13 | game／unauthorized | 保留 | 沒有經過 LOGIN 交接直接連 GAME 要被拒 | mode unauthorized | 無 |
| 14 | inventory／inventory | 保留 | 選角後背包、名稱、圖示由同一批封包建立 | mode inventory；`native_inventory` 7 測試 | 無 |
| 15 | inventory／inventory-reselect | 保留 | 重新選角後背包屬於新的選擇，不沿用舊的 | mode inventory-reselect；`native_inventory::selected_connection_and_reselection_own_inventory_not_world_generation` | 無 |

### 3.2 大廳與房間（保留 11）

| # | 群組／mode | 裁定 | 原因 | 現行對應 | E149 結果 |
|---:|---|---|---|---|---|
| 16 | lobby／enter | 保留 | `C_LITO_ENTER_CHANNEL` 成功路徑沒變 | probe:enter；`native_lobby` 3 測試 | probe 無；E1 走過同一步驟 |
| 17 | lobby／switch | 保留 | 換頻道 | probe:switch | 無 |
| 18 | lobby／return | 保留 | 離開頻道回大廳 | probe:return | probe 無；E1 有「結算後回大廳」 |
| 19 | lobby／reselect | 保留 | 回選角再選一次。T062（#194）改過選角分頁與選服流程，probe 走的是 `LoginController`，是否受影響要實跑才知道 | probe:reselect；`native_character_flow` 2 測試 | 無 |
| 20 | lobby／restricted | 保留 | 選角結果 7（參加限制）不能進頻道 | probe:restricted | 無 |
| 21 | room／room-join | 保留 | `C_LITO_JOIN`→`S_LITO_JOIN_RESULT` 接受 | probe:room-join；`native_room` 4 測試 | probe 無；E1 走過同一步驟 |
| 22 | room／room-ready | 保留 | 準備狀態廣播。**注意**：E149 單人準備後是否出現 bot 取決於 harness 的 `testBotEnabled`／`testRoom`（§2 第 3 點），重跑時要把這兩個值記進收據 | probe:room-ready | probe 無；E1 有雙人準備 |
| 23 | room／room-leave | 保留 | `C_LITO_EXIT_ROOM` 回頻道 | probe:room-leave | 無 |
| 24 | room／room-rejoin | 保留 | 離房後再進同一房 | probe:room-rejoin | probe 無；E1 有「結算後重入同房」 |
| 25 | room／room-reject | 保留 | 進房被拒（失敗碼）後留在頻道 | probe:room-reject | 無 |
| 26 | room／room-retry | 保留 | 被拒後重試成功 | probe:room-retry | 無 |

### 3.3 世界內單人情境（保留 16）

這些情境只需要自己、地面或背包。2026-10-13 已把「等待 actor 2」的前置拿掉（下表 #33、#36–#42），或本來就不需要對手。

| # | 群組／mode | 裁定 | 原因 | 現行對應 | E149 結果 |
|---:|---|---|---|---|---|
| 27 | motion／room-world-move | 保留 | 八方向移動送包與預測 | probe:room-world-move；`native_motion` 6 測試 | 無 |
| 28 | motion／room-world-correct | 保留 | 伺服器校正取代預測 | probe:room-world-correct | 無 |
| 29 | navigation／room-world-route | 保留 | 依原始地圖尋路 | probe:room-world-route；`native_navigation` 8 測試 | probe 無；E1 有「依正式地圖導航靠近對手」 |
| 30 | actor-states／room-world-states | 保留 | 純畫面 fixture：明確注入展示用角色，不是戰鬥對手。範圍維持「只驗顯示」 | probe:room-world-states；`native_actor_states` 8 測試 | 無 |
| 31 | label-updates／room-world-labels | 保留 | 同上，稱號／血盟標記的顯示 fixture | probe:room-world-labels；`native_label_updates` 3 測試 | 無 |
| 32 | combat／room-world-refused | 保留 | 攻擊對象是場上真實的修理 NPC，伺服器拒絕，不需要對手 | probe:room-world-refused | 無 |
| 33 | skills／room-world-shield | 保留 | 對自己施法 | probe:room-world-shield；`native_skills` | 無 |
| 34 | item-actions／inventory-actions | 保留 | 頻道內的背包操作，不進戰鬥 | probe:inventory-actions；`native_item_actions` | 無 |
| 35 | item-actions／inventory-actions-refused | 保留 | 同上的拒絕分支 | probe:inventory-actions-refused | 無 |
| 36 | item-actions／room-world-item-drop | 保留 | 丟物品到地面 | probe:room-world-item-drop | 無 |
| 37 | item-actions／room-world-item-drop-refused | 保留 | 丟物品被拒，背包不變 | probe:room-world-item-drop-refused | 無 |
| 38 | reagent-skills／room-world-stream | 保留 | 材料技能（生命之泉）成功 | probe:room-world-stream；`native_reagent_skills` 7 測試、`native_t028_ground` | 無 |
| 39 | reagent-skills／room-world-stream-no-reagent | 保留 | 沒有材料被拒 | probe:room-world-stream-no-reagent | 無 |
| 40 | reagent-skills／room-world-stream-no-mana | 保留 | MP 不足被拒（harness 把 maxMp 設成 2） | probe:room-world-stream-no-mana | 無 |
| 41 | ground-pickup／room-world-pickup | 保留 | 丟出後撿回 | probe:room-world-pickup；`native_ground_pickup` 6 測試 | 無 |
| 42 | ground-pickup／room-world-pickup-stale | 保留 | harness 在伺服器記憶體移除地面物件但不廣播；客戶端兩次請求都沒有回覆，不得自行預測成功。同時是 §4.2 D7 與 §4.3 S6 的現成案例 | probe:room-world-pickup-stale；`native_ground_pickup::stale_pickup_choice_never_rebinds_same_id_or_new_owner` | 無 |

### 3.4 需要對手的情境（改寫 11、失效 1）

舊定義等待訓練 bot（actor 2）。E149 的 bot 是條件式（§2 第 3 點），而且 bot 的行為不等於真實玩家，所以一律改成兩個真實 `LoginController`（P1＝`native_two_player_battle::two_real_players_enter_same_battle_without_training_bot`，P2＝`…::two_real_players_attack_vitals_death_settlement_and_reentry`）。

| # | 群組／mode | 裁定 | 原因 | 現行對應 | E149 結果 |
|---:|---|---|---|---|---|
| 43 | world／room-world | 改寫 | 舊斷言是固定的世界角色數（含 bot）。改成「兩端都看到對方真實角色、沒有 bot ID 2」 | P1 | **E1 已驗證**（#174、#175） |
| 44 | combat／room-world-attack | 改寫 | 攻擊對象改成真實對手；HP 以伺服器回覆為準並與 Java 端觀測交叉核對 | P2 的 `melee-hurt` 階段 | **E1 已驗證**（#175） |
| 45 | combat／room-world-victory | 改寫 | 勝利改成由對手真實死亡與死亡確認觸發的結算 | P2 的 `death`、`settlement` 階段（勝方視角） | **E1 已驗證**（#175） |
| 46 | combat／room-world-defeat | **失效** | 兩個真實玩家的同一次執行裡，敗方視角已經包含在 #45 的同一場結算（#175 兩端計分表逐值相同）。單獨成案只會把同一場算兩次 | 併入 #45 | 隨 #45 |
| 47 | combat／room-world-cycle | 改寫 | 「打完一場再開一場」改成結算→回大廳→同連線重入同房。**缺口**：P2 的第二場只驗到再次施法，沒有打到第二次結算；舊版「兩場互換勝敗」在 E149 沒有結果 | P2 的 `returned-lobby`、`reentered`、`reentry-spell-hp-mp` 階段 | 重入：**E1 已驗證**；第二場結算：無 |
| 48 | combat／room-world-ranged | 改寫 | 遠程攻擊對象改成真實對手 | probe:room-world-peer-a／b＋`verify-native-real-peers.ps1 -ActionProfile ranged`（舊 runner）；Rust 端 `native_ranged` 17 測試 | 無 |
| 49 | skills／room-world-heal | 改寫 | 對手改成真實玩家；2026-10-13 的取代版標註「真實玩家動作＋宣告過的初始 fixture」，fixture 內容要在重寫時寫進收據 | 同上 `-ActionProfile heal` | 無 |
| 50 | attack-skills／room-world-bolt | 改寫 | 對真實對手施放 Energy Bolt，核對 HP／MP | P2 的 `spell-hp-mp` 階段 | **E1 已驗證**（#175） |
| 51 | attack-skills／room-world-bolt-refused | 改寫 | MP 不足時伺服器拒絕（harness 把 maxMp 設成 2）；目標改成真實玩家 | `-ActionProfile bolt-refused`（舊 runner） | 無 |
| 52 | ground-skills／room-world-wall | 改寫 | 地面技能（技能 58）；在場的對手改成真實玩家 | `-ActionProfile wall`；Rust 端 `native_ground_skills` 7 測試 | 無 |
| 53 | ground-skills／room-world-wall-refused | 改寫 | MP 不足被拒 | `-ActionProfile wall-refused` | 無 |
| 54 | item-actions／room-world-item-heal | 改寫 | 用物品補血；對手改成真實玩家，同 #49 要記錄初始 fixture | `-ActionProfile item-heal`；probe 另有 `room-world-item-heal-194／197` | 無 |

### 3.5 統計

| 裁定 | 案數 | 編號 |
|---|---:|---|
| 保留 | 42 | 1–42 |
| 改寫 | 11 | 43–45、47–54 |
| 失效 | 1 | 46 |
| 合計 | 54 | |

E149 結果：5 案的改寫版在 E1 已驗證（43、44、45、47 的重入部分、50）；其餘 49 案沒有 E149 的情境結果。42 個保留案的 probe 在 E149 一次都沒跑過。

## 4. 補充情境定義

舊 54 案沒有涵蓋、或只涵蓋一小部分的五類。每案列出前置、步驟、預期，以及 repo 內現成的入口。「層級」欄：**M**＝只有 Rust 端的封包／狀態測試；**T**＝有真實 TCP 與 Java harness 的入口；**缺**＝還沒有入口，第 2 張要補。

共通預期（每案都要檢查，不逐案重複）：
- 客戶端不得自行預測成功：沒有伺服器回覆就不改 HP／MP／背包／位置／房間成員。
- 失敗後控制器要停在明確的狀態（Failed 或回到上一個穩定階段），worker 回收，可以再操作。
- 測試結束時 harness 的 session 與房間成員清空。

### 4.1 拒絕（R）

| ID | 情境 | 前置與步驟 | 預期 | 現成入口 | 層級 | E149 結果 |
|---|---|---|---|---|---|---|
| R1 | 版本被拒 | 握手時伺服器回拒絕 | 回報版本被拒、不送帳密 | 舊 #2 | T | 無 |
| R2 | 登入被拒後重試 | 錯的帳密→失敗→正確帳密 | 第一次保留完整失敗內容；第二次成功，沒有殘留第一次的狀態 | 舊 #8、#9 | T | 無 |
| R3 | 選角被拒 | `S_LITO_SELECT_CHAR_RESULT` 回 1（靜默拒絕）、7（參加限制）、8（名稱不合法） | 留在選角畫面，可改選；7 對應舊 #20 | 舊 #20；`native_character_flow::second_source_eight_characters_second_page_refusal_retry_and_return` | 7：T；1、8：M | 無 |
| R4 | 進頻道被拒 | `S_LITO_ENTER_CHANNEL_RESULT` 回 1（未開放）、10（等級限制） | 留在大廳，頻道人數不變 | `native_lobby::deployed_channel_results_are_exact_and_keep_full_unsigned_tails` | M | 無 |
| R5 | 進房被拒 | `S_LITO_JOIN_RESULT` 拒絕碼。動態至少做：3 房間已滿、4 已開戰、5 已關閉、7 被踢、11 密碼錯；其餘（1、10、12、13、14、20、21）以封包向量驗 | 留在頻道；不建立房間成員；先到的 `S_LITO_ROOM_USER` 不得留到下一次進房 | 舊 #25、#26（只有一種拒絕碼，哪一種要看 harness）；`native_room::room_envelopes_match_deployed_jar_and_reject_partial_or_trailing_fields`、`native_early_roster::pending_roster_cannot_cross_connection_room_rejection_or_new_request` | 一種：T；其餘：M 或缺 | 無（`native_early_roster` 3 項 E2） |
| R6 | 房間設定被拒 | 非房主或已開戰時送 `C_LITO_EDIT_ROOM`、`C_LITO_GAME_INFO` subtype 2／3 | 伺服器單播 subtype 0＋1（契約 §5.3 發送時機 A–C）；客戶端要接受 0＋1 當作回覆，還原顯示 | `native_room_admin` 4 測試、`native_room_mode` 2 測試 | M | 無（`native_room_mode` 2 項 E2） |
| R7 | 戰鬥內操作被拒 | 攻擊不可攻擊的對象、MP 不足、沒有材料、丟不掉的物品、死亡後攻擊或移動 | 沒有本地預扣；死亡後的請求不增加送出佇列 | 舊 #32、#35、#37、#39、#40、#51、#53；死亡後拒絕在 P2 | T | 死亡後拒絕：E1；其餘無 |
| R8 | GAME 未授權 | 不經 LOGIN 交接直接連 GAME；或交接的端點與登入回覆不符 | 不進選角 | 舊 #13；`native_character_flow::mismatched_redirect_cannot_start_game_handoff` | T／M | 無 |

### 4.2 延遲或漏回覆（D）

| ID | 情境 | 前置與步驟 | 預期 | 現成入口 | 層級 | E149 結果 |
|---|---|---|---|---|---|---|
| D1 | 握手沒有回覆 | 連上後對方不送 seed | 到期失敗（#175 記錄的握手期限是 5 秒）；可取消；取消後能重連，舊 worker 不殘留 | 舊 #6；`native_network::stalled_seed_can_cancel_and_reconnect_without_stale_owner_or_thread` | T | 無 |
| D2 | 封包只到一半 | 長度欄或本體不完整就停住或斷線 | 回報 PartialHeader／PartialBody，不把半包當成完整封包 | 舊 #4、#5 | T | probe 無；Rust 端 E2 |
| D3 | 選角沒有回覆或目錄不完整 | 送 `C_LITO_SELECT_CHAR` 後不回，或角色目錄只到一部分 | 有界等待後終止，不重送；目錄不完整時不開放選角 | `native_teleport_failure::c8_timeout_and_cancellation_are_terminal`、`c8_waits_for_complete_source_directory`、`partial_inventory_never_retries_c8`、`natural_eight_second_deadlines`（都是 lib `native_login::teleport_wire_tests` 的 wrapper） | M | 本體在 lib，CI 的 `cargo test --lib` 會跑（本張沒有逐項對 CI log）；E149 動態無 |
| D4 | 進房沒有回覆 | 送 `C_LITO_JOIN` 後不回 `S_LITO_JOIN_RESULT`，但 `S_LITO_ROOM_USER` 先到 | 先到的名單只能暫存，要等對應的接受才生效；換連線、被拒或改送別的請求就丟掉；暫存上限 256 筆。**待核對**：G03 要求驗「期限」，現有三個測試沒有以時間為條件的案例，暫存有沒有期限、多長，第 2 張要先讀 `native_room.rs` 的 `EarlyRoster` 再定 | `native_early_roster` 3 測試；動態 harness 缺 | M；T 缺 | Rust 端 E2 |
| D5 | 攻擊或施法沒有回覆 | 送出後伺服器不回 | 回饋要分得出「逾時」與「被權威中斷」；按住攻擊不能在沒有回覆時連發；不產生 buff、扣費或成功動畫 | `native_combat::attack_feedback_distinguishes_reply_timeout_and_interrupted_authority`、`held_attack_waits_for_authority_and_animation_and_stops_on_release_timeout`；`native_t021::rejection_and_timeout_never_fabricate_costs_buff_or_refinement_success`；`native_t027_batch2::rejection_capacity_and_silent_timeout_do_not_create_success`；`native_t028_targets::all_ten_pending_timeout_boundary_and_server_rejection_precedence` | M；T 缺 | 無 |
| D6 | 回覆遲到 | 等待期限過後才收到回覆（徽章、施法） | 遲到的回覆不能套到新的請求或新的場景 | `native_emblems::bounded_requests_silent_miss_late_response_and_atomic_rejection`；`native_t027_batch3::summon_late_rejection_survives_cast_action_and_unrelated_messages` | M | 無 |
| D7 | 拾取沒有回覆 | 舊 #42 | 兩次請求都沒有回覆時，不播 action 15、背包不增加、地面物件不被本地刪除 | 舊 #42 | T | 無 |
| D8 | 傳送目的端沒有回應 | 目的端接受連線但不回，或不接受 | 有界失敗，舊權威退役，可以重新登入 | G06 `timeout`；`native_teleport_failure::silent_continuation_is_bounded` | T | **E3 已驗證**（timeout） |

### 4.3 過期世代（S）

「世代」指客戶端用來隔離舊資料的計數：連線 epoch、世界 generation、房間成員身分、背包擁有者、傳送交接。共通預期：屬於舊世代的封包、畫面、選取或回呼，不能改動新世代的狀態，也不能授權新世代的輸入。

| ID | 情境 | 前置與步驟 | 預期 | 現成入口 | 層級 | E149 結果 |
|---|---|---|---|---|---|---|
| S1 | 舊連線的封包 | 取消連線後，舊 socket 還有一批已收到但未處理的封包；接著重新登入 | 舊封包全部丟棄，不改目的地、不重送退出 | `native_network::accepted_connection_keeps_socket_until_explicit_cancel_then_reuses_epoch`；`native_teleport_controller` 的 `cancel_discards_old_inbound_burst_before_fresh_login`、`coalesced_old_socket_burst_cannot_retarget_or_repeat_quit`、`retired_endpoint_duplicate_and_old_world_packets_do_not_change_quit_barrier` | M（自有 loopback） | `native_network` 項 E2；teleport_controller lib 版 CI 有跑 |
| S2 | 舊世界的畫面 | 同角色、同座標回到同一張地圖（重登或重入） | 世界 generation 不重用；舊 Frame 的移動、輸入、死亡確認授權全部失效 | `native_world::clearing_authority_does_not_reuse_a_prior_render_generation`、`native_motion::old_render_cannot_publish_after_map_generation_or_connection_changes`；動態：probe `recovery-world`、P2 的 `reentered`（generation 1→2） | M＋T | 重入：E1；重登：無 |
| S3 | 舊房間的通知（換房隔離） | 離開 A 房後進 B 房（或重進 A 房），A 房的 `S_LITO_EXIT_ROOM`／`S_LITO_ROOM_USER`／S_D4（換隊通知）晚到 | 晚到的通知不能改 B 房的成員、隊伍或準備狀態；已退役成員的通知只消耗一次 | `native_room_departures` 2 測試；`native_room_force::incomplete_roster_force_is_bounded_and_cannot_cross_room_or_connection`、`captured_force_before_roster_does_not_create_a_member_or_override_full_snapshot`；`native_early_roster::pending_roster_cannot_cross_connection_room_rejection_or_new_request` | M | `native_room_departures`、`native_early_roster` 項 E2；`native_room_force` 無 |
| S4 | 舊選角的背包 | 重新選角或被拒絕的切換 | 背包屬於目前選定的角色與連線；切換被拒時回復原狀 | `native_inventory::selected_connection_and_reselection_own_inventory_not_world_generation`、`same_character_resume_without_replay_keeps_authority_and_rejected_switch_rolls_back`；舊 #15 | M＋T | 無 |
| S5 | 舊的傳送交接 | 交接過期、取消後重試、目的端回的是別的世代 | 過期的交接要重新選角；暫存的角色只能用一次；跨世代的回覆被拒 | G06 `cancel-expiry`、`cancel-retry`、`cancel-expiry-source`；`native_teleport_failure::stale_generation_endpoint_in_recovery_game`、`cross_generation_continuation_replies` | T＋M | cancel-expiry、cancel-retry：**E3 已驗證**；cancel-expiry-source：無 |
| S6 | 舊的選取（同 ID 換了東西） | 選取目標後，該 ID 的物件被移除又以同 ID 出現，或場景換了 | 選取失效，要重新選；不把請求送到新物件 | `native_ground_pickup::stale_pickup_choice_never_rebinds_same_id_or_new_owner`；`native_actor_states::hidden_targets_are_rejected_until_visible_and_identity_replacement_invalidates_selection`；各技能批次的 `…reject_removed_hidden_dead_and_replaced_targets` | M | 無 |

S3 與「S_CE／S_D4／S_CD 順序重播」是同一件事的兩面：第 2 張的重播要以真實擷取的 `S_LITO_JOIN_RESULT`（0xCE）、`S_LITO_ROOM_USER`（0xCD）、S_D4（換隊通知；`native_room.rs:162` 註明正式伺服器會把 S_D4 插在 S_CE 與 S_CD 名單之間）在不同到達順序下驗證人數上限（256）、期限與換房隔離。

### 4.4 斷線（X）

| ID | 情境 | 前置與步驟 | 預期 | 現成入口 | 層級 | E149 結果 |
|---|---|---|---|---|---|---|
| X1 | 房間內自己斷線 | 進房後關閉自己的 GAME socket | 控制器回報失敗、worker 回收；伺服器端房間、頻道、來源交接清空 | probe `recovery-room`＋`NativeRecoveryHarness`（`verify-native-recovery.ps1 -Mode room`，舊 runner） | T | 無（只有 BBC59308 的結果） |
| X2 | 戰鬥中自己斷線 | 開戰並載入世界後關閉 GAME socket | 同上；伺服器的離線清理是延遲的，harness 要有界等待清空 | probe `recovery-world`（`-Mode world`） | T | 無（只有 BBC59308 的結果） |
| X3 | 登入階段對方斷線 | 版本回覆後對方立刻 EOF，後面黏著一個封包 | 已收到的完整封包照常處理，然後回報 EOF | `native_network::coalesced_post_version_packet_survives_peer_eof_and_owner_retirement` | M（自有 loopback） | E2 |
| X4 | 對手在戰鬥中斷線 | 雙玩家開戰後，其中一端關閉 GAME socket | 留下的一端收到對手離場與斷線結算，回大廳後狀態清空 | **缺**。P2 結束時會取消連線，但 #175 明寫「取消期間產生的斷線勝負不計入驗收」 | 缺 | 無 |
| X5 | 房主在房間內斷線 | 兩人在房，房主關閉 GAME socket | 留下的一端收到 `S_LITO_EXIT_ROOM` 與 `S_LITO_GAME_INFO` subtype 5（房主轉移，契約 §5.3 發送時機 D） | **缺** | 缺 | 無 |
| X6 | 斷線後不能再送操作 | 斷線狀態下按換隊、準備、攻擊 | 請求不進佇列 | `native_room_force::disconnected_controller_cannot_queue_a_force_change`；`native_room::room_actions_require_current_server_lobby_and_membership` | M | 無 |
| X7 | 傳送途中目的端拒絕或不是遊戲服務 | 目的端沒有 listener，或回的不是 LiTo 握手 | 有界失敗，舊權威退役，可以重新登入 | G06 `refused`、`wrong-service` | T | **E3 已驗證** |

### 4.5 重新登入（L）

| ID | 情境 | 前置與步驟 | 預期 | 現成入口 | 層級 | E149 結果 |
|---|---|---|---|---|---|---|
| L1 | 房間斷線後重登 | 接 X1：同一個 `LoginController` 重新登入→選來源→選角→進房 | 連線 epoch 前進；第二次進房的身分、座標、HP／MP 與伺服器觀測一致；結束後清空 | probe `recovery-room` 後半 | T | 無（只有 BBC59308 的結果） |
| L2 | 戰鬥斷線後重登再開戰 | 接 X2：重登後再進房、開戰 | 世界 generation 不重用（舊紀錄是 epoch 2→4、generation 1→2）；重連前的 Frame 不能操作新世界 | probe `recovery-world` 後半；`native_world::clearing_authority_does_not_reuse_a_prior_render_generation` | T＋M | 無（只有 BBC59308 的結果） |
| L3 | 伺服器還沒清完就重登 | 接 X2，但不等清理完成就立刻重登 | **待核對**：E149 是拒絕、踢掉舊 session 還是排隊，目前沒有靜態或動態資料。先以 javap 讀 `ClientSession` 的登入與斷線路徑，再定預期 | **缺** | 缺 | 無 |
| L4 | 傳送失敗後重登 | 接 D8／X7／S5：失敗或取消後明確按登入 | 只保留一份已驗證的恢復狀態；重複取消不產生第二份；一次登入接回原角色 | G06 八案；`native_teleport_failure::second_cancel_then_explicit_login`、`repeated_cancel_keeps_one_recovery`、`refused_then_repeated_cancel_keeps_recovery`、`late_cancel_keeps_recovery` | T＋M | 五案 **E3 已驗證**；cancel-repeat、refused-cancel-repeat、cancel-expiry-source 無 |
| L5 | 正常退回選服後換來源再登入 | 頻道 selection 100 回選服→選另一個來源→選角 | 舊來源的角色目錄與背包清空；第二個來源的 8 角色分頁正確 | 舊 #18、#19；`native_character_flow::second_source_eight_characters_second_page_refusal_retry_and_return` | T＋M | 無 |
| L6 | 結算後不斷線重入 | 結算→回大廳→同連線進同房 | 同一個 GAME epoch、世界 generation 前進、死亡與戰鬥計數清空 | P2 | T | **E1 已驗證** |

## 5. 現行必備情境與第 2 張的工作

「現行必備」＝ §3 的保留 42 案＋改寫 11 案（#46 併入 #45），加上 §4 的 R1–R8、D1–D8、S1–S6、X1–X7、L1–L6。其中與舊案重複的（R1、R2、R7 的大部分、R8、D1、D2、D7）不另外計數。

依目前證據分三組：

| 組 | 內容 | 第 2 張要做的 |
|---|---|---|
| A. E149 已有動態結果 | #43、#44、#45、#47（重入）、#50；D8、X7、S5 的兩案、L4 的五案、L6 | 在當前 main 重跑一次（`native_two_player_battle` 兩個 ignored 測試、G06 八案），取得當前 EXE 的結果 |
| B. 有入口、沒有 E149 結果 | 保留 42 案的 probe；#48、#49、#51–#54（`verify-native-real-peers.ps1` 的 action profile）；X1、X2、L1、L2（`recovery-*`）；G06 後補三案；§4 標 M 的 Rust 測試 | 先把 runner 改成 owned 模式（§2 第 1 點），再逐案跑。M 類以 `--exact` 逐項跑並確認 `running 1 test` |
| C. 沒有入口 | #47 的第二場結算；R5 的多種拒絕碼動態案；D4、D5 的動態案；X4、X5；L3 | 要新寫 harness 步驟或測試。L3 要先做 javap 靜態核對才能定預期 |

計數規則沿用舊矩陣的 `counting_rule`：取代不是案數；不同 EXE 或不同 JAR 的結果不能相加；略過與 filtered out 不算通過。
