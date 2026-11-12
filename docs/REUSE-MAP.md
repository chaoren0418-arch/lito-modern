# REUSE-MAP — M2 任務的現成資源對照表（#267）

更新：2026-11-12。實作 M2 任務前先查本表（規則見 `CLAUDE.md` 第 7 條）。兩個來源都是**唯讀**：只讀原始碼與文件，不複製資源檔（pak／map／sprite／surf／XML），不改對方 repo。

## 0. 來源與版本

| 代號 | repo | 本機位置 | 讀取版本 | 讀法 |
|---|---|---|---|---|
| **TW** | tw-bin-engine（C#／.NET 10，天堂 3.81 TW1608262200.bin 的重新實作） | `D:\BinEngine` | `origin/main` `41b0fe2c`（2026-11-11） | 本機 working tree 落後，請用 `git -C D:\BinEngine show origin/main:<path>` |
| **LS** | lito-launcher-server（LiTo 伺服器 Java 17） | `D:\lineage開發\3-lito開發`（本 repo 上層） | `origin/main` `291474f2`（2026-11-12） | 本機 working tree 落後 602 commits，請用 `git -C "D:\lineage開發\3-lito開發" show "origin/main:<path>"` |

路徑縮寫：
- `TW:src/` = `engine/source/`；`TW:ui/` = `engine/source/NativeUi/`；`TW:docs/` = `docs/`
- `LS:S/` = `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`；`LS:P` = `LS:S/Protocol.java`；`LS:CS` = `LS:S/ClientSession.java`；`LS:M/` = `lito伺服器/LiTo381-Server/docs/maintenance/`；`LS:docs/` = repo 根的 `docs/`

### 0.1 三個必讀的差異（每一列「要調整」幾乎都來自這裡）

1. **opcode 編號不同（已確認）**。TW 對接的是 `D:\lineage開發\1-伺服器`（L1J 系，`com.lineage`）的 d287 JAR（`TW:src/LoginProtocol.cs:7` `JarSha256=d287bc4b…`，port 42061）。TW 文件用十進位：移動 C29、近戰 C229、弓 C123、施法 C6、物件 S87、傷害 S30。LiTo 3.81C 用 `LS:P` 的常數：`C_MOVE=0x0A`（:118）、`C_USE_SPELL=0x14`（:130）、`C_ATTACK=0x17`（:132），另有 `C_LITO_*`／`S_LITO_*` 0xC8–0xFD。**TW 的 opcode 數字一律不能照抄**；欄位排列（例如攻擊 `d target, h x, h y`、施法 `(id-1)>>3, (id-1)&7`）多半相同，可以參考。
2. **登入、大廳、房間、競技場在 TW 不存在（已確認）**。TW 用 L1J 的 S150 seed＋XOR 鏈（`TW:src/LoginProtocol.cs:30-43`）、C119 帳密、S178／S93 角色清單；LiTo 有自己的來源選服（S_FB＝S_LITO_SERVER_LIST）、頻道（C_CA＝C_LITO_ENTER_CHANNEL）、房間與結算。這一段只能看 LS 和本 repo 的 `docs/PROTOCOL-CONTRACT.md`。
3. **LS 的 `src/main` 不等於 E149（已確認＋推測）**。本 repo 以正式唯讀 JAR E149（`e1498871`）為準。LS 的 `src/main` 已經併入戰鬥候選（B-land #138），比 E149 新（推測：包含 E149 沒有部署的 8d26 以後的層，見 `PLAN.md` 事實 5）。所以 **LS 原始碼只能當「行為與格式的閱讀入口」**；寫進客戶端的格式和數值，要用 `javap` 對 E149 核對過，或已經寫在 `docs/PROTOCOL-CONTRACT.md` 裡。

### 0.2 判定欄的意思
- **照做**：格式或做法與 LiTo 3.81C／E149 一致，讀懂後可以直接照著寫 Rust（仍要寫測試）。
- **調整**：做法可以參考，但 opcode、欄位、數值、資源或 UI 的來源要換成 LiTo／E149 版本。
- **沒有**：兩邊都找不到可用的對應，只能依本 repo 既有研究（`docs/NATIVE-*`、`維護工具/`）或逆向。

## 1. 通用參考（多個任務共用）

