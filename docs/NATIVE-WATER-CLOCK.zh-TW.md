# 原水下時鐘、捲動與排程證據

2026-10-16，私有x86隔離程式執行八個原函式，共968 bytes；僅做69個資料／呼叫位址重定位，原分支、運算常數與指令不改。8,656組相位、9,668組捲動及15組佇列情境通過Python獨立核對，全部global／配置緩衝區守衛通過，原碼與重定位碼差異逐byte驗證。原完整客戶端未執行，產品仍是純Rust／Windows x64。

限定資格：[water-clock-original-current.json](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/water-clock-original-current.json)。原始資料位於`驗證紀錄/current/native-water-clock-original/20261016-074852-110050`；raw ZIP SHA256為`A97D38AFB45E85FE4FE13E719DA71E26D76A1791372F2D0C35D0758A627B14F0`，含實際x86 EXE、來源、八個原片段、重定位表、原九筆保留表、向量與日誌，已逐檔讀回。

| 原函式 | 本批驗證內容 |
|---|---|
| 487510 | water非零：counter在99回0，phase大於20回-20，否則各加1，從當下重新排100ms；water零：只重設phase／counter，不再排程。 |
| 4874D0 | X加增量後模640，Y模420，沒有water gate；乾地呼叫仍累積。向量增量使被除數非負，不推廣到未驗任意i32溢位輸入。 |
| 4D65F0 | 期限使用呼叫當下clock加delay，相同期限插在既有記錄之後；不依callback／owner去重，並驗到兩次原配置擴張。 |
| 4D6570 | 每次dispatch只擷取一次截止clock，先移除到期項再呼叫；回呼重排時另讀新clock，不按漏掉的100ms步數快轉。 |
| 4D6870 | 移除全部同callback／owner項，保留其他項及順序。 |
| 4D63A0 | 換圖清理只保留原7＋2筆callback，其餘移除；487510不在保留表，舊water chains會被清除。 |
| 482370、4D67F0 | 確認482370／ownerFFFFFFFF已排，缺時排一筆10000ms；重複呼叫不增加項目。這條原路徑也實際執行，未當空操作。 |

晚到測試中，兩筆原50ms到期項在dispatch截止1000ms被處理，重排分別讀到1007與1014，期限因此是1107、1114；不是補跑所有漏幀。實際到1106未觸發、1107僅第一筆觸發，之後晚到2000仍只各跑一回。

S28與maploader的呼叫鏈由原ASM靜態保留，沒有執行整段S28／maploader：S28true每次可排50ms；S28false取消，不直接重設phase／counter。485800／48BC80會先經4D63A0清舊water，再於water開啟時排新的50ms。先前只追直接呼叫而推測mapload也可能一直疊加的敘述，已由這份證據修正；重複S28未經換圖清理的情況另論。

第一份靜態phase提取混入相鄰487590，未知全域寫入守衛在任何建置／執行之前拒絕；失敗來源、部分資料與ZIP保留。正確phase片段為119 bytes，不能使用136 bytes的舊提取範圍。六函式中間草稿也只作靜態準備；最終動態資格綁八函式版本。

clock來源由測試值提供，CRT配置／移動使用私有真實記憶體，audio dispatch tail以空函式替代，換圖的次要向量供空集合。沒有證明完整原clock來源、音效、非空次要向量、完整maploader、登入初值或整個原主迴圈。向量中的phase／scroll初值都是明示輸入，不能據此聲稱每次登入會清零。

本次執行前，999來源正常兩主的TW14 GUI已全部完成且同handle61939 exit0，owned GUI／Java／compiler／test全退才建置隔離程式。產品999及兩正常主雜湊前後不變。**目前Rust尚未接入這套phase／scroll排程與額外水下像素pass**；後續須以同一人物Pose／camera、GAME owner及退役規則接線，再驗新版本，不能以本批原函式向量代替產品資格。


2026-10-16後續：已將數值行為移植產品Rust並接場景，Debug/Release各92項與正常兩主已限定核對。原隔離oracle本身的stub與範圍不變；新的產品整合與尚待事項見[NATIVE-WATER-PORT](NATIVE-WATER-PORT.zh-TW.md)。
