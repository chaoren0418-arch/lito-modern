# 原生風刃7

實作已安裝為revision15。修正載入入口漏接1799後，33項相關Rust測試與wind-cutter／wind-cutter-refused兩個真實雙端情境通過，原始飛行與命中PNG已核對。見[限定證據](../驗證紀錄/current/native-world-integration/wind-cutter-qualified.json)；完整整合、實體GUI和音效裝置仍未完成。

## 正式來源與請求

正式JAR SHA256為BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43。NativeWindCutterOracle直接呼叫BattleSpellCatalog和Protocol取得技能7的action18、gfx1799、MP4、範圍10、重用間隔10ms，以及命中／零傷害／無目標／近距離／無號邊界封包。範圍由正式伺服器决定；客戶端不先扣MP或HP。

技能選單使用已學會ID；有生命、仍有效且在範圍內的目標才可以形成請求，拒絕角色／世界已替換的選取。實際施法仍走現行LoginController待回覆和動作完成閘門。

## 原始動畫差異

1799的image_count雖然是4，action0為Undirected，八個飛行方向應重用同一原始圖像。它有class4→1800次級特效，飛行起始聲音719。1800同樣非方向圖，第一個實際播放來源影格1同時含Sound711和Hit。

原始class4在到達時對合格目標安排一次受傷表現，再建立secondary。唯讀恢復PE核對顯示secondary的目標參照為零，因此這個Hit不能再回頭當成角色的施法／受傷回呼。引擎保留它為可稽核表現事件；HP、MP與傷害只服從正式封包。

來源與限制見 [原始回呼核對](../驗證紀錄/current/native-world-integration/wind-cutter-native-callback.json)。該證據固定恢復PE SHA；並非本輪重新執行舊BIN或完成全部特殊目標分支。

## 必要驗收

- 四项專用測試：正式向量和原子拒絕、原素材時序與兩音效／secondary Hit各一次、八方向一致圖像及目標身份退役、零距離／零傷害／未命中不污染權威。
- 回歸光箭、火箭、技能選擇、法術與戰鬥相關現有測試。
- 兩玩家wind-cutter連續兩次施法，獨立伺服器狀態與實際接收封包核對；wind-cutter-refused使用明確初始MP2，驗正式拒絕和無本地預扣。
- 固定新版本整合，以及後續完整GUI和音效裝置驗收；不使用schema44舊EXE結果宣稱新工作樹通過。

其他法術、完整戰鬥目標、全部資源和整體遊戲尚未完成。
