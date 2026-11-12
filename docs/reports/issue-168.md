# Issue #168：T029-2 非技能戰鬥基礎——操作

本報告只涵蓋 T029 第 2 張（操作）；不把 T029 整項標成完成。遠程（第 1 張）不在本張範圍。

## 做了什麼

| 項目 | 內容 | 主要檔案 |
|---|---|---|
| 空揮封包 | `native_combat::empty_request(x, y)` 產生 `17 00 00 00 00 <x> <y>`；`native_combat_ops::empty_melee_request` 取面向前一格當座標。測試用紀錄封包 `17 00 00 00 00 DE 7F 1D 80` 逐位元組核對。控制器新增 `LoginController::empty_attack_with_input`；GUI 在**沒有選取目標**時按 Space 送空揮（原本只顯示「請先點選有效目標」）。 | `native_combat.rs`、`native_combat_ops.rs`、`native_login.rs`、`native_login_window.rs` |
| 空揮拒絕條件 | 依正式 `EmptyMelee.handle` 的順序做本地 preflight：戰鬥已結束、觀戰、未就緒、死亡（HP 0 或 dead）、隱身、麻痺（`movement_blocks`）、弓／投擲 stance20／62、座標不合法（x 或 y 為 0、等於自己位置、超出 `isInNativeScreen`）、超重（S_STATUS／能力值的 weight240 ≥ 197）。被拒時不送封包，`attack_feedback = Refused(原因)` 並顯示中文原因。 | `native_combat_ops.rs` |
| 目標 0 回覆 | 正式伺服器空揮時會廣播（含自己）`23 01 <attacker> 00000000 00 <heading> 00000000`。原本 `decode_attack` 拒絕近身目標 0，等於自己的空揮回覆會被當成格式錯誤；現在接受「目標 0 且傷害 0」，目標 0 但傷害非 0 仍拒絕。 | `native_combat.rs` |
| 延遲回覆與重試交錯 | `World` 新增有界 `own_attack_log`（最近 16 筆：回覆序號、回覆中的目標）。`Pending::observe` 只在「送出後」且「目標相同」（空揮為 0）的回覆才回 Acknowledged；指向其他目標的延遲回覆不會確認重試。`native_combat::Ledger` 計算這種延遲回覆數，放在 `Snapshot.attack_replies`。 | `native_world.rs`、`native_combat.rs`、`native_login.rs` |
| 拒絕提示 | `Feedback` 新增 `Rejected(code)`（伺服器 S_MESSAGE_CODE 110 = 負重過重，正式 `ClientSession.attack` 與 `EmptyMelee` 都只送 110 不送 S23）與 `Refused(原因)`（本地拒絕）。其他訊息碼不結束等待。 | `native_combat.rs` |
| 攻擊專用速度 | `attack_interval_ms(base, speed, brave)` 依正式 `EmptyMelee.interval`（引用 ActionIntervalCalculator 的 ATTACK 倍率）：速度1 ×0.755、速度2 ×2、brave6 ×0.775、brave1／5 ×0.755、brave3 ×0.855 再 ×0.9；brave4 只影響走路、不加速攻擊。另有 `Pacer` 供空揮節奏 gate。 | `native_combat_ops.rs` |
| 自動追擊 | `HeldAttack::press_chase`：目標在其他條件都通過、只是距離不夠時保留攻擊意圖，`chase_heading` 用貪婪單步（直行方向，再試左右相鄰方向，必須嚴格縮短 Chebyshev 距離）提出方向，經既有 `move_presented`（地形、門／守護塔、動畫抵達）送出；最多 64 步，進入範圍後同一按壓改為一般攻擊。GUI：Space 對超出範圍的目標、或再點一次已選取的目標，都會啟動追擊。不做尋路。 | `native_combat.rs`、`native_world.rs`（`attack_reach`／`attack_out_of_reach`）、`native_login_window.rs` |
| 目標圈 | `target_ring` 依本地條件分四色（可攻擊／超出範圍／已死亡／不可攻擊）；`Frame::target_ring` 只在畫面與目前世界一致時給出位置；`paint_world` 在 StretchDIBits 後以 GDI 畫橢圓（48×24 菱形外接、與畫面相同的 letterbox 換算）。配色是本客戶端的選擇，不是還原原版。 | `native_combat_ops.rs`、`native_world_scene.rs`、`native_login_window.rs` |
| alpha 命中 | 查證既有 renderer：點擊選取用逐像素 owner 平面，`blit_prepared_world_fragment` 只有 alpha 255 的像素寫 owner，透明與半透明像素不算命中——也就是已是 alpha 命中，`NATIVE-COMBAT` 文件寫的「尚未作 alpha 像素命中」已過時。本張補單元測試鎖住這個語意並更新文件，沒有改 renderer。 | `compositor.rs`（只讀）、測試 |

