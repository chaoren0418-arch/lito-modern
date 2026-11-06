# Issue #161 — T024 第 1／2 張：4183 時序與 S0B 重送

本張只處理技能 #87 的持續物件 4183；不包含 T024 第二張的 LT hurt／base、速度倍率及 uint32 rollover。工作分支為 `task/161`，未套用隔離候選、未修改 `docs/REPORT.md` 或任務總表。

## 做了什麼

- 在 `native_connection` 解密出完整封包時記錄 `received_ms`；GAME 控制器另記錄實際 `dispatched_ms`。4183 的 584 ms 起始等待以派送時刻為準，收包佇列等待可由兩值辨識。兩者共用程序內的單調時鐘，不使用 worker 開始載入、首次畫面或 paint 時間。
- 新增 `native_status4183`。控制器在一批收包處理後，以固定 sample 派送到期 callback；Model 只讀快照。保留 source frame `1→2→3→4→5→0`、一般 168 ms、末幀 166 ms、晚到 remaining 最低 1 ms 再加 2 ms。沒有 elapsed 取模追幀，也沒有本機 2–4 秒刪除或修改 HP／MP／S25。
- 重送同 ID 時，保留 4183 專用排程，重設動作及影格。idle 已排隊時新增 1 ms Queue callback；動作中重送則另排 584 ms，既有 Frame callback 仍可先派送。空 Queue callback 不會再次啟動動作。相同 deadline 按全域插入序號派送，不按角色 ID 排序。一般 Actor incarnation 的原有規則未變。
- 專用場景路徑嚴格驗證 4183 的原六幀 action3、type0、Screen attr8、source1 overlap1453；僅接納遠端、空名稱／標題、靜止、一般速度、無動作佇列／狀態旗標的物件。一般 `Animator` 的 `breath`／`idle` 標籤守衛保持原樣。
- 原 SPR 透過既有 SharedEffectBank 提交 Screen fragments，overlap1453 使用既有 typed audio。每次 callback 使用獨立序號，快照共享一次性音效 payload。S0B 重送、移除與換世界會退役舊租約；cached frame、晚到 worker、deferred audio 及最終 submit 都有守衛。來源退役後的相同物件不能由 A→B→A 重新接納。
- 每物件最多 64 個待派送 callback、每次 poll 最多 4096 次派送、音效待處理 ring 最多 32 筆；超限明示錯誤，不無限累積。

## 靜態來源核對

依主線已追蹤的 `維護工具/native-E149-status4183.md`、原機唯讀證據及反組譯追查，沒有複製候選程式到主線。

| 位置 | 核對事項 |
|---|---|
| `4E7B54–4E7BC5` | S0B 查找既有 ID；存在時重用 actor |
| `4E7C1D／4E7C24／4E7C3F` | 兩條路徑均重設目前 action、stance、frame |
| `4E8034→4E02C0` | S0B 呼叫 idle scheduler，參數 1 |
| `4E02C8／4E0408` | queue 非空時不再加入 action3，只用傳入 delay 排 Queue callback |
| `4E0383–4E03BE` | queue 空、目前 action 非3時用 584 ms；已是3時取幀時間 166 ms |
| `4E85F0／4E9300` | 先 pop action，再進入動作；空 queue 直接返回 |
| `4E4913–4E496A` | frame0 先增加為 source1，再排 duration+2；保存 ideal deadline |
| `4E3CB6–4E3D93` | 終幀回 idle；其他幀保留 ideal deadline，remaining 最低 1+2 ms |
| `4E5CA7→4E62A1／4E63AA` | 重送後舊 Frame callback 遇到無動畫的 action0，走 static 分支、回傳 0 |
| `4D6570／4D65F0` | 固定 sample drain、callback 前移除項目、同 deadline 穩定插入 |

本機證據根目錄：`D:\lineage開發\3-lito開發\lito重建版\驗證紀錄\current`。

- `native-E149-status4183-inspection/20261024-174320-375652/4E7A40.bin`：SHA256 `46F7A4CC5F7681AFA0CCC374A8217CE15E445C6723F669C39E7197A3941BC5B3`。
- 同目錄 `4E02C0.bin`：SHA256 `34018ECD6279CC009A1162FD138AE76A04CBA5A62B55DFCEC842ED06A42E944E`。
- `native-E149-freezing80-shock87-inspection/20261024-172131-694095/4183.definition.txt`：SHA256 `7CC97CDE0A4184D737B077DE82A1AB29F66C45F7D028DF40049E8D2D60A421F2`。
- 移除封包使用當版契約的 `S_REMOVE_OBJECT 0x15`；歷史文字「S08」是舊命名，不能把十六進位 opcode 誤寫為 `0x08`。

## 已驗證

