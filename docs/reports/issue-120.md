# Issue #120 — T092-R2：de-native-* 建置鏈唯讀全讀

盤點日期：2026-11-06。執行分支：`task/120`。產出：`docs/reports/issue-120.md`。

## 已驗證

- 實際找到 **18 個目錄、62 個 `build*.py`**，與任務預期相同；其中 12 個目錄有符合檔案，6 個目錄為 0。
- **已讀檔數 62／62，全文合計 445,520 bytes、2,997 行**。62 檔均完整載入，保留全文、大小、SHA256 與行數，再從原路徑完整讀取位元組複核。共有 17 種不同的腳本文字；17 種全文均逐行審查，其餘 45 份在全位元組相等比對後沿用該文字解讀。沒有以檔名、頭尾抽樣或僅雜湊代替內容核對。附錄逐檔列出相等版本與各自的建置紀錄。
- 同一 Python 文字可能搭配不同 Java 輸入，產生不同 JAR；因此 **62 份快照的 manifest 各自讀取核對**，不把相同 builder SHA 當成相同建置結果。F01、F18、F26、F30、F40、F41 的歷史 manifest 為 `failed`，沒有 `runtime`，不能列為成功輸出。
- combat-continuation 根目錄 `build.py` 的指定輸入確實為 **`df9464f06c41821f3abf7c29ccc5876abe1ead66e570a40308f1274e90521a37`**；本次對實體 JAR 重新計算 SHA256，也得到同值。
- **`df9464f0` 是正式後續入口選取的基底，不是所有 de-native 候選的最末端。** 另有 magical-weapon → solid-carriage → thunder-teleport → triple-arrow → invisibility → retired-reuse → pair-material 分支，最後的 current 指標指向 `17502220…`；不能把它誤接成 combat-continuation 的 parent。
- 只新增本報告；沒有修改 TASKS、PLAN、REPORT、客戶端程式或伺服器 repo。沒有執行任何被盤點的 Python／Java／prepare／probe／verify 腳本，沒有建置、測試、遊戲／伺服器啟停或 DB 連線。執行的輔助程式只做唯讀列舉、文字與 JSON 解析、雜湊及本 worktree 報告生成。

## 路徑與證據記法

`B` 一律代表 `D:\lineage開發\3-lito開發\lito伺服器\LiTo381-Server\tools\resource-database\build\`，`S` 代表其伺服器根目錄 `D:\lineage開發\3-lito開發\lito伺服器\LiTo381-Server\`。下表目錄、附錄腳本與 manifest 路徑均相對 `B`，保留完整相對路徑，可直接還原本機絕對位置。

`Fxx build-名稱.py:行號` 指附錄該 Fxx 的完整路徑；`Fxx/Native*.java:行號` 指同一 Python 所在資料夾內的輔助 Java；`Fxx/sources/名稱.java:行號` 指其 `sources/` 子目錄。行號從 1 起算，不因報告排版改變。補充 Java 只讀取與影響判斷相關的檔案／段落，不列入 62 檔 Python 全讀計數。

輸入／輸出 SHA 若標「紀錄」，代表讀到歷史 manifest 的聲明，不等於本次重建通過。只有 §「df9464f0 交接」明列的實體 JAR 另做本次重新雜湊。腳本使用的 `B=Path(__file__).resolve().parent` 是原 builder 的執行位置約定；`inputs/`、`seed/` 的檔案是凍結副本，不能搬到快照位置直接執行。本報告依腳本與同次 manifest 解讀其來源。

## 18 個目錄與檔案數

| 相對 B 的目錄 | build*.py 數量 | 檔案 ID |
|---|---:|---|
| `de-native-cast-action-development/` | 0 | 無符合檔案 |
| `de-native-cast-material-development/` | 17 | F01、F02、F03、F04、F05、F06、F07、F08、F09、F10、F11、F12、F13、F14、F15、F16、F17 |
| `de-native-cast-timeline-development/` | 0 | 無符合檔案 |
| `de-native-cube-material-development/` | 8 | F18、F19、F20、F21、F22、F23、F24、F25 |
| `de-native-invisibility-cast-policy-development/` | 4 | F26、F27、F28、F29 |
| `de-native-item-skill-material-development/` | 4 | F30、F31、F32、F33 |
| `de-native-item-weapon-clock-development/` | 2 | F34、F35 |
| `de-native-magical-weapon-material-development/` | 4 | F36、F37、F38、F39 |
| `de-native-pair-material-development/` | 7 | F40、F41、F42、F43、F44、F45、F46 |
| `de-native-retired-reuse-timer-development/` | 3 | F47、F48、F49 |
| `de-native-reuse-lease-development/` | 4 | F50、F51、F52、F53 |
| `de-native-self-cast-development/` | 0 | 無符合檔案 |
| `de-native-skill-consumption-development/` | 0 | 無符合檔案 |
| `de-native-solid-carriage-policy-development/` | 3 | F54、F55、F56 |
| `de-native-thunder-teleport-policy-development/` | 3 | F57、F58、F59 |
| `de-native-triple-arrow-policy-development/` | 3 | F60、F61、F62 |
| `de-native-vital-writers-development/` | 0 | 無符合檔案 |
| `de-native-zero-mp-development/` | 0 | 無符合檔案 |
| **合計 18 個目錄** | **62** | **F01–F62** |

0 檔表示此目錄及子目錄沒有名稱符合 `build*.py` 的一般檔案，不表示該層沒有 JAR、Java、證據或其他命名的建置工具。未把 `B` 根目錄或其他 tools 目錄的腳本混入 62 檔；`combat-continuation/build.py` 是額外的交接證據。

## df9464f0 交接與候選分岔

### 本次核實的交接

| 證據 | 內容 |
|---|---|
| `S/tools/combat-continuation/build.py:5` | 讀取 `native-item-weapon-material-progress-current.json`，對 parent 與實體 JAR 同時要求完整 `df9464f0…21a37`。 |
| `S/tools/combat-continuation/build.py:6`、`:8` | 從 item-weapon-clock 的 `build-current.json` 取 manifest 及凍結的 `NativeLightAction.java`。 |
| `B/native-item-weapon-material-progress-current.json:3–5` | runtime 指向 `de-native-item-skill-material-development/builds/20261018T052807601508Z/candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`，SHA 為 `df9464f0…`。 |
| `B/de-native-item-weapon-clock-development/build-current.json:3–9` | manifest 指向上述 item-skill 的同次建置，manifest SHA 為 `fc067df4…dc1ec`；runtime 同為 `df9464f0…`。此 clock current 指標借用 item-skill 成品，並未指向 clock 目錄下另一個成品。 |
| `B/de-native-item-skill-material-development/builds/20261018T052807601508Z/manifest.json:7–9`、`:97–106` | base 為 `218d54c4…`，runtime 為 `df9464f0…`，改 `ClientSession`、`NativeLightAction`，加 `NativeItemWeaponCastAction`。對應 F32。 |
| 本次讀取該實體 JAR | 檔案大小 **19,734,523 bytes**；一般檔案，無 LinkType／ReparsePoint；重新計算完整 SHA256 為 `df9464f06c41821f3abf7c29ccc5876abe1ead66e570a40308f1274e90521a37`。未載入 JVM、未執行 JAR。 |

上述四個文字證據的本次 SHA256：

| 相對位置 | SHA256 |
|---|---|
| `S/tools/combat-continuation/build.py` | `2cac5761092951681a11295917ab9b7b880ef68574a23f8e2203797ea3349aeb` |
| `B/native-item-weapon-material-progress-current.json` | `a403d72bd8194b07abd6bab3a4d61b5fbfc074c47ae295e1d823317da97bea06` |
| `B/de-native-item-weapon-clock-development/build-current.json` | `8f84fdb95cc702bf47411e4b549c5724737d7a53c590b0939e62fdedef503f38` |
| `B/de-native-item-skill-material-development/builds/20261018T052807601508Z/manifest.json` | `fc067df4dfa511d74703ef966447e5a1e0b607f70d973323c61fe3bd2b8dc1ec` |

### SHA 血統圖（短值只作索引，完整值見逐檔摘要）

```text
self-cast 71ea214a
  └─ reuse-lease 35853617       F50–F53
      └─ cast-material a9c10856 F08／F13／F17 的選定版本
          └─ cube 218d54c4     F21／F23／F25 的計時版本
              └─ item-weapon df9464f0 F32／F34／F35
                  ├─ combat-continuation/build.py:5 → candidate-magical-weapon.jar
                  └─ de-native magical-weapon fb2e26b5 F37–F39
                      └─ solid-carriage 7c6a29ba F54–F56
                          └─ thunder-teleport 363ceed9 F57–F59
                              └─ triple-arrow c2a68fcc F60–F62
                                  └─ invisibility 7fbc49e7 F27–F29
                                      └─ retired-reuse 0269195c F47–F49
                                          └─ pair-material 17502220 F43／F44／F46
