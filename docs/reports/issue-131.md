# #131 M1 完成同步與 M2 開放申請

## 做了什麼
- TASKS.md：T011-H、T011-B 改為 done；M1 標題改為「主線 0 項（完成）」；主線封頂說明改為 11／11 完成。
- PLAN.md：M1 列改為完成；主線分母一節改為 11／11。

## 依據（已驗證）
- #118（PR #129，審查通過）：timeout、refused、wrong-service、cancel-retry、cancel-expiry 各實跑一次，每案 Rust 都是 `running 1 test`、`1 passed; 0 failed; 0 ignored`，0 失敗、0 阻塞、0 未執行。報告是 `docs/reports/issue-118.md`。
- #122（PR #124）：runner 的程序身分修正已審查通過。
- V1：#101、#102 已完成。

## 待使用者裁定
依 #92 凍結規則，M2（57 張）要經使用者同意才能原樣升級成主線。清單見 TASKS.md「預定 M2」。

## 未驗證
- 完整原生 GUI 驗收（T030）照規定仍在 M3。
