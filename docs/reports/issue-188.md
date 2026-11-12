# Issue #188 — T054-1：G06.1 房間管理契約整合

本張只處理建房、設定、密碼、踢人、房主轉移，以及同頻道跨來源房間目錄、預設房重用／改名。基底為 `6536a8b901f0fe3951aec3baf20b0b5aed3ef8e2`，在既有 `task/188` worktree 修改。沒有套用凍結候選、變更 `TASKS.md` 整項狀態或修改 `docs/REPORT.md`。

## 做了什麼

- `native_lobby.rs` 補上正式 `S_CC` subtype 1～5：名稱、屬性、人數、容量、模式欄位更新。依伺服器房號更新既有列；未知／已刪除房號的增量不建立新列。保留完整 WORD 數值與清單順序，不從來源名稱、列索引或本地人數推算房間。
- `native_login.rs` 將完整列及增量接入同一階段檢查與 refresh buffer。刷新期間只更新待提交清單，收到對應 `S_CA` 才替換顯示；`S_CF` 同時刪除顯示與暫存列。原有依房號保留選取的 `RoomChoice` 可繼續使用。
- `native_room.rs` 共用送出／接收設定的選項驗證，拒絕不合法 `S_CE` 設定、`S_D5/1` 選項與零房號的房主通知；拒絕在改寫客戶端快照前完成。保留 wire 容量 2～200 與一般建房／修改入口容量 2～100 的區別。
- 踢人回覆必須對應目前房間的待處理請求，拒絕重複成功回覆；只有目標的離房通知才完成成功的踢人流程。重新加入成功時清除前一房間的踢人結果與通知。
- 新增 `native_room_admin_tests.rs`，直接測試正式 `LoginController` 接收路徑與入口，並用自建 `127.0.0.1:0` 加密對端檢查實際發送封包。沒有啟動遊戲或 Java 伺服器。

## 契約與證據邊界

| 行為 | 客戶端契約 | 核對來源 |
|---|---|---|
| 建房 | `C_CC` 不帶指定房號；`S_CE` 決定房號，還需自己的 `S_CD` 才完成進房。接受新 ID 與既有預設房 ID，不自行分配或假設重用。 | `docs/PROTOCOL-CONTRACT.md` §5；`ClientSession.java:11460` |
| 設定、密碼 | `C_DD` 成功與拒絕都回權威 `S_D5/0＋1`；收到 subtype 1 設定回覆後回到可操作階段，不把草稿當成成功。密碼只在請求／接收解析時使用，快照只保留是否需要密碼。 | `ClientSession.java:16709`；既有 `native-room-admin-current.json` 144 組 wire 向量 |
| 加入拒絕 | 接受 1、3、4、5、7、10、11、12、13、14、20、21，清掉待加入名冊與期限；11 為密碼錯誤、7 為被踢後拒絕。測試回覆不代表已動態驗證伺服器的拒絕條件。 | `docs/PROTOCOL-CONTRACT.md` §5 |
| 踢人 | 使用完整收到的顯示身分，禁止自己、未知成員與非房主操作；請求者收到 `S_FC` 後仍等待目標 `S_D0`，`S_D6` 只顯示通知；被踢者以自己的 `S_D0` 返回大廳並刷新清單。 | `ClientSession.java:16600`；`GameHub.java:9997` |
| 房主轉移 | 以同一房間的 `S_D5/5` 或摘要指定的名稱為準，不能從離房或名冊順序選新房主；收到轉移後，舊編輯表單與成員選擇失效。`Member.master` 是既有玩家欄位，不拿來推算房主。 | `ClientSession.java:14792`；既有 `native_room_ui.rs` |
| 跨來源清單 | 同頻道共享伺服器房號，清單封包沒有來源篩選欄位。更新清單不等於加入房間，也不賦予房主權限。 | `LobbyRoomDirectory.java:35`；`NativeRoomDirectoryRow.java:25` |
| 預設房／新房 | 重用、改名、重設與刪除均由收到的 ID 和封包決定；沒有「收到建房要求就覆蓋第一列」的本地規則。 | `room-directory-0.6.54.json:21`；`GameHub.java:1389`、`:12849`；`ClientSession.java:14754` |

相鄰伺服器來源只讀取 `src/main/java/local/lito381/server/` 與指定維護文件，未讀取機密設定、未開啟封存、未寫入相鄰 repo。來源根目錄為 `D:\lineage開發\3-lito開發\lito伺服器\LiTo381-Server`。

| 唯讀來源 | SHA256 |
|---|---|
| `ClientSession.java` | `6CE00B818E73BF509152A460DAB4831FD5EEFC29E6344E7081C4948D40046F8B` |
| `GameHub.java` | `6A123687BB65FDCE4A6B7F8FEC8317DA0B3E949C48758F43766FAC2F9FE4886D` |
| `NativeRoomDirectoryRow.java` | `8E41577E40F804674885DC5E4CF9250392C7A461E7E4529123640CC2622437A4` |
| `LobbyRoomDirectory.java` | `646062C610124055BD6893214363783DE447448A87A2FA326BFA3A174B7BC03D` |

## 已驗證／怎麼驗證

