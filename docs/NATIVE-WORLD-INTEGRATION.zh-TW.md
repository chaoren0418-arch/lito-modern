# 多人世界、競技場門與權威 metadata

## 2026-10-15 原整庫失敗案例的限定修復

整庫unit曾以固定EAC2ABDB、test debug=0實跑2402項，1903通過／26失敗／473忽略。6項舊報告create_new衝突已改獨立測試輸出；19項歷史PetRace補全27個環境鍵與353份自有輸入pins；最後1項確認為已支援Tile22的map506仍被舊測試期待拒絕，現驗真實預覽與關閉退役，並以合成Tile255保留完整場景未知類型拒絕。直接引用原模組的整合目標D72D41C1以968來源完成26／0／0，保留原六圖30張獨立像素比較、74負向與12次過時結果，31原PNG封存；1719既有JSON無變更。map506完整獨立像素oracle、當前2402整庫／完整TCP、主程式新版本GUI、實體輸入與完整G01–G12仍待。單體整庫重編的記憶體失敗有量測日誌，未以此26項替代整庫驗收。 見[26項限定收據](../驗證紀錄/current/native-teleport-wire/targeted-original-tests-v4/qualified.json)。

## 2026-10-15 兩版本退役守衛與十四案傳送恢復

舊 socket 重複 F4 的退役順序已修復。正常 Debug64AAEA66／Release9DC43D91 各完成跨端點及六種失敗恢復，共14案112條正式 ClientSession、220筆權威移動零多送、80張原世界PNG、56次原7311／7312圖像比對及128次舊GUI通知重播，角色接續、自然過期後手動選角與新物品ID、舊場景退役及完整回收均有當版證據。兩版本各6項合成故障測試／37情境／41 TCP，另各22項一般feature關閉的相關Rust通過；合成連線不列正式sessions。965來源與12現行產物保存核對。整庫unit仍有編譯記憶體缺口，正在量測；完整當前TCP、同端點C02、實體輸入／多DPI、音效GUI、乾淨主機可攜與G11尚待，完整G01–G12未完成。 詳[當版收據](../驗證紀錄/current/native-teleport-wire/qualified.json)。

## 2026-10-15 原生攻城入口與房間選圖

2026-10-15：新增房主C_D1選圖／MapPending與原tag101／102來源辨識。修正前主Debug34AE886C／ReleaseBA531442各18相關Rust及各30名實際客戶端、原攻城15對15自然開戰通過map573 tag101守方往返／攻方兩次無回覆，兩案共120 LOGIN／GAME sessions，由capture與被動權威核對。GUI發現選圖ID127／128與租裝撞號，已改136／137並加全部child ID唯一守衛；目前Debug3EC95013另通過564→514→564雙端GUI、跨端點14／11租裝接續與4張原圖像素，共6 sessions。先前Rust數未移記至此新EXE；Release尚未重建UI修正。map563的30人實測在原19個sector全載入時超過64MiB像素預算，102未通過，須補有界原生地形駐留與跨sector重載。收據另明列既有runner漏pin native_room.rs的來源證據缺口，兩runner已修正供下輪關閉；964當前輸入、10份開發變更、主程式與8測試EXE已保留。這不是完整當前回歸、實體輸入、多DPI或G01–G12完成。詳[限定收據](../驗證紀錄/current/native-arena-portals/arena-qualified.json)。


G04 依當前 BBC59308 核心拆為雙向移動／玩家佔格、競技場門、觀戰範圍／隨從、世界生命週期、移動中斷、守護塔及其他模式物件七項。整項仍未完成；schema39 的 AD02BC2A 基準與此開發分開保存。

## 已實測範圍

兩個實際原生客戶端連至共用正式 GameHub，使用記憶體角色，沒有运行中 HP、座標或勝負注入。雙端每一步完成後互相等待，並以正式核心獨立座標／朝向／門狀態追蹤交叉檢查。

