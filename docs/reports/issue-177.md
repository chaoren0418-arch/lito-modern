# Issue #177：T044-2 動態碰撞、場地邊界與持續移動中斷

本張只處理 T044 第 2 張，不將 T044 或完整 G04 標為完成。沿用 #176 的移動鎖握手、畫面抵達與送出前檢查。

## 做了什麼

- `Navigation::can_step_with_edges` 將最新 `S_CHANGE_ATTR` 與 LTmap Layer3 方向邊合併後，再計算正交／對角通行。未綁定門物件的更新也能作為場地邊界；不把方向邊誤判成整格物件。開放更新覆寫對應靜態邊，但不能創造缺失的 sector，也不能超出 u16 座標範圍。對角線的中繼格也必須已載入。
- `World::checked_move_request` 使用上述合併結果，每次送出都讀取當前世界的動態占位。一般門與 Siege War 內門沿用既有門／屬性封包；已確認 owner 的競技場門按伺服器規則只阻擋目的格，其屬性不另作方向牆，避免關門困住已在門格的角色或開門清除靜態地形。多格城門可直接使用入口各格的方向邊更新，不從 sprite 位置猜矩形。
- 動態物件增加 Quest Battle 與 Time Attack 的存活怪物占位；只阻擋目的格，不將旁邊怪物當成靜態牆。沿用守護塔的 0／32／33／34 阻擋、35 解除規則。一般玩家、隨從及未符合契約的物件不新增占格規則。
- 怪物辨識同時需要伺服器房間模式（2／4）、對應的 `0x6C`／`0x6D` 物件區段及原生走路物件格式。控制器從已接收的房間資料設定模式，在一般 WORLD 清除；不由 map ID、名字、HP 比例或 sprite 單獨猜測。租裝道具重用部分 ID 區段，因此未知／其他模式不啟用此規則。
- `LoginController::continue_movement` 統一 GUI 的持續移動流程：無畫面／尚在途先等待，權威中斷或非活動狀態清除輸入，抵達後重新檢查下一步；碰撞或送出失敗後清除按住意圖，障礙解除不會自行重送。
- 本地角色的新權威動作中斷持續移動；與連續攻擊的意圖分開。保留已送步伐的 `position_predicted`，不以受傷、動畫抵達或解除阻擋冒充移動 ACK。觀戰開啟／關閉即使發生在兩次取樣之間，也會退休舊輸入。
- **第 2 次嘗試（審查退回第 1 點）：延遲命中受傷也中斷持續輸入。** 一般箭矢（`flight_type=0x7f`、`graphics=66`、`damage_marker>0`）在 `native_world.rs` 刻意不更新受擊者 action sequence，受傷由畫面的投射物抵達釋放，因此舊版 `synchronize_action` 看不到。修正：
  - `native_world_scene.rs` 新增 `dispatch_impact`，`Model::draw` 的所有抵達（箭矢、法術、class1、disintegrate、call lightning、cone cold、many）都經它呼叫原本的 `Animator::impact`；只有**被接受**且目標是本地角色（非觀戰）時才累加 `ImpactStamp`。命中時序、`impact` 內的死亡／incarnation／佇列上限／守護塔檢查完全不變。
  - `ImpactStamp`（`native_navigation.rs`）只遞增、不重置；`Model` 換世代或換圖重建時保留計數，避免重建後計數回到相同值而漏判。每個 `Frame` 帶出目前值。
  - `HeldInput::synchronize_impact`：第一次只建立基準，之後值改變即 `clear()`；`continue_movement` 在有畫面時呼叫。中間畫面被略過也不會漏（計數留在之後每一幀）。
  - 命中其他角色不改變本地計數，本地按住輸入保留。

## 已驗證

### 第 2 次嘗試（2026-11-12）

