"""Update the compact continuation report from the current source-only checkpoint."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib, json, re
from native_E149_recovery_contract import R, SOURCE, FORMAL, read, sha, encoded, require, check_pins

T=R/'維護工具'; V=R/'驗證紀錄/current'; REPORT=R.parent/'LiTo381-總開發報告.md'

def link(label, path):
    path=Path(path); require(path.is_file(), 'missing report reference '+str(path))
    return '['+label+'](<'+path.as_posix()+'>)'

def main():
    w=read(R/'工作佇列.json'); ref=w['current_E149_appearance27_binding_development']
    require(sha(ref['qualification'])==ref['qualification_sha256'], 'qualification changed')
    q=read(ref['qualification']); check_pins(q['source_pins']); check_pins(q['normal_product_pins'])
    require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['S27_World_and_Animator_source_integrated'] and q['live_body_callbacks_not_dropped_by_raw_admission'] and not q['production_applied'], 'source-only scope changed')
    require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'], 'phase changed')
    require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'], 'formal/registry/G06 changed')
    checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json'); after=read(checkpoint)['after_files']
    require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()), 'checkpoint changed')
    c=read(Path(q['compile_run'])/'cohort.json'); inputs=read(c['base_compiled_inputs'])
    require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()), 'normal source changed')
    old=REPORT.read_bytes(); require(sha(REPORT)=='FF828A9198F94088146CD919908AF9668E6CFA30E941D67FC23703B05D8EC0A6', 'master predecessor changed')
    out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'); out.mkdir(parents=True)
    (out/'before.md').write_bytes(old)
    entry=T/'update-master-report-E149-current.py'; (out/'entry.before.py').write_bytes(entry.read_bytes())
    body=old.decode('utf-8-sig'); date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')
    body=re.sub(r'更新：[^（]+（Asia/Taipei）', '更新：'+date+'（Asia/Taipei）', body, count=1)
    for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
        body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)', lambda _:link(label,path), body)
        require(n>=1, 'report reference anchor absent '+label)
    start=body.index('## 已完成的來源開發'); end=body.index('## G06與尚缺前提',start)
    latest=f'''## 已完成的來源開發

**最新S27已串接World／Animator／圖層／History來源候選**：World先核對八byte，再更新同生命權威外觀與target revision；有序Tape保存動作、外觀、移動／預測與轉向。Animator忙於舊cast／walk／held death時保持舊logical gfx／family與motion，idle依收包順序切換；Scene依presented logical gfx綁shadow／附件，pending body操作時不授權新移動／施法frame。不改HP、復活或造cast ACK，LT-relative未綁base便回錯。

同來源History保留自己的released ceiling與後段Hit／音效，不由raw admission跳過未播放操作或刪除回呼。root退役永久標記stored graph，A→B→A及晚到worker只合併admission；新來源以受理floor的權威snapshot重建body並清舊動作。queued S27／轉向沿原tag39／28傳入0x248的分支，以現行整數ms render clock建模584 deadline，late poll不補跑；**真正clock import／動態初始化、逐包到達時刻、全域scheduler交錯、原資源與GUI仍未資格。**

五Runtime差分，新增兩個既有module的overlay，候選39檔／25 Runtime／11 Rust QA／3 Java、368module與原六nested。最終兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新10純＋前312共322案全未跑**，不是完整library inventory。入口仍72／剩108：#67變形術／monlist96選項及#68聖結界請求尚未接。{link('S27串接與待驗界線',T/'native-E149-appearance27-binding.md')}；{link('存活graph回呼floor修正',T/'native-E149-appearance27-binding-callback.md')}。

原首輪Debug library退出1（`{q['observed_original_failure_tool_chunk']}`）：Tape缺PartialEq／Eq，原來源／stderr保留。比較修正版中間18metadata退出0（`{q['intermediate_eq_observed_original_tool_chunk']}`），僅保留其範圍，未移記最終callback差分；最終來源獨立重建與重編譯。{link('原型別失敗與修正版界線',T/'native-E149-appearance27-binding-eq.md')}。正常1049、所有既有PE、正式JAR、registry與G06 binding／640次不變。

前輪#65／66、#62／64、#61復活與#59等來源沿qualification祖源保留；G06四類finite History／root stored退役與音效一次性不變。所有玩法、成本／機率／狀態／自然期限、圖層／像素／音效／owner、G11與GUI仍待當版測試；constructor時鐘偏差、倍率／u32 rollover及LT相對hurt未資格。{link('65／66來源與限制',T/'native-E149-storm65-sleep66.md')}；{link('四類History與限制',T/'native-E149-finite-history.md')}。歷史分支與失敗見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
    body=body[:start]+latest+body[end:]
    body=body.replace('最新隔離候選37檔：23 Runtime／11 Rust QA／3 Java，368module','最新隔離候選39檔：25 Runtime／11 Rust QA／3 Java，368module').replace('不含最新37檔','不含最新39檔')
    start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
    body=body[:start]+'1. 暫緩期間繼續開發：續接#67完整monlist96選項／C_HACTION與一次性Owner、無控制能力的server隨機路徑，以及#68角色ID／同非空clan資格／兩寶石；入口仍72／剩108。S27原clock import／584排程／逐包到達時刻／全域交錯、圖層／音效／重建／owner／GUI待明確恢復測試後驗；自然7200／32秒不縮時。並查G06物品／source/account／timer前提、hurt原guard與LT gfx初始化，不猜base；完整companion指令／owner及1800／3600秒仍待GAME。\n'+body[end:]
    require(all(Path(p).is_file() for p in re.findall(r'\]\(<([^>]+)>\)',body)), 'report reference missing')
    require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()), 'concurrent writer')
    (out/'after.md').write_text(body,encoding='utf-8',newline='\n'); (out/'updater.py').write_bytes(Path(__file__).read_bytes())
    selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n'
    entry.write_text(selected,encoding='utf-8',newline='\n'); (out/'entry.py').write_bytes(entry.read_bytes())
    REPORT.write_bytes((out/'after.md').read_bytes()); require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
    result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False)
    (out/'readback.json').write_bytes(encoded(result)); print(json.dumps(result,ensure_ascii=False))

if __name__=='__main__': main()