- `20261013-081005-435`：競技場門 14 階段；每位玩家靠近、確認關門阻擋、開門、穿越、返回、關門及再次確認阻擋。A 走 8 步、B 走 6 步；各送 2 次開關請求、拒絕 2 次關門移動。
- `20261013-081049-168`：35 階段；兩位玩家分別走八方向並原路返回，接著 A 靠近 B、走入 B 所在格、返回。A 29 步、B 16 步，沒有權威修正。正式核心與雙端確認兩人同站 `32741,32801`；玩家佔格規則不能假定一律阻擋。
- 兩次均直接從實際捕獲封包解碼 S_MASTER 的動態隨從 ID／主人與 S_E2 半開區域，核對兩端狀態。名字不作物件身份；不存在的物件更新不建立幽靈角色。

以上各有 receipt／authority-comparison／server-trace，位於 `驗證紀錄/current/native-real-peers/<時間戳>`。程序、房間及交接資料回收均通過。29 項門相關 Rust 測試與之後 12 項 metadata／門／world 測試分開記錄；不可把重複測試相加成新版總數。

## 實作契約

S22 嚴格讀取 x、y、方向 0／1 與通行值 0／65。只有已知 gfx2083、姿態 28／29 的競技場門會影響通行；屬性與物件 incarnation 綁定，替換／移除／換圖清除，未擁有者的屬性不猜作障礙。開關 C0E 綁定連線／世界／物件／屬性版本，範圍遵循正式核心的 X 差不超過 2、Y 差不超過 1。

開關不預測成功，等待 S20 動作及 S22 通行屬性一致，重複請求暫時拒絕，兩秒未回覆可恢復。開閉動作完成後保留新姿態，避免回到初始關門外觀。視窗已接入選门後 O；實際 GUI 操作仍屬 G08。

S4F 限定非零 unsigned ID、63 bytes 以內的 MS950 名字，可空字串清除，不容許控制字元。S_E2 保存非空半開範圍，換圖清除；收到範圍不等於進入觀戰，也不以觀戰範圍限制普通玩家。觀戰切換與鏡頭操作仍待 G06／G08。

## 失敗證據與限制

`20261013-080036-216` 的第一次門測試被交叉檢查拒絕：探針送出返回移動後誤接受上一張 idle 畫面。已改成同時等待畫面位置與朝向一致；`080417-679`／`080513-878` 通過，加入 metadata 後另以上述最新兩次重跑。

兩張原資源門圖已目視檢查開閉方向不同、動畫後持續顯示；雜湊在 `native-world-integration/door-visual-inspection.json`。這是共用渲染器輸出，尚非完整 GUI、城門、全部資源或所有碰撞模式驗收。復活、更多換圖／替換／離場事件與持續輸入中斷仍待補齊。

```powershell
pwsh -NoProfile -File 維護工具/verify-native-real-peers.ps1 -ActionProfile doors
pwsh -NoProfile -File 維護工具/verify-native-real-peers.ps1 -ActionProfile movement
```

## 自然復活契約與驗證過程

S11 的 target／action／source／gfx 已按正式編碼器接入。復活建立新生命版本，清除舊死亡動作與舊投射物關聯，使舊 Frame、技能目標與 hover 選取失效；HP、HP 比例仍等待各自正式封包，沒有本地回滿。27 組 unsigned 封包向量及 malformed／orphan／生命退休測試通過。

正式規則確認：死鬥無自動復活，榮譽有延遲原地復活且最少兩人；攻城最少 30 人。以下保留首次驗證過程；後續雙次自然復活的通過範圍見末節。其前置功能 C_D1 `D1 02 01` 已實際把普通房從死鬥切換到榮譽 map510；NEXT 的順序由正式規則決定，不以模式編號加一。S_D5 的模式、地圖、摘要與設定同步接入，等待回覆期間禁止準備，房間密碼不進入序列化快照。

首輪 `082521-942` 發現空預建房會被正式核心清除，改用實際房主請求切換；`083558-903` 已正常切換並開戰，但探針誤要求 54 格外的對手在起始畫面內而失敗。探針現在先走近，跨區塊路線改為有界 A*，原始 map510 出生點路線及動態避讓測試通過。失敗證據保留，尚不等同自然復活成功。

