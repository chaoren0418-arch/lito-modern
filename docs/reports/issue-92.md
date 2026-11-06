# #92 範圍凍結：完成定義、主線封頂、backlog 分流

結論（第 6 版）：
- **步驟 0 還差最後一段**：
  - 已讀完：
    - 雲端的 HISTORY、51 份 NATIVE 文件、75 份研究紀錄；
    - 伺服器已追蹤的 398 個 build*.py，以及 `integrate_buff.py` 與登入器 repo 的 `world-resource-compat/build_*.py`；
    - `docs/maintenance/` 86 檔、`server-source-gap.md`；
    - 本機的 G01–G12 共 46 項、候選 554 個測試宣告（#106）。
  - **還沒讀**：伺服器 `resource-database/build/de-native-*`（本機 18 個目錄、62 個 `build*.py`），被 `.gitignore` 排除，雲端讀不到。已開 **#120** 由本機唯讀全讀，回來後補完；在那之前列為待驗證。
  - 另外 11,285 個未追蹤的 `build*.py` 只有 #106 的路徑分類，沒有讀內容；#106 推測是建置產物或輸入快照。
  - 伺服器隔離候選讀到「建置鏈、收據與 class 清單」層級；候選 JAR 沒有反編譯，理由見 PLAN 的來源表。
- **對照**：G01–G12 的 46 項（附錄 E1）、候選 44 檔（附錄 E2）、伺服器 maintenance 86 檔（附錄 F）都已歸到任務、不確定項或有理由的排除。
- **主線**（M1＋V1）剩 **1 項，封頂**：T011-B，重跑 #118。T011-H、T011-F、T012、T013 已完成；分母 11。
- **M2、M3** 的完成定義已凍結，任務各 **57 張、44 張**（#120 若有新發現再調整），都已一次列完並放在 Backlog。
- **證據**：「已驗證」的說法都可以用 `維護工具/check-issue-92-evidence.sh` 重跑，原始輸出附在「證據」一節。

## 第 6 次嘗試的修正（對應第 5 輪 claude-local 審查，進行中）
1. **[P1] `de-native-*` 沒讀**：同意。結論、PLAN 來源表、PR 描述都已改成符合實際的說法；開 **#120**（本機 codex-local，唯讀）讀完這 62 個腳本，回來後把新發現歸到任務或排除項。未讀的 11,285 個未追蹤 build 檔也已寫明。
2. **[P2] 文字殘留**：
   - PLAN 的 T011-B 改指向 #118。
   - T022 批 2 補上 107 的候選參考。
   - 附錄 A 的 T033 改為 T037。
   - PLAN 標題改為第 6 版。
   - T026 的保留槽改用 39，並寫明舊文件記的是 34。

## 第 5 次嘗試的修正（對應第 4 輪 claude-local 審查）
1. **[P1] 主線落後 main**：
   - T011-F（PR #115）改為 done。
   - 開了 G06.3d 重跑 Issue #118（codex-local，已排入佇列），T011-B 指向 #118、狀態 todo。
   - 主線改為完成 10、剩 1、分母 11；PLAN、TASKS、報告、PR 描述都已同步。
2. **[P1] 伺服器研究紀錄**：
   - `docs/maintenance/` 86 檔（約 2.6 MB）與 `server-source-gap.md` 都在雲端的伺服器 clone 裡，所以直接逐檔全讀，不需要調整範圍：md／txt 70 份全文；16 份 JSON 用 python 走訪全部元素，`combat-coverage-current.json` 的 808 列與 `layout-cleanup` 的 1474 筆都核對「處理數＝陣列長度」。
   - 結果見附錄 F。新增 T061（Ctrl 強制目標 LFC1：Rust 客戶端完全沒有實作）、T062（選角 8 角色分頁）、T063（寵物與召喚物控制），並擴大 T023、T025、T026、T029、T038、T044、T045、T046、T047、T054、T057、T060。GM 工具依 G 定義排除。M2 改為 57 張。
   - 已刪除 PLAN 裡「這批已延後，不影響主線」的說法，改為引用 #106 的 554／493 與差額 18。計數口徑的裁定是主控決定，不是不可逆決策，所以不另外開 `needs:user`。
