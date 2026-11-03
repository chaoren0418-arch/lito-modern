# #71 藥水霜化術／#74 流星雨：來源開發界線

正式 E149 方法、material callback 與原 Text 定義已逐段唯讀核對，證據為 `驗證紀錄/current/native-E149-decay71-meteor74-review/20261024-165448-673361/review.json`。不載入遊戲 class，不執行原 PE。

- #71：C14 `[20,8,6] + u32 actor + u16 x/y`，限其他活體、三格；reuse／client clock 500ms。MP80、兩寶石、機率與16秒狀態均由server判定。S37只有接受狀態後才送2232，視覺21原幀、Screen、Sound838 source1；模型1952ms不代表狀態期限，不造本機圖示／計時器或成功ACK。音效保存實際cast key並使用world actor0，目標離場不會截掉已接受效果。
- #74：C14 `[20,9,1] + u32 actor + u16 x/y`，其他活體、十格，reuse3000ms。S39 action18／type8／762使用原class1第一目標收包快照；即使miss也取第一個，缺第一個或空清單沿wire caster origin＋heading一步，不取第二個。20原幀、Screen、Sound130 source1、Hit source4；模型123／2164ms。Hit只診斷，LT相對hurt仍未解。原start預設路徑，不套用#70四節點特例。

同來源History保留cursor／admission與活動音效序號；source退役與失敗重備只退役一次。保留原64 casts／32 audio／512 fragments等限制，不宣告G11資格。

新增14案（12純＋2原資源opt-in）僅寫成來源，全部未跑。metadata編譯不會連結或執行測試，亦不等於完整library Debug／Release或GUI通過。使用者目前使用電腦，所有測試、GUI、server／DB／fixture啟動及播放暫緩。

恢復測試後核對成本／低MP／材料／機率／拒絕／重施／自然16秒、第一目標missing／miss／離場／heading fallback、原資源像素／音效、History接續／來源退役／重備失敗，以及當版完整library兩版、Release與GUI。整體G01–G12及G06仍未完成。
