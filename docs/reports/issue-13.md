# Issue #13 — T004-1 正式 JAR 協定常數靜態核對

核對日期：2026-11-05。分支：task/13。契約基準：此 worktree 的 e71f99982f358219fdbfead60afde3615f44afe9，docs/PROTOCOL-CONTRACT.md §1–§6。

## 結論與證據邊界

**已驗證（靜態）**：71 個在 §1–§6 以名稱直接搭配十六進位數值列出的 opcode，加上 §2 在封包形狀內指定值的 S_KEY，共 72 個全部一致。KEY_HIGH、版本常數、versionAccepted 全部欄位常數、五個 LoginCheckResult、兩個 LitoVsServerAvailability、max.frame 與兩層 login.bootstrap.sequence 均一致。指定的類別及常數成員皆存在，沒有「JAR 無此成員」的指定核對項。

**不一致 1 項（文字敘述，非數值）**：§6.2 將 NumericBreakthrough 說成「在啟動時讀環境變數」。JAR 的 schemaVersion() 每次呼叫均讀取系統屬性，屬性不存在才讀環境變數；enabled 等方法每次也會呼叫 schemaVersion()，沒有啟動時快取。有效範圍 1..4 與其他值回退 0 均一致。詳見下方完整 javap 片段。

本報告只證明列出的靜態項目；不代表 §1–§6 所有 parser、handler、執行時順序或端到端功能已驗證。讀取時機的差異也不代表實際伺服器一定會在運行途中改變 schema。

## 做了什麼／怎麼驗證

先讀 AGENTS.md、docs/REPORT.md、TASKS.md 的 T004、docs/plans/issue-4.json 與契約。依本次指派擴及 §6 與 NumericBreakthrough。原始專案與 JAR 均只讀；僅使用 Get-FileHash 與 javap 靜態分析，未載入執行 JAR 中的類別或呼叫其方法。

正式 JAR 路徑：

~~~text
D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar
~~~

Get-FileHash -Algorithm SHA256 實際計算值：

~~~text
E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498
~~~

符合指定的 e1498871…d99d498。工具為 Microsoft OpenJDK javap 17.0.17（C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe）。使用正式選項 -constants，並加 -p 顯示非 public 成員。

可於 worktree 根目錄重做的靜態指令（不啟動伺服器）：

~~~powershell
$jarPath = 'D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar'
Get-FileHash -LiteralPath $jarPath -Algorithm SHA256
New-Item -ItemType Directory -Force scratch/issue-13 | Out-Null
$classes = @(
  'Protocol', 'Protocol$CipherState', 'Protocol$LoginCheckResult',
  'Protocol$LitoVsServerAvailability', 'ServerConfig',
  'UnifiedConfiguration', 'NumericBreakthrough',
  'Protocol$Reader', 'Protocol$Writer'
)
foreach ($class in $classes) {
  & javap -classpath $jarPath -p -c -constants ('local.lito381.server.' + $class) |
    Set-Content -Encoding UTF8 -LiteralPath ('scratch/issue-13/' + $class + '.javap.txt')
  if ($LASTEXITCODE -ne 0) { throw "javap failed: $class" }
}
~~~

以下 bytecode offset 均指方法內偏移，與來源行號不同；表中的十六進位是 javap 十進位常數的等值轉寫。enum 核對的是建構子最後一個參數 wireValue，並非 enum ordinal。

## Opcode 逐項結果

來源均為 Protocol 的 static final int 宣告。S_KEY 的契約值取自 §2 的 c(0x01)。其餘 71 項由 §1–§6 的「名稱 + 0x數值」逐項擷取、去除相同重複列，與 javap 常數逐一比對。

