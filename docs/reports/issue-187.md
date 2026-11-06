# Issue #187 — T050 subtype33／C_ADDR 協定調查與未支援診斷

## 結論

**部分完成；T050 仍受阻，不能標記完成，也不能宣稱長時間戰鬥完整性相容。** 已核實正式 E149 JAR 的 wire contract，新增獨立解析、C_ADDR 編碼與離線測試，並把既有「只收到 challenge」的快照明確標為 `UnsupportedLegacyImage`。**執行中的客戶端仍不發送 C_ADDR**：缺少可代表 Rust 客戶端的有效證明契約，不能用伺服器的固定預期 CRC 冒充原版程式記憶體實測。

本次只寫 task/187 worktree；相鄰伺服器與 JAR 只讀，沒有修改 `docs/REPORT.md`、TASKS、正式設定或封存，沒有啟動遊戲、伺服器或連線 DB。

## 做了什麼／產出路徑

- `LiTo381-Modern/src/modern_lito/native_integrity.rs`：精確解析 14-byte `S_EVENT/33`，拒絕截斷、尾碼、錯誤 opcode/subtype 與零欄位；加入 13-byte C_ADDR 純編碼函式（不產生／不送出證明），及可序列化診斷狀態。
- `LiTo381-Modern/src/modern_lito/native_world.rs`：沿用 `integrity_challenge: Option<[u32; 3]>`，避免破壞既有 probe；增加 `integrity_status`，世界替換與退役時跟著清除。非零但不支援的地址／長度只記錄，不讀記憶體或依長度配置空間。
- `LiTo381-Modern/src/modern_lito/native_opcodes.rs`、`mod.rs`：登錄 `S_EVENT=0x7B`／`C_ADDR=0x2A` 與模組。
- `LiTo381-Modern/src/modern_lito/native_login.rs`：新增 GAME ingress 的離線單元測試，驗證沒有傳送、沒有延長原有 deadline、沒有把未支援挑戰視為成功；未修改正式傳送流程。
- 本報告：`docs/reports/issue-187.md`。

## 已驗證：正式協定的靜態證據

正式檔案：`D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar`。

本次 `Get-FileHash -Algorithm SHA256`：

```text
E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498
```

使用 `javap -classpath <上述 JAR> -p -c -constants local.lito381.server.<類別>`，對 `Protocol`、`Protocol$AddressProof`、`NativeClientIntegrity`、`ClientSession` 靜態反組譯，全部 exit 0；沒有執行 JAR 類別。輸出在不入 git 的 `scratch/issue-187/{Protocol,AddressProof,NativeClientIntegrity,ClientSession}.txt`，可用上述指令重建。

| 項目 | E149 實際契約／證據 |
|---|---|
| 挑戰 | `c(0x7B) c(33) d(challengeId) d(address) d(length)`；共 14 bytes；三個 d 為 little-endian u32 且非零。`Protocol.addressChallenge` offset 0–92 |
| 回覆 | `c(0x2A) d(challengeId) d(checksum) d(protectedState)`；共 13 bytes。`Protocol.addressProof` offset 0–95；`Protocol$AddressProof` 建構子 offset 4–17 確認欄位順序 |
| 回覆形狀限制 | challengeId 非零、protectedState 僅 0 或 1；checksum 的匹配在 session 另外檢查，Protocol parser 不拒絕零 checksum |
| 原版範圍 | `NativeClientIntegrity.RANGE_START=5167168 (0x004ED840)`，`RANGE_LENGTH=5` |
| CRC | seed `3632513051 (0xD883C01B)`；標準反射多項式 `0xEDB88320`；初值與結果取反。canonical bytes `E8 00 00 00 00`；預期 `359258456 (0x1569D958)`。`NativeClientIntegrity.seededCrc`、`crcTable`、static initializer |
| 挑戰綁定 | 非零隨機 u32 ID、currentRoomId、`nanoTime + 30_000_000_000`；`ClientSession.sendClientIntegrityChallenge` offset 0–67 |
| session 驗證 | 必須有 pending、BATTLE phase、同一非零房間、未過期、相同 ID、CRC 匹配。拒絕原因包括 `no-pending-challenge`／`invalid-phase`／`expired`／`challenge-mismatch`／`checksum-mismatch`。`submitAddressProof` offset 13–121 |
| 驗證成功 | 清除 pending 並設 `clientIntegrityVerified=true`，記錄 protectedState；此方法沒有回傳成功封包。`submitAddressProof` offset 153–183 |

另唯讀核對伺服器來源 `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/NativeClientIntegrity.java`：註解說明該地址為原版固定基址的 call/pop image-anchor helper，五 bytes 是 `call $+5`。來源與本次 JAR 的範圍、seed、canonical bytes、CRC 常數一致；註解語意與 bytecode 證據分開看待。

