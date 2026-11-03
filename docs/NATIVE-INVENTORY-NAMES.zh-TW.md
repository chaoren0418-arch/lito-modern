# 原始物品名稱與可回收字表載入

原生背包透過 `native_inventory_names` 顯示原始中文名稱。正式 ClientSession 的 `$26 (2)` 展開為「治癒藥水 (2)」，字面名稱 `Magic Gem (4)` 保持不變。Win32 視窗與 TCP 探針共用同一個 label 函式；物件 ID、原始名稱、數量、detail 與其他 wire 欄位不修改。

## 原版路徑證據

重新解碼只讀 recovered PE `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`：

- `47C96F` 將 `LTdesc` 載入 `834B80`；`47C986` 將 `desc` 載入 `834BB0`。
- `4D4450` 以 `%s-%c.tbl` 組合名稱；語言選擇分支在 950 時回傳 `c`。
- `4D3E40` 經 BASE Text 物件呼叫 `42EA50`。該入口對 `list.spr` 以外名稱先查 `647994` 的 `LiTo/LTText`，未找到才回到 BASE。
- `4D44A0` 先檢查主字表實際筆數，之後才把 20000 以上名稱代碼減去 20000，查 `834B80` 的 LiTo 字表。十進位數字後可帶分號。

`驗證紀錄/current/native-inventory-names/loader-evidence.json` 記錄當前 PE 的指令、字串及語言分支；先前 `native-inventory/name-helper-next.json` 記錄完整名稱 helper 至第一個 ret。IDA 清單只用於定位，判讀以重新解碼的當前 PE 為準，沒有執行原版 BIN。

## 實際字表

| 來源 | 解碼 bytes | 宣告／實際筆數 | SHA256 |
| --- | ---: | ---: | --- |
| `Text.pak/desc-c.tbl` | 249300 | 17001／16945 | `F06C94F47271FCB01F1D5AF63725DFA9A230B5F6BE3C0DD1D53CD37628ADB7DF` |
| `LiTo/LTtext.pak/LTdesc-c.tbl` | 2053 | 173／171 | `73243064A856FADE75DE110C5D25F6B1F19ED51C3BA7B1219BBE44ED7E84006B` |

主表在 LTtext 索引中明確缺席後才讀 Text；LTdesc 命中 LTtext。缺失或損壞的主查詢 archive 不能當成 index miss 而靜默改讀其他來源。輸入採 CP950、零起始行號，保留空行位置與宣告／實際筆數差異。不存在的尾端槽位不補值、不重新編號。

已核對 `$26`＝治癒藥水、`$20000`＝指導員-潘塔休、`$20026`＝咒術師法杖。共享表雜湊也符合既有獨立 C# 解碼 golden；本輪 probe 直接使用目前 Rust L1 cipher，LT 表的解碼／解析測試與目前資源相互核對，不宣稱新增獨立第二套解碼器。

## 載入與呈現

名稱 Service 綁定視窗的資源根目錄，單一背景 worker 載入兩表。它不捕捉角色或物品快照，因此換角色／連線不會帶入舊角色的列。主執行緒輪詢只回收已完成 worker；關窗時取消並丟棄遲到結果，等待 worker 回收才退出。底層同步檔案讀取仍使用有界資料與前後取消檢查，不宣稱可以中斷任意 OS read。

解析上限為 768 UTF-8 bytes 輸入、4096 bytes 輸出及 128 個引用，容納 wire 255 MS950 bytes 展開後的長度。只有完整名稱解析成功才使用翻譯；未知代碼、空字表項、溢位、控制字元或尚未實作的格式分支保留整個原始名稱。顯示時控制字元清理沿用背包邏輯，模型保留原值。

目前保留伺服器已附加的 `(數量)`，另顯示獨立權威 count。例如數量封包改變時，不會猜測名稱尾端哪一段數字可以刪除。名稱代碼不是物品 catalog ID，翻譯不能用來判定耗材或修改數量。

## 驗證與未完成範圍

六項新增 Rust 測試涵蓋主表／LT 偏移、字面前後綴與分號、UTF-8 長度擴張、不支援分支整筆回退、原物品不變、原始資源雜湊、背景載入的成功／失敗／取消及損壞 primary 不得 fallback。正式 inventory、inventory-reselect 兩個 ClientSession 案例核對同一 formatter 的名字、ID、數量、字表來源及 worker 回收；這兩案已包含於既有 42 TCP 總數，沒有重複計數。20000 偏移名稱目前由原始字表／模型測試驗證，不宣稱已由正式遊戲操作觸發。

此階段僅使用繁體中文 c 表。原版 runtime 模板替換、格式顏色、遞迴引用、其他語言選擇、物品圖示、使用／裝備／丟棄與 GUI 人工驗收仍待實作或驗證。最終測試結果、EXE 與來源 pins 見 `驗證紀錄/current/native-inventory-names/summary.json`；唯一現行入口不變，完整客戶端目標持續進行。
