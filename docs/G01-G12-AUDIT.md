# G01–G12 稽核（T037）

- 本檔是 `docs/G01-G12.md`（46 項：12 主項、34 子項）的逐項核對結果，對應 G12 的 work 第 2 項與 M3 完成定義。
- 稽核日期：2026-11-12。執行者：claude-local（Issue #325，T037 第 2 張）。第 1 張（證據保存與冗餘產物清單）見 `docs/reports/issue-324.md`。
- 本次是**靜態稽核**：讀文件、原始碼、收據與 JAR，重算雜湊，查 GitHub Issue 與 CI 狀態。沒有執行任何編譯出來的程式，沒有開遊戲，沒有連伺服器或 DB。所以本檔不會把任何一項新判為「驗收成立」；它記的是每一項離成立還差什麼、證據在哪、怎麼重跑。

## 1. 結論

1. **46 項裡，當版來源沒有任何一項驗收成立。** G12 的 acceptance（「G01 至 G11 驗收成立、沒有未處理必備缺口」）因此不成立，M3 還不能宣告完成。
2. 46 項的分布（定義見 §3）：

   | 結論 | 項數 | 項目 |
   |---|---:|---|
   | 待實跑：程式已在 main，等當版實跑或實機 | 30 | G02、G04、G04.1–G04.7、G05.3、G06、G06.1、G06.3、G06.3a–G06.3e、G07.2、G07.5、G08、G08.1、G08.2、G08.2a–G08.2e、G08.3、G08.4 |
   | 未做完：實作、盤點或測試還有 Issue 沒關 | 14 | G01、G03、G05、G05.1、G05.2、G06.2、G07、G07.1、G07.3、G07.4、G09、G10、G11、G12 |
   | 待伺服器 | 1 | G05.4 |
   | 排除 | 1 | G06.4 |
   | 驗收成立 | 0 | — |
3. **歷史收據都還在，但都綁在舊來源與舊 JAR。** `docs/G01-G12.md` 引用的 139 個本專案路徑全部存在；68 組附 SHA256 的，67 組重算相符、1 組是目錄無法比對、0 組不符（§7）。這些收據對應的來源是 `4C7DE771`、`89D4B630`、`A9E802DB`、`3E58268A` 等舊 manifest，JAR 多為 `C02AAAE8`，不是現行的 main 與 E149。依 G03 的 acceptance（「不得沿用舊版計數宣稱通過」），它們只能當設計與比對參考。
4. **map563：判定在範圍內**（§4）。它是房間模式 3（攻城）的地圖，從建房介面就能選到，不是 WORLD 地圖。兌現 U-M3-6。
5. **ARGO：判定在範圍內**（§5）。E149 有完整的 `/argo` 觀戰目錄（虛擬頻道 99），重建版只解析回覆、送不出請求。兌現 U-M3-8。
6. 另外發現一個歸類問題：TASKS.md 把「攻城 P 鍵」列為 WORLD 專屬而排除，但 HISTORY 記載它是攻城房間（map563／573）的 101／102 入口。請主控重新確認（§4.3）。

## 2. 稽核基準與來源漂移

### 2.1 本次稽核對應的版本

| 項目 | 值 | 怎麼確認的 |
|---|---|---|
| 客戶端 commit | `d7a0ecb`（`origin/main`，2026-11-12） | `git rev-parse origin/main` |
| 正式 JAR（E149） | SHA256 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`，20,054,975 bytes | 重算 `lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar` |
| JAR 固定副本 | 同上 SHA256 | 重算原始專案目錄 `LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar` |
| G 定義來源 | `工作佇列.json` SHA256 `0EC8ACBF…9830AC` | #324 已重算；本次沒有再算 |
| TW 資源包 | `0-客戶端/launcher/TW13081901.pak` SHA256 `92B512A3…E59CA7` | 重算；與 #167 報告記的相同 |
| CI | `cargo check --locked --all-targets` 與 `cargo test --locked --lib --no-fail-fast`；`b3d2a27` 那次（`d7a0ecb` 只比它多一個報告索引 commit）：2326 通過、0 失敗、597 ignored | `gh run view 43212926382 --log` 的 `test result:` 原文 |
| 伺服器 repo | `origin/main` `edfb5c8`；最新的發布紀錄仍是 `release-0.6.71.json`（＝E149） | `git ls-tree`（唯讀） |

### 2.2 漂移

| 對象 | 舊基準 | 現況 | 影響 |
|---|---|---|---|
| 客戶端來源 | `docs/G01-G12.md` 各項 state 對應來源 `3E58268A…`（1048 個宣告輸入）；repo 第一個 commit `92bbda0`（2026-11-04）是匯入時的樣子 | `92bbda0..d7a0ecb`：`src/` 187 檔變更（+29,452／−1,152 行），檔數 777 → 853；`tests/` 53 檔變更（+8,910／−526 行），整合測試目標 141 個 | 46 項的來源 state 全部是舊來源的結論。M2 併入了技能、移動、房間、租裝、傳送、物品、快捷列等改動，舊收據都不能移記 |
| 正式 JAR | 歷史收據多用 `C02AAAE8`，另有 `BBC59308`、`9719F45D`、`6928A147`、`76EBE06D`、`75DA69B0` | E149（0.6.71）；本次重算沒有變 | 舊收據的伺服器行為不等於 E149。E149 之後伺服器沒有發新版，所以 T025 的換版重核清單目前不必執行 |
| 伺服器原始碼 | PLAN 引用 `2066774` | `origin/main` 已到 `edfb5c8`，多 587 個 commit | `src/main` 已含 E149 之後的候選。本檔引用伺服器原始碼的地方都標明「當版原始碼」；有對 E149 JAR 靜態核對過的另外寫明 |
| 原始專案目錄 | — | 停在 `92bbda0`，落後 `d7a0ecb` 517 個 commit；`target/current` 裡的 `formal-server.jar` 是 `C02AAAE8`，不是 E149 | 在原始目錄直接建置或跑舊 runner，會得到舊來源、舊 JAR 的結果 |
| 舊 runner | — | `維護工具/run-native-arena-portal.ps1` 把專案根寫死成 `D:\lineage開發\3-lito開發\lito重建版`，允許的 JAR 清單沒有 E149 | 攻城（map563／573）harness 目前不能對 E149 執行（§4） |
| 可執行檔 | 來源 `3E58268A` 的 Debug `AB99BED1`／Release `4340DA5F` | 當版 main 沒有發布過 EXE；`交付/` 與 `dist/` 裡的是舊候選（PLAN 記為已過期） | G01 的「已發布 EXE 雜湊」與 G10 的可攜包要等 T031 |

本機規定（2026-11-09）之後，claude-local 不再執行編譯出來的程式。本次讀到的報告裡，11-09 以後由 claude-local 併入的改動只有 CI 的 `cargo test --lib` 結果；整合測試、`#[ignore]` 的原資源測試、owned harness 都寫在各報告的「待驗證」。有本機實跑紀錄的是 #118（G06 五案）、#174／#175（雙真實玩家）、#101（測試套件 80 步），都在 2026-11-06，之後 main 又變了很多。

