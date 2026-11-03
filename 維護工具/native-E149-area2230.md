# E149 寒冰氣息22／巖牢30來源整合

本批接請求／選單、S39 class1與原資源呈現；全部只在隔離來源，沒有執行測試或GUI。#25 class5仍待，不以兩技能代替完整玩法。

正式E149：22為C14 02/05、MP20基值／reuse300ms／range6／radius3／gfx1804；30為03/05、MP24／reuse500／range8／radius3／1805。兩者後接unsigned actor ID與當前x/y；book、HP／MP>0、目標live／visible／非本人與原owner／revision守衛保留。成本、命中、傷害、實際range／遮擋與radius都由伺服器決定。名稱「寒冰氣息」「巖牢」只取相鄰唯讀靜態skills資料的文字；不執行SQL，也不以它替代正式E149協定。

- 只新增S39 action18／type8／1804或1805；原header、空字串、0額外byte與128目標上限不變，全decode成功才修改World。
- Cue新增target_origin，在S39接收時固定第一名target目前座標，不看其hit位；第一名缺席時從wire caster origin沿heading走原一步，不任選第二人，也不在caster Hit時重查。既有170／3934／758保留origin=None與原wire-origin規則。
- caster Hit啟動一組效果，整份hit／incarnation名單隨cue保留；每個原Hit只釋放目前有效的目標受擊呈現。位置凍結與後到／替換／死者、history cursor／場景退役守衛保留，不寫HP／MP。
- 1804原21frame／24FPS、Sound714在source1、Hit在source8；attr104=8為Screen。1805原17frame、Sound718在source2、Hit在source4及7；身體Over、1806 shadow與1807 Screen smoke必須全數載入。不同圖號或attrs／cadence變更即拒絕；不把兩次Hit壓成一次，也不把呈現事件當傷害權威。
- 714／718沿bank準備NativeAudioGroup／same-ID抑制，與非音效Hit分開。分層沿既有Plan／Bindings／Bank與submit_batch，完整預檢queue後提交；沒有臨時拼圖或通用效果fallback。

四Runtime差分：native_skills、native_skill_selection、native_world、native_attack_many。最新26檔：十二Runtime／十一Rust QA／三Java；保留#53四副本、class1位置更正與171→218。新增九案（其中一案需原資源），callback向量移除Sound只隔離位置／目標／clock，非原資源或伺服器證據。兩版library／library-test／七skill整合共18metadata只編譯；新9與前65案未跑。候選請求識別由49加22／30成51，180目錄尚129無入口；這不是施法／像素／音效或GUI通過率。

接續#25：S39 class5仍須接一個flight／218secondary與整份目標名單、原首目標snapshot及fallback、arrival／hurt時序、世代與history退役，再接14 03 00請求／選單。兩個本批技能待E149技能書、成功／低MP／reuse／range／遮擋、目標替換與離場、兩Hit、原像素／WAV與GUI實測。

正常1049輸入／所有PE、原registry與G06 binding／16案640次不變；候選未套用／選入、未連結或執行PE／Java／測試／GUI／播放。使用者暫緩維持，恢復後按總報告先元件／G06、當版完整library Debug／Release、Release與完整GUI自行確認後才交人工。
