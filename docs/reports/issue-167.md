# Issue #167 — T029-1 遠程戰鬥基礎

## 做了什麼

本次工作在既有 `task/167` worktree 進行，範圍為 T029 第一張的遠程封包、弓與投擲、投射物資源及生命週期。

- S23 非零 projectile 的固定 29-byte 尾段會保留所有 u8 flightType。截斷、額外資料、錯誤 heading、零 graphics、未命中卻有傷害標記仍在修改 World 前拒絕。
- 原生接收器按 graphics 的 `list.spr` effect class 分派。#66 arrow 與 #2989 sting knife 均使用已核對的 class3 路徑，flightType 會保留在呈現收據。
- `RangedAssets::load` 與既有 catalog 呼叫入口繼續載入 #66。新增指定圖形入口，依各自 graphics 載入八個 SPR bank，逐一檢查三影格、八方向、class3、無 secondary、無事件的資源契約。
- stance20 預載 #66，stance62 預載 #2989。實際發射圖形取自伺服器 S23。兩種圖形共用每 30ms 的 fixed-point 路徑、Hit 事件一次發射、發射時固定終點、抵達受傷回呼與物件世代防護。
- World 將兩種 class3 圖形歸為物理攻擊，含 type0 的 class3 封包。抵達回呼只驅動受傷呈現。
- 以 E149 正式 JAR 重產 `native-ranged-current.json`，加入 action63／gfx2989 以及 ResourceDatabase snapshot 映射向量。

## 已驗證

### 正式版本與映射靜態核對

唯讀正式 JAR 路徑為 `D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar`。本次前後 SHA256 均為下列值。

```text
E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498
```

使用 Microsoft JDK 17.0.17 的 `javap -p -c` 核對下列方法。

| 方法 | bytecode 證據與結論 |
| --- | --- |
| `Protocol.rangedPhysicalAttack` | offset 2 呼叫 `ResourceDatabase.arrow`，offset 197 呼叫 `flightType`，輸出 `h graphics,c type,h fromX,h fromY,h toX,h toY,c 0,s empty` |
| `ResourceDatabase.load` | offset 1577–1614 要求 graphics 66、type127、opcode35，其他組合會拋 SQLException |
| `ResourceDatabase.arrow` | snapshot 存在、shared profile 啟用且原圖形為66時使用 snapshot.arrow，其他原圖形保留 |
| `ResourceDatabase.flightType` | snapshot 存在且 shared profile 啟用時使用 snapshot.flightType |
| `SourceProjectilePhysicalStats.effectGfx` | 正值 arrowGfx 直接採用，缺少自訂圖形時 useType -3 使用2989，其餘使用66 |

Oracle 僅執行封包 writer/parser 與映射方法，以空集合建立記憶體 Snapshot 並於 finally 還原。映射 fixture 使用目前資格檢查允許的66／127。Oracle 過程沒有呼叫 `ResourceDatabase.load` 或建立網路／SQL 連線。

### 原生 class3 與資源核對

