# #59 暴風雪候選開發範圍

只開發及編譯；使用者暫緩測試與桌面操作。候選未套用正常來源或登記選用，沒有測試執行、GUI、GAME或聲音播放資格。

正式 E149 的 AreaAttackSpell：C14 `14 07 02` 三 bytes，無角色／座標尾巴；選單移除既有目標，仍沿技能書／owner／controller守衛與1500ms reuse基值。MP60、HP10、radius10、3d10與base2、attr4及lawful-10均由server決定，不預扣或造傷害。

原757比較分支建24個無target list的child，再啟動帶原名單的root：heading0..7依序各radius5、3，再heading0..7 radius7，最後中心。新Stage為每節點保存獨立Playback／birth／位置，載共享757單圖23frame、各3ticks、Sound123 source1／Hit source3與Screen，逐幀整數callback／late poll不catch up。根Hit只診斷原完整名單，不在相對gfx／hurt guard未綁定前送替代Impact。所有音效均保留真cast及節點occurrence，使用world actor0域；只準備typed cue，不播放。

原方向初始化程式宣告native X步長2；靜態解碼映射到world X步長1。分析PE檔內方向表是零，不把零當runtime表；初始化未執行，方向宣告不是動態資格。核心接受25個birth，目前整合採共同frame now；原逐constructor讀時鐘偏差、action0可選倍率／u32 rollover與全域scheduler交錯仍待資格，不宣稱精確重現全部原時序。

Many History納入25節點graph／active序號及同代admission合併，同來源重建續播；來源退役清graph保留admission。原Many也保留active seen防raw ring回入重啟。維持64 cast／32音效／512 fragment／4096 key上限；整組預檢投影及fragment容量，音效批次放不下只記omitted並不重試，不截斷視覺graph。最大共存／G11未驗。

新增12案例來源（11純＋1共享原資源opt-in），全部未執行；加前240共252個待執行來源案例，並非完整library inventory。兩版18metadata只作型別／編譯核對，不是測試通過。#61復活仍需屍體意圖、question321／回答、owner／世代與server復活結果完整流程；3944特效不能當復活成功。
