# Release 執行版本的限定量測

唯一入口現在可用 `-BuildProfile Debug|Release` 明確選擇建置與執行版本，預設 Debug。兩者都使用 x86_64-pc-windows-msvc、鎖定離線依賴、`-j 1` 與唯一 target/current；各自保留自己的 EXE 和建置日誌。Release 原有 Cargo 設定為 opt-level=z、LTO、codegen-units=1。可用 `pwsh -NoProfile -File .\啟動現代客戶端.ps1 -BuildOnly -BuildProfile Release` 取得路徑。

同一個正式 E5F2E01B… JAR 的隔離場景中，Release EXE 7290880 bytes、SHA256 `EA4C0C6D6D56D6D6C22BD0F4874DD63C2FAF0A46C1A254AA8BDA8D87CCD1094D`，直接載入原資源的 RGBA 雜湊與 Debug 相同。六種正式 ClientSession 戰鬥情境全部通過；每一種初張世界像素與既有 Debug EXE `A7F327484D67A122E89F92177B8488192A1EEEB4FACDE894E2EB96F557DF80BD` 完全相同，攻擊封包數、遠程投射物抵達與權威結果保持一致。

| 場景 | Debug 首張渲染 ms | Release 首張渲染 ms | 擷取影格 Debug / Release |
| --- | ---: | ---: | ---: |
| room-world-attack | 4443.858 | 1364.480 | 12 / 49 |
| room-world-victory | 4446.634 | 1589.738 | 13 / 49 |
| room-world-defeat | 4404.909 | 1421.610 | 150 / 344 |
| room-world-refused | 4691.039 | 1610.411 | 24 / 97 |
| room-world-cycle | 4392.891 | 1608.154 | 166 / 344 |
| room-world-ranged | 6012.201 | 1715.423 | 12 / 49 |

六個首張渲染的中位數為 Debug 4445.246 ms、Release 1598.946 ms，相差 64.03%。這是同一主機上依序執行的 Debug / Release 隔離案例；OS 快取、CPU 負載及畫面擷取時間未控制，不是硬即時保證。`render_profile_us.total` 是進入世界 render 後的首張畫面耗時，不包含整段登入及場景建立。Release 的 54 種完整回歸、GUI、IME、DPI、音效裝置及玩家完整操作仍待驗收；schema35 的完整 Debug 驗收仍是廣範圍基準。未改寫 Debug EXE 或將 Release 偽裝成 Debug。

收據與逐案來源見 `驗證紀錄/current/native-optimized-profile`。下一步縮短剩餘 Sprite 分片索引與首次影格耗時，並完成原版血盟圖案的請求／快取／繪製與完整玩法。
