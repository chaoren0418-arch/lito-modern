"""Update the compact master from the latest bound source proof; preserve its predecessor."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def link(label,p):
 p=Path(p);require(p.is_file(),'missing report reference '+str(p));return '['+label+'](<'+p.as_posix()+'>)'
def main():
 w=read(R/'工作佇列.json');ref=w['current_E149_resurrection61_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'latest source proof differs');q=read(ref['qualification']);check_pins(q['source_pins']);check_pins(q['normal_product_pins']);require(q['source_manifest_sha256']==SOURCE and q['formal_target_sha256']==FORMAL and q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['explicit_consent_and_dead_life_owner'] and q['command_id_widening_only'] and not q['production_applied'],'latest scope differs')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'phase differs');require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'],'formal/registry/G06 differs')
 checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json');before=read(checkpoint)['after_files'];require(len(before)==8 and all(sha(R/n)==h for n,h in before.items()),'checkpoint differs');c=read(Path(q['compile_run'])/'cohort.json');inputs=read(c['base_compiled_inputs']);require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()),'normal source differs')
 old=REPORT.read_bytes();require(hashlib.sha256(old).hexdigest().upper()=='CEF975E06070CB1B766AFE9D365FCF4516C2F3E929FA4D36393FC2C3086062E4','master predecessor differs');out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);(out/'before.md').write_bytes(old);body=old.decode('utf-8-sig');date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M');body=re.sub(r'更新：[^（]+（Asia/Taipei）','更新：'+date+'（Asia/Taipei）',body,count=1)
 body=body.replace('最新隔離候選35檔：21 Runtime／11 Rust QA／3 Java，366module','最新隔離候選36檔：22 Runtime／11 Rust QA／3 Java，367module').replace('不含最新35檔','不含最新36檔')
 for label,p in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
  body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)',lambda _:link(label,p),body);require(n>=1,'report reference anchor absent '+label)
 start=body.index('## 已完成的來源開發');end=body.index('## G06與尚缺前提',start)
 latest=f'''## 已完成的來源開發

**最新#61復活來源候選**：C14 `14 07 04`＋屍體u32 ID七bytes，原畫面、顯式death action意圖、滑鼠／可見像素T鍵與原controller守衛已接。S24/question321恰為三bytes；C23同意／拒絕為五bytes含空字串。MP50、玩家HP除2／NPC除4、模板資格、經驗與材料均由server決定，未造本機復活或成功ACK。

死者的非modal詢問綁connection、inventory selection、角色／來源、map／room／phase、generation、原incarnation／death action。明確同意、拒絕或放棄等待；同意先取用token再傳送，失敗關閉該controller、不自動重送，等待時暫擋自動C_E0。替換詢問採不同command ID，舊queued click不接受新問題；每controller上限30000、不回繞，耗盡明確失敗並需新視窗。放棄等待僅解除本機等待，不送第二個C23；沒有預設同意或自訂server TTL。

3944為施法光效，原八bank／24frame／Sound1362 source2；成功先S37 gfx230（七frame／Sound124 source1）再S11，HP／ratio另到。兩種已收到視覺固定原位置，3944另存heading，以免同poll S11換life將其丟棄；typed音效使用world actor0、真sequence／Scope，整數視覺3371／830ms。S11繼續獨立更新life，光效不作ACK。

九Runtime差分含一新module，共36檔／367module；controller五nested＋window一rental nested同原target重建。修正版兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0。**新15（14純＋1原資源opt-in）＋前252共267案全未跑**，非完整library inventory；入口68／剩112，並非68項玩法驗收。原0033e1的Debug library E0277與部分準備候選保留，修正版只改兩處command ID轉換，全部18重新編譯。{link('精確來源與待驗範圍',T/'native-E149-resurrection61.md')}；{link('原失敗與修正',T/'native-E149-resurrection61-cmdid.md')}。

前輪#59暴風雪25節點／Many History、#55／56方向與Multiply、#51召喚選項及跨poll拒絕、#50無音效class4、#45／46／47與S23接續等來源均沿qualification祖源保留。原constructor時鐘偏差／scheduler交錯、action0倍率／u32 rollover與LT相對hurt仍待；正常1049、所有既有PE、正式JAR、registry及G06 binding／640次計畫不變。詳細分支與歷史失敗見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
 body=body[:start]+latest+body[end:]
 line=next(l for l in body.splitlines() if l.startswith('1. 暫緩期間'))
 body=body.replace(line,'1. 暫緩期間繼續開發：先依當版#62／#64定義續接112個請求缺口與原呈現；#61來源已接候選，玩家／NPC資格、同意／拒絕／放棄、S37→S11順序／八方向／音效與owner退役待測試恢復後驗。並續查G06物品／source/account／timer前提、#45／46／50 hurt原guard與gfx初始化／LT綁定；不猜base=1／5894／11297。完整companion指令、owner及1800／3600秒自然期限仍待GAME。')
 require(all(Path(p).is_file() for p in re.findall(r'\]\(<([^>]+)>\)',body)),'report reference absent');require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in before.items()),'concurrent report/checkpoint writer');(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());(out/'entry.py').write_bytes((T/'update-master-report-E149-current.py').read_bytes());REPORT.write_bytes((out/'after.md').read_bytes());require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs');result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_bytes(encoded(result));print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':main()
