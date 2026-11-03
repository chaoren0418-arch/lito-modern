# S27 串接：原比較型別失敗與修正版

第一個串接版只有Debug library metadata編譯，退出1：Actor的PartialEq／Eq需要新增Tape具有相同trait（E0369／E0277）。原candidate、cohort、stderr與實際工具收據c4630f完整保存；不是測試本文失敗。首版沒有其餘17項或runtime資格。

修正版唯一額外Runtime差分為Tape加入PartialEq／Eq，重建五個Runtime覆寫及所有附加來源核對一致後，重新編譯完整18metadata。contract另外把supplement內容的修正放在post_supplement_changes，以便獨立重建來源；未藉型別修正變更玩法、正常1049來源、PE、正式JAR、registry或G06計畫。

新增案例仍10純、合前312共322案全未跑。S27 World／Animator／layers／History來源串接範圍與未資格的clock import／到達時刻／全域scheduler，沿native-E149-appearance27-binding.md；#67選單／C14與#68請求仍未完成。測試及桌面持續暫緩。
