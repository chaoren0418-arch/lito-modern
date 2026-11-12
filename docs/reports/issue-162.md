# Issue #162 — T024 第 2／2 張：LT hurt／base、action0 倍率、uint32 rollover、hurt queue threshold、特殊變身與中斷分支

本張只處理 T024 第二張。#87 持續 4183 屬第一張（#161），沒有改動。工作分支 `task/162`，沒有修改 `docs/REPORT.md` 和 `TASKS.md`。

## 做了什麼

### 1. 靜態逆向（只讀，沒有執行 PE）
分析 PE 是 `D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe`（SHA256 `4271D92F…AB6E9`，本輪重新算過，相符）。工具是 Python `pefile` 2024.8.26 加 Capstone 5.0.6（`python -I`）。腳本放在 `scratch/t162_dis.py`、`t162_xref.py`、`t162_ltbase.py`，不進 git。

| 項目 | 原指令證據 | 結論 |
|---|---|---|
| LT base `[0x647E90]` | 全 .text 線性掃描後，只有 `43DE5A` 一處寫入。寫入值是 `4D3FF0("list.spr")` 的筆數。4D3FF0 從 PE 內嵌的 10 段資料（指標 `0x62B7F0..0x62B814`，大小 `[0x5091B0]`＝1,189,423）組回 list.spr，用 CRC16（表 `0x636380`）比對 `[0x5091B4]`，再以 `0xF9` 分筆、遇 `0xB3` 結束 | 照原指令重算：CRC `0x48CE` 相符，解析出 **5894** 筆，和宣告值 `[0x62B818]` 相同。組回的 buffer SHA256 是 `A5F7BBFC…ADE9EC7`。base 是從靜態資料推出來的，不是在 1／5894／11297 之間猜一個 |
| LT 相對 gfx（4DF7C0） | 表 `0x636940` 有 9 個 u16：`8,52,53,54,55,110,111,273,277`，加 base 後比對；另外比對常值 `0xA6B／0xA6F／0xA69／0xA6D／0xA72` | gfx `5902,5946–5949,6004,6005,6167,6171`，加上方尖碑 `2665,2667,2669,2671,2674` |
| 原 hurt guard | `4E461A`（4E45F0 起始）、`44ECCE..44ED1A`、`44F258..44F2A4`、`4237BC..423808`、`42F3ED..42F43A` 都是同一個順序 | 依序跳過：死亡（`[+0x10]==8`）、4E8EC0（目前或排隊中的動作是 44／45）、gfx `1245／1590／2001／5620`、4DF7C0、gfx＝`base+26`（5920） |
| 動作中斷分支 | 上面那些位置在通過 guard 後，都呼叫 `4E6B70`（目前動作＝stance 且移動旗標 `+0x13`） | 走路中 → `4E63C0(2)` 排隊；其他狀態 → `4E45F0(2)` 立即取代目前動作，排隊中的命令保留（若 4E6AF0 成立，會先取消 4E34C0／4E9300 timer） |
| hurt queue threshold（4E63C0） | `cmp [+0x74],3; jle`；最後一項 opcode 是 `0x20`，動作相同，而且是 2 | 排隊數**大於 3**，而且最後一項也是 action 2 時，新的 action 2 才會被丟掉；其他情況一律追加（原陣列沒有上限）。`4E85F0` 開始執行時就先 pop 掉，所以排隊數不含正在執行的動作 |
| 特殊變身（落地 hit，40E5A2..40E698） | — | 4 個 boss gfx → 生成 effect `0x4E0`；4DF7C0 → 生成 effect `base+14`（5908）；`base+26` → 生成 effect `base+27`（5921）；其他 → `4E63C0(2)` |
| action0 倍率 `[0x647EA4]` | 只有 `4D42AE` 一處寫入：LTlist.spr 第一行的第二個 43DCC0 整數。唯一讀取點 `43E898..43E8F1`：倍率≠0 且動作屬於 `0,4,11,20,24,40,46,50,54,58,62`（沒有 83）時，`ms*m/100` | 正式 LTlist.spr 第一行只有 `285`。43DCC0 讀到行尾時回傳 0，所以**倍率＝0，不套用** |
| uint32 rollover（4D65F0／4D6570） | `add esi,edi`（環繞加法）；從尾端往前掃，遇到 `jbe` 停（無號、同時限 FIFO）；drain 只取一次 cutoff，`jb` 結束 | 環繞後的 deadline 排在最前面，會**提早觸發**；時鐘環繞前排的 deadline，會**卡住**直到 dword 再長回原值 |

LTlist.spr 的來源：`0-客戶端\Text.pak` 用 PakViewer CLI（`D:\lineage開發\PakViewer-main\publish\win-x64\PakViewer.exe -cli list ...\Text.idx <filter>`）查過，`ltlist`／`LTlist` 是 0 筆，`list.spr` 只有 1 筆（No.15938，4,067,417 bytes，`0x00780010`）。也就是說，LTlist 不在共享 Text.pak，而在 LiTo 專用的 `LTtext.pak`。`D:\380\天堂(Lineage 3.81C) (1)\LiTo\LTtext.idx`／`.pak` 本輪重算的 SHA256 是 `B8C0BB8C…`／`517D853E…`，和 repo 釘選值（`tools/snapshot-r429-native-catalog.ps1`）相同。header 是從已解出的 `lito登入器\舊版分析與驗收\analysis\lttext-extracted-audit-0.7.593\LTlist.spr` 讀的（39,135 bytes，SHA256 `F55779AE…42603`，等於 repo 釘選的 decoded 值）。

