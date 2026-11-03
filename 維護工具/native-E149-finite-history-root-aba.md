# Source root A→B→A 的立即退役

首候選兩檔僅補通用finite profile的restore／admission／active音效記錄，保留其完整18metadata原結果；不把它當擴充版資格。

進一步靜態核對發現View的root_changed只重置資源／lighting，stored finite_visual_history直到交給worker才取得for_source(false)副本。若B worker未完成，回A仍可能從原stored A History恢復graph。擴充候選在root_changed當下就永久把stored History標為source retired，不等待新worker；同world晚到worker仍只能合併admission，無法恢復退休cursor。

新增一個來源A→B→A、沒有替代worker的純metadata測試來源；Scene的既有QA helper只在cfg(test)內公開供其使用。三Runtime差分、九新案（8純＋1原資源opt-in），加前267共276案全未跑。正常36overlay／22 Runtime／367module及原六nested、技能入口68／剩112與所有正常產物不變。所有兩版18metadata須對這組擴充來源重新編譯；沒有跑測試或GUI。
