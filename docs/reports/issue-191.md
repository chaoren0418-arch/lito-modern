# Issue #191 — T055-2：rental-source（C_CA、S_FB 目錄、租裝村與商店交接、map500 入口種類）

## 做了什麼

本張只做 rental-source 模式的客戶端契約。返回／重新入村生命週期屬 T055-1（#190，已合併），本張沒有修改；也沒有修改伺服器、沒有套用隔離候選。

盤點結果：C_CA 選配裝 → S_CA → C_D9 → S_F0 → S_F5 → C_CF → 加入結果 → 進入 map500 的主流程在 main 已經接好（`select_rental_set`／`join_rental_room`，並有既有的正式 harness 探針）。本張補上三個與伺服器契約不一致、會送出或接受原生流程不可能產生之內容的缺口：

1. **C_CA 配裝選擇依職業限制**（`native_rental_source::item_set_choices`／`admit_item_set`）
   - 原本客戶端接受 index 0..=3，UI 固定列 4 組「配裝 1..4」。
   - 伺服器 `ClientSession.selectRentalItemSet`：職業 0／1／4 有 3 組，2／3 有 2 組，其餘 0 組；`RentalItemSetCatalog` 的 13 組 key 與此一致。職業取自所選角色 S_CB 的第一個數值位元組（伺服器 `Protocol` 寫入 `row.type()`）。
   - 現在超出範圍的 C_CA 由客戶端本地拒絕、不送出；沒有目錄列（職業未知）時也不送出。新增 `LoginController::selected_character_class()`、`rental_set_choices()`。
   - `native_rental_window.rs`：配裝下拉選單改成依所選角色職業填入 2／3 組（或 0 組），不再固定 4 組。
2. **S_FB 目錄在進村交接期間的守門，以及加入結果的模式檢查**
   - 在 `RentalArmPending`／`RentalAwaitPermission`／租裝 `RoomJoining` 期間收到的 S_FB，必須仍提供已快取的直連房 id，否則拒絕，而且不取代目前的目錄（伺服器在 C_CE 後會重送同一份目錄，`sendRoomList`→`sendLitoServerList`）。
   - 租裝路徑（已選配裝、未進入普通頻道）的 C_CF 加入結果必須是同一房間 id、mode 6、map500 或 map560（`ServerConfig` 只接受這兩張租裝村地圖；`joinRentalTown` 回 `litoRentalTownJoinResult`）。這可以避免把 Tournament 房間當成租裝村接受（符合 G06「不得以 Tournament 通過代替租裝」）。
3. **租裝村 S_PORTAL 入口種類分類**（`native_rental_source::classify`、`Snapshot.rental_town_transition`）
   - 原本 mode 6 的 S_PORTAL 完全不檢查目的地。
   - 依伺服器的三個產生點，只接受以下三種：
     - `GroundPortal`：C73 原圖 tag100 入口（`enterRentalTownPortal` 只接受同圖目的地；map500 有 24 處、map560 有 32 處來源，全部是同圖）。等待逾時後晚到的同圖回覆也接受。
     - `HotelEnter`：map500 → 501..503（`HOTEL_ENTER_ACTION`→`beginRentalTownMapTransition`）。
     - `HotelReturn`：501..503 → map500（手動返回、租約失效或到期）。
   - 其餘目的地（跨村 500↔560、從 560 進旅館、旅館之間互轉、非租裝地圖、原圖入口等待中卻收到非同圖目的地）一律拒絕、不送 C08。種類只在 World 接受 S_PORTAL 之後才寫入 snapshot，拒絕時狀態不變。選角、來源刷新、重置等清除 `rental_set` 的地方也一併清除種類。
   - 跨端點的 TeleportMe（S_F3／S_F4）與 C10 返回原本就有各自的路徑，本張未改。
4. **測試**
   - `native_rental_source.rs` 單元測試 2 案：職業配裝表、入口種類的接受與拒絕矩陣。
   - `native_rental_source_tests.rs`（合成加密 loopback，沿用 `teleport_wire_tests::FaultPeer`）3 案：
     - 職業 1／3／6 的配裝數，超出範圍不送 C_CA；合法時只送一次 `[0xCA, index]`；
     - 交接期間 S_FB 的接受與拒絕、加入結果的 mode 或地圖錯誤都拒絕，正常時進入 `RoomAwaitSelf`，送出的指令依序是 `CA D9 CF`；
     - 8 組「目前地圖→目的地」：接受的只送一次 C08，拒絕的不送 C08、World 沒有 pending portal。
   - `native_rental_lifecycle_tests.rs`：fixture 角色列的職業原本是 6（幻術師，沒有租裝配裝），改成 1（騎士），讓既有的重新入村流程符合新的職業表。
   - `native_peer_rental_probe.rs`（正式 harness 探針，預設不跑）：stage 0 原本送出 index 3，用來看伺服器拒絕；現在改成驗證本地拒絕、不送出。