```

分岔的直接證據：F37 `build-native-magical-weapon-material.py:5–6` 也讀入 `df9464f0`，但 `:43` 輸出不同的 `candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`；combat-continuation `build.py:42–45` 則加 `NativeMagicalWeaponAction`／`NativeCastMaterial$Effect` 並輸出 `candidate-magical-weapon.jar`，兩者新增 class 都不相同。候選尾端指標見 `B/de-native-pair-material-development/build-current.json:4–9`，完整 SHA 為 `175022207c3c5b7954eb0f86edc01ec879f849314e978d2eee35d176961ded71`。

結論限定為「交接基底的 SHA 相符」。本次沒有把後續政策候選的行為直接宣告成 E149 正式行為，也沒有重新驗證 df9464f0 至 E149 的所有後續步驟。

## 客戶端影響與 task/92 對照

對照來源為 [PR #99](https://github.com/chaoren0418-arch/lito-modern/pull/99) 的 `task/92`，固定 commit **`aae229fea9b3c423fe55b45ca3ca3398147ab41f`**，[TASKS.md](https://github.com/chaoren0418-arch/lito-modern/blob/aae229fea9b3c423fe55b45ca3ca3398147ab41f/TASKS.md) blob **`acda4f5f4f09861f97a5e509cfbf9b0b05a111ba`**。本機 `origin/task/92` 當時較舊，已改以 GitHub 連接器讀取此 PR head 的 TASKS，未 checkout／改分支。

任務行號：T025:105、T026:106、T023:153、T045:173、T046:174、T047:175、T054:179、T055:184、T061:191、T057:271。下表「程式內容」是已驗證的靜態證據；對玩家實際行為的說明是**推測／待正式版本驗證**，不算功能通過。

| 面向／層 | 程式內容與檔名:行號 | 客戶端需核對的影響與任務歸屬 |
|---|---|---|
| opcode 與線上封包欄位 | F32 `build-native-item-weapon-material.py:31`；F54 `build-native-solid-carriage-policy.py:27`；F60 `build-native-triple-arrow-policy.py:27`；F57 `build-native-thunder-teleport-policy.py:27`；F40 `build-native-pair-material.py:65–66`：列明變動 class，未列 `Protocol` 或其巢狀 class。已讀成功 manifest 的 changed／fresh 聯集也沒有 `Protocol*`。 | **T025**：本批未找到直接改寫 opcode 數值或 Protocol 序列化器的證據；不能把新的 Java 方法參數誤認為新 wire 欄位。這是腳本／紀錄範圍的結論，非全 JAR 位元組逆向結論。 |
| 封包順序、延後輸出與舊角色抑制 | F37 `build-native-magical-weapon-material.py:21` 加 `accepted.binding.matches(pc)`；F40 `build-native-pair-material.py:17–18`、`:26–39` 在輸出與 receiver 綁定再驗 pair、角色世代及斷線。 | **T025、T023、T047**：同格式封包仍可能延後、被丟棄，換角／換場後不可用過期效果更新新角色；需核對效果、音效、MP 等權威輸出的順序。 |
| 自施、材料、技能結果 | F08 `build-native-cast-material.py:20`、`:29–30` 裝入 canonical material 層；F08/`NativeCastMaterialMethods.java:11–12` 列 CounterMagic、AbsoluteBarrier、PhysicalProcBuff、MagicResistance、ExoticVitalize、AdditionalFire、Invisibility、EnchantVenom、CounterMirror、ClearMind、ResistElemental、ElementalProtection、Awaken、Bloodlust 的 method 家族。 | **T026、T045、T046、T047**，各技能由 T020／T021／T022／T027／T028 執行：材料扣除、拒絕、buff、MP／背包更新須以權威結果為準。`actual_required_material_consumer=31` 是腳本欄位，不能解讀成全部 58 個材料技能都完成。 |
| reuse／冷卻 | F50 `build-native-reuse-lease.py:12` 指定消費者 2、3、14、99、151；`:32` 投影 Source delay、`:45–50` 接 shutdown／lifetime。 | **T026、T046、T023**：施法是否接納、連按冷卻及角色退出的殘留完成回覆；不是要求客戶端移植伺服器鎖或 scheduler。 |
| 立方體放置、材料與生存時間 | F21 `build-native-cube-material.py:17–32` 將放置與材料納入同一操作，`:45` 列 205／210／215／220，`:62` 改 GameHub 與四種 BattleCube Effects，`:71` 明列成功 admission 後取單調時間。F21/sources/`NativeCubeCastAction.java:9–24` 列四 ID 及 BATTLE／WORLD 的放置入口。 | **T026、T045、T046、T047**；失敗與素材診斷由 **T057** 承接。玩家看到的立方體出現、拒絕、到期與扣料需一致；本批沒有新增 GFX／資源檔的直接證據。WORLD 變體不因此升級到主線。 |
| 物品目標 12／107 與武器 buff 時計 | F32 `build-native-item-weapon-material.py:14` 分別列有材料 107、無材料 12；`:31` 改 ClientSession／NativeLightAction；F32/`NativeItemWeaponMaterialMethods.java:10`、`:19–22` 在 admission 後取時鐘，改接 previous／next inventory 快照。 | **T026、T045、T046**：使用物品目標、材料與背包差量、武器 buff 期限；這裡保留 player-buff bridge，不可誤稱是把物品實體改造成新武器。 |
| 技能 73 魔法武器與物品變更 | F37 `build-native-magical-weapon-material.py:12–20` 把 itemChange 放入材料交易，`:25` 列 73；F37/sources/`NativeMagicalWeaponCastAction.java:8–18` 取 `targetObjectId`、對 staged inventory 呼叫 `createMagicalWeapon`。 | **T026、T045、T046**：物品 objid、扣料與效果原子性、成功／失敗背包描述。此分支不是 combat-continuation 使用的同名實作；由 **T025** 保留血統限制。 |
| 堅固防護裝備條件 | F54 `build-native-solid-carriage-policy.py:12`、`:23–27` 接政策；F54/`NativeSolidCarriageMethods.java:12–15` 改 `hasSolidCarriageEquipment`／`CombatRules.snapshot`；F54/sources/`NativeSolidCarriagePolicy.java:6–17` 的 mode=1 接受盾／臂甲，其他 mode 只接受盾（原始 armorType 7／13）。 | **T026、T045、T046**：技能是否可用、裝卸後狀態；不把外控設定當客戶端自己判定的權威。 |
| 奪命之雷與傳送 | F57 `build-native-thunder-teleport-policy.py:21–27` 改 ClientSession、BattleTimedStatus、CombatRules；F57/`NativeThunderTeleportMethods.java:17–23` 改 5／131 admission 與 portal 建立分支；F57/sources/`NativeThunderTeleportPolicy.java:7–11` 在拒絕時送系統訊息、TELEPORT_UNLOCK、MP。 | **T023、T025、T026、T046**：拒絕可能沒有 portal 但仍須解鎖，需保持權威 MP 與座標狀態。131 的 WORLD 範圍仍遵守 TASKS 排除，不因此要求實作 WORLD。 |
| 三重矢傷害政策 | F60 `build-native-triple-arrow-policy.py:11`、`:21–27` 接政策；F60/`NativeTripleArrowMethods.java:14–39` 僅為三重矢建立三個 attack alias 並傳遞一次捕捉的倍率；F60/sources/`NativeTripleArrowPolicy.java:12–14` 在武器 type 4／13 使用 `max(existing,triple)`。 | **T026、T047**；彈藥既有範圍為 **T045**。此層直接變更的是傷害倍率路徑，不是新扣箭規則。HP／傷害／命中特效需與權威回覆一致，普通 attack producer 未被此工具改成新倍率。 |
| 隱身可見性 | F26 `build-native-invisibility-cast-policy.py:11`、`:21–27` 接政策；F26/`NativeInvisibilityCastMethods.java:5–19` 改 `revealInvisibility`／`nativeMaterialInvisibilityBody`；F26/sources/`NativeInvisibilityCastPolicy.java:7–9` 區分 spell 原因、重施與裝備 20077／120077／20062。 | **T025、T026、T047**，暗隱術另對 T020／T021：施法解除、重施不現身、裝備條件；不可把施法 reveal 政策外推到受傷、偵測、一般攻擊及自然到期。 |
| 技能 68 雙角色目標與退役 | F40 `build-native-pair-material.py:35–47` 保護 receiver／材料安裝；F43/sources/`NativePairCastAction.java:9`、`:16`、`:27–42` 限技能 68、以 object request 查同房／同世界目標、檢查雙角色 binding／hub／application。 | **T061、T026、T023、T047**：目標有效性、換房／退役不污染新角色，扣料前和延後回覆時都須守門。該 Python 本身沒有增加 LFC1 bytes；Ctrl wire 擴充仍按 T061 的獨立證據處理。 |
| STOPPED reuse timer | F47 `build-native-retired-reuse-timer.py:15–18` 只換 NativeReuseActor 並加 timer helper；F47/`NativeRetiredReuseMethods.java:6–7` 改 `pcSchedule`；F47/`NativeRetiredReuseTimer.java:5–17` 在 STOPPED 且 scheduler 關閉時回傳已取消 Future。 | 純伺服器 scheduler 機制按 TASKS「內部生命週期」排除；若確認 E149 承襲且有可達晚回覆，客戶端觀測對 **T023／T025**。本次沒有證據顯示須新增客戶端 scheduler。 |
| 房間 | F21 `build-native-cube-material.py:62` 雖改 GameHub，但 F21/`NativeCubeMaterialMethods.java:34–38` 指定的是 cube placement；F43/sources/`NativePairCastAction.java:27–30` 只是查 roomCombatants。 | **T054** 保留房間契約；本批沒有直接增加建房、密碼、隊伍設定、準備、踢人或結算規則的證據。不要僅因 GameHub 出現就新增房間任務。房內技能行為歸 T026／T047／T061。 |
| 租裝 | F08 `build-native-cast-material.py:29` 改 NativeDeItemApplication／Owner／Transaction；F08/`NativeCastMaterialMethods.java:59–71` 是 owner／adopt／nativeCall 的材料生命週期包裝。 | 與 inventory／owner 退役相關的部分對 **T023、T045**；**T055** 的租裝目錄、商店交接、返回及自然租期沒有在本批找到直接新增規則，不可宣告已補完。 |

### 「新」項目與歸屬

**未找到必須新增任務 ID 的客戶端功能：0 項。** 上述可見結果皆能落在 task/92 現有任務；以下是既有任務尚未明列的具體驗收細節，標「新」供審查者補充，**不增加主線或 M2／M3 張數，也不代表已核實 E149 有此行為**。

| 編號 | 新的具體核對點 | 建議歸屬與理由 | 證據 |
|---|---|---|---|
| 新 N1 | 技能 68 在扣料／安裝前目標已退役，以及延後輸出時 receiver 已換世代，應拒絕或丟棄，且不得扣錯世代材料。 | **T061＋T026** 加目標生命週期案例，跨場景清理接 **T023**；目前 T061 明列距離／死亡等一般合法性，未逐項寫 pair commit 與 receiver binding 的兩次邊界。 | F40 `build-native-pair-material.py:35`、`:43–47`；F43/sources/`NativePairCastAction.java:9`、`:32–42`。 |
| 新 N2 | 隱身重施在指定隱身裝備 20077／120077／20062 下的 toggle 例外，不只一般「重施不現身」。 | **T026＋T045＋T047** 的裝備／技能矩陣細列；先由 T025 確定正式版本是否承襲，避免拿研究候選當現行規格。 | F26/sources/`NativeInvisibilityCastPolicy.java:9`，由 F26 `build-native-invisibility-cast-policy.py:11`、`:24` 裝入。 |
| 新 N3 | 同一 builder SHA 可對應不同輸入 Java 與不同候選 JAR；`df9464f0` 的兩條 magical-weapon 後繼不能混為同一鏈。 | **T025** 補可追溯的 branch／manifest pin；屬既有版本綁定工作的證據補強，不是新玩法。 | F04–F08 相同 Python SHA 卻有五個輸出 SHA；F37 `build-native-magical-weapon-material.py:5`、`:42–43` 與 `S/tools/combat-continuation/build.py:5`、`:42–45`。 |

純伺服器的 Source 編譯閉包、JDBC 清理、鎖／FIFO／scheduler 實作不建立客戶端任務；若審查者要追其生產者，應交伺服器維護者，不能直接把 ASM 工具搬入 Rust。

## 推測

- 上述政策層若被某一正式 JAR 採用，會透過既有權威封包改變施法接納、扣料、buff 到期、HP／MP、可見性與傳送失敗恢復。此判斷源自靜態呼叫與修改邊界，沒有執行驗證。
- 現有任務足以承接本批的客戶端影響；N1–N3 建議只補既有矩陣／版本核對細節。是否納入 E149 的驗收列，由 #92 審查者依正式血統再裁定。

## 待驗證／未驗證什麼

- 62 份 builder 沒有重跑；其內 `verify`、`compare-c0`、inverse assertion 與歷史 manifest 的 `passed` 均只是讀到的程式／紀錄，不是本次驗證結果。
- 除 df9464f0 的實體檔案 hash 外，沒有重算所有歷史輸入／輸出 JAR，也沒有對每個 JAR 做 class 反組譯或逐 entry 差異重演。附錄其他 JAR SHA 均明確取自 manifest。
- 沒有確認候選 magical-weapon 至 pair-material 的政策在 E149 最終版是否用另一條鏈重新接入；不能把候選全部排除於 E149，也不能憑候選來源宣告已存在於 E149。
- 未執行任何網路遊戲互動、DB 查詢、GUI、cargo／Java 測試；未存取機密設定、封存目錄，未操作硬連結。0 個測試執行，不使用「略過＝通過」。

## 做了什麼／怎麼驗證／給下一棒的提示

1. 先讀本 worktree 的 `AGENTS.md`、`docs/REPORT.md`，確認既有 `task/120` 且開始時工作樹乾淨。
2. 以 `de-native-*` 直接子目錄為根，遞迴列出 `build*.py`；所有檔案全文載入、SHA256、大小、行數及完整位元組重讀比對，附錄逐項記錄。未執行被盤點腳本。
3. 讀每個快照的 manifest；checkpoint／reproduction 另外沿其 runtime 路徑回讀原建置 manifest，取得輸入 SHA 與 changed／fresh。副本與原建置的關係在附錄逐檔列明，沒有假設 checkpoint 自己又建了一個 JAR。
4. 用 Python 文字中的明確改寫／預期 class，加上相關 Java 輔助程式的靜態內容，判定客戶端影響；核對 PR #99 固定 head 的 TASKS。建置失敗列與成功紀錄分開。
5. 重新雜湊 df9464f0 的實體 JAR，核對 progress-current → clock build-current → 原 manifest → combat-continuation 入口。未執行重現建置。
6. **下一棒：#92 雲端 Claude 審查者。** 用本報告補上步驟 0 的本機未追蹤鏈缺口，尤其改正「全部 de-native 最終輸出＝df9464f0」的過度概括；採用本報告的兩分支說明與 N1–N3 既有任務細節，再裁定 PR #99。任何尚需驗證 E149 method 的工作，另交本機唯讀／測試任務；本次不排新測試、不改主線分母。

唯一追蹤產出是 `docs/reports/issue-120.md`。一次性盤點資料及報告生成輔助檔位於本 worktree 的 `scratch/`，不進 git；`docs/REPORT.md` 索引留給調度器合併後更新。

## 附錄：62 檔逐檔清單與四項摘要

每項包含路徑、bytes、全文行數、完整 SHA256、輸入 JAR、輸出 JAR、修改／新增 class、這一層用途及定位行號。class 名稱省略共同 package `local/lito381/server/` 及 `.class`，`$` 巢狀 class 保留。成功歷史紀錄的 class 清單取自原建置 manifest；失敗列僅寫腳本預期或工具目標，不能當成已產生的 class。

### F01 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/builds/20261018T012404133588Z/inputs/build-native-cast-material.py`。
- 大小：**4813 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F01（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`a8966dccf3dedad64105587940f110381bf2c5d3a1da8d58e6f692c6215f6fec`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：預定 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:29`）；此快照 manifest 為 `failed`，未記錄 runtime／輸出 SHA，不宣告成功產物。
- **修改的 class／方法**（靜態預期，無成功產物聲明）：ClientSession.useCounterMagicSpell／nativeMaterialCounterMagicBody；LegacyMySqlRepository.nativeMaterialCharacterConnection；編譯 NativeCastMaterial、NativeCastMaterialPolicy、NativeCastTimeline、NativeLightAction、NativeMaterialItem、NativeReuseActor、NativeSelfCastAction、NativeSourceMaterialPlan；入口 `build-native-cast-material.py:20`。
- **這一層在做什麼**：從 reuse-lease 基底接入施法材料與 canonical inventory 交易，早期未加 changed／fresh 精確集合斷言。
- 此快照紀錄狀態：`failed`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/builds/20261018T012404133588Z/manifest.json`；本次 SHA256 `f5d85334981a30386d31e116f35b0a0a3656bbe8b2cdd716bcc2bd5338db176a`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T012404133588Z/manifest.json`；runtime 所在位置：未記錄。

### F02 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/builds/20261018T012631300842Z/inputs/build-native-cast-material.py`。
- 大小：**4813 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F01（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`a8966dccf3dedad64105587940f110381bf2c5d3a1da8d58e6f692c6215f6fec`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:29`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`e23df4583e4c2acdfbe5d678ab5f834393bb2cf93dbb4498c3ae81f4d80f018e`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:20`。
- **這一層在做什麼**：從 reuse-lease 基底接入施法材料與 canonical inventory 交易，早期未加 changed／fresh 精確集合斷言。
- 此快照紀錄狀態：`passed-prototype-canonical-cast-material-build-real-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/builds/20261018T012631300842Z/manifest.json`；本次 SHA256 `38026a6d564ccbe3aa6b4b8f901a83b5369b5eb8d6dbcc003c98045edebff409`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T012631300842Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T012631300842Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F03 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/builds/20261018T013617908878Z/inputs/build-native-cast-material.py`。
- 大小：**5958 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F03（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`8de001ceb27c9e245b85c014da617844b75a03be62103136a338f2a27c532608`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`e23df4583e4c2acdfbe5d678ab5f834393bb2cf93dbb4498c3ae81f4d80f018e`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：將反魔法的材料消耗接入共用交易，新增精確 7 個 changed／13 個 fresh 及其餘 entry 不變的檢查。
- 此快照紀錄狀態：`passed-prototype-canonical-cast-material-build-real-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/builds/20261018T013617908878Z/manifest.json`；本次 SHA256 `415f7a680d208c28d17432fdcd9c49a64c934706b31f376aa48fe82fd6a9c765`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T013617908878Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T013617908878Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F04 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/builds/20261018T014241274633Z/inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`7de9584c7d364d7efc8bb357b7769c95ec8be573bbfe2442aebc9d8077b21dda`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`passed-prototype-canonical-cast-material-build-real-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/builds/20261018T014241274633Z/manifest.json`；本次 SHA256 `8f082de240e16b73c31b63aec30607c9626dfc5ed9cb26d00ca97d0767eb76d2`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T014241274633Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T014241274633Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F05 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/builds/20261018T020503181960Z/inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`f9104ae9b5d0004ab8a068ded369d3821159e9acfd3b4bbf106ab77324dcfc63`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`passed-prototype-canonical-cast-material-build-real-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/builds/20261018T020503181960Z/manifest.json`；本次 SHA256 `0d56b8306adc9be259f78aaf04e6c39c0f17c791968abd29a8ac5014f7c9d09b`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T020503181960Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T020503181960Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F06 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/builds/20261018T020657365023Z/inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`867580177413e15fc4f10cbf7fd060e8a5ffe4af04802e636d082ea471d8f7d8`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`passed-prototype-canonical-cast-material-build-real-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/builds/20261018T020657365023Z/manifest.json`；本次 SHA256 `13e72092b8387c1953a71cd58ff780f811b448f12262e32bc6b24e6bd4c15bd7`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T020657365023Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T020657365023Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F07 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/builds/20261018T021206388612Z/inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`adf4c3e89beb579bb61adaecb257bd67bebb936085f9c8fa61f596152077d3ee`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`passed-prototype-canonical-cast-material-build-real-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/builds/20261018T021206388612Z/manifest.json`；本次 SHA256 `e6cf17cf14be276710c030aaf58ae2113f2393ec9671b35b8160096f83b2c650`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T021206388612Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T021206388612Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F08 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/builds/20261018T024730005416Z/inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`passed-prototype-canonical-cast-material-build-real-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/builds/20261018T024730005416Z/manifest.json`；本次 SHA256 `eb41c6be4be0e94d58748d89e18f4aace6f4ebbb3ac7c756c2335ed3d0cb2a4f`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T024730005416Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T024730005416Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F09 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/progress-checkpoints/20261018T013839005159Z/inputs/build-native-cast-material.py`。
- 大小：**5958 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F03（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`8de001ceb27c9e245b85c014da617844b75a03be62103136a338f2a27c532608`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`e23df4583e4c2acdfbe5d678ab5f834393bb2cf93dbb4498c3ae81f4d80f018e`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：將反魔法的材料消耗接入共用交易，新增精確 7 個 changed／13 個 fresh 及其餘 entry 不變的檢查。
- 此快照紀錄狀態：`in-progress-prototype-required-material-counterMagic-real-handler-evidence`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/progress-checkpoints/20261018T013839005159Z/manifest.json`；本次 SHA256 `96e5c95461ccbe39235754aa26ba99f6d13d014e91b04a7daeee073c671d369e`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T013617908878Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T013617908878Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F10 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/progress-checkpoints/20261018T020253850829Z/inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`7de9584c7d364d7efc8bb357b7769c95ec8be573bbfe2442aebc9d8077b21dda`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`in-progress-prototype-required-material-counterMagic-real-handler-evidence`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/progress-checkpoints/20261018T020253850829Z/manifest.json`；本次 SHA256 `eefc017e76f951c5155873cfa1cf889b957007bfb6755a0096f21562b084ce8b`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T014241274633Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T014241274633Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F11 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/progress-checkpoints/20261018T024312906710Z/inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`adf4c3e89beb579bb61adaecb257bd67bebb936085f9c8fa61f596152077d3ee`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`in-progress-prototype-required-material-counterMagic-real-handler-evidence`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/progress-checkpoints/20261018T024312906710Z/manifest.json`；本次 SHA256 `8aaffdf61231048df833ac69427de7a558ffe0347118974b15b4166aad902630`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T021206388612Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T021206388612Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F12 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/progress-checkpoints/20261018T024331633206Z/inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`adf4c3e89beb579bb61adaecb257bd67bebb936085f9c8fa61f596152077d3ee`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`in-progress-prototype-fourteen-material-self-skills-real-handler-evidence`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/progress-checkpoints/20261018T024331633206Z/manifest.json`；本次 SHA256 `fce8a3dcfd32516798cb4a81c494368629013db333ceeab44890bfdcfb6c2657`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T021206388612Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T021206388612Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F13 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/progress-checkpoints/20261018T031843936510Z/inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`in-progress-prototype-eighteen-material-self-skills-real-handler-evidence`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/progress-checkpoints/20261018T031843936510Z/manifest.json`；本次 SHA256 `38996f0e2b4ab2ac552409ddc7a1fb74bf678976810bd0c91f4ed55e26542c05`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T024730005416Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T024730005416Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F14 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/reproductions/20261018T013627413461Z/inputs/seed-inputs/build-native-cast-material.py`。
- 大小：**5958 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F03（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`8de001ceb27c9e245b85c014da617844b75a03be62103136a338f2a27c532608`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`e23df4583e4c2acdfbe5d678ab5f834393bb2cf93dbb4498c3ae81f4d80f018e`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：將反魔法的材料消耗接入共用交易，新增精確 7 個 changed／13 個 fresh 及其餘 entry 不變的檢查。
- 此快照紀錄狀態：`passed-two-frozen-input-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/reproductions/20261018T013627413461Z/manifest.json`；本次 SHA256 `bc1627a5305dfc818f41e70af82b46ecd85dfbbf35e136ab92a3c187a132532d`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T013617908878Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T013617908878Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F15 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/reproductions/20261018T020143157053Z/inputs/seed-inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`7de9584c7d364d7efc8bb357b7769c95ec8be573bbfe2442aebc9d8077b21dda`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`passed-two-frozen-input-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/reproductions/20261018T020143157053Z/manifest.json`；本次 SHA256 `764e47d8d5d6d3385b85dc776cc77d2c97d88f1717acb41204363fbf7c14d8c7`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T014241274633Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T014241274633Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F16 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/reproductions/20261018T024205698977Z/inputs/seed-inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`adf4c3e89beb579bb61adaecb257bd67bebb936085f9c8fa61f596152077d3ee`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`passed-two-frozen-input-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/reproductions/20261018T024205698977Z/manifest.json`；本次 SHA256 `47842025c7370a7df847a7b2c4c8ece815fb2f6c5062b9a86a6bbed0772961bc`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T021206388612Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T021206388612Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F17 — build-native-cast-material.py

