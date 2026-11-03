# 原生地圖／角色分片遮擋與像素選取

現行 `native_world_scene` 已將地圖與角色／地面物品的原始 SPR 分片放進同一個半格繪圖佇列。先前完整地形快取後再疊整張角色的路徑退出現行世界繪製；`native_terrain_cache` 仍保留其既有獨立測試，沒有以它承擔動態遮擋。

## 來源與排序

`native_multisector_scene` 保存原始 terrain 座標、depth、來源插入序號，以及有界的 PreparedTile。每幀只取可見掃描列，依來源順序排入 `world_raster`。角色使用 `decode_world_spr_with_limits` 保存共享 compact atlas 與原始 SPR 的 signed half-X、Y、depth、block ID；快取計量包含展平資料及共享 atlas，總上限 64 MiB。prepared terrain 另有 64 MiB 上限。

排序採半格掃描 row／column、depth，以及反向 insertion。地板 depth 為 -1；同深度後插入先畫。SPR 分片的原點是整格中心左 24、上 12 pixels，保留原始半格 parity，不能把展平圖片任意重排後宣稱等價。地圖依 canonical sector／來源順序，角色依現行投影 roster 插入；跨角色順序是明示的新引擎 snapshot 策略，尚未重建原版 timer 的插入歷史。

本輪從固定原生 PE 位元組重新解碼並保存 `native-depth-raw.json`：

- `0x44014B` 讀 signed 分片座標、depth byte 與 block ID。
- `0x4CDA3C..0x4CDAA7` 加上 anchor，將 block 及 depth 傳入 `0x483770` 的半格清單。
- `0x483815..0x483828` 比較新 depth，`jle` 在相同 depth 時插到既有項目前面。
- `0x488C34..0x488F2B` 每格先處理地板再走清單，地板 bit8 分支略過；後續掃描步進仍保存於同份原始解碼。

固定 PE SHA256 是 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`。沒有執行原始 BIN，也沒有把反組譯文字本身當作執行證明。

移動時，新引擎從已取樣的投影位置選最近整格作 depth anchor，再把其分片平移剩餘像素。這使呈現維持連續插值；深度格在邊界切換。這是明示的新引擎規則，不宣稱已核對原版移動中細分座標／更新時序。

## 遮擋與命中

每幀同一合成器產生 RGBA 與 800×600×2 bytes 的共享 owner 表，最多 128 個角色物件。透明像素不取得 owner；不透明地形會清除底下角色的 owner，部分透明地形保留底下可選物件。Frame 的矩形只保留幾何資訊，`pick` 改查實際 terrain/body owner。

命中仍要求目前已畫出的 Frame、viewport、visual revision 與自己 incarnation 相符。自己遮住的物品不會因為排除自己而穿透選取。點到可見地面物品時，經 `Choice::at` 與既有 owner／generation／correction／完整 Actor／三格距離檢查後開啟數量面板；點選本身不送拾取封包。按拾取時再次檢查目前 Frame 與權威狀態。GUI／IME／DPI／焦點的實際人工操作尚未驗收。

特效目前仍在 terrain/body 合成後繪製，不改動這張 owner 表。完整特效交錯、角色陰影／附件，以及屋頂 runtime 不在已完成範圍；收據仍標示完整 actor／terrain／effect 等價未完成。

## 證據與回歸

新增 `native_world_layers` 整合測試使用真實地圖與 `3225-0.spr`／`773-0.spr`：

- 八組位置／插值取樣，完整 RGBA 及每個像素的 owner 均對照獨立 scalar 合成器。參照不呼叫 NativeDrawKey、NativeDrawCell 投影、Raster、Canvas blit 或 compact atlas；它共用已限定的來源解碼器，因此**不是獨立解碼器驗收**。
- 真實牆面案例同時保留可見角色像素及被地形遮住的角色像素；另以不畫地形的參照核對，沒有把透明洞誤算成牆面遮擋。
- 同深度重疊遵循反向 insertion，透明洞不取得命中；取消、尺寸／物件數上限、重複 ID、缺少 qualified fragments 均拒絕。
- 世界／Frame 世代失效、透明地面區域及同 ID 重建均不能沿用選取。

`verify-native-world-layers.py` 核對八張 PNG 的 RGBA、原始 SPR bytes／SHA256、PE 身份、現行 TCP frame 的合成旗標与記憶體計量。完整 TCP 仍覆蓋先前所有 52 情境。原有 18 動畫與 1 導航畫面 hash 保存在 schema27 收據，現在逐項記錄刻意變更，沒有偷偷改寫 golden；本階段比較數是 8 個新分片參照加 6 個未變的背包圖示，不再宣稱 25 個舊畫面全相同。

最終測試計數、EXE／JAR 雜湊及來源 pins 以 `驗證紀錄/current/native-world-layers/summary.json` 與 `驗證紀錄/20261012/current-build-receipt.json` 為準。`run-final.ps1` 通過所有測試、Debug build、Release compile check、TCP 與稽核後才可發布 schema28；沒有 Release 執行檔或音效裝置的完整驗收宣告。

所有寫入限於重建版；正式 JAR、原始資源、玩家資料與現行正式程序保持唯讀邊界。完整客戶端目標仍在進行。
