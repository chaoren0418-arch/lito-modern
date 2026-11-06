# Issue #184 — T047 第 1 張

## 結論

本張完成影格事件音效接線、S_EFFECT 有限特效載入、GFX 7049 八方向、Source169 移除後播放、立方體圖層與動畫，以及畫面發布時的 HP／MP 快照。凍結 palette 與光術封包接收順序補上回歸驗證。

**GFX 6983 仍受原資源缺幀阻擋**。共享與 TW 動作都引用 subframe 9，現行八方向 SPR 各有 9 幀，索引範圍為 0–8。程式輸出具名診斷，整張驗收仍待這項資源問題解決。

## 已驗證

### 做了什麼

- `native_combat_animation.rs` 與 `native_world_animation.rs` 接入 `<` 的音效選取。既有 `FrameEvent::Effect` 是解析器的歷史名稱，其對應音檔為 `Sound/<id>.wav`。沿用 `combat_animation::classify_native_cues`，一個加權候選群組選取一次，保留重複 ID 的權重與 0 的靜音選取。Hit 保留為呈現事件，既有投射物繼續於攻擊 Hit 入口發射。
- `native_skill_effects.rs` 與 `tower_assets.rs` 增加收到 S37／S53 後才載入的有限特效路徑。限定固定共享 catalog 中可解析的單一 action0、24 FPS、一向或八方向、class0、已支援的混色與事件。原始空白 sentinel 保留停留時間。需要附件或其他控制器的定義會產出資源診斷。新快取上限為 32 個載入結果／錯誤，以及 64 MiB 保留資料。
- `native_world.rs` 保存 S37 接收時的方向，與原本的位置／物件世代一起使用。方向歷史隨 128 筆特效環形紀錄淘汰，換世界時回收。舊程式直接建立、缺少方向紀錄的 Effect 使用方向 0。
- GFX 7049 的八方向 SPR 已實際解碼並繪圖，後續轉向維持原特效方向。`proc_spr` 由伺服器決定是否送出 S_EFFECT，客戶端依收到的 GFX 選擇合格資源。單純收到攻擊動作時，特效紀錄維持空集合。
- Source169 保存接收位置，目標移除與同 ID 重建後仍完成一次播放。其 sound119 改以已接受特效的生命週期交付，目標移除後仍可準備音效。
- `native_cube.rs` 核對四種 GFX 6706／6712／6718／6724 的 action3、attr128、兩個衣物層與陰影。`native_actor_layers.rs` 接納這四個精確定義，`native_world_animation.rs` 播放 action3 循環並產生影格事件。一次延遲抽樣最多補出最近一個循環的事件，避免恢復畫面時連播歷史聲音。
- `FrameVitals` 隨世界渲染影格保存 HP、MP、目標血量文字與物件世代。原生登入視窗從已發布 Frame 取值，封包接收端仍立即保存權威 HP／MP。測試直接驗證攻擊前後 SPR subframe 與像素變化，以及舊畫面的 HP 快照。
- 凍結使用既有 S32 的 `(green=0, gray=1)` 與 RGB555 Dim 路徑，測試核對實際像素。色盤解除與 S25 凍結移動鎖各自處理。
- 光術 fixture 依序送入隱身解除、S20 motion、S37 effect、S4A sound、S1B light、S4D MP，核對接收 revision、動作／特效順序、光源與 MP。

### 怎麼驗證

工作目錄為指定的 `task/184` worktree，基於本次工作開始時的 `origin/main`。執行 Rust 檢查與測試時，`TEMP`／`TMP` 指向 worktree 的 `scratch/tmp`，Cargo 使用 `--locked --offline -j1`、Windows x64 MSVC 與 `target/current`。