| 成員 | 契約值 | JAR 值（十進位） | 判定 |
|---|---|---|---|
| C_ALIVE | 0x20 | 0x20（32） | 一致 |
| C_ATTACK | 0x17 | 0x17（23） | 一致 |
| C_CHANGE_DIRECTION | 0x09 | 0x09（9） | 一致 |
| C_CHAT | 0x77 | 0x77（119） | 一致 |
| C_FAR_ATTACK | 0x18 | 0x18（24） | 一致 |
| C_LITO_CONFIRM_DEATH | 0xE0 | 0xE0（224） | 一致 |
| C_LITO_CREATE_ROOM | 0xCC | 0xCC（204） | 一致 |
| C_LITO_ENTER_CHANNEL | 0xCA | 0xCA（202） | 一致 |
| C_LITO_EXIT_ROOM | 0xD0 | 0xD0（208） | 一致 |
| C_LITO_JOIN | 0xCF | 0xCF（207） | 一致 |
| C_LITO_LEAVE_CHANNEL | 0xCB | 0xCB（203） | 一致 |
| C_LITO_LIN_CHARACTER_INFO | 0xC8 | 0xC8（200） | 一致 |
| C_LITO_QUIT_FIGHT | 0xD9 | 0xD9（217） | 一致 |
| C_LITO_READ_NEWS | 0xE2 | 0xE2（226） | 一致 |
| C_LITO_READY_GAME | 0xD2 | 0xD2（210） | 一致 |
| C_LITO_ROOM_LIST | 0xCE | 0xCE（206） | 一致 |
| C_LITO_ROOM_USER | 0xCD | 0xCD（205） | 一致 |
| C_LITO_SELECT_CHAR | 0xC9 | 0xC9（201） | 一致 |
| C_LOGIN | 0x01 | 0x01（1） | 一致 |
| C_LOGIN_RESULT | 0x61 | 0x61（97） | 一致 |
| C_MOVE | 0x0A | 0x0A（10） | 一致 |
| C_READ_NEWS | 0x33 | 0x33（51） | 一致 |
| C_READ_NOTICE | 0x46 | 0x46（70） | 一致 |
| C_SAY | 0x13 | 0x13（19） | 一致 |
| C_SERVER_SELECT | 0x35 | 0x35（53） | 一致 |
| C_USE_ITEM | 0x1C | 0x1C（28） | 一致 |
| C_USE_SPELL | 0x14 | 0x14（20） | 一致 |
| C_VERSION | 0x00 | 0x00（0） | 一致 |
| S_ADD_INVENTORY | 0x16 | 0x16（22） | 一致 |
| S_ADD_INVENTORY_BATCH | 0x7C | 0x7C（124） | 一致 |
| S_ATTACK | 0x23 | 0x23（35） | 一致 |
| S_CHANGE_DIRECTION | 0x1C | 0x1C（28） | 一致 |
| S_EFFECT | 0x37 | 0x37（55） | 一致 |
| S_HIT_POINT | 0x0D | 0x0D（13） | 一致 |
| S_KEY | 0x01 | 0x01（1） | 一致 |
| S_LITO_CHANNEL_INFO | 0xCA | 0xCA（202） | 一致 |
| S_LITO_CHARACTER_INFO | 0xCB | 0xCB（203） | 一致 |
| S_LITO_CONFIRM_DEATH | 0xE3 | 0xE3（227） | 一致 |
| S_LITO_COUNTDOWN | 0xE1 | 0xE1（225） | 一致 |
| S_LITO_DELETE_ROOM | 0xCF | 0xCF（207） | 一致 |
| S_LITO_END_GAME | 0xD3 | 0xD3（211） | 一致 |
| S_LITO_ENTER_CHANNEL_RESULT | 0xC9 | 0xC9（201） | 一致 |
| S_LITO_EXIT_ROOM | 0xD0 | 0xD0（208） | 一致 |
| S_LITO_JOIN_RESULT | 0xCE | 0xCE（206） | 一致 |
| S_LITO_LOGIN_OK | 0xE7 | 0xE7（231） | 一致 |
| S_LITO_NEWS | 0xE6 | 0xE6（230） | 一致 |
| S_LITO_OBSERVER_BOX | 0xE2 | 0xE2（226） | 一致 |
| S_LITO_QUIT_FIGHT | 0xF0 | 0xF0（240） | 一致 |
| S_LITO_READY_GAME | 0xD1 | 0xD1（209） | 一致 |
| S_LITO_ROOM_INFO | 0xCC | 0xCC（204） | 一致 |
| S_LITO_ROOM_USER | 0xCD | 0xCD（205） | 一致 |
| S_LITO_SELECT_CHAR_RESULT | 0xC8 | 0xC8（200） | 一致 |
| S_LITO_SERVER_LIST | 0xFB | 0xFB（251） | 一致 |
| S_LITO_START_GAME | 0xD2 | 0xD2（210） | 一致 |
| S_LITO_TIMELIMIT | 0xE9 | 0xE9（233） | 一致 |
| S_LITO_VS_SERVER_IP | 0xF4 | 0xF4（244） | 一致 |
| S_LITO_VS_SERVER_STATUS | 0xF3 | 0xF3（243） | 一致 |
| S_LOGIN_CHECK | 0x00 | 0x00（0） | 一致 |
| S_MANA_POINT | 0x4D | 0x4D（77） | 一致 |
| S_MOVE_OBJECT | 0x12 | 0x12（18） | 一致 |
| S_NEW_ACCOUNT_CHECK | 0x02 | 0x02（2） | 一致 |
| S_NEWS | 0x38 | 0x38（56） | 一致 |
| S_NOTICE | 0x4B | 0x4B（75） | 一致 |
| S_NUM_CHARACTER | 0x03 | 0x03（3） | 一致 |
| S_PUT_OBJECT | 0x0B | 0x0B（11） | 一致 |
| S_REMOVE_OBJECT | 0x15 | 0x15（21） | 一致 |
| S_SAY | 0x13 | 0x13（19） | 一致 |
| S_SERVER_LIST | 0x3B | 0x3B（59） | 一致 |
| S_STATUS | 0x0C | 0x0C（12） | 一致 |
| S_TIME | 0x21 | 0x21（33） | 一致 |
| S_VERSION_CHECK | 0x3A | 0x3A（58） | 一致 |
| S_WORLD | 0x28 | 0x28（40） | 一致 |

