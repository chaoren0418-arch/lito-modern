"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_mass69_fire70_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['fire70_request_and_four_node_presentation_integrated'] and q['mass69_components_source_complete'] and not q['mass69_controller_or_GUI_integrated'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='93F44864235FF89443DF2998D4546B15353852C1686CF42A5FE740BFB4525198','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發

**最新#70火風暴來源已接，#69目前為未串接的協定核心**。#70 C14為三byte selector，S39為action18／1819／type0；reuse2000ms，HP／MP與傷害由server判定。原PE1819命中44F4F1特例，依序生成方向3／7／5的無descriptor子節點，再把根移至方向1，皆距離1；四個offset是[1,1]／[-1,-1]／[-1,1]／[1,-1]，不含中心，只有最後根保留伺服器目標清單。

新增四節點獨立Playback、callback／audio occurrence與History來源退役，接到AttackMany及既有64 casts／32 cues／512 fragments限額；原14幀／Sound710來源1／Hit7／Screen保留，Hit只診斷，不造傷害或現代hurt。現行整數ms模型Hit873／結束2369ms、late poll不補跑；真正direction initializer／constructor逐節點clock skew／全域scheduler交錯、像素／可聽音效／GUI／LT-relative hurt均未資格。四個prepared音效事件不代表四次可聽聲音。

#69新增S30書籤（name、x/y/map）、九byte C14目的地（map/x/y）、隨機與正式書籤Choice、S24/748一次性同意／拒絕，以及自己S55→C4E→一次S5A的owner元件。30秒邀請由server在施法時起算，client顯示上限不是授權；recipient不需已學69／MP。**尚未接控制器、原生GUI或已接入口，不可當69完成。** 2236沿既有七幀／Sound836 bank，僅補world typed audio及opt-in來源。{link('69核心／70四節點與待驗界線',T/'native-E149-mass69-fire70.md')}。

四既有Runtime差分＋兩新module，候選42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新24（22純／2原資源opt-in）＋前338共362案全未跑**，不是完整library inventory。入口75／剩105，先完整接69再71。無新PE連結、正常套用／registry選入或GAME／GUI資格。

初次讀取工具對無BootstrapMethods的class退出1；原default-start假設也被1819專用分支否決。兩次原失敗／未編譯單clip草稿及修正後完整原證據分開保存，沒有執行PE或GAME類別。前輪67／68選單與S27 World／Animator／圖層／History來源資格沿祖源保留，原Eq失敗與各中間編譯不移記本輪。正常1049、所有既有PE、正式JAR、registry與G06 binding／640次不變。歷史分支及尚缺前提見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:];body=body.replace('最新隔離候選40檔：26 Runtime／11 Rust QA／3 Java，369module','最新隔離候選42檔：28 Runtime／11 Rust QA／3 Java，371module').replace('不含最新40檔','不含最新42檔');start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：先將69完整接入控制器／原生GUI：S30角色owner、隨機與正式書籤目的地、748非模態同意／拒絕與token命令、自己S55→一次C4E→S5A→C08／場景重建與舊輸入退役，保留G06與原八秒失敗策略；recipient不需已學69／MP。再71，目前入口75／剩105。69自然30秒邀請、70四節點原資源／clock skew／Hit／音效／owner與GUI待恢復測試後驗；同時續查G06物品／source/account／timer前提、hurt原guard及LT gfx初始化，不猜base。\n'+body[end:]

 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
