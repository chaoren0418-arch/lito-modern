# #141 T026（M2 2/2）：玩法支援矩陣

基準：分支 `task/141`，從 `main` 的 `dd4fabb4` 分出。第一版是 `0949a69`，本版依 codex-review 退回的 5 點修正。

## 做了什麼
- 在 `docs/SKILLS-STATUS.md` 新增一節「玩法支援矩陣：技能以外的玩家可達操作」，共 27 列，8 欄：操作、封包、伺服器權威、呈現、失敗路徑、客戶端現況、狀態、後續。
- 依審查修正 5 點：
  1. **新增 G05.4 商人附加屬性列**，明寫「依伺服器權威值顯示」。
     - 屬性由 S52 等抗性封包帶來（`native_world.rs:555`，0x50／0x51／0x52／0x56）。
     - 伺服器漏算屬性（29047 火抗正式值是 0、應為 10，`修補提案/商人屬性來源/README.md:10`）是伺服器缺陷：客戶端不補算，修正交伺服器 T040。
  2. **物品使用拆成三列**：
     - 一般使用（use_type 0、1、2、11、18～25）：`use_request`（`native_item_actions.rs:138-143`），格式 `1C d 00`。
     - use_type 5，例如 Storm_Walk 42501：**已實作**。建包在 `target_location_request`（`:80-89`），格式 `1C d d h h`，沒有結尾；目標游標在 `native_login_window.rs:1886-1897`，送出在 `native_login.rs:546-568`。失敗路徑與呈現引用 `gm-travel-0.6.68.md:23-27`。
     - 其他需要目標或選項的分支：客戶端不送，標「拒絕」，交 T045。
  3. **遠程攻擊拆成兩列**：
     - 姿態 20／62 的遠程請求選擇已實作（`native_world.rs:345`），標「支援」。
     - 特殊箭、投擲的 projectile／effect discriminator 尚未實作，標「缺失」。
  4. **不再用破折號**。沒有證據的欄位改寫成以下其中一種：
     - 「待核對：要查什麼（交哪張任務）」；
     - 「尚未實作」；
     - 「不適用」，附理由。
  5. 本報告同步更新統計、已驗證清單與靜態檢查。
- 狀態統計：支援 8、部分 7、缺失 6、拒絕 1、待核對 3、不適用 2，共 27。
- 武器 Proc 列改寫，各欄都寫成明確的值（待核對，交 T047）。Storm_Walk 併入 use_type 5 那一列。寵物控制改標「不適用（不在本張，見 T063）」。

## 已驗證（靜態，commit `0949a69` 的原始碼）
- 物品封包建構：
  - 一般使用：`native_item_actions.rs:138` `use_request`，格式 `1C d 00`。
  - 目標或座標使用：`:83` `target_location_request`，格式 `1C d d h h`，沒有結尾（註解在 `:80-82`）。`:72` 的 `target_request` 會先檢查目標再呼叫它。
  - 一般使用允許的 use_type：`:96` `default_use_type`。
  - 丟棄：`:145` `drop_request`，即 `C0C`；`:130` 檢查數量。
- 拾取：`native_ground_pickup.rs:98` 建 `C0B`。
- 遠程姿態：`native_world.rs:345` `matches!(own.walk_action, 20 | 62)`。
- 屬性封包：`native_world.rs:555` 處理 0x50／0x51／0x52／0x56。
- 銷毀：`src/modern_lito` 裡沒有 C76 的建構。grep 只命中雜湊字串，不是 opcode。
- 文件格式：矩陣 27 列，沒有任何儲存格只填「—」。

使用的檢查指令：

```bash
grep -n "fn " LiTo381-Modern/src/modern_lito/native_item_actions.rs
grep -n "walk_action, 20" LiTo381-Modern/src/modern_lito/native_world.rs
grep -n "0x0b\|0x0B" LiTo381-Modern/src/modern_lito/native_ground_pickup.rs
grep -rn "C76\|0x76" LiTo381-Modern/src/modern_lito
grep -c "| — |" docs/SKILLS-STATUS.md
```

結果：上面的位置全部吻合；最後一條是 0。

## 推測
- 「支援」只代表兩件事：客戶端已實作，NATIVE 文件有隔離驗證。那些驗證對照的是較早的正式 JAR（例如 `9C97F868`、`58973343`），**還沒有對 E149 重驗**。
- Storm_Walk 的伺服器規則取自 0.6.68 文件。E149 是 0.6.71，推測承襲，待 T045 實機確認。

## 待驗證
- 每一列在 E149 上的實際封包與結果。由以下任務驗證後回填：T045、T029、T046、T047、T048、T052，以及 T043（雙真實玩家）。
- 隱身裝備例外是否在 E149 生效，由 T025 的換版重核清單確認。
- 商人附加屬性：E149 的實際數值，以及 GUI 是否照伺服器值顯示（T049、T034）。
