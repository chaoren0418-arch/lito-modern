# Issue #143：T020-2 暗隱術（第二次嘗試）

**結論：部分修正，未完成；前次 review:fail 的正式接線問題仍在。請勿標記 T020／#143 完成。**

本次基底 `98981f4a2510bf83c8b74b3c32884b68c0de0bd5`，沿用 `task/143`，未切換或另建分支。本次修正共享 queue 在重入退役／重綁後繼續執行舊 drain 的錯誤，補原始控制旗標的上游核對，並重跑本張測試及 metadata。**沒有把只有測試呼叫的 helper 包一層便宣稱正式 producer 已接入。**

## 做了什麼

- `native_static2941.rs`：新增獨立於數值 `Owner` 的 `Arc<()>` queue 身分。每次 retire 都換身分；外層 drain 在 callback 返回後，及 World 同步後，檢查原身分是否仍有效。重入 callback 即使退役後綁回相同 connection／generation，新事件也留給新的 drain／sample，不能被舊 stack 的 cutoff 消費。
- `drain_4857b0`：無有效 owner 時不讀 clock／不 draw；每輪 drain 後與 draw 後檢查 queue 身分及 World 是否退役。取消時離開 helper，不繼續畫新場景或讀取舊相對時鐘。此取消機制是現代 queue 的生命週期保護，不是原 `4857B0` 完整實作，也沒有補寫原 `75AB1C` 完成旗標。
- `native_static2941_callback_tests.rs`：新增三個回歸案例，分別覆蓋相同 Owner 的退役／重綁、drain 內退役、draw 內清空 World。修正前逐一重現，修正後全部通過。正常同一 queue 的巢狀 drain、固定外層 cutoff、fresh enqueue sample、同期限 FIFO 與 rollover 案仍通過。
- `native_static2941_source_tests.rs`：在既有完整 PE／ASM SHA gate 下，新增 76 個有序指令核對，涵蓋 `4ADDA0 → 4857B0` 三層條件、取消 callback／camera globals、兩個 suffix 的分岔及取消路徑、`641498` 真正 producer、玩家 `4E8750` 額外分支。這是唯讀靜態證據，不當成真機 trace 或正式接線測試。

前次已加入的 SPR fragments 繪圖、共享 lease、Frame／InputSurface／cache 退役守衛、History 與 12 ready gates 均保留。本次沒有套用候選 Runtime，沒有修改 `docs/REPORT.md`、TASKS、其他分張功能或原始專案工作檔。

## 已驗證：正式呼叫路徑仍有缺口

靜態檢查本次 `src/modern_lito/`：`.enqueue_callback(` 與 `.drain_4857b0(` 的呼叫者仍全部在 `native_static2941_callback_tests.rs`。正式路徑仍為：

1. `LoginController::accept_game_received` 接受 World 封包與 static receipt。
2. `Presentation::tick_environment`／worker 交付 `Prepared`，UI 呼叫 `enqueue_static2941`。
3. `LoginController::poll` 直接取一次 native clock 後呼叫 `drain_due`；正式 queue 目前只有 static retirement producer。
4. `Model::draw_at_labels_environment` 仍呼叫 `Animator::pose`；該 pose 由 elapsed／FPS 與 speed factor 推導，沒有 `4E2150` 的 source frame／control-state authority。

這是**尚未完成的本張實作**，不是 GitHub 登入、測試授權、缺 clock export 或使用者批准造成的阻擋。本次沒有要求重新批准測試，也沒有把 #142 的外部 clock 身分要求恢復成門檻。

## 已驗證：來源與不可直接替換的狀態

唯讀來源與前次相同：

