# Issue #324 — T037（M3 1/2）G12 稽核：證據保存與冗餘產物清單

執行者：claude-local（本機，唯讀盤點）。盤點時間 2026-11-12 17:43；對象 `D:\lineage開發\3-lito開發\lito重建版`（不含 `.git`）。

## 結論

- **沒有刪除任何檔案。** 本張完成的是「保存清單」與「刪除前清單」；實際刪除還沒做，原因有兩個：本機執行器被規定不得在原始專案目錄寫檔，而且刪除不可逆，需要使用者先同意。
- 專案目錄（不含 worktree）共 357,123 檔、370.99 GiB，其中 `LiTo381-Modern/target` 佔 336.54 GiB。
- 可以確定是冗餘的只有一類：cargo 的中間產物，45,320 檔、**108.51 GiB**（A 類）。同意後用一個指令就能刪。
- 另有 57 個各輪留存的大檔是其他檔的完全相同副本（SHA256 相同），合計 33.01 GiB；這些是刻意留存的產物，刪不刪要另外裁定。
- 不在 git 裡的原始碼、逆向與回復證據共 2,203 檔、403 MB，已逐檔記下 SHA256；這些檔目前只有本機一份。
- 另外發現 `scratch/wt` 下有 52 個 worktree、261.46 GiB（其中 `target` 257.10 GiB）。worktree 歸調度器清理，本張沒有動，只回報數字。
- D 槽剩餘 5,900 GiB，沒有空間壓力，刪除不急。

## 做了什麼

1. 讀 `CLAUDE.md`、`docs/REPORT.md`、`docs/REUSE-MAP.md`、`docs/G01-G12.md` 的 G12 定義（work 第 1 項：「保存唯一原始碼／逆向／回復證據後整理已核實冗餘產物」）。REUSE-MAP 沒有 G12 稽核的對應項，本張不引用 TW／LS 的檔案。
2. 新增唯讀工具 `維護工具/g12-artifact-inventory.py`：`scan` 列出全部檔案（大小、硬連結數、修改時間）並分類；`clean` 預設只預演，加 `--apply` 才刪。
3. 對原始專案目錄跑 `scan` 與 `clean` 預演（都不寫入該目錄，輸出在本 worktree 的 `scratch/issue-324/`）。
4. 對不在 git 的原始碼／逆向／回復證據逐檔算 SHA256，寫成 `docs/reports/issue-324/preserve-manifest.tsv`。
5. 對 `target` 內 cargo 版面以外、50 MB 以上且大小相同的檔算 SHA256，找出完全相同的副本，寫成 `duplicates.tsv`。
6. 量了 `scratch/wt` 各 worktree 的大小（不跟隨 junction），寫成 `wt-sizes.tsv`。

## 一、保存：唯一原始碼、逆向與回復證據

| 項目 | 位置 | 數量 | 保存狀態 |
|---|---|---|---|
| 現行原始碼 | git（`origin/main`） | 追蹤檔 4,765 個 | 原始目錄停在 `92bbda0`，`git status` 無任何變更，且 `92bbda0` 是 `origin/main` 的祖先。所以原始目錄裡的追蹤檔沒有 git 以外的獨有內容 |
| 44 檔隔離候選與其他候選本體 | `維護工具/candidates/` | 1,663 檔、65.6 MB | 不在 git（`.gitignore`）。其中 129 檔的 SHA256 與 `docs/CANDIDATES.md`／`docs/reports/issue-41.md` 記錄的一致 |
| G01–G12 定義來源 | `工作佇列.json`、`監製進度.json` | 11.9 MB、11.4 MB | 不在 git。`工作佇列.json` 的 SHA256 `0EC8ACBF…9830AC` 與 `TASKS.md` 記錄的相同 |
| 回復證據 | `封存/before-cleanup-20261012.zip`（SHA256 `AA253015FC2957E6…`）、`封存/before-rebuild-20261012.zip`（`C538562B21A1F0C6…`） | 2 檔、26.9 MB | 不在 git，未開啟、未更動 |
| 上次整理的紀錄 | `整理紀錄/`（2026-10-03 的移動計畫、移動紀錄、連結檢查、執行腳本） | 8 檔 | 不在 git |
| 逆向與來源證據 | `LiTo381-Modern/provenance/`、`歷史暫存證據/` | 327 檔、181 檔 | 不在 git |
| 交付候選與舊版成品 | `交付/`、`LiTo381-Modern/dist/`、`LiTo381-Modern/reports/` | 7、8、2 檔 | 不在 git |
| 各輪來源快照 | `LiTo381-Modern/target/` 內 cargo 版面以外的 `.rs`、`Cargo.*` | 58,326 檔、1.11 GiB | 不在 git；屬 E 類，清單不會碰 |
| 驗證證據 | `驗證紀錄/`（32.46 GiB，102,449 檔）、`LiTo381-Modern/logs/`（1.55 GiB，19,481 檔） | — | 不在 git；只統計大小，沒有逐檔雜湊 |

