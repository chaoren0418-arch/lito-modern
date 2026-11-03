# LiTo Modern 玩家驗證候選

玩家只需開啟 **LiToModern.exe**。選取由伺服器管理者提供的 LiTo 設定檔後，再於登入畫面輸入帳號密碼；取消選檔不會連線。

也可使用：`LiToModern.exe --profile <設定檔絕對路徑>`。

## 執行前確認

- Windows 與 x86 Visual C++ Runtime（VCRUNTIME140.dll）已安裝。
- 設定檔指定正確的伺服器、TLS 信任及既有 Lineage 資源目錄。
- `config/profile.example.json` 僅為格式範例，不是可直接登入的設定；請取得管理者配置的版本，勿將帳密或資料庫密碼放入檔案。

本程式不啟動 TW、LHZ 或原生 LiTo.bin，不需要管理員權限，也不使用 LiTo 專用資源包。畫面／音效設定獨立存於 `%LOCALAPPDATA%/LiToModern/preferences.json`。

## 本候選範圍

此 EXE 的一般入口依設定選擇協定：新設定明確使用 schema2／modern_v47，舊 schema1 仍使用 v37；不會因連線失敗而降級或切去 TW。範例採用 v47。這是開發中的玩家驗證候選，**不是完整遊戲正式交付**；所有按鈕、長時間操作、其餘技能／地圖與真正視窗仍須驗收，不能以版本資訊或建置成功代替。

## 管理者設定

請調整 `shared_resource_root` 為既有 Lineage 資源目錄、`effect_catalog_path` 為現有 Lineage `effectlist2.xml` 的本機絕對路徑、`endpoint` 為數字 IP 與端口，以及 `tls_server_name` 為憑證中的伺服器名稱。完整 VIP 本場狀態圖只在明確設定此路徑且共享圖像逐項資格化後顯示；XML 不會被打包或自動尋找。範例的 localhost 配合 web_pki 不構成可用的本機 TLS；私人測試伺服器須改用 `trust.kind = private_ca`，並提供 `certificate_der_base64` 公開 DER CA 憑證。不要停用憑證檢查，也不要交付私人金鑰。

需要角色治癒擴充時，在相容的 v47 設定加入 `gameplay_extension = player_healing_v1`（JSON 字串欄位）。伺服器亦須啟用對應擴充及來源／地圖資格；不支援時會在認證前拒絕。沒有此欄位的 v47 設定維持原行為，不表示角色治癒已啟用。

目前原始碼另提供 `armor_protection_v1` 與 `player_healing_armor_v1`，分別明確要求護甲資料及治癒加護甲資料。這兩項仍是開發驗證設定：**護甲場景尚不能完成共享資源呈現與載入，不可作為可遊玩的護甲功能交付。** 不相容伺服器會在認證前拒絕，不會自動回退治癒或舊協定；設定識別也與舊設定分開。既有發布候選未因此更新，預設範例未啟用這些擴充。

## 版本與完整性

目前原始碼支援 `LiToModern.exe --probe-profile <設定檔絕對路徑>`，供管理者在不輸入帳密時檢查 TLS 與協定配對。須使用包含此功能的新建置；既有發布候選不會自動更新。

此命令只讀指定設定，沿用一般玩家的憑證驗證、版本／擴充要求與逾時設定，嘗試一次握手後關閉連線。輸出一份 JSON：成功退出碼為 0，參數、設定、連線或關閉失敗為 2；不彈出登入或錯誤視窗。它不保存帳密或設定，不讀共享資源、不登入、不查來源資料庫，也不自動降級或重試。`not_checked` 列出尚未驗證的範圍；握手成功不等於帳號可登入、地圖能載入或遊戲已完成。

管理者應依 JSON 的 `stage` 與 `error.code` 檢查設定、端點、公開 CA、憑證名稱／有效期或伺服器支援能力，不要停用 TLS 驗證。成功後仍須另行實測登入、來源／選角、共享資源載入與完整遊戲流程。

`LiToModern.exe --version` 輸出 JSON 診斷，可透過重新導向標準輸出擷取；不讀設定、不連線、不顯示視窗。列出的協定與擴充是程式支援能力，不代表伺服器已啟用或全遊戲已驗收。

目前原始碼的版本清單另列出 `armor_presentation_v1` 與 `player_healing_armor_presentation_v1`，明確要求新的護甲呈現資料；與前述僅護甲資料設定不同。共享呈現只在已核對資源下驗證，11101 缺件仍未解決，不能由擴充清單推定原設定已可遊玩。

`SHA256.json` 記錄這份候選的檔案雜湊與建置來源摘要。開發原始碼、工具、建置收據與測試報告留在 `LiTo381-Modern` 專案，不放入玩家包。本包只含一個玩家 EXE，不附 DLL、原生 BIN 或資源包；TW/LHZ 使用它自己的 EXE。
