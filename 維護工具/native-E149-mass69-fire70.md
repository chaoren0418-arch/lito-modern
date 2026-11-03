# #69 協定核心／#70 四節點來源候選

使用者要求測試與桌面操作暫緩。本輪只讀正式JAR／原Text／原PE，寫來源與案例並做metadata編譯。沒有測試本文、--list、GAME、GUI或聲音播放；正常來源、既有PE、registry、相鄰專案不變。

#69 **尚未接控制器或原生GUI，未計入已接入口**。新增`native_mass69`包含S30書籤、目的地Choice、S24/748一次性邀請及S55接續。S30格式是name NUL、x/y/map三個u16；C14/69則為`14 08 04`＋map/x/y，九byte。隨機目的地送目前map／0／0，指定目的地只能取自己收到的正式書籤；不能自行填座標或只做隨機子集。書籤以epoch／source／inventory角色owner保存，50筆上限，完整解析後更新；revision防書籤A→B→A選取復活，自己生命／場景更換使舊Choice失效。

正式邀請是`24 EC 02 00`，接收者不需學會69或有MP；C23回覆是`23 EC 02 yes/no 00`。Owner綁角色生命／連線／場景，序號不回繞，送出同意前消耗重送權限。已同意後只可本機放棄等待，不再送第二次回答。server在施法時建立30秒期限，client從收包計的30秒只是顯示上限，不能證明server仍接受。血盟、距離3、隱身、私店、材料40318×1、MP60與實際目的地均由server核對。

原S55 `55 00 40 ownID` 是明確server傳送接續，不能從S37特效推測。元件只對目前存活自己角色產生一次`4E 00`，之後只受理一個S5A，綁定owner並在新生命／場景／連線退役。C4E的byte由正式parser讀取但不分支授權，零是codec值；**原客戶端回覆時點與其byte來源尚未資格**。一般同生命S27外觀變更不冒充新生命、不能誤殺接續。此元件還沒有掛上TCP控制器；後續須串S55→C4E→S5A→C08／重建／舊輸入退役，並保持既有G06與八秒失敗策略。

#70 已接學會選單與C14三byte `14 08 05`，不能指定actor；reuse2000ms。S39嚴格為action18／gfx1819／type0／空text／精確目標尾端，HP、MP、命中與傷害留server。

**原PE1819命中特例44F4F1，不能套單一clip。** 原4B9390呼叫依序方向3、7、5、1，各距離1；前三個是descriptor-free child，最後改根的位置。依原方向宣告與world X比例，四個offset為`[1,1]、[-1,-1]、[-1,1]、[1,-1]`，不含中心。只有最後根保留完整目標清單，四個Hit不能當四次傷害。新增`native_fire70`保存每節點獨立Playback、callback/audio occurrence與source retirement，接到AttackMany History及既有64 casts／32 cues／512 fragments限額；送整組前先核對投影與queue預算。

1819保留14原幀／Sound710來源幀1／Hit來源幀7／Screen。現行整數ms來源模型從來源幀1起，正常deadline為Hit873ms、結束2369ms；late poll不補跑。四個prepared audio事件遵循原聲音分類及voice抑制，**不代表四次可聽聲音**。direction initializer實際執行、constructor逐節點clock skew、全域scheduler交錯、原資源像素／聲音／GUI與LT-relative hurt仍未資格，沒有造現代hurt替代。

#69的2236七幀／Sound836定義已存在既有finite bank，本輪僅補world actor0的typed audio及原資源opt-in案例來源；不由target離場刪除已接收音效，也不視為傳送成功ACK。

新增24案來源（22純／2原資源opt-in），連前338共362案全未跑；不是完整library inventory。候選42檔／28 Runtime／11 Rust QA／3 Java、371 modules、原六nested保持原路徑。入口75／剩105，#69仍缺整合；下一步先完整接69，再71。初次讀取工具對無BootstrapMethods的class退出1；原default-start假設也被1819特例否決，兩次原失敗／草稿與修正後原證據分開保存，未執行原PE或遊戲方法。
