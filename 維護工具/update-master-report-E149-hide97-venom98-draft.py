"""Compact source-only continuation report; preserve the exact previous report."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,path):
 path=Path(path);require(path.is_file(),'missing report reference '+str(path));return '['+label+'](<'+path.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');dref=w['current_E149_hide97_venom98_draft_development'];require(sha(dref['evidence'])==dref['evidence_sha256'],'draft evidence changed');e=read(dref['evidence']);check_pins(e['source_pins']);ref=w['current_E149_selector80_development'];require(sha(ref['qualification'])==ref['qualification_sha256'] and ref==e['previous_qualified_metadata'],'prior metadata changed');q=read(ref['qualification']);check_pins(q['normal_product_pins']);require(e['metadata_compilers_started']==e['tests_executed']==0 and not e['tests_compiled'] and e['full_source_review_required'] and e['hide97_static_frame_zero_and_retirement_unresolved'],'draft scope changed')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase changed');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and e['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and e['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and e['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 changed')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');after=read(checkpoint)['after_files'];require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()),'checkpoint changed');inputs=read(e['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source changed')
 old=REPORT.read_bytes();require(sha(REPORT)=='6030DECEDF689B7463E7A9880E75885581DB727B5A481A5752B60F823E70E270','master predecessor changed');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);entry=T/'update-master-report-E149-current.py';(out/'entry.before.py').write_bytes(entry.read_bytes());body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,path),body);require(n>=1,'report anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發與目前界線

**最新97暗隱術／98附加劇毒為未編譯草稿**：四Runtime覆寫與4個QA草稿（3純／1原資源opt-in）。共用selector-only請求／選單、97 reuse9990／98 reuse0；98原2942九幀Sound1505 source1、有限1496ms接既有finite History與world音效。成本、材料、隱身及中毒仍server掌管，S37不是狀態ACK。**本輪沒有啟動metadata或測試；4案未編譯未跑，不併入455個既有metadata案例。** {link('草稿與本輪來源證據',dref['evidence'])}；{link('完整待查路徑與草稿',T/'native-E149-hide97-venom98-draft.md')}。

97呈現發現關鍵路徑：2941雖只有type9／無action，原4E5B50／4E5C00會回source0並讀靜態第0圖，缺圖另有0x460後備；**無action不等於無像素**。casefold索引可找到2941-0.SPR（865 bytes）、1120-0.SPR（211 bytes）。目前僅快取索引與五段native函式，未綁canonical pak／圖像；44EF80第一次顯示、0ms callback及退役／全域scheduler順序未完成。材料已補29 class Code與S34 codecs，但仍須補NativeCastTimeline.admitSelf到NativeCastMaterial.commit的實際鏈。不得把2941當無畫面或套一般finite Clip。

目前經metadata的候選仍為#80修補：42檔／28 Runtime／11 Rust QA／3 Java、371module與原六nested，上一輪18metadata退出0（原5da16a），**455案全未跑**。request／menu共用selector_only，#80 live／stale目標先清除，直接request帶target仍拒絕；Owner／learned與其他格式守衛保留。來源入口86／剩94不變，不代表86技能完整。97／98草稿尚未加入該編譯圖，不得移記此前資格。

90／91原三有限特效Hit診斷續播、server反擊、88／89與279回饋、87持續4183的Scene／History／typed音效及S08／S0B守衛沿用；4183 wire到model／原clock初始化／rollover／全域交錯、重送S0B與首圖推論仍未資格。69同socket／server原子授權、direction initializer、constructor clock skew、LT-relative hurt及當版TCP／原資源／自然期限／GUI皆未驗。正常1049、既有PE、正式JAR、registry及G06 binding／640次不變，未套用／選入／連結新PE。{link('此前完整報告與歷史證據',out/'before.md')}。

本輪原生取證工具首版因Python參數名稱以數字開頭，parser exit1、本文未執行；原工具／錯誤封存，v2只修名稱並完成唯讀取證。這是工具語法錯誤，沒有啟動或重試遊戲／測試。{link('原工具失敗與未執行範圍',e['first_syntax_failure'])}。

'''
 body=body[:start]+latest+body[end:];start=body.index('1. 暫緩期間繼續開發：');end=body.index('2. 使用者明確恢復測試後：',start)
 body=body[:start]+'1. 暫緩期間繼續開發：先完成97靜態2941-0.SPR／0x460的canonical archive與圖像綁定、44EF80／44F330／scheduler的初次顯示及0ms退役順序，補NativeCastTimeline→NativeCastMaterial.commit實際鏈；完整review後才完成97／98候選並編譯草稿4案。當前metadata仍455案／入口86／剩94，不借舊結果。其後續下一組目錄，並續4183 clock／S0B、G06物品／source/account／timer與LT hurt/base。測試、GUI與自然期限仍待明確恢復。\n'+body[end:]



 require(all(Path(p).is_file()for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference missing');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()),'concurrent writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n';entry.write_text(selected,encoding='utf-8',newline='\n');(out/'entry.py').write_bytes(entry.read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
 result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],latest_draft_evidence=dref['evidence'],latest_draft_evidence_sha256=dref['evidence_sha256'],metadata_compilers_started=0,tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