## 3. 46 項逐項核對

### 3.1 欄位說明

- **來源 state**：`docs/G01-G12.md` 記的原值（舊來源的結論），過長的取開頭。
- **當版的工作**：併入 main 的 Issue 與報告（`docs/reports/issue-<號>.md`）。「未關」表示 Issue 還開著。Issue 狀態查於 2026-11-12。
- **結論**：
  - **待實跑**：程式已在 main，CI 的 lib 測試通過，但整合測試、owned harness 或實機還沒在當版跑過。
  - **未做完**：還有 Issue 沒關（實作、盤點或測試還沒開始），或有已知缺口沒有對應的實作。
  - **待伺服器**：要伺服器先改。
  - **排除**：依 #92 的範圍凍結不做。
- **重跑**：對到 §9 的編號。

### 3.2 G01–G03

| 項目 | 來源 state | 當版的工作 | 結論 | 已知限制 | 重跑 |
|---|---|---|---|---|---|
| G01 重建現況與版本基準 | catalog_normal_two_profile_related_baseline_qualified_limited | T025 #139（版本綁定 E149、換版重核清單）；#92（每項有歸屬與驗收條件）；本檔 | **未做完** | acceptance 的「四項雜湊一致」：JAR 來源與固定副本本次重算相符，修訂前佇列（`工作佇列.json`）已保存；但當版 main 沒有已發布的 EXE，缺這一半。等 T031（#303 未關）產出後補 | R1、R7 |
| G02 兩名實際玩家的雙場戰鬥與結算 | current_source_required_scope_incomplete…；`current_main_two_round_verified=false` | T043 #174、#175（owned loopback E149，兩條正式 LOGIN／GAME session，實跑到死亡、結算、重入第二場；2026-11-06） | **待實跑** | #175 之後 main 又併入大量戰鬥與技能改動，沒有重跑。#175 的第二場做到重新靠近與施法就取消連線，沒有打到結算，所以「互換勝敗」沒驗；SQL 持久化、跨程序重連也沒驗。兩個原生 GUI 客戶端對戰要等 T030（#304 未關） | R3（`native_two_player_battle`）、R6 |
| G03 修復回歸探針與驗收新技能／服務整合 | current_full_library_default_both_qualified_broad_not_repeated_GUI_pending | T041 #173（測試套件加技能 66 列、G06 八案）；T036 #306、#307 已併入，#308–#320 未關；T053 #332、#333 未關；T042 #327 未關 | **未做完** | 舊 54 情境的保留／改寫／失效盤點還沒做（T053）。白名單外 1055 個測試（574＋481）還沒放行。#173 的完整套件在當版沒有實跑紀錄。S_CE／S_D4／S_CD 順序重播沒做 | R1、R2 |

### 3.3 G04 多人移動、碰撞及世界生命週期

| 項目 | 來源 state | 當版的工作 | 結論 | 已知限制 | 重跑 |
|---|---|---|---|---|---|
| G04（主項） | current_source_required_scope_incomplete… | T044 #176、#177、#178；T048 #186；T029 #168 | **待實跑** | 三張都只有 lib 測試（CI）。多模式雙端實測、真實網路時序、GUI 按鍵與動畫都沒驗。伺服器成功步伐不回 self echo，客戶端分不出「成功沒回覆」與「靜默拒絕」（#176） | R1、R3 |
| G04.1 雙端八方向、玩家同格及權威位置 | verified_defined_scope | 沒有專屬 Issue；回歸歸 T036 | **待實跑** | 舊來源的結論。當版只有 lib 測試 | R3（`native_motion`、`native_navigation`） |
| G04.2 競技場門雙向開關、動態通行及持續姿態 | verified_defined_scope | T044 #177（動態門占位）；`native_doors` 有 4 案在整合白名單 | **待實跑** | 白名單 4 案由 R2 執行，當版沒有實跑紀錄。開門 O 鍵的實機項是 GUI 清單 W15–W18 | R2、R3（`native_doors`）、R6 |
| G04.3 觀戰範圍與隨從主人權威封包 | verified_packet_state_scope | T054 #189（觀戰、初始鏡頭）；T063 #195（寵物與召喚物） | **待實跑** | 觀戰初始畫面沒有與原版截圖比對（#189）。寵物九種控制的端到端沒驗（#195） | R3（`native_observer`、`native_observer_camera`）、R6 |
| G04.4 加入離開、死亡復活、換圖及同 ID 替換 | in_progress | T044 #178；T048 #186 | **待實跑** | 晚加入者 bootstrap、classic 死亡重生、換圖、Quest／Siege 模式的自然死亡都沒有對 E149 實測（#178、#186） | R3（`native_lifecycle`、`native_combat`） |
| G04.5 移動鎖、速度、追擊與持續輸入中斷 | verified_input_lifecycle_scope | T044 #176（C_BLINK、模式 17 解鎖）；T029 #168（追擊、目標圈） | **待實跑** | 攻擊專用間隔在 GUI 沒啟用；追擊沒有尋路，遇到要繞路的障礙會停（#168）。E149 對重複或未請求的 C_BLINK 的拒絕沒驗（#176） | R3（`native_motion`、`native_combat`）、R6 |
| G04.6 守護塔生命狀態與動態路徑阻擋 | verified_contract_and_route_scope | 沒有專屬 Issue；回歸歸 T036 | **待實跑** | 舊來源的結論 | R3（`native_guardian_attack`、`native_obstacles`） |
| G04.7 其他模式的城門、任務怪物、限時模式物件與進出世界 | pending | T044 #178（其他模式物件）；T046 #183（任務／限時怪物 HPBAR） | **待實跑** | 任務／限時怪物自然死亡、攻城內門實際開啟都沒有對 E149 實測；攻城內門的 GUI 提示沒做畫面驗收（#178）。攻城房間的當版回歸見 §4 | R3、R5 |