### 2. 程式碼
- 新增 `src/modern_lito/native_hurt.rs`：
  - `LT_BASE=5894`、`LT_RELATIVE`、`lt_relative`（4DF7C0）
  - `hurt_exempt`
  - `landed_hit`（40E5A2 的 effect 替代，只做分類，**沒有接 effect 生成**）
  - `queue_suppresses`（4E63C0）
  - `dispatch`（Ignore／Enqueue／Interrupt）
  - `ltlist_action0_multiplier`（4D4275＋43DCC0，含 32 位元環繞、0x200 行長界線）
  - `scale_action0_frame_ms`（43E898）
  - `FORMAL_LTLIST_ACTION0_MULTIPLIER=0`
  - `deadline`／`insertion_index`／`drain`（4D65F0／4D6570 的 u32 語意）
- 修改 `native_combat_animation.rs`（Playback）：
  - 伺服器的 action 2：原本是「前一個 pending 是 2 就丟」（比原版嚴格）。改成原版的 `>3` 門檻，再加上 `hurt_exempt`。開始執行時再檢查一次 guard，避免排隊後變身成豁免 gfx 的情況。
  - `impact()`（effect Hit 到達）：改成原版 guard 順序。走路中排隊（同樣用 >3 門檻）；站立時中斷目前動作（被中斷的攻擊或施法，剩下的影格事件不再進入），hurt 放到佇列最前面，下一次 sample 從 elapsed 0 開始。
  - 新增 `State.walking`：`hold_for_motion` 設為 true，`sample` 設為 false。
  - 保留現代端原有的守衛：guardian、incarnation、16 筆上限、排隊中有死亡就拒絕。
- `tests/native_combat.rs`：移除 burst 案例中兩行 `!impact`。這兩行原本斷言的是舊的現代策略；在原版語意下，站立的身體被 Hit 會被中斷。中斷行為改由新的單元測試覆蓋。
- `mod.rs` 註冊 `native_hurt`。

## 怎麼驗證

| 項目 | 命令 | 結果 |
|---|---|---|
| 編譯 | `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`（在 `LiTo381-Modern` 下） | 結束碼 0，`Finished dev profile [unoptimized + debuginfo] target(s) in 5m 10s`。警告都是既有檔案，`native_hurt*` 和 `native_combat_animation*` 沒有新警告 |
| 靜態 base 重算 | `python -I scratch/t162_ltbase.py <PE>` | `crc_calc=0x48ce crc_want=0x48ce match=True`、`declared_count=5894 parsed_count(base)=5894` |

新增的測試都已編譯通過，但**本機沒有執行**：
- `native_hurt::tests`（8 案）：LT 家族與鄰值、豁免集合、落地 effect 替代、>3 門檻、dispatch 三分支、LTlist header 的倍率（正式 `285`→0、符號、CR／NUL、環繞、0x200 界線）、action0 截斷、u32 環繞的提早觸發與卡住、FIFO。
- `native_combat_animation::tests`（6 案，用合成 catalog）：伺服器 hurt 依門檻排隊，攻擊的 Hit 各進一次；豁免 gfx 不會受擊；排隊後變身成豁免 gfx 時，開始執行就丟掉；站立中斷並跳過剩下的 Hit；走路中排隊並套用門檻，走完後改為中斷；44／死亡／排隊中死亡一律拒絕。

### 第 2 次嘗試：修正 CI 失敗（PR #275）
- CI run `43160549009`（head `470e611`）的 `cargo test (lib, no GUI)` 原文：`test result: FAILED. 2125 passed; 2 failed; 590 ignored; 0 measured; 0 filtered out; finished in 20.09s`。
- 失敗的 2 案是 `server_hurts_queue_until_more_than_three_pending_then_drop_consecutive_action2`（:59）和 `exempt_bodies_never_take_server_or_arrival_hurt`（:98）。兩案都是 `left: "action"`、`right: "attack"`。
- 原因出在測試本身：`ActionPose.phase` 看的是 `cue.source`（`"attack"` 才回 `"attack"`），不是看動作編號。合成 cue 的 source 是 `"action"`，所以 phase 本來就該是 `"action"`。產品邏輯沒有錯。
- 修正：兩處斷言改成 `(pose.action, pose.phase) == (1, "action")`，只改 `native_combat_animation_tests.rs`。
- 修正後本機 `cargo check --all-targets` 結束碼 0。
- **CI 已驗證**：run `43167897400`（head `717de35`）的 `cargo check` job 結果 pass。lib 測試原文是 `test result: ok. 2217 passed; 0 failed; 591 ignored; 0 measured; 0 filtered out; finished in 22.42s`。本張新增的 14 案（`native_hurt::tests` 8 案、`native_combat_animation::tests` 6 案）全部 `... ok`。

