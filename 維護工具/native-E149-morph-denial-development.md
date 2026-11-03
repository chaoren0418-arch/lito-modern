# G06 原物品模板／ID 入口的退役拒絕候選

新增兩個正式 BoundItem 入口 `setItem(NativeDeItemTemplate)` 與 `setItemId(int)` 的隔離 QA 原始碼，保留原八 setter／32 筆 trace 與 reader。每個原 admission 規劃兩次新呼叫，四個原 owner 共新增八次；擴充候選合計四十次。**現行 canonical binding 與十六份計畫仍是三十二次，本候選尚未接入，四十次不是已執行結果。**

固定 E1498871 的 BoundItem 位元碼：有 store 的分支先在原 inventory monitor 內 `check()`，才比較模板／ID 並拋原參數驗證例外。沒有 store 的 constructor 分支不在本資格。基底 `Item.setItem` 會寫模板、item ID，並呼叫 `setGreater`；因此必須證明 owner guard 先攔截，不能把參數錯誤當退役拒絕。

新候選從原八 setter 捕捉的同一個實際物件出發，找同 inventory 中另一個仍附著、模板參考與 item ID 都不同的真實 BoundItem。正向 `owner.requireActive` 成功後保留原參考；不 new／adopt owner、actor、inventory 或 template，不用 reflection 寫遊戲欄位。四個 admission 都必須有兩個真實模板，缺少時應明確失敗，不補造資料。

呼叫時已等原 session Futures 退出、原四筆 owner 退役與三十二 setter 拒絕結束，且 app／hub 尚未 close；再次核對兩個原 actor／entry／store／template 參考皆未替換或 detached、未 staged。`setItem` 直接傳另一 actor 的原模板，`setItemId` 傳該模板的原 ID。只接受精確 `IllegalStateException("source item owner is stale")` 與前後完整觀測相同。原參數驗證、同值、錯誤型別、detached／staged 都不合格。

新 observer 只反射讀實際欄位，包住既有唯讀 snapshot，額外保存兩個模板／ID 與 alternate entry／view 身分。仍保留原十八個 record accessor，不觸發能力 getter 或 inventory refresh。實際結果（含意外成功或不同 RuntimeException）先寫入並 flush，再判定接受／重拋原錯誤；不重試。

`native_E149_morph_denials.py` 為未呼叫的純 reader：先要求原三十二筆 reader 全部核對，再將每對新呼叫連回同一 owner／session／inventory／actor、最後原 before/after、原 trace 裡的兩個真實物件與不同 catalog ID，核對兩份觀測不變。來源／帳號仍沿原 source-qualified identity；這不證明其每個條件已獨立觸發。

本輪只做 javap、javac 與 Python AST。初稿 f2b633 exit0／六 class 封存；再增加「先保存實際結果」後另建候選並重新編譯，43da2d exit0／六 class。當版兩類 javap d7c76a exit0，未初始化遊戲類別。初稿與新稿不是測試重跑，均沒有執行案例。Rust／正常來源、兩正常 EXE、四元件 PE、正式 JAR 只讀未變。

下一步先把本候選接入新的 fixture binding、producer／outer reader 與計畫，保留原三十二筆與新八筆兩條 trace 的來源綁定及診斷封存；完成後待使用者恢復測試才執行。擴充永久屬性透過正常 GAME 的非預設狀態、source／account 各分支、timer lease／自然到期、Main 與完整 owner 仍待，不能以十個 BoundItem 入口宣告完整物品資格。
