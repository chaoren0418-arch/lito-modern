# Issue #143：T020-2 暗隱術 callback、繪圖守衛與 ready gates

基底 `e83403140e814e45fc7c53ed0cb8929424f5a079`，分支 `task/143`，工作目錄 `scratch/wt/task-143`。沿用已合併 #142 的 ingress／clock／bank／queue；沒有套用隔離候選 Runtime，也沒有修改 `docs/REPORT.md`、TASKS 或原專案工作檔。

**本張尚未完全閉合。** 已補當版資源繪圖、生命週期守衛、共享 callback queue 的重入機制及 12 個 gate；原 `4E2150` producer 的正式接線仍缺。目前正式 `native_world_animation` 使用 elapsed/FPS，沒有原控制旗標的 authority adapter；新增共享 producer 與巢狀 pump API 目前由測試呼叫，不能當成已完成原 active-world 全域排程。保留此界線，不把機制測試當成 producer 實機資格。

## 做了什麼

- `native_static2941.rs`：同一 queue 可容納資源退役與共享 callback；依 unsigned deadline 排序，同期限 FIFO；先移除再 callback。callback 可重新 enqueue 或巢狀 drain，外層恢復後仍使用原 cutoff。callback 失敗不重播；owner 變更清除舊 callback。退役 revision 每個 callback 個別更新，避免巢狀 drain 後寫回過期 revision。
- `drain_4857b0` 實作 timer／draw 子流程：至少執行一次，drain 各自取樣、draw 後再取相對時間，以 wrapping dword／signed 累加滿 500 ms 結束。沒有把收到 2941 當成進入 169 特殊分支的理由；原取消 482F50、camera globals 與 169 分支接線不在此 helper 中。
- `static2941_assets.rs`、`native_world_scene.rs`：先備妥真正 bank，再將 frame0／原 fragments 提交既有 Over／深度合成路徑。worker 可先合成尚待 enqueue 的像素，但持有的 lease 未 queued 時不可發佈；不靠等待下一張 worker 畫面才開始繪製，也不延長 callback 期限。
- `Frame`、`InputSurface`、`native_world_view.rs`、`native_login_window.rs`：快取、晚到結果、paint 與輸入都保留同一退休 token。退休後 copied World 也無法讓舊畫面或輸入重新取得資格；清除 stale raster 時保留已接收的 audio／hurt callback 交付。
- `FiniteHistory` 保存同源資源錯誤與共享 lease；換來源可重新載入，但 merge／restore 不建立新生命週期。
- `native_static2941_ready_tests.rs` 以當版 API 重寫歷史 12 案（8 純、4 原資源 opt-in），精確名稱維持 #57 清單。另加 callback 重入、巢狀 pump、實際 SPR 像素／fragments、Frame／InputSurface／cache 撤銷與原 PE 靜態來源測試。

## 已驗證：原來源與適用界線

唯讀來源：

