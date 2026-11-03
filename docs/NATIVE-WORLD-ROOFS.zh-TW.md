# 原始屋頂與遮擋群組

現行世界場景已接入 S32 Layer5 控制的半透明／完全隱藏群組。群組不一定是屋頂，也可能是樹、牆或大型裝飾；實際作用範圍由原始地圖決定。原版 BIN 未執行，共用資源與相鄰專案保持唯讀。

## 來源與執行路徑

`native_multisector_scene` 將 Layer4 的來源群組 ordinal 留在地圖分片上，不能改用序列化 group ID。Scene 載入時建立屋頂定義，並為受控圖格保留原始 RGB555 coverage；特效 TIL 在漸變時需要原始 coverage，不能拿已轉成預覽 alpha 的像素反推。這些快取受原有資料與 64 MiB prepared terrain 上限限制，載入與合成可取消。

`native_world_roofs` 以本地角色的邏輯格座標查 Layer5，精確轉換到 native half-cell。相機位置與人物顯示內插不控制屋頂。座標包含現行導航已檢查的本地預測位置；收到伺服器修正後依更新座標重選群組，不寫入世界權威狀態。

漸變使用每 50 ms、每次 4 級、0～31 權重的 RGB555 參照；穩定半透明使用分量平均，並非固定 16/31。離開會回復來源正常／基礎半透明狀態，中途反轉從當前權重繼續。長時間停頓只需有限步數追上，不建立無界 timer 工作。世界世代、角色 ID、角色 incarnation 或地圖 geometry digest 改變會清除舊群組；不同地圖的屋頂 Frame 不能傳給現行合成器。

屋頂、地形、本體、附件與陰影共用現行分片深度排序。完全遮蔽的來源會覆蓋底下 owner；半透明與漸變保留底下像素 owner，完全隱藏群組不提交。選取因此沿用同一畫面像素表。

## 原始位元組證據

`驗證紀錄/current/native-world-roofs/native-roof-raw.json` 保存固定 PE SHA256、各地址的位元組及獨立 Capstone 解碼。0x483890 使用 0x62F3AC 表的 kind0→flag1、kind1→flag2；0x48AFC0 依舊位置與本地角色 +0x2C／+0x30 選取群組。0x432F70／0x4330D0 包含步長4及50ms排程，0x488DAD 為繪製旗標優先序，0x4C1A17 為權重表證據。

獨立 Python 盤點全部 495 個 LTmap S32，共讀取 52,285,859 bytes：有效座標內只有 kind0／1，群組引用皆合法；49,975 筆區塊外紀錄依載入規則略過，不捲入鄰接區塊。這項盤點只證明 metadata 範圍，不等於全部地圖已通過渲染驗收。

## 驗證範圍

六項 Rust 測試檢查群組 ordinal、半格與區塊邊界、非法資料、漸變進出／反轉、身份與地圖切換、倒退時鐘、取消，以及世界狀態不被顯示處理修改。

真實 LTmap 564 有390筆 Layer5、369個有效觸發格、49個群組；八張時點圖逐像素比對獨立 scalar RGBA 與 owner 表。樣本可見本體像素118→778→118。真實 LTmap 565 的四區塊有1,056筆 Layer5、682個有效觸發格、17個群組、354筆區塊外紀錄；六張完全隱藏／回復時點圖同樣逐點比對，樣本454→778→454。

Python 獨立讀取原始 S32 bytes 並重播 clock，另核對資源 pins、十四張 PNG，以及整套回歸中20個世界案例的角色身份、座標、elapsed clock 和群組狀態。既有八張地形／本體、六張角色附件／陰影與六張背包圖示仍保留各自比對。

本階段完整 gate 為同一 Debug x64 EXE 的167 Rust／31組、52隔離TCP、34靜態像素比對。最終結果以 `驗證紀錄/current/native-world-roofs/` 的 `summary.json`、`roofs-receipt.json`、`final-state-check.json` 為準；Release 只有編譯檢查。

## 尚未完成

這是依已核實來源規則實作的 elapsed-time catch-up；未重建原版 timer queue 的排程延遲及多群組共用 tile pointer 的 alias 行為。SPR／TIL 解碼器共用，scalar 合成與 Python clock replay 才是獨立路徑；RGB555 參照不代表原版 RGB565 framebuffer 像素等價。完整動態角色旗標、特效交錯、原版插入時序、GUI／IME／DPI、音效裝置及完整玩家流程仍待驗收。

本輪最終結果：167 Rust／31組、52隔離TCP、34像素比對皆通過。20個世界案例共873個影格的屋頂狀態獨立重播一致，但這些TCP路線沒有進入屋頂觸發格（active roof frames=0）；實際半透明／隱藏觸發的證據是兩張原始地圖的14張CPU合成樣本與Model測試，並未宣稱連線路線或GUI已驗收屋頂進出。