### 3.4 G05 物品、裝備、技能及狀態完整性

| 項目 | 來源 state | 當版的工作 | 結論 | 已知限制 | 重跑 |
|---|---|---|---|---|---|
| G05（主項） | static_request38_of_book74_missing36… | T026 #140、#141（`docs/SKILLS-STATUS.md` 技能表與玩法矩陣）；T020 #142、#143；T021 #144；T027 #145–#150；T022 #151–#158 已併入，#159、#160 未關；T024 #161、#162；T028 #163–#166；T045 #179、#180；T046 #181–#183；T063 #195；T052 #330、#331 未關 | **未做完** | T022 還有 2 批技能沒做。矩陣各列在 E149 的實際封包與結果大多沒有回填（#141）。useType 6／29／28／31–36 被伺服器阻擋；擂台商店 UI 與地圖頁沒畫（#179、#180）。裝備與彈藥 UI 沒做（T052） | R1、R2（`-Category Skills`） |
| G05.1 能力值、等級／經驗、AC／抗性及 SP／MR 權威更新 | verified_world_schema0_subset | 沒有專屬 Issue；`docs/GUI-ACCEPTANCE.md` §1.1 寫明現況 | **未做完** | 當版只接受數值 schema 0：`native_status_receipt.rs:29` 要求 `S_STATUS` body 37 bytes。schema 1–4 會被當成協定錯誤，與來源 remaining 寫的限制相同。沒有任務實作 schema 1–4；T034 的處理方式是在清單寫明用 schema 0 啟動伺服器 | R3（`native_attributes`）、R6（P02、N01、N02） |
| G05.2 可達技能、道具與裝備成功／拒絕矩陣 | historical_skill18_scope_and_current_higher_heal_self_scope_verified… | 同 G05 的技能任務；T061 #192、#193（Ctrl 強制目標）；T029 #167（遠程） | **未做完** | acceptance 第 4 條「實際雙玩家權威」：新增技能只有封包 oracle 與 lib 測試。class3 遠程圖形還有 15 種沒接；TW 三種呈現器被資源資格擋住（#167）。GFX 6983 的越界 subframe 沒有來源依據（#184） | R2（`-Category Skills`）、R3 |
| G05.3 實際守護塔攻擊與狀態型目標生命週期 | verified_actual3000hp_water_guardian_honor_render_lifecycle_subset | 沒有專屬 Issue；回歸歸 T036 | **待實跑** | 舊來源的結論。其他狀態物件、其他守護塔模式沒驗 | R3（`native_guardian_attack`、`native_guardian_inventory_audit`） |
| G05.4 78 項商人附加屬性正式來源缺失 | current_C02_detached_reference_gap_confirmed… | T040 #299：已開 lito-launcher-server#371（另三項是 #368、#369、#370） | **待伺服器** | 伺服器主控還沒回覆是否接收。E149 是否有別的路徑補上屬性沒核對。客戶端照權威值顯示的部分歸 T049（#328、#329 未關） | — |

### 3.5 G06 房間管理與跨服務分支