3. **[P1] 凍結內容**：
   - 屋頂從 WORLD 排除項拿掉（G07.3，已歸 T058）。
   - M3 完成定義改為由 T037 產出 `docs/G01-G12-AUDIT.md`。
   - T037 依 G12 擴為 2 張（證據保存與冗餘整理、46 項稽核），M3 改為 44 張。
4. **[P2] 文字同步**：
   - 「最多 +3／13」改為「分母固定 11」。
   - 「等 #106」改為「#106 已完成」。
   - L1052 改歸 T054；T033／U-M3-7 的引用都改為現行項目；T044、T047 的張數更正為 3 張、2 張。
   - E3 註明「候選獨有 48」的計算方式，以及 107 的例外（嚴格是 49／90）。

## 第 4 次嘗試的修正（對應第 3 輪 claude-local 審查）
1. **[P1] G01–G12 定義與排除衝突**：已依 #106 取回的原文（`docs/G01-G12.md`）逐條對照 46 項，結果見附錄 E1。
   - **重新歸屬**：
     - G06（房間與跨服務）：新增 T054（G06.1、G06.2：建房、設定、密碼、踢人、房主轉移、模式、隊伍、觀戰）與 T055（G06.3 租裝來源）。
     - G07（徽章、FA／S69 狀態、屋頂、附件）：新增 T056、T057、T058；不再排除 FA／S69 狀態呈現與附件缺件，缺件改成要求明確診斷。
     - G11（斷線注入）：T038 擴為 3 張。
     - G09：新增 T059。
     - G03：新增 T053（舊 54 情境）。
     - G04.4、G04.7：T044 擴為 3 張。
     - G05：T026 擴為玩法支援矩陣。
     - G07.2：T047 擴為 2 張。
     - 聊天：新增 T060。
   - **交易**：G08 的「交易控件」指 G08.2 的 NPC／租裝商店交易數量與權威貨幣提示，已歸 T055、T034。玩家之間的交易在 G01–G12 裡沒有這一項，所以排除；理由來自 G 定義，不是「只有競技場」。
   - **WORLD 排除**：只剩 G06.4（普通世界 C05）與 HISTORY 列出的 WORLD 專屬項目，依據是伺服器的產品範圍（E6）。
   - **G06 名稱**：工作佇列的 G06 是「房間管理與跨服務分支」，傳送只是 G06.3d。PLAN 已寫明，CLAUDE.md 的舊說法建議另開 Issue 修正。
   - **T033**：已由 #106 完成，從 M3 移除。結果是 M2 由 46 張改為 53 張，M3 由 31 張改為 43 張。
2. **[P1] 475 案**：#106 逐檔、逐函式列出 554 個測試宣告（545 直接＋9 條件），其中 61 個和 main 同名、493 個是候選新增。主控裁定：以 #106 的具名清單取代歷史計數 475；差額 18 沒有歷史具名名冊可以核對，記為「差額未解」。44 檔逐檔對到重寫任務，見附錄 E2。這些測試隨候選凍結，不另外執行。
3. **[P1] PR base**：已改為 main。
4. **[P2] 文字修正**：
   - M3 完成定義的啟動入口已改；T023、T035 的張數寫清楚。
   - T011-H 改為 done，主線同步。
   - T043 改為照原文轉述 EBF5EE66。

## 第 3 次嘗試的修正（對應第 2 輪 codex-review）
第 2 輪審查的完整內容存在審查者本機的 `scratch/review-92.md`。本機 `gh` 回 HTTP 401，沒能寫回 GitHub，所以這裡依調度器轉貼的 3 點摘要修正。若完整內容還有其他點，請再退回。
1. **盤點前置未完成**：
   - 補讀研究紀錄：`docs/NATIVE-*.zh-TW.md` 51 檔、`維護工具/*.md` 75 檔，逐檔全文讀完（附錄 C）。
   - 補讀建置鏈：伺服器 398 個 `build*.py`，以及鏈中不在 build*.py 的 3 段，重建 0.6.35→0.6.71 的 patch 鏈（附錄 D）。
   - 依此新增 M2 任務：T041、T043、T044、T045、T046、T047、T048、T050；新增 M3 任務：T042、T049、T052。
   - 依此擴大既有任務：T023、T028、T029 各加 1 張；T025、T033、T034、T035、T036 擴大範圍（張數不變）。
   - 新增不確定項 U-M3-8，以及多項排除（U-M3-7 後來在第 4 次改為任務 T056）。
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
- **預定 M2（57 張）**：
  - T025 1、T026 2、T020 2、T021 1；
  - T027 6、T022 10；
  - T024 2、T028 4、T029 2、T023 4；
  - T041 1、T043 2、T044 3、T045 2、T046 3、T047 2、T048 1、T050 1；
  - T054 2、T055 2、T061 2、T062 1、T063 1。
