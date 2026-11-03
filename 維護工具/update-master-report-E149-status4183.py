"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_status4183_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['shock87_persistent_object_loop_source_complete'] and q['status4183_typed_overlap_History_and_S08_fenced'] and q['status4183_live_audio_pruned_before_publication_and_after_environment_deferral'] and q['status4183_no_local_expiry'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='FAE4CFF7C0EAFF69738CBC928B774E8F161679D5708B70C4B0BCFEDFD3ABEAB1','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發與目前界線

**最新 #87 持續物件4183已接Scene／History／typed音效來源**。原九段完整函式15496 bytes已唯讀核對；標準S0B／walk0／speed0／brave0／空佇列模型先等584ms啟動，再按source1／2／3／4／5／0循環；一般callback加2ms、末幀0等166ms回佇列，空閒模型1006ms週期，晚到最低3ms下一次callback。原六幀Screen／source1 overlap1453由專用呈現處理；不從有限4434推定狀態，不設本機2–4秒刪除，S25及HP／MP仍為server權威。

同來源History保留cursor／ideal deadline／loop occurrence；source退役留admission，A→B→A及晚到worker不復活舊圖形。S08／新S0B incarnation會退役舊物件；目前frame、worker完成與環境deferred音效都在現行World核對lease，原source1與loop occurrence保留typed provenance。既有可見actor／metadata／audio／fragment／4096音效History上限保留。**仍是條件來源模型**：wire收到到Model admission時間、原clock import／初始化／u32 rollover／全域交錯與已有物件重送S0B的原佇列交錯未資格；初始靜態subframe0為來源推論，待原像素核對。{link('4183來源接線與未驗界線',T/'native-E149-status4183.md')}。

兩Runtime覆寫，仍42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested。最終兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新16（15純／1原資源opt-in）＋前415共431案全未跑**，不是完整library inventory。入口82／剩98未增加，續88／89；不代表82技能完整。首版library metadata0／QA E0425少函式限定路徑（`7b0bd7` exit1）保留，v2只改QA路徑並完整重編，未移記首版成功。

80施法、87請求／4434、75／76精確詢問、71／74、69 owner傳送／748／八秒deadline、70四節點、67／68選單與S27 World／Animator／圖層／History沿祖源保留。69同socket／server原子授權、direction initializer、constructor clock skew、LT-relative hurt及當版TCP／原資源／自然期限／GUI皆未驗。正常1049、既有PE、正式JAR、registry及G06 binding／640次不變；未套用／選入、未連結新PE。此前87缺口、工具判讀錯誤與歷史資格見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:];start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：續88／89與剩餘98請求，入口82／剩98只是來源涵蓋。續查4183 wire-to-model、原clock import／初始化／S0B refresh與首圖／全域時序，G06物品／source/account／timer、hurt原guard及LT gfx初始化。原4183 Screen／overlap1453／584與晚到callback／S08新生命／History、自然期限及GUI待明確恢復測試後驗，不把metadata當runtime資格。\n'+body[end:]



 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
