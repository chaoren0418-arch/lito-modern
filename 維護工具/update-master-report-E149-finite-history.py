"""Bind the compact report to finite source retirement, preserving the prior report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,p):
 p=Path(p);require(p.is_file(),'report reference absent '+str(p));return '['+label+'](<'+p.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_finite_history_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'latest qualification differs');q=read(ref['qualification']);check_pins(q['source_pins']);check_pins(q['normal_product_pins']);require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['root_retirement_sticky_before_worker'] and not q['production_applied'],'finite source scope differs');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase differs');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 differs')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');before=read(checkpoint)['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint differs');c=read(Path(q['compile_run'])/'cohort.json');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source differs');old=REPORT.read_bytes();require(sha(REPORT)=='B6C341EF49E2EF9E7484615C83D539118C609B9EAA67AE1D083E7529668B790E','master predecessor differs');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,p in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,p),body);require(n>=1,'report reference anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發

**最新G06 finite History來源退役候選**：原View會隔離不同資源root的graph並合併晚到worker admission，但四類Skills／Items／Reagent／Travel在來源false時整份History被略過，晚到合併與活動音效序號亦缺。新來源同源恢復cursor，來源退役清active但保留generation／seen／計數，同代晚到worker只合併已受理序號、不導入退休cursor／音效／資源；四類活動序號納入原音效history保留，raw ring淘汰不提早釋key。

進一步修正root A→B→A：View在root_changed當下就把stored History永久標為來源退役，不等B worker完成；回A不能重新授權舊graph，保留admission供後續去重。原generation／connection／map／publication fence及64active／32音效／512fragments／4096keys上限不改，最大共存／G11未驗。

三Runtime差分，36檔／22 Runtime／367module與原六nested不變。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；原兩檔18metadata（`{q['original_metadata_observed_tool_chunk']}` actual exit0）僅保留首候選範圍，未移記擴充版。**新9（8純＋1原資源opt-in）＋前267共276案全未跑**，不是完整library inventory；請求入口仍68／剩112，未新增／驗收#62／#64。{link('四類History與限制',T/'native-E149-finite-history.md')}；{link('root ABA與原版本界線',T/'native-E149-finite-history-root-aba.md')}。

#61復活的屍體意圖／question321／同意、拒絕與放棄等待、3944／230跨S11光效接續沿前輪候選；#59暴風雪25節點、#55／56、#51、#50及S23等沿qualification祖源保留。所有玩法接受／拒絕、成本／狀態／自然期限、原資源／像素／音效與GUI仍待正式當版測試；原constructor時鐘偏差／scheduler交錯、倍率／u32 rollover與LT相對hurt未資格。正常1049、所有既有PE、正式JAR、registry與G06 binding／640次計畫不變。詳細分支與原失敗見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:];body=body.replace('先依當版#62／#64定義續接112個請求缺口與原呈現；','先依當版#62／#64定義續接112個請求缺口與原呈現；本輪四profile／root ABA／晚到取消與音效一次性仍待恢復後驗，勿把metadata當GUI資格；')
 p=T/'native_E149_process_birth_readonly.py';a=body.index('原最終reader `4da1cd`');b=body.index('## 下一步，依序接續',a);body=body[:a]+'前輪原PID查詢拒絕與CIM建立時間讀回分開封存。本輪接續以原FILETIME為先；無法開啟的PID只可由成功CIM觀測證明缺席或不同birth，不查詢無關影像、不控制或重啟程序。'+link('唯讀原身分工具',p)+'。\n\n'+body[b:]
 require(all(Path(p).is_file() for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference absent');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in before.items()),'concurrent report/checkpoint writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());(out/'entry.py').write_bytes((T/'update-master-report-E149-current.py').read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs');result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
