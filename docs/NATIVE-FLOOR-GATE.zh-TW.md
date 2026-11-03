# 原地板像素與 map303 floor gate

本實作屬於重建版的純 Rust／Windows x64 核心。原客戶端僅作唯讀行為參考，沒有把原 x86 指令放進產品，也不替換登入器產物。

2026-10-16來源1005：Debug/Release各19實際測試EXEs、20群120過0敗6既有忽略，唯一入口正常AFF5AF96/C7F36CC4與source/test/main/raw已核對。另六項真實scene opt-in兩版各5過1缺root catalog失敗，原失敗保留，Disintegrate來源路徑未修。本版TW14 GUI全部獨立核對：112sessions/220move零多送/80PNG/56glyph/128stale、850份raw ZIP728496BC讀回；同26334實際exit0且owned全退，兩版新圖均目視。僅192DPI私有桌面訊息，實體輸入/多DPI未驗。原背景geometry1294bytes、2688案/17472call已隔離核對，但混色/modern viewport adapter與map303 overlay/background未接；完整default/lib/TCP及G01–G12未完成。

原地板分派片段120bytes與三個真實 raster writer，共1225bytes、12個重定位，在私有x86程式執行64案。原片段只在其尾端附加私有 return，原分支、常數及指令不改；surface Unlock 用私有stub。16種實際支援的 TIL type、兩種半格奇偶、map302／303，完整52,568,064個色彩／gate位置與獨立scalar一致。[原始限定收據](../驗證紀錄/current/native-floor-gate-original/20261016-095251-214865/receipt.json)與ZIP `4F77B89D`保留完整輸出。沒有執行完整原世界、資源cache查找或原GUI。

結果確認，地板直接複製原RGB555，忽略供overlay使用的半透明／雲色alpha。bit8地板跳過，raw左右半片由目的半格奇偶決定，不能只看檔案type低位。產品因此分開處理預覽RGBA與實際floor像素；對雲色／色鍵效果，額外保留已解碼的原RGB555與二值覆蓋，並在既有資源預算內計算。普通預覽和overlay效果資料仍保留，不以透明色零取代編碼覆蓋。

31KB的固定JSON fixture `1B0AAD36`只存數值TIL、原完整色彩／gate SHA及案例資料。新Rust測試真的經現有TIL decoder及Canvas逐案還原64個完整表面；另核對極端裁切、黑色覆蓋、重複mark及錯誤輸入先驗證再修改。Raster新增檢查證明：後續body覆蓋不清除floor gate，overlay也不冒充floor。Gate每幀重新建立，沒有跨場景全域狀態。

首輪新增測試調用crate私有decoder，Cargo返回101、外層Python返回1；改用既有公開RawRgb555Coverage入口，未擴大API。原source已在編譯前封存，首個封存observer誤把Cargo碼當1而停下，後以實際碼與EXE修改時間確認，保留一份原partial EXE。第二輪十八整合目標94項通過，但一個舊壓縮fixture為普通RLE塞入半透明alpha，raster群5過1敗；同已編原EXE補跑scene19過6忽略。這批19個EXE、來源與raw完整保留，沒有算合格。修fixture為格式有效alpha，仍保留所有dense／compact像素、roof、深度與hit對照，才完成本版120項重驗。

目前只有floor gate接入場景，map303 overlay與背景仍明示未完成，沒有把不完整plane套入照明。原背景已追到IMG 800／803／892／922／923及native優先、BASE Sprite後續路由；五張圖在LTsurf缺席、BASE Sprite存在，已逐member核對並檢視opaque RGB555來源圖。這不是產品map303畫面，也沒有驗完整背景blend／camera／clock。完整G01–G12、正常本版GUI、正式水下wire／GUI、OS輸入、多DPI、可攜與穩定性仍待。

