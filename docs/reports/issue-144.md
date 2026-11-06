# Issue #144 — T021：技能 98、99、100 與 97／98 權威驗證

日期：2026-11-06。分支 `task/144`，基底 `928d0a3`。只處理本張範圍；沒有修改 `TASKS.md` 狀態或 `docs/REPORT.md`。

## 做了什麼

| 技能 | 送出 body（含 opcode） | 接收／呈現 |
|---|---|---|
| 97 暗隱術 | `14 0C 00`，無尾段，reuse 9990ms | 沿用 #142／#143 的 2941 靜態 bank／queue；隱身只依 S34，移除與現身只依權威物件封包 |
| 98 附加劇毒 | `14 0C 01`，無尾段，reuse 0 | S37/2942，九幀、source1 Sound1505，逐幀整數 callback 共1496ms；不建立本機320秒 buff、毒發機率、傷害或材料扣除 |
| 99 影之防護 | `14 0C 02`，無尾段，reuse 0 | S37/2943 主體 Screen＋2954 附層 Multiply，共用13格進度；source1 Sound1509。AC 只接受 S52 |
| 100 提煉魔石 | `14 0C 03 + d(inventory object ID)`，共7 bytes，reuse 0 | S37/2944，九幀、source1 Sound1502。結果／消耗只依訊息與背包權威更新 |

- 新增四個技能選單名稱。97～99 忽略 UI 中殘留的角色選取；直接傳入角色目標的請求仍拒絕。100 沿現有背包 `ItemTarget`，綁定連線／背包 revision／物品內容／角色 incarnation／地圖與控制戳記，不能走角色目標路徑。共用提示改為「施法物品」。
- 排除100的 LFC1 Ctrl 強制攻擊 trailer，避免七位元組物品請求被誤認為角色請求。
- 補入伺服器訊息279的拒絕追蹤與「HP 不足」回饋。保留278／299等既有拒絕優先序；S37 特效不是成功 ACK。
- 原資源 bank 僅在收到2942～2944後載入。精確核對三個主體定義、2954的無獨立動畫附層、音效／ticks／控制屬性與四個 SPR 雜湊。
- 2943 的 bitmap subframe0～3 是原始 no-block 空白。新增專用載入策略，只在固定雜湊、尺寸、13格與完整空白欄位相符時接受；此時只畫2954。後續兩層使用同一批次提交，保留既有預檢與混色。
- 沿現行 Scene／finite History 保存 callback、seen sequence 與原資源重新載入；不重播 source1 音效，離開來源 actor 後仍在收包座標播放至有限期限，世界換代清除舊進度。

## 來源核對