- `D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`，SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`。
- 同根 `ida-client-oep-v2/full-disassembly.asm`，SHA256 `2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A`。
- 原資源 `D:/lineage開發/0-客戶端`；bank 仍沿用 #142 對 list／SPR size、SHA、frame、metadata、fragments 的檢查。沒有複製資源進 git。

本次從 PE 解碼完整 `004E2150..004E34B8`（含 epilogue／ret），1281 個 instruction 節點；建立直接 jump／fallthrough CFG。拓樸可達不等於有符合所有 globals／外部 call 副作用的動態執行；沒有把最短 CFG 路徑當成可執行 trace。一次性查核在 `scratch/cfg143.py`、`scratch/cfg143.txt`、`scratch/callback143-disassembly.txt`。追蹤的 Rust opt-in 測試固定完整函數 bytes、上游 branch bytes 與實際 call target。

| 節點 | 已核對的上游／分支 |
|---|---|
| 4E2150 入口 | action table entry 非零、frame count ≥3、graphics 非負且小於上界，才繼續；入口清 `834EB3`。早退不能套用 suffix 條件。 |
| 4E2D07 | 中途移動的 camera 分支直接匯入，或起步／路徑分支經 4E2CB1、逐幀 enqueue、4E2CF6 匯入；經移除舊 body／attachments，再到 4E2E7C／4E2E93。 |
| 4E2E93 | 呼叫 4E5B50 推進 frame；非零走 4E3096，不經 EB3=1。原 4E5B50 在末幀將 actor `+13h` 歸零並回傳零。 |
| 4E307E | 只在 frame 完成路徑、actor `+74h <= 0`，且 `760098 != 0` 或 `64BBC3 != 0` 時進入。4E3088 是本函數唯一直接 EB3=1 store，接著 4E308F enqueue 4E2150。其他有 queued action 的分支 enqueue 4E9300／4E9530。 |
| 4E3112 → 4E3207 | 匯入 body 4E0200 與 optional attachments；4E31EA／4E3203／4E3205 匯入 suffix。不能跳過這些上游直接宣稱兩 suffix 都會畫。 |
| 4E33AD | suffix 的 actor frame `+13h=0`、`834F1D=0`、`834EB2=0`、`834EB3!=0`、`641498=0` 才呼叫 488860。取消分支清 EB3，pending action 分支繞過此繪圖。 |
| 4E3432 | 到 suffix 時 actor frame `+13h!=0` 且 `641498=0`，呼叫另一個 488860；不要求 EB3。 |
| 4D6570 | cutoff 存於 stack local，只供此 invocation；每次移除 queue 首項、遞減 count，再 `call esi`，回來重新讀 queue。尾端另呼叫 4C89F0；此音效工作不是更新原 cutoff。 |
| 4857B0 | 原 S37 handler 在 `63DF4C!=0`、graphics=169、target=own actor 時呼叫。每輪 4D6570 → 488860 → 4D69F0，累加相對 delta，以 signed `<500` 回圈。它可以清掉同一共享 queue 裡其他事件，2941 自己不直接走此入口。 |
| direction 6／shadow | 43DE10 普通初始化 flags table 647E68=0、alias 647E54=-1、shadow 647E70=-1；2941 的固定 list row 只有 attr102/type9，沒有 attr101/104/112 覆寫。4DFDD0 取 flags；4E62A1 只在 alias 非負且方向6才替換，4E6355 只在 shadow 非負才載 shadow。故本 bank 使用普通 Over、單 primary frame、無 alias／shadow。 |

這是固定 PE／list 的普通初始化與控制流程證據；不是任意後續 global writes、原像素 oracle 或真機可見幀數的證明。

## 已驗證：建置與測試

以下皆在本 worktree `LiTo381-Modern/` 執行。`CARGO_HOME=D:/dev-cache/cargo`、`RUSTUP_HOME=D:/dev-cache/rustup`；TEMP／TMP 指向本 worktree `scratch/tmp`。`scratch/run143.ps1` 以 AST 取得現行 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult` 原函數；沒有放寬 0 tests、failed 或解析失敗的門檻。最終三項均 gate PASS。

