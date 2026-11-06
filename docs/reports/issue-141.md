# #141 T026（M2 2/2）：玩法支援矩陣

## 做了什麼
- 在 `docs/SKILLS-STATUS.md` 新增一節「玩法支援矩陣：技能以外的玩家可達操作」，共 26 列，8 欄：操作、封包、伺服器權威、呈現、失敗路徑、客戶端現況、狀態、後續。
- 涵蓋本張範圍：
  - 物品使用、裝卸、丟棄、銷毀、拾取；
  - 遠程與彈藥；
  - 耗材與 MP；
  - buff 與 debuff；
  - 復活；
  - #120 新 N2 的隱身裝備例外。
- 另外列出：近身攻擊、空揮、死亡確認、物品目標技能、武器 Proc、Storm_Walk、擂台 NPC 商店。寵物控制只指向 T063，玩家之間的交易標「不適用」。
- 狀態統計：支援 6、部分 7、缺失 8、拒絕 1、待核對 2、不在本張 1、不適用 1。

## 已驗證
- 客戶端封包建構位置：
  - `native_item_actions.rs:85`、`:140`：`C_USE_ITEM 0x1C`；
  - `native_item_actions.rs:147`：`C0C`；
  - `native_ground_pickup.rs:98`：`C0B`；
  - `native_combat.rs`：`0x18` 遠程。
  - `src/modern_lito` 裡沒有 `C76` 的建構，所以銷毀列為缺失。
- 伺服器端的引用都附了檔名與行號：`equipment-toggle-0.6.65.md`、`empty-melee-0.6.49.json`、`native-triple-arrow-policy-progress.zh-TW.md`、`native-invisibility-cast-policy-progress.zh-TW.md`、`gm-travel-0.6.68.md`、`data-sources-0.6.60.json`、`combat-coverage-current.md`。

## 推測
- 「支援」只代表客戶端已實作，並且有 NATIVE 文件的隔離驗證；那些驗證對照的是較早的正式 JAR（例如 `9C97F868`、`58973343`），**還沒有對 E149 重驗**。
- 空揮近戰、隱身裝備例外列為待核對：伺服器端有文件，客戶端是否已經處理沒有確認。

## 待驗證
- 每一列在 E149 上的實際封包與結果，由 T045、T029、T046、T047、T048、T052 與 T043（雙真實玩家）驗證後回填。
- 隱身裝備例外是否在 E149 生效，由 T025 的換版重核清單確認。
