# 主伺服器封包對照表

Issue #261；2026-11-06。本表的「已驗證」均指唯讀原始碼核對，**不代表線上互通、執行測試或正式 JAR 驗證**。規則見 [server-rules.md](server-rules.md)，現有 Rust 差異見 [任務報告](../reports/issue-261.md)。

## 0. 適用範圍與查表順序

權威來源是 `1-伺服器/src/com/lineage/`，opcode 檔自述 `380C_TW / 13081901`。此來源是天堂主伺服器；本專案既有 [PROTOCOL-CONTRACT](../PROTOCOL-CONTRACT.md) 對象則是 `lito-launcher-server 0.6.71 / E149`，兩者握手、加密、opcode 均不同。**不得直接拿本表數值覆寫 LiTo 常數，也不得由同名封包推定格式相同。** 本任務沒有更換現行驗收對象。

後續封包／狀態任務一律先查本表與規則表：先標明「主伺服器」或「LiTo E149」→ 查方向與 opcode → 查 builder／handler 多載與分支 → 查規則與回覆 → 核對來源是否漂移。主伺服器行為以本來源方法為準；LiTo 的 transport／大廳／房間仍依既有契約，借用主伺服器規則前必須明列轉換。查無定義時寫待驗證，不從客戶端外觀補猜欄位。

來源路徑一律以 `1-伺服器/...` 表示，是與本 repo 相鄰的唯讀來源，非 repo 內檔案。沒有讀取 `config/`、DB、備份／封存或運行程序；沒有複製 Java 檔。簽章內 `Config*` 只代表原始碼中的條件，實際部署值未讀取。

## 1. 傳輸與分派

| 項目 | 已驗證結論 | 來源類別／方法（路徑均在 1-伺服器/src/com/lineage/） |
|---|---|---|
| frame | `H(total) + encrypted body`；total 含 2-byte header；Netty inbound 上限 10240 bytes，先移除 header 再解密 | `netty/NettyChannelInitializer.java`：`initChannel`；`netty/codec/L1CipherEncoder.java`：`encode` |
| 第一包 | 明文 total=18；body 為 `C(0x96), D(seed), raw[11]`；固定尾段 `9D D1 D6 7A F4 62 E7 A0 66 02 FA` | `netty/handler/L1HandshakeHandler.java`：`channelActive` L41、67；seed 每連線產生 |
| 舊常數 | `OpcodesClient._firstPacket` 也有初始化樣板，但活動 Netty 路徑是上列 `channelActive`，不能以 `_seed` 推論每連線固定 seed | `echo/OpcodesClient.java` L17；`L1HandshakeHandler.channelActive` |
| 加密 | `Encryption.initKeys(long)` 由 seed 與 `0x930FD7E2` 建立分向 EKEY／DKEY；`encrypt/decrypt` 依封包 mask XOR 第一個 key word，第二個 word 加 `0x287EFFC3`；不是 LiTo 的長度累加 counter | `echo/encryptions/Encryption.java`：`initKeys` L74、`encrypt` L103、`decrypt` L189；逐位元運算須讀該方法與相依工具，不由此摘要重寫 |
| S padding | `getBytes()` 在邏輯 payload 尾端補零至 8 的倍數；`L=欄位寬度和` 時常規 S body=`8×ceil(L/8)`，frame 再加 2；初始化明文例外 | `server/serverpackets/ServerBasePacket.java`：`getBytes` L334、`prepareForWrite` L403 |
| C 尾段 | 結構 preflight 核對必要欄位，許多封包容許 native trailing/padding；不可全域套「剩餘 bytes 必須為零」 | `server/clientpackets/ClientPacketStructure.java`：`validate`、`layoutFor` |
| 中央入口 | 先 `sessionPhase.allowNow`，再 `ClientPacketStructure.validate`，再忽略控制包／分派 `C_*.start`；未知 opcode 在 phase policy 即遭拒絕 | `echo/PacketHandler.java`：`handlePacket` L142；`echo/SessionPacketPolicy.java`：`family/allows` |
| 特殊控制 | C `0xD2` subcode=11：仍要求 `C,S,S` 完整結構，但驗證後忽略，不再登入 | `SessionPacketPolicy.isIgnoredControl`；`PacketHandler.handlePacket`；`C_AutoLogin.start` |
| 可送出性 | 過期的移動／校正／傳送封包可在實際送出前失效；builder 建立不等於送達 | `echo/EncryptExecutor.java`：`encrypt/preparePacket`；`S_MoveCharPacket`、`S_PcPositionCorrection`、`S_Teleport2.isValidForWrite` |

## 2. 型別、欄位長度與記號

| 記號 | bytes／wire 型別 | 實際來源 |
|---|---|---|
| C | 1；u8；read 回傳 0..255 | `ClientBasePacket.readC`／`ServerBasePacket.writeC` |
| H | 2；little-endian；read 回傳 u16 | `readH`／`writeH` |
| CH | 3；little-endian u24 | `ClientBasePacket.readCH` |
| D | 4；little-endian bit pattern；Java reader 使用 **int**，最高位設立時是負數；不能對 count 一律當 u32 | `ClientBasePacket.readD/readUnsigned`／`ServerBasePacket.writeD` |
| F | 8；IEEE-754 double，little-endian | `readF`／`writeF` |
| S | 編碼後 n bytes + 1-byte NUL | reader 使用 `Config.CLIENT_LANGUAGE_CODE`；writer 同設定並套 BAD_TEXT_LIST 過濾。部署編碼值未讀取；不宣稱固定 UTF-8／MS950 |
| Byte（讀） | 剩餘 n bytes，全部消費 | `ClientBasePacket.readByte` |
| Byte（寫） | byte[] 的 n bytes + **1-byte NUL** | `ServerBasePacket.writeByte`；與 readByte 不對稱 |
| Byte1 | byte[] 的 n bytes，不加 NUL | `ServerBasePacket.writeByte1` |
| Boolean | 1；0 或 1 | `ServerBasePacket.writeBoolean` |
| P／L | 各 1；只寫低 8 bits，L **不是 8-byte long** | `ServerBasePacket.writeP/writeL` |
| Exp | 4；long 的低 32 bits | `ServerBasePacket.writeExp`，8-byte 版本是註解 |
| H[依 ClientLogin] | 2；RO 分支寫 dmg，公版寫 0x000A | `ServerBasePacket.damageWordU16` L247；不讀部署設定，不把該 word 一律解作真實 HP 減量 |

兩個 base packet 的完整相對路徑為 `1-伺服器/src/com/lineage/server/clientpackets/ClientBasePacket.java` 與 `1-伺服器/src/com/lineage/server/serverpackets/ServerBasePacket.java`。欄位定長以 wire 寬度計算，Java `int/long` 不決定 wire 長度；S 自訂 Classic helper 常有 UTF-8 長度前綴，須查各 helper，不能套 S 的 NUL 規則。

## 3. Client → Server opcode 全表

來源：`1-伺服器/src/com/lineage/echo/OpcodesClient.java`；實際分派：同目錄 `PacketHandler.handlePacket(byte[])`。下表共 107 個常數（保留同值別名），96 個明確分派分支。所有處理器入口為 `start(byte[], ClientExecutor)`。

「前綴」取自 `ClientPacketStructure.layoutFor(int)`，**僅是最低必要欄位**；完整順序、變長分支與輔助方法見 §6。`—` 表示未定義 layout；`空` 表示不要求固定前綴，不能據此推論沒有 payload。長度含 opcode、不含兩位元組 frame header。

