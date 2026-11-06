# Issue #142：T020-1 暗隱術 ingress、資源 gate 與專用 queue

本張以 `task/142`、基底 `dd4fabb` 重寫 T020 第 1／2 張的來源；沒有套用 44 檔隔離候選，也沒有改 `TASKS.md` 或 `docs/REPORT.md`。第二次嘗試接續 `91e0b0a`／PR #222 的退回項；第三次嘗試接續 `84fd054`，結果與查核限制見下節。

**結論：本張驗收尚未閉合，不能標記完成。** 第二次嘗試補到同一 PE 內的啟動呼叫鏈，並修正首次取樣才初始化的行為；三個原外部跳板的 module/export 身分與原始初始化前狀態仍缺材料。沒有自行縮小 #142 驗收範圍，也沒有以新增測試通過替代原 API 身分證據。

## 第三次嘗試：原始材料查核與交接

**仍受阻；沒有解除兩項 P1，也沒有變更驗收。** 已讀原專案 `scratch/review-142.md`。這次只更新本報告，未找到足以修正 adapter 的新原始證據，因此保留既有來源與測試；沒有以臆測的 DLL 名稱改程式。下方「做了什麼」起保留前兩次工作的實作與驗證紀錄。

### 已驗證

- 唯讀核對重建 metadata 與 `tools/rebuild_mapped_pe.py`。以 `D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/` 為根：

  | 檔案 | SHA256 |
  |---|---|
  | `runtime/LiTo.unpacked-oep-current.exe.json` | `0EDEEE9F8BEC81D8A3DFD4BCA510BEAF5B6F7FEFDC5B45EAF1BD8BB815BC0D34` |
  | `tools/rebuild_mapped_pe.py` | `7A7B1FA73A7F9703A85240002A7E4589E8978A6CF1C6CF2E7BCD8CBBBD13FF45` |

  腳本 L79 讀 mapped image、L116 從 packed PE 複製 header、L128 從 mapped image 複製 section；L96 起套用明列的 recovery patch。metadata 記錄 `imports_reconstructed: false`、`tls_reconstructed: false` 與入口 40-byte patch。這只能說明保留檔的重建來源與限制，沒有證明 clock 全域在捕獲時尚未初始化，也沒有追完保護器／TLS。沒有執行此腳本或原 PE。
- 下列三個具體路徑以 `Test-Path -LiteralPath` 查核皆不存在：
  - `D:/lineage開發/LiTo381-開發/live-inspection-lito-oep-20260804-112513/LiTo.bin.live-image.bin`
  - `D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/live-inspection-lito-oep-20260804-112513/LiTo.bin.live-image.bin`
  - `D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/launch-reports/LiTo-20260805-214334.json`
- 本機檔名盤點涵蓋 `lito登入器/舊版分析與驗收`（7443 檔）、`lito脫殼資源`（5227 檔）、`lito-dev-runtime`（262751 檔）、上層 `scratch/t020c`（dump 目錄為空）與上層 `tools`（11 檔）。未找到上述同次捕獲。排除 reparse point、封存、backups、config、sealed-batch3、target／node_modules 等目錄；匹配的硬連結不讀取。最終已走完上述根目錄可列舉的佇列；仍有無法讀取與主動排除的目錄，且僅依檔名比對，不能宣稱全磁碟不存在。原始限制及結果見 worktree `scratch/audit-clock-materials.ps1`、`scratch/clock-materials.txt`。較早的內容掃描過慢，已停止自己啟動的掃描程序，未將未完成的內容掃描算成證據。
- 查到的 `supplier_envelope_import_20261007/original-clock-binding.json` 說明 Python supplier 的 `GetTickCount64()` 時鐘；`manual-acceptance-20261004-015859/readonly-module-mapping-diagnostic.json` 則記錄另一個 PID／建立時間／image 的 module mapping。兩者都不能連結到 `20260804-112513` 的三個跳板，本次沒有拿它們補原 import 身分。
- 在本 worktree `LiTo381-Modern/` 重跑下表，仍使用 `scratch/run142.ps1` 擷取現行 `run-local-test-suite.ps1` 的 `Measure-TestResult`；TEMP／TMP 在 worktree `scratch/tmp`，五步皆 exit 0／gate PASS。沒有新增測試或放寬門檻。

