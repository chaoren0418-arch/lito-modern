# Issue #21 / T004-5：PROTOCOL-CONTRACT 補上 S_LITO_GAME_INFO 0xD5

## 做了什麼

- `docs/PROTOCOL-CONTRACT.md` §5.3：在 `S_LITO_DELETE_ROOM` 之後新增 9 列。
  - `S_LITO_GAME_INFO 0xD5` 的共通開頭。
  - subtype 0 摘要、1 設定、2 模式、3 地圖、4／6／7／8／9 摘要純量、5 房主，每列都寫出欄位版型、值域、builder 方法與 `P:` 行號。
  - 兩列「發送時機」（第 2 次嘗試改寫）：一般房間依請求類型分成 A 改設定、B 換模式、C 換地圖、D 房主轉移；租裝城鎮依換地圖、換模式、改設定分列。每條成功或拒絕路徑都寫出收件者（單播或廣播）、送哪些 subtype，以及分支層級的 `CS:` 行號。
  - 對應的請求 `C_LITO_GAME_INFO 0xD1`：subtype 2 換模式方向、subtype 3 換地圖。
- `docs/NATIVE-ROOM.zh-TW.md`：只改「S_CF / S_D5 subtype 5」那一列的欄位定義，改成連到 §5.3，取代 #15 留下的「契約尚未涵蓋」。
- `維護工具/check-protocol-contract-citations.py`：第 1 次新增 23 條方法層級的引用；第 2 次再新增 21 條分支層級的引用（實際送出那一行、條件判斷行、GameHub.updateMap 的拒絕條件）。

### 證據等級

- **來源**：全部欄位都直接讀自 lito-launcher-server `2066774` 的 builder 與 parser 原碼，包括：
  - `litoRoomSummary`／`litoRoomConfiguration`／`litoRoomMode`／`litoRoomMap`／`litoRoomSummaryScalar`／`litoRoomOwner`
  - `litoGameInfoRequest`
- **推測**：subtype 1 的 option1..4 是否依序對應 eventFlags／eventType／winnerCount／allowedClassMask。builder 驗證的是後四個欄位，寫出的卻是 option1..4，兩者的關係在 `GameHub.RoomSettings` 裡，沒有逐一核對。契約已在該列標為推測。
- **伺服器原碼註解的說法**：subtype 4／6／7／8／9 在 client 映像中沒有讀取者，伺服器也沒有發送點；契約寫的是「解析後忽略」。
- **未驗證**：沒有對正式 JAR e1498871 核對這些 builder；#13 只核對了 opcode 常數，`S_LITO_GAME_INFO = 0xD5` 不在它的清單內。

### 第 2 次嘗試的修正（對應 codex-local 審查）

1. **換地圖有兩條拒絕路徑**：地圖不屬於目前模式或等於 0xFFFF 時，只單播原本的 subtype 3（CS:16540–16546）；`hub.updateMap` 被拒時（非房主、已開戰、地圖不合法，GameHub.java:9874–9878），單播的是 **subtype 0＋1**（CS:16549–16551）。契約已分開寫，並提醒 client 不能只等 subtype 3。
2. **租裝城鎮依請求類型分列，全部是單播**：換地圖不論合法與否都回 3（CS:16435）；換模式回 2＋3（CS:16443 → 16446）；改設定一律拒絕並回 2＋3（CS:16725–16730）。
3. 換模式的拒絕路徑也一併寫清楚：模式不是戰鬥場景時單播 0＋1（CS:16486–16492）；`updateSettings` 被拒時單播 0＋1（CS:16503–16505）。
4. 核對腳本新增分支層級的檢查項，例如 CS:16549、16550、16435、16443、16729、16735、16757、16760，以及 GameHub:9874、9876。

## 怎麼驗證

本輪在雲端工作環境實際執行，伺服器 clone 位於 `2066774`。

```
$ python3 -I 維護工具/check-protocol-contract-citations.py /home/user/lito-launcher-server; echo "exit=$?"
source HEAD 2066774824b314b2c65834f9afe72b68f7391f06; 184/184 citations OK; spellSelector(payload) parsers=11
exit=0
```

（第 2 次嘗試的輸出）引用數從 140 條增加到 184 條：第 1 次新增 23 條、第 2 次新增 21 條，全部通過。

NATIVE-ROOM 的連結錨點用 #15 報告附的錨點腳本重跑，五份 NATIVE 文件加上契約：`links checked; broken = 0`。

差異範圍：`docs/PROTOCOL-CONTRACT.md`（+9 列）、`docs/NATIVE-ROOM.zh-TW.md`（1 列）、`維護工具/check-protocol-contract-citations.py`、本報告。

## 未驗證什麼

- 上面提到的 option1..4 對應、沒有對 JAR 核對 builder。
- 重建版 client 對 subtype 0／1／2／3 的實際處理；NATIVE-ROOM 目前只記載 subtype 5。
- 沒有跑測試、沒有建置、沒有啟動遊戲。

## 產出路徑

- `docs/PROTOCOL-CONTRACT.md`
- `docs/NATIVE-ROOM.zh-TW.md`
- `維護工具/check-protocol-contract-citations.py`
- `docs/reports/issue-21.md`（本檔）

## 給下一棒的提示

- client 要支援房主修改模式或地圖時，依「發送時機」這一列：被拒時伺服器會回送原本的狀態，所以 client 不要先改畫面，要等回覆再更新。
