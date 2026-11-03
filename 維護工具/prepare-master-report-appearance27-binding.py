"""Prepare compact continuation text for the actual World/body binding scope."""
import ast
from native_E149_recovery_contract import R,sha,require
T=R/'維護工具';V=R/'驗證紀錄/current';p=T/'update-master-report-E149-appearance27-queue.py';require(sha(p)==sha(V/'master-report-current-work/20261024-141948-158396/updater.py'),'previous report updater changed');s=p.read_text('utf-8-sig')
def replace(a,b):
 global s
 require(s.count(a)==1,'report binding anchor differs '+a[:90]);s=s.replace(a,b)
replace("ref=w['current_E149_appearance27_queue_development']","ref=w['current_E149_appearance27_binding_development']")
replace("q['request_registered_ids']==[] and q['component_source_only'] and not q['S27_world_or_animator_integrated']","q['request_registered_ids']==[] and q['S27_World_and_Animator_source_integrated'] and q['live_body_callbacks_not_dropped_by_raw_admission']")
replace('EE44A101CB07A2F3C4302288503920FF933FB95623278813B99012BD6A94CA73','FF828A9198F94088146CD919908AF9668E6CFA30E941D67FC23703B05D8EC0A6')
lo=s.index("    latest=f'''");hi=s.index("    require(all(Path(p).is_file()",lo)
s=s[:lo]+'''    latest=f\'''## 已完成的來源開發

**最新S27已串接World／Animator／圖層／History來源候選**：World先核對八byte，再更新同生命權威外觀與target revision；有序Tape保存動作、外觀、移動／預測與轉向。Animator忙於舊cast／walk／held death時保持舊logical gfx／family與motion，idle依收包順序切換；Scene依presented logical gfx綁shadow／附件，pending body操作時不授權新移動／施法frame。不改HP、復活或造cast ACK，LT-relative未綁base便回錯。

同來源History保留自己的released ceiling與後段Hit／音效，不由raw admission跳過未播放操作或刪除回呼。root退役永久標記stored graph，A→B→A及晚到worker只合併admission；新來源以受理floor的權威snapshot重建body並清舊動作。queued S27／轉向沿原tag39／28傳入0x248的分支，以現行整數ms render clock建模584 deadline，late poll不補跑；**真正clock import／動態初始化、逐包到達時刻、全域scheduler交錯、原資源與GUI仍未資格。**

五Runtime差分，新增兩個既有module的overlay，候選39檔／25 Runtime／11 Rust QA／3 Java、368module與原六nested。最終兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新10純＋前312共322案全未跑**，不是完整library inventory。入口仍72／剩108：#67變形術／monlist96選項及#68聖結界請求尚未接。{link('S27串接與待驗界線',T/'native-E149-appearance27-binding.md')}；{link('存活graph回呼floor修正',T/'native-E149-appearance27-binding-callback.md')}。

原首輪Debug library退出1（`{q['observed_original_failure_tool_chunk']}`）：Tape缺PartialEq／Eq，原來源／stderr保留。比較修正版中間18metadata退出0（`{q['intermediate_eq_observed_original_tool_chunk']}`），僅保留其範圍，未移記最終callback差分；最終來源獨立重建與重編譯。{link('原型別失敗與修正版界線',T/'native-E149-appearance27-binding-eq.md')}。正常1049、所有既有PE、正式JAR、registry與G06 binding／640次不變。

前輪#65／66、#62／64、#61復活與#59等來源沿qualification祖源保留；G06四類finite History／root stored退役與音效一次性不變。所有玩法、成本／機率／狀態／自然期限、圖層／像素／音效／owner、G11與GUI仍待當版測試；constructor時鐘偏差、倍率／u32 rollover及LT相對hurt未資格。{link('65／66來源與限制',T/'native-E149-storm65-sleep66.md')}；{link('四類History與限制',T/'native-E149-finite-history.md')}。歷史分支與失敗見{link('此前完整報告與歷史證據',out/'before.md')}。

\'''
    body=body[:start]+latest+body[end:]
    body=body.replace('最新隔離候選37檔：23 Runtime／11 Rust QA／3 Java，368module','最新隔離候選39檔：25 Runtime／11 Rust QA／3 Java，368module').replace('不含最新37檔','不含最新39檔')
    start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
    body=body[:start]+'1. 暫緩期間繼續開發：續接#67完整monlist96選項／C_HACTION與一次性Owner、無控制能力的server隨機路徑，以及#68角色ID／同非空clan資格／兩寶石；入口仍72／剩108。S27原clock import／584排程／逐包到達時刻／全域交錯、圖層／音效／重建／owner／GUI待明確恢復測試後驗；自然7200／32秒不縮時。並查G06物品／source/account／timer前提、hurt原guard與LT gfx初始化，不猜base；完整companion指令／owner及1800／3600秒仍待GAME。\\n'+body[end:]
'''+s[hi:]
out=T/'update-master-report-E149-appearance27-binding.py';require(not out.exists(),'binding report updater exists');ast.parse(s);out.write_text(s,encoding='utf-8',newline='\n');print(out)
