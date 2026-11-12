# Issue #267 報告 — T900 現成資源對照表（TW BIN 引擎與 LiTo 伺服器）

## 做了什麼
1. 找到兩個參考 repo，全程只讀：
   - tw-bin-engine：`D:\BinEngine`。remote 是 `chaoren0418-arch/tw-bin-engine`；本機 working tree 停在 task/184，落後 origin，所以改讀 `origin/main` `41b0fe2c`（2026-11-11）。
   - lito-launcher-server：`D:\lineage開發\3-lito開發`。本機 main 落後 origin 602 commits，所以改讀 `origin/main` `291474f2`（2026-11-12）。
   - 只用了 `git ls-tree`、`git show`、`git grep`。沒有 checkout、fetch，也沒有寫入對方 repo。
2. 新增 `docs/REUSE-MAP.md`：
   - §0：來源版本與三個必讀差異（opcode 編號不同；TW 沒有登入、大廳、房間、競技場；LS 的 `src/main` 不等於 E149）。
   - §1：通用參考。
   - §2：M2 全部 23 項任務（T025、T026、T020、T021、T027、T022、T024、T028、T029、T023、T041、T043、T044、T045、T046、T047、T048、T050、T054、T055、T061、T062、T063），逐項列出 TW 與 LS 的檔案、類別或方法、行號，並判定「照做／調整／沒有」。
   - §3：彙總。
   - §4：基底評估的結論。
3. 結論：建議**繼續 Rust 重建**，把 TW 當成參考來源，不換基底。依據與各自缺口寫在 REUSE-MAP §4。
4. 規則更新：
   - `CLAUDE.md` 加上第 7 條「實作前先查 `docs/REUSE-MAP.md`；可以唯讀參考兩個 repo，不複製資源、不改對方 repo」，並在檔頭註明 tw-bin-engine 是唯讀參考。`AGENTS.md`、`.cursor/rules/project.mdc` 已逐 byte 同步。
   - `PLAN.md`：改寫「tw-bin 不在範圍內」那句；「與 lito-launcher-server 的關係」一節改成「與參考 repo 的關係」，加上同一條規則與兩個注意事項。

## 怎麼驗證
### 已驗證（親自用 git 指令核對）
- `TW:engine/source/LoginProtocol.cs:7` 的 `JarSha256=d287bc4b…` 存在。
- 下列 TW 檔案在 origin/main 都存在：`SkillCast.cs`、`OnlineMovement.cs`、`Resurrection.cs`、`Pets.cs`、`NativeUi/MacroPanel.cs`、`BuffIconsPanel.cs`、`SelectCharPanel.cs`、`docs/reference/server-protocol.md`、`ui-layout.md`、`skills-visual.md`、`protocol-gaps.md`、`docs/protocol/skills.md`、`status-icons.md`。
- LS `Protocol.java` 的方法與常數都在：`C_MOVE=0x0A`（:118）、`C_USE_SPELL=0x14`（:130）、`C_ATTACK=0x17`（:132）、`showMap`（:12438）、`addressChallenge`（:14483）、`movementLock`（:14517）、`movementLockRelease`（:14524）、`hitRatio`（:16006）、`resurrect`（:16126）。
- LS 的 `ForcedCombat.java`、`NativeSourceIcons1.java`、`EmptyMelee.java`、`BattleCompanion.java`，以及 `docs/design/gameplay.md`、`skill-targeting.md`、`docs/reference/client-protocol-needs.md` 都存在。
- 本 repo 的 Rust 規模：933 個 .rs、16,423,241 bytes。56 個檔案用到 Win32 EDIT／LISTBOX／BUTTON，97 個檔案用到 surf（`git grep -l`）。
- REUSE-MAP 的 `### T0xx` 標題去重後共 23 個，與 M2 的 23 項一致。檔案是 UTF-8 無 BOM。

### 推測（來自唯讀調查摘要，沒有逐行核對的部分）
- 其餘行號和方法名稱來自三個唯讀調查代理的摘要，上面只抽查了關鍵的幾項。例如 `ItemUseArguments.cs` 的 use_type 清單、`Inventory.cs:69-86`、`ObjectRendering.cs:827`、`CS:1737-1899` 都沒有逐一打開核對。
- TW 的 M7 狀態（第十輪 234 已有、K1 55、K5 1；R13-A 已合併，可能可以結案）是讀文件得到的。tw-bin 的 PLAN 還沒更新成結案。
- LS `src/main` 現在能不能啟動，文件裡沒有實測證據。
- REUSE-MAP §4.4 的工作量比較是依證據推估的，沒有實作試算。

### 待驗證
- 兩個客戶端的 list.spr、Tile.pak XML、effectlist2 版本是否相同。TW 預設也讀 `0-客戶端`，但會用 SHA pin 核對，而且部分改讀 kor25。這要用 PakViewer 比對，本任務沒有做。
- use_type 編號、S_HIT_RATIO 0xFF 隱藏、1480ms 窗等數值，要在對應任務裡用 `javap` 對 E149 核對。
- Rust 的 Win32 控件畫面與原版外觀差多少，要等 M3 GUI 驗收。

## 未驗證什麼
- 沒有建置、沒有執行任何程式。這個任務只改文件；CI 會在 push 後跑 `cargo check`。
- 沒有用 PakViewer 查 pak。本任務沒有宣稱任何資源缺失；表中的「沒有」指兩個 repo 裡沒有對應的程式碼或文件，不是指缺資源。

## 產出路徑
- `docs/REUSE-MAP.md`（新增）
- `CLAUDE.md`、`AGENTS.md`、`.cursor/rules/project.mdc`（規則第 7 條）
- `PLAN.md`（範圍說明、參考 repo 一節）
- `docs/reports/issue-267.md`（本檔）

## 給下一棒的提示
- 接 M2 任務時，先看 REUSE-MAP §0.1 的三個差異，再看 §2 對應的那一列。TW 的 opcode 一律不能照抄。
- 讀對方 repo 時一定要讀 `origin/main`：兩個本機 working tree 都落後，直接讀檔會讀到舊版。
- 雲端或 Claude 要把 §4 的結論交給使用者裁定（`needs:user`），包括「把 T043 與流程 smoke 提到技能批次前面」這個排序建議。本任務沒有改 TASKS.md 的排序。
- 不需要拆後續任務，所以沒有產生 `docs/plans/issue-267.json`。
