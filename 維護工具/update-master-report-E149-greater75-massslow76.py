"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_greater75_massslow76_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['greater75_request_and_owned322_controller_window_integrated'] and q['normal61_exact321_preserved'] and q['massslow76_request_and_S37_overlap_integrated'] and q['server_only_hp_experience_cost_probability_and_status'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='3F6B8243D9015AC3BDC0404A83A80B9318D340C505AEC48C562675114593098D','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發

**最新 #75 終極返生術／#76 集體緩速術已接來源**。75為C14七byte屍體ID、native screen／reuse10；322詢問嚴格四byte `[36,66,1,0]`，普通61的321三byte保留。Token綁實際question、dead-life／owner與不回繞命令ID，C23回原question一次，同意後只本機放棄等待；recipient不需已學75或MP。GUI來源依322顯示正式名稱，沿既有非模態同意拒絕流程。MP100／一寶石、HP／NPC資格／經驗恢復由server掌管；原3944／1362八方向與230／124視覺不是成功ACK。

76為C14十一byte角色xy、十格／reuse3000；MP20／兩寶石／機率／64秒速度狀態皆server權威。正式body僅成功數>0時在原主目標送S37/3932，不造S39多目標名單。3932原13frame、class1／Screen、source2 `<1349`為可重疊音效、無Hit；模型166／2451ms不作狀態期限。已接world typed音效、接收定點與finite History，目標離場不截已接受效果，同來源重建不重送音效。{link('75／76來源接線與待驗界線',T/'native-E149-greater75-massslow76.md')}。

七覆寫（含一controller純QA），仍42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新16（15純／1原資源opt-in）＋前386共402案全未跑**，不是完整library inventory。入口80／剩100，接續80／87。無當版GAME、TCP、像素／聲音、自然期限或GUI通過聲明。

71／74 actorxy與原2232／762呈現、69書籤／隨機與748一次同意拒絕／S55→C4E→owned S5A→C08／八秒deadline、70四節點、67／68原生選單、S27 World／Animator／圖層／History沿祖源保留。69同socket交錯／server原子授權、direction initializer、constructor clock skew、全域scheduler與LT-relative hurt未驗。正常1049、所有既有PE、正式JAR、registry及G06 binding／640次不變；候選未套用／選入，未連結新PE。此前失敗與歷史資格見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:];start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：續接80／87，入口80／剩100；逐一核對正式方法／協定／原資源。續查G06物品／source/account／timer前提、hurt原guard及LT gfx初始化，不猜base。321／322同意拒絕／舊token／新生命、76機率／材料／緩速抵消與自然64秒、71自然16秒、69完整owner傳送／自然30秒、70四節點及clock skew、History／音效／GUI待明確恢復測試後驗。\n'+body[end:]



 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