| Hex / 十進位 | 常數（省略 C_OPCODE_） | 處理器／欄位索引 | 最低前綴 | 最低 bytes | 定義行 |
|---|---|---|---|---|---|
| 0x02 / 2 | `TRADE` | [C_Trade.start](#c_trade) | 空 | 1 | 28 |
| 0x03 / 3 | `BOOKMARKDELETE` | [C_DeleteBookmark.start](#c_deletebookmark) | `S` | 2 | 29 |
| 0x04 / 4 | `BUDDYLIST` | [C_Buddy.start](#c_buddy) | 空 | 1 | 30 |
| 0x05 / 5 | `FIGHT` | [C_Fight.start](#c_fight) | 空 | 1 | 31 |
| 0x06 / 6 | `USESKILL` | [C_UseSkill.start](#c_useskill) | `CC` | 3 | 32 |
| 0x07 / 7 | `CHANGECHAR` | [C_NewCharSelect.start](#c_newcharselect) | 空 | 1 | 33 |
| 0x0A / 10 | `BOARD` | [C_Board.start](#c_board) | `D` | 5 | 34 |
| 0x0B / 11 | `AMOUNT` | [C_Amount.start](#c_amount) | `DDCS` | 11 | 35 |
| 0x0D / 13 | `WAREHOUSELOCK` | 未分派；UNKNOWN gate 拒絕 | — | — | 36 |
| 0x0E / 14 | `CLIENTVERSION` | [C_ServerVersion.start](#c_serverversion) | 空 | 1 | 37 |
| 0x10 / 16 | `COMMONCLICK` | [C_CommonClick.start](#c_commonclick) | 空 | 1 | 38 |
| 0x12 / 18 | `EMBLEMUPLOAD` | [C_Emblem.start](#c_emblem) | 空 | 1 | 39 |
| 0x13 / 19 | `TAXRATE` | [C_TaxRate.start](#c_taxrate) | `DC` | 6 | 40 |
| 0x14 / 20 | `SELECTLIST` | [C_SelectList.start](#c_selectlist) | `DD` | 9 | 41 |
| 0x17 / 23 | `BOARDNEXT` | [C_BoardBack.start](#c_boardback) | `DD` | 9 | 42 |
| 0x19 / 25 | `DROPITEM` | [C_DropItem.start](#c_dropitem) | `HHDD` | 13 | 43 |
| 0x1A / 26 | `LOGINTOSERVEROK` | [C_LoginToServerOK.start](#c_logintoserverok) | `CC` | 3 | 44 |
| 0x1D / 29 | `MOVECHAR` | [C_MoveChar.start](#c_movechar) | `HHC` | 6 | 45 |
| 0x1E / 30 | `DUNGEONTELEPORT` | 未分派；UNKNOWN gate 拒絕 | — | — | 46 |
| 0x21 / 33 | `LEAVEPARTY` | [C_LeaveParty.start](#c_leaveparty) | 空 | 1 | 47 |
| 0x22 / 34 | `NPCTALK` | [C_NPCTalk.start](#c_npctalk) | `D` | 5 | 48 |
| 0x23 / 35 | `防掛續存` | 未分派；UNKNOWN gate 拒絕 | — | — | 49 |
| 0x24 / 36 | `GMTELEPORT` | 未分派；UNKNOWN gate 拒絕 | — | — | 50 |
| 0x25 / 37 | `TRADEADDITEM` | [C_TradeAddItem.start](#c_tradeadditem) | `DD` | 9 | 51 |
| 0x26 / 38 | `SHOP` | [C_Shop.start](#c_shop) | `C` | 2 | 52 |
| 0x27 / 39 | `SKILLBUYOK` | [C_SkillBuyOK.start](#c_skillbuyok) | 空 | 1 | 53 |
| 0x28 / 40 | `CHATGLOBAL` | [C_ChatGlobal.start](#c_chatglobal) | `CS` | 3 | 54 |
| 0x29 / 41 | `DOOR` | [C_Door.start](#c_door) | `DHH` | 9 | 55 |
| 0x2B / 43 | `PARTYLIST` | [C_Party.start](#c_party) | 空 | 1 | 56 |
| 0x2C / 44 | `DRAWAL` | [C_Drawal.start](#c_drawal) | `DD` | 9 | 57 |
| 0x2D / 45 | `GIVEITEM` | [C_GiveItem.start](#c_giveitem) | `DHHDD` | 17 | 58 |
| 0x2F / 47 | `PRIVATESHOPLIST` | [C_ShopList.start](#c_shoplist) | `CD` | 6 | 59 |
| 0x32 / 50 | `PROPOSE` | [C_Propose.start](#c_propose) | `C` | 2 | 60 |
| 0x33 / 51 | `CHECKPK` | [C_CheckPK.start](#c_checkpk) | 空 | 1 | 61 |
| 0x34 / 52 | `TELEPORT` | [C_Teleport.start](#c_teleport) | 空 | 1 | 62 |
| 0x38 / 56 | `DEPOSIT` | [C_Deposit.start](#c_deposit) | `DD` | 9 | 63 |
| 0x3D / 61 | `LEAVECLANE` | [C_LeaveClan.start](#c_leaveclan) | 空 | 1 | 64 |
| 0x3E / 62 | `FISHCLICK` | [C_FishClick.start](#c_fishclick) | 空 | 1 | 65 |
| 0x3F / 63 | `RESTARTMENU` | [C_Rank.start](#c_rank) | `C` | 2 | 66 |
| 0x3F / 63 | `RANK` | [C_Rank.start](#c_rank) | `C` | 2 | 67 |
| 0x44 / 68 | `PLEDGE` | [C_Pledge.start](#c_pledge) | 空 | 1 | 68 |
| 0x45 / 69 | `BANCLAN` | [C_BanClan.start](#c_banclan) | `S` | 2 | 69 |
| 0x47 / 71 | `TRADEADDOK` | [C_TradeOK.start](#c_tradeok) | 空 | 1 | 70 |
| 0x48 / 72 | `CLAN` | [C_Clan.start](#c_clan) | `D` | 5 | 71 |
| 0x4C / 76 | `CLANMATCHING` | [C_PledgeRecommendation.start](#c_pledgerecommendation) | `C` | 2 | 72 |
| 0x4E / 78 | `PLEDGECONTENT` | [C_PledgeContent.start](#c_pledgecontent) | `C` | 2 | 73 |
| 0x54 / 84 | `NEWCHAR` | [C_CreateChar.start](#c_createchar) | `SCCCCCCCC` | 10 | 74 |
| 0x56 / 86 | `TRADEADDCANCEL` | [C_TradeCancel.start](#c_tradecancel) | 空 | 1 | 75 |
| 0x57 / 87 | `MAIL` | [C_Mail.start](#c_mail) | `C` | 2 | 76 |
| 0x5A / 90 | `TITLE` | [C_Title.start](#c_title) | `SS` | 3 | 77 |
| 0x5F / 95 | `KEEPALIVE` | [C_KeepALIVE.start](#c_keepalive) | 空 | 1 | 78 |
| 0x62 / 98 | `CHARRESET` | [C_CharReset.start](#c_charreset) | `C` | 2 | 79 |
| 0x63 / 99 | `CHECK` | [C_Check.start](#c_check) | `D` | 5 | 80 |
| 0x67 / 103 | `PETMENU` | [C_PetMenu.start](#c_petmenu) | `D` | 5 | 81 |
| 0x68 / 104 | `USEPETITEM` | [C_UsePetItem.start](#c_usepetitem) | `CDC` | 7 | 82 |
| 0x70 / 112 | `PICKUPITEM` | [C_PickUpItem.start](#c_pickupitem) | `HHDD` | 13 | 83 |
| 0x72 / 114 | `BOARDREAD` | [C_BoardRead.start](#c_boardread) | `DD` | 9 | 84 |
| 0x76 / 118 | `FIX_WEAPON_LIST` | [C_FixWeaponList.start](#c_fixweaponlist) | 空 | 1 | 85 |
| 0x77 / 119 | `LOGINPACKET` | [C_AuthLogin.start](#c_authlogin) | `SS` | 3 | 86 |
| 0x78 / 120 | `EXTCOMMAND` | [C_ExtraCommand.start](#c_extracommand) | `C` | 2 | 87 |
| 0x79 / 121 | `ATTR` | [C_Attr.start](#c_attr) | 空 | 1 | 88 |
| 0x7A / 122 | `QUITGAME` | [C_Disconnect.start](#c_disconnect) | 空 | 1 | 89 |
| 0x7B / 123 | `ARROWATTACK` | [C_AttackBow.start](#c_attackbow) | `DHH` | 9 | 90 |
| 0x7D / 125 | `NPCACTION` | [C_NPCAction.start](#c_npcaction) | `DS` | 6 | 91 |
| 0x80 / 128 | `CASTLESECURITY` | 未分派；UNKNOWN gate 拒絕 | — | — | 92 |
| 0x81 / 129 | `CLANATTENTION` | [C_ClanAttention.start](#c_clanattention) | `C` | 2 | 93 |
| 0x88 / 136 | `CHAT` | [C_Chat.start](#c_chat) | `CS` | 3 | 94 |
| 0x89 / 137 | `LOGINTOSERVER` | [C_LoginToServer.start](#c_logintoserver) | `S` | 2 | 95 |
| 0x8A / 138 | `DELETEINVENTORYITEM` | [C_DeleteInventoryItem.start](#c_deleteinventoryitem) | `D` | 5 | 96 |
| 0x8D / 141 | `BOARDWRITE` | [C_BoardWrite.start](#c_boardwrite) | `DSS` | 7 | 97 |
| 0x91 / 145 | `SKILLBUY` | [C_SkillBuy.start](#c_skillbuy) | `D` | 5 | 98 |
| 0x99 / 153 | `BOARDDELETE` | [C_BoardDelete.start](#c_boarddelete) | `DD` | 9 | 99 |
| 0xA1 / 161 | `RESULT` | [C_Result.start](#c_result) | 空 | 1 | 100 |
| 0xA2 / 162 | `DELETECHAR` | [C_DeleteChar.start](#c_deletechar) | `S` | 2 | 101 |
| 0xA4 / 164 | `USEITEM` | [C_ItemUSe.start](#c_itemuse) | `D` | 5 | 102 |
| 0xA5 / 165 | `BOOKMARK` | [C_AddBookmark.start](#c_addbookmark) | `S` | 2 | 103 |
| 0xAB / 171 | `EXCLUDE` | [C_Exclude.start](#c_exclude) | `S` | 2 | 104 |
| 0xAD / 173 | `EXIT_GHOST` | [C_ExitGhost.start](#c_exitghost) | 空 | 1 | 105 |
| 0xB1 / 177 | `RESTART` | [C_Restart.start](#c_restart) | 空 | 1 | 106 |
| 0xB8 / 184 | `CHATWHISPER` | [C_ChatWhisper.start](#c_chatwhisper) | `SS` | 3 | 107 |
| 0xB9 / 185 | `CALL` | [C_CallPlayer.start](#c_callplayer) | `S` | 2 | 108 |
| 0xBF / 191 | `SKILLBUYOKITEM` | [C_SkillBuyItemOK.start](#c_skillbuyitemok) | 空 | 1 | 109 |
| 0xC2 / 194 | `JOINCLAN` | [C_JoinClan.start](#c_joinclan) | 空 | 1 | 110 |
| 0xC7 / 199 | `CAHTPARTY` | [C_ChatParty.start](#c_chatparty) | `C` | 2 | 111 |
| 0xCA / 202 | `DELBUDDY` | [C_DelBuddy.start](#c_delbuddy) | `S` | 2 | 112 |
| 0xCE / 206 | `WHO` | [C_Who.start](#c_who) | `S` | 2 | 113 |
| 0xCF / 207 | `ADDBUDDY` | [C_AddBuddy.start](#c_addbuddy) | `S` | 2 | 114 |
| 0xD0 / 208 | `WEB_WINDOW` | [C_WebWindow.start](#c_webwindow) | 空 | 1 | 115 |
| 0xD2 / 210 | `BEANFUNLOGINPACKET` | [C_AutoLogin.start](#c_autologin) | `CSS` | 4 | 116 |
| 0xD3 / 211 | `CUSTOM_RESERVED_211` | 未分派；UNKNOWN gate 拒絕 | — | — | 117 |
| 0xD4 / 212 | `CUSTOM_RESERVED_212` | 未分派；UNKNOWN gate 拒絕 | — | — | 118 |
| 0xD5 / 213 | `CUSTOM_RESERVED_213` | 未分派；UNKNOWN gate 拒絕 | — | — | 119 |
| 0xD6 / 214 | `CUSTOM_RESERVED_214` | 未分派；UNKNOWN gate 拒絕 | — | — | 120 |
| 0xDB / 219 | `ENTERPORTAL` | [C_EnterPortal.start](#c_enterportal) | `HH` | 5 | 121 |
| 0xDE / 222 | `CREATECLAN` | [C_CreateClan.start](#c_createclan) | `S` | 2 | 122 |
| 0xDF / 223 | `SELECTTARGET` | [C_SelectTarget.start](#c_selecttarget) | `DCD` | 10 | 123 |
| 0xE1 / 225 | `CHANGEHEADING` | [C_ChangeHeading.start](#c_changeheading) | `C` | 2 | 124 |
| 0xE3 / 227 | `WAR` | [C_War.start](#c_war) | `CS` | 3 | 125 |
| 0xE5 / 229 | `ATTACK` | [C_Attack.start](#c_attack) | `DHH` | 9 | 126 |
| 0xE6 / 230 | `CREATEPARTY` | [C_CreateParty.start](#c_createparty) | `C` | 2 | 127 |
| 0xE7 / 231 | `SHIP` | [C_Ship.start](#c_ship) | `HHH` | 7 | 128 |
| 0xF4 / 244 | `CHARACTERCONFIG` | [C_CharcterConfig.start](#c_charcterconfig) | 空 | 1 | 129 |
| 0xF5 / 245 | `SKILLBUYITEM` | [C_SkillBuyItem.start](#c_skillbuyitem) | `D` | 5 | 130 |
| 0xF6 / 246 | `TELEPORTLOCK` | [C_UnLock.start](#c_unlock) | `C` | 2 | 131 |
| 0xFD / 253 | `SMS` | 未分派；UNKNOWN gate 拒絕 | — | — | 132 |
| 0xFE / 254 | `SENDLOCATION` | [C_Windows.start](#c_windows) | `C` | 2 | 133 |
| 0xFF / 255 | `BANPARTY` | [C_BanParty.start](#c_banparty) | `S` | 2 | 134 |

## 4. Server → Client opcode 全表

來源：`1-伺服器/src/com/lineage/server/serverpackets/OpcodesServer.java`。共 112 個常數；同值別名分列。S 封包無 inbound handler：下列為建構／序列化類別，其多載建構子、builder 與欄位順序見 §6；輸出經 `getContent()` → `prepareForWrite()` → `L1CipherEncoder.encode(...)`。類別可能寫入多種 opcode，必須再選方法與分支。

| Hex / 十進位 | 常數（省略 S_OPCODE_） | 直接引用的 builder 類別 | 定義行 |
|---|---|---|---|
| 0x00 / 0 | `CLANMATCHING` | [S_PacketBoxPledgeRecommend](#s_packetboxpledgerecommend) | 16 |
| 0x04 / 4 | `DEPOSIT` | [S_Deposit](#s_deposit) | 18 |
| 0x05 / 5 | `INVLIST` | [S_InvList](#s_invlist) | 19 |
| 0x06 / 6 | `DETELECHAROK` | [S_DeleteCharOK](#s_deletecharok) | 20 |
| 0x08 / 8 | `OWNCHARSTATUS` | [S_OwnCharStatus](#s_owncharstatus) | 22 |
| 0x0A / 10 | `MOVEOBJECT` | [S_MoveCharPacket](#s_movecharpacket)、[S_MoveNpcPacket](#s_movenpcpacket) | 24 |
| 0x0B / 11 | `TRUETARGET` | [S_TrueTarget](#s_truetarget) | 25 |
| 0x0D / 13 | `SKILLBUYITEM` | [S_SkillBuyItem](#s_skillbuyitem)、[S_SkillBuyItemCN](#s_skillbuyitemcn) | 27 |
| 0x0F / 15 | `ADDITEM` | [S_AddItem](#s_additem) | 29 |
| 0x16 / 22 | `SOUND` | [S_Sound](#s_sound) | 31 |
| 0x18 / 24 | `ITEMSTATUS` | 未找到直接引用；不得因名稱推定支援 | 33 |
| 0x18 / 24 | `ITEMAMOUNT` | [S_ItemAmount](#s_itemamount)、[S_ItemStatus](#s_itemstatus) | 34 |
| 0x1E / 30 | `ATTACKPACKET` | [S_AttackPacketNpc](#s_attackpacketnpc)、[S_AttackPacketPc](#s_attackpacketpc)、[S_UseArrowSkill](#s_usearrowskill)、[S_UseAttackSkill](#s_useattackskill) | 36 |
| 0x21 / 33 | `MPUPDATE` | [S_MPUpdate](#s_mpupdate) | 38 |
| 0x22 / 34 | `LAWFUL` | [S_Lawful](#s_lawful) | 39 |
| 0x23 / 35 | `TRADEADDITEM` | [S_TradeAddItem](#s_tradeadditem) | 40 |
| 0x24 / 36 | `ABILITY` | [S_Ability](#s_ability) | 41 |
| 0x25 / 37 | `SPMR` | [S_SPMR](#s_spmr) | 42 |
| 0x27 / 39 | `SHOWHTML` | [S_AuctionBoardRead](#s_auctionboardread)、[S_Bonusstats](#s_bonusstats)、[S_Buddy](#s_buddy)、[S_CloseList](#s_closelist)、[S_GMHtml](#s_gmhtml)、[S_NoSell](#s_nosell)、[S_NPCTalkActionTPUrl](#s_npctalkactiontpurl)、[S_NPCTalkReturn](#s_npctalkreturn)、[S_Party](#s_party)、[S_PetMenuPacket](#s_petmenupacket)、[S_Pledge](#s_pledge)、[S_ShowPolyList](#s_showpolylist)、[S_ShowSummonList](#s_showsummonlist)、[S_SkillBuyCN](#s_skillbuycn) | 44 |
| 0x28 / 40 | `LIGHT` | [S_Light](#s_light) | 45 |
| 0x29 / 41 | `SKILLBUY` | [S_SkillBuy](#s_skillbuy)、[S_SkillBuyCN](#s_skillbuycn) | 46 |
| 0x2A / 42 | `RANGESKILLS` | [S_RangeSkill](#s_rangeskill) | 47 |
| 0x2E / 46 | `CHANGENAME` | [S_ChangeName](#s_changename) | 49 |
| 0x2F / 47 | `CURSEBLIND` | [S_CurseBlind](#s_curseblind) | 50 |
| 0x30 / 48 | `COMMONNEWS` | [S_CommonInfo](#s_commoninfo)、[S_CommonNews](#s_commonnews) | 51 |
| 0x34 / 52 | `TRADE` | [S_Trade](#s_trade) | 53 |
| 0x37 / 55 | `SKILLSOUNDGFX` | [S_SkillSound](#s_skillsound) | 55 |
| 0x39 / 57 | `DELETEINVENTORYITEM` | [S_DeleteInventoryItem](#s_deleteinventoryitem) | 57 |
| 0x3C / 60 | `PINKNAME` | [S_PinkName](#s_pinkname) | 59 |
| 0x40 / 64 | `CHARSYNACK` | 未找到直接引用；不得因名稱推定支援 | 61 |
| 0x40 / 64 | `CHARRESET` | [S_BookmarkList](#s_bookmarklist)、[S_CharReset](#s_charreset)、[S_CharResetInfo](#s_charresetinfo)、[S_EquipmentWindow](#s_equipmentwindow)、[S_InitialAbilityGrowth](#s_initialabilitygrowth)、[S_PetCtrlMenu](#s_petctrlmenu)、[S_Unknown_B](#s_unknown_b) | 62 |
| 0x41 / 65 | `SHOWSHOPSELLLIST` | [S_ShopBuyList](#s_shopbuylist)、[S_ShopBuyListAll](#s_shopbuylistall)、[S_ShopBuyListGam](#s_shopbuylistgam)、[S_ShopBuyListLotto](#s_shopbuylistlotto) | 63 |
| 0x43 / 67 | `SKILLBRAVE` | [S_SkillBrave](#s_skillbrave) | 65 |
| 0x44 / 68 | `BOARD` | [S_Board](#s_board) | 66 |
| 0x45 / 69 | `CASTLEMASTER` | [S_CastleMaster](#s_castlemaster) | 67 |
| 0x46 / 70 | `SHOWSHOPBUYLIST` | [S_AuctionShopSellList](#s_auctionshopselllist)、[S_BoxChoice](#s_boxchoice)、[S_ShopSellList](#s_shopselllist)、[S_ShopSellListCn](#s_shopselllistcn)、[S_ShopSellListCnX](#s_shopselllistcnx)、[S_ShopSellListGam](#s_shopselllistgam)、[S_ShopSellListLotto](#s_shopselllistlotto)、[S_ShopSellListTmp](#s_shopselllisttmp)、[S_ShopSellListX1](#s_shopselllistx1) | 68 |
| 0x47 / 71 | `SERVERMSG` | [S_ServerMessage](#s_servermessage)、[S_WhoAmount](#s_whoamount)、[S_WhoCharinfo](#s_whocharinfo) | 69 |
| 0x48 / 72 | `UPDATECLANID` | [S_ClanUpdate](#s_clanupdate) | 70 |
| 0x49 / 73 | `WHISPERCHAT` | [S_ChatWhisperFrom](#s_chatwhisperfrom) | 71 |
| 0x4C / 76 | `POLY` | [S_ChangeShape](#s_changeshape) | 73 |
| 0x51 / 81 | `NORMALCHAT` | [S_Chat](#s_chat)、[S_ChatParty2](#s_chatparty2)、[S_ChatShouting](#s_chatshouting) | 75 |
| 0x53 / 83 | `SELECTLIST` | [S_FixWeaponList](#s_fixweaponlist)、[S_PetList](#s_petlist) | 77 |
| 0x54 / 84 | `WAR` | [S_War](#s_war) | 78 |
| 0x55 / 85 | `RESURRECTION` | [S_Resurrection](#s_resurrection) | 79 |
| 0x57 / 87 | `CHARPACK` | [S_BoxPack](#s_boxpack)、[S_DoorPack](#s_doorpack)、[S_FollowerPack](#s_followerpack)、[S_NPCPack](#s_npcpack)、[S_NPCPack_D](#s_npcpack_d)、[S_NPCPack_De](#s_npcpack_de)、[S_NPCPack_Doll](#s_npcpack_doll)、[S_NPCPack_Eff](#s_npcpack_eff)、[S_NPCPack_F](#s_npcpack_f)、[S_NPCPack_Ill](#s_npcpack_ill)、[S_NPCPack_Item](#s_npcpack_item)、[S_NPCPack_M](#s_npcpack_m)、[S_NPCPack_Pet](#s_npcpack_pet)、[S_NPCPack_Signboard](#s_npcpack_signboard)、[S_NPCPack_Summon](#s_npcpack_summon)、[S_OtherCharPacks](#s_othercharpacks)、[S_OwnCharPack](#s_owncharpack)、[S_Trap](#s_trap)、[S_PcPositionCorrection](#s_pcpositioncorrection) | 81 |
| 0x57 / 87 | `DROPITEM` | 未找到直接引用；不得因名稱推定支援 | 82 |
| 0x5C / 92 | `BOOKMARKS` | [S_Bookmarks](#s_bookmarks) | 84 |
| 0x5D / 93 | `CHARLIST` | [S_CharPacks](#s_charpacks) | 85 |
| 0x48 / 72 | `NEWMASTER` | [S_NewMaster](#s_newmaster) | 87 |
| 0x62 / 98 | `NEWCHARWRONG` | [S_CharCreateStatus](#s_charcreatestatus) | 88 |
| 0x64 / 100 | `ITEMNAME` | [S_ItemName](#s_itemname) | 90 |
| 0x67 / 103 | `LIQUOR` | [S_Liquor](#s_liquor) | 92 |
| 0x69 / 105 | `REDMESSAGE` | [S_BlueMessage](#s_bluemessage) | 94 |
| 0x6A / 106 | `EFFECTLOCATION` | [S_EffectLocation](#s_effectlocation) | 95 |
| 0x70 / 112 | `TRADESTATUS` | [S_TradeStatus](#s_tradestatus) | 97 |
| 0x71 / 113 | `EXP` | [S_Exp](#s_exp) | 98 |
| 0x73 / 115 | `WEATHER` | [S_Weather](#s_weather) | 100 |
| 0x75 / 117 | `PUTSOLDIER` | 未找到直接引用；不得因名稱推定支援 | 102 |
| 0x76 / 118 | `EMBLEM` | [S_Emblem](#s_emblem) | 103 |
| 0x77 / 119 | `CHARVISUALUPDATE` | [S_CharVisualUpdate](#s_charvisualupdate) | 104 |
| 0x78 / 120 | `REMOVE_OBJECT` | [S_RemoveObject](#s_removeobject) | 105 |
| 0x7A / 122 | `CHANGEHEADING` | [S_ChangeHeading](#s_changeheading) | 107 |
| 0x7B / 123 | `GAMETIME` | [S_GameTime](#s_gametime) | 108 |
| 0x7E / 126 | `BLESSOFEVA` | [S_SkillIconBlessOfEva](#s_skilliconblessofeva) | 110 |
| 0x7F / 127 | `NEWCHARPACK` | [S_NewCharPacket](#s_newcharpacket) | 111 |
| 0x88 / 136 | `INPUTAMOUNT` | [S_ApplyAuction](#s_applyauction)、[S_HowManyMake](#s_howmanymake)、[S_ItemCount](#s_itemcount)、[S_SellHouse](#s_sellhouse) | 113 |
| 0x8B / 139 | `SERVERVERSION` | [S_ServerVersion](#s_serverversion) | 115 |
| 0x8C / 140 | `PRIVATESHOPLIST` | [S_PrivateShop](#s_privateshop) | 116 |
| 0x8D / 141 | `DRAWAL` | [S_Drawal](#s_drawal) | 117 |
| 0x94 / 148 | `BOARDREAD` | [S_BoardRead](#s_boardread) | 119 |
| 0x95 / 149 | `TELEPORT` | [S_Teleport](#s_teleport)、[S_Teleport2](#s_teleport2) | 120 |
| 0x96 / 150 | `INITPACKET` | L1HandshakeHandler.channelActive（§1） | 121 |
| 0x9B / 155 | `OWNCHARSTATUS2` | [S_OwnCharStatus2](#s_owncharstatus2) | 123 |
| 0x9C / 156 | `HOUSELIST` | [S_AuctionBoard](#s_auctionboard) | 124 |
| 0x9E / 158 | `DOACTIONGFX` | [S_DoActionGFX](#s_doactiongfx)、[S_DoActionShop](#s_doactionshop)、[S_Fishing](#s_fishing) | 126 |
| 0xA0 / 160 | `DELSKILL` | [S_DelSkill](#s_delskill) | 128 |
| 0xA1 / 161 | `NPCSHOUT` | [S_BoxMessage](#s_boxmessage)、[S_HelpMessage](#s_helpmessage)、[S_KillMessage](#s_killmessage)、[S_NpcChat](#s_npcchat)、[S_NpcChatGlobal](#s_npcchatglobal)、[S_NpcChatShouting](#s_npcchatshouting)、[S_ServerMessage](#s_servermessage)、[S_SystemMessage](#s_systemmessage)、[S_ToGmMessage](#s_togmmessage) | 129 |
| 0xA4 / 164 | `ADDSKILL` | [S_AddSkill](#s_addskill) | 131 |
| 0xA5 / 165 | `POISON` | [S_Poison](#s_poison) | 132 |
| 0xA6 / 166 | `STRUP` | [S_Strup](#s_strup) | 133 |
| 0xAB / 171 | `INVIS` | [S_Invis](#s_invis) | 135 |
| 0xAE / 174 | `OWNCHARATTRDEF` | [S_OwnCharAttrDef](#s_owncharattrdef) | 137 |
| 0xB0 / 176 | `SHOWRETRIEVELIST` | [S_AuctionShopRetrieve](#s_auctionshopretrieve)、[S_PetInventory](#s_petinventory)、[S_PowerItemList](#s_poweritemlist)、[S_RetrieveChaList](#s_retrievechalist)、[S_RetrieveDevilKingList](#s_retrievedevilkinglist)、[S_RetrieveElfList](#s_retrieveelflist)、[S_RetrieveList](#s_retrievelist)、[S_RetrievePledgeList](#s_retrievepledgelist)、[S_RetrievePowerItem](#s_retrievepoweritem)、[S_RetrieveShiftingList](#s_retrieveshiftinglist) | 139 |
| 0xB2 / 178 | `CHARAMOUNT` | [S_CharAmount](#s_charamount) | 141 |
| 0xB7 / 183 | `CHARTITLE` | [S_CharTitle](#s_chartitle) | 143 |
| 0xB9 / 185 | `TAXRATE` | [S_TaxRate](#s_taxrate) | 145 |
| 0xBA / 186 | `MAIL` | [S_Mail](#s_mail) | 147 |
| 0xBB / 187 | `HOUSEMAP` | [S_HouseMap](#s_housemap) | 148 |
| 0xBC / 188 | `DEXUP` | [S_Dexup](#s_dexup) | 149 |
| 0xC8 / 200 | `CLANATTENTION` | [S_ClanAttention](#s_clanattention) | 151 |
| 0xCA / 202 | `PARALYSIS` | [S_Paralysis](#s_paralysis) | 153 |
| 0xCE / 206 | `MAPID` | [S_MapID](#s_mapid)、[S_PcPositionMapReset](#s_pcpositionmapreset) | 155 |
| 0xCE / 206 | `UNDERWATER` | 未找到直接引用；不得因名稱推定支援 | 156 |
| 0xD1 / 209 | `ATTRIBUTE` | [S_Door](#s_door) | 158 |
| 0xD8 / 216 | `SKILLICONSHIELD` | [S_SkillIconShield](#s_skilliconshield) | 160 |
| 0xDB / 219 | `YES_NO` | [S_Message_YN](#s_message_yn) | 162 |
| 0xDF / 223 | `LOGINTOGAME` | [S_EnterGame](#s_entergame) | 164 |
| 0xE1 / 225 | `HPUPDATE` | [S_HPUpdate](#s_hpupdate) | 166 |
| 0xE3 / 227 | `DISCONNECT` | [S_Disconnect](#s_disconnect) | 168 |
| 0xE6 / 230 | `SPOLY` | 未找到直接引用；不得因名稱推定支援 | 170 |
| 0xE9 / 233 | `LOGINRESULT` | [S_LoginResult](#s_loginresult) | 172 |
| 0xEC / 236 | `SELECTTARGET` | [S_SelectTarget](#s_selecttarget) | 174 |
| 0xED / 237 | `HPMETER` | [S_HPMeter](#s_hpmeter) | 175 |
| 0xF0 / 240 | `ITEMCOLOR` | [S_ItemColor](#s_itemcolor) | 177 |
| 0xF1 / 241 | `MOVELOCK` | [S_Lock](#s_lock) | 178 |
| 0xF3 / 243 | `GLOBALCHAT` | [S_ChatClan](#s_chatclan)、[S_ChatClanUnion](#s_chatclanunion)、[S_ChatGlobal](#s_chatglobal)、[S_ChatParty](#s_chatparty)、[S_ChatTransaction](#s_chattransaction)、[S_ChatWhisperTo](#s_chatwhisperto)、[S_SystemMessage](#s_systemmessage) | 180 |
| 0xF5 / 245 | `IDENTIFYDESC` | [S_IdentifyDesc](#s_identifydesc) | 182 |
| 0xFA / 250 | `PACKETBOX` | [S_BroadcastPos](#s_broadcastpos)、[S_ClassicBadge](#s_classicbadge)、[S_ClassicBagCombine](#s_classicbagcombine)、[S_ClassicBoss](#s_classicboss)、[S_ClassicCollection](#s_classiccollection)、[S_ClassicCraft](#s_classiccraft)、[S_ClassicInventoryFilter](#s_classicinventoryfilter)、[S_ClassicPass](#s_classicpass)、[S_ClassicQuest](#s_classicquest)、[S_ClassicRanking](#s_classicranking)、[S_ClassicReward](#s_classicreward)、[S_GlobalEffect](#s_globaleffect)、[S_IconConfig](#s_iconconfig)、[S_Icons](#s_icons)、[S_ItemBoard](#s_itemboard)、[S_Karma](#s_karma)、[S_MagicEquipment](#s_magicequipment)、[S_PacketBox](#s_packetbox)、[S_PacketBoxActiveSpells](#s_packetboxactivespells)、[S_PacketBoxConfig](#s_packetboxconfig)、[S_PacketBoxCooking](#s_packetboxcooking)、[S_PacketBoxDk](#s_packetboxdk)、[S_PacketBoxExp](#s_packetboxexp)、[S_PacketBoxGame](#s_packetboxgame)、[S_PacketBoxGm](#s_packetboxgm)、[S_PacketBoxGree](#s_packetboxgree)、[S_PacketBoxHpMsg](#s_packetboxhpmsg)、[S_PacketBoxIcon1](#s_packetboxicon1)、[S_PacketBoxIcon2](#s_packetboxicon2)、[S_PacketBoxIconAura](#s_packetboxiconaura)、[S_PacketBoxItemLv](#s_packetboxitemlv)、[S_PacketBoxLoc](#s_packetboxloc)、[S_PacketBoxMapTimer](#s_packetboxmaptimer)、[S_PacketBoxParty](#s_packetboxparty)、[S_PacketBoxProtection](#s_packetboxprotection)、[S_PacketBoxSelect](#s_packetboxselect)、[S_PacketBoxTest](#s_packetboxtest)、[S_PacketBoxThirdSpeed](#s_packetboxthirdspeed)、[S_PacketBoxWar](#s_packetboxwar)、[S_PacketBoxWaterLife](#s_packetboxwaterlife)、[S_PacketBoxWindShackle](#s_packetboxwindshackle)、[S_PacketBoxWisdomPotion](#s_packetboxwisdompotion)、[S_PetEquipment](#s_petequipment)、[S_Pledge](#s_pledge)、[S_PledgeWarehouseHistory](#s_pledgewarehousehistory)、[S_ShowDrop](#s_showdrop)、[S_WeaponRange](#s_weaponrange) | 184 |
| 0xFA / 250 | `ACTIVESPELLS` | 未找到直接引用；不得因名稱推定支援 | 185 |
| 0xFA / 250 | `SKILLICONGFX` | 未找到直接引用；不得因名稱推定支援 | 186 |
| 0xFA / 250 | `UNKNOWN2` | 未找到直接引用；不得因名稱推定支援 | 187 |
| 0xFF / 255 | `SKILLHASTE` | [S_SkillHaste](#s_skillhaste) | 189 |

### 4.1 常數表外的實際寫入與不完整舊 builder

| 首 byte／類型 | builder 與來源結論 |
|---|---|
| 0xF2 subcode=1 | `S_ItemStatus.writeLongStatus`：C(F2),C(1),D(itemId),S(viewName),D(count),H(status.length),raw[status.length]；不在 OpcodesServer 常數表，但已有寫入 |
| 0xF2 subcode=2 | `S_HitReactionSuppressList()`：C(F2),C(2),H(count),count×H(gfxId)；同類 CUSTOM_OPCODE/SUBOPCODE 常數。由登入RO分支發送 |
| 0xA7／0xAA | `S_Pledge(int ClanId)`／`S_Pledge(L1Clan)` 直接以 HTML_PLEDGE_ANNOUNCE=167／MEMBERS=170 為首byte；原本opcode寫入被註解。按實際來源列記，**不能自行補0xFA**；是否活動使用／相容仍待驗證 |
| caller supplied | `S_EffectLocation(int opid,L1Location)`、`S_PacketBoxItemLv(int opid)`、`S_Resurrection(L1PcInstance,int opid,int type)`、`S_WarTime(int op)` 由參數決定首byte；不是固定opcode定義 |
| raw test | `S_PacketBoxTest(byte[])` 逐byte輸出傳入資料；不推定固定opcode |
| 無固定正常header | `S_RedMessage.buildPacket` 首先寫S(acc)，`S_WarTime(Calendar)` 首先寫H(6)，其opcode行被註解；這些舊builder不能作正常封包範本 |

以上相對來源皆為 `1-伺服器/src/com/lineage/server/serverpackets/<類別>.java`；完整後續欄位見 §6。常數有定義、builder存在、實際會被目前狀態路徑送出，是三件不同的事。

## 5. 常用欄位與變長分支速查

### 5.1 核心 C 封包（不含共同 opcode）

此表長度含 1-byte opcode、未含 frame header。來源均為 `1-伺服器/src/com/lineage/server/clientpackets/<類別>.java` 的 `start(byte[], ClientExecutor)`，另註方法者除外。

| Opcode／類別 | 按 wire 順序的欄位 | 邏輯長度與注意事項 |
|---|---|---|
| 0x0E C_ServerVersion | 不消費 payload 欄位 | 最低 1；回覆 S_ServerVersion，不可把 client 版本欄位由註解補出 |
| 0x77 C_AuthLogin | S(loginName), S(password) | 1+B(name)+1+B(password)+1；轉 `checkLogin` |
| 0xD2 C_AutoLogin | C(subcode), S(loginName), S(password) | 最低 4；subcode=11 為忽略控制，其餘轉 `checkLogin(...,true)` |
| 0x10 C_CommonClick | 不消費 payload | `start` 發角色數與清單 |
| 0x89 C_LoginToServer | S(charName) | 選取角色名稱，不是 objId；帳號歸屬由伺服器檢查 |
| 0x54 C_CreateChar | S(name), C(type), C(sex), C(str), C(dex), C(con), C(wis), C(cha), C(int) | 9+B(name)+1；完整規則由 handler 檢查 |
| 0x1A C_LoginToServerOK | C(type), C(button) | 3；實際為聊天顯示／密語接收設定，**不是進世界 ACK** |
| 0x1D C_MoveChar | H(locx), H(locy), C(heading) | 6；heading 0..7；client 座標是基準觀測，落點由伺服器基準加方向 |
| 0xE1 C_ChangeHeading | C(heading) | 2 |
| 0xE5 C_Attack／0x7B C_AttackBow | D(targetId), H(locx), H(locy)，C_Attack 尚有剩餘時另讀 C(未使用尾碼)，C_AttackBow 不讀該尾碼 | 必要 9，可有尾碼；不攜帶 client 決定的傷害 |
| 0x19 C_DropItem／0x70 C_PickUpItem | H(x), H(y), D(itemObjectId), D(count) | 13；instance ID，不是 item template；負 count 不可視為巨大合法數量 |
| 0xA4 C_ItemUSe | D(itemObjid), use_type 尾段 | 最低 5；解析在 `useItemRequest`；type2/use_type 與 classname 取自擁有的物品 |
| 0x88 C_Chat／0x28 C_ChatGlobal | C(chatType), S(chatText) | 2+B(text)+1；依頻道規則路由 |
| 0xB8 C_ChatWhisper | S(targetName), S(chatText) | 1+B(name)+1+B(text)+1 |
| 0x06 C_UseSkill | C(row), C(column), skill 尾段 | 最低 3；skillId=(row×8)+column+1 |
| 0xDB C_EnterPortal | H(locx), H(locy) | 5；入口解析由伺服器控制 |
| 0x34 C_Teleport | 不讀 payload | 最低 1；只對當前 pending teleport 接受 client ACK |
| 0xF6 C_UnLock | C(type) | 2；與 C_Teleport 不同 |
| 0xF4 C_CharcterConfig | D(declared), 剩餘 byte[] | declared≥3 且 declared−3≤剩餘 bytes；不能把 declared 當 frame total |
| 0xD0 C_WebWindow | 不讀取封包 body | `start` 依當前玩家開啟 UI；不從 opcode 名稱臆造 URL 欄位 |

### 5.2 C_USESKILL 尾段

來源：`ClientPacketStructure.validateSkill` L180 與 `C_UseSkill.start` L198 起；兩檔均在上述 clientpackets 目錄。`decrypt.length≤4` 時只讀 row/column，最多一個未解讀尾 byte；大於 4 才走下表。以下各長度是尾段 bytes，必須再加前綴 3。

| skillId | 尾段順序 | 尾段長度 |
|---|---|---|
| 116、118 | S(charName) | B(name)+1 |
| 113 | D(targetId), H(targetX), H(targetY), S(trueTargetText) | 8+B(text)+1 |
| 5、69 | H(mapId；讀後未使用), D(bookmarkId) | 6 |
| 58、63 | H(targetX), H(targetY) | 4 |
| 4,6,7,10,15,16,17,18,22,25,28,30,34,38,45,46,50,65,74,77,108,132,184,187,192,194,202,203,207,208,213 | D(targetId), H(targetX), H(targetY) | 8 |
| 51 | D(summonId／未裝召喚戒時僅消費) | 4 |
| 其他 | D(targetId) | 4；88／91 等另有裝備 gate；target=none 不當世界物件驗證 |

### 5.3 C_USEITEM 尾段

來源：`ClientPacketStructure.validateItemArguments(byte[],int)` L196、`C_ItemUSe.useItemRequest`。已解析擁有的 item 後，具有 classname 的 wire 請求才進此 preflight；無 classname 的執行分支不得憑表推定可用。

| use_type | D(itemObjid) 後的順序 | 尾段 bytes |
|---|---|---|
| -8 | C(data0), C(data1) | 2 |
| 5、17、39 | D(targetObjId), H(x), H(y) | 8 |
| 6、29 | H(mapId), D(target/bookmarkObjId) | 6 |
| 7、8、14、26、27、30、46、55 | D(targetObjId) | 4 |
| 16 | S(polyCommand) | B(command)+1 |
| 28 | C(selection) | 1 |
| 12、31、33、35 | H(data0), S(text), Byte(剩餘信件內容) | 2+B(text)+1+n |
| 42 | H(x), H(y) | 4 |
| 其餘 | preflight 不要求額外欄位 | 0；仍須依實際 item executor 判斷 |

### 5.4 其他 C 變長前綴補充

來源：`ClientPacketStructure.validate` L17、`validateAttr` L213；實際欄位名稱再查 §6 對應 handler。所有序列均為 opcode 後；count×width 必須在剩餘 bytes 內，禁止以宣告 count 直接配置無界記憶體。

| 封包 | 分支／長度 |
|---|---|
| C_SHOP 0x26 | C(type)；type=0 後接 H(sellCount), 每筆12 bytes，再 H(buyCount), 每筆12 bytes；餘下店名為 opaque bytes |
| C_SKILLBUYOK 0x27／C_SKILLBUYOKITEM 0xBF | H(count) + count×D；preflight 的空前綴不等於空包 |
| C_RESULT 0xA1 | D(objectId), C(type), C(count), C(保留)；每筆通常8 bytes；type=1 且目標 L1DeInstance 時7 bytes；私人收購 CH(count)+C(order) 仍為8-byte record |
| C_CHARRESET 0x62 | C(mode)；mode=1/3 追加6 bytes；mode=2 追加 C，該值8時再 C |
| C_CREATEPARTY 0xE6 | C(mode)；0/1/3 追加 D，2 追加 S |
| C_CAHTPARTY 0xC7 | C(mode)；0 追加 S |
| C_RANK 0x3F | C(mode)；1 追加 C(rank), S(name)；其餘不讀階級姓名 |
| C_CLANATTENTION 0x81 | C(mode)；餘長>2 才驗證 S；恰2 bytes 可為原生 tags |
| C_MAIL 0x57 | C(mode)；10/11/12/30/31/32/40（hex）追加 D；20/21 追加 H,S，後接二進位信件 |
| C_PLEDGECONTENT 0x4E | C(mode)；13:DDD；14:DHDD；15/16:S |
| C_CLANMATCHING 0x4C | C(mode)；0:CS；5:D；6:DC |
| C_ATTR 0x79 | H(mode)；0 時先 D，再 H(mode)；325/512:CS；479:C且值1再S；97,217,221,222,252,321,322,630,653,654,729,738,748,2751,951,953,3333,3312,3313,3589,3348:C |
| C_SENDLOCATION 0xFE | C(subcode)；0:D；0x0B:SHHHD；0x2E:C；Classic 0x5B/0x5F 要求整 body 7或8；0x60 要求 packet[2]=1、packet[3] 在0..3，0時總長8否則24；0x67／0x61 委派 ClassicRewardProtocol.decode／ClassicBagCombineService.decode |

Classic 0x67／0x61 是另有授權與完整版本 schema 的自訂功能，不得依最低長度填零送出。直接 byte[] parser 的確切欄位補充於本節尾端；`C_Windows.start` 的所有入口讀取仍列於 §6。

### 5.5 核心 S 封包（包含 opcode）

來源：`1-伺服器/src/com/lineage/server/serverpackets/<類別>.java` 建構子／builder；括號內 H/D 的常數也佔其型別寬度。下列 L 是 **padding 前** 的邏輯長度。

| 類別／opcode | 順序與語意 | L |
|---|---|---|
| S_MoveCharPacket 0x0A | C(op), D(objectId), H(sourceX), H(sourceY), C(heading), C(0x80) | 11；body16；source 是起點，不能當終點 |
| S_ChangeHeading 0x7A | C(op), D(objectId), C(heading) | 6 |
| S_RemoveObject 0x78 | C(op), D(objectId) | 5 |
| S_MapID／S_PcPositionMapReset 0xCE | 常用含 underwater 多載：C(op), H(mapId), C(underwater) | 4；S_MapID(int) 的 GM 多載另加12 bytes，L=16；校正 map reset 不代表跨服 |
| S_Teleport2 0x95 | C(op), H(mapId), D(id) | 7；id 為入口／物件來源，沒有目的座標 |
| S_AttackPacketPc 0x1E | C(op), C(action), D(attacker), D(resolvedTarget), H(damageWord), C(heading), H(0), H(0), C(type) | 18；body24；命中用 damageWordU16，未命中／空擊為0；type=2暴擊、4雙擊、8鏡反射 |
| S_UseAttackSkill 0x1E | C(op), C(action), D(actor或0), D(resolvedTarget), H(dmg或公版命中標記), C(heading), D(sequence), H(spellgfx), C(0), H(sourceX), H(sourceY), H(targetX), H(targetY), D(0), C(0) | 33；body40；另有 S_UseArrowSkill 與 NPC 分支，查 §6 |
| S_SkillSound 0x37 | C(op), D(objectId), H(gfxId)，部分多載追加額外欄位 | 最低7；依多載，不以類名當純 WAV |
| S_EffectLocation 0x6A | C(op), H(x), H(y), H(gfxId) | 7；測試多載可由參數指定 opcode |
| S_Sound 0x16 | C(op), C(repeat), H(sound) | 4 |
| S_DoActionGFX 0x9E | C(op), D(objectId), C(actionId) | 6 |
| S_HPUpdate 0xE1／S_MPUpdate 0x21 | 公版：C(op), H(current), H(maximum)；RO：C(op), D(current), D(maximum) | 公版5／RO9；clamp 依各 builder |
| S_InvList 0x05 | C(op), C(count)，重複 item record | 變長；本來源非 H(count) |
| S_AddItem 0x0F | C(op) 後接單筆 add record（與 list 的 charge／保留位順序不同，見 §6） | 變長；不要與 inventory list count 混淆 |
| S_PacketBox 家族 0xFA | C(op), C(subcode)，後續依建構子／subcode | 變長；完整欄位與 subcode 常數見 §6 |

背包 record 的 `item.getStatusBytes()` 是子格式，不可把文字顯示推論為全部屬性。`S_InvList` 每筆依序含 D(id), H(descId), C(useType), C(0), C(charge), H(gfx), C(bless/sealed), D(count), C(statusX), S(viewName), C(statusLength), raw[statusLength], C(10), H(0), D(0), D(0)。未鑑定或另送長 status 時 statusLength=0；`S_ItemStatus.shouldUseLongStatus/writeLongStatus` 另有長狀態分支，必須查 §6 及 ItemStatusPacketSupport，不截斷為舊短包。
### 5.6 直接 byte[] 自訂請求與封包外包裝

| 來源 | 順序（包含 opcode）／長度 |
|---|---|
| `1-伺服器/src/com/lineage/server/model/ClassicRewardProtocol.java:decode` | C(0xFE), C(0x67), C(version=4), C(command=0..6), D(nonce), D(token), D(npc), D(serial), D(page), D(id), D(0)；恰32 bytes。nonce>0、page≥0且≤INT_MAX/8、id≥0；command=0 要其餘全0，command=6 要token/npc/page/id=0、serial>0；其他要token/npc/serial>0，command=2要id>0、其他id=0 |
| `1-伺服器/src/com/lineage/server/model/item/ClassicBagCombineService.java:decode` | C(0xFE), C(0x61), C(version=2), C(command=0..3), D(nonce), D(request), D(token)。非command=1恰16 bytes；command=1追加 C(n=1..16), 固定16筆[D(objectId), D(count)]，合計145或帶3-byte原生尾段的148。前n筆id/count>0且id不重複，其餘筆必為0；nonce/request>0，command0/3的token=0，其餘token>0 |
| `1-伺服器/src/com/lineage/server/serverpackets/S_ClassicCollection.java:fragments/Chunk` | 超過MAX_PACKET時分片；每片複用原訊息前33 bytes、offset8改為command4，再H(index),H(count),H(whole.length),H(n)，接最多4000 bytes資料；外層仍補8-byte對齊，片長≤4096。原訊息中 `text(String,int)` 是H(UTF-8 byte length)+raw，沒有NUL |
| `1-伺服器/src/com/lineage/server/utils/Teleportation.java:S_TeleportRecovery` | 內部包裝類以 `writeByte(payload)` 寫入已定稿payload，**再加1個NUL並重新補8-byte對齊**，常規情況較原body多8 bytes；不新增opcode。另有current generation送出檢查，不能誤寫為writeByte1或等長轉送 |

其餘 `C_Windows` 自訂入口仍以其 `start` 分支與委派模型為準；本表對該類列出的 C/H/D/S 是直接讀取簽章，不代表每個Classic功能都可跳過UI授權。`ItemStatusPacketSupport` 與 `L1ItemStatus` 產生的status blob內容為巢狀物品屬性格式；本表標示長度與傳送位置，具體屬性碼須依該類別追查，不能以opaque bytes宣稱屬性已完整呈現。
### 5.7 C_WINDOWS 其餘 Classic 委派欄位

以下皆由 `1-伺服器/src/com/lineage/server/clientpackets/C_Windows.java:start` 分派；沒有額外的主 opcode。D 值採 Java signed int；每列已包含 C(0xFE) 與 C(subcode)。UI票據、nonce與session授權仍由各service檢查。

| subcode | 欄位順序／總body長度 | 完整相對來源與方法 |
|---|---|---|
| 0x5B／0x5F | C(FE),C(subcode),C(mode=0或1),D(positiveField)，可有1byte尾段；7或8 bytes | `1-伺服器/src/com/lineage/server/clientpackets/C_Windows.java:classicInventoryField/classicInventoryTarget` |
| 0x60 | C(FE),C(60),C(version=1),C(command),D(nonce)；command=0恰8；1..3再D(token),D(npcId),D(recipeId),D(amount)，恰24 | `1-伺服器/src/com/lineage/server/model/npc/action/ClassicCraftService.java:handle` |
| 0x62 | C(FE),C(62),C(version=2),C(command=1..4),D(nonce),D(token),D(npc),D(row),D(column)；24；command2時row>0、column=0..1，其餘兩欄0 | `1-伺服器/src/com/lineage/server/model/ClassicPassActions.java:decode` |
| 0x63 | C(FE),C(63),C(version=2),C(command=0..6),D(nonce),D(token),D(npc),D(page),D(quest)；24；nonce>0、page/quest≥0；command0其餘0，否則token/npc>0，command2..4或6需quest>0 | `1-伺服器/src/com/lineage/server/model/ClassicQuestProtocol.java:decode` |
| 0x64 | C(FE),C(64),C(version=2),C(command=0..3),D(nonce),D(token),D(npc),D(spawn)；20；command0其餘0，否則token/npc>0；command2要求spawn>0，其餘spawn=0 | `1-伺服器/src/com/lineage/server/model/ClassicBossProtocol.java:decode` |
| 0x65 | C(FE),C(65),C(version=1),C(command=0..2),D(nonce),D(token),D(npc),D(serial),C(board),C(job),C(0),C(0)；24；nonce>0、job≤7、board需model認可；command0其餘0，否則token/npc/serial>0 | `1-伺服器/src/com/lineage/server/model/ClassicRankingProtocol.java:decode` |
| 0x66 | C(FE),C(66),C(version=1),C(command=0..4),D(nonce),D(token),D(npc),D(serial),D(page),D(quest),D(offset)；32；nonce>0、page=0..100000、quest≥0、offset=0..1024且為5的倍數；command0其餘0，否則token/npc/serial>0 | `1-伺服器/src/com/lineage/server/model/ClassicCollectionProtocol.java:decode` |

## 6. 全部封包類別的欄位簽章索引

以下 346 個類別以實際讀寫位置整理；是**靜態查表**，不是封包範例，也不是一律串接的可執行 schema。每段方法下按來源先後列出欄位；不同多載各自獨立。條件欄是詞法所在的 if／else／switch／迴圈提示；`else` 對應同層前一條件，連續 case 可能共用同一段。提前 return、無大括號條件、變數計算仍以指定方法為準；不能把互斥列全部相加。`呼叫` 要在該位置展開對應 helper／多載；`super` 依繼承類別查表。變長長度為實際走到之欄位寬度和，迴圈乘實際元素數，再依 §1 加 padding；不宣稱固定長度。

`L數字` 是欄位讀寫行；括號保留參數／變數名稱以追溯語意，非 Java 實作複製。C 欄位省略共同的 opcode；S 欄位包含 writer 寫出的 opcode。`未命名/內嵌讀取` 表示沒有獨立變數，不自行賦予意義。沒有讀寫的入口明確註明委派／不讀取，直接 byte[] parser 另見 §5。

### C_AddBookmark

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_AddBookmark.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L52）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L82 `S(locName)` |

### C_AddBuddy

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_AddBuddy.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L53）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L59 `S(charName)` |

### C_Amount

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Amount.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L135）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L169 `D(objectId)` → L171 `D(未命名/內嵌讀取)` → L177 `C(未命名/內嵌讀取)` → L179 `S(s)` |

### C_Attack

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Attack.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L33）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L94 `D(targetId)` → L95 `H(locx)` → L96 `H(locy)` |
| `if (remainingBytes() > 0)` | L99 `C(未命名/內嵌讀取)` |

### C_AttackBow

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_AttackBow.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L53）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L136 `D(targetId)` → L137 `H(locx)` → L138 `H(locy)` |

### C_Attr

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Attr.java`；繼承 `ClientBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`MAX_DELAYED_DISPATCH_RETRIES=8`。

`void start(byte[] decrypt, ClientExecutor client)`（L275）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L297 `H(mode)` |
| `if (mode == 0)` | L300 `D(confirmationId)` → L301 `H(mode)` |
| `switch (mode) / case 622:` | L308 `C(c)` |
| `switch (mode) / case 97:` | L318 `C(c)` |
| `switch (mode) / case 222:` | L423 `C(c)` |
| `switch (mode) / case 252:` | L465 `C(c)` |
| `switch (mode) / case 321:` | L472 `C(c)` |
| `switch (mode) / case 322:` | L497 `C(c)` |
| `switch (mode) / case 325:` | L528 `C(c)` → L529 `S(petName)` |
| `switch (mode) / case 512:` | L602 `C(c)` → L603 `S(未命名/內嵌讀取)` |
| `switch (mode) / case 630:` | L607 `C(c)` |
| `switch (mode) / case 653:` | L628 `C(c)` |
| `switch (mode) / case 654:` | L665 `C(c)` |
| `switch (mode) / case 729:` | L698 `C(c)` |
| `switch (mode) / case 738:` | L708 `C(c)` |
| `switch (mode) / case 748:` | L725 `C(c)` |
| `switch (mode) / case 2751:` | L739 `C(c)` |
| `switch (mode) / case 951:` | L837 `C(c)` |
| `switch (mode) / case 953:` | L867 `C(c)` |
| `switch (mode) / case 479:` | L900 `C(c)` |
| `switch (mode) / if (c == 1) / case 479:` | L902 `S(s)` |
| `switch (mode) / case 3333:` | L989 `C(c)` |
| `switch (mode) / case 3312:` | L1067 `C(c)` |
| `switch (mode) / case 3313:` | L1080 `C(c)` |
| `switch (mode) / case 3589:` | L1093 `C(c)` |
| `switch (mode) / case 3348:` | L1106 `C(attentionYn)` |

### C_AuthLogin

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_AuthLogin.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L100）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L113 `S(loginName)` → L128 `S(password)` |

### C_AutoLogin

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_AutoLogin.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `C(un)` → L26 `S(loginName)` → L27 `S(srcpassword)` |

### C_BanClan

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_BanClan.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L73）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L79 `S(s)` |

### C_BanParty

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_BanParty.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L37）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L49 `S(userName)` |

### C_Board

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Board.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L43）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L48 `D(objId)` |

### C_BoardBack

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_BoardBack.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L40 `D(objId)` → L41 `D(topicNumber)` |

### C_BoardDelete

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_BoardDelete.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L40 `D(objId)` → L41 `D(topicId)` |

### C_BoardRead

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_BoardRead.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L40 `D(objId)` → L41 `D(topicNumber)` |

### C_BoardWrite

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_BoardWrite.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L56 `D(objId)` → L63 `S(title)` → L68 `S(content)` |

### C_Buddy

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Buddy.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L34）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_CallPlayer

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_CallPlayer.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L92）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L113 `S(arg)` |

### C_ChangeHeading

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ChangeHeading.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L52 `C(heading)` |

### C_ChangeWarTime

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ChangeWarTime.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L73）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_CharcterConfig

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_CharcterConfig.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L51）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L67 `D(length)` → L72 `Byte(未命名/內嵌讀取)` |

### C_CharReset

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_CharReset.java`；繼承 `ClientBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`QUEST_STEP_CLASS_CHANGE=30677`。

`void start(byte[] decrypt, ClientExecutor client)`（L81）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L96 `C(stage)` |
| `switch (stage) / case 0x01:` | L99 `C(str)` → L100 `C(intel)` → L101 `C(wis)` → L102 `C(dex)` → L103 `C(con)` → L104 `C(cha)` |
| `switch (stage) / case 0x02:` | L138 `C(type2)` |
| `switch (stage) / switch (type2) / case 0x08:` | L182 `C(未命名/內嵌讀取)` |
| `switch (stage) / case 0x03:` | L212 `C(read1)` → L213 `C(read2)` → L214 `C(read3)` → L215 `C(read4)` → L216 `C(read5)` → L217 `C(read6)` |

### C_Chat

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Chat.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L139）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L201 `C(chatType)` → L202 `S(chatText)` |

### C_ChatGlobal

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ChatGlobal.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L95）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L183 `C(chatType)` → L184 `S(chatText)` |

### C_ChatParty

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ChatParty.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L38）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L53 `C(type)` |
| `switch (type) / case 0:` | L56 `S(name)` |

### C_ChatWhisper

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ChatWhisper.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L85）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L123 `S(targetName)` → L124 `S(text)` |

### C_Check

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Check.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `D(checkType)` → L26 `Byte(remainingData)` |

### C_CheckPK

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_CheckPK.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L34）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_Clan

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Clan.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L75）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L103 `D(emblemId)` |

### C_ClanAttention

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ClanAttention.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L44）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L52 `C(type)` |
| `switch (type) / case 0 ->` | L79 `S(clanName)` |
| `switch (type) / case 1 ->` | L85 呼叫 `handleTypeRemove(pc, clan, decrypt, remain)` |
| `switch (type) / case 2 ->` | L86 呼叫 `handleTypeList(pc, clan, decrypt, remain)` |

`void handleTypeList(L1PcInstance pc, L1Clan clan, byte[] decrypt, int remain)`（L105）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (remain > 2)` | L125 `S(未命名/內嵌讀取)` |

`void handleTypeRemove(L1PcInstance pc, L1Clan clan, byte[] decrypt, int remain)`（L131）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (remain > 2)` | L146 `S(clanName)` |

### C_CnItem

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_CnItem.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L31）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_CommonClick

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_CommonClick.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L55）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_CreateChar

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_CreateChar.java`；繼承 `ClientBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`TYPE_DRAGON_KNIGHT=5`、`TYPE_ILLUSIONIST=6`、`CREATE_CHAR_STAT_BYTES=6`。

`void skipCreateCharStats()`（L126）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < CREATE_CHAR_STAT_BYTES; i++)` | L128 `C(未命名/內嵌讀取)` |

`void start(byte[] decrypt, ClientExecutor client)`（L133）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L145 `S(未命名/內嵌讀取)` → L178 `C(type)` → L179 `C(sex)` |
| `if (type < 0 \|\| type >= TYPE_COUNT \|\| sex < 0 \|\| sex >= CLASS_LIST.length)` | L182 呼叫 `skipCreateCharStats()` |
| `if ((type == TYPE_DRAGON_KNIGHT && !ConfigCharSetting.CREATE_CHAR_ALLOW_DRAGON_KNIGHT) \|\| (type == TYPE_ILLUSIONIST && !ConfigCharSetting.CREATE_CHAR_ALLOW_ILLUSIONIST))` | L189 呼叫 `skipCreateCharStats()` |
| 本方法此處 | L198 `C(未命名/內嵌讀取)` → L199 `C(未命名/內嵌讀取)` → L200 `C(未命名/內嵌讀取)` → L201 `C(未命名/內嵌讀取)` → L202 `C(未命名/內嵌讀取)` → L203 `C(未命名/內嵌讀取)` |

### C_CreateClan

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_CreateClan.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L74）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L84 `S(name)` |

### C_CreateParty

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_CreateParty.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L42）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L61 `C(type)` |
| `switch (type) / case 1:` | L66 `D(targetId)` |
| `switch (type) / case 2:` | L100 `S(name)` |
| `switch (type) / if (pc.isInParty()) / if (pc.getParty().isLeader(pc)) / case 3:` | L147 `D(objid)` |

### C_DelBuddy

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_DelBuddy.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L48）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L59 `S(charName)` |

### C_DeleteBookmark

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_DeleteBookmark.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L48）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L54 `S(bookmarkname)` |

### C_DeleteChar

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_DeleteChar.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L82）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L88 `S(name)` |

### C_DeleteInventoryItem

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_DeleteInventoryItem.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L43 `D(itemObjectId)` |

### C_Deposit

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Deposit.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L82）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L88 `D(objid)` → L89 `D(count)` |

### C_Disconnect

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Disconnect.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L39）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_Door

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Door.java`；繼承 `ClientBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`MAX_DISPATCH_RETRIES=8`。

`void start(byte[] decrypt, ClientExecutor client)`（L128）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L158 `H(未命名/內嵌讀取)` → L159 `H(未命名/內嵌讀取)` → L160 `D(objectId)` |

### C_Drawal

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Drawal.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L88）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L116 `D(objectId)` → L117 `D(count)` |

### C_DropItem

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_DropItem.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L40）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L45 `H(x)` → L46 `H(y)` → L47 `D(objectId)` → L48 `D(count)` |

### C_Emblem

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Emblem.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L81）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (clan_id > 0)` | L119 `Byte(iconByte)` |

### C_EnterPortal

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_EnterPortal.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L71）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L99 `H(locx)` → L100 `H(locy)` |

### C_Exclude

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Exclude.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L41 `S(name)` |

### C_ExitGhost

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ExitGhost.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L33）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_ExtraCommand

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ExtraCommand.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L37）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L60 `C(actionId)` |

### C_Fight

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Fight.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L36）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_FishClick

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_FishClick.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L33）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_FixWeaponList

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_FixWeaponList.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L38）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_GiveItem

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_GiveItem.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L97）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L148 `D(targetId)` → L149 `H(未命名/內嵌讀取)` → L150 `H(未命名/內嵌讀取)` → L152 `D(itemObjId)` → L153 `D(count)` |

### C_HireSoldier

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_HireSoldier.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L32）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 `H(未命名/內嵌讀取)` → L38 `H(未命名/內嵌讀取)` → L39 `D(未命名/內嵌讀取)` → L40 `D(未命名/內嵌讀取)` → L41 `H(未命名/內嵌讀取)` |

### C_ItemUSe

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ItemUSe.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L123）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L124 呼叫 `useItemRequest(decrypt, client, 0, 0)` |

`void useItemRequest(byte[] decrypt, ClientExecutor client, int configuredItemId, int configuredTargetId)`（L212）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L260 `D(未命名/內嵌讀取)` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case -8 ->` | L481 `C(newData[0])` → L482 `C(newData[1])` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 5 ->` | L578 `D(newData[0])` → L579 `H(newData[1])` → L580 `H(newData[2])` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 6 ->` | L597 `H(newData[1])` → L598 `D(newData[0])` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 7 ->` | L611 `D(未命名/內嵌讀取)` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 8 ->` | L624 `D(newData[0])` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 14 ->` | L657 `D(未命名/內嵌讀取)` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 16 ->` | L674 `S(cmd)` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 17 ->` | L684 `D(newData[0])` → L685 `H(newData[1])` → L686 `H(newData[2])` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 26, 27, 46 ->` | L700 `D(未命名/內嵌讀取)` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 28 ->` | L713 `C(newData[0])` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 29 ->` | L731 `H(newData[1])` → L733 `D(newData[0])` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 30 ->` | L745 `D(obj)` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 12, 31, 33, 35 ->` | L759 `H(newData[0])` → L760 `S(未命名/內嵌讀取)` → L761 `Byte(未命名/內嵌讀取)` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 39 ->` | L789 `D(newData[0])` → L790 `H(newData[1])` → L791 `H(newData[2])` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 42 ->` | L804 `H(newData[0])` → L805 `H(newData[1])` |
| `if (pc.getCurrentHp() > 0) / switch (use_type) / if (isClass) / case 55 ->` | L818 `D(未命名/內嵌讀取)` |

### C_JoinClan

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_JoinClan.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L36）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_KeepALIVE

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_KeepALIVE.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L38）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_LeaveClan

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_LeaveClan.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L88）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_LeaveParty

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_LeaveParty.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L34）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_LoginToServer

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_LoginToServer.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L192）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L233 `S(charName)` |

### C_LoginToServerOK

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_LoginToServerOK.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L33）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `C(type)` → L39 `C(button)` |

### C_Mail

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Mail.java`；繼承 `ClientBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`TYPE_NORMAL_MAIL=0x00`、`TYPE_CLAN_MAIL=0x01`、`TYPE_MAIL_BOX=0x02`。

`void start(byte[] decrypt, ClientExecutor client)`（L137）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L142 `C(type)` |
| `switch (type) / case 0x12:` | L191 `D(id)` |
| `switch (type) / case 0x20:` | L194 `H(未命名/內嵌讀取)` → L195 `S(receiverName)` → L196 `Byte(textr)` |
| `switch (type) / case 0x21:` | L199 `H(未命名/內嵌讀取)` → L200 `S(clanName)` → L201 `Byte(text)` |
| `switch (type) / case 0x32:` | L206 `D(delid)` |
| `switch (type) / case 0x40:` | L209 `D(saveid)` |

### C_MoveChar

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_MoveChar.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L151）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L170 `H(locx)` → L171 `H(locy)` → L172 `C(heading)` |

### C_NewAccess

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_NewAccess.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L78）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L84 `S(loginName)` → L85 `S(password)` → L98 `S(未命名/內嵌讀取)` → L99 `S(spwd)` → L101 `C(未命名/內嵌讀取)` → L102 `S(未命名/內嵌讀取)` → L103 `S(未命名/內嵌讀取)` → L104 `S(未命名/內嵌讀取)` |

### C_NewCharSelect

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_NewCharSelect.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L148）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_NPCAction

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_NPCAction.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L209）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L243 `D(objid)` → L244 `S(s)` |
| `if (obj instanceof L1NpcInstance) / if (remainingBytes() > 0 && tmp.getNpcTemplate().get_classname().equalsIgnoreCase("other.Npc_AuctionBoard"))` | L518 `S(s2)` |
| `if (obj instanceof L1NpcInstance) / else if (remainingBytes() > 0 && tmp.getNpcTemplate().get_classname().equalsIgnoreCase("other.Npc_Board"))` | L520 `S(s2)` |

### C_NPCTalk

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_NPCTalk.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L41）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L66 `D(objid)` |

### C_Party

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Party.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L36）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_Password

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Password.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L85）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L91 `C(mode)` → L92 `D(olepwd)` → L93 `D(newpwd)` |

### C_PetMenu

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_PetMenu.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L38）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L61 `D(petId)` |

### C_PickUpItem

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_PickUpItem.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L45）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L87 `H(x)` → L88 `H(y)` → L89 `D(objectId)` → L90 `D(pickupCount)` |

### C_Pledge

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Pledge.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L33）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_PledgeContent

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_PledgeContent.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L72）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L87 `C(data)` |
| `switch (data) / case 13:` | L92 `D(npcObjId)` → L93 `D(itemObjId)` → L94 `D(assistItemObjId)` |
| `switch (data) / case 14:` | L113 `D(npcObjId)` → L114 `H(actionId)` → L115 `D(plusItemObjId)` → L116 `D(plusItemCount)` |
| `switch (data) / case 15:` | L142 `S(announce)` |
| `switch (data) / case 16:` | L173 `S(notes)` |

### C_PledgeRecommendation

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_PledgeRecommendation.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L100）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L116 `C(data)` |
| `if (data == 0)` | L140 `C(clanType)` → L141 `S(typeMessage)` |
| `else if (data == 5)` | L341 `D(clan_id)` |
| `else if (data == 6)` | L417 `D(objid)` → L418 `C(type)` |

### C_Propose

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Propose.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L41 `C(c)` |

### C_Rank

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Rank.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L84）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L94 `C(data)` |
| `if (data == 1)` | L96 `C(rank)` → L97 `S(name)` |

### C_Restart

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Restart.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L34）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_Result

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Result.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L285）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L316 `D(npcObjectId)` → L317 `C(resultType)` → L318 `C(size)` → L319 `C(未命名/內嵌讀取)` |
| `if (resultType == 0 && size > 0 && pc.get_otherList().hasLottoBuyIntent()) / for (int i = 0; i < size; i++)` | L326 `D(orderId)` → L327 `D(未命名/內嵌讀取)` |
| `if (pc.getTempID() > 0 && resultType == 0 && npcObjectId == 71702)` | L337 呼叫 `mode_buy(pc, 71702, size)` |
| `if (findObject != null) / if (findObject == pc && com.lineage.server.datatables.AuctionEventTable.get().isEnabled() && size == 1 && (resultType == 12 \|\| resultType == 0))` | L357 `D(dataId)` → L358 `D(count)` |
| `if (findObject != null) / if (findObject instanceof L1NpcInstance) / if (npcId == 70535)` | L387 `D(dataId)` → L388 `D(count)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1MerchantInstance) / case 0:` | L426 呼叫 `mode_buy(pc, npcId, size)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1GamblingInstance) / case 0:` | L431 呼叫 `mode_gambling(pc, npcId, size, true)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1LottoInstance \|\| pc.get_otherList().hasLottoBuyIntent()) / case 0:` | L436 呼叫 `mode_lotto(pc, npcId, size, true)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1CnInstance) / case 0:` | L441 呼叫 `mode_cn(pc, npcId, size, true)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1DeInstance) / case 0:` | L447 呼叫 `mode_buyde(pc, de, size)` |
| `switch (resultType) / if (size > 0) / if (pc.equals(findObject)) / case 0:` | L452 呼叫 `mode_cn(pc, 0, size, true)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1PcInstance) / if (isPrivateShop) / case 0:` | L459 呼叫 `mode_buypc(pc, targetPc, size)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1MerchantInstance) / switch (npcId) / default:` | L471 呼叫 `mode_sell(pc, npcId, size)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1GamblingInstance) / default:` | L478 呼叫 `mode_gambling(pc, npcId, size, false)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1LottoInstance) / default:` | L483 呼叫 `mode_lotto(pc, npcId, size, false)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1DeInstance) / if (findObject instanceof L1DeInstance) / default:` | L490 呼叫 `mode_sellde(pc, de, size)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1PcInstance) / if (isPrivateShop) / default:` | L498 呼叫 `mode_sellpc(pc, targetPc, size)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1DwarfInstance \|\| pcStorageContext) / if (level >= 5) / case 2:` | L509 呼叫 `mode_warehouse_in(pc, npcId, size)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1DwarfInstance \|\| pcStorageContext) / if (level >= 5) / case 3:` | L520 呼叫 `mode_warehouse_out(pc, npcId, size)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1DwarfInstance \|\| pcStorageContext) / if (level >= 5) / case 4:` | L531 呼叫 `mode_warehouse_clan_in(pc, npcId, size)` |
| `switch (resultType) / if (size > 0 && (findObject instanceof L1DwarfInstance \|\| pcStorageContext) && pc.getLevel() >= 5) / case 5:` | L540 呼叫 `mode_warehouse_clan_out(pc, npcId, size)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1DwarfInstance) / if ((level >= 5) && pc.isElf()) / case 8:` | L562 呼叫 `mode_warehouse_elf_in(pc, npcId, size)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1DwarfInstance) / if ((level >= 5) && pc.isElf()) / case 9:` | L573 呼叫 `mode_warehouse_elf_out(pc, npcId, size)` |
| `switch (resultType) / if (size > 0) / switch (npcId) / case 91143:` | L585 呼叫 `mode_update_item(pc, size, npcObjectId)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1DwarfInstance \|\| pcStorageContext) / if (level >= 5) / case 17:` | L622 呼叫 `mode_warehouse_cha_in(pc, npcId, size)` |
| `switch (resultType) / if (size > 0) / if (findObject instanceof L1DwarfInstance \|\| pcStorageContext) / if (level >= 5) / case 18:` | L633 呼叫 `mode_warehouse_cha_out(pc, npcId, size)` |

`void mode_update_item(L1PcInstance pc, int size, int npcObjectId)`（L648）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L658 `D(orderId)` → L659 `D(未命名/內嵌讀取)` |

`void mode_cn(L1PcInstance pc, int npcId, int size, boolean isShop)`（L712）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (isShop) / for (int i = 0; i < size; i++)` | L719 `D(orderId)` → L720 `D(未命名/內嵌讀取)` |

`void mode_gambling(L1PcInstance pc, int npcId, int size, boolean isShop)`（L742）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (isShop) / for (int i = 0; i < size; i++)` | L753 `D(orderId)` → L754 `D(未命名/內嵌讀取)` |
| `else / for (int i = 0; i < size; i++)` | L765 `D(objid)` → L766 `D(未命名/內嵌讀取)` |

`void mode_lotto(L1PcInstance pc, int npcId, int size, boolean isShop)`（L779）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (isShop) / for (int i = 0; i < size; i++)` | L793 `D(orderId)` → L794 `D(未命名/內嵌讀取)` |
| `else / for (int i = 0; i < size; i++)` | L803 `D(objid)` → L804 `D(未命名/內嵌讀取)` |

`void mode_warehouse_elf_out(L1PcInstance pc, int npcId, int size)`（L820）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L827 `D(objectIds[i])` → L828 `D(未命名/內嵌讀取)` |

`void mode_warehouse_elf_in(L1PcInstance pc, int npcId, int size)`（L878）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L885 `D(objectIds[i])` → L886 `D(未命名/內嵌讀取)` |

`void mode_warehouse_clan_out(L1PcInstance pc, int npcId, int size)`（L971）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L978 `D(objectIds[i])` → L979 `D(未命名/內嵌讀取)` |

`void mode_warehouse_clan_in(L1PcInstance pc, int npcId, int size)`（L1047）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L1054 `D(objectIds[i])` → L1055 `D(未命名/內嵌讀取)` |

`void mode_warehouse_cha_out(L1PcInstance pc, int npcId, int size)`（L1153）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L1160 `D(objectIds[i])` → L1161 `D(未命名/內嵌讀取)` |

`void mode_warehouse_cha_in(L1PcInstance pc, int npcId, int size)`（L1211）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L1222 `D(objectIds[i])` → L1223 `D(未命名/內嵌讀取)` |

`void mode_warehouse_out(L1PcInstance pc, int npcId, int size)`（L1312）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L1318 `D(objectId)` → L1319 `D(未命名/內嵌讀取)` |

`void mode_warehouse_in (L1PcInstance pc, int npcId, int size)`（L1349）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L1356 `D(objectIds[i])` → L1357 `D(未命名/內嵌讀取)` |

`void mode_sellde(L1PcInstance pc, L1DeInstance de, int size)`（L1445）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L1446 呼叫 `modeSellDeLocked(pc, de, size)` |

`void modeSellDeLocked(L1PcInstance pc, L1DeInstance de, int size)`（L1449）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L1463 `D(itemObjectId)` → L1464 `CH(count)` |

`void mode_buyde(L1PcInstance pc, L1DeInstance de, int size)`（L1545）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L1546 呼叫 `modeBuyDeLocked(pc, de, size)` |

`void modeBuyDeLocked(L1PcInstance pc, L1DeInstance de, int size)`（L1549）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L1563 `D(order)` → L1564 `D(未命名/內嵌讀取)` |

`void mode_sellpc(L1PcInstance pc, L1PcInstance targetPc, int size)`（L1657）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L1702 `D(itemObjectId)` → L1703 `CH(count)` → L1709 `C(order)` |

`void mode_buypc(L1PcInstance pc, L1PcInstance targetPc, int size)`（L1862）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < finalSize; i++)` | L1874 `D(orders[i])` → L1875 `D(未命名/內嵌讀取)` |

`void mode_sell(L1PcInstance pc, int npcId, int size)`（L2139）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L2146 `D(objids[i])` → L2147 `D(未命名/內嵌讀取)` |

`void mode_buy(L1PcInstance pc, int npcId, int size)`（L2191）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i = 0; i < size; i++)` | L2198 `D(orderIds[i])` → L2199 `D(未命名/內嵌讀取)` |

### C_ReturnToLogin

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ReturnToLogin.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L35）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_SelectList

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_SelectList.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L97）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L133 `D(itemObjectId)` → L134 `D(npcObjectId)` |

### C_SelectTarget

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_SelectTarget.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L44）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L49 `D(petId)` → L50 `C(未命名/內嵌讀取)` → L51 `D(targetId)` |

### C_ServerVersion

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ServerVersion.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L33）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_Ship

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Ship.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (item_id != 0)` | L67 `H(shipMapId)` → L68 `H(locX)` → L69 `H(locY)` |

### C_Shop

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Shop.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L46）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L58 `C(type)` |
| `if (type == 0)` | L130 `H(sellTotalCount)` |
| `if (type == 0) / for (int i = 0; i < sellTotalCount; i++)` | L139 `D(sellObjectId)` → L140 `D(未命名/內嵌讀取)` → L145 `D(未命名/內嵌讀取)` |
| `if (type == 0)` | L209 `H(buyTotalCount)` |
| `if (type == 0) / for (int i = 0; i < buyTotalCount; i++)` | L218 `D(buyObjectId)` → L219 `D(未命名/內嵌讀取)` → L226 `D(未命名/內嵌讀取)` |
| `if (type == 0)` | L317 `Byte(chat)` |

### C_ShopList

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_ShopList.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L90 `C(type)` → L91 `D(objectId)` |

### C_SkillBuy

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_SkillBuy.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L85）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L121 `D(objid)` |

### C_SkillBuyItem

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_SkillBuyItem.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L37）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L61 `D(objid)` |

### C_SkillBuyItemOK

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_SkillBuyItemOK.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L81）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L151 `H(count)` |
| `for (int i = 0; i < count; i++)` | L161 `D(skillIds[i])` |

### C_SkillBuyOK

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_SkillBuyOK.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L112）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L152 `H(count)` |
| `for (int i = 0; i < count; i++)` | L195 `D(skillIds[i])` |

### C_TaxRate

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_TaxRate.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L82）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L88 `D(i)` → L89 `C(j)` |

### C_Teleport

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Teleport.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L35）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_Title

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Title.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L39）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L45 `S(charName)` → L48 `S(未命名/內嵌讀取)` |

### C_Trade

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Trade.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L17）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_TradeAddItem

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_TradeAddItem.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L39）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L59 `D(itemObjid)` → L61 `D(itemcount)` |

### C_TradeCancel

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_TradeCancel.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L34）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_TradeOK

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_TradeOK.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L15）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_Unkonwn

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Unkonwn.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L35）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_UnLock

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_UnLock.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L39 `C(type)` |

### C_UsePetItem

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_UsePetItem.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L89）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L95 `C(data)` → L96 `D(petId)` → L97 `C(listNo)` |

### C_UseSkill

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_UseSkill.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L52）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L106 `C(row)` → L107 `C(column)` |
| `if (decrypt.length > 4) / switch (skillId) / case 43:` | L203 `D(targetId)` |
| `if (decrypt.length > 4) / switch (skillId) / case 150:` | L216 `D(targetId)` |
| `if (decrypt.length > 4) / switch (skillId) / case 186:` | L225 `D(targetId)` |
| `if (decrypt.length > 4) / switch (skillId) / case 118:` | L235 `S(charName)` |
| `if (decrypt.length > 4) / switch (skillId) / case 113:` | L244 `D(targetId)` → L245 `H(targetX)` → L246 `H(targetY)` → L248 `S(requestedTrueTargetText)` |
| `if (decrypt.length > 4) / switch (skillId) / case 69:` | L260 `H(未命名/內嵌讀取)` → L261 `D(targetId)` |
| `if (decrypt.length > 4) / switch (skillId) / case 63:` | L271 `H(targetX)` → L272 `H(targetY)` |
| `if (decrypt.length > 4) / switch (skillId) / if ((pc.getInventory().checkEquipped(20284)) \|\| (pc.getInventory().checkEquipped(120284))) / case 51:` | L280 `D(requestedSummonId)` |
| `if (decrypt.length > 4) / switch (skillId) / else / case 51:` | L287 `D(未命名/內嵌讀取)` |
| `if (decrypt.length > 4) / switch (skillId) / case 213:` | L322 `D(targetId)` → L323 `H(targetX)` → L324 `H(targetY)` |
| `if (decrypt.length > 4) / switch (skillId) / case 219:` | L398 `D(targetId)` |
| `if (decrypt.length > 4) / switch (skillId) / case 220:` | L486 `D(targetId)` |
| `if (decrypt.length > 4) / switch (skillId) / if (pc.getInventory().getTypeEquipped(2, 7) >= 1) / case 88:` | L497 `D(targetId)` |
| `if (decrypt.length > 4) / switch (skillId) / if (pc.getInventory().getTypeEquipped(1, 3) >= 1) / case 91:` | L510 `D(targetId)` |
| `if (decrypt.length > 4) / switch (skillId) / default:` | L521 `D(targetId)` |

### C_War

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_War.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L83）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L126 `C(type)` → L127 `S(tgname)` |

### C_WebWindow

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_WebWindow.java`；繼承 `ClientBasePacket`。

無直接 read/write 欄位；參見入口 `void start(byte[] decrypt, ClientExecutor client)`（L22）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### C_Who

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Who.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L51）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L56 `S(s)` |

### C_Windows

來源：`1-伺服器/src/com/lineage/server/clientpackets/C_Windows.java`；繼承 `ClientBasePacket`。

`void start(byte[] decrypt, ClientExecutor client)`（L77）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L92 `C(type)` |
| `switch (type) / case 0x00:` | L125 `D(objid)` |
| `switch (type) / case 0x0b:` | L141 `S(name)` → L146 `H(mapid)` → L147 `H(x)` → L148 `H(y)` → L149 `D(zone)` |
| `switch (type) / case 0x2e:` | L168 `C(emblemStatus)` |

### S_Ability

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Ability.java`；繼承 `ServerBasePacket`。

`S_Ability(int type, boolean equipped)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket(type, equipped)` |

`void buildPacket(int type, boolean equipped)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_ABILITY)` → L23 `C(type)` → L24 `C(equipped? 0x01 : 0x00)` |

### S_AddItem

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_AddItem.java`；繼承 `ServerBasePacket`。

`S_AddItem(L1ItemInstance item)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 `C(S_OPCODE_ADDITEM)` → L20 `D(item.getId())` → L21 `H(item.getItem().getPacketItemDescId())` → L22 `C(item.getItem().getPacketUseType())` → L24 `C(item.getChargeCount())` → L26 `C(0)` → L27 `H(item.get_gfxid())` → L29 `C(item.isSealed() ? L1Item.toPacketBless(item.getBless()) + 128 : L1Item.toPacketBless(item.getBless()))` → L30 `D((int)Math.min(item.getCount(), 2000000000))` → L55 `C(statusX)` → L56 `S(item.getViewName())` |
| `if (!item.isIdentified())` | L58 `C(0)` |
| `else if (S_ItemStatus.shouldUseLongStatus(item))` | L60 `C(0)` |
| `else` | L63 `C(status.length)` |
| `else / for (int i = 0; i < j; i++)` | L68 `C(b)` |
| 本方法此處 | L71 `C(10)` → L72 `H(0)` → L73 `D(0)` → L74 `D(0)` |

### S_AddSkill

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_AddSkill.java`；繼承 `ServerBasePacket`。

`S_AddSkill(L1PcInstance pc, int skillid)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 `C(S_OPCODE_ADDSKILL)` → L30 `C(skill_list_values)` |
| `for (final int element : skillList)` | L33 `C(element)` |

`S_AddSkill(L1PcInstance pc, int[] skills)`（L47）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L52 `C(S_OPCODE_ADDSKILL)` |
| `if ((i6 > 0) && (j6 == 0))` | L55 `C(0x32)` |
| `else if (j6 > 0)` | L58 `C(0x64)` |
| `else` | L61 `C(0x20)` |
| 本方法此處 | L64 `C(skills[0])` → L65 `C(skills[1])` → L66 `C(skills[2])` → L67 `C(skills[3])` → L68 `C(skills[4])` → L69 `C(skills[5])` → L70 `C(skills[6])` → L71 `C(skills[7])` → L72 `C(skills[8])` → L73 `C(skills[9])` → L75 `C(skills[10])` → L76 `C(skills[11])` → L78 `C(skills[12])` → L79 `C(skills[13])` → L81 `C(skills[14])` → L83 `C(skills[15])` → L85 `C(skills[16])` → L86 `C(skills[17])` → L87 `C(skills[18])` → L88 `C(skills[19])` → L89 `C(skills[20])` → L90 `C(skills[21])` → L92 `C(skills[22])` → L93 `C(skills[23])` → L94 `C(skills[24])` → L96 `C(skills[25])` → L97 `C(skills[26])` → L98 `C(skills[27])` |

`S_AddSkill(L1PcInstance pc, int level1, int level2, int level3, int level4, int level5, int level6, int level7, int level8, int level9, int level10, int knight1, int knight2, int de1, int de2, int royal, int un, int elf1, int elf2, int elf3, int elf4, int elf5, int elf6, int k1, int k2, int k3, int l1, int l2, int l3 )`（L205）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L219 `C(S_OPCODE_ADDSKILL)` |
| `if ((i6 > 0) && (j6 == 0))` | L221 `C(0x32)` |
| `else if (j6 > 0)` | L224 `C(0x64)` |
| `else` | L227 `C(0x20)` |
| 本方法此處 | L230 `C(level1)` → L231 `C(level2)` → L232 `C(level3)` → L233 `C(level4)` → L234 `C(level5)` → L235 `C(level6)` → L236 `C(level7)` → L237 `C(level8)` → L238 `C(level9)` → L239 `C(level10)` → L241 `C(knight1)` → L242 `C(knight2)` → L244 `C(de1)` → L245 `C(de2)` → L247 `C(royal)` → L249 `C(un)` → L251 `C(elf1)` → L252 `C(elf2)` → L253 `C(elf3)` → L254 `C(elf4)` → L255 `C(elf5)` → L256 `C(elf6)` → L258 `C(k1)` → L259 `C(k2)` → L260 `C(k3)` → L262 `C(l1)` → L263 `C(l2)` → L264 `C(l3)` |

`S_AddSkill(L1PcInstance pc, int mode, int dx)`（L334）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L337 `C(S_OPCODE_ADDSKILL)` → L338 `C(skill_list_values)` |
| `for (int i = 0 ; i < 28 ; i++) / if (i == mode)` | L342 `C(dx)` |
| `for (int i = 0 ; i < 28 ; i++) / else` | L345 `C(0x00)` |

### S_ApplyAuction

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ApplyAuction.java`；繼承 `ServerBasePacket`。

`S_ApplyAuction(int objectId, String houseNumber)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 呼叫 `buildPacket(objectId, houseNumber)` |

`void buildPacket(int objectId, String houseNumber)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 `C(S_OPCODE_INPUTAMOUNT)` → L38 `D(objectId)` → L39 `D(0x00000000)` → L43 `D((int) minimum)` → L44 `D((int) minimum)` → L46 `D(0x77359400)` → L47 `H(0x0000)` → L48 `S("agapply")` → L49 `S("agapply " + houseNumber)` |

### S_AttackPacketNpc

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_AttackPacketNpc.java`；繼承 `ServerBasePacket`。

`S_AttackPacketNpc(L1NpcInstance npc, L1Character target, int type, int dmg )`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `C(S_OPCODE_ATTACKPACKET)` → L39 `C(type)` → L40 `D(npc.getId())` → L41 `D(HitReactionSuppressTable.get().resolveTargetObjId(target.getId(), target))` |
| `if (com.lineage.config.ConfigClient.isROClient())` | L47 `H(dmg)` |
| `else` | L49 `H(dmg > 0 ? 0x0a : 0x00)` |
| 本方法此處 | L52 `C(npc.getHeading())` → L54 `H(0x00)` → L55 `H(0x00)` → L57 `C(0x00)` |

`S_AttackPacketNpc(L1NpcInstance npc, int targetid, int type, int dmg )`（L68）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L73 `C(S_OPCODE_ATTACKPACKET)` → L74 `C(type)` → L75 `D(npc.getId())` → L76 `D(HitReactionSuppressTable.get().resolveTargetObjId(targetid))` |
| `if (com.lineage.config.ConfigClient.isROClient())` | L82 `H(dmg)` |
| `else` | L84 `H(dmg > 0 ? 0x0a : 0x00)` |
| 本方法此處 | L87 `C(npc.getHeading())` → L89 `H(0x00)` → L90 `H(0x00)` → L92 `C(0x00)` |

`S_AttackPacketNpc(L1NpcInstance npc, L1Character target, int type )`（L102）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L106 `C(S_OPCODE_ATTACKPACKET)` → L107 `C(type)` → L108 `D(npc.getId())` → L109 `D(HitReactionSuppressTable.get().resolveTargetObjId(target.getId(), target))` → L110 `H(0x00)` → L111 `C(npc.getHeading())` → L113 `H(0x00)` → L114 `H(0x00)` → L116 `C(0x00)` |

### S_AttackPacketPc

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_AttackPacketPc.java`；繼承 `ServerBasePacket`。

`S_AttackPacketPc(L1PcInstance pc, L1Character target, int type, int dmg)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `C(S_OPCODE_ATTACKPACKET)` → L39 `C(ActionCodes.ACTION_Attack)` → L40 `D(pc.getId())` → L41 `D(HitReactionSuppressTable.get().resolveTargetObjId(target.getId(), target))` → L44 `H[依 ClientLogin](this, dmg)` → L46 `C(pc.getHeading())` → L48 `H(0x00)` → L49 `H(0x00)` → L50 `C(type)` |

`S_AttackPacketPc(L1PcInstance pc, L1Character target)`（L59）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L61 `C(S_OPCODE_ATTACKPACKET)` → L62 `C(ActionCodes.ACTION_Attack)` → L63 `D(pc.getId())` → L64 `D(HitReactionSuppressTable.get().resolveTargetObjId(target.getId(), target))` → L65 `H(0x00)` → L66 `C(pc.getHeading())` → L68 `H(0x00)` → L69 `H(0x00)` → L70 `C(0x00)` |

`S_AttackPacketPc(L1PcInstance pc)`（L78）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L79 `C(S_OPCODE_ATTACKPACKET)` → L80 `C(ActionCodes.ACTION_Attack)` → L81 `D(pc.getId())` → L82 `D(0x00)` → L83 `H(0x00)` → L84 `C(pc.getHeading())` → L86 `H(0x00)` → L87 `H(0x00)` → L88 `C(0x00)` |

### S_AuctionBoard

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_AuctionBoard.java`；繼承 `ServerBasePacket`。

`S_AuctionBoard(L1NpcInstance board)`（L27）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 呼叫 `buildPacket(board)` |

`void buildPacket(L1NpcInstance board)`（L33）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L57 `C(S_OPCODE_HOUSELIST)` → L59 `D(board.getId())` → L60 `H(houseListX.size())` |
| `for (final L1AuctionBoardTmp boardX : houseListX)` | L62 `D(boardX.getHouseId())` → L63 `S(boardX.getHouseName())` → L64 `H(boardX.getHouseArea())` → L66 `C(cal.get(Calendar.MONTH) + 1)` → L67 `C(cal.get(Calendar.DATE))` → L68 `D((int) boardX.getPrice())` |

### S_AuctionBoardRead

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_AuctionBoardRead.java`；繼承 `ServerBasePacket`。

`S_AuctionBoardRead(int objectId, String house_number)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket(objectId, house_number)` |

`void buildPacket(int objectId, String house_number)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_SHOWHTML)` → L26 `D(objectId)` → L27 `S("agsel")` → L28 `S(house_number)` → L29 `H(0x0009)` → L30 `S(board.getHouseName())` → L31 `S(board.getLocation() + "$1195")` → L32 `S(String.valueOf(board.getHouseArea()))` → L33 `S(board.getOldOwner())` → L34 `S(board.getBidder())` → L35 `S(String.valueOf(board.getPrice()))` → L40 `S(String.valueOf(month))` → L41 `S(String.valueOf(day))` → L42 `S(String.valueOf(hour))` |

### S_AuctionShopRetrieve

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_AuctionShopRetrieve.java`；繼承 `ServerBasePacket`。

`S_AuctionShopRetrieve(int objid, L1PcInstance pc)`（L31）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L51 `C(S_OPCODE_SHOWRETRIEVELIST)` → L52 `D(objid)` → L53 `H(itemList.size())` → L54 `C(0x0c)` |
| `for (final L1ItemInstance item : itemList)` | L57 `D(item.getId())` → L62 `C(i)` → L63 `H(item.get_gfxid())` → L64 `C(L1Item.toPacketBless(item.getBless()))` → L65 `D((int) Math.min(item.getCount(), 2000000000))` → L66 `C(item.isIdentified() ? 0x01 : 0x00)` → L67 `S(item.getViewName())` |
| 本方法此處 | L71 `D(terms.fee())` |

### S_AuctionShopSellList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_AuctionShopSellList.java`；繼承 `ServerBasePacket`。

`S_AuctionShopSellList(int objid, L1PcInstance pc)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L41 呼叫 `buildPacket(objid, filteredList)` |

`S_AuctionShopSellList(int objid, List<AuctionShopItem> filteredList)`（L47）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L48 呼叫 `buildPacket(objid, filteredList)` |

`void buildPacket(int objid, List<AuctionShopItem> filteredList)`（L54）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L59 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L60 `D(objid)` |
| `if (filteredList == null \|\| filteredList.isEmpty())` | L63 `H(0x0000)` |
| 本方法此處 | L67 `H(filteredList.size())` |
| `for (final AuctionShopItem auction : filteredList)` | L72 `D(i)` → L80 `H(gfx)` → L81 `D(auction.getPrice())` → L100 `S(nameInfo)` |
| `for (final AuctionShopItem auction : filteredList) / if (ConfigOther.SHOPINFO && template != null) / if (tempItem != null)` | L136 `C(status.length)` |
| `for (final AuctionShopItem auction : filteredList) / if (ConfigOther.SHOPINFO && template != null) / if (tempItem != null) / for (final byte b : status)` | L138 `C(b)` |
| `for (final AuctionShopItem auction : filteredList) / if (ConfigOther.SHOPINFO && template != null) / else` | L141 `C(0x00)` |
| `for (final AuctionShopItem auction : filteredList) / if (ConfigOther.SHOPINFO && template != null)` | L144 `C(0x00)` |
| `for (final AuctionShopItem auction : filteredList) / else` | L147 `C(0x00)` |
| 本方法此處 | L153 `H(0x0007)` |

### S_BlueMessage

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_BlueMessage.java`；繼承 `ServerBasePacket`。

`S_BlueMessage(int type)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 呼叫 `buildPacket(type, null)` |

`S_BlueMessage(int type, String msg1)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 呼叫 `buildPacket(type, new String[]{ msg1 })` |

`S_BlueMessage(int type, String msg1, String msg2)`（L37）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 呼叫 `buildPacket(type, new String[]{ msg1, msg2 })` |

`S_BlueMessage(int type, String[] info)`（L47）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L48 呼叫 `buildPacket(type, info)` |

`void buildPacket(int type, String[] info)`（L52）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L53 `C(S_OPCODE_REDMESSAGE)` → L54 `H(type)` |
| `if (info == null)` | L56 `C(0x00)` |
| `else` | L59 `C(info.length)` |
| `else / for (int i = 0 ; i < info.length ; i++)` | L61 `S(info[i])` |

### S_Board

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Board.java`；繼承 `ServerBasePacket`。

`S_Board(L1NpcInstance npc)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(npc, 0)` |

`S_Board(L1NpcInstance npc, int number)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 呼叫 `buildPacket(npc, number)` |

`void buildPacket(L1NpcInstance npc, int number)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 `C(S_OPCODE_BOARD)` → L38 `C(0x00)` → L39 `D(npc.getId())` → L40 `C(0xff)` → L41 `C(0xff)` → L42 `C(0xff)` → L43 `C(0x7f)` → L44 `H(showList.size())` → L45 `H(0x012c)` |
| `for (int i = 0 ; i < showList.size() ; i++)` | L48 `D(boardInfo.id())` → L49 `S(boardInfo.name())` → L50 `S(boardInfo.date())` → L51 `S(boardInfo.title())` |

`S_Board(int objectid)`（L59）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L60 `C(S_OPCODE_BOARD)` → L61 `C(0x00)` → L62 `D(objectid)` → L63 `C(0xFF)` → L64 `C(0xFF)` → L65 `C(0xFF)` → L66 `C(0x7F)` → L67 `H(2)` → L68 `H(300)` → L70 `D(2)` → L71 `S("佈告欄列表")` → L72 `S("2010-2-2")` → L73 `S("佈告欄列表TITLE")` → L75 `D(1)` → L76 `S("佈告欄列表2")` → L77 `S("2010-2-3")` → L78 `S("佈告欄列表TITLE2")` |

### S_BoardRead

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_BoardRead.java`；繼承 `ServerBasePacket`。

`S_BoardRead(int number)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(number)` |

`void buildPacket(int number)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_BOARDREAD)` → L27 `D(board.get_id())` → L28 `S(board.get_name())` → L29 `S(board.get_date())` → L30 `S(board.get_title())` → L31 `S(board.get_content())` |

`S_BoardRead()`（L38）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L39 `C(S_OPCODE_BOARDREAD)` → L40 `D(10)` → L41 `S("測試NAME")` → L42 `S("2010-02-02")` → L43 `S("測試TITLE")` → L44 `S("測試內容")` |

### S_Bonusstats

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Bonusstats.java`；繼承 `ServerBasePacket`。

`S_Bonusstats(int objid)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 呼叫 `buildPacket(objid)` |

`void buildPacket(int objid)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_SHOWHTML)` → L23 `D(objid)` → L24 `S("RaiseAttr")` |

### S_BookmarkList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_BookmarkList.java`；繼承 `ServerBasePacket`。

`S_BookmarkList(List<L1BookMark> bookmarks)`（L37）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `C(S_OPCODE_CHARRESET)` → L39 `C(0x2a)` → L40 `H(0x0080)` → L41 `C(0x02)` |
| `for (int i = 1; i < 128; ++i)` | L44 `C(0xff)` |
| 本方法此處 | L47 `H(BookmarkService.MAX_BOOKMARKS)` → L48 `H(bookmarks.size())` |
| `for (L1BookMark book : bookmarks)` | L51 `D(book.getId())` → L52 `S(book.getName())` → L53 `H(book.getMapId())` → L54 `H(book.getLocX())` → L55 `H(book.getLocY())` |

### S_Bookmarks

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Bookmarks.java`；繼承 `ServerBasePacket`。

`S_Bookmarks(String name, int map, int x, int y, int id)`（L40）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L44 `C(S_OPCODE_BOOKMARKS)` → L45 `S(name)` → L46 `H(map)` → L47 `H(x)` → L48 `H(y)` → L49 `D(id)` |

### S_BoxChoice

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_BoxChoice.java`；繼承 `ServerBasePacket`。

`S_BoxChoice(int boxObjId, L1BoxSelect box)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L22 `D(boxObjId)` |
| `if (size <= 0)` | L26 `H(0x0000)` |
| 本方法此處 | L30 `H(size)` |
| `for (int i = 0; i < size; i++)` | L42 `D(i)` → L45 `H(template.getGfxId())` → L48 `D(0)` → L59 `S(name.toString())` |
| `for (int i = 0; i < size; i++) / if (status != null && status.length > 0)` | L77 `C(status.length)` |
| `for (int i = 0; i < size; i++) / if (status != null && status.length > 0) / for (byte b : status)` | L79 `C(b)` |
| `for (int i = 0; i < size; i++) / else` | L82 `C(0x00)` |
| 本方法此處 | L88 `H(0x0007)` |

### S_BoxMessage

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_BoxMessage.java`；繼承 `ServerBasePacket`。

`S_BoxMessage(String msg)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_NPCSHOUT)` → L18 `C(0x00)` → L19 `D(0x00000000)` → L20 `S(msg)` |

### S_BoxPack

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_BoxPack.java`；繼承 `ServerBasePacket`。

`S_BoxPack(L1NpcBoxInstance box)`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L16 呼叫 `buildPacket(box)` |

`void buildPacket(L1NpcBoxInstance box)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_CHARPACK)` → L21 `H(box.getX())` → L22 `H(box.getY())` → L23 `D(box.getId())` → L24 `H(box.getGfxId())` → L25 `C(box.getOpenStatus())` → L26 `C(0x00)` → L27 `C(0x00)` → L28 `C(0x00)` → L29 `D(0x00000001)` → L30 `H(0x0000)` → L31 `S(null)` → L32 `S(null)` → L33 `C(0x00)` → L34 `D(0x00000000)` → L35 `S(null)` → L36 `S(null)` → L38 `C(0x00)` → L40 `C(0xff)` → L41 `C(0x00)` → L42 `C(0x00)` → L43 `C(0x00)` → L44 `C(0xff)` → L45 `C(0xff)` → L46 `C(0x00)` |

### S_BroadcastPos

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_BroadcastPos.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`CUSTOM_MARKER=0xFF`、`SUB_BROADCAST_POS=0x08`。

`S_BroadcastPos(int posIdx)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 `C(S_OPCODE_PACKETBOX)` → L20 `C(CUSTOM_MARKER)` → L21 `C(SUB_BROADCAST_POS)` → L22 `C(posIdx & 0x07)` |

### S_Buddy

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Buddy.java`；繼承 `ServerBasePacket`。

`S_Buddy(int objId)`（L27）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L28 呼叫 `buildPacket(objId)` |

`void buildPacket(int objId)`（L32）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L33 `C(S_OPCODE_SHOWHTML)` → L34 `D(objId)` → L35 `S(_buddy)` → L36 `H(0x02)` → L37 `H(0x02)` → L59 `S(result)` → L60 `S(onlineBuddy)` |

### S_CastleMaster

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CastleMaster.java`；繼承 `ServerBasePacket`。

`S_CastleMaster(int type, int objecId)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket(type, objecId)` |

`void buildPacket(int type, int objecId)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `C(S_OPCODE_CASTLEMASTER)` → L24 `C(type)` → L25 `D(objecId)` |

### S_ChangeHeading

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChangeHeading.java`；繼承 `ServerBasePacket`。

`S_ChangeHeading(L1Character cha)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 呼叫 `buildPacket(cha)` |

`void buildPacket(L1Character cha)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_CHANGEHEADING)` → L25 `D(cha.getId())` → L26 `C(cha.getHeading())` |

### S_ChangeName

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChangeName.java`；繼承 `ServerBasePacket`。

`S_ChangeName(int objectId, String name)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_CHANGENAME)` → L21 `D(objectId)` → L22 `S(name)` |

`S_ChangeName(int objectId, String name, int mode)`（L32）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L69 `C(S_OPCODE_CHANGENAME)` → L70 `D(objectId)` → L71 `S(color + "GM \\f=" + name)` |

`S_ChangeName(L1PcInstance pc, boolean isName)`（L80）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L81 `C(S_OPCODE_CHANGENAME)` → L82 `D(pc.getId())` → L91 `S(stringBuilder.toString())` |

### S_ChangeShape

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChangeShape.java`；繼承 `ServerBasePacket`。

`S_ChangeShape(L1Character obj, int polyId)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 呼叫 `buildPacket(obj, polyId, false)` |

`S_ChangeShape(L1Character obj)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 呼叫 `buildPacket(obj, obj.getGfxId(), false)` |

`S_ChangeShape(L1Character obj, int polyId, boolean weaponTakeoff)`（L40）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L41 呼叫 `buildPacket(obj, polyId, weaponTakeoff)` |

`void buildPacket(L1Character obj, int polyId, boolean weaponTakeoff)`（L45）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L46 `C(S_OPCODE_POLY)` → L47 `D(obj.getId())` → L48 `H(polyId)` → L50 `H(weaponTakeoff ? 0 : 29)` |

`S_ChangeShape(L1PcInstance pc, L1NpcInstance npc, int polyId)`（L59）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L60 `C(S_OPCODE_POLY)` → L61 `D(npc.getId())` → L62 `D(pc.getId())` → L63 `H(polyId)` → L64 `S(pc.getName())` |

### S_CharAmount

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CharAmount.java`；繼承 `ServerBasePacket`。

`S_CharAmount(int value, ClientExecutor client)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 呼叫 `buildPacket(value, client)` |

`void buildPacket(int value, ClientExecutor client)`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 `C(S_OPCODE_CHARAMOUNT)` → L32 `C(value)` → L33 `C(maxAmount)` |

### S_CharCreateStatus

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CharCreateStatus.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`REASON_OK=0x02`、`REASON_ALREADY_EXSISTS=0x06`、`REASON_INVALID_NAME=0x09`、`REASON_WRONG_AMOUNT=0x15`。

`S_CharCreateStatus(int reason)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_NEWCHARWRONG)` → L27 `C(reason)` |

### S_CharPacks

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CharPacks.java`；繼承 `ServerBasePacket`。

`S_CharPacks(String name, String clanName, int type, int sex, int lawful, int hp, int mp, int ac, int lv, int str, int dex, int con, int wis, int cha, int intel, int time)`（L32）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L35 `C(S_OPCODE_CHARLIST)` → L36 `S(name)` → L37 `S(clanName)` → L38 `C(type)` → L39 `C(sex)` → L40 `H(lawful)` |
| `if (ConfigClient.isROClient())` | L44 `D(hp)` → L45 `D(mp)` |
| `else` | L47 `H(hp)` → L48 `H(mp)` |
| `if (ac > 0x0a)` | L52 `C(0x0a)` |
| `else` | L55 `C(ac)` |
| `if (lv > 0x7f)` | L59 `C(0x7f)` |
| `else` | L62 `C(lv)` |
| `if (str > 0x7f)` | L66 `C(0x7f)` |
| `else` | L68 `C(str)` |
| `if (dex > 0x7f)` | L71 `C(0x7f)` |
| `else` | L73 `C(dex)` |
| `if (con > 0x7f)` | L76 `C(0x7f)` |
| `else` | L78 `C(con)` |
| `if (wis > 0x7f)` | L81 `C(0x7f)` |
| `else` | L83 `C(wis)` |
| `if (cha > 0x7f)` | L86 `C(0x7f)` |
| `else` | L88 `C(cha)` |
| `if (intel > 0x7f)` | L91 `C(0x7f)` |
| `else` | L93 `C(intel)` |
| 本方法此處 | L97 `C(0x00)` → L98 `D(time)` → L100 `C(checkcode & 0xFF)` |

### S_CharReset

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CharReset.java`；繼承 `ServerBasePacket`。

`S_CharReset(L1PcInstance pc, int lv, int hp, int mp, int ac, int str, int intel, int wis, int dex, int con, int cha)`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_CHARRESET)` → L18 `C(0x02)` → L19 `C(lv)` → L20 `C(pc.getTempMaxLevel())` → L21 `H(hp)` → L22 `H(mp)` → L23 `H(ac)` → L24 `C(str)` → L25 `C(intel)` → L26 `C(wis)` → L27 `C(dex)` → L28 `C(con)` → L29 `C(cha)` |

`S_CharReset(int point)`（L33）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L34 `C(S_OPCODE_CHARRESET)` → L35 `C(0x03)` → L36 `C(point)` |

`S_CharReset(int pcObjId, int emblemId)`（L44）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L45 `C(S_OPCODE_CHARRESET)` → L46 `C(0x3c)` → L47 `D(pcObjId)` → L48 `D(emblemId)` |

`S_CharReset(L1PcInstance pc)`（L55）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L56 `C(S_OPCODE_CHARRESET)` → L57 `C(0x01)` |
| `if (pc.isCrown())` | L59 `H(14)` → L60 `H(2)` |
| `else if (pc.isKnight())` | L63 `H(16)` → L64 `H(1)` |
| `else if (pc.isElf())` | L67 `H(15)` → L68 `H(4)` |
| `else if (pc.isWizard())` | L71 `H(12)` → L72 `H(6)` |
| `else if (pc.isDarkelf())` | L75 `H(12)` → L76 `H(3)` |
| `else if (pc.isDragonKnight())` | L79 `H(15)` → L80 `H(4)` |
| `else if (pc.isIllusionist())` | L83 `H(15)` → L84 `H(4)` |
| 本方法此處 | L86 `C(0x0a)` → L87 `C(pc.getTempMaxLevel())` |

### S_CharResetInfo

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CharResetInfo.java`；繼承 `ServerBasePacket`。

`S_CharResetInfo(L1PcInstance pc)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L40 `C(S_OPCODE_CHARRESET)` → L41 `C(0x04)` → L42 `C((upInt << 4) + upStr)` → L43 `C((upDex << 4) + upWis)` → L44 `C((upCha << 4) + upCon)` → L45 `C(0x00)` → L46 `H(0x00)` |

### S_CharTitle

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CharTitle.java`；繼承 `ServerBasePacket`。

`S_CharTitle(int objid, StringBuilder title)`（L28）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 `C(S_OPCODE_CHARTITLE)` → L30 `D(objid)` → L31 `S(title.toString())` |

`S_CharTitle(int objid)`（L40）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L41 `C(S_OPCODE_CHARTITLE)` → L42 `D(objid)` → L43 `S("")` |

### S_CharVisualUpdate

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CharVisualUpdate.java`；繼承 `ServerBasePacket`。

`S_CharVisualUpdate(int objid, int weaponType)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_CHARVISUALUPDATE)` → L21 `D(objid)` → L22 `C(weaponType)` |

`S_CharVisualUpdate(L1PcInstance cha)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 `C(S_OPCODE_CHARVISUALUPDATE)` → L32 `D(cha.getId())` → L33 `C(cha.getCurrentWeapon())` |

### S_Chat

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Chat.java`；繼承 `ServerBasePacket`。

`S_Chat(L1PcInstance pc, String chat)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 呼叫 `buildPacket(pc, chat)` |

`void buildPacket(L1PcInstance pc, String chat)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 `C(S_OPCODE_NORMALCHAT)` → L28 `C(0x00)` → L29 `D(pc.isInvisble()? 0 : pc.getId())` → L30 `S(pc.getName() + ": " + chat)` |

`S_Chat(L1NpcInstance npc, String chat)`（L38）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L39 `C(S_OPCODE_NORMALCHAT)` → L40 `C(0x00)` → L41 `D(npc.isInvisble()? 0 : npc.getId())` → L42 `S(npc.getNameId() + ": " + chat)` |

`S_Chat(L1Object object, String chat, int x)`（L46）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L47 `C(S_OPCODE_NORMALCHAT)` → L48 `C(0x00)` → L49 `D(object.getId())` → L50 `S(chat)` |

### S_ChatClan

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChatClan.java`；繼承 `ServerBasePacket`。

`S_ChatClan(L1PcInstance pc, String chat)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(pc, chat)` |

`void buildPacket(L1PcInstance pc, String chat)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_GLOBALCHAT)` → L26 `C(0x04)` → L27 `S("{" + pc.getName() + "} " + chat)` |

### S_ChatClanUnion

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChatClanUnion.java`；繼承 `ServerBasePacket`。

`S_ChatClanUnion(L1PcInstance pc, String chat)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(pc, chat)` |

`void buildPacket(L1PcInstance pc, String chat)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_GLOBALCHAT)` → L26 `C(0x04)` → L27 `S("{{" + pc.getName() + "}} " + chat)` |

### S_ChatGlobal

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChatGlobal.java`；繼承 `ServerBasePacket`。

`S_ChatGlobal(L1PcInstance pc, String chat)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 呼叫 `buildPacket(pc, chat)` |

`void buildPacket(L1PcInstance pc, String chat)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 `C(S_OPCODE_GLOBALCHAT)` → L31 `C(0x03)` → L32 `S(pc.isGm()? "[******] " + chat : "[" + pc.getName() + "] " + chat)` |

`S_ChatGlobal(L1NpcInstance npc, String chat)`（L40）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L41 `C(S_OPCODE_GLOBALCHAT)` → L42 `C(0x03)` → L48 `S("[" + (nameId != null ? nameId : "") + "] " + chat)` |

`S_ChatGlobal(String chat)`（L57）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L58 `C(S_OPCODE_GLOBALCHAT)` → L59 `C(0x03)` → L60 `S(chat)` |

### S_ChatOut

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChatOut.java`；繼承 `ServerBasePacket`。

`S_ChatOut(int objid)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 呼叫 `buildPacket(objid)` |

`S_ChatOut(L1PcInstance pc)`（L27）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L28 呼叫 `buildPacket(pc.getId())` |

`void buildPacket(int objid)`（L32）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L34 `D(objid)` → L35 `D(0x00000000)` → L36 `D(0x00000000)` → L37 `D(0x00000000)` |

### S_ChatParty

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChatParty.java`；繼承 `ServerBasePacket`。

`S_ChatParty(L1PcInstance pc, String chat)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(pc, chat)` |

`void buildPacket(L1PcInstance pc, String chat)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_GLOBALCHAT)` → L26 `C(0x0b)` → L27 `S("(" + pc.getName() + ") " + chat)` |

### S_ChatParty2

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChatParty2.java`；繼承 `ServerBasePacket`。

`S_ChatParty2(L1PcInstance pc, String chat)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(pc, chat)` |

`void buildPacket(L1PcInstance pc, String chat)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_NORMALCHAT)` → L26 `C(0x0e)` → L27 `D(pc.isInvisble()? 0 : pc.getId())` → L28 `S("(" + pc.getName() + ") " + chat)` |

### S_ChatShouting

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChatShouting.java`；繼承 `ServerBasePacket`。

`S_ChatShouting(L1PcInstance pc, String chat)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 呼叫 `buildPacket(pc, chat)` |

`void buildPacket(L1PcInstance pc, String chat)`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_NORMALCHAT)` → L27 `C(0x02)` → L28 `D(pc.isInvisble()? 0 : pc.getId())` → L29 `S("<" + pc.getName() + "> " + chat)` → L31 `H(pc.getX())` → L32 `H(pc.getY())` |

`S_ChatShouting(L1NpcInstance npc, String chat)`（L40）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L41 `C(S_OPCODE_NORMALCHAT)` → L42 `C(0x02)` → L43 `D(npc.isInvisble()? 0 : npc.getId())` → L44 `S("<" + npc.getNameId() + "> " + chat)` → L46 `H(npc.getX())` → L47 `H(npc.getY())` |

### S_ChatTransaction

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChatTransaction.java`；繼承 `ServerBasePacket`。

`S_ChatTransaction(L1PcInstance pc, String chat)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 呼叫 `buildPacket(pc, chat)` |

`void buildPacket(L1PcInstance pc, String chat)`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_GLOBALCHAT)` → L27 `C(0x0c)` → L28 `S("[" + pc.getName() + "] " + chat)` |

`S_ChatTransaction(L1NpcInstance npc, String chat)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 `C(S_OPCODE_GLOBALCHAT)` → L38 `C(0x0c)` → L39 `S("[" + npc.getNameId() + "] " + chat)` |

### S_ChatWhisperFrom

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChatWhisperFrom.java`；繼承 `ServerBasePacket`。

`S_ChatWhisperFrom(L1PcInstance pc, String chat)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(pc, chat)` |

`void buildPacket(L1PcInstance pc, String chat)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_WHISPERCHAT)` → L26 `S(pc.getName())` → L27 `S(chat)` |

### S_ChatWhisperTo

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ChatWhisperTo.java`；繼承 `ServerBasePacket`。

`S_ChatWhisperTo(L1PcInstance pc, String chat)`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L16 呼叫 `buildPacket(pc, chat)` |

`void buildPacket(L1PcInstance pc, String chat)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 `C(S_OPCODE_GLOBALCHAT)` → L22 `C(0x09)` → L23 `S("-> (" + pc.getName() + ") " + chat)` |

`S_ChatWhisperTo(L1DeInstance de, String chat)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 `C(S_OPCODE_GLOBALCHAT)` → L28 `C(0x09)` → L29 `S("-> (" + de.getNameId() + ") " + chat)` |

### S_ClanAttention

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClanAttention.java`；繼承 `ServerBasePacket`。

`S_ClanAttention(L1Clan clan, int type)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(clan, type)` |

`S_ClanAttention()`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_CLANATTENTION)` → L27 `H(2)` → L28 `D(0)` → L29 `H(0)` |

`S_ClanAttention(List<String> names)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L35 `C(S_OPCODE_CLANATTENTION)` → L36 `H(2)` |
| `if (names == null \|\| names.isEmpty())` | L38 `D(0)` |
| `else` | L40 `D(names.size())` |
| `else / for (final String name : names)` | L42 `S(name)` |
| 本方法此處 | L45 `H(0)` |

`void buildPacket(L1Clan clan, int type)`（L49）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L50 `C(S_OPCODE_CLANATTENTION)` → L51 `H(type)` |
| `if (type == 2) / if (clan == null)` | L54 `D(0)` |
| `if (type == 2) / else` | L58 `D(size)` |
| `if (type == 2) / else / for (int i = 0; i < size; i++)` | L60 `S(list.get(i))` |
| 本方法此處 | L64 `H(0)` |

### S_ClanUpdate

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClanUpdate.java`；繼承 `ServerBasePacket`。

`S_ClanUpdate(int objid, int clanid,String Clanname, int rank)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 `C(S_OPCODE_UPDATECLANID)` → L22 `D(objid)` → L23 `S(Clanname)` → L24 `S(null)` → L25 `D(0x00000000)` → L26 `C(rank)` |

`S_ClanUpdate(int objid)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 `C(S_OPCODE_UPDATECLANID)` → L38 `D(objid)` → L39 `S(null)` → L40 `S(null)` → L41 `D(0x00000000)` → L42 `C(0)` |

### S_ClassicBadge

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClassicBadge.java`；繼承 `ServerBasePacket`。

`S_ClassicBadge(int objectId, int badgeId)`（L6）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L6 呼叫 `this(1, objectId, badgeId)` |

`S_ClassicBadge(int command, int objectId, int badgeId)`（L7）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L10 `C(S_OPCODE_PACKETBOX)` → L10 `C(0xff)` → L10 `C(0x34)` → L10 `D(0x31474442)` → L11 `C(1)` → L11 `C(command)` → L11 `D(objectId)` → L11 `D(badgeId)` |
| `for (int n=0;n<7;n++)` | L12 `C(0)` |

`static S_ClassicBadge reset()`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L15 呼叫 `S_ClassicBadge(0, 0, 0)` |

### S_ClassicBagCombine

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClassicBagCombine.java`；繼承 `ServerBasePacket`。

`S_ClassicBagCombine(int command,int nonce,int request,int token,UiOperationContext.Offer offer)`（L10）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L14 `C(S_OPCODE_PACKETBOX)` → L14 `C(0xff)` → L14 `C(0x33)` → L14 `D(0x47414243)` → L15 `C(2)` → L15 `C(command)` → L15 `D(nonce)` → L15 `D(request)` → L15 `D(token)` |

### S_ClassicBoss

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClassicBoss.java`；繼承 `ServerBasePacket`。

`S_ClassicBoss(int command,int nonce,int player,int token,int prior,int npc,int total,int start,List<Row> rows)`（L11）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L15 `C(S_OPCODE_PACKETBOX)` → L15 `C(255)` → L15 `C(0x37)` → L15 `D(0x31534243)` → L15 `C(2)` → L15 `C(command)` → L16 `D(nonce)` → L16 `D(player)` → L16 `D(token)` → L16 `D(prior)` → L16 `D(npc)` → L16 `D(total)` → L16 `D(start)` → L16 `H(rows.size())` |
| `for(var r:rows)` | L17 `D(r.id())` → L17 `C(r.state())` → L17 呼叫 `text(r.name(),96)` → L17 呼叫 `text(r.status(),96)` |

`void text(String text,int max)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `H(bytes.length)` |
| `for (byte b:bytes)` | L23 `C(b&255)` |

### S_ClassicCollection

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClassicCollection.java`；繼承 `ServerBasePacket`。

`S_ClassicCollection(int command,int nonce,int player,int token,int npc,int serial,int version, int page,int total,int selected,int flags,int materialTotal,int offset, List<Row> rows,List<String> lines,List<Material> materials)`（L11）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 `C(S_OPCODE_PACKETBOX)` → L18 `C(255)` → L18 `C(0x39)` → L18 `D(0x314c4f43)` → L18 `C(1)` → L18 `C(command)` → L19 `D(nonce)` → L19 `D(player)` → L19 `D(token)` → L19 `D(npc)` → L19 `D(serial)` → L19 `D(version)` → L20 `D(page)` → L20 `D(total)` → L20 `D(selected)` → L20 `C(flags)` → L20 `H(materialTotal)` → L20 `H(offset)` → L21 `C(rows.size())` → L21 `C(lines.size())` → L21 `C(materials.size())` |
| `for(var r:rows)` | L22 `D(r.quest)` → L22 `C(r.state)` → L22 呼叫 `text(r.name,96)` |
| `for (String line:lines)` | L23 呼叫 `text(line,120)` |
| `for(var m:materials)` | L25 `D(m.item)` → L25 `H(m.icon)` → L25 `D(m.required)` → L25 `D((int)Math.min(Integer.MAX_VALUE,m.available))` → L25 `D(m.enchant)` → L25 呼叫 `text(m.name,144)` |

`void text(String s,int max)`（L28）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 `H(b.length)` |
| `for (byte v:b)` | L31 `C(v&255)` |

`List<ServerBasePacket> fragments()`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 呼叫 `Chunk(content,i,count)` |

`Chunk(byte[] whole,int index,int count)`（L42）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (int i=0;i<33;i++)` | L43 `C(i==8?4:whole[i]&255)` |
| 本方法此處 | L44 `H(index)` → L44 `H(count)` → L44 `H(whole.length)` → L44 `H(n)` |
| `for (int i=start;i<start+n;i++)` | L45 `C(whole[i]&255)` |

### S_ClassicCraft

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClassicCraft.java`；繼承 `ServerBasePacket`。

`void header(int command,int nonce)`（L14）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L15 `C(S_OPCODE_PACKETBOX)` → L15 `C(0xff)` → L15 `C(0x32)` → L15 `D(0x46524343)` → L16 `C(1)` → L16 `C(command)` → L16 `D(nonce)` |

`void text(String text)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 `H(bytes.length)` |
| `for (byte b:bytes)` | L21 `C(b&255)` |

`void item(Item item)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `D(item.id())` → L24 `D(item.icon())` → L24 `D(item.enchant())` → L24 `D(item.count())` → L24 `D(item.owned())` → L24 呼叫 `text(item.name())` |

`S_ClassicCraft(int command,int nonce,int token)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L28 呼叫 `header(command,nonce)` → L28 `D(token)` |

`S_ClassicCraft(Preview p,UiOperationContext.Ticket prerequisite)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L33 呼叫 `header(1,p.nonce())` → L33 `D(p.token())` → L33 `D(p.priorToken())` → L33 `D(p.npc())` → L33 `D(p.recipe())` → L33 `D(p.maximum())` → L34 `H(p.level())` → L34 `H(p.rate())` → L34 `C(p.eligible()?1:0)` → L35 `C(p.materials().size())` → L35 `C(p.products().size())` → L35 呼叫 `text("全職業")` |
| `for (Item item:p.materials())` | L36 呼叫 `item(item)` |
| `for (Item item:p.products())` | L37 呼叫 `item(item)` |

### S_ClassicInventoryFilter

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClassicInventoryFilter.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`MAX_TARGETS=2048`。

`S_ClassicInventoryFilter(int command, int request, List<Integer> targets)`（L10）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L13 `C(S_OPCODE_PACKETBOX)` → L13 `C(0xff)` → L13 `C(0x31)` → L14 `D(0x46495543)` → L15 `C(1)` → L15 `C(command)` → L15 `D(request)` → L15 `H(targets.size())` |
| `for (final int target : targets)` | L18 `D(target)` |

### S_ClassicPass

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClassicPass.java`；繼承 `ServerBasePacket`。

`S_ClassicPass(int nonce,int player,int sequence,int level,boolean premium,int costItem,int costCount,String costName,List<Row> rows)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 呼叫 `this(nonce,player,sequence,level,premium,costItem,costCount,costName,rows,null,null)` |

`S_ClassicPass(int nonce,int player,int sequence,int level,boolean premium,int costItem,int costCount,String costName,List<Row> rows, UiOperationContext.Offer offer,UiOperationContext.Ticket prerequisite)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `C(S_OPCODE_PACKETBOX)` → L23 `C(255)` → L23 `C(0x35)` → L23 `D(0x31535043)` → L23 `C(offer==null?1:2)` → L24 `D(nonce)` → L24 `D(player)` → L24 `D(sequence)` |
| `if(offer!=null)` | L25 `D(offer.originId())` → L26 `D(prerequisite!=null&&prerequisite.offer().subject() instanceof com.lineage.server.model.ClassicPassService.Session old?old.token():0)` |
| 本方法此處 | L28 `H(level)` → L28 `C(premium?1:0)` → L28 `C(rows.size())` → L29 `D(costItem)` → L29 `D(costCount)` → L29 呼叫 `text(costName)` |
| `for(Row row:rows)` | L31 `D(row.id())` → L31 `H(row.level())` → L31 `C(row.gold().size())` → L31 呼叫 `item(row.ordinary())` |
| `for(Row row:rows) / for (Item i:row.gold())` | L32 呼叫 `item(i)` |

`void item(Item i)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `D(i.id())` → L38 `H(i.icon())` → L38 `D(i.count())` → L38 `C(i.state())` → L38 呼叫 `text(i.name())` |

`void text(String value)`（L40）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L44 `H(raw.length)` |
| `for (byte b:raw)` | L44 `C(b&255)` |

### S_ClassicQuest

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClassicQuest.java`；繼承 `ServerBasePacket`。

`S_ClassicQuest(int command,int nonce,int player,int token,int prior,int npc,int page,int total, int selected,int state,int permissions,List<Row> rows,List<Reward> rewards,List<String> lines)`（L11）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L15 `C(S_OPCODE_PACKETBOX)` → L15 `C(255)` → L15 `C(0x36)` → L15 `D(0x31535143)` → L15 `C(2)` → L15 `C(command)` → L16 `D(nonce)` → L16 `D(player)` → L16 `D(token)` → L16 `D(prior)` → L16 `D(npc)` → L16 `D(page)` → L16 `D(total)` → L16 `D(selected)` → L17 `C(state)` → L17 `C(permissions)` → L17 `C(rows.size())` → L17 `H(rewards.size())` → L17 `H(lines.size())` |
| `for(var r:rows)` | L18 `D(r.id())` → L18 `C(r.state())` → L18 呼叫 `text(r.name())` |
| `for(var r:rewards)` | L19 `D(r.id())` → L19 `H(r.icon())` → L19 `D(r.count())` → L19 呼叫 `text(r.name())` |
| `for (var line:lines)` | L20 呼叫 `text(line)` |

`void text(String text)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 `H(raw.length)` |
| `for (byte b:raw)` | L27 `C(b&255)` |

### S_ClassicRanking

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClassicRanking.java`；繼承 `ServerBasePacket`。

`S_ClassicRanking(int command,int nonce,int player,int token,int npc,int serial,int version,int board,int job,List<ClassicRankingModel.Row> rows)`（L10）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_PACKETBOX)` → L17 `C(255)` → L17 `C(0x38)` → L17 `D(0x314b5243)` → L17 `C(1)` → L17 `C(command)` → L18 `D(nonce)` → L18 `D(player)` → L18 `D(token)` → L18 `D(npc)` → L18 `D(serial)` → L18 `D(version)` → L18 `C(board)` → L18 `C(job)` → L18 `C(rows.size())` |
| `for(var r:rows)` | L22 `D(r.id())` → L22 `C(r.job())` → L22 `D(r.value())` → L22 呼叫 `text(r.name(),96,false)` → L22 呼叫 `text(r.item(),192,board==0\|\|board==4)` |

`void text(String s,int max,boolean empty)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 `H(b.length)` |
| `for (byte v:b)` | L29 `C(v&255)` |

### S_ClassicReward

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ClassicReward.java`；繼承 `ServerBasePacket`。

`S_ClassicReward(int command,int nonce,int player,int token,int npc,int serial,int version,int page,int total,int claimedTotal,List<ClassicRewardProtocol.Row> rows)`（L8）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_PACKETBOX)` → L17 `C(255)` → L17 `C(0x3a)` → L17 `D(0x31445752)` → L17 `C(4)` → L17 `C(command)` → L18 `D(nonce)` → L18 `D(player)` → L18 `D(token)` → L18 `D(npc)` → L18 `D(serial)` → L18 `D(version)` → L18 `D(page)` → L18 `D(total)` → L18 `D(claimedTotal)` → L18 `C(rows.size())` |
| `for(var r:rows)` | L22 `D(r.id())` → L22 `H(r.icon())` → L22 `D(r.count())` → L22 `D(r.enchant())` → L22 `C(r.claimed()?1:0)` → L22 `H(text.length)` |
| `for(var r:rows) / for (byte v:text)` | L22 `C(v&255)` |

### S_CloseList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CloseList.java`；繼承 `ServerBasePacket`。

`S_CloseList(int objid)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_SHOWHTML)` → L21 `D(objid)` → L22 `S("")` |

### S_CommonInfo

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CommonInfo.java`；繼承 `ServerBasePacket`。

`S_CommonInfo(int type, String[] info)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 呼叫 `buildPacket(type, info)` |

`void buildPacket(int type, String[] info)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_COMMONNEWS)` → L23 `H(type)` |
| `if (info == null)` | L26 `C(0x00)` |
| `else` | L29 `C(info.length)` |
| `else / for (int i = 0 ; i < info.length ; i++)` | L31 `S(info[i])` |

### S_CommonNews

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CommonNews.java`；繼承 `ServerBasePacket`。

`S_CommonNews()`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_COMMONNEWS)` → L27 `S(messagePack.toString())` |

`S_CommonNews(String s)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L36 `C(S_OPCODE_COMMONNEWS)` → L37 `S(s)` |

### S_CurseBlind

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_CurseBlind.java`；繼承 `ServerBasePacket`。

`S_CurseBlind(int type)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 呼叫 `buildPacket(type)` |

`void buildPacket(int type)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_CURSEBLIND)` → L23 `H(type)` |

### S_DeleteCharOK

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_DeleteCharOK.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`DELETE_CHAR_NOW=0x05`、`DELETE_CHAR_AFTER_7DAYS=0x51`。

`S_DeleteCharOK(int type)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_DETELECHAROK)` → L23 `C(type)` |

### S_DeleteInventoryItem

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_DeleteInventoryItem.java`；繼承 `ServerBasePacket`。

`S_DeleteInventoryItem(L1ItemInstance item)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_DELETEINVENTORYITEM)` → L21 `D(item.getId())` |

`S_DeleteInventoryItem(int objid)`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_DELETEINVENTORYITEM)` → L27 `D(objid)` |

### S_DelSkill

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_DelSkill.java`；繼承 `ServerBasePacket`。

`S_DelSkill(L1PcInstance pc, int skillid)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L34 `C(S_OPCODE_DELSKILL)` → L35 `C(skill_list_values)` |
| `for (final int element : skillList)` | L38 `C(element)` |

`S_DelSkill(L1PcInstance pc, int level1, int level2, int level3, int level4, int level5, int level6, int level7, int level8, int level9, int level10, int knight1, int knight2, int de1, int de2, int royal, int un, int elf1, int elf2, int elf3, int elf4, int elf5, int elf6, int k1, int k2, int k3, int l1, int l2, int l3 )`（L87）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L102 `C(S_OPCODE_DELSKILL)` |
| `if ((i6 > 0x00) && (j6 == 0x00))` | L105 `C(0x32)` |
| `else if (j6 > 0x00)` | L108 `C(0x64)` |
| `else` | L111 `C(0x20)` |
| 本方法此處 | L114 `C(level1)` → L115 `C(level2)` → L116 `C(level3)` → L117 `C(level4)` → L118 `C(level5)` → L119 `C(level6)` → L120 `C(level7)` → L121 `C(level8)` → L122 `C(level9)` → L123 `C(level10)` → L125 `C(knight1)` → L126 `C(knight2)` → L128 `C(de1)` → L129 `C(de2)` → L131 `C(royal)` → L133 `C(un)` → L135 `C(elf1)` → L136 `C(elf2)` → L137 `C(elf3)` → L138 `C(elf4)` → L139 `C(elf5)` → L140 `C(elf6)` → L142 `C(k1)` → L143 `C(k2)` → L144 `C(k3)` → L146 `C(l1)` → L147 `C(l2)` → L148 `C(l3)` |

### S_Deposit

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Deposit.java`；繼承 `ServerBasePacket`。

`S_Deposit(int objecId)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 呼叫 `this(objecId, null, null)` |

`S_Deposit(int objecId, Offer offer, Ticket prerequisite)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 `C(S_OPCODE_DEPOSIT)` → L30 `D(objecId)` |

### S_Dexup

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Dexup.java`；繼承 `ServerBasePacket`。

`S_Dexup(L1PcInstance pc, int type, int time)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_DEXUP)` → L23 `H(time)` → L24 `C(pc.getDex())` → L25 `C(type)` |

### S_Disconnect

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Disconnect.java`；繼承 `ServerBasePacket`。

`S_Disconnect()`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 `C(S_OPCODE_DISCONNECT)` → L19 `H(0x01f4)` → L20 `D(0x00000000)` |

### S_DoActionGFX

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_DoActionGFX.java`；繼承 `ServerBasePacket`。

`S_DoActionGFX(int objectId, int actionId)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_DOACTIONGFX)` → L21 `D(objectId)` → L22 `C(actionId)` |

### S_DoActionShop

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_DoActionShop.java`；繼承 `ServerBasePacket`。

`S_DoActionShop(int object, byte[] message)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 `C(S_OPCODE_DOACTIONGFX)` → L19 `D(object)` → L20 `C(ActionCodes.ACTION_Shop)` → L21 `Byte(message)` |

`S_DoActionShop(int object, String message)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 `C(S_OPCODE_DOACTIONGFX)` → L32 `D(object)` → L33 `C(ActionCodes.ACTION_Shop)` → L34 `S(message)` |

### S_Door

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Door.java`；繼承 `ServerBasePacket`。

`S_Door(L1DoorInstance door)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 呼叫 `buildPacket(door.getEntranceX(), door.getEntranceY(), door.getDirection(), door.getPassable())` |

`S_Door(int x, int y, int direction, int passable)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 呼叫 `buildPacket(x, y, direction, passable)` |

`void buildPacket(int x, int y, int direction, int passable)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L36 `C(S_OPCODE_ATTRIBUTE)` → L37 `H(x)` → L38 `H(y)` → L39 `C(direction)` → L40 `C(passable)` |

### S_DoorPack

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_DoorPack.java`；繼承 `ServerBasePacket`。

`S_DoorPack(L1DoorInstance door)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(door)` |

`void buildPacket(L1DoorInstance door)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_CHARPACK)` → L26 `H(door.getX())` → L27 `H(door.getY())` → L28 `D(door.getId())` → L29 `H(door.getGfxId())` |
| `if (door.isDead())` | L34 `C(doorStatus)` |
| `else if (openStatus == ActionCodes.ACTION_Open)` | L37 `C(openStatus)` |
| `else if ((door.getMaxHp() > 1) && (doorStatus != 0))` | L40 `C(doorStatus)` |
| `else` | L43 `C(openStatus)` |
| 本方法此處 | L45 `C(0x00)` → L46 `C(0x00)` → L47 `C(0x00)` → L48 `D(0x00000001)` → L49 `H(0x0000)` → L50 `S(null)` → L51 `S(null)` → L52 `C(0x00)` → L53 `D(0x00000000)` → L54 `S(null)` → L55 `S(null)` → L57 `C(0x00)` → L59 `C(0xff)` → L60 `C(0x00)` → L61 `C(0x00)` → L62 `C(0x00)` → L63 `C(0xff)` → L64 `C(0xff)` |

### S_Drawal

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Drawal.java`；繼承 `ServerBasePacket`。

`S_Drawal(int objectId, long count)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 呼叫 `this(objectId, count, null, null)` |

`S_Drawal(int objectId, long count, Offer offer, Ticket prerequisite)`（L27）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 `C(S_OPCODE_DRAWAL)` → L31 `D(objectId)` → L32 `D((int) Math.min(count, 2000000000))` |

### S_EffectLocation

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_EffectLocation.java`；繼承 `ServerBasePacket`。

`S_EffectLocation(Point pt, int gfxId)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 呼叫 `this(pt.getX(), pt.getY(), gfxId)` |

`S_EffectLocation(L1Location loc, int gfxId)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 呼叫 `this(loc.getX(), loc.getY(), gfxId)` |

`S_EffectLocation(int x, int y, int gfxId)`（L51）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L53 `C(S_OPCODE_EFFECTLOCATION)` → L54 `H(x)` → L55 `H(y)` → L56 `H(gfxId)` |

`S_EffectLocation(int opid, L1Location loc)`（L65）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L66 `C(opid)` → L67 `H(loc.getX())` → L68 `H(loc.getY())` → L69 `H(4842)` |

### S_Emblem

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Emblem.java`；繼承 `ServerBasePacket`。

`S_Emblem(int clanid)`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_EMBLEM)` → L23 `D(clanid)` |
| `for (int i = 0 ; i < icon.length ; i++)` | L26 `P(icon[i])` |

`S_Emblem(L1EmblemIcon emblemIcon)`（L31）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L33 `C(S_OPCODE_EMBLEM)` → L34 `D(emblemIcon.get_clanid())` → L35 `Byte(emblemIcon.get_clanIcon())` |

`S_Emblem(int clanid, byte[] clanIcon)`（L43）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L44 `C(S_OPCODE_EMBLEM)` → L45 `D(clanid)` → L46 `Byte(clanIcon)` |

### S_EnterGame

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_EnterGame.java`；繼承 `ServerBasePacket`。

`S_EnterGame()`（L13）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L15 `C(S_OPCODE_LOGINTOGAME)` → L16 `C(0x03)` → L17 `C(0x15)` → L18 `C(0x8b)` → L19 `C(0x7b)` → L20 `C(0x94)` → L21 `C(0xf0)` → L22 `C(0x2f)` |

### S_EquipmentWindow

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_EquipmentWindow.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`TYPE_1=65`、`TYPE_2=66`、`TYPE_3=67`、`TYPE_4=68`、`TYPE_FIRE_REFINE=48`、`TYPE_FIRE_CRAFT=49`。

`S_EquipmentWindow(int itemObjId, int index, boolean equipped)`（L44）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L46 `C(S_OPCODE_CHARRESET)` → L47 `C(TYPE_2)` → L48 `D(itemObjId)` → L49 `C(index)` → L50 `Boolean(equipped)` |

`S_EquipmentWindow(List<L1ItemInstance> items)`（L59）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L60 `C(S_OPCODE_CHARRESET)` → L61 `C(TYPE_1)` → L62 `C(items.size())` |
| `for (final L1ItemInstance item : items)` | L65 `D(item.getId())` → L66 `H(item.getEquipWindow())` → L67 `H(0x00)` |

`S_EquipmentWindow(int type, int count, int type2)`（L78）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L79 `C(S_OPCODE_CHARRESET)` → L80 `C(type)` → L81 `D(count)` → L84 `D(type2)` → L85 `D(type2)` → L86 `D(type2)` → L87 `D(type2)` |

`S_EquipmentWindow(int size)`（L95）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L96 `C(S_OPCODE_CHARRESET)` → L97 `C(TYPE_4)` → L98 `D(size)` |

`S_EquipmentWindow(int smithUiType, int npcObjId)`（L108）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L111 `C(S_OPCODE_CHARRESET)` → L112 `C(smithUiType)` → L113 `D(npcObjId)` → L114 `C(0x95)` → L115 `C(0x19)` |

### S_Exp

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Exp.java`；繼承 `ServerBasePacket`。

`S_Exp(L1PcInstance pc)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_EXP)` → L21 `C(pc.getLevel())` → L23 `Exp(pc.getExp())` → L24 `C(0x01)` |

`S_Exp()`（L33）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L34 `C(S_OPCODE_EXP)` → L35 `C(59)` → L36 `D(414931028)` → L37 `C(0x01)` |

### S_Fishing

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Fishing.java`；繼承 `ServerBasePacket`。

`S_Fishing(int objectId, int motionNum, int x, int y)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 呼叫 `buildPacket(objectId, motionNum, x, y)` |

`void buildPacket(int objectId, int motionNum, int x, int y)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 `C(S_OPCODE_DOACTIONGFX)` → L28 `D(objectId)` → L29 `C(motionNum)` → L30 `H(x)` → L31 `H(y)` |

### S_FixWeaponList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_FixWeaponList.java`；繼承 `ServerBasePacket`。

`S_FixWeaponList(List<L1ItemInstance> weaponList)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `C(S_OPCODE_SELECTLIST)` → L24 `D(0x000000c8)` → L26 `H(weaponList.size())` |
| `for (final L1ItemInstance weapon : weaponList)` | L29 `D(weapon.getId())` → L30 `C(weapon.get_durability())` |

`S_FixWeaponList(L1PcInstance pc)`（L39）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L40 呼叫 `buildPacket(pc)` |

`void buildPacket(L1PcInstance pc)`（L44）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L45 `C(S_OPCODE_SELECTLIST)` → L46 `D(0x000000c8)` → L62 `H(weaponList.size())` |
| `for (final L1ItemInstance weapon : weaponList)` | L66 `D(weapon.getId())` → L67 `C(weapon.get_durability())` |

`S_FixWeaponList(L1ItemInstance weapon)`（L74）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L75 `C(S_OPCODE_SELECTLIST)` → L76 `D(0x000000c8)` → L78 `H(1)` → L80 `D(weapon.getId())` → L81 `C(weapon.get_durability())` |

### S_FollowerPack

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_FollowerPack.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`STATUS_POISON=0x01`。

`S_FollowerPack(L1FollowerInstance follower, L1PcInstance pc)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 `C(S_OPCODE_CHARPACK)` → L31 `H(follower.getX())` → L32 `H(follower.getY())` → L33 `D(follower.getId())` → L34 `H(follower.getGfxId())` → L35 `C(follower.getStatus())` → L36 `C(follower.getHeading())` → L37 `C(follower.getChaLightSize())` → L38 `C(follower.getMoveSpeed())` → L39 `D(0x00000000)` → L40 `H(0x0000)` → L41 `S(follower.getNameId())` → L42 `S(follower.getTitle())` → L49 `C(status)` → L50 `D(0x00000000)` → L51 `S(null)` → L52 `S(null)` → L54 `C(0x00)` → L56 `C(0xff)` → L57 `C(0x00)` → L58 `C(0x00)` → L59 `C(0x00)` → L60 `C(0xff)` → L61 `C(0xff)` |

### S_GameTime

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_GameTime.java`；繼承 `ServerBasePacket`。

`S_GameTime(int time)`（L11）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L12 呼叫 `buildPacket(time)` |

`S_GameTime()`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket(time)` |

`void buildPacket(int time)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_GAMETIME)` → L25 `D(time)` |

### S_GlobalEffect

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_GlobalEffect.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`GLOBAL_EFFECT=83`。

`S_GlobalEffect(int triggerNum)`（L10）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L11 `C(S_OPCODE_PACKETBOX)` → L12 `C(GLOBAL_EFFECT)` → L13 `C(triggerNum)` |

### S_GMHtml

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_GMHtml.java`；繼承 `ServerBasePacket`。

`S_GMHtml(int _objid, String html)`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L16 `C(S_OPCODE_SHOWHTML)` → L17 `D(_objid)` → L18 `S(html)` |

### S_HelpMessage

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_HelpMessage.java`；繼承 `ServerBasePacket`。

`S_HelpMessage(String name, String info)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_NPCSHOUT)` → L18 `C(0x00)` → L19 `D(0x00000000)` → L20 `S(name + " --> \\f4" + info)` |

`S_HelpMessage(String string)`（L37）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `C(S_OPCODE_NPCSHOUT)` → L39 `C(0x00)` → L40 `D(0x00000000)` → L41 `S(string)` |

### S_HireSoldier

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_HireSoldier.java`；繼承 `ServerBasePacket`。

`S_HireSoldier(L1PcInstance pc)`（L11）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L13 `H(0x0000)` → L14 `H(0x0000)` → L15 `H(0x0000)` → L16 `S(pc.getName())` → L17 `D(0x00000000)` → L18 `H(0x0000)` |

### S_HitReactionSuppressList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_HitReactionSuppressList.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`CUSTOM_OPCODE=0xF2`、`SUBOPCODE=0x02`、`MAX_ENTRIES=512`。

`S_HitReactionSuppressList()`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(CUSTOM_OPCODE)` → L27 `C(SUBOPCODE)` → L28 `H(count)` |
| `for (final int gfxId : gfxIds) / if (gfxId >= 0 && gfxId <= 0xFFFF && written < count)` | L32 `H(gfxId)` |

### S_HouseMap

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_HouseMap.java`；繼承 `ServerBasePacket`。

`S_HouseMap(int objectId, String house_number)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket(objectId, house_number)` |

`void buildPacket(int objectId, String house_number)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_HOUSEMAP)` → L26 `D(objectId)` → L27 `D(number)` |

### S_HowManyMake

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_HowManyMake.java`；繼承 `ServerBasePacket`。

`S_HowManyMake(int objId, int max, String htmlId)`（L27）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 `C(S_OPCODE_INPUTAMOUNT)` → L30 `D(objId)` → L31 `D(0x00000000)` → L32 `D(0x00000000)` → L33 `D(0x00000000)` → L34 `D(max)` → L35 `H(0x0000)` → L36 `S("request")` → L37 `S(htmlId)` |

`S_HowManyMake(int objId,int max,String action,Object recipe)`（L41）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L42 呼叫 `this(objId,max,action)` |

### S_HPMeter

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_HPMeter.java`；繼承 `ServerBasePacket`。

`S_HPMeter(int objId, int hpRatio)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(objId, hpRatio)` |

`S_HPMeter(L1Character cha)`（L28）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L35 呼叫 `buildPacket(objId, hpRatio)` |

`void buildPacket(int objId, int hpRatio)`（L39）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L40 `C(S_OPCODE_HPMETER)` → L41 `D(objId)` → L42 `C(hpRatio)` |

### S_HPUpdate

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_HPUpdate.java`；繼承 `ServerBasePacket`。

`S_HPUpdate(int currentHp, int maxHp)`（L41）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L42 呼叫 `buildPacket(currentHp, maxHp)` |

`S_HPUpdate(L1PcInstance pc)`（L50）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L51 呼叫 `buildPacket(pc.getCurrentHp(), pc.getMaxHp())` |

`void buildPacket(int currentHp, int maxHp)`（L55）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L57 `C(S_OPCODE_HPUPDATE)` |
| `if (isRO)` | L61 `D(clampCurrent(currentHp, true))` → L62 `D(clampMax(maxHp, true))` |
| `else` | L65 `H(clampCurrent(currentHp, false))` → L66 `H(clampMax(maxHp, false))` |

### S_IconConfig

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_IconConfig.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`SKILL_ICON=0x9a`。

`S_IconConfig(int subCode, int type, int time, boolean second)`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_PACKETBOX)` → L27 `C(subCode)` |
| `switch (subCode) / case SKILL_ICON:` | L31 `H(time)` → L32 `H(type)` → L33 `H(0x00)` → L35 `H(second ? 0x01 : 0x00)` |

### S_Icons

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Icons.java`；繼承 `ServerBasePacket`。

`S_Icons(L1PcInstance pc)`（L75）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L76 呼叫 `buildIcons1(pc, 0, 0)` |

`S_Icons(L1PcInstance pc, int overrideSkillId, int overrideTimeSec)`（L82）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L83 呼叫 `buildIcons1(pc, overrideSkillId, overrideTimeSec)` |

`void buildIcons1(L1PcInstance pc, int overrideSkillId, int overrideTimeSec)`（L86）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L87 `C(S_OPCODE_PACKETBOX)` → L88 `C(S_PacketBox.ICONS1)` → L92 `D(data.length)` → L326 `Byte(data)` |

### S_IdentifyDesc

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_IdentifyDesc.java`；繼承 `ServerBasePacket`。

`S_IdentifyDesc(L1ItemInstance item)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 呼叫 `buildPacket(item)` |

`void buildPacket(L1ItemInstance item)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_IDENTIFYDESC)` → L25 `H(item.getItem().getItemDescId())` |
| `switch (item.getItem().getType2()) / case 1:` | L43 `H(0x0086)` → L44 `C(0x03)` → L45 `S(name.toString())` → L46 `S(item.getItem().getDmgSmall() + "+" + item.getEnchantLevel())` → L47 `S(item.getItem().getDmgLarge() + "+" + item.getEnchantLevel())` |
| `switch (item.getItem().getType2()) / switch (item.getItem().getItemId()) / default:` | L53 `H(0x0087)` → L54 `C(0x02)` → L55 `S(name.toString())` → L56 `S(Math.abs(item.getItem().get_ac()) + "+" + item.getEnchantLevel())` |
| `switch (item.getItem().getType2()) / switch (item.getItem().getType()) / case 1:` | L65 `H(0x0089)` → L66 `C(0x03)` → L67 `S(name.toString())` → L68 `S(String.valueOf(item.getChargeCount()))` |
| `switch (item.getItem().getType2()) / switch (item.getItem().getType()) / case 2:` | L72 `H(0x008a)` → L73 `C(0x02)` → L76 `S(name.toString())` |
| `switch (item.getItem().getType2()) / switch (item.getItem().getType()) / case 7:` | L80 `H(0x0088)` → L81 `C(0x03)` → L82 `S(name.toString())` → L83 `S(String.valueOf(item.getItem().getFoodVolume()))` |
| `switch (item.getItem().getType2()) / switch (item.getItem().getType()) / default:` | L87 `H(0x008a)` → L88 `C(0x02)` → L89 `S(name.toString())` |
| `switch (item.getItem().getType2()) / default:` | L92 `S(String.valueOf(item.getWeight()))` |

`S_IdentifyDesc()`（L100）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L104 `C(S_OPCODE_IDENTIFYDESC)` → L105 `H(item.getItem().getItemDescId())` → L107 `H(134)` → L108 `C(3)` → L109 `S(item.getName())` → L110 `S(item.getItem().getDmgSmall() + "+" + item.getEnchantLevel())` → L111 `S(item.getItem().getDmgLarge() + "+" + item.getEnchantLevel())` → L112 `S(String.valueOf(item.getWeight()))` |

### S_InitialAbilityGrowth

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_InitialAbilityGrowth.java`；繼承 `ServerBasePacket`。

`S_InitialAbilityGrowth(L1PcInstance pc)`（L12）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L93 呼叫 `buildPacket(pc, growth[0], growth[1], growth[2], growth[3], growth[4], growth[5])` |

`void buildPacket(L1PcInstance pc, int Str, int Dex, int Con, int Wis, int Cha, int Int)`（L115）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L120 `C(S_OPCODE_CHARRESET)` → L121 `C(0x04)` → L122 `C(write1)` → L123 `C(write2)` → L124 `C(write3)` → L125 `C(0x00)` |

### S_Invis

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Invis.java`；繼承 `ServerBasePacket`。

`S_Invis(int objid, int type)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket(objid, type)` |

`void buildPacket(int objid, int type)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_INVIS)` → L25 `D(objid)` → L26 `C(type)` |

### S_InvList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_InvList.java`；繼承 `ServerBasePacket`。

`S_InvList(List<L1ItemInstance> items)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_INVLIST)` → L24 `C(count)` |
| `for (int index = 0; index < count; index++)` | L28 `D(item.getId())` → L29 `H(item.getItem().getPacketItemDescId())` → L30 `C(item.getItem().getPacketUseType())` → L32 `C(0)` → L34 `C(item.getChargeCount())` → L35 `H(item.get_gfxid())` → L37 `C(item.isSealed() ? L1Item.toPacketBless(item.getBless()) + 128 : L1Item.toPacketBless(item.getBless()))` → L38 `D((int) Math.min(item.getCount(), 2000000000))` → L61 `C(statusX)` → L62 `S(item.getViewName())` |
| `for (int index = 0; index < count; index++) / if (!item.isIdentified())` | L64 `C(0)` |
| `for (int index = 0; index < count; index++) / else if (S_ItemStatus.shouldUseLongStatus(item))` | L66 `C(0)` |
| `for (int index = 0; index < count; index++) / else` | L69 `C(status.length)` |
| `for (int index = 0; index < count; index++) / else / for (int i = 0; i < j; i++)` | L74 `C(b)` |
| `for (int index = 0; index < count; index++)` | L77 `C(10)` → L78 `H(0)` → L79 `D(0)` → L80 `D(0)` |

### S_ItemAmount

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ItemAmount.java`；繼承 `ServerBasePacket`。

`S_ItemAmount(L1ItemInstance item, int test)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 呼叫 `buildPacket(item)` |

`void buildPacket(L1ItemInstance item)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 `C(S_OPCODE_ITEMAMOUNT)` → L31 `D(item.getId())` → L32 `S(item.getViewName())` → L47 `D(count)` |
| `if (!item.isIdentified())` | L54 `C(0x00)` |
| `else` | L58 `C(status.length)` |
| `else / for (final byte b : status)` | L60 `C(b)` |

### S_ItemBoard

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ItemBoard.java`；繼承 `ServerBasePacket`。

`S_ItemBoard(int gfxid, String msg)`（L13）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L14 `C(S_OPCODE_PACKETBOX)` → L15 `C(190)` → L16 `H(gfxid)` → L17 `S(msg)` |

### S_ItemColor

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ItemColor.java`；繼承 `ServerBasePacket`。

`S_ItemColor(L1ItemInstance item)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 呼叫 `buildPacket(item)` |

`void buildPacket(L1ItemInstance item)`（L28）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 `C(S_OPCODE_ITEMCOLOR)` → L30 `D(item.getId())` → L34 `C(item.isSealed() ? L1Item.toPacketBless(item.getBless()) + 128 : L1Item.toPacketBless(item.getBless()))` |

`S_ItemColor(L1ItemInstance item, int id)`（L55）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L56 `C(S_OPCODE_ITEMCOLOR)` → L57 `D(item.getId())` → L60 `C(id)` |

### S_ItemCount

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ItemCount.java`；繼承 `ServerBasePacket`。

`S_ItemCount(int objId, int max, String cmd)`（L27）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 `C(S_OPCODE_INPUTAMOUNT)` → L30 `D(objId)` → L31 `D(0x00000000)` → L32 `D(0x00000000)` → L33 `D(0x00000000)` → L34 `D(max)` → L35 `H(0x0000)` → L36 `S("request")` → L37 `S(cmd)` |

`S_ItemCount(int objId, int max, String html, String cmd)`（L49）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L51 `C(S_OPCODE_INPUTAMOUNT)` → L52 `D(objId)` → L53 `D(0x00000000)` → L54 `D(0x00000000)` → L55 `D(0x00000000)` → L56 `D(max)` → L57 `H(0x0000)` → L58 `S(html)` → L59 `S(cmd)` |

`S_ItemCount(int objId, int min, int max, String html, String cmd, String[] data)`（L63）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L65 `C(S_OPCODE_INPUTAMOUNT)` → L66 `D(objId)` → L67 `D(0x00000000)` → L68 `D(min)` → L69 `D(min)` → L70 `D(max)` → L71 `H(0x0000)` → L72 `S(html)` → L73 `S(cmd)` |
| `if ((data != null) && (1 <= data.length))` | L75 `H(data.length)` |
| `if ((data != null) && (1 <= data.length)) / for (final String datum : data)` | L77 `S(datum)` |

`S_ItemCount(int objId, int min, int max, String html, String cmd)`（L92）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L94 `C(S_OPCODE_INPUTAMOUNT)` → L95 `D(objId)` → L96 `D(0x00000000)` → L97 `D(min)` → L98 `D(min)` → L99 `D(max)` → L100 `H(0x0000)` → L101 `S(html)` → L102 `S(cmd)` |

`S_ItemCount(int objId,int min,int max,String html,String cmd,UiOperationContext.AuctionSelection selection)`（L106）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L107 呼叫 `this(objId,min,max,html,cmd)` |

`S_ItemCount(int objId,int min,int max,String html,String cmd,String[] data,Object recipe)`（L111）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L112 呼叫 `this(objId,min,max,html,cmd,data)` |

### S_ItemError

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ItemError.java`；繼承 `ServerBasePacket`。

`S_ItemError(int skillid)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 呼叫 `buildPacket(skillid)` |

`void buildPacket(int skillid)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `D(skillid)` |

### S_ItemName

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ItemName.java`；繼承 `ServerBasePacket`。

`S_ItemName(L1ItemInstance item)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `C(S_OPCODE_ITEMNAME)` → L24 `D(item.getId())` → L25 `S(item.getViewName())` |

`S_ItemName(L1ItemInstance item, int id)`（L32）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `C(S_OPCODE_ITEMNAME)` → L39 `D(item.getId())` → L40 `S(item.getViewName() + " ($" + id + ")")` |

### S_ItemStatus

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ItemStatus.java`；繼承 `ServerBasePacket`。

`S_ItemStatus(L1ItemInstance item)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 呼叫 `buildPacket(item)` |

`void buildPacket(L1ItemInstance item)`（L28）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (shouldUseLongStatus(item, status))` | L31 呼叫 `writeLongStatus(item, item.getViewName(), (int) Math.min(item.getCount(), 2000000000), status)` |
| 本方法此處 | L36 `C(S_OPCODE_ITEMAMOUNT)` → L37 `D(item.getId())` → L38 `S(item.getViewName())` → L43 `D(count)` |
| `if (!item.isIdentified())` | L48 `C(0x00)` |
| `else` | L51 `C(inlineStatus.length)` |
| `else / for (final byte b : inlineStatus)` | L53 `C(b)` |

`S_ItemStatus(L1ItemInstance item, long count)`（L61）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (shouldUseLongStatus(item, status))` | L65 呼叫 `writeLongStatus(item, item.getNumberedViewName(count), out_count, status)` |
| 本方法此處 | L71 `C(S_OPCODE_ITEMAMOUNT)` → L72 `D(item.getId())` → L73 `S(item.getNumberedViewName(count))` → L76 `D(out_count)` |
| `if (!item.isIdentified())` | L80 `C(0x00)` |
| `else` | L83 `C(inlineStatus.length)` |
| `else / for (final byte b : inlineStatus)` | L85 `C(b)` |

`void writeLongStatus(L1ItemInstance item, String viewName, int count, byte[] status)`（L107）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L108 `C(0xF2)` → L109 `C(0x01)` → L110 `D(item.getId())` → L111 `S(viewName)` → L112 `D(count)` → L113 `H(status.length)` |
| `for (final byte b : status)` | L115 `C(b)` |

### S_Karma

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Karma.java`；繼承 `ServerBasePacket`。

`S_Karma(L1PcInstance pc)`（L9）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L10 `C(S_OPCODE_PACKETBOX)` → L11 `C(0x57)` → L12 `D(pc.getKarma())` |

### S_KillMessage

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_KillMessage.java`；繼承 `ServerBasePacket`。

`S_KillMessage(String name, String msg, int i)`（L33）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L34 `C(S_OPCODE_NPCSHOUT)` → L35 `C(0x00)` → L36 `D(0x00000000)` → L37 `S(" \\fY[" + name + "] " + msg)` |

### S_Lawful

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Lawful.java`；繼承 `ServerBasePacket`。

`S_Lawful(L1PcInstance pc)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 呼叫 `buildPacket(pc)` |

`void buildPacket(L1PcInstance pc)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_LAWFUL)` → L26 `D(pc.getId())` → L27 `H(pc.getLawful())` |

`S_Lawful(int objid)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L36 `C(S_OPCODE_LAWFUL)` → L37 `D(objid)` → L38 `H(-32768)` → L39 `H(-32768)` → L40 `H(-32768)` |

### S_Letter

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Letter.java`；繼承 `ServerBasePacket`。

無直接 read/write 欄位；參見入口 `S_Letter(L1ItemInstance item)`（L14）。若為空處理器則不消費內容；繼承／委派不代表空封包。

### S_Light

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Light.java`；繼承 `ServerBasePacket`。

`S_Light(int objid, int type)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket(objid, type)` |

`void buildPacket(int objid, int type)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_LIGHT)` → L25 `D(objid)` → L26 `C(type)` |

### S_Liquor

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Liquor.java`；繼承 `ServerBasePacket`。

`S_Liquor(int objecId)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_LIQUOR)` → L18 `D(objecId)` → L19 `C(0x01)` |

`S_Liquor(int objecId, int type)`（L28）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 `C(S_OPCODE_LIQUOR)` → L30 `D(objecId)` → L31 `C(type)` |

### S_Lock

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Lock.java`；繼承 `ServerBasePacket`。

`S_Lock()`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket()` |

`void buildPacket()`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `C(S_OPCODE_MOVELOCK)` → L24 `C(0x00)` |

### S_LoginResult

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_LoginResult.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`REASON_LOGIN_OK=0x0000`、`REASON_ACCESS_FAILED=0x0008`、`REASON_ACCOUNT_IN_USE=0x0016`、`EVENT_CANT_USE=0x001a`、`EVENT_CANT_LOGIN=0x001c`、`EVENT_REGISTER_ACCOUNTS=0x002f`、`EVENT_LAN_ERROR=0x0027`、`EVENT_MAIL_ERROR=0x000b`、`REASON_ACCOUNT_ALREADY_EXISTS=0x0007`、`EVENT_C_ERROR=0x0011`、`EVENT_RE_LOGIN=0x0004`、`EVENT_CLOCK_ERROR=0x0032`、`EVENT_NAME_ERROR=0x0034`、`EVENT_PASS_ERROR=0x0035`、`EVENT_PASS_CHECK=0x000a`、`EVENT_ERROR_USER=0x009b`、`EVENT_ERROR_PASS=0x0095`。

`S_LoginResult(int reason)`（L103）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L104 呼叫 `buildPacket(reason)` |

`void buildPacket(int reason)`（L108）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L110 `C(S_OPCODE_LOGINRESULT)` → L111 `H(reason)` → L113 `D(0x00000000)` → L114 `D(0x00000000)` → L115 `D(0x00000000)` |

### S_MagicEquipment

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_MagicEquipment.java`；繼承 `ServerBasePacket`。

`S_MagicEquipment(int timeSec, int gfxId)`（L14）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L15 `C(S_OPCODE_PACKETBOX)` → L16 `C(0x9a)` → L17 `H(timeSec)` → L18 `D(gfxId)` |

### S_Mail

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Mail.java`；繼承 `ServerBasePacket`。

`S_Mail(ArrayList<L1Mail> mails, int type)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `C(S_OPCODE_MAIL)` → L24 `C(type)` → L25 `H(mails.size())` |
| `for (int i = 0; i < mails.size(); i++)` | L28 `D(mail.getId())` → L29 `C(mail.getReadStatus())` |
| `for (int i = 0; i < mails.size(); i++) / if (dateText != null && !dateText.isEmpty()) / if (st.length >= 3)` | L43 `C(Integer.parseInt(st[0].trim()))` → L44 `C(Integer.parseInt(st[1].trim()))` → L45 `C(Integer.parseInt(st[2].trim()))` → L48 `C(0)` → L49 `C(0)` → L50 `C(0)` |
| `for (int i = 0; i < mails.size(); i++) / if (dateText != null && !dateText.isEmpty()) / else` | L54 `C(0)` → L55 `C(0)` → L56 `C(0)` |
| `for (int i = 0; i < mails.size(); i++) / else` | L60 `C(0)` → L61 `C(0)` → L62 `C(0)` |
| `for (int i = 0; i < mails.size(); i++)` | L65 `S(mail.getSenderName())` → L66 `Byte(mail.getSubject())` → L67 `C(0x00)` |

`S_Mail(int type)`（L80）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L81 `C(S_OPCODE_MAIL)` → L82 `C(type)` |

`S_Mail(L1Mail mail, int type)`（L92）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `switch (type) / case 0x40:` | L98 呼叫 `buildPacket_1(mail, type)` |
| `switch (type) / default:` | L102 呼叫 `buildPacket_2(mail, type)` |

`void buildPacket_1(L1Mail mail, int type)`（L108）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L109 `C(S_OPCODE_MAIL)` → L110 `C(type)` → L111 `D(mail.getId())` → L112 `C(0x01)` |

`void buildPacket_2(L1Mail mail, int type)`（L115）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L116 `C(S_OPCODE_MAIL)` → L117 `C(type)` → L118 `D(mail.getId())` → L119 `Byte(mail.getContent())` |

### S_MapID

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_MapID.java`；繼承 `ServerBasePacket`。

`S_MapID(int mapid, boolean isUnderwater)`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L16 `C(S_OPCODE_MAPID)` → L17 `H(mapid)` → L18 `C(isUnderwater ? 0x01 : 0x00)` |

`S_MapID(L1PcInstance pc, int mapid, boolean isUnderwater)`（L28）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L47 `C(S_OPCODE_MAPID)` → L48 `H(mapid)` → L49 `C(isUnderwater ? 0x01 : 0x00)` |

`S_MapID(int mapid)`（L69）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L72 `C(S_OPCODE_MAPID)` → L73 `H(mapid)` → L74 `C(0x00)` → L76 `C(0x10)` → L77 `H(0x00f8)` → L78 `C(0x00)` → L79 `C(0x00)` → L80 `C(0x00)` → L81 `C(0x00)` → L82 `C(0x00)` → L83 `C(0x10)` → L84 `C(0x0d)` → L85 `C(0x35)` → L86 `C(0xc5)` |

### S_Message_YN

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Message_YN.java`；繼承 `ServerBasePacket`。

`S_Message_YN(int type)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 呼叫 `this(null, null, type)` |

`S_Message_YN(Offer offer, Ticket prerequisite, int type)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L33 `C(S_OPCODE_YES_NO)` → L34 `H(0x0000)` → L35 `D(0x00000000)` → L36 `H(type)` |

`S_Message_YN(String name)`（L45）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L48 `C(S_OPCODE_YES_NO)` → L49 `H(0x0000)` → L50 `D(_sequentialNumber.incrementAndGet())` → L51 `H(0x00fc)` → L52 `S(name)` |

`S_Message_YN(int type, String msg)`（L61）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L64 `C(S_OPCODE_YES_NO)` → L65 `H(0x0000)` → L66 `D(0x00000000)` → L67 `H(type)` → L68 `S(msg)` |

`S_Message_YN(Offer offer, Ticket prerequisite, int type, String msg)`（L73）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L76 `C(S_OPCODE_YES_NO)` → L77 `H(0x0000)` → L78 `D(0x00000000)` → L79 `H(type)` → L80 `S(msg)` |

`S_Message_YN(int type, String msg1, String msg2)`（L90）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L93 `C(S_OPCODE_YES_NO)` → L94 `H(0x0000)` → L95 `D(0x00000000)` → L96 `H(type)` → L97 `S(msg1)` → L98 `S(msg2)` |

`static S_Message_YN confirmation(Offer offer, Ticket prerequisite, int requestId, String text)`（L105）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L106 呼叫 `S_Message_YN(offer, prerequisite, requestId, text, true)` |

`S_Message_YN(Offer offer, Ticket prerequisite, int requestId, String text, boolean raw)`（L108）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L113 `C(S_OPCODE_YES_NO)` → L113 `H(0)` → L113 `D(requestId)` → L113 `H(622)` → L113 `S(text)` |

### S_MoveCharPacket

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_MoveCharPacket.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`FINALIZED_LENGTH=16`。

`S_MoveCharPacket(L1Character cha)`（L53）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L54 呼叫 `this(cha, captureLegacyMovement(cha))` |

`S_MoveCharPacket(L1Character cha, int sourceX, int sourceY, int heading)`（L79）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L81 呼叫 `this(cha, captureSuppliedMovement(cha, sourceX, sourceY, heading))` |

`S_MoveCharPacket(L1Character cha, L1Character.MovementSnapshot snapshot)`（L98）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L124 `C(S_OPCODE_MOVEOBJECT)` → L125 `D(_objectId)` → L126 `H(snapshot.sourceX())` → L127 `H(snapshot.sourceY())` → L128 `C(heading)` → L129 `C(0x80)` |

### S_MoveNpcPacket

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_MoveNpcPacket.java`；繼承 `ServerBasePacket`。

`S_MoveNpcPacket(L1MonsterInstance npc, int x, int y, int heading)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_MOVEOBJECT)` → L25 `D(npc.getId())` → L26 `H(x)` → L27 `H(y)` → L28 `C(heading)` → L30 `C(0x80)` |

`S_MoveNpcPacket(L1Character cha)`（L38）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L40 `C(S_OPCODE_MOVEOBJECT)` → L41 `D(cha.getId())` → L42 `H(cha.getX())` → L43 `H(cha.getY())` → L44 `C(cha.getHeading())` → L46 `C(0x80)` |

### S_MPUpdate

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_MPUpdate.java`；繼承 `ServerBasePacket`。

`S_MPUpdate(int currentmp, int maxmp)`（L41）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L42 呼叫 `buildPacket(currentmp, maxmp)` |

`S_MPUpdate(L1PcInstance pc)`（L50）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L51 呼叫 `buildPacket(pc.getCurrentMp(), pc.getMaxMp())` |

`void buildPacket(int currentmp, int maxmp)`（L60）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L62 `C(S_OPCODE_MPUPDATE)` |
| `if (isRO)` | L66 `D(clampCurrent(currentmp, true))` → L67 `D(clampMax(maxmp, true))` |
| `else` | L70 `H(clampCurrent(currentmp, false))` → L71 `H(clampMax(maxmp, false))` |

### S_NewCharPacket

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NewCharPacket.java`；繼承 `ServerBasePacket`。

`S_NewCharPacket(L1PcInstance pc)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 呼叫 `buildPacket(pc)` |

`void buildPacket(L1PcInstance pc)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 `C(S_OPCODE_NEWCHARPACK)` → L28 `S(pc.getName())` → L29 `S("")` → L30 `C(pc.getType())` → L31 `C(pc.get_sex())` → L32 `H(pc.getLawful())` |
| `if (ConfigClient.isROClient())` | L36 `D(pc.getMaxHp())` → L37 `D(pc.getMaxMp())` |
| `else` | L39 `H(pc.getMaxHp())` → L40 `H(pc.getMaxMp())` |
| 本方法此處 | L43 `C(pc.getAc())` → L44 `C(pc.getLevel())` → L45 `C(pc.getStr())` → L46 `C(pc.getDex())` → L47 `C(pc.getCon())` → L48 `C(pc.getWis())` → L49 `C(pc.getCha())` → L50 `C(pc.getInt())` → L53 `C(0x00)` → L57 `D(time)` → L60 `C(checkcode & 0xFF)` |

### S_NewMaster

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NewMaster.java`；繼承 `ServerBasePacket`。

`S_NewMaster(String name, L1NpcInstance npc)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(name, npc)` |

`void buildPacket(String name, L1NpcInstance npc)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_NEWMASTER)` → L26 `D(npc.getId())` → L27 `S(name)` |

`S_NewMaster(L1NpcInstance npc)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 `C(S_OPCODE_NEWMASTER)` → L32 `D(npc.getId())` → L33 `S("")` |

### S_NoSell

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NoSell.java`；繼承 `ServerBasePacket`。

`S_NoSell(L1NpcInstance npc)`（L8）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L9 呼叫 `buildPacket(npc)` |

`void buildPacket(L1NpcInstance npc)`（L13）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L14 `C(S_OPCODE_SHOWHTML)` → L15 `D(npc.getId())` → L16 `S("nosell")` → L17 `C(0x01)` |

### S_NpcChat

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NpcChat.java`；繼承 `ServerBasePacket`。

`S_NpcChat(L1NpcInstance npc, String chat)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 `C(S_OPCODE_NPCSHOUT)` → L23 `C(0x00)` → L24 `D(npc.getId())` → L25 `S(npc.getNameId() + ": " + chat)` |

`S_NpcChat(L1NpcInstance npc, String chat, boolean name)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L36 `C(S_OPCODE_NPCSHOUT)` → L38 `C(0x00)` → L39 `D(npc.getId())` → L40 `S((name? npc.getNameId() + ": ":"") + chat)` |

`S_NpcChat(L1PcInstance pc, String chat)`（L49）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L50 `C(S_OPCODE_NPCSHOUT)` → L52 `C(0x00)` → L53 `D(pc.getId())` → L54 `S(pc.getName() + ": " + chat)` |

### S_NpcChatGlobal

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NpcChatGlobal.java`；繼承 `ServerBasePacket`。

`S_NpcChatGlobal(L1NpcInstance npc, String chat)`（L14）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L15 呼叫 `buildPacket(npc, chat)` |

`void buildPacket(L1NpcInstance npc, String chat)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_NPCSHOUT)` → L21 `C(0x03)` → L22 `D(npc.getId())` → L23 `S("[" + npc.getNameId() + "] " + chat)` |

### S_NpcChatShouting

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NpcChatShouting.java`；繼承 `ServerBasePacket`。

`S_NpcChatShouting(L1NpcInstance npc, String chat)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(npc, chat)` |

`void buildPacket(L1NpcInstance npc, String chat)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_NPCSHOUT)` → L27 `C(0x02)` → L28 `D(npc.getId())` → L29 `S("<" + npc.getNameId() + "> " + chat)` |

### S_NPCPack

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`STATUS_POISON=1`、`STATUS_PC=4`。

`S_NPCPack(L1NpcInstance npc)`（L28）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 `C(S_OPCODE_CHARPACK)` → L30 `H(npc.getX())` → L31 `H(npc.getY())` → L32 `D(npc.getId())` |
| `if (npc.getTempCharGfx() == 0)` | L35 `H(npc.getGfxId())` |
| `else` | L38 `H(npc.getTempCharGfx())` |
| `if (npc.getNpcTemplate().is_doppel() && (npc.getGfxId() != 31))` | L42 `C(0x04)` |
| `else` | L45 `C(npc.getStatus())` |
| 本方法此處 | L48 `C(npc.getHeading())` → L49 `C(npc.getChaLightSize())` → L50 `C(npc.getMoveSpeed())` → L51 `D((int) npc.getExp())` → L52 `H(npc.getTempLawful())` → L53 `S(npc.getNameId())` → L55 `S(npc.getTitle())` → L76 `C(status)` → L78 `D(0x00000000)` → L79 `S(null)` → L80 `S(null)` → L82 `C(0x00)` → L84 `C(0xff)` → L85 `C(0x00)` → L87 `C(npc.getLevel())` → L88 `C(0x00)` → L89 `C(0xff)` → L90 `C(0xff)` |

`S_NPCPack(L1NpcInstance npc, L1PcInstance pc)`（L99）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L100 `C(S_OPCODE_CHARPACK)` → L101 `H(npc.getX())` → L102 `H(npc.getY())` → L103 `D(npc.getId())` |
| `if (npc.getTempCharGfx() == 0)` | L106 `H(npc.getGfxId())` |
| `else` | L109 `H(npc.getTempCharGfx())` |
| `if (npc.getNpcTemplate().is_doppel() && (npc.getGfxId() != 31))` | L113 `C(0x04)` |
| `else` | L116 `C(npc.getStatus())` |
| 本方法此處 | L119 `C(npc.getHeading())` → L120 `C(npc.getChaLightSize())` → L121 `C(npc.getMoveSpeed())` → L122 `D((int) npc.getExp())` → L123 `H(npc.getTempLawful())` → L131 `S(name)` → L134 `S(npc.getTitle())` → L155 `C(status)` → L157 `D(0x00000000)` → L158 `S(null)` → L159 `S(null)` → L161 `C(0x00)` → L163 `C(0xff)` → L164 `C(0x00)` → L166 `C(npc.getLevel())` → L167 `C(0x00)` → L168 `C(0xff)` → L169 `C(0xff)` |

### S_NPCPack_D

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_D.java`；繼承 `ServerBasePacket`。

`S_NPCPack_D(L1DwarfInstance npc)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_CHARPACK)` → L21 `H(npc.getX())` → L22 `H(npc.getY())` → L23 `D(npc.getId())` → L25 `H(npc.getGfxId())` → L27 `C(npc.getStatus())` → L29 `C(npc.getHeading())` → L30 `C(npc.getChaLightSize())` → L31 `C(npc.getMoveSpeed())` → L32 `D((int) npc.getExp())` → L33 `H(npc.getTempLawful())` → L34 `S(npc.getNameId())` → L35 `S(null)` → L37 `C(0)` → L39 `D(0x00000000)` → L40 `S(null)` → L41 `S(null)` → L42 `C(0x00)` → L43 `C(0xff)` → L44 `C(0x00)` → L45 `C(0x00)` → L46 `C(0x00)` → L47 `C(0xff)` → L48 `C(0xff)` |

`S_NPCPack_D(L1DwarfInstance npc, L1PcInstance pc)`（L57）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L58 `C(S_OPCODE_CHARPACK)` → L59 `H(npc.getX())` → L60 `H(npc.getY())` → L61 `D(npc.getId())` → L63 `H(npc.getGfxId())` → L65 `C(npc.getStatus())` → L67 `C(npc.getHeading())` → L68 `C(npc.getChaLightSize())` → L69 `C(npc.getMoveSpeed())` → L70 `D((int) npc.getExp())` → L71 `H(npc.getTempLawful())` → L79 `S(name)` → L82 `S(null)` → L84 `C(0)` → L86 `D(0x00000000)` → L87 `S(null)` → L88 `S(null)` → L89 `C(0x00)` → L90 `C(0xff)` → L91 `C(0x00)` → L92 `C(0x00)` → L93 `C(0x00)` → L94 `C(0xff)` → L95 `C(0xff)` |

### S_NPCPack_De

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_De.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`STATUS_POISON=1`、`STATUS_INVISIBLE=2`、`STATUS_PC=4`、`STATUS_BRAVE=16`、`STATUS_ELFBRAVE=32`、`STATUS_FASTMOVABLE=64`。

`S_NPCPack_De(L1DeInstance de)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L52 `C(S_OPCODE_CHARPACK)` → L53 `H(de.getX())` → L54 `H(de.getY())` → L55 `D(de.getId())` → L57 `H(de.getTempCharGfx())` → L59 `C(de.getStatus())` → L61 `C(de.getHeading())` → L63 `C(de.getChaLightSize())` → L64 `C(de.getMoveSpeed())` → L65 `D(0x00000000)` → L66 `H(de.getLawful())` → L68 `S(de.getNameId())` → L69 `S(de.getTitle())` → L70 `C(status)` → L71 `D(de.getClanid())` → L72 `S(de.getClanname())` → L73 `S(null)` → L75 `C(0x00)` → L77 `C(0xff)` → L78 `C(de.hasSkillEffect(STATUS_BRAVE3) ? 0x08 : 0x00)` → L79 `C(0x00)` → L80 `C(0x00)` → L81 `C(0xff)` → L82 `C(0xff)` |

### S_NPCPack_Doll

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_Doll.java`；繼承 `ServerBasePacket`。

`S_NPCPack_Doll(L1DollInstance pet, L1PcInstance pc)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_CHARPACK)` → L27 `H(pet.getX())` → L28 `H(pet.getY())` → L29 `D(pet.getId())` → L30 `H(pet.getGfxId())` → L31 `C(pet.getStatus())` → L32 `C(pet.getHeading())` → L33 `C(0x00)` → L34 `C(pet.getMoveSpeed())` → L35 `D(0x00000000)` → L36 `H(0x0000)` → L38 `S(pet.getNameId())` → L39 `S(pet.getTitle())` → L41 `C(0x00)` → L42 `D(0x00000000)` → L44 `S(null)` |
| `if (pet.getMaster() != null) / if (pet.getMaster() instanceof L1PcInstance)` | L48 `S(pet.getMaster().getName())` |
| `if (pet.getMaster() != null) / else if (pet.getMaster() instanceof L1NpcInstance)` | L51 `S(npc.getNameId())` |
| `else` | L55 `S("")` |
| 本方法此處 | L58 `C(0x00)` → L60 `C(0xff)` → L61 `C(0x00)` → L62 `C(0x00)` → L64 `C(0x00)` → L65 `C(0xff)` → L66 `C(0xff)` |

### S_NPCPack_Eff

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_Eff.java`；繼承 `ServerBasePacket`。

`S_NPCPack_Eff(L1EffectInstance npc)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 `C(S_OPCODE_CHARPACK)` → L20 `H(npc.getX())` → L21 `H(npc.getY())` → L22 `D(npc.getId())` → L24 `H(npc.getGfxId())` → L26 `C(npc.getStatus())` → L28 `C(npc.getHeading())` → L29 `C(npc.getChaLightSize())` → L30 `C(npc.getMoveSpeed())` → L31 `D((int) npc.getExp())` → L32 `H(npc.getTempLawful())` → L33 `S(npc.getNameId())` → L35 `S(npc.getTitle())` → L37 `C(0x00)` → L39 `D(0x00000000)` → L40 `S(null)` → L41 `S(null)` → L43 `C(0x00)` → L45 `C(0xff)` → L46 `C(0x00)` → L47 `C(0x00)` → L49 `C(0x00)` → L50 `C(0xff)` → L51 `C(0xff)` |

### S_NPCPack_F

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_F.java`；繼承 `ServerBasePacket`。

`S_NPCPack_F(L1FieldObjectInstance npc)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 `C(S_OPCODE_CHARPACK)` → L22 `H(npc.getX())` → L23 `H(npc.getY())` → L24 `D(npc.getId())` → L26 `H(npc.getGfxId())` → L28 `C(npc.getStatus())` → L30 `C(npc.getHeading())` → L31 `C(npc.getChaLightSize())` → L32 `C(npc.getMoveSpeed())` → L33 `D((int) npc.getExp())` → L34 `H(npc.getTempLawful())` → L35 `S(npc.getNameId())` |
| `if (sceneryHtml != null)` | L39 `S(sceneryHtml)` |
| `else` | L42 `S(null)` |
| 本方法此處 | L45 `C(0x00)` → L47 `D(0x00000000)` → L48 `S(null)` → L49 `S(null)` → L51 `C(0x00)` → L53 `C(0xff)` → L54 `C(0x00)` → L55 `C(0x00)` → L56 `C(0x00)` → L57 `C(0xff)` → L58 `C(0xff)` |

`S_NPCPack_F(L1FieldObjectInstance npc, L1PcInstance pc)`（L67）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L68 `C(S_OPCODE_CHARPACK)` → L69 `H(npc.getX())` → L70 `H(npc.getY())` → L71 `D(npc.getId())` → L73 `H(npc.getGfxId())` → L75 `C(npc.getStatus())` → L77 `C(npc.getHeading())` → L78 `C(npc.getChaLightSize())` → L79 `C(npc.getMoveSpeed())` → L80 `D((int) npc.getExp())` → L81 `H(npc.getTempLawful())` → L89 `S(name)` |
| `if (sceneryHtml != null)` | L94 `S(sceneryHtml)` |
| `else` | L97 `S(null)` |
| 本方法此處 | L100 `C(0x00)` → L102 `D(0x00000000)` → L103 `S(null)` → L104 `S(null)` → L106 `C(0x00)` → L108 `C(0xff)` → L109 `C(0x00)` → L110 `C(0x00)` → L111 `C(0x00)` → L112 `C(0xff)` → L113 `C(0xff)` |

### S_NPCPack_Ill

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_Ill.java`；繼承 `ServerBasePacket`。

`S_NPCPack_Ill(L1IllusoryInstance de)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 `C(S_OPCODE_CHARPACK)` → L20 `H(de.getX())` → L21 `H(de.getY())` → L22 `D(de.getId())` → L24 `H(de.getTempCharGfx())` → L26 `C(de.getStatus())` → L28 `C(de.getHeading())` → L29 `C(de.getChaLightSize())` → L30 `C(de.getMoveSpeed())` → L31 `D(0x00000000)` → L32 `H(de.getLawful())` → L34 `S(de.getNameId())` → L35 `S(de.getTitle())` → L37 `C(0x00)` → L38 `D(0x00000000)` → L39 `S(null)` → L40 `S(null)` → L42 `C(0x00)` → L44 `C(0xff)` → L45 `C(0x00)` → L46 `C(0x00)` → L47 `C(0x00)` → L48 `C(0xff)` → L49 `C(0xff)` |

### S_NPCPack_Item

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_Item.java`；繼承 `ServerBasePacket`。

`S_NPCPack_Item(L1ItemInstance item)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 呼叫 `buildPacket(item)` |

`void buildPacket(L1ItemInstance item)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_CHARPACK)` → L25 `H(item.getX())` → L26 `H(item.getY())` → L27 `D(item.getId())` → L28 `H(item.getItem().getGroundGfxId())` → L29 `C(0x00)` → L30 `C(0x00)` → L31 `C(item.isNowLighting()? item.getItem().getLightRange() : 0x00)` → L32 `C(0x00)` → L33 `D((int) Math.min(item.getCount(), 2000000000))` → L34 `H(0x0000)` → L69 `S(name)` → L70 `S(null)` → L71 `C(0x00)` → L72 `D(0x00000000)` → L73 `S(null)` → L74 `S(null)` → L76 `C(0x00)` → L78 `C(0xff)` → L79 `C(0x00)` → L80 `C(0x00)` → L81 `C(0x00)` → L82 `C(0xff)` → L83 `C(0xff)` |

### S_NPCPack_M

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_M.java`；繼承 `ServerBasePacket`。

`S_NPCPack_M(L1MerchantInstance npc)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_CHARPACK)` → L21 `H(npc.getX())` → L22 `H(npc.getY())` → L23 `D(npc.getId())` → L25 `H(npc.getGfxId())` → L27 `C(npc.getStatus())` → L29 `C(npc.getHeading())` → L30 `C(npc.getChaLightSize())` → L31 `C(npc.getMoveSpeed())` → L32 `D((int) npc.getExp())` → L33 `H(npc.getTempLawful())` → L34 `S(npc.getNameId())` → L35 `S(null)` → L37 `C(0)` → L39 `D(0x00000000)` → L40 `S(null)` → L41 `S(null)` → L42 `C(0x00)` → L43 `C(0xff)` → L44 `C(0x00)` → L45 `C(0x00)` → L46 `C(0x00)` → L47 `C(0xff)` → L48 `C(0xff)` |

`S_NPCPack_M(L1MerchantInstance npc, L1PcInstance pc)`（L57）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L58 `C(S_OPCODE_CHARPACK)` → L59 `H(npc.getX())` → L60 `H(npc.getY())` → L61 `D(npc.getId())` → L63 `H(npc.getGfxId())` → L65 `C(npc.getStatus())` → L67 `C(npc.getHeading())` → L68 `C(npc.getChaLightSize())` → L69 `C(npc.getMoveSpeed())` → L70 `D((int) npc.getExp())` → L71 `H(npc.getTempLawful())` → L79 `S(name)` → L82 `S(null)` → L84 `C(0)` → L86 `D(0x00000000)` → L87 `S(null)` → L88 `S(null)` → L89 `C(0x00)` → L90 `C(0xff)` → L91 `C(0x00)` → L92 `C(0x00)` → L93 `C(0x00)` → L94 `C(0xff)` → L95 `C(0xff)` |

### S_NPCPack_Pet

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_Pet.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`STATUS_POISON=1`。

`S_NPCPack_Pet(L1PetInstance pet, L1PcInstance pc)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 呼叫 `buildPacket(pet, pc)` |

`void buildPacket(L1PetInstance pet, L1PcInstance pc)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L40 `C(S_OPCODE_CHARPACK)` → L41 `H(pet.getX())` → L42 `H(pet.getY())` → L43 `D(pet.getId())` → L44 `H(pet.getGfxId())` → L45 `C(pet.getStatus())` → L46 `C(pet.getHeading())` → L47 `C(pet.getChaLightSize())` → L48 `C(pet.getMoveSpeed())` → L49 `D((int) pet.getExp())` → L50 `H(pet.getTempLawful())` → L51 `S(pet.getName())` → L52 `S(pet.getTitle())` → L59 `C(status)` → L60 `D(0x00000000)` → L61 `S(null)` → L77 `S(stringBuilder.toString())` → L79 `C(0x00)` |
| `if ((pet.getMaster() != null) && (pet.getMaster().getId() == pc.getId()))` | L83 `C(100 * pet.getCurrentHp() / pet.getMaxHp())` |
| `else` | L86 `C(0xFF)` |
| 本方法此處 | L89 `C(0x00)` → L90 `C(0x00)` → L92 `C(0x00)` → L93 `C(0xFF)` → L94 `C(0xFF)` |

### S_NPCPack_Signboard

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_Signboard.java`；繼承 `ServerBasePacket`。

`S_NPCPack_Signboard(L1SignboardInstance signboard)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 `C(S_OPCODE_CHARPACK)` → L20 `H(signboard.getX())` → L21 `H(signboard.getY())` → L22 `D(signboard.getId())` → L23 `H(signboard.getGfxId())` → L24 `C(0x00)` → L25 `C(this.getDirection(signboard.getHeading()))` → L26 `C(0x00)` → L27 `C(0x00)` → L28 `D(0x00000000)` → L29 `H(0x0000)` → L30 `S(null)` → L31 `S(signboard.getName())` → L32 `C(0x00)` → L33 `D(0x00000000)` → L34 `S(null)` → L35 `S(null)` → L46 `C(0x00)` → L48 `C(0xFF)` → L49 `C(0x00)` → L50 `C(0x00)` → L51 `C(0x00)` → L52 `C(0xFF)` → L53 `C(0xFF)` |

### S_NPCPack_Summon

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCPack_Summon.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`STATUS_POISON=1`。

`S_NPCPack_Summon(L1SummonInstance pet, L1PcInstance pc)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 呼叫 `buildPacket(pet, pc, true)` |

`S_NPCPack_Summon(L1SummonInstance pet, L1PcInstance pc, boolean isCheckMaster)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 呼叫 `buildPacket(pet, pc, isCheckMaster)` |

`void buildPacket(L1SummonInstance pet, L1PcInstance pc, boolean isCheckMaster)`（L41）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L43 `C(S_OPCODE_CHARPACK)` → L44 `H(pet.getX())` → L45 `H(pet.getY())` → L46 `D(pet.getId())` → L47 `H(pet.getGfxId())` → L48 `C(pet.getStatus())` → L49 `C(pet.getHeading())` → L50 `C(pet.getChaLightSize())` → L51 `C(pet.getMoveSpeed())` → L52 `D(0x00000000)` → L53 `H(0x0000)` → L54 `S(pet.getNameId())` → L55 `S(pet.getTitle())` → L62 `C(status)` → L63 `D(0x00000000)` → L64 `S(null)` → L88 `S(stringBuilder.toString())` → L90 `C(0x00)` |
| `if (!(pet.getMaster() instanceof L1NpcInstance)) / if ((pet.getMaster() != null) && (pet.getMaster().getId() == pc.getId()))` | L96 `C(percent)` |
| `if (!(pet.getMaster() instanceof L1NpcInstance)) / else` | L98 `C(0xFF)` |
| `else` | L101 `C(0xFF)` |
| 本方法此處 | L104 `C(0x00)` → L105 `C(0x00)` → L107 `C(0x00)` → L108 `C(0xFF)` → L109 `C(0xFF)` |

### S_NPCTalkActionTPUrl

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCTalkActionTPUrl.java`；繼承 `ServerBasePacket`。

`S_NPCTalkActionTPUrl(L1NpcTalkData cha, Object[] prices, int objid)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 呼叫 `buildPacket(cha, prices, objid)` |

`void buildPacket(L1NpcTalkData npc, Object[] prices, int objid)`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L28 `C(S_OPCODE_SHOWHTML)` → L29 `D(objid)` → L30 `S(htmlid)` → L31 `H(0x01)` → L32 `H(prices.length)` |
| `for (final Object price : prices)` | L35 `S(String.valueOf(((Integer) price).intValue()))` |

### S_NPCTalkReturn

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_NPCTalkReturn.java`；繼承 `ServerBasePacket`。

`S_NPCTalkReturn(int objid, String htmlid, L1PcInstance pc, List<Integer> list)`（L37）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L39 `C(S_OPCODE_SHOWHTML)` → L40 `D(objid)` → L41 `S(htmlid)` → L42 `H(0x01)` → L43 `H(0x0b)` |
| `for (final Integer v : list)` | L49 `S(datum.getNameId())` |
| `if (list.size() < 11) / for (int i = 0 ; i < x ; i++)` | L54 `S(" ")` |

`S_NPCTalkReturn(int objid, String htmlid, List<L1ItemInstance> list, L1PcInstance pc)`（L67）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L69 `C(S_OPCODE_SHOWHTML)` → L70 `D(objid)` → L71 `S(htmlid)` → L72 `H(0x01)` → L73 `H(0x0b)` |
| `for (final L1ItemInstance datum : list)` | L78 `S(datum.getViewName())` |
| `if (list.size() < 11) / for (int i = 0 ; i < x ; i++)` | L83 `S(" ")` |

`S_NPCTalkReturn(L1NpcTalkData npc, int objid, int action, String[] data)`（L96）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L110 呼叫 `buildPacket(objid, htmlid, data)` |

`S_NPCTalkReturn(L1NpcTalkData npc, int objid, int action)`（L120）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L121 呼叫 `this(npc, objid, action, null)` |

`S_NPCTalkReturn(int objid, String htmlid, String[] data)`（L131）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L132 呼叫 `buildPacket(objid, htmlid, data)` |

`S_NPCTalkReturn(int objid, String htmlid)`（L141）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L142 呼叫 `buildPacket(objid, htmlid, null)` |

`S_NPCTalkReturn(int objid, L1NpcHtml html)`（L151）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L152 呼叫 `buildPacket(objid, html.getName(), html.getArgs())` |

`S_NPCTalkReturn(int objid,String htmlid,String[] data,UiOperationContext.Offer offer, UiOperationContext.Ticket prerequisite)`（L157）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L159 呼叫 `this(objid,htmlid,data)` |

`void buildPacket(int objid, String htmlid, String[] data)`（L165）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L167 `C(S_OPCODE_SHOWHTML)` → L168 `D(objid)` → L169 `S(htmlid)` |
| `if ((data != null) && (1 <= data.length))` | L171 `H(0x01)` → L172 `H(data.length)` |
| `if ((data != null) && (1 <= data.length)) / for (final String datum : data)` | L174 `S(datum)` |
| `else` | L178 `H(0x00)` → L179 `H(0x00)` |

### S_OtherCharPacks

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_OtherCharPacks.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`STATUS_POISON=1`、`STATUS_INVISIBLE=2`、`STATUS_PC=4`、`STATUS_BRAVE=16`、`STATUS_ELFBRAVE=32`、`STATUS_FASTMOVABLE=64`。

`S_OtherCharPacks(L1PcInstance pc)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 呼叫 `this(pc, null)` |

`S_OtherCharPacks(L1PcInstance pc, L1PcInstance viewer)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L60 `C(S_OPCODE_CHARPACK)` → L61 `H(pc.getX())` → L62 `H(pc.getY())` → L63 `D(pc.getId())` |
| `if (pc.isDead())` | L66 `H(pc.getTempCharGfxAtDead())` |
| `else` | L69 `H(pc.getTempCharGfx())` |
| `if (pc.isDead())` | L73 `C(pc.getStatus())` |
| `else` | L75 `C(pc.getCurrentWeapon())` |
| 本方法此處 | L78 `C(pc.getHeading())` → L80 `C(pc.getChaLightSize())` → L81 `C(pc.getMoveSpeed())` → L82 `D(0x00000000)` → L83 `H(pc.getLawful())` → L91 `S(stringBuilder.toString())` → L92 `S(hideOthersIdentity ? "" : pc.getTitle())` → L93 `C(status)` → L94 `D(hideOthersIdentity ? 0 : pc.getClanid())` → L95 `S(hideOthersIdentity ? "" : pc.getClanname())` → L96 `S(null)` → L107 `C(hideOthersIdentity ? 0 : (pc.getClanRank() << 4))` → L110 `C(0xff)` |
| `if (pc.hasSkillEffect(STATUS_BRAVE3))` | L112 `C(0x08)` |
| `else` | L115 `C(0x00)` |
| 本方法此處 | L117 `C(0x00)` → L118 `C(0x00)` → L119 `C(0xff)` → L120 `C(0xff)` |

### S_OwnCharAttrDef

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_OwnCharAttrDef.java`；繼承 `ServerBasePacket`。

`S_OwnCharAttrDef(L1PcInstance pc)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(pc)` |

`void buildPacket(L1PcInstance pc)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_OWNCHARATTRDEF)` → L28 `C(ac)` → L29 `H(pc.getFire())` → L30 `H(pc.getWater())` → L31 `H(pc.getWind())` → L32 `H(pc.getEarth())` |

`S_OwnCharAttrDef()`（L39）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L40 `C(S_OPCODE_OWNCHARATTRDEF)` → L41 `C(-99)` → L42 `H(90)` → L43 `H(85)` → L44 `H(80)` → L45 `H(75)` |

### S_OwnCharPack

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_OwnCharPack.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`STATUS_POISON=1`、`STATUS_INVISIBLE=2`、`STATUS_PC=4`、`STATUS_BRAVE=16`、`STATUS_ELFBRAVE=32`、`STATUS_FASTMOVABLE=64`、`STATUS_GHOST=128`。

`S_OwnCharPack(L1PcInstance pc)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L35 呼叫 `buildPacket(pc)` |

`void buildPacket(L1PcInstance pc)`（L39）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L67 `C(S_OPCODE_CHARPACK)` → L68 `H(pc.getX())` → L69 `H(pc.getY())` → L70 `D(pc.getId())` |
| `if (pc.isDead())` | L73 `H(pc.getTempCharGfxAtDead())` |
| `else` | L76 `H(pc.getTempCharGfx())` |
| `if (pc.isDead())` | L80 `C(pc.getStatus())` |
| `else` | L82 `C(pc.getCurrentWeapon())` |
| 本方法此處 | L85 `C(pc.getHeading())` → L87 `C(pc.getOwnLightSize())` → L88 `C(pc.getMoveSpeed())` → L89 `D((int) pc.getExp())` → L90 `H(pc.getLawful())` → L98 `S(stringBuilder.toString())` → L99 `S(pc.getTitle())` → L100 `C(status)` → L101 `D(pc.getClanid())` → L102 `S(pc.getClanname())` → L103 `S(null)` → L117 `C(pc.getClanRank() > 0 ? pc.getClanRank() << 4 : 0xb0)` |
| `if (pc.isInParty())` | L120 `C(100 * pc.getCurrentHp() / pc.getMaxHp())` |
| `else` | L123 `C(0xff)` |
| `if (pc.hasSkillEffect(STATUS_BRAVE3))` | L127 `C(0x08)` |
| `else` | L130 `C(0x00)` |
| 本方法此處 | L132 `C(0x00)` → L133 `C(0x00)` → L134 `C(0xff)` → L135 `C(0xff)` |

### S_OwnCharStatus

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_OwnCharStatus.java`；繼承 `ServerBasePacket`。

`S_OwnCharStatus(L1PcInstance pc)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 `C(S_OPCODE_OWNCHARSTATUS)` → L32 `D(pc.getId())` → L34 `C(pc.getLevel())` → L36 `Exp(pc.getExp())` → L38 `C(pc.getStr())` → L39 `C(pc.getInt())` → L40 `C(pc.getWis())` → L41 `C(pc.getDex())` → L42 `C(pc.getCon())` → L43 `C(pc.getCha())` |
| `if (com.lineage.config.ConfigClient.isROClient())` | L47 `D(pc.getCurrentHp())` → L48 `D(pc.getMaxHp())` → L49 `D(pc.getCurrentMp())` → L50 `D(pc.getMaxMp())` |
| `else` | L52 `H(pc.getCurrentHp())` → L53 `H(pc.getMaxHp())` → L54 `H(pc.getCurrentMp())` → L55 `H(pc.getMaxMp())` |
| 本方法此處 | L57 呼叫 `writeAc(pc)` → L58 `D(time)` → L59 `C(pc.get_food())` → L60 `C(pc.getInventory().getWeight240())` → L61 `H(pc.getLawful())` → L63 `H(pc.getFire())` → L64 `H(pc.getWater())` → L65 `H(pc.getWind())` → L66 `H(pc.getEarth())` → L67 `D(pc.getKillCount())` |

`S_OwnCharStatus(L1PcInstance pc, int str)`（L76）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L80 `C(S_OPCODE_OWNCHARSTATUS)` → L81 `D(pc.getId())` → L83 `C(pc.getLevel())` → L85 `Exp(pc.getExp())` → L87 `C(str)` → L88 `C(pc.getInt())` → L89 `C(pc.getWis())` → L90 `C(pc.getDex())` → L91 `C(pc.getCon())` → L92 `C(pc.getCha())` |
| `if (com.lineage.config.ConfigClient.isROClient())` | L96 `D(pc.getCurrentHp())` → L97 `D(pc.getMaxHp())` → L98 `D(pc.getCurrentMp())` → L99 `D(pc.getMaxMp())` |
| `else` | L101 `H(pc.getCurrentHp())` → L102 `H(pc.getMaxHp())` → L103 `H(pc.getCurrentMp())` → L104 `H(pc.getMaxMp())` |
| 本方法此處 | L106 呼叫 `writeAc(pc)` → L107 `D(time)` → L108 `C(pc.get_food())` → L109 `C(pc.getInventory().getWeight240())` → L110 `H(pc.getLawful())` → L111 `H(pc.getFire())` → L112 `H(pc.getWater())` → L113 `H(pc.getWind())` → L114 `H(pc.getEarth())` → L115 `D(pc.getKillCount())` |

`void writeAc(L1PcInstance pc)`（L132）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (com.lineage.config.ConfigClient.isROClient())` | L134 `D(pc.getAc())` |
| `else` | L136 `C(pc.getAc())` |

### S_OwnCharStatus2

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_OwnCharStatus2.java`；繼承 `ServerBasePacket`。

`S_OwnCharStatus2(L1PcInstance pc)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 呼叫 `buildPacket(pc)` |

`void buildPacket(L1PcInstance pc)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 `C(S_OPCODE_OWNCHARSTATUS2)` → L31 `C(pc.getStr())` → L32 `C(pc.getInt())` → L33 `C(pc.getWis())` → L34 `C(pc.getDex())` → L35 `C(pc.getCon())` → L36 `C(pc.getCha())` → L37 `C(pc.getInventory().getWeight240())` |

`S_OwnCharStatus2(L1PcInstance pc, int str)`（L45）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L46 `C(S_OPCODE_OWNCHARSTATUS2)` → L47 `C(str)` → L48 `C(pc.getInt())` → L49 `C(pc.getWis())` → L50 `C(pc.getDex())` → L51 `C(pc.getCon())` → L52 `C(pc.getCha())` → L53 `C(pc.getInventory().getWeight240())` |

### S_PacketBox

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBox.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`MSG_DUEL=5`、`MSG_SMS_SENT=6`、`MSG_MARRIED=9`、`WEIGHT=10`、`FOOD=11`、`HTML_UB=14`、`MSG_ELF=15`、`ADD_EXCLUDE2=17`、`ADD_EXCLUDE=18`、`REM_EXCLUDE=19`、`ICONS1=20`、`ICONS2=21`、`ICON_AURA=22`、`MSG_TOWN_LEADER=23`、`PLEDGE_TWO=24`、`PLEDGE_REFRESH_PLUS=25`、`PLEDGE_REFRESH_MINUS=26`、`MSG_RANK_CHANGED=27`、`PLEDGE_ONE=29`、`MSG_WIN_LASTAVARD=30`、`MSG_FEEL_GOOD=31`、`SOMETHING1=33`、`ICON_BLUEPOTION=34`、`ICON_POLYMORPH=35`、`ICON_CHATBAN=36`、`GUI_PET_EQUIPMENT=37`、`HTML_CLAN1=38`、`ICON_I2H=40`、`CHARACTER_CONFIG=41`、`LOGOUT=42`、`MSG_CANT_LOGOUT=43`、`MSG_COLOSSEUM=49`、`HTML_CLAN2=51`、`FISHING=55`、`DOLL=56`、`WISDOM_POTION=57`、`DEL_ICON=59`、`ALLIANCE_LIST=62`、`DMG=74`、`TGDMG=75`、`LEAVES=82`、`BLUE_EXPREPLY=86`、`DRAGON_BLOOD_ICON=100`、`DRAGON=101`、`EVASION_UPDATE=132`、`WEAPON_DURABILITY=138`、`BOOKMARK_EXPAND=141`、`PROTECTOR_ICON=144`、`APPRENTICE_HTML=146`、`APPRENTICE_ICON=147`、`ITEM_STATUS_RENEW=149`、`MAP_LIMIT_TIMER=153`、`WEAPON_BUFF_ICON=154`、`POISON_ICON=161`、`HTML_PLEDGE_ANNOUNCE=167`、`HTML_PLEDGE_REALEASE_ANNOUNCE=168`、`HTML_PLEDGE_WRITE_NOTES=169`、`HTML_PLEDGE_MEMBERS=170`、`HTML_PLEDGE_ONLINE_MEMBERS=171`、`PLEDGE_EMBLEM_STATUS=173`、`TOWN_TELEPORT=176`、`NONE_TIME_ICON=180`、`HTML_CLAN_WARHOUSE_RECORD=117`。

`S_PacketBox(int subCode, String msg, int value)`（L393）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L394 `C(S_OPCODE_PACKETBOX)` → L395 `C(LEAVES)` → L396 `S(msg)` → L397 `H(value)` |

`S_PacketBox(L1PcInstance master, ArrayList<L1PcInstance> totalList)`（L406）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L408 `C(S_OPCODE_PACKETBOX)` → L409 `C(APPRENTICE_HTML)` → L410 `C(totalList.size())` |
| `for (L1PcInstance l1Char : totalList)` | L414 `S(l1Char.getName())` → L415 `C(l1Char.getLevel())` → L416 `C(l1Char.getType())` |
| 本方法此處 | L420 `S(master.getName())` → L421 `C(master.getLevel())` → L422 `C(master.getType())` |

`S_PacketBox(int subCode)`（L426）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L427 `C(S_OPCODE_PACKETBOX)` → L428 `C(subCode)` |
| `switch (subCode) / case DEL_ICON:` | L444 `H(0x0000)` |

`S_PacketBox(int subCode, int value, String name)`（L453）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L454 `C(S_OPCODE_PACKETBOX)` → L455 `C(subCode)` |
| `switch (subCode) / case MSG_RANK_CHANGED:` | L459 `C(value)` → L460 `S(name)` |

`S_PacketBox(int subCode, int value)`（L466）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L467 `C(S_OPCODE_PACKETBOX)` → L468 `C(subCode)` |
| `switch (subCode) / case TGDMG:` | L474 `C(time)` |
| `switch (subCode) / case ICON_POLYMORPH:` | L481 `H(value)` |
| `switch (subCode) / case WISDOM_POTION:` | L492 `C(0x2c)` → L493 `H(value)` |
| `switch (subCode) / case EVASION_UPDATE:` | L501 `C(value)` |
| `switch (subCode) / case MSG_COLOSSEUM:` | L507 `C(value)` |
| `switch (subCode) / case DOLL:` | L513 `H(value)` |
| `switch (subCode) / case MAP_LIMIT_TIMER:` | L517 `D(value)` |
| `switch (subCode) / case WEAPON_DURABILITY:` | L521 `C(value)` |
| `switch (subCode) / case PLEDGE_EMBLEM_STATUS:` | L524 `C(1)` |
| `switch (subCode) / if (value==1) / case PLEDGE_EMBLEM_STATUS:` | L527 `C(1)` |
| `switch (subCode) / else / case PLEDGE_EMBLEM_STATUS:` | L529 `C(0)` |
| `switch (subCode) / case PLEDGE_EMBLEM_STATUS:` | L531 `D(0x00)` |
| `switch (subCode) / case 121:` | L534 `D(value)` |

`S_PacketBox(int subCode, int type, int time)`（L544）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L545 `C(S_OPCODE_PACKETBOX)` → L546 `C(subCode)` |
| `switch (subCode) / case MSG_DUEL:` | L551 `D(type)` → L552 `D(time)` |
| `switch (subCode) / if (type == 0) / case 12:` | L557 `C(time)` → L558 `C(100)` → L559 `C(type)` |
| `switch (subCode) / else if (type == 1) / case 12:` | L561 `C(0)` → L562 `C(time)` → L563 `C(type)` |
| `switch (subCode) / case 20:` | L568 `C(type)` → L569 `H(time)` |
| `switch (subCode) / case 21:` | L573 `C(time)` → L574 `C(type)` |
| `switch (subCode) / case 22:` | L578 `H(type)` → L579 `H(time)` |
| `switch (subCode) / case BLUE_EXPREPLY:` | L583 `C(0x70)` → L584 `C(0x01)` → L585 `C(type)` → L586 `H(time)` |
| `switch (subCode) / case DRAGON_BLOOD_ICON:` | L590 `C(type)` → L591 `H(time)` |
| `switch (subCode) / case 114:` | L595 `D(type)` → L596 `D(time)` |
| `switch (subCode) / case APPRENTICE_ICON:` | L600 `C(type)` → L601 `C(time)` |
| `switch (subCode) / case WEAPON_BUFF_ICON:` | L605 `H(time)` → L606 `H(type)` |
| `switch (subCode) / case ITEM_STATUS_RENEW:` | L610 `D(type)` → L611 `C(time)` |
| `switch (subCode) / case NONE_TIME_ICON:` | L615 `C(type)` → L616 `D(time)` → L617 `D(0x00000D67)` → L618 `H(0x00)` |

`S_PacketBox(int subCode, int type, int time, int stringId)`（L627）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L628 `C(S_OPCODE_PACKETBOX)` → L629 `C(subCode)` |
| `switch (subCode) / case NONE_TIME_ICON:` | L633 `C(type)` → L634 `D(time)` → L635 `D(stringId)` → L636 `H(0x00)` |

`S_PacketBox(int subCode, String name)`（L645）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L646 `C(S_OPCODE_PACKETBOX)` → L647 `C(subCode)` |
| `switch (subCode) / case HTML_PLEDGE_REALEASE_ANNOUNCE:` | L655 `S(name)` |

`S_PacketBox(int subCode, Object[] names)`（L664）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L665 `C(S_OPCODE_PACKETBOX)` → L666 `C(subCode)` |
| `switch (subCode) / case HTML_PLEDGE_ONLINE_MEMBERS:` | L671 `C(names.length)` |
| `switch (subCode) / for (final Object name : names) / case HTML_PLEDGE_ONLINE_MEMBERS:` | L673 `S(name.toString())` |

`S_PacketBox(int subCode, L1PcInstance[] names)`（L683）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L684 `C(S_OPCODE_PACKETBOX)` → L685 `C(subCode)` |
| `switch (subCode) / case HTML_PLEDGE_ONLINE_MEMBERS:` | L689 `H(names.length)` |
| `switch (subCode) / for (final L1PcInstance name : names) / case HTML_PLEDGE_ONLINE_MEMBERS:` | L691 `S(name.getName())` → L692 `C(0x00)` |

### S_PacketBoxActiveSpells

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxActiveSpells.java`；繼承 `ServerBasePacket`。

`S_PacketBoxActiveSpells(L1PcInstance pc)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 `C(S_OPCODE_PACKETBOX)` → L19 `C(0x14)` |
| `for (int i : type) / switch(i) / case 72:` | L24 `D((int) (System.currentTimeMillis() / 1000))` |
| `for (int i : type) / switch(i) / default:` | L27 `C(i)` |

### S_PacketBoxConfig

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxConfig.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`CHARACTER_CONFIG=0x29`。

`S_PacketBoxConfig(L1Config config)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 `C(S_OPCODE_PACKETBOX)` → L28 `C(CHARACTER_CONFIG)` → L29 `D(length)` → L30 `Byte(data)` |

### S_PacketBoxCooking

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxCooking.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`COOK_WINDOW=52`、`ICON_COOKING=53`。

`S_PacketBoxCooking(int value)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 `C(S_OPCODE_PACKETBOX)` → L19 `C(COOK_WINDOW)` → L20 `C(0xdb)` → L21 `C(0x31)` → L22 `C(0xdf)` → L23 `C(0x02)` → L24 `C(0x01)` → L25 `C(value)` |

`S_PacketBoxCooking(L1PcInstance pc, int type, int time)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 `C(S_OPCODE_PACKETBOX)` → L31 `C(ICON_COOKING)` |
| `switch (type) / case 0x07:` | L39 `C(pc.getStr())` → L40 `C(pc.getInt())` → L41 `C(pc.getWis())` → L42 `C(pc.getDex())` → L43 `C(pc.getCon())` → L44 `C(pc.getCha())` → L45 `H(food)` → L46 `C(type)` → L47 `C(0x24)` → L48 `H(time)` → L49 `C(0x0)` |
| `switch (type) / case 54:` | L53 `C(0x00)` → L54 `C(0x00)` → L55 `C(0x00)` → L56 `C(0x00)` → L57 `C(0x00)` → L58 `C(0x00)` → L59 `H(0x00)` → L60 `C(type)` → L61 `C(0x2a)` → L62 `H(time)` → L63 `C(0x0)` |
| `switch (type) / default:` | L67 `C(pc.getStr())` → L68 `C(pc.getInt())` → L69 `C(pc.getWis())` → L70 `C(pc.getDex())` → L71 `C(pc.getCon())` → L72 `C(pc.getCha())` → L73 `H(food)` → L74 `C(type)` → L75 `C(0x26)` → L76 `H(time)` → L77 `C(0x0)` |

### S_PacketBoxDk

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxDk.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`LV1=0x01`、`LV2=0x02`、`LV3=0x03`。

`S_PacketBoxDk(int lv)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_PACKETBOX)` → L23 `C(0x4b)` → L24 `C(lv)` |

### S_PacketBoxExp

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxExp.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`LEAVES=82`。

`S_PacketBoxExp(int exp)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_PACKETBOX)` → L21 `C(LEAVES)` → L22 `C(exp)` → L23 `C(0x00)` → L24 `C(0x00)` → L25 `C(0x00)` |

`S_PacketBoxExp()`（L32）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L33 呼叫 `this(0)` |

### S_PacketBoxGame

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxGame.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`GAMESTART=64`、`TIMESTART=65`、`GAMEINFO=66`、`GAMEOVER=69`、`GAMECLEAR=70`、`STARTTIME=71`、`STARTTIMECLEAR=72`。

`S_PacketBoxGame(int subCode)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 `C(S_OPCODE_PACKETBOX)` → L38 `C(subCode)` |

`S_PacketBoxGame(int subCode, int value)`（L56）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L57 `C(S_OPCODE_PACKETBOX)` → L58 `C(subCode)` |
| `switch (subCode) / case GAMEOVER:` | L63 `C(value)` |
| `switch (subCode) / case STARTTIME:` | L67 `H(value)` |

`S_PacketBoxGame(ArrayList<StringBuilder> list)`（L77）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L78 `C(S_OPCODE_PACKETBOX)` → L79 `C(GAMEINFO)` → L80 `C(list.size())` → L81 `C(0x00)` → L82 `C(0x00)` → L83 `C(0x00)` |
| `if (list != null) / for (final StringBuilder string : list)` | L87 `S(string.toString())` |

`S_PacketBoxGame(StringBuilder title, ArrayList<StringBuilder> list)`（L99）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L100 `C(S_OPCODE_PACKETBOX)` → L101 `C(GAMEINFO)` → L102 `C(list.size() + 1)` → L103 `C(0x00)` → L104 `C(0x00)` → L105 `C(0x00)` → L107 `S(title.toString())` |
| `if (list != null) / for (final StringBuilder c : list)` | L110 `S(c.toString())` |

### S_PacketBoxGm

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxGm.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`CALL_SOMETHING=0x2d`。

`S_PacketBoxGm(L1PcInstance srcpc, int mode)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L35 `C(S_OPCODE_PACKETBOX)` → L36 `C(CALL_SOMETHING)` |
| `if (srcpc.isGm())` | L38 呼叫 `callSomething(srcpc, mode)` |

`void callSomething(L1PcInstance srcpc, int mode)`（L43）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L47 `C(World.get().getAllPlayers().size())` |
| `while (itr.hasNext()) / if (acc == null)` | L55 `D(0)` |
| `while (itr.hasNext()) / else` | L63 `D(time)` |
| `while (itr.hasNext())` | L67 `S(mode + ":" + pc.getName())` → L69 `S(String.valueOf(pc.getLevel()))` |

### S_PacketBoxGree

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxGree.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`GREEN_MESSAGE=0x54`、`SECRETSTORY_GFX=0x53`。

`S_PacketBoxGree(String msg)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 `C(S_OPCODE_PACKETBOX)` → L32 `C(GREEN_MESSAGE)` → L33 `C(0x02)` → L34 `S(msg)` |

`S_PacketBoxGree(int type)`（L44）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L45 `C(S_OPCODE_PACKETBOX)` → L46 `C(SECRETSTORY_GFX)` → L47 `D(type)` → L48 `C(0x00)` → L49 `C(0x00)` → L50 `C(0x00)` |

### S_PacketBoxHpMsg

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxHpMsg.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`MSG_FEEL_GOOD=31`。

`S_PacketBoxHpMsg()`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_PACKETBOX)` → L18 `C(MSG_FEEL_GOOD)` |

### S_PacketBoxIcon1

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxIcon1.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`_icons_1=0x14`、`_dodge_up=0x58`、`_dodge_down=0x65`。

`S_PacketBoxIcon1(int type, int time)`（L14）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L15 `C(S_OPCODE_PACKETBOX)` → L16 `C(_icons_1)` → L17 `H(time)` → L18 `C(type)` |

`S_PacketBoxIcon1(boolean type, int i)`（L27）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L28 `C(S_OPCODE_PACKETBOX)` |
| `if (type)` | L30 `C(_dodge_up)` → L31 `C(i)` |
| `else` | L34 `C(_dodge_down)` → L35 `C(i)` |

### S_PacketBoxIcon2

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxIcon2.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`ICONS2=0x15`。

`S_PacketBoxIcon2(int type, int time)`（L10）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L11 `C(S_OPCODE_PACKETBOX)` → L12 `C(ICONS2)` → L13 `H(time)` → L14 `C(type)` |

### S_PacketBoxIconAura

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxIconAura.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`ICON_AURA=0x16`、`ICON_OS=0x7D`、`ICON_E3=0xE3`。

`S_PacketBoxIconAura(int iconid, int time)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 `C(S_OPCODE_PACKETBOX)` → L28 `C(ICON_AURA)` → L29 `C(iconid)` → L30 `H(time)` |

`S_PacketBoxIconAura(int iconid, int time, int type)`（L40）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L41 `C(S_OPCODE_PACKETBOX)` → L42 `C(ICON_AURA)` → L43 `C(iconid)` → L44 `H(time)` → L45 `C(type)` |

### S_PacketBoxItemLv

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxItemLv.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`MSG_LEVEL_OVER=0x0c`。

`S_PacketBoxItemLv(int minLv, int maxLv)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_PACKETBOX)` → L25 `C(MSG_LEVEL_OVER)` → L26 `C(minLv)` → L27 `C(maxLv)` |

`S_PacketBoxItemLv(int opid)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L36 `C(opid)` → L37 `C(MSG_LEVEL_OVER)` → L38 `C(10)` → L39 `C(1249)` |

### S_PacketBoxLoc

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxLoc.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`SEND_LOC=111`。

`S_PacketBoxLoc(String name,int map,int x,int y, int zone)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_PACKETBOX)` → L25 `C(SEND_LOC)` → L26 `S(name)` → L27 `H(map)` → L28 `H(x)` → L29 `H(y)` → L30 `D(zone)` |

### S_PacketBoxMapTimer

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxMapTimer.java`；繼承 `ServerBasePacket`。

`S_PacketBoxMapTimer(L1PcInstance pc)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_PACKETBOX)` → L21 `C(0x9f)` → L22 `D(mapLimitList.size())` |
| `for (final L1MapsLimitTime mapLimit : mapLimitList)` | L29 `D(order_id)` → L30 `S(mapLimit.getMapName())` → L31 `D(time_str)` |

### S_PacketBoxParty

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxParty.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`MSG_PARTY1=0x68`、`MSG_PARTY2=0x69`、`MSG_PARTY3=0x6e`、`MSG_LEADER=0x6a`。

`S_PacketBoxParty(L1PcInstance pc, L1Party party)`（L50）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L54 `C(S_OPCODE_PACKETBOX)` → L55 `C(MSG_PARTY1)` → L56 `C(map.size() - 1)` |
| `for (final Iterator<Entry<Integer, L1PcInstance>> iter = map.iterator(); iter.hasNext();)` | L69 `D(tgpc.getId())` → L70 `S(tgpc.getName())` → L75 `C((int) ((nowhp / maxhp) * 100D))` → L76 `D(tgpc.getMapId())` → L77 `H(tgpc.getX())` → L78 `H(tgpc.getY())` |

`S_PacketBoxParty(L1Party party, L1PcInstance pc)`（L95）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L99 `C(S_OPCODE_PACKETBOX)` → L100 `C(MSG_PARTY3)` → L101 `C(map.size() - 1)` |
| `for (final Iterator<Entry<Integer, L1PcInstance>> iter = map.iterator(); iter.hasNext();)` | L114 `D(tgpc.getId())` → L115 `D(tgpc.getMapId())` → L116 `H(tgpc.getX())` → L117 `H(tgpc.getY())` |

`S_PacketBoxParty(L1PcInstance tgpc)`（L132）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L134 `C(S_OPCODE_PACKETBOX)` → L135 `C(MSG_PARTY2)` → L136 `D(tgpc.getId())` → L137 `S(tgpc.getName())` → L142 `C((int) ((nowhp / maxhp) * 100D))` → L143 `D(tgpc.getMapId())` → L144 `H(tgpc.getX())` → L145 `H(tgpc.getY())` |

`S_PacketBoxParty(int objid, String name)`（L159）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L161 `C(S_OPCODE_PACKETBOX)` → L162 `C(MSG_LEADER)` → L163 `D(objid)` → L164 `S(name)` |

### S_PacketBoxPledgeRecommend

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxPledgeRecommend.java`；繼承 `ServerBasePacket`。

`S_PacketBoxPledgeRecommend(boolean postStatus, int clan_id)`（L33）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L34 `C(S_OPCODE_CLANMATCHING)` → L35 `C(postStatus ? 0x00 : 0x01)` → L36 `C(0x00)` |

`S_PacketBoxPledgeRecommend( List<L1ClanRecommend> allClanRecommend, Map<Integer, CopyOnWriteArrayList<Integer>> allApply)`（L40）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L43 `C(S_OPCODE_CLANMATCHING)` → L44 `C(0x02)` → L45 `C(0x00)` → L52 `C(rows.size())` |
| `for (final RecommendationRow row : rows)` | L56 `D(clan.getClanId())` → L57 `S(clan.getClanName())` → L58 `S(clan.getLeaderName())` → L59 `D(clan.getLoginLevel())` → L60 `C(recommend.getTypeId())` → L61 `C(clan.getHouseId() > 0 ? 1 : 0)` → L62 `C(clan.getCastleId() > 0 ? 1 : 0)` → L63 `C(0)` → L64 `S(recommend.getTypeMessage())` → L65 `D(clan.getEmblemId())` |

`S_PacketBoxPledgeRecommend( Map<Integer, L1ClanRecommend> allRecommend, List<Integer> allClanApply)`（L70）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L73 `C(S_OPCODE_CLANMATCHING)` → L74 `C(0x03)` → L75 `C(0x00)` → L84 `C(rows.size())` |
| `for (final RecommendationRow row : rows)` | L89 `D(clan_id)` → L90 `C(0x00)` → L91 `D(clan_id)` → L92 `S(clan.getClanName())` → L93 `S(clan.getLeaderName())` → L94 `D(clan.getLoginLevel())` → L95 `C(recommend.getTypeId())` → L96 `C(clan.getHouseId() > 0 ? 1 : 0)` → L97 `C(clan.getCastleId() > 0 ? 1 : 0)` → L98 `C(0)` → L99 `S(recommend.getTypeMessage())` → L100 `D(clan.getEmblemId())` |

`S_PacketBoxPledgeRecommend(L1ClanRecommend recommend, List<Integer> apply)`（L105）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L107 `C(S_OPCODE_CLANMATCHING)` → L108 `C(0x04)` |
| `if (recommend == null)` | L111 `C(0x82)` |
| `else` | L114 `C(0x00)` → L115 `C(recommend.getTypeId())` → L116 `S(recommend.getTypeMessage())` |
| `else / if (apply == null)` | L119 `C(0x00)` |
| `else` | L123 `C(apply.size())` |
| `else / for (final int objid : apply)` | L126 `D(objid)` → L127 `C(0x00)` |
| `else / for (final int objid : apply) / if (name != null) / if (pc == null)` | L139 `C(0)` |
| `else / for (final int objid : apply) / if (name != null) / else` | L142 `C(1)` |
| `else / for (final int objid : apply) / else` | L146 `C(0)` |
| `else / for (final int objid : apply)` | L149 `S(name)` → L150 `C(pc == null ? 0 : pc.getType())` → L151 `H(pc == null ? 0 : pc.getLawful())` → L152 `H(pc == null ? 0 : pc.getLevel())` |

`S_PacketBoxPledgeRecommend(int type, int record_id, int acceptType, int actionType)`（L158）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L161 `C(S_OPCODE_CLANMATCHING)` → L162 `C(type)` → L163 `C(actionType)` → L164 `D(record_id)` → L165 `C(acceptType)` |

### S_PacketBoxProtection

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxProtection.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`JUSTICE_L1=0`、`JUSTICE_L2=1`、`JUSTICE_L3=2`、`EVIL_L1=3`、`EVIL_L2=4`、`EVIL_L3=5`、`ENCOUNTER=6`。

`S_PacketBoxProtection(int model, int type)`（L38）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L39 `C(S_OPCODE_PACKETBOX)` → L40 `C(0x72)` → L41 `D(model)` → L42 `D(type)` |

### S_PacketBoxSelect

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxSelect.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`LOGOUT=0x2a`。

`S_PacketBoxSelect()`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 `C(S_OPCODE_PACKETBOX)` → L22 `C(LOGOUT)` → L23 `C(0x00)` → L24 `C(0x00)` → L25 `C(0x00)` → L26 `C(0x00)` → L27 `C(0x00)` → L28 `C(0x00)` |

### S_PacketBoxTest

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxTest.java`；繼承 `ServerBasePacket`。

`S_PacketBoxTest()`（L12）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L14 `C(S_OPCODE_PACKETBOX)` → L15 `C(0x52)` → L16 `C(150)` → L17 `C(0x00)` → L18 `C(0x00)` → L19 `C(0x00)` |

`S_PacketBoxTest(byte ocid, int time)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_PACKETBOX)` → L25 `C(ocid)` → L26 `C(time)` → L27 `C(0x00)` → L28 `C(0x00)` → L29 `C(0x00)` |

`S_PacketBoxTest(int type, int time)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L35 `C(S_OPCODE_PACKETBOX)` → L36 `C(type)` → L37 `H(time)` → L38 `H(0x00)` |

`S_PacketBoxTest(int value, String[] clanName)`（L42）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L43 `C(S_OPCODE_PACKETBOX)` → L44 `C(value)` |
| `for (int i = 0; i < value ; i++)` | L46 `S(clanName[i])` |

`S_PacketBoxTest(int time)`（L51）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L52 `C(S_OPCODE_PACKETBOX)` → L53 `C(79)` → L54 `C(2)` → L55 `S("TEMP")` → L56 `S("AASS")` |

`S_PacketBoxTest(byte[] bs)`（L61）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `for (byte outbs : bs)` | L63 `C(outbs)` |

### S_PacketBoxThirdSpeed

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxThirdSpeed.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`CAKE=0x3c`、`CAKE_TYPE=0x08`。

`S_PacketBoxThirdSpeed(int time)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_PACKETBOX)` → L21 `C(CAKE)` → L22 `C(time >> 2)` → L23 `C(CAKE_TYPE)` |

### S_PacketBoxWar

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxWar.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`MSG_WAR_BEGIN=0`、`MSG_WAR_END=1`、`MSG_WAR_GOING=2`、`MSG_WAR_INITIATIVE=3`、`MSG_WAR_OCCUPY=4`、`MSG_WIN_LASTAVARD=30`、`MSG_WAR_OVER=79`。

`S_PacketBoxWar()`（L68）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L71 `C(S_OPCODE_PACKETBOX)` → L72 `C(MSG_WAR_OVER)` → L73 `C(count)` |
| `for (int key = 1; key < count ; key++) / if (clanName != null)` | L77 `S(clanName)` |
| `for (int key = 1; key < count ; key++) / else` | L80 `S(" ")` |

`S_PacketBoxWar(int subCode)`（L95）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L96 `C(S_OPCODE_PACKETBOX)` → L97 `C(subCode)` |

`S_PacketBoxWar(int subCode, int value)`（L111）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L112 `C(S_OPCODE_PACKETBOX)` → L113 `C(subCode)` → L115 `C(value)` → L116 `H(0x0000)` |

`S_PacketBoxWar(int subCode, int id, String name, String clanName)`（L131）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L132 `C(S_OPCODE_PACKETBOX)` → L133 `C(MSG_WIN_LASTAVARD)` → L135 `D(id)` → L136 `S(name)` → L137 `S(clanName)` |

### S_PacketBoxWaterLife

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxWaterLife.java`；繼承 `ServerBasePacket`。

`S_PacketBoxWaterLife()`（L13）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L14 `C(S_OPCODE_PACKETBOX)` → L15 `C(0x3b)` → L16 `H(0x0000)` |

### S_PacketBoxWindShackle

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxWindShackle.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`WIND_SHACKLE=0x2c`。

`S_PacketBoxWindShackle(int objectId, int time)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_PACKETBOX)` → L25 `C(WIND_SHACKLE)` → L26 `D(objectId)` → L27 `H(buffTime)` |

### S_PacketBoxWisdomPotion

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketBoxWisdomPotion.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`WISDOM_POTION=0x39`。

`S_PacketBoxWisdomPotion(int time)`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L16 `C(S_OPCODE_PACKETBOX)` → L17 `C(WISDOM_POTION)` → L18 `C(0x2c)` → L19 `H(time)` |

### S_PacketNew

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PacketNew.java`；繼承 `無`。

無直接 read/write 欄位；參見入口 。若為空處理器則不消費內容；繼承／委派不代表空封包。

### S_Paralysis

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Paralysis.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`TYPE_PARALYSIS=0x01`、`TYPE_PARALYSIS2=0x02`、`TYPE_SLEEP=0x03`、`TYPE_FREEZE=0x04`、`TYPE_STUN=0x05`、`TYPE_BIND=0x06`、`TYPE_TELEPORT_UNLOCK=0x07`。

`S_Paralysis(int type, boolean flag)`（L38）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L39 `C(S_OPCODE_PARALYSIS)` |
| `switch (type) / if (flag == true) / case TYPE_PARALYSIS:` | L43 `C(0x02)` |
| `switch (type) / else / case TYPE_PARALYSIS:` | L45 `C(0x03)` |
| `switch (type) / if (flag == true) / case TYPE_PARALYSIS2:` | L51 `C(0x04)` |
| `switch (type) / else / case TYPE_PARALYSIS2:` | L53 `C(0x05)` |
| `switch (type) / if (flag == true) / case TYPE_TELEPORT_UNLOCK:` | L59 `C(0x06)` |
| `switch (type) / else / case TYPE_TELEPORT_UNLOCK:` | L61 `C(0x07)` |
| `switch (type) / if (flag == true) / case TYPE_SLEEP:` | L67 `C(0x0a)` |
| `switch (type) / else / case TYPE_SLEEP:` | L69 `C(0x0b)` |
| `switch (type) / if (flag == true) / case TYPE_FREEZE:` | L75 `C(0x0c)` |
| `switch (type) / else / case TYPE_FREEZE:` | L77 `C(0x0d)` |
| `switch (type) / if (flag == true) / case TYPE_STUN:` | L83 `C(0x16)` |
| `switch (type) / else / case TYPE_STUN:` | L85 `C(0x17)` |
| `switch (type) / if (flag == true) / case TYPE_BIND:` | L91 `C(0x18)` |
| `switch (type) / else / case TYPE_BIND:` | L93 `C(0x19)` |

### S_Party

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Party.java`；繼承 `ServerBasePacket`。

`S_Party(String htmlid, int objid)`（L12）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L13 呼叫 `buildPacket(htmlid, objid, "", "", 0)` |

`S_Party(String htmlid, int objid, String partyname, String partymembers)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(htmlid, objid, partyname, partymembers, 1)` |

`void buildPacket(String htmlid, int objid, String partyname, String partymembers, int type)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_SHOWHTML)` → L27 `D(objid)` → L28 `S(htmlid)` → L29 `H(type)` → L30 `H(0x02)` → L31 `S(partyname)` → L32 `S(partymembers)` |

### S_PcPositionCorrection

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PcPositionCorrection.java`；繼承 `S_OwnCharPack`。

`S_PcPositionCorrection(L1PcInstance pc, long connectionGeneration, short mapId, int x, int y, int heading, long locationRevision)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L34 呼叫 `super(pc)` |

### S_PcPositionMapReset

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PcPositionMapReset.java`；繼承 `ServerBasePacket`。

`S_PcPositionMapReset(L1PcInstance pc, long connectionGeneration, short mapId, int x, int y, int heading, long locationRevision, boolean underwater)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L41 `C(S_OPCODE_MAPID)` → L42 `H(mapId)` → L43 `C(underwater ? 0x01 : 0x00)` |

### S_PetCtrlMenu

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PetCtrlMenu.java`；繼承 `ServerBasePacket`。

`S_PetCtrlMenu(L1Character cha, L1NpcInstance npc, boolean open)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_CHARRESET)` → L23 `C(0x0c)` |
| `if (open)` | L25 `H(0x03)` → L26 `D(0x00000000)` → L27 `D(npc.getId())` → L28 `D(0x00000000)` → L29 `H(npc.getX())` → L30 `H(npc.getY())` → L31 `S(npc.getName())` |
| `else` | L34 `H(0x0000)` → L35 `D(0x00000001)` → L36 `D(npc.getId())` → L37 `S(null)` |

### S_PetEquipment

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PetEquipment.java`；繼承 `ServerBasePacket`。

`S_PetEquipment(int data, L1PetInstance pet, int listNo)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 `C(S_OPCODE_PACKETBOX)` → L20 `C(S_PacketBox.GUI_PET_EQUIPMENT)` → L21 `C(data)` → L22 `D(pet.getId())` → L23 `C(listNo)` → L24 `C(pet.getAc())` |

### S_PetInventory

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PetInventory.java`；繼承 `ServerBasePacket`。

`S_PetInventory(L1PetInstance pet, boolean b)`（L50）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L51 呼叫 `isTrue(pet)` |

`void isTrue(L1PetInstance pet)`（L55）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L58 `C(S_OPCODE_SHOWRETRIEVELIST)` → L59 `D(pet.getId())` → L60 `H(itemList.size())` → L61 `C(0x0b)` |
| `for (final L1ItemInstance item : itemList) / if (item != null)` | L64 `D(item.getId())` → L66 `C(resolvePetInventoryItemType(item))` → L67 `H(item.get_gfxid())` → L68 `C(L1Item.toPacketBless(item.getBless()))` → L69 `D((int) Math.min(item.getCount(), 2000000000))` |
| `for (final L1ItemInstance item : itemList) / if (item != null) / if (item.getItem().getType2() == 0 && item.getItem().getType() == 11 && item.isEquipped())` | L72 `C(item.isIdentified() ? 3 : 2)` |
| `for (final L1ItemInstance item : itemList) / if (item != null) / else` | L74 `C(item.isIdentified() ? 1 : 0)` |
| `for (final L1ItemInstance item : itemList) / if (item != null)` | L76 `S(item.getViewName())` |
| 本方法此處 | L79 `C(pet.getAc())` |

### S_PetList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PetList.java`；繼承 `ServerBasePacket`。

`S_PetList(int npcObjId, L1PcInstance pc)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(npcObjId, pc)` |

`S_PetList(int npcObjId, L1PcInstance pc, List<L1ItemInstance> amuletList)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 呼叫 `buildPacketFromList(amuletList)` |

`void buildPacket(int npcObjId, L1PcInstance pc)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (amuletList.size() != 0)` | L49 呼叫 `writeSelectListBody(amuletList)` |

`void buildPacketFromList(List<L1ItemInstance> amuletList)`（L53）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (amuletList != null && amuletList.size() != 0)` | L55 呼叫 `writeSelectListBody(amuletList)` |

`void writeSelectListBody(List<L1ItemInstance> amuletList)`（L59）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L60 `C(S_OPCODE_SELECTLIST)` → L61 `D(0x00000046)` → L62 `H(amuletList.size())` |
| `for (final L1ItemInstance item : amuletList)` | L64 `D(item.getId())` → L65 `C((int) Math.min(item.getCount(), 2000000000))` |

### S_PetMenuPacket

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PetMenuPacket.java`；繼承 `ServerBasePacket`。

`S_PetMenuPacket(L1NpcInstance npc, int exppercet)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 呼叫 `buildpacket(npc, exppercet)` |

`void buildpacket(L1NpcInstance npc, int exppercet)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 `C(S_OPCODE_SHOWHTML)` |
| `if (npc instanceof L1PetInstance pet)` | L30 `D(pet.getId())` → L31 `S("anicom")` → L32 `C(0x00)` → L33 `H(0x0a)` |
| `if (npc instanceof L1PetInstance pet) / switch (pet.getCurrentPetStatus()) / case 1 ->` | L35 `S("$469")` |
| `if (npc instanceof L1PetInstance pet) / switch (pet.getCurrentPetStatus()) / case 2 ->` | L36 `S("$470")` |
| `if (npc instanceof L1PetInstance pet) / switch (pet.getCurrentPetStatus()) / case 3, 5 ->` | L37 `S("$471")` |
| `if (npc instanceof L1PetInstance pet) / switch (pet.getCurrentPetStatus()) / default ->` | L38 `S("$471")` |
| `if (npc instanceof L1PetInstance pet)` | L40 `S(Integer.toString(pet.getCurrentHp()))` → L41 `S(Integer.toString(pet.getMaxHp()))` → L42 `S(Integer.toString(pet.getCurrentMp()))` → L43 `S(Integer.toString(pet.getMaxMp()))` → L44 `S(Integer.toString(pet.getLevel()))` → L45 `S(pet.getName())` → L46 `S("$611")` → L47 `S(Integer.toString(exppercet))` → L48 `S(Integer.toString(pet.getLawful()))` |
| `else if (npc instanceof L1SummonInstance summon)` | L52 `D(summon.getId())` → L53 `S("moncom")` → L54 `C(0x00)` → L55 `H(0x06)` |
| `else if (npc instanceof L1SummonInstance summon) / switch (summon.get_currentPetStatus()) / case 1 ->` | L57 `S("$469")` |
| `else if (npc instanceof L1SummonInstance summon) / switch (summon.get_currentPetStatus()) / case 2 ->` | L58 `S("$470")` |
| `else if (npc instanceof L1SummonInstance summon) / switch (summon.get_currentPetStatus()) / case 3 ->` | L59 `S("$471")` |
| `else if (npc instanceof L1SummonInstance summon) / switch (summon.get_currentPetStatus()) / case 5 ->` | L60 `S("$472")` |
| `else if (npc instanceof L1SummonInstance summon) / switch (summon.get_currentPetStatus()) / default ->` | L61 `S("$471")` |
| `else if (npc instanceof L1SummonInstance summon)` | L63 `S(Integer.toString(summon.getCurrentHp()))` → L64 `S(Integer.toString(summon.getMaxHp()))` → L65 `S(Integer.toString(summon.getCurrentMp()))` → L66 `S(Integer.toString(summon.getMaxMp()))` → L67 `S(Integer.toString(summon.getLevel()))` |

### S_PinkName

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PinkName.java`；繼承 `ServerBasePacket`。

`S_PinkName(int objecId, int time)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 `C(S_OPCODE_PINKNAME)` → L20 `D(objecId)` → L21 `C(time)` |

### S_Pledge

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Pledge.java`；繼承 `ServerBasePacket`。

`S_Pledge(int objid, String clanName, StringBuilder olmembers, StringBuilder allmembers)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 呼叫 `buildPacket(objid, clanName, olmembers, allmembers)` |

`void buildPacket(int objid, String clanname, StringBuilder olmembers, StringBuilder allmembers)`（L41）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L43 `C(S_OPCODE_SHOWHTML)` → L44 `D(objid)` → L45 `S("pledgeM")` → L46 `H(0X02)` → L47 `H(0x03)` → L48 `S(clanname)` → L49 `S(olmembers.toString())` → L50 `S(allmembers.toString())` |

`S_Pledge(int objid, String clanName, StringBuilder olmembers)`（L61）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L62 呼叫 `buildPacket(objid, clanName, olmembers)` |

`void buildPacket(int objid, String clanname, StringBuilder olmembers)`（L66）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L68 `C(S_OPCODE_SHOWHTML)` → L69 `D(objid)` → L70 `S("pledge")` → L71 `H(0X01)` → L72 `H(0x02)` → L73 `S(clanname)` → L74 `S(olmembers.toString())` |

`S_Pledge(int ClanId)`（L77）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L80 `C(S_PacketBox.HTML_PLEDGE_ANNOUNCE)` → L81 `S(clan.getClanName())` → L82 `S(clan.getLeaderName())` → L83 `D(clan.getEmblemId())` → L84 `D((int) (clan.get_clanBirthday().getTime() / 1000L))` → L93 `Byte1(text)` |

`S_Pledge(String name, String notes)`（L99）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L100 `C(S_OPCODE_PACKETBOX)` → L101 `C(S_PacketBox.HTML_PLEDGE_WRITE_NOTES)` → L102 `S(name)` → L115 `Byte1(text)` |

`S_Pledge(L1Clan clan)`（L118）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L120 `C(S_PacketBox.HTML_PLEDGE_MEMBERS)` → L121 `H(1)` → L122 `C(clan.getAllMembersSize())` |
| `if (clan.getAllMembersSize() > 0) / for (String name : membername)` | L132 `S(pc.getName())` → L133 `C(pc.getClanRank())` → L134 `C((short)pc.getLevel())` → L149 `Byte1(text)` → L150 `D(pc.getId())` → L151 `C(pc.getType())` |

### S_PledgeWarehouseHistory

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PledgeWarehouseHistory.java`；繼承 `ServerBasePacket`。

`S_PledgeWarehouseHistory(int clanId)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L39 `C(S_OPCODE_PACKETBOX)` → L40 `C(S_PacketBox.HTML_CLAN_WARHOUSE_RECORD)` → L41 `D(history.size())` |
| `for (final L1ClanWarehouseHistory record : history)` | L45 `S(safe(record.get_char_name()))` → L46 `C(record.get_type())` → L47 `S(item != null ? item.getName() : itemName)` → L48 `D(record.get_item_count())` → L49 `D((int) ((System.currentTimeMillis() - record.get_record_time().getTime()) / 60000L))` |

### S_Poison

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Poison.java`；繼承 `ServerBasePacket`。

`S_Poison(int objId, int type)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_POISON)` → L23 `D(objId)` |
| `switch (type) / case 0:` | L26 `C(0x00)` → L27 `C(0x00)` |
| `switch (type) / case 1:` | L31 `C(0x01)` → L32 `C(0x00)` |
| `switch (type) / case 2:` | L36 `C(0x00)` → L37 `C(0x01)` |

### S_PowerItemList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PowerItemList.java`；繼承 `ServerBasePacket`。

`S_PowerItemList(L1PcInstance pc, int objid, List<L1ItemInstance> items)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_SHOWRETRIEVELIST)` → L25 `D(objid)` → L26 `H(items.size())` → L27 `C(0x0a)` |
| `for (final L1ItemInstance item : items)` | L30 `D(itemobjid)` → L31 `C(0x00)` → L32 `H(item.get_gfxid())` → L33 `C(L1Item.toPacketBless(item.getBless()))` → L34 `D(0x01)` → L35 `C(item.isIdentified() ? 0x01 : 0x00)` → L36 `S(item.getViewName())` |

`S_PowerItemList(int objid, List<L1ItemInstance> items)`（L42）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L43 `C(S_OPCODE_SHOWRETRIEVELIST)` → L44 `D(objid)` → L45 `H(items.size())` → L46 `C(0x0c)` |
| `for (final L1ItemInstance item : items)` | L49 `D(itemobjid)` → L50 `C(0x00)` → L51 `H(item.get_gfxid())` → L52 `C(L1Item.toPacketBless(item.getBless()))` → L53 `D(0x01)` → L54 `C(item.isIdentified() ? 0x01 : 0x00)` → L55 `S(item.getViewName())` |

### S_PrivateShop

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_PrivateShop.java`；繼承 `ServerBasePacket`。

`S_PrivateShop(L1PcInstance pc, int objectId, int type)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (shopObj instanceof L1PcInstance)` | L27 呼叫 `isPc(pc, objectId, type)` |
| `else if (shopObj instanceof L1DeInstance)` | L29 呼叫 `isDe(pc, objectId, type)` |
| `else` | L32 `C(S_OPCODE_PRIVATESHOPLIST)` → L33 `C(type)` → L34 `D(objectId)` → L35 `H(0x0000)` |

`void isDe(L1PcInstance pc, int objectId, int type)`（L46）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L52 `C(S_OPCODE_PRIVATESHOPLIST)` → L53 `C(type)` → L54 `D(objectId)` |
| `if (type == 0) / if (map == null \|\| map.isEmpty())` | L61 `H(0x0000)` |
| `if (type == 0) / if (map.size() <= 0)` | L66 `H(0x0000)` |
| `if (type == 0)` | L70 `H(map.size())` |
| `if (type == 0) / for (L1ItemInstance item : map.keySet())` | L74 `C(i)` → L75 `C(L1Item.toPacketBless(item.getBless()))` → L76 `H(item.getItem().getGfxId())` → L77 `D((int) item.getCount())` → L79 `D(price)` → L83 `S(name)` → L84 `C(0)` |
| `else if (type == 1) / if (list == null \|\| list.isEmpty())` | L96 `H(0x0000)` |
| `else if (type == 1) / if (list.size() <= 0)` | L101 `H(0x0000)` |
| `else if (type == 1)` | L105 `H(list.size())` |
| `else if (type == 1) / for (int key : list.keySet()) / for (final L1ItemInstance pcItem : pc.getInventory().getItems()) / if (pcItem.getItemId() == key && pcItem.getEnchantLevel() == level)` | L116 `C(i)` → L117 `D(pcItem.getId())` → L118 `D(count)` → L119 `D(price)` |

`void isPc(L1PcInstance pc, int objectId, int type)`（L133）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L139 `C(S_OPCODE_PRIVATESHOPLIST)` → L140 `C(type)` → L141 `D(objectId)` |
| `if (type == 0) / if (list.isEmpty())` | L150 `H(0x0000)` |
| `if (type == 0) / if (size <= 0)` | L157 `H(0x0000)` |
| `if (type == 0)` | L163 `H(size)` |
| `if (type == 0) / for (int i = 0; i < size; i++) / if (item != null)` | L171 `C(i)` → L172 `C(L1Item.toPacketBless(item.getBless()))` → L173 `H(item.getItem().getGfxId())` → L174 `D(count)` → L175 `D(price)` → L179 `S(name)` → L180 `C(0x00)` |
| `else if (type == 1) / if (list.isEmpty())` | L202 `H(0x0000)` |
| `else if (type == 1) / if (size <= 0)` | L209 `H(0x0000)` |
| `else if (type == 1)` | L213 `H(size)` |
| `else if (type == 1) / for (int i = 0; i < size; i++) / for (final L1ItemInstance pcItem : pc.getInventory().getItems()) / if ((item.getItemId() == pcItem.getItemId()) && (item.getEnchantLevel() == pcItem.getEnchantLevel()))` | L223 `C(i)` → L224 `D(pcItem.getId())` → L225 `D(count)` → L226 `D(price)` |

### S_RangeSkill

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_RangeSkill.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`TYPE_NODIR=0`、`TYPE_DIR=8`、`DAMAGE_EXTENSION_MAGIC=0xD14A6E3F`。

`S_RangeSkill(L1Character cha, ArrayList<TargetStatus> targetList, int spellgfx, int actionId, int type)`（L40）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L43 呼叫 `this(cha, targetList, spellgfx, actionId, type, false)` |

`S_RangeSkill(L1Character cha, ArrayList<TargetStatus> targetList, int spellgfx, int actionId, int type, boolean includeDamage)`（L52）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L60 `C(S_OPCODE_RANGESKILLS)` → L61 `C(actionId)` → L63 `D(cha.getId())` → L64 `H(cha.getX())` → L65 `H(cha.getY())` |
| `switch (type) / case TYPE_NODIR:` | L69 `C(cha.getHeading())` |
| `switch (type) / case TYPE_DIR:` | L81 `C(cha.getHeading())` |
| 本方法此處 | L85 `D(_sequentialNumber.incrementAndGet())` → L86 `H(spellgfx)` → L87 `C(type)` → L88 `H(0x0000)` → L89 `H(targetList.size())` |
| `for (TargetStatus target : targetList)` | L95 `D(resolvedId)` |
| `for (TargetStatus target : targetList) / if (target.isCalc())` | L97 `H(0x20)` |
| `for (TargetStatus target : targetList) / else` | L100 `H(0x00)` |
| `for (TargetStatus target : targetList) / if (includeDamage && ConfigClient.isROClient() && resolvedId != 0)` | L103 `D(DAMAGE_EXTENSION_MAGIC)` → L104 `D(Math.max(0, target.getDamage()))` |

### S_RedMessage

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_RedMessage.java`；繼承 `ServerBasePacket`。

`S_RedMessage(String acc, String msg1)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket(acc, new String[]{ msg1 })` |

`void buildPacket(String acc, String[] info)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `S(acc)` |
| `if (info == null)` | L26 `C(0x00)` |
| `else` | L29 `C(info.length)` |
| `else / for (int i = 0 ; i < info.length ; i++)` | L31 `S(info[i])` |

### S_RemoveObject

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_RemoveObject.java`；繼承 `ServerBasePacket`。

`S_RemoveObject(L1Object obj)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_REMOVE_OBJECT)` → L21 `D(obj.getId())` |

### S_Resurrection

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Resurrection.java`；繼承 `ServerBasePacket`。

`S_Resurrection(L1PcInstance target, L1Character use, int type)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_RESURRECTION)` → L23 `D(target.getId())` → L24 `C(type)` → L25 `D(use.getId())` → L26 `D(target.getClassId())` |

`S_Resurrection(L1Character target, L1Character use, int type)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 `C(S_OPCODE_RESURRECTION)` → L38 `D(target.getId())` → L39 `C(type)` → L40 `D(use.getId())` → L41 `D(target.getGfxId())` |

`S_Resurrection(L1PcInstance target, int opid, int type)`（L51）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L52 `C(opid)` → L53 `D(target.getId())` → L54 `C(type)` → L55 `D(target.getId())` → L56 `D(target.getClassId())` |

### S_RetrieveChaList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_RetrieveChaList.java`；繼承 `ServerBasePacket`。

`S_RetrieveChaList(int objid, L1PcInstance pc)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_SHOWRETRIEVELIST)` → L26 `D(objid)` → L27 `H(size)` → L28 `C(0x12)` |
| `if (inventoryHasRoom) / for (final Object itemObject : pc.getDwarfForChaInventory().getItems())` | L32 `D(item.getId())` → L37 `C(i)` → L38 `H(item.get_gfxid())` → L39 `C(L1Item.toPacketBless(item.getBless()))` → L40 `D((int) Math.min(item.getCount(), 2000000000))` → L41 `C(item.isIdentified() ? 0x01 : 0x00)` → L42 `S(item.getViewName())` |

### S_RetrieveDevilKingList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_RetrieveDevilKingList.java`；繼承 `ServerBasePacket`。

`S_RetrieveDevilKingList(L1PcInstance pc, int objid, List<L1ItemInstance> items)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_SHOWRETRIEVELIST)` → L25 `D(objid)` → L26 `H(items.size())` → L27 `C(0x0c)` |
| `for (final L1ItemInstance item : items)` | L30 `D(itemobjid)` → L32 `C(0x00)` → L33 `H(item.get_gfxid())` → L34 `C(L1Item.toPacketBless(item.getBless()))` → L35 `D(1)` → L36 `C(item.isIdentified() ? 0x01 : 0x00)` → L37 `S(item.getViewName())` |

### S_RetrieveElfList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_RetrieveElfList.java`；繼承 `ServerBasePacket`。

`S_RetrieveElfList(int objid, L1PcInstance pc)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_SHOWRETRIEVELIST)` → L26 `D(objid)` → L27 `H(size)` → L28 `C(0x09)` |
| `if (inventoryHasRoom) / for (final Object itemObject : pc.getDwarfForElfInventory().getItems())` | L32 `D(item.getId())` → L37 `C(i)` → L38 `H(item.get_gfxid())` → L39 `C(L1Item.toPacketBless(item.getBless()))` → L40 `D((int) Math.min(item.getCount(), 2000000000))` → L41 `C(item.isIdentified() ? 0x01 : 0x00)` → L42 `S(item.getViewName())` |

### S_RetrieveList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_RetrieveList.java`；繼承 `ServerBasePacket`。

`S_RetrieveList(int objid, L1PcInstance pc)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_SHOWRETRIEVELIST)` → L27 `D(objid)` → L28 `H(size)` → L29 `C(0x03)` |
| `if (inventoryHasRoom) / for (final Object itemObject : pc.getDwarfInventory().getItems())` | L34 `D(item.getId())` → L39 `C(i)` → L40 `H(item.get_gfxid())` → L41 `C(L1Item.toPacketBless(item.getBless()))` → L42 `D((int) Math.min(item.getCount(), 2000000000))` → L43 `C(item.isIdentified() ? 0x01 : 0x00)` → L44 `S(item.getViewName())` |
| 本方法此處 | L50 `D(30)` |

### S_RetrievePledgeList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_RetrievePledgeList.java`；繼承 `ServerBasePacket`。

`S_RetrievePledgeList(int objid, L1PcInstance pc)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L28 `C(S_OPCODE_SHOWRETRIEVELIST)` → L29 `D(objid)` |
| `if (clan == null \|\| !inventoryHasRoom)` | L32 `H(0x0000)` → L33 `C(0x05)` → L34 `D(30)` |
| `if (inventoryHasRoom) / if (size > 0) / if (!clan.tryClaimWarehouse(pc.getId()))` | L48 `H(0x0000)` → L49 `C(0x05)` → L50 `D(30)` |
| `if (inventoryHasRoom) / if (size > 0)` | L53 `H(size)` → L54 `C(0x05)` |
| `if (inventoryHasRoom) / if (size > 0) / for (final Object itemObject : clan.getDwarfForClanInventory().getItems())` | L57 `D(item.getId())` → L62 `C(i)` → L63 `H(item.get_gfxid())` → L64 `C(L1Item.toPacketBless(item.getBless()))` → L65 `D((int) Math.min(item.getCount(), 2000000000))` → L66 `C(item.isIdentified() ? 0x01 : 0x00)` → L67 `S(item.getViewName())` |
| `if (inventoryHasRoom) / if (size > 0)` | L70 `D(30)` |
| `if (inventoryHasRoom) / else` | L72 `H(0x0000)` → L73 `C(0x05)` → L74 `D(30)` |

### S_RetrievePowerItem

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_RetrievePowerItem.java`；繼承 `ServerBasePacket`。

`S_RetrievePowerItem(L1PcInstance pc, int objid, List<L1ItemInstance> items)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_SHOWRETRIEVELIST)` → L25 `D(objid)` → L26 `H(items.size())` → L27 `C(0x0a)` |
| `for (final L1ItemInstance item : items)` | L30 `D(itemobjid)` → L32 `C(0x00)` → L33 `H(item.get_gfxid())` → L34 `C(L1Item.toPacketBless(item.getBless()))` → L36 `D(0x01)` → L37 `C(item.isIdentified() ? 0x01 : 0x00)` → L38 `S(item.getViewName())` |

### S_RetrieveShiftingList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_RetrieveShiftingList.java`；繼承 `ServerBasePacket`。

`S_RetrieveShiftingList(L1PcInstance pc, List<L1ItemInstance> items)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_SHOWRETRIEVELIST)` → L25 `D(pc.getId())` → L26 `H(items.size())` → L27 `C(0x02)` |
| `for (final L1ItemInstance item : items)` | L30 `D(itemid)` → L32 `C(0x00)` → L33 `H(item.get_gfxid())` → L34 `C(L1Item.toPacketBless(item.getBless()))` → L35 `D(1)` → L36 `C(item.isIdentified() ? 0x01 : 0x00)` → L37 `S(item.getViewName())` |

### S_SelectTarget

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SelectTarget.java`；繼承 `ServerBasePacket`。

`S_SelectTarget(int ObjectId)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_SELECTTARGET)` → L18 `D(ObjectId)` → L19 `C(0x00)` → L20 `C(0x00)` → L21 `C(0x02)` |

### S_SellHouse

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SellHouse.java`；繼承 `ServerBasePacket`。

`S_SellHouse(int objectId, String houseNumber)`（L26）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L27 呼叫 `buildPacket(objectId, houseNumber)` |

`S_SellHouse(int objectId,String houseNumber,Object house)`（L31）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L32 呼叫 `this(objectId,houseNumber)` |

`void buildPacket(int objectId, String houseNumber)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `C(S_OPCODE_INPUTAMOUNT)` → L39 `D(objectId)` → L40 `D(0)` → L41 `D(100000)` → L42 `D(100000)` → L43 `D(2000000000)` → L44 `H(0)` → L45 `S("agsell")` → L46 `S("agsell " + houseNumber)` |

### S_ServerMessage

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ServerMessage.java`；繼承 `ServerBasePacket`。

`S_ServerMessage(String name)`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_NPCSHOUT)` → L27 `C(0x00)` → L28 `D(0x00000000)` → L29 `S(name)` |

`S_ServerMessage(int type)`（L37）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `C(S_OPCODE_SERVERMSG)` → L39 `H(type)` → L40 `C(0x00)` |

`S_ServerMessage(int type, String msg1)`（L49）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L50 呼叫 `buildPacket(type, new String[]{ msg1 })` |

`S_ServerMessage(int type, String msg1, String msg2)`（L60）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L61 呼叫 `buildPacket(type, new String[]{ msg1, msg2 })` |

`S_ServerMessage(int type, String msg1, String msg2, String msg3)`（L72）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L73 呼叫 `buildPacket(type, new String[]{ msg1, msg2, msg3 })` |

`S_ServerMessage(int type, String[] info)`（L107）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L108 呼叫 `buildPacket(type, info)` |

`void buildPacket(int type, String[] info)`（L112）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L113 `C(S_OPCODE_SERVERMSG)` → L114 `H(type)` |
| `if (info == null)` | L117 `C(0x00)` |
| `else` | L120 `C(info.length)` |
| `else / for (int i = 0 ; i < info.length ; i++)` | L122 `S(info[i])` |

### S_ServerVersion

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ServerVersion.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`SERVER_NO=0x01`。

`S_ServerVersion()`（L14）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L15 `C(S_OPCODE_SERVERVERSION)` → L16 `C(0)` → L17 `C(SERVER_NO)` → L18 `D(0x07cbf4dd)` → L19 `D(0x07cbf4dd)` → L20 `D(0x77fc692d)` → L21 `D(0x07cbf4d9)` → L23 `D(0)` → L25 `C(0)` → L26 `C(0)` → L27 `C(CLIENT_LANGUAGE)` → L28 `D(0x77d82)` → L29 `D(UPTIME)` → L30 `H(1)` → L31 `C(0)` |

### S_ShopBuyList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopBuyList.java`；繼承 `ServerBasePacket`。

`S_ShopBuyList(int objid, L1PcInstance pc)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `C(S_OPCODE_SHOWSHOPSELLLIST)` → L24 `D(objid)` |
| `if (!(object instanceof L1NpcInstance npc))` | L28 `H(0x0000)` |
| `if (shop == null)` | L35 `H(0x0000)` |
| `if (assessedItems.isEmpty())` | L44 `H(0x0000)` |
| 本方法此處 | L48 `H(assessedItems.size())` |
| `for (final L1AssessedItem item : assessedItems)` | L51 `D(item.getTargetId())` → L52 `D(item.getAssessedPrice())` |

### S_ShopBuyListAll

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopBuyListAll.java`；繼承 `ServerBasePacket`。

`S_ShopBuyListAll(L1PcInstance pc, L1NpcInstance npc)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `C(S_OPCODE_SHOWSHOPSELLLIST)` → L24 `D(npc.getId())` |
| `if (assessedItems.isEmpty())` | L29 `H(0x0000)` |
| 本方法此處 | L33 `H(assessedItems.size())` |
| `for (final L1ItemInstance key : assessedItems.keySet())` | L36 `D(key.getId())` → L37 `D(assessedItems.get(key))` |

### S_ShopBuyListGam

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopBuyListGam.java`；繼承 `ServerBasePacket`。

`S_ShopBuyListGam(L1PcInstance pc, L1NpcInstance npc, Map<Integer, L1Gambling> sellList)`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_SHOWSHOPSELLLIST)` → L27 `D(npc.getId())` |
| `if (sellList.isEmpty())` | L30 `H(0x0000)` |
| `if (sellList.size() <= 0)` | L35 `H(0x0000)` |
| 本方法此處 | L39 `H(sellList.size())` |
| `for (final Integer itemobjid : sellList.keySet())` | L42 `D(itemobjid)` → L45 `D(adena)` |

### S_ShopBuyListLotto

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopBuyListLotto.java`；繼承 `ServerBasePacket`。

`S_ShopBuyListLotto(L1PcInstance pc, L1NpcInstance npc, Map<Integer, Long> sellList)`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L16 `C(S_OPCODE_SHOWSHOPSELLLIST)` → L17 `D(npc.getId())` |
| `if (sellList == null \|\| sellList.isEmpty())` | L19 `H(0x0000)` |
| 本方法此處 | L22 `H(sellList.size())` |
| `for (final Integer itemobjid : sellList.keySet())` | L24 `D(itemobjid)` → L27 `D(pack)` |

### S_ShopSellList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopSellList.java`；繼承 `ServerBasePacket`。

`S_ShopSellList(int objId)`（L32）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L33 呼叫 `this(objId, null)` |

`S_ShopSellList(int objId, L1PcInstance pc)`（L42）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L44 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L45 `D(objId)` |
| `if (!(npcObj instanceof L1NpcInstance npcInstance))` | L49 `H(0x0000)` |
| `if (shopItems.size() <= 0)` | L59 `H(0x0000)` |
| 本方法此處 | L63 `H(shopItems.size())` |
| `for (int i = 0; i < shopItems.size(); i++)` | L69 `D(i)` → L71 `H(shopItem.getItem().getGfxId())` → L73 `D(price)` → L91 `S(nameBuilder.toString())` |
| `for (int i = 0; i < shopItems.size(); i++) / if (ConfigOther.SHOPINFO)` | L98 `C(status.length)` |
| `for (int i = 0; i < shopItems.size(); i++) / if (ConfigOther.SHOPINFO) / for (final byte b : status)` | L100 `C(b)` |
| `for (int i = 0; i < shopItems.size(); i++) / else` | L105 `C(0x00)` |
| 本方法此處 | L108 `H(0x0007)` |

`S_ShopSellList(L1NpcInstance npc)`（L115）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L116 呼叫 `this(npc, null)` |

`S_ShopSellList(L1NpcInstance npc, L1PcInstance pc)`（L125）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L127 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L128 `D(npc.getId())` |
| `if (shopItems.size() <= 0)` | L136 `H(0x0000)` |
| 本方法此處 | L140 `H(shopItems.size())` |
| `for (int i = 0; i < shopItems.size(); i++)` | L146 `D(i)` → L147 `H(shopItem.getItem().getGfxId())` → L148 `D(price)` → L165 `S(nameBuilder.toString())` → L168 `C(0x00)` |
| 本方法此處 | L170 `H(0x0007)` |

### S_ShopSellListCn

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopSellListCn.java`；繼承 `ServerBasePacket`。

`S_ShopSellListCn(L1PcInstance pc, L1NpcInstance npc)`（L31）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L32 呼叫 `this(pc, npc, true)` |

`S_ShopSellListCn(L1PcInstance pc, L1NpcInstance npc, boolean showItemDetails)`（L42）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L43 呼叫 `buildPacket(pc, npc.getId(), npc.getNpcId(), showItemDetails)` |

`void buildPacket(L1PcInstance pc, int tgObjid, int npcid, boolean showItemDetails)`（L47）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L55 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L57 `D(tgObjid)` → L59 `H(shopItems.size())` |
| `for (final L1ShopItem shopItem : shopItems)` | L68 `D(i)` → L69 `H(item.getGfxId())` → L70 `D(shopItem.getPrice())` → L83 `S(nameBuilder.toString())` |
| `for (final L1ShopItem shopItem : shopItems) / if (showItemDetails)` | L89 `C(status.length)` |
| `for (final L1ShopItem shopItem : shopItems) / if (showItemDetails) / for (final byte b : status)` | L91 `C(b)` |
| `for (final L1ShopItem shopItem : shopItems) / else` | L95 `C(0x00)` |
| 本方法此處 | L99 `H(0x17d4)` |

### S_ShopSellListCnX

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopSellListCnX.java`；繼承 `ServerBasePacket`。

`S_ShopSellListCnX(L1PcInstance pc, int tgObjid)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 呼叫 `buildPacket(pc, tgObjid)` |

`void buildPacket(L1PcInstance pc, int tgObjid)`（L28）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L32 `D(tgObjid)` |
| `if (shopItems.size() <= 0)` | L37 `H(0x0000)` |
| 本方法此處 | L41 `H(shopItems.size())` |
| `for (final L1ShopItem shopItem : shopItems)` | L50 `D(i)` → L51 `H(item.getGfxId())` → L52 `D(shopItem.getPrice())` → L66 `S(nameBuilder.toString())` → L72 `C(status.length)` |
| `for (final L1ShopItem shopItem : shopItems) / for (final byte b : status)` | L74 `C(b)` |
| 本方法此處 | L80 `H(0x17d4)` |

### S_ShopSellListGam

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopSellListGam.java`；繼承 `ServerBasePacket`。

`S_ShopSellListGam(L1PcInstance pc, L1NpcInstance npc)`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L27 `D(npc.getId())` |
| `if (list.size() <= 0)` | L32 `H(0x0000)` |
| 本方法此處 | L36 `H(list.size())` |
| `for (final GamblingNpc gamblingNpc : list.values())` | L46 `D(i)` → L47 `H(item.getGfxId())` → L49 `D(GamblingManager.GAMADENA)` → L56 `S(stringBuilder.toString())` → L57 `C(0x00)` |
| 本方法此處 | L60 `H(0x0007)` |

### S_ShopSellListLotto

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopSellListLotto.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`ORDER_ID=1`。

`S_ShopSellListLotto(L1PcInstance pc, L1NpcInstance npc)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L20 `D(npc.getId())` → L21 `H(1)` → L28 `D(ORDER_ID)` → L29 `H(gfx)` → L30 `D(price)` → L36 `S(suf.isEmpty() ? name : name + suf)` → L37 `C(0x00)` → L38 `H(0x0007)` |

### S_ShopSellListTmp

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopSellListTmp.java`；繼承 `ServerBasePacket`。

`S_ShopSellListTmp(int objId, int tmp)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L27 `D(objId)` → L29 `H(size)` |
| `for (int i = 0; i < size; i++)` | L33 `D(i)` → L34 `H(gfx)` → L35 `D(0x00)` → L36 `S("gfxid = " + gfx)` → L38 `C(0x00)` |
| 本方法此處 | L40 `H(0x0007)` |

### S_ShopSellListX1

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShopSellListX1.java`；繼承 `ServerBasePacket`。

`S_ShopSellListX1(int objId)`（L27）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L30 `D(objId)` |
| `if (!(npcObj instanceof L1NpcInstance))` | L34 `H(0x0000)` |
| `if (shopItems.size() <= 0)` | L44 `H(0x0000)` |
| 本方法此處 | L48 `H(shopItems.size())` |
| `for (int i = 0; i < shopItems.size(); i++)` | L54 `H(i)` → L56 `H(shopItem.getItem().getGfxId())` → L58 `H(price)` |
| `for (int i = 0; i < shopItems.size(); i++) / if (shopItem.getPackCount() > 1)` | L61 `S(item.getNameId() + " (" + shopItem.getPackCount() + ")")` |
| `for (int i = 0; i < shopItems.size(); i++) / else` | L64 `S(item.getNameId())` |
| `for (int i = 0; i < shopItems.size(); i++)` | L68 `C(0x00)` |
| 本方法此處 | L83 `H(0x0000)` |

`S_ShopSellListX1(L1NpcInstance npc)`（L90）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L91 `C(S_OPCODE_SHOWSHOPBUYLIST)` → L92 `D(npc.getId())` |
| `if (shopItems.size() <= 0)` | L100 `H(0x0000)` |
| 本方法此處 | L104 `H(shopItems.size())` |
| `for (int i = 0; i < shopItems.size(); i++)` | L110 `D(i)` → L111 `H(shopItem.getItem().getGfxId())` → L112 `D(price)` |
| `for (int i = 0; i < shopItems.size(); i++) / if (shopItem.getPackCount() > 1)` | L114 `S(item.getNameId() + " (" + shopItem.getPackCount() + ")")` |
| `for (int i = 0; i < shopItems.size(); i++) / else` | L117 `S(item.getNameId())` |
| `for (int i = 0; i < shopItems.size(); i++)` | L121 `C(0x00)` |
| 本方法此處 | L136 `H(0x0000)` |

### S_ShowDrop

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShowDrop.java`；繼承 `ServerBasePacket`。

本類數值常數（含 subcode，仍須依方法判斷用途）：`SHOW_EXP=0`、`SHOW_ADENA=1`。

`S_ShowDrop(int type, int amount)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_PACKETBOX)` → L18 `C(192)` → L19 `C(type)` → L20 `D(amount)` |

### S_ShowPolyList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShowPolyList.java`；繼承 `ServerBasePacket`。

`S_ShowPolyList(int objid)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 `C(S_OPCODE_SHOWHTML)` → L20 `D(objid)` → L21 `S("monlist")` |

`S_ShowPolyList(L1Character target)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 `C(S_OPCODE_SHOWHTML)` → L31 `D(target.getId())` → L32 `S("monlist")` |

### S_ShowSummonList

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ShowSummonList.java`；繼承 `ServerBasePacket`。

`S_ShowSummonList(int objid)`（L12）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L13 `C(S_OPCODE_SHOWHTML)` → L14 `D(objid)` → L15 `S("summonlist")` |

### S_SkillBrave

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SkillBrave.java`；繼承 `ServerBasePacket`。

`S_SkillBrave(int objid, int mode, int time)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_SKILLBRAVE)` → L25 `D(objid)` → L26 `C(mode)` → L27 `H(time)` |

`S_SkillBrave(int objid, int mode)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `C(S_OPCODE_SKILLBRAVE)` → L39 `D(objid)` → L40 `C(mode)` → L41 `H(300)` |

### S_SkillBuy

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SkillBuy.java`；繼承 `ServerBasePacket`。

`S_SkillBuy(L1PcInstance pc, ArrayList<Integer> newSkillList)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 呼叫 `this(pc, null, newSkillList)` |

`S_SkillBuy(L1PcInstance pc, com.lineage.server.model.Instance.L1NpcInstance npc, ArrayList<Integer> newSkillList)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (newSkillList.size() <= 0)` | L38 `C(S_OPCODE_SKILLBUY)` → L39 `H(0x0000)` |
| `else` | L42 `C(S_OPCODE_SKILLBUY)` → L43 `D(6000)` → L44 `H(newSkillList.size())` |
| `else / for (final Integer integer : newSkillList)` | L46 `D(integer)` |

`S_SkillBuy(L1PcInstance pc,com.lineage.server.model.Instance.L1NpcInstance npc, ArrayList<Integer> skills,UiOperationContext.Ticket prerequisite)`（L56）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L58 呼叫 `this(pc,npc,skills)` |

### S_SkillBuyCN

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SkillBuyCN.java`；繼承 `ServerBasePacket`。

`S_SkillBuyCN(L1PcInstance pc, L1NpcInstance npc)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (newSkillList.size() <= 0)` | L75 `C(S_OPCODE_SHOWHTML)` → L76 `D(npc.getId())` → L77 `S("y_skill_02")` → L78 `H(0x00)` → L79 `H(0x00)` |
| `else` | L86 `C(S_OPCODE_SKILLBUY)` → L87 `D(startAdena)` → L88 `H(newSkillList.size())` |
| `else / for (final Integer integer : newSkillList)` | L90 `D(integer)` |

### S_SkillBuyItem

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SkillBuyItem.java`；繼承 `ServerBasePacket`。

`S_SkillBuyItem(L1PcInstance pc, L1NpcInstance npc)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (newSkillList.size() <= 0)` | L47 `C(S_OPCODE_SKILLBUYITEM)` → L48 `H(0x0000)` |
| `else` | L52 `C(S_OPCODE_SKILLBUYITEM)` → L53 `H(newSkillList.size())` |
| `else / for (final Integer integer : newSkillList)` | L55 `D(integer)` |

### S_SkillBuyItemCN

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SkillBuyItemCN.java`；繼承 `ServerBasePacket`。

`S_SkillBuyItemCN(L1PcInstance pc, L1NpcInstance npc)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (newSkillList.size() <= 0)` | L59 `C(S_OPCODE_SKILLBUYITEM)` → L60 `H(0x0000)` |
| `else` | L65 `C(S_OPCODE_SKILLBUYITEM)` → L66 `H(newSkillList.size())` |
| `else / for (final Integer integer : newSkillList)` | L68 `D(integer)` |

### S_SkillHaste

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SkillHaste.java`；繼承 `ServerBasePacket`。

`S_SkillHaste(int objid, int mode, int time)`（L21）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L23 `C(S_OPCODE_SKILLHASTE)` → L24 `D(objid)` → L25 `C(mode)` → L26 `H(time)` |

### S_SkillIconBlessOfEva

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SkillIconBlessOfEva.java`；繼承 `ServerBasePacket`。

`S_SkillIconBlessOfEva(int objectId, int time)`（L15）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L16 `C(S_OPCODE_BLESSOFEVA)` → L17 `D(objectId)` → L18 `H(time)` |

### S_SkillIconShield

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SkillIconShield.java`；繼承 `ServerBasePacket`。

`S_SkillIconShield(int type, int time)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 `C(S_OPCODE_SKILLICONSHIELD)` → L20 `H(time)` → L21 `C(type)` |

### S_SkillSound

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SkillSound.java`；繼承 `ServerBasePacket`。

`S_SkillSound(int objid, int gfxid)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket(objid, gfxid)` |

`void buildPacket(int objid, int gfxid)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_SKILLSOUNDGFX)` → L25 `D(objid)` → L26 `H(gfxid)` |

`S_SkillSound(int objid, int gfxid, int time)`（L35）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L37 `C(S_OPCODE_SKILLSOUNDGFX)` → L38 `D(objid)` → L39 `H(gfxid)` → L40 `H(time)` |

### S_Sound

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Sound.java`；繼承 `ServerBasePacket`。

`S_Sound(int sound)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 呼叫 `buildPacket(sound, 0)` |

`S_Sound(int sound, int repeat)`（L28）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L29 呼叫 `buildPacket(sound, repeat)` |

`void buildPacket(int sound, int repeat)`（L33）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L35 `C(S_OPCODE_SOUND)` → L36 `C(repeat)` → L37 `H(sound)` |

### S_SPMR

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SPMR.java`；繼承 `ServerBasePacket`。

`S_SPMR(L1PcInstance pc, boolean isLogin)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L18 `C(S_OPCODE_SPMR)` → L20 呼叫 `writeSp(pc.getTrueSp())` → L21 呼叫 `writeMr(pc.getBaseMr())` |

`S_SPMR(L1PcInstance pc)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 呼叫 `buildPacket(pc)` |

`void buildPacket(L1PcInstance pc)`（L34）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L35 `C(S_OPCODE_SPMR)` → L74 呼叫 `writeSp(sp)` → L75 呼叫 `writeMr(mr)` |

`void writeSp(int sp)`（L78）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (ConfigClient.isROClient())` | L80 `D(sp)` |
| `else` | L82 `C(sp)` |

`void writeMr(int mr)`（L257）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| `if (ConfigClient.isROClient())` | L259 `D(mr)` |
| `else` | L261 `C(mr)` |

`S_SPMR()`（L269）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L270 `C(S_OPCODE_SPMR)` → L271 `C(50)` → L272 `C(100)` |

### S_Strup

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Strup.java`；繼承 `ServerBasePacket`。

`S_Strup(L1PcInstance pc, int type, int time)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_STRUP)` → L23 `H(time)` → L24 `C(pc.getStr())` → L25 `C(pc.getInventory().getWeight240())` → L26 `C(type)` |

### S_SystemMessage

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_SystemMessage.java`；繼承 `ServerBasePacket`。

`S_SystemMessage(String msg)`（L14）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L15 `C(S_OPCODE_GLOBALCHAT)` → L16 `C(0x09)` → L17 `S(msg)` |

`S_SystemMessage(String msg, boolean nameid)`（L29）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L30 `C(S_OPCODE_NPCSHOUT)` → L31 `C(2)` → L32 `D(0)` → L33 `S(msg)` |

### S_TaxRate

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_TaxRate.java`；繼承 `ServerBasePacket`。

`S_TaxRate(int objecId)`（L13）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L14 呼叫 `this(objecId, null, null)` |

`S_TaxRate(int objecId, Offer offer, Ticket prerequisite)`（L17）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_TAXRATE)` → L21 `D(objecId)` → L22 `C(0x0a)` → L23 `C(0x32)` |

### S_Teleport

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Teleport.java`；繼承 `ServerBasePacket`。

`S_Teleport()`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L22 `C(S_OPCODE_TELEPORT)` → L24 `C(0xeb)` → L25 `C(0xa7)` → L27 `D(_nextId.incrementAndGet())` |

### S_Teleport2

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Teleport2.java`；繼承 `ServerBasePacket`。

`S_Teleport2(int mapid, int id, L1PcInstance owner, long generation, long connection)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `this(mapid, id)` |

`S_Teleport2(int mapid, int id)`（L36）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `C(S_OPCODE_TELEPORT)` → L39 `H(mapid)` → L40 `D(id)` |

### S_ToGmMessage

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_ToGmMessage.java`；繼承 `ServerBasePacket`。

`S_ToGmMessage(String info)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_NPCSHOUT)` → L18 `C(0x00)` → L19 `D(0x00000000)` → L20 `S("\\fY" + info)` |

### S_Trade

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Trade.java`；繼承 `ServerBasePacket`。

`S_Trade(String name)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_TRADE)` → L18 `S(name)` |

### S_TradeAddItem

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_TradeAddItem.java`；繼承 `ServerBasePacket`。

`S_TradeAddItem(L1ItemInstance item, long count, int type)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 `C(S_OPCODE_TRADEADDITEM)` → L22 `C(type)` → L23 `H(item.getItem().getGfxId())` → L27 `S(name)` → L36 `C(L1Item.toPacketBless(item.getBless()))` |

`S_TradeAddItem()`（L43）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L44 `C(S_OPCODE_TRADEADDITEM)` → L45 `C(0x01)` → L46 `H(714)` → L47 `S("測試物品(55)")` → L49 `C(0x00)` |

### S_TradeStatus

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_TradeStatus.java`；繼承 `ServerBasePacket`。

`S_TradeStatus(int type)`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L17 `C(S_OPCODE_TRADESTATUS)` → L18 `C(type)` |

### S_Trap

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Trap.java`；繼承 `ServerBasePacket`。

`S_Trap(L1TrapInstance trap, String name)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 `C(S_OPCODE_CHARPACK)` → L21 `H(trap.getX())` → L22 `H(trap.getY())` → L23 `D(trap.getId())` → L24 `H(0x0007)` → L25 `C(0x00)` → L26 `C(0x00)` → L27 `C(0x00)` → L28 `C(0x00)` → L29 `D(0x00000000)` → L30 `H(0x0000)` → L31 `S(name)` → L32 `S(null)` → L33 `C(0x00)` → L34 `D(0x00000000)` → L35 `S(null)` → L36 `S(null)` → L37 `C(0x00)` → L38 `C(0xFF)` → L39 `C(0x00)` → L40 `C(0x00)` → L41 `C(0x00)` → L42 `C(0xFF)` → L43 `C(0xFF)` |

### S_TrueTarget

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_TrueTarget.java`；繼承 `ServerBasePacket`。

`S_TrueTarget(int targetId, int objectId, String message)`（L18）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L19 呼叫 `buildPacket(targetId, objectId, message)` |

`void buildPacket(int targetId, int objectId, String message)`（L23）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L24 `C(S_OPCODE_TRUETARGET)` → L25 `D(targetId)` → L26 `D(objectId)` → L27 `S(message)` |

`S_TrueTarget(int targetId, int gfxid)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L31 `C(S_OPCODE_TRUETARGET)` → L32 `D(targetId)` → L33 `D(targetId)` → L34 `S(null)` → L35 `H(gfxid)` |

### S_Unknown_B

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Unknown_B.java`；繼承 `ServerBasePacket`。

`S_Unknown_B()`（L16）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L21 `C(S_OPCODE_CHARRESET)` → L23 `C(0x0a)` → L24 `C(0x02)` → L25 `C(0x00)` → L26 `C(0x00)` → L27 `C(0x00)` → L28 `C(0x2b)` → L29 `C(0x7f)` |

### S_UseArrowSkill

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_UseArrowSkill.java`；繼承 `ServerBasePacket`。

`S_UseArrowSkill(L1Character cha, int targetobj, int spellgfx, int x, int y, int dmg)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L52 `C(S_OPCODE_ATTACKPACKET)` → L53 `C(aid)` → L54 `D(cha.getId())` → L55 `D(HitReactionSuppressTable.get().resolveTargetObjId(targetobj))` → L58 `H[依 ClientLogin](this, dmg)` → L60 `C(cha.getHeading())` → L63 `D(0x00000152)` → L65 `H(spellgfx)` → L66 `C(0x7f)` → L67 `H(cha.getX())` → L68 `H(cha.getY())` → L69 `H(x)` → L70 `H(y)` → L72 `D(0x00000000)` → L73 `C(0x00)` |

`S_UseArrowSkill(L1Character cha, int targetobj, int spellgfx, int x, int y, int dmg, int actionId)`（L77）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L84 `C(S_OPCODE_ATTACKPACKET)` → L85 `C(actionId)` → L86 `D(cha.getId())` → L87 `D(HitReactionSuppressTable.get().resolveTargetObjId(targetobj))` → L90 `H[依 ClientLogin](this, dmg)` → L92 `C(cha.getHeading())` → L93 `D(0x00000152)` → L95 `H(spellgfx)` → L96 `C(0x7f)` → L97 `H(cha.getX())` → L98 `H(cha.getY())` → L99 `H(x)` → L100 `H(y)` → L102 `D(0x00000000)` → L103 `C(0x00)` |

`S_UseArrowSkill(L1Character cha, int spellgfx, int x, int y )`（L115）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L126 `C(S_OPCODE_ATTACKPACKET)` → L127 `C(aid)` → L128 `D(cha.getId())` → L129 `D(0x00)` → L130 `H(0x00)` → L131 `C(cha.getHeading())` → L134 `D(0x00000152)` → L136 `H(spellgfx)` → L137 `C(0x7f)` → L138 `H(cha.getX())` → L139 `H(cha.getY())` → L140 `H(x)` → L141 `H(y)` → L143 `D(0x00000000)` → L144 `C(0x00)` |

### S_UseAttackSkill

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_UseAttackSkill.java`；繼承 `ServerBasePacket`。

`S_UseAttackSkill(L1Character cha, int targetobj, int spellgfx, int x, int y, int actionId, boolean motion)`（L33）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L35 呼叫 `buildPacket(cha, targetobj, spellgfx, x, y, actionId, 0, motion)` |

`S_UseAttackSkill(L1Character cha, int targetobj, int spellgfx, int x, int y, int actionId, int dmg)`（L49）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L53 呼叫 `buildPacket(cha, targetobj, spellgfx, x, y, resolvedActionId, dmg, true)` |

`void buildPacket(L1Character cha, int targetobj, int spellgfx, int x, int y, int actionId, int dmg, boolean withCastMotion)`（L68）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L111 `C(S_OPCODE_ATTACKPACKET)` → L112 `C(actionId)` → L113 `D(withCastMotion ? cha.getId() : 0x00000000)` → L114 `D(HitReactionSuppressTable.get().resolveTargetObjId(targetobj))` |
| `if (com.lineage.config.ConfigClient.isROClient())` | L120 `H(dmg)` |
| `else` | L122 `H(dmg > 0 ? 0x000a : 0x0000)` |
| 本方法此處 | L125 `C(newheading)` → L128 `D(_sequentialNumber.incrementAndGet())` → L130 `H(spellgfx)` → L131 `C(0x00)` → L132 `H(cha.getX())` → L133 `H(cha.getY())` → L134 `H(x)` → L135 `H(y)` → L137 `D(0x00000000)` → L138 `C(0x00)` |

### S_War

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_War.java`；繼承 `ServerBasePacket`。

`S_War(int type, String clan_name1, String clan_name2)`（L25）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L26 呼叫 `buildPacket(type, clan_name1, clan_name2)` |

`void buildPacket(int type, String clan_name1, String clan_name2)`（L30）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L39 `C(S_OPCODE_WAR)` → L40 `C(type)` → L41 `S(clan_name1)` → L42 `S(clan_name2)` |

### S_WarTime

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_WarTime.java`；繼承 `ServerBasePacket`。

`S_WarTime(Calendar cal)`（L22）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L38 `H(0x0006)` → L39 `S(Config.TIME_ZONE)` → L40 `C(0x00)` → L41 `C(0x00)` → L42 `C(0x00)` → L43 `D(time)` → L44 `C(0x00)` → L45 `D(time - 1)` → L46 `C(0x00)` → L47 `D(time - 2)` → L48 `C(0x00)` → L49 `D(time - 3)` → L50 `C(0x00)` → L51 `D(time - 4)` → L52 `C(0x00)` → L53 `D(time - 5)` → L54 `C(0x00)` |

`S_WarTime(int op)`（L62）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L76 `C(op)` → L77 `H(6)` → L78 `S(Config.TIME_ZONE)` → L79 `C(0)` → L80 `C(0)` → L81 `C(0)` → L82 `D(time)` → L83 `C(0)` → L84 `D(time - 1)` → L85 `C(0)` → L86 `D(time - 2)` → L87 `C(0)` → L88 `D(time - 3)` → L89 `C(0)` → L90 `D(time - 4)` → L91 `C(0)` → L92 `D(time - 5)` → L93 `C(0)` |

### S_WeaponRange

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_WeaponRange.java`；繼承 `ServerBasePacket`。

`S_WeaponRange(int size)`（L12）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L13 `C(S_OPCODE_PACKETBOX)` → L14 `C(160)` → L15 `C(size == -1 ? 15 : size)` |

### S_Weather

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_Weather.java`；繼承 `ServerBasePacket`。

`S_Weather(int weather)`（L19）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L20 呼叫 `buildPacket(weather)` |

`void buildPacket(int weather)`（L24）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L25 `C(S_OPCODE_WEATHER)` → L26 `C(weather)` |

### S_WhoAmount

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_WhoAmount.java`；繼承 `ServerBasePacket`。

`S_WhoAmount(String amount)`（L7）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L8 `C(S_OPCODE_SERVERMSG)` → L9 `H(0x0051)` → L10 `C(0x01)` → L11 `S(amount)` |

### S_WhoCharinfo

來源：`1-伺服器/src/com/lineage/server/serverpackets/S_WhoCharinfo.java`；繼承 `ServerBasePacket`。

`S_WhoCharinfo(L1Character cha)`（L20）

| 分支／重複條件 | 欄位順序（型別寬度見 §2） |
|---|---|
| 本方法此處 | L33 `C(S_OPCODE_SERVERMSG)` → L34 `H(0x00a6)` → L35 `C(0x01)` → L62 `S(title + name + " " + lawfulness + " " + clan)` |

## 7. 來源辨識與更新

盤點日期：2026-11-06。這是本機來源快照，未核實運行中的 JAR 或正式 DB 與此來源完全一致。以下 SHA-256 可協助下一棒確認核心規格是否漂移；每次封包任務先查本表，再核對其來源方法。若變更涉及非本快照的 builder，應同步更新本文件。

| 相對來源 | SHA-256 |
|---|---|
| `1-伺服器/src/com/lineage/echo/OpcodesClient.java` | `3b9df4518cad4ab3021d7a6454f88fe4513bd715eb3eb461ffaa873a64fe8931` |
| `1-伺服器/src/com/lineage/server/serverpackets/OpcodesServer.java` | `9044cb98cf8a0548a421fbb05e6cdcef26147b2a2b85a1ef896fde8a8016f152` |
| `1-伺服器/src/com/lineage/echo/PacketHandler.java` | `001014e4f858abaef5734efe558d1759f97f85d4314d27bd44fefd72d4b3cd30` |
| `1-伺服器/src/com/lineage/server/clientpackets/ClientPacketStructure.java` | `ca88c05673188ec1b99a961be0d67561808f9bd42adf8a937b9ebda6767e96a9` |
| `1-伺服器/src/com/lineage/server/clientpackets/ClientBasePacket.java` | `83b48cc84680c6238d2491c71a776ec5beb542c39b6232f3a1b4e620aaf77636` |
| `1-伺服器/src/com/lineage/server/serverpackets/ServerBasePacket.java` | `3348c199e9694bd1eec050b130f28ab99590cae16f0047964c17b32df309a439` |
| `1-伺服器/src/com/lineage/echo/SessionPhaseGate.java` | `ea7e28fd276ba9b469222b3a655be279d729b69811f06978aaff74877b52c13a` |
| `1-伺服器/src/com/lineage/echo/SessionPacketPolicy.java` | `e6b33150986f64550a1f02b148c68fc27f5085b41179d2796aac833a80dce8b2` |
| `1-伺服器/src/com/lineage/netty/handler/L1HandshakeHandler.java` | `4bae3485738fa984599dc9115bbc75bf97f1589045cdc992a94aaeec25a3a961` |
| `1-伺服器/src/com/lineage/echo/encryptions/Encryption.java` | `ee1ad689385c1734e41f325681ca207e3be6424b7f88c4186fb6250660a9cb4d` |

346 個封包來源的集合指紋：`5dfe4247e8fd1e41ff7604b423e2aceff9df8ef128640e45671745543cb4d3a1`。計算法：各類別檔案的 `相對路徑<TAB>SHA256<LF>` 以字串順序排序後串接，再取 SHA-256；只供漂移偵測，不代表已執行測試。