文件縮寫 C_C8、S_C8、S_CB、S_D2、S_START_GAME、S_OBSERVER_BOX 分別指向表中的完整 opcode 名稱，不當作另六個 Java 成員。§6.2 另只提名稱的 S_ADD_SPELL=30（0x1E）、S_REMOVE_INVENTORY=23（0x17）、S_PING=120（0x78）均存在；文件未對這三者宣告數值，因此不計入 72 個數值比對。S_PING 出現在「不是 S_PING」的說明中。

## §1–§2 型別、金鑰與框架

| 比對項 | 契約 | JAR 靜態讀值／依據 | 判定 |
|---|---|---|---|
| c | u8 | Reader.c 遮罩 255；Writer.c 寫出低 8 bits | 一致 |
| h | u16 LE | Reader.h 將第二 byte 左移 8；Writer.h 低 byte 先寫（HP/MP 擴寬例外見 §6） | 一致 |
| d | u32 LE | Reader.d 位移 0/8/16/24 後 toUnsignedLong；Writer.d 同序寫出 | 一致 |
| LEGACY_TEXT／s 結尾 | MS950／單一 0x00 | Protocol 靜態初始化 Charset.forName("MS950")；Writer.s 寫入編碼 bytes 後補 0 | 一致 |
| WIDE_TEXT／S 結尾 | UTF-16LE／0x00 0x00 | Protocol 使用 StandardCharsets.UTF_16LE；Writer.S 檢查 wellFormedWideString 後補兩個 0 | 一致 |
| KEY_HIGH | 0x87546CA1 | Protocol.KEY_HIGH = 2270456993L | 一致 |
| CipherState 初值 | (0x87546CA1 << 32) OR unsigned(seed) | 建構子 offset 5 使用 -8695205541800050688L（位元樣式 0x87546CA100000000），offset 9 toUnsignedLong、12 lor | 一致 |
| 金鑰 byte 序 | 8 個 LE bytes | keyBytes 配置 8 bytes，每輪取低 byte 並無號右移 8 | 一致 |
| XOR 鏈 | 首 byte XOR k[0]；後續與前一密文及 k[i&7] XOR | encrypt offset 17–81；decrypt offset 17–81 | 一致 |
| state 推進 | 加 body.length；空 body 不推進 | encrypt offset 0–6／88–97；decrypt 0–6／92–101 | 一致 |
| header／total | u16 LE、含 2-byte header；2 ≤ total ≤ max.frame | readFrame offset 31–48 組合與檢查，67–71 讀取 total−2 | 一致 |
| 送出上限 | total ≤ 0xFFFF | writeFrame offset 0–9：body.length+2，與 65535 比較 | 一致 |

未將 §1 個別字串欄位的長度與控制字元政策、ClientSession 的連線生命週期納入此常數核對。

## §4 版本常數與 versionAccepted

