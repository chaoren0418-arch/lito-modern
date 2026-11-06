# Issue #142：T020-1 暗隱術 ingress、資源 gate 與專用 queue

本張以 `task/142`、基底 `dd4fabb` 重寫 T020 第 1／2 張的來源；沒有套用 44 檔隔離候選，也沒有改 `TASKS.md` 或 `docs/REPORT.md`。這是 #97 的基礎接線，不代表暗隱術完整呈現或整項 T020 已驗收。

## 做了什麼

- `native_connection.rs`：完整封包解密後、event channel 背壓前取得 `IngressReceipt`，保留 connection epoch、rx counter 與 `Instant`。LOGIN 檢查事件和 receipt 身分相符；World 只為實際接受的 2941 S37 建立 receipt，無時間的 `accept()` 不補造時間，JSON 不輸出本機時鐘。
- `native_static2941.rs`：Controller 擁有專用 queue。收包僅建立 WAITING lease；背景載入器必須交回持有實際 `Arc<Static2941Bank>` 的 `Prepared`，通過世代、cue、目標 incarnation、root 及 token 檢查後才取 fresh clock。零 delay 先入列，同 deadline FIFO；poll 固定一次 cutoff，逐筆 pop 後退役。
- World ring 淘汰、清空、map／portal 換代、connection 換代、Controller cancel／fail／reset／drop 都撤銷共享 lease。World 快照只能觀察不可復活的 token，不能重建 timer。pending／due 受 128 筆 effect ring 限制，重複 proof 不讀 clock、不再次入列。
- `static2941_assets.rs` 是 `tower_assets` 的子模組，共用既有同 root archive resolver。核對 list digest、2941 無 action／type9、SPR 大小與 SHA、單 frame、原 fragments／depth／atlas。只有 `MissingEntry` 才讀 1120 後備；損壞、digest 不符、IO 或取消不能偽裝缺圖。不建立 Animation、FPS、WAV 或狀態期限。
- `native_world_scene.rs`／`native_world_view.rs`／`native_login_window.rs` 只加入必要的 worker proof 交付：接受目前 root／publication／world 的完成結果後交 Controller enqueue。過期環境畫面可以交付仍有效的資源 proof；錯誤 root 或退役 worker 不能交付。此處尚未加入 2941 像素提交、History 或重入繪圖。
- `native_clock.rs`：一次初始化的 Windows clock adapter；成功的 frequency query 使用 counter，失敗走明確的本機 `GetTickCount` 後備，拒絕成功但非正數的 frequency。保留原低 64 位乘 1000、signed divide、低 32 位回傳，以及 unsigned deadline 比較；不把 rollover 改成 epoch 延伸或 half-range 比較。Ingress／resource 的 `Instant` 只記錄事件先後，不用作 native deadline。

## 已驗證

在此 worktree 的 `LiTo381-Modern/` 執行，target 固定為 `target/current`，TEMP／TMP 指向 worktree 的 `scratch/tmp`。使用 `scratch/run142.ps1` 執行命令，直接擷取現行 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult`；三步皆為 **exit 0／gate PASS**。0 tests、failed > 0、缺少或無法解析結果仍會失敗，未放寬門檻。

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

最終來源在原資源案之後重新跑過 all-target check 與完整 library；未修改測試來源以掩蓋失敗。原始輸出為 `scratch/check.log`、`scratch/lib.log`、`scratch/resource.log`；對應 `*-result.txt` 保留退出碼與 gate 結果。`git -c core.whitespace=cr-at-eol diff --check` 通過；保留既有 `native_login_window.rs` 的 CRLF，沒有整檔換行重寫。

## 推測

原 `4D6360`／`4D69F0` 的 frequency／counter 型態與 QPF／QPC 相符，但**原 PE 匯入函式名稱尚未取得完整靜態資格**。本張實際連結的是明確指定的本機 `QueryPerformanceFrequency`／`QueryPerformanceCounter`／`GetTickCount`，不是把未證實的舊跳板名稱當事實。

唯讀核對來源：

- `D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/ida-client-oep-v2/full-disassembly.asm`，SHA256 `2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A`。核對 `4D6360` 初始化、`4D69F0`、`4F11F0` low-64 multiply、`4F1230` signed divide，以及 `4D65F0` 的 `jbe`、`4D6570` 的 `jb`。
- 同 repo 的 `runtime/LiTo.unpacked-oep-current.exe`，SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`。只解碼靜態位元組，未執行 PE。三個實際 call 為 `4D636A → 05010A68`、`4D6A04 → 05010863`、`4D6A37 → 0501095F`；它們和 ASM 的 `05A10E20`／`05A10C4E`／`05A10D09` 不同，而且不在保留的 image 中。PE metadata 明記 `imports_reconstructed: false`。
- 既有 `維護工具/audit-native-cone-cold-scheduler.py` 也保留此 import 缺口。`lito脫殼資源/resolve_priority.py` 所引用的 `runtime/launch-reports/LiTo-20260805-214334.json` 在現有位置不存在；未以目前 DLL 位址猜測歷史 API，也未啟動或附加遊戲追蹤。
- `docs/HISTORY.md` L1–L13、候選 queue／ready／static asset 模板僅作設計與資源 pin 參考，未套用 overlay、未沿用舊編譯或測試資格。

## 待驗證／未驗證什麼

- 原 PE 外部 clock 跳板的實際 API 身分、原始初始化呼叫鏈與真機長 uptime oracle 仍待補證；本張可證明的是本機 adapter 與已核對算術，不能宣稱原時鐘全面等價。
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

Claude 審查本張範圍與 clock 證據界線；調度器在合併後交 T020-2 本機執行器接續。請沿 `Receipt::lease()` → `Lease` → `Prepared` → `Queue` 使用同一 token；不要從 History／root reload 重建 lease 或用 paint 時間替換 native sample。`Stage` 目前只快取資源及同 root 錯誤；History 跨 Model 重建的錯誤 latch／source ABA 守門須由第 2 張接上。

本張原資源測試是新增的獨立核心整合案，不是移植 HISTORY 所列 12 案；下一棒仍需依原清單逐案移植。原 import 身分缺口應保留為明確待驗證項，若要提升為原 PE 等價資格，需補齊同一 dump 世代的外部跳板與 module／export 證據。
