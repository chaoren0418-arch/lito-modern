# LiToModern 現行原始碼

請從上層 [LITO 現代客戶端重建](../README.md) 使用唯一現行入口。

```powershell
pwsh -NoProfile -File ..\啟動現代客戶端.ps1
pwsh -NoProfile -File ..\啟動現代客戶端.ps1 -Verify
```

預設 Windows 原生 x64，Cargo 快取集中 target/current，關閉 incremental。LiToModern 是新的 Rust 執行核心；預設啟動離線共享資源場景。點擊只放置本地角色，Space 播放來源動作／音效，Esc 關閉。

--profile 的歷史 Modern v37／v47 與 LiToModern-485F-Candidate 是保留候選，不代表現行正式伺服器相容。後者仍有歷史 READY／固定測試角色等限制；不作現行玩家入口。正式協定及可玩流程尚未完成。

| 位置 | 定位 |
| --- | --- |
| src/main.rs | 現行主程式及明確候選路由 |
| src/modern_lito/resources.rs、sprite.rs、tile.rs | 唯讀 archive 與有界原始格式解碼 |
| scene_model.rs、scene_preview.rs、compositor.rs | 當前離線渲染、動作與輸入切片 |
| native_wire.rs、native_version.rs、native_connection.rs | 現行正式原生 TCP／版本、持續 owner 與有界雙向 IO |
| native_login.rs、native_login_window.rs、tests/native_login.rs | 現行原生帳密／來源選服、Unicode 控件與正式 ClientSession 隔離驗證 |
| rebuild_verification.rs、tests/rebuild_*.rs | 當前資源／縮放及正式 JAR 加密子集驗證 |
| tests/oracle、tests/fixtures | 正式 JAR 的 CipherState／版本／登入 oracle，以及真正 ClientSession 記憶體測試端 |
| src/modern_lito/native_* | 歷史 Native 候選與可重用子元件，逐項核實後再採用 |
| logs、provenance、tools | 原始證據／來源記錄／歷史工具；不任意清除 |

舊版约 114 KB 的 README、R459～R542 交接與歷史 Markdown 報告已納入上層 SHA256 封存。以 [更新紀錄](../docs/HISTORY.md)、[來源與版本](../docs/SOURCES.zh-TW.md)、[架構與驗收](../docs/ARCHITECTURE.zh-TW.md) 作閱讀入口。具体刪除結果以驗證紀錄/20261012收據為準。
