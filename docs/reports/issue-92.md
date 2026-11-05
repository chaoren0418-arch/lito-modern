# #92 範圍凍結：完成定義、主線封頂、backlog 分流

結論（第 3 版）：
- 步驟 0 已完成，所有來源都逐檔讀完，沒有抽讀：
  - HISTORY 全文；
  - 51 份 NATIVE 文件；
  - 75 份研究紀錄；
  - 伺服器 398 個 build*.py，另加不在 build*.py 裡的 3 段。
- 主線（M1＋V1）剩 **4 項，封頂**。
- M2、M3 的完成定義已凍結，任務各 **46 張、31 張**，都已一次列完並放在 Backlog，等開放時原樣升級。
- 完成定義會用到的腳本擴充，已列成 T041（M2）與 T042（M3）。
- 「已驗證」的說法都可以用 `維護工具/check-issue-92-evidence.sh` 重跑，原始輸出附在「證據」一節。

## 第 3 次嘗試的修正（對應第 2 輪 codex-review）
第 2 輪審查的完整內容存在審查者本機的 `scratch/review-92.md`。本機 `gh` 回 HTTP 401，沒能寫回 GitHub，所以這裡依調度器轉貼的 3 點摘要修正。若完整內容還有其他點，請再退回。
1. **盤點前置未完成**：
   - 補讀研究紀錄：`docs/NATIVE-*.zh-TW.md` 51 檔、`維護工具/*.md` 75 檔，逐檔全文讀完（附錄 C）。
   - 補讀建置鏈：伺服器 398 個 `build*.py`，以及鏈中不在 build*.py 的 3 段，重建 0.6.35→0.6.71 的 patch 鏈（附錄 D）。
   - 依此新增 M2 任務：T041、T043、T044、T045、T046、T047、T048、T050；新增 M3 任務：T042、T049、T052。
   - 依此擴大既有任務：T023、T028、T029 各加 1 張；T025、T033、T034、T035、T036 擴大範圍（張數不變）。
   - 新增不確定項 U-M3-7、U-M3-8，以及多項排除。
   - 結果：M2 由 30 張改為 46 張，M3 由 26 張改為 31 張；主線不變。
   - 更正：上一版說 HISTORY「去掉 850 行逐字重複」，不正確。實際是 849 行空白加 2 行重複，內容 738 行，4 段都已逐行讀完。
2. **漏列驗收所需的腳本擴充**：
   - M2 的完成定義要求 `run-local-test-suite.ps1` 總表多出「技能」「G06」兩類，新增 T041。
   - M3 的完成定義要求 `-Full` 模式，新增 T042。
3. **證據缺完整指令與原始輸出**：
   - 新增 `維護工具/check-issue-92-evidence.sh`，涵蓋 E0–E8，可以重跑；本報告附上它的完整原始輸出。
   - 新證據 E8：E149 已內含伺服器戰鬥候選鏈到 `8d26c9d7` 為止，所以「候選未部署」改為「8d26 之後的層未部署」。

（第 2 次嘗試的修正內容保留在 git 歷史：commit 6009727。）

## 移到 backlog 的項目
- **預定 M2（46 張）**：
  - T025 版本綁定 1、T026 技能狀態表 1、T020 #97 2、T021 98–100 1；
  - T027 候選獨有 48 技能 6、T022 未接 91 技能 10；
  - T024 4183 與 hurt 2、T028 main 38 技能補驗 4、T029 非技能戰鬥 2、T023 G06 客戶端 4；
  - T041 測試腳本加類別 1、T043 雙真實玩家 fixture 2、T044 移動正確性 2、T045 物品系統 2、T046 快捷列與狀態 UI 3、T047 影格事件 1、T048 復活分支 1、T050 完整性挑戰 1。
- **預定 M3（31 張）**：
  - T040 修補提案移交 1、T033 G01–G12 定義 1、T034 GUI 清單 1、T035 GUI／輸入缺陷 2、T030 實機 GUI 1；
  - T031 可攜交付 2、T036 廣域回歸 15、T038 G11 1、T037 稽核 1、T032 ARCHITECTURE 1；
  - T042 `-Full` 1、T049 背包與介面文字 2、T052 裝備與地面物品 UI 2。
