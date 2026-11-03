# G06 原 StateCopy 前置條件：唯讀觀測候選

固定 E1498871 的 `NativeDeItemStateCopy.copy` 先掃描原 `NativeDeWorldKinds.Item` 的所有非靜態 declared fields，再呼叫 `copyFields`。兩個原拒絕訊息分別是 `active source item lifetime cannot be duplicated` 與 `pending source item expiry cannot be duplicated`。

原條件比「計時器仍在執行」更嚴格：

- 任一欄位的實際值是非空 `ScheduledFuture` 就拒絕，**不呼叫 isDone／isCancelled**。已完成或已取消但仍保留的參考不能自行當成可複製。
- declared type 的 simple name 是 `EquipmentTimerLease` 且非空，就拒絕；目前為 `_activeEquipmentTimer`、`_pendingEquipmentExpiry`。
- `_magicStoneExpirySubmissionInFlight` 或 `_magicStoneExpiryRetryScheduled` 為 Boolean.TRUE 就拒絕。
- `_pendingMagicStoneExpiryGeneration >= 0` 就拒絕，**包含零**；負值才不命中此條件。

新候選只改原唯讀 observer 一個 Java 檔。先核對靜態原始欄位清單，再反射讀完六十二個非靜態欄位；常態輸出七個已知條件欄，若其他欄位實際含 ScheduledFuture 也另記。保留完整 scan names／型別核對，不能只看兩個宣告為 Future 的欄位。只對 Future／lease 記錄參考身分，避免 primitive boxing 造成虛假的不穩定 ID。

每個物品在 native_attributes 加入 `state_copy_preconditions` sidecar。它只觀測宣告型別、實際 Future presence、lease presence、兩個 pending boolean 與 generation，**不呼叫 Future 狀態／cancel、計時器、遊戲 getter、StateCopy 或 setter**；setAccessible 僅作用於 Java Field metadata，沒有反射寫遊戲欄位。原八項永久欄、十四項診斷及十八個 record accessor 保留。

純 `native_E149_state_copy_preconditions.classify` 尚未呼叫。來源要求完整六十二欄清單、七個必要條件、正確型別／參考與觀測次序；以原 bytecode predicate 計算阻擋欄。回傳的 `clear_of_observed_blockers` 僅描述該次觀測，`copy_authorized` 永遠 false。記錄次序不是原方法真正執行的證據，兩次穩定讀取也不是原子授權。

原6fd6ad actual exit0／Java六class、兩類 javap 8454bb exit0；僅新 observer class 改變，其他 class 與原正式 fixture 分開核對。三個新 Python 工具只做 AST，分類函式／observer／計時器／copy／案例全部未執行。候選未選入 canonical binding／producer／reader／十六案；現行仍採原32＋新8、合計640次的未執行計畫，不移記新觀測資格。

下一步先查正常 GAME 如何產生原計時物品、真實 lease／pending 分支及清理，再補使用原物件的拒絕／自然到期矩陣與 observer／reader 綁定。不得造 timer／lease、反射寫欄位、注入時鐘或縮短租期。非預設擴充屬性、source／account、完整 owner／Main／G06及G01–G12仍待；桌面與測試繼續暫緩。