測試門檻直接載入 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult`。每個測試指令各自解析一份 harness，檢查結束碼、`running` 數量與 `test result:`，零案、失敗與解析錯誤均判為 FAIL。

以下為最終指令與結果。

所有最終指令的結束碼均為 **0**，門檻均為 **PASS**。測試共 **45 passed、0 failed、0 ignored**。

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

`cargo check` 結束碼 0。編譯器仍列出專案既有 warnings。

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib modern_lito::native_t047_tests -- --include-ignored --nocapture
```

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 2502 filtered out; finished in 13.65s
```

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_combat
```

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s
```

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_ranged
```

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.24s
```

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_actor_states
```

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.54s
```

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_effect_lifecycle
```

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.18s
```

新增 9 案中，3 案需原資源，執行時明確加上 `--include-ignored`。涵蓋下列具名案例。

1. `effect_marker_selects_audio_and_hit_preserves_authoritative_hp`
2. `freeze_palette_uses_wire_flags_and_native_rgb555_pixels`
3. `source169_receipt_survives_remove_and_same_id_replacement`
4. `light_packet_order_and_vitals_are_bound_to_published_animation_frame`
5. `proc_receipts_keep_direction_and_bound_history_without_local_inference`
6. `proc_resource_qualification_rejects_unimplemented_controllers`
7. `original_t047_resource_definitions`
8. `original_kiringku_eight_directions_and_source169_pixels`
9. `original_cube_layers_and_attack_frames_reach_world_renderer`

原資源測試使用 GUI 共用的 Model／Animator／Effects／合成器，產生離線 PNG。已檢視 `cubes-initial.png`、`attack-start.png`、`attack-later.png`、`kiringku-7049.png` 與 `source169-after-remove.png`，攻擊姿勢與特效像素可見。

開發過程曾遇到 Frame 測試建構器缺欄位，以及測試 fixture 的光源 opcode、速度與未支援 modifier 設定錯誤，均已修正後重跑。6983 的資源失敗另以負例保留，測試通過代表診斷符合實際缺幀狀況。

既有 `native_ranged` 首次回歸因新 worktree 缺少 `target/current/verification` 輸出目錄而失敗，結束碼 101。補建該產物目錄後重跑全部相關回歸，首輪記錄保留於 `scratch/native-ranged-missing-output.log`。

```text
test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.69s
```

### 資源核對

- 正式唯讀 JAR 實際 SHA256 為 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。
- 共享 `Text/list.spr` 使用既有 pinned catalog `523c61ad5cc362a56a36f58b262c84efa8b4435f017e7074d35b5834e8b9f127`。
- 本機唯讀檢視 `launcher/TW13081901.pak` 的實際 SHA256 為 `92b512a32349d0775e1036f1832e21f1141ae66e64821d6978f0879246e59ca7`，解碼文字 SHA256 為 `28c4fc600833fb8c609ab195a67ff926e8ffa571100d8d4b389a23be8c2c8db8`。6983／7049 的相關動作定義與共享 catalog 相同，本次新增路徑使用共享 catalog。
- `6983-0.spr` 位於 `Sprite10.pak`，40,057 bytes，SHA256 `fac6a3ed211d2319da23393598914bd6698269ffbb610294f704ea6bb1908ced`。八方向各有 9 個解碼幀，基底 `Sprite.idx` 均缺少同名項目。
- 執行時會留下 `finite_banks.errors["6983"]`，包含 `SPR 6983-0.spr requires subframe 9, decoded frame count is 9`。

## 推測

- 6983 的 catalog／SPR 組合存在版本或資源完整性落差，具體來源仍待資源維護者核對。
- 本次延續現有現代渲染器的影格時鐘與圖層排序。四種立方體的原始資源與層組已驗證，與舊引擎每個 timer callback／遮擋旗標的逐項同源比較仍待後續驗收。
- `<` 加權音效使用 Rust `RandomState` 提供呈現端選取值，原生遊戲 RNG 的逐次序列對照仍待驗證。

## 待驗證

- GFX 6983 的完整可見播放，先取得可對應 subframe 9 的原始資源或具來源依據的 catalog 修正。
- 真正 E149 場次中的奇古獸開關／自訂 SPR、各類 `proc_spr` 生產端與完整光術封包鏈。本次以離線封包 fixture 驗證消費端。
- 實體音效裝置、完整 GUI 操作、長時間效能與使用者真機驗收。
- 複雜 class、附件型／多 action／加權音效的泛用 S_EFFECT 資源，仍由專用 renderer 或明確錯誤診斷處理。

## 產出路徑

- 程式與新增測試位於 `LiTo381-Modern/src/modern_lito/`。
- 本報告為 `docs/reports/issue-184.md`。
- 離線圖像位於 `LiTo381-Modern/target/current/issue-184/`。
- 本機原始輸出位於 `scratch/check-final.log`、`scratch/t047-final.log`、`scratch/native_combat.log`、`scratch/native_ranged.log`、`scratch/native_actor_states.log`、`scratch/native_effect_lifecycle.log`。
- 可重跑的本機輔助程式為 `scratch/run-checks.ps1`，接受 `Check`、`T047`、`Regression`。scratch 與 target 產物保留在此 worktree，Git 提交只包含程式與本報告。

## 給下一棒的提示

1. Claude 審查本張對 Effect 歷史命名的處理、class0 接納條件、方向快照與 HUD 發布邊界。
2. 本機資源執行者接手 6983 的八方向 SPR／catalog 缺幀比對。解決後沿正常 S37 入口重跑八方向與攻擊影格驗證。
3. 調度器依本報告保留 6983 待驗證狀態，再安排正式離線／真機驗收。T047 第 2 張仍由其 Issue 接手。