- **排除或延後**：
  - WORLD 地圖專屬；凍結候選資格檢查（含 475 案、四元件、元件矩陣）；G06 正向前提；
  - 伺服器缺陷（商人屬性來源）；超出 G11 的效能最佳化；`TW13081901.pak` 缺檔；
  - 畫面等價研究；建置雜項；
  - 盟戰；see-invisible／GM 視野；C0B remembered-ID；資源缺件；原版等價研究（中斷順序、延遲補償、細分座標）；
  - 交易；VS-server directory；多客戶端即時競態。

## 主線剩餘數
| 區段 | 完成 | 剩餘 |
|---|---|---|
| M0 | 5（T001–T005） | 0 |
| M1 | 1（T010） | 2（T011-H、T011-B） |
| M1-V1 | 0 | 2（T012、T013） |
| 合計 | 6 | **4**（分母 10） |

- 分母只有在 U-M1-1、U-V1-1、U-V1-2 兌現時可以增加，最多到 13。兌現時 PR 要引用編號與估算。
- M2、M3 開放時（需要 `needs:user`）分別加 46、31，不確定項上限各為 +8、+20。

## 證據
重跑指令（唯讀；只在暫存目錄解出 class，不建置、不連服務）：

```bash
bash 維護工具/check-issue-92-evidence.sh /path/to/lito-modern /path/to/lito-launcher-server /path/to/scratch/issue-92-ev
```

各段說明：
- E0：兩個 repo 的 HEAD 與 HISTORY 的 blob。
- E1：E149 的 SHA256。
- E2：catalog 常數比對，以及 `new *Spell(` 的出現次數。
- E3：技能集合，逐 ID 去重。
- E4：`ClientSession` 四個方法的 javap 輸出。
- E5、E6：伺服器文件的原句，屬於轉述。
- E7：HISTORY 的行數統計。
- E8：E149 內含的候選層。

判定：
- 已驗證：E1、E2、E3、E4 的 quitFight、broadcastDefaultRoomReset、exitRoom、E7、E8。
- 待驗證：E4 的 `rentalItemSelectionContext`（offset 59 呼叫 `hasRentalBattleHandoff`）與修補提案是否等價；商人屬性來源沒核對。
- 只是轉述：E5、E6。

原始輸出（2026-11-06 在雲端環境執行，原樣貼上）：

