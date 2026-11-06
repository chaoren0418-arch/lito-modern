# Issue #181 — T046 第 1／3 張：原生快捷列、技能書與施法中斷

日期：2026-11-06。分支：`task/181`。起點：`59f0c2bcd7dc4435ba43186f7767130ab7869af1`。

**結論：原生 UI 與控制器接線完成；新增 14 案通過，cargo check 結束碼 0。相關回歸仍有 3 個 TW 治癒資源資格失敗，另有 53 筆技能缺少原始圖示對應，未宣稱整體全通過。**

## 做了什麼

- 接入正常原生入口 `native_login_window`，把原技能下拉選單改為可開關、捲動的技能書。完整列出 S1E／S1F 已學會的 1–220 技能 ID，補齊 `docs/SKILLS-STATUS.md` 的 181 筆技能名稱目錄。尚未接入的技能仍標示「尚未支援」；本張沒有擴大任何技能的送包格式或伺服器規則。
- 新增八格 F5–F12 快捷列。先選技能，再按 Ctrl＋F5–F12 或 Ctrl＋點擊格子綁定；Shift＋快捷鍵／點擊清除；直接按快捷鍵或點格子施放。保留 F1／F2／F3。F10 處理 `WM_SYSKEYDOWN`，Alt 組合鍵交給 Windows，自動重複按鍵不重送。
- 綁定儲存技能 ID，不使用清單位置。技能移除後清除綁定，再學會不自動恢復；連線、世界、施法者 incarnation、位置校正與死亡使舊選取／綁定失效。綁定僅限目前工作階段，沒有新增持久設定檔。
- 技能書、快捷列與原快捷鍵共用 `choice_id_at`、`LoginController::cast`／`cast_ground`。快捷格中的火牆／生命之泉使用該格技能 ID，不使用技能書當時選取的另一個技能。送出前重新驗證已學會、HP／MP、目標身分、範圍、地面選點與目前畫面。
- 技能圖示使用原始 `SpellIcon.idx` 的一基技能 ID 對應，再由既有原生圖示讀取器載入 TBT／LBT。載入採背景工作、最多 64 個目前可見 ID、取消與關閉回收；不把技能 ID 當成圖形 ID。缺圖仍顯示名稱／編號，沒有猜測替代圖。
- 新增待回覆施法 context，涵蓋連線、世界、施法者、位置校正及控制中斷計數。死亡、離場、技能移除、目標失效、傳送鎖與限制狀態會退休待回覆請求；同批封包中限制插入後又解除也能辨識。已收到施法動作後目標死亡，不誤判成尚未送出的取消。不預測扣 MP、不撤回已送封包、不新增重試。
- 把 UI 與最終送出前的動作准入集中到 `skill_action_ready`，保留「等待回覆、技能 reuse、客戶端時鐘、資源動畫完成」各自的門檻。1480ms 不是全技能固定冷卻；測試用 37 ticks／25fps 的明示合成動畫驗證 1479ms 拒絕、1480ms 開放，並驗證速度 1／2 對應 1110／2960ms，以及 5000ms reuse 不被較短動畫提前解鎖。

第 2／3 張的 buff／AC／冷卻／MP／狀態圖示，以及第 3／3 張 HPBAR 未在本張變更。`TASKS.md` 的整張 T046 未改為完成；`docs/REPORT.md` 未修改。

## 已驗證

### 指令與門檻

工作目錄為此 worktree 的 `LiTo381-Modern/`。使用 Windows x64 MSVC、既有離線 Cargo cache、`target/current`；暫存及日誌放此 worktree 的 `scratch/`。以 `維護工具/run-local-test-suite.ps1` 原有 `Measure-TestResult` 函式解析各次結果，拒絕非零退出碼、0 tests、failed > 0、結果缺漏或計數不符。未執行整份本機套件，沒有把 ignored 算通過。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib skill_ui_tests -- --test-threads=1 --nocapture
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib real_skill_icons_and_async_service -- --test-threads=1 --nocapture --ignored
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test <下表 target> -- --test-threads=1 --nocapture
```

圖示資源測試明確設定 `LITO_CLIENT_ROOT=D:\lineage開發\0-客戶端`，只讀取資源。以下為最終實際執行結果；整體回歸不是全綠。

| 檢查／測試 | 結束碼 | 門檻 | `test result:` 原文 |
|---|---:|---|---|
| `cargo check --all-targets` | 0 | PASS | 不適用（編譯檢查） |
| 新增 `--lib skill_ui_tests` | 0 | PASS | `test result: ok. 13 passed; 0 failed; 1 ignored; 0 measured; 2502 filtered out; finished in 1.29s` |
| 原始圖示 `--lib real_skill_icons_and_async_service --ignored` | 0 | PASS | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2515 filtered out; finished in 2.10s` |
| `native_skills` | 101 | FAIL | `test result: FAILED. 10 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.21s` |
| `native_skill_selection` | 0 | PASS | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.81s` |
| `native_attack_skills` | 0 | PASS | `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.94s` |
| `native_ground_skills` | 0 | PASS | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.92s` |
| `native_reagent_skills` | 0 | PASS | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.57s` |
| `native_inventory_icons` | 0 | PASS | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.49s` |

