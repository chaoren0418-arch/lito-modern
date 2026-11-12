# Issue #326 — T032：`docs/ARCHITECTURE.zh-TW.md` 重寫為現況版

執行者：claude-local。分支 `task/326`。只改兩個檔：`docs/ARCHITECTURE.zh-TW.md`（全文重寫）、本報告。

## 做了什麼
- 把 `docs/ARCHITECTURE.zh-TW.md` 從 387 行的逐輪流水帳（2026-10-12～10-25 的每輪結果）改寫成現況版（非空行 172 行、12 節）：一張圖、目標與邊界、repo 佈局、建置與 `main.rs` 路由、原始碼兩個世代、原生路由六層（線路／連線／控制／世界模型／呈現／資源，加音效與視窗）、貫穿各層的不變條件、歷史 profile 路由、測試分層、外部相依、常見修改對照、已知結構性問題。
- 進度、每輪結果、技能狀態不再寫在這份文件，改成檔頭的索引表指到 `PLAN.md`／`TASKS.md`／`docs/REPORT.md`／`docs/SKILLS-STATUS.md`／`docs/HISTORY.md`。
- 舊版內容沒有搬到別處：依 CLAUDE.md 第 3 條「歷史靠 git」，新檔頭註明舊版在 commit `92bbda0`。

查過 `docs/REUSE-MAP.md`：本張是文件任務，沒有引用 tw-bin-engine 或 lito-launcher-server 的原始碼；§10 的外部相依敘述取自 REUSE-MAP §0 與 `PLAN.md`。

## 怎麼驗證
全部是靜態閱讀與字串計數，依據 `main` @ `64a3796`，沒有執行任何編譯出來的程式。

### 已驗證（直接從檔案讀到）
- `Cargo.toml`：lib＋兩個 bin、feature `teleport-wire-test`、相依清單、release profile。
- `src/main.rs` 全文：`Route` 列舉與 `route()` 的每個參數、Debug 限定路由、`--version` 輸出欄位。
- `src/lib.rs`、`modern_lito/mod.rs` 全文：模組清單、`#[allow(dead_code)]` 的實驗模組註記。
- 檔案與行數：`git ls-files` 計數（`LiTo381-Modern` 2158、`src` 851 檔約 35.4 萬行、`modern_lito` 847 檔；`native_*` 365 檔約 7.5 萬行，其中 `_tests` 109、`_study` 60、`_probe` 20；`tests/*.rs` 138 檔）。
- `rustls` 只被 `transport.rs`、`profile_probe.rs` 使用（grep）。
- 執行緒：`lito-native-network`（`native_connection.rs:538`）、`native-world-load`（`native_world_scene.rs:301`）；UI 計時器 `SetTimer(hwnd, 1, 20)`（`native_login_window.rs:2770`）；`StretchDIBits`、waveOut 的使用位置（grep）。
- `LoginPhase` 48 個狀態、`LoginController` 與 `Snapshot` 的欄位、`native_connection` 的 `Event`／`Phase`／`Limits` 預設值、epoch 在連線時 `checked_add(1)`（`native_connection.rs:528-532`）。
- `native_world_view::Presentation` 的 `Key { connection, generation, map }` 柵欄；`native_world_scene` 的 `Job`／`Model`／`Frame`／`Completion`。
- 各 `native_*` 模組的檔頭 `//!` 說明（wire、version、opcodes、integrity、clock、audio_delivery、terrain_cache、resources、skills、world、world_animation）。
- `windows/app.manifest` 是 `dpiAware=true`；原生視窗檔沒有 `Imm*`／`WM_IME` 處理（grep 無結果）。
- 共用層被原生路由引用的檔數（`compositor` 23、`world_map` 24、`sprite` 11、`sprite_actions` 10、`tower_assets` 10…）。
- 測試的靜態計數：`src` 內 `#[test]` 2875、`#[ignore` 583；`tests/*.rs` 內 `#[test]` 700。
- `docs/REPORT.md` 索引：M2 的 57 張中有 52 張已列為合併；沒出現的是 T022 批 7–10 與 T027 批 6（這項只寫在本報告，沒有寫進架構文件）。

### 推測（文件內已標「推測」）
- `維護工具/`、`LiTo381-Modern/tools/` 裡「現行還在用」的腳本清單，是依 `TASKS.md`／`docs/REPORT.md` 的引用整理的，沒有逐檔確認其餘腳本沒人呼叫。
- `native_offline_*`、`native_multi*`、`native_mode_menu*`、`native_city_*` 等實驗視窗不在預設原生路由上：依 `main.rs` 沒有路由指向它們判斷，沒有逐一追呼叫鏈。
- 歷史 profile 世代的 lib 測試佔多數：依檔案數與行數比例推得。
- §6.4 的模組分組是依檔名與檔頭註解歸類，沒有逐檔讀完；個別模組可能跨主題。

### 待驗證（需要執行或實機）
- 當版 CI 的測試通過數：本次沒有重跑，文件只引用 `docs/TESTING.md` 的 2026-11-04 基線。本 PR 的 CI 結果請看 PR。
- IME 組字／取消／焦點轉移、多螢幕 DPI、UI 外觀與原版的差距：歸 T034／T030。
- 文件 §1 的資料流圖是從結構體與執行緒名稱整理的；實際每個 tick 的呼叫順序沒有用除錯器或 log 對過。

## 未驗證／沒做的
- 沒有逐一讀完 365 個 `native_*` 檔；`native_login.rs`、`native_login_window.rs` 只讀了結構定義與檔頭。
- 沒有改 `LiTo381-Modern/README.md`（內容已過時，架構文件 §12 第 6 點有註明）、`CLAUDE.md`、`PLAN.md`、`TASKS.md`、`docs/REPORT.md`。
- 沒有執行 `cargo` 或任何編譯產物；本張只動 Markdown。

## 產出路徑
- `docs/ARCHITECTURE.zh-TW.md`
- `docs/reports/issue-326.md`

## 給下一棒的提示
- **審查者**：重點看 §5（兩個世代的劃分）與 §7（不變條件）是否符合你對程式的理解；這兩節是歸納，不是從單一檔案抄來的。
- **T037（G12 稽核與冗餘整理）**：架構文件 §3、§12 第 5 點列了 `維護工具/`（2387 檔）與 `tools/`（903 檔）的現況，可以當整理的起點；「現行還在用」的腳本清單要先實際確認。
- `LiTo381-Modern/README.md` 仍寫「預設啟動離線共享資源場景」「`native_*` 是歷史候選」，與 `main.rs` 現況不符（預設是原生登入路由）。建議另開一張小任務改成指向架構文件，或併入 T037。
- CLAUDE.md「目前狀態」仍寫「開發停在 #97」「475 個測試案全未跑」，與 `docs/REPORT.md` 的 M2 進度不一致；更新它屬於主控對話的範圍。
- 之後新增子系統時，請更新架構文件 §6 的對應表與 §11，不要在檔尾追加逐輪紀錄；逐輪結果寫 `docs/reports/`。
