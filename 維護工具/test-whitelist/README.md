# 本機測試套件清單

在 repo 根目錄執行：

```powershell
pwsh -NoProfile -File 維護工具/run-local-test-suite.ps1
```

預設依序執行 T012 基線、技能、G06，任一失敗都回傳 1。總表與逐步 log 存在本 worktree 的 `LiTo381-Modern/target/current/verification/local-test-suite/`。技能與 G06 每項必須只有一個完整 Rust harness，`running 1 test`、`1 passed`、`failed/ignored/measured=0`、程序結束碼 0；0 tests、結果缺失或解析失敗均不通過。

可選參數：

| 參數 | 範圍 |
|---|---|
| `-Quick` | 保留 T012 的 check 與 Debug lib 兩步；不是 M2 完整驗收 |
| `-Category Baseline` | 原 T012 完整 80 步 |
| `-Category Skills` | 只執行技能表列名的精確測試 |
| `-Category G06` | 只執行八個 G06 scenario，含 no-run 建置與 owned runner |
| `-Full` | M3 完成定義用（T042）：每一類都跑 Debug 與 Release，另加 T036 放行項目；見下方「`-Full` 與 T036 總帳」。不能和 `-Quick` 併用 |
| `-Full -Category T036` | 只跑 T036 放行項目（Debug＋Release）並列出排除項；`T036` 類只在 `-Full` 下可用 |
| `-ReferenceJar <絕對路徑>` | 本 worktree `target/current` 內的 E149 JAR；預設 `target/current/server-reference/formal-server-E1498871.jar`（相對 Cargo 專案） |
| `-PowerShellPath <pwsh.exe>` | 從 Windows PowerShell 5.1 呼叫 G06 時，可指定 PowerShell 7+；否則從 PATH 尋找 |

G06 使用既有 `run-teleport-failure.ps1` 的 JDK、memory 模式、loopback listener、來源綁定、原子 lease、程序身分與清理守門。JAR 必須先由本機執行者驗證為 E149（SHA256 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`），再準備於本 worktree 的上述路徑；套件不掃描或修改外部伺服器目錄。測試 EXE 從本次 `cargo test --no-run --message-format=json` 的單一 artifact 取得，不以目錄中「最新 EXE」猜測。Cargo／pwsh 子程序清除繼承的 `LITO_*`，TEMP／TMP 使用本 worktree 的 `target/current/scratch`。

G06 必跑順序與契約 `native_E149_recovery_contract.py` 一致：`timeout`、`refused`、`wrong-service`、`cancel-retry`、`cancel-expiry`、`cancel-repeat`、`refused-cancel-repeat`、`cancel-expiry-source`。後三案由 T023 第 2 張（#170，已合併）補回，現行 runner 與 `native_teleport_failure.rs` 已有全部八案。套件先核對 owned runner 是否支援全部八案（以 UTF-8 讀取 runner 原始碼後解析 `ValidateSet`，PowerShell 5.1 也可讀）；缺案時八列都顯示 `FAIL: precondition: incomplete-owned-scenarios (T023-2)` 並列出缺項，結束碼為 1，不建置或啟動 G06 子程序。支援齊全後才建置，並以測試 EXE 的 `--list --ignored --format terse` 檢查八個精確測試名稱各出現一次；列舉失敗、缺項或重複都不啟動 Java。此列舉不是測試通過證據。開始實跑後若任一案失敗，後續案均列為未執行的失敗，不重試。前置條件缺失同樣會列出八個失敗項；只有八案全部實跑通過才可稱 G06 通過。

## `-Full` 與 T036 總帳

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具\run-local-test-suite.ps1 -Full
```

不加 `-Full` 時行為與原本相同（白名單只跑 Debug；總表標題的模式名稱由 `Full` 改為 `Standard`，避免和本參數混淆）。加上 `-Full` 後：

