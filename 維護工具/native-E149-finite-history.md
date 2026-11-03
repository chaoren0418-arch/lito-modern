# G06 有限光效來源退役接續

本輪只開發與編譯；測試本文、GUI、server／DB與播放仍暫緩。正常1049輸入、所有PE／正式JAR、registry、G06 fixture binding／16案640次均不改。

原View在同一world key切換資源root時，以for_source(false)隔離graph並合併晚到worker的admission；原Scene只為projectile／class1／Many等做這兩件事，四個通用finite profile卻在restore_other_effects=false時整份略過，晚到合併亦漏掉它們。因此同world的raw S37／S53仍留ring時可重新admit舊cue，不符合既有root ABA防重播設計。

修正Skills／Items／Reagent／Travel共同metadata History：同來源保留cursor，來源退役清active graph但保留generation／seen及計數；同generation晚到worker只合併seen，不導入其cursor／音效／資源。Scene四profile皆執行來源限定restore，活動序號納入原WorldView音效history保留，ring淘汰不過早釋一次性key。來源false仍不保留active音效，for_source(false)再true不能重新授權退休graph。generation／connection／map與publication fence沿原View，不新增資源或播放授權。

不改影格、聲音、位置、混色、server權威或現行64active／32音效／512fragments／4096keys上限。活動bank重備失敗沿原退役，待驗案例要求只記一次omitted並釋活動序號；最大共存／G11未驗。

新增8案例來源（7純＋1共享原230資源opt-in），與前267共275案全未跑，不是完整library inventory。請求來源仍68／剩112，沒有新增或驗收#62／#64；兩者為接續玩法。root ABA／晚到取消、原資源、所有四profile與最終GUI仍待恢復測試後驗，型別編譯不代替這些資格。
