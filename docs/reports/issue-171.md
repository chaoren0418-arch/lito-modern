# Issue #171 — T023 第 3／4 張：G06.3d 旅館、傳送與跨端點服務

## 做了什麼

在 worktree `task/171`（基底 `9729613`）上改了下面這些。沒有套用隔離候選，沒有改 `docs/REPORT.md`，沒有改原專案目錄，也沒有改相鄰的 lito-launcher-server（只讀）。

### 1. 沒有 S_PORTAL 的解鎖（奪命之雷、失敗分支）

**依據（唯讀參考 lito-launcher-server 工作目錄，未 checkout／未寫檔）：**

- `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/Protocol.java`：`teleportSpellRequest` 的註解寫明原生客戶端送出技能 5 前會先鎖住動作／游標，所以伺服器每一個終止分支都要用 S_PARALYSE mode 7 解鎖；`ParalysisMode.TELEPORT_UNLOCK(0x07)`，`paralysis()` 寫出 `c(0x25) c(7)`，body 2 bytes，加上 2-byte 長度標頭剛好是報告說的「4-byte 解鎖」。
- `ClientSession.java` 的拒絕分支（例如 :32248–32258）先送 mode 7，**再**送 `S_MESSAGE_CODE 285`。
- `docs/maintenance/native-thunder-teleport-policy-progress.zh-TW.md:5`：奪命之雷政策開啟時，技能 5／131 照樣扣 MP 與 reuse，送出 Source 中文錯誤、傳送解鎖、MP，**不建立 portal**。
- `docs/maintenance/source-mass-teleport-exit.zh-TW.md:15`：技能 69 施法失敗時補送一次 4-byte 解鎖。
- `tools/classic_world_mass_teleport_selftest.py:58-68`：成功路徑的解鎖出現在 S_PORTAL→C 0x08→重建**之後**。

**客戶端修改（`native_login.rs`、`native_skills.rs`）：**

- 有等待中的技能 5（`teleport_spell`）或 131（`world_tree_request`）時，收到**剛好** `25 07`：
  - 兩個等待都清掉，不建 portal、不換地圖、不移動角色、不重設場景；
  - 計數 `teleport_unlocks_without_portal`；
  - 之後才到的 S_PORTAL 不再被當成這次請求的回覆（WORLD 階段會照原規則拒收）。
- 技能等待（`skill_pending`）如果還在，`observe_skill` 會回 `Feedback::TeleportReleased`（「傳送未執行，已解除鎖定」），不用再等 2 秒逾時。同一批封包裡如果有編碼拒絕（例如 276／285）或施法動作，以那個結果為準。
- 沒有傳送等待時，`25 07` 跟以前一樣只解除 mode 6 的移動限制，不影響其他等待中的動作。畸形的 `25 07 00`／`25` 會失敗，而且不會解除等待。
- 成功路徑不受影響：S_PORTAL 到的時候 `teleport_spell` 已經被取走，重建後才到的解鎖不會觸發任何動作。

技能 69（集體傳送術）客戶端**還不能施放**（`native_skills::request` 沒有接 69），所以 69 的解鎖在客戶端看到的就是同一個 `25 07`。這裡不新增 69 的施放入口。

### 2. 權威座標與 showId

- 新增測試確認：技能 5 的 S_PORTAL 只檢查地圖要跟請求一致；座標用伺服器給的（隨機傳送送 0,0、書籤送書籤座標，兩種都以 S_PORTAL 座標為準）；地圖不同或沒學技能就拒收。
- World 收到 S_PORTAL 後，pending 目的地是伺服器的格子；在伺服器重建自己的角色前，輸入一律關閉（`ready()==false`）。重建後的位置必須等於 S_PORTAL 的格子（既有 `finish_portal_if_ready`）。
- **showId**：`combat-integration-lifecycle.zh-TW.md:86` 的 showId 是伺服器內部的副本 ID。客戶端 wire（S_PORTAL `5a h h h`、S_PUT）都沒有這個欄位，客戶端無法觀察，也不能自己推測；只記錄，不做客戶端處理。

### 3. TeleportMe 原圖示排版（HISTORY L1150）

原頁（LTtext 343 `teleportme-c.html`，323 bytes，fixture `tests/fixtures/343-teleportme-c.html`）的寫法是 `<a action="LT_TELEPORT_B">…</a>   <img src=#7311><br>`：圖示在選項文字**後面**、同一行。之前的做法是只用 command 對應圖示，然後把圖示畫在文字**左邊**。

- `native_npc_document.rs`：`Document` 新增 `trailing_images: Vec<(action index, image id)>`。規則：`<img>` 要緊接在一個已經結束的 `<a>` 後面，中間只有空白、在同一行，才算是這個選項的圖示。遇到換行、文字、其他標籤就斷開；每個選項只取一張圖。其他圖片還是留在 `images`，但沒有排版位置。
- `native_npc_images.rs`：`Bank::trailing_glyph` 用文件的排版位置找圖，而且 command 與圖示 ID 必須一致（B→7311、C→7312）。新增純函式 `trailing_row`：文字放左邊，接著一個 HTML 合併後的空白，再放圖示並垂直置中；文字太長會被截掉，讓圖示留在列內；列太窄就只畫文字。
- `native_rental_window.rs`：owner-draw 列改用上面的排版。字寬用 `GetTextExtentPoint32W` 量，文字太長加 `DT_END_ELLIPSIS`。

