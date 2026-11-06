# Issue #193 — T061-2 目標選取政策

基底 `683b603`，工作分支 `task/193`。本張實作客戶端的送出前目標政策、26／42／68 自施分派及技能 68 選取生命週期。伺服器 N1 的材料交易與最終接收者驗證仍列待驗證。

## 做了什麼

- 新增 `native_target_policy`，在控制器既有請求檢查後、LFC1 附加及傳輸排隊前檢查選定玩家。一般 `cast`／`attack`、帶輸入版本及長按普攻共用此入口。
- 依目前 `S_CD` 房間名冊的名稱與 force 對應畫面 actor。mode 1／3／5 依兩隊辨識敵方，對敵方玩家要求當次前景視窗且 Ctrl 按住。mode 2／4 沿用合作模式。自身及友方合法輔助可直接送出。重名、無效 force 或已辨識目標卻缺少自己的隊伍資料時拒絕。
- 身分對應採目前 wire 可取得的房間名冊與唯一 actor 名稱。未列名冊的 actor 及有主人名稱的同伴交由伺服器原目標型別規則裁定。客戶端無權威 PC 型別／來源角色 ID／隊伍物件 ID，因此這個對應的完整性仍需伺服器驗證。
- 26／42／68 明確選自身及未選目標時，都使用自身 object ID，產生七位元組 `14 row column id32-le`。26／42 沿用既有格式，補入 68 的 row 8／column 3、原生畫面射程及 1500ms reuse。26 名稱依文件改為通暢氣脈術，68 加入已學會技能選單。
- 26／42／68 對其他角色增加非空同血盟名稱檢查，自施可獨立分派。血盟名稱只用於客戶端拒絕已知不合法目標，最終資格由伺服器來源角色血盟資料判定。
- 控制器補齊 `world.observer_mode` 與退出等待的拒絕，技能 serializer 也拒絕觀戰。原有死亡、HP／MP、學習清單、控制狀態、可見性、距離及動作等待檢查仍在送包前執行，Ctrl 路徑經過相同檢查。
- 新增 `cast_selected_with_input` 並接入原生視窗，以原 `Target` token 在派送時重驗場景世代、物件 incarnation、target revision 與存活狀態。失效的明確選取回報錯誤。
- 技能 68 排隊成功後保留目標 token，等待期間若施法者或目標退役／換世代，回報 `Interrupted`。場景切換、重新選角及等待結束一併清除 token。客戶端的材料、MP、技能狀態仍由伺服器封包更新。

## 依據

以下伺服器資料皆唯讀，根目錄為 `D:\lineage開發\3-lito開發\lito伺服器\LiTo381-Server`。

- `docs/maintenance/combat-continuation-current.zh-TW.md`、`combat-integration-lifecycle.zh-TW.md` 的指定技能新政策，以及 `native-pair-material-progress.zh-TW.md` 的技能 68 自身／血盟與角色生命週期條件。
- `tools/combat-continuation/integration-lifecycle/NativeSelectedPlayerIntent.java` 的已選玩家意圖規則，以及 `src/main/java/local/lito381/server/{BattleRules,ForcedCombat}.java` 的模式／隊伍及原有伺服器門檻。
- `ClientSession.java` 的 `useAbilityBuffSpell`、`useDamageProtectionSpell` 與 `BattleSpellCatalog.java` 的技能 68 row／column、reuse、原生畫面範圍。
- #120 F43 的 `tools/resource-database/build/de-native-pair-material-development/builds/20261020T080606934231Z/inputs/{build-native-pair-material.py,NativePairCastAction.java}`。builder 第 35 行的 `PairOutput.run` 重驗接收者 binding，材料接點在安裝前重驗 pair。這些檢查位於伺服器角色鎖／材料交易／輸出批次內。
- 本 repo `docs/PROTOCOL-CONTRACT.md` 的 26／42／68 物件請求格式。

## 已驗證

### 建置與測試

執行入口為 `powershell.exe -NoProfile -ExecutionPolicy Bypass -File 維護工具/run-local-test-suite.ps1 -Quick`。沿用腳本的 `running`／`test result:` 門檻，零測試、failed>0、解析失敗均不列通過。Cargo 暫存與產物位於本 worktree 的 `LiTo381-Modern/target/current`。

首輪 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` 結束碼 `0`。首輪 `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast` 結束碼 `101`，Quick 結束碼 `1`。

```text
test result: FAILED. 1980 passed; 1 failed; 577 ignored; 0 measured; 0 filtered out; finished in 9.17s
```

唯一失敗來自新增的 68 pending fixture。沿用的 actor 帶有圖徽 ID，`poll()` 觸發圖徽下載，因該案沒有 transport 而走既有連線失敗清理，清空測試背包。已將 combat fixture 的圖徽 ID 設為零，並增加 `BattlePending` 階段斷言。首輪紀錄位於 `LiTo381-Modern/target/current/verification/local-test-suite/20261106-194908-316-20032/{001.log,002.log,summary.txt}`。

修正後重跑同一 Quick 入口。

- `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` 結束碼 `0`，腳本判定 `PASS`。
- `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast` 結束碼 `0`，腳本判定 `PASS`。
- Quick 整體結束碼 `0`，`running-0-tests = 0`，解析成功。八個新增測試全部顯示 `ok`。577 個 ignored 不計入通過。

```text
running 2559 tests
test result: ok. 1982 passed; 0 failed; 577 ignored; 0 measured; 0 filtered out; finished in 9.63s
```

最終 Quick 紀錄位於 `LiTo381-Modern/target/current/verification/local-test-suite/20261106-195919-537-22104/{001.log,002.log,summary.txt}`。編譯仍有既有 warnings。

另以 `scratch/run-issue-193-regressions.ps1` 執行四組既有回歸，直接載入官方 runner 的 `Invoke-SuiteStep` 與 `Measure-TestResult`，共 17 案通過、0 失敗。四步與 runner 結束碼均為 `0`，各步 `running-0-tests = 0` 且判定 `PASS`。紀錄位於 `LiTo381-Modern/target/current/verification/issue-193-regressions/{001.log,002.log,003.log,004.log,summary.txt}`。

```text
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_skills -- --exact spell_requests_use_formal_selectors_and_never_predict_mana
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_skill_selection -- --skip actual_render_camera_ground_cells_and_object_hits_reject_stale_worlds
exit=0
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_attack_skills -- --exact formal_attack_spell_layout_and_request_guards
exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_t027_batch2
exit=0
test result: ok. 9 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

