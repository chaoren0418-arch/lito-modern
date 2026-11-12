# Issue #261：主伺服器封包與規則查表

日期：2026-11-06；階段：resource。工作分支 `task/261`，核對的客戶端基準 commit 為 `928d0a3783298f5b878562d4262301f25a6c526a`。

## 做了什麼

- 唯讀盤點 `1-伺服器/src/com/lineage/`；整理107個C opcode常數、112個S opcode常數、96個明確C分派分支，以及103個C／243個S封包類別。別名不去重，未分派／未引用與常數表外0xF2及caller-supplied opcode另行標明。
- 建立 [server-protocol.md](../reference/server-protocol.md)：transport、primitive型別、雙向opcode、handler與多載、欄位順序／寬度／變長分支、Classic委派格式、核心來源SHA-256。大型索引以來源位置與簽章呈現，沒有複製Java檔或貼大段實作。
- 建立 [server-rules.md](../reference/server-rules.md)：登入／切角狀態機、座標權威、攻擊命中與傷害提交、施法／消耗、背包裝備、聊天音效、傳送初始化／ACK／落點與失敗恢復；每條附來源類別與方法。
- 確認主伺服器是 `380C_TW / 13081901`；本專案現行契約是 LiTo E149。兩者並非可互換的wire profile，本次沒有更改E149驗收對象、程式碼、AGENTS或 `docs/REPORT.md`。

## 本專案目前猜錯或缺少：按影響排序（20條）

**判定邊界：** 下列為已驗證的靜態格式差異／主伺服器支援缺口，不據此裁定現有E149實作「猜錯」。若目標仍為E149，不能直接照主伺服器數值修改。主伺服器引用的完整相對路徑見兩份reference；Rust路徑均在 `LiTo381-Modern/src/modern_lito/`。未實際連線重現，執行期結果屬推測。

| 排序／影響 | 現有實作證據 | 主伺服器權威與缺口／下一步 |
|---|---|---|
| 1／阻擋互通：第一包 | `native_opcodes.rs:9` S_KEY=0x01；既有契約要求5-byte body | `L1HandshakeHandler.channelActive` 為18-byte frame、0x96+seed+11-byte尾段；缺主server握手profile |
| 2／阻擋互通：加密 | `native_wire.rs:initial_counter/encrypt_in_place` 使用KEY_HIGH=0x87546CA1與body長度推進 | `Encryption.initKeys/encrypt/decrypt` 是獨立EKEY/DKEY、mask更新與0x287EFFC3累加；必須隔離實作並做離線golden，不能只換seed |
| 3／阻擋互通：opcode命名空間 | `native_opcodes.rs` 是LiTo Protocol.java常數；`protocol.rs:133` 另有LTM1版本協定 | 主server `OpcodesClient/OpcodesServer` 與兩者不同；缺可明示選擇與防混用的主server契約，禁止單一全域映射覆蓋 |
| 4／登入：選角語意 | `native_opcodes.rs:18-20` C_LITO_*；`native_login_wire.rs:serialize_fixture_login` 有LiTo帳密後的附加欄位 | 主server C_AuthLogin=0x77、C_AutoLogin=0xD2、C_LoginToServer=0x89且選角是S(name)；應另建主server登入／選角流程 |
| 5／登入：GAME許可 | `native_world.rs:578-582` 以0xD2更新started | 主server S_EnterGame=0xDF，且SessionPhaseGate等初始化全完成才IN_GAME；不能以相同名稱或收到單包推定主server可操作 |
| 6／所有接收器：padding | `native_login.rs:Reader.end`、`native_world.rs:235` 要求完全消費；`native_combat.rs:decode_attack` 要求16/29等精確長度 | 主server S body補至8-byte倍數；Teleportation.S_TeleportRecovery還經writeByte額外補零。缺按主server schema區分邏輯尾端與padding的解析 |
| 7／傳送：ACK與初始化 | `native_world.rs:acknowledge_movement_lock` 送0x5A；0x63設AckPending，0x25/mode17解除 | 主server S_Teleport2=0x95:H(map),D(id)，C_ACK=0x34，原生ACK後才動畫／落點提交；需另接完整pending世代與timeout流程 |
| 8／戰鬥：接收攻擊欄位寬度 | `native_combat.rs:171-218` 接0x23，damage_marker=b[10]單byte、heading=b[11]，近戰長16 | `S_AttackPacketPc` 為0x1E，damageWord=H、heading offset12，邏輯18/body24；僅重映射opcode會整段錯位 |
| 9／戰鬥：複合效果與無施法者 | 同一 `decode_attack` 要attacker!=0，遠程只收flight 0/0x7F與長29 | `S_UseAttackSkill.buildPacket` 可actor=0，含H傷害、sequence、gfx與座標，邏輯33/body40；另有S_UseArrowSkill，需按builder區分 |
| 10／戰鬥：HP/MP寬度 | `native_world.rs` 現有狀態解析依LiTo封包格式 | `S_HPUpdate/S_MPUpdate.buildPacket` 公版HH、RO版DD，opcode分別E1/21；部署profile尚未核對，不能固定成u16，也不能從傷害標記扣HP |
| 11／移動：送出 | `native_world.rs:move_request` 送0x0A + HH C | 主server C_MOVECHAR=0x1D，同樣HH C但server座標才是落點權威；既有client地形預檢不可取代主server結果 |
| 12／移動：接收 | `native_world.rs:995` 收0x12 + D HH C後end；已正確把source格加方向 | 主server S_MoveCharPacket=0x0A，額外C(0x80)再padding；語意可參考，wire缺相應decoder。不得誤報現有source/destination邏輯猜錯 |
| 13／物件與校正 | `native_world.rs:975/1026/1038` 物件/面向/刪除為0x0B/0x1C/0x15 | 主server CHARPACK/CHANGEHEADING/REMOVE為0x57/0x7A/0x78；同圖map reset與own packet可為座標校正。缺主server物件與校正組合，不應一律判新角色／新地圖 |
| 14／物品：拾取座標 | `native_ground_pickup.rs:wire_x/request` 送0x0B且x經縮放，再附1byte | 主server C_PickUpItem=0x70直接H(x),H(y),D(objId),D(count)，比對groundItem.x/y；沿用wire_x會產生位置差異 |
| 15／物品：使用與丟棄 | `native_item_actions.rs:use_request/target_location_request/drop_request` 送0x1C／0x0C | 主server C_USEITEM=0xA4、C_DROPITEM=0x19，使用尾段依use_type；需按所持instance模板選簽章，不能統一target+xy或只送D |
| 16／背包：record欄位與更新 | `native_inventory.rs:Reader.item/parse` 沒有descId，useType緊接id；blessing≤3；add/list=0x16/0x7C | 主server add/list=0x0F/0x05，增加H(descId)、保留位、sealed高位與尾段；S_ItemStatus另有0xF2/1長status。缺主serverrecord與長狀態解析 |
| 17／技能：選擇尾段 | `native_skills.rs:item_request/teleport_request/ground_request/request` 使用0x14；多個技能表依LiTo設計 | 主server C_USESKILL=0x06；row/column公式相同，但短body與技能分支、target=none／召喚51例外須查本表。不能以ID相同推定所有尾段相同 |
| 18／音效特效 | `native_world.rs:680/756` 音效=0x4A、座標效果=0x53、物件效果0x37只讀D,H後end | 主server音效=0x16、座標效果=0x6A、物件效果0x37另有H(time)多載；同值0x37也不保證完全同長，需覆蓋變長與padding |
| 19／訊息／聊天 | `native_world.rs:737/741` 系統字串/訊息碼分支0x0F/0x10；既有LiTo契約定義其頻道 | 主server一般/廣播/密語C=0x88/0x28/0xB8，S_SERVERMSG=0x47，C_Chat限制Java長度52與狀態gate；需主server訊息與頻道路由，不可沿用LiTo opcode解碼 |
| 20／數值與自訂分支 | `native_inventory.rs:Reader.item` count為u32；現行native_opcodes未列主server Classic與F2擴充 | ClientBasePacket.readD使用signed int；C_DropItem/C_PickUpItem拒負數；0xFE的Classic版本／nonce／token與0xF2長status、受擊表需獨立契約與分支測試。未讀DB，不能宣稱支援所有擴充功能 |

