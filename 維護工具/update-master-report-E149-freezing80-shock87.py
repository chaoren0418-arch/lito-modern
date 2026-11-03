"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_freezing80_shock87_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['freezing80_request_and_finite_source_integrated'] and q['shock87_request_and_impact_source_integrated'] and not q['shock87_persistent_object_loop_source_complete'] and q['shock87_status_object_audio_History_pending'] and q['server_only_damage_cost_probability_and_status'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='56D48A9AE7391B08C9BEAF24A4CD34FA4B2BFFD0FC6B97066664102E90C746AD','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發與目前缺口

**最新 #80 冰雪颶風及 #87 衝擊之暈的請求／有限特效已接來源**。80為三byte selector-only `[20,9,7]`、周圍十格／reuse3000；正式S39 action18／type0／3933固定wire caster位置，原start單節點預設路徑。31原幀Screen、source1 overlap1353、無Hit，模型0／5440ms。MP60／傷害／機率／32秒冰凍與名單由server判定，S25／freeze176沿既有權威生命週期。

87為十一byte actorxy `[20,10,6]`、一格／authoritative walk50或54／reuse8000。S23 physical wire與S37/4434先於狀態成功分支，並非眩暈ACK；4434六原幀Screen／source2 Sound1498，模型41／246ms。MP15、傷害／機率／2–4秒皆server權威。**87持續狀態物件4183尚未完成**：正式S0B獨立物件僅action3，現有stationary actor的idle標籤守衛會拒絕animation，重複overlap1453／History亦未接。下一步先綁原構造／循環callback、per-incarnation音效／History與S08退役，不在本機設2–4秒刪除。{link('80／87來源與4183缺口',T/'native-E149-freezing80-shock87.md')}。

五覆寫，仍42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新13（11純／2原資源opt-in）＋前402共415案全未跑**，不是完整library inventory。請求來源入口82／剩98，不代表82技能完整；先補87持續物件，再續88／89。無當版GAME、TCP、像素／聲音、自然期限或GUI通過聲明。

此前75／76精確321／322與3932、71／74、69 owner傳送／748與八秒deadline、70四節點、67／68選單、S27 World／Animator／圖層／History沿祖源保留。69同socket交錯／server原子授權、direction initializer、constructor clock skew、全域scheduler與LT-relative hurt未驗。首次review把stance0解析誤視為無條件，候選寫出前退出1；原失敗與v2真實stationary分支讀回分開保存。正常1049、既有PE、正式JAR、registry及G06 binding／640次不變；候選未套用／選入，未連結新PE。歷史資格見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:];start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：優先補87持續4183原S0B構造／action3循環時序／overlap1453、per-incarnation History與S08退役；不以短暫4434當狀態，不設本機2–4秒刪除。再續88／89與剩餘98請求，入口82／剩98只是來源涵蓋。續查G06物品／source/account／timer、hurt原guard與LT gfx初始化。80自然32秒／87自然眩暈期限、321／322、69完整owner／自然30秒、70 clock skew、原資源／音效／History與GUI待明確恢復測試後驗。\n'+body[end:]



 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
