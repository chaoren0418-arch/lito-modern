"""Publish G06.3d code development; preserve deferred execution and prior evidence."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
source=w['current_product_identity']['source_manifest_sha256'];assert source=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
manifest_path=V/'native-installed-resource-profile-normal/application-20261020-123323-122063/applied-compiled-inputs.json'
manifest=read(manifest_path);assert len(manifest)==1049 and all(sha(R/n)==h for n,h in manifest.items())
candidate=R/'維護工具/candidates/native-teleport-cancel-idempotence-20261023-183222-325900/candidate.json';c=read(candidate)
assert c['base_source_manifest_sha256']==source and c['tests_executed']==c['builds_executed']==0 and not c['production_applied']
for n,h in c['candidate_files'].items():assert sha(candidate.parent/n)==h
for n,h in c['base_files'].items():assert sha(R/n)==h
assert sha(candidate.parent/'candidate.zip')==c['candidate_archive_sha256']
assert sha(c['baseline_regression_tests'])==c['baseline_regression_tests_sha256']
prior=dict(w['current_isolated_development']);assert prior['tests_executed']==prior['builds_executed']==0
paths={k:Path(v['path']) for k,v in prior['candidates'].items()};paths['cancel-recovery']=candidate
all_names=set();runtime_count=test_count=0
for key,p in paths.items():
    entry=read(p);assert entry['base_source_manifest_sha256']==source
    if key in prior['candidates']:assert sha(p)==prior['candidates'][key]['sha256']
    assert not(all_names&set(entry['candidate_files']));all_names.update(entry['candidate_files'])
    runtime_count+=len(entry['runtime_files']);test_count+=len(entry['test_files'])
assert len(all_names)==9 and runtime_count==6 and test_count==3
out=V/'native-development-continuation'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
review=dict(scope='static source review, no compiler/test/game/GUI execution',base_source_manifest_sha256=source,
 candidate=str(candidate),candidate_sha256=sha(candidate),
 problem='First cancel saves TeleportRecovery then clears teleport/credentials; original second cancel unconditionally assigns None and loses that unused correlation',
 changes=['Assign a new saved correlation only when an authorized destination still exists',
 'Keep all original retirement calls and credential removal on every cancel',
 'At one-shot consumption check the current LOGIN peer as well as account and source',
 'Retain ordinary phase/authentication and source send gates; no background login or gameplay is introduced'],
 static_boundary_review=['Source choice sends its ordinary request before consuming the correlation, as before',
 'A mismatched peer/account/source or absent credentials consumes the correlation and cannot revive by changing fields later',
 'A second cancel after consumption finds neither pending teleport nor correlation and cannot recreate it',
 'Failure keeps its error/Failed state; closing still waits for transport quiescence through original poll',
 'All original 8/30/60-second budgets, staged-expiry fallback and original server behavior remain unchanged',
 'Existing tests remain a byte-preserved prefix. Baseline red fixture contains only two appended cases because the other four use the new private consumer'],
 new_tests_written=6,baseline_additional_cases_written=2,baseline_cases_are_part_of_six=True,
 actual_baseline_reproduction_executed=False,actual_candidate_tests_executed=False,
 static_review_limit='No test results are inferred from code inspection. Actual repeated-cancel and recovery binding still require isolated/formal execution.',
 production_applied=False,GUI_verified=False,whole_goal_complete=False)
original=(R/'LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs').read_bytes()
assert (candidate.parent/'LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs').read_bytes().startswith(original)
(out/'static-review.json').write_bytes(data(review))
files={'candidate.json':candidate,'candidate.zip':candidate.parent/'candidate.zip','before-source.zip':candidate.parent/'before.zip',
 'baseline-regression-tests.rs':Path(c['baseline_regression_tests']),'preparer.py':candidate.parent/'preparer.py',
 'static-review.json':out/'static-review.json','checkpoint.py':Path(__file__),'base-compiled-inputs.json':manifest_path,
 'prior-development-evidence.json':Path(w['last_goal_turn_evidence'])}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in files.items())
registry={k:dict(path=str(p),sha256=sha(p),files=len(read(p)['candidate_files']),tests_written=read(p)['new_tests_written'],applied=False,built=False,tested=False) for k,p in paths.items()}
record=dict(status='G06_cancel_recovery_concrete_code_development_tests_deferred',source_manifest_sha256=source,
 recorded_utc=datetime.now(timezone.utc).isoformat(),previous_goal_turn_classification='progress',
 previous_goal_turn_evidence=w['last_goal_turn_evidence'],candidates=registry,overlay_files_disjoint=True,
 overlay_total_files=9,runtime_files=6,test_files=3,new_tests_written=16,tests_executed=0,builds_executed=0,desktop_inputs=0,
 static_review=str(out/'static-review.json'),static_review_sha256=sha(out/'static-review.json'),
 raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),test_execution_deferred_by_user=True,
 GUI_full_flow_verified=False,whole_goal_complete=False)
(out/'evidence.json').write_bytes(data(record))
work='先續G06.3d核心：連續取消會遺失尚未取用回復關聯之靜態缺口已做隔離修補；一次性取用再核對LOGIN端點／帳號／來源，原世界退役、帳密清空及時限不變。新增6案例（含2個原版回歸對照），三候選共九檔／16案例，均未編譯、測試或套用。依使用者要求測試／桌面操作暫緩，正常3E及既有資格保留，目標未完成。'
date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d');staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p);v.setdefault('isolated_development_history',[]).append(dict(prior_registry=v.get('current_isolated_development'),prior_evidence=v['last_goal_turn_evidence']))
        v['current_native_cancel_recovery_candidate']=dict(candidate=str(candidate),candidate_sha256=sha(candidate),source_manifest_sha256=source,
            applied=False,built=False,tested=False,new_tests_written=6,static_review_path=str(out/'static-review.json'))
        v['current_isolated_development']=record
        v.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        v['current_focus']=dict(item='G06_3d_cancel_failure_recovery_development',work=work,applies_to_current_source=False,goal_complete=False)
        v['current_source_goal_validation'].setdefault('G06-room-service-branches',{}).update(complete=False,development_candidate=v['current_native_cancel_recovery_candidate'])
        for item in v.get('items',[]):
            if item.get('id')=='G06-room-service-branches':item.update(complete=False,development_candidate=v['current_native_cancel_recovery_candidate'])
        if n.endswith('next-validation-action.json'):v['next_action']=work
        staged[n]=data(v)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in manifest.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),candidate=str(candidate),tests_executed=0,normal_source_unchanged=True),ensure_ascii=False))
