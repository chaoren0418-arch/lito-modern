# E149 #45 地裂術：核心與共享資源候選

目前只做開發；桌面、測試、--list、遊戲／server／DB與音效播放均暫緩。

來源在 `candidates/native-E149-eruption45-core-20261024-085557/`，尚未接入場景，也未登記#45請求。既有30檔候選與正常1049編譯輸入保持原樣，請求來源識別仍59／剩121。

`native_eruption45.rs` 實作原class7→class6獨立節點：原生座標與u32 fixed16溢位、logical shift／even X、vector半X與remaining全X分開、原方向fold、source2才傳播、終點候選先刪再建child。每段立即source1、各自毫秒callback，晚到不追幀；parent先排程再巢狀建立下一段。source0保留240tick／10000ms。class7退役先檢查terminal，不重畫；class6則先提交source1再退役，最後提交須以同節點／occurrence確認。歷史只含中繼資料，無SPR／PCM。

音效只由Entered事件授權：129/source1 Sound128、129/source0 Sound129、123/source2 Effect3080是可重疊音效，123/source4才Hit。Hit不是HP授權，child anchor只在建立時由未來場景提供同incarnation解析；沒有本機傷害或消耗。

`native_eruption45_assets.rs` 借用Model的SharedActionCatalog，限定目前共享Text SHA523C61AD；逐一檢查129／130／123／124的原定義，129四方向bank及配對shadow同影格，123單bank。限定10 SPR與3 WAV／PCM的預期digest、索引／encoded／retained／音效預算與取消。舊TW載入器已移除，原預期SPR／WAV pin只是fail-closed條件；尚未實際讀取資源，不能移記TW資源資格。繪圖主圖原flags=0、shadow OR5的實際consumer整合仍待，不採一般Screen finite替代。

靜態證據：`native-E149-eruption45-core-inspection/20261024-085557-421299/inspection.json`。同PE14個範圍逐byte完整解碼；原始共享四定義與六個正式AttackSpell overload各自保存，沒有以同名覆蓋。分析PE不是可執行原版；仍須完整綁定#45的正式per-skill dispatch、S23、該分支傷害／成本／回覆，不能只用方法索引宣稱完成。

新增10個待跑回歸：終點候選無多餘音效身分、同點fallback、fixed16邊界、全域毫秒phase／晚到不追幀、child建立時重查後固定、超預算整份拒絕、倒退時鐘不消耗、class6最終提交、class7十秒terminal與無重入、clone接續occurrence。前155案全未跑；這165筆是已寫案例數，不是library inventory。

編譯工具只在既有完整362-module graph加兩個尚未使用的module，共364，原Window rental及controller五nested配對保留；兩版library／library-test共4個metadata命令，不連結或執行PE。前輪七技能整合命令沒有在此重跑，不擴張其當版資格。

接續：先整合S23 admission、世代／owner退役、metadata History、draw batch與typed audio occurrence，再完成child hurt的原排除條件／相對gfx綁定與#45正式入口。原圖庫base=1／5894／11297均未取得runtime資格，不代入猜測值。使用者明確恢復測試後才跑新回歸、共享資源、施法與GUI；完整library Debug／Release及G06／Release／GUI依總報告後續順序驗收。