## 怎麼驗證

- **已驗證（靜態）**：由剔除Java註解後的常數、PacketHandler實際if分派、ClientPacketStructure、全部C/S類別read/write呼叫位置建立索引；檢查所有primitive呼叫均落入索引，補上巢狀Chunk及byte[]直接解析委派。手動回讀核心移動、攻擊、技能、背包、HP/MP、傳送與登入builder；特別校正writeByte追加NUL、RO的DD及表外0xF2，沒有照註解猜wire。
- **已驗證（文件）**：核對346類別、3401個primitive讀寫位置、336個Markdown內部連結及規則表來源方法／常數引用，執行 `git diff --check`；變更範圍只包含下列三份文件。
- **未執行**：cargo check/test、Java建置、socket、DB或GUI。這是resource文件任務，不把未跑項目當通過。

## 未驗證什麼

部署JAR與來源是否相同、config的語系與ClientLogin值、資料庫的技能／物品／地圖實值、全部動態分支的golden bytes與畫面時序均待驗證。全類別欄位表是帶來源位置的靜態索引，互斥分支不能串接；變數計算、提前return、巢狀status屬性語意仍需按任務回查方法。未宣稱已具備主伺服器互通或全技能行為驗收。

## 產出路徑

1. `docs/reference/server-protocol.md`
2. `docs/reference/server-rules.md`
3. `docs/reports/issue-261.md`（本報告，含20條差異）

## 給下一棒的提示

Claude雲端先審查來源／profile邊界，再由調度器合併已審查分支，讓兩份reference進main；本機執行者依優先序做離線golden與主server profile設計／實作。不要把此resource任務當作更換E149版本綁定的授權。之後封包／狀態任務先查reference，記錄來源方法與SHA；查無或分支不確定即回查唯讀來源，不從client畫面猜。

本次不新增計畫拆單JSON（resource非plan/design），不修改 `docs/REPORT.md`；main合併由既定Claude審查／使用者或調度器流程完成，本機只提交與推送 `task/261`。
