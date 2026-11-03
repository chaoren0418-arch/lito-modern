# 原生死亡確認、攻擊等待及結算明細

本輪延伸 [近身戰鬥](NATIVE-COMBAT.zh-TW.md)，以固定正式 JAR `9C97F86821738CD4E039F20BF69CAE286BB1F095BDDA77E3305799A77F5C1A23` 完成正常角色血量的敗北流程。完整遊戲、GUI 及音效裝置仍未驗收。

## 自己死亡與正式確認

正式 ClientSession 在 HP 歸零後依序送出自己的 HP、血量比例、S20/action8 死亡及 opcode-only SE3。伺服器收到 CE0 後才執行 playerDefeated、勝負或重生分支；僅呈現倒地動畫不能完成敗北流程。

World 只接受已有自己死亡動作且權威 HP=0 的 SE3；拒絕多餘尾端、重复要求或相同死亡動作已確認後的重送。世界更換會重置這些狀態。Frame 增加動作完成資訊，依原 list.spr 完整 duration 等待，不能把「已進入最後影格」誤當成「最後影格已播放完」。

LoginController 的 confirm_death 同時核對 BattlePending、當前世界／物件／位置／速度／動作序列及死亡動畫完成，才將 CE0 放入同一有界 transport，並在本地退休該要求。UI timer 與隔離 probe 都使用同一方法；WM_PAINT 不發包。重複呼叫不能再排入確認。其後等待正式結果，並沿用 CD9／SF0／CD0／SD0 返回。

這輪只驗證普通 deathmatch。榮譽模式延遲重生、S_RESURRECT 及其他死亡／復活分支仍未宣告支援。死亡確認的本地計數表示已提交給 transport；實測另由真正伺服器的結算及返回證明傳輸完成。

## 攻擊與目標狀態

控制器提供 idle、pending、acknowledged、no_response、interrupted 五種回饋。一次未回覆攻擊等待期間不排入第二次攻擊或移動；處理已收到的網路事件後，再判斷兩秒期限，避免把同一輪收到的回覆先判為逾時。自己的攻擊封包可解除等待；世界世代變更、目標移除或自己死亡可中止尚未確認的等待。新世界清空舊回饋。

兩秒未收到攻擊回覆只顯示「未收到攻擊回覆，可再試」，不自行認定命中、傷害或某種伺服器拒絕原因。後續新請求重新等待。此協定沒有請求序號，尚未處理所有延遲回覆與重試交錯情形。

目標 HP 比例超過 100 時顯示未知，不顯示 255%；死亡目標另標為已死亡。這是顯示及可操作狀態，不能代替伺服器的隊伍、職業、距離或技能判斷。

## 結算畫面

原生視窗的結算區新增可捲動列表，顯示收到的每筆角色／Bot 名字、分數、擊殺、死亡、助攻及隊伍。依實際字型量測水平捲動範圍；結果階段保留 260 logical pixels 的 HUD，平時為 140，畫面與滑鼠逆轉換共用同一高度。結果階段的方向鍵不再被移動輸入攔截。

此改動已編譯並以正式結果資料驗證模型，沒有將 renderer PNG 當作結算 GUI 截圖。實際版面、跨 DPI、鍵盤焦點及音效裝置仍待人工／可用 UI 工具驗收。

## 真正 ClientSession 情境

`維護工具/verify-native-combat.ps1` 現在執行四個情境，沿用同一正式記憶體 LOGIN／GAME harness，沒有 SQL、正式 Main 或玩家資料。原有單次攻擊及明確 1 HP Bot 勝利案例保留；增加以下兩個。

- `room-world-defeat`：角色正常 50/50 HP、Bot 正常 100 HP，走 11 格到近身範圍，待動作完成才重複攻擊，直到伺服器裁定自己死亡。沒有調低任一方 HP、注入死亡／結果封包或以本地計算扣血。實測 11 次攻擊及 11 次反擊，期間觀察到伺服器自然回血，最終 HP=0、原資源死亡動畫完成、一次 CE0、第二隊獲勝、自己死亡數 1 及返回 LobbyReady。Bot 仍存活。
- `room-world-refused`：走到可見維修 NPC 的鄰格，使用正常 C17 請求。正式伺服器沒有送攻擊回覆；客戶端保留 HP 50/50、attack_packets=0，兩秒後顯示 no_response，再成功提交第二次請求並再次逾時。没有自造拒絕 ACK 或傷害。GUI 自動連打、追擊與完整中斷策略仍待完成。

第一輪敗北已在真正 session 完成，但驗證器將次數寫死為 10，因兩次自然回血而拒絕通過。已保留 `current/native-outcomes/first-defeat-observation.json`，改為核對實際請求與自己的攻擊數一致、反擊數對應、正常起始 HP、最終權威 HP=0、死亡確認及結算返回；同時保留 10–32 次的隔離預算。沒有關掉回血來配合原假設。

Rust 另增加死亡前置條件／重複確認和 pending／回覆／逾時／世界中斷測試。主要執行證據在 `驗證紀錄/current/native-outcomes`、四案例在 `current/native-combat/receipt.json`，最終來源及 EXE 範圍見 `驗證紀錄/20261012/current-build-receipt.json`。

剩餘工作：連續攻擊輸入、取消／切換目標與跨場次回歸、遠程 effect、技能、動態碰撞、其他復活模式、完整 GUI 與長時間可玩性。


最終驗收：同一 Debug x64 EXE 通過 67 Rust、32 TCP、19 張既有像素及離線資源回歸；Release 僅編譯檢查。四戰鬥情境合計 371 幀、81 PNG、96 個不重複影格事件與 8 個 WAV 來源雜湊核對。`維護工具/verify-native-outcomes.ps1` 可重新審核完成的收據、死亡計分、CE0 及聲音事件。

EXE 25,818,624 bytes，SHA256 BE53E6BEED53BA79AAEF0D7BD0303015D0F15972D3E2D38299288C291304E40A；schema13 固定 738 份來源，前份保留 native-combat-build-receipt.json。最後正式 JAR 仍匹配 9C97F868。

後續連續攻擊與同連線跨場次已接入並另行驗證，見 [原生連續攻擊與跨場次](NATIVE-CONTINUOUS.zh-TW.md)。上列數量保留為本階段 schema13 證據。