基底為 `6629b96`。以下皆為本 worktree 最終版本的實跑結果；每項結束碼為 0，門檻為 PASS。所有 Cargo 命令在 `LiTo381-Modern` 執行。

| 本機紀錄（scratch/） | 命令 | 結束碼 |
|---|---|---|
| `final-check.log` | `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` | 0 |
| `status-tests.log` | `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib native_status4183::tests` | 0 |
| `status-resources.log` | `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_status4183::tests::original_bank_screen_fragments_audio_and_refresh_lease -- --exact --ignored` | 0 |
| `movement-tests.log` | `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_world_scene::movement_tests` | 0 |
| `animation-tests.log` | `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_animation` | 0 |
| `network-tests.log` | `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_network` | 0 |

`test result:` 原文：

`status-tests`：

```text
test result: ok. 15 passed; 0 failed; 1 ignored; 0 measured; 2502 filtered out; finished in 0.00s
```

`status-resources`：

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2517 filtered out; finished in 1.03s
```

`movement-tests`：

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 2508 filtered out; finished in 0.06s
```

`animation-tests`：

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`network-tests`：

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.08s
```

原資源案例明確設定 `LITO_SHARED_RESOURCE_ROOT=D:\lineage開發\0-客戶端`，只讀取／解碼原 SPR 和 WAV、驗證 Screen fragments 與 overlap1453 payload，並以禁止播放的測試 Player 驗證退役音效無法進入播放。一般執行中 ignored 的該案，只計入後續 exact 執行的 1 passed。

開發中曾有 Frame 測試建構欄位及區域旗標作用域的編譯失敗（結束碼 101），修正後重新編譯。第一次純測試為 `12 passed; 2 failed; 1 ignored`（結束碼 101）：兩案均因單筆測試 catalog 誤宣告 11297 筆而無法解析；改成實際 1 筆後重跑，未移除斷言。以上失敗均不計為通過。

另以 `git -c core.whitespace=cr-at-eol diff --check` 檢查差異，結束碼 0；保留既有檔案的 CRLF。

測試門檻直接取用 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult`，檢查結束碼、唯一完整 `running`／`test result:`、計數一致、非 0 tests、failed=0；ignored 不算通過。原資源案例另外以完整名稱 `--exact --ignored` 執行。

## 推測

- action3 啟動前的靜態 SPR subframe0 仍是原 static lookup 的來源推論，收據保留 `initial_static_inferred`，不是原版像素 oracle 的已驗結果。
- 主線採單調 u64 毫秒時鐘及「收包批次後 drain」策略。靜態指令支持上述 canonical 4183 佇列分支，但不能由單元測試推論原 PE 的所有全域 callback／重入繪圖交錯等價。

## 待驗證／未驗證什麼

- 原 PE 時鐘 import／初始化、原資源同步載入耗時、全域 scheduler／重入繪圖、原像素與實際播放仍待原版對照。現在記錄的是現代端 decode／dispatch 時刻，不能冒充原版 constructor 的執行資格。
- 沒有啟動遊戲、操作既有遊戲程序、連正式服務或 DB；沒有自然眩暈期限、完整技能 #87 或 GUI 驗收的通過聲明。
- 本張未驗非 canonical 4183（移動、加速、動作佇列、角色變身等）；它們不會放寬一般 idle 守衛。
- 僅執行上述指定套件／filter；全量 library、Release 未執行。T024 第二張及整個 M2 尚未因本張而完成。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_status4183.rs`：控制器排程、資源驗證、專用呈現。
- `LiTo381-Modern/src/modern_lito/native_status4183_tests.rs`：純封包／時序回歸及唯讀原資源案例。
- 接線：`mod.rs`、`native_connection.rs`、`native_login.rs`、`native_world.rs`、`native_world_scene.rs`、`native_world_view.rs`、`native_audio_delivery.rs`；`native_movement_tests.rs` 僅補 Frame 建構欄位。
- 本報告：`docs/reports/issue-161.md`。
- 本機未追蹤紀錄：`scratch/verify.ps1`、`scratch/*.log`、`scratch/*.json`；建置只在 `LiTo381-Modern/target/current`。

## 給下一棒的提示

Claude 審查時請優先核對重送後「保留 queue／callback」與「退役舊呈現租約」的分離，尤其是 idle pending、動畫中重送、恰逢 deadline 三個測試。不要以重建 Model 重設 584 ms，也不要改用 Clip 取模。

後續本機驗證者可重跑本報告列出的命令；完整畫面／自然眩暈驗收由 M2 的後續驗收工作接手。T024 第二張執行者只接 LT hurt／base 等既定範圍，勿把本張的 u64 時鐘當作原 uint32 rollover 已解決。
