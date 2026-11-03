"""Update the compact continuation report from the current source-only checkpoint."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib, json, re
from native_E149_recovery_contract import R, SOURCE, FORMAL, read, sha, encoded, require, check_pins

T=R/'維護工具'; V=R/'驗證紀錄/current'; REPORT=R.parent/'LiTo381-總開發報告.md'

def link(label, path):
    path=Path(path); require(path.is_file(), 'missing report reference '+str(path))
    return '['+label+'](<'+path.as_posix()+'>)'

def main():
    w=read(R/'工作佇列.json'); ref=w['current_E149_earthquake62_silence64_development']
    require(sha(ref['qualification'])==ref['qualification_sha256'], 'qualification changed')
    q=read(ref['qualification']); check_pins(q['source_pins']); check_pins(q['normal_product_pins'])
    require(q['compiler_exit_codes']==[0]*18 and q['tests_executed']==0 and q['request_registered_ids']==[62,64] and q['prior_G06_finite_history_unchanged'] and not q['production_applied'], 'source-only scope changed')
    require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and w['current_user_requested_goal']['desktop_automation_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'], 'phase changed')
    require(sha(w['current_formal_reference_identity']['formal'])==FORMAL and q['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and q['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development'] and q['canonical_registry_unchanged']==w['current_isolated_development']['candidates'], 'formal/registry/G06 changed')
    checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json'); after=read(checkpoint)['after_files']
    require(len(after)==8 and all(sha(R/n)==h for n,h in after.items()), 'checkpoint changed')
    c=read(Path(q['compile_run'])/'cohort.json'); inputs=read(c['base_compiled_inputs'])
    require(len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items()), 'normal source changed')
    old=REPORT.read_bytes(); require(sha(REPORT)=='A88D0BCBE668E74640AE110269DF83837E9D5001541F84D92B509986E1CA1AE7', 'master predecessor changed')
    out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'); out.mkdir(parents=True)
    (out/'before.md').write_bytes(old)
    entry=T/'update-master-report-E149-current.py'; (out/'entry.before.py').write_bytes(entry.read_bytes())
    body=old.decode('utf-8-sig'); date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')
    body=re.sub(r'更新：[^（]+（Asia/Taipei）', '更新：'+date+'（Asia/Taipei）', body, count=1)
    for label,path in [('最新來源與18metadata',ref['qualification']),('最新八檔 checkpoint',checkpoint),('此前完整報告與歷史證據',out/'before.md'),('舊映像與原始記錄',out/'before.md')]:
        body,n=re.subn(r'\['+re.escape(label)+r'\]\(<[^>]+>\)', lambda _:link(label,path), body)
        require(n>=1, 'report reference anchor absent '+label)
    start=body.index('## 已完成的來源開發'); end=body.index('## G06與尚缺前提',start)
    latest=f'''## 已完成的來源開發

**最新#62震裂術／#64魔法封印隔離候選**：62接三byte無目標請求、1500ms reuse；S39 action18／type0／1812固定原wire施法位置。保留Over本體、1813 Shadow、1814 Screen三層、16原frame與末四格長hold；Sound716 source1／Hit5、視覺10038ms，晚poll不補跑。Hit僅診斷，原guard／LT相對hurt未綁；重備失敗只退役一次，不改HP。

64接角色ID／x/y十一byte、其他可見活體／三格／reuse500。已核對原NativeSilenceMaterialAction及實際Bootstrap篩選；MP、寶石40318、機率、16秒與資格皆由server決定。既有2177／Sound808／2822ms有限呈現接world actor0音效，目標離開不截斷已收到的音效，不造本機沉默timer或成功ACK。

四Runtime差分，候選仍36檔／22 Runtime／11 Rust QA／3 Java、367module與原六nested。兩版library／library-test／七skill整合18metadata退出0，原`{q['observed_original_tool_chunk']}` actual exit0；**新14（12純＋2原資源opt-in）＋前276共290案全未跑**，不是完整library inventory。來源請求入口70／剩110，不等於玩法已驗收。{link('62／64來源、原始證據與待驗範圍',T/'native-E149-earthquake62-silence64.md')}。

前輪G06四類finite History同源cursor、來源退役保留admission、晚到worker去重與音效序號保留沿用；root_changed立即永久退役stored History，A→B→A不能復活舊graph。#61復活／question321／音效跨S11、#59暴風雪25節點、#55／56、#51、#50等沿qualification祖源保留。64active／32音效／512fragments／4096keys上限不變，G11最大共存未驗。所有玩法、成本／狀態／自然期限、原資源／像素／音效與GUI仍待當版測試；constructor時鐘偏差／全域交錯、倍率／u32 rollover與LT相對hurt未資格。正常1049、所有既有PE、正式JAR、registry與G06 binding／640次計畫不變。{link('四類History與限制',T/'native-E149-finite-history.md')}；{link('root ABA與原版本界線',T/'native-E149-finite-history-root-aba.md')}。詳細分支與原失敗見{link('此前完整報告與歷史證據',out/'before.md')}。

'''
    body=body[:start]+latest+body[end:]
    previous='先依當版#62／#64定義續接112個請求缺口與原呈現；'
    require(body.count(previous)==1, 'next action anchor differs')
    body=body.replace(previous,'先依當版#65／#66定義續接110個請求缺口與原呈現；#62三層／長hold／Hit guard與#64材料／機率／自然16秒待恢復後驗；')
    require(all(Path(p).is_file() for p in re.findall(r'\]\(<([^>]+)>\)',body)), 'report reference missing')
    require(REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in after.items()), 'concurrent writer')
    (out/'after.md').write_text(body,encoding='utf-8',newline='\n'); (out/'updater.py').write_bytes(Path(__file__).read_bytes())
    selected='"""Current compact continuation report entry."""\nfrom pathlib import Path\nimport runpy\nrunpy.run_path(str(Path(__file__).with_name("'+Path(__file__).name+'")),run_name="__main__")\n'
    entry.write_text(selected,encoding='utf-8',newline='\n'); (out/'entry.py').write_bytes(entry.read_bytes())
    REPORT.write_bytes((out/'after.md').read_bytes()); require(REPORT.read_bytes()==(out/'after.md').read_bytes(),'master readback differs')
    result=dict(source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),prior_report=str(out/'before.md'),latest_qualification=ref['qualification'],latest_qualification_sha256=ref['qualification_sha256'],tests_executed=0,GUI_full_flow_verified=False,whole_goal_complete=False)
    (out/'readback.json').write_bytes(encoded(result)); print(json.dumps(result,ensure_ascii=False))

if __name__=='__main__': main()
