"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_mass69_integration_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'qualification changed');q=read(ref['qualification']);check_pins(q['normal_product_pins'])
 require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[] and q['mass69_components_source_complete'] and q['mass69_controller_or_GUI_integrated'] and q['mass69_request_supported_in_source'] and not q['production_applied'],'source-only scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');c=read(Path(q['compile_run'])/'cohort.json');require(sha(Path(q['compile_run'])/'cohort.json')==q['compile_cohort_sha256'],'cohort changed');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='657734588772CD61ADC2E148FAEDE01C67038FED6A624CA04343C2F3A75F6291','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發

**最新#69已接控制器與原生GUI來源**：明確選擇隨機／自己正式書籤目的地後才送九byte C14，generic actor請求不會預設隨機。S30、書籤revision、角色／場景owner與目的地下拉的HWND＋不回繞命令ID逐次核對；過期通知完全忽略，不清新Choice。沒有有效owner的合法晚到S30只解析後消化，不能建立authority；同socket重選交錯／server原子授權仍待GAME。

748非模態詢問預設聚焦不同意，Enter只處理聚焦且啟用按鈕，Escape明確拒絕；同意後只本機放棄等待、不送第二次C23。recipient不需已學69或MP，血盟／距離／隱身／私店／MP60／一寶石與server自施法時起算的30秒由server判定，client收包後30秒只是顯示上限。自己S55才回一次C4E，再受理一次owned S5A，沿既有C08／精確重建；等待期間清舊目標／持續輸入及背包／租裝頁面，控制器阻擋世界操作，聚焦取消可Enter結束連線。過期grant不能落回ordinary／rental admission；獨立八秒deadline失敗會清連線權限。{link('69來源接線與待驗界線',T/'native-E149-mass69-integration.md')}。

五Runtime差分，仍42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested。最終兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新10純＋前362共372案全未跑**，不是完整library inventory。首版18metadata（`{q['source_review_precursor_observed_original_tool_chunk']}` exit0）僅保留邊界修正前範圍，最終來源獨立重編譯。入口76／剩104，接續71／74。沒有當版TCP、GUI、自然30／8秒或玩法通過聲明。

前輪70四節點森林、2236音效、67／68選單與S27 World／Animator／圖層／History資格沿祖源保留；direction initializer、constructor clock skew、全域scheduler、原資源像素／聲音、LT-relative hurt仍未驗。原reset-anchor工具次序錯誤於候選寫出前退出1，失敗／首版／修正後收據分開保存。正常1049、所有既有PE、正式JAR、registry與G06 binding／640次不變；未套用正常來源或選入registry，沒有新PE連結。歷史分支、原1819特例否決單clip草稿與既有失敗見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:];body=body.replace('最新隔離候選40檔：26 Runtime／11 Rust QA／3 Java，369module','最新隔離候選42檔：28 Runtime／11 Rust QA／3 Java，371module').replace('不含最新40檔','不含最新42檔');start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：續接71／74，入口76／剩104；逐一核對正式方法／協定／原資源。續查G06物品／source/account／timer前提、hurt原guard及LT gfx初始化，不猜base。69書籤／隨機、748同意拒絕／自然30秒、舊owner、S55→C4E→S5A→C08／重建與原八秒失敗，70四節點／clock skew、History／音效／GUI皆待恢復測試後驗。\n'+body[end:]


 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
