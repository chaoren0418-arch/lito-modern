# 水下光源來源與正常主程式

2026-10-16，999份編譯輸入已固定。唯一入口正常主為Debug **A09486F2**、Release **6E2FEDF2**，均為features空的AMD64／PE32+。Debug十二個照明／場景目標65過，另五個既有標籤回歸16過，合17份實際測試EXE81過；Release十二目標65過。兩版均0敗／0忽略，來源、測試EXE、正常兩主與原始日誌ZIP均已獨立讀回。

限定資格見[water-source-current.json](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/water-source-current.json)。先前同為999份輸入的Debug76是整合前內容，未移記至本版。Release／正常建置pipeline同一session49076實際exit0。

## 實作

Model把人物當下同一Pose的action、來源格及畫面中心傳給native_world_lighting。來源分類讀取真SPR的102／108、原始MS950主人、事件1F／50及unknown低四bit；不以generic dead或最後一筆動作歷史代替當下action。11類在原順序直接回傳，略過1F歷史查詢；其他類別需要時才解析事件當時的graphics，後續變身不改寫歷史。

判斷以原有號ID順序、U±35／V±15範圍及累積範圍700為準；adapter與compose共用source_range。達700後不再查後續角色的SPR或action，範圍外亦不查分類。範圍內所需資料缺失、GAME關係owner不符時明確失敗，Canvas像素保持不變。

未失明的水下現在確實生成alternate3遮罩，但普通RGB mask pass仍按原外層條件略過。**額外water shader、phase與scroll尚未接入場景，因此水下畫面效果尚未完成。** 本版完成的是來源選取與遮罩產生。

## 實際驗證

新增四項adapter測試涵蓋主人三態、1F／50、signed action、mode低四bit、原始負光源值仍受class3／5覆蓋、盟戰結束後變身與歷史SPR缺失、缺action／owner不動像素、50個range14達700後不查第51個未知gfx。舊864分類／光源向量及既有mask／色表／water kernel測試照跑。

真實原資源Model另驗同人物class5／range14，接收S20死亡action8後轉class3／range5；人物與光源共用同一投影中心加(0,6)，renderer不改World。這是實際Model及純Rust合成路徑，沒有新增正式GAME sessions或完整原世界framebuffer資格。

首次正常主資格讀取器把PowerShell輸出視為UTF-8而失敗，原讀取器、錯誤原因與原始stdout雜湊已保留。實際兩份輸出為可嚴格往返的CP950，修正解碼後仍核對完整主程式路徑、原始log、PE及ZIP；產品測試與建置沒有因此重跑或改標失敗為成功。

## 下一步

同版TW14 GUI已在兩正常主完成並獨立核對：112sessions／220move零多送／80原世界PNG／56glyph／128stale，850份raw ZIP2A7D1899逐檔讀回，原pipeline61939實際exit0。四個case／authority／recovery／window關鍵驗證工具與舊994 GUI原始ZIP逐byte相同；只增加明確的新來源／測試範圍綁定，未放寬移動、背包、wire、glyph或失敗恢復條件。GUI結果另存於本版normal-qualified目錄，不以舊APP/GUI-current.json代表本版。

水下時鐘／鏡頭靜態補證位於`驗證紀錄/current/native-lighting-actor-motion-static/water-lifecycle-20261016-071847-339171`：4874D0沒有water gate，乾地自行走路仍累積水紋捲動；新補查發現485800／48BC80先呼叫4D63A0清掉不在原7+2保留表中的回呼（含water），再於water狀態排50ms；482370另確保一筆10秒aux回呼；4D65F0不去重，4D6870移除全部同callback／owner；S28false不直接寫phase／counter。尚未動態核實整條scheduler、間接初始化與完整map-remap dispatch，後續不能簡化成單一timer、進水清零或elapsed catchup。

本版已取得192DPI私有桌面訊息限定的TW14 GUI資格；完整default／library／TCP回歸與完整水下效果仍待。map303 gate、其他global filters及完整G01–G12仍待完成。登入器與相鄰伺服器唯讀，正式C02 JAR雜湊再核對，無外部產物替換。

原八函式時鐘/捲動/排程動態補證已完成，見[NATIVE-WATER-CLOCK.zh-TW.md](NATIVE-WATER-CLOCK.zh-TW.md)。這是後續Rust接線依據，並非當前產品已有水下時鐘。