| 主題 | TW | LS | 判定 |
|---|---|---|---|
| 協定總表 | `TW:docs/reference/server-protocol.md`（4179 行；C／S 全表、C6 技能尾段、C164 use_type 尾段、易混淆的共用 opcode）、`TW:docs/protocol/*.md`（約 45 篇）、`TW:docs/reference/protocol-gaps.md` | `LS:P` 常數區（約 :120–:720）、`LS:docs/reference/client-protocol-needs.md`（逐 opcode 列出 c/h/d/s、客戶端 sub_ 位址、證據等級；行號是舊版 fe38ce40 的，要重新定位；風險 R1：S_PACKET_BOX 與 S_LITO_CLAN 都用 0xFA） | TW：**調整**（只看欄位語意）；LS：**照做**（先對 E149） |
| 主分派 | `TW:src/WorldSession.cs` 的 `ReceiveCore`、`TW:src/WorldExtensions.cs` 的 `[WorldReceive(op)]` 註冊 | `LS:CS` :1737–1899 的封包 switch | 架構參考 |
| 資源讀取 | `TW:src/GroundSpriteReader.cs`（Sprite idx／pak）、`TW:ui/ClientArchive.cs`、`TW:src/TextArchive.cs`（Text.pak DES-ECB）、`TW:ui/UiXmlSource.cs`（Tile.pak XML AES 解密）、`TW:ui/SurfCatalog.cs`、`TW:src/OriginalMapCache.cs` | — | **調整**：TW 預設也讀 `D:\lineage開發\0-客戶端`，但角色／地圖多數走 `engine/assets`（r28 轉檔的 json，不在 git），`catalog.json` 也不在 git。Rust 端只能參考解析邏輯，不能拿 TW 的轉檔產物 |
| 原版位址 | TW 各份 docs 都記錄 TW16 的 VA／FileOffset | `LS:docs/reference/client-protocol-needs.md` 記錄 LiTo.bin 的 sub_ 位址 | TW 的位址屬於 TW16 BIN，**不能**套到 LiTo.bin；LiTo 的位址看 LS 與 `lito脫殼資源/` |
| 公式與技能資料 | — | `LS:docs/design/formulas-physical.md`、`formulas-magic.md`、`skill-targeting.md`（`target_to` 位元：0 自己、1 PC、2 NPC、4 血盟、8 隊伍、16 寵物、32 座標）、`skills-wizard.md`、`skills-elf.md`、`skills-others.md`、`skills-npc.md`、`items-consumables.md`、`gameplay.md`、`database.md` | **照做**（客戶端只用來決定目標型態與顯示；數值仍以伺服器權威封包為準） |

## 2. M2 逐任務對照

### T025 版本綁定與伺服器換版重核清單（文件）
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 換版重核清單、Protocol 變動 | 沒有 | `LS:P` 常數區；`LS:docs/combat-merge/class-diff.md`、`class-diff.json`（候選與正式版的 class 差異）；`LS:M/combat-coverage-current.md` | **照做**：class-diff 可以直接當「`Protocol$*`／ClientSession 變動 class」的來源 |
| TW 對應 | TW 對接 d287 JAR，與 E149 無關 | — | **沒有** |

### T026 技能狀態表與玩法支援矩陣（文件）
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 181 技能的目標型態、是否要 Ctrl | `TW:docs/protocol/skills.md` §5（七職業技能列表）、`TW:src/SkillCast.cs` 的 `SkillTargetKind` | `LS:docs/design/skill-targeting.md`、`skills-*.md`、`LS:S/BattleSpellCatalog.java`（12743 行）、`LS:M/combat-coverage-current.md` | LS **照做**（E149 與 `2066774` catalog 常數相同）；TW **調整**（TW 是 1-伺服器的 skills 表，ID 範圍與分派不同） |
| 物品／遠程／耗材／buff／復活矩陣 | `TW:docs/reference/server-protocol.md` C164 use_type、`TW:docs/protocol/item-use.md`、`resurrection.md` | `LS:docs/design/items-consumables.md`、`gameplay.md`、`LS:M/equipment-toggle-0.6.65.md` | 架構參考；LiTo 的值看 LS |

### T020 #97 暗隱術（ingress clock、queue、ready gate）
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 施法排隊、施法者忙碌時延後 | `TW:src/Combat.cs` 的 `QueueIfCasterBusy`、`TW:src/AttackSkillPackets.cs` 的 `SkillCastQueue` | `LS:M/native-self-cast-current.zh-TW.md` | **調整**：TW 是自己的設計，不是 LiTo.bin 4E5C00／4D65F0 的還原；只能當「排隊與就緒判斷」的寫法參考 |
| 資源就緒 gate、Static2941Bank、4D6570 交錯 | 沒有 | 沒有 | **沒有**：只能依本 repo HISTORY L1–L13 與候選第 29–30 列 |

### T021 技能 98／99／100
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 送包、材料、HP／MP 邊界 | `TW:src/SkillCast.cs` 的 `Wire`（D／DHH／DHHS／HH／HD／S 形狀） | `LS:S/BattleSpellCatalog.java`、`LS:docs/design/skills-others.md`（黑暗妖精）、`LS:M/combat-coverage-current.md` | LS **照做**；TW 的形狀分類可借用，**調整** opcode |
| S34／S08 現身、音效、History | `TW:src/Invisibility.cs`（S171） | `LS:M/native-de-visibility-current.zh-TW.md` | **調整** |