`084208-975` 捕獲走近時同 ID S0B 視野更新；探針分開驗證物件替換与 S11 復活。`084812-571` 發現路線穿過守護塔，正式核心拒絕並校正至前一格。已加入守護塔動態阻擋：保留 ID 0x6b000000–0x6bffffff、gfx 2665／2667／2669／2671，姿態 0／32／33／34 阻擋、35 摧毀解除；不以四捨五入 HP 比例推断死亡。正式 Oracle 涵蓋 1 HP 與摧毀邊界；原生與實測正在驗證。

守護塔 Oracle 24 個状态、原資源四種 gfx、1 HP 阻擋／摧毀解除通過；15 項動畫／路線／生命／塔相關 Rust 測試通過。`090453-802` 成功繞塔並自然復活一次後，捕獲客戶端仍留著死亡確認 8 秒 deadline 的錯誤，已在自己的 S11 清除。完整循環重跑會在第二次復活後等待 9 秒再移動，直接檢查此斷線錯誤沒有重現。這輪尚未通過，沒有合併成完整復活驗收。

## 自然復活限定驗收通過

`20261013-091223-886`，EXE `70C79988471EB855299098C7E8C20C3DBDB762DB369574D41C2B141375BE37FE`：A 走 75 步、121 次真實近戰，使 B 自然死亡兩次；各自 C_E0 後 5.033／5.010 秒收到正式 S11，兩端保留同世界 generation 1／GAME epoch 2。新生命退休舊選取與舊死亡 Frame，第二次復活後 9.422 秒再送出 1 步並雙端收斂；兩端零位置修正，四個實際 session 與 room membership 全回收。整體 G04 與完整遊戲仍未完成。

交叉比較初版誤要求本人與遠端 hp_ratio 相同而拒絕結果。实际捕獲確認本人物件使用 255 sentinel、以 S0D 提供 HP，遠端才用 S68 百分比；驗證器改為檢查各自正式欄位，對同一份已完成捕獲重新檢查通過，沒有改寫收到的世界。守護塔原始 SPR／list.spr 雜湊與 72 個姿態取樣保存在 target/current/verification/guardian-assets.json。


## 2026-10-13 G06 房間及服務邊界

2B9B5224 的 12 Rust／4 組、房間管理與雙場近戰通過。房間管理 14 個雙端快照已對上正式核心，證據 驗證紀錄/current/native-world-integration/room-admin-qualified.json。包含密碼拒絕與正確加入、設定變更重置準備、觀戰開關、房主移交、踢人及再次加入拒絕；GUI 仍未驗收。

318A1203 的 14 Rust／4 組與雙端 C05 邊界實測，確認原生清單 PeerFixture(測試乙)／PeerFixture2(測試乙) 不是正式角色清單 TestKnight／TestMage；原封包均回 NO_TRANSITION，未收到世界場景。classic-entry-boundary.json 保存四個實際請求／回覆。不剝除別名猜角色，也不把此結果推論為所有普通世界路徑均不支援。

目前正在接入 S_FB 租裝直連房與 VS 清單、C_CA 配裝選擇、C_D9 → S_F0 → S_F5 → C_CF 的一次性入村鏈。隔離測試才啟用 rental-town，並將端點配置為自有 loopback；尚未把租裝入口、地圖、商店、返回及跨端點交接標記完整。


## 2026-10-13 租裝入口與完整城市容量

EXE 7682F61B／正式 BBC59308 的兩次租裝入口及同檔雙場近戰回歸通過。map500 全18個 S32、351509 筆參照、17653 tile slots 已載入，decoded 40672512 bytes，仍受64MiB上限限制。兩玩家及13 NPC權威相符；執行中取樣的兩端 peak working set 為342458368／341725184 bytes（最後取樣之後的峰值可能未捕獲）。證據為 `驗證紀錄/current/native-world-integration/rental-entry-qualified.json`。

保留四輪失敗：S_DB 點數被當作結算列、載入器及場景各一處小地圖容量限制、探針一端先退出造成另一端等待不結束。均分別修復，不刪失敗紀錄。此為原資源首幀與入口子集；全城遮擋／屋頂、商店、正常返回、GUI及全資源仍未驗收。商店使用原始光之寶石（隔離角色入村20個），與 S_DB 5000 點不同。


## 2026-10-13 商店交易與返回路徑

