# E149 龍捲風53來源整合

在既有23檔來源上接通#53的請求／選單／S39 decode／原finite／音效及受擊名單，尚未套用正常來源或執行測試。#22／#25／#30仍待，不能把本批視為四技能完成。

- 原SELECTOR_ONLY／maximumRange0：C14只送14 06 04，不送角色ID或座標。選單顯示自己周圍四格，忽略不相關的選取目標，仍綁caster owner／connection／generation／correction；直接帶target的API拒絕。
- MP40、reuse1500ms、半徑4及其他傷害基值取正式E149。ResourceDatabase設定可覆寫成本；不以本地預扣、命中、血量或假成功取代伺服器權威。
- 只新增S39 action18／type0／gfx758、原空字串／額外byte0的受控profile。沿原完整decode後才改World、128名target上限、32bit unsigned ID、hit mask與target incarnation快照。零目標仍可有一組效果。
- 原758單bank／Undirected／20frame／24FPS／各3tick，sub19,0..18；Sound125與Hit均在source1，attr104=8。原定義變更即拒絕；不把其他class1／class5當成758。
- caster Hit只啟動一組效果；以原列表逐一釋放目前incarnation且hit=true的受擊呈現，不改HP／MP；已消失／替換／未知時後到的actor都不能取得舊回呼。同一cue不重播，沿原history cursor／換代退役與terminal dwell。
- 原Sound125由該bank準備為NativeAudioGroup／SuppressWhileSameIdPlaying，保留同frame Hit為非音效事件；不走新的臨時WAV讀取或重複播放。
- 原PE start758分支44F388另有三child＋original四副本，按方向3／7／5／1、距離2得到世界偏移(+2,+2)、(-2,-2)、(-2,+2)、(+2,-2)，投影為(+96,0)、(-96,0)、(0,+48)、(0,-48)。這是一組四圖，不是每victim再複製四圖；整組共用一個cursor／音效與受擊列表。新增完整anchor／queue預檢，四圖不能只送出一部分。

三Runtime差分：native_skills、native_skill_selection、native_attack_many；新增九案內嵌於many，其中一案需明確原資源根目錄。純callback單元向量移除Sound只為隔離身分／history，不是原資源、正式伺服器或GUI證明。

最新投影24檔：十Runtime／十一Rust QA／三Java；保留原火球171→218及所有先前技能／G06。兩版各library／library-test／七獨立skill整合只編譯18項metadata，新增9與前51個已撰寫案例均未跑；實際完整library數量須恢復後重新inventory。候選請求識別新增53為49個，當版180個目錄剩131個無入口；僅識別數，不是通過率。

下一步繼續#22／#25／#30的S39 class1／class5全列表、首目標anchor、原分層／音效／時序，再接請求與選單；當版#53另待技能書、伺服器接受／低MP／reuse／範圍與遮擋、空target、替換／退役／history、像素／Sound125及GUI。原21檔四PE／ZIP不含本批；最新24檔尚無PE。

使用者測試／桌面暫緩維持。正常1049輸入／兩EXE、原registry與G06 binding／16案640次不變；恢復後依總報告先元件／G06，再完整library Debug／Release、Release回歸與完整GUI，全數自行核對後才交人工。

首版單圖來源已完成18次metadata但未執行，原來源／cohort／template／工具仍保留；原PE新線索顯示單圖不完整，因此不將該批結果移記修正版。修正後需另跑兩版18metadata；全部仍不是實際測試或像素資格。