| 項目 | 來源 state | 當版的工作 | 結論 | 已知限制 | 重跑 |
|---|---|---|---|---|---|
| G06（主項） | current_source_API_controller_and_GUI_branches_pending… | T054 #188、#189；T055 #190、#191；T023 #169–#172；#118（G06.3d 五案實跑） | **待實跑** | acceptance「每個正式可達分支可實際進出、拒絕與返回」：攻城模式（§4）與 ARGO（§5）在當版都沒有驗；ARGO 連請求都送不出 | R2（`-Category G06`）、R5 |
| G06.1 建房、設定、密碼、踢人與房主轉移 | verified_controller_live_subset | T054 #188 | **待實跑** | #188 是離線回覆測試。雙來源建房、密碼拒絕、房主斷線轉移、踢人重入拒絕沒有對 E149 實測 | R3（`native_room`、`native_room_admin`、`native_room_departures`）、R6（R07–R16） |
| G06.2 模式切換、隊伍、準備與觀戰分支 | original_observer_view_live_updates_and_returns_candidate_verified_formal_integration_pending | T054 #189 | **未做完** | 觀戰者單獨主動離場做不到：伺服器把它當成整場結算，已開 lito-launcher-server#366，客戶端目前拒絕這個操作。最後成員離房的缺口是 lito-launcher-server#370。ARGO 入口沒做（§5） | R3（`native_room_mode`、`native_observer`）、R6（R17–R20、O01–O13） |
| G06.3 租裝來源、村莊、商店與服務交接 | in_progress | T055 #190、#191 | **待實跑** | 正式 harness（`native_peer_rental_probe`、`native_map_portal`、旅館探針）沒有在當版跑。map500 的 24 處入口只驗過 1 組雙向。配裝名稱只顯示「配裝 n」（#191） | R3（`native_rental_routes`、`native_rental_entry_guard`、`native_map_portal`） |
| G06.3a 服務清單、配裝拒絕／成功與原資源入村 | verified_entry_subset | T055 #191（C_CA、S_FB 目錄） | **待實跑** | 客戶端沒有補送原生的 C_CE，是否要對齊要看正式 capture（#191） | R3（`native_services`、`native_rental_entry_guard`） |
| G06.3b NPC 對話、收到的商品列、光之寶石不足／租用／歸還 | all78_merchant_controller_widgets_and_defined_expiry_subsets_verified | 沒有專屬 Issue；回歸歸 T036，實機歸 T030 | **待實跑** | 舊來源的結論。全部武器種類到期、正式附加屬性（G05.4）、普通角色重量可達性沒驗 | R3（`native_merchant`、`native_npc_document`、`native_weapon_expiry`）、R6（M01–M15） |
| G06.3c 正常返回、重新入村、服務物品與互動清單生命週期 | candidate_defined_return_scope_verified_formal_integration_pending | T055 #190；T040 #299（lito-launcher-server#368） | **待實跑** | 伺服器 `quitFight` 的缺口在當版 `src/main` 還在（#299）。#190 是合成測試，不是正式認證 session。伺服器修好後的重驗是 U-M3-1 | R3（`native_rental_routes`） |
| G06.3d 旅館、傳送與跨端點服務可達性 | current_source_controller_qualified_same_endpoint_Presentation_audio_other_services_GUI_pending | #118（五案在當時的 main 實跑通過）；T023 #169–#172（補到八案） | **待實跑** | 後補的三案（cancel-repeat、refused-cancel-repeat、cancel-expiry-source）與 `--features teleport-wire-test` 的入口沒有實跑（#170、#172）。旅館自然到期、手動退房、Presentation 與音效實際播放沒驗；`complete=false` 不變（#171）。同端點（lito-launcher-server#369）是 U-M3-1 | R2（`-Category G06`）、R4 |
| G06.3e 原始對話資源與完整玩家 GUI 操作 | original_documents_room_all_merchants_citizen_hotel_widgets_verified_physical_full_flow_pending | T034 #300（清單）；T030 #304 未關 | **待實跑** | 清單 237 項全部未執行 | R6 |
| G06.4 普通世界 C05 入口與後續場景交接可達性 | native_directory_boundary_verified_other_routes_pending | — | **排除** | 伺服器產品範圍只有競技場，沒有 WORLD 地圖（#92 證據 E6；TASKS「排除或延後」） | — |

### 3.6 G07 既有資源與完整世界呈現

| 項目 | 來源 state | 當版的工作 | 結論 | 已知限制 | 重跑 |
|---|---|---|---|---|---|
| G07（主項） | current_source_required_scope_incomplete… | T047 #184、#185；T056 #334、#335、T057 #336、#337、T058 #338、#339、T049 #328、#329 全部未關 | **未做完** | acceptance「可重跑矩陣綁定資源 SHA 與實際呈現證據」：矩陣還沒有（T057） | R1、R3 |
| G07.1 資源命名空間、來源雜湊與格式支援矩陣 | source_and_qualified_scope_matrix_audited_exhaustive_member_coverage_pending | T057 #336、#337 未關 | **未做完** | 還沒開始 | — |
| G07.2 全部玩法可見角色、NPC、狀態型物件與原始動作 | pending | T047 #184、#185；T024 #161、#162 | **待實跑** | GFX 6983 八方向完整播放沒完成，等主控裁定（#184）。proc_spr 需要的 GFX 清單在伺服器 DB，沒有取得。S_POLYMORPH 已接上，變身與 7200 秒還原的畫面沒驗（#185） | R3（`native_animation`、`native_animation_resources`、`native_actor_states`） |
| G07.3 地形、跨區塊、遮擋、屋頂、附件、名稱與特效排序 | partially_verified | T058 #338、#339 未關；T056 #334、#335 未關 | **未做完** | 還沒開始。屋頂與遮擋的實機項（W13、W14）在 GUI 清單裡標為前置 T058 | R3（`native_world_roofs`、`native_world_layers`、`native_world_labels`） |
| G07.4 失敗資源診斷與玩家操作完整性 | pending | T057 #336、#337 未關 | **未做完** | 還沒開始 | — |
| G07.5 正式 S_E8 小地圖標記、世界退役與原生視窗呈現 | verified_fixed_integration_subset_full_observer_gui_pending | T034 #300（O08、O09、O11） | **待實跑** | 觀戰 GUI 的小地圖沒有實機驗過 | R3（`native_minimap`）、R6 |

### 3.7 G08 唯一入口的完整玩家操作

G08 的 acceptance 是「以實際 GUI 操作及畫面證據驗收，headless 探針不能代替」。所以 G08 與全部子項只能由 T030（#304，`needs:user`）成立。T034（#300）已把清單寫成 `docs/GUI-ACCEPTANCE.md`，237 項全部未執行。

