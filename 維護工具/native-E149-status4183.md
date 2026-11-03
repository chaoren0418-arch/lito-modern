# #87 持續物件4183：條件時序、來源接線與未驗界線

本輪接續上一版 `native-E149-freezing80-shock87.md` 的4183缺口。原PE僅作資料讀取，九段完整函式共15496 bytes及逐指令輸出保存在 `驗證紀錄/current/native-E149-status4183-inspection/20261024-174320-375652/inspection.json`。原actor-green-clock工具只用來定位共用排程資料；其歷史執行資格沒有移記此物件。

在正式S0B的4183／walk0／speed0／brave0、其他角色、空名稱與空動作佇列條件下：4E7A40初始化frame0並呼叫4E02C0；目前action尚非3時排入584ms idle啟動。4E45F0→4E5B50先加frame才呈現，依1／2／3／4／5／0循環。4E34C0一般幀保留ideal deadline、過期remaining最低1ms，再加2ms；末幀0返回idle佇列等166ms後重啟，空閒模型首音584ms、其後1006ms週期。晚到poll每次最多派一個callback，最低3ms再派，不以elapsed直接取模。

原4183只有action3、六幀／4ticks、Screen及source1 `<1453`。專用Scene子模組處理原幀與typed overlap1453，不經一般角色的idle標籤與body flag守衛；沒有本機眩暈期限，也不修改HP／MP／S25控制。正式S08或新S0B incarnation取代才退役；有限4434仍只作命中特效。

同來源History保存callback位置／ideal deadline／loop occurrence；source false清活動圖形、保留admission，A→B→A不能復活舊物件。晚到worker只合併admission；每個S0B incarnation與原source1／loop occurrence構成typed音效身份。WorldView保留活動音效序號；目前frame、晚到完成與環境deferred音效均在現行World再核對lease，S08與新生命使舊音效失效。保留原128可見actor、2048 metadata、32 audio、512 fragments與4096音效History上限，沒有G11通過聲明。

**目前仍是條件來源模型。** warmup起點使用Model首次admission的現有時鐘；wire收到時間到Model admission、原clock import／初始化、u32 rollover、全域scheduler交錯及已有物件重送S0B時原佇列交錯尚未資格。首個靜態SPR subframe0取自原static lookup的推論，待原資源像素核對。現有World把每個S0B視為新incarnation，本輪依此重新建立生命與584ms warmup；不能宣告這些未驗邊界或完整87玩法已通過。

新增16案例（15純／1原資源opt-in）僅寫成來源，全未執行。涵蓋條件callback／晚到3ms／末幀、S25獨立權威、S08／S0B新生命、History／root ABA／晚到worker、重備失敗、原Screen／PCM型別與clone latch。初版library本體metadata退出0，library-test的E0425是資源案例缺函式限定路徑；原 `7b0bd7` actual exit1及兩原compiler紀錄完整保留，v2僅改該測試路徑，最終版本另行完整編譯，不移記初版成功。

使用者目前使用電腦；未執行測試本文／--list、GUI、fixture／server／DB、原PE或播放。metadata可編譯不等於原時序、原像素／聲音或完整library／GUI通過。接續88／89，並續查本節時序邊界、G06物品／source/account／timer與LT hurt/base；恢復測試後依總報告驗兩版完整library、Release、原資源、自然眩暈／傳送退役及完整GUI，整體G01–G12未完成。
