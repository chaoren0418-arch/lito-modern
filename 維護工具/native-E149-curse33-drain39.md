# #33 木乃伊的詛咒與 #39 魔力奪取候選來源

依唯讀 E1498871 正式 catalog、ClientSession 完整方法與 Protocol reader 接入口。尚未套用正常來源、選入 registry 或執行；metadata 編譯不是行為驗收。

|技能|C14 prefix／其後 payload|本機目標與節流|正式伺服器基值與權威|
|---|---|---|---|
|33 木乃伊的詛咒|14 04 00；actor u32 LE＋x/y u16 LE，共11 bytes|指定其他存活可見 actor、Chebyshev≤2；reuse／client cast clock 各5000ms|MP150、40318×2、lawful -500、PC警告5000ms／麻痺10000ms、機率與抵抗由伺服器決定；NPC採正式另一條duration路徑|
|39 魔力奪取|14 04 06；同上11 bytes|未選取時對自己；有選取就驗該 actor，不回退自己；Chebyshev≤4；reuse500ms|MP1、HP50、drain／gain／機率／目標資格全由伺服器決定；HP≤cost 的所選guard只記log返回，沒有專用拒絕封包|

兩技能正式 caster action19 沿既有 S_OBJECT_ACTION(0x20)；target effect 分別746／2171，沿既有 S_EFFECT_OBJECT(0x37)與incarnation／finite history。746原8frame／Sound116、2171原9frame／Sound806已在 finite bank；不另造圖或播放入口。影像不能推定詛咒成功／倒數，本機不造5秒／10秒控制，不預扣HP／MP／材料、不預判drain。S_PARALYSIS及狀態更新仍由伺服器封包驅動。Pending 的 CastAction 只表示收到動作，不表示命中；silent guard 沿既有兩秒 NoResponse、reuse gate另算，不造拒絕碼。

候選保留先前S23時序／音效latch、G06來源與完整目標owner／connection／generation／correction／incarnation／target_revision守衛；#39不得混入禁止自用的傷害群組。新增10個純來源行為案例，涵蓋封包、兩種self政策、對角邊界、低HP／MP不變、死亡／缺書／control、目標ABA／來源退役、action與effect非控制、silent guard與拒絕優先、壞尾／orphan；全部待恢復測試後執行。既有109案不重跑也不轉移成功。

下一步：繼續依正式180項目錄剩餘入口核對reader及實際分支；#23壞物術／#36迷魅術先確認目標領域、C14長度與正式材料／狀態，未核對前不填推測封包。使用者目前使用電腦，測試／--list／GUI／server／fixture／DB／音效播放暫緩。未來先四元件與G06，再當版完整library Debug／Release、Release回歸與完整GUI，全部自行驗證後才交人工。