背景的間接呼叫亦已追到原 initializer：`dword_768878` 在 RGB565 模式指定 `sub_4BF0B0`，RGB555 指定 `sub_4BF500`；兩者都先跳過非零 terrain gate，再寫背景像素。原 `sub_4878C0` 每次遞增兩個 32 位 offset，重新排入 64ms 回呼；另一個 `sub_4878F0` 遞增 counter，重新排入 50ms。原初值宣告按原樣保存，未猜成 signed16，也未推定換圖重設。這些是[唯讀靜態證據](../驗證紀錄/current/native-map303-background-kernel-static/20261016-104424-701728/retained-static.json)，原完整 FPU／blend table initializer、背景幾何與像素函式尚未動態執行。初版 observer 把兩種模式的 gate 暫存器都寫為 ecx，guard 在輸出／原碼執行前擋下；失敗腳本保留後，依原 RGB555 的 edx 修正 observer，產品不變。

本輪也準備了 `run-native-floor-scene-optin.py`，在兩版正常產物核對後，直接使用已編 library EXE，明確執行六項原資源 scene opt-in。它會清除 capture／reference-write 環境參數、唯讀使用原資源，原預設六項忽略計數另留，不冒稱預設已執行；目前僅完成工具語法檢查。


## 正常兩主與新增原資源檢查

2026-10-16來源1005：原地板RGB555／半格奇偶與floor gate已接入，Debug/Release各19實際EXEs、20群120過0敗6既有忽略已獨立讀回；同handle14931實際exit0，唯一入口正常Debug AFF5AF96／Release C7F36CC4與source/test/main/raw封存核對。另明示執行六項真實scene opt-in，兩版各5過1缺root TW catalog失敗，原失敗不作PASS；尚未修Disintegrate舊來源路徑。新正常兩主TW14 GUI正在同handle26334，來源/QA凍結；map303 overlay/background仍未接，沒有套用不完整gate。1002正常BF88BA78/694B7EC6與各92/TW14保留變更前資格，完整G01–G12未完成。

資源失敗已定位於 world_scene.rs:4061 的 DisintegrateAssets::load，第一步原TW reader明確要求 root/TW13081901.pak；當前來源只有 launcher/TW13081901.pak。測試在baseline render前停止，未改產品路由、未搬移封包來冒充原來源。其餘五項含Honor raw屋頂cache、完整武裝family、unarmed combat、屋頂淡入淡出與14外觀世界移動均實際通過；兩版各5/1/0及各actual exit101、正常兩主/source不變、原ZIP已[獨立讀回](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/floor-scene-Release-boundary-20261016-105521-774851/five-scene-and-catalog-boundary-qualified.json)。前述工具「僅準備」是這次實跑前狀態；原預設120/0/6與額外5/1/0分開記錄，不宣稱全部六項通過。


來源1005正常兩主AFF5AF96/C7F36CC4保持；Debug7 GUI全部完成56sessions/110move零多送/40PNG/28glyph/64stale，Release仍沿同26334執行。兩版a-new-battle.png已目視。背景1294byte原geometry已準備2688案/17472預期call、31data與15error-call重定位，只完成source/metadata/ZIP讀回，沒有編譯或執行。已實測CIM idle守衛在live GUI時返回1阻止helper。原背景四段表在分析PE都是零，不能當初始化golden；原FPU常數為1/31與31。原4F1592跳轉與另外56個CRT片段，在同一推定base77380000下全部對應本機x86 msvcrt exports、0不符；這是靜態layout推論，未取得歷史DLL binary identity、未載入/執行DLL，完整原initializer/背景像素未驗。兩次observer守衛（常數先猜32、只查IDA已命名函式漏掉資料標籤）失敗原腳本保留後，依原bytes修為31與57個實際trampoline，不影響產品。


## 當版TW14與原背景geometry收尾

2026-10-16來源1005：Debug/Release各19實際測試EXEs、20群120過0敗6既有忽略，唯一入口正常AFF5AF96/C7F36CC4與source/test/main/raw已核對。另六項真實scene opt-in兩版各5過1缺root catalog失敗，原失敗保留，Disintegrate來源路徑未修。本版TW14 GUI全部獨立核對：112sessions/220move零多送/80PNG/56glyph/128stale、850份raw ZIP728496BC讀回；同26334實際exit0且owned全退，兩版新圖均目視。僅192DPI私有桌面訊息，實體輸入/多DPI未驗。原背景geometry1294bytes、2688案/17472call已隔離核對，但混色/modern viewport adapter與map303 overlay/background未接；完整default/lib/TCP及G01–G12未完成。