依使用者 2026-11-09 規定，本機**只編譯、不執行**測試程式；測試執行交給 PR 的 GitHub Actions（`build.yml` 的 `cargo test --locked --lib --no-fail-fast` 涵蓋本張全部 lib 測試，含新案例）。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
exit=0   （Finished `dev` profile；沒有 test result:，既有警告仍在，修改檔案無新增警告）

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-run
exit=0   （只編譯，未執行；沒有 test result:，不算通過）
```

新增回歸 `collision_tests_delayed_arrow_hurt_on_local_body_retires_held_walk_but_bystander_hit_does_not`（`native_login_movement_tests.rs`）走真實路徑：`World::accept` 解 29 位元組 S_ATTACK 箭矢封包 → `Animator::pose` 由合成 sprite 目錄的攻擊影格產生 `Hit` → `native_projectiles::Projectiles::update` 發射並在抵達時產生 impact → `dispatch_impact`（與 `Model::draw` 同一函式）→ 依本地姿勢是否 `idle` 建立 `Frame` → `continue_movement`。驗證：封包不改受擊者 action sequence；命中旁觀者被接受但本地輸入保留並送到 C_MOVE 送出端；命中本地時受傷姿勢期間不可移動、輸入已清空；受傷動畫結束後舊輸入仍為空、`predicted_moves==0`；重新按下才再送到送出端。**本案例的執行結果待 CI**，不在本機宣稱通過。

以下為第 1 次嘗試的本機紀錄（規定生效前）：

2026-11-06，在本 worktree 的 `LiTo381-Modern/` 執行。每個子程序清除 `LITO_*`，暫存置於 worktree 的 `scratch/tmp/`，產物置於 `target/current/`。沒有啟動遊戲或正式服務。

結果直接使用 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult` 原函式：結束碼 0、恰好一組可解析且數量相符的 running／result、不得有 0 tests 或 failed。以下八組測試皆為 PASS。19 個碰撞測試中的 7 個控制器案例也包含在 21 個移動測試內，故 **78 次通過、71 個不同測試、0 failed、0 ignored**，不重複加總。

### 編譯檢查

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

最終版本重跑結束碼 **0**，門檻結果 **PASS**。此指令沒有 `test result:`；專案既有警告仍存在，不宣稱零警告。

