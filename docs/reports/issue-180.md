# Issue #180 — T045 第 2 張：擂台商店、地圖道具、三重矢扣箭、12／107 武器目標

分支：`task/180`（worktree `scratch/wt/task-180`）。runner：claude-local，第 1 次嘗試。
只改本張範圍；未改 `docs/REPORT.md`；未啟動遊戲、伺服器或任何編譯出的程式；未連正式服務或 DB；未寫原始專案目錄與客戶端資源。

## 結論

本張四項的**協定與客戶端邏輯**已實作並加上單元測試：擂台點數商店（C_DA／C_DE／S_DF／S_E0／C_DB）、S_SHOW_MAP 地圖道具 18 筆、三重矢扣箭核對、12／107 固定以裝備中的武器為目標。地面物品與陷阱標「伺服器待定」。
**尚未完成**：商店 UI 與地圖頁繪製沒有做；技能 132 的送包屬 T022 批 3，本張只提供扣箭核對；新單元測試依規定沒有在本機執行，等 CI 結果。

## 做了什麼

| 項目 | 實作 | 檔案 |
|---|---|---|
| 擂台商店 16 列 | 解析 `S_DF h(n)×[d(catalog) h(icon) d(價) s(名) c(len) raw]`、`S_E0 h(n)×[d(背包物件) d(價)]`；`C_DA`／`C_DE` 只送 opcode；`C_DB c(op) h(n)×[d(id) d(數量)]`（一般形式，不帶原生 8 bytes 尾段）。Session 綁定連線、角色、phase、channel、room、背包 owner，清單 60 秒到期（同正式 `PendingTournamentShop`），一次只開一張單，8 秒沒清單就逾時；賣單要求物件仍在背包、數量不超過持有量；買單拒絕總價超過 u32。點數與背包不預扣。附正式 16 列對照表 `E149_OFFERS`（買價＝賣價），只供對照，不覆寫伺服器價格 | `src/modern_lito/native_tournament_shop.rs`（新） |
| 控制器接線 | `Snapshot.tournament_shop`、`tournament_scope()`（只在 `LobbyReady`／`RoomReady`）、`open_tournament_shop(sell)`、`tournament_order(revision,index,qty)`；`accept_game_received` 收 S_DF／S_E0，格式錯誤是協定錯誤，格式正確但沒有對應開單只計數 `unsolicited`；`poll` 依範圍逾時／清除 | `src/modern_lito/native_login.rs` |
| S_SHOW_MAP 18 筆 | 解析 `6E d(objid) c(page 1..18)`（長度必須 6）；依原版 `sub_4AFFB0`→`447B70` 只確認物件在背包，不在就忽略並計數；頁碼照伺服器，不和物品比對；名稱以正式 nameId 開頭時附 catalog 對照。物品移除、換角色或連線時關閉。附正式 18 筆 `E149_ITEMS`（40373 是第 16 頁） | `src/modern_lito/native_map_item.rs`（新）、`native_login.rs`（`Snapshot.map_view`、`close_map_view`） |
| 三重矢扣箭 | 箭＝useType 254（0xFE）。`Ledger::capture` 背包沒有箭時拒絕；`observe` 只看伺服器 S1A／S17 計算消耗、移除、增加；`matches_cast`：成功施法扣 min(3, 原有箭數)，被拒絕時 0；之後的一般攻擊另算 1 支 | `src/modern_lito/native_triple_arrow.rs`（新） |
| 12／107 依已裝備武器 | `weapon_bound(12|107)`、`equipped_weapon()`：背包名稱帶 ` ($9)` 的唯一一件（0 件或多件都拒絕）；`ItemTarget::equipped_weapon()` 沿用既有快照守衛。視窗施放、可施放判斷與提示文字改用裝備中的武器，不再要求開背包選物品；技能清單標籤改為「對裝備中的武器」 | `native_skills.rs`、`native_login_window.rs`、`native_skill_selection.rs` |
| 地面物品與陷阱 | 不實作，`docs/SKILLS-STATUS.md` 新增一列標「伺服器待定」 | `docs/SKILLS-STATUS.md` |
| 狀態表 | 更新擂台商店、彈藥、物品目標技能三列，新增地圖道具、地面物品與陷阱兩列 | `docs/SKILLS-STATUS.md` |

