# 特效與地形、角色共用分片排序

現行世界畫面將普通箭66、光箭167／命中219、技能221／744、物品189／194／197、生命之泉2231及火牆168送進地形、角色、附件、陰影與屋頂共用的半格深度佇列。這些已限定的特效不再於整張場景完成後疊圖。

`native_world_effects::Target` 接收借用的 SPR frame、世界座標與明確 Over／Screen 混色。取樣仍由原有各特效控制器負責，每幀只取樣一次；投射物到達、受擊呈現、terminal dwell、聲音去重及場次世代清理保持各自的規則。保留 Canvas sink 供既有平面資源／時序測試使用。Queue 不複製整張像素，不持有跨幀資源；限制512個特效提交，總分片沿用262,144上限。普通箭資源現在保留 compact SPR 分片，記憶體上限包含 flat 與 compact retained bytes。

特效使用既有來源／路徑取樣的世界整格，轉成半格原點，逐片使用來源座標及depth。沒有新增角色跟隨、HP／MP修改或伺服器持續時間推測。相同key的跨類型順序明確採新引擎 profile／active snapshot順序，未重建原版timer插入歷史。特效維持穿透點選：Screen及普通箭Over均保留底層owner；這是延續現行客戶端的操作政策，未宣稱原版選取行為等價。

唯讀固定PE `4271D92F…B6E9` 的Capstone直接解碼見 `驗證紀錄/current/native-world-effects/native-effect-insertion-raw.json`。Class0的`44F170`呼叫`4E0200`；後者讀`+19` subframe、`+2c/+30`座標及`+1a`旗標，呼叫`4CD9D0`。`4CDA3C`之後加上原始分片座標，`4CDA93`讀depth，`4CDAA7`進入`483770`的排序清單。來源比對未執行原始BIN。先前普通箭／光箭路徑與Screen旗標的限定证据仍適用；完整動態旗標及RGB565 framebuffer等價尚未成立。

新增測試涵蓋十種實際SPR的逐像素與owner參照、Canvas／Fragments兩個sink的時鐘與事件一致性、七種固定特效的Model提交、重複取樣不重播聲音、場次清理、取消、缺少分片與512提交上限、普通箭全部八個方向的分片載入。獨立scalar參照自行排序、投影、混色與建立owner表，但共用SPR／TIL解碼器，因此不是decoder-independent驗收。

最終實際數量、EXE／正式JAR雜湊與測試結果以同目錄 `summary.json`、`effects-receipt.json` 和現行build receipt為準。隔離TCP逐幀核對提交數及來源；正向遮擋另以實際資源scalar測試驗證。未進行GUI操作、音效裝置、完整原版渲染或完整可玩遊戲驗收。

重跑：`pwsh -NoProfile -File 驗證紀錄/current/native-world-effects/run-final.ps1`，成功後執行同目錄 `finalize-receipt.py` 及 `final-state-check.ps1`。所有生成物僅寫本重建版。

#339（2026-11-13）補充排序的界線：原版的特效也是 actor，走同一個 `4E0200`（本體 → 附件 → 陰影）與 `483770`（同 depth 時新項目插在前面、先畫）。同一格、同 depth 的兩個 actor 誰在上面，取決於誰最後換影格，沒有固定答案；現行固定為「特效排在本體名單之後提交」，所以同 depth 時特效畫在本體與附件底下，depth 不同時照來源的 depth byte。這個策略由 `native_world_layer_order_tests::effects_share_the_depth_queue_and_stay_click_through` 固定下來，不是原版時序的還原。帶附件的特效（Class1、土牢）沿用 `native_actor_layers` 的附件字，附件字 1 用陰影同一支半透明寫入器，`0x80` 用 multiply。

十張最終離線圖採開放地面與樹木／圍牆交替位置。獨立coverage probe以相同原始分片／深度，分別啟用與停用地形；五個正向遮擋案例為66／219／744／194／2231，分別有13／143／7／2995／1463個特效coverage像素被後繪地形遮住。這項判定不依賴Screen混色後RGB是否改變，另保留舊flat overlay差異作歷史對照。