EXE BBC84502 的雙端NPC Love_Start、17列正式租用清單、寶石不足拒絕、租用及歸還通過：20→20→17→20 顆宝石，新租用物件新增後移除，正式runtime物件ID／数量／到期與余额一致。同EXE近戰雙場回歸通過。S0F系統文字與S10模板訊息分開接收，255-byte MS950 上限明確檢查；GUI 尚未接線。證據 `rental-shop-qualified.json`。

EXE 6CB3426B 實送兩端 C10→S65→CD9，6秒窗口內仍是 BATTLE／RESTART_AWAIT_QUIT_FIGHT，無 F5來源許可，背包沒有清理。靜態 quitFight 的 BATTLE 分支先於 pending restart，對合成租裝房60001進入空GameHub成員結算。此為該正式路徑缺口，其他退出／傳送分支尚未驗；不修改只讀正式核心，也不把斷線退出視為正常返回。新EXE的商店、5個既有來源返回情境與雙場近戰均通過，詳 `rental-return-boundary.json`。後續繼續 G05 角色屬性更新，保留 G06 返回未完成。


## 2026-10-13 入房名冊交錯與當前回歸

09FBA295 整合回歸在遠程前置入房捕獲 S_CE → S_D4 → S_CD。換隊者已是正式房間成員，但接收端尚未取得成員快照，舊控制器因未知姓名中止。失敗記錄保存在 `驗證紀錄/current/native-real-peers/20261013-121511-060`；未將失敗重跑覆蓋。

E78DCFE1 於 RoomAwaitSelf 階段允許有界的早到換隊通知，等待 S_CD 補齊身份；沒有創建虛構成員。後到完整快照為權威，早到通知不能再次覆蓋它。待補名單受房間容量限制，從首次通知計算八秒，重複通知不延期；新連線、接受新房間、離房、返回、取消與失敗清除。初始化以外的未知姓名仍拒絕。

38 項相關 Rust／6 組及雙玩家遠程 `20261013-122549-390` 通過；特定交錯由實收第94／95／97行重播驗證，未宣稱實際重跑再次遇到相同排程。完整候選回歸 `20261013-122719-198` 已通過242項Rust／53組及43單端TCP，其後在火牆情境失敗，未發布；916個開發輸入檔確認在該輪未變更。


## 2026-10-13 火牆探針修正與火箭15

E78DCFE1 的 `20261013-124726-439` 並非火牆未渲染：目標在距離內，卻不在正式八方向投射格線上，正式HP維持100。探針改為相鄰格施法，正式Oracle重播捕獲的未命中及兩組出生點共16個相鄰方向，傷害驗收以過程中的雙端／伺服器HP為準，避免自然回血遮蔽已發生傷害。

BAB6EB4D 的火箭15使用正式C14 selector1/6、MP6、S23 gfx1583。原始Text/list.spr的class4鏈1583→218共五張SPR、四bank折疊八方向，飛行97與命中99音效事件各一次；保留發射時目標座標，物件身份替換及換世界退休。不能把此鏈推廣為其他技能已支援。原始飛行及命中PNG已檢視（native-fire-arrow-2000／2450）。實際裝置播放仍待G09。

31 Rust／6組與五個正式雙端情境通過：wall `130303-635`、fire-arrow `130339-721`、fire-arrow-refused `130413-408`、wall-refused `130442-653`、bolt `130519-192`。正向火箭兩次均有1583/218圖像與97/99事件、權威扣魔及傷害；拒絕保持MP2且沒有施法／飛行。詳 `驗證紀錄/current/native-world-integration/fire-arrow-qualified.json`。245 Rust／61TCP整合回歸正在執行；schema40仍是已發布基準。


## 2026-10-13 schema41 限定整合通過

BAB6EB4D／BBC59308 的245 Rust（54組）、61成功TCP（43單端、16雙玩家、2重登）通過；920個實作／测试／工具輸入雜湊及檔案集合一致。兩次先前失敗保留，入房名冊競態、火牆探針站位修正及火箭15皆納入本輪。C05拒絕及租裝restart未返回另列2個未完成邊界，未計為成功情境。收據：`驗證紀錄/current/candidate-regression/20261013-130631-087/published-services/receipt.json`。接續G08房間及租裝GUI，整體產品與App目標均未宣稱完成。


