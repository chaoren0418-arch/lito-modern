# Issue #167 — T029-1 遠程戰鬥基礎

## 結論

本輪接續 PR #205 的 `bd2a423`，補上 #1156 stone、#2349 energy arrow、S23 跨 class 的 type 分派與可變長尾段。**整張 #167 仍有未完成項目，尚不能判定完整驗收通過。**

工作在指定 worktree、既有 `task/167` 進行。已讀 AGENTS.md、docs/REPORT.md、TASKS.md T029。原始專案及客戶端資源採唯讀引用，寫入限本 worktree。

## 做了什麼

- `ranged_effect.rs` 載入石頭1156、陰影1157、能量箭2349的各自原SPR。依原定義檢查方向、影格、事件、混合模式、secondary與陰影。保留既有66／2989路徑。
- 能量箭Sound97使用既有 `PreparedCue`／Model音效發布路徑，保留 `SuppressWhileSameIdPlaying`。單影格依class3回呼回到source0，含抵達最後一筆音效。同一時間重取樣不重播。
- 石頭依原生body→shadow順序提交。新增非黑背景逐像素比較，核對Screen及RGB555 shadow。
- S23依 `h graphics,c type,h fromX,h fromY,h toX,h toY,c flag,s name` 解析，保留flag及原MS950名稱bytes，上限127 bytes。完整驗證長度、NUL終止與欄位後才修改World。
- 非零flag抵達後轉為原圖source0的留地呈現，停止新增音效。依原生規則保留0／-1兩個物件ID，同ID由較晚抵達者替換。S15移除-1物件，新世界清除全部留地呈現。
- 已接class1／class4／class5呈現器按graphics分派，移除wire type0限制。action18／19施法回覆識別與已接效果的延後受傷回呼一併對齊。
- Oracle以E149 writer新增特殊箭與十種graphics各兩種type。type0來自 `Protocol.attackSpell`，type127來自 `Protocol.rangedPhysicalAttack`。具名尾段用正式 `Protocol.Writer` 按原生reader格式建立。

## 已驗證

### 來源與映射

| 來源 | SHA256 |
| --- | --- |
| 正式JAR | `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498` |
| common_catalog | `B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01` |
| 原Text/list.spr解碼bytes | `523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127` |
| 原生PE | `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9` |

JAR唯讀引用相鄰登入器的 `LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar`。common catalog位於原始專案 `LiTo381-Modern/target/current/shared-profile-inputs/common_catalog-B64ED920.bin`。PE位於相鄰登入器 `舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`。

本輪重產oracle前核對JAR及common catalog雜湊，啟用 `tw13081901-v1`。`ResourceDatabase.arrow`／`flightType`沿用前次E149 bytecode核對，本輪再以隔離記憶體Snapshot驗證66／2989映射與127。未呼叫 `ResourceDatabase.load` 或建立SQL／網路連線。

### 特殊投射物的原生契約

| 圖形 | 原資源 | 生命週期證據 |
| --- | --- | --- |
| 66／2989 | 八方向三影格、type9、class3 -1 | stance20／62、action21／63、Hit一次發射、抵達回呼與退休 |
| 1156 | `0.fly(0 1,0.0:1)`、shadow1157、type9、class3 -1 | 單一原SPR與陰影、八方向路徑、Model成功呈現、一次hurt |
| 2349 | `0.fly(1 1,0.0:10[97)`、type9、attr8、class3 219 | 八個原SPR、Screen、原Sound97、Model成功呈現、最後回呼後退休 |

**#2349的secondary219依原生class3契約保留宣告，沒有建立219物件。** `4A5ED0`按graphics的109 class table跳轉，class3於 `4A60A8` 呼叫 `40E2E0`。該建構子及 `40E420`／`40E8F0` 回呼沒有讀取secondary table `647E50`。class4／5分支則明確讀取此table。

`40E7E0–40E803`設定影格索引後呼叫 `4E5B50`，後者先加一再依影格數回繞。2349只有一影格，因此每個30ms回呼都進入帶Sound97的source0。測試核對12次回呼各一筆原音效事件、同時間重取樣零新增、一次hurt及退休後零事件。`40E828`／`40E854`的body／shadow提交順序也已核對。

非零flag的 `40E868–40E8A6` 呼叫 `4E7A40`，`4E7C3F` 將來源影格設為0，`4E7E8F–4E7E91` 以事件關閉參數載入。`4A602D–4A603A`保留-1 sentinel並將其他ID歸零。因此留地呈現最多兩筆，名稱、type、方向與抵達座標保留於收據。回歸涵蓋飛行途中S15取消、取消後新發射、同ID替換、延遲取樣的抵達順序、攻擊者／目標離開後持續顯示、S15移除及新世界清除。Model測試驗證留地source0實際提交原圖，抵達hurt僅一次，留地期間零新增音效。

已檢視1156與2349離線Model飛行PNG，並用收據核對各自SPR。這項檢視屬離線場景驗證。

### S23矩陣

