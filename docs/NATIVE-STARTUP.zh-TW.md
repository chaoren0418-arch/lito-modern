# 重建版原生登入入口

一般開啟 `LiToModern.exe` 會使用原生登入流程。首次啟動先填登入伺服器的 `IP:連接埠` 與本機遊戲資源根目錄；完成設定後，才開啟既有帳密與來源選服視窗。開啟帳密視窗不會自動送出登入。

勾選「記住伺服器與資源路徑」後，設定儲存在 EXE 同一資料夾的 `LiToModern.native.json`；不勾選則只用於本次啟動。此檔只含設定版本、登入端點與資源路徑。帳號、密碼、角色或伺服器權威狀態不存入這份設定。

唯一開發入口：

```powershell
.\啟動現代客戶端.ps1 -BuildProfile Release
.\啟動現代客戶端.ps1 -BuildProfile Release -ConfigureNative
```

第二個指令重新開啟設定畫面。資源不存在或設定無法讀取時，一般啟動會讓使用者重新設定。所有資源仍由原生讀取器以唯讀方式載入；沒有複製資源或讀取登入器設定。

也可明確指定一份重建版原生登入設定：

```json
{
  "schema": 1,
  "login_endpoint": "192.168.1.10:32000",
  "resource_root": "D:\\遊戲資源"
}
```

範例位址與目錄須改成實際值。端點使用明確 IP（亦支援含中括號的 IPv6）；資源根目錄使用本機絕對路徑。

```powershell
.\啟動現代客戶端.ps1 -BuildProfile Release -NativeConfig 'D:\我的套件\LiToModern.native.json'
& 'D:\我的套件\LiToModern.exe' --check-native-config 'D:\我的套件\LiToModern.native.json'
```

`--check-native-config` 是不開視窗、不連線、不寫設定的檢查，回報 JSON。它核對設定格式、端點及目錄可讀性，不檢查整套資源內容或伺服器可用性。格式錯誤、未知欄位、相對設定路徑或不可讀目錄會回傳非零結束碼。

明確指定端點的既有入口仍可使用：

```powershell
.\啟動現代客戶端.ps1 -BuildProfile Release -NativeLoginEndpoint '192.168.1.10:32000' -ResourceRoot 'D:\遊戲資源'
```

離線共享資源展示改由 `-OfflineScene` 明確啟動；歷史 Modern profile 也保留明確指定入口。一般啟動使用現有原生 LOGIN／GAME 控制器。

本次來源為 1024 個宣告輸入，加一個 Windows manifest 編譯資源。改動的編譯、設定檔回歸與正常 EXE 檢查，見 `驗證紀錄/current/native-startup` 的實際收據。新設定畫面的實際顯示、實體輸入、多 DPI 及完整遊戲流程仍須另行驗收；此入口改動不代表 G01–G12 完成。
