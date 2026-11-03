# E149 #51 召喚術來源候選

本輪只開發與編譯metadata，沒有執行測試、--list、GUI、PE、遊戲類別、DB或音效。未套用正常來源／選入registry。正式唯讀JAR維持E149。

- C14 `14 06 02`＋召喚selection u32LE，共七bytes；不是actor、NPC或背包物品ID。原Protocol讀d()並拒絕剩餘bytes。正式31個Map字面呼叫，經summon／summonSpecial兩helper及SummonChoice七欄constructor核對後展開；特殊三項adjustment7，因此有效容量49／49／51。原catalog的31個opaque helper不改寫歷史；本輪另存解析結果，不移記其他factory／getter的資格。
- WORLD／BATTLE皆經NativeSummonCastAction→NativeLightAction的self生命週期，以及nativeMaterialSummonBody；controlAbility5存在時看selection，否則由server directGrade(level)選擇。31名稱來自唯讀NPC SQL，只作基礎選項label，非當前DB模板／數值觀測。沒有以名稱猜裝備、控制能力、等級或容量。
- 實際Native Window新增召喚種類下拉選單。僅#51、活世界且背包／租裝面板關閉時顯示；原技能列水平分配，不新增modal loop。顯式CBN_SELCHANGE才捕捉SummonTarget；prompt不授權。選項切換、背包revision／完整owner、connection／generation／actor incarnation／corrections／control interruption／map變更均需重新選取。
- Controller獨立cast_summon接共用cast_request互斥、body completion、cooldown與pending；與actor／ground／item混用遭拒，裸request51仍不接收。reuse基值1000ms，MP30／HP30／寶石40318×3、lawful-10、模板與1800秒lifetime皆由server決定，不本機預扣／召喚／造ACK。
- 原材料admit前於action及2510，後面仍可能353地圖、743等級或319容量拒絕。新增summon-only pending觀測353／743，能覆蓋已到action回饋；世代／角色退役Interrupted優先。其他技能的全域message分類與equipment79不變。2510／755與caster action都不是companion成功ACK。
- 2510 action0 alias149／Over、755原六frame／attr8已由先前候選處理，本輪不改資源／音效來源，也不移記舊載入／像素資格。完整companion指令、自然1800秒與owner／傳送退役仍待當版GAME與GUI。
- 四Runtime差分仍33 overlays（19 Runtime／11 Rust QA／3 Java），365 top-level module；重建controller原五nested與window原rental一nested，原六resolved target不變。新10個純QA案例只寫入並編譯，前214合224全未跑；不是library inventory或通過數。
- 原三javap退出0；第一次外層reader誤用CP950在解碼時退出1，raw完整保留。改用嚴格UTF-8讀原輸出，沒有重跑javap／遊戲或將外層exit1改寫成功。

後續#55／#56及其餘116項入口；完整hurt／gfx基準、G06.3d、library雙版、Release與GUI仍待。只有使用者明確恢復測試後才執行。正常1049輸入與兩EXE、所有其他PE、registry與G06 binding／16案640次均保持原版本，整體未完成。
