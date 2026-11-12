# LiTo Modern 可攜包（開發候選）

這是 LiTo 重建版客戶端的可攜包：解壓後就能啟動，不需要安裝程式。它仍是**開發候選**，不是完整遊戲的正式交付；哪些流程已驗收，以專案的驗收紀錄為準，不能用「能啟動」或版本資訊代替。

## 系統需求

- **Windows 10 版本 1703（組建 15063）以上的 64 位元版本，或 Windows 11。** 不支援 32 位元 Windows、Windows 7／8／8.1。
- 不需要安裝 Visual C++ 可轉散發套件，也不需要 `VCRUNTIME140.dll`：C 執行階段已靜態連結在 `LiToModern.exe` 裡。
- 不需要 PowerShell 7、.NET 或系統管理員權限。
- 一份既有的遊戲資源資料夾（本包不含任何遊戲資源，也不含、不啟動 `LiTo.bin`）。
- 可連線的登入伺服器 `IP:連接埠`。

## 啟動

1. 把整個 ZIP 解壓到一個**可寫入**的一般資料夾（例如 `D:\LiToModern`；路徑可以含中文與空白）。不要直接在 ZIP 預覽視窗裡執行，也不要放在 `C:\Program Files` 這類需要管理員權限才能寫入的位置。
2. 雙擊 **`Start-LiToModern.cmd`**，或直接雙擊 **`LiToModern.exe`**，兩者相同。
3. 第一次啟動會出現「LiTo 連線設定」：填入登入伺服器的 `IP:連接埠`，以及本機遊戲資源資料夾的絕對路徑。
4. 設定完成後進入帳號密碼與選服畫面；開啟這個畫面不會自動送出登入。

勾選「記住伺服器與資源路徑」時，設定存成 EXE 旁邊的 `LiToModern.native.json`，內容只有設定版本、登入端點與資源路徑，不含帳號密碼。刪除這個檔案，或執行 `LiToModern.exe --configure-native`，可以重新設定。畫面與音效偏好另存於 `%LOCALAPPDATA%\LiToModern\preferences.json`。

## 包內檔案

| 檔案 | 用途 |
|---|---|
| `LiToModern.exe` | 唯一的程式檔（x64）。 |
| `Start-LiToModern.cmd` | 可雙擊的啟動入口，只用到 `cmd.exe`。 |
| `README.md` | 本說明。 |
| `manifest/source.json` | **來源版本清單**：git commit、建置輸入摘要與逐檔 SHA-256。 |
| `manifest/executable.json` | **EXE 版本清單**：SHA-256、`--version` 內容、匯入的系統 DLL、連結器與 Rust 工具鏈版本、最低 Windows 版本。 |
| `manifest/resources.json` | **資源版本清單**：打包時對照的外部遊戲資源資料夾逐檔長度與 SHA-256。資源本身不在包內。 |
| `SHA256.json` | 以上所有檔案的長度與 SHA-256。 |

本包不附任何 DLL、原生 BIN 或資源封存檔；`manifest/resources.json` 只記錄雜湊，供比對你手上的資源是否與打包時相同。

## 其他啟動參數

- `LiToModern.exe --help`：列出全部參數。
- `LiToModern.exe --version`：輸出版本 JSON；不讀設定、不連線、不開視窗。列出的協定與擴充是程式支援的能力，不代表伺服器已啟用。
- `LiToModern.exe --check-native-config <設定檔絕對路徑>`：不開視窗、不連線，檢查一份連線設定的格式與資源資料夾是否可讀，輸出 JSON。
- `LiToModern.exe --native-config <設定檔絕對路徑>`：使用指定的連線設定啟動。
- `LiToModern.exe --profile <設定檔絕對路徑>`：歷史的 Modern 協定候選入口，與現行伺服器的相容性尚未驗證；一般使用者不需要。

`--probe-native-*` 是開發診斷，只接受本機 loopback 端點；需要資源時讀取環境變數 `LITO_CLIENT_ROOT`，或 EXE 旁已儲存的連線設定。程式內沒有寫死任何開發機路徑。

## 啟動失敗時

- 出現「啟動失敗」訊息視窗：依訊息修正（常見是資源資料夾不存在或無法讀取），或用 `--configure-native` 重新設定。
- 雙擊後出現「找不到 LiToModern.exe」（`LiToModern.exe was not found`）：ZIP 沒有完整解壓，請整包解壓後再試。
- 完全沒有反應或系統提示無法執行：先確認 Windows 版本符合上面的系統需求。
