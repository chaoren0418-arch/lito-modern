# map303 地形遮罩的來源追查

本紀錄只讀參考原客戶端證據，新增檔案均在重建版。產品仍為 Rust／Windows x64；不替換登入器產物，也不執行完整舊客戶端。

2026-10-16 的[靜態封存](../驗證紀錄/current/native-map303-gate-static/20261016-090752-113308/qualified-static.json)綁定原 PE `4271D92F`、ASM `2A3A6E10`、產品 1002 份編譯輸入及正常 Debug `BF88BA78`／Release `694B7EC6`。原換行以 LF 分段保留 CRCRLF，全部選段有實際行號。16 處 `6476C4` 全域指標引用、12 個相關函式 ASM、三個地形寫入原片段均封存，ZIP `401670DD00E897A80C47CAAFB62402E680A580DD26674E5BD9719EBDEC59AB40` 已逐檔讀回。

原世界繪製先清除該 byte plane；處理地形時，把 map303 相等條件傳給三個寫入函式。這些函式在複製地形色彩時，依相同落點與 span 長度寫入 byte1。後續選定的背景／sprite 路徑遇到非零 byte 會跳過該像素；已另隔離驗過的普通光照色彩 kernel 則在 gated 模式跳過零 byte。這顯示 gate 依賴地形寫入來源與繪製順序，不能由最後 RGBA 畫面是否不透明推得。

| 原函式 | 完整原片段 | 所見資料流 |
|---|---:|---|
| 4BE000 | 263 bytes | RLE 的 signed x/y、每行 spans、skip/2、像素長度；複製時可同時填 gate。 |
| 4C14C0 | 424 bytes | raw 右半片段；呼叫處第一 stack 參數是 pitchBytes−8，另外傳 frame origin 與 pitchPixels。 |
| 4C1670 | 418 bytes | raw 左半片段；第一 stack 參數是 pitchBytes−4。 |

三段合 1105 bytes，只需七個全域指標重定位；相對跳躍全部留在各函式內。raw 兩段末尾各呼叫一次 surface vtable `+80h`；隔離測試將明示以私有 Unlock stub 接住，不能把它當完整原 surface／裝置驗證。位址計算的 gate row stride 是 744 bytes；所見配置大小為 `0x64440`、world 清除大小為 `0x15CD2*4`。這些原始數值分開保留，不自行推廣成任意新視窗尺寸。

初次靜態提取的 left 上界截在 ret 指令中間，完整 return 守衛在任何建置或執行之前拒絕；失敗來源與兩份已提取片段已獨立封存。改為完整 418 bytes 後，三段的所有分支、全域讀取及唯一間接呼叫均通過靜態檢查，沒有帶入鄰近函式。

[36 案隔離工具準備收據](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/map303-gate-prepared-current.json)涵蓋三段、三種明示 pitch、gate 開關及兩種原 plane 初值。預計核對原色彩輸出、整個 gate、來源不變、緩衝區守衛與每一允許重定位。撰寫本紀錄時仍在跑產品 1002 的 TW14 GUI，這批隔離工具尚未建置或執行。後續動態結果須另立收據，不能以此靜態紀錄宣稱原片段已跑過。

產品 map303 尚保留明示未完成分支；還需核實背景資源、所有相應繪製路徑及新 800×600 視窗的來源覆蓋，再接真正 gate。完整原 framebuffer、普通世界服務可達性、特殊 global filters 與 G01–G12 仍未完成。


後續動態限定結果：產品TW14 GUI與所有owned負載退場後，三個原片段已建置並執行36案，29,251,584色彩／gate cells與獨立Python scalar一致；七個重定位、完整原分支／常數、來源與緩衝區守衛均核對，實際x86 helper exit0。完整raw 93檔ZIP C13028C428E02BD36022F418FCCE92006A2275A5DB540E8DA8337DCB7A3D3D88再次獨立讀回；產品1002與兩正常主不變。兩個surface Unlock仍為私有stub，沒有驗完整world/map303背景或產品gate；見[動態獨立讀回](../驗證紀錄/current/native-map303-gate-original/20261016-093419-178126/independent-readback.json)。