| 比對項 | 契約 | JAR 值／證據 | 判定 |
|---|---|---|---|
| VERSION_MAGIC | 0x8033 | 32819 | 一致 |
| VERSION_PRODUCT | 950／0x3B6 | 950L | 一致 |
| VERSION_BUILD | 0x14F8 | 5368 | 一致 |
| VERSION_ACCEPTED | 0 | 0 | 一致 |
| VERSION_REJECTED | 1 | 1 | 一致 |
| supportedVersion 比對欄位 | magic、product、build；不比 fromCd | 方法 offset 1／11／22 讀取三欄並比較上述常數，無 fromCd 呼叫 | 一致 |
| C_VERSION 欄位／長度 | c h d c h，共 10 bytes，拒絕尾端資料 | versionRequest 同序呼叫 Reader，offset 52 檢查 remaining | 一致 |
| versionAccepted opcode | c(0x3A) | offset 7：58 | 一致 |
| versionAccepted result | c(0) | offset 12：0 | 一致 |
| versionAccepted minor | c(0x0D) | offset 16：13 | 一致 |
| versionAccepted 第 1 個 d | 0x0734FD33 | offset 21：120913203L | 一致 |
| versionAccepted 第 2 個 d | 0x0734FD30 | offset 27：120913200L | 一致 |
| versionAccepted 第 3 個 d | 0x77CF6EBA | offset 33：2010083002L | 一致 |
| versionAccepted 第 4 個 d | 0x0734FD31 | offset 39：120913201L | 一致 |
| versionAccepted 第 5 個 d | 0 | offset 45：0L | 一致 |
| versionAccepted 後續第 1 個 c | 0 | offset 49：0 | 一致 |
| versionAccepted 後續第 2 個 c | 0 | offset 53：0 | 一致 |
| versionAccepted country | c(3) | offset 57：3 | 一致 |
| versionAccepted serverType | d(0x00077D82) | offset 61：490882L | 一致 |
| versionAccepted body 長度 | 30 | 靜態加總 3 + 5×4 + 3 + 4 = 30 bytes | 一致 |
| versionRejected | c(0x3A) c(1) | offset 7：58；offset 12：1 | 一致 |

fromCd=0 是契約建議的客戶端送出值，不是 supportedVersion 的接受條件；本次未執行客戶端送包。

## §4 登入結果與伺服器可用性

| 成員 | 契約 wire 值 | JAR wire 值／靜態初始化 offset | 判定 |
|---|---|---|---|
| LoginCheckResult.ACCEPTED | 0x00 | 0；offset 7 | 一致 |
| LoginCheckResult.SYSTEM_ERROR | 0x05 | 5；offset 21 | 一致 |
| LoginCheckResult.ACCESS_FAILED | 0x08 | 8；offset 35 | 一致 |
| LoginCheckResult.ACCOUNT_BANNED | 0x1A | 26；offset 50 | 一致 |
| LoginCheckResult.ACCOUNT_IN_USE | 0x16 | 22；offset 65 | 一致 |
| LitoVsServerAvailability.AVAILABLE | 0 | 0L；offset 7 | 一致 |
| LitoVsServerAvailability.UNAVAILABLE | 0xFFFFFFFF | 4294967295L；offset 21 | 一致 |

## §2–§4 設定預設值

| 比對項 | 契約 | JAR 靜態讀值／依據 | 判定 |
|---|---|---|---|
| ServerConfig 的 max.frame 預設 | 8192 | load(Path,String) offset 281：8192 | 一致 |
| max.frame 最小值 | 64 | offset 284：64 | 一致 |
| max.frame 最大值 | 65535 | offset 286：65535；integer 方法檢查上下界 | 一致 |
| ServerConfig 的 login.bootstrap.sequence fallback | §4.2 明述包含 login_ok；正式設定不送 login_ok/time_limit | offset 315：login_ok,time_limit,classic_server_list,lito_server_list | 一致 |
| UnifiedConfiguration.apply 的 login.bootstrap.sequence | login_check → classic_server_list → lito_server_list；不含 login_ok/time_limit | offset 239：login_check,classic_server_list,lito_server_list | 一致 |
| UnifiedConfiguration.apply 的步驟間隔 | 250 ms | offset 250：字串 250，寫入 login.bootstrap.step.delay.ms | 一致 |
| UnifiedConfiguration.apply 的 advertised.host | game.bind | offset 294–304 取 game.bind 並設 advertised.host | 一致 |
| UnifiedConfiguration.apply 的 advertised.port | port | offset 309–318 取 port 並設 advertised.port | 一致 |