唯讀 `LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar`（原專案路徑）的 SHA256：`E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。以 `javap -p -c` 靜態核對 BattleSpellCatalog、Protocol、NativeSelfCastAction、NativeSourceSkillCastAdmission、NativeCastMaterial；沒有載入或執行遊戲 class。

`NativeSourceSkillCastAdmission` 的 preflight→reservation→material→commit／失敗cancel順序僅作靜態核對。`ClientSession.stdout.txt` 使用原專案 `驗證紀錄/current/native-formal-reference-refresh/20261023-192435-953238/` 的唯讀來源，核對100的物品查找、MP、結果與inventory replace路徑。

**歷史「S08」命名差異**：E149 `Protocol.removeObject` 實際寫入 decimal21（wire `0x15`）；`Protocol.invisible` 寫入decimal52（wire `0x34`）。本次依正式 JAR 驗證，沒有新增虛構的0x08移除處理器。

原 list 定義來自唯讀 `D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr`，現行載入器仍要求正式 catalog SHA256 `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127`。測試 fixture 只摘錄四個文字定義；沒有複製 SPR／WAV 或套用隔離候選 Runtime。

| 原 SPR | bytes／格數 | SHA256 |
|---|---|---|
| 2942-0.spr | 10474／9 | `AF84AF8C0CE0BD7459545C6E94E1676C9DF2826C785802362848FDB0DD8ABAFC` |
| 2943-0.spr | 21345／13 | `FD49E50E4EB17E19DE23048925AB15CA9FC3505A152C42B72D4E9762AD3FE716` |
| 2944-0.spr | 8981／9 | `482C7D8E9685376D72E8060CB00415FF26512FDB0D9DD79A7BB7DE0FB8FF64F9` |
| 2954-0.spr | 8244／13 | `03FB56D0D1A61EEC7CB903692B4C6C04C5FE49664A40254A3D58235DCE3CA982` |

## 已驗證

執行 `powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/run-local-test-suite.ps1 -Quick`，兩步均 PASS，套件exit=0。另行 opt-in 的專項測試與97回歸使用相同固定Cargo參數。新案例的結果再直接載入原runner的 `Measure-TestResult` 函式判讀，並要求passed>0；0 tests、failed>0、結束碼非零、解析失敗均不能通過。不是只看編譯完成或退出碼。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
exit=0; gate=PASS

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast
exit=0; gate=PASS
test result: ok. 2025 passed; 0 failed; 587 ignored; 0 measured; 0 filtered out; finished in 9.14s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib t021_ -- --include-ignored --test-threads=1
exit=0; gate=PASS
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2606 filtered out; finished in 14.41s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_static2941::tests:: -- --include-ignored --test-threads=1
exit=0; gate=PASS
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 2581 filtered out; finished in 7.72s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t021
exit=0; gate=PASS
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

97的opt-in環境：`LITO_STATIC2941_ROOT=D:/lineage開發/0-客戶端`；`LITO_NATIVE_CLOCK_PE=D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`；`LITO_NATIVE_CALLBACK_ASM=D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm`。僅唯讀原PE／ASM，沒有執行原程式。31案包含全部12 ready gates。

新案共13個：七個整合案例驗證四技能封包／選單、MP與HP正值與零／未知值邊界、材料缺失、物品換代、S34／移除現身、拒絕／逾時、HP／MP／inventory／AC／中毒色權威與截斷反例；六個library案例驗證精確定義反例、有限callback、Ctrl排除、History換代、原SPR／PCM與圖層，以及真實controller加密loopback。最後兩案opt-in，其餘純案納入一般library或整合執行。

- `scratch/t021-resources.log`：原圖／音效、兩層混色、2943合法空白、History同序號不重播、來源actor移除後固定位置、1496ms退役；所有技能無本機扣款或狀態改寫。原PCM已載入，不代表裝置聲音已播放。
- `scratch/t021-final.log`：整合案例；`scratch/static2941.log`：97回歸；`scratch/gate-results.txt`：runner門檻判讀。
- Quick總表與原始日誌：`LiTo381-Modern/target/current/verification/local-test-suite/20261106-210442-278-9236/{summary.txt,001.log,002.log}`。既有warnings仍存在。

開發中失敗保留：`scratch/t021-resources-first.log` exit=101，找到2943真實空白格，未降低通用資源門檻、刪除案例或當作通過。原文：

```text
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 2608 filtered out; finished in 12.67s
```

另一次性SPR診斷工具初次rustc呼叫未帶完整host／native依賴、以及誤選x86匯入庫而連結失敗；修正為x64後唯讀解碼四個原SPR，沒有啟動遊戲。診斷資料在 `scratch/spr-shapes.txt`，不計測試通過。

## 推測

原控制屬性與有限 callback／混色對應沿既有 renderer；原檔解碼與記憶體提交可支持實作，但不能取代原生視窗的逐像素並排驗收。

## 待驗證／未驗證什麼

- 未啟動遊戲、客戶端視窗或正式服務，未連正式 DB；沒有附加／停止既有程序。隔離連線只綁定本次測試建立的 `127.0.0.1:0` listener。
- 未做實體聲音播放、滑鼠／鍵盤、DPI 或完整 GUI 驗收。
- **L13 的正式伺服器自然期限仍待驗證**：本次證明客戶端的有限特效自然退役、History不重播，以及32／320／961秒測試時間不會自行解除權威狀態；沒有實際等待正式伺服器的32秒隱身、320秒附加劇毒或960秒護甲到期後送包。
- MP／HP／材料邊界驗證的是客戶端送包與拒絕回饋；沒有宣稱正式服務的折扣、HP扣除、40321消耗、原子rollback、毒發機率、提煉機率或產物來源已通過。98仍保留「伺服器待定：材料40321」分類。
- 未執行其他分張、G06 640次、Release 測試或 Full 全白名單套件。

## 產出路徑與給下一棒的提示

- 接線：`LiTo381-Modern/src/modern_lito/native_skills.rs`、`native_skill_selection.rs`、`native_login_window.rs`、`native_forced_combat.rs`、`native_world.rs`。
- 呈現：同目錄 `native_dark_elf_effects.rs`、`native_skill_effects.rs`、`tower_assets.rs`、`mod.rs`。
- 測試：`tests/native_t021.rs`、`tests/fixtures/native-dark-elf.spr.txt`、`native_dark_elf_effect_tests.rs`、`native_t021_render_tests.rs`，另在既有forced-combat與controller callback測試加入本張案例。新案已加入 `維護工具/test-whitelist/` 對應清單。
- 狀態表只回填 `docs/SKILLS-STATUS.md` 的97～100；本報告為 `docs/reports/issue-144.md`。診斷／日誌在此 worktree `scratch/`，產物在 `LiTo381-Modern/target/current/`，不提交。
- 下一棒由雲端 Claude 審查封包尾段、LFC1排除、279回饋、2943空白／2954附層與History範圍；調度器接續原定本機驗收安排。正式自然期限與材料等玩法驗證仍由本機測試任務接手，不能將本張離線結果當作完整 M2／GUI 通過。