### T027 候選獨有 48 技能（6 批）
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 各技能目標型態、送包形狀 | `TW:src/SkillCast.cs`、`TW:docs/protocol/skills.md` | `LS:S/BattleSpellCatalog.java`、`LS:docs/design/skills-wizard.md`、`skills-elf.md`、`skill-targeting.md` | LS **照做**；TW **調整** |
| 特效、投射物、子特效 | `TW:src/MagicRendering.cs` 的 `EffectSprite.Parse`、`TW:src/SubEffects.cs`、`TW:src/ProjectileRendering.cs`、`TW:docs/reference/skills-visual.md` | `LS:M/illusion-presentation-0.6.50.json`、`dragon-presentation-0.6.48.json` | TW **照做**：list.spr 是兩邊共用的格式（推測：兩個客戶端的 list.spr 版本可能不同，要對本機 `0-客戶端` 核對） |
| 5 TELEPORT | `TW:src/Teleport.cs`（S149→C52 ACK） | `LS:P.portal`（S_PORTAL `hhh`）、`LS:CS.completePortal`、`requestTeleport` | LS **照做**；TW **調整** |

### T022 未接的 91 技能（10 批，101–220）
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 1–220 全部 ID 的編碼 | `TW:src/SkillCast.cs` 的 `Supported`（1–220 都能編碼）、`Wire` | `LS:S/BattleSpellCatalog.java` | TW 的 `(id-1)>>3,(id-1)&7` 與 LiTo 的 row／column 相同（契約 T004-4）：**照做**；opcode 改成 0x14。21 個 wrapper 走 11-byte 形狀、113 帶 MS950 文字，以 `docs/PROTOCOL-CONTRACT.md` 為準 |
| 冷卻、施法時間 | `TW:src/ActionCadence.cs` 的 `SpellListCooldowns`（讀 kor25 SpellList.xml）、`MagicRules.CastMs`（1320／1480ms） | `LS:docs/design/skills-*.md` | **調整**：TW 讀的是參考客戶端 kor25，不是 LiTo 資源；LiTo 的 1480ms 窗見 T046 |
| 技能名稱、gfx | `TW:src/MagicSpells.cs`（讀 `engine/assets/magic/catalog.json`，不在 git） | `LS:docs/design/skills-*.md` | TW **沒有**可用的表；用 LS 與本機 Text.pak |
| 騎士／龍騎士／幻術士技能（181–220） | `TW:docs/protocol/skills.md` §5 | `LS:docs/design/skills-others.md`、`LS:M/dragon-presentation-0.6.48.json`、`illusion-presentation-0.6.50.json` | LS **照做** |

### T024 時序與 hurt（#87 持續 4183、LT hurt／base）
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 動作時間軸、hurt 影格 | `TW:src/SubEffects.cs` 的 `ActionTimeline`、`TW:src/ObjectRendering.cs` 的 `ListSprRuntime`、`TW:src/ActionCadence.cs`（加速 3/4 倍）、`NormalHaste.cs` | — | **調整**：TW 有「list.spr 受擊格 `!`」與動作速度倍率的寫法；LT 相對 gfx、uint32 rollover、class7 child 是 LiTo.bin 專有，只能看本 repo HISTORY |
| 4183、S0B 重送 | 沒有 | 沒有 | **沒有** |

### T028 main 已接 38 技能補驗與候選修正重寫
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 2 日光術：夜間亮度、3600 秒 | `TW:src/Ambience.cs`（`AmbienceCurve`）、`TW:src/MagicRendering.cs` 的 `MagicBrighten`、`TW:src/MapEffects.cs` 的 `MapEffectLightTable` | `LS:M/native-light-action-current.zh-TW.md`、`LS:S/NativeSourceIcons1.java` 的 `duration()` | TW **調整**（亮度曲線是 TW 的實作）；LS 期限 **照做**（對 E149） |
| 13 偵測、60 隱身 | `TW:src/Invisibility.cs` | `LS:M/native-de-visibility-current.zh-TW.md` | **調整** |
| 19／35／57 治癒其他目標、8／9／10／11／18／28 | `TW:src/SkillCast.cs` 的物件目標分派 | `LS:docs/design/skill-targeting.md`、`skills-wizard.md` | LS **照做** |
| 6 冰箭、火球 class5、S23 class4、NativeSpellClock、class1 起點 | `TW:src/ProjectileRendering.cs`（kind 3 每格 30ms、kind 4／5 每格 60ms，`TW:docs/reference/skills-visual.md` §2.2、§4） | — | **調整**：可以參考 kind→速度的對照，但 LiTo 的 S23 時鐘以本 repo `維護工具/native-E149-S23-clock*.md` 為準 |
| list.spr 614 行 parser 錯誤 | `TW:src/MagicRendering.cs` 的 `EffectSprite.Parse`、`ListSprFields`（105 cloth、111 stride、112 altdir）、`TW:src/ObjectRendering.cs` 的 `ListSprRuntime` | — | **照做**（推測：TW 的 parser 容錯可能較寬，要拿同一份 list.spr 比對逐行結果） |
| 72 的 3934 空影格、77 Disintegrate | 沒有 | — | **沒有** |

