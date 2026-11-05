# #92 範圍凍結：完成定義、主線封頂、backlog 分流

結論（第 2 版，依 codex-review 退回的 4 點修正）：
- 步驟 0 已完成：HISTORY 全文逐行讀完，技能目錄逐 ID 去重，「已驗證」的說法都附上指令與輸出。
- 主線（M1＋V1）剩 **4 項，封頂**。
- M2、M3 的完成定義已凍結，任務各 30 張、26 張，都已一次列完並放在 Backlog，等開放時原樣升級。

## 第 2 次嘗試的修正
1. **[P1] 盤點補完**：
   - HISTORY（git blob `7dd82ba`，1589 行）先去掉 850 行逐字重複，剩 738 行，分 4 段逐行讀完，沒有抽讀。逐項對照見附錄 A。
   - 475 案：HISTORY L1–L65 的逐輪計數已全部列出，見附錄 B。前 141 案在 HISTORY 沒有分項，逐案名稱只在使用者電腦上未追蹤的候選目錄。這批已延後，不影響主線，已在 TASKS「排除或延後」寫明。
   - 依全文補上的工作：
     - T028：main 38 技能的當版補驗，3 張。
     - T029：非技能的戰鬥基礎，1 張。
     - T023 擴為 3 張：觀戰、更多取消與跨世代 wire。
     - T024 擴為 2 張：idle 守衛、action0 倍率、rollover。
     - T035 擴為 2 張：SendInput 探針。
     - T038：G11 效能驗收，1 張。
     - T031：5 處 hardcode。
     - T036：C72D 1 敗、66470 紅測試。
   - 新增的不確定項：U-M3-5 畫面等價研究、U-M3-6 map563。
   - 新增的排除項：畫面等價研究、WORLD 專屬、建置雜項。
   - M2 由 24 張改為 30 張，M3 由 24 張改為 26 張。主線不變。
2. **[P1] 技能數量**：逐 ID 去重，合計 38＋48＋4＋91＝181（證據 E3）。
   - main 實際是 38 個，不是 39。
   - T027 改為 48 個：原列的 47 個，再加上 5 TELEPORT。這張是 dispatcher 特例，main 沒有它的請求。
   - T022 改為明列 91 個：原寫「129–138」會含進已實作的 131，以及目錄沒有的 135、136。
   - 180／181 查清：181＝180 個 catalog 記錄＋TELEPORT 特例；E149 與 `2066774` 的 catalog 常數 223／223 相同（證據 E2）。
   - T027、T022 都補上逐批 ID。
3. **[P2] 證據**：下面「證據」一節附完整 hash、指令與原始輸出。依實際核對結果修正兩處：
   - 同端點傳送：E149 的 `rentalItemSelectionContext()` 有呼叫 `GameHub.hasRentalBattleHandoff`。所以「E149 四項都沒修」改為「租裝返回、最後離開已驗證沒修；同端點有另一種守衛，是否等價待驗證；商人屬性來源沒核對」。
   - 「候選未部署」改為依伺服器端文件的轉述，不列為已驗證。
4. **[P2] PR 與相依隔離**：開 PR 時 base 設為 `task/86`，送審差異只剩 #92 自己的檔案（PLAN.md、TASKS.md、本報告）。#86 合併後，GitHub 會自動把 base 改成 main。PR 描述列出「移到 backlog 的項目」與「主線剩餘數」。

## 移到 backlog 的項目
- **預定 M2（30 張）**：
  - 原有：T020 #97（2 張）、T021 #98–#100（1 張）、T022 未接 91 技能（10 張）、T023 G06 客戶端（3 張）、T024 4183 與 hurt（2 張）。
  - 新增：T025 版本綁定（1 張）、T026 技能狀態表（1 張）、T027 候選獨有 48 技能（6 張）、T028 main 38 技能補驗（3 張）、T029 戰鬥基礎（1 張）。
- **預定 M3（26 張）**：
  - 原有：T030 實機 GUI（1 張）、T031 可攜交付（2 張）、T032 ARCHITECTURE（1 張）、T040 修補提案移交（1 張）。
  - 新增：T033 G01–G12 定義（1 張）、T034 GUI 清單（1 張）、T035 GUI／輸入缺陷（2 張）、T036 廣域回歸（15 張）、T037 稽核（1 張）、T038 G11（1 張）。