[GUI終止與850份原證據讀回](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/floor-tests-20261016-102632-227125/normal-qualified/GUI-completion-observation.json)綁1005 manifest DF23B9DB及正常兩主；正式C02 JAR唯讀hash再核，沒有登入器部署。

[背景geometry獨立資格](../驗證紀錄/current/native-background-geometry-original/20261016-110731-202085/qualified.json)：原完整1294byte函式、31data與15error-call允許重定位，2688案/17472次kernel呼叫與兩套scalar核對。暖cache IMG物件和pixel recorder是私有fixture，loader、真pixel/blend與完整世界未執行，0新GAME sessions。第一次一般552矩形交集模型失敗已保留：原函式測底部溢出552，卻用138+384=522裁切；依原bytes保留這條有條件30px規則，未改原指令、原EXE或第一套scalar，也未重跑native來更換結果。來源1005與正常兩主不變。geometry不能代替modern source cell/camera/viewport接線或完整背景像素驗收。


## 2026-10-16 原背景混色與Rust核心

2026-10-16工作樹1009：新增純Rust背景RGB555/565混色與有界取樣核心，135168原channel words、1584原整frame hash與地板64案已Debug2實際EXE7過0敗0忽略、source/test/raw封存獨立讀回；同93070實際exit0。新core尚未接geometry/真IMG/完整overlay gate/正常場景，最新Release與唯一入口正常主/GUI未驗。4246byte原initializer/kernel已用明示固定host _ftol實跑並核對，歷史DLL binary identity未核實；第一次WOW64 loader path observer失敗保留，新增physical handle path及已載入callee bytes後通過，兩輪原數值輸出完全一致。1005正常AFF5AF96/C7F36CC4與TW14另保留變更前資格。完整G01–G12未完成。

原三函式2030/1101/1115共4246bytes保留全部分支/常數，只做私有data/明示callee重定位。原兩種mode共2048次host _ftol，已載入39byte leaf含截斷與控制字還原指令、48byte capture與固定x86 DLL C1BE919E吻合；歷史DLL image仍不可聲稱相同。135168 channel words、1584pixel案/327376改變words、1584Lock/3168Unlock及所有來源/gate/frame guards核對。[原始限定資格](../驗證紀錄/current/native-background-blend-original/20261016-114436-881465/qualified.json)使用私有surface與synthetic IMG，0新GAME，不是完整map303 frame。首輪1584已原生exit0，但System32/SysWOW64文字path不同，metadata observer失敗已封存；第二輪擷取physical path與callee後通過，10份input/table/pixel outputs逐byte一致，未改原shader指令來更換結果。

產品只有純Rust整數運算；不含原機器碼。312191-byte JSON 1E4F702B與270336-byte原numeric LUT03D786BB作編譯fixture。核心先驗source/dimension/stride/format/phase/強度，才修改pixel；非零gate保留原word，covered black仍寫入。RGB565沿原五位green lookup，screen用原31域截斷，attenuation保留兩次量化，不用一般RGBA alpha替代。[Debug限定資格](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/background-pixel-tests-20261016-115714-009940/qualified.json)為2EXE7/0/0，包含全部原數值向量、invalid-before-mutation與floor回歸；尚未接正常世界scene，也未建正常新主或執行本版GUI。


## 2026-10-16 Rust背景geometry與場景來源邊界

2026-10-16來源1012：純Rust背景geometry已補齊，2688組原輸入與17472次有序pixel calls全部吻合；連同原背景135168 channel words／1584整frame與64地板案，Debug三實際EXE10過0敗0忽略，同96668實際exit0、source/test/raw封存讀回。保留signed負奇數與552溢出判斷／522條件裁切。正常場景、真IMG、完整overlay gate、modern viewport adapter與最新Release／正常主／GUI仍待；1005正常AFF5AF96/C7F36CC4及TW14保留變更前資格，完整G01–G12未完成。