| 命令 | 結束碼 | `test result:` 原文／check 結果 |
|---|---:|---|
| `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` | 0 | check 通過，無 `test result:` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib` | 0 | `test result: ok. 1932 passed; 0 failed; 577 ignored; 0 measured; 0 filtered out; finished in 3.28s` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_clock::tests::original_pe_clock_startup_sites_and_snapshot_match -- --ignored --exact` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2508 filtered out; finished in 0.11s` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_clock::tests::controller_initializes_clock_before_any_queue_sample -- --exact` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2508 filtered out; finished in 0.00s` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_static2941::tests::original_bank_ready_gate_fifo_rollover_and_retirement -- --ignored --exact` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2508 filtered out; finished in 1.29s` |

原 PE／資源案沿用下文所列明確環境變數與唯讀來源。`scratch/check.log`、`lib.log`、`clock-pe.log`、`clock-startup.log`、`resource.log` 及對應 `*-result.txt` 已更新為第三次嘗試；下方前兩次原文屬歷史結果。577 ignored 中僅兩案另跑，其餘 575 案未跑。check 仍有既有 warnings。

### 推測

QPF／QPC 型態相似與現有 host binding 可運作，都不能推導原 export 身分；仍無依據排除後備是 `timeGetTime` 或 wrapper。重建腳本只解釋輸出如何形成，無法逆推出捕獲前的原始狀態。

### 待驗證與給下一棒的提示

1. **調度器／本機資源執行器先補件**：同次捕獲的主 image、涵蓋 `05010A68`／`05010863`／`0501095F` 的外部頁、module base/size、相符 DLL hash/export；每個跳轉需可逐跳複核。另需初始化前後狀態與捕獲階段、保護器／TLS 的來源證據。現有檔名盤點不取代補件。
2. 若材料無法取得，**任務負責人明確裁定驗收是否調整**後才有可執行的下一步；目前沒有這項裁定，不應再次只派重跑相同 host tests 的 code 重試。本機沒有權限自行把缺件視為通過。
3. 來源未改；Release、GUI、真機長 uptime、T020-2、七個整合 metadata 未重跑。未啟動／附加遊戲、未連正式服務或 DB、未讀寫機密設定或封存。不要標記 #142 完成或關閉 Issue。
4. `gh issue view 142` 仍回 HTTP 401，未取得新的雲端裁定，未寫 GitHub 留言／標籤。Git HTTPS 的預設 Schannel 在此環境回 `SEC_E_NO_CREDENTIALS`；單次 `-c http.sslBackend=openssl` 可讀遠端分支，沒有停用 TLS 憑證驗證或更改全域 Git 設定。

第三次嘗試交付路徑仍為 `docs/reports/issue-142.md`；`docs/REPORT.md`、TASKS 與原始專案工作檔未修改。

## 做了什麼

- `native_connection.rs`：完整封包解密後、event channel 背壓前取得 `IngressReceipt`，保留 connection epoch、rx counter 與 `Instant`。LOGIN 檢查事件和 receipt 身分相符；World 只為實際接受的 2941 S37 建立 receipt，無時間的 `accept()` 不補造時間，JSON 不輸出本機時鐘。
- `native_static2941.rs`：Controller 擁有專用 queue。收包僅建立 WAITING lease；背景載入器必須交回持有實際 `Arc<Static2941Bank>` 的 `Prepared`，通過世代、cue、目標 incarnation、root 及 token 檢查後才取 fresh clock。零 delay 先入列，同 deadline FIFO；poll 固定一次 cutoff，逐筆 pop 後退役。
- World ring 淘汰、清空、map／portal 換代、connection 換代、Controller cancel／fail／reset／drop 都撤銷共享 lease。World 快照只能觀察不可復活的 token，不能重建 timer。pending／due 受 128 筆 effect ring 限制，重複 proof 不讀 clock、不再次入列。
- `static2941_assets.rs` 是 `tower_assets` 的子模組，共用既有同 root archive resolver。核對 list digest、2941 無 action／type9、SPR 大小與 SHA、單 frame、原 fragments／depth／atlas。只有 `MissingEntry` 才讀 1120 後備；損壞、digest 不符、IO 或取消不能偽裝缺圖。不建立 Animation、FPS、WAV 或狀態期限。
- `native_world_scene.rs`／`native_world_view.rs`／`native_login_window.rs` 只加入必要的 worker proof 交付：接受目前 root／publication／world 的完成結果後交 Controller enqueue。過期環境畫面可以交付仍有效的資源 proof；錯誤 root 或退役 worker 不能交付。此處尚未加入 2941 像素提交、History 或重入繪圖。
- `native_clock.rs`：一次初始化的 Windows clock adapter；成功的 frequency query 使用 counter，失敗走明確的本機 `GetTickCount` 後備，拒絕成功但非正數的 frequency。保留原低 64 位乘 1000、signed divide、低 32 位回傳，以及 unsigned deadline 比較；不把 rollover 改成 epoch 延伸或 half-range 比較。Ingress／resource 的 `Instant` 只記錄事件先後，不用作 native deadline。
- 第二次嘗試只改 `native_clock.rs`、`native_login.rs` 及本報告：`LoginController::default()` 在建立其他狀態前執行 process-wide clock 初始化；`sample()` 不再初始化。`ProcessClock` 分開未初始化、counter、後備及初始化錯誤；錯誤鎖存至後續取樣回報。Controller 重建／重連及 counter 取樣失敗不重查 frequency、不重新選來源或重設 epoch。這是依已核對啟動次序修正本機 adapter，**尚不是原 import 等價裁定**。

## 已驗證

### 前次基線（`91e0b0a`，不當作第二次嘗試重跑結果）

在此 worktree 的 `LiTo381-Modern/` 執行，target 固定為 `target/current`，TEMP／TMP 指向 worktree 的 `scratch/tmp`。使用 `scratch/run142.ps1` 執行命令，直接擷取現行 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult`；前次三步皆為 **exit 0／gate PASS**。0 tests、failed > 0、缺少或無法解析結果仍會失敗，未放寬門檻。