## 已驗證
- base＝5894：依 4D3FF0 原指令重算，CRC 相符，筆數和宣告值相同。
- 4DF7C0 的表值、hurt guard 的順序、4E63C0 門檻、4E6B70 走路判斷與中斷分支、40E5A2 的 effect 替代、`[0x647EA4]` 的唯一寫入與讀取點：都是本輪逐條反組譯讀出來的。
- 正式 LTtext 兩檔的 hash 和 repo 釘選值相同；LTlist decoded hash 也和 repo 釘選值相同，header 是 `285`。
- `cargo check --all-targets` 結束碼 0。

## 推測
- 43DCC0 呼叫的 import（`0x773AFFA0`）推定是 `isdigit`：控制流程相符，但 dump 的 import 沒有重建。因此 ≥0x80 的 byte 在本實作中一律報錯。
- 原 actor 的 `+0x14` 推定就是線上 gfx 編號：S_PUT_OBJECT 寫入 `+0x14` 的路徑本輪沒有追。
- 分析 PE 內嵌的 list.spr 推定和正式 `LiTo.bin` 解殼後一致：分析 PE 是 OEP dump，不是正式檔本身。
- 現代 Playback 的佇列只放動作命令。原陣列還有 `0x12` 等其他命令也算在 >3 的筆數裡，所以夾雜移動命令時，門檻生效的時機可能不同。
- 4E8750 在排隊數 ≥2 時會讓影格時間減半（`native_event_scheduler::nonplayer_duration` 已有研究函式），但沒有接進 Playback。純 hurt 爆量時，佇列最多 4＋1 筆；攻擊和 hurt 交錯時，目前靠現代端的 16 筆上限。

## 待驗證／未完成
- **依 2026-11-09 規定，本機沒有執行任何 `cargo test`**。lib 測試只在 CI 跑，結果是 2217 passed／0 failed（見上方「第 2 次嘗試」）。
- `tests/native_combat.rs` 是整合測試，需要 `D:\lineage開發\0-客戶端`，CI 不跑；要交給 Codex 本機執行。修改後要確認 spell Hit 仍是 `expected_hits*2`，最大 pending 推算是 6，不會觸發 16 上限。
- 特殊變身的 effect 生成（`0x4E0`、5908、5921）**沒有接上**：`landed_hit` 只做分類，Playback 對這些 gfx 只是不顯示受擊。
- 伺服器的 S20／S23 hurt 是否也該走中斷分支沒有接：4237xx／42F3xx 是哪個封包的 handler 還沒綁定，所以伺服器 cue 仍然排隊。
- u32 rollover：`native_hurt::deadline/insertion_index/drain` 重現原版語意，但 Playback 仍用 u64 程序時鐘，原版約 49.7 天後的提早觸發與卡住**沒有在 Playback 重現**。要不要統一到 `native_clock` 的 u32，需要審查者裁定。
- action0 倍率：正式資源是 0。如果之後換成有第二個整數的 LTlist，`sprite_actions` 解析還沒接 `scale_action0_frame_ms`。
- #45 class7 child hurt：class7 的程式碼不在 main（只在候選裡）。本張提供的 `dispatch`／`hurt_exempt` 就是原版 44ECCE 的 effect-Hit guard，child Hit 接入時應該直接呼叫它，不要另外推一套。
- 沒有啟動遊戲、連正式服務或 DB，也沒有做 GUI 或實機受擊驗收。

## 產出路徑
- `LiTo381-Modern/src/modern_lito/native_hurt.rs`、`native_hurt_tests.rs`（新增）
- `LiTo381-Modern/src/modern_lito/native_combat_animation.rs`、`native_combat_animation_tests.rs`（新增）
- `LiTo381-Modern/src/modern_lito/mod.rs`、`LiTo381-Modern/tests/native_combat.rs`
- 本報告：`docs/reports/issue-162.md`
- 未追蹤的逆向腳本與輸出：`scratch/t162_*.py`、`scratch/t162-*.txt`

## 給下一棒的提示
- 審查者：請核對 `impact()` 的中斷是否合理——原版 4E45F0(2) 會直接取代施法中的動作，被中斷的施法剩下的 Hit／Sound 不再出現。另外，`tests/native_combat.rs` 那兩行斷言的移除是否可以接受。
- CI／Codex：確認 `native_hurt::tests`（8 passed）和 `native_combat_animation::tests`（6 passed）；本機用 `--exact` 跑 `tests/native_combat.rs::npc_damage_bursts_preserve_spell_events_authority_and_death_preemption` 與 `tests/native_animation.rs` 的 `hurt_queued_after_motion`。
- 後續：(1) 把 `landed_hit` 的 effect 接到世界特效；(2) 綁定 4237xx／42F3xx 對應的封包，再決定伺服器 hurt 要不要中斷；(3) 統一 u32 時鐘；(4) 接 4E8750 的 queue≥2 減半。