兩個 ignored 資源案與各組 filtered out 均未執行，也不列通過。選取回歸中的實際資源渲染案明確略過。

`git -c core.whitespace=cr-at-eol diff --check` 通過，保留既有原生視窗檔案的 CRLF。

### 新增測試範圍

八個非 ignored 單元測試位於 `native_target_policy_tests.rs`。

1. mode 0–6、自身／同隊／敵隊、四種前景／Ctrl 組合，逐一核對 1／26／42／68 及普攻。
2. 同血盟仍可為敵隊、隊伍更新立即生效、NPC／同伴原規則及重名拒絕。
3. 26／42／68 未選目標及明確自選的完整七位元組，不按 Ctrl 送出。
4. 觀戰兩種旗標、死亡、目標缺失、不可見、距離、血盟、退出等待、控制限制及送出失敗，驗證失敗時無 pending／冷卻／快照改動。
5. 68 保留選取後目標移除、換 incarnation、死亡、target revision／場景世代更新及隱身，拒絕重新綁定或自施退回。
6. 68 等待期間施法者／目標換世代或退役，遲來的施法動作仍回報中斷，背包及 MP 保持伺服器權威。
7. 真正經 `World::accept` 的 S_REMOVE_OBJECT→同 ID S_PUT_OBJECT，以及 S_ACTION 死亡→S_RESURRECT，舊選取均失效。
8. 自有 `127.0.0.1:0` loopback 經正式 controller、worker、cipher，核對九筆請求的明文、frame 長度及加密計數器。涵蓋三技能兩種自施、友方 68、敵方 68 的 Ctrl 尾碼、敵方長按普攻，並驗證非前景／未按 Ctrl 不入隊、放開 Ctrl 後連發中止。角色資料及回覆節奏使用合成 fixture。

## 推測

- `S_CD` 名稱與目前唯一 actor 名稱相符時，視為同一名房間玩家，依其 force 判斷敵我。這依賴目前封包資料的名稱一致性，現場跨來源角色／名稱變更尚待驗收。
- 七位元組物件請求及 LFC1 尾碼符合已讀伺服器 parser／文件，仍需隔離伺服器確認新政策與效果。

## 待驗證／未驗證什麼

- **N1 伺服器案例仍待執行**。目標在材料扣除／安裝前退役、材料儲存失敗回復，以及延後輸出前 receiver 換角色／場景世代，均需伺服器自有隔離 harness 驗證。Rust 客戶端持有選取／顯示快照，材料交易及輸出接收者 binding 在伺服器 repo，依本張唯讀限制無法在這裡修改。
- **LOS、技能免疫、目標旁觀狀態及權威血盟限制仍需伺服器驗證**。本張保留原封包、原請求門檻與每次 Ctrl 意圖，沒有建立這些未提供給客戶端的權威狀態。含合法 LOS 例外的各技能實際效果仍待隔離測試。
- 原生物件與物品更新封包沒有伺服器角色世代欄位。同連線、同 object ID 換世代後才抵達的封包，須由伺服器在最終送出邊界丟棄。此限制涵蓋 N1 的延後輸出與錯世代扣料保證。
- 技能 68 的完整特效、圖示、buff、材料成本與實際免傷效果，及 26 歷史自施回報，仍需其餘技能任務／隔離伺服器及後續真機驗收。
- 未啟動遊戲或 GUI，未操作既有遊戲程序，未連正式服務或 DB。未執行 Release、ignored、真機 Ctrl／前景操作或完整整合套件，略過項目不計通過。

## 產出路徑

- 新增 `LiTo381-Modern/src/modern_lito/native_target_policy.rs`
- 新增 `LiTo381-Modern/src/modern_lito/native_target_policy_tests.rs`
- 接線與技能分派位於 `LiTo381-Modern/src/modern_lito/{mod.rs,native_forced_combat.rs,native_login.rs,native_login_window.rs,native_skill_selection.rs,native_skills.rs}`
- 本報告 `docs/reports/issue-193.md`

## 給下一棒的提示

- 請 Claude 審查客戶端政策，特別核對房間名冊名稱對應的限制及每技能血盟條件。
- 請調度器安排伺服器 repo 的本機執行者補跑 N1 原角色鎖／材料交易／最終 receiver 案例，並核對 LOS／免疫與目標旁觀拒絕。結果需附實際材料及世代證據。
- 後續合併技能 handler 時，物件型別請求仍須經本張 controller 政策；非物件技能格式須同步核對 `native_forced_combat::object_target`。
- T061 整體完成狀態請待上述伺服器案例與實機驗收裁定後更新。