- 路徑：`de-native-cast-material-development/reproductions/20261018T031712646692Z/inputs/seed-inputs/build-native-cast-material.py`。
- 大小：**6042 bytes**；全文 **44 行**；讀取範圍 **1–44 全部**；全文版本：F04（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`c9300a1676b62fda2cb0beb51ac1168cb6dff1fe8908ce1d630b34aad565e08e`。
- **輸入 JAR**：`candidate-NATIVE-REUSE-LEASE-experimental.jar`；SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。腳本來源：`build-native-cast-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`（`build-native-cast-material.py:34`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CAST-MATERIAL-unqualified.jar`，SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`LegacyMySqlRepository`、`NativeCastTimeline`、`NativeDeItemApplication$Owner`、`NativeDeItemApplication`、`NativeDeItemTransaction`、`NativeLightAction$Batch`、`NativeLightAction`、`NativeReuseActor`、`NativeSelfCastAction`；新增 `NativeCastMaterial$Cast`、`NativeCastMaterial$ItemView`、`NativeCastMaterial`、`NativeCastMaterialPolicy$Source`、`NativeCastMaterialPolicy$State`、`NativeCastMaterialPolicy`、`NativeMaterialItem`、`NativeSourceMaterialPlan$ConsumeRequirement`、`NativeSourceMaterialPlan$Debit`、`NativeSourceMaterialPlan$ItemConsumeCommitResult`、`NativeSourceMaterialPlan$PlannedConsumption`、`NativeSourceMaterialPlan$Result`、`NativeSourceMaterialPlan`。腳本修改入口／集合：`build-native-cast-material.py:29`。
- **這一層在做什麼**：延伸自施材料管線的 owner／transaction 生命週期守門，預期 changed 擴為 10 個；實際涵蓋技能隨各快照 Java 輸入演進。
- 此快照紀錄狀態：`passed-two-frozen-input-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cast-material-development/reproductions/20261018T031712646692Z/manifest.json`；本次 SHA256 `886039c4d55bed219bc61f7691f3cfd188a5b0b6b2a28b171030d05edb4c5b46`。
- 輸入／class 清單原建置出處：`de-native-cast-material-development/builds/20261018T024730005416Z/manifest.json`；runtime 所在位置：`de-native-cast-material-development/builds/20261018T024730005416Z/candidate-NATIVE-CAST-MATERIAL-unqualified.jar`。

### F18 — build-native-cube-material.py

- 路徑：`de-native-cube-material-development/builds/20261018T032458745588Z/inputs/build-native-cube-material.py`。
- 大小：**9083 bytes**；全文 **65 行**；讀取範圍 **1–65 全部**；全文版本：F18（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`a43c599b70365ff7120ebb6ea656acfcf2b1a3b87450fcdd6a3401d2237aac06`。
- **輸入 JAR**：`candidate-NATIVE-CAST-MATERIAL-unqualified.jar`；SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。腳本來源：`build-native-cube-material.py:5`。
- **輸出 JAR**：預定 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`（`build-native-cube-material.py:55`）；此快照 manifest 為 `failed`，未記錄 runtime／輸出 SHA，不宣告成功產物。
- **修改的 class／方法**（靜態預期，無成功產物聲明）：ClientSession、GameHub、NativeCastMaterial、NativeCastMaterial$Cast、BattleCubeIgnitionEffects、BattleCubeQuakeEffects、BattleCubeShockEffects、BattleCubeHarmonyEffects；新增 NativeCubeCastAction（腳本第 54 行預期）；入口 `build-native-cube-material.py:54`。
- **這一層在做什麼**：把 205／210／215／220 立方體的放置與材料消耗納入同一 canonical 操作，此版尚未補 NativeLightAction 的 cube 接納分支。
- 此快照紀錄狀態：`failed`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cube-material-development/builds/20261018T032458745588Z/manifest.json`；本次 SHA256 `4c7033aff2841b42dfef746389d01372cc37dd6496844023b75247b3f79034e0`。
- 輸入／class 清單原建置出處：`de-native-cube-material-development/builds/20261018T032458745588Z/manifest.json`；runtime 所在位置：未記錄。

### F19 — build-native-cube-material.py

- 路徑：`de-native-cube-material-development/builds/20261018T032510535623Z/inputs/build-native-cube-material.py`。
- 大小：**9083 bytes**；全文 **65 行**；讀取範圍 **1–65 全部**；全文版本：F18（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`a43c599b70365ff7120ebb6ea656acfcf2b1a3b87450fcdd6a3401d2237aac06`。
- **輸入 JAR**：`candidate-NATIVE-CAST-MATERIAL-unqualified.jar`；SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。腳本來源：`build-native-cube-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`（`build-native-cube-material.py:55`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`，SHA256（紀錄）`a495f419485b5385130aabd47cf764e67f750efe7ffb46e3db0b2a79d98d1643`。
- **修改的 class／方法**（原建置 manifest）：修改 `BattleCubeHarmonyEffects`、`BattleCubeIgnitionEffects`、`BattleCubeQuakeEffects`、`BattleCubeShockEffects`、`ClientSession`、`GameHub`、`NativeCastMaterial$Cast`、`NativeCastMaterial`；新增 `NativeCubeCastAction`。腳本修改入口／集合：`build-native-cube-material.py:54`。
- **這一層在做什麼**：把 205／210／215／220 立方體的放置與材料消耗納入同一 canonical 操作，此版尚未補 NativeLightAction 的 cube 接納分支。
- 此快照紀錄狀態：`passed-cube-material-prototype-build-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cube-material-development/builds/20261018T032510535623Z/manifest.json`；本次 SHA256 `92d140c76559cdf668e48ced0d02653f4765120ff4ceef623fc9a24f59cefb67`。
- 輸入／class 清單原建置出處：`de-native-cube-material-development/builds/20261018T032510535623Z/manifest.json`；runtime 所在位置：`de-native-cube-material-development/builds/20261018T032510535623Z/candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`。

### F20 — build-native-cube-material.py

- 路徑：`de-native-cube-material-development/builds/20261018T033341639834Z/inputs/build-native-cube-material.py`。
- 大小：**9689 bytes**；全文 **67 行**；讀取範圍 **1–67 全部**；全文版本：F20（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`8eba0b805e0a71fb5a949ba27153d88c801441a82f8390c2cf7414aaa4fb5f3e`。
- **輸入 JAR**：`candidate-NATIVE-CAST-MATERIAL-unqualified.jar`；SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。腳本來源：`build-native-cube-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`（`build-native-cube-material.py:57`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`，SHA256（紀錄）`46c5bee56e732bc837ada08303ecde1fd494526dbc84844609b34a0c9a425cdd`。
- **修改的 class／方法**（原建置 manifest）：修改 `BattleCubeHarmonyEffects`、`BattleCubeIgnitionEffects`、`BattleCubeQuakeEffects`、`BattleCubeShockEffects`、`ClientSession`、`GameHub`、`NativeCastMaterial$Cast`、`NativeCastMaterial`、`NativeLightAction`；新增 `NativeCubeCastAction`。腳本修改入口／集合：`build-native-cube-material.py:56`。
- **這一層在做什麼**：在立方體材料放置層補 NativeLightAction 對 cube 的接納，保留原始材料 selection／operation 本體。
- 此快照紀錄狀態：`passed-cube-material-prototype-build-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cube-material-development/builds/20261018T033341639834Z/manifest.json`；本次 SHA256 `438c607819c7a21ddcbe98137471f443e4d152181bcf36f6ac3bfdf747b147a5`。
- 輸入／class 清單原建置出處：`de-native-cube-material-development/builds/20261018T033341639834Z/manifest.json`；runtime 所在位置：`de-native-cube-material-development/builds/20261018T033341639834Z/candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`。

### F21 — build-native-cube-material.py

- 路徑：`de-native-cube-material-development/builds/20261018T042438524701Z/inputs/build-native-cube-material.py`。
- 大小：**10873 bytes**；全文 **73 行**；讀取範圍 **1–73 全部**；全文版本：F21（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`44e1f5df2fd05d1ef27430f5eba1966543f253f16e09aed6be8072ce5d175daa`。
- **輸入 JAR**：`candidate-NATIVE-CAST-MATERIAL-unqualified.jar`；SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。腳本來源：`build-native-cube-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`（`build-native-cube-material.py:63`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`，SHA256（紀錄）`218d54c4c301fe16b7519fe1c57b5a7254ed6718a1ec0de1f7483427dac61086`。
- **修改的 class／方法**（原建置 manifest）：修改 `BattleCubeHarmonyEffects`、`BattleCubeIgnitionEffects`、`BattleCubeQuakeEffects`、`BattleCubeShockEffects`、`ClientSession`、`GameHub`、`NativeCastMaterial$Cast`、`NativeCastMaterial`、`NativeLightAction`；新增 `NativeCubeCastAction`。腳本修改入口／集合：`build-native-cube-material.py:62`。
- **這一層在做什麼**：在立方體接納／材料成功後、原 Cube 建立前取單調時間，並加入來源保存漂移的明確核對。
- 此快照紀錄狀態：`passed-cube-material-prototype-build-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cube-material-development/builds/20261018T042438524701Z/manifest.json`；本次 SHA256 `f7e75f79d3c111a74cef909c58865ba60960b8c20800ca4f77bb2a5467bcbe8f`。
- 輸入／class 清單原建置出處：`de-native-cube-material-development/builds/20261018T042438524701Z/manifest.json`；runtime 所在位置：`de-native-cube-material-development/builds/20261018T042438524701Z/candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`。

### F22 — build-native-cube-material.py