```
== E0 版本
6ff18e60a29c70a7e5c0738abac4d2c123acd229
2066774824b314b2c65834f9afe72b68f7391f06
7dd82ba63afc6f86cac8018f70d0e66990343104
== E1 JAR SHA256
e1498871c7be6b287821c57ef5d95933db1740d9b4d4a80f6c3465f21d99d498  lito伺服器/LiTo381-Server/reference/base-0.6.71.jar
== E2 catalog 常數
223
223
diff: 相同
src new *Spell( = 186
jar new BattleSpellCatalog$*Spell = 186
for m in re.finditer(r"new (\w+Spell)\(", cat):
rows.append((5, "TELEPORT", 0, 4, "（dispatcher 特例）", SHAPE["teleportSpellRequest"],
             f"useTeleportSpell CS:{tl}；dispatcher 在 phase 檢查前轉交（useSpellBeforeDragonTargets CS:32165 起）"))
== E3 技能集合
catalog 181
main 38 [1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 13, 14, 15, 16, 17, 18, 19, 26, 28, 31, 32, 34, 35, 38, 42, 43, 49, 52, 54, 57, 58, 60, 63, 72, 77, 78, 131, 206]
candidate-only 48 [5, 12, 20, 21, 22, 23, 25, 27, 29, 30, 33, 36, 37, 39, 40, 41, 44, 45, 46, 47, 48, 50, 51, 53, 55, 56, 59, 61, 62, 64, 65, 66, 67, 68, 69, 70, 71, 73, 74, 75, 76, 79, 80, 87, 88, 89, 90, 91]
97-100 4
rest 91 [101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 129, 130, 132, 133, 134, 137, 138, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 201, 202, 203, 204, 205, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220]
sum 181 overlap 0
/home/user/lito-modern/LiTo381-Modern/src/bin/native_backend_485f_candidate.rs:122:                assert_eq!(maps.manifest_pin.sha256[..4], [0x14, 0x80, 0x5b, 0xc5]);
（以上為 native_skills.rs 以外的 0x14 開頭請求，空白表示沒有）
== E4 ClientSession（E149）
--- private void quitFight() throws（cs.txt:43123）
  private void quitFight() throws java.io.IOException;
    Code:
       0: aload_0
       1: getfield      #3506               // Field phase:Llocal/lito381/server/ClientSession$Phase;
       4: getstatic     #4078               // Field local/lito381/server/ClientSession$Phase.BATTLE:Llocal/lito381/server/ClientSession$Phase;
       7: if_acmpne     15
      10: aload_0
      11: invokevirtual #16685              // Method settleRequestedBattleExit:()V
      14: return
--- private void broadcastDefaultRoomReset(long) throws（cs.txt:42951）
  private void broadcastDefaultRoomReset(long) throws java.io.IOException;
    Code:
       0: aload_0
       1: getfield      #3441               // Field hub:Llocal/lito381/server/GameHub;
       4: aload_0
       5: getfield      #3689               // Field currentChannelId:I
       8: invokevirtual #13115              // Method local/lito381/server/GameHub.roomSnapshots:(I)Ljava/util/List;
      11: invokeinterface #4744,  1         // InterfaceMethod java/util/List.stream:()Ljava/util/stream/Stream;
      16: lload_1
      17: invokedynamic #16649,  0          // InvokeDynamic #852:test:(J)Ljava/util/function/Predicate;
      22: invokeinterface #4761,  2         // InterfaceMethod java/util/stream/Stream.filter:(Ljava/util/function/Predicate;)Ljava/util/stream/Stream;
      27: invokeinterface #8942,  1         // InterfaceMethod java/util/stream/Stream.findFirst:()Ljava/util/Optional;
      32: lload_1
      33: invokedynamic #16657,  0          // InvokeDynamic #853:get:(J)Ljava/util/function/Supplier;
      38: invokevirtual #8955               // Method java/util/Optional.orElseThrow:(Ljava/util/function/Supplier;)Ljava/lang/Object;
      41: checkcast     #857                // class local/lito381/server/GameHub$RoomSnapshot
--- private void exitRoom(boolean) throws（cs.txt:302531）
  private void exitRoom(boolean) throws java.io.IOException;
    Code:
       0: aload_0
       1: iload_1
       2: invokestatic  #52216              // Method local/lito381/server/GmPolyMe.exitRoom:(Llocal/lito381/server/ClientSession;Z)V
       5: return
}
--- private boolean rentalItemSelectionContext();（cs.txt:29507）
  private boolean rentalItemSelectionContext();
    Code:
       0: aload_0
       1: getfield      #4032               // Field config:Llocal/lito381/server/ServerConfig;
       4: invokevirtual #10736              // Method local/lito381/server/ServerConfig.rentalTownDirectoryEnabled:()Z
       7: ifeq          69
      10: aload_0
      11: getfield      #10738              // Field rentalTownDirectoryAdvertised:Z
      14: ifeq          69
      17: aload_0
      18: getfield      #3506               // Field phase:Llocal/lito381/server/ClientSession$Phase;
      21: getstatic     #10931              // Field local/lito381/server/ClientSession$Phase.LOBBY:Llocal/lito381/server/ClientSession$Phase;
      24: if_acmpne     69
      27: aload_0
      28: getfield      #4556               // Field selectedCharacter:Ljava/lang/String;
      31: invokevirtual #6224               // Method java/lang/String.isEmpty:()Z
      34: ifne          69
      37: aload_0
      38: getfield      #3441               // Field hub:Llocal/lito381/server/GameHub;
      41: aload_0
      42: invokevirtual #4839               // Method accountName:()Ljava/lang/String;
      45: aload_0
      46: invokevirtual #4562               // Method stateCharacter:()Ljava/lang/String;
      49: astore_2
      50: astore_1
      51: astore_3
      52: aload_3
      53: aload_0
      54: aload_1
      55: invokestatic  #10597              // Method local/lito381/server/SourceLogin.account:(Llocal/lito381/server/ClientSession;Ljava/lang/String;)Ljava/lang/String;
      58: aload_2
      59: invokevirtual #13025              // Method local/lito381/server/GameHub.hasRentalBattleHandoff:(Ljava/lang/String;Ljava/lang/String;)Z
      62: ifne          69
      65: iconst_1
      66: goto          70
== E5 伺服器文件
| 戰鬥技能同源化 | 大量隔離候選（最新 `50635f7e…`，2,400 案／5,164 組合）**全部未部署**；開放缺口見 TASKS M2 |
目前候選：`046c58c5a3609404a8a673e16d0e98366207850f43de2523e0c032f0fc69e76f`。
== E6
| **產品場景澄清** | 使用者確認 **LiTo只有競技場，沒有WORLD地圖**。全部目標／Ctrl與游標以競技模式、實際隊伍／關係及回大廳驗收；WORLD只保留既有後端回歸，不再列產品完成門檻。歷史收據維持原樣。[範圍](../.
== E7 HISTORY 去重
lines 1589 nonempty 740 unique 738 dup 2
== E8 E149 內含的戰鬥候選層（8d26 鏈）與未含的下一層
    17020  1980-01-01 00:00   local/lito381/server/NativeCancellationBattleRuntime.class
     3314  1980-01-01 00:00   local/lito381/server/NativeMagicalWeaponAction.class
     5746  1980-01-01 00:00   local/lito381/server/NativeMassTeleportAnswerLife.class
（NativeStaticNpcScene 未出現＝cancellation-static-runtime 之後未部署）
| 候選配對 | 採既有後端已驗證核心 `8d26c9d7` 為輸入，保留其他已完成戰鬥工作。啟動入口核對成對 DLL／JAR，沿用原 READY／所有權／關閉程式，伺服器執行輸出隔離。未覆蓋其他工作中的候選。 |
```