原source cell明示使用NativeDrawCell.half_x，不能將server whole-cell X直接代入；本API只規劃原744×552座標域，未接modern 800×600 camera。866189-byte純數值fixture F4B9F257由已核對原run-raw匯出，不含原指令。兩次runner文字準備守衛因誤猜空行失敗，皆發生在產生runner前，原preparer及原因保留；修正LF邊界後才建置，沒有削弱測試。

已更正原先before-all-bodies假設：原488C34第一輪按cell畫地板及同cell物件清單，488F49才在map303呼叫背景；物件plain RLE可走4BE000，不能用所有RGBA不透明像素代替gate。原map loader明示LiTo/LTMap的s32失敗再seg；目前原生303目錄不存在，BASE map/303雖有30份S32，尚未證明可fallback，未增加替代路徑。這是static來源觀察，非原分派實跑或正常303 GUI。重建版與登入器產物獨立，未替換或部署登入器JAR。

[當版限定資格](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/background-geometry-tests-20261016-122035-301179/qualified.json)；[原排序與來源觀察](../驗證紀錄/current/native-background-scene-boundary/20261016-122149-466090/observation.json)。


## 2026-10-16 真IMG背景bank與物件分派

2026-10-16來源1015：真IMG背景bank已加入純Rust，LTsurf優先且索引缺項才讀BASE Sprite，保留原RGB555／key字值並先驗整批矩形。Debug/Release各四實際x64測試EXE12過0敗0忽略，同43971/85319實際exit0、1015同來源887A9D54與source/test/raw封存讀回。原init/geometry/RGB555共4439bytes配五真IMG已隔離24完整平面/132calls核對，Rust bank對map303十二整frame完全吻合。完整scene gate、camera/viewport、clock、正常新主/GUI仍待；1005正常AFF5AF96/C7F36CC4與TW14保留原版資格，G01–G12未完成。

389byte原第一輪物件selector另已實跑263423案：262144全16bit旗標組（兩map、兩完整option組）＋1279針對header、shade、單項option與uint32 counter案。九條route及其實際引數逐項核對；普通4BE000僅map303傳gate低byte1，非此map為0。roof shade即使31仍走另一分支，不可憑不透明度推定gate。這是記錄callee的分派驗證，沒有執行完整物件像素或證明整個scene gate。只有私有尾端附加1byte ret，原389bytes除14允許重定位不改；所有定義的guard及輸入不變。

背景回呼先增相位再排64ms；static已找到直接開始、取消，以及取消後立即重啟路徑，所有具名直接phase寫入只在該callback。這尚未排除間接寫入，未執行生命週期，不能據此把phase在每次GAME歸零或做elapsed補步。原生LiTo/LTmap/303仍缺目錄，未把BASE map/303加入fallback；真IMG路由不代表地圖路由。

[Debug限定資格](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/background-bank-Debug-tests-20261016-123538-210580/qualified.json)；[Release限定資格](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/background-bank-Release-tests-20261016-123914-342973/qualified.json)。

[原真IMG完整背景平面](../驗證紀錄/current/native-background-real-original/20261016-123005-086741/qualified.json)；[原物件分派](../驗證紀錄/current/native-background-dispatch-original/20261016-124150-153377/qualified.json)。


## 2026-10-16 物件分派與RLE覆蓋來源

2026-10-16來源1019進行中：純Rust物件selector已對263423原route/參數/gate數值核對，Raster已接floor＋普通RLE物件寫入來源，保留TIL真header。修正版Debug23實際x64 EXE/24群135過0敗6既有opt-in忽略、source/test/raw獨立讀回，同58895實際exit0。Release及唯一入口正常兩主正沿同94643執行，來源/對應QA凍結；尚未當作通過。raw物件預覽仍列unqualified fragments，完整map303背景/viewport/clock未接，當版正常GUI未驗；1005 TW14僅保留原版資格。完整G01–G12未完成。