| 命令 | 結束碼 | 原始輸出 |
|---|---:|---|
| `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` | 0 | check 無 `test result:`，最終全 target 檢查通過 |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib` | 0 | `test result: ok. 1930 passed; 0 failed; 576 ignored; 0 measured; 0 filtered out; finished in 3.62s` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_static2941::tests::original_bank_ready_gate_fifo_rollover_and_retirement -- --ignored --exact` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2505 filtered out; finished in 1.11s` |

第三步明確設 `LITO_STATIC2941_ROOT=D:/lineage開發/0-客戶端`，以唯讀 loader 實際核對 2941-0.spr（865 bytes、原 SHA、4 fragments）；沒有以假 bank 或 bool 通過 gate。完整 library 的 576 ignored 未計為通過，其中本張原資源案由第三步單獨驗證；其餘 575 個 ignored 未跑。

本張新增 **16 案**，15 個純邏輯／owned loopback 案由完整 library 執行，1 個原資源案另行執行：

- clock 初始化選路、失敗／非法 frequency、host import 呼叫、低 dword rollover、signed multiply 邊界。
- 32 封包／8 格 channel 的 loopback burst，逐筆核對實際 worker 解密後的 epoch、rx counter、body 及 receipt 時間順序；只綁定 `127.0.0.1:0` 的自建 listener。
- 無時間入口不補 receipt、有效 S37 篩選、WAITING 無 deadline、錯誤／重複身分、World 清空／map 換代、128 ring 淘汰、connection／generation／incarnation 撤銷、Queue drop、資源錯誤 latch。
- MissingEntry 後備政策的純測試：digest 錯誤、取消、IO 與 malformed index 不可走 fallback。1120 後備 SPR 的實際解碼仍留給原資源案例移植。
- 原 bank 整合案：取消、錯 root、clock 失敗、重複 proof、ready／enqueue 先後、同 deadline FIFO、`u32::MAX → 0` 排序與 due 比較、退役後不能重建 proof、取 clock 期間退役不能復活、World 清空後拒收 late proof。

前次來源在原資源案之後重新跑過 all-target check 與完整 library；未修改測試來源以掩蓋失敗。`scratch/check.log`、`scratch/lib.log`、`scratch/resource.log` 與對應 `*-result.txt` 為固定路徑，第二次嘗試重跑會更新它們，不新增時間戳 evidence JSON。

### 第二次嘗試靜態證據：同一 PE 的初始化鏈

唯讀來源為 `D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`，7,094,272 bytes，SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`。以下以此 PE 的原始 bytes 為準，VA 減 `0x400000` 即該重建檔的 raw offset；不混用 ASM 中不同的外部位址。

