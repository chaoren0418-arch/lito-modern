# Issue #178：T044-3 G04.4 生命週期同步與 G04.7 其他模式物件

本張只處理 T044 第 3 張（G04.4 加入／離開、死亡復活、換圖、同 ID 替換；G04.7 其他模式的城門、任務怪物、限時模式物件與進出場）。不把 T044、G04.4、G04.7 或 G04 整項標為完成；狀態欄位由審查決定。

## 做了什麼

先以唯讀方式核對伺服器各模式的生命週期 producer 順序（見「推測與來源核對」），再逐一對照客戶端 `World`／控制器的處理。多數事件（S_ACTION 8 死亡不再占位、S_REMOVE 移除門屬性擁有權、同 ID S_PUT_OBJECT 換新 incarnation、S_MAP_ID 清空世界與房間資格）已正確，本張補上兩個實際缺口，並為各模式補齊生命週期回歸測試。

1. **攻擊等待跨生命週期退休（G04.4 同 ID 替換）**
   `native_combat::Pending` 原本只比對目標 ID：目標在等待期間被移除後以相同 ID 重新加入（Return-to-Nature 釋放、租屋傳送重建、屍體重送），或本地角色被同 ID 重建時，舊的攻擊等待仍會當成有效。
   - 新增 `Pending::capture(world, target, since)`，記錄送出當下的 `(自身 ID, incarnation)` 與目標 incarnation。
   - `observe` 在已計數的伺服器回覆（`own_attack_packets` 增加）之後才檢查生命週期：目標不在、目標或自身換了 incarnation → `Interrupted`。與既有 S11 復活退休攻擊等待的規則一致，也與 `native_skills::Pending` 已比對自身 incarnation 的做法一致。
   - `LoginController::attack_with_input` 改用 `capture`；既有測試兩處（`native_resurrection_tests.rs`、`tests/native_combat.rs`）改用同一建構函式，斷言不變。
2. **攻城戰內門不可由 C_OPEN 開關（G04.7 其他模式的城門）**
   伺服器在 Siege War（房間模式 3）只啟用 `room.siegeGate`，不啟用互動門 `room.door`；內門與競技場門同為 gfx 2083、同一 `0x62` ID 區段，客戶端無法由物件本身區分。`openBattleDoor` 只接受 `room.door`，其餘 C_OPEN 一律靜默忽略，原客戶端會送出後等 2 秒才顯示「無回應」。
   - `EncounterMode` 新增 `Siege`（`from_room_mode(3)`）；它沒有遭遇怪物命名空間，`encounter_actor` 對它回傳 false，占位規則不變。
   - `native_doors::operable(world)`：Siege 房間不可操作；`Choice::capture` 與 `Choice::request` 都檢查，先前選取的舊 Choice 在模式改為 Siege 後也不能送出。訊息「攻城內門須摧毀外側守護塔才會開啟」。
   - 內門的通行仍完全由伺服器 S_ACTION 28 + S_CHANGE_ATTR 決定（#177 的 owner 規則不變）。