| 項目 | 來源 state | 當版的工作 | 結論 | 已知限制 | 重跑（GUI 清單項目） |
|---|---|---|---|---|---|
| G08（主項） | current_normal_GUI_case_terminal_scope_qualification_pending | T034 #300；T035 #301 已併入，#302 未關；T046 #181–#183；T062 #194；T061 #192、#193；T030 #304 未關 | **待實跑** | `final_acceptance_depends_on` 是 G04–G07，那四組都還沒成立。#301 改了視窗還原與輸入前置檢查，截圖逾時是否因此消失沒有實機驗過。聊天（T060 #342）還沒做 | R6 全部 |
| G08.1 房間建立／修改、踢人、觀戰與模式操作控件 | widget_events_and_formal_states_verified_physical_input_pending | T054 #188、#189 | **待實跑** | 同 G06.1、G06.2 | R6：R07–R09、R14–R20、O01–O03 |
| G08.2 租裝服務入口、NPC 對話、交易數量與權威貨幣提示 | shop_hotel_citizen_widget_subsets_verified_other_services_pending | T055 #190、#191；T045 #180（擂台商店協定） | **待實跑** | 擂台商店 UI 沒做（#180） | R6：E01–E13、M01–M15 |
| G08.2a 四枚徽章與市民資格頁 | protocol_and_native_widget_walk_verified_physical_input_pending | — | **待實跑** | 舊來源的結論 | R6：C01–C06 |
| G08.2b 瑪雅／菲佛／克里商人與容量／重量／到期 | 78_rows_widgets_…_weapon_bonus_physical_pending | T040 #299（weapon bonus 歸伺服器） | **待實跑** | 附加屬性等 lito-launcher-server#371 | R6：M01–M03、M08、M09、M11–M13 |
| G08.2c 旅館租房、鑰匙、退房、進出世界 | three_map_natural_expiry_and_hotel_widgets_verified_self_exit_physical_input_pending | T023 #171 | **待實跑** | 房客自行退房沒解決（來源 `guest_self_exit_unresolved`）；3600／7200 秒自然租期要長時間實測 | R6：H01–H13 |
| G08.2d 四位傳送 NPC 與跨服務交接 | formal_wire_vectors_and_original_page_contract_verified_client_transfer_pending | T023 #171（TeleportMe 排版）、#172（T 鍵） | **待實跑** | 新排版與 T 鍵都沒有開視窗看過（#171、#172） | R6：T01–T07 |
| G08.2e 鍵盤實際走訪與完整 NPC GUI 流程 | hotel_and_citizen_native_key_message_walk_verified_physical_input_pending | — | **待實跑** | 舊來源只在 192 DPI 私有桌面用控件訊息驗過，不是實體鍵鼠 | R6：K01–K07 |
| G08.3 屬性／狀態／系統訊息與小地圖玩家資訊 | hp_mp_quantity_status_layout_verified_full_attributes_multidpi_pending | T046 #181–#183；T049 #328、#329 未關 | **待實跑** | 完整屬性面板沒有；S0F／S10 系統訊息 GUI 歸 T049，還沒做。HUD 的 HP／MP 已改成等動畫開始才換值（#184） | R6：W01–W12 |
| G08.4 從唯一入口實際操作完整流程與 IME／DPI／焦點／取消 | loading_cancel_same_window_reconnect_widget_subset_verified_ime_dpi_physical_input_pending | T035 #301；T062 #194 | **待實跑** | IME、多 DPI、實體輸入從來沒有驗過（歷史收據都是 192 DPI 私有桌面） | R6：L01–L17、I01–I11、D01–D08、F01–F04 |

### 3.8 G09–G12

| 項目 | 來源 state | 當版的工作 | 結論 | 已知限制 | 重跑 |
|---|---|---|---|---|---|
| G09 原始音效與裝置生命週期 | current_source_required_scope_incomplete… | T059 #340、#341 未關；T034 #300（A01–A12） | **未做完** | 無裝置、拔插、切換場景與退出回收還沒做（T059）。acceptance 要「實際裝置證據」，要等 T030 | R3（`native_audio_device`）、R6（A01–A12） |
| G10 Release 全流程與可攜交付 | current_source_required_scope_incomplete…；`current_clean_host_verified=false` | T031 #303、#305 未關；T042 #327 未關 | **未做完** | 當版沒有可攜包。`交付/Windows-x64-候選` 是 2026-10 的舊候選，依賴 VS 預覽 runtime | R2（`-Full`，T042 完成後） |
| G11 效能、長時間及故障恢復 | current_source_required_scope_incomplete… | T050 #187（subtype33 完整性挑戰）；T038 #321–#323 未關 | **未做完** | 驗收預算 `驗證紀錄/current/native-performance/acceptance-budget-v1.json` 還在（§7），標準沒有被改過。60 分鐘／20 場循環與斷線注入都沒做。#187 的 C_ADDR 回覆是相容設定檔，伺服器是否接受只能看伺服器日誌，沒驗 | 等 T038 |
| G12 整理與整體完成稽核 | pending_full_G01_G12_current_source_acceptance | T037 #324（保存清單、冗餘清單；沒有刪除）、#325（本檔）；T032 #326（架構文件） | **未做完** | 見 §8 | R7 |

## 4. map563 是否在範圍內（U-M3-6）

**判定：在範圍內。** 歸 G06（房間模式分支）與 G04.7（其他模式的城門）；不是 WORLD 項目。

### 4.1 依據

已驗證（靜態）：

1. map563 是房間模式 3（攻城）的三張地圖之一。重建版 `LiTo381-Modern/src/modern_lito/native_room.rs:109–120` 的 `mode_maps(3)` 是 `[573, 563, 547]`；房間介面的模式切換（`MODE_PREV`／`MODE_NEXT`）與地圖選單（`native_login_window.rs:2126` 取 `mode_maps(room.mode)`，`native_login.rs:1561` 送 `map_selection`）已經接上，玩家從房間介面就能選到。
2. E149 JAR 的 `BattleRules$Mode.class`、`ArenaMapCatalog.class`、`ArenaPortalCatalog.class`、`NativeLitoMapTableCatalog.class` 都含 547、563、573 三個常數（對 class 位元組找 `sipush`，各出現一次）。伺服器把它當競技場地圖，不是 Classic WORLD。
3. G06 的 acceptance 原文：「每個正式可達分支可實際進出、拒絕與返回」「不得以 Tournament 通過代替租裝或其他模式」。G04.7 的 note 原文：「正式 move 契約另有 siege gates／quest monsters／time-attack actors；門與守護塔通過不能代替這些分支」。
4. `docs/G01-G12.md` 的 G06 自己就帶著 `current_Debug_map563_scope` 與 `current_qualified_siege_scope` 兩個欄位，原佇列是把 map563 三十人回歸算在 G06 裡的。