| VA | bytes | 已驗證的局部控制流 |
|---|---|---|
| `004F1759` | `E8420CF9FF` | 重建 entry `004F162A` 的正常路徑呼叫 `004823A0` |
| `00482537` | `E824E5FFFF` | `004823A0` 內呼叫 application startup `00480A60` |
| `00480A88` | `E8632F0500` | 呼叫 `004D39F0` |
| `00480A8D` | `E8DED90000` | 上一呼叫返回後呼叫 `0048E470` |
| `00480A92` | `E8C9580500` | 上一呼叫返回後呼叫 clock initializer `004D6360` |
| `004D636A` | `E8F9A6B304` | 傳入 stack 上 8-byte buffer，呼叫 `05010A68`，**外部身分未知** |
| `004D6376` | `A3F04B8300` | 把 EAX 寫入 `00834BF0` |
| `004D637B` | `890DE84B8300` | 把輸出低 dword 寫入 `00834BE8` |
| `004D6381` | `8915EC4B8300` | 把輸出高 dword 寫入 `00834BEC` |
| `004D69F6` | `A1F04B830085C07437` | 讀旗標、test；為零時跳到 `004D6A36` 後備路徑 |
| `004D6A04` | `E85A9EB304` | counter 路徑呼叫 `05010863`，**外部身分未知** |
| `004D6A37` | `E8239FB304` | 後備路徑呼叫 `0501095F`，**外部身分未知** |

PE metadata 的 entry RVA 是 `000F162A`，並明列該 entry 前 40 bytes 是 recovery patch。因此上述是**保留重建 PE 的靜態正常路徑**，不宣稱已追完 Themida/TLS 的原始入口、所有外部呼叫或真機執行順序。`00480A88`、`00480A8D`、`00480A92` 在此函式起始是順序指令，足以否定把 clock initializer 放到本機首次 queue sample 的設計。

`00834BE8..00834BF3` 在此 PE 是 12 個零；ASM L565691–L565695 則是 `00989680, 00000000, 00000001`。兩者只能各自證明**快照內容**，不能把 PE 的零稱為已驗證的初始化前值，也不能把 ASM 的非零值當成靜態預設。原 `4D69F0` 本身沒有 lazy init：只依旗標選擇分支。

新增 ignored test `original_pe_clock_startup_sites_and_snapshot_match` 先核對完整 SHA，再重查 entry、上述 call target／旗標寫入、快照值與 image 範圍。以 `LITO_NATIVE_CLOCK_PE` 指向唯讀檔即可重跑；測試不載入或執行 PE，亦不以通過結果代表外部 API 已解析。

### 第二次嘗試建置／測試

