# 原E0277與修正版

原0033e1 actual exit1：Debug library型別編譯拒絕兩處 `isize::from(u16)`，未進行其他17編譯或任何測試。原cohort、候選、日誌與腳本保留，不沿用部分資格。

修正版僅將這兩處改為有界u16 command ID轉Windows x64 isize；原上限61001、不回繞的ID與token／owner／等待規則不變。其他七Runtime差分、新module、15案例來源不變；兩版18metadata需全重編，測試／桌面／播放仍暫緩。