### T029 非技能戰鬥（遠程、操作）
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 弓、arrow／flightType、投射物 | `TW:src/Ranged.cs`（`RangedRules`、`ArrowCount`）、`TW:src/ProjectileRendering.cs`、`TW:docs/reference/skills-visual.md`（箭 gfx 66／2349／2989、光箭 167） | `LS:P` 的 `C_FAR_ATTACK`（:8850 註明 `cdhh`） | TW **調整**：TW 弓送 C123、補零到 16 bytes；LiTo 是 `cdhh` 9 bytes。投射物速度與 gfx 對照可以參考 |
| 近戰收包、施法者忙碌排隊 | `TW:src/Combat.cs` 的 `ReceiveCombat`、`ApplyMeleeAttack`、`ApplyArrowAttack`、`QueueIfCasterBusy` | `LS:CS.attack` | **調整**：TW 的 S30 對應 LiTo 的 S23 |
| 目標圈、走近後攻擊 | `TW:src/ObjectRendering.cs:827`（寫死的 38×14 金色橢圓，推測不是原版）、`TW:src/ApproachWalk.cs`（依 sub_500E40 mode 3）、`TW:src/PlayerPointer.cs` 的 `ActAsync` | — | **調整**；TW 沒有自動追擊（`Combat.cs` 寫明），追擊要自己做 |
| 攻擊速度 | `TW:src/ActionCadence.cs`、`NormalHaste.cs`、`PlayerProfiles.cs` | `LS:P.speed` | **調整** |
| 空揮 `17 00 00 00 00 DE 7F 1D 80` 與拒絕條件 | `TW:docs/protocol/combat.md`「靜默拒絕、miss」（target=0） | `LS:S/EmptyMelee.java`、`LS:M/empty-melee-0.6.49.json` | LS **照做** |
| miss／critical 顯示 | TW 沒有浮動傷害數字 | — | **沒有**（歸 T047） |

### T023 G06.3d 傳送與失敗恢復（客戶端部分）
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 傳送封包、ACK、移動鎖 | `TW:src/Teleport.cs`（S149，:88 回 C52）、`TW:src/Portals.cs`、`TW:src/Afflictions.cs` 的 `ReceiveParalysis` | `LS:P.portal`、`LS:CS.completePortal`、`requestTeleport` | LS **照做**；TW **調整** |
| 旅館、TeleportMe 圖示排版 | `TW:src/NpcServices.cs`、`TW:docs/demo/README.md`（入住 map 16384 的證據） | `LS:CS` 的 `PendingHotelRental`、`LS:S/NativeLitoRentalTeleportEndpointCatalog.java`、`ServerConfig` :718 | LS **照做**；TW 的旅館是 L1J 的一般旅館，**調整** |
| 第二次 cancel 的 LOGIN 恢復、跨端點、世代 | 沒有（TW 沒有跨端點服務） | `LS:CS` 登入／選角流程 | TW **沒有** |
| 奪命之雷不建 portal、69 失敗 4-byte 解鎖 | 沒有 | `LS:M/` 的 `source-*`、`native-*` 施工紀錄（TASKS T023 已列路徑） | LS **照做** |

### T041 測試腳本加「技能」「G06」兩類
| TW | LS | 判定 |
|---|---|---|
| `TW:.github/workflows/build.yml`、`npc-checks.yml`（windows-latest 多組離線檢查）、`[WorldModuleTest]` 標記 218 處 | — | **調整**：只借分類與總表格式；本 repo 腳本是 PowerShell＋cargo |

### T043 雙真實玩家戰鬥 fixture
| TW | LS | 判定 |
|---|---|---|
| 沒有（TW 只有單人對 L1J 伺服器） | `LS:S/GameHub.java`、`BattleRules`、`DurableBattleSettlement`、`SettlementJournal`、`TeamDisconnectSettlement`；`LS:docs/design/gameplay.md`；`LS:M/room-rules-0.6.61.md`、`bot-room-count-0.6.64.md` | LS **照做**（startBattle 是否關訓練 bot 仍要 javap 對 E149） |