- 路徑：`de-native-cube-material-development/progress-checkpoints/20261018T041743275112Z/inputs/build-native-cube-material.py`。
- 大小：**9689 bytes**；全文 **67 行**；讀取範圍 **1–67 全部**；全文版本：F20（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`8eba0b805e0a71fb5a949ba27153d88c801441a82f8390c2cf7414aaa4fb5f3e`。
- **輸入 JAR**：`candidate-NATIVE-CAST-MATERIAL-unqualified.jar`；SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。腳本來源：`build-native-cube-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`（`build-native-cube-material.py:57`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`，SHA256（紀錄）`46c5bee56e732bc837ada08303ecde1fd494526dbc84844609b34a0c9a425cdd`。
- **修改的 class／方法**（原建置 manifest）：修改 `BattleCubeHarmonyEffects`、`BattleCubeIgnitionEffects`、`BattleCubeQuakeEffects`、`BattleCubeShockEffects`、`ClientSession`、`GameHub`、`NativeCastMaterial$Cast`、`NativeCastMaterial`、`NativeLightAction`；新增 `NativeCubeCastAction`。腳本修改入口／集合：`build-native-cube-material.py:56`。
- **這一層在做什麼**：在立方體材料放置層補 NativeLightAction 對 cube 的接納，保留原始材料 selection／operation 本體。
- 此快照紀錄狀態：`in-progress-prototype-twenty-two-material-skills-including-four-cubes`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cube-material-development/progress-checkpoints/20261018T041743275112Z/manifest.json`；本次 SHA256 `8bb7671910ef701baf1b9639cda037cc708b91fb41a410ffb01f3e138effa0ae`。
- 輸入／class 清單原建置出處：`de-native-cube-material-development/builds/20261018T033341639834Z/manifest.json`；runtime 所在位置：`de-native-cube-material-development/builds/20261018T033341639834Z/candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`。

### F23 — build-native-cube-material.py

- 路徑：`de-native-cube-material-development/progress-checkpoints/20261018T051029817743Z/inputs/build-native-cube-material.py`。
- 大小：**10873 bytes**；全文 **73 行**；讀取範圍 **1–73 全部**；全文版本：F21（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`44e1f5df2fd05d1ef27430f5eba1966543f253f16e09aed6be8072ce5d175daa`。
- **輸入 JAR**：`candidate-NATIVE-CAST-MATERIAL-unqualified.jar`；SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。腳本來源：`build-native-cube-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`（`build-native-cube-material.py:63`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`，SHA256（紀錄）`218d54c4c301fe16b7519fe1c57b5a7254ed6718a1ec0de1f7483427dac61086`。
- **修改的 class／方法**（原建置 manifest）：修改 `BattleCubeHarmonyEffects`、`BattleCubeIgnitionEffects`、`BattleCubeQuakeEffects`、`BattleCubeShockEffects`、`ClientSession`、`GameHub`、`NativeCastMaterial$Cast`、`NativeCastMaterial`、`NativeLightAction`；新增 `NativeCubeCastAction`。腳本修改入口／集合：`build-native-cube-material.py:62`。
- **這一層在做什麼**：在立方體接納／材料成功後、原 Cube 建立前取單調時間，並加入來源保存漂移的明確核對。
- 此快照紀錄狀態：`in-progress-prototype-twenty-two-material-skills-including-four-cubes`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cube-material-development/progress-checkpoints/20261018T051029817743Z/manifest.json`；本次 SHA256 `46b4028a9cefb6a52ce7fc2158b307dca97e4968df02195d930badbe898071bd`。
- 輸入／class 清單原建置出處：`de-native-cube-material-development/builds/20261018T042438524701Z/manifest.json`；runtime 所在位置：`de-native-cube-material-development/builds/20261018T042438524701Z/candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`。

### F24 — build-native-cube-material.py

- 路徑：`de-native-cube-material-development/reproductions/20261018T040922205062Z/inputs/seed-inputs/build-native-cube-material.py`。
- 大小：**9689 bytes**；全文 **67 行**；讀取範圍 **1–67 全部**；全文版本：F20（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`8eba0b805e0a71fb5a949ba27153d88c801441a82f8390c2cf7414aaa4fb5f3e`。
- **輸入 JAR**：`candidate-NATIVE-CAST-MATERIAL-unqualified.jar`；SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。腳本來源：`build-native-cube-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`（`build-native-cube-material.py:57`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`，SHA256（紀錄）`46c5bee56e732bc837ada08303ecde1fd494526dbc84844609b34a0c9a425cdd`。
- **修改的 class／方法**（原建置 manifest）：修改 `BattleCubeHarmonyEffects`、`BattleCubeIgnitionEffects`、`BattleCubeQuakeEffects`、`BattleCubeShockEffects`、`ClientSession`、`GameHub`、`NativeCastMaterial$Cast`、`NativeCastMaterial`、`NativeLightAction`；新增 `NativeCubeCastAction`。腳本修改入口／集合：`build-native-cube-material.py:56`。
- **這一層在做什麼**：在立方體材料放置層補 NativeLightAction 對 cube 的接納，保留原始材料 selection／operation 本體。
- 此快照紀錄狀態：`passed-two-frozen-input-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cube-material-development/reproductions/20261018T040922205062Z/manifest.json`；本次 SHA256 `1992251b38cbc709fecdb6091372bc38233862e5ffb3bd025e2c2a31777bff12`。
- 輸入／class 清單原建置出處：`de-native-cube-material-development/builds/20261018T033341639834Z/manifest.json`；runtime 所在位置：`de-native-cube-material-development/builds/20261018T033341639834Z/candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`。

### F25 — build-native-cube-material.py

- 路徑：`de-native-cube-material-development/reproductions/20261018T050615060107Z/inputs/seed-inputs/build-native-cube-material.py`。
- 大小：**10873 bytes**；全文 **73 行**；讀取範圍 **1–73 全部**；全文版本：F21（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`44e1f5df2fd05d1ef27430f5eba1966543f253f16e09aed6be8072ce5d175daa`。
- **輸入 JAR**：`candidate-NATIVE-CAST-MATERIAL-unqualified.jar`；SHA256（紀錄）`a9c108568d567ae6d4304201dfb69a34c3258502a375733650842880d513e955`。腳本來源：`build-native-cube-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`（`build-native-cube-material.py:63`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`，SHA256（紀錄）`218d54c4c301fe16b7519fe1c57b5a7254ed6718a1ec0de1f7483427dac61086`。
- **修改的 class／方法**（原建置 manifest）：修改 `BattleCubeHarmonyEffects`、`BattleCubeIgnitionEffects`、`BattleCubeQuakeEffects`、`BattleCubeShockEffects`、`ClientSession`、`GameHub`、`NativeCastMaterial$Cast`、`NativeCastMaterial`、`NativeLightAction`；新增 `NativeCubeCastAction`。腳本修改入口／集合：`build-native-cube-material.py:62`。
- **這一層在做什麼**：在立方體接納／材料成功後、原 Cube 建立前取單調時間，並加入來源保存漂移的明確核對。
- 此快照紀錄狀態：`passed-two-frozen-input-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-cube-material-development/reproductions/20261018T050615060107Z/manifest.json`；本次 SHA256 `6959a7756ee846a396d8deeab986cb6b30d60cfa8100601099350401d8f47447`。
- 輸入／class 清單原建置出處：`de-native-cube-material-development/builds/20261018T042438524701Z/manifest.json`；runtime 所在位置：`de-native-cube-material-development/builds/20261018T042438524701Z/candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`。

### F26 — build-native-invisibility-cast-policy.py

- 路徑：`de-native-invisibility-cast-policy-development/builds/20261019T183654750134Z/inputs/build-native-invisibility-cast-policy.py`。
- 大小：**6702 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F26（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`7a5aa8d98dfb5022c97bdb01304b768b61b48198fd8136ea78e5a9bd16021541`。
- **輸入 JAR**：`candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`；SHA256（紀錄）`c2a68fccea6a9a2f80cf288867bcf59a7c12c926173123e610e052499fa8c0d9`。腳本來源：`build-native-invisibility-cast-policy.py:7`。
- **輸出 JAR**：預定 `candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`（`build-native-invisibility-cast-policy.py:28`）；此快照 manifest 為 `failed`，未記錄 runtime／輸出 SHA，不宣告成功產物。
- **修改的 class／方法**（靜態預期，無成功產物聲明）：ClientSession、CombatRules.snapshot；新增 NativeInvisibilityCastPolicy（第 27 行預期）；工具目標 revealInvisibility、nativeMaterialInvisibilityBody；入口 `build-native-invisibility-cast-policy.py:27`。
- **這一層在做什麼**：對施法造成的隱身解除與隱身重施套用 Source 政策，僅置換 CombatRules.snapshot 並加入政策 helper。
- 此快照紀錄狀態：`failed`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-invisibility-cast-policy-development/builds/20261019T183654750134Z/manifest.json`；本次 SHA256 `79160f08fd440f72561332c344c92c0683160d179f3a67b8f0146dfeaa0c4589`。
- 輸入／class 清單原建置出處：`de-native-invisibility-cast-policy-development/builds/20261019T183654750134Z/manifest.json`；runtime 所在位置：未記錄。

### F27 — build-native-invisibility-cast-policy.py

- 路徑：`de-native-invisibility-cast-policy-development/builds/20261019T183704250111Z/inputs/build-native-invisibility-cast-policy.py`。
- 大小：**6702 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F26（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`7a5aa8d98dfb5022c97bdb01304b768b61b48198fd8136ea78e5a9bd16021541`。
- **輸入 JAR**：`candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`；SHA256（紀錄）`c2a68fccea6a9a2f80cf288867bcf59a7c12c926173123e610e052499fa8c0d9`。腳本來源：`build-native-invisibility-cast-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`（`build-native-invisibility-cast-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`，SHA256（紀錄）`7fbc49e745581aa3be40caae54d561010a43061a6d9f1362593b56d9fd9c5ed7`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`CombatRules`；新增 `NativeInvisibilityCastPolicy`。腳本修改入口／集合：`build-native-invisibility-cast-policy.py:27`。
- **這一層在做什麼**：對施法造成的隱身解除與隱身重施套用 Source 政策，僅置換 CombatRules.snapshot 並加入政策 helper。
- 此快照紀錄狀態：`passed-invisibility-cast-policy-build-native-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-invisibility-cast-policy-development/builds/20261019T183704250111Z/manifest.json`；本次 SHA256 `7b2345a6aef9b00bb282f771adbd5d3e94d8cdbe82ef0a894c09bf0af4679a62`。
- 輸入／class 清單原建置出處：`de-native-invisibility-cast-policy-development/builds/20261019T183704250111Z/manifest.json`；runtime 所在位置：`de-native-invisibility-cast-policy-development/builds/20261019T183704250111Z/candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`。

### F28 — build-native-invisibility-cast-policy.py

- 路徑：`de-native-invisibility-cast-policy-development/progress-checkpoints/20261019T184745562459Z/inputs/build-native-invisibility-cast-policy.py`。
- 大小：**6702 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F26（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`7a5aa8d98dfb5022c97bdb01304b768b61b48198fd8136ea78e5a9bd16021541`。
- **輸入 JAR**：`candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`；SHA256（紀錄）`c2a68fccea6a9a2f80cf288867bcf59a7c12c926173123e610e052499fa8c0d9`。腳本來源：`build-native-invisibility-cast-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`（`build-native-invisibility-cast-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`，SHA256（紀錄）`7fbc49e745581aa3be40caae54d561010a43061a6d9f1362593b56d9fd9c5ed7`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`CombatRules`；新增 `NativeInvisibilityCastPolicy`。腳本修改入口／集合：`build-native-invisibility-cast-policy.py:27`。
- **這一層在做什麼**：對施法造成的隱身解除與隱身重施套用 Source 政策，僅置換 CombatRules.snapshot 並加入政策 helper。
- 此快照紀錄狀態：`in-progress-Source-invisibility-cast-targeted-verified-full-goal-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-invisibility-cast-policy-development/progress-checkpoints/20261019T184745562459Z/manifest.json`；本次 SHA256 `a595f1b0cbccb67de35ea55e000bb0f2afe14d75c9a1fd384aeb9bd8f8307735`。
- 輸入／class 清單原建置出處：`de-native-invisibility-cast-policy-development/builds/20261019T183704250111Z/manifest.json`；runtime 所在位置：`de-native-invisibility-cast-policy-development/builds/20261019T183704250111Z/candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`。

### F29 — build-native-invisibility-cast-policy.py

- 路徑：`de-native-invisibility-cast-policy-development/reproductions/20261019T183957749253Z/inputs/seed/build-native-invisibility-cast-policy.py`。
- 大小：**6702 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F26（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`7a5aa8d98dfb5022c97bdb01304b768b61b48198fd8136ea78e5a9bd16021541`。
- **輸入 JAR**：`candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`；SHA256（紀錄）`c2a68fccea6a9a2f80cf288867bcf59a7c12c926173123e610e052499fa8c0d9`。腳本來源：`build-native-invisibility-cast-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`（`build-native-invisibility-cast-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`，SHA256（紀錄）`7fbc49e745581aa3be40caae54d561010a43061a6d9f1362593b56d9fd9c5ed7`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`CombatRules`；新增 `NativeInvisibilityCastPolicy`。腳本修改入口／集合：`build-native-invisibility-cast-policy.py:27`。
- **這一層在做什麼**：對施法造成的隱身解除與隱身重施套用 Source 政策，僅置換 CombatRules.snapshot 並加入政策 helper。
- 此快照紀錄狀態：`passed-two-frozen-invisibility-cast-policy-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-invisibility-cast-policy-development/reproductions/20261019T183957749253Z/manifest.json`；本次 SHA256 `5b87a429a4ff010212c2ed94ef441537e42b80987d1df4dc48e2ff75d548c510`。
- 輸入／class 清單原建置出處：`de-native-invisibility-cast-policy-development/builds/20261019T183704250111Z/manifest.json`；runtime 所在位置：`de-native-invisibility-cast-policy-development/builds/20261019T183704250111Z/candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`。

### F30 — build-native-item-weapon-material.py

- 路徑：`de-native-item-skill-material-development/builds/20261018T051702034602Z/inputs/build-native-item-weapon-material.py`。
- 大小：**5925 bytes**；全文 **42 行**；讀取範圍 **1–42 全部**；全文版本：F30（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`d40b9516d3b9614dd6c2b822a3132a18b8e221e371170833a0b1a12f165b46ad`。
- **輸入 JAR**：`candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`；SHA256（紀錄）`218d54c4c301fe16b7519fe1c57b5a7254ed6718a1ec0de1f7483427dac61086`。腳本來源：`build-native-item-weapon-material.py:6`。
- **輸出 JAR**：預定 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`（`build-native-item-weapon-material.py:32`）；此快照 manifest 為 `failed`，未記錄 runtime／輸出 SHA，不宣告成功產物。
- **修改的 class／方法**（靜態預期，無成功產物聲明）：ClientSession、NativeLightAction；新增 NativeItemWeaponCastAction（第 31 行預期）；工具目標 useItemWeaponEnchantSpell／nativeMaterialItemWeaponEnchantBody；入口 `build-native-item-weapon-material.py:31`。
- **這一層在做什麼**：將技能 12／107 的物品目標武器 buff 送入同一施法／材料／輸出管線，107 需材料、12 無材料。
- 此快照紀錄狀態：`failed`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-item-skill-material-development/builds/20261018T051702034602Z/manifest.json`；本次 SHA256 `32ae34d6a306b00bb10dd796839230f8f3c7035bb157b6dccc95867269dbcad6`。
- 輸入／class 清單原建置出處：`de-native-item-skill-material-development/builds/20261018T051702034602Z/manifest.json`；runtime 所在位置：未記錄。

### F31 — build-native-item-weapon-material.py

- 路徑：`de-native-item-skill-material-development/builds/20261018T051708303627Z/inputs/build-native-item-weapon-material.py`。
- 大小：**5925 bytes**；全文 **42 行**；讀取範圍 **1–42 全部**；全文版本：F30（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`d40b9516d3b9614dd6c2b822a3132a18b8e221e371170833a0b1a12f165b46ad`。
- **輸入 JAR**：`candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`；SHA256（紀錄）`218d54c4c301fe16b7519fe1c57b5a7254ed6718a1ec0de1f7483427dac61086`。腳本來源：`build-native-item-weapon-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`（`build-native-item-weapon-material.py:32`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`，SHA256（紀錄）`76f725ddeb8b359f952acdf50f191382e33186e43fadbae50cc9f568a1c6f485`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeLightAction`；新增 `NativeItemWeaponCastAction`。腳本修改入口／集合：`build-native-item-weapon-material.py:31`。
- **這一層在做什麼**：將技能 12／107 的物品目標武器 buff 送入同一施法／材料／輸出管線，107 需材料、12 無材料。
- 此快照紀錄狀態：`passed-item-weapon-material-prototype-build-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-item-skill-material-development/builds/20261018T051708303627Z/manifest.json`；本次 SHA256 `86a49d41774cc2ed62ed4ee798ea1b4928b4d5b087e3c33ef371097cd87f7214`。
- 輸入／class 清單原建置出處：`de-native-item-skill-material-development/builds/20261018T051708303627Z/manifest.json`；runtime 所在位置：`de-native-item-skill-material-development/builds/20261018T051708303627Z/candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`。

### F32 — build-native-item-weapon-material.py

- 路徑：`de-native-item-skill-material-development/builds/20261018T052807601508Z/inputs/build-native-item-weapon-material.py`。
- 大小：**5995 bytes**；全文 **42 行**；讀取範圍 **1–42 全部**；全文版本：F32（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`b9891cb448d1989852c2a747482e7f4aac8c716d66be82b5c574a80e99934946`。
- **輸入 JAR**：`candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`；SHA256（紀錄）`218d54c4c301fe16b7519fe1c57b5a7254ed6718a1ec0de1f7483427dac61086`。腳本來源：`build-native-item-weapon-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`（`build-native-item-weapon-material.py:32`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`，SHA256（紀錄）`df9464f06c41821f3abf7c29ccc5876abe1ead66e570a40308f1274e90521a37`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeLightAction`；新增 `NativeItemWeaponCastAction`。腳本修改入口／集合：`build-native-item-weapon-material.py:31`。
- **這一層在做什麼**：武器 buff 材料建置支援 --clock-candidate 選擇 timed-build-current 指標，配合此快照 Java 在成功接納後取武器 buff 時計。
- 此快照紀錄狀態：`passed-item-weapon-material-prototype-build-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-item-skill-material-development/builds/20261018T052807601508Z/manifest.json`；本次 SHA256 `fc067df4dfa511d74703ef966447e5a1e0b607f70d973323c61fe3bd2b8dc1ec`。
- 輸入／class 清單原建置出處：`de-native-item-skill-material-development/builds/20261018T052807601508Z/manifest.json`；runtime 所在位置：`de-native-item-skill-material-development/builds/20261018T052807601508Z/candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`。

### F33 — build-native-item-weapon-material.py

- 路徑：`de-native-item-skill-material-development/reproductions/20261018T060051536241Z/inputs/seed-inputs/build-native-item-weapon-material.py`。
- 大小：**5925 bytes**；全文 **42 行**；讀取範圍 **1–42 全部**；全文版本：F30（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`d40b9516d3b9614dd6c2b822a3132a18b8e221e371170833a0b1a12f165b46ad`。
- **輸入 JAR**：`candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`；SHA256（紀錄）`218d54c4c301fe16b7519fe1c57b5a7254ed6718a1ec0de1f7483427dac61086`。腳本來源：`build-native-item-weapon-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`（`build-native-item-weapon-material.py:32`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`，SHA256（紀錄）`76f725ddeb8b359f952acdf50f191382e33186e43fadbae50cc9f568a1c6f485`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeLightAction`；新增 `NativeItemWeaponCastAction`。腳本修改入口／集合：`build-native-item-weapon-material.py:31`。
- **這一層在做什麼**：將技能 12／107 的物品目標武器 buff 送入同一施法／材料／輸出管線，107 需材料、12 無材料。
- 此快照紀錄狀態：`passed-two-frozen-input-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-item-skill-material-development/reproductions/20261018T060051536241Z/manifest.json`；本次 SHA256 `670905c192e87ca703c8ab64c46434fb1f4917fdcefb7ebdc583dda3618c3035`。
- 輸入／class 清單原建置出處：`de-native-item-skill-material-development/builds/20261018T051708303627Z/manifest.json`；runtime 所在位置：`de-native-item-skill-material-development/builds/20261018T051708303627Z/candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`。

### F34 — build-native-item-weapon-material.py

- 路徑：`de-native-item-weapon-clock-development/progress-checkpoints/20261018T063928148456Z/inputs/build-native-item-weapon-material.py`。
- 大小：**5995 bytes**；全文 **42 行**；讀取範圍 **1–42 全部**；全文版本：F32（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`b9891cb448d1989852c2a747482e7f4aac8c716d66be82b5c574a80e99934946`。
- **輸入 JAR**：`candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`；SHA256（紀錄）`218d54c4c301fe16b7519fe1c57b5a7254ed6718a1ec0de1f7483427dac61086`。腳本來源：`build-native-item-weapon-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`（`build-native-item-weapon-material.py:32`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`，SHA256（紀錄）`df9464f06c41821f3abf7c29ccc5876abe1ead66e570a40308f1274e90521a37`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeLightAction`；新增 `NativeItemWeaponCastAction`。腳本修改入口／集合：`build-native-item-weapon-material.py:31`。
- **這一層在做什麼**：武器 buff 材料建置支援 --clock-candidate 選擇 timed-build-current 指標，配合此快照 Java 在成功接納後取武器 buff 時計。
- 此快照紀錄狀態：`in-progress-prototype-twenty-three-material-skills-including-weapon-buffs`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-item-weapon-clock-development/progress-checkpoints/20261018T063928148456Z/manifest.json`；本次 SHA256 `cfb02dc7385165e3b4cfd45955e996ced92099b910b223fe8aed6dbf55bc7788`。
- 輸入／class 清單原建置出處：`de-native-item-skill-material-development/builds/20261018T052807601508Z/manifest.json`；runtime 所在位置：`de-native-item-skill-material-development/builds/20261018T052807601508Z/candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`。

