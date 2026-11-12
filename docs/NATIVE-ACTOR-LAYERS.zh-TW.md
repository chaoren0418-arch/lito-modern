# 原生角色陰影與附件

本階段將 list.spr 的 101 陰影與 105 附件接入現行地圖／角色共用分片深度佇列。範圍是中性角色繪製旗標；完整角色狀態旗標、屋頂 runtime、特效交錯、原版插入時序與實際 GUI 尚未驗收。

## 來源與行為

唯讀原始資源位於 `D:/lineage開發/0-客戶端`，原版程式未執行。`驗證紀錄/current/native-actor-layers/native-submission-raw.json` 保存固定 PE SHA256 與獨立 Capstone 位元組解碼；入口 0x4E0200 提交本體、0x4E7000 附件，再提交陰影。三者共用 actor 座標及 signed subframe。0x4E6EA0 按本體 image 選附件 bank；任何附件指標缺失會清空該組數量。陰影提交前另檢查指標。

層定義從 logical sprite 解析，不從 dummy graphics 反查：例如 logical 0 使用本體 3225，陰影 3226、附件 6262；3225 dummy 本身沒有這些屬性。附件不建立自己的 action clock，不觸發自己的聲音或遊戲事件。每次都沿用本體已選取的 image、SPR subframe 及 display center。

`Bank` 以 member ASCII 位元組總和 `& 15` 選 Sprite00～15，只有選定 shard 沒有 entry 才回 Sprite。actual MissingEntry 可略過整個附件組或單一陰影，並在 receipt 記錄 `missing_attachments`／`missing_shadow`；解碼錯誤、缺少 subframe、取消或不支援的旗標不當成正常缺件。實際 727 資源有缺少的 383 附件與 262 陰影 bank；部分 action 對應的本體 bank 也不存在。覆蓋掃描將本體缺件另外列為 missing_bodies，不計入成功樣本；runtime 本體錯誤仍明確報錯，未宣稱支援這些缺件姿勢。

## 合成與上限

本體、附件、陰影依來源提交順序進入同一 fragment queue，維持半格 row／column／depth／反向 insertion key。中性附件 attr104=0 使用 Over，attr104=8 使用 Screen；陰影使用既有 Shadow。Screen／Shadow 保留既有像素 owner，因此陰影不會把地面擴成可點選角色。附件自己的繪製字、0x40／0x100 的繼承與隱藏分支在 #339 補上，見文末「附件與陰影的繪製字」。

本體與各層共用最多 1024 members、64 MiB 解碼快取，包含 flat RGBA 與 compact fragments；每個 SPR 最多 8 MiB 輸入、4M 像素。選定 archive 索引合計最多 100 萬紀錄，最多 8 個附件；讀取及合成保留取消檢查。

Scene 直接提交已按深度整理的 terrain，攜帶原始 source sequence；移除每影格先回排 source、再由 raster 排深度的冗餘排序。新舊八個 terrain/body 靜態樣本的完整 RGBA 必須完全相同，像素 owner 仍逐點比對。

## 驗證與限制

`native_actor_layers` 五項測試涵蓋 logical alias、非法屬性、取消、缺件、方向／動作 bank 綁定、六組真實資源合成，以及 live scene 移除角色後無殘留。獨立 scalar compositor 直接使用 SPR 解出的 fragment RGBA，另算位置、排序及混色；不呼叫現行 Raster、compact atlas 或 blit。共用 SPR decoder，故不宣稱 decoder-independent。Screen 使用獨立等價式，Shadow 使用 RGB555 分量平均；不宣稱與原生 RGB565 framebuffer 等價。

六張新樣本均逐像素比對 RGBA 與命中表；另保留八張既有 terrain/body 比對和六張 inventory icons。另以 Python 直接解析原始 old28 index，重新驗證所有覆蓋與 runtime SPR 的選定 shard／base、位元組數及 SHA256；20 個世界回歸案例檢查附件／陰影和本體 image／subframe 一致。

最終 gate 是同一 EXE 的 161 Rust／30 組、52 隔離 TCP、20 個靜態像素比對，發布結果以 `驗證紀錄/current/native-actor-layers/summary.json`、`actor-layers-receipt.json`、`final-state-check.json` 為準。正式 JAR 原始位置與本目錄 snapshot 必須始終一致；測試不接玩家實表。

Debug 導航效能與 schema28 留存值並列。新結果多做陰影／附件合成，因此不能把差值當成只量測排序優化；Release 只有 compile check。圖片檢視不等於 GUI、IME、DPI、音效裝置或玩家驗收。整體客戶端仍在開發。

## 附件與陰影的繪製字（2026-11-13，#339）

來源是同一個 unpacked 映像（SHA256 `4271D92F…5C2AB6E9`）的靜態反組譯，沒有執行原版。位址與重現指令在 `docs/reports/issue-339.md`。

- **附件自己的字**：`4E6EA0` 把每個附件 sprite 自己的 list.spr 屬性 104（表 `647E68`）存到 `actor+B0+2i`。`4E7000` 提交附件時用的是這個字，不是本體的字。
- **附件只繼承三種本體狀態**（`4E7000`，本體字在 `actor+1A`）：bit 2（隱藏）整個不提交；`0x40`（別人的隱身）加上 `0x40`；`0x100`（自己的隱身）加上 1。中毒綠（`0x20`）、變暗（`0x200`）、本體目錄旗標 1 都不會傳給附件。
- **寫入器的選擇**（`488DAD`）：先看 8，再看 `0x101`，再看 `0x80`。所以附件字 0＝一般、1＝半透明、8＝screen、`0x80`＝multiply；本體半透明時 `8|1` 仍是 screen，`0x80|1` 變成半透明。
- **陰影**（`4E0233`／`4E7F03`）：提交的字是 `(actor+1A & 0x0FFF) | 5`。陰影 sprite 自己的屬性 104 沒有任何讀取點。
- **提交順序**：本體 → 附件依表內順序 → 陰影。`483770` 把新項目插在同 depth 的舊項目前面，所以同 depth 時陰影最先畫、附件由後往前、本體最後。

現行實作對應：

- `native_actor_layers::Mix` 增加 `Half`（附件字 1）與 `Multiply`（附件字 `0x80`）；`Mix::submitted` 表達「本體半透明時附件的寫入器」。附件字 12、`0x1008`、`0x2001` 等沒有解出寫入器行為的值仍然拒絕（`unsupported layer flags`），不用近似值代替。
- 陰影 sprite 的屬性 104 不再檢查。
- 共用 list.spr（SHA256 `523C61AD…E8B9F127`）有 3598 列帶陰影或附件；layer plan 接受的列從 3036 增加到 3225。新接受的 189 列以前會回報 `layer_error`，連陰影一起不畫。清單與統計在 `docs/reports/issue-339/list-spr-layers.json`。
- 仍被拒絕的 373 列裡有 371 列是因為**本體**的屬性 104 不是 0 或 1（多數是 8 或 128 的特效型物件）。這類本體目前以一般方式繪製並回報 `layer_error`，沒有在 #339 處理。

不需要客戶端資源的測試：`native_actor_layers::tests`（3 個）與 `native_world_layer_order_tests`（3 個）。後者用合成場景走現行分片 raster，核對同 depth 的繪製順序、四種附件寫入器在一般／半透明／綠／暗／目錄旗標 1／隱藏六種本體狀態下的像素，以及特效與本體、附件的相對順序。