前七列（`target` 快照與驗證證據除外）的逐檔 SHA256 在 `preserve-manifest.tsv`（2,203 檔、403,073,098 bytes）。整個專案目錄沒有 junction 或 symlink；沒有 `LiTo.bin` 的副本。

## 二、冗餘產物分類（`LiTo381-Modern/target`，588 個第一層子目錄）

| 類別 | 定義 | 檔數 | 大小 | 處置 |
|---|---|---|---|---|
| **A** | 路徑含 `<debug\|release>/<deps\|incremental\|.fingerprint>/` 的 cargo 中間產物（`deps` 內只認 `.rlib .rmeta .pdb .o .d .lib .exp .ilk`），硬連結數 1 | 45,320 | 108.51 GiB | **列入刪除清單** |
| AH | 條件同 A，但硬連結數大於 1 | 1,989 | 14.92 GiB | 不動（規定不動硬連結） |
| X | cargo 版面內的其他檔：測試與主程式 exe、dll、build script 輸出 | 12,838 | 11.64 GiB | 不動（`tools/prepare-r519.py` 等腳本會引用 `deps/*.exe`） |
| B | cargo 版面以外、各輪留存的 `.rlib .rmeta .pdb .o .lib` | 1,769 | 165.11 GiB | 不動，待裁定 |
| Bexe | cargo 版面以外的 `.exe .dll` | 1,380 | 17.64 GiB | 不動，待裁定 |
| E | 其餘：來源快照、PNG、JSON、pak／jar 副本等 | 165,119 | 18.72 GiB | 不動 |

A 類再分兩塊：
- 歷史輪次（`current` 以外的 136 個 profile 目錄，2026-09-26 到 10-08，多為 i686 建置）：38,459 檔、81.90 GiB。
- `target/current`（4 個 profile 目錄，原始目錄的現用建置快取）：6,861 檔、26.61 GiB。刪掉後下次在原始目錄建置要完整重編。

B 與 Bexe 合計最大的幾組（詳見 `target-tiers.tsv`）：`r4xx–r5xx-player-entry-candidate` 65 個目錄 51.2 GiB、`fixed-finite-appearance-host` 43 個目錄 19.4 GiB、`fixed-consumer` 25 個目錄 17.6 GiB、`target/current` 內 cargo 版面以外的部分 27.4 GiB。

完全相同的副本（`duplicates.tsv`）：39 組、96 檔。每組至少留一份、只算硬連結數 1 的副本，可以去掉 57 檔、33.01 GiB。例如 `r540-…-final/library/liblito_modern.rlib` 與 `r542-native-backend-candidate/library/liblito_modern.rlib` 的 SHA256 相同。

## 三、刪除清單與執行方式（尚未執行）

- 目錄層級的清單：`docs/reports/issue-324/tierA-roots.tsv`（140 個 profile 目錄，各列 A 類位元組數與檔數、不動的硬連結與其他檔）。
- 逐檔清單有 45,320 行（5.5 MB），沒有進 git。它由 `scan` 重新產生；本次產生的 `tierA-files.tsv` SHA256 是 `DD0CF38C86950E796304E66FEC3EC674AFE4954D9A68ADAF61432A78CED0D31B`。
- 使用者同意後，在原始專案目錄執行：

```powershell
python -I 維護工具\g12-artifact-inventory.py scan  . scratch\g12-cleanup          # 重新盤點
python -I 維護工具\g12-artifact-inventory.py clean . scratch\g12-cleanup          # 預演，看 clean-dryrun.txt
python -I 維護工具\g12-artifact-inventory.py clean . scratch\g12-cleanup --apply  # 實際刪除
```