2026-10-14 revision48：重建版修復取消後立即重新登入的選角拒絕。取消仍停止連線並清除密碼與舊權威，只保留同登入端點／帳號／來源的一次性接續識別；新認證後若無選角回覆與背包資料，8秒有界等待後回到当前角色清單。Debug／Release立即重試與真實33秒自然逾期各雙端，共32sessions；同當前Release目標5秒逾時恢復再8、跨端點map564接續再6，合計46sessions均由capture／被動權威獨立比對。Debug／Release各10Rust，4真實情境預設ignored另明確執行；原Release首次OOM，改單獨重建后通過，未降低建置profile。主Release48634AB2由唯一入口生成；仍非GUI／SQL／完整當前樹或G01–G12驗收，登入器僅唯讀線索。

詳[本次限定收據](../驗證紀錄/current/native-world-integration/teleport-cancel-retry-qualified.json)。


2026-10-15：NPC已选取文字的即时截取曾得到0深色像素；同一诊断Debug等待250ms后有1485像素。撤回所有配色／DC／Debug属性实验，恢复原产品953输入并逐项SHA核对；原来源重新生成Release558A5617，在固定C02租退／市民GUI4 sessions及文字1485像素核对通过。旧545版本16 sessions、156商人交易及旅馆501／502／503保留独立范围，不移记到新EXE；新Debug06D73BD7已生成但未运行GUI。详见[截取时序收据](../驗證紀錄/current/native-world-integration/NPC-caption-capture-qualified.json)。完整HTML／实物输入／多DPI、当前完整回归、音效／可携／稳定性及G01–G12仍未完成。


2026-10-15：新增錯誤服務端點驗證。兩端由正式S_F4轉至真正LOGIN服務，10次原協定握手與6次LOGIN認證皆由capture核對；客戶端無GAME角色回覆而約9秒有界失敗，舊世界／背包／NPC對話清空。新LOGIN／GAME取回原角色與14／11件租裝、無重送C_C9，回map500權威走路。Debug與Release控制器各10、主Release558A5617原生GUI10，共30正式sessions全部回收；僅測試／harness與工具變更，產品Rust未改。GUI停止及恢復PNG已檢視，非實體輸入／完整回歸／全G01–G12驗收。詳[錯誤服務端點收據](../驗證紀錄/current/native-world-integration/teleport-wrong-service-qualified.json)。


2026-10-15：PcmVoice新增唯讀WinMM裝置識別；Debug／Release固定測試程式均在真實預設對應器播放原97／99／102 PCM，自然完成、替換、四聲同播、各20次停止／開啟及播放中Drop通過，暖機後handle均197→197。Python獨立核對原WAV／PCM雜湊與時間；初次錯把WAVE_MAPPER視為實體索引的失敗保留。唯一入口主Debug8EB12785／ReleaseD4CD5143已保存；新Release另重驗C02跨端點GUI6 sessions、角色14／11租裝接續、舊對話退役與4張原圖像素。未宣稱實體可聽見、選定實體喇叭、GUI音效事件、無裝置／拔插或當前完整回歸。G09及完整G01–G12仍未完成。詳[實際音效裝置收據](../驗證紀錄/current/native-world-integration/audio-device-qualified.json)。


2026-10-15：固定Debug8EB12785／ReleaseD4CD5143在C02原生GUI各重播64次舊控件通知：選擇傳送後立即重按、目的端TCP建立中、取消回收後、重新入村後四階段。FOLLOW／BACK／租退／交易／配裝／進村／對話通知不得帶回舊作用域；正式capture各仍僅2次NPC對話、2次B/C傳送、0交易、2次初始選角，無重入選角重送。角色14／11租裝及map500步行、8張原圖像素與16正式sessions完整回收獨立通過。954開發輸入及產品Rust未變；這是舊GUI通知重播，尚非畸形／跨世代wire回覆或實體輸入驗收。完整G01–G12仍未完成。詳[舊控件重播收據](../驗證紀錄/current/native-world-integration/teleport-stale-command-qualified.json)。