2026-11-06，在本 worktree 的 `LiTo381-Modern/` 執行。子程序清除 `LITO_*`，`TEMP`／`TMP` 指向工作樹 `target/current/scratch`。所有步驟共用 `--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`。

一次性 runner 從 `維護工具/run-local-test-suite.ps1` 的 PowerShell AST 取得原始 `Measure-TestResult` 函式，直接用同一函式判定：結束碼 0、一組可解析的 `running`／`test result:`、執行數吻合、不得為 0 tests 或 failed>0；沒有將 ignored 算成通過。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
exit=0; GATE=PASS
```

編譯檢查沒有 `test result:`。仍有專案既有警告，不宣稱零警告。

### 新增控制器與加密 loopback 測試

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib room_admin_tests
exit=0; GATE=PASS
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 2490 filtered out; finished in 0.57s
```

12 個測試涵蓋：CE＋自己的 CD 才完成建房、伺服器新 ID／重用 ID、12 種加入拒絕碼、設定成功／拒絕回覆、密碼移除與快照不含密碼、房主轉移與舊表單失效、精確成員身分、踢人成功／拒絕／重複回覆、跨來源清單、改名／重設／刪除、refresh buffer、畸形封包不改寫狀態、送出失敗不預先改狀態、逾時清除與合法 wire 容量上限。

loopback 在動態 loopback port 使用真實 `CipherState`／TCP owner，採 2-byte read chunk、3-byte write chunk。獨立 literal request 驗證建密碼房 → 修改／移除密碼 → 踢人拒絕 → 踢人成功 → 主動離房與清單刷新 → 錯誤密碼拒絕 → 正確密碼再入房 → 被踢者離房與刷新。對端只依固定腳本回覆，不模擬或驗收 GameHub 規則。測試最後取消連線、poll 至 worker 回收並 join 自建對端。

初輪檢查曾因測試 helper 的 `Zeroizing<Vec<u8>>` 使用方式及連線 phase 名稱編譯失敗（exit 101），已修正。初輪測試為 10 passed／1 failed（exit 101），原因是 `cancel()` 只發關閉訊號，測試立即檢查 worker 回收；已改成限時 poll 等待。最終重跑上述 12 passed，沒有將初輪結果重複加總或算為通過。初輪日誌保留在相同驗證目錄。

### 既有相關回歸

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_lobby
exit=0; GATE=PASS
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room
exit=0; GATE=PASS
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room_admin
exit=0; GATE=PASS
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room_departures
exit=0; GATE=PASS
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room_force
exit=0; GATE=PASS
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room_mode
exit=0; GATE=PASS
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_room_ui
exit=0; GATE=PASS
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

八組測試合計 **35 passed、0 failed、0 ignored**，各組均符合原 runner 的解析門檻。`git -c core.whitespace=cr-at-eol diff --check` 通過。

## 推測

- 0.6.54 維護紀錄列出「default room reuse/rename」；現行 `GameHub.createRoom` 明確建立新 ID，空的預設房則保留並重設。這兩份證據屬不同版本。本張驗證客戶端可以處理兩種權威房號結果，不宣稱現行建房一定重用預設房。
- 唯讀原始碼與 `PROTOCOL-CONTRACT.md` 支持上述封包及轉移契約，但不等同這輪實際執行正式 JAR。既有 `native-room-current.json`、`native-room-admin-current.json` 宣告的來源 JAR SHA 是 `bbc59308…`，不能改稱本輪 `E1498871…` 的 live 證據。
- 跨來源測試是兩個不同來源快照接收同頻道封包的客戶端驗證；伺服器實際廣播對象與 channel 隔離由服務端維護紀錄及原始碼支持，這輪沒有連線重驗。

## 待驗證／未驗證什麼

- 正式服務中的雙來源建房、密碼拒絕、房主斷線／離房轉移、踢人與重入拒絕、真實網路併發順序，以及實體 GUI 操作。未連正式服務或 DB，未啟停任何既有遊戲程序。
- 完整 G06、所有模式的伺服器規則、完整測試庫與 Release 未驗收。本輪離線回覆測試不能升級為完整 G06.1 live 通過。
- G06.2 的模式切換、隊伍、準備、觀戰與鏡頭，以及 ArenaBot 人數計算、測試房結算、隊伍人數、全員準備及中文拒絕訊息，由 T054-2 接手。本張只保留清單更新中收到的原值，未實作這些規則。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_lobby.rs`
- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_room.rs`
- `LiTo381-Modern/src/modern_lito/native_room_admin_tests.rs`
- `docs/reports/issue-188.md`
- 驗證日誌：`LiTo381-Modern/target/current/verification/issue-188/`（不入 git）。一次性驗證 runner：`scratch/verify-188.ps1`（不入 git）。

## 給下一棒的提示

- Claude 審查 `task/188` 的房間契約與測試證據，使用者決定合併；調度器在合併後補 `docs/REPORT.md` 索引。
- T054-2 延續現有房間狀態機；不要把 `S_CC` 清單更新當作 `S_D5` 房內設定確認，也不要因摘要改變就宣稱伺服器接受了使用者草稿。
- 測試若要連正式環境，須另有符合專案規範的測試任務；本張的 loopback 僅檢查客戶端，不能代替伺服器規則或 GUI 驗收。