- 只想刪歷史輪次、保留現用快取時：`--apply` 之前先把 `scratch\g12-cleanup\tierA-files.tsv` 裡 `LiTo381-Modern\target\current\` 開頭的行拿掉。
- `clean` 逐檔重新檢查：路徑在 `LiTo381-Modern/target` 內、不是 reparse point、硬連結數 1、大小與清單一致、重新分類仍是 A。不符就略過並記錄。只刪檔案，不刪目錄。

## 已驗證／推測／待驗證

**已驗證**
- 上面所有檔數與大小（`scan` 實際掃描；掃描 0 錯誤、0 個 reparse point）。
- 原始目錄的追蹤檔與 `92bbda0` 相同，且該 commit 在 `origin/main` 的歷史裡。
- `工作佇列.json` 與 129 個候選檔的 SHA256 與既有文件記錄一致。
- `clean` 對真實目錄的預演結果：會刪 45,320 檔、116,509,290,682 bytes，略過 0 檔。
- `clean --apply` 的行為：在 `scratch/` 內的假目錄樹實測（`scratch/issue-324/selftest.py`）。A 類被刪；exe、`build/`、cargo 版面以外的 rlib、`target` 以外的同名路徑都保留；掃描後才變成硬連結或大小改變的檔被略過；清單被改成指向 `target` 以外或用 `..` 跳出的路徑被略過；沒有刪目錄。
- 追蹤中的腳本引用 `deps/` 的地方只有 `deps/*.exe` 與一行註解（27 個腳本檔，已逐行看過），A 類不含 exe。
- 39 組副本的 SHA256 完全相同。

**推測**
- A 類刪除後不影響任何既有證據的可讀性：證據 JSON（7.4 GiB）沒有全文搜尋過是否有引用 `deps/` 內 rlib／pdb 的路徑或雜湊。即使有，這些檔是編譯器中間產物，不是驗收對象。
- 歷史輪次的 A 類無法重建成位元組完全相同的檔（當輪原始碼只剩 `target` 內的快照，工具鏈也換成 x86_64）；「可重建」指的是同類產物可以由 cargo 再產生，不是指原檔可還原。
- B 類的各輪 rlib／pdb 多半已被後續輪次取代。

**待驗證**
- `clean --apply` 還沒有在真實目錄執行過。
- 2,203 個獨有證據檔是否有本機以外的備份：不知道。
- `驗證紀錄/` 與 `logs/` 沒有逐檔雜湊，也沒有檢查內部重複。
- B、Bexe 類是否被驗收收據引用，要等第 2 張（46 項逐項核對）對照收據後才能判定。
- 刪除 `target/current` 的 A 類後，原始目錄重新建置是否正常：要實際建置才知道。

## 未做的事

- 沒有刪除、移動或改寫原始專案目錄的任何檔。
- 沒有動 `封存/`、硬連結、`D:\lineage開發\0-客戶端`、`scratch/wt` 的其他 worktree。
- 沒有改 `TASKS.md`、`PLAN.md`、`docs/REPORT.md`。
- 沒有執行任何編譯出來的程式；只跑了 python 腳本與 git。

## 產出路徑

- `docs/reports/issue-324.md`（本檔）
- `docs/reports/issue-324/summary.txt`：掃描總表
- `docs/reports/issue-324/preserve-manifest.tsv`：獨有證據逐檔 SHA256
- `docs/reports/issue-324/tierA-roots.tsv`：刪除清單（目錄層級）
- `docs/reports/issue-324/target-tiers.tsv`：`target` 各子目錄的六類大小
- `docs/reports/issue-324/duplicates.tsv`：完全相同的副本
- `docs/reports/issue-324/wt-sizes.tsv`：各 worktree 大小
- `維護工具/g12-artifact-inventory.py`：盤點與清理工具

## 給下一棒的提示

1. **需要使用者裁定**（不可逆）：
   - 是否刪 A 類歷史輪次 81.90 GiB（建議刪）。
   - 是否連 `target/current` 的 26.61 GiB 一起刪（建議先留；原始目錄已落後 `origin/main` 471 個 commit，之後若不再在那裡建置再刪）。
   - 57 個完全相同的副本 33.01 GiB 是否去重（建議等第 2 張核對收據後再決定）。
   - 獨有證據是否要做本機以外的備份。
2. 同意後由本機對話在原始專案目錄執行第三節的三個指令；執行前先 `git pull`，讓工具出現在原始目錄。刪完再跑一次 `scan`，用 `preserve-manifest.tsv` 抽查獨有證據仍在。
3. **調度器**：`scratch/wt` 有 52 個 worktree、261.46 GiB，其中 37 個帶 `target`（最大的 `task-54` 28.4 GiB）。清理由調度器處理。這些 worktree 裡有 11,603 個硬連結檔，沒有 junction 或 symlink（本次掃描 0 個）。
4. 第 2 張（46 項逐項核對）可以直接用 `target-tiers.tsv` 與 `duplicates.tsv` 查各輪產物是否仍在；判定 B 類能不能刪時，要對照各項收據引用的 rlib／exe 雜湊。