| 類別 | 內容 |
|---|---|
| Baseline | `check`、`lib-debug`、`lib-release`，再把 `lib.txt` 與 `tests.tsv` 各跑 Debug、Release 一輪 |
| Skills | `skills.tsv` 跑 Debug、Release 各一輪 |
| T036 | `full.tsv` 裡 `run` 的項目跑 Debug、Release 各一輪；`excluded` 的項目不執行，逐項列出理由 |
| G06 | 八個 scenario 以 Debug 測試 EXE 跑一輪，再以 Release 測試 EXE 跑一輪 |

總表多一欄 `Profile`（`Debug`／`Release`；沒有執行的列是 `-`），結尾多出合計、Debug／Release 分計與各排除理由的項數。結束碼規則：有執行的列必須全部 `PASS` 才是 0；`EXCLUDED` 列不影響結束碼，但也**不算通過**。

`full.tsv` 是 T036「白名單以外 1055 個待驗測試」的總帳，一列一個測試，1055 列都在（清單來源是 `docs/reports/issue-54.md` 的逐項表，扣掉當時已放行的 77 項）。八欄、Tab 分隔、沒有標題列：

| 欄 | 內容 |
|---|---|
| target | 整合測試 target；lib 測試寫 `__lib` |
| name | 完整 Rust 測試名稱 |
| ignored | `yes`／`no`（執行時一律 `--include-ignored --exact`，這欄只是紀錄） |
| features | 沒有就寫 `-` |
| verdict | `run`：已審查放行，`-Full` 會執行。`excluded`：不能執行。`covered`：別的類別已經在跑（`lib.txt`、`tests.tsv`、`skills.tsv` 或 G06 八案） |
| requires | `client`：會唯讀讀取寫死的 `D:\lineage開發\0-客戶端`；其餘寫 `-`。只有 `run` 列可以寫 `client` |
| source | 放行或排除的依據（`#306`、`#307`、`T030`…） |
| reason | `excluded` 列必填，格式是「代碼: 說明」；其他列寫 `-` |

腳本每次執行都會檢查總帳：欄數、格式、重複項、`excluded` 沒寫理由、`covered` 但其實沒有別的類別在跑、`excluded` 但白名單其實在跑，任何一種都會讓 T036 類只輸出一列 `FAIL: T036-manifest`，一個測試都不執行。`run` 的項目後來被加進其他白名單時不算錯，只在原類別跑一次，不重複。

`requires=client` 的項目在本機找不到資源根時，各自列為 `FAIL: precondition: client-resource-root-missing`（放行了卻跑不了，不能當成排除）。

目前內容（#327 建立時）：`run` 111（#306 的 73 項、#307 的 38 項；其中 76 項要客戶端資源）、`covered` 38、`excluded` 906。排除理由代碼：

| 代碼 | 項數 | 意思 | 等哪一張 |
|---|---:|---|---|
| `helper-pending` | 302 | 跨函式 helper 閉合審查還沒做 | #308、#309、#310 |
| `java-peer` | 167 | 需要 Java peer／TLS fixture | #314–#316 |
| `shared-root` | 159 | 需要 `LITO_SHARED_RESOURCE_ROOT` | #312、#313 |
| `db` | 120 | SQL／MySQL peer 流程（#319 已盤點，尚未放行） | #320 |
| `snapshot` | 99 | 依賴原機 snapshot 的絕對路徑與 SHA pins | #317 |
| `env` | 40 | 其他環境變數或權限前置 | #318 |
| `tracked-evidence` | 10 | 會覆寫 git 追蹤的證據檔（#306 裁定不放行） | — |
| `gui` | 6 | 需要視窗或操作員流程 | T030 |
| `helper-open` | 2 | #307 審到一半、轉 #309 複核 | #309 |
| `real-db` | 1 | 需要正式 DB 設定 | T030 |

#308 之後（技能／戰鬥 2／2 審完）：`run` 154（多了 #308 的 43 項：25 項不需前置、18 項要客戶端資源；`run` 裡共 94 項要客戶端資源）、`covered` 38、`excluded` 863；`helper-pending` 剩 259，全部在等 #309、#310 的結果併入。`native_ranged` 有一列在 #167 改過名，總帳已改用現行名稱 `malformed_ranged_packets_are_atomic_for_every_flight_type`（#54 的舊名是 `…_and_other_effect_branches_remain_unknown`）。

