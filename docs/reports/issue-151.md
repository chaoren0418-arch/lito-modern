# Issue #151 — T022 批 1 技能 101–109

## 做了什麼

本張完成九個技能的請求、選單與控制器接線，交付範圍以 `C_USE_SPELL` 格式、施法前置條件及既有收包模型為主。工作分支為 `task/151`，起點為 `3acbeb1`。

| 技能 | 選擇器 row／column | 封包 body | 本機目標檢查 | reuse 毫秒 |
|---|---|---|---|---|
| 101 行走加速 | 12／4 | 3 bytes | 自施 | 0 |
| 102 燃燒鬥志 | 12／5 | 3 bytes | 自施 | 2990 |
| 103 暗黑盲咒 | 12／6 | 11 bytes，物件 ID＋x＋y | 有效可見目標，三格內 | 990 |
| 104 毒性抵抗 | 12／7 | 3 bytes | 自施 | 0 |
| 105 雙重破壞 | 13／0 | 3 bytes | 自施 | 0 |
| 106 暗影閃避 | 13／1 | 3 bytes | 自施 | 0 |
| 107 暗影之牙 | 13／2 | 7 bytes，背包物品 ID | 明確選取目前角色背包內的物品 | 0 |
| 108 會心一擊 | 13／3 | 11 bytes，物件 ID＋x＋y | 有效可見目標，一格內 | 0 |
| 109 力量提升 | 13／4 | 3 bytes | 自施 | 0 |

- `native_skills::selector_only` 同時供請求分派與選單清除游標目標使用。沿用既有自施技能分類，加入本批六個 selector 技能。
- 107 使用獨立 `ItemTarget`，重用 `native_item_actions::Choice` 與 `native_navigation::InputStamp`。送出前重核連線、角色選擇、物品內容、背包 revision、地圖、世代、角色 incarnation、控制中斷及觀戰狀態。
- 背包顯示期間可選擇技能並按施法。只有實際點選背包列會建立物品目標，定期畫面刷新會檢查並撤銷失效目標。關閉或切換背包也會清除目標。
- `LoginController::cast_item` 共用既有施法 pending、冷卻與畫面 gate。訊息 79 在物品施法回覆中顯示拒絕，161／308 不作為施法成功依據。
- MP、HP、材料、武器適用性與附魔結果交由伺服器裁定。107 採明確背包選取，已裝備武器相關選取政策留給 T049 核對。
- coverage parser 抽出可獨立測試的 `source_ids`，涵蓋多行／緊湊寫法的 `selector_only`、`item_targeted`，並在 helper 語法未知或 selector 未接入 request 時拒絕產生錯誤統計。
- 新增九個 Rust 離線測試並登錄 `維護工具/test-whitelist/tests.tsv`，另有四個 Python parser 回歸測試。測試容許後續批次擴增技能支援。

## 已驗證

### 契約與來源

- 封包形狀依 `docs/PROTOCOL-CONTRACT.md` §6.1 的 101–109 列。
- 射程、reuse 依原專案唯讀 `驗證紀錄/current/native-E149-skill-catalog-audit/20261024-040044-690336/catalog-facts.json` 的九筆 definition。103 使用 E149 catalog 的 `maximumRange=3`，技能狀態表中的 SQL 射程欄不能直接用於此請求檢查。
- 實際重算上述 facts SHA256 為 `1EE2338D209CE41154B42B9B51B2A564261AB07FF1340EC7BCCBB188E216B996`，與相鄰 `inspection.json` 相符。
- 原專案唯讀 `LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar` 的 SHA256 重算為 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。
- `維護工具/templates/native_shadow107.rs` 與物品施法 preparer 僅作設計參考。本次依 main 的 API 接線。

### 離線測試內容

- S1E 收包讀出完整 101–109 技能集合，六種 selector 的精確三位元組封包與過期 hover 清除。
- 103／108 精確十一位元組封包、unsigned 高位物件 ID、送出時最新座標、三格／一格邊界，及已移除、隱身、死亡、替換角色與跨世代目標。
- 九技能逐一重核未學會、MP 0／未知、HP 0、死亡、控制限制及戰鬥結算狀態。請求前後權威 snapshot 相同。
- 107 精確七位元組封包、actor／ground 路徑拒絕、背包 ABA revision、角色切換、物品更新、地圖更換、斷線與觀戰。
- S37 的九個 effect ID 正確進入 model，畸形封包具原子性，跨圖會清除 cue。S62 mode 4、S6B kind 2 與 S6F 物品更新沿用權威接收路徑。
- 訊息 79、278、280、299 的拒絕回饋，161／308、無回覆、S20 施法動作與跨世代中斷。
- 既有技能選單、selector、物品操作與技能收包回歸。

### 命令與結果

所有 Cargo 命令均在 `LiTo381-Modern/` 執行，使用 `--locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current`。下列測試未啟動遊戲程序或伺服器，未連線正式服務或 DB。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_skills_101_109
結束碼 0
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

