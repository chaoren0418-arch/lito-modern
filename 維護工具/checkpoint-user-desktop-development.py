"""Publish user-deferred testing and isolated development, without test execution."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
source=w['current_product_identity']['source_manifest_sha256'];assert source=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
candidate=R/'維護工具/candidates/native-keyboard-target-20261020-162623-551177/candidate.json';c=read(candidate)
assert c['base_source_manifest_sha256']==source and not c['production_applied'] and c['tests_executed']==c['builds_executed']==0
assert sha(candidate.parent/'candidate.zip')==c['candidate_archive_sha256']
for n,h in c['base_files'].items():assert sha(R/n)==h
for n,h in c['candidate_files'].items():assert sha(candidate.parent/n)==h
prior=R/'維護工具/candidates/native-keyboard-target-20261020-162448-278551/candidate.json'
cases=[]
for name,session,chunk in [('case-20261020-143412-505402-debug-rental-entry',15112,'8d394a'),('case-20261020-161557-898775-debug-rental-entry',23695,'f89e10')]:
    case=V/'native-installed-resource-profile-sky-gui'/name;cohort=read(case/'cohort.json')
    terminal=list(case.glob('checkpoint-terminal-*/evidence.json'));assert len(terminal)==1;t=read(terminal[0])
    assert t['original_session_id']==session and t['terminal_tool_chunk']==chunk and t['observed_actual_exit_code']==1 and t['original_identities_terminal']
    assert cohort['status']=='failed_preserved' and cohort['recorded_owned_processes_terminal']
    cases.append(dict(case=str(case),original_session_id=session,terminal_tool_chunk=chunk,exit_code=1,cohort_sha256=sha(case/'cohort.json'),qualification_granted=False))
out=V/'native-user-desktop-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
files={}
for i,entry in enumerate(cases):
    case=Path(entry['case'])
    for p in case.rglob('*'):
        if p.is_file() and not any(part.startswith('checkpoint-') for part in p.relative_to(case).parts):
            files[f'GUI-{i}/'+p.relative_to(case).as_posix()]=p
    terminal=next(case.glob('checkpoint-terminal-*/evidence.json'));files[f'GUI-{i}/terminal.json']=terminal
files.update({'candidate.json':candidate,'prior-candidate.json':prior,'flow-guards.mjs':R/'維護工具/sky-native-GUI-flow-guards.mjs','checkpoint.py':Path(__file__)})
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in files.items())
review=dict(scope='manual static source review only; no compile/test/UI execution',candidate=str(candidate),candidate_sha256=sha(candidate),
 superseded_candidate=str(prior),superseded_candidate_sha256=sha(prior),
 static_changes=['Scan each shared owner pixel plane once; stable sorted/deduplicated object IDs',
 'Keep pixel occlusion, minimap exclusion, actual painted epoch/viewport/revision and owner gates',
 'Skip self, ground items, dead/invisible/retired targets; keep controller send and pending gates',
 'Reincarnation case rejects the old frame and captures a fresh target only after a fresh frame',
 'T is handled only with root world focus, excludes inventory/NPC/item targeting/observer, suppresses autorepeat',
 'Future QA guards reject movement before map-ready and quantity Enter outside the received focused field'],
 new_tests_written=4,tests_executed=0,builds_executed=0,production_applied=False,
 remaining_review=['Compile/type correctness and actual test outcomes remain unverified',
 'Multi-target GUI cycling, keyboard focus, repeated key, minimap overlap, incarnation/cancel/epoch/DPI must be exercised after testing resumes',
 'Physical audio, screenshots and pointer support still unqualified'],whole_goal_complete=False)
(out/'static-review.json').write_bytes(data(review))
record=dict(status='development_continues_tests_deferred_by_user',source_manifest_sha256=source,
 requested_utc=datetime.now(timezone.utc).isoformat(),user_instruction='現在我要使用電腦 你先做開發 測試稍後在進行',
 test_execution_deferred=True,desktop_automation_deferred=True,development_active=True,
 candidate=str(candidate),candidate_sha256=sha(candidate),static_review=str(out/'static-review.json'),static_review_sha256=sha(out/'static-review.json'),
 tests_executed_since_defer=0,builds_executed_since_defer=0,production_applied=False,
 previous_GUI_attempts=cases,input_diagnostics='Separate key actions did not connect; Tab+space connected B. Minimized/input-guard/geometry errors observed; root cause not proven. No further desktop input after user defer.',
 raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files={n:sha(p) for n,p in files.items()},
 GUI_full_flow_verified=False,whole_goal_complete=False)
(out/'evidence.json').write_bytes(data(record))
work='依使用者要求先做開發，GUI及其他測試稍後；兩Debug租裝未通過批次已終止封存。T鍵切換已呈現有效目標之隔離候選已寫4案例並靜態審查，未編譯／測試／套用。正常3E與Release租裝等資格保留；待恢復測試先驗候選，再接當版Debug租裝及其餘GUI，不交人工。'
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p);v['current_user_desktop_development']=record
        v['current_native_keyboard_target_candidate']=dict(candidate=str(candidate),candidate_sha256=sha(candidate),source_manifest_sha256=source,applied=False,built=False,tested=False,new_tests_written=4,static_review=review)
        v.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        v['current_focus']=dict(item='isolated_native_keyboard_target_development',work=work,applies_to_current_source=False,goal_complete=False)
        v['current_user_requested_goal'].update(development_active=True,test_execution_deferred_by_user=True,desktop_automation_deferred_by_user=True)
        if n.endswith('next-validation-action.json'):v['next_action']=work
        staged[n]=data(v)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),candidate=str(candidate),tests_deferred=True,development_active=True),ensure_ascii=False))