### F35 — build-native-item-weapon-material.py

- 路徑：`de-native-item-weapon-clock-development/reproductions/20261018T063642118412Z/inputs/seed-inputs/build-native-item-weapon-material.py`。
- 大小：**5995 bytes**；全文 **42 行**；讀取範圍 **1–42 全部**；全文版本：F32（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`b9891cb448d1989852c2a747482e7f4aac8c716d66be82b5c574a80e99934946`。
- **輸入 JAR**：`candidate-NATIVE-CUBE-MATERIAL-unqualified.jar`；SHA256（紀錄）`218d54c4c301fe16b7519fe1c57b5a7254ed6718a1ec0de1f7483427dac61086`。腳本來源：`build-native-item-weapon-material.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`（`build-native-item-weapon-material.py:32`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`，SHA256（紀錄）`df9464f06c41821f3abf7c29ccc5876abe1ead66e570a40308f1274e90521a37`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeLightAction`；新增 `NativeItemWeaponCastAction`。腳本修改入口／集合：`build-native-item-weapon-material.py:31`。
- **這一層在做什麼**：武器 buff 材料建置支援 --clock-candidate 選擇 timed-build-current 指標，配合此快照 Java 在成功接納後取武器 buff 時計。
- 此快照紀錄狀態：`passed-two-frozen-input-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-item-weapon-clock-development/reproductions/20261018T063642118412Z/manifest.json`；本次 SHA256 `3f1f48440a132d714c72a106deea4e803ad6bb0d2c726ca002b2a93b5f7db610`。
- 輸入／class 清單原建置出處：`de-native-item-skill-material-development/builds/20261018T052807601508Z/manifest.json`；runtime 所在位置：`de-native-item-skill-material-development/builds/20261018T052807601508Z/candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`。

### F36 — build-native-magical-weapon-material.py

- 路徑：`de-native-magical-weapon-material-development/builds/20261018T064453468915Z/inputs/build-native-magical-weapon-material.py`。
- 大小：**8581 bytes**；全文 **53 行**；讀取範圍 **1–53 全部**；全文版本：F36（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2c2d9dae87f9f7cfa0f948c7807486b62d2ae244dad54bae6b6430933fdc9741`。
- **輸入 JAR**：`candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`；SHA256（紀錄）`df9464f06c41821f3abf7c29ccc5876abe1ead66e570a40308f1274e90521a37`。腳本來源：`build-native-magical-weapon-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`（`build-native-magical-weapon-material.py:43`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`，SHA256（紀錄）`ff647f9eee261876e138ebb8b4d0817f5b06e7bcad37663ab7da6de59525b633`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeCastMaterial$Cast`、`NativeCastMaterial`、`NativeLightAction`；新增 `NativeMagicalWeaponCastAction$Change`、`NativeMagicalWeaponCastAction`。腳本修改入口／集合：`build-native-magical-weapon-material.py:42`。
- **這一層在做什麼**：讓技能 73 的 staged 物品效果和材料扣除同屬一筆交易，包含無材料／GM bypass 的效果執行。
- 此快照紀錄狀態：`passed-magical-weapon-material-prototype-build-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-magical-weapon-material-development/builds/20261018T064453468915Z/manifest.json`；本次 SHA256 `193ca8171979ee1f20d68614d90765c54a7827ecbb8b9be6d8072240d5fcce4b`。
- 輸入／class 清單原建置出處：`de-native-magical-weapon-material-development/builds/20261018T064453468915Z/manifest.json`；runtime 所在位置：`de-native-magical-weapon-material-development/builds/20261018T064453468915Z/candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`。

### F37 — build-native-magical-weapon-material.py

- 路徑：`de-native-magical-weapon-material-development/builds/20261019T165226501415Z/inputs/build-native-magical-weapon-material.py`。
- 大小：**8858 bytes**；全文 **53 行**；讀取範圍 **1–53 全部**；全文版本：F37（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`f7c97dcec90c222987e5837c8a0ad276b022df353a2c193cf6002ae4266e4603`。
- **輸入 JAR**：`candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`；SHA256（紀錄）`df9464f06c41821f3abf7c29ccc5876abe1ead66e570a40308f1274e90521a37`。腳本來源：`build-native-magical-weapon-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`（`build-native-magical-weapon-material.py:43`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`，SHA256（紀錄）`fb2e26b5ef4b7304d45904a15d32558c1a314c53d07138573ff3e171d8d92555`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeCastMaterial$Cast`、`NativeCastMaterial`、`NativeLightAction$Batch`、`NativeLightAction`；新增 `NativeMagicalWeaponCastAction$Change`、`NativeMagicalWeaponCastAction`。腳本修改入口／集合：`build-native-magical-weapon-material.py:42`。
- **這一層在做什麼**：在技能 73 材料與物品交易上加入延後輸出的 accepted.binding 檢查，防止舊角色發出結果。
- 此快照紀錄狀態：`passed-magical-weapon-material-prototype-build-handler-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-magical-weapon-material-development/builds/20261019T165226501415Z/manifest.json`；本次 SHA256 `489aebc17f0ac6d16394055dbf99293a280da628e88dd6bbcb58ffe821103374`。
- 輸入／class 清單原建置出處：`de-native-magical-weapon-material-development/builds/20261019T165226501415Z/manifest.json`；runtime 所在位置：`de-native-magical-weapon-material-development/builds/20261019T165226501415Z/candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`。

### F38 — build-native-magical-weapon-material.py

- 路徑：`de-native-magical-weapon-material-development/progress-checkpoints/20261019T180416756951Z/inputs/build-native-magical-weapon-material.py`。
- 大小：**8858 bytes**；全文 **53 行**；讀取範圍 **1–53 全部**；全文版本：F37（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`f7c97dcec90c222987e5837c8a0ad276b022df353a2c193cf6002ae4266e4603`。
- **輸入 JAR**：`candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`；SHA256（紀錄）`df9464f06c41821f3abf7c29ccc5876abe1ead66e570a40308f1274e90521a37`。腳本來源：`build-native-magical-weapon-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`（`build-native-magical-weapon-material.py:43`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`，SHA256（紀錄）`fb2e26b5ef4b7304d45904a15d32558c1a314c53d07138573ff3e171d8d92555`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeCastMaterial$Cast`、`NativeCastMaterial`、`NativeLightAction$Batch`、`NativeLightAction`；新增 `NativeMagicalWeaponCastAction$Change`、`NativeMagicalWeaponCastAction`。腳本修改入口／集合：`build-native-magical-weapon-material.py:42`。
- **這一層在做什麼**：在技能 73 材料與物品交易上加入延後輸出的 accepted.binding 檢查，防止舊角色發出結果。
- 此快照紀錄狀態：`in-progress-prototype-twenty-four-material-skills-including-skill73`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-magical-weapon-material-development/progress-checkpoints/20261019T180416756951Z/manifest.json`；本次 SHA256 `0454c986907d5b5a2130f159c38a268876cbfb65dd92d60a0dbe9c06025016e9`。
- 輸入／class 清單原建置出處：`de-native-magical-weapon-material-development/builds/20261019T165226501415Z/manifest.json`；runtime 所在位置：`de-native-magical-weapon-material-development/builds/20261019T165226501415Z/candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`。

### F39 — build-native-magical-weapon-material.py