2026-10-15：重建版新增 P 鍵使用附近原圖入口。從原LTmap保存tag100來源座標，只送C73來源要求，不預測目的地；角色／地圖／控制世代、距離、重送與操作競合有守衛，等待回覆最多4秒後顯示可重試提示。原map500讀到24處來源，本輪只驗一組入口雙向往返。固定主Debug6335FD56／ReleaseC8F04F94各12相關Rust（1個真實harness預設ignored另明確執行）、雙玩家TCP及P鍵GUI共16正式sessions通過；C73→S5A→C08、舊scene退役、45／43件背包保持及後續權威走路獨立核對。955開發輸入、兩主與6測試EXE已保存；縮短HUD排版並保留一張黑擷取，最終GUI另核對實際地形像素與可見文字。這不代表C05普通WORLD、其餘入口種類／22處未走入口、實體輸入、完整回歸或G01–G12已完成。詳[原圖入口收據](../驗證紀錄/current/native-world-integration/native-map-portal-qualified.json)。


C8F04F94另完成優先跨端點GUI6正式sessions：兩端原傳送頁面、14／11租裝接續、舊對話退役、新共同map564及原7311／7312四張圖像像素獨立核對；新戰場兩端實際地形取樣通過。與原圖入口四案16sessions分開綁定，合計選定22sessions，非完整當前回歸。詳同一原圖入口收據的current_Release_cross_endpoint_regression。


## 2026-10-15：世界樹與普通WORLD限定進場

2026-10-15：重建版已接入技能131世界樹進普通WORLD。以當前角色InputStamp與2秒請求守衛接納固定map4目的，C08後用本人完整重建確認切換，再退役room／channel；S_D0只通知對方，不製造本人離房回覆。新增WorldEntering／WorldReady及原map/4＋BASE Tile的3×3窗口，保留原LiTo LTmap路徑。固定主Debug23C689E8／ReleaseF8FA4B0E各17相關Rust通過、2個live tests預設ignored另明確執行；兩profile WORLD與原map500入口共16正式sessions，背包28／29與45／43件分別保持、步行及WORLD成員清理通過。新Release另通過跨端點GUI6 sessions、14／11租裝接續、舊對話退役與4張原圖像素，合計選定22sessions。957開發輸入、9原S32／Tile來源及兩主／6其他測試EXE已保存核對。普通WORLD兩端原PNG已檢視，但869／$748 NPC因action3缺口未畫出；跨sector邊界、獨入WORLD、WORLD主視窗GUI／其他玩法及完整G01–G12仍未完成。詳[限定收據](../驗證紀錄/current/native-world-integration/world-entry-qualified.json)。


2026-10-15 範圍更正：先前技能131進map4的成功案均明示啟用classic.world.compatibility.enabled=true，屬可選classic相容實驗；C02非memory預設為false，不能作為實際部署已開普通WORLD或exact-native LTmap覆蓋的證明。外部部署設定未讀取、未修改。false分支的276／unlock7、零扣魔／零換圖及明確重試／步行已開始獨立驗證，收據見驗證紀錄/current/native-world-integration/world-entry-scope-correction.json。


## 2026-10-15：拒絕分支、type12與驗收範圍更正

2026-10-15：修復技能131在相容旗標關閉時的276拒絕回饋與立即釋放等待；雙端各兩次明確重試皆無扣魔／換圖／物品變更，後續步行正常。原869是type12單圖、無動畫，現按原Sprite/869-0.spr的72×72單幀提交，不再誤找breath3；attr1仍保留不支援的layer診斷，多數身體受地形遮擋，未宣稱完整原版flags。主Debug2BDE0AA0／Release4D62B2C9各20相關Rust（3 live tests預設ignored另明確執行）；兩profile控制器拒絕／相容實驗／原入口24sessions、最終主視窗兩分支16sessions及新版Release跨端點6sessions，共選定46sessions由capture／被動權威／圖像獨立核對。959輸入、兩主與10其他測試EXE已保留。保留三張黑擷取；最終GUI3秒有界等待實際地形重繪並保存probe，未修圖或以HUD代替場景。map4成功仍限明示啟用classic相容實驗，不能證明實際部署已開WORLD或exact-native LTmap全覆蓋。實際跨sector／原生101、102入口、attr1／其他flags、全回歸／實體輸入／可攜／穩定性及完整G01–G12仍待。詳[限定收據](../驗證紀錄/current/native-world-integration/world-tree-native-qualified.json)。