- **預定 M3（44 張）**：
  - T040 1、T034 1、T035 2、T030 1、T031 2、T036 15、T038 3、T037 2、T032 1；
  - T042 1、T049 2、T052 2；
  - T053 2、T056 2、T057 2、T058 2、T059 2、T060 1。
- **排除或延後**：
  - G06.4 與 WORLD 專屬；凍結候選資格檢查與候選測試；G06 正向前提；伺服器缺陷；
  - 畫面等價研究（沒有列在 G01–G12）；建置雜項；
  - 盟戰玩法（不含狀態呈現）；see-invisible／GM 視野；C0B remembered-ID；原版等價研究；
  - 玩家之間的交易；GM 工具；VS-server directory；多客戶端即時競態；`TW13081901.pak` 缺檔；
  - 超出 G11 的效能最佳化；伺服器內部項目（附錄 F）。

## 主線剩餘數
| 區段 | 完成 | 剩餘 |
|---|---|---|
| M0 | 5（T001–T005） | 0 |
| M1 | 3（T010、T011-H、T011-F） | 1（T011-B，重跑 #118） |
| M1-V1 | 2（T012、T013） | 0 |
| 合計 | 10 | **1**（分母 11；U-M1-1 兌現） |

- U-M1-1 已兌現為 T011-F；U-V1-1、U-V1-2 沒有兌現，已關閉。主線分母固定為 11。
- M2、M3 開放時（需要 `needs:user`）分別加 57、44，不確定項上限各為 +8、+18。

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
- 已驗證：E1、E2、E3（目錄 181、main 38 是解析得到；「候選獨有 48」是用 1–98 扣掉 main 與 97／98 算出來的，候選的 `item_targeted` 另含 107，嚴格是 49／90）、E4 的 quitFight、broadcastDefaultRoomReset、exitRoom、E7、E8。
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
  - M2：1+2+2+1+6+10+2+4+2+4+1+2+3+2+3+2+1+1+2+2+2+1+1=57；M3：1+1+2+1+2+15+3+2+1+1+2+2+2+2+2+2+2+1=44；主線分母 5+5+1=11。
  - 技能：38+48+4+91=181；T022 各批 9+9+7+11+11+10+8+7+9+10=91；T027 每批 8 個，共 6 批＝48。
- `grep -n "^## \|^### " TASKS.md`：`## M*` 區段下只有主線項目，M2／M3 下沒有 `###` 項目。

