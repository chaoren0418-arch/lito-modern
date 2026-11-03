# #79 靈魂昇華：離線開發候選

僅供重建版開發；正常來源、現行十三檔候選 registry、G06 綁定與十六份正式案例計畫均未替換。桌面與測試依使用者要求暫緩。

候選沿未選入的 #20／#29 擴充，補 #79 的 C14 `14 09 06 target-u32-LE`、未指定時對自己、原畫面菱形及已學技能選單。重用 10ms；MP、最大 HP／MP、1200 秒期限及成功／拒絕由固定 E1498871 的原 handler 決定，客戶端不預扣、不新增狀態計時。HP／MP 仍由原 S0D／S4D／S0C 接收路徑更新。

3935 以完整 header、三項屬性及原十九影格加入既有嚴格 selector bank；保留原九個定義、別名、tick 與拒絕條件。原 callback 次序為 source 1..18,0；每段 floor(ticks*1000/24) 毫秒，晚到只進一段並從該次 dispatch 重排。

**更正前報告：`FrameEvent::Effect(1356)` 是 parser 對原 `<` 的命名，表示模式 0 可重疊音效，並非子特效 sprite。** 固定原 OEP 的 43E977／43E9D6 與 4E61BC..4E61D1 可連到既有音效路徑；原 `sound/1356.wav` 只讀 RIFF metadata 為 PCM、雙聲道、22050Hz、16bit，沒有播放。109 的兩個值在 43E6B2 分別存入 647E5C／647E50；class 0 走 4A6288 預設分支，與現有 3936／3934 的嚴格 `(0 -1)` 定義一致。沒有新增通用子特效 renderer。

原音效已由既有 NativeAudioGroup／PreparedCue 處理，保留 scope、一次性 latch、退役與可重疊語意。新增六個案例程式：封包及不對稱範圍、無效狀態拒絕、選單與舊目標、伺服器權威生命值、原資源十九影格／1356 cue、晚輪詢與退役。另保留原 #20 六案與 #29 六案。案例均未執行，音效 player 在所寫案例中使用記錄器，仍不代表實體音效資格。

原 PE 路徑誤用 packed `LiTo.bin` 的 guard failure 6319bd exit1 已另存；改讀既有唯讀 `LiTo.unpacked-oep-current.exe`，保持要求 SHA 4271D92F 不變。失敗不是遊戲結果。

編譯只使用原完整 362 模組圖，以四個 runtime 路徑覆蓋及只改 fixture include 的配對 QA，兩版各 library／library-test／#20／#29／#79 metadata。原編譯日誌與結果由獨立 qualifier 核對；未連結／執行 PE、未啟動 server／DB、未做 GUI 或音效播放。

待使用者恢復測試後，依總報告接續 G06 與指定原版反例，再驗本候選的原資源、當版正式技能書、實際自用／對人／範圍／低 MP／拒絕／期限與重施放、最大值同步、聲音及 GUI。必須獨立合併、驗證後再套用正常來源；新當版完整 library Debug／Release、Release 回歸及完整 GUI 仍待，G01–G12 未完成。