**重要界線**：30 秒是 `submitAddressProof` 收到回覆時的過期判定。本次 `ClientSession` 反組譯中的 `clientIntegrityVerified` 只有宣告／寫入，沒有讀取；不能把它說成已證實的 30 秒踢人機制，也不能從這點保證長時間相容。沒有發送回覆就沒有取得伺服器的完整性驗證成功。

## 已驗證：本機編譯與測試

以下命令工作目錄為此 worktree 的 `LiTo381-Modern/`；新測試使用現有 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult` 原函式判定，透過 PowerShell AST 只載入該函式，沒有執行整套或 ignored 測試。測試 runner 為不入 git 的 `scratch/issue-187/run-validation.ps1`，TEMP／TMP 均指向此 worktree 的 scratch。

新增測試命令：

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_integrity
```

修正後 **exit 0，Measure-TestResult=PASS**；日誌 `scratch/issue-187/integrity-validation.log`。包含 7 項 codec／世界狀態測試及 2 項 GAME ingress 測試。非零測試數、結果解析與 running 數量一致，failed／ignored 均為 0：

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 2476 filtered out; finished in 0.00s
```

既有世界回歸命令：

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_world
```

**exit 0，Measure-TestResult=PASS**；日誌 `scratch/issue-187/world-validation.log`：

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

最終編譯檢查（修改測試後重新執行）：

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

**exit 0，Measure-TestResult（Check）=PASS**；日誌 `scratch/issue-187/check-validation.log`。這是編譯檢查，沒有 `test result:`；不計入 14 個通過測試。編譯有 warnings，本次沒有擴及其他模組清理警告。`git diff --check` 亦通過。

首輪新測試 **exit 101，未通過**；日誌 `scratch/issue-187/integrity-first.log`：

```text
test result: FAILED. 4 passed; 5 failed; 0 ignored; 0 measured; 2476 filtered out; finished in 0.00s
```

原因是 fixture 未先送 `S_WORLD`／`S_D2`，既有 `World::accept` 在沒有地圖或尚未 started 時回傳 false，沒有進入挑戰分支。修正為先建立有效世界生命週期，再測試 receipt；並加強重設測試，明確斷言重設前確實接受過挑戰，避免因空狀態而假通過。沒有放寬正式世界守門。首輪編譯檢查 exit 0（`scratch/issue-187/check.log`）；修改測試後的最終檢查結果如上。

## 推測

- 要讓 Rust 客戶端取得有意義的完整性驗證，可能需要伺服器支援可區分客戶端種類／版本的證明契約。這是下一階段設計建議，尚未取得契約裁定。
- 回送硬編的預期 CRC 可能通過目前 checksum 比較，但不證明執行中的 Rust 客戶端或原版記憶體完整性；本次沒有實作、執行或宣稱通過這條路徑。

## 待驗證／未完成事項

1. **真正的證明來源與 C_ADDR runtime 傳送仍未完成**。現有編碼器只是 wire codec，不是整合完成或 stub 成功路徑。
2. Rust 的 protectedState 語意、證明版本協商、伺服器驗證方式尚無裁定；不能把 Rust EXE 檔案 CRC、磁碟上的原 BIN、或伺服器 canonical bytes 當成原版執行中記憶體證明。
3. 日後傳送必須綁 connection epoch／room／challenge／deadline，處理重複、換房、換世界、斷線與排隊失敗；本次沒有實作待發 proof，所以不宣稱這些傳送情境已驗證。
4. 真正的 ClientSession 接收與接受 C_ADDR、長時間戰鬥、GUI、正式服務端到端均未執行。沒有藉 codec 單元測試宣稱這些項目通過。
5. 診斷沿用世界封包生命週期，只涵蓋 `S_WORLD`／`S_D2` 之後的挑戰；伺服器可選的 pre-world probe 與跨世界的證明所有權仍須隨正式 producer／sender 設計處理。

## 給下一棒的提示

交 **Claude 雲端審查／調度器**：維持 T050 未完成，先裁定現有 E149 的原版記憶體挑戰如何適用 Rust 客戶端。若採新證明契約，另交伺服器 repo 的 plan/design 任務；本任務只讀相鄰 repo，不能在這裡修改其驗證規則。若因此需要更換 E149 驗收版本，依 `docs/PROTOCOL-CONTRACT.md` 的版本變更程序處理。

契約確定後，再由本機 code runner 完成合法證明 producer 與 runtime sender，搭配隔離 fixture 驗證過期、重複、舊 epoch／房間、CRC 拒絕及成功路徑；仍不得把「送出」當「伺服器已接受」。這項技術阻擋不需要使用者操作遊戲解除。
