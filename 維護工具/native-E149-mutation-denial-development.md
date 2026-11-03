# G06 原物件退役後的 setter 拒絕開發

這是隔離候選的編譯／位元碼與來源資格，沒有執行任何 guard、setter、parser、伺服器、SQL、測試或 GUI。正常 Rust 來源、四份元件 PE、正式 controller 映像及目前 canonical fixture／16案計畫皆沿原資格；本候選尚未接入。

原 E149 BoundItem 的八個覆寫入口（setCount、setEquipped、setIdentified、setSealed、setEnchantLevel、set_durability、setBless、setChargeCount）在 store 非 null 的分支，先經 check → requireAttached → Owner.requireActive，再做參數檢查與原基底修改。constructor 使用的 store=null 分支不在本探測範圍。setItem／setItemId 的模板限制、未覆寫的擴充屬性、所有正常 GAME 非預設物品建立路徑與完整 owner 矩陣仍另驗。

候選保留原登入建立的四個 owner，各取一個真實、仍附著於原背包的 BoundItem；取樣須有原正向 requireActive 與原唯讀 source observation。先完成原 session Futures 及既有四個退役 guard，然後在 hub close 前，逐物件嘗試八個 setter，參數與原欄位值不同。只接受精確 java.lang.IllegalStateException／source item owner is stale；脫離、staged、參數錯誤或其他異常都不能冒充退役拒絕。

每次呼叫前後保存原背包 record view、native 永久屬性、已快取能力、診斷欄位、actor 核心欄位與物件參考識別；兩份觀測必須完全相同。32次是計畫數，不是已發生的拒絕數。全檔案 SHA、程序退出與 Future 順序仍須由外層正式 reader 合併核對。

native_E149_mutation_denials.py 是尚未呼叫的純資料 reader，嚴格要求32筆、四個實際角色 admission、每組八個固定 setter、不同值與正確型別、精確拒絕、同一保留 actor，以及原 trace 的正向觀測與最終退役。它不啟動程序、不呼叫遊戲方法，也不能單獨授予完整 owner／物品／G06 資格。

下一步是核對並接入新的 Java fixture binding、正式 producer／reader 與 suite 計畫，保留原 Rust 映像來源綁定；使用者恢復測試後，才實跑並核對32次拒絕。取樣是否能捕獲四個非空原背包、原物件是否仍附著、計時器背景變動與完整輸出形狀都尚未驗證。不得用這個有限探測代替 source/account 分支、staged/detached、非預設擴充屬性、自然期限或完整 Main。
