# 測試（lito-modern）

所有指令在 `LiTo381-Modern/` 執行，target 為 `x86_64-pc-windows-msvc`（Linux 無法建置：依賴 `windows` crate 與 WinRes）。

## 乾淨機器可跑的部分
```
cargo check --locked --all-targets
cargo test  --locked --lib
```
CI（`.github/workflows/build.yml`，windows-latest）每次 push／PR 都跑以上兩項。

2026-11-04 基線（CI）：`cargo test --lib` 2463 個測試 → **1888 通過、0 失敗、575 ignored**。

## ignored 的三類
| 類別 | 數量 | 原因 | 怎樣才能跑 |
|---|---|---|---|
| 原本就 `#[ignore]` | 488 | 需要視窗、socket、正式 JAR、`LITO_SHARED_RESOURCE_ROOT`（`D:\lineage開發\0-客戶端`）等 | 在使用者機器設定對應環境變數後 `cargo test --lib -- --ignored` |
| 2026-11-04 新增 `#[ignore = "requires: local evidence roots…"]` | 87 | 測試讀 `logs/continuation-20261003/*.json`，快照內記錄的是原機器的絕對路徑（候選目錄 `D:\...`）與 manifest SHA；屬於「證據驗證」而非單元測試 | 只能在原機器；長期應改寫為以 repo 內 fixture 為輸入或刪除 |
| `tests/` 整合測試（`cargo test --tests`） | — | 多數需要 `LITO_MODERN_TEST_JAVA`／`_CLASSPATH`／`_SERVER_ROOT` 指向正式 JAR 與 JDK | 使用者機器設定後執行 |

## 測試輸入檔
`logs/continuation-20261003/` 下 98 個檔（76 JSON + 22 PNG，6.8 MB）以 `.gitignore` 白名單納入 git，其餘 `logs/` 不追蹤。新增測試請把輸入放 `tests/fixtures/`，不要再放 `logs/`。

## 環境變數一覽（測試用）
`LITO_SHARED_RESOURCE_ROOT`、`LITO_EFFECTLIST_REFERENCE`、`LITO_MODERN_TEST_JAVA`、`LITO_MODERN_TEST_JAVA_CLASSPATH`、`LITO_MODERN_TEST_SERVER_ROOT`、`LITO_TELEPORT_SCENARIO`、`LITO_WORLD_MAP_WRITE_REFERENCE`、`LITO_NATIVE_PEER_PROFILE`、`LITO_ARENA_GUI_SUPPORT`、`LITO_MODERN_DISINTEGRATE_CAPTURE_DIR`。

`LITO_CLIENT_ROOT`（#303 起）：`--probe-native-lobby` 系列開發診斷（`native_*_probe`、`維護工具/verify-native-*.ps1`、`tests/native_item_sessions.rs`）需要載入資源時，讀這個環境變數指定的資源根目錄；沒設定時改讀 EXE 旁的 `LiToModern.native.json`，兩者都沒有就拒絕執行。程式內不再寫死 `D:\lineage開發\0-客戶端`。診斷的輸出目錄（`<專案>/target/current/…`）改在執行時由 EXE 所在位置往上找含 `Cargo.toml` 的專案，不再用編譯時的 `CARGO_MANIFEST_DIR`；把 EXE 複製到專案外（例如可攜包）就不能跑這些診斷。
