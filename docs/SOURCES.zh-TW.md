# 來源、版本與可重用程度

2026-10-14 現行參照：正式唯讀JAR與固定測試副本均為 `BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43`，以下其他JAR為歷史記錄。資源支援、現行程式雜湊及未完成項目整理於 `驗證紀錄/current/native-world-integration/resource-capability-matrix.json`。已可直接載入限定原始地圖／角色／圖示／NPC頁面，不表示所有原資源或完整遊戲均已通過。

2026-10-12 初始實際讀取與雜湊收據：[reference-pins.json](../驗證紀錄/20261012/reference-pins.json)。本任務未修改来源或複製共用遊戲資源；為避免測試期間的外部更新，只把一份小型正式 JAR 唯讀快照放在重建版 current。所有新測試產物在重建版。

| 來源 | 已核對身份／用途 | 可重用範圍／未知 |
| --- | --- | --- |
| `0-客戶端/LiTo.bin` | 1,941,568 bytes；SHA256 `CE6EE68ADA8FD37E6C096B1A27E46CB6888F1179F85DEFB5007AC60287360292` | 現行客戶端唯讀逆向依據；不能作新引擎執行核心 |
| `0-客戶端/TW13081901.bin` | 3,113,320 bytes，Hidden 屬性；SHA256 `1FE1D8F9529A0982CDA01A3EFBB127CA2B93248A6D7D5EB198A1A69D9707F4E8` | 天堂對照版本；本次未執行或注入。初始非 Force 列表漏列，已重新核實 |
| 初始觀測 SharedResources candidate JAR | 6,231,240 bytes；SHA256 `004B1BA8CC258F2F95A04ED3E6F1CBD63C1FE7737DC3DB5D6B4C0DE009130951` | 當時正式部署核心；Rust golden 向量來源，本輪只反組譯 Protocol/CipherState 與獨立呼叫 CipherState |
| 後續觀測同一路徑 JAR | 6,234,456 bytes；SHA256 `87EF4A65B8128851C32285578139E997E56B3EEBE0422BBC43F9815C14050BB6` | 12:49 外部更新；重新產生 24 組向量及 CipherState javap，與初始內容逐項一致。後續完整流程需以此新身份驗證 |
| 原生握手固定快照／本輪最後觀測 | 6,234,416 bytes；SHA256 `205DC1BAF5598523E67899E6C92A4B9BB07494D42DFA995CD741E67330D9B542` | 再次外部更新；固定唯讀快照重新生成版本契約及 24 組 cipher 向量，向量與初始一致。6 種實際 TCP Protocol 測試端往返已驗證；完整正式 ClientSession 待驗證 |
| 伺服器根目錄原始 JAR | 5,765,902 bytes；SHA256 `3025E3DF1C28F55019E79D5F21F9C1587B7268FB6417D6A75A591B6AECACB4B4` | 原始／診斷版本，不混作正式部署 |
| `0-客戶端` Text／Sprite／Tile、map4／Sound1539 | IDX、PAK、S32、WAV 均保存實際 SHA256 | 既有有界讀取、解码、CPU 合成與動作可重用；本次只驗證指定場景 |
| `0-客戶端/LiTo` | 現有 LTmap、LTsprite、LTtile、LTtext、LTsurf、LTicon 與 BMP | 本次目錄盤點；LT／BIN UI 的完整映射與正式來源優先順序未重新驗收 |
| `lito脫殼資源` | 原始逆向 contracts／resources／handoff／佇列保留在唯讀來源 | 可當證據候選，載入前需逐份版本／雜湊／格式核對，不照抄完成宣稱 |
| `LiTo381-Modern/src` | Rust 主程式、共享格式解碼器、舊 Modern 協定與 Native 候選混存 | 現行 x64 資源切片沿用驗證過的讀取／合成器；其餘完整遊戲接線待整理 |
| `1-伺服器` | 已確認原始專案、config、db、src、maps、jar 等目錄 | 本次未全面反組譯或重新建置；下一階段按協定／玩法具體需求唯讀查閱 |