## 未驗證
- 伺服器 `de-native-*` 建置鏈：62 個 `build*.py` 的內容由 #120 讀取中；另外 11,285 個未追蹤的 `build*.py` 沒有讀內容。
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
| L137–L497、L1145–L1588 | 完整 G01–G12 未完成 | T037 |
| L141–L197、L217–L471、L711–L1054 | 完整 library、TCP、Release 廣域回歸 | T036 |
| L327、L339、L1265 | 19／35／57 較高治癒的其他目標 | T028 |
| L1282–L1304 | 日光術：夜間亮度、3600 秒自然到期 | T028 |
| L1580、L1584 | 72：3934 空影格 slots policy | T028 |
| L163、L203、L1425 | Disintegrate 舊來源路徑 | T028 |
| L1026–L1131 | 6、8、9、11、10、28、18 的其他目標、clock、自然到期 | T028 |
| L635、L671–L725 | 63 的 320 秒；其他地面、攻擊、投射物技能 | T028、T022 |
| L1106、L1109、L753 | 遠程 arrow／flightType、alpha 命中、拒絕提示 | T029 |
| L713、L751、L1109 | 特殊變身與中斷、hurt queue | T024 |
| L1052–L1062 | 觀戰單獨離場、初始鏡頭 | T054 |
| L1155、L1172、L1227 | 更多取消／舊世代、跨世代 wire | T023 |
| L1150 | TeleportMe 原圖示排版 | T023 |
| L515、L541、L957–L986、L1527–L1567 | 租裝、旅館、商人 GUI；3600／7200 秒自然租期 | T034、T030 |
| L1060、L1062、L505–L511、L1008 | 最後成員 room reset、租裝返回錯序（伺服器候選未部署） | T040、U-M3-1 |
| L529、L531、L996、L1268 | 商人 78 來源 | 排除（伺服器缺陷） |
| L399、L431 | 大廳 reselect 後斷線（C02 清空 equipment-vital） | 排除（伺服器端） |
| L1106–L1131 | 正式來源漂移（D7222DED、6928A147）後全面重驗 | U-M2-2 |
| L163–L305、L1298–L1578 | 照明、光罩、水下、map303、背景 scheduler、相機、Bank viewport、RGB565 | 排除（畫面等價研究），U-M3-5 |
| L69–L135、L179、L453 | map563 30 人回歸 | U-M3-6（T037 判定） |
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
- PR #99 的 base 已改為 main（#98 已合併）；送審差異只含 #92 自己的 4 個檔案。
- #106 已完成（G01–G12 原文、候選 554 個測試宣告）。
- 後續建議（不在本張範圍）：CLAUDE.md 那句「475 個測試案全未跑」容易和 574／481 混淆，建議另開 Issue 改寫。

## 附錄 C：研究紀錄全讀 → 新增或擴大的任務
NATIVE 文件 51 檔、`維護工具/*.md` 75 檔，逐檔全文讀完，總數與清單一致。下表只列「舊規劃沒涵蓋」的項目；其他已對到既有任務或排除項。

| 出處 | 事項 | 歸屬 |
|---|---|---|
| NATIVE-EMBLEMS:29–31 | `EBF5EE66`（E149 的祖先）起，startBattle 關閉訓練 bot（E149 待核對），54 情境要用兩個真實玩家重做 | T043（M2，2 張） |
| NATIVE-ANIMATION:27、NATIVE-MOTION:17、NATIVE-NAVIGATION:15、NATIVE-WORLD-INTEGRATION:16、:40 | 在途移動、靜默拒絕、碰撞、動態門與占位、城門、G04 剩餘項 | T044（3 張） |
| NATIVE-ITEM-ACTIONS:9、:39、NATIVE-GROUND-SKILLS:33 | 帶目標或選項的物品、type3、194／197、C76、耗材 | T045（2 張） |
| NATIVE-SKILLS:41、NATIVE-SKILL-SELECTION:27、NATIVE-REAGENT-SKILLS:31、NATIVE-ATTACK-SKILLS:3、:32 | 快捷列、技能書、buff／AC／冷卻／MP UI、技能中斷 | T046（3 張） |
| NATIVE-ANIMATION:11、NATIVE-COMBAT:30、NATIVE-WORLD:24、:34 | 影格事件 Effect／Hit；gfx1544 action0 | T047（2 張） |
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
| NATIVE-OPTIMIZED-PROFILE:18、NATIVE-WORLD-LABELS:13、:25、NATIVE-ROOM:48、NATIVE-WORLD-INTEGRATION:129 | emblem／稱號／名稱權限、ARGO、map500 入口 | emblem 與名稱歸 T056、map500 入口歸 T055、ARGO 歸 U-M3-8（T037 判定） |
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

## 附錄 E1：G01–G12（46 項）→ 歸屬
來源是 `docs/G01-G12.md`（#106），「state」是來源記載的原值，不是重新驗收的結論。

