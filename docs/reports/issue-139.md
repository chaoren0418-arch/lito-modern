# #139 T025（M2 1/1）：版本綁定、§8 對齊、180／181 來源、伺服器換版重核清單

## 做了什麼（只改 `docs/PROTOCOL-CONTRACT.md`）
- **§0 版本綁定**：寫明 M2、M3 的所有客戶端開發與驗收，都以 E149（`e1498871…d498`）為唯一伺服器對象；隔離候選與 Maven 產物都不是驗收對象。另加 E149 的血統：0.6.70 `daedec7c…` → 0.6.71 `e1498871…`，變動 4 個 entry，`litoddraw.dll` 兩版都是 `e47bd6c6…`。
- **§8 對齊**：原本寫「§1–§7 都還沒有對 `e1498871` 核對」，和 T004-1（#13）已完成的事實矛盾，改寫為三類：
  - 已驗證（靜態）：72 個 opcode 與各常數一致。
  - 仍待驗證：沒有動態驗證；§6.3 讀的是 `src/main`，但 catalog 常數與 E149 相同。
  - 已釐清：NumericBreakthrough 每次呼叫都重新讀取。
- **§6.3 的 181／180**：181 項＝180 個 catalog 記錄＋TELEPORT（skill 5，dispatcher 特例），並引用產生器的 `:85`、`:192` 兩行。
- **§9 伺服器換版重核清單**：14 項，涵蓋 TASKS T025 列出的全部內容：
  - 血統分支與 manifest pin（#120 新 N3）；
  - `Protocol$*`、ClientSession／GameHub、技能與圖示 ID、房間與結算、移動與傳送、裝備與租賃、GM 指令、登入與來源伺服器；
  - 相容 DLL 與 hook；
  - 11 項候選協定變更；
  - Maven 產物差異；
  - 資源 pin；
  - 回歸。

## 已驗證
- `python3 -I 維護工具/check-protocol-contract-citations.py <lito-launcher-server>`，輸出原樣如下，結束碼 0：

```
source HEAD 2066774824b314b2c65834f9afe72b68f7391f06; 184/184 citations OK; spellSelector(payload) parsers=11
```

- E149 的血統與 DLL SHA 取自登入器 repo `docs/maintenance/release-0.6.70.json`、`release-0.6.71.json`（`2066774`）。`build` 欄位是 base `daedec7c…`、candidate `e1498871…`、changed_entries 4 個。
- 產生器 `:85`（`for m in re.finditer(r"new (\w+Spell)\(", cat)`）與 `:192`（`rows.append((5, "TELEPORT", …`）的內容已核對。
- §8 的 T004-1 結論引自 `docs/reports/issue-13.md` 第 7–9 行。

## 推測
- 「歷史紀錄的 180 項目錄＝catalog 記錄數」：依 HISTORY L49 的用語與產生器邏輯推得。

## 待驗證
- §9 是清單，本張沒有執行換版重核（目前沒有新正式版）。
- §1–§7 的動態驗證仍未做（屬各功能任務的範圍）。