## 2026-10-15：原生地形有界駐留與攻城入口限定驗證

2026-10-15：原LTmap保留全圖幾何／導航／入口，只解碼目前8×8世界格相機聯集1328×864；64MiB預算不變，跨區塊只交換地形並保留角色動畫快取。新增同房／同transport epoch的有界EarlyRoster，處理30人加入時S_CD早於S_CE；入口長路徑另限已驗證圖格，戰鬥路徑限制不變。主Debug61EAEB0E／Release47DC80C8各40相關Rust通過、2個live tests預設ignored另明確執行；兩profile的30人原15對15攻城101／102入口、map500入口及選圖／跨端點GUI共選定260正式sessions，capture／被動權威獨立核對。map563兩端實際跨sector通過，Debug243／Release216次C0A；Release573僅列入口回歸，因一端近入口出生、未跨sector，嚴格地形驗證未過且保留。GUI為192 DPI私有桌面，非實體鍵鼠。962編譯輸入、14份變更來源及28產物封存核對；兩profile各4個搬移診斷亦過。現行Release89整合suite＋main unit仍在建置／測試，不先記通過；lib unit未列入。攻城P鍵主視窗、完整當前TCP、多DPI／實體輸入、圖層flags、可攜與穩定性及G01–G12仍待。詳[限定收據](../驗證紀錄/current/native-terrain-residence/terrain-qualified.json)。


2026-10-15：043516-654捕獲大廳房間更新清除選取；重建版新增RoomChoice以連線epoch／角色／頻道／room ID保存選取，清單重排與人數變動可保持，移除／換作用域即失效。Debug03CD6005七組27項過、2個live預設ignored另待GUI，Release73289建置中；舊47DC的340／260證據仍限舊版。041327-824模式失敗重讀capture確認三個D1 02 01皆送出，撤回「掉點／按鈕過時」推論，實際是驅動預設模式順序及寬字串比對錯誤。重建版／登入器專案獨立，所有修改僅在重建版。


2026-10-15：房間更新清除選取已修復：Debug03CD6005／Release354EAE2D各27相關Rust通過、2 live預設ignored另以實際GUI支援執行。兩profile各60正式sessions的原30人15對15攻城GUI獨立通過，房間28→29仍保留選取、雙端跨S32、各255次移動／4C73／2S5A／2C08、28件背包與完整回收核對。僅192DPI私有桌面。新89整合組＋main回歸53322建置中，lib unit未列入。原Release長入口提示換行有下方操作列裁切待修；G11／完整G01–G12仍未完成。 見native-room-selection/qualified.json。


2026-10-15：HUD 狀態區已改為唯讀多行捲動，保留完整訊息與原世界顯示範圍。Debug32F27796／ReleaseED7F3E40各16相關Rust通過、2 live預設ignored另以GUI支援執行；兩profile各60正式sessions的原30人15對15攻城GUI獨立通過。Debug255／Release200次權威移動、各4C73／2S5A／2C08、雙端跨S32、28件背包與回收核對；長訊息捲底、唯讀輸入、焦點方向鍵零多餘移動及原圖最後一行21px餘量亦過。僅192DPI私有桌面。舊354E的342項廣域結果及90產物保留原版範圍；新UI跨端點／恢復重驗、G11與完整G01–G12仍待。詳[本輪限定收據](../驗證紀錄/current/native-status-scroll/qualified.json)。 初次獨立verifier錯把status-scroll-proof當PNG，另以觀察程序GDI取得20px而非實際41px行距；失敗／初次比較與原收據皆保留，最終以原圖文字墨跡完整性及Release EDIT自身行位置交叉核對，不改遊戲規則或降低門檻。


## 2026-10-15：目前HUD版傳送與恢復限定重驗

