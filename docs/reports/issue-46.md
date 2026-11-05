# Issue #46 / T003-R2：依 #41 結果更新 CANDIDATES.md

## 做了什麼

依 `docs/reports/issue-41.md`（T003-R1，使用者電腦上的靜態核對，已合併）更新 `docs/CANDIDATES.md`，共四處：

1. **一覽表 44 列「差異行數」欄**：原本是「無法計算」。
   - 有同名 predecessor 的 25 列，改填 `+增／−刪`（#41 的 `git diff --no-index --numstat`）。
   - 沒有同名檔的 19 列，改填「新檔 N 行」。
   - 第 3、13、35、42 列另加註「CRLF→LF 整檔替換，非語意修改量」。
2. **「來源與重建方法」**：
   - 刪掉錯誤的「ready 目錄＋overlay key」路徑公式，改寫成透過 contract（SHA256 `A5EAFAB9…31A97`）的 `effective_overlay_paths` 取檔，並記錄 44 檔分散在 25 個目錄、只有 6 個在 ready 目錄。
   - 「差異行數」段改寫為 #41 的結論：44／44 SHA256 一致；25 檔合計 +8,050／−3,250；19 個新檔共 3,343 行；以及 CRLF 的注意事項。
   - 一覽表上方的候選路徑說明，也改成以 contract 為準。
3. **「裁定」一節**：
   - 重新評估條件更新為：使用者已同意恢復測試（#43），等 M1 結果。
   - 靜態準備改成「已完成」，並寫入依賴圖結論：
     - 497 條引用、171 組檔案依賴；
     - 三組之間的跨組依賴數字；
     - **23 檔強連通分量**，含 #97 的第 29、30 列；另有 3 檔強連通分量（第 7、8、20 列）；
     - 直接依賴 #97 的列：第 2、3、6、12、15、35 列。
   - 新增「裁定補充」：不改寫就不存在可逐批套用的切法；未來只能改寫成依賴 `src/` 上新的 #97 實作（逐檔 PR），或只當參考、重新實作。
   - 刪掉「待依賴分析」的舊研究分組表。
   - 「對 T020 的影響」同步改為 T010 已在本機執行中。
4. **「未驗證」一節**：候選 SHA、diff、stamp 這兩項改為已由 #41 完成（刪除線），另外加上依賴圖只是來源層級分析的界線。

**裁定本身不變**：現在不套用，凍結為參考。

## 怎麼驗證

1. 一次性腳本 `scratch/merge41.py` 把 #41 報告的 44 列解析後寫入一覽表，輸出為：`parsed #41 rows: 44 all 一致: True cand==exp: True`、`rows updated: 44 same-name mismatches: []`。最後一項代表一覽表「正常來源有同名檔」欄和 #41 的同名檔判定 44 列全部相符。
2. 加總核對：同名檔 25 檔 +8050／−3250，新檔 19 檔 3343 行，和 #41 報告的總計完全一致。
3. 可重跑的比對（腳本全文附在下方，存成 `scratch/check46.py`，在 repo 根目錄執行）：

   ```
   $ python3 -I scratch/check46.py
   #41 rows=44; CANDIDATES rows checked=44; match=44; mismatch=0
   ```

   ```python
   import re
   rep=open('docs/reports/issue-41.md',encoding='utf-8').read()
   r41={m.group(1):(m.group(2),m.group(3),m.group(4),m.group(5)) for m in re.finditer(r'^\| \d+ \| `([^`]+)` \| `[^`]+` \| `[0-9A-F]{64}` \| `[0-9A-F]{64}` \| \S+ \| (無|`[0-9A-F]{64}`) \| ([^|]+?) \| ([^|]+?) \| ([^|]+?) \|$',rep,re.M)}
   cand=open('docs/CANDIDATES.md',encoding='utf-8').read()
   ok=bad=0
   for m in re.finditer(r'^\| (\d+) \| `([^`]+)` \|(?:[^|]*\|){3} ([^|]+) \|',cand,re.M):
       k,cell=m.group(2),m.group(3).strip(); src,a,d,n=r41[k]
       exp=f"+{a}／−{d}" if src!='無' else f"新檔 {n} 行"
       if cell.startswith(exp): ok+=1
       else: bad+=1; print('MISMATCH',m.group(1),k,cell,exp)
   print(f"#41 rows={len(r41)}; CANDIDATES rows checked={ok+bad}; match={ok}; mismatch={bad}")
   ```

4. 依賴結論逐項對照 #41「依賴圖與研究分組」一節：矩陣數字、兩個強連通分量的成員、直接依賴 G3 的 6 個檔案。強連通分量的成員以 #41 列出的檔名換算成本表的列號。

## 未驗證什麼

- 依賴圖只是來源層級：沒有展開巨集，沒有驗證 cfg 可達性或名稱解析，也沒有編譯任何分組（#41 的界線）。
- 候選內容本身仍不在 repo，本任務只搬運 #41 的實算結果，沒有獨立重算 SHA 或 diff。
- 沒有建置、沒有測試。

## 產出路徑

- `docs/CANDIDATES.md`
- `docs/reports/issue-46.md`（本檔）

## 給下一棒的提示

- 等 M1（#43 拆出的 T010-A～E、T011-A）結果與 T020（#97）完成後，再依「裁定補充」決定改寫或重做；在那之前，不要開候選套用任務。