初輪同94991實際exit1：22整合104過，library Raster10過1 Hidden tone錯誤，world_scene群未執行；23實際測試EXE/source/原log與failed-scope-raw已封存讀回，未當合格。另依原first-pass的signed四byte header守衛覆核，raw半片的預覽rectangle不能代替原RLE覆蓋。修正Hidden在SPR提交時略過、Normal轉普通覆蓋，從原TIL資料表保存prefix並核對RLE解析header；raw物件只記未具資格，不產生假的完整gate。新增raw來源完整性案例，保留原畫面/命中、黑色覆蓋、屋頂weight31與special混色斷言。改後12項Raster與19項world_scene均過，六個既有opt-in仍未執行。

[修正版Debug限定資格](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-131624-113375/profile-qualified.json)；[初輪失敗保留](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-130245-955826/failed-scope-readback.json)。


## 2026-10-16 source1019正常入口與兩版回歸

2026-10-16來源1019：Debug/Release各23實際x64測試EXE、24群135過0敗6既有opt-in忽略與唯一入口正常兩主05F325C8/CD4F29A8已獨立讀回，同94643實際exit0。floor＋普通RLE物件write provenance及TIL真header已接Raster；raw物件預覽仍列unqualified fragments。當版TW14 GUI待啟動，來源與對應QA凍結；map303完整背景/viewport/clock與完整G01–G12未完成。1005 TW14保留原版資格。

Release保持z／ThinLTO／CGU16／strip symbols，兩版135/0/6與23實際測試映像、來源及raw封存核對；六項ignored名稱回查既有來源，未執行本文。兩次正常入口BuildOnly的原輸出（含嚴格UTF8或CP950 roundtrip）及PE AMD64/PE32+、features空集合、正常EXE封存均獨立讀回。這不是最新完整default/lib/TCP或正常GUI資格；接著做當版TW＋extended14。

[正常兩主資格](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified/qualified.json)；[限定Rust範圍](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified/scoped-qualified.json)。

## 2026-10-16 source1019 TW14完成及範圍

2026-10-16來源1019：唯一入口正常Debug **05F325C8**／Release **CD4F29A8**，各23實際x64測試EXE、24群135過0敗6既有opt-in忽略已獨立核對。當版TW＋extended GUI十四案全部完成：112實際sessions（56 LOGIN／56 GAME）、220次移動零多送、80原世界PNG、56原glyph及128舊通知；850份raw封存讀回，兩版新戰場PNG已目視。原12012實際exit0，原程序與自有workload均已退出。僅192DPI私有桌面控件訊息；實體輸入／多DPI、同端點、最新完整default/lib/TCP與完整G01–G12仍待。floor及普通RLE物件寫入來源已接Raster，raw物件預覽仍未具完整coverage資格；map303背景、viewport及clock尚未整合。

[十四案資格](../驗證紀錄/current/native-teleport-current/20261016-135449-330-TW-object-gate/qualified.json)；[終止後獨立讀回](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified/GUI-completion-observation.json)。完整default108＋main雙版本回歸已準備但未執行；整庫與完整TCP不包含在本次資格。

## 2026-10-16 source1019預設108整合與main雙版本完成

來源1019的108個預設整合目標加main unit雙版本回歸已獨立核對：Debug471過／Release467過，各0敗21既有忽略。兩版各109實際AMD64/PE32+測試映像、1019來源及各471份raw封存已逐項讀回；正常05F325C8/CD4F29A8不變，原29540實際exit0且相關程序全數退出。此資格包含預設整合與main unit，不包含整庫、ignored本文或完整TCP；完整G01–G12仍待。

[終止後獨立核對](../驗證紀錄/current/native-lighting-candidate/application-20261016-050159-638788/object-gate-tests-20261016-132623-487623/normal-qualified/default108-completion-observation.json)。