`item_request` 本身沒有加「必須是裝備中武器」的硬性檢查：既有整合測試（`native_item_actions`、`native_skills_101_109`、`native_t027_batch1`）用一般物品測 12／107 的封包形狀，而本機不能執行測試確認修改後結果。目標選擇放在 `ItemTarget::equipped_weapon`，視窗只走這條路徑。

## 依據（唯讀靜態證據）

- 正式 JAR：原始專案 `LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar`，本次實算 SHA256 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。只用 `javap -p -c -constants`（另以 `javap -v` 讀字串串接常數）反組譯，輸出在本 worktree `scratch/issue180/E149-*.txt`；沒有執行 JAR 或自寫 Java 程式。
  - `Protocol`：`C_LITO_BUY_LIST=218`、`C_LITO_BUY_SELL=219`、`C_LITO_SELL_LIST=222`、`S_SHOW_MAP=110`、`S_LITO_POINT=219`、`S_LITO_BUY_LIST=223`、`S_LITO_SELL_LIST=224`；`showMap` 頁碼 1..18。
  - `ClientSession.sendBuyList`／`addShopItem`／`sendSellList`／`openTournamentShop`（60 秒）／`tournamentShopPhaseAllowed`（GAME、已選角、LOBBY／CHANNEL／ROOM）／`validTournamentShopConfirmation`（phase、channel、room、期限）。
  - `TournamentShopCatalog.<clinit>`：16 列 `offer(id, price)`，買價＝賣價：40010/40、40011/200、40013/200、40007/5000、40744/20、40068/1000、40017/70、40012/600、40318/900、40746/30、40014/800、40016/600、40088/1300、40317/150、40015/700、40319/700。和 `data-sources-0.6.60.json` L402–439 的 16 列（seed 與 live SHA 相同）一致的只有列數；價格欄只以 JAR 為準。
  - `NativeMapItemCatalog.<clinit>`：18 筆 `(catalog, page, icon, groundGfx, nameId)`；`ClientSession` 的 `Effect.MAP` 分支在 BATTLE、非觀戰、HP>0 才 `requirePage`＋`showMap`。
  - `useTripleArrowSpell`：要求弓、目標、MP（不足回 278），以 `hitCount` 次 `nativeTripleArrowAttempt`＝`CombatTripleArrow.enter`＋一般 `attack()`。
  - `nativeMaterialItemWeaponEnchantBody`：讀請求物件 `definition.weapon()` → `CombatWeaponBuffs.equippedWeapon`，buff 掛在 `inventory.equippedWeaponItem()`。
  - `RuntimeInventory$Item.nativeViewName`：裝備中武器加 ` ($9)`、照明 ` ($10)`、防具 ` ($117)`。
  - `InventoryCatalog.<clinit>`：40743／40744／40746 的 useType 為 254。
- 原版客戶端：`lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`，複製到 `scratch/issue180/pe.bin` 後實算 SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`。以 pefile＋capstone（`scratch/issue180/dis.py`，只讀 bytes，未執行 PE）反組譯 `4AFFB0`、`447B70`、`4EBEB0`、`4EBBD0`。（`lito脫殼資源/embedded/` 下同名檔只有 1 byte 且無法讀取，未使用。）
- 伺服器文件：`native-triple-arrow-policy-progress.zh-TW.md:7`（成功施法扣 3 支，一般攻擊再扣 1 支）、`native-item-weapon-material-progress.zh-TW.md:5`（12／107 依已裝備武器）。

## 怎麼驗證

共同參數：`--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`（`CARGO_HOME=D:\dev-cache\cargo`、`RUSTUP_HOME=D:\dev-cache\rustup`）。

| 指令 | 結束碼 | 備註 |
|---|---:|---|
| `cargo check <共同參數> --all-targets` | 0 | 全文 `scratch/issue180/check.log`；本張檔案沒有新增 warning |
| `cargo test <共同參數> --lib --no-run` | 0 | 只編譯，產生 `lito_modern-64170f6ff127c89f.exe`；**沒有執行** |

依使用者 2026-11-09 規定，本機不執行編譯出的測試程式，所以**沒有 `test result:` 原文**。新測試都放在 `src/` 的 `#[cfg(test)]`，會被 GitHub Actions 的 `cargo test --locked --lib` 執行：