### T044 移動正確性與 G04 剩餘項
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 本地預測與伺服器校正 | `TW:src/OnlineMovement.cs`（`MovementRules.Move`，C29 `hhc`＋C225 轉向，S87 校正） | `LS:P` :8862（LiTo C_MOVE 是 `chhc`） | **調整**：做法相同，opcode 與欄位順序改 LiTo |
| 通行與碰撞 | `TW:src/OriginalMapCache.cs` 的 `Passable`（:83）、`MovementRules.Passable` | `LS:src/main/resources/maps/` | **調整**：TW 讀轉檔後的地圖快取；LiTo 只有競技場與租裝城鎮 |
| 門、動態物件 | `TW:src/Doors.cs`（`DoorBlocks`、`ReceiveDoorPack`、S209 門屬性） | `LS:S/` 的競技場門（G04.2） | **調整** |
| 移動鎖握手 S_BLINK→C_BLINK→`25 11` | `TW:src/Resurrection.cs` 的 `ReceiveMoveLock`（S241，等下一個 S87 才解，**不同**） | `LS:P.movementLock`（:14517，S_BLINK 0x63）、`movementLockRelease`（:14524，`25 11`）、`LS:CS.issueMovementLock`、`movementUnlock`、`LS:M/movement-lock-0.6.66.md` | LS **照做**；TW 的鎖定模型不同 |
| 加入／離開、AOI、換圖、同 ID 替換 | `TW:src/WorldObjects.cs`（S87）、`TW:src/WorldEntry.cs` | `LS:CS` | **調整** |

### T045 物品系統缺口
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| use_type 與需要目標的道具 | `TW:src/ItemUseArguments.cs` 的 `ItemUseCodec.Kind`／`Encode`（背包目標 7／14／26／27／46／55、物件 8／30、物件＋座標 5／17／39、座標 42、書籤 6／29…）、`TW:docs/protocol/item-use.md` | `LS:docs/design/items-consumables.md` | **調整**：use_type 編號是 3.81 共通的（推測），但要對 E149 的 C_USE_ITEM 解析核對 |
| charge、數量、銷毀 | `TW:src/Inventory.cs` :69–86（S57 移除、S24／S242 數量為 0 時移除）；charge 只解析不扣 | — | **調整** opcode |
| 箭的扣除 | `TW:src/Ranged.cs` 的 `ArrowCount`（伺服器扣、客戶端只看回包） | — | **照做**（做法） |
| S_SHOW_MAP 18 筆 | TW 未實作（`TW:docs/reference/protocol-gaps.md` :26、:172、:228） | `LS:P.showMap`（:12438，`dc`，頁 1..18） | LS **照做** |
| 擂台 NPC 商店 | `TW:src/NpcServices.cs`、`TW:docs/protocol/npc.md`（S70／C161 買、S65 賣） | `LS:CS.sendBuyList`、`addShopItem`、`LS:P.litoRentalBuyList`、`LS:M/native-de-shop-current.zh-TW.md` | LS **照做**；程式裡沒有 16 列上限，「16 列」只能依 `data-sources-0.6.60.json` |
| unsigned 物件 ID | TW 一律 `uint` | — | **照做** |

### T046 快捷列、技能書與狀態 UI
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| 快捷列 | `TW:ui/MacroPanel.cs`（F5–F12、3 分頁、存檔）、`TW:ui/ExMacroPanel.cs`、`TW:src/CharacterConfig.cs` | — | **調整**：TW 的版面讀 Tile.pak XML（推測：LiTo 的 `0-客戶端` Tile.pak 有沒有同名 XML 要先用 PakViewer 查） |
| 技能書 | `TW:ui/SpellWindowPanel.cs`（備援 `TW:src/MagicUi.cs`）、S164 已學清單（`TW:src/Magic.cs:83`） | — | **調整**：LiTo 用 S1E／S1F 位圖 |
| 狀態圖示 | `TW:ui/BuffIconsPanel.cs`（effectlist2.xml＋.tbt，每列 22 格）、`TW:src/StatusIcons.cs`、`TW:src/MagicStatus.cs`、`TimedBuffs.cs`、`PacketBoxStatus.cs`（S250 子碼）、`TW:docs/protocol/status-icons.md` | `LS:S/NativeSourceIcons1.java`（ICONS1 204 bytes、技能→slot、`duration()`）、`LS:S/NativeSourceProc68Icon`（ICON_I2H `FA 28 h`）、`LS:P.characterStatus`（S_STATUS 0x0C）、`LS:P.speed`（`dch`） | 呈現：TW **照做**（本 repo 也引用 `effectlist2.xml`）；格式：LS **照做** |
| HP／MP、頭頂血條 | `TW:ui/HpMpPanel.cs`（`HpMpHeadBarPanel`，座標依原版反組譯寫死） | `LS:P.hitRatio`（:16006，S_HIT_RATIO 0x68＝104，`dc` 0..100；0xFF 隱藏） | TW 座標 **調整**（TW16 與 LiTo.bin 的 HUD 位置可能不同，推測）；LS **照做** |
| AC／屬性 | `TW:ui/LeftStatusPanel.cs`；`StatInfoPanel.cs` 數值留白（TW 的 K1） | `LS:P.characterStatus` | **調整** |
| 版面總表 | `TW:docs/reference/ui-layout.md`（約 100KB）、`classic-ui-audit.md`（61 個面板）、`TW:docs/design/native-ui.md`、`TW:ui/UiLayout.cs` | — | **調整**：TW 缺的 XML 會改讀 kor25 參考客戶端（`RefClientXmlSource.cs`），LiTo 不能用 kor25 |
| 1480ms 動畫窗連按拒絕 | `TW:src/ActionCadence.cs`（`MagicRules.CastMs` 預設 1480） | `LS:docs/design/skills-*.md` | **照做**（數值一致；推測：來源不同，要對 E149） |
| 龍之護鎧 3080、9 個幻術 lease | — | `LS:M/dragon-presentation-0.6.48.json`、`illusion-presentation-0.6.50.json` | LS **照做** |