引用（唯讀，lito-launcher-server main 工作樹 `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`）：`ClientSession.java`（`selectRentalItemSet`、`rentalItemSelectionContext`、`sendLitoServerList`、`sendRoomList`、`joinRentalTown`、`quitFight` 的 rental-town direct-room 分支、`enterRentalTownPortal`、`beginRentalTownMapTransition`）、`RentalItemSetCatalog.java`、`NativeLitoTownPortalCatalog.java`、`NativeLitoRentalHotelCatalog.java`、`NativeLitoRentalTownMapCatalog.java`、`ServerConfig.java`（`rental-source` profile、`lito.rental.town.map` 500／560）、`Protocol.java`（S_CB 職業位元組）。沒有複製任何資源檔。

## 已驗證

在 `LiTo381-Modern/` 執行（本機只編譯，不執行測試 EXE，依 2026-11-09 本機限制）：

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
ExitCode=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 06s
```

合併 `origin/main`（ce85b82，自動合併，`native_login.rs` 沒有衝突）後重跑：

```text
ExitCode=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 09s
```

我的變更檔沒有新增編譯警告。

## 推測

- 正式 rental-source 伺服器（`rental-source` profile）下，職業 0..4 的角色選擇範圍內的配裝，送出的 wire 和改動前相同；只有超出範圍時不再送出 C_CA。
- 正式端租裝村的所有 S_PORTAL 都屬於上面三種，所以既有的 map500 原圖入口、旅館 501／502／503 探針應不受影響。這是依伺服器原始碼推論，本張沒有用正式 JAR 重跑。
- 伺服器表中職業 5／6 沒有配裝；這兩個職業的角色現在在客戶端就看不到可選配裝。

## 待驗證／未完成

- **`cargo test` 結果交給 PR CI**：本機沒有執行測試。新增的 5 案（2 單元＋3 loopback）與修改過的 `rental_lifecycle_tests` 要看 CI 的 `test result:` 原文；CI 結果出來前不算通過。
- 預設不跑的正式 harness（`native_peer_rental_probe`、`native_map_portal`、hotel 相關探針）需要 E149 JAR 與本機 owned runner，交給 Codex／本機執行。
- map500 其餘 22 處原圖入口（24 處只驗過 1 組雙向）的實際往返、map560 入口、旅館入住／返回的 GUI 都還沒在實機驗證。
- 配裝名稱（平衡型／攻擊型等）應該從客戶端 `LTItemSet<class>-<selection>-c.tbl` 讀取；本張只顯示「配裝 n」，沒有查 pak，也**沒有**主張缺資源。
- 本張沒有讓客戶端在 C_CA 後補送原生的 C_CE（伺服器會重送 S_FB＋S_F3）；目前沿用 LOGIN 端的 S_FB，正式端已在 GAME bootstrap 重新開放直連房，所以不需要 C_CE。要不要對齊原生的 C_CE 序列，需要正式 capture 比對後再決定。
- 原生 GUI、多 DPI、實體輸入都沒有驗證。伺服器端 quitFight 的缺口仍列在 U-M3-1。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_rental_source.rs`（新）
- `LiTo381-Modern/src/modern_lito/native_rental_source_tests.rs`（新）
- `LiTo381-Modern/src/modern_lito/native_login.rs`、`native_rental_window.rs`、`native_peer_rental_probe.rs`、`native_rental_lifecycle_tests.rs`、`mod.rs`
- 本報告 `docs/reports/issue-191.md`；本機 check log 在 `scratch/i191/`（不追蹤）

## 給下一棒的提示

- 先看 PR CI 的 lib 測試：`native_login::rental_source_tests::*`、`native_rental_source::tests::*`、`native_login::rental_lifecycle_tests::*`。如果 loopback 案失敗，先看 `selected()` 的前置狀態（environment、readiness）是否和 `rental()` 一致。
- 本機執行者：用 E149 跑 `native_peer_rental_probe`（stage 0 現在驗本地拒絕）、`native_map_portal`、旅館探針，並在 trace 中核對 `rental_town_transition` 是 `GroundPortal`／`HotelEnter`／`HotelReturn`。
- TASKS.md 的 T055 狀態沒有改（兩張都完成後由審查者更新）。