### 4.2 現況

- 歷史收據還在：`驗證紀錄/current/native-startup-siege563/` 有 6 輪。G06 引用的兩份 `qualified.json` 都存在，第一份的 SHA256 `DB8C9892…08F7BB` 重算相符。第二份（`20261016-221755-210184`）記的是兩個 profile 各 60 條正式 session、Debug 255／Release 257 次移動、tag102 入口的攻守權限。
- 但那兩份的來源是 `A9E802DB`、`4C7DE771`（1024 檔），JAR 是 `C02AAAE8`。**對當版 main 與 E149 沒有任何攻城房間的實跑結果。**
- 當版 main 有對應的程式與測試：`tests/native_arena_portal.rs`（`actual_siege_portal_checks_both_teams_and_retires_scene`、`actual_siege_gui_supports_twenty_eight_players`，都是 `#[ignore]`）、`tests/native_terrain_residence.rs`（`original_siege563_residence_reloads_at_sector_boundary_and_retires_old_input`）、`native_login_window.rs:1027` 的 `enter_nearby_portal`。
- runner 不能直接用：`維護工具/run-native-arena-portal.ps1` 把專案根寫死、JAR 白名單沒有 E149、`-MapId` 只收 573 與 563。

### 4.3 建議的一張（U-M3-6 兌現，估 1 張，本機＋Codex）

1. 改 `run-native-arena-portal.ps1`：路徑改成相對 repo 根，JAR 白名單加 E149，比照 `run-teleport-failure.ps1` 的 owned 程序守門。
2. 對 E149 跑 map563 的 15 對 15 攻城：進場、tag101／102 入口的攻守權限（拒絕與通過）、跨 sector 地形駐留、內門與守護塔、結算與返回。Debug、Release 各一輪。
3. 547 與 573 至少各做一次進出；HISTORY L1199 記著 Release 573 的嚴格地形驗證沒過，要一併重驗。
4. 加進 `run-local-test-suite.ps1` 的分類，之後 `-Full` 才會涵蓋。

**要主控重新確認的歸類**：TASKS.md 第 432、439 行把「攻城 P 鍵」列為 WORLD 專屬而排除，出處寫 HISTORY L1175–L1199。HISTORY L1199 的原文是「兩 profile 的 30 人原 15 對 15 攻城 101／102 入口…map563 兩端實際跨 sector 通過」，講的是攻城房間裡的入口，不是 WORLD 地圖。如果 map563 在範圍內，這個入口也跟著在範圍內；程式已經在 main（P 鍵走 `enter_nearby_portal`），缺的只是當版重驗，可以併在上面那一張。

## 5. ARGO 是否在範圍內（U-M3-8）

**判定：在範圍內。** 歸 G06.2（觀戰分支）；GUI 驗收項要補進 `docs/GUI-ACCEPTANCE.md`。

### 5.1 ARGO 是什麼

`/argo` 是大廳的觀戰目錄：玩家在普通頻道大廳輸入後，進入虛擬頻道 99，看到的是**已經開戰、而且允許觀戰**的房間；點房間列就以觀戰者身分加入，不問密碼。普通頻道的房間清單進不了已開戰的房間，所以這是「中途觀戰一場進行中的戰鬥」的唯一入口。

### 5.2 依據

已驗證（E149 JAR 靜態；對 class 位元組找字串）：

1. `ClientSession.class` 有 `ARGO_CHANNEL_ID`、`argoChannelSnapshot`、`argoRoomSnapshot(s)`、`detachArgoRoomMembership`，以及日誌字串 `S_LITO_ENTER_CHANNEL_RESULT accepted /argo channel=99`、`rejected argo-without-source-channel`、`argo-channel-full`、`S_LITO_SET_OBSERVER_RESULT enabled=true trigger=/argo`、`entered /argo active-battle browser from channel=`。
2. `GameHub.class` 有 `argoChannelSnapshot`、`argoRoomSnapshots` 與字串 `Argo`；`Protocol$LitoChannelSelection.class`、`Protocol$LitoChannelTarget.class` 有列舉值 `ARGO`。
3. `docs/PROTOCOL-CONTRACT.md:189`：`C_LITO_ENTER_CHANNEL 0xCA` 的 selection 值 99 是 ARGO。

已驗證（重建版當版原始碼）：

4. 收得到、送不出：`native_lobby.rs:52–54` 的 `active(id)` 接受 99，所以 `S_LITO_ENTER_CHANNEL_RESULT` 帶 99 可以解析；但 `native_lobby.rs:176–179` 的 `enter(id)` 要求 `id <= 3`，`transport.rs:1203` 也擋掉大於 3 的值。沒有任何 UI 入口。與 `docs/NATIVE-LOBBY.zh-TW.md:44`「ARGO 99 封包可解析，但未開放 UI 入口」一致。

推測（讀的是伺服器當版原始碼 `origin/main` `edfb5c8`，方法名與字串和 E149 相同，但沒有反編譯 E149 逐行比對）：

5. 流程（`ClientSession.java:8814–8856`、`:9201–9289`；`GameHub.java:4815–4837`）：
   - 只能從普通頻道大廳發起。原版客戶端先送 C_CB，再送 C_CA 99，並在本地記住原來的 0–3 頻道，供切回時使用。沒有先進過普通頻道就送 99，伺服器回「頻道未開放」。
   - 成功時回 `S_LITO_ENTER_CHANNEL_RESULT`（成功、99）與頻道資訊（名稱 `Argo`，人數是全部頻道合計）。
   - 房間清單只含 `battleStarted && allowsObservers` 的房間，跨全部頻道。
   - 加入時不檢查密碼，由房間的「允許觀戰」旗標把關；伺服器在 `S_LITO_JOIN_RESULT` 之前先送 `S_LITO_SET_OBSERVER_RESULT enabled=true`。
   - 拒絕情況：房間不允許觀戰、房間滿、實際頻道滿（`argo-channel-full`）、戰鬥已結束。