- 路徑：`de-native-magical-weapon-material-development/reproductions/20261019T165356943308Z/inputs/seed-inputs/build-native-magical-weapon-material.py`。
- 大小：**8858 bytes**；全文 **53 行**；讀取範圍 **1–53 全部**；全文版本：F37（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`f7c97dcec90c222987e5837c8a0ad276b022df353a2c193cf6002ae4266e4603`。
- **輸入 JAR**：`candidate-NATIVE-ITEM-WEAPON-MATERIAL-unqualified.jar`；SHA256（紀錄）`df9464f06c41821f3abf7c29ccc5876abe1ead66e570a40308f1274e90521a37`。腳本來源：`build-native-magical-weapon-material.py:5`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`（`build-native-magical-weapon-material.py:43`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`，SHA256（紀錄）`fb2e26b5ef4b7304d45904a15d32558c1a314c53d07138573ff3e171d8d92555`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeCastMaterial$Cast`、`NativeCastMaterial`、`NativeLightAction$Batch`、`NativeLightAction`；新增 `NativeMagicalWeaponCastAction$Change`、`NativeMagicalWeaponCastAction`。腳本修改入口／集合：`build-native-magical-weapon-material.py:42`。
- **這一層在做什麼**：在技能 73 材料與物品交易上加入延後輸出的 accepted.binding 檢查，防止舊角色發出結果。
- 此快照紀錄狀態：`passed-two-frozen-input-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-magical-weapon-material-development/reproductions/20261019T165356943308Z/manifest.json`；本次 SHA256 `2621a1204d5cab3294f0bf778e7e812a38318872240fe94976b4c389ccf77cb5`。
- 輸入／class 清單原建置出處：`de-native-magical-weapon-material-development/builds/20261019T165226501415Z/manifest.json`；runtime 所在位置：`de-native-magical-weapon-material-development/builds/20261019T165226501415Z/candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`。

### F40 — build-native-pair-material.py

- 路徑：`de-native-pair-material-development/builds/20261020T074247786923Z/inputs/build-native-pair-material.py`。
- 大小：**9942 bytes**；全文 **73 行**；讀取範圍 **1–73 全部**；全文版本：F40（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2bb3dcf42c5bdf031fdf7dd585986d6f045c1aa726e87605cf732258f4ef5b2d`。
- **輸入 JAR**：`candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`；SHA256（紀錄）`0269195c2bd3626a5a2ec9366dd655362045c4fc16ba138a40a209572716f0c2`。腳本來源：`build-native-pair-material.py:7`。
- **輸出 JAR**：預定 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`（`build-native-pair-material.py:67`）；此快照 manifest 為 `failed`，未記錄 runtime／輸出 SHA，不宣告成功產物。
- **修改的 class／方法**（靜態預期，無成功產物聲明）：ClientSession、NativeLightAction、NativeLightAction$Batch、NativeCastMaterial；新增 NativePairCastAction、NativePairCastAction$Pair、NativeLightAction$PairOutput（第 65–66 行預期）；入口 `build-native-pair-material.py:65`。
- **這一層在做什麼**：將技能 68 的施法者／目標綁定、材料安裝與延後輸出納入雙角色生命週期守門。
- 此快照紀錄狀態：`failed`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-pair-material-development/builds/20261020T074247786923Z/manifest.json`；本次 SHA256 `034c3e833804df472314b363532aad1668d9141b5d852e8435e3989be6e77552`。
- 輸入／class 清單原建置出處：`de-native-pair-material-development/builds/20261020T074247786923Z/manifest.json`；runtime 所在位置：未記錄。

### F41 — build-native-pair-material.py

- 路徑：`de-native-pair-material-development/builds/20261020T074259348571Z/inputs/build-native-pair-material.py`。
- 大小：**9942 bytes**；全文 **73 行**；讀取範圍 **1–73 全部**；全文版本：F40（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2bb3dcf42c5bdf031fdf7dd585986d6f045c1aa726e87605cf732258f4ef5b2d`。
- **輸入 JAR**：`candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`；SHA256（紀錄）`0269195c2bd3626a5a2ec9366dd655362045c4fc16ba138a40a209572716f0c2`。腳本來源：`build-native-pair-material.py:7`。
- **輸出 JAR**：預定 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`（`build-native-pair-material.py:67`）；此快照 manifest 為 `failed`，未記錄 runtime／輸出 SHA，不宣告成功產物。
- **修改的 class／方法**（靜態預期，無成功產物聲明）：ClientSession、NativeLightAction、NativeLightAction$Batch、NativeCastMaterial；新增 NativePairCastAction、NativePairCastAction$Pair、NativeLightAction$PairOutput（第 65–66 行預期）；入口 `build-native-pair-material.py:65`。
- **這一層在做什麼**：將技能 68 的施法者／目標綁定、材料安裝與延後輸出納入雙角色生命週期守門。
- 此快照紀錄狀態：`failed`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-pair-material-development/builds/20261020T074259348571Z/manifest.json`；本次 SHA256 `5d3725accf98746183c9dd8e42680416593ef85b17507c218103ccf6cdf5cfe1`。
- 輸入／class 清單原建置出處：`de-native-pair-material-development/builds/20261020T074259348571Z/manifest.json`；runtime 所在位置：未記錄。

### F42 — build-native-pair-material.py

- 路徑：`de-native-pair-material-development/builds/20261020T074343530589Z/inputs/build-native-pair-material.py`。
- 大小：**9942 bytes**；全文 **73 行**；讀取範圍 **1–73 全部**；全文版本：F40（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2bb3dcf42c5bdf031fdf7dd585986d6f045c1aa726e87605cf732258f4ef5b2d`。
- **輸入 JAR**：`candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`；SHA256（紀錄）`0269195c2bd3626a5a2ec9366dd655362045c4fc16ba138a40a209572716f0c2`。腳本來源：`build-native-pair-material.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`（`build-native-pair-material.py:67`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`，SHA256（紀錄）`c7b3d99a84c4183fb4625d6993dd1d39bc8833507a85e500813835400ca25b0a`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeCastMaterial`、`NativeLightAction$Batch`、`NativeLightAction`；新增 `NativeLightAction$PairOutput`、`NativePairCastAction$Pair`、`NativePairCastAction`。腳本修改入口／集合：`build-native-pair-material.py:65`。
- **這一層在做什麼**：將技能 68 的施法者／目標綁定、材料安裝與延後輸出納入雙角色生命週期守門。
- 此快照紀錄狀態：`passed-pair-material-build-native-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-pair-material-development/builds/20261020T074343530589Z/manifest.json`；本次 SHA256 `462388aa4a91d55c5e8486c30e544b611732994b6c45ca06401b4a9f1d565fbc`。
- 輸入／class 清單原建置出處：`de-native-pair-material-development/builds/20261020T074343530589Z/manifest.json`；runtime 所在位置：`de-native-pair-material-development/builds/20261020T074343530589Z/candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`。

### F43 — build-native-pair-material.py

- 路徑：`de-native-pair-material-development/builds/20261020T080606934231Z/inputs/build-native-pair-material.py`。
- 大小：**9942 bytes**；全文 **73 行**；讀取範圍 **1–73 全部**；全文版本：F40（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2bb3dcf42c5bdf031fdf7dd585986d6f045c1aa726e87605cf732258f4ef5b2d`。
- **輸入 JAR**：`candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`；SHA256（紀錄）`0269195c2bd3626a5a2ec9366dd655362045c4fc16ba138a40a209572716f0c2`。腳本來源：`build-native-pair-material.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`（`build-native-pair-material.py:67`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`，SHA256（紀錄）`175022207c3c5b7954eb0f86edc01ec879f849314e978d2eee35d176961ded71`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeCastMaterial`、`NativeLightAction$Batch`、`NativeLightAction`；新增 `NativeLightAction$PairOutput`、`NativePairCastAction$Pair`、`NativePairCastAction`。腳本修改入口／集合：`build-native-pair-material.py:65`。
- **這一層在做什麼**：將技能 68 的施法者／目標綁定、材料安裝與延後輸出納入雙角色生命週期守門。
- 此快照紀錄狀態：`passed-pair-material-build-native-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-pair-material-development/builds/20261020T080606934231Z/manifest.json`；本次 SHA256 `381bcf1e4de61f5c3f49cb83f1a9916252a9523da21b6e17e540aef041f41288`。
- 輸入／class 清單原建置出處：`de-native-pair-material-development/builds/20261020T080606934231Z/manifest.json`；runtime 所在位置：`de-native-pair-material-development/builds/20261020T080606934231Z/candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`。

### F44 — build-native-pair-material.py

- 路徑：`de-native-pair-material-development/progress-checkpoints/20261020T082030850712Z/inputs/build-native-pair-material.py`。
- 大小：**9942 bytes**；全文 **73 行**；讀取範圍 **1–73 全部**；全文版本：F40（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2bb3dcf42c5bdf031fdf7dd585986d6f045c1aa726e87605cf732258f4ef5b2d`。
- **輸入 JAR**：`candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`；SHA256（紀錄）`0269195c2bd3626a5a2ec9366dd655362045c4fc16ba138a40a209572716f0c2`。腳本來源：`build-native-pair-material.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`（`build-native-pair-material.py:67`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`，SHA256（紀錄）`175022207c3c5b7954eb0f86edc01ec879f849314e978d2eee35d176961ded71`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeCastMaterial`、`NativeLightAction$Batch`、`NativeLightAction`；新增 `NativeLightAction$PairOutput`、`NativePairCastAction$Pair`、`NativePairCastAction`。腳本修改入口／集合：`build-native-pair-material.py:65`。
- **這一層在做什麼**：將技能 68 的施法者／目標綁定、材料安裝與延後輸出納入雙角色生命週期守門。
- 此快照紀錄狀態：`in-progress-skill68-pair-material-qualified25-of58-full-Source-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-pair-material-development/progress-checkpoints/20261020T082030850712Z/manifest.json`；本次 SHA256 `b8fad7673790b4e258aa94ecc6f11bb5323d533ab5a59306e9a22728be22f43c`。
- 輸入／class 清單原建置出處：`de-native-pair-material-development/builds/20261020T080606934231Z/manifest.json`；runtime 所在位置：`de-native-pair-material-development/builds/20261020T080606934231Z/candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`。

### F45 — build-native-pair-material.py

- 路徑：`de-native-pair-material-development/reproductions/20261020T075747480420Z/inputs/seed/build-native-pair-material.py`。
- 大小：**9942 bytes**；全文 **73 行**；讀取範圍 **1–73 全部**；全文版本：F40（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2bb3dcf42c5bdf031fdf7dd585986d6f045c1aa726e87605cf732258f4ef5b2d`。
- **輸入 JAR**：`candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`；SHA256（紀錄）`0269195c2bd3626a5a2ec9366dd655362045c4fc16ba138a40a209572716f0c2`。腳本來源：`build-native-pair-material.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`（`build-native-pair-material.py:67`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`，SHA256（紀錄）`c7b3d99a84c4183fb4625d6993dd1d39bc8833507a85e500813835400ca25b0a`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeCastMaterial`、`NativeLightAction$Batch`、`NativeLightAction`；新增 `NativeLightAction$PairOutput`、`NativePairCastAction$Pair`、`NativePairCastAction`。腳本修改入口／集合：`build-native-pair-material.py:65`。
- **這一層在做什麼**：將技能 68 的施法者／目標綁定、材料安裝與延後輸出納入雙角色生命週期守門。
- 此快照紀錄狀態：`passed-two-frozen-pair-material-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-pair-material-development/reproductions/20261020T075747480420Z/manifest.json`；本次 SHA256 `cc9379daf72500ea54e194ca22f828ed28d5ffbf10f33cb97c08b9046fb0bc26`。
- 輸入／class 清單原建置出處：`de-native-pair-material-development/builds/20261020T074343530589Z/manifest.json`；runtime 所在位置：`de-native-pair-material-development/builds/20261020T074343530589Z/candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`。

### F46 — build-native-pair-material.py

- 路徑：`de-native-pair-material-development/reproductions/20261020T080635414759Z/inputs/seed/build-native-pair-material.py`。
- 大小：**9942 bytes**；全文 **73 行**；讀取範圍 **1–73 全部**；全文版本：F40（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2bb3dcf42c5bdf031fdf7dd585986d6f045c1aa726e87605cf732258f4ef5b2d`。
- **輸入 JAR**：`candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`；SHA256（紀錄）`0269195c2bd3626a5a2ec9366dd655362045c4fc16ba138a40a209572716f0c2`。腳本來源：`build-native-pair-material.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`（`build-native-pair-material.py:67`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`，SHA256（紀錄）`175022207c3c5b7954eb0f86edc01ec879f849314e978d2eee35d176961ded71`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`NativeCastMaterial`、`NativeLightAction$Batch`、`NativeLightAction`；新增 `NativeLightAction$PairOutput`、`NativePairCastAction$Pair`、`NativePairCastAction`。腳本修改入口／集合：`build-native-pair-material.py:65`。
- **這一層在做什麼**：將技能 68 的施法者／目標綁定、材料安裝與延後輸出納入雙角色生命週期守門。
- 此快照紀錄狀態：`passed-two-frozen-pair-material-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-pair-material-development/reproductions/20261020T080635414759Z/manifest.json`；本次 SHA256 `ce84f9c6320d57f69d591b409c979d8641d338ef155d43d7edfdac09161243f3`。
- 輸入／class 清單原建置出處：`de-native-pair-material-development/builds/20261020T080606934231Z/manifest.json`；runtime 所在位置：`de-native-pair-material-development/builds/20261020T080606934231Z/candidate-NATIVE-PAIR-MATERIAL-unqualified.jar`。

### F47 — build-native-retired-reuse-timer.py

- 路徑：`de-native-retired-reuse-timer-development/builds/20261019T184359510191Z/inputs/build-native-retired-reuse-timer.py`。
- 大小：**3819 bytes**；全文 **25 行**；讀取範圍 **1–25 全部**；全文版本：F47（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2522ece1d0c68210a5eb743513010cfe302c2c271d1cc7ac14390a3dda0a3e61`。
- **輸入 JAR**：`candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`；SHA256（紀錄）`7fbc49e745581aa3be40caae54d561010a43061a6d9f1362593b56d9fd9c5ed7`。腳本來源：`build-native-retired-reuse-timer.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`（`build-native-retired-reuse-timer.py:19`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`，SHA256（紀錄）`0269195c2bd3626a5a2ec9366dd655362045c4fc16ba138a40a209572716f0c2`。
- **修改的 class／方法**（原建置 manifest）：修改 `NativeReuseActor`；新增 `NativeRetiredReuseTimer$Retired`、`NativeRetiredReuseTimer`。腳本修改入口／集合：`build-native-retired-reuse-timer.py:18`。
- **這一層在做什麼**：替換 NativeReuseActor.pcSchedule 埠，STOPPED 且 scheduler 已關閉時回傳已取消 Future，避免再保留角色／任務。
- 此快照紀錄狀態：`passed-retired-reuse-timer-port-build-native-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-retired-reuse-timer-development/builds/20261019T184359510191Z/manifest.json`；本次 SHA256 `53de46790fc908e1ffeeb60cd1004b3efb8e37e0b9ec18c85803cf8b1fbb1716`。
- 輸入／class 清單原建置出處：`de-native-retired-reuse-timer-development/builds/20261019T184359510191Z/manifest.json`；runtime 所在位置：`de-native-retired-reuse-timer-development/builds/20261019T184359510191Z/candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`。

### F48 — build-native-retired-reuse-timer.py

- 路徑：`de-native-retired-reuse-timer-development/progress-checkpoints/20261019T185801261031Z/inputs/build-native-retired-reuse-timer.py`。
- 大小：**3819 bytes**；全文 **25 行**；讀取範圍 **1–25 全部**；全文版本：F47（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2522ece1d0c68210a5eb743513010cfe302c2c271d1cc7ac14390a3dda0a3e61`。
- **輸入 JAR**：`candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`；SHA256（紀錄）`7fbc49e745581aa3be40caae54d561010a43061a6d9f1362593b56d9fd9c5ed7`。腳本來源：`build-native-retired-reuse-timer.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`（`build-native-retired-reuse-timer.py:19`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`，SHA256（紀錄）`0269195c2bd3626a5a2ec9366dd655362045c4fc16ba138a40a209572716f0c2`。
- **修改的 class／方法**（原建置 manifest）：修改 `NativeReuseActor`；新增 `NativeRetiredReuseTimer$Retired`、`NativeRetiredReuseTimer`。腳本修改入口／集合：`build-native-retired-reuse-timer.py:18`。
- **這一層在做什麼**：替換 NativeReuseActor.pcSchedule 埠，STOPPED 且 scheduler 已關閉時回傳已取消 Future，避免再保留角色／任務。
- 此快照紀錄狀態：`in-progress-four-Source-controls-and-skill73-Main-with-retired-timer-verified-full-goal-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-retired-reuse-timer-development/progress-checkpoints/20261019T185801261031Z/manifest.json`；本次 SHA256 `179ffd63aecdfcd84346e4065dc73cb3a811f7b7fcc22ec30a5adeb200cbab91`。
- 輸入／class 清單原建置出處：`de-native-retired-reuse-timer-development/builds/20261019T184359510191Z/manifest.json`；runtime 所在位置：`de-native-retired-reuse-timer-development/builds/20261019T184359510191Z/candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`。

### F49 — build-native-retired-reuse-timer.py

- 路徑：`de-native-retired-reuse-timer-development/reproductions/20261019T184620341363Z/inputs/seed/build-native-retired-reuse-timer.py`。
- 大小：**3819 bytes**；全文 **25 行**；讀取範圍 **1–25 全部**；全文版本：F47（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`2522ece1d0c68210a5eb743513010cfe302c2c271d1cc7ac14390a3dda0a3e61`。
- **輸入 JAR**：`candidate-NATIVE-INVISIBILITY-CAST-POLICY-unqualified.jar`；SHA256（紀錄）`7fbc49e745581aa3be40caae54d561010a43061a6d9f1362593b56d9fd9c5ed7`。腳本來源：`build-native-retired-reuse-timer.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`（`build-native-retired-reuse-timer.py:19`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`，SHA256（紀錄）`0269195c2bd3626a5a2ec9366dd655362045c4fc16ba138a40a209572716f0c2`。
- **修改的 class／方法**（原建置 manifest）：修改 `NativeReuseActor`；新增 `NativeRetiredReuseTimer$Retired`、`NativeRetiredReuseTimer`。腳本修改入口／集合：`build-native-retired-reuse-timer.py:18`。
- **這一層在做什麼**：替換 NativeReuseActor.pcSchedule 埠，STOPPED 且 scheduler 已關閉時回傳已取消 Future，避免再保留角色／任務。
- 此快照紀錄狀態：`passed-two-frozen-retired-reuse-timer-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-retired-reuse-timer-development/reproductions/20261019T184620341363Z/manifest.json`；本次 SHA256 `8d3141c6b80038429198b600d8359d770066d179c49231c7472a92a98c0bd013`。
- 輸入／class 清單原建置出處：`de-native-retired-reuse-timer-development/builds/20261019T184359510191Z/manifest.json`；runtime 所在位置：`de-native-retired-reuse-timer-development/builds/20261019T184359510191Z/candidate-NATIVE-RETIRED-REUSE-TIMER-unqualified.jar`。