| 項目 | 來源 state（摘） | 歸屬 |
|---|---|---|
| G01 重建現況與版本基準 | catalog_normal_two_profile_related_baseline_qualified_limited | T025；#92 本身處理「每個剩餘項目有依賴、驗收條件」 |
| G02 兩名實際玩家的雙場戰鬥與結算 | current_source_required_scope_incomplete… | T043 |
| G03 回歸探針與新技能／服務整合 | …broad_not_repeated_GUI_pending | T053、T036、T041、T042 |
| G04 多人移動、碰撞及世界生命週期 | …incomplete… | T044 |
| G04.1 八方向、同格、權威位置 | verified_defined_scope | T036 回歸重驗 |
| G04.2 競技場門、動態通行、持續姿態 | verified_defined_scope | T036 回歸重驗 |
| G04.3 觀戰範圍與隨從主人封包 | verified_packet_state_scope | T054（觀戰）、T036 |
| G04.4 加入離開、死亡復活、換圖、同 ID 替換 | in_progress | T044、T048 |
| G04.5 移動鎖、速度、追擊、持續輸入中斷 | verified_input_lifecycle_scope | T029、T036 |
| G04.6 守護塔生命狀態與動態路徑阻擋 | verified_contract_and_route_scope | T036 |
| G04.7 其他模式城門、任務怪物、限時物件、進出 | pending | T044 |
| G05 物品、裝備、技能及狀態完整性 | static_request38_of_book74_missing36… | T026 矩陣，以及 T020–T022、T027、T028、T045、T046、T052 |
| G05.1 能力值、經驗、AC、抗性、SP／MR | verified_world_schema0_subset | T036、T049（屬性顯示） |
| G05.2 技能、道具、裝備成功／拒絕矩陣 | …full_gameplay_matrix_GUI_pending | T026、T027、T022、T028、T043 |
| G05.3 守護塔攻擊與狀態型目標 | verified_…_subset | T036 |
| G05.4 78 項商人附加屬性 | …client_authority_handling_and_real_modes_pending | T026（依權威值顯示）、T040（伺服器缺陷移交） |
| G06 房間管理與跨服務分支 | …API_controller_and_GUI_branches_pending | T054、T055、T023 |
| G06.1 建房、設定、密碼、踢人、房主轉移 | verified_controller_live_subset | T054、T034 |
| G06.2 模式切換、隊伍、準備、觀戰 | …candidate_verified_formal_integration_pending | T054 |
| G06.3 租裝來源、村莊、商店與交接 | in_progress | T055 |
| G06.3a 服務清單、配裝、原資源入村 | verified_entry_subset | T055、T036 |
| G06.3b NPC 對話、商品、光之寶石、租還 | all78_…_verified | T036、T034 |
| G06.3c 正常返回、重新入村、服務物品 | candidate_…_formal_integration_pending | T055、T040、U-M3-1 |
| G06.3d 旅館、傳送與跨端點服務 | current_source_controller_qualified_same_endpoint_…_pending | T011-F、T011-B（主線）、T023、U-M3-1（同端點） |
| G06.3e 原始對話資源與完整玩家 GUI | …physical_full_flow_pending | T034、T030 |
| G06.4 普通世界 C05 入口 | native_directory_boundary_verified_other_routes_pending | **排除**（WORLD，E6） |
| G07 既有資源與完整世界呈現 | …incomplete… | T056、T057、T058、T047、T049 |
| G07.1 資源命名空間、雜湊、格式矩陣 | …exhaustive_member_coverage_pending | T057 |
| G07.2 可見角色、NPC、狀態物件、原始動作 | pending | T047 |
| G07.3 地形、跨區塊、遮擋、屋頂、附件、名稱、特效排序 | partially_verified | T058 |
| G07.4 失敗資源診斷與玩家操作完整性 | pending | T057 |
| G07.5 S_E8 小地圖、世界退役、原生視窗 | …full_observer_gui_pending | T034、T030 |
| G08 唯一入口的完整玩家操作 | …terminal_scope_qualification_pending | T034、T035、T030、T046 |
| G08.1 房間建立／修改、踢人、觀戰、模式控件 | …physical_input_pending | T034、T030、T054 |
| G08.2 租裝服務、NPC 對話、交易數量、貨幣提示 | …other_services_pending | T055、T034、T030 |
| G08.2a 徽章與市民資格頁 | …physical_input_pending | T034、T030 |
| G08.2b 商人與容量／重量／到期 | …weapon_bonus_physical_pending | T034、T030、T040（weapon bonus） |
| G08.2c 旅館租房、鑰匙、退房 | …self_exit_physical_input_pending | T034、T030 |
| G08.2d 四位傳送 NPC 與跨服務交接 | …client_transfer_pending | T023、T034 |
| G08.2e 鍵盤走訪與完整 NPC GUI | …physical_input_pending | T034、T030 |
| G08.3 屬性、狀態、系統訊息、小地圖 | …full_attributes_multidpi_pending | T049、T034 |
| G08.4 唯一入口完整流程、IME、DPI、焦點、取消 | …ime_dpi_physical_input_pending | T034、T035、T030 |
| G09 原始音效與裝置生命週期 | …incomplete… | T059、T030 |
| G10 Release 全流程與可攜交付 | …incomplete… | T031、T042、T036 |
| G11 效能、長時間、故障恢復 | …incomplete… | T038、T050 |
| G12 整理與整體完成稽核 | pending_full_G01_G12_current_source_acceptance | T037（2 張：證據保存與冗餘整理、46 項稽核）、T032 |

