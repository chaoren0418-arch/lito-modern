# Issue #14 / T004-2 — Rust 集中 opcode 常數表

## 做了什麼

- 新建 `LiTo381-Modern/src/modern_lito/native_opcodes.rs`：依 `docs/PROTOCOL-CONTRACT.md` §2–§5 建立 39 個 `pub const`（`C_*` 16 個、`S_*` 23 個，含 §2 的 `S_KEY`）。名稱沿用伺服器 `Protocol.java`。C／S 方向即使數值相同也各自宣告（例如 `C_LITO_LIN_CHARACTER_INFO` 與 `S_LITO_SELECT_CHAR_RESULT` 都是 `0xC8`）。
- `mod.rs` 只加 `pub mod native_opcodes;`。
- 四個指定檔只把 opcode 字面值換成常數，不改行為、不改 wire 格式：
  - `native_version.rs`：刪除本地 `C_VERSION`／`S_KEY`／`S_VERSION` 定義，改 `pub use` 常數表；`S_VERSION` 仍為公開別名，指向 `S_VERSION_CHECK`。
  - `native_login_wire.rs`：`body.push(1)` → `C_LOGIN`；version fixture 第一 byte 明確寫 `C_VERSION`。
  - `native_game.rs`：收包／選角／通知 ACK 的 opcode 字面值改為對應 `S_*`／`C_*`。
  - `native_lobby.rs`：進頻、頻道資訊、房間列、進頻請求的 opcode 字面值改為對應常數。

### 替換的字面值

| 檔案 | 原字面值 | 常數 |
|---|---|---|
| `native_version.rs` | `0`／`1`／`0x3a`（本地 `pub const`） | `C_VERSION`／`S_KEY`／`S_VERSION_CHECK as S_VERSION` |
| `native_login_wire.rs` | `1`（C_LOGIN）；fixture body 第一 byte 隱含 `0` | `C_LOGIN`／`C_VERSION` |
| `native_game.rs` | `0x4b` | `S_NOTICE` |
| `native_game.rs` | `0xe6` | `S_LITO_NEWS` |
| `native_game.rs` | `0x38` | `S_NEWS` |
| `native_game.rs` | `3` | `S_NUM_CHARACTER` |
| `native_game.rs` | `0xcb` | `S_LITO_CHARACTER_INFO` |
| `native_game.rs` | `0xc8` | `S_LITO_SELECT_CHAR_RESULT` |
| `native_game.rs` | `0xc9` | `C_LITO_SELECT_CHAR` |
| `native_game.rs` | `0x61` | `C_LOGIN_RESULT` |
| `native_game.rs` | `0x46` | `C_READ_NOTICE` |
| `native_lobby.rs` | `0xc9` | `S_LITO_ENTER_CHANNEL_RESULT` |
| `native_lobby.rs` | `0xca`（parse） | `S_LITO_CHANNEL_INFO` |
| `native_lobby.rs` | `0xcc` | `S_LITO_ROOM_INFO` |
| `native_lobby.rs` | `0xca`（`enter`） | `C_LITO_ENTER_CHANNEL` |

未替換（不在 §2–§5 契約表，或不是 opcode）：

- `native_game.rs` 經典進場 `5`／`7`（不在 §2–§5）。
- `native_lobby.rs` `0xf5`（`S_LITO_VS_SERVER_ALLOW_JOIN`，契約 §7）。
- `VERSION_MAGIC`／`0x8033`、NUL 結尾 `0`、結果碼 `0|1|7|8` 等欄位值。

§6／§7 戰鬥與退出 opcode（`C_MOVE`、`C_USE_SPELL`、`S_WORLD` 等）未列入本表。`native_login.rs` 仍有大量 opcode 字面值，不在本任務範圍。

## 怎麼驗證

### 已驗證

- 靜態對照契約 §2–§5 與 Issue #13 正式 JAR 核對表：39 個常數名稱與十六進位值一致。
- 單元測試 `opcode_constants_match_protocol_contract_sections_2_to_5` 逐項 `assert_eq!` 契約值。Linux 無法建置完整 crate（依賴 `windows` crate），改以獨立編譯該模組：

  ```bash
  rustc --edition 2021 --crate-type lib --test native_opcodes.rs
  ./opcode_tests
  ```

  實際輸出：`test tests::opcode_constants_match_protocol_contract_sections_2_to_5 ... ok`（1 passed）。
- 靜態 diff 確認四個檔只換 opcode 常數、沒有改欄位解析或組包順序。
- `native_opcodes.rs`、`native_version.rs` 通過 `rustfmt --edition 2021 --check`。
- GitHub Actions [run 41889440513](https://github.com/chaoren0418-arch/lito-modern/actions/runs/41889440513)（程式碼 head `ee1b196`，windows-latest；其後若僅修訂本報告，以 PR 最新 Checks 為準）：
  - `cargo check --locked --all-targets`：success。
  - `cargo test --locked --lib --no-fail-fast`：success；**1889 passed、0 failed、575 ignored**（較基線 1888 多 1，即新增的 opcode 測試）。
  - 日誌含 `test modern_lito::native_opcodes::tests::opcode_constants_match_protocol_contract_sections_2_to_5 ... ok`。

## 未驗證什麼

- 未跑 GUI、未啟動遊戲、未連正式服務或 DB。
- 未跑 `tests/` 整合測試、Java oracle、fixture 往返。
- 未把 `native_login.rs` 或其他模組的 opcode 字面值納入常數表。
- 未證明執行時封包與改前 bit-identical（常數值與契約相同，屬靜態等價，不是動態抓包）。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_opcodes.rs`（新建）
- `LiTo381-Modern/src/modern_lito/mod.rs`（只加 mod）
- `LiTo381-Modern/src/modern_lito/native_version.rs`
- `LiTo381-Modern/src/modern_lito/native_login_wire.rs`
- `LiTo381-Modern/src/modern_lito/native_game.rs`
- `LiTo381-Modern/src/modern_lito/native_lobby.rs`
- `docs/reports/issue-14.md`（本報告）
- 未改 `docs/REPORT.md`、`docs/PROTOCOL-CONTRACT.md`。

## 給下一棒的提示

- 請 Claude 審查 PR #29：確認 C／S 未共用常數、四個檔沒有行為變更。CI 已綠（run 41889440513）。通過後由使用者合併。
- 後續若要繼續去魔術數字：優先 `native_login.rs`（LOGIN 收包與大廳／戰鬥送包），再補 §6／§7 opcode 到同一張表。`native_lobby.rs` 的 `0xf5` 可在補 §7 時改成 `S_LITO_VS_SERVER_ALLOW_JOIN`。
- `native_version::S_VERSION` 是公開別名，外部測試若仍用此名不必改。
- 本任務不另拆 plan。調度器合併後自行加 `docs/REPORT.md` 索引。