| class／圖形 | 封包來源 | 驗證結果 |
| --- | --- | --- |
| class3 66、2989、1156、2349 | E149 rangedPhysicalAttack的127 | 各自原資源、Model、發射、hurt、退休 |
| class1 252、236、1801 | E149兩種writer，type0／127 | 6組Model正向呈現、一次hurt、退休及World不變 |
| class4 167、1583、1799 | 同上 | 6組Model正向呈現，使用各自飛行與secondary資源 |
| class5 1797 | 同上 | 2組Model正向呈現，飛行、secondary、hurt、退休 |
| TW 11738、11746、11748 | 同上 | 6組不可用負向案例，明確unavailable，零圖像及hurt；成功呈現待驗證 |
| 非空名稱、flag0 | 正式Writer按原生reader格式建立 | 具名stone經World→Model使用1156／1157；截斷、內嵌NUL、額外尾段與過長名稱原子拒絕 |
| 非零flag | 同上 | 固定封包終點、原音效、抵達原圖source0留地呈現、同ID替換、S15及世界更換清除 |

14組跨class正向案例合計338個drawn frame與14次hurt，最後active皆為0，完整World序列化前後相等。前次256種type解析合成測試保留，class3合成生命週期擴為四圖形1024組。這些合成案例只提供邊界回歸，真實雙玩家發送組合仍待T043。

### 建置與測試

以原 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult` 判讀。要求結束碼0、可解析的 `running N tests`／`test result:`、N大於0、failed為0。TEMP／TMP導向本worktree的 `scratch/issue-167/tmp`。

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
$common = @('--locked','--offline','-j1','--target','x86_64-pc-windows-msvc','--target-dir','target/current')
cargo test @common --test native_ranged -- --test-threads=1
cargo test @common --test native_s23_profiles -- --test-threads=1
cargo test @common --test native_ranged_profiles
cargo test @common --test native_combat
cargo test @common --test native_world
cargo test @common --test native_fire_arrow -- --test-threads=1
cargo test @common --test native_ice_arrow -- --test-threads=1
cargo test @common --lib ranged_effect::tests
```

最終 `cargo check --all-targets` 結束碼 **0**，gate **PASS**。八組測試合計 **61 passed、0 failed、0 ignored**，每組結束碼0且gate PASS。完成全部測試後再執行一次同參數cargo check，結束碼同為0。

| 組別 | 結束碼／gate | test result原文 |
| --- | --- | --- |
| native_ranged | 0／PASS | `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 47.41s` |
| native_s23_profiles | 0／PASS | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 280.83s` |
| native_ranged_profiles | 0／PASS | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.57s` |
| native_combat | 0／PASS | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s` |
| native_world | 0／PASS | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| native_fire_arrow | 0／PASS | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.16s` |
| native_ice_arrow | 0／PASS | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.47s` |
| ranged_effect | 0／PASS | `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 2468 filtered out; finished in 0.02s` |

首次跨class測試結束碼101、3 passed／1 failed，失敗於11738資源資格。已將TW不可用案例獨立驗證，六組不得計為成功呈現。初輪log保留於 `native_s23_profiles.initial.log`。建置有既有warnings。

## 推測

- 原生class分派與所有u8 type的class3合成回歸吻合。正式服務每個自訂arrowGfx實際搭配的type、動作與裝備，仍需T043收據。
- 本輪以原生class3靜態回呼裁定2349 secondary不建立。原BIN執行畫面及實體音效裝置仍待真機驗收。

## 待驗證與未完成事項

1. 原Text/list.spr另有23種class3，尚未接入。圖形為2740、2748、3584、3692、3705、4026、4523、4530、4547、4552、4621、4799、5415、5526、6767、7288、8721、8771、8904、8916、9545、10699、10782，各有影格、聲音、overlay、cloth或shadow契約。自動審核拒絕一次重寫核心載入器並擴充全部27種的腳本，理由為程式損壞與範圍風險。已提出逐組擴充的授權問題，該重寫未執行。
2. TW三種呈現器受Issue #111既有資格限制阻擋。現用 `0-客戶端/launcher/TW13081901.pak` 的SHA256為 `92B512A32349D0775E1036F1832E21F1141AE66E64821D6978F0879246E59CA7`。本輪保留資格檢查，成功呈現待資源資格處理。
3. 真實雙玩家、正式扣彈藥、TCP、Release、GUI及實體音效裝置待後續驗收。執行期間沒有啟動遊戲、操作既有遊戲程序、連正式服務或DB。
4. T029第二張的操作、空揮與拒絕條件由該張接續。

## 產出路徑

- 程式位於 `LiTo381-Modern/src/modern_lito/` 的 `ranged_effect.rs`、`native_projectiles.rs`、`native_combat.rs`、`native_world.rs`、`native_world_scene.rs`、`native_spell_projectiles.rs`、`native_class1.rs`、`native_disintegrate.rs`、`native_cone_cold.rs`。
- 測試位於 `LiTo381-Modern/tests/native_ranged.rs`、`native_ranged_profiles.rs`、`native_s23_profiles.rs`。
- Oracle與向量位於 `LiTo381-Modern/tests/oracle/NativeRangedOracle.java`、`tests/fixtures/native-ranged-current.json`。
- 本報告為 `docs/reports/issue-167.md`。
- 本機log與Model矩陣為 `LiTo381-Modern/target/current/verification/issue-167/`。飛行PNG為 `target/current/verification/native-projectile1156-*`、`native-projectile2349-*`，排除於git。
- 原生靜態解碼、oracle runner與暫存為 `scratch/issue-167/`，排除於git。

## 給下一棒的提示

Claude可先審查本輪具體修正、14組正向矩陣與留地生命週期。請維持#167未完成，待其他特殊class3補齊。TW資格由本機資源任務接續，處理後再重跑六組正向呈現。沿用task/167與PR #205繼續工作。本輪沒有修改docs/REPORT.md。