ServerConfig fallback 與 UnifiedConfiguration 覆寫值是兩個不同層次。契約沒有列出 fallback 的完整字串，本報告補錄實際值；沒有將它誤認為正式套用後的序列。此處驗證 apply 方法內的指定值，未讀取實機機密設定，也未驗證執行時載入結果或 ClientSession 實際送出的完整 bootstrap。

設定證據（javap 原文節錄）：

~~~text
ServerConfig.load(Path,String):
     278: ldc_w         #847                // String max.frame
     281: sipush        8192
     284: bipush        64
     286: ldc_w         #799                // int 65535
     289: invokestatic  #803                // Method integer:(Ljava/util/Properties;Ljava/lang/String;III)I
...
     312: ldc_w         #857                // String login.bootstrap.sequence
     315: ldc_w         #859                // String login_ok,time_limit,classic_server_list,lito_server_list
     318: invokestatic  #778                // Method value:(Ljava/util/Properties;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
     321: invokestatic  #863                // Method parseLoginBootstrapSequence:(Ljava/lang/String;)Ljava/util/List;

UnifiedConfiguration.apply(Properties,Path):
     236: ldc_w         #520                // String login.bootstrap.sequence
     239: ldc_w         #522                // String login_check,classic_server_list,lito_server_list
     242: invokevirtual #116                // Method java/util/Properties.setProperty:(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;
     245: pop
     246: aload_0
     247: ldc_w         #524                // String login.bootstrap.step.delay.ms
     250: ldc_w         #526                // String 250
     253: invokevirtual #116                // Method java/util/Properties.setProperty:(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;
~~~

## §6 NumericBreakthrough 與 schema

| 比對項 | 契約 | JAR 靜態讀值／依據 | 判定 |
|---|---|---|---|
| ENVIRONMENT | LITO_NUMERIC_BREAKTHROUGH | 同名字串常數 | 一致 |
| PROPERTY | lito.numeric.breakthrough | 同名字串常數；非 null 時優先於環境變數 | 一致 |
| schema 1 | HP/MP/AC 擴寬 | VITAL_ARMOR_SCHEMA_VERSION=1；enabled 判斷 schemaVersion()≥1 | 一致 |
| schema 2 | 六能力擴寬 | LEVEL_ABILITY_SCHEMA_VERSION=2；levelAbilityEnabled 判斷 ≥2 | 一致 |
| schema 3 | 有效版本，S_STATUS 與 schema 2 同寬 | MAGIC_MODIFIER_SCHEMA_VERSION=3；magicModifierEnabled 判斷 ≥3，characterStatus 不使用此旗標改欄位寬度 | 一致 |
| schema 4 | 四屬性抗性擴寬 | ELEMENTAL_RESISTANCE_SCHEMA_VERSION=4；elementalResistanceEnabled 判斷 ≥4 | 一致 |
| 有效範圍 | 1..4 | schemaVersion offset 31–46 檢查兩端 | 一致 |
| 未設定／超界／非整數 | 0 | null、範圍外或 NumberFormatException 都回傳 0 | 一致 |
| AC 範圍 | ±999999 | MIN_ARMOR_CLASS=-999999、MAX_ARMOR_CLASS=999999；characterStatus offset 186–199 同值檢查 | 一致 |
| S_HIT_POINT／S_MANA_POINT 的欄位寬度 | schema 0：h×2；schema ≥1：d×2 | Writer.c 第一 byte 為 13／77 且 enabled 時設 wideVitalFieldsRemaining=2；Writer.h 消耗此計數並改呼叫 d | 一致 |
| HP/MP body 長度 | 5／9 | 1+2×2=5；1+2×4=9，靜態推導 | 一致 |
| S_STATUS schema 0 長度 | 37 | characterStatus：固定頭10 + 六能力6 + HP/MP/AC 9 + 中段8 + 抗性4 | 一致 |
| S_STATUS schema 1 長度 | 48 | HP/MP/AC 由9變20：37+11=48 | 一致 |
| S_STATUS schema 2／3 長度 | 66 | 六能力由6變24：48+18=66 | 一致 |
| S_STATUS schema 4 長度 | 78 | 抗性由4變16：66+12=78 | 一致 |
| schema 讀取時機 | 「在啟動時讀環境變數」 | schemaVersion 每次呼叫讀取；enabled 等方法重新呼叫 schemaVersion | **不一致** |

