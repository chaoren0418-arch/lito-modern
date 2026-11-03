"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_hide97_request_venom98_finite_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['hide97_request_and_S34_authority_source_integrated'] and q['venom98_request_and_finite_source_integrated'] and q['actual_timeline_material_supplier_bound'] and not q['hide97_2941_visual_and_retirement_integrated'] and q['hide97_static_frame_zero_and_retirement_unresolved'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='DB2799A0AE0E718C8DAE20980E0C4D8D0DC4CA14342372643CB6C0416531B4F2','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發與目前界線

**最新 #97 暗隱術的請求／S34權威與 #98 附加劇毒的請求／有限特效來源已接並編譯**。均三byte selector-only，reuse9990／0；成本、材料、切換隱身及320秒／中毒由server掌管。NativeSelfCastAction／NativeCastTimeline實際bootstrap材料supplier已核對：preflight→reserve→material→commit，拒絕／例外cancel；是靜態caller鏈，不是whole GAME原子資格。98原2942九幀、source1 Sound1505、有限1496ms與world typed音效沿既有History，不造狀態timer。{link('97請求／98有限特效與未驗界線',T/'native-E149-hide97-request-venom98-finite.md')}。

**97的2941靜態圖呈現與退役尚未接入，不能宣告97完整**。canonical Sprite.idx／1.9GB Sprite.pak已唯讀核對整檔hash與三成員：2941-0.SPR 865 bytes／一幀四refs，1120-0.SPR 211 bytes，2942-0.spr 10474 bytes／九幀，沒有解碼像素或跑probe。原七段完整native已補尾端ret；0 frame-count回source0載靜態圖，44EF80置finished並排0ms／44F330，下一callback退役。4D65F0只入列，4D6570固定now排due；main drain在active函式之前。已tracenetwork與active World；4E2150另有兩次重入488860繪圖，兩分支條件及全域callback交錯仍缺，不假設無像素／固定一幀，不借32秒本機刪除。目前S37/2941仍unsupported／omitted診斷，S34／S08現身由server處理。

四Runtime覆寫，仍42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；前輪草稿4案首次編譯（3純／1原資源opt-in），**＋前455共459案全未跑**，不是完整library inventory。入口88／剩92，只是請求來源涵蓋，不代表88技能完整。目錄下一組99／100，先補97呈現；草稿未編譯收據保留，不借selector80舊18metadata。

#80共用selector_only及目標清除、90／91三有限特效Hit續播、88／89與279回饋、87持續4183 Scene／History／typed音效及S08／S0B守衛沿用。4183 wire到model／原clock初始化／rollover／全域交錯、重送S0B與首圖推論、69同socket／server原子授權、direction initializer、constructor clock skew及LT-relative hurt仍未資格；當版TCP／原資源／自然期限／GUI亦未驗。正常1049、既有PE、正式JAR、registry及G06 binding／640次不變；未套用／選入、未連結新PE。{link('此前完整報告與歷史證據',out/'before.md')}。

取證工具原parser錯誤及首次native range漏尾端各自保留：截短原工具exit0不授予完整函式資格，v2擴正range並逐段核對末端ret；canonical parser只修間隔。沒有以工具失敗重啟或重試遊戲／測試，細節見上方來源說明與raw包。

'''
 body=body[:start]+latest+body[end:];start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：優先補97靜態圖呈現／退役，已trace一般network／active World；優先核對4E2150兩次重入488860繪圖的分支條件、4D6570 due queue交錯與S37先後；不得假設無像素、固定一幀或32秒本機刪除。97完成後再接99／100與剩餘92請求，selector_only共用helper及coverage parser保留。再續4183 clock／S0B、G06物品／source/account／timer與LT hurt/base。明確恢復測試後驗97／98權威S34／S08、低MP／HP／材料、自然32／320秒、原資源／History及GUI，不把metadata當runtime資格。\n'+body[end:]



 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
