# Issue #327 — T042（M3 1/1）`run-local-test-suite.ps1` 的 `-Full` 模式

執行者：claude-local（本機，只寫腳本與靜態檢查）。基準：`origin/main` `3a81fc2`；push 前合併到 `409726c` 並重跑腳本回歸。

## 結論

`-Full` 已加進 `維護工具/run-local-test-suite.ps1`，腳本自身的回歸檢查通過。**本張沒有執行任何 Rust 測試，也沒有啟動 cargo；`-Full` 的實跑還沒做，交 Codex。**

有一件事會擋住實跑的結束碼，而且不是本張造成的：現行 main 的 `skills.tsv` 已經落後 `docs/SKILLS-STATUS.md`，技能類現在會整類失敗（詳見「發現的既有問題」）。這要先修，`-Full` 才可能拿到結束碼 0。

## 做了什麼

### 1. `-Full` 參數

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具\run-local-test-suite.ps1 -Full
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具\run-local-test-suite.ps1 -Full -Category T036
```

| 類別 | 不加 `-Full`（行為不變） | 加 `-Full` |
|---|---|---|
| Baseline | `check`、`lib-debug`、`lib-release`；`lib.txt`、`tests.tsv` 只跑 Debug | 同左，另外把 `lib.txt`、`tests.tsv` 再跑一輪 Release |
| Skills | `skills.tsv` 只跑 Debug | Debug、Release 各一輪 |
| T036（新） | 不執行 | `full.tsv` 的放行項目跑 Debug、Release 各一輪；排除項逐項列出理由 |
| G06 | 八案，Debug 測試 EXE | Debug 一輪，再用 Release 測試 EXE 一輪 |

- 總表在 `-Full` 下多一欄 `Profile`，結尾多三種統計：合計（`Counted／PASS／FAIL／EXCLUDED`）、Debug 與 Release 分計、各排除理由的項數。
- 結束碼：有執行的列全部 `PASS` 才是 0。`EXCLUDED` 列不影響結束碼，也不計入 PASS。
- `-Full` 不能和 `-Quick` 併用；`-Category T036` 一定要搭配 `-Full`。兩種誤用都在啟動 cargo 之前就報錯結束。
- 不加 `-Full` 時唯一看得到的差別：總表第一行的模式名稱從 `Mode=Full` 改成 `Mode=Standard`（原本「非 Quick」就叫 Full，會和新參數混淆）。全庫搜尋過，沒有別的腳本或文件解析這個字串。

### 2. T036 總帳 `維護工具/test-whitelist/full.tsv`

TASKS 的 T036 是「白名單以外的 1055 個待驗測試」。這 1055 項的逐項清單在 `docs/reports/issue-54.md`（1132 項扣掉當時放行的 77 項）。我把它整理成一份總帳，一列一個測試，每列有一個去向：

| verdict | 項數 | 意思 |
|---|---:|---|
| `run` | 111 | 已審查放行，`-Full` 會執行。#306 的 73 項＋#307 的 38 項 |
| `covered` | 38 | 其他類別已經在跑：33 項在 `tests.tsv`／`skills.tsv`，5 項是 G06 的 scenario |
| `excluded` | 906 | 不能執行，逐項附理由 |

111 個放行項目裡，76 項會唯讀讀取寫死的 `D:\lineage開發\0-客戶端`（`requires=client`）。

排除理由（906 項）：

| 代碼 | 項數 | 理由 | 等哪一張 |
|---|---:|---|---|
| `helper-pending` | 302 | 跨函式 helper 閉合審查還沒做 | #308（43 項）、#309／#310（259 項） |
| `java-peer` | 167 | 需要 Java peer／TLS fixture | #314–#316 |
| `shared-root` | 159 | 需要 `LITO_SHARED_RESOURCE_ROOT` | #312、#313 |
| `db` | 120 | SQL／MySQL peer 流程（#319 已盤點，尚未放行） | #320 |
| `snapshot` | 99 | 依賴原機 snapshot 的絕對路徑與 SHA pins | #317 |
| `env` | 40 | 其他環境變數或權限前置（每列寫明是哪個變數） | #318 |
| `tracked-evidence` | 10 | 會覆寫 git 追蹤的 `logs/continuation-20261003` 證據檔 | #306 已裁定不放行 |
| `gui` | 6 | 需要原生視窗或操作員流程 | T030 |
| `helper-open` | 2 | #307 沒有閉合的兩項 | #309 |
| `real-db` | 1 | 需要正式 DB 設定 | T030 |

`shared-root` 159、`snapshot` 99、`env` 40、`gui` 6、`real-db` 1 這幾個數字和 TASKS 的 T036 分張描述一致。

T036 目前只完成 2 張（#306、#307），所以現在的 `-Full` 只會多跑 111 項。後續各張審完，改總帳對應列的 verdict 就會納入，不用再改腳本。更新規則寫在 `維護工具/test-whitelist/README.md`。

### 3. 腳本的防呆

- 總帳每次執行都會檢查：欄數與格式、重複項、`excluded` 沒寫理由、標成 `covered` 但沒有任何類別在跑、標成 `excluded` 但白名單其實在跑。任何一種都讓 T036 類只輸出一列 `FAIL: T036-manifest`，不執行任何測試。
- 放行項目後來被加進 `skills.tsv` 等清單時，只在原類別跑一次，T036 類不重複跑。
- `requires=client` 的項目在主機上找不到資源根時，逐項列為 `FAIL: precondition: client-resource-root-missing`。已經放行卻跑不了，不當成排除。
- #306 提醒「37 項不會自己建 `target/current/verification/`」：腳本建立自己的輸出目錄時，這個目錄和 `target/current/scratch/` 都會一併建好。
- 子程序仍然清除繼承的 `LITO_*`，TEMP／TMP 指向本 worktree。

### 4. 腳本回歸 `維護工具/test-local-test-suite.ps1`

新增一段 `-Full` 斷言（合成輸出，不啟動 cargo）：真實總帳 1055 列可解析、和現行白名單沒有矛盾；9 種壞列被拒絕；Debug／Release 的 cargo 參數；放行項目各跑兩輪、`covered` 與 `excluded` 不會被執行；排除理由逐項出現在總表；只有排除項時結束碼 0；缺資源根、測試失敗、總帳重複項時結束碼 1；不加 `-Full` 的總表格式沒變。

## 已驗證

- **腳本回歸（Windows PowerShell 5.1）：合併 main 後 325 個斷言通過。** 跑的是去掉「技能清單」那一段的副本（`scratch/issue-327/selftest-without-skills.ps1`），原因見下一節。新增的 `-Full` 斷言和原有的 G06、子程序、結果解析斷言都在裡面。
- `run-local-test-suite.ps1` 解析無語法錯誤（PowerShell 5.1 parser）。
- `-Category T036`（不加 `-Full`）與 `-Quick -Full` 都在啟動 cargo 前報錯，結束碼 1（實際執行過這兩個指令）。
- 總帳 1055 列和 #54 逐項表一對一；#306 的 83 項、#307 的 38＋2 項都能在 #54 表裡找到同名項目（產生腳本內的斷言）。
- 放行的 111 項，函式名稱在現行 `src/`、`tests/` 都找得到（靜態比對函式名）。

## 發現的既有問題（不是本張造成，本張沒有修）

**`skills.tsv` 落後 `docs/SKILLS-STATUS.md`，技能類現在會整類失敗。** 在本分支基準 `3a81fc2` 與合併後的 `409726c`（兩個檔案本張都沒動）直接跑 `test-local-test-suite.ps1`，在既有的技能清單檢查就停了：

```text
Unmapped T026 test: native_skills::t027_batch4_tests::eight_wire_shapes_are_exact_and_costs_are_not_predicted; review and update test-whitelist/skills.tsv
```

影響：

- `run-local-test-suite.ps1`（不論加不加 `-Full`）的技能類會只輸出一列 `FAIL: T026-manifest`，結束碼 1。
- 我只看到第一個缺項；腳本遇到第一個就停，實際缺幾項沒有清點。
- 補清單前要先核對那些測試的副作用（README 的既有規定），這超出本張範圍，所以沒有動。

## 推測

- Release 的 G06：`run-teleport-failure.ps1` 只要求測試 EXE 在本 worktree 的 `target/current` 之下，沒有限定 debug 目錄，所以我判斷可以直接吃 Release 測試 EXE。沒有實跑過。
- `-Full` 全部跑完大約 820 次 cargo 呼叫（Baseline 433、Skills 約 136、T036 222、G06 兩輪）。大部分是已編譯 target 的單一測試，但 Release 第一次要整個重編，總時間沒有量過，估計以小時計。
- T036 的寫檔測試用固定檔名寫 `target/current/verification/`（#306），Debug 那一輪的證據會被 Release 那一輪覆寫。只影響證據保存，不影響通過與否。

## 待驗證（交 Codex 實跑）

- **`-Full` 整體沒有實跑過。** 111 個 T036 放行項目從來沒有執行過（#306、#307 都只做靜態審查），Debug 或 Release 會不會通過都不知道。
- Release 下的 `lib.txt`、`tests.tsv`、`skills.tsv`、G06 八案也沒有跑過（原本只跑 Debug）。
- PowerShell 7 下的腳本回歸：本機 PATH 沒有 `pwsh`，只驗了 5.1。
- 完整的 `test-local-test-suite.ps1`（含技能清單那一段）要等 `skills.tsv` 補齊後才跑得完。

## 產出路徑

- `維護工具/run-local-test-suite.ps1`（`-Full`、`T036` 類、總表）
- `維護工具/test-whitelist/full.tsv`（T036 總帳，1055 列）
- `維護工具/test-whitelist/README.md`（`-Full` 說明、總帳格式與更新規則）
- `維護工具/test-local-test-suite.ps1`（新增 `-Full` 斷言）
- `docs/reports/issue-327.md`（本檔）
- 一次性產生腳本在 `scratch/issue-327/gen.py`（不進 git）
- 分支 `task/327`

`docs/REUSE-MAP.md` 已查：T041 那一列寫「只借分類與總表格式」，本張沒有引用 tw-bin-engine 或 lito-launcher-server 的檔案。沒有宣稱缺資源，所以沒有做 pak 查詢。

## 給下一棒的提示

- **先修 `skills.tsv`**（見「發現的既有問題」），否則任何模式的結束碼都是 1。建議另開一張，或併入正在加技能測試的那幾張。
- **Codex 實跑**：建議先 `-Full -Category T036` 單獨跑一次（222 步），確認 111 項的實際結果，再跑整個 `-Full`。跑的時候不要同時啟動或更新遊戲客戶端（測試會以唯讀獨占方式開 pak），同一個 worktree 不要同時跑兩份。
- T036 放行項目若實跑失敗，請在對應的 T036 Issue 處理（修測試，或把總帳那一列改回 `excluded` 並寫明理由），不要從總帳刪列。
- **T036 後續各張（#308–#320）**：審完直接改 `full.tsv` 對應列。需要環境變數、JAR 或 fixture 目錄的項目，腳本目前沒有準備步驟，而且會清掉子程序的 `LITO_*`；要放行這類項目得先擴充腳本（例如 requires 欄增加新的值）。
- #311（T036 的執行張）原本要自己決定把 #306／#307 的清單併進哪一份白名單；現在已經在 `full.tsv`，不必再併進 `tests.tsv`／`skills.tsv`。
- CI 目前不跑 `test-local-test-suite.ps1`。等 `skills.tsv` 修好後可以考慮加進 `build.yml`（它不啟動 cargo，只需要 PowerShell）。