### 5.3 為什麼算在範圍內

- G06 的 work 第 1 項原文是「核實建房、密碼房、設定、踢人、**觀戰**、隊伍及可達模式的正式契約」，acceptance 是「每個正式可達分支可實際進出、拒絕與返回」。ARGO 是正式核心有、原版客戶端送得出來的分支。
- 它不是 WORLD、不是 GM 工具，也不依賴伺服器修正；E149 已經支援。
- M3 的完成定義流程是「登入→大廳→進房→戰鬥→退出」。不做 ARGO，觀戰者只能在開戰前先進房再切觀戰；戰鬥開始後就沒有辦法進去看。

### 5.4 建議的一張（U-M3-8 兌現，估 1 張，Cursor／claude-local）

1. `native_lobby.rs`、`transport.rs`、`native_login.rs`：加上 C_CB ＋ C_CA 99 的請求，本地保存原頻道；再送一次原頻道就切回。
2. 大廳 UI：ARGO 模式下的房間清單、點列加入（空密碼）、收到 S_E5 後才建立觀戰場景。入口可以是大廳按鈕；`/argo` 聊天指令等 T060（#342）做完再接。
3. 失敗與返回：五種拒絕各有提示；觀戰中的退出目前受 lito-launcher-server#366 限制，要寫明。
4. lib 測試用伺服器封包 oracle；實機項補進 GUI 清單的觀戰一節（O 系列）。
5. 動手前要先做的核對：C_CB 的格式、頻道資訊封包在 99 時的欄位、原版 `sub_452C20` 的送包順序。這三點目前只有伺服器原始碼的註解當依據，要對 E149 位元組碼或原版 PE 核對。

## 6. 附帶紀錄：U-M3-5（畫面等價研究）

本張沒有被要求判定這一項，以下只是核對 46 項原文時看到的事實，供主控參考：

- 46 項的 work 與 acceptance 裡，沒有任何一條要求照明／光罩、水下後處理、背景 64ms scheduler、Bank viewport 或 RGB565 framebuffer 與原版逐像素等價。G07 的 acceptance 是「可重跑矩陣綁定資源 SHA 與實際呈現證據」「無靜默缺圖／錯資源」。
- G07 帶著 `current_map303_asset_scope` 欄位，但 map303 是 WORLD 地圖，已隨 WORLD 排除。
- 日光術的夜間亮度與自然到期在 G05／G07／G08／G09 的 `current_native_light_defined_scope` 裡，屬技能驗收（T028），不是畫面等價研究。

依這三點，G01–G12 本身沒有要求做畫面等價研究。

## 7. 收據與產物對照

### 7.1 `docs/G01-G12.md` 引用的收據是否還在

工具：`維護工具/g12-audit-evidence-check.py receipts`。結果在 `docs/reports/issue-325/evidence-summary.tsv`（每項一列）與 `evidence-detail.tsv`（每個路徑一列）。

| 項目 | 引用路徑數 | 存在 | 附 SHA256 | 重算相符 | 不符 |
|---|---:|---:|---:|---:|---:|
| G05 | 2 | 2 | 1 | 1 | 0 |
| G05.2 | 2 | 2 | 0 | — | — |
| G06（含 `rental_acceptance_steps`） | 113 | 113 | 65 | 64 | 0 |
| G06.2 | 1 | 1 | 0 | — | — |
| G07 | 3 | 3 | 2 | 2 | 0 |
| G07.5 | 1 | 1 | 0 | — | — |
| G08 | 1 | 1 | 0 | — | — |
| G08.2 | 14 | 14 | 0 | — | — |
| G09 | 1 | 1 | 0 | — | — |
| G11 | 1 | 1 | 0 | — | — |
| 其餘 36 項 | 0 | — | — | — | — |
| **合計** | **139** | **139** | **68** | **67** | **0** |

G06 有 1 組附了 SHA256 但路徑是目錄，無法比對。其餘 36 項在來源裡沒有引用任何本專案路徑，它們的 state 沒有可以直接讀回的收據。

### 7.2 收據引用的 EXE 是否還在（回答 #324 留下的問題）

工具：`維護工具/g12-audit-evidence-check.py cited-exe`。結果在 `docs/reports/issue-325/cited-exe.tsv`。

做法：從上面 139 個路徑裡仍存在的 130 份收據 JSON 取出全部 64 位十六進位值（4,667 個不同的值，含來源檔、PNG、JAR 等各種雜湊），再對原始專案目錄 `LiTo381-Modern/target` 下的 4,111 個 `.exe`（25,249,641,616 bytes）逐檔算 SHA256 比對。

| 結果 | 數量 |
|---|---:|
| 雜湊被收據引用、而且檔案還在的 EXE | 152 檔（63 個不同的雜湊），1.33 GiB |
| 其中在 `target/current/` 以外的 | 0 |
| 其中在 cargo 的 `deps/` 內（#324 的 X 類，本來就不動） | 5 |

152 檔分布在 `target/current/` 的 16 個子目錄：`native-teleport-failure` 51、`native-real-peers` 44、`native-teleport-handoff` 13、`qualified-artifacts` 13、`x86_64-pc-windows-msvc` 5，其餘 11 個目錄各 1–4 檔。引用它們的是 G06（146 檔）、G05.2、G08.2、G06.2、G05、G07。

對 #324 的意義：