CLAUDE.md 目標裡的「聊天」在 G01–G12 沒有單獨一項，另外新增 T060。

## 附錄 E2：候選 44 檔（554 個測試宣告）→ 重寫任務
數字是 #106 報告的「候選新增」欄（候選測試數減掉和 main 同名的測試數），合計 493。這些測試隨候選凍結，不另外執行；重寫時只當設計參考。

| 任務 | 候選檔（#106 表格列號） | 候選新增測試 |
|---|---|---:|
| T020 #97 | 2 native_world_scene、29 native_world_view、34 tower_assets、43 native_connection、44 native_static2941 | 39 |
| T023 G06.3d | 8 native_login、9 native_rental_return_tests、10 native_teleport_controller_tests、11 tests/native_teleport_failure；12–14 是 Java oracle，0 個測試 | 27 |
| T024 | 26 native_world、38 native_world_animation | 10 |
| T028 | 1 world_raster、23 light_arrow_effect、25 native_class1、27 native_audio_delivery、28 native_spell_projectiles | 52 |
| T046 | 4 tests/native_skill_selection、6 native_skill_selection | 3 |
| T022 | 21 tests/native_shadow107（107 在「未接 91」裡，候選有參考實作） | 6 |
| T027 | 5 native_skills、7 tests/native_blind、15 native_additional_finite_effects、16 tests/native_slow29、18 tests/native_advance79、19 tests/native_item_spells、20 tests/native_magical73、22 tests/native_object374448、24 native_attack_many、30 native_skill_effects、31 native_eruption45、33 native_eruption45_scene、35 native_blizzard59、36 native_resurrection61、37 native_appearance27、40 native_shape67、41 native_mass69、42 native_fire70 | 356 |
| 沒有新增測試 | 3 native_login_window、17 native_selector_effects、32 native_eruption45_assets、39 native_combat_animation（4 檔都是 0 個測試）；另外 34 tower_assets 有 7 個測試都和 main 同名，已列在 T020 | 0 |
| **合計** | 44 檔 | **493** |

## 附錄 F：伺服器 `docs/maintenance/` 86 檔與 `server-source-gap.md` → 歸屬
路徑都在 lito-launcher-server（`2066774`）的 `lito伺服器/LiTo381-Server/docs/maintenance/`。分 4 組逐檔讀完：1、32、37 份 md／txt，加 16 份 JSON，合計 86，和目錄的檔案數相同。下表只列對客戶端有影響的項目；鎖、FIFO、completion 佇列、生命週期、外控設定鍵、目錄整理等純伺服器內部項目，各組報告都標為「無客戶端影響」。