- **排除或延後**：
  - WORLD 地圖專屬（含 HISTORY 列出的 C05、map4、869／$748、WORLD 分支、攻城 P 鍵）；
  - 凍結候選資格檢查，含 475 案；
  - G06 正向前提（E149 沒有生產者）；
  - 商人屬性來源（伺服器缺陷）；
  - 超出 G11 預算的效能最佳化；
  - `TW13081901.pak` 缺檔；
  - 畫面等價研究（照明、水下、map303 背景、scheduler、相機、RGB565）；
  - 建置雜項（編譯警告、外部 LiTo.bin、封存 zip）。

## 主線剩餘數
| 區段 | 完成 | 剩餘 |
|---|---|---|
| M0 | 5（T001–T005） | 0 |
| M1 | 1（T010） | 2（T011-H、T011-B） |
| M1-V1 | 0 | 2（T012、T013） |
| 合計 | 6 | **4**（分母 10） |

- 分母只有在 U-M1-1、U-V1-1、U-V1-2 兌現時可以增加，最多到 13。兌現時 PR 要引用編號與估算。
- M2、M3 開放時（需要 `needs:user`）分別加 30、26，不確定項上限各為 +8、+17。

## 證據
**E1 正式 JAR 的 SHA256**（已驗證）：
```
$ cd lito-launcher-server && sha256sum lito伺服器/LiTo381-Server/reference/base-0.6.71.jar
e1498871c7be6b287821c57ef5d95933db1740d9b4d4a80f6c3465f21d99d498  lito伺服器/LiTo381-Server/reference/base-0.6.71.jar
```

**E2 catalog 比對**（已驗證）：
- 指令：用 `unzip` 解出 E149 的 `BattleSpellCatalog*.class`，再用 `javap -p -constants` 取出 `static final int` 常數，和 `git show 2066774:…/BattleSpellCatalog.java` 的常數排序後做 `diff`。
- 結果：兩邊各 223 行，`diff` 沒有輸出。
- `new \w+Spell(` 的出現次數：原始碼 186，E149 `javap -c` 裡 `new … BattleSpellCatalog$…Spell` 也是 186。
- 產生器的來源：`維護工具/generate-spell-request-shapes.py:85` 從 `new (\w+Spell)\(` 收列；`:192` 另外 `rows.append((5, "TELEPORT", …「dispatcher 特例」))`。

**E3 技能集合**（已驗證，可以重現）：
- 讀 `docs/PROTOCOL-CONTRACT.md` 表格，共 181 列。
- main 集合由 `LiTo381-Modern/src/modern_lito/native_skills.rs` 取得：`request()` 的 match 臂（:103–141）36 個，加上 `ground_request` 的 58、63（:47），共 38 個。`grep -rnE "vec!\[0x14|\[0x14," src` 和 `grep -rn C_USE_SPELL src` 都沒有找到其他技能請求來源。
- 計算結果：

```
catalog 181
main 38 []                      # main 的 ID 全部在目錄內
T027 listed 47 overlap main []
catalog<=98 86
candidate-only (<=98 - main - 97/98) 48 [5] []   # 比原列多出 5
rest >98 not main 93            # 99、100 和 91 個未接
```

**E4 修補提案在 E149 上的狀態**（`javap -p -c local/lito381/server/ClientSession.class`，E149 解出）：
- `quitFight()`：已驗證沒有租裝守衛。

  ```
  0: aload_0 / 1: getfield phase / 4: getstatic Phase.BATTLE / 7: if_acmpne 15
  10: aload_0 / 11: invokevirtual settleRequestedBattleExit:()V / 14: return
  ```

- `broadcastDefaultRoomReset(long)`：已驗證沒有 `roomExists` 檢查，找不到房間時丟例外。

  ```
  8: invokevirtual GameHub.roomSnapshots:(I) … 27: Stream.findFirst … 38: Optional.orElseThrow
  ```

- `exitRoom(boolean)`：已驗證整段委派，所以修補提案的 ASM 前綴假設不成立。

  ```
  0: aload_0 / 1: iload_1 / 2: invokestatic GmPolyMe.exitRoom:(Llocal/lito381/server/ClientSession;Z)V / 5: return
  ```

- `rentalItemSelectionContext()`：E149 在 config 檢查之後，offset 59 有 `invokevirtual GameHub.hasRentalBattleHandoff`。和修補提案的 `rentalBattleLoadoutActive` 守衛是否等價，**待驗證**。
- 商人屬性來源（`RentalEquipmentBonuses`）：**沒核對**，待驗證。

**E5 伺服器候選與部署**（轉述伺服器端文件，沒有獨立驗證部署狀態）：
- lito-launcher-server `2066774` 的 `PLAN.md:15`：「大量隔離候選（最新 `50635f7e…`）**全部未部署**」。
- `lito伺服器/LiTo381-Server/docs/maintenance/combat-closeout-current.zh-TW.md:7`：「目前候選：`046c58c5a3609404a8a673e16d0e98366207850f43de2523e0c032f0fc69e76f`」。
- 兩份文件寫的最新候選不同，這是伺服器端的文件不一致。