| 命令 | 結束碼 | `test result:` 原文／結果 |
|---|---:|---|
| `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` | 0 | check 通過；無 `test result:` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib` | 0 | `test result: ok. 1962 passed; 0 failed; 583 ignored; 0 measured; 0 filtered out; finished in 4.96s` |
| `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_static2941::tests:: -- --include-ignored --test-threads=1` | 0 | `test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 2517 filtered out; finished in 7.14s` |

28 案包括原 #142 十案、本張 12-gate、五案 callback／draw 與一案原來源。原資源命令明確設定 `LITO_STATIC2941_ROOT`；PE／ASM 命令設定 `LITO_NATIVE_CLOCK_PE`、`LITO_NATIVE_CALLBACK_ASM` 為上節來源。12-gate 的 8 純／4 原資源案都實跑通過，沒有以 metadata 或 ignored 代替執行。library 的 583 ignored 中，本張另跑七案，其餘 576 案未跑。Release 只做下節 metadata，未執行 Release 測試。

最終日誌：`scratch/check-final.log`、`scratch/lib-final.log`、`scratch/static2941.log` 與對應 `*-result.txt`。`git diff --check` 通過。編譯仍有 warnings；共享 producer／pump 未有正式呼叫者的 dead-code 警告與本報告的未完成界線一致。

## 七個整合 metadata

依歷史 `native-E149-static2941-queue-metadata/20261024-203626-169258/cohort.json` 的 `paired_tests`，核對原 `input_pins` 與單一 hardlink count 後，只將七個 QA 本文放入本 worktree `scratch/metadata/`。重綁 fixture include 到本 worktree，另做下述一項方法名適配；保留所有預期值與分支，沒有套用候選 Runtime。

| QA | 原文 SHA256 |
|---|---|
| blind20 | `03DEB74A4883C28A96058AEC59ABE0E2EBE0BE9514B7364F3FE5210E7A77C5B4` |
| slow29 | `CFA9685F0B3EBB1098C83CA140D17955B42AE59E3AE90EDD8F3CE8006EDE3B44` |
| advance79 | `96DB321632AA8868E4F5230C1E22E143090B6E1F777F488F4D4E4F9E4FBFA935` |
| item_spells | `FFFA7E2FD2476715C558D146A745228D1924A4D23D30686B958FDC15D5D06556` |
| magical73 | `5F90100B56394690393168F2F4B41720E928642B0F3F9EFFDC7C839CDC6DE1DE` |
| shadow107 | `F73F627CAB38B179228F0B61F13593E3E8A4ABF805DBCF8E4CD89BB426D22919` |
| object374448 | `F170F1B1B857297EB579283C7AF897A73019CD65AA33C5AB6A7C65B4787587EE` |

Debug／Release 各執行 `cargo check --lib --tests --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --message-format=json`（Release 加 `--release`），兩者 exit 0。已從 Cargo `compiler-artifact` 核對當版 library 與 library-test 皆有對應產物，`build-finished.success=true`。

七個 QA 以同一 rustc、`--test --emit=metadata --edition=2021 --target x86_64-pc-windows-msvc`、當版 library／直接相依的 `--extern`，以及 target／host 兩個 `-L dependency` 重編；Release 加 `-C opt-level=z -C debug-assertions=off`。每次完整命令保存在 `scratch/metadata/<profile>-<name>-command.txt`，輸出只到 `target/current/issue143/metadata/<profile>/`。

| QA | Debug 最終結束碼 | Release 最終結束碼 |
|---|---:|---:|
| blind20 | 0 | 0 |
| slow29 | 0 | 0 |
| advance79 | 0 | 0 |
| item_spells | 0 | 0 |
| magical73 | 0 | 0 |
| shadow107 | 0 | 0 |
| object374448（方法名適配） | 0 | 0 |

保留的失敗與修正：

- 初版一次性 metadata driver 漏了 host proc-macro 搜尋目錄，兩版七組皆 exit 1／E0463。補上 `target/current/<profile>/deps` 的 `-L dependency`，不改 Runtime 或 QA 邏輯；初始輸出仍在 `scratch/metadata/<profile>-<name>.log`，沒有把 cascading diagnostics 算成產品失敗或通過。
- 補搜尋目錄後，原文 `object374448` 在 Debug exit 1／E0599，五處 `Pending::observe_equipment` 不存在；其餘六組 exit 0。查核 `維護工具/prepare-native-E149-object374448.py:47`，舊候選只是把既有 `observe_item` 方法本體移名並加 wrapper。當版 `native_skills.rs:248` 仍保留這個方法本體。因此僅在暫存 QA 將五處呼叫改回 `observe_item`，預期的 79／161、timeout、cast 與 generation 斷言不變，重編 exit 0。這不授予 #37／44／48 的正式裝備回饋接線資格，也沒有為了 metadata 新增假 Runtime API。

最終紀錄在 `scratch/metadata/retry-debug-results.txt`（六過、一原文失敗）、`adapted-debug-results.txt`（該案適配後通過）、`retry-release-results.txt`（七過）及相應 log。兩版 library／library-test 與 14 個整合 metadata 均已重編；七個整合測試本文未執行，metadata 沒有 `test result:`，不列為測試通過。

## 推測

- 資源預先合成、controller enqueue 後的 paint window，可讓 2941 在 callback 前呈現；Windows message 排序可能讓該 window 沒有實際 paint。沒有保證顯示一幀，也沒有用「永遠不可見」作完成條件。
- 已釘住普通初始化的無 alias／shadow 可支持現代 Over 路徑；獨立原生像素比對仍未進行。

## 待驗證／未完成與給下一棒的提示

1. **Claude／調度器：本張不要標成完整完成。** `Queue::enqueue_callback`、`drain_4857b0` 已有可執行的交錯測試，但正式 controller 仍只送入 static retirement；原 4E2150 actor producer 與 169 nested-pump branch 沒有接入。請接續本分支，以本報告的 upstream CFG 建立明確的 movement/control-state adapter，再接正式 producer；不要直接拿 `World` 的 heading 或 paint elapsed 代替原 frame／held／pending flags。
2. 本機下一棒在接線後，重跑本張測試並加入 controller → 真正 producer → 畫面提交的整合案例；目前 actual-bank callback 測試驗證的是 queue 與 renderer 守衛，Frame／InputSurface 的測試 shell 是合成 fixture，沒有冒充原 GUI。
3. 原生 GUI、正式服務／DB、原遊戲程序、獨立原像素 oracle、G06 640 次均未執行。沒有啟動／附加／停止遊戲，也沒有修改機密設定、封存或硬連結。
4. 不重做 #142 的 clock 初始化／rollover，也不恢復已被主控降為推測的外部 clock export 身分要求；本張沒有宣告那些未知項已驗證。

## 產出路徑

追蹤的產出為本報告、`LiTo381-Modern/src/modern_lito/` 下上述 Runtime／QA 與既有 movement fixture 的新增空 guard 欄位。一次性腳本及日誌在本 worktree `scratch/`；Cargo 與 metadata 產物只在 `LiTo381-Modern/target/current/`，不入 git。
