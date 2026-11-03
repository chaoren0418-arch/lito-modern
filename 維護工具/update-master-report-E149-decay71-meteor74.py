"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_decay71_meteor74_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['decay71_request_and_finite_visual_integrated'] and q['meteor74_request_and_class1_first_target_integrated'] and q['server_only_cost_probability_and_status'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='18B50B5B8F25B488070E78C165FE0E84D76B7880FDF58B208552B07D4841A320','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發

**最新 #71 藥水霜化術／#74 流星雨已接來源**。兩者皆十一byte C14角色ID／x/y，限其他可見活體；71三格、reuse／client clock500ms，74十格、reuse3000ms。學習清單與選取生命／revision守衛沿既有入口；不從無目標或過期目標改送自己。

71正式material commitPair與成功狀態分支已核對，MP80／兩寶石／機率／16秒皆由server判定，S37接受後才播放原2232：21frame、Screen、Sound838 source1，模型1952ms不作狀態期限。74正式S39 action18／type8／762依原class1固定第一目標收包快照，即使miss仍取第一個；缺首目標或空清單沿wire caster origin＋heading一步，不取第二個。原20frame、Screen、Sound130 source1／Hit4，模型123／2164ms；Hit只診斷。原start是預設路徑，不套70四節點特例。兩者typed world音效與History／重備失敗一次退役已接。{link('71／74來源接線與待驗界線',T/'native-E149-decay71-meteor74.md')}。

五檔差分，仍42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新14（12純／2原資源opt-in）＋前372共386案全未跑**，不是完整library inventory。入口78／剩102，接續75／76。無當版GAME、TCP、像素／聲音、自然期限或GUI通過聲明。

69書籤／隨機、748一次性同意拒絕、S55→C4E→owned S5A→C08及八秒deadline來源已接；recipient不需已學69或MP，舊通知／owner失效不授權新Choice或ordinary admission。同socket交錯與server原子授權待GAME。70四節點森林、67／68原生選單、S27 World／Animator／圖層／History沿祖源保留。direction initializer、constructor clock skew、全域scheduler與LT-relative hurt未驗。正常1049、所有既有PE、正式JAR、registry及G06 binding／640次不變；候選未套用／選入，未連結新PE。此前失敗、修正前後編譯範圍與歷史資格見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:];start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：續接75／76，入口78／剩102；逐一核對正式方法／協定／原資源。續查G06物品／source/account／timer前提、hurt原guard及LT gfx初始化，不猜base。71機率／材料／自然16秒、74第一目標／原資源／Hit、69完整owner傳送與自然30秒、70四節點及clock skew、History／音效／GUI皆待恢復測試後驗。\n'+body[end:]



 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
