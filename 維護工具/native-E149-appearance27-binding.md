# S27 World／Animator／圖層／History 串接候選

測試與桌面操作暫緩。本輪只來源及metadata編譯；未套用正常來源、連結或執行PE、GAME／fixture／server／DB、GUI或播放音效。#67變形術選單／請求與#68聖結界請求仍未接，入口72／剩108不變。

World嚴格解析S27八byte：actor u32／graphics u16／walk family u8；先完成封包、revision邊界核對才更新同生命的權威外觀及target revision，不造Actor／HP／施法成功。第一次S27保留原body／motion與已收動作，之後把S20／S23動作、S27、移動／預測及S1C轉向寫入有序數字Tape。128個raw操作滿載記錄被淘汰floor，未觀察的順序缺口回錯，不猜漏掉的操作；S0B新物件、S11新生命與map重建撤銷舊Tape。

Animator以真render epoch／generation／map／actor incarnation及來源revision建立presentation。舊cast／walk／held death仍忙時，保留舊logical gfx／family／motion，不把新圖像套舊clip；idle直接依收包順序接新外觀。queued S27／轉向處理後依原tag39／28傳入0x248的排程分支，沿現行整數ms render clock建模584 deadline；late poll由實際poll時間重排，不補跑多個deferred callback。**原clock import／動態初始化、真正逐封包到達時刻及全域scheduler交錯仍未資格**，不可把模型編譯當原核心timing runtime通過。

每個released動作保存原source／sequence；HP／dead等仍以World權威判斷。呈現Pose帶logical gfx／family／heading，Scene的圖層Plan依presented logical gfx取shadow／附件，避免以新Actor gfx綁舊body frame；pending body queue／deadline未清前不授權新移動／施法frame。LT-relative保留raw，未綁base便回錯，不拿同數字shared SPR代替。

同來源History保存queue／已放行動作ceiling／cursor。普通raw admission不再提高存活body graph的已播放floor，避免skip尚未播放的S27後續動作。root退役會永久標記stored appearance History，A→B→A與晚到worker只合併admission、不導入queue／body；新來源以已受理floor的權威snapshot建立新body並清舊動作。舊生命／context不相容則不復用。16pending／4096live admission、128raw及原pixel／audio上限不是G11最大共存資格。

最初五檔來源候選在編譯前靜態審查發現live History admission問題，保存為precursor；修正版只更正動畫admission、queued hurt2合併與已完成key pruning，並補同來源admission案例，未把首版算作通過。兩個正常module首次加入overlay，隔離候選39檔（25 Runtime／11 Rust QA／3 Java）、368module與原六nested。

新增10純案例來源，加前312共322案全未跑，非完整library inventory。原資源／圖層／音效／owner實際畫面、同poll真實順序、重建／取消／低階timing及完整GUI待明確恢復測試後驗。正常1049輸入、所有既有PE、正式JAR、registry與G06 binding／16案640次計畫不變。