**T036 後續各張怎麼更新總帳**：審查放行一個項目，就把那一列的 verdict 改成 `run`、reason 改成 `-`、source 改成自己的 Issue 號，並依審查結果填 requires。判定永久不能跑的，保留 `excluded` 並把 reason 改成確定的理由。不要刪列：1055 列要一直都在，才看得出每一項的去向。需要客戶端以外前置條件（環境變數、JAR、fixture 目錄）的項目，目前腳本沒有對應的準備步驟，放行前要先在腳本補上，否則會以測試失敗的形式出現。

## 技能精確對照

`lib.txt` 與 `tests.tsv` 保留 T012 原白名單；`skills.tsv` 使用相同四欄 TSV：target、完整 Rust 測試名稱、ignored 註記（yes/no）、features（無則 `-`）。保留 target `__lib` 表示 `cargo test --lib`。執行時一律 `--include-ignored --exact`，不會因 ignored 而誤報通過。

`skills.tsv` 對應 T026 `docs/SKILLS-STATUS.md` 的「送包測試」欄。#173 第 4 次（合併 main 後）基準有 181 技能列、36 個去重參照，精確展開為 66 項（63 整合、3 lib）；58 列有測試、123 列「尚無」。原 41 項之外新增：

- 整檔參照 `native_t027_batch1`（13 案，#145）與 `native_t027_batch2`（11 案，#146）：清單必須列出該檔所有 `#[test]` 函式，腳本以靜態原始碼掃描（不編譯、不執行）核對，缺一個、多一個、「（N 案」與原始碼數量不符、檔內出現 `mod`／`include!` 或無法解析的 `#[test]`，技能類都失敗。其中 3 項是 `#[ignore]` 的唯讀原資源記憶體繪製測試（`ignored=yes`），以 `--include-ignored --exact` 照跑，需要 `D:\lineage開發\0-客戶端`。
- 限定參照 `native_t021::wire_shapes_and_menu_keep_selectors_separate_from_inventory_objects`：以 target 加名稱對照。

原 41 項的細節：

- `formal_attack_spell_layout_and_request_guards` 對應 attack／fire arrow／ice arrow／wind cutter 四個 target。
- attack skills 中三個同名函式分別存在 `disintegrate_request_contract` 與 `call_lightning_request_contract`，各自執行。
- `unqualified_loader_keeps_world_update_ok_without_retry_or_authority_mutation` 經 `include!` 引入三個 `availability_tests` 模組，各自執行。
- 其餘測試使用清單中的精確 target 與函式名；共享多個技能的同一測試只列一次。與基線重複的測試仍在技能類獨立回報，不能累加成不同案例。

已檢查這 41 個選定入口及所用 fixture helper：使用記憶體狀態、嵌入 fixture、選擇器／封包／時鐘斷言；fire arrow 的入口另呼叫 `class4_zero_distance_zero_damage_and_miss_keep_authority`，會從 `D:\lineage開發\0-客戶端` 唯讀載入原資源，在記憶體 canvas 驗證投射物與事件。這些入口沒有外連、DB、GUI 或遊戲程序操作。`native_world_entry` 同檔的真實服務測試及其他未列入的資源／GUI 測試不會被選到。

每次執行均核對 181 列、唯一 ID、非空測試名稱、清單重複、未對照名稱及過期清單項。狀態表新增測試但清單未更新時，技能類會失敗，不能靜默漏跑；更新清單前須核對精確名稱與副作用。同一短名稱若新增 target 或 include 模組，也須同步補上精確對照。目前 123 列「尚無」測試只回報未驗證，不代表那些技能已通過；本腳本通過不會自動更新狀態表，也不代表技能的資源／畫面／伺服器同源性已驗證。

腳本自身回歸（合成輸出不計入 Rust 通過數）：

```powershell
pwsh -NoProfile -File 維護工具/test-local-test-suite.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/test-local-test-suite.ps1
```