### T047 影格事件、gfx1544、可見角色原始動作
| 項目 | TW | LS | 判定 |
|---|---|---|---|
| Effect／Hit 影格事件 | `TW:src/SubEffects.cs` 的 `FrameTokens`（`]` 子特效、`{` `}` 淡入淡出）、`ActionTimeline`、`SubEffectSet`；`TW:src/MagicRendering.cs`（`!` 受擊格、109 kind、104 混色、110 fps） | — | **照做**：list.spr 標記語意兩邊共用 |
| 音效觸發 | `TW:src/CombatSfx.cs`（list.spr `[` `<` `^`）、`TW:src/SoundPackets.cs`（S55／S106）、`MapEnvSound.cs`、`BackgroundMusic.cs`、`TW:docs/reference/sound.md` | — | list.spr 音效 **照做**；封包 **調整** |
| 變身、隱身 | `TW:src/Polymorph.cs`（S76）、`TW:src/Invisibility.cs`（自身 50% 半透明） | `LS:M/gm-polyme-0.6.71.md`、`native-de-visibility-current.zh-TW.md` | **調整** |
| 光術封包順序 | — | `LS:M/native-light-action-current.zh-TW.md` | LS **照做** |
| 傷害數字、miss／critical | TW 沒有 | — | **沒有** |
| gfx1544 action0、首幀 11／12 物件 | 沒有 | — | **沒有** |

### T048 其他復活分支
| TW | LS | 判定 |
|---|---|---|
| `TW:src/Resurrection.cs`、`TW:src/Recovery.cs`（S219＋321／322 確認回 C121、S85 動作、C177 重新開始）、`TW:docs/protocol/resurrection.md` | `LS:P.resurrect`（:16126，`dcdh`）、`LS:CS.confirmDeath` | LS **照做**；TW 流程 **調整**；榮譽模式延遲重生只在 LS |

### T050 S_EVENT subtype33 完整性挑戰與 C_ADDR
| TW | LS | 判定 |
|---|---|---|
| TW 未實作，`TW:docs/reference/protocol-gaps.md` :186、:233 標「不建議補」 | `LS:P.addressChallenge`（:14483）、`addressProof`、`LS:CS.submitAddressProof`、`ServerEventType` | LS **照做**；TW **沒有**。注意：回應內容是對 LiTo.bin 位址區段的證明（推測），Rust 客戶端沒有 LiTo.bin 記憶體可讀，要先研究伺服器怎麼驗 |

### T054 房間分支（G06.1、G06.2）
| TW | LS | 判定 |
|---|---|---|
| **沒有**（TW 沒有大廳與房間；只有 UB 幽靈觀戰 `TW:docs/protocol/ghost-mode.md`） | `LS:CS` 的 `createRoom`、`joinRoom`、`setReady`、`setForce`、`kickUser`、`editRoom`、`setObserver`；`LS:P.litoRoomSummary`、`litoRoomConfiguration`、`readyState`、`litoForceChanged`、`litoKickStatus`、`litoObserverResult`、`observerBox`；`LS:S/GameHub.java`（`RoomSettings`）、`LobbyRoomDirectory`、`TournamentRoomPolicy`、`TournamentPresence`；`LS:docs/design/gameplay.md`（模式 wire 0–5、房間旗標 0x04 密碼／0x08 觀戰／0x10 允許上位／0x20 人數一致、進房檢查順序、準備後倒數 3 秒、被踢者不能再進、觀戰頻道 99）；`LS:M/room-rules-0.6.61.md`、`bot-room-count-0.6.64.md`、`room-directory-0.6.54.json` | LS **照做** |

