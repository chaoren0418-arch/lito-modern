"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_selector80_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['selector80_menu_source_fixed'] and q['shared_selector_only_request_and_menu_classification'] and q['actor_ground_item_summon_mass_guards_preserved'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='35779B01344E4B66B87F938EA1C4D24C1CD90EF1C8E67DCCD647D60E16350FC4','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發與目前界線

**最新 #80 冰雪颶風的選單舊目標來源缺口已修入隔離候選**。原本request要求無target，menu卻漏列80；現在live／stale選取均在resolve前清除。原18個無目標技能集中於native_skills::selector_only，request與menu共用；直接request附帶目標仍拒絕。另2／3／13／60／131自身特殊格式、actor／ground／item／summon／mass、Owner／learned／MP守衛保留。不新增技能、特效、timer或ACK。{link('80來源修補與接續規則',T/'native-E149-selector80.md')}。

下一組97暗隱術／98附加劇毒已有唯讀原始擷取，97共用InvisibilitySpell另補齊；兩批仍待完整review，未接線或資格。2941只有type9無action，不能當普通有限SPR。{link('97／98第一批取證',Path(q['next_primary_source_review'][0]['path']))}；{link('97隱身流程補充',Path(q['next_primary_source_review'][1]['path']))}。

兩Runtime覆寫，仍42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新2純QA＋前453共455案全未跑**，不是完整library inventory。入口86／剩94不增加，續97／98；不代表86技能完整。歷史gap原檔及其source_fix_applied:false保留；本輪resolved_source_issues只表明未選入候選來源已修，正常來源／runtime／GUI尚未套用或驗證。

前輪90堅固防護／91反擊屏障、walk50（54不合格）、實際type3武器／防具／HP100／MP10／64秒由server判定的接線保留。原5831／5832／無聲10710均class0 Screen，Hit只診斷續播；真正反擊由server送防守方S37/10710及攻擊方S20/action2，不造傷害或hurt。{link('90／91來源與待驗界線',T/'native-E149-solid90-counter91.md')}。

88／89及279回饋、87持續4183 Scene／History／typed overlap1453與S08／S0B新生命守衛保留；4183標準S0B空佇列584ms啟動、六幀循環、一般callback+2ms／模型1006ms週期、晚到最低3ms。wire到model、原clock import／初始化／rollover／全域交錯、重送S0B佇列及首圖sub0推論仍未資格。{link('4183來源接線與未驗界線',T/'native-E149-status4183.md')}。

此前75／76、71／74、69 owner傳送／748／八秒deadline、70四節點、67／68及S27呈現沿祖源保留。69同socket／server原子授權、direction initializer、constructor clock skew、LT-relative hurt及當版TCP／原資源／自然期限／GUI皆未驗。正常1049、既有PE、正式JAR、registry及G06 binding／640次不變；未套用／選入、未連結新PE。先前缺口與失敗證據各自保存，詳見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:];start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：續97／98與剩餘94請求，入口86／剩94只是來源涵蓋。selector-only新技能只擴充native_skills::selector_only，request／menu共用，coverage parser須計入此helper。續查4183 wire-to-model／原clock／S0B refresh、G06物品／source/account／timer、hurt原guard及LT gfx初始化。#80 live／stale目標／Owner／learned負向、90／91三有限特效Hit後續播／自然64秒與真正反擊，待明確恢復測試後驗，不把metadata當runtime資格。\n'+body[end:]



 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
