"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_shape67_protection68_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['shape67_menu_or_request_integrated'] and q['protection68_request_integrated'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='76AE51EF8A5BC7C95B4F94B950E2EFA0CA8F6BE1A26CAB14CC503CD8F26B5BC1','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發

**最新#67變形術與#68聖結界來源已接**：兩者都是C14七byte角色ID，未選取時對自己，失效目標不退回自己。#67使用原screen diamond／reuse10ms，#68 reuse1500ms；正式MP80／30、寶石1／2顆、玩家與同非空clan資格、控制能力／隨機外觀、自然7200／32秒仍由server判定，不造本機成功／狀態／期限。

新增monlist原生選單：被施法角色可接收自己S2A grant，不需學會#67／本機cast pending或MP。完整96個正式standard_menu、明確解除／套用／關閉；關閉及Escape只本機收起，C_HACTION一次性consume-before-send。Owner綁epoch、場景／map、phase／room、source、inventory角色owner、actor生命／S27 revision與control interruption；三原生控制項以HWND＋不回繞token命令ID核對，Enter不自選／施法、選單期間清持續輸入並阻擋世界捷徑。#228沿原七幀／Sound132且用world actor0音效；S37特效不是成功ACK。{{DETAIL}}

五既有Runtime差分＋一新module，候選40檔／26 Runtime／11 Rust QA／3 Java、369module與原六nested。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新16（15純／1原資源opt-in）＋前322共338案全未跑**，不是完整library inventory。首版18metadata（`{q['source_review_precursor_observed_original_tool_chunk']}` exit0）僅保留修正前範圍；來源檢查補上開選單聚焦、Tab與關閉返回焦點後，最終版重新完成18metadata。入口74／剩106，接續#69／70。尚未套用正常來源／registry或連結新版PE，沒有GAME／GUI資格。

前輪S27 World／Animator／圖層／History有序body整合保留；busy保持舊logical gfx／family，idle依收包順序切換，live queue不被raw admission跳過，root ABA退役及晚到worker只合併admission。queued584沿現行ms clock建模，真正clock import／動態初始化、逐包時刻、全域scheduler交錯、原資源與GUI仍未驗。{link('S27串接與待驗界線',T/'native-E149-appearance27-binding.md')}；{link('存活graph回呼floor修正',T/'native-E149-appearance27-binding-callback.md')}。原Eq失敗／中間metadata與此前技能來源沿祖源保留，未移記最終或本輪資格。

正常1049、所有既有PE、正式JAR、registry與G06 binding／640次不變。所有玩法、成本／機率／狀態／自然期限、圖層／像素／音效／owner、G11及GUI待當版測試；constructor clock skew、倍率／u32 rollover、LT相對hurt仍未資格。歷史分支與失敗見{link('此前完整報告與歷史證據',out/'before.md')}。

'''.replace('{DETAIL}',link('67／68來源與待驗界線',T/'native-E149-shape67-protection68.md'))
 body=body[:start]+latest+body[end:];body=body.replace('最新隔離候選39檔：25 Runtime／11 Rust QA／3 Java，368module','最新隔離候選40檔：26 Runtime／11 Rust QA／3 Java，369module').replace('不含最新39檔','不含最新40檔');start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：續接#69／70，入口74／剩106；逐一核對正式方法、協定與原資源，並查G06物品／source/account／timer前提、hurt原guard及LT gfx初始化，不猜base。#67完整monlist、#68與S27原clock import／584排程／逐包時刻／交錯、圖層／音效／重建／owner／GUI待恢復測試後驗；自然7200／32秒不縮時，companion指令／owner及1800／3600秒仍待GAME。\n'+body[end:]
 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
