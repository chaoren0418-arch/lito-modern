# Issue #176：T044-1 移動正確性與移動鎖握手

本報告只涵蓋 T044 第 1 張；不將 T044 整項或 G04 標為完成。

## 做了什麼

- `World` 新增獨立的 `MovementLock`：`Open → AckPending → AwaitRelease → Open`。有效 S_BLINK 立即阻擋 C_MOVE；單一 TCP owner 接受精確的一位元組 C_BLINK（`5a`）後，才允許 S_PARALYSE 模式 17（`25 11`）解鎖。等待期間重複 S_BLINK 不重送確認；送出失敗保留鎖；未送確認的模式 17、重複解除與錯誤格式不會提前解鎖或清除其他麻痺／綁定限制。
- GUI 改走 `LoginController::move_presented`，送出前核對角色、場景、動作／移動版本、速度與動畫抵達狀態。`Frame` 保存輸入版本及握手狀態，舊畫面不能授權第二個在途步伐，跨越限制／解除事件的舊畫面也不能繼續送出。
- 將靜態地形預判收斂到 `World::checked_move_request`，供控制器與測試共用。地圖不符、缺 sector、封閉方向與既有伺服器限制都在送出及本地預測之前拒絕；沿用已存在的 LTmap 正交／對角通行規則。
- Quake 模式 24／25 不包含座標，也不是前一個 C_MOVE 的回覆。保留 `position_predicted`，不自行回滾可能已被接受的步伐，也不把動畫抵達／限制解除當作 ACK。既有按住輸入在限制事件後清除，後續 S_MOVE_OBJECT 才能取代預測；不新增自動重送或自行推定的逾時解鎖。
- 新增封包／畫面／控制器回歸測試與自有加密 loopback 案例；既有 `native_combat` 測試補齊 C_BLINK，確保它驗到的是真正解鎖後仍保留獨立限制。

## 已驗證

2026-11-06，在本工作樹的 `LiTo381-Modern/` 執行；所有 Cargo 呼叫使用 `--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`。暫存留在工作樹 `scratch/`，執行前清除子程序的 `LITO_*` 環境變數。沒有啟動遊戲程序或正式服務。

測試結果直接交由 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult` 原函式判定：結束碼必須為 0、只有一組可解析的 `running`／`test result:`、執行數吻合、不能有 0 tests 或失敗。以下五組均為 PASS，共 43 個測試、0 failed、0 ignored；初輪 13 個單元測試已被最終 14 個重跑取代，沒有重複加總。

### 建置檢查

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

最終版本檢查結束碼 **0**，門檻結果 **PASS**。此為編譯檢查，沒有 `test result:`；編譯器仍有專案既有警告，不宣稱零警告。

### 新增單元及加密 loopback 測試

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib movement_tests
exit=0
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 2476 filtered out; finished in 0.20s
```

涵蓋握手順序、重複請求合併、佇列失敗、格式錯誤的原子性、七組獨立限制、在途／舊畫面拒絕、Quake 限制與預測競態、權威修正、靜態碰撞及場景清除。使用合成 LTmap／Frame 的測試不宣稱 GUI 驗收。

loopback 案例使用本測試擁有的 `127.0.0.1:0` listener、真實加密 TCP owner 及分片讀寫；精確驗證 C_MOVE、單次 `5a`、`25 11` 後的下一步，以及 TX 計數沒有重複確認或額外在途移動。socket／worker 在測試末關閉並回收；不是正式 JAR／帳號登入測試。

### 既有相關回歸

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_motion
exit=0
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_navigation
exit=0
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_combat
exit=0
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_animation
exit=0
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

其中 `native_navigation` 唯讀原始 map564 地形，逐一比對 32,768 個方向判斷與既有 oracle；也包含 map510 路線、速度變化及按鍵中斷回歸。戰鬥與動畫測試涵蓋獨立限制解除、動作／死亡順序與插值。

`git -c core.whitespace=cr-at-eol diff --check` 通過；尊重既有追蹤檔案的 CRLF，沒有為格式檢查重寫整份檔案。

## 推測與協定限制

- 唯讀伺服器 `ClientSession.java:26820` 的 C_MOVE 路徑顯示：Cube: Quake 綁定期間直接返回；一般成功步伐不回送自己的 S_MOVE_OBJECT。`serverCubeQuakeBind` 送模式 24／25，沒有座標。因此純粹沒有回覆時，客戶端無法區分「成功但沒有 self echo」與「靜默拒絕」。本張保留明確預測及後續權威修正，不宣稱消除所有暫時座標偏差。
- 同一來源 `issueMovementLock`／`movementUnlock` 與 `docs/maintenance/movement-lock-0.6.66.md` 支持 opcode-only C_BLINK、模式 17 及獨立限制保留。這是唯讀原始碼核對，並非本輪執行正式 JAR 的證據。
- 原始碼 SHA256：`6CE00B818E73BF509152A460DAB4831FD5EEFC29E6344E7081C4948D40046F8B`；維護文件 SHA256：`6D5E7E14F642AE91292DC4B4A20D995ABAE862DC0DA28A7DF0AE9476B8A5F553`。來源位於相鄰 `lito伺服器/LiTo381-Server`，未修改。

## 待驗證／未驗證什麼

- 正式伺服器自然施放 Cube: Quake、真實網路時序及 GUI 按鍵／動畫驗收。本輪不啟動遊戲、不連正式服務或 DB，也不執行相鄰伺服器。
- 正式 JAR 對重複、未請求及格式錯誤 C_BLINK 的拒絕仍屬服務端待驗證；本輪客戶端測試只證明無鎖不送、待解除時不重送、輸出固定為 `5a`，以及接收端錯誤格式不改寫狀態。
- 完整測試庫、Release 與完整 G04 驗收不在本輪通過聲明內。
- 動態門／物件占位、場地邊界、城門及其他模式碰撞由 T044-2 接手；G04.4／G04.7 的生命週期同步由 T044-3 接手。本張沒有擴充這些規則。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_world.rs`
- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`
- `LiTo381-Modern/src/modern_lito/native_navigation.rs`
- `LiTo381-Modern/src/modern_lito/native_world_scene.rs`
- `LiTo381-Modern/src/modern_lito/native_movement_tests.rs`
- `LiTo381-Modern/src/modern_lito/native_login_movement_tests.rs`
- `LiTo381-Modern/tests/native_combat.rs`
- 本報告：`docs/reports/issue-176.md`
- 原始檢查／測試日誌：`LiTo381-Modern/target/current/verification/issue-176/`（不入 git）。

## 給下一棒的提示

- Claude 雲端審查本張差異及測試證據；使用者決定合併。保留 T044 三張分工，不因本張完成改動 `TASKS.md` 的整項狀態。
- 後續互動移動入口必須使用 `move_presented`。`move_checked`／`move_step` 保留給既有協定診斷探針，呼叫端仍須管理動畫節拍；它們不是畫面抵達證明。
- 不要以解除麻痺清除移動握手，也不要以模式 17 清除麻痺。新動態碰撞預判可接在現有預判流程，但不可推定所有玩家占格都阻擋。
- 如需判定真正靜默拒絕的每一步結果，需由協定設計／伺服器工作提供可辨識的權威回覆；不可用本地計時器冒充伺服器確認。
- `docs/REPORT.md` 保持未修改，由調度器合併後補索引。