### F50 — build-native-reuse-lease.py

- 路徑：`de-native-reuse-lease-development/builds/20261018T010559902997Z/inputs/build-native-reuse-lease.py`。
- 大小：**7639 bytes**；全文 **53 行**；讀取範圍 **1–53 全部**；全文版本：F50（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`f37c4269206f086ad403d42116671cf7f7a2c50981ea6f6f966fca19d5b41e6d`。
- **輸入 JAR**：`candidate-NATIVE-SELF-CAST-experimental.jar`；SHA256（紀錄）`71ea214a486cb7d25f5b2f01aa7a533a0dd7f4ea4d4b3cf993c251588ff8b664`。腳本來源：`build-native-reuse-lease.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-REUSE-LEASE-unqualified.jar`（`build-native-reuse-lease.py:41`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-REUSE-LEASE-unqualified.jar`，SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。
- **修改的 class／方法**（原建置 manifest）：修改 `NativeCastTimeline$1`、`NativeCastTimeline$2`、`NativeCastTimeline$State`、`NativeCastTimeline`、`NativeLightAction`、`NativeSelfCastAction`、`NativePlayerLifetime`、`NativePlayerVitalCompletions$Receipt`；新增 `NativeSourceSkillDelay$Lease`、`NativeSourceSkillDelay`、`NativeReuseActor`、`NativeReuseGate`、`NativeReuseLog$Log`、`NativeReuseLog$Logger`、`NativeReuseLog`、`NativeReuseShutdown`。腳本修改入口／集合：`build-native-reuse-lease.py:50`。
- **這一層在做什麼**：投影 Source L1SkillDelay 的 reuse lease／token 至原生時間線，為技能 2／3／14／99／151 接入 shutdown 與角色生命週期。
- 此快照紀錄狀態：`passed-complete-Source-reuse-lease-projection-build-consumer-integration-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-reuse-lease-development/builds/20261018T010559902997Z/manifest.json`；本次 SHA256 `5d8e7541efdc38ac245bd9e16a1345faafbc0c864cfd53e445f6d3674d0040c4`。
- 輸入／class 清單原建置出處：`de-native-reuse-lease-development/builds/20261018T010559902997Z/manifest.json`；runtime 所在位置：`de-native-reuse-lease-development/builds/20261018T010559902997Z/candidate-NATIVE-REUSE-LEASE-unqualified.jar`。

### F51 — build-native-reuse-lease.py

- 路徑：`de-native-reuse-lease-development/builds/20261018T010605104466Z/inputs/build-native-reuse-lease.py`。
- 大小：**7639 bytes**；全文 **53 行**；讀取範圍 **1–53 全部**；全文版本：F50（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`f37c4269206f086ad403d42116671cf7f7a2c50981ea6f6f966fca19d5b41e6d`。
- **輸入 JAR**：`candidate-NATIVE-SELF-CAST-experimental.jar`；SHA256（紀錄）`71ea214a486cb7d25f5b2f01aa7a533a0dd7f4ea4d4b3cf993c251588ff8b664`。腳本來源：`build-native-reuse-lease.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-REUSE-LEASE-unqualified.jar`（`build-native-reuse-lease.py:41`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-REUSE-LEASE-unqualified.jar`，SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。
- **修改的 class／方法**（原建置 manifest）：修改 `NativeCastTimeline$1`、`NativeCastTimeline$2`、`NativeCastTimeline$State`、`NativeCastTimeline`、`NativeLightAction`、`NativeSelfCastAction`、`NativePlayerLifetime`、`NativePlayerVitalCompletions$Receipt`；新增 `NativeSourceSkillDelay$Lease`、`NativeSourceSkillDelay`、`NativeReuseActor`、`NativeReuseGate`、`NativeReuseLog$Log`、`NativeReuseLog$Logger`、`NativeReuseLog`、`NativeReuseShutdown`。腳本修改入口／集合：`build-native-reuse-lease.py:50`。
- **這一層在做什麼**：投影 Source L1SkillDelay 的 reuse lease／token 至原生時間線，為技能 2／3／14／99／151 接入 shutdown 與角色生命週期。
- 此快照紀錄狀態：`passed-complete-Source-reuse-lease-projection-build-consumer-integration-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-reuse-lease-development/builds/20261018T010605104466Z/manifest.json`；本次 SHA256 `4938044a3b5feab2cce55921bb748d4948e88cc99ebfd74c574b095808d29717`。
- 輸入／class 清單原建置出處：`de-native-reuse-lease-development/builds/20261018T010605104466Z/manifest.json`；runtime 所在位置：`de-native-reuse-lease-development/builds/20261018T010605104466Z/candidate-NATIVE-REUSE-LEASE-unqualified.jar`。

### F52 — build-native-reuse-lease.py

- 路徑：`de-native-reuse-lease-development/reproducibility-runs/20261018T010333210661Z/seed-inputs/build-native-reuse-lease.py`。
- 大小：**7639 bytes**；全文 **53 行**；讀取範圍 **1–53 全部**；全文版本：F50（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`f37c4269206f086ad403d42116671cf7f7a2c50981ea6f6f966fca19d5b41e6d`。
- **輸入 JAR**：`candidate-NATIVE-SELF-CAST-experimental.jar`；SHA256（紀錄）`71ea214a486cb7d25f5b2f01aa7a533a0dd7f4ea4d4b3cf993c251588ff8b664`。腳本來源：`build-native-reuse-lease.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-REUSE-LEASE-unqualified.jar`（`build-native-reuse-lease.py:41`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-REUSE-LEASE-experimental.jar`，SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。
- **修改的 class／方法**（原建置 manifest）：修改 `NativeCastTimeline$1`、`NativeCastTimeline$2`、`NativeCastTimeline$State`、`NativeCastTimeline`、`NativeLightAction`、`NativeSelfCastAction`、`NativePlayerLifetime`、`NativePlayerVitalCompletions$Receipt`；新增 `NativeSourceSkillDelay$Lease`、`NativeSourceSkillDelay`、`NativeReuseActor`、`NativeReuseGate`、`NativeReuseLog$Log`、`NativeReuseLog$Logger`、`NativeReuseLog`、`NativeReuseShutdown`。腳本修改入口／集合：`build-native-reuse-lease.py:50`。
- **這一層在做什麼**：投影 Source L1SkillDelay 的 reuse lease／token 至原生時間線，為技能 2／3／14／99／151 接入 shutdown 與角色生命週期。
- 此快照紀錄狀態：`passed-reproducible-reuse-lease-builds-from-identical-frozen-Java-and-compile-only-types`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-reuse-lease-development/reproducibility-runs/20261018T010333210661Z/manifest.json`；本次 SHA256 `ff075475000b2d43c948f4588494126b24db86ae3b875a9148c7ba4c3d9b6131`。
- 輸入／class 清單原建置出處：`de-native-reuse-lease-development/reproducibility-runs/20261018T010333210661Z/manifest.json`；runtime 所在位置：`de-native-reuse-lease-development/objects/35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b/candidate-NATIVE-REUSE-LEASE-experimental.jar`。

### F53 — build-native-reuse-lease.py