- `native_tournament_shop::tests`：6 案（16 列對照表、清單解析與所有截斷、C_DB 版面、開單範圍／種類／60 秒、賣單持有量、開單逾時）。
- `native_map_item::tests`：3 案（18 筆涵蓋 1..18 頁、`dc` 解析與錯誤頁碼／長度、背包成員判斷與關閉）。
- `native_triple_arrow::tests`：3 案（無箭拒絕、3＋1 支只依回包、移除與增加）。
- `native_skills::tests::weapon_bound_skills_resolve_only_the_single_equipped_weapon`：1 案。

## 已驗證／推測／待驗證

**已驗證**
- 上表兩個 cargo 指令的結束碼。
- 上列協定格式、16 列價格、18 筆地圖、扣箭次數來源、12／107 目標與 ` ($9)` 標記：皆有 E149 JAR 或原版 PE 的靜態位置，見「依據」。

**推測**
- 客戶端 `LobbyReady`／`RoomReady` 對應伺服器 CHANNEL／ROOM；伺服器的 LOBBY phase（選角後、進頻道前）對應哪個客戶端 phase 沒有核對，目前不開放。
- 不帶原生 8 bytes 尾段的 C_DB 會被接受：依 `Protocol.shopTransaction` 的長度判斷（`len&7==4`），未實送。
- 三重矢在箭少於 3 支時扣 min(3, 原有箭數)：依每次 `attack()` 扣 1 支推得，未看 `attack()` 沒有箭時的分支。
- 12／107 在沒有裝備武器時客戶端不送：伺服器會受理並回錯誤（伺服器文件），原版客戶端行為未核對。

**待驗證**
1. 13 個新單元測試的 CI 結果（`cargo test --lib`）。
2. 商店 UI：控制器已有 `open_tournament_shop`／`tournament_order`，視窗尚未加按鈕與清單（T034 或後續 UI 張）。
3. 地圖頁繪製：原版地圖面板 `[0x75AC68]`（`+0xC` 存 `page-1`，`4EBBD0` 經 vtable 開啟）用哪個資源畫各頁尚未定位；本張**沒有宣稱缺資源**，也還沒用 PakViewer 查 pak。
4. 技能 132 的 C_USE_SPELL（`d(target) h(x) h(y)`）與 `native_triple_arrow::Ledger` 接線，屬 T022 批 3。
5. 實機：買／賣成功與伺服器拒絕訊息（點數不足、背包滿、負重）、地圖道具使用、12／107 對裝備武器施放、三重矢實際扣箭（交 Codex）。
6. 地面物品與陷阱：伺服器待定。

## 產出路徑

- 新增：`LiTo381-Modern/src/modern_lito/native_tournament_shop.rs`、`native_map_item.rs`、`native_triple_arrow.rs`
- 修改：`LiTo381-Modern/src/modern_lito/mod.rs`、`native_login.rs`、`native_login_window.rs`、`native_skills.rs`、`native_skill_selection.rs`、`docs/SKILLS-STATUS.md`
- 本報告：`docs/reports/issue-180.md`
- 本機未追蹤證據：`scratch/issue180/`（E149 反組譯、`dis.py`、`check.log`、`test-norun.log`）

## 給下一棒的提示

- 商店 UI 直接用 `LoginController::open_tournament_shop(sell)`、`tournament_order(state.tournament_shop.revision, 列索引, 數量)`；送出後看 `state.shop_points`（S_DB）與 `world.last_message`／系統訊息（S_0F type 9）顯示結果，不要自己扣點數或加物品。
- 地圖頁繪製：先靜態追 `[0x75AC68]` 物件的 vtable（`4EBBD0` 呼叫 `[eax+0x24]`）找出圖片資源，再用 PakViewer 查所有 pak 後才可以判定有沒有資源；頁碼索引是 `page-1`。
- T022 批 3 接 132 時：送包前 `native_triple_arrow::Ledger::capture(&state.inventory)`，回覆後 `observe` 顯示扣箭；不要預扣。
- 12／107 目標只走 `ItemTarget::equipped_weapon`；不要再加第二套目標選擇。
- 地面物品與陷阱等伺服器 Claude 補契約後再排 code 任務。