第二次嘗試在相同 worktree 重跑，沿用上述 `Measure-TestResult` 門檻，以下五步皆 **exit 0／gate PASS**。新增三案：初始化與錯誤鎖存、Controller 啟動接線、唯讀原 PE 靜態核對。Controller 接線另用 `--exact` 單獨啟動測試程序，避免完整 library 中其他測試先初始化全域 clock 而掩蓋漏接。

所有命令在 `LiTo381-Modern/` 執行：

| 命令 | 結束碼 | `test result:` 原文／check 結果 |
|---|---:|---|
| `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` | 0 | `Finished dev profile`；check 無 `test result:` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib` | 0 | `test result: ok. 1932 passed; 0 failed; 577 ignored; 0 measured; 0 filtered out; finished in 3.44s` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_clock::tests::controller_initializes_clock_before_any_queue_sample -- --exact` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2508 filtered out; finished in 0.00s` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_clock::tests::original_pe_clock_startup_sites_and_snapshot_match -- --ignored --exact` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2508 filtered out; finished in 0.23s` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_static2941::tests::original_bank_ready_gate_fifo_rollover_and_retirement -- --ignored --exact` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2508 filtered out; finished in 1.33s` |

原 PE 案明確設 `LITO_NATIVE_CLOCK_PE=D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`；原資源案設 `LITO_STATIC2941_ROOT=D:/lineage開發/0-客戶端`。library 的 577 ignored 不計通過，其中兩案由獨立步驟執行，其餘 **575 ignored 未跑**。未重跑 Release、七個整合 metadata 或 T020-2 的 12 gate 案。

原始命令與輸出：`scratch/check.log`、`scratch/lib.log`、`scratch/clock-startup.log`、`scratch/clock-pe.log`、`scratch/resource.log`；各自的 `*-result.txt` 保存退出碼及 gate。`git -c core.whitespace=cr-at-eol diff --check` 通過；第二次嘗試未改 `native_login_window.rs` 或既有換行格式。建置仍有 warnings，沒有當作零警告回報。

## 推測

原 `4D6360`／`4D69F0` 的 frequency／counter 型態與 QPF／QPC 相符，但**原 PE 匯入函式名稱尚未取得完整靜態資格**。本張實際連結的是明確指定的本機 `QueryPerformanceFrequency`／`QueryPerformanceCounter`／`GetTickCount`，不是把未證實的舊跳板名稱當事實。後備回傳 u32 不足以區分 `GetTickCount`、`timeGetTime` 或其他 wrapper；第二次嘗試沒有因為 host 測試通過就補上 module/export 名稱。

唯讀核對來源：

- `D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm`，SHA256 `2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A`。核對 `4D6360` 初始化、`4D69F0`、`4F11F0` low-64 multiply、`4F1230` signed divide，以及 `4D65F0` 的 `jbe`、`4D6570` 的 `jb`。
- 同 repo 的 `runtime/LiTo.unpacked-oep-current.exe`，SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`。只解碼靜態位元組，未執行 PE。三個實際 call 為 `4D636A → 05010A68`、`4D6A04 → 05010863`、`4D6A37 → 0501095F`；它們和 ASM 的 `05A10E20`／`05A10C4E`／`05A10D09` 不同，而且不在保留的 image 中。PE metadata 明記 `imports_reconstructed: false`。
- 既有 `維護工具/audit-native-cone-cold-scheduler.py` 也保留此 import 缺口。`lito脫殼資源/resolve_priority.py` 所引用的 `runtime/launch-reports/LiTo-20260805-214334.json` 在現有位置不存在；未以目前 DLL 位址猜測歷史 API，也未啟動或附加遊戲追蹤。
- `docs/HISTORY.md` L1–L13、候選 queue／ready／static asset 模板僅作設計與資源 pin 參考，未套用 overlay、未沿用舊編譯或測試資格。

## 待驗證／未驗證什麼