### 4. 同端點以外的 Presentation／音效／其他服務

`docs/G01-G12.md` G06.3d 的 state 是 `…same_endpoint_Presentation_audio_other_services_GUI_pending`。剩下的部分是 GUI 實機呈現、音效實際播放和旅館自然到期／手動退房（`current_hotel_*_verified=false`），這些都要開客戶端或連線伺服器才能驗，本機規定不能做。這張沒有改這部分的程式，列在「待驗證」。

## 怎麼驗證

**本機規定（2026-11-09）不得執行編譯出來的程式，所以本機只做編譯與靜態檢查，`cargo test` 交給 PR 的 GitHub Actions CI。**

工作目錄 `LiTo381-Modern/`：

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
# 結束碼 0；Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 20s
```

- 警告共 390 條，都在既有檔案；本張改動的檔案（native_login、native_skills、native_npc_document、native_npc_images、native_rental_window、新的 native_teleport_unlock_tests）沒有新警告。日誌在 `scratch/check171.log`（不入 git）。

新增的單元測試（`cargo test --lib` 會跑，CI 執行）：

- `modern_lito::native_login::teleport_unlock_tests`（7 案）：
  `skill5_unlock_without_portal_releases_the_wait_and_builds_no_portal`、`world_tree_unlock_without_portal_releases_the_request`、`coded_rejection_after_unlock_in_the_same_batch_keeps_its_reason`、`unsolicited_unlock_and_server_lock_pair_do_not_release_unrelated_work`、`malformed_unlock_fails_without_releasing_the_request`、`skill5_admits_server_coordinates_not_the_requested_ones`、`accepted_portal_destination_is_the_server_cell_and_rebuild_must_match_it`
- `modern_lito::native_npc_document::layout_tests`（2 案）：原頁兩張圖分別接在 B／C 後面；圖在前、中間有換行、中間有文字、同一選項第二張圖，這些都不算排版位置；圖放在 `<a>` 裡面仍然拒收。
- `modern_lito::native_npc_images::layout_tests`（3 案）：`trailing_row` 的位置、長文字截斷、列太窄。
- integration `tests/native_npc_document.rs` 另加 `trailing_images==[(0,7311),(1,7312)]` 斷言（這個檔案會讀 `D:\lineage開發\0-客戶端`，CI 不跑）。

## 已驗證

- 上面的 `cargo check --all-targets` 結束碼 0。
- CI 結果：見下方「CI」段（push 後補上）。

## 推測

- 原生 HTML 呈現會不會把 `</a>` 和 `<img>` 中間的三個空白合併成一個，沒有從 PE 逆向確認；這裡照一般 HTML 規則當成一個空白。
- 奪命之雷的「Source 中文錯誤」如果是沒有編碼的 S_MESSAGE 文字，客戶端用解鎖結束等待；如果是有編碼的訊息，就照編碼顯示拒絕原因。兩種情況都有測試，但正式伺服器實際送哪一種沒有實際抓封包確認。

## 待驗證／未驗證什麼

- `cargo test` 本機沒跑（規定）；以 CI 為準。
- TeleportMe 新排版沒有實際開視窗看畫面、沒有截圖、沒有跟原客戶端畫面比對（要等 GUI 驗收／Codex 不連線截圖）。
- 奪命之雷／技能 69 失敗的解鎖沒有接真的伺服器跑（只有離線重播）；技能 69 客戶端施放入口沒做。
- 旅館自然到期、手動退房、同端點以外的 Presentation 與音效實際播放、完整 GUI：都沒做，維持 G06.3d `complete=false`。
- 沒有連正式服務或 DB，沒有啟停遊戲程序或 GUI。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_login.rs`、`native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_npc_document.rs`、`native_npc_images.rs`、`native_rental_window.rs`
- 新增 `LiTo381-Modern/src/modern_lito/native_teleport_unlock_tests.rs`
- `LiTo381-Modern/tests/native_npc_document.rs`
- 報告：`docs/reports/issue-171.md`

## 給下一棒的提示

1. Claude 審查重點：
   - `25 07` 只在有 5／131 傳送等待時才結束等待；`teleport_released` 只把 Pending／NoResponse 改成 TeleportReleased，不會蓋掉編碼拒絕。
   - `trailing_images` 的斷開規則：遇到任何標籤或非空白文字就斷開。
2. 本機測試執行者（Codex）：`cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib -- teleport_unlock_tests layout_tests --test-threads=1`；不連線截圖時，打開 TeleportMe 對話確認圖示在文字右邊（7311 接祝福之地、7312 接勇氣之地）。
3. 第 4／4 張（物品接續拒絕與期限、T 鍵入口、cancel-late、畸形與跨世代 wire 回覆）這張沒有動。要做技能 69 施放的話，可以沿用 `native_teleport_spell::Choice`，request 形狀是 ⑥ `14 08 04 h(map) h(x) h(y)`（PROTOCOL-CONTRACT §6.3 第 385 行）。