以 capstone 5.0.6 靜態解碼唯讀原生 PE，SHA256 為 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`。

- `4A5ED0` 讀取共同尾段 `hchhhhcs`。`4A5FBx–4A5FD9` 從 graphics 對應的 class table 取得分支，class3 於 `4A60A8` 呼叫 `40E2E0`。
- `40E375–40E382` 將 wire type 存進 effect 的 byte12。既有 class3 路徑與影格回呼由 `40E420`、`40E8F0` 執行。
- 原解碼 `Text/list.spr` 的 #66 與 #2989 均為 `0.fly(1 3,0.2:1 0.0:1 0.1:1) 102.type(9) 109.effect(3 -1)`。解碼檔 SHA256 為 `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127`。資源測試另外從現用 `D:\lineage開發\0-客戶端` 的 Text／Sprite archive 重新讀取並檢查。

### 建置與測試

本次使用 `維護工具/run-local-test-suite.ps1` 原本的 `Measure-TestResult` 函式判讀各次結果。每個測試程序必須有一組可解析的 `running N tests` 與 `test result`，N 大於0、failed為0、結束碼為0。

執行目錄為 worktree 內的 `LiTo381-Modern`。檢查指令如下。

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

測試共用參數及各組指令如下。建置暫存均導向本 worktree 的 scratch，產物使用 `target/current`。

```powershell
$common = @('--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current')
cargo test @common --test native_ranged -- --test-threads=1
cargo test @common --test native_ranged_profiles
cargo test @common --test native_combat
cargo test @common --test native_fire_arrow -- --test-threads=1
cargo test @common --test native_ice_arrow -- --test-threads=1
cargo test @common --lib ranged_effect::tests
```

最終 `cargo check` 結束碼為 **0**，gate 為 **PASS**。六組測試合計 **45 passed、0 failed、0 ignored**，各次結束碼均為 **0**，gate 均為 **PASS**。以下保留 `test result:` 原文。

| 組別 | 結束碼 | 原文 |
| --- | --- | --- |
| native_ranged | 0 | `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.15s` |
| native_ranged_profiles | 0 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.51s` |
| native_combat | 0 | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s` |
| native_fire_arrow | 0 | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.75s` |
| native_ice_arrow | 0 | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.03s` |
| ranged_effect::tests | 0 | `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 2468 filtered out; finished in 0.02s` |

初次工具呼叫曾遇到沙箱檔案存取限制及 PowerShell 5 的 native stderr 判讀問題，之後改由 PowerShell 7／子程序 runner 執行。最終表格均取實際成功程序的結束碼及輸出。建置仍有既有 warnings。全套 library／Release／網路測試待後續任務安排。

新增回歸涵蓋所有256種 type 的解析與截斷原子性、兩種圖形的512組發射／抵達組合、stance62 預載、實際 action63 到 Model 的呈現、弓與刺針交錯、同 ID 替換／新世界退休、miss／零傷害／死亡目標、八方向原始 SPR 像素與改動資源契約的拒絕。既有近戰、火箭、冰箭一併回歸。

已檢視離線 Model 產出的 #2989 飛行 PNG，並核對收據實際引用 `2989-7.spr`、gfx2989、type127、原生 action63。場景影格與真實玩家 GUI 驗收分開記錄。

## 推測

- 已核對的 class3 建構子及路徑支持以 graphics 分派、完整保留 type byte。對所有256種 type 的回歸使用合成變體，實際正式 writer 目前產生的物理 type 為127。
- 自訂 arrowGfx 可由伺服器來源帶入。每種自訂圖形仍需依自己的動畫、音效、secondary 與 shadow 契約實作。

## 待驗證與未完成事項

- 特殊投射物本次具體完成 #2989 投擲刺針。#2349 energy arrow、#1156 stone 等其餘 class3 特殊圖形仍待各自的資源／事件實作。#2349 會明確拒絕載入普通箭 profile，執行時未知圖形仍有 omitted 計數與既有受傷回退行為。
- S23 的非零末尾旗標與非空字串仍超出目前固定尾段解析範圍。其他 effect class 的 type 變體亦待各自的呈現器支援。
- 雙真實玩家 fixture、現版實際扣彈藥、真實 TCP、Release 執行、GUI 與實體音效仍待 T043 與後續驗收。本次採用離線 World／Model／資源測試。
- T029 第二張的操作、空揮封包及拒絕條件由該張任務接續。

## 產出路徑

- 程式碼位於 `LiTo381-Modern/src/modern_lito/{native_combat,native_world,native_projectiles,ranged_effect}.rs`。
- 測試位於 `LiTo381-Modern/tests/native_ranged.rs`、`native_ranged_profiles.rs`、`tests/oracle/NativeRangedOracle.java`、`tests/fixtures/native-ranged-current.json`。
- 本報告位於 `docs/reports/issue-167.md`。
- 本機建置與測試紀錄位於 `LiTo381-Modern/target/current/verification/issue-167/`，離線場景影格位於 `target/current/verification/native-projectile{66,2989}*`，均排除於 git。
- 暫存 oracle classes、javap 摘錄與 PE 靜態檢查位於 `scratch/issue-167/`，排除於 git。

## 給下一棒的提示

請 Claude 審查上述有限支援範圍，特別留意「其餘特殊圖形」與「非空 S23 尾段」仍需接續，因此 T029 整項維持待完成。T043 本機執行者可用本版重產向量作為封包基準，再建立雙玩家弓／投擲連線驗收。原始專案與相鄰來源全程唯讀，本次僅在 task-167 worktree 寫入，未啟動遊戲或操作正式服務。
