# 租裝返回候選：返回／重入及主動服務清理已隔離驗證，正式檔未更新

正式 JAR BBC59308 的兩個 BATTLE 優先分派會擋住已開始的租裝返回。`current-guard-candidate` 以固定正式類別為輸入，只在 quitFight／exitRoom 插入限定於既有租裝 pending 狀態的分派條件；保留原方法內容，其餘2,148個方法的實際指令與例外範圍均核對不變。JAR只有ClientSession.class一個項目改變，候選SHA256為6887A337E347D561366E0EB7EB8E93A2B7EA03ADD7A3888FC4BE2B70C3479DA7。

兩名實際角色已通過C10／S65／CD9／SF0／CD0／SD0／SF5、角色清單刷新與重新入村；舊租裝背包清空、背包選擇代次及世界代次更新，原資源重新繪製。相同客戶端與候選JAR也通過普通雙場近戰；對未改正式JAR重驗仍重現原返回缺口。這是隔離候選的限定成果，主動旅館租約與額外租用品亦已補驗：兩角色從503／501退出，9把鑰匙及2件租用品收到正式移除封包，租約清除後重新入村；BEB8BC26另完成屋主返回使map502共享訪客回到500、兩把鑰匙失效及雙方重入。普通戰鬥中途退出與伺服器錯誤ACK仍另驗。

可審查來源為兩個Java工具及客戶端的native_login.rs；獨立證據入口是`驗證紀錄/current/native-world-integration/rental-return-candidate-qualified.json`。`build-review-candidate.ps1`只在本重建版target/current建立新候選並核對固定雜湊，不寫入相鄰來源、不啟動正式服務。部署與完整回歸尚未執行。

## 直接重編譯來源的失敗紀錄

候選只調整 quitFight／exitRoom 的第一個 BATTLE 分派：已存在的租裝返回或傳送階段可進入後面的既有處理，普通戰鬥退出維持原路徑。未擴大允許未請求或過時 ACK。

對固定正式 JAR BBC59308 編譯時發現四個既有介面差異，包含 CaptureLog.Connection／connection 與 GameHub.selectCharacter 多參數呼叫。這些差異存在於相鄰原始碼，與候選的兩個 guard 變更無關；不能靠改動其他 API 強行產生可執行檔。

`source-provenance.json` 保存來源與候選 SHA256、四個錯誤及正式 JAR 身份；`compile.log` 保留原始編譯失敗。後續改用固定 JAR 的兩個受限指令前綴，已完成前述雙端返回及服務清理；共享訪客清理其後亦已通過；普通戰鬥中途退出與伺服器錯誤 ACK 仍待驗證。若日後採完整來源重編，仍須取得與正式 JAR 相符的來源。相鄰來源與正式程序均未變更。

## 重建與隔離驗證

客戶端3971F1C8另完成返回交握守衛：提早S_F0、退出確認前的S_D0及離房前的S_F5不能跳過階段。六項控制器負面測試與十四項明確整合測試通過，同版候選主動服務返回／普通雙場近戰也通過；未改正式版仍重現返回邊界。證據為`驗證紀錄/current/native-world-integration/rental-return-ack-guards-qualified.json`，這不代表伺服器端錯誤封包注入測試已完成。

執行 `current-guard-candidate/build-review-candidate.ps1`，會在本專案 `LiTo381-Modern/target/current/rental-return-review-build` 建立唯一目錄，核對輸出 JAR 雜湊及 Java 類別驗證。重建腳本已實際執行成功，不部署正式服務。

將輸出絕對路徑傳給 `current-guard-candidate/run-review-peer.ps1 -SkipBuild -ReferenceJar <輸出JAR> -ActionProfile rental-return -ReturnServices`。候選 JAR 必須位於本專案 target/current，且符合固定候選雜湊；執行後依序呼叫 `verify-candidate-return.py <證據目錄>` 與 `verify-candidate-services-return.py <證據目錄>` 進行獨立比較，參數是含有 receipt.json 的目錄。此參數避免依賴提案目錄內不存在的 JAR。