**E6 產品範圍**（轉述）：lito-launcher-server `lito伺服器/LiTo381-Server/docs/maintenance/combat-continuation-current.zh-TW.md:14`：「使用者確認 **LiTo只有競技場，沒有WORLD地圖**…WORLD只保留既有後端回歸，不再列產品完成門檻」。

## 怎麼驗證
- 張數核對：
  - M2：1+1+2+1+6+10+2+3+1+3=30；M3：1+1+1+2+1+2+15+1+1+1=26；主線分母 5+1+4=10。
  - 技能：38+48+4+91=181；T022 各批 9+9+7+11+11+10+8+7+9+10=91；T027 每批 8 個，共 6 批＝48。
- `grep -n "^## \|^### " TASKS.md`：`## M*` 區段下只有主線項目，M2／M3 下沒有 `###` 項目。

## 未驗證
- 同端點守衛是否等價、商人屬性來源（E4）。
- 伺服器候選是否真的未部署（E5）。
- 475 案前 141 案的逐案名稱。
- 各環境前置條件在使用者電腦上是否存在。
- 所有張數估計都是推測。

## 附錄 A：HISTORY 全文 → 歸屬對照
下表的行號是原始行號（blob `7dd82ba`）。同一件事重複出現的，只列代表行。

| 原始行號 | 事項 | 歸屬 |
|---|---|---|
| L1–L15 | #97：共享 callback、重入繪圖、4D6570 交錯、native clock／rollover、原像素 oracle、資源就緒 gate 位置、4E2150 suffix、4857B0 drain、方向 6／shadow、library-test 與七個 metadata 重編 | T020 |
| L13、L15 | #97／#98：S34／S08 現身、材料、低 MP／HP、自然期限 | T021 |
| L1–L13、L11–L17 | 剩下的請求入口；selector_only helper 與 coverage parser | T022、T027 |
| L17–L65 | #41–#91 各技能的待測條件（原資源、History、音效、自然期限、同意／拒絕、HP 成本 279／278 等） | T027 |
| L1–L25 | #87 4183：wire→model、S0B 重送、idle 守衛 | T024 |
| L1–L65 | LT hurt／base、action0 倍率、uint32 rollover、#45 class7 | T024 |
| L1–L47 | G06 物品、source／account、timer 正向前提 | 排除（E149 沒有生產者） |
| L45 | G06 finite History 四 profile | T023 |
| L31、L33 | #69 S55-C4E-S5A-C08、EOF 取消、八秒失敗、同 socket 重選 | T027 批 5、T023 |
| L1、L3–L37 | 12 gate 案、四元件、G06 16 案 640 次 | 排除（候選資格） |
| L69–L75、L1230 | 同端點 C02（selection=0） | U-M3-1（依賴伺服器） |
| L93–L107、L185–L193 | G06 舊失敗 45538／80000／75086／4694，根因未定 | U-M1-1 |
| L93–L149 | 13／60 Release 真實施法、SELF_TICK 呼叫者時鐘 | T028 |
| L71、L73、L1586 | C72D 499 過 1 敗 | T036 |
| L83、L85 | 66470 紅測試要用同一斷言重驗 | T036 |
| L159、L171、L1204 | G11：60 分／20 週期、warm reload | T038 |
| L159–L197、L1150–L1573 | 實體輸入、IME、多 DPI、alt-tab；只在 192DPI 私有桌面 | T034、T030 |
| L1411–L1417 | Release OS SendInput 探針失敗 | T035 |
| L833、L866、L909 | FrameArrived 逾時 | T035、U-V1-2 |
| L163–L193、L1569 | clean host、VCRUNTIME140、5 處 hardcode | T031 |
| L1169、L1293、L1571 | 音效：實體裝置、無裝置／拔插 | T034、T030 |
| L137–L497、L1145–L1588 | 完整 G01–G12 未完成 | T033、T037 |
| L141–L197、L217–L471、L711–L1054 | 完整 library、TCP、Release 廣域回歸 | T036 |
| L327、L339、L1265 | 19／35／57 較高治癒的其他目標 | T028 |
| L1282–L1304 | 日光術：夜間亮度、3600 秒自然到期 | T028 |
| L1580、L1584 | 72：3934 空影格 slots policy | T028 |
| L163、L203、L1425 | Disintegrate 舊來源路徑 | T028 |
| L1026–L1131 | 6、8、9、11、10、28、18 的其他目標、clock、自然到期 | T028 |
| L635、L671–L725 | 63 的 320 秒；其他地面、攻擊、投射物技能 | T028、T022 |
| L1106、L1109、L753 | 遠程 arrow／flightType、alpha 命中、拒絕提示 | T029 |
| L713、L751、L1109 | 特殊變身與中斷、hurt queue | T024 |
| L1052–L1062 | 觀戰單獨離場、初始鏡頭 | T023 |
| L1155、L1172、L1227 | 更多取消／舊世代、跨世代 wire | T023 |
| L1150 | TeleportMe 原圖示排版 | T023 |
| L515、L541、L957–L986、L1527–L1567 | 租裝、旅館、商人 GUI；3600／7200 秒自然租期 | T034、T030 |
| L1060、L1062、L505–L511、L1008 | 最後成員 room reset、租裝返回錯序（伺服器候選未部署） | T040、U-M3-1 |
| L529、L531、L996、L1268 | 商人 78 來源 | 排除（伺服器缺陷） |
| L399、L431 | 大廳 reselect 後斷線（C02 清空 equipment-vital） | 排除（伺服器端） |
| L1106–L1131 | 正式來源漂移（D7222DED、6928A147）後全面重驗 | U-M2-2 |
| L163–L305、L1298–L1578 | 照明、光罩、水下、map303、背景 scheduler、相機、Bank viewport、RGB565 | 排除（畫面等價研究），U-M3-5 |
| L69–L135、L179、L453 | map563 30 人回歸 | U-M3-6（T033 判定） |
| L555、L1078、L1086、L1175–L1199 | 普通 WORLD C05、131 進 map4、869／$748、WORLD 分支、攻城 P 鍵 | 排除（WORLD） |
| L711 等 | Debug 281／Release 282 編譯警告 | 排除（建置雜項） |
| L919、L93、L1014 | before-rebuild zip、NUL 診斷映像、外部 LiTo.bin | 排除（建置雜項） |
| L555–L809、L1044 | G04 生命週期：守護塔、碰撞、復活、尋路、C_ADDR | T034 清單項，T037 判定；不另開實作張 |
| L1, L65, L83–L89, L245–L295, L513 | 舊候選欄位描述、#41 原 TW 資格、重疊 Model、DBCS 血盟、S0B global、CRT UTC／local、schema45 | 只屬歷史版本的資格紀錄，當版不移記，不另開張 |

