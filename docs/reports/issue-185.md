# Issue #185 — T047 第 2 張（第 1 次嘗試）

## 結論

本張 7 項逐一核對過，結果如下：

| 項目 | 結果 |
|---|---|
| gfx1544 action0 缺定義、首幀 11／12 | **前一輪已修好**。舊文件（NATIVE-WORLD:24、:34）描述的是首輪狀態。之後的靜態看板路徑已經讓 12／12 都畫出來（有收據）。本張補一個 CI 單元測試把這條路徑固定住，並更新 NATIVE-WORLD:34 |
| 變身 7200 秒還原 | **本張修正**。原生世界原本**沒有處理 S_POLYMORPH（0x27）**，封包進了 `unhandled_opcodes`，所以變身和到期還原在畫面上都不會換外型。現在已接上 |
| 武器族退回 spear24 | **本張修正**。客戶端新增鎖鏈劍族 83／84／86 → 長矛 24／25／27 的退回規則，規則與伺服器相同 |
| 隱身重施不顯形 | 收包狀態正確（有測試）。「同一批封包之間不會畫出中間那一幀」**待驗證** |
| miss／critical | 收包行為正確（有測試）。S_ATTACK 只帶命中標記（0＝miss、10＝命中），**協定裡沒有傷害值，也沒有物理 critical 旗標** |
| 傷害數字 | **沒有實作，原因是缺來源**：協定沒有傷害值，TW 也沒有浮動傷害數字（REUSE-MAP:102、:160）。不能編造數值來顯示 |
| G07.2 原始動作 | 只涵蓋上述動作族與看板。全部玩法的原始動作沒有全面重驗 |

## 已驗證

### 做了什麼

- `native_world.rs`：新增 `0x27` S_POLYMORPH 接收。格式 `dhc`（object、gfx、walk action），依據伺服器 `Protocol.polymorph` 的註解「Exact LiTo 3.81C S_POLYMORPH receiver sub_4A70B0」。
  - 收到後更新 `graphics` 和 `walk_action`。只有值真的變了才增加 `visual_revision`。
  - 新增計數 `polymorph_packets`。
  - 未知物件記為 `orphan_updates`。id=0、截斷、多餘尾端都會拒絕。
  - 客戶端不自己計時：GM `.polyme` 的 7200 秒到期時，伺服器會呼叫 `broadcastPolymorphAppearance("polymorph-expired")`，送出原外型的 S_POLYMORPH（`ClientSession.java:86080-86082`、`GmPolyMe.java:8`）。客戶端照收就會還原。
- `native_world_animation.rs`：新增 `weapon_family_action`。規則是：sprite 沒有 action 83、但有 24 時，83→24、84→25、86→27。sprite 兩族都沒有時保留原值，缺圖錯誤照樣報出來。依據是伺服器 `BattleAnimationCatalog.resolveEquippedWalkAction`，以及 `ClientSession.battleWalkAction` 的註解「The original 3.81 chain-sword path falls back to spear 24/25 when the active sprite does not provide the later 83/84 family」。套用位置：
  - `pose` 的 stance
  - 最終取 clip 的動作
  - `source_motion`
  - `native_combat_animation.rs` 的戰鬥 clip key
  - `native_player_callbacks.rs` 的 `WalkPlan::load`
- `native_t047b_tests.rs`（新檔，已在 `mod.rs` 登記為 `#[cfg(test)]`）：
  - `board_without_action0_is_presented_as_static_prop`：catalog 真的沒有 action0 時，`#1544 1 board a` + type14 仍以 `static_prop` 呈現。看板被要求走路時拒絕；image_count=2 的 sprite 不會走看板路徑。
  - `chain_sword_family_falls_back_to_spear24_only_when_83_absent`：沒有 83 時，待機 27、攻擊 25，而且**攻擊影格依序前進到 0→1→2**，不是只有轉向。有 83 時維持 86。兩族都沒有時保留 83。
  - `polymorph_receipt_applies_and_server_restore_returns_base_body`：涵蓋變身、重複封包、本機計時跑過 7200 秒仍不會自己還原、伺服器還原封包、王族 gfx0、孤兒物件、id=0、截斷、尾端。
  - `invisibility_recast_does_not_leave_hider_visible`：自己這端收到 S_INVISIBLE 0 → S_ACTION 19 → S_INVISIBLE 1 之後，仍是 Half 且選取 token 不變。旁人那端收到 rebuild → action → remove 之後，物件已移除。遠端重複收到隱身只會讓 token 失效一次。
  - `attack_marker_zero_is_miss_and_positive_marker_hurts_target`：標記 0 時攻擊者有攻擊動作、目標不受傷；標記 10 時目標排入 action 2。
  - `original_first_frame_twelve_objects_and_spear_fallback`（`#[ignore]`，需要原資源）：以真實 list.spr 對 map564 首幀的 12 個 gfx 逐一取 pose，並核對基本職業 sprite 的退回規則。