新增 14 個測試分兩次執行全部通過（一般 13、明確啟用資源測試 1）；相關既有回歸為 54 passed／3 failed。一般測試的 1 ignored 是另次已實際執行的原始圖示案例，沒有把忽略本身計為通過。


### 驗證界線與修正紀錄

- 新增測試包括完整技能清單、ID 綁定／重排／清除、生命週期失效、地面技能與目標重新驗證、限制解除同批競態、拒絕／逾時、送出失敗不保留 pending、動畫邊界／速度／reuse，以及 F10／自動重複輸入。
- 自建 `127.0.0.1:0` 加密 loopback 僅驗證客戶端請求與佇列：對端逐一比對兩份解密 C14；同格與跨格重複請求、收到回覆但動畫未完成時，佇列計數必須不變。測試中的施法回覆是明示合成狀態，不是正式伺服器技能行為證據。
- 首次編譯修正新增圖示服務的 Drop 所有權初始化，以及 DPI 型別。loopback 首輪測試把加密計數器當成封包數，錯誤期待每包加 1；已改為拒絕時計數完全不變、成功時計數改變，並保留對端兩份完整解密封包比對。修正前的失敗不列通過。
- 實讀 181 筆目錄圖示：128 筆可解碼，53 筆無可用原始對應。缺少 ID：`87–91, 112, 132–134, 160, 169–176, 181–195, 201–220`。例如技能 2 → 圖形 384、3 → 385、4 → 386、58 → 419、63 → 784；沒有把高 ID 技能的 buff 圖示當成施法圖示。

### 未通過的回歸與前置修正

`native_skills` 下列三案失敗，錯誤均為 `unsupported shared Honor presentation: unreviewed TW healing catalog generation`：

- `original_tw_basic_heal_frames_sound_clock_and_retirement_preserve_authority`
- `tw_higher_full_heal_group_budget_rejects_both_layers_atomically`
- `tw_higher_heal_original_slots_clothes_sound_and_retirement_preserve_authority`

已核對 `tower_assets.rs` 的 `load_reviewed_tw_healing_family`：要求 pack SHA `B3788DF4C7F077EF3FF15B070A5D020CCB224E48DEC3757C4CD34528FCA4B218`，實際 `D:\lineage開發\0-客戶端\launcher\TW13081901.pak` 的 SHA256 為 `92B512A32349D0775E1036F1832E21F1141AE66E64821D6978F0879246E59CA7`，與 [#111 資源盤點](issue-111.md) 一致。本張未修改 `tower_assets.rs`、`native_skill_effects.rs` 或 `tests/native_skills.rs`，也沒有更換資源、放寬 pin 或略過失敗案例。未另外建置起點基線重跑；此處證據限於未變更的資格門檻與實際 pack 不符。請由 T057／資源任務釐清並重跑這三案。

`native_inventory_icons` 首輪 6 passed／1 failed，原因是新 worktree 缺少測試預期的 `target/current/scratch` 父目錄，`create_dir` 回傳 NotFound。只建立此前置目錄後原封不動重跑，7／7 通過，沒有修改既有測試斷言。

## 推測

- 此次准入與中斷檢查可阻止原生 UI 在舊目標、舊角色或未結束動畫上重送技能；實際操作體感仍需要 GUI 驗收。
- 1480ms 來源描述位於伺服器 `docs/maintenance/native-cast-timeline-current.zh-TW.md`，描述的是隔離候選時間窗。此次合成動畫回歸只證明客戶端遵守動畫完成門檻，不代表重新驗證 E149 全職業、全 sprite 或伺服器動畫換算。

## 待驗證／未完成事項

- 未啟動遊戲、未連正式服務或 DB，未做真人 GUI、DPI／縮放、視窗焦點、讀屏或音效裝置驗收。未執行 Release、完整本機套件、G06 或 T030。
- 53 筆技能圖示的正式來源／映射仍待資源核對；目前採明確缺圖顯示，不能稱全部技能圖像已完成。
- 目錄完整不等於 181 技能皆可施放：尚未接入的技能送包、目標及效果由 T020／T021／T022／T027 等主線任務處理，本張維持既有拒絕。
- 不宣稱還原原版所有特殊投射物／動畫中斷規則；本張處理待回覆請求與 UI 意圖的生命週期，保留伺服器已確認的效果。

## 產出路徑與給下一棒的提示

- 報告：`docs/reports/issue-181.md`。
- 原生 UI：`LiTo381-Modern/src/modern_lito/native_skill_window.rs`、`native_login_window.rs`。
- 模型／目錄／資源：同目錄 `native_skill_selection.rs`、`native_skill_catalog.rs`、`native_inventory_icons.rs`。
- 控制器／中斷：同目錄 `native_login.rs`、`native_skills.rs`。
- 新測試：同目錄 `native_skillbook_tests.rs`、`native_skill_controller_tests.rs`，及圖示服務／UI 模組內的 `skill_ui_tests`。
- 本機未追蹤證據：`scratch/*.log`；既有資源整合測試的產物在 `LiTo381-Modern/target/current/verification/`。不提交建置產物或日誌。
- 請 Claude 審查本分支，再由使用者決定合併。T046 第 2 張接線時沿用共用准入方法，避免另建快捷列送包入口；第 3 張不需依賴本張圖示快取。圖示補齊交資源任務查核上述 53 個 ID 的正式映射。M2 完成後，再由使用者進行完整 GUI 驗收。