| 出處 | 事項 | 歸屬 |
|---|---|---|
| `forced-combat-0.6.67.md:5`、`:11-13`；`source-material-control-ingress.zh-TW.md:5`、`:10`；`combat-continuation-current.zh-TW.md:21`、`:52`、`:76-77`；`combat-integration-lifecycle.zh-TW.md:282-302`、`:757-763`；`source-completion-work-current.zh-TW.md:41` | Ctrl 強制目標 LFC1 擴充與目標選取政策；Rust 客戶端沒有實作 | **T061（新）** |
| `buff-schema-and-character-pages-20261018.zh-TW.md:15-39`；`combat-closeout-current.zh-TW.md:74`；`release-verification-20261018.zh-TW.md:16` | 8 角色分頁、選服、血盟名稱 16 bytes | **T062（新）** |
| `combat-continuation-current.zh-TW.md:64-65` | 寵物 69 項控制、召喚物期限與解放 | **T063（新）** |
| `movement-lock-0.6.66.md:5`、`:11` | C_BLINK 與 S_PARALYSE 模式 17 解鎖 | T044 |
| `native-thunder-teleport-policy-progress.zh-TW.md:5`；`source-mass-teleport-exit.zh-TW.md:15`；`combat-integration-lifecycle.zh-TW.md:27`、`:66`、`:86` | 沒有 S_PORTAL 的解鎖；portal 的 showId | T023、T011-B |
| `empty-melee-0.6.49.json` L34–42 | 空揮封包與拒絕條件 | T029 |
| `room-rules-0.6.61.md:21-25`；`bot-room-count-0.6.64.md:5-12`；`room-directory-0.6.54.json` L21–29 | 房間規則、ArenaBot 人數、跨來源同頻道 | T054 |
| `dragon-presentation-0.6.48.json` L37–44；`dragon-combat-0.6.47.json` L38–42；`illusion-presentation-0.6.50.json` L232–236；`combat-continuation-current.zh-TW.md:255-282` | 狀態圖示 3080、`6D h c(181)`、幻術 lease、ICONS1／ICON_I2H／OwnCharStatus | T046、T047 |
| `native-light-action-current.zh-TW.md:25`；`native-self-cast-current.zh-TW.md:5`；`native-cast-timeline-current.zh-TW.md:7`、`:15`；`native-invisibility-cast-policy-progress.zh-TW.md:6` | 光術封包順序、1480ms 動畫窗、SprTable 截斷、隱身重施 | T046、T047、T025 |
| `combat-coverage-current.json`（808 列）；`combat-coverage-current.md:202-253`；`combat-parity-0.6.68.json` L9 | 技能矩陣（180＋1 列對應本目錄 181）、待查處理器、別名、208 秒數、卡瑞／莎爾 | T026 |
| `native-de-inventory-root-current.zh-TW.md:7`、`:13`；`native-de-item-binding-current.zh-TW.md:7`；`gm-travel-0.6.68.md:19`、`:32`；`native-triple-arrow-policy-progress.zh-TW.md:7`；`native-item-weapon-material-progress.zh-TW.md:5`；`data-sources-0.6.60.json` L402–439、L1000–1029 | unsigned 物件 ID、5-byte remove、Storm_Walk、三重矢扣箭、依裝備武器決定目標、擂台商店、S_SHOW_MAP | T045 |
| `native-de-terrain-current.zh-TW.md:11`；`data-sources-0.6.60.json` L972–999；`resource-database-0.6.63.md:9-16` | 經典地圖差異、504 雙域、LTzone、map_page | T057 |
| `native-de-chat-current.zh-TW.md:9-11` | NPC 對話 opcode 45 | T060 |
| `combat-integration-lifecycle.zh-TW.md:221`、`:252`、`:742`；`docs/server-source-gap.md:37-1454`；`source-administration-validation.json` L21–23 | 換版重核清單、Maven 產物差異、來源伺服器清單 | T025 |
| `dragon-skills-0.6.46.json` L45 | 斷線延遲結算記一敗 | T038 |
| `combat-integration-lifecycle.zh-TW.md:295`；`source-completion-work-current.zh-TW.md:5`；`combat-continuation-current.zh-TW.md:14` | LiTo 只有競技場（BATTLE），WORLD 只保留後端回歸 | 支持 WORLD 排除 |
| `gm-travel-0.6.68.*`、`gm-polyme-0.6.71.md`、`source-administration-validation.json` | GM 工具 | 排除（G01–G12 沒有這一項） |
| `combat-parity-0.6.68.json` L10–13；`native-skill-consumption-current.zh-TW.md:5`；`native-de-item-publication-current.zh-TW.md:7` | 治癒 +100、AC 減免、MP 公式、數量落後 | 排除（伺服器缺陷或權威值，客戶端只顯示） |
| `layout-cleanup-20261012.json`（1474 筆，src 0 筆）、`layout-validation-20261012.json`、`combat-report-simplification-20261019.json` | 目錄整理、純文件精簡 | 無客戶端影響 |