上列長度為設定值維持不變時，依 javap 的 Writer 呼叫順序與分支加總，沒有執行 builder。schemaVersion 使用 Integer.parseInt，未 trim；空字串、純空白及非法整數均回退 0。系統屬性若存在但無效，會直接回退 0，不會再改採環境變數。這是契約可補充的精確規則。

### 不一致證據與建議修訂

建議 Claude 將「在啟動時讀」改為：「schemaVersion() 呼叫時先讀系統屬性 lito.numeric.breakthrough，若為 null 再讀環境變數 LITO_NUMERIC_BREAKTHROUGH；有效整數為 1..4，其他結果為 0。」啟動器是否固定傳入 4，需與啟動器自身證據分開描述。

以下為正式 JAR 的完整方法輸出，保留 offset 與常數池編號：

~~~text
  static int schemaVersion();
    Code:
       0: ldc           #9                  // String lito.numeric.breakthrough
       2: invokestatic  #11                 // Method java/lang/System.getProperty:(Ljava/lang/String;)Ljava/lang/String;
       5: astore_0
       6: aload_0
       7: ifnull        14
      10: aload_0
      11: goto          19
      14: ldc           #17                 // String LITO_NUMERIC_BREAKTHROUGH
      16: invokestatic  #19                 // Method java/lang/System.getenv:(Ljava/lang/String;)Ljava/lang/String;
      19: astore_1
      20: aload_1
      21: ifnonnull     26
      24: iconst_0
      25: ireturn
      26: aload_1
      27: invokestatic  #22                 // Method java/lang/Integer.parseInt:(Ljava/lang/String;)I
      30: istore_2
      31: iload_2
      32: iconst_1
      33: if_icmplt     45
      36: iload_2
      37: iconst_4
      38: if_icmpgt     45
      41: iload_2
      42: goto          46
      45: iconst_0
      46: ireturn
      47: astore_2
      48: iconst_0
      49: ireturn
    Exception table:
       from    to  target type
          26    46    47   Class java/lang/NumberFormatException

  static boolean enabled();
    Code:
       0: invokestatic  #30                 // Method schemaVersion:()I
       3: iconst_1
       4: if_icmplt     11
       7: iconst_1
       8: goto          12
      11: iconst_0
      12: ireturn
~~~

## 未驗證什麼

- 未跑 cargo check、cargo test、fixture、Java oracle 或任何動態測試；未啟動 JAR、遊戲、GUI 或連線正式服務／DB。
- 未讀取 config/server.properties 等機密設定、封存目錄或硬連結資源；未驗證實機目前生效值。
- 未讀取啟動器 C# 類別，因此契約所述 NumericBreakthroughEnabled=true 與 LocalServerService 傳入 schema 4，仍待啟動器證據確認；不能用此 JAR 的 schemaVersion 反證或證明啟動器行為。
- 未全面核對 §1–§6 的 handler 流程、欄位限制、其他 enum 結果碼、技能尾段映射、bootstrap 實際送包順序、無線上協商的全程性質或 §7 退出流程；上述均不得因本報告升級為完整驗證。
- **推測**：本報告沒有以來源版本相近或 patch 說明推測常數相同；所有「一致」皆有指定 JAR 的靜態依據。靜態相符不表示動態功能已通過。

## 產出路徑／給下一棒的提示

- 唯一納入 git 的產出：docs/reports/issue-13.md。
- 一次性整理腳本與完整 javap 輸出：scratch/issue-13/，已確認受 git ignore 排除，不納入提交；必要證據與重做指令已保留在本報告。
- docs/PROTOCOL-CONTRACT.md、docs/REPORT.md 與其他追蹤檔案均不修改。
- 請 Claude 審閱上述唯一文字不一致，修訂 §6.2 的讀取時機與屬性優先順序，並將契約 §8 的待驗證範圍精確縮減為本次未核對部分。
- T004-2 可引用本報告的 72 項 opcode 與常數結果；T004-3 可引用本報告作為正式 JAR 的靜態證據，保留舊 205DC1BA 動態驗證與本次靜態核對的區別。
- 調度器於合併後加入 docs/REPORT.md 索引；本任務不修改索引、不另拆 plan 任務。後續測試仍須依使用者授權安排。
