# Issue #15 / T004-3：NATIVE-PROTOCOL／LOGIN／GAME／LOBBY／ROOM 改指向 PROTOCOL-CONTRACT

## 做了什麼

- 五份文件開頭都加上同一句：「協定欄位以 PROTOCOL-CONTRACT.md 為準（框架、欄位、opcode、錯誤碼）。本文只記錄重建版 client 的處理方式、上限與驗證證據；……」。
- 五份文件的封包表統一改成三欄：「封包｜欄位定義｜重建版 client 處理」：
  - 「欄位定義」欄直接連到 `docs/PROTOCOL-CONTRACT.md` 的對應章節。
  - 原本寫在表中的 wire 版型（bytes 長度、`ccdsch`、欄位順序、u16／u32 等）全部刪除。
  - 「client 處理」欄只保留重建版自己的行為：上限、等待條件、拒絕條件、暫停期限等。
- 各文件獨有的驗證證據、205DC1BA 快照說明、重跑指令、程序生命週期段落全部保留，沒有改動。
- NATIVE-PROTOCOL 表下方那段記載 205DC1BA 成功回覆數值（revision `0x0734fd33` 等）的文字，屬於快照證據，予以保留。
- 保留原檔的換行格式：NATIVE-PROTOCOL、LOGIN、GAME 原本是 CRLF，LOBBY、ROOM 原本是 LF。
- 沒有改 PROTOCOL-CONTRACT.md 與 docs/REPORT.md。

| 文件 | 改動的表 | 連到的契約章節 |
|---|---|---|
| NATIVE-PROTOCOL | 「封包與欄位」5 列 | §2、§4.1 |
| NATIVE-LOGIN | 「已實作的正式欄位」7 列 | §4.2、§4.4 |
| NATIVE-GAME | 「正式封包」7 列 | §4.3、§5.1、§1 |
| NATIVE-LOBBY | 「協定與狀態」7 列 | §5.2、§5.3、§7（「房間更新結束」「切換頻道」是 client 流程，欄位定義欄填 —） |
| NATIVE-ROOM | 「權威資料與邊界」6 列 | §5.3、§6.1 |

## 怎麼驗證

本輪在雲端工作環境實際執行。

1. 連結錨點核對：一次性腳本 `scratch/anchors.py`（不進 git）依 GitHub 的錨點規則產生契約所有標題的錨點，再逐一檢查五份文件裡每個 `PROTOCOL-CONTRACT.md#…` 連結都能對到標題。

   ```
   $ python3 -I scratch/anchors.py docs/PROTOCOL-CONTRACT.md docs/NATIVE-PROTOCOL.zh-TW.md docs/NATIVE-LOGIN.zh-TW.md docs/NATIVE-GAME.zh-TW.md docs/NATIVE-LOBBY.zh-TW.md docs/NATIVE-ROOM.zh-TW.md
   links checked; broken = 0
   ```

   對照的是 main 上的契約，也就是 §6.3 合併之前的版本；所有連結都只指到 §1–§7 已存在的章節。

   本任務範圍只允許改五份文件和本報告，所以腳本不進 repo，全文附在這裡，可以存成 `scratch/anchors.py` 後重跑：

   ```python
   import re,sys,unicodedata
   def anchor(h):
       h=h.strip().lower()
       out=''.join(ch for ch in h if ch in ' -_' or unicodedata.category(ch)[0] in 'LMN')
       return out.replace(' ','-')
   heads={}
   for l in open(sys.argv[1],encoding='utf-8'):
       m=re.match(r'^(#{1,6}) (.*)',l)
       if m: heads[anchor(m.group(2))]=m.group(2).strip()
   if len(sys.argv)>2:
       bad=0
       for f in sys.argv[2:]:
           for a in re.findall(r'PROTOCOL-CONTRACT\.md#([^)\s]+)',open(f,encoding='utf-8').read()):
               if a not in heads: print('BROKEN',f,a); bad+=1
       print('links checked; broken =',bad); sys.exit(1 if bad else 0)
   for a,h in heads.items(): print(a,'<=',h)
   ```

2. 差異範圍：`git diff --stat origin/main...task/15` 只有五份 NATIVE 文件加上本報告。忽略行尾差異後，五份文件共 +52／−42 行，只動到表格與開頭那一句。

## 未驗證什麼

- 錨點規則是依 GitHub 的規則自行實作，沒有在 GitHub 網頁上逐一點擊確認。中文標題的錨點如果 GitHub 有特殊處理，可能需要微調。
- 發現契約有一個缺口，沒有在本任務處理，因為範圍禁止修改契約：NATIVE-ROOM 提到的 `S_D5 subtype 5` 是 `S_LITO_GAME_INFO 0xD5`，伺服器有 subtype 0／1／2／3／5（Protocol.java @2066774 的 16959、17033、17079、17107、17151、17171 行），但 PROTOCOL-CONTRACT 目前沒有這個封包。NATIVE-ROOM 的該列已標為「契約尚未涵蓋」。建議另開設計任務，把它補進契約 §5.3。
- 沒有跑測試、沒有建置。

## 產出路徑

- `docs/NATIVE-PROTOCOL.zh-TW.md`、`docs/NATIVE-LOGIN.zh-TW.md`、`docs/NATIVE-GAME.zh-TW.md`、`docs/NATIVE-LOBBY.zh-TW.md`、`docs/NATIVE-ROOM.zh-TW.md`
- `docs/reports/issue-15.md`（本檔）

## 給下一棒的提示

- 契約補上 `S_LITO_GAME_INFO 0xD5` 之後，把 NATIVE-ROOM 那一列改成連結。
- 之後新增 NATIVE-* 文件時，請照同樣的格式：欄位只連到契約，不要重抄 wire 版型。
