# 火球171循環飛行／218原生爆炸時序

本次為#25後續S39整合準備時序與原資源API，尚未接通請求、選單或世界派送。使用者正在使用電腦，測試與桌面操作暫緩。

只讀原PE4271D92F的class5 constructor／start／movement／timer及S39分支，再核對共用動畫函式4E5B50：每次movement遞增來源幀，超出數量歸0，兩條路都呼叫4E5C00。171原五幀因此依1、2、3、4、0循環；不是獨立24Hz finite timer，也不是末幀停留。原source1 Sound98的cue每五步再進入一次；是否實際發聲仍受原same-ID播放政策及後續音效派送影響，本輪沒有播放。

NativeAreaFireballFlight只持不可變path與Animation metadata：首步立即、每60ms一格，延遲只做一步並從實際dispatch重排，arrival只發布一次，時鐘倒退拒絕；八方向不折成四bank。Sample提供callback occurrence、source index、arrival及其後才能派送的171 cues。呼叫端必須先啟動arrival每位有效接收者的218，再派送171 cue；相鄰距離即99→98。尚無世界或音效派送接線。

171／218 exact-source-frame API沿原SPR／WAV bank；只為218提供native finite Clip，保留source0的41ms末幀停留，不新增原資源不存在的Hit。class5專用path允許u16起點及[-1,65536]fallback邊界；原public path入口與256格presentation上限保留。

初版曾誤用finite flight、停在source0；18metadata退出0也不能證明行為正確。初版candidate、三個工具／三個模板／說明、cohort、18份日誌與superseded-source側記完整保留；修正版獨立編譯，沒有沿用初版成功。先前class4／冰箭rational播放亦需另查其原分支，不能移記本次資格。

新增八項延後QA，其中一項需明示原資源根；涵蓋late／same poll／倒退、相鄰先secondary cue、零距離、218末幀停留、fallback與clone、八方向循環／cue重入、非法原定義及exact frame／profile fence。所有測試均未執行。Debug／Release各library、library-test及七項既有skill integration共18metadata，只在原程序actual exit0及來源／日誌讀回後記編譯成功。

下一步：#25 actor ID+x/y請求／六格／500ms與選單；S39 action18/type8/171保存fallback，launch只重查同incarnation首目標再凍結flight；arrival按原完整列表逐一解析，含miss／dead的secondary與有效hit的hurt分開，拒絕未知／替換角色，duplicates保留。整組anchor／fragment queue預檢、每個flight cue occurrence的音效身分、history及generation退役仍待接線。正常1049輸入／所有PE、registry與G06 binding／16案640次未變，識別仍51／180、129個請求缺口不減，沒有當版施法／資源／音效／GUI資格。