最終版再依白名單逐案執行九個新測試，並執行相關回歸，共 39 案。每次呼叫均使用 `run-local-test-suite.ps1` 的原始 `Measure-TestResult` 函式判定，16 次 Cargo 呼叫全部 PASS。新測試以 `--test native_skills_101_109 -- --exact <名稱>` 執行，三個完整回歸 target 使用 `--test <target>`，四個既有技能測試使用 `--test native_skills -- --exact <名稱>`。

```text
actor_spells_reject_removed_hidden_dead_and_replaced_targets | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s
actor_spells_use_current_unsigned_target_coordinates_and_formal_range | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s
every_skill_rechecks_learning_life_mana_and_control_without_prediction | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.04s
item_rejection_is_scoped_and_cast_feedback_never_infers_success | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s
menu_exposes_this_batch_and_retains_existing_selector_behavior | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s
received_effect_speed_strength_and_inventory_updates_remain_authoritative | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s
selectors_clear_stale_hover_and_emit_exact_three_bytes_without_spending | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s
shadow_fang_revokes_inventory_aba_scene_owner_and_observer_changes | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s
shadow_fang_uses_only_explicit_inventory_intent_and_preserves_authority | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s
native_skill_selection | exit=0 | PASS
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.29s
native_selector_only | exit=0 | PASS
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
native_item_actions | exit=0 | PASS
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
current_book_is_atomic_add_remove_and_survives_map_but_not_new_world_owner | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
spell_requests_use_formal_selectors_and_never_predict_mana | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
effects_preserve_formal_unsigned_ids_and_are_bounded_atomic_and_map_scoped | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
skill_feedback_requires_authority_and_expires_without_guessing_success | exit=0 | PASS
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
```

正式 Quick 套件也已執行完成，使用 PowerShell 7.6.5。

```text
pwsh -NoProfile -ExecutionPolicy Bypass -File 維護工具/run-local-test-suite.ps1 -Quick
結束碼 0
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast
結束碼 0
test result: ok. 1927 passed; 0 failed; 575 ignored; 0 measured; 0 filtered out; finished in 3.29s
```

Quick 原始日誌位於 `LiTo381-Modern/target/current/verification/local-test-suite/20261106-151143-691-1256/`。575 個 ignored 案保留為未驗證。建置有既存警告，驗證結論以程序結束碼與測試摘要為準。

Python parser 回歸命令為 `python -B 維護工具/test_native_skill_coverage.py`，使用本機已安裝的 Python runtime，結束碼 0，`Ran 4 tests` 與 `OK`。parser 靜態識別本版合計 47 個請求 ID，其中本批新增九個，107 歸於 inventory 類別。

Windows PowerShell 5 的暫存 gate wrapper 曾在執行政策／UTF-8 路徑解析階段停止，當時未執行 Cargo。後續使用 PowerShell 7 執行並完成上述 gate，未將先前啟動失敗計入通過。


## 推測

離線封包與既有 model 測試支持本批請求可由 E149 對應 handler 解析。實際技能成功率、材料消耗、HP／MP 結算與目標政策仍需經正式驗收流程確認。

## 待驗證與未驗證什麼

- 未執行正式伺服器施法、原生 GUI 操作、Ctrl 同隊目標、材料不足的真實端到端流程及持續時間自然到期。
- 本張沒有新增九技能的特效／音效資源資格與渲染接線。S37 model 測試僅涵蓋收包 cue，完整動畫、像素、音效及 106 的 TW 11765 命名空間仍待接線與驗收。
- 107 的已裝備武器選取政策交由 T049，材料處理器差異依 `docs/SKILLS-STATUS.md` 的伺服器待定項追蹤。
- 未執行完整 Full 套件、Release、其他 ignored／整合 harness 與 T030 GUI 驗收。略過項目不列入通過數。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`
- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`
- `LiTo381-Modern/tests/native_skills_101_109.rs`
- `維護工具/audit-native-E149-skill-catalog.py`
- `維護工具/test_native_skill_coverage.py`
- `維護工具/test-whitelist/tests.tsv`
- 本報告 `docs/reports/issue-151.md`
- 本機原始日誌在 `LiTo381-Modern/target/current/verification/issue-151/` 與 Quick 套件輸出目錄，均為未追蹤產物。

## 給下一棒的提示

- Claude 審查本批請求、背包施法 UI 路徑與驗證範圍，使用者依既定流程合併。
- 下一批 110–118 沿用 `selector_only`，僅加入該批 ID。113 的 MS950 尾段與後續 21 個 wrapper 的十一位元組形狀依各張 Issue 實作。
- 後續物品技能可重用 `ItemTarget` 與 `cast_item`。延伸 helper 時同步執行 parser 回歸測試。
- 調度器應保留上述特效／音效接線及真機驗收待辦，請勿把本張請求測試記成九技能完整玩法驗收。
