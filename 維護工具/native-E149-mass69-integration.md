# #69 原生控制器與GUI來源接線

本輪只做隔離來源與metadata編譯；使用者仍暫緩測試／桌面。沒有執行測試本文、--list、GAME、GUI、fixture／server／DB或音效。正常1049輸入、既有EXE、正式JAR、registry及G06 binding／640次不變；來源未套用，無新PE。

新增集體傳送目的地下拉：佔位項之後才是隨機傳送與自己收到的正式書籤。玩家必須明確選取，不預設隨機，不讓generic actor／ground request替代目的地。Choice、書籤revision及角色／場景owner在送出前再次核對，C14僅九byte map/x/y；沒有本機扣MP或材料。正常入口來源盤點增至76／剩104，下一步71／74。

下拉列表以不回繞的native命令ID及來源HWND處理通知；書籤／角色／場景更新、關閉再開均換ID，舊通知完全忽略，不清掉新選取。S30完整name、x/y/map先解析再更新，最多50筆；沒有有效GAME角色owner的合法晚到記錄只消化、不建立authority，格式錯誤仍保留診斷。**這是客戶端owner守衛；相同socket重選的封包交錯與server原子授權仍待當版GAME，不能由來源綁定推論通過。**

原生非模態748詢問預設聚焦「不同意」，Tab可操作；Enter只回覆聚焦且啟用的按鈕，重複鍵不重送，Escape為明確拒絕。已送同意後，拒絕／Escape只放棄本機等待，不送第二次C23。recipient不需學會69或有MP；血盟、距離、隱身、私店、成本與server自施法時起算的30秒由正式server判定，client收包後30秒只是顯示上限。

收到精確自己S55才回一次C4E，並綁定owner等待S5A；圖像／施法動作不當作成功ACK。等待期間清除持續移動／攻擊、舊目標與背包／租裝頁面，控制器同時攔住施法、移動、攻擊、拾取、物品、門、入口及NPC操作。可聚焦「取消」再按Enter或Space結束連線；沒有由Enter觸發預設登入或隱藏按鈕。

S5A受理時消耗該grant；過期owner不能落回ordinary／rental admission。WORLD經既有WorldEntering→精確角色／目的地重建，BATTLE維持原membership；一般戰鬥僅同map，rental mode6保留既有跨map策略。現行控制器八秒等待策略獨立記錄在mass_deadline，不被普通通知清掉；owner失效或逾時關閉該連線並清除舊權限。S5A後C08、原場景退役與重建沿既有流程，**完整TCP／G06／自然時間尚未執行**。

本輪五Runtime差分，仍42檔／28 Runtime／11 Rust QA／3 Java、371 modules、原六nested解析目標未變。新增10純案例來源（含原9及晚到書籤1），加前362共372案全未跑，不是完整library inventory。首版18metadata只保留修正前範圍；補上無owner書籤消化、舊下拉通知不改新Choice、明確取消Enter後，最終來源獨立重編譯18目標。原工具reset-anchor次序錯誤在候選寫出前退出1並保存，未藉此啟動或重跑測試。

下一步：先71／74剩餘開發及G06前提。恢復測試後依總報告：四元件／原反例、G06兩版16案640次、69目的地與邀請／拒絕／逾時／重施／舊owner／EOF／取消／重建、新技能原資源與source History、完整library Debug／Release、Release回歸及租裝等完整GUI；未確認前不交人工。
