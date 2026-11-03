# G06 觀測補強的來源綁定

此輪只選入已完成 Java／javap 核對的 observer stability 候選；不合併另一個 StateCopy 前提 observer，不修改正常客戶端或任何 PE，不執行 fixture、reader、server、DB、guard、setter 或測試。

binding v3 新增 `observer_stability_extension` v1，明確引用原 morph qualification 與原 binding。共用驗證器仍先核對實際 javac cohort、四份 Java 輸入與六份 class，另要求只改 NativeRecoveryReadonlySnapshot.java、只改同名 observer class，其餘五份 class 完全一致。原 morph 候選必須仍直接延伸原八 setter 候選，原 32＋8 次方法清單與範圍不變。沒有此 extension 的舊綁定保留原檢查。

新綁定引用實際 b56d33 javac 結果與六 class，沿原完整十四檔 overlay；十一份 Rust 與兩版 controller 映像不變。只把更嚴格的唯讀取樣接入未來計畫，不把來源／編譯資格當作 runtime 通過。binding v3、case plan／execution v4、suite plan v3 的外層格式保留，延伸來源必須通過新 extension 的全部檢查。

重新產生 Debug／Release 八分支共十六案，每案 32 core＋8 morph、合計640筆的計畫；執行數仍為零。原十六案與原來源控制已封存，保留當時版本資格。新計畫釘住目前控制模組、Java classes 與新 binding；恢復測試後依新選定計畫，不使用舊計畫的來源 SHA 啟動新版控制器。

各先前開發紀錄中的「當時 binding／suite 未改」屬歷史事實。總報告工具會核對它們的原綁定、原 suite、來源封存檔與目前整合的前身關係，不要求歷史紀錄冒充最新綁定，也不改寫那些資格檔。正常來源、兩正常 PE、四元件 PE 與兩隔離正常入口 PE 都保留。

取樣仍非原子快照，不排除 ABA，也不取代 life.admits／requireActive。實際 G06、完整 owner、library Debug／Release、Release 回歸及完整 GUI 仍待使用者恢復測試後完成。
