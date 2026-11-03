"""Prepare a compact report that keeps the67/68 integration gap explicit."""
import ast
from native_E149_recovery_contract import R,sha,require
T=R/'維護工具';V=R/'驗證紀錄/current';p=T/'update-master-report-E149-storm65-sleep66.py';require(sha(p)==sha(V/'master-report-current-work/20261024-135318-637860/updater.py'),'previous report helper differs');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'report helper anchor differs '+a[:90]);s=s.replace(a,b)
replace("ref=w['current_E149_storm65_sleep66_development']","ref=w['current_E149_appearance27_queue_development']")
replace("q['request_registered_ids']==[65,66] and q['prior_G06_finite_history_unchanged']","q['request_registered_ids']==[] and q['component_source_only'] and not q['S27_world_or_animator_integrated']")
replace('8A21BC8E3C9C27E8101296BA148407380AF8ECBE6FA7D585B92880E8A9EFB454','EE44A101CB07A2F3C4302288503920FF933FB95623278813B99012BD6A94CA73')
lo=s.index("    latest=f'''");hi=s.index("    require(all(Path(p).is_file()",lo)
s=s[:lo]+'''    latest=f\'''## 已完成的來源開發

**最新S27身體操作佇列元件，尚未串接**：續查#67變形術／#68聖結界，發現目前World未處理八byte S27外觀更新，動作播放器亦缺異質外觀操作。原native4A70B0於body busy經4E6860排tag27，idle才4E0490切換外觀；後續串接須先完成舊動作邊界，保留同poll的動作與外觀順序。

新增native_appearance27元件：嚴格S27、同Owner動作／外觀FIFO、真busy由caller提供，每次明確邊界最多一步；動作取得當時body snapshot。來源退役永久清queue／隱藏body並保留seen，晚到worker只合併admission、不導入舊queue／body；不計時、改HP、造cast ACK或播放。16queue／4096admission滿載回錯，G11未資格；LT-relative不猜base。**World／Animator／controller／GUI尚未串接，#67／#68請求與選單未完成，來源入口仍72／剩108。**

候選新增一個Runtime module，37檔／23 Runtime／11 Rust QA／3 Java、368module，原六nested保留；此前36檔未變。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0。**新7純＋前305共312案全未跑**，不是完整library inventory。正式67／68方法、material owner Code、變身CSV與96個standard_menu選項已唯讀封存；不代表其角色／資源或GUI已驗收。{link('S27元件、原分支與下一步串接',T/'native-E149-appearance27-queue.md')}；{link('tag39後續回呼C15E1132（參數584、時間單位與交錯待綁）',V/'native-E149-appearance27-callback-inspection/20261024-141755-428460/inspection.json')}。

前輪#65雷霆風暴定點／overlap音效與#66沉睡之霧、#62三層／#64、#61復活及#59等來源沿qualification祖源保留；G06四類finite History／root_changed永久退役stored History與音效一次性不變。所有玩法、成本／狀態／自然期限、原資源／像素／音效、G11與GUI仍待當版測試；constructor時鐘偏差／全域交錯、倍率／u32 rollover與LT相對hurt未資格。正常1049、所有既有PE、正式JAR、registry及G06 binding／640次計畫不變。{link('65／66來源與限制',T/'native-E149-storm65-sleep66.md')}；{link('四類History與限制',T/'native-E149-finite-history.md')}；{link('root ABA與原版本界線',T/'native-E149-finite-history-root-aba.md')}。原失敗與歷史分支見{link('此前完整報告與歷史證據',out/'before.md')}。

\'''
    body=body[:start]+latest+body[end:]
    body=body.replace('最新隔離候選36檔：22 Runtime／11 Rust QA／3 Java，367module','最新隔離候選37檔：23 Runtime／11 Rust QA／3 Java，368module').replace('不含最新36檔','不含最新37檔')
    start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
    body=body[:start]+'1. 暫緩期間繼續開發：先把S27元件接World／Animator／History，綁原4E6AF0 busy／idle與原排程邊界，同poll保留舊動作gfx／walk與後續新動作；死亡末姿勢與root退役不得復活舊graph。再接#67完整monlist96選項／C_HACTION與一次性Owner、無控制能力的server隨機路徑，以及#68角色ID／資格／兩寶石，完成108個請求缺口；自然7200／32秒不縮時。G06物品／source/account／timer前提、hurt原guard／LT gfx初始化繼續查，不猜base。完整companion指令／owner與1800／3600秒期限仍待GAME。\\n'+body[end:]
'''+s[hi:]
out=T/'update-master-report-E149-appearance27-queue.py';require(not out.exists(),'report updater exists');ast.parse(s);out.write_text(s,encoding='utf-8',newline='\n');print(out)
