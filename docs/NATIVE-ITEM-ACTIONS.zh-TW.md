# 原生物品操作、藥水與地面物件

本階段將使用／裝卸及指定數量丟棄接入現行 x64 原生視窗、共同控制器與正式 JAR 的隔離 ClientSession。完整道具系統與 GUI 操作仍在開發。

## 請求與權威

`native_item_actions::Choice` 保存 connection、selection、character 與完整 Item。視窗選擇的是已呈現的物件快照；發送前重新比對，不能把舊列的點擊套用至更新後物品、其他角色或新連線。使用 wire object ID，不從 icon 或 catalog ID 推算。

C1C 使用封包為 `c d s`，即 opcode、物件 ID、MS950 字串；普通使用的空字串仍需要結尾零。重新解碼原版 445BD0／445D96 分派後，先支援 use_type 0、1、2、11、18～25。需要目標或其他參數的分支會明確拒絕，不落入普通請求。原版 type3 是另一個帶 charge 的分支，不能混同普通藥水。

原版443FF0數量完成回呼及445900單件路徑均以508CC8的`chhdd`格式呼叫4C6AE0，確認opcode0C及item+4物件ID；數量回呼先核對非零且不超過item+0x74。C0C 丟棄封包為 `c h h d d`，本階段只丟在腳下，使用目前世界角色座標、背包物件 ID 及數量；C76 銷毀是另一個操作。UI 數量限制為正整數且不超過當前持有量。世界、角色生命週期、死亡／結算與移動限制也會阻止失效請求。

一次只允許一個物品請求等待更新。控制器不預測背包、裝備、HP 或地面物件；所選物品真正變更後顯示「已收到物品更新」。這是觀察到的資料變更，協定沒有可與請求對應的成功 ACK。兩秒沒有選中物品更新時顯示未收到回覆，保留手動重新操作；不自動重送。phase／owner／世界／incarnation 變更會中止等待。

## 原始資源

藥水效果 189、194、197 使用獨立延遲載入的 SharedEffectBank，不改變既有技能 bank 的来源集合。三者都是 action0、24 FPS、undirected、attr104(8)，以 class0 的來源 1～9 接終止 0 播放，共 18 ticks。終止影格停留三 ticks 後退場，沒有音效事件。S37 位置維持接收時的固定快照；效果不修改 HP。

地面物件由正式 `Protocol.groundItem` 發出 S0B；共用 `experience` 欄位在此用途保存數量，其他原始欄位仍完整保留。地面 ID 由伺服器新建，與背包 ID 不同。原始 gfx45／773 是 type9、image_count=1、沒有動畫動作的物件；只有完整 header、stance0 及 SPR 解碼後單影格的條件都通過，才進入靜態呈現。既有 type14 看板條件保留。

## 隔離驗證

正式 JAR 固定為 `589733435518D92C05BAEEAE37A686D384EC3F7B9897E975AD1EF50D67A74AA7`。只使用重建版內的記憶體帳號／GameStateStore，SQL 不存取、state 不落盤。正式配置與玩家資料保持唯讀。

| 情境 | 核對內容 |
| --- | --- |
| inventory-actions | 武器／戒指裝上再卸下，四次權威更新，數量不變且名稱恢復 |
| inventory-actions-refused | CHANNEL 內普通藥水兩次靜默拒絕，背包不變，兩秒後可手動重試 |
| room-world-item-heal | 真實普通攻擊及反擊後使用兩瓶藥水，HP 回到50，數量2→1→移除，兩次189效果退場 |
| room-world-item-drop | 僅隔離配置開啟丟棄，寶石4→2→移除，兩個地面ID及原始773-0.spr呈現 |
| room-world-item-drop-refused | 隔離預設禁止丟棄，兩次手動操作均無背包或地面變更 |

九項請求／生命週期測試核對 wire、無號欄位、截斷／多餘位元組、過期快照、限制、等待與中止；另兩項原始資源測試核對三個藥水效果的13個時序樣本、終止停留，以及45／773地面圖。原始 Sprite archive 的五份 bytes／SHA256 由 Python 獨立讀取核對。新增 PNG 使用來源與時序核對，不計作新增獨立像素參照比對。

集中重跑使用 `維護工具/verify-native-item-actions.ps1`，稽核使用 `維護工具/verify-native-item-actions.py`。最終完整數量、EXE 雜湊與建置狀態以 `驗證紀錄/current/native-item-actions/summary.json` 和 `驗證紀錄/20261012/current-build-receipt.json` 為準；只有最終所有關卡通過才發布 schema25。

## 尚未涵蓋

194／197 只驗證原始資源生命週期，未宣稱對應藥水的正式世界流程已通過。完整 GUI、焦點、數量輸入、選取更新與 DPI 操作仍待驗收。特殊目標／選項物品、拾取、銷毀、交易及快捷列尚未完成；原始地面排序、名稱／陰影與拾取呈現仍需核對。技能63耗材／gfx2231、祝福／詛咒圖示、其他技能與租裝來源分支繼續列在工作佇列。

後續 schema27 已接入原生附近物品清單與指定數量拾取，並以實際 inventory 封包重播驗證；本頁上述拾取待辦已由此子集接續。地面排序／遮擋、直接滑鼠拾取與完整 GUI 仍未完成，詳見 [地面拾取](NATIVE-GROUND-PICKUP.zh-TW.md)。