- **A 類（cargo 中間產物，108.51 GiB）不含 EXE**，刪除不影響這些收據。
- **`target/current/` 以外的歷史輪次目錄**（#324 的 `r4xx–r5xx-player-entry-candidate`、`fixed-finite-appearance-host`、`fixed-consumer` 等）裡，沒有任何 EXE 被 `docs/G01-G12.md` 的收據引用。
- **`target/current/` 內 cargo 版面以外的部分（#324 記 27.4 GiB）不能整批當冗餘**：上面 152 檔就在這裡，是舊收據唯一還能對上的執行檔。

限制：只比對了 `.exe`。`.rlib`、`.pdb`、`.dll`（#324 的 B 類 165 GiB）沒有算雜湊；4,667 個被引用的值裡有多少原本是 EXE、現在已經不在，無法從雜湊值本身分辨。`驗證紀錄/` 裡沒有被 `docs/G01-G12.md` 直接引用的收據也沒有納入。

## 8. G12 自己的 acceptance

| acceptance 原文 | 結論 | 說明 |
|---|---|---|
| G01 至 G11 驗收成立、沒有未處理必備缺口 | **不成立** | §1：0 項成立。必備缺口都有歸屬的任務或伺服器 Issue；沒有歸屬的三項見下 |
| 完成聲明與可重跑收據一致；若外部來源變更先重驗受影響項目 | **不成立** | 目前沒有任何完成聲明。外部來源：E149 沒變，伺服器沒有發新版；客戶端來源大幅變更（§2.2），所以舊收據全部要重驗 |

work 第 1 項（保存證據後整理冗餘產物）：#324 完成了保存清單與刪除前清單，**還沒有刪除任何檔**，等使用者同意。work 第 2 項（核對驗收、已知限制、重跑指令與來源漂移）：本檔。

**沒有歸屬的缺口**（本次核對發現，目前沒有任務會處理）：

1. G05.1 數值 schema 1–4：當版只收 schema 0。目前的處理是驗收時用 schema 0 啟動伺服器（GUI 清單 §1.1），沒有任務實作 schema 1–4。這個限制要不要接受，需要主控裁定。
2. 攻城房間（map563／547／573）在當版與 E149 的重驗：§4，等主控依 U-M3-6 開張。
3. ARGO：§5，等主控依 U-M3-8 開張。

G12 要成立，至少要等：T030（#304）、T031（#303、#305）、T036（#308–#320）、T038（#321–#323）、T042（#327）、T053、T056–T060、T022 的最後 2 批，以及上面三項缺口有結論；之後再依本檔逐項更新結論欄。

## 9. 重跑指令

共通：在 `LiTo381-Modern/` 執行；`CARGO_HOME=D:\dev-cache\cargo`、`RUSTUP_HOME=D:\dev-cache\rustup`；客戶端資源用 `LITO_CLIENT_ROOT`（預設 `D:\lineage開發\0-客戶端`）。下面的 `$C` 代表 `--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`。本機規定禁止執行編譯出來的程式，所以 R2–R5 要由獲准的本機測試執行者（Codex）做，R6 由使用者做。

| 編號 | 用途 | 指令 | 通過條件 |
|---|---|---|---|
| R1 | CI 等級：型別檢查與 lib 單元測試 | `cargo check --locked --all-targets`；`cargo test --locked --lib --no-fail-fast`（push 後 GitHub Actions `build` 會自動跑） | `test result: ok`，0 failed。ignored 不算通過 |
| R2 | 本機測試套件 | `pwsh -NoProfile -File 維護工具/run-local-test-suite.ps1`；分類：`-Category Baseline`、`-Category Skills`、`-Category G06`；`-Quick` 只跑 check 與 Debug lib。先把 E149 JAR 放到 `LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar`，或用 `-ReferenceJar` 指定 | 總表每列 `PASS`，最後一行 `Final exit code=0`（#173） |
| R3 | 單一整合測試目標 | `cargo test $C --test <目標> -- --test-threads=1`；`#[ignore]` 的原資源或 owned harness 案例加 `--ignored`，環境變數見 `docs/TESTING.md` | `test result: ok`，而且實際執行數大於 0 |
| R4 | G06.3d 八個 scenario | 由 R2 的 `-Category G06` 呼叫 `維護工具/run-teleport-failure.ps1 -Scenario <timeout\|refused\|wrong-service\|cancel-retry\|cancel-expiry\|cancel-repeat\|refused-cancel-repeat\|cancel-expiry-source>` | 八案全部實跑通過；前置條件缺失算失敗 |
| R5 | 攻城房間（map563／573） | `維護工具/run-native-arena-portal.ps1 -TestBinary … -ReferenceJar … -BuildProfile <Debug\|Release> -MapId <563\|573> -BuildInputManifest …`。**目前不能對 E149 執行**，要先做 §4.3 第 1 步 | — |
| R6 | 原生 GUI 實機驗收 | 依 `docs/GUI-ACCEPTANCE.md` 逐項做（T030） | 每項有截圖或錄影與「通過／失敗／待伺服器」 |
| R7 | 本檔 §7 的收據核對（唯讀） | 在 repo 根：`python -I -X utf8 維護工具/g12-audit-evidence-check.py receipts docs/G01-G12.md <原始專案目錄> <輸出目錄>`；`… cited-exe <原始專案目錄> <輸出目錄>` | `missing=0`、`bad=0` |

Release 版的回歸要等 T042（`-Full` 模式，#327）。

## 10. 之後怎麼更新本檔

- 某一項在當版實跑或實機驗過之後，改 §3 該列的「結論」，並在「當版的工作」加上報告連結與當時的客戶端 commit、JAR SHA。
- 伺服器發新正式版時，先依 `docs/PROTOCOL-CONTRACT.md` §9 的換版重核清單處理，再更新 §2。
- 「驗收成立」只在該項 acceptance 的每一條都有當版收據時才寫；略過或 ignored 的測試不算。