### 新增碰撞與持續輸入測試

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib collision_tests
exit=0
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 2490 filtered out; finished in 0.18s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib movement_tests
exit=0
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 2488 filtered out; finished in 0.20s
```

涵蓋 1,024 組四方向對角線靜態／動態邊組合、正交雙向更新、最新開關覆寫、u16 與缺 sector／中繼格、城門多格方向邊、門格八方向進出、怪物八方向占位、跨模式反例、格式錯誤原子性，以及新障礙／受傷／觀戰切換／瞬間限制／HP 歸零後的輸入退休。既有加密 loopback 案例改走共用 `continue_movement`，確認真實 TCP owner 只送一次在途 C_MOVE、鎖解除後不恢復舊按住意圖、重新按下才送下一步。

首輪有 1 個測試前置失敗（`stale GAME environment update`）：使用時間封包觸發了與本案例無關的環境世代 gate。改以 HP 更新封包驗證房間模式傳遞後，最終 19 個全部通過。初輪不計通過，原始失敗日誌保留於 `initial-collision.log`。

### 既有回歸

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_navigation
exit=0
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_doors
exit=0
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_obstacles
exit=0
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.23s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_motion
exit=0
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_combat
exit=0
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_animation
exit=0
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`native_navigation` 包含原始 map564 的 32,768 個方向判斷與既有正式 oracle 比對，以及 map510 跨 sector 路線。`native_obstacles` 包含原始守護塔資源／動作狀態回歸。測試中的合成 World／Frame／地形不是 GUI 驗收。

`git -c core.whitespace=cr-at-eol diff --check` 通過。沒有改寫既有檔案的整份行尾或執行全庫格式化。

## 推測與來源核對

唯讀核對相鄰 `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`，沒有修改或執行伺服器：

- `ClientSession.java:26820` 的 Tournament C_MOVE 分支依次檢查場地／原生邊、任務邊界、一般門、Siege War 內門、守護塔、Quest 怪物、Time Attack 物件；沒有一般玩家占格拒絕。`sendSiegeGate` 使用與一般門相同的 `battleDoor`／`doorAttribute`。
- `Protocol.java:15735`、`:15769` 的怪物 producer 使用 WALK=0；`BattleQuestEncounter.java:1235`、`BattleTimeAttackEncounter.java:1146` 只依存活與目的格占位。角色死亡、移動、替換與移除沿用 World 現有處理，不新增另一份占位快取。
- `BattleQuestBoundary.java` 說明方向邊必須合併後計算對角通行。其 573／574 固定邊界配置明列為隔離實驗，因此本張**沒有搬入該配置**，只消費實際收到的 `S_CHANGE_ATTR`。
- `ArenaMapCatalog` 以已載入 sector 集合限制場地。觀戰用 `S_LITO_BOUNDS` 不會被套成玩家移動範圍。
- 多格城門方向邊依 `ClientSession.java` 實際的逐格 `Protocol.doorAttribute(x, y, direction, passable)` producer 支援：`sendDwarfCastleDoorAttributes`（第 4993–5005 行，`direction==0` 時沿 x 走 `edgeLow..edgeHigh`、否則沿 y）、`sendAdenCastleDoorAttributes`（第 5034–5044 行，固定 `entranceY`、沿 x）、`sendHeineCastleDoorAttributes`（第 5066–5076 行，固定 `y()+1`、沿 x），以及同形式的 `sendMainlandCastleDoorAttributes`（第 29376 行起）。四者每格各送一個 `S_CHANGE_ATTR`，客戶端只消費這些方向邊、不從 sprite 推矩形。（前一版誤寫為不存在的 `sendClassicCastleDoorAttributes`，第 2 次嘗試已依同一 SHA256 的檔案重新核對並更正。）傳統 WORLD 的額外伺服器城門控制區（包含未逐格傳送的延伸格）仍由伺服器裁定；本張不聲稱重建所有傳統 WORLD 城堡資料庫規則。

上述是本機來源的靜態核對，不是執行正式 JAR 的證據。來源 SHA256：

| 檔案 | SHA256 |
|---|---|
| ClientSession.java | `6CE00B818E73BF509152A460DAB4831FD5EEFC29E6344E7081C4948D40046F8B` |
| Protocol.java | `9A0A1E67FF94BD08C3FA5403582807F1A5D0CB74E28AED2ABCB4E7788107F213` |
| BattleDoor.java | `B716AF8820A4371A032FA01B635D74562CA44BA88B914A2B653BE81E2148CBD5` |
| BattleQuestBoundary.java | `F629E61E19B40B443A2F605B4287CAA00D6781834F8D82FEB4C969C5FFF8353B` |
| BattleQuestEncounter.java | `3A2E35E26DCBC5C551DAA16CC2BACD3EED9CB6334395E8B2D69F8CCBB0234F96` |
| BattleTimeAttackEncounter.java | `5C7DCBF4A3E74069E0F48487F8BF6E73822A77D55CC45D0650D1B721FCCCCC3C` |
| ArenaMapCatalog.java | `178D9DC2B5F88C541E93C793828CB8FD67E737383500715BCB8B090470D189B0` |

## 待驗證／未驗證什麼

- 正式 JAR 的多模式雙端碰撞、自然怪物 AI、實際城門開關時序，以及 GUI 實體鍵盤、焦點與動畫驗收。本輪不啟動遊戲、不連正式服務或 DB。
- 仍無伺服器每步成功 ACK；在途封包不能撤回。收到障礙更新時，只阻止後續送出，不自行回滾已送步伐。
- G04.4／G04.7 的完整進出場、死亡復活、換圖與同 ID 替換同步由 T044-3 處理。本張只測其既有狀態如何影響占位，不宣稱完成全部生命週期。
- 尚未收到的物件／場地狀態不能本地預判；保留伺服器權威修正。既有探針的路線搜尋仍以靜態地形及目的格避障為主，每一步經 `checked_move_request` 時再核對動態方向邊；不宣稱新增動態自動尋路。
- 完整測試庫與 Release 不在本輪驗證聲明內。
- 第 2 次嘗試的新案例與修改後的既有 lib 測試：本機只編譯，執行結果以 PR CI 的 `cargo test --lib` 為準。`tests/` 下的整合測試（`native_navigation` 等）CI 不跑，第 2 次修改後未重跑。
- 實機上箭矢命中時按住方向鍵的手感（受傷結束後需重按）待真機驗收。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_navigation.rs`
- `LiTo381-Modern/src/modern_lito/native_doors.rs`
- `LiTo381-Modern/src/modern_lito/native_obstacles.rs`
- `LiTo381-Modern/src/modern_lito/native_world.rs`
- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`
- `LiTo381-Modern/src/modern_lito/native_collision_tests.rs`
- `LiTo381-Modern/src/modern_lito/native_login_movement_tests.rs`
- `LiTo381-Modern/src/modern_lito/native_world_scene.rs`、`native_movement_tests.rs`（第 2 次嘗試）
- 本報告：`docs/reports/issue-177.md`
- 指令日誌：`LiTo381-Modern/target/current/verification/issue-177/`（不入 git）。

## 給下一棒的提示

- Claude 審查本張程式與證據；使用者決定合併。調度器接續 T044-3，不變更 T044 整項狀態。
- 新互動移動入口沿用 `continue_movement`／`move_presented`；`move_step` 是既有低階診斷入口，不提供地形或動畫抵達證明。
- 不要以所有 actor 的座標建立全域占位，不要單靠 `0x6C`／`0x6D` ID 區段套用怪物規則。保留房間模式來源與一般 WORLD 的清除行為。
- `S_CHANGE_ATTR` 無物件 ID，無法由任意 sprite 可靠反推多格城門所有權。T044-3 如擴充跨生命週期關聯，需維持無主方向邊可用、已退休 owner 不生效，以及換圖清除的規則。
- 新的抵達型受傷來源必須經 `dispatch_impact`，不要直接呼叫 `self.animator.impact`，否則本地持續輸入不會退休。
- `docs/REPORT.md` 保持未修改，由調度器合併後補索引。
