# 可攜候選的 DLL 與封裝範圍

正常入口已使用程式旁 LiToModern.native.json 及首次設定視窗，沒有再新增另一套資源路徑流程。唯讀來源比對仍綁目前21檔候選／Release GUI PE AA766180；沒有變更程式來源或重編。

Microsoft SDK dumpbin 與獨立 raw PE reader 對兩版正常入口、VC runtime 的依賴清單一致。Release PE 有20個 import descriptor（其中 kernel32 大小寫兩列）、沒有 delay import；其中非 Windows 基底項為 VCRUNTIME140.dll。程式匯入該DLL的九個具名符號，都存在於所選 x64 runtime 的71項 export table。這只是表格／符號存在性核對，不是執行或 ABI 行為驗證。

DLL取自本機 Visual Studio 18/Insiders 的 Redist/x64/Microsoft.VC145.CRT/vcruntime140.dll，檔案 SHA D1F4225D，資源內 file/product version14.51.36247.0。封裝保留這份實際來源與版本，**只建立本地隔離驗證候選，不主張正式對外發佈資格**。正式工具鏈／runtime來源、維護更新及最終發佈仍需在Release交付階段確認。

程式旁部署方式與版本配對依據 [Microsoft部署方式](https://learn.microsoft.com/en-us/cpp/windows/choosing-a-deployment-method?view=msvc-170)及[DLL相依判定](https://learn.microsoft.com/en-us/cpp/windows/determining-which-dlls-to-redistribute?view=msvc-170)。文件說明本地部署可將所需VC DLL置於EXE旁，但更新由應用交付負責；這不是對此預覽來源的外發資格判定。

內部ZIP只含 Release LiToModern.exe、vcruntime140.dll、說明及payload manifest；不帶工作站設定檔、帳密、伺服器JAR、遊戲資源或測試EXE，不安裝runtime、不寫registry或System32。對ZIP每個entry直接讀回SHA，不解壓／載入／執行，保留唯一正常開發入口不變。

其餘靜態匯入按明列的Windows系統／API-set契約記錄，並非已在乾淨主機解析。程式與runtime都有LoadLibrary／GetProcAddress匯入，動態載入閉包尚未證明；最低Windows版本、乾淨主機、首次設定、中文字路徑、原資源、連線／GUI／音效都未驗證。測試與桌面仍暫緩，G12及整體未完成，未交人工。