## 附錄 B：候選 overlay QA 案逐輪計數（HISTORY，全部標「全未跑」）
| 行號 | 新增 | 累計 | 技能／功能 |
|---|---|---|---|
| L65 | 14 | 155 | #41 |
| L63 | 10 | 165 | #45 核心 |
| L61 | 10 | 175 | #45 接收呈現 |
| L59 | 10 | 185 | #45 本機入口 |
| L57 | 17 | 202 | #46／#47 |
| L55 | 12 | 214 | #50 |
| L53 | 12 | 226 | #51 |
| L51 | 14 | 240 | #55／#56 |
| L49 | 12 | 252 | #59 |
| L47 | 15 | 267 | #61 |
| L45 | 9 | 276 | G06 finite History |
| L43 | 14 | 290 | #62／#64 |
| L41 | 15 | 305 | #65／#66 |
| L39 | 7 | 312 | S27 佇列 |
| L37 | 10 | 322 | S27 World |
| L35 | 16 | 338 | #67／#68 |
| L33 | 24 | 362 | #70（含 #69 核心） |
| L31 | 10 | 372 | #69 |
| L29 | 14 | 386 | #71／#74 |
| L27 | 16 | 402 | #75／#76 |
| L25 | 13 | 415 | #80／#87 |
| L23 | 16 | 431 | #87 4183 |
| L21 | 10 | 441 | #88／#89 |
| L19 | 12 | 453 | #90／#91 |
| L17 | 2 | 455 | #80 選單 |
| L13 | 4 | 459 | #97／#98 |
| L5 | 4 | 463 | #97 ingress |
| L3 | 8 | 471 | #97 queue |
| L1 | 4（另改寫原 8 案） | 475 | #97 ready-gate |

L65 之前的 141 案，HISTORY 沒有分項。L137–L1588 沒有「新 N 案／共 M 案全未跑」這類計數（三段都用 grep 確認過），只有已執行的 session 情境數。

## 相依
- 本分支以 `task/86` 為基底；PR 的 base 設為 `task/86`，送審差異只含 #92 自己的檔案。
- 後續建議（不在本張範圍）：CLAUDE.md 那句「475 個測試案全未跑」容易和 574／481 混淆，建議另開 Issue 改寫。