## 怎麼驗證
- 張數核對：
  - M2：1+1+2+1+6+10+2+4+2+4+1+2+2+2+3+1+1+1=46；M3：1+1+1+2+1+2+15+1+1+1+1+2+2=31；主線分母 5+1+4=10。
  - 技能：38+48+4+91=181；T022 各批 9+9+7+11+11+10+8+7+9+10=91；T027 每批 8 個，共 6 批＝48。
- `grep -n "^## \|^### " TASKS.md`：`## M*` 區段下只有主線項目，M2／M3 下沒有 `###` 項目。

## 未驗證
- 同端點守衛是否等價、商人屬性來源（E4）。
- 伺服器文件之間的候選 SHA 不一致（E5）。
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

## 附錄 C：研究紀錄全讀 → 新增或擴大的任務
NATIVE 文件 51 檔、`維護工具/*.md` 75 檔，逐檔全文讀完，總數與清單一致。下表只列「舊規劃沒涵蓋」的項目；其他已對到既有任務或排除項。

| 出處 | 事項 | 歸屬 |
|---|---|---|
| NATIVE-EMBLEMS:29–31 | E149 的 startBattle 關閉訓練 bot，54 情境要用兩個真實玩家重做 | T043（M2，2 張） |
| NATIVE-ANIMATION:27、NATIVE-MOTION:17、NATIVE-NAVIGATION:15、NATIVE-WORLD-INTEGRATION:16、:40 | 在途移動、靜默拒絕、碰撞、動態門與占位、城門、G04 剩餘項 | T044（2 張） |
| NATIVE-ITEM-ACTIONS:9、:39、NATIVE-GROUND-SKILLS:33 | 帶目標或選項的物品、type3、194／197、C76、耗材 | T045（2 張） |
| NATIVE-SKILLS:41、NATIVE-SKILL-SELECTION:27、NATIVE-REAGENT-SKILLS:31、NATIVE-ATTACK-SKILLS:3、:32 | 快捷列、技能書、buff／AC／冷卻／MP UI、技能中斷 | T046（3 張） |
| NATIVE-ANIMATION:11、NATIVE-COMBAT:30、NATIVE-WORLD:24、:34 | 影格事件 Effect／Hit；gfx1544 action0 | T047（1 張） |
| NATIVE-OUTCOMES:13、NATIVE-CONTINUOUS:22 | S_RESURRECT 等復活分支 | T048（1 張） |
| NATIVE-WORLD:18 | subtype33 完整性挑戰，沒有送 C_ADDR | T050（1 張） |
| NATIVE-COMBAT:36、:38、NATIVE-NAVIGATION:23、NATIVE-OUTCOMES:19 | 自動追擊、目標圈、攻擊速度、延遲回覆重試 | T029 擴為 2 張 |
| `維護工具/native-E149-S23-clock*.md`、`class1-origin.md`、`fireball-*.md`；NATIVE-ATTACK-SKILLS:42 | 候選對 main 既有技能的修正要在 src 重寫；golden 測試；614 行 parser 錯誤 | T028 擴為 4 張 |
| `native-E149-extended-item-observation.md:15–27`、`integrated-client.md:15`、`recovery-case-contract.md:11`；NATIVE-WORLD-INTEGRATION:126、:164 | 可到達的物品接續拒絕與期限、T 鍵、cancel-late、畸形 wire | T023 擴為 4 張 |
| NATIVE-INVENTORY-ICONS:41、-NAMES:39、NATIVE-GAME:21、NATIVE-LOBBY:46、NATIVE-WORLD-INTEGRATION:84 | 祝福／詛咒、名稱模板、通知 Text、LTstring、S0F／S10 | T049（M3，2 張） |
| NATIVE-RANGED:13、:37、NATIVE-REAGENT-SKILLS:31、:33 | 裝備與彈藥 UI、地面直接拾取、特殊物品選項 | T052（M3，2 張） |
| NATIVE-INVENTORY:44；NATIVE-LOGIN:14 | 數量重複顯示；截圖逾時 | T035 |
| NATIVE-ROOM:46、NATIVE-TELEPORT:22、NATIVE-WORLD-ROOFS:35、NATIVE-WORLD-INTEGRATION:123 | 建房、中文密碼、圖示排版、屋頂進出、G09 音效事件 | T034 |
| NATIVE-WORLD-INTEGRATION:5、:9、:13、:150、:177；NATIVE-SCENE-LOADING:9；NATIVE-RESOURCE-LATENCY:17；NATIVE-LABEL-UPDATES:50 | 整庫 lib 記憶體失敗、runner 漏 pin、根因未定的失敗 | T036 |
| NATIVE-OPTIMIZED-PROFILE:18、NATIVE-WORLD-LABELS:13、:25、NATIVE-ROOM:48、NATIVE-WORLD-INTEGRATION:129 | emblem／稱號／名稱權限、ARGO、map500 入口 | T033 判定；U-M3-7、U-M3-8 |
| `native-E149-component-*.md`、`owned-mysql-README.md:19`、`mass69-integration.md:17` | 四元件、原版 6 反例、候選 14 案、元件矩陣 28 案 | 排除（候選資格，#58） |