文件更新：`docs/NATIVE-COMBAT.zh-TW.md`（原生操作兩段）、`docs/NATIVE-OUTCOMES.zh-TW.md`（延遲回覆一句）、`docs/NATIVE-NAVIGATION.zh-TW.md`（攻擊專用速度一句）。

### 參考來源（唯讀，lito-launcher-server，本機工作樹，未修改）

- `lito伺服器/LiTo381-Server/docs/maintenance/empty-melee-0.6.49.json` L34–42（空揮封包、拒絕條件）
- `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/EmptyMelee.java`（拒絕順序、回覆格式、`interval`）
- 同目錄 `ClientSession.java` 約 L74755–74780（一般攻擊：麻痺、超重 110）、`BattleSpellCatalog.java` L3401（`isInNativeScreen`）、`BattleInventoryWeight.java` L6（`BUFFED_REGEN_LIMIT = 197`）、`BattleAnimationCatalog.java` L174／L192（stance 20／62）

## 怎麼驗證

依本機規定（2026-11-09），本機只編譯、不執行任何編譯產物（含 `cargo test` 產生的測試執行檔）；測試交給 GitHub Actions CI。

### 已驗證

- 已和 `origin/main`（`edc5e94`）合併；衝突在 `native_login.rs`（main 把技能觀察抽成 `observe_skill`，保留 main 版本再加一行 `attack_replies.observe`）與 `native_login_window.rs`（main 把移動改成 `continue_movement`，保留 main 版本，追擊區塊放在它前面）。
- 合併後本機 `cargo check`：

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 44s
exit=0
```

  這是編譯檢查，沒有 `test result:`；有專案既有的警告，不宣稱零警告。
- `git -c core.whitespace=cr-at-eol diff --check` 通過；`native_login_window.rs` 原本換行混用，未改動的行保留原換行。
- 原始碼審閱（不是執行結果）：點擊選取用的 owner 平面只寫 alpha 255 像素。

### 新增測試（`src/modern_lito/native_combat_ops_tests.rs`，CI `cargo test --lib` 會跑）

16 個：紀錄封包位元組、八方向空揮座標、每個拒絕條件（含 weight240 196／197 邊界、stance 20／62、x=0）、`isInNativeScreen` 鏡像（距離 18／19／20、邊緣規則）、攻擊間隔倍率、Pacer、追擊單步（直行、繞相鄰、全擋、遠程射程 19）、目標 0 回覆解碼與確認、他目標延遲回覆不確認重試、同目標重試、110 拒絕與其他訊息碼、追擊按壓存活與轉攻擊、目標圈四色、目標圈座標與畫面外、alpha 命中 owner 平面。

另外修改既有測試：`tests/native_combat.rs` 的回覆模擬補上 `own_attack_log`（新規則下只加計數不算確認）；`native_resurrection_tests.rs` 的 `Pending` 補 `messages` 欄位。

CI 結果：見 PR 的 `build` 工作（`cargo check --locked --all-targets` 與 `cargo test --locked --lib --no-fail-fast`）；本報告送出時尚未取得，完成後由 PR 留言補上 `test result:` 原文。整合測試 `tests/native_combat.rs` 不在 CI 範圍（部分案例要讀本機客戶端資源），本輪**未執行**。

## 推測

- 原生客戶端空揮座標取「面向前一格」：紀錄封包只有一筆，無法從中反推方向；正式伺服器只要求非 0、不等於自己座標且在畫面內，所以任何面向前一格都會被接受。原生是否改用滑鼠位置等其他座標，未以反組譯核對。
- 本地超重判斷使用最後收到的 weight240；若伺服器端負重已變而客戶端尚未收到更新，本地判斷可能比伺服器早或晚一拍，最終仍以伺服器（110）為準。
- 沒有請求序號，同一目標的逾時請求與重試無法區分：第一筆同目標回覆會確認最新的請求。這是協定限制，不是已解決。

## 待驗證／未驗證什麼

- **CI 的 `cargo test --lib` 結果**（新 16 個與既有 lib 測試）。
- 整合測試 `tests/native_combat.rs` 等未執行。
- 實機（交給 Codex／使用者）：Space 空揮動作與伺服器回覆、各拒絕提示文字、追擊行走（含門、牆、目標移動）、目標圈位置／顏色／DPI、再點目標啟動追擊的手感、110 提示。本輪不啟動遊戲、不連正式服務或 DB。
- 攻擊專用間隔目前**沒有在 GUI 啟用**：GUI 尚未取得自己攻擊動作的時長，呼叫 `empty_attack_with_input(heading, None, …)`，所以太快的空揮由伺服器靜默丟棄，客戶端兩秒後顯示「未收到攻擊回覆」。攻擊動畫播放速度也沒有改成攻擊專用倍率（缺反組譯證據）。
- 目標圈畫在角色的權威格子，不跟隨移動中的插值位置；畫面不會因為只換選取目標而主動重畫（等下一次畫面更新）。
- 追擊沒有尋路，遇到需要繞路的障礙會停止並清除意圖。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_combat_ops.rs`（新增）
- `LiTo381-Modern/src/modern_lito/native_combat_ops_tests.rs`（新增）
- `LiTo381-Modern/src/modern_lito/native_combat.rs`、`native_world.rs`、`native_login.rs`、`native_login_window.rs`、`native_world_scene.rs`、`mod.rs`
- `LiTo381-Modern/src/modern_lito/native_resurrection_tests.rs`、`LiTo381-Modern/tests/native_combat.rs`
- `docs/NATIVE-COMBAT.zh-TW.md`、`docs/NATIVE-OUTCOMES.zh-TW.md`、`docs/NATIVE-NAVIGATION.zh-TW.md`
- 本報告 `docs/reports/issue-168.md`

## 給下一棒的提示

1. 先看 PR 的 CI；若 `native_combat_ops_tests` 有失敗，多半是 fixture（`tests/fixtures/native-world-current.json`）的角色位置／狀態假設，調整測試裡的座標即可，不要放寬拒絕條件。
2. 要啟用空揮節奏：從畫面模型取出自己 `attackForWalk(walk_action)` 動作的總時長（毫秒），傳給 `empty_attack_with_input(heading, Some(attack_interval_ms(base, speed, brave)), input)`。
3. 實機核對清單（交 Codex）：(a) 無目標 Space → 伺服器回目標 0 S23、狀態「攻擊已回覆」；(b) 觀戰／死亡／隱身／麻痺／持弓／超重時各自的提示且 TX 不增加；(c) 目標在 5 格外按 Space → 逐步靠近後攻擊；(d) 目標圈顏色隨距離變化；(e) 超重攻擊 → 「負重過重，無法攻擊」。
4. 遠程（T029-1）若要讓弓／投擲也能追擊，`attack_reach` 已用 19 格當遠程射程，`chase_step` 吃 `reach` 參數即可共用。