### T055 租裝來源（G06.3、G06.3c）
| TW | LS | 判定 |
|---|---|---|
| **沒有**（TW 只有一般商店與旅館） | C_CA＝C_LITO_ENTER_CHANNEL（`LS:CS` :671 `enterChannel`，同一 byte 選租裝組合）；S_FB＝S_LITO_SERVER_LIST；`LS:S/RentalItemSetCatalog`、`RentalLoadoutPlan`、`RentalTownHotelRegistry`、`NativeLitoRental*Catalog`、`NativeLitoRentalTownNpcCatalog`、`NativeLitoTownCatalog`；`ServerConfig` :415（map500）；`LS:CS.quitFight` | LS **照做**（quitFight 的伺服器缺口在 U-M3-1） |

### T061 Ctrl 強制目標（LFC1）與目標政策
| TW | LS | 判定 |
|---|---|---|
| 沒有 LFC1（L1J 沒有這個擴充） | `LS:S/ForcedCombat.java`、`LS:CS.handleBeforeForcedCombat`、`LS:M/forced-combat-0.6.67.md`（附加 `4C 46 43 31 <opcode> 01 67 A5`）；目標政策 `LS:docs/design/skill-targeting.md` | LS **照做**；TW **沒有** |

### T062 選角 8 角色分頁
| TW | LS | 判定 |
|---|---|---|
| `TW:ui/SelectCharPanel.cs` :30–56（每頁 4 格、上下翻 2 頁、共 8 槽，上限由 S178 slots 決定，`TW:src/CharacterManagement.cs:129`） | `LS:M/buff-schema-and-character-pages-20261018.zh-TW.md`（伺服器送 8 筆；原生 4 格由登入器 DLL 分頁；第二場選角崩潰，分頁候選未發布） | TW 的分頁 UI **調整**（opcode 與清單封包換成 LiTo）；LS 規則 **照做** |

### T063 寵物與召喚物控制
| TW | LS | 判定 |
|---|---|---|
| `TW:src/Pets.cs`、`PetTarget.cs`、`PetEquipment.cs`、`TW:ui/PetControlPanel.cs`（C125 aggressive／stay／dismiss…；C103／C104、S83／S176）、`TW:docs/protocol/pets.md` | `LS:P.slaveControl`（S 0x57／C 0x4F）、`LS:CS.slaveControl`、`LS:P.classicPet`、`LS:S/BattleCompanion.java` | 控制面板與指令清單：TW **調整**；格式：LS **照做** |

## 3. 彙總

| 判定 | 任務 |
|---|---|
| LS 可直接照做為主 | T025、T026、T043、T048、T050、T054、T055、T061；T021／T022／T027 的目標型態與送包 |
| TW 可直接照做為主 | T028／T047 的 list.spr 解析與影格事件、特效與音效標記；T046 狀態圖示的 effectlist2 呈現 |
| 兩邊都要調整 | T029、T044、T045、T046（快捷列、技能書、HUD 座標）、T062、T063、T023 |
| 兩邊都沒有 | T020 的 ready gate／Static2941Bank、T024 的 4183 與 LT hurt、T047 的 gfx1544 與傷害數字、T028 的 72／77 |
| 不適用 | T041（只借格式） |

## 4. 結論：繼續 Rust 重建，還是改用 TW 的 C# 引擎當基底（給使用者決定）

### 4.1 兩條路的現況（只列看得到的證據）
| 面向 | A：繼續 Rust（本 repo） | B：TW C# 引擎＋LiTo 大廳／競技場 |
|---|---|---|
| 程式量 | 933 個 .rs、約 16.4 MB（`git ls-files`，已確認） | 515 個 .cs、約 88,000 行、約 6.8 MB（已確認） |
| 登入 | LiTo 來源選服、帳密、選角已接線（`docs/NATIVE-LOGIN.zh-TW.md`，已確認） | 只有 L1J 的 S150 seed／C119／S178（`TW:src/LoginProtocol.cs`）；**沒有** LiTo 的來源選服（已確認） |
| 大廳／房間 | 4 個頻道、大廳清單、進房、成員、準備、離開已接線（`NATIVE-LOBBY`、`NATIVE-ROOM`，已確認） | **完全沒有**（搜尋 arena／lobby／競技／大廳都是 0，已確認） |
| 戰鬥與結算 | Tournament 近戰、伺服器傷害與死亡、結算 SD3、返回已接線（`NATIVE-COMBAT`、`NATIVE-OUTCOMES`）；main 已接 38 技能（已確認） | 近戰、弓、施法 1–220 編碼、buff、特效、音效都有，但對的是 L1J 協定；沒有 LiTo 的結算與房間返回（已確認） |
| 世界內 UI | 多數畫面用 Win32 標準控件（56 檔用 EDIT／LISTBOX／BUTTON；97 檔有 surf 原資源，已確認）；HUD 是唯讀多行文字區（`NATIVE-WORLD-INTEGRATION:159`）。推測：外觀離原版較遠 | 原生 UI 讀 Tile.pak XML、61 個面板、快捷列、技能書、buff 圖示（已確認）；缺的 XML 改讀 kor25 參考客戶端 |
| 驗證 | M1：lib 2464 測試 1889 過 0 敗、G06 5 scenario 通過（已確認）。G02 雙真實玩家在當版 main 未驗（`docs/G01-G12.md` G02 `current_main_two_round_verified=false`）；完整 GUI 驗收未做 | M7 的「已有」都是靜態核對；`smoke-m7.md` 59 項結果表全空（已確認）。真機只有 M1–M5 的局部：登入、選角、裝備、近戰子項、光箭音量、旅館 |
| 資源 | 讀 `D:\lineage開發\0-客戶端` | 預設也讀 `0-客戶端`（`GroundSpriteReader.cs:8`），但角色／地圖多數讀 `engine/assets`（r28 轉檔，不在 git）、技能表讀不在 git 的 `catalog.json`（已確認） |
| 剩餘範圍 | M2 57 張＋M3 44 張（已凍結，`PLAN.md`） | 沒有現成計畫，要新開 |

