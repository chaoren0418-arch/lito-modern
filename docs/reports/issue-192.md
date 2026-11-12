# Issue #192 — T061-1：LFC1 Ctrl 強制目標擴充

本張僅實作 T061 第 1／2 張。基底 `c8708c2`，工作分支 `task/192`；不修改目標選取政策、技能 68 生命週期或 `docs/REPORT.md`。

## 做了什麼

- 新增 `native_forced_combat`，在控制器原有攻擊／施法檢查成功後、交給傳輸層之前，附加 `4C 46 43 31 <opcode> 01 67 A5`。不新增持久的強制攻擊旗標。
- 原生視窗每次施法與長按普攻的每次派送都讀取 `GetForegroundWindow`、視窗啟用狀態及 `GetAsyncKeyState(VK_CONTROL)`；只有目前世界視窗在前景且 Ctrl 按住才提供意圖。既有無輸入參數 API 保持一般請求，供離線探針／其他呼叫者使用。
- 狀態限 `BattlePending`、世界就緒、房間 mode 1–5；Pet Race（mode 0）、租裝村、未知或缺失房間、非戰鬥階段、觀戰、退出等待及戰鬥結果均不附加。
- 接受 9-byte 的 `0x17`／`0x18`／`0x7A` 攻擊 body，以及 7／11-byte 的 `0x14` 物件技能 body；物件 ID 必須非零。技能 column、ID 範圍及特殊格式另行檢查，不以長度單獨推定物件。
- 排除無物件目標、座標、傳送、時鐘、召喚／選項等非物件技能格式；已接入的物品附魔 12／21 也不把背包 ID 當作戰鬥目標。原本無目標的短 body 維持不變；若既有 serializer 已把自施轉成自身 object ID，則仍屬物件格式，本張不改自施選取政策。
- 沿用 `native_connection::Session::send_body` 與 `native_wire::frame`：擴充後整個 body 加密一次，再以新 `body.len() + 2` 寫入 little-endian 長度。攻擊 frame 11→19 bytes，物件技能 frame 9→17 或 13→21 bytes。

## 依據（唯讀）

- 伺服器 `lito伺服器/LiTo381-Server/docs/maintenance/forced-combat-0.6.67.md`：前景／Ctrl／戰鬥條件、八位元組尾碼、Pet Race 例外及不放寬其他戰鬥檢查。
- 同目錄 `source-material-control-ingress.zh-TW.md`：decoded request 使用既有 LFC1，非物件格式不可算作 Ctrl 通過。
- 登入器 `lito登入器/LiTo381-Launcher/native/forced_combat.h` 與 `.inc`：核對三種攻擊 opcode、7／11-byte 技能格式、技能排除清單及加密前擴充位置。物品附魔另依本專案 `native_skills::item_targeted` 排除，控制器物品施法仍走一般送包入口。

## 已驗證

執行入口：`powershell.exe -NoProfile -ExecutionPolicy Bypass -File 維護工具/run-local-test-suite.ps1 -Quick`。使用腳本原有門檻解析 `running` 與 `test result:`，零測試、失敗或解析錯誤均不得算通過。Cargo 暫存及產物均置於本 worktree 的 `LiTo381-Modern/target/current`。

### 首輪與修正

- `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`：結束碼 `0`。
- `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast`：結束碼 `101`；Quick 整體結束碼 `1`，不算通過。

```text
test result: FAILED. 1961 passed; 1 failed; 577 ignored; 0 measured; 0 filtered out; finished in 3.42s
```

唯一失敗是本張新增的 loopback 測試，測試在合成回覆後立即連發九筆請求，遇到 `native send queue full or retiring`。已改為每筆等待隔離 peer 完成接收，並排空 worker 事件，再合成下一次回覆狀態；未改正式佇列容量或程式檢查。首輪紀錄：`LiTo381-Modern/target/current/verification/local-test-suite/20261106-183140-573-11144/{001.log,002.log,summary.txt}`。

### 修正後重跑

- `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets`：結束碼 `0`，腳本判定 `PASS`。
- `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast`：結束碼 `0`，腳本判定 `PASS`。
- Quick 整體結束碼 `0`；`running-0-tests = 0`，解析成功。577 個 ignored 不計入通過。

```text
running 2539 tests
test result: ok. 1962 passed; 0 failed; 577 ignored; 0 measured; 0 filtered out; finished in 3.22s
```

新增 8 案均實際執行且顯示 `ok`，涵蓋：

1. 三種攻擊 opcode 與兩種技能長度的精確尾碼／frame 長度，及重複附加拒絕。
2. 前景與 Ctrl 的四種組合。
3. 戰鬥階段、mode 0／1–5／6／未知、缺失房間、觀戰、退出等待及未就緒狀態。
4. 非物件技能排除，包括非零座標／時鐘／選項／物品 ID。
5. 零目標、無目標短 body、異常技能 row／column、異常攻擊長度及其他 opcode。
6. 逐 byte frame 解碼、加密計數器，及強制→一般→非前景→強制請求間無狀態殘留。
7. 控制器保留觀戰、死亡、目標缺失／死亡、距離及送出失敗的拒絕；失敗不建立 pending 或改變狀態。
8. 真正的自有 `127.0.0.1:0` loopback，經 `LoginController`、正式傳輸 worker 與 cipher 逐筆核對長按普攻、放開 Ctrl 後連發、物件技能、一般呼叫、Pet Race 普攻／技能、無目標、座標技能及既有自施物件格式；同時核對加密計數器和長度。測試的角色／回覆狀態為合成 fixture。

最終紀錄：`LiTo381-Modern/target/current/verification/local-test-suite/20261106-184209-327-13468/{001.log,002.log,summary.txt}`（本機產物，不進 git）。`git -c core.whitespace=cr-at-eol diff --check` 通過，保留既有視窗檔案的 CRLF。

## 推測

- 依文件及登入器實作，這些位元組應可供現有伺服器 `ForcedCombat` 解析；本次僅核對格式與客戶端加密流程，沒有向正式伺服器驗證。

## 待驗證／未驗證什麼

- 未啟動遊戲或 GUI、未操作既有遊戲程序、未連正式服務或 DB。實際 Windows 前景切換、Ctrl 按放與雙角色戰鬥效果仍需後續驗收；合成 loopback 不代表伺服器戰鬥規則通過。
- 未執行 Release、ignored 或完整整合套件；略過者不列為通過。
- T061 第 2 張的自身／友方／敵方目標政策、不能繞過的完整伺服器限制、26／42／68 自身分派調整及技能 68 退役／世代／扣料案例均留給下一張，不在本張宣稱完成。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_forced_combat.rs`
- `LiTo381-Modern/src/modern_lito/native_forced_combat_tests.rs`
- `LiTo381-Modern/src/modern_lito/native_forced_combat_controller_tests.rs`
- 接線：`LiTo381-Modern/src/modern_lito/{mod.rs,native_login.rs,native_login_window.rs}`
- 本報告：`docs/reports/issue-192.md`

## 給下一棒的提示

- 請 Claude 審查本張的封包條件與測試證據，再依流程交使用者合併；不要因本張通過就將整個 T061 標為完成。
- 第 2 張可沿用 `attack_with_input`／`cast_with_input`／`continue_attack_with_input` 的每請求輸入，目標政策檢查仍須在送出前完成；不要快取 Ctrl 或將它當成繞過既有檢查的授權。
- 新增特殊技能格式時，請同步確認 LFC1 物件格式辨識，避免把座標、背包物件或時鐘值誤當作角色 ID。