- `D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`，SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`。
- 同分析根目錄 `ida-client-oep-v2/full-disassembly.asm`，SHA256 `2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A`。
- SPR／list 根目錄 `D:/lineage開發/0-客戶端`，由既有 `Static2941Bank` loader 核對 digest、單 frame、metadata 與四個原 fragments；資源沒有複製進 git。

| 原控制流 | 本次核對與正式接線缺口 |
|---|---|
| `4E2CD1` | 起步依 `4E8750` 逐幀累加 delay，各次 `4D65F0` 取 fresh sample。不是在 worker 回傳一個 elapsed pose 後補排同名 callback。 |
| `4E2E93 → 4E5B50` | 推進 actor `+13h`，末幀回零並進入 `4E5C00`；非零回傳跳過 EB3=1。正式 Animator 沒有同一 source frame 的 owner／timer。 |
| `4E307E → 4E308F` | frame 完成、actor `+74h <= 0`，且 `760098 != 0` 或 `64BBC3 != 0`，才設 EB3=1 並排後續 `4E2150`。目前 `HeldInput::direction`／`Actor.heading` 不能單獨證明這些條件。 |
| `4E3207` suffix | frame 非零走 `4E3429`；frame 零先檢查 `834F1D` 取消，再檢查 `834EB2` pending action，最後才是 `834EB3`。`4E33AD` 與 `4E3432` 都受 `641498` 控制。 |
| `488860` normal tail／`4E8E70` | fresh native draw sample 減前一 sample，先保存 raw dword delta；actor graphics `+14h`、base action `+18h` 的 frame count 為 12 且 unsigned delta >40，才留下 `641498=1`；之後才 cap 至 500。不能用最小化、worker pending、paint FPS 或 walk 完成布林代替。 |
| `4E8750` | 玩家讀特有 globals，還有 `64BC13 != 0 && 64AFBF == 0` 時的 7/16 分支；普通 speed／brave 使用逐段整數運算。現有 `native_speed::factor` 不是整個原 duration producer。 |
| `4ADDA0 → 4857B0` | 特效虛擬呼叫返回後，依 `63DF4C != 0`、封包 graphics=169、target=own actor 判斷。不能用收到 2941 或 `skill_pending.is_some()` 直接冒充完整控制旗標。 |
| `4857B0` | 入口先以 `4D6870` 取消 `482F50`／參數0，再設 `62F32C=-1`、`75A988=0`；至少一次 drain→draw→fresh relative sample，signed 累加達500後設 `75AB1C=1`。現有 helper 只覆蓋 timer／draw 子流程；這些 state／取消 callback 與正式 S37 adapter 尚未實作。 |
| 方向6／shadow | 前次釘住的普通初始化與固定2941 list row仍成立：flags=0、alias=-1、shadow=-1；本 bank 使用普通 Over／單 primary frame，沒有 alias／shadow。未取得獨立原生像素 oracle。 |

`native_active_tick_study` 明文拒絕假稱已執行 `488860` surface/list/music/provider body；`native_locomotion_input_study` 的 inputs 是條件式研究值，不是實際 Windows／controller 的 authority。`fixed_walk` 只提供四／八 source 的無節奏參考步伐，也明文不擁有 world controller／`4E8750` 的全部狀態。沒有把這些研究預設值強行接入正式流程。

## 已驗證：建置與測試

命令都在本 worktree 的 `LiTo381-Modern/` 執行，`CARGO_HOME=D:/dev-cache/cargo`、`RUSTUP_HOME=D:/dev-cache/rustup`，TEMP／TMP 指向本 worktree `scratch/tmp`。沿用 `scratch/run143.ps1`，以 AST 讀取現行 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult` 原函數，未放寬 0 tests、failed>0、解析失敗與 command exit 門檻。