目前Debug32F27796／ReleaseED7F3E40的跨端點傳送與逾時、拒絕、錯誤服務、立即取消重試、延後取消、取消後自然逾期恢復，兩profile共14案112條正式LOGIN／GAME sessions通過。220筆C_MOVE逐筆核對、無多送；80張原世界PNG、56次原7311／7312圖像比對與128次過時GUI通知重播通過。962份來源及固定C02參考綁定各案。原取樣失敗及三張黑擷取保留並排除於通過計數；QA改為單次按鍵／完整座標對等待及有界地形重繪探測，產品Rust與Java參考未改。僅192DPI私有桌面；同端點當前參考、畸形／跨世代wire、完整當前回歸、實體輸入、G11與完整G01–G12仍待。重建版與登入器獨立，不以替換登入器產物作依賴。 見[收據](../驗證紀錄/current/native-status-scroll/teleport-revalidation/qualified.json)。


2026-10-15：當前C02同端點另以新Debug測試產物D7F431C2／962不變來源實測6條正式sessions：雙端已到達新epoch、角色／租裝owner及舊frame退役檢查點；後續兩筆CA00卻收到selection=0的rental-item-set回覆，兩端有界deadline失敗，未進共同戰鬥。server正常退出、session／membership／handoff／帳號皆回收，無SQL／持久化／權威注入。這是目前參考行為邊界，不列同端點成功，也不以替換登入器解決。 見[邊界證據](../驗證紀錄/current/native-status-scroll/same-endpoint-revalidation/boundary.json)。


## 2026-10-15：catalogue104=1身體半混合限定驗證

原catalogue104恰為1的身體已按RGB555半混合繪製，附件保留自身mix／隱形狀態，深度與點選owner不變。新主DebugC43FC219／Release34F16A07各16相關Rust通過、2live預設ignored另以隔離支援執行；各30組原圖scalar像素／owner比較通過，錯誤整體Half對照分別有589／678像素差異。兩profile的控制器相容實驗與GUI拒絕／成功共24正式sessions，GUI8筆C_MOVE無多送，962來源與10產物封存讀回核對。初版附件不可見的20組測試保留且未列入最終60組。WORLD成功仍限明示classic相容實驗；原執行檔framebuffer／RGB565／其餘flags、實體輸入／IME／多DPI、全當前回歸、音效GUI、乾淨主機可攜及G11仍待。前版112傳送GUI不移記到新主程式，完整G01–G12未完成。 見[收據](../驗證紀錄/current/native-draw-flags/body-half/qualified.json)。


## 2026-10-15：舊socket重複F4退役順序修復

傳送目的端已授權、C_D9尚未寫出時，舊socket重複F4曾誤觸未授權錯誤；已把既有退役守衛移至F3／F4分派之前，嚴格F0檢查與原guard內容不變。原版4過1失敗的EXE／來源已保留，原失敗測試本文未改。修復後Debug6項故障測試、37情境／41合成TCP、一般feature關閉的22相關Rust皆過；合成範圍不計正式sessions。正常主Debug64AAEA66另通過C02跨端點GUI6正式sessions、14筆移動零多送、角色14／11租裝及共同戰鬥／回收、4張原世界PNG與4原圖示像素核對。965來源及Debug6個現行產物封存核對。Release31078正在驗證，不先記PASS；完整lib編譯記憶體失敗、完整當前回歸／恢復廣度、同端點C02、實體輸入與G11仍待。測試feature只用於合成故障入口，交付主程式由唯一入口不開此feature重建。完整G01–G12仍未完成。 見[Debug收據](../驗證紀錄/current/native-teleport-wire/Debug-limited-qualified.json)。


2026-10-15續：舊socket重複F4退役順序修復現已通過兩profile限定核對：各6項故障測試／37情境／41合成TCP、一般feature關閉各22相關Rust，並以唯一入口正常主Debug64AAEA66／Release9DC43D91各驗C02跨端點GUI6正式sessions；合計12正式sessions、28移動零多送、角色14／11租裝接續、共同戰鬥及完整回收、8張原世界PNG／8原圖示比對通過。965來源與12個現行產品保留，兩主皆AMD64／PE32+。六種恢復正在新主程式重驗，未沿用舊HUD112。合成故障與正式連線分列；full lib編譯記憶體失敗、完整當前TCP、同端點C02、實體輸入／多DPI、音效／乾淨主機可攜與G11仍待，完整G01–G12未完成。 見[兩版本收據](../驗證紀錄/current/native-teleport-wire/both-profiles-core-qualified.json)。