3. **新增生命週期測試** `src/modern_lito/native_lifecycle_tests.rs`（掛在 `native_login` 下，模組名 `lifecycle_tests`，7 個案例）：
   - `lifecycle_tests_encounter_death_release_removal_and_map_change_track_each_life`：Quest（模式 2，`0x6c`）與 Time Attack（模式 4，`0x6d`）各跑一輪：出生占位 → S_ACTION 8 屍體不占位、不可攻擊、舊選取退休 → 馴服 S_REMOVE → Return-to-Nature 同 ID 重新加入後重新占位、舊選取仍退休、新選取有效 → S_MAP_ID 換圖後 actor／門屬性／房間資格清空、世代前進、無預測步伐。
   - `lifecycle_tests_corpse_reput_is_a_new_object_until_its_death_action`：屍體以 S_PUT_OBJECT + S_ACTION 8 重送（`ClientSession.java` classic bootstrap 順序）時，在 S_ACTION 8 之前是新物件、之後回到死亡；一般物件生死都不占位。
   - `lifecycle_tests_siege_inner_gate_opens_by_authority_and_survives_reentry`：關閉內門阻擋 → Siege 模式不能建立 Choice、同 gfx 在競技場模式可操作且舊 Choice 換成 Siege 後不能送出 → 兩座外塔摧毀的 S_ACTION 28 + 開啟屬性放行 → 晚加入者 bootstrap 以同 ID 重送開啟中的門時舊屬性擁有權退休、收到新屬性後恢復 → S_REMOVE 清除全部擁有屬性。
   - `lifecycle_tests_controller_qualifies_siege_from_room_mode`：控制器由已接收房間模式 3 設定 `Siege`，改為模式 1 後門恢復可操作。
   - `lifecycle_tests_attack_wait_retires_on_target_or_attacker_replacement`：目標同 ID 重送、移除後再加入、自身重建都中斷；已計數回覆仍為 `Acknowledged`；無關玩家加入／離開不影響等待。
   - `lifecycle_tests_controller_poll_interrupts_attack_after_same_id_reentry`：控制器 `accept_game` + `poll` 實際路徑：無關物件重送保留等待，目標移除再加入後等待清除、回饋為 `Interrupted`，階段不變。
   - `lifecycle_tests_local_removal_and_map_rebuild_retire_press_and_old_frame`：classic 死亡重生順序（自身 S_REMOVE → S_MAP_ID → 開始 → 狀態 → 自身 S_PUT_OBJECT）：移除時按住輸入退休、舊畫面不能送出；重建後舊畫面仍不能授權、舊按鍵不恢復，重新按下才走到 C_MOVE 送出端；`predicted_moves == 0`。

## 已驗證