### 修正前的回歸證據

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_static2941::tests::callbacks:: -- --test-threads=1
exit=101
gate=FAIL: command-exit,test-failed
test result: FAILED. 4 passed; 3 failed; 1 ignored; 0 measured; 2540 filtered out; finished in 0.01s
```

三個失敗分別為 `retired_outer_drain_cannot_consume_replacement_queue_even_with_same_owner`、`nested_pump_retirement_during_drain_prevents_stale_draw_and_clock_reads`、`nested_pump_draw_retirement_does_not_resume_old_relative_timer`。不是編譯錯誤或門檻解析失敗；原始 assert／panic 保存在 `scratch/reentry-before.log`。

### 最終來源

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
exit=0
gate=PASS
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib
exit=0
gate=PASS
test result: ok. 1965 passed; 0 failed; 583 ignored; 0 measured; 0 filtered out; finished in 3.45s
```

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_static2941::tests:: -- --include-ignored --test-threads=1
exit=0
gate=PASS
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 2517 filtered out; finished in 7.64s
```

本張31案包含已移植的12 ready gates（8純邏輯、4原資源 opt-in），不是以 ignored 或 metadata 代替執行。另有原 #142 核心案、callback／draw 案、原 PE／ASM 案。完整 library 的 ignored 不計通過；本張另外執行其中7案，其餘576案未跑。

原資源命令明確設定 `LITO_STATIC2941_ROOT`，原來源命令明確設定 `LITO_NATIVE_CLOCK_PE`、`LITO_NATIVE_CALLBACK_ASM` 為上述唯讀來源。日誌與 gate 分別為 `scratch/check-final.log`、`lib-final.log`、`static2941.log` 及同名 `*-result.txt`。仍有既有 warnings，包含未有正式 producer 呼叫者的 dead-code 警告；沒有宣稱零警告。

### Library-test 與七個整合 metadata

從原專案 `驗證紀錄/current/native-E149-static2941-queue-metadata/20261024-203626-169258/cohort.json` 的 `paired_tests` 與 `input_pins` 重新唯讀取樣，核對七個 QA 本文 SHA256、單一 hardlink count；僅將 QA 本文放在本 worktree `scratch/metadata/`，沒有套用候選 Runtime。

fixture include 重綁至本 worktree；`object374448` 的五處 `.observe_equipment(` 依前次確認的方法名適配改回當版 `.observe_item(`，所有預期值與分支不變。這不授予裝備接線的行為資格。

| QA | 原文 SHA256 |
|---|---|
| blind20 | `03DEB74A4883C28A96058AEC59ABE0E2EBE0BE9514B7364F3FE5210E7A77C5B4` |
| slow29 | `CFA9685F0B3EBB1098C83CA140D17955B42AE59E3AE90EDD8F3CE8006EDE3B44` |
| advance79 | `96DB321632AA8868E4F5230C1E22E143090B6E1F777F488F4D4E4F9E4FBFA935` |
| item_spells | `FFFA7E2FD2476715C558D146A745228D1924A4D23D30686B958FDC15D5D06556` |
| magical73 | `5F90100B56394690393168F2F4B41720E928642B0F3F9EFFDC7C839CDC6DE1DE` |
| shadow107 | `F73F627CAB38B179228F0B61F13593E3E8A4ABF805DBCF8E4CD89BB426D22919` |
| object374448 | `F170F1B1B857297EB579283C7AF897A73019CD65AA33C5AB6A7C65B4787587EE` |

兩版先執行 `cargo check --lib --tests --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --message-format=json`（Release加 `--release`），核對 `build-finished.success=true`、library 與 library-test artifacts。七個 QA 用該版直接相依與 library metadata、target／host dependency 搜尋路徑、`rustc --test --emit=metadata --edition=2021 --target x86_64-pc-windows-msvc` 重編，Release另加 `-C opt-level=z -C debug-assertions=off`。

```text
debug library-test=False exit=0 fresh=True
debug library-test=True exit=0 fresh=True
debug advance79 exit=0
debug blind20 exit=0
debug item_spells exit=0
debug magical73 exit=0
debug object374448 exit=0
debug shadow107 exit=0
debug slow29 exit=0
release library-test=False exit=0 fresh=False
release library-test=True exit=0 fresh=False
release advance79 exit=0
release blind20 exit=0
release item_spells exit=0
release magical73 exit=0
release object374448 exit=0
release shadow107 exit=0
release slow29 exit=0
```

完整命令在 `scratch/metadata/<profile>-<name>-command.txt`，輸出在 `current-<profile>-<name>.log` 與 `current-results.txt`，一次性 runner 為 `scratch/rebuild143.py`。metadata 沒有 `test result:`，**僅算編譯成功，七個整合測試本文未執行**。產物只在 `LiTo381-Modern/target/current/issue143/metadata/`，不入 git。

## 推測

- worker 預先合成、controller enqueue 與下一次 poll 之間可能有可見窗口，但無法保證實際 paint 或可見幀數。
- 普通初始化的無 alias／shadow 支持現有 Over 路徑；未做獨立原像素比對。
- 將上述來源狀態映射為現代輸入／動作 authority 的完整方案尚未實作；不能因 helper 或來源核對通過，就推導正式 player scheduler 等價。

## 待驗證／未完成與給下一棒的提示

1. **本機 code 執行器接續同一 #143，先完成正式玩家 callback adapter。** 必須有真正的 source frame 推進、`4E8750` duration 狀態、held／active／pending／cancel producers，並讓 `4E2150` 與 static retirement 共用同一 queue；不可從 history、heading 或 elapsed pose 反造旗標。
2. 建立同步繪圖邊界與 normal-tail clock 狀態：callback 內真正提交既有資源／fragments，處理 `641498`，在 callback 返回前允許合法 reentry。只發出 `InvalidateRect` 或排下一張 worker，不足以證明原 suffix 的 draw→enqueue→外層固定 cutoff 交錯。
3. 補 `63DF4C` 的正式 set／clear authority 與 S37／169 own-target adapter；補 `482F50` 的識別／取消、camera／完成狀態，才能把 `drain_4857b0` 接入正式分支。
4. 加入 controller→真正 producer→畫面提交的整合正例及反例（兩 suffix、取消／pending 分支、169 wrong-target／flag=false、巢狀 drain、同 sample 與 rollover、退役／換代）。接線後重跑本報告命令與兩版 metadata；目前31案沒有取代這些必要驗收。
5. **Claude／調度器保留 review:fail／未完成狀態。** 目前可審查的是 queue 退役修正與來源證據；正式接線仍須本張後續實作，不應只派重跑相同 helper tests，亦不需使用者再批准已授權的本機測試。

未做原生 GUI、正式服務／DB、原遊戲程序、獨立原像素 oracle、Release測試或 G06 640次。未啟動／附加／停止任何遊戲、未讀寫機密設定或封存，未修改硬連結。未新增 GitHub 留言或標籤。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_static2941.rs`
- `LiTo381-Modern/src/modern_lito/native_static2941_callback_tests.rs`
- `LiTo381-Modern/src/modern_lito/native_static2941_source_tests.rs`
- `docs/reports/issue-143.md`（本報告，取代前次版本；歷史在 git）
- 一次性腳本／日誌：本 worktree `scratch/`；建置產物：`LiTo381-Modern/target/current/`，均不提交。