- 路徑：`de-native-reuse-lease-development/reproducibility-runs/20261018T010559605886Z/seed-inputs/build-native-reuse-lease.py`。
- 大小：**7639 bytes**；全文 **53 行**；讀取範圍 **1–53 全部**；全文版本：F50（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`f37c4269206f086ad403d42116671cf7f7a2c50981ea6f6f966fca19d5b41e6d`。
- **輸入 JAR**：`candidate-NATIVE-SELF-CAST-experimental.jar`；SHA256（紀錄）`71ea214a486cb7d25f5b2f01aa7a533a0dd7f4ea4d4b3cf993c251588ff8b664`。腳本來源：`build-native-reuse-lease.py:6`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-REUSE-LEASE-unqualified.jar`（`build-native-reuse-lease.py:41`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-REUSE-LEASE-experimental.jar`，SHA256（紀錄）`35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b`。
- **修改的 class／方法**（原建置 manifest）：修改 `NativeCastTimeline$1`、`NativeCastTimeline$2`、`NativeCastTimeline$State`、`NativeCastTimeline`、`NativeLightAction`、`NativeSelfCastAction`、`NativePlayerLifetime`、`NativePlayerVitalCompletions$Receipt`；新增 `NativeSourceSkillDelay$Lease`、`NativeSourceSkillDelay`、`NativeReuseActor`、`NativeReuseGate`、`NativeReuseLog$Log`、`NativeReuseLog$Logger`、`NativeReuseLog`、`NativeReuseShutdown`。腳本修改入口／集合：`build-native-reuse-lease.py:50`。
- **這一層在做什麼**：投影 Source L1SkillDelay 的 reuse lease／token 至原生時間線，為技能 2／3／14／99／151 接入 shutdown 與角色生命週期。
- 此快照紀錄狀態：`passed-reproducible-reuse-lease-builds-from-identical-frozen-Java-and-compile-only-types`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-reuse-lease-development/reproducibility-runs/20261018T010559605886Z/manifest.json`；本次 SHA256 `bbbaad92ca7be0f9a8c277b67f0da81b95675a945c090bb3cbff8d0216a103fd`。
- 輸入／class 清單原建置出處：`de-native-reuse-lease-development/reproducibility-runs/20261018T010559605886Z/manifest.json`；runtime 所在位置：`de-native-reuse-lease-development/objects/35853617c9b92cfba01a55e64599454c697064b6f272b6d25285c38100295e9b/candidate-NATIVE-REUSE-LEASE-experimental.jar`。

### F54 — build-native-solid-carriage-policy.py

- 路徑：`de-native-solid-carriage-policy-development/builds/20261019T170906387267Z/inputs/build-native-solid-carriage-policy.py`。
- 大小：**5473 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F54（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`5a31ae6524bba2df922edb8e7ec0025c6badfadfa93cfc363950a0d69f2c79b1`。
- **輸入 JAR**：`candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`；SHA256（紀錄）`fb2e26b5ef4b7304d45904a15d32558c1a314c53d07138573ff3e171d8d92555`。腳本來源：`build-native-solid-carriage-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`（`build-native-solid-carriage-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`，SHA256（紀錄）`7c6a29baf443fb0f69f3411b5151422379bc3a9a3ff3291ac260833e03e4a0a0`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`CombatRules`；新增 `NativeSolidCarriagePolicy`。腳本修改入口／集合：`build-native-solid-carriage-policy.py:27`。
- **這一層在做什麼**：將堅固防護的盾／臂甲允許條件接入 CombatRules.snapshot 的 SOLID_CARRIAGE_MODE 政策。
- 此快照紀錄狀態：`passed-solid-carriage-policy-build-native-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-solid-carriage-policy-development/builds/20261019T170906387267Z/manifest.json`；本次 SHA256 `03c021e83526e31a93a08d6c4f604bc6eacbb5945b3fc01e331ec9401fd318e0`。
- 輸入／class 清單原建置出處：`de-native-solid-carriage-policy-development/builds/20261019T170906387267Z/manifest.json`；runtime 所在位置：`de-native-solid-carriage-policy-development/builds/20261019T170906387267Z/candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`。

### F55 — build-native-solid-carriage-policy.py

- 路徑：`de-native-solid-carriage-policy-development/progress-checkpoints/20261019T180455598406Z/inputs/build-native-solid-carriage-policy.py`。
- 大小：**5473 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F54（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`5a31ae6524bba2df922edb8e7ec0025c6badfadfa93cfc363950a0d69f2c79b1`。
- **輸入 JAR**：`candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`；SHA256（紀錄）`fb2e26b5ef4b7304d45904a15d32558c1a314c53d07138573ff3e171d8d92555`。腳本來源：`build-native-solid-carriage-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`（`build-native-solid-carriage-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`，SHA256（紀錄）`7c6a29baf443fb0f69f3411b5151422379bc3a9a3ff3291ac260833e03e4a0a0`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`CombatRules`；新增 `NativeSolidCarriagePolicy`。腳本修改入口／集合：`build-native-solid-carriage-policy.py:27`。
- **這一層在做什麼**：將堅固防護的盾／臂甲允許條件接入 CombatRules.snapshot 的 SOLID_CARRIAGE_MODE 政策。
- 此快照紀錄狀態：`in-progress-Source-solid-carriage-mode-targeted-verified-full-goal-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-solid-carriage-policy-development/progress-checkpoints/20261019T180455598406Z/manifest.json`；本次 SHA256 `681a844c5a3b142a3bfa45ad10631c8cd27614ceaacc54db660a0dbdd23ac3e5`。
- 輸入／class 清單原建置出處：`de-native-solid-carriage-policy-development/builds/20261019T170906387267Z/manifest.json`；runtime 所在位置：`de-native-solid-carriage-policy-development/builds/20261019T170906387267Z/candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`。

### F56 — build-native-solid-carriage-policy.py

- 路徑：`de-native-solid-carriage-policy-development/reproductions/20261019T172048007823Z/inputs/seed/build-native-solid-carriage-policy.py`。
- 大小：**5473 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F54（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`5a31ae6524bba2df922edb8e7ec0025c6badfadfa93cfc363950a0d69f2c79b1`。
- **輸入 JAR**：`candidate-NATIVE-MAGICAL-WEAPON-MATERIAL-unqualified.jar`；SHA256（紀錄）`fb2e26b5ef4b7304d45904a15d32558c1a314c53d07138573ff3e171d8d92555`。腳本來源：`build-native-solid-carriage-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`（`build-native-solid-carriage-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`，SHA256（紀錄）`7c6a29baf443fb0f69f3411b5151422379bc3a9a3ff3291ac260833e03e4a0a0`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`CombatRules`；新增 `NativeSolidCarriagePolicy`。腳本修改入口／集合：`build-native-solid-carriage-policy.py:27`。
- **這一層在做什麼**：將堅固防護的盾／臂甲允許條件接入 CombatRules.snapshot 的 SOLID_CARRIAGE_MODE 政策。
- 此快照紀錄狀態：`passed-two-frozen-solid-carriage-policy-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-solid-carriage-policy-development/reproductions/20261019T172048007823Z/manifest.json`；本次 SHA256 `4648eb82c728d7cf20e19d188c9c58b3e90f6c4033a1b97163bf3bae3259c350`。
- 輸入／class 清單原建置出處：`de-native-solid-carriage-policy-development/builds/20261019T170906387267Z/manifest.json`；runtime 所在位置：`de-native-solid-carriage-policy-development/builds/20261019T170906387267Z/candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`。

### F57 — build-native-thunder-teleport-policy.py

- 路徑：`de-native-thunder-teleport-policy-development/builds/20261019T181402975452Z/inputs/build-native-thunder-teleport-policy.py`。
- 大小：**6650 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F57（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`012e65b413798c48f5707a881229a5a5974adb3a95446c4220773bcff24f9676`。
- **輸入 JAR**：`candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`；SHA256（紀錄）`7c6a29baf443fb0f69f3411b5151422379bc3a9a3ff3291ac260833e03e4a0a0`。腳本來源：`build-native-thunder-teleport-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`（`build-native-thunder-teleport-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`，SHA256（紀錄）`363ceed9643540e4845e66a8a5098c0dc6fcb8e27e314d0c5e34785ad80992a9`。
- **修改的 class／方法**（原建置 manifest）：修改 `BattleTimedStatus`、`ClientSession`、`CombatRules`；新增 `NativeThunderTeleportPolicy`。腳本修改入口／集合：`build-native-thunder-teleport-policy.py:27`。
- **這一層在做什麼**：分離奪命之雷移動束縛與完全麻痺，對技能 5／131 的傳送拒絕分支補訊息、解鎖及 MP 回覆。
- 此快照紀錄狀態：`passed-thunder-teleport-policy-build-native-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-thunder-teleport-policy-development/builds/20261019T181402975452Z/manifest.json`；本次 SHA256 `10273c5e38beb9a11434577e9268b2bc9c3ee7a0feb6980ffea1c96d14df499e`。
- 輸入／class 清單原建置出處：`de-native-thunder-teleport-policy-development/builds/20261019T181402975452Z/manifest.json`；runtime 所在位置：`de-native-thunder-teleport-policy-development/builds/20261019T181402975452Z/candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`。

### F58 — build-native-thunder-teleport-policy.py

- 路徑：`de-native-thunder-teleport-policy-development/progress-checkpoints/20261019T182515716290Z/inputs/build-native-thunder-teleport-policy.py`。
- 大小：**6650 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F57（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`012e65b413798c48f5707a881229a5a5974adb3a95446c4220773bcff24f9676`。
- **輸入 JAR**：`candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`；SHA256（紀錄）`7c6a29baf443fb0f69f3411b5151422379bc3a9a3ff3291ac260833e03e4a0a0`。腳本來源：`build-native-thunder-teleport-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`（`build-native-thunder-teleport-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`，SHA256（紀錄）`363ceed9643540e4845e66a8a5098c0dc6fcb8e27e314d0c5e34785ad80992a9`。
- **修改的 class／方法**（原建置 manifest）：修改 `BattleTimedStatus`、`ClientSession`、`CombatRules`；新增 `NativeThunderTeleportPolicy`。腳本修改入口／集合：`build-native-thunder-teleport-policy.py:27`。
- **這一層在做什麼**：分離奪命之雷移動束縛與完全麻痺，對技能 5／131 的傳送拒絕分支補訊息、解鎖及 MP 回覆。
- 此快照紀錄狀態：`in-progress-Source-thunder-teleport-targeted-verified-full-goal-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-thunder-teleport-policy-development/progress-checkpoints/20261019T182515716290Z/manifest.json`；本次 SHA256 `c007ead4c867cb166a77c6557123bdf032df6bef350773f980113393e6254fac`。
- 輸入／class 清單原建置出處：`de-native-thunder-teleport-policy-development/builds/20261019T181402975452Z/manifest.json`；runtime 所在位置：`de-native-thunder-teleport-policy-development/builds/20261019T181402975452Z/candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`。

### F59 — build-native-thunder-teleport-policy.py

- 路徑：`de-native-thunder-teleport-policy-development/reproductions/20261019T182414815401Z/inputs/seed/build-native-thunder-teleport-policy.py`。
- 大小：**6650 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F57（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`012e65b413798c48f5707a881229a5a5974adb3a95446c4220773bcff24f9676`。
- **輸入 JAR**：`candidate-NATIVE-SOLID-CARRIAGE-POLICY-unqualified.jar`；SHA256（紀錄）`7c6a29baf443fb0f69f3411b5151422379bc3a9a3ff3291ac260833e03e4a0a0`。腳本來源：`build-native-thunder-teleport-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`（`build-native-thunder-teleport-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`，SHA256（紀錄）`363ceed9643540e4845e66a8a5098c0dc6fcb8e27e314d0c5e34785ad80992a9`。
- **修改的 class／方法**（原建置 manifest）：修改 `BattleTimedStatus`、`ClientSession`、`CombatRules`；新增 `NativeThunderTeleportPolicy`。腳本修改入口／集合：`build-native-thunder-teleport-policy.py:27`。
- **這一層在做什麼**：分離奪命之雷移動束縛與完全麻痺，對技能 5／131 的傳送拒絕分支補訊息、解鎖及 MP 回覆。
- 此快照紀錄狀態：`passed-two-frozen-thunder-teleport-policy-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-thunder-teleport-policy-development/reproductions/20261019T182414815401Z/manifest.json`；本次 SHA256 `95592ee2f41028e4c5fd4c0527194fcc06d3633efa071ad2243c99b32e1ad2f3`。
- 輸入／class 清單原建置出處：`de-native-thunder-teleport-policy-development/builds/20261019T181402975452Z/manifest.json`；runtime 所在位置：`de-native-thunder-teleport-policy-development/builds/20261019T181402975452Z/candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`。

### F60 — build-native-triple-arrow-policy.py

- 路徑：`de-native-triple-arrow-policy-development/builds/20261019T182710221216Z/inputs/build-native-triple-arrow-policy.py`。
- 大小：**6655 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F60（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`965a32ae4aad1994b774aae05bfbd6aa09bcc80687d038f91a005a084aea7b76`。
- **輸入 JAR**：`candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`；SHA256（紀錄）`363ceed9643540e4845e66a8a5098c0dc6fcb8e27e314d0c5e34785ad80992a9`。腳本來源：`build-native-triple-arrow-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`（`build-native-triple-arrow-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`，SHA256（紀錄）`c2a68fccea6a9a2f80cf288867bcf59a7c12c926173123e610e052499fa8c0d9`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`CombatRules`；新增 `NativeTripleArrowPolicy`。腳本修改入口／集合：`build-native-triple-arrow-policy.py:27`。
- **這一層在做什麼**：在三重矢開始前取得一次倍率，沿專用 attack alias 傳遞，保留一般攻擊 producer。
- 此快照紀錄狀態：`passed-triple-arrow-policy-build-native-qualification-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-triple-arrow-policy-development/builds/20261019T182710221216Z/manifest.json`；本次 SHA256 `e7b7d75d37e97996e487f9ef3e8d055cf09343f90289bc5ea702358ba0f846f7`。
- 輸入／class 清單原建置出處：`de-native-triple-arrow-policy-development/builds/20261019T182710221216Z/manifest.json`；runtime 所在位置：`de-native-triple-arrow-policy-development/builds/20261019T182710221216Z/candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`。

### F61 — build-native-triple-arrow-policy.py

- 路徑：`de-native-triple-arrow-policy-development/progress-checkpoints/20261019T183431211310Z/inputs/build-native-triple-arrow-policy.py`。
- 大小：**6655 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F60（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`965a32ae4aad1994b774aae05bfbd6aa09bcc80687d038f91a005a084aea7b76`。
- **輸入 JAR**：`candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`；SHA256（紀錄）`363ceed9643540e4845e66a8a5098c0dc6fcb8e27e314d0c5e34785ad80992a9`。腳本來源：`build-native-triple-arrow-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`（`build-native-triple-arrow-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`，SHA256（紀錄）`c2a68fccea6a9a2f80cf288867bcf59a7c12c926173123e610e052499fa8c0d9`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`CombatRules`；新增 `NativeTripleArrowPolicy`。腳本修改入口／集合：`build-native-triple-arrow-policy.py:27`。
- **這一層在做什麼**：在三重矢開始前取得一次倍率，沿專用 attack alias 傳遞，保留一般攻擊 producer。
- 此快照紀錄狀態：`in-progress-Source-triple-arrow-targeted-verified-full-goal-open`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-triple-arrow-policy-development/progress-checkpoints/20261019T183431211310Z/manifest.json`；本次 SHA256 `71d0f3974b8ced6ef58f12ca5a4a0aa042c2c9e8c0757d9e597f66bf74bc3df2`。
- 輸入／class 清單原建置出處：`de-native-triple-arrow-policy-development/builds/20261019T182710221216Z/manifest.json`；runtime 所在位置：`de-native-triple-arrow-policy-development/builds/20261019T182710221216Z/candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`。

### F62 — build-native-triple-arrow-policy.py

- 路徑：`de-native-triple-arrow-policy-development/reproductions/20261019T183244145265Z/inputs/seed/build-native-triple-arrow-policy.py`。
- 大小：**6655 bytes**；全文 **35 行**；讀取範圍 **1–35 全部**；全文版本：F60（相同編號表示本版本首份；不同編號表示全位元組相等）。
- SHA256：`965a32ae4aad1994b774aae05bfbd6aa09bcc80687d038f91a005a084aea7b76`。
- **輸入 JAR**：`candidate-NATIVE-THUNDER-TELEPORT-POLICY-unqualified.jar`；SHA256（紀錄）`363ceed9643540e4845e66a8a5098c0dc6fcb8e27e314d0c5e34785ad80992a9`。腳本來源：`build-native-triple-arrow-policy.py:7`。
- **輸出 JAR**：腳本命名 `candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`（`build-native-triple-arrow-policy.py:28`）；此快照關聯的 runtime 名稱 `candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`，SHA256（紀錄）`c2a68fccea6a9a2f80cf288867bcf59a7c12c926173123e610e052499fa8c0d9`。
- **修改的 class／方法**（原建置 manifest）：修改 `ClientSession`、`CombatRules`；新增 `NativeTripleArrowPolicy`。腳本修改入口／集合：`build-native-triple-arrow-policy.py:27`。
- **這一層在做什麼**：在三重矢開始前取得一次倍率，沿專用 attack alias 傳遞，保留一般攻擊 producer。
- 此快照紀錄狀態：`passed-two-frozen-triple-arrow-policy-reproductions`（歷史紀錄，不是本次實跑）。
- 快照 manifest：`de-native-triple-arrow-policy-development/reproductions/20261019T183244145265Z/manifest.json`；本次 SHA256 `cad0f18388b64f6dae555bb63ddfcba81f9ad08cb526cea041e3d4d38b4b19eb`。
- 輸入／class 清單原建置出處：`de-native-triple-arrow-policy-development/builds/20261019T182710221216Z/manifest.json`；runtime 所在位置：`de-native-triple-arrow-policy-development/builds/20261019T182710221216Z/candidate-NATIVE-TRIPLE-ARROW-POLICY-unqualified.jar`。
