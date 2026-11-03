"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_reduction88_bounce89_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['reduction88_request_and_finite_visual_integrated'] and q['bounce89_request_and_finite_visual_integrated'] and q['hp279_rejection_visible'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='EAC21DAB01AEBDC4C82866936D1D47A81F205AF9B2E17A2C4A73AD6C0B3ACBAD','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發與目前界線

**最新 #88 增幅防禦／#89 尖刺盔甲已接來源候選**。兩者三byte selector-only C14；技能選單清除先前角色目標，reuse3000／5000ms。正式HP50／120、MP5基值及192／64秒均由server核對與提交；HP須嚴格大於成本，等於亦回279。已補279辨識與「HP 不足」回饋；實際HP／MP／AC只由權威封包更新，不以S20／S37當狀態ACK。89近戰資格、機率及武器損壞留server。

原4824十九frame／4648二十二frame，單bank／undirected／class0 Screen；source3／4的Effect(3089)／Effect(1924)為可重疊音效，無Hit／子SPR。逐格floor24Hz模型音效82／123ms，有限圖形2366／2281ms，不用有限動畫推定192／64秒。既有收包定點與finite History沿用，typed音效採world actor0域，避免目標離開而截斷。{link('88／89來源與待驗界線',T/'native-E149-reduction88-bounce89.md')}。

五Runtime覆寫，仍42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新10（8純／2原資源opt-in）＋前431共441案全未跑**，不是完整library inventory。入口84／剩96，下一組90／91；不代表84技能完整。

前輪87持續4183 Scene／History／typed overlap1453與S08／S0B新生命守衛保留：標準S0B空佇列584ms啟動、六幀1／2／3／4／5／0、一般callback+2ms／模型1006ms週期、晚到最低3ms。仍是條件來源模型：wire到model、原clock import／初始化／rollover／全域交錯、重送S0B原佇列與初始靜態sub0推論未資格。{link('4183來源接線與未驗界線',T/'native-E149-status4183.md')}。

此前80／87、75／76、71／74、69 owner傳送／748／八秒deadline、70四節點、67／68及S27呈現沿祖源保留。69同socket／server原子授權、direction initializer、constructor clock skew、LT-relative hurt及當版TCP／原資源／自然期限／GUI皆未驗。正常1049、既有PE、正式JAR、registry及G06 binding／640次不變；未套用／選入、未連結新PE。首版4183 QA E0425與修正版18metadata各別保留，詳見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:];start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：續90／91與剩餘96請求，入口84／剩96只是來源涵蓋。續查4183 wire-to-model、原clock import／初始化／S0B refresh與首圖／全域時序，G06物品／source/account／timer、hurt原guard及LT gfx初始化。88／89實際HP邊界與279／278、HP／MP／AC、自然192／64秒、原Screen／overlap／History及GUI待明確恢復測試後驗，不把metadata當runtime資格。\n'+body[end:]



 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