## 附錄 D：伺服器建置鏈（0.6.35 → 0.6.71）
0.6.35 base（`3f31ef32`）依序經過：
1. shared-database
2. source-administration（`205dc1ba`）
3. dragon-skill-dispatch 0.6.46
4. dragon-combat 0.6.47
5. dragon-presentation 0.6.48
6. empty-melee 0.6.49
7. illusion-presentation 0.6.50
8. illusion-targets 0.6.51
9. room-directory 0.6.54
10. native-room-state 0.6.58
11. native-room-admission 0.6.59
12. durable-settlement 0.6.60
13. room-rules 0.6.61（`9719f45d`）
14. resource-database 0.6.62–0.6.68（`c02aaae8`）

接著分成兩支：
- **0.6.69 這支沒接上正式版**：buff-schema（`75da69b0`），已被 0.6.70 取代。
- **0.6.70 這支是正式版的路**：
  1. `de-native-*`（沒進 git）→ `df9464f0`
  2. → combat-continuation 根目錄（magical-weapon）
  3. → `integrate_buff.py`（`97b1318a`）
  4. → 約 65 層 → cancellation-battle-runtime（`8d26c9d7`）
  5. → 登入器 repo `tools/world-resource-compat/build_presentation_server.py`、`build_selection_recovery.py`、`build_class_icon_server.py`（`daedec7c`＝0.6.70）
  6. → `tools/gm-polyme/build.py`（`e1498871`＝0.6.71）

對照依據：登入器 repo 的 `docs/maintenance/release-0.6.71.json` 與 `release-0.6.70.json`。

**會影響客戶端的層**：
- Protocol 變動：empty-melee、native-room-state、dragon-skill-dispatch、summon-parity、presentation-server、class-icons。
- 技能結果：dragon 兩層、illusion-targets、resource-database、combat 鏈、buff。
- 房間與結算：room-directory、native-room-admission、room-rules、durable-settlement。
- 移動與傳送：native-room-state、MovementLock、GmTravel、mass-teleport。
- 租賃：shared-database 的 RentalBattleHandoff、moveRentalTown。

**伺服器換版時客戶端要重核的 12 項**已寫進 T025。

**要回報給伺服器端雲端對話**（不在本 repo 處理）：
- `combat-continuation-current.zh-TW.md:9-10`「本戰鬥候選未部署」應改為「8d26 之後未部署」。
- 伺服器 `PLAN.md` 寫的最新候選（`50635f7e`）與收尾頁（`046c58c5`）不一致。
- `de-native-*` 那段鏈沒進 git。
