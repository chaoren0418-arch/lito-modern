# Issue #187 — T050 subtype33 完整性挑戰與 C_ADDR

## 結論

**程式接線完成（第 3 輪，claude-local）；實機驗收待驗證。** GAME ingress 收到 `S_EVENT/33` 後，若範圍是正式範圍（`0x004ED840`、長度 5），客戶端會在送達該挑戰的同一條連線送出 13-byte `C_ADDR`。其他範圍只記錄、不回覆；同一挑戰只回一次；換世界或退役時清除。

這個回覆是**相容設定檔（compatibility profile）**，不是原版程式記憶體的實測：

- Rust 客戶端不讀伺服器給的位址，也不讀 LiTo.bin。
- 回覆的 checksum 是用正式 seed／CRC 演算法，對伺服器原始碼記載的 relocation-free `call $+5` 指令編碼（`E8 00 00 00 00`）算出來的。
- 伺服器常數 `0x1569D958` 只在測試裡當 oracle 用，產生回覆的程式碼沒有寫死這個值。
- protectedState 回報 0，因為 Rust 客戶端沒有原版保護層。

前兩輪不敢接線，是因為沒有「Rust 該量測什麼」的契約。本輪的依據是：伺服器原始碼明說該範圍是固定、與重定位無關的指令編碼；伺服器 `nativeIntegrityPreWorldProbe` 的註解也預想了「compatibility-only client」。所以照這個指令編碼回覆屬於協定相容，並且在程式、文件、報告都標明不是記憶體實測。若雲端審查判定此語意不可接受，只要拿掉 `native_login.rs` 的送出那 6 行，就會退回「只記錄」。

## 做了什麼

- `LiTo381-Modern/src/modern_lito/native_integrity.rs`
  - 新增 `seeded_crc`（反射多項式 0xEDB88320，seed 取反當初值，結果再取反），以及常數 `LEGACY_ANCHOR_ADDRESS`／`LEGACY_ANCHOR_BYTES`／`CRC_SEED`／`PROTECTED_STATE`。
  - 新增 `Challenge::compatibility_proof()`：只有位址與長度都完全符合時才產生回覆。
  - `Status` 改為 `NotRequested`／`ReplyPending`／`ReplyQueued`／`UnsupportedRange`。`ReplyQueued` 只代表已排進送出佇列，不代表伺服器已接受。
- `LiTo381-Modern/src/modern_lito/native_world.rs`
  - 收到挑戰時判定狀態：同一挑戰已回過就不重設；新 id 重新進入 pending。
  - 新增 `integrity_reply()`／`integrity_reply_queued()`。
- `LiTo381-Modern/src/modern_lito/native_login.rs`
  - `world.accept` 之後，若有 pending 回覆，就以目前 epoch 呼叫 `transport.send`，成功才標記 `ReplyQueued`；送出失敗會往上傳錯誤，狀態維持 pending。
  - 解決與 main 的合併衝突：`resurrection_tests` 與 integrity 測試兩個模組都保留。
  - 改寫兩個控制器單元測試，新增一個。
- `LiTo381-Modern/src/modern_lito/native_integrity_controller_tests.rs`（新檔）：自有 127.0.0.1 loopback peer 測試。依序送 `CHALLENGE, CHALLENGE, UNKNOWN, FRESH`，並逐 byte 比對只收到兩個 C_ADDR（重複的與範圍外的都沒有回覆）。
- `docs/NATIVE-WORLD.zh-TW.md:18`：更新為目前行為與界線。
- 已合併 `origin/main`。`docs/REPORT.md` 沒有變動。

## 已驗證

- 正式 JAR SHA256 `E1498871…D498`。反組譯證據沿用第 1 輪，見 git 歷史中本報告的前一版：
  - 挑戰：`7B 21 id addr len`，14 bytes。
  - 回覆：`2A id checksum protectedState`，13 bytes。
  - 伺服器驗證條件：pending、BATTLE、同房、30 秒內、id 相同、CRC 相同。
  - 伺服器沒有回傳成功封包，`clientIntegrityVerified` 只寫不讀。
- 伺服器原始碼 `NativeClientIntegrity.java`（只讀）：RANGE `0x004ED840`/5、seed `0xD883C01B`、canonical `E8 00 00 00 00`、expected `0x1569D958`。伺服器測試 `ProtocolAddressIntegrityTest.reproducesNativeSeededCrcOverRelocationFreeAnchor` 也是同一組值。
- `LiTo.bin`（SHA `CE6EE68A…`）靜態 PE 解析（`scratch/issue-187/pe_probe.py`，只讀）：VA `0x4ED840` 落在第一個區段，但 raw size 只有 0x98000，超出 raw 範圍；另有 `Themida` 區段。所以磁碟上的檔案不能當量測來源，這個決定已寫進程式註解。
- 以下命令在 `LiTo381-Modern/` 執行：

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
EXIT=0（Finished `dev` profile ... in 4m 43s；只有既有 unused 警告）

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-run
EXIT=0（Finished `test` profile ... in 6m 54s；只編譯，未執行）
```

## 未驗證／待驗證

- **單元測試沒有在本機執行**。依使用者 2026-11-09 規定，本機不執行編譯出來的程式（含 cargo test 執行檔），所以沒有 `test result:` 原文。請看 PR CI 的 `cargo test --lib native_integrity`（預期 9 個 codec／世界測試，加 3 個 ingress 測試、1 個 loopback 測試）與 `--test native_world`。CI 結果出來前不算通過。
- 「伺服器端接受」只是推測：E149 的 `submitAddressProof` 只在日誌印 `verified C_ADDR native-integrity`，沒有回傳封包。要在隔離伺服器的日誌確認這行，並確認沒有 `rejected ... reason=...`。這需要啟動伺服器與客戶端，交給 Codex／使用者做真機驗證。
- `nativeIntegrityPreWorldProbe=true`（挑戰在 S_WORLD 之前送達）的路徑：程式上不限 phase，應該同樣會回覆，但沒有 fixture 驗證。
- 長時間戰鬥相容：伺服器目前不會因為沒回覆而踢人（推測，依據是反組譯結果），所以無法用「沒被踢」來證明。
- 前次背景搜尋 `LiTo.unpacked*`（全 3-lito開發 遞迴）逾時被停止，沒有找到脫殼映像。本輪的設計也不依賴脫殼映像。

## 推測

- protectedState 在原版的語意（是否處於保護殼狀態）沒有證據。伺服器只把它記進日誌，所以 Rust 回報 0。
- 若日後伺服器改變挑戰範圍，Rust 會標成 `UnsupportedRange` 而不回覆。到時需要伺服器 repo 提供新的 canonical bytes 契約。

## 產出路徑

- 分支 `task/187`；報告 `docs/reports/issue-187.md`。
- 本機證據放在 `scratch/issue-187/`，不入 git。

## 給下一棒的提示

1. 先看 PR CI：integrity 相關測試必須是非零數量，並且 failed=0。
2. 雲端審查請裁定「相容設定檔回覆」的語意是否可接受。不接受的話，移除 `native_login.rs` 中 `integrity_reply()` 那段送出即可。
3. 真機驗證（Codex）：在隔離伺服器進房開戰，伺服器日誌應出現 `verified C_ADDR native-integrity ... protected-state=0`；預檢模式（PreWorldProbe）也要驗一次。
4. TASKS.md 的 T050 狀態由調度器在真機驗證後更新。