- `docs/NATIVE-WORLD.zh-TW.md:34`：補上 1544 已由靜態看板路徑修好的說明，以及收據位置。

### 12／12 的證據（唯讀查看本機未追蹤收據）

`驗證紀錄/current/native-world/visual-receipt.json`（schema3，2026-10-12）的內容是 `objects_received=12`、`objects_rendered=12`、`missing_graphics={}`、`static_board_graphics=1544`。

`room-world-client.json` 的 `world_render.frame.actors` 共 12 列，全部 `rendered=true`：

| gfx | 解析結果 |
|---|---|
| 61 | →3213 |
| 143 | →54 |
| 2083 | 門 29 |
| 834 | — |
| 460 | — |
| 936 | — |
| 148 | →51 |
| 0 | →3225 |
| 1544 | `static_prop`，`1544-0.spr` |
| 237 | — |
| 727 | — |
| 98 | — |

本輪沒有宣稱任何資源缺失，所以不需要另做 PakViewer 全 pak 查詢。

### 怎麼驗證

```powershell
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
```

- 合併 origin/main（0137ced）**之前**：結束碼 **0**，輸出在 `scratch/check185.log`。
- 合併**之後**：結束碼 **0**，最後一行 `Finished dev profile [unoptimized + debuginfo] target(s) in 2m 28s`，輸出在 `scratch/check185-merged.log`。

改動的檔案沒有新的 warning。`native_player_callbacks.rs:23` 的 `Camera` never constructed 是原本就有的。

**本機沒有執行 `cargo test`**：使用者 2026-11-09 規定，本機不能執行編譯出來的程式，測試二進位也算在內。所以本報告沒有 `test result:` 原文，新測試的結果以 PR 的 CI（`cargo test --lib`）為準。

## 推測

- 原生 sub_4A70B0 收到變身時，可能還會清除或保留動作佇列、重設走路插值。目前只更新 gfx 和動作族，其他交給既有 track signature 處理（gfx 一變，track 就重設）。走路途中變身時，可能會從來源格重播一次走路。
- PET_RACE 也用 S_POLYMORPH（`petRacePolymorph`，例如牛 945 只有走路動作）。現在收到後會真的換外型。如果該 sprite 缺待機 action 3，`world_scene` 會把那個物件記為未呈現並附錯誤，不會整個崩潰。之前是維持舊外型，現在改成明確報錯。
- 隱身重施：伺服器端同技能重施是 toggle-off，換技能時才會 rebuild 再隱身（`ClientSession.java:46048-46100`）。隔離候選 `NativeInvisibilityCastPolicy` 的裝備 20077／120077／20062 例外，E149 是否承襲仍未確認（SKILLS-STATUS:319）。

## 待驗證

- CI：新的 6 項 lib 測試（5 項一般、1 項 ignored）與既有回歸，push 後看 PR。
- Codex 本機：`cargo test --lib modern_lito::native_t047b_tests -- --include-ignored`（ignored 那項需要 `D:\lineage開發\0-客戶端`）。
- 真機或 GUI：
  - `.polyme 6080` 的變身畫面，以及 7200 秒後的還原畫面。
  - 裝備鎖鏈劍的角色用沒有 83 的外型時，攻擊影格是否看得到。
  - 隱身換技能重施時，是否有一幀閃現。這要看同一批封包之間會不會畫出中間狀態。
- 傷害數字：要先確認 LiTo 3.81C 原版畫面到底有沒有浮動傷害數字，例如看脫殼 PE 或實機錄影。如果有，還要找到數值的來源封包。目前協定沒有傷害值，本張不實作。
- 魔法 critical：伺服器的 disintegrate critical（11746／11748）都映射成同一個原生 gfx 1815，客戶端無從區分。是否要有不同的畫面，要等原生證據。

## 產出路徑

- 程式：`LiTo381-Modern/src/modern_lito/{native_world,native_world_animation,native_combat_animation,native_player_callbacks,native_t047b_tests,mod}.rs`
- 文件：`docs/NATIVE-WORLD.zh-TW.md`、`docs/reports/issue-185.md`
- 本機 log：`scratch/check185.log`、`scratch/check185-merged.log`（不進 git）

## 給下一棒的提示

1. 審查重點：`weapon_family_action` 每次姿勢取樣都會呼叫 `resolve`，但只在 83／84／86 才會呼叫，而且 clip 有快取。如果要更省，可以改成直接查 catalog。
2. Codex：執行上面的 `--include-ignored`，並把真機待驗證三項列入 T030 清單。
3. 主控：傷害數字要嘛拆成「原生證據研究」，要嘛從 T047 驗收移除。協定本身不帶數值。