### 4.2 B 路線要補的東西
1. **協定層全換**：TW 沒有集中的 opcode 表，數字寫死在各檔（已確認），所以要逐檔把 L1J 的 opcode 與欄位改成 LiTo 3.81C／E149（例如 C29→0x0A 而且欄位順序 `hhc`→`chhc`、C229→0x17 而且 16→9 bytes、S30→S23）。登入要整段重寫成 LiTo 的來源選服與 `C_LITO_*`／`S_LITO_*`。
2. **大廳、頻道、房間、準備、觀戰、結算、返回選角的 UI 與狀態**：TW 完全沒有。Rust 這段已經有，但 UI 是 Win32 控件，C# 一樣要重做畫面。
3. LFC1、subtype33、租裝城鎮、map500、競技場門：TW 都沒有。
4. **要開新 repo 或 fork**：規則禁止改 tw-bin-engine。另外 `engine/assets` 與 `catalog.json` 不在 git，換到新 repo 時要重新產生；它們依賴 r28 封存（推測：要重跑 TW 的轉檔工具）。
5. TW 自己的 M7 真機 smoke 也還沒跑，基底本身的可玩性沒有同版真機證據。

### 4.3 A 路線還缺的東西
1. 「登入→大廳→進房→戰鬥→退出」的功能子集**已經接線**；缺的主要是驗證，不是功能：G02 雙真實玩家（T043）、完整 GUI 驗收（M3 的 T034／T030），以及可攜交付（T031）。
2. 世界內的完整度：91＋48 個技能、快捷列、技能書、狀態圖示、HUD 外觀，也就是 M2 的大部分。這些可以照本表 §2 參考 TW 的寫法。
3. 風險：程式量已經很大（16.4 MB），而且大量候選凍結、測試待驗（`PLAN.md`「真正的缺口」）；UI 外觀與原版的差距沒有量化過（待驗證）。

### 4.4 結論
**建議繼續 Rust 重建（A），把 TW 當參考來源，不換基底。** 依據：
- 目標流程裡 LiTo 專有的那一半（來源選服、頻道、大廳、房間、準備、結算、返回）Rust 已經有，TW 一行都沒有；TW 比較強的是 LiTo 流程以外的世界內功能與 UI 外觀。
- B 不只是「加上大廳／競技場」，還要把整個協定層從 L1J 換成 LiTo（opcode、欄位長度、登入）。這部分沒有集中的表可以一次改。
- 兩邊的基底都沒有同版的完整真機證據，所以 B 沒有「已經驗證過所以比較短」的優勢。

**推測（沒有實測，只是依上面證據估的）**：如果只看「登入→大廳→進房→戰鬥→退出」能不能走完，A 剩下的是 T043（2 張）加上一次真機 smoke；B 至少要重寫協定層、登入與大廳／房間／結算的狀態機與畫面，量級明顯比較大。

**給使用者的另一個決定點（建議，不是本任務的變更）**：M2 現在的排序是技能長尾在前（T022 10 張、T027 6 張）。如果優先目標是先走通流程，可以考慮把 T043（雙真實玩家 fixture）和一次流程 smoke 提前，技能批次放後面。要改排序需要 `needs:user`（`TASKS.md` 範圍凍結規則）。

**改走 B 的時機（推測）**：如果之後 GUI 驗收認定 Rust 的 Win32 控件畫面不能接受，而且要重做成原版外觀的量大於移植協定層，再重新評估。到時可以先只移植 TW 的 UI 版面邏輯（`TW:ui/UiLayout.cs`、`UiXmlSource.cs`、`SurfCatalog.cs`）到 Rust，不必換整個基底。
