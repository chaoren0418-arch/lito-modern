# 原生火牆與位置效果

本階段將火牆58接入現行 Windows x64 世界控制器、技能選單及正式 C14 地面請求。選擇已學會的火牆、點選地面，再按施放技能；選點必須在八格內，且不是自己的所在格。原有治癒／護盾／光箭與快捷鍵保留。其他座標技能尚未接入。

## 正式協定與伺服器權威

正式 JAR 的 SHA256 仍為 `589733435518D92C05BAEEAE37A686D384EC3F7B9897E975AD1EF50D67A74AA7`。`NativeGroundSkillOracle` 直接呼叫此 JAR 的 Protocol、BattleSpellCatalog 與 BattleGroundEffects：火牆 C14 是七個 bytes，`14 07 01 x:u16 y:u16`；技能58的動作18、gfx168、最大範圍8、基本MP60、reuse10ms、八段上限、傷害間隔1650ms與十秒有效期間均由 oracle 檢查。

火牆由伺服器決定朝向與可放置段數，依地圖邊界／方向性障礙停止。接受後送出 S1C 方向、S20 動作18、多個 S53 位置效果，以及 S4D MP。客戶端直接使用每個 S53 的座標，不自行補滿八段。S20 的自身動作18／19都能結束施法等待；S23光箭仍使用獨立的投射物分支。

Ground 選點保存連線、世界、位置校正及施法者 incarnation。技能清單模型與控制器在送出前再次檢查；已載入地圖邊界、存活、學會清單、目前MP、八格範圍、動作完成與等待狀態也會重新核對。原生控制器仍是唯一傳輸 owner。沒有本地扣魔、地面傷害定時器或自行修改HP。

## 原始效果播放

直接重新解碼 recovered PE `4271D92F…5C2AB6E9`，核對位置處理入口 `4ADF60 → 44EDE0 → vtable+4 / 44F350 → 44EF80`；C14 座標輸出見 `4C9270` 的 `ccchh` 分支。這是既有 recovered image 的獨立指令檢查，不是重新解包、執行舊BIN，也不把不同 hash 的 IDA image 當成相同程式。

位置效果直接啟動 class0。原始 Text/list.spr 的168/action0包含十一個來源影格，24 FPS；播放順序1..10、0，合計26 ticks，約1.083秒。終止0的兩個 ticks 仍保留，下一次回呼才退休。168/action3雖存在於資源，但這個 S53 入口沒有選用呼吸循環。僅讀共享 `Sprite/168-0.spr`，67468 bytes，SHA256 `254d10c15cdd1c8a250a8037c0cf39135e8b532a5beed6134c83320137295ba1`。

`native_ground_effects` 只處理已核實的 S53 gfx168，使用同一個 render camera 和原始 screen-family 參考混色。每個來源聲音事件111只進入一次；音效準備層依原始 WAV 的 PCM 長度，在呈現時鐘上抑制仍在播放期間的相同ID。原始 `Sound/111.wav` 是86520 bytes、SHA256 `424c8d1656370f0d45257f8b984d421aafd8911cbde644b994c67acd450fffab`。這項檢查不代表已驗收實際音效裝置／原 middleware。

效果最多64個同時播放；重複更新不重播，換世界退休，未支援或超出容量會明確計入 omitted。位置效果不跟隨物件ID，也不捏造 S_REMOVE_OBJECT；S53 本來就沒有可移除的物件ID。既有 S37/S53 其他效果的完整跟隨、終止影格及中斷語義仍需分別核實。

## 視覺期間與傷害期間

約1.083秒是這條原版 S53 畫面的播放期間；十秒是正式 BattleGroundEffects 的遊戲有效期間。兩者分開驗證。直接 JAR oracle 在1650ms之前沒有 pulse，1650ms有 pulse，9999.999999ms仍有效，10000ms清除。真正 ClientSession 案例也確認目標在畫面效果結束之後才收到正式傷害；客戶端只呈現伺服器送來的HP與受傷動作。

## 驗證與限制

七項新增 Rust 檢查涵蓋正式向量／無號座標、範圍與權威不變、S20/S53截斷原子性、失效選點／物件目標分流、class0終止停留、八段原始資源與音效去重、容量／換世界，以及WAV有效期間邊界與世界重設。離線資源取樣49幀、4張PNG。

兩個真正 ClientSession 情境使用隔離記憶體角色資料學會58，MP120／2，經正常走路到目標旁施法。正常案例要求MP120→60、正式放置1..8段、目標HP下降、全段播放並退休；低MP案例要求無動作／效果／扣魔，兩秒後 NoResponse，維持正式伺服器的沉默拒絕行為。沒有注入效果封包、改寫戰場HP、SQL或正式玩家資料。

最終103 Rust／40隔離TCP／19既有像素回歸，以及同一EXE、JAR、來源雜湊，見 schema20 回執與 `驗證紀錄/current/native-ground-skills/summary.json`。Release只做compile check；GUI／DPI／IME、音效裝置、其他技能、生命之泉耗材、完整buff與原版特殊中斷仍未完成。
