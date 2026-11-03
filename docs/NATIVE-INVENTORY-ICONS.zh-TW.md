# 原生背包基礎圖示與可見列載入

原生背包以伺服器提供的 graphic ID 直接載入原始 TBT／LBT。Win32 背包列現在包含 32×32 圖格，名稱與權威數量維持原值。圖示 ID 不等於物品 catalog ID，不由圖案推算耗材、裝備或數量。

## 原版證據與來源

只讀 recovered PE SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9` 的當前 bytes 經獨立 Capstone 解碼。IDA 清單僅用於定位，沒有執行原版 BIN。

- 442860 物品建構子保存 icon word 於 +0x6E；442A30 繪圖分支以 signed word 判讀，20000 起查 LT 表並扣除 20000。
- 普通圖示以十進位 `%d.tbt` 及 ASCII 檔名字元和 modulo16 查 Sprite 分片。分片明確缺席後，原 42EA50 的 BASE Sprite 入口先查 LTicon，再查 BASE Sprite。
- 42EF22 綁定 `LiTo\LTicon`，不能用 LTsprite 代替。LTlist.ico 是 `233\r\n`，當前 LTicon archive 實際含 226 個 LBT。
- 4B9980 呼叫 4BE000；後者以 signed byte 讀 X／Y 偏移及高度。跨度內的字全部複製，包含 RGB555 的零值，因此「已覆蓋黑色」不透明，只有未覆蓋區域透明。
- 4B9E80 的圖格尺寸參數為 0x20。現代呈現將 signed 偏移套入 32×32 cell 並裁切越界部分；這不宣稱複製舊版全部 UI 佈局。

證據位於 `驗證紀錄/current/native-inventory-icons/native-draw-evidence.json`、`lt-icon-index.json` 及上一階段的 `native-inventory-names/icon-native-next.json`。有界指令切片可能包含相鄰函式，判讀以上述實際入口／呼叫為界。

## 解碼與失敗處理

native_inventory_icons 重用嚴格的跨度 parser，但不使用歷史預覽工具的黑色透明規則。RGB555 以 5→8 bit 複製擴展；covered pixels 的 alpha 為 255。偏移保持 i8，高度大於 127 的原版 signed 不支援範圍明確拒絕；wire icon 仍保存 u16，圖形查詢對大於 32767 的值拒絕，不做負索引或猜測替代圖。

NativeIconArchive 為唯讀 LTicon.idx／pak handle；索引與 pak 採來源 pin，驗證 old28 項目、偏移及長度。只有經驗證的明確 member miss 可進入下一 namespace。損毀或缺失 archive、過大資料及不支援格式回傳錯誤，不當成合法 miss。LT0.lbt 是有效的全透明圖，沒有因為畫面空白而改用其他圖示。

## 工作生命週期與 UI

Service 固定 resource root，單一背景 worker 只接收 intrinsic graphic ID，不捕捉角色、物件或列索引。UI 依 viewport 取得可見項及少量預取；Service 最多保存 64 項，每張只留 4096 bytes 的 cell 及來源 metadata。可見集合改變取消舊工作，快取僅保留目前需要的 ID。載入失敗有界保存，避免每次 timer 重試；取消中的讀取不會留下取消錯誤。

繪圖時重新比對目前 inventory owner，從當前物件查 graphic ID，然後取相應 cell，避免列重排或改圖時沿用舊物件關係。系統選取／背景色與文字保留。關閉背包清除可見快取；關窗取消並等待 worker 回收。OS 同步讀取有界且前後检查取消，不宣稱能強制中斷任意磁碟 read。

## 驗證範圍

7 項 Rust 測試涵蓋覆蓋黑色、signed 偏移／裁切、截斷及不支援資料、原始六樣本及所有 226 張 LT 圖格、可見集合切換與關閉、有界快取／錯誤不重試，以及 primary 損毀不能 fallback。

Python 直接解析原始 old28 索引及 selected PAK spans，獨立算出 RGBA，先釘選 fixture 再與 Rust 結果比對。六個樣本為 0、80、235、20000、20001、20147；全部 226 張 LBT 另以 ID 與 cell SHA256 的有序彙總核對。`icon-contact-sheet.png` 只呈現這六個資源樣本，並非 GUI 截圖。

正式隔離 inventory／inventory-reselect 兩個既有 ClientSession 案例追加對圖示 80／235 的 object ID、graphic ID、來源雜湊、像素雜湊、偏移及 worker 回收核對。這兩案包含在 42 TCP 總數，不另加兩案。LT 圖示由原始資源與模型測試核對，沒有宣稱正式 GUI 遊戲操作已觸發全部 LT 圖示。

最終狀態、EXE、來源 pins 與計數以 `驗證紀錄/current/native-inventory-icons/summary.json` 為準；整體客戶端目標仍進行中。

## 尚未完成

目前僅基礎圖示。原版 identified／blessing 的祝福光暈、詛咒色盤等分支、物品使用／裝備／丟棄、GUI 捲動／焦點／DPI 人工驗收仍待完成。沒有把靜態原版證據或離線圖像對照宣稱為完整 UI 相容。
