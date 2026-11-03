# E149 #97 請求／#98 有限特效來源；97靜態圖尚待接入

97暗隱術請求／S34權威與98附加劇毒請求／原2942有限特效來源已接並編譯；均三byte selector-only，reuse9990／0，成本／材料／隱身及中毒皆server掌管。NativeCastTimeline實際bootstrap材料supplier與admission順序已核對；98九幀Sound1505 source1、有限1496ms／world音效，無本機320秒狀態。97 canonical 2941-0.SPR（865 bytes／一幀四refs）及0x460後備已綁包；零delay只入列、drain固定now／due排序已確認，仍缺active world的network與draw先後／97初次顯示及退役接線，不能宣告97完整。四Runtime仍42檔28 Runtime／11 Rust QA／3 Java、371module／原六nested；兩版18metadata退出0，原草稿4案（3純／1原資源opt-in）首次編譯，＋前455共459案全未跑；請求來源88／剩92，不是88技能完整。正常1049／既有PE／JAR／registry／G06640次不變，未套用／連結／測試／桌面／播放。

優先完成97靜態圖呈現與0ms callback退役：沿off_62C9C0的active WORLD函式核對network pump／draw與4D6570 drain順序，原clock import初始化／全域交錯仍未資格；不得假設無action無像素、固定一幀或用32秒本機刪除。97完成後才續目錄99／100與剩餘92請求，selector-only擴充共用helper並計入coverage parser。再續4183、G06物品／source/account／timer與LT hurt/base。使用者明確恢復測試後，驗97／98權威S34／S08現身、材料／低MP／HP邊界、原資源／音效／History與自然期限，並依總報告完成G06640次、完整library Debug／Release、Release及租裝等GUI。

正式review：`驗證紀錄/current/native-E149-hide97-venom98-review/20261024-191035-111212/review.json`，SHA256 4F03C35247CBA003BD29D4EDF8752FC452BD0B93CFE821A77314106907877B22。既有四Runtime草稿逐字重建後首次進此編譯圖，前草稿4案的未編譯／未執行收據保留，未移記selector80的18metadata；本輪獨立完整重編。

97靜態呈現明確未完成：不是無畫面、不是一般finite Clip，沒有固定一幀保證，也不設32秒本機期限。目前S37/2941仍走既有unsupported／omitted診斷；S34隱身、S08遠端移除、現身重送沿server權威。原程式已證明frame-count0退回source0讀靜態圖、44EF80置finished／排44F330，第一次callback退役；可見與否取決於ingress、timer drain與draw先後，active-world完整綁定仍缺。不得宣告97完整玩法或像素資格。

NativeSelfCastAction實際bootstrap→NativeLightAction.commitSelf／admitSelf→NativeCastTimeline.admitSelf的lambda$admitSelf$7→NativeSourceSkillCastAdmission.admit已核對；其material supplier實際綁lambda$admitSelf$6→NativeCastMaterial.commit。admission在22 preflight、35 reserve factory、53 material callback、72 reservation commit；失敗62／例外84 cancel。這是靜態caller鏈，不是whole GAME或ABA／原子資格。

正式Sprite.pak SHA256 4DD4E5348FBAD2E69BAE9CC388BE23D7BECEBE8423AD9F79F393C1457A69EE62、1904012586 bytes，讀取期間size／mtime不變；index SHA327C0637。2941-0.SPR SHA E1A842865E4CC1B61B1A6A9B43CD451DB97AEF515CBE6F5FAF9A74CE922CA080，一幀四refs；1120-0.SPR SHA1302F1D0；2942-0.spr SHA AF84AF8C0CE0BD7459545C6E94E1676C9DF2826C785802362848FDB0DD8ABAFC。只讀原始成員與headers，沒有解碼像素或執行資源probe；編譯只需所存快照及index／manifest pins，不反覆全檔讀取1.9GB pak冒稱runtime驗證。

原七函式第一批有六個range漏尾端（原ca70b2 exit0，讀回d04921發現），完整原檔保留並拒絕其完整函式資格；v2只擴正range，七段皆檢查末端ret、含對齊padding，579901 exit0。canonical取證首版parser命名間隔錯誤98db48 exit1／本文未執行，修正版a94a87 exit0。這些工具來源／範圍錯誤與遊戲測試分開封存，未藉此重啟或重試測試。

追加WORLD順序原始證據：`驗證紀錄/current/native-E149-hide97-world-order/20261024-192511-648815/world-order.json`，SHA1343A1C373470AB675B1799B4FC97D7E943BFA4B4C0C4BD39AF5AD1EEABA91D6。完整ASM六函式：47F340→4C6B30／loc4C7062→4B37C0→4B2970；active World經loc47CDB0→loc48AF60→488860。但4E2150另有兩次重入488860，不能只由外層timer-drain在active之前推斷97不畫；須先綁兩分支條件、callback queue與S37先後。此追加僅靜態來源，尚未納入97呈現或新runtime資格。