- **阻擋 #142 完成**：原 PE 外部 clock 跳板的實際 module/export 身分，以及原始初始化前狀態仍待補證。重建 PE 內的 initializer 呼叫鏈已新增證據，但原保護器／TLS 入口與外部跳板鏈尚未閉合，不能宣稱原時鐘全面等價。真機長 uptime oracle 也未做。
- PE 的 image 範圍為 `[00400000, 00AC4000)`，三個 `0501xxxx` 跳板均在範圍外。ASM 所屬 `segments.csv` 也只列主 image 的三個段，沒有 `05A1xxxx` 跳板段。第二次嘗試在 `lito登入器/舊版分析與驗收`、`lito脫殼資源`、`lito-dev-runtime` 的非 junction 目錄做檔名盤點，未找到 `112513` 原 live-image／外部頁或 `214334` launch report；沒有讀取封存／backups／機密設定或開啟遊戲程序。檔名盤點不是全磁碟不存在的證明。
- T020-2 的共享 callback、4D6570 固定 sample 與重入 producer 交錯、4E2150 suffix CFG、4857B0 巢狀 drain、direction 6 alias／shadow、2941 原 fragments 的畫面提交、FiniteHistory／cached／late-frame 防復活、12 個既有 gate 案移植，以及依第 2 張最終來源重編 library-test／七個整合 metadata，本張未完成也未計分。
- 目前正常 UI 已接到資源 proof／queue，但尚無 2941 的 draw consumer；不能以此推論原效果無像素，亦不能把現在下一個 poll 退役視為完整顯示時序。第 2 張需在合法 callback window 接原圖提交，不能靠固定顯示一幀改寫生命週期。
- 未做 Release、GUI、實服／DB、音效播放或 G06 驗收。未啟動、附加或停止任何遊戲程序；原客戶端資源唯讀，不複製到專案。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_clock.rs`
- `LiTo381-Modern/src/modern_lito/native_static2941.rs`
- `LiTo381-Modern/src/modern_lito/static2941_assets.rs`
- `LiTo381-Modern/src/modern_lito/native_static2941_tests.rs`
- `LiTo381-Modern/src/modern_lito/native_connection_ingress_tests.rs`
- 同目錄的 `mod.rs`、`native_connection.rs`、`native_login.rs`、`native_world.rs`、`tower_assets.rs`、`native_world_scene.rs`、`native_world_view.rs`、`native_login_window.rs` 必要接點。
- 本報告：`docs/reports/issue-142.md`。當次命令、原始輸出與 runner 放 `scratch/`，建置產物在 `LiTo381-Modern/target/current`，均不提交。

## 給下一棒的提示

**先由任務負責人／調度器處理原始證據缺件，不要把第二次嘗試標記 code 完成或以重跑相同 host tests 解除退回。** 請交本機資源執行器找回 metadata 指向的 `live-inspection-lito-oep-20260804-112513` 同次捕獲：主 image、涵蓋 `05010863`／`0501095F`／`05010A68` 的外部頁、module base/size 與相符 DLL export／hash；須逐跳對應到 module/export。另需能區分初始化前／後的原始狀態材料。僅拿不同世代的 module base 或目前 Windows DLL RVA 配對不合格。若材料無法取得，由任務負責人明確裁定是否調整驗收；第二次嘗試沒有代為放寬。

Claude 可先審查第二次嘗試啟動時機修正與新增的有限靜態證據；待缺件解決、本張核准後再交 T020-2。本機下一棒沿 `Receipt::lease()` → `Lease` → `Prepared` → `Queue` 使用同一 token；不要從 History／root reload 重建 lease 或用 paint 時間替換 native sample。`Stage` 目前只快取資源及同 root 錯誤；History 跨 Model 重建的錯誤 latch／source ABA 守門須由第 2 張接上。

本張原資源測試是新增的獨立核心整合案，不是移植 HISTORY 所列 12 案；下一棒仍需依原清單逐案移植。`scratch/clock-init.txt`、`scratch/clock-upstream.txt`、`scratch/clock-inventory.txt` 保存第二次嘗試查核輸出；可提交、可重跑的有限證據核對在 `native_clock.rs` 的 ignored test，不依賴 scratch 腳本才可複核。