正式加密相容測試用 `tests/oracle/FormalCipherOracle.java` 從上表正式 JAR 生成向量，Rust 測試直接使用既有 Native Sender／Receiver。覆蓋 4 個 seed、每序列 6 個封包、155-byte 上限、32-bit low word 進位、1-byte RX 分段與 3-byte TX 部分寫入。測試只證明該加密及框架子集，不證明登入、來源權威、封包全集或玩法。

新 JAR 的獨立輸出在 `驗證紀錄/20261012/formal-cipher-current.json`，比較收據在 `formal-cipher-drift-comparison.json`。Rust 測試保留原始來源 SHA，不改寫舊向量為新來源；新向量內容完全一致，因此可延用已驗證的加密子集結論，不能推論其餘部署行為未變。

同名共享 Sprite 在不同分包可能有不同版本。初期採明確 archive-qualified 來源與嚴格限制，禁止依檔案排序猜測優先權，禁止把 LT 編號當作共享 sprite/action ID。資源組字／色盤、透明度與技能映射未有證據時保留未知。

固定快照的新 oracle、反組譯與比較收據在 `驗證紀錄/20261012/native-handshake`；實際 socket 結果在 `驗證紀錄/current/native-handshake/receipt.json`。原始 golden 保持初始 JAR 身份，沒有把舊證據重新標成新版本。

後續相同固定快照增加 `NativeLoginOracle` 的中文 MS950 與 LOGIN／來源／轉址向量，以及真正 ClientSession 的 3 個記憶體驗證案例。JAR 本身沒有修改；隔離配置和記憶體帳號由本重建版建立，不是正式帳號／SQL 配置。收據在 `驗證紀錄/current/native-login/receipt.json`，只能證明 LOGIN endpoint 的已測路徑；完整 GAME／玩法仍未驗證。

## 前階段 E35CB91C 核心

2026-10-12 已將重建版隔離測試基準遷移至 6,239,420-byte E35CB91C43A18A690CC304DF8E5A9B2BA15014822261651616AE9420FA8CC5A2。現行 pin 為 `驗證紀錄/current/reference-snapshot.json`，八個驗證工具共同讀取它；原 `20261012/native-handshake/reference-snapshot.json` 保留歷史範圍。旧 JAR 保存為 target/current/server-reference/formal-server-205DC1BA.jar，不覆寫歷史來源宣稱。

新 JAR 八份封包／地圖 oracle 與原資料相同（除來源 hash）；24 組 cipher 另重新生成並相等。類別差異集中於 ClientSession 的技能目標與斷線結算，新增兩個 helper。記憶體 ClientSession 測試不建立 SQL repository、不啟動 Main，所有配置路徑留在重建版；目前相容驗證只涵蓋既有登入／房間／世界／移動子集，不涵蓋完整技能或 SQL 結算。證據見 current/native-performance/reference-migration.json、cipher-comparison.json 及最終建置收據。


## 近身戰鬥階段的核心更新

本輪先驗證 E35CB91C，再觀測 AE063385DCDF3898AEB003137F807E6C3365073AECE62B8531FB7E86886DCD06（6,242,453 bytes）。後者 6 個 class 差異及 ClientSession 方法比較記在 current/native-combat；已實際完成 30 TCP 案例。之後同一路徑又更新為 9C97F86821738CD4E039F20BF69CAE286BB1F095BDDA77E3305799A77F5C1A23，差異只有 ClientSession.useFoeSlayerSpell 內一處 sendEncrypted 被三個 pop 取代。建構、配置、帳號／狀態儲存與正常近身攻擊分支未變。

現行 pin 指向 9C97F868 的固定快照；9 個 oracle 契約（含新增 combat）及 24 組 cipher 向量均重新產生並比較相等。前兩核心的實測摘要獨立保存，最新實測狀態以 current-build-receipt.json 為準。未測的完整技能不能因類別差異範圍小而宣稱相容。
