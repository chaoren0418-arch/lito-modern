# 原生世界連續動畫

世界畫面使用一個單調時鐘，持續重繪原始資源的待機和走路影格。角色不再全部停在走路第一張：已核對的武器 stance 使用對應的 breath／idle family，原始 catalog 同時必須有符合名稱及語法的定義。門使用伺服器指定的開／關動作，完成後停在最後有停留時間的影格。

```powershell
pwsh -NoProfile -File .\維護工具\verify-native-animation.ps1
```

## 時序與位移

`native_world_animation::Clip` 使用 list.spr 的 duration_ticks 與 FPS，以整數有理數選取影格。零時間影格沒有顯示停留；循環以總 ticks 取餘數，長時間跳躍不逐圈追趕，也不累積每張影格取整誤差。未理解的 Skip／numeric 事件、語法隔離項、錯誤 alias 及零總時間會明確拒絕。Effect／Sound 等事件保留在圖像收據，尚未執行；沒有把 Hit cue 當成傷害權威。

World 額外記錄每次移動的來源格及 motion_revision。渲染器依這筆資料播放走路動作，在來源與目的格之間插值，完成後回到待機。網路／預測位置仍是封包整數座標；插值只影響顯示與鏡頭。自己的 S_MOVE_OBJECT 修正直接回正，不把伺服器校正當成另一個預測步伐。其他物件更新、刪除及重新加入會更新或回收自己的時序狀態。

原生視窗持續要求背景重繪（目標間隔 50 ms，實際頻率受 CPU 合成時間限制），仍然同時只有一個工作。連線 epoch／世界 generation 過濾保留；時鐘獨立於 WM_PAINT，不以視窗重畫次數推算動畫。鏡頭與物件使用同一份抽樣位置。

目前依原資源正常 FPS 播放；加速／緩速及其他速度修正、連按排程、延遲補償仍未完整接入。NPC 的未知走路事件不猜測執行。整體地形／角色／陰影遮擋、動畫音效和特效仍待整合。

## 靜態看板

實際 list.spr 的 gfx1544 為 `#1544 1 board a`，只有 shadow1545 與 type14 屬性，沒有 action0。現行 Sprite 的 `1544-0.spr` 也確實只有一個解碼影格。新引擎只在「合法 header、image_count=1、無動作、type14、stance0、解碼影格數=1」條件都成立時以靜態物件呈現；不把其他缺動作或 dummy 定義套到這條路徑。shadow1545 的疊合仍待實作。

## 移動限制

正式 S_PARALYSE=0x25 的七組 apply／release，來自固定 JAR 的 `Protocol.ParalysisMode`：paralysis、second paralysis、teleport lock、sleep、freeze、stun、bind。各類鎖獨立記錄；解除其中一類不會解除另一類。狀態有效時 C_MOVE 在送出前拒絕，也不建立預測。新世界清除舊鎖。截斷、尾端或未核對模式不會部分改寫狀態。

這輪限制封包以正式 JAR oracle 與 Rust 測試核對，尚未完成真正對戰施法到鎖定／解除的全程情境；已在途的移動、Cube: Quake 等未通知／靜默拒絕條件仍有缺口。

## 驗證範圍

連續動畫驗證使用真正 LOGIN／GAME ClientSession 的世界快照：原始進場、正式成功移動、過期起點修正。每個情境在同一 Model 產生 0、1/6、1/3、1/2、2/3、1 秒附近六個確定時點的 PNG。核對 idle 循環、walk 0→1→2→3→idle、中途半格、最終目的格、鏡頭中心與靜態看板；世界 12 個物件皆須呈現。這些是 GUI 共用渲染器的實際收包證據，完整 GUI 操作與實際畫面更新頻率仍待人工驗收。

獨立資源測試讀取現行 Text／Sprite，核對十種角色／NPC 的待機定義與每個 subframe、單張看板；原始動作摘要輸出 `LiTo381-Modern/target/current/verification/native-animation-definitions.json`。時序測試涵蓋分數邊界、零停留、長時間跳躍、回正、七組限制與多鎖解除。

本輪 53 項相關 Rust 測試、27 種 TCP 情境（動畫重用其中 3 種）、Debug x64 建置與 Release 編譯檢查全部通過；離線原資源回歸不變。Release 未做執行驗收。

收據在 `驗證紀錄/current/native-animation`，重跑工具如上。完整可玩客戶端仍在進行；下一步接入速度與連按／碰撞、影格事件、戰鬥及返回。

後續版本已接入共同速度倍率、按住輸入及 map564 原生地形檢查，實測與目前限制見 [原生速度與通行驗證](NATIVE-NAVIGATION.zh-TW.md)。本頁既有測試描述保留該階段範圍。