依使用者 2026-11-09 規定，本機**只編譯、不執行**測試程式；測試執行交給 PR 的 GitHub Actions（`build.yml` 的 `cargo test --locked --lib --no-fail-fast` 涵蓋本張全部新案例與修改後的既有 lib 測試）。在本 worktree 的 `LiTo381-Modern/` 執行：

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
exit=0   （Finished `dev` profile；此指令沒有 test result:。專案既有警告仍在；本張修改／新增的檔案無新增警告）

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-run
exit=0   （只編譯，未執行；沒有 test result:，不算通過）
```

CI 的 `test result:` 原文待 PR 跑完後補在下方「CI 結果」；在此之前新案例**不算通過**。

### CI 結果

（待 PR CI 完成後補上）

## 推測與來源核對

唯讀核對相鄰 lito-launcher-server（commit `6bb4a27b`）`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`，未修改、未執行：

- 任務怪物／限時物件死亡只送 `S_HIT_RATIO` + `S_ACTION DIE`，不送移除（`ClientSession.java` 約 78610–78640、78940–78970）；`BattleQuestEncounter.blocks`／`BattleTimeAttackEncounter.blocks` 只看存活與目的格（`:1235`、`:1146`）。客戶端 `dead` 後不占位與之一致。
- 馴服：`publishQuestTaming`／`publishTimeAttackTaming` 送 `S_REMOVE_OBJECT reason=tamed`，寵物用 `COMPANION_SEQUENCE` 的新 ID（`ClientSession.java` 約 72600–72710），不沿用 `0x6c`／`0x6d`，因此不會被誤判成遭遇怪物。Return-to-Nature 先移除寵物，再以 `questMonster`／`timeAttackActor` 重送被釋放的怪物（約 50060–50090）——這是本張「同 ID 重新加入」案例的依據；重送的 ID 是否與原怪物相同屬**推測**（取自 `questReleased.objectId()`，未執行驗證）。
- Siege War：`GameHub.java:10139–10144` 只在模式 SIEGE_WAR 啟用 `siegeGate`、否則（Pet Race／Quest 以外）啟用 `door`；`openBattleDoor`（`ClientSession.java:27819`）只比對 `hub.battleDoor`；內門開啟為 `S_ACTION` + `S_CHANGE_ATTR` source=two-outer-towers-destroyed（約 78544–78560）；`BattleRules.Mode.SIEGE_WAR` wire 值 3。
- Classic 死亡重生先送自身 `S_REMOVE_OBJECT classic-death local-player`（約 15129），之後才進入新地圖。
- 晚加入者 bootstrap `sendBattleDoor`／`sendSiegeGate` 先 S_PUT_OBJECT 後 S_CHANGE_ATTR（`ClientSession.java:12692–12722`）。

來源 SHA256：

| 檔案 | SHA256 |
|---|---|
| ClientSession.java | `6CE00B818E73BF509152A460DAB4831FD5EEFC29E6344E7081C4948D40046F8B` |
| GameHub.java | `6A123687BB65FDCE4A6B7F8FEC8317DA0B3E949C48758F43766FAC2F9FE4886D` |
| BattleDoor.java | `B716AF8820A4371A032FA01B635D74562CA44BA88B914A2B653BE81E2148CBD5` |
| BattleRules.java | `85D39762551FF8503EAFC4B1E4BAD56582C7C454EC73445D1AE95C0642312000` |
| Protocol.java | `9A0A1E67FF94BD08C3FA5403582807F1A5D0CB74E28AED2ABCB4E7788107F213` |
| BattleCompanion.java | `A8B26B216B4048DC7B6F8AD5D22F6D3CBF9AACB90FE0B27A8D584D70C401D472` |
| BattleQuestEncounter.java | `3A2E35E26DCBC5C551DAA16CC2BACD3EED9CB6334395E8B2D69F8CCBB0234F96` |
| BattleTimeAttackEncounter.java | `5C7DCBF4A3E74069E0F48487F8BF6E73822A77D55CC45D0650D1B721FCCCCC3C` |

以上是靜態原始碼核對，不是執行正式 JAR 的證據。本張沒有引用 tw-bin-engine。本張不涉及客戶端資源（沒有宣稱缺資源），因此未查 pak。

## 待驗證／未驗證什麼

- 新增 7 個與修改後的既有 lib 測試：本機只編譯，執行結果以 PR CI 為準。`tests/native_combat.rs`（整合測試）只改用 `Pending::capture`，CI 不跑整合測試，本輪未執行。
- 正式 JAR 的多模式雙端實測：任務／限時怪物自然死亡、馴服與 Return-to-Nature、攻城內門實際開啟、晚加入者 bootstrap、classic 死亡重生、換圖。本輪不啟動遊戲、不連正式服務或 DB。
- 自身被移除後到下一個 S_MAP_ID 之間，伺服器端的移動鎖（S_BLINK 待確認）是否也重置；客戶端在 S_MAP_ID 一律重置為 Open，未與正式服務比對。
- 攻城內門的 GUI 提示（點選後顯示「須摧毀外側守護塔」）與目標名稱顯示未做畫面驗收。
- 完整測試庫、Release 與 G04 整體驗收不在本輪聲明內。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_combat.rs`（`Pending` 生命週期欄位與 `capture`）
- `LiTo381-Modern/src/modern_lito/native_obstacles.rs`（`EncounterMode::Siege`）
- `LiTo381-Modern/src/modern_lito/native_doors.rs`（`operable`、Choice 檢查）
- `LiTo381-Modern/src/modern_lito/native_login.rs`（改用 `Pending::capture`、掛載 `lifecycle_tests`）
- `LiTo381-Modern/src/modern_lito/native_lifecycle_tests.rs`（新增）
- `LiTo381-Modern/src/modern_lito/native_resurrection_tests.rs`、`LiTo381-Modern/tests/native_combat.rs`（改用 `Pending::capture`）
- 本報告：`docs/reports/issue-178.md`

## 給下一棒的提示

- Claude 審查本張差異與 CI 證據；使用者決定合併。G04.4／G04.7 的 `state` 是否升級由審查決定，本張沒有改 `docs/G01-G12.md` 或 `TASKS.md`。
- `EncounterMode` 現在也代表「已接收的房間模式資格」（含 Siege）。新增模式規則時沿用 `from_room_mode`，不要從 map ID、sprite 或 ID 區段推測模式。
- 新的「等待伺服器回覆」狀態（攻擊、技能、門、拾取…）都應記錄送出當下的物件 incarnation；只比對 ID 會讓同 ID 重新加入的物件冒用舊等待。
- 若要讓攻城內門在 GUI 顯示不同名稱，`native_combat::target_label` 目前不帶 world，需要另外傳入模式。
- `docs/REPORT.md` 保持未修改，由調度器合併後補索引。
