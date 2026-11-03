"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_solid90_counter91_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['solid90_request_and_original_finite_integrated'] and q['counter91_request_and_cast_trigger_finite_integrated'] and q['finite_Hit_diagnostic_only_integrated'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='4624271C3CEA2BB156B604233C017F962435279148C4148D59A9555BDF061A89','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發與目前界線

**最新 #90 堅固防護／#91 反擊屏障已接來源候選**。兩者三byte selector-only C14、選單清除舊目標、reuse各3000ms。91精確walk50（54不合格），實際type3武器、90合格防具與HP100／MP10基值／64秒都由server判定；同步區重核裝備及owner，接受才扣點／啟用能力。既有279「HP 不足」與278辨識保留；不從有限動畫安裝狀態或推算反擊。

原5831十四frame：overlap3973 source1於0ms、Hit10於1494ms／末幀2324ms；5832十七frame：overlap3974 source2於83ms、Hit15於1993ms／末幀2491ms；獨立10710六frame無聲、Hit5於374ms／末幀665ms。均單bank／undirected／class0 Screen、type0／attr8。補僅此三圖Hit的診斷續播，保存Playback／History進度，不改傷害／hurt。真正反擊由server送防守方S37/10710及攻擊方S20/action2，成本／機率／反射與換裝移除皆server權威。{link('90／91來源與待驗界線',T/'native-E149-solid90-counter91.md')}。

四Runtime覆寫，仍42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新12（9純／3原資源opt-in）＋前441共453案全未跑**，不是完整library inventory。入口86／剩94；先修靜態核對發現的#80選單漏清舊目標，再續97／98。此缺口會讓已有角色選取碰到selector-only守衛，未執行request／GUI觀察，尚未修正；不代表86技能完整。

88／89 selector-only、world typed overlap3089／1924、279與權威HP／MP／AC接線保留。87持續4183 Scene／History／typed overlap1453與S08／S0B新生命守衛保留：標準S0B空佇列584ms啟動、六幀1／2／3／4／5／0、一般callback+2ms／模型1006ms週期、晚到最低3ms；wire到model、原clock import／初始化／rollover／全域交錯、重送S0B佇列與初始靜態sub0推論仍未資格。{link('4183來源接線與未驗界線',T/'native-E149-status4183.md')}。

此前80／87、75／76、71／74、69 owner傳送／748／八秒deadline、70四節點、67／68及S27呈現沿祖源保留。69同socket／server原子授權、direction initializer、constructor clock skew、LT-relative hurt及當版TCP／原資源／自然期限／GUI皆未驗。正常1049、既有PE、正式JAR、registry及G06 binding／640次不變；未套用／選入、未連結新PE。先前失敗與修正版證據各自保存，詳見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:];start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：先修#80選單漏清舊目標（已封存來源推導，未執行），再續97／98與剩餘94請求，入口86／剩94只是來源涵蓋。續查4183 wire-to-model、原clock import／初始化／S0B refresh與首圖／全域時序，G06物品／source/account／timer、hurt原guard及LT gfx初始化。90／91防具與實際type3武器／walk50、HP邊界／279／278、三有限特效Hit後續播／音效／History、自然64秒／換裝移除與真正反擊，待明確恢復測試後驗，不把metadata當runtime資格。\n'+body[end:]



 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
