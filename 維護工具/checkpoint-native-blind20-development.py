"""Publish concrete isolated development and static review, with tests deferred."""
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
paths={
 'keyboard':R/'維護工具/candidates/native-keyboard-target-20261023-181848-311325/candidate.json',
 'blind20':R/'維護工具/candidates/native-blind20-20261023-182240-211519/candidate.json'}
entries={k:read(p) for k,p in paths.items()}
names=set()
for k,c in entries.items():
    assert c['base_source_manifest_sha256']==source and c['tests_executed']==c['builds_executed']==0 and not c['production_applied']
    assert not(names & set(c['candidate_files'])),'Overlay file collision requires an explicit merge'
    names.update(c['candidate_files'])
    for n,h in c['base_files'].items():assert sha(R/n)==h
    for n in c.get('new_files',[]):assert not(R/n).exists()
    for n,h in c['candidate_files'].items():assert sha(paths[k].parent/n)==h
    assert sha(paths[k].parent/'candidate.zip')==c['candidate_archive_sha256']
assert len(names)==7
out=V/'native-development-continuation'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
failure=V/'native-blind-formal75-contract/20261023-181438-228426'
failure_record=dict(status='static_inspector_output_encoding_failure_retained',original_tool_chunk='a4fcfc',python_exit_code=1,
 error='UnicodeDecodeError: utf-8 could not decode byte 0xb6 at position 40766',
 original_inspector_snapshot_captured=False,product_defect_proven=False,game_or_fixture_executions=0,tests_executed=0,
 retained_stdout=str(failure/'Protocol.stdout.txt'),retained_stdout_sha256=sha(failure/'Protocol.stdout.txt'),
 correction='Javap explicitly uses -J-Dfile.encoding=UTF-8; new independent static extraction retained separately')
(out/'static-inspector-failure.json').write_bytes(data(failure_record))
old_target=Path(w['current_native_keyboard_target_candidate']['candidate'])
review=dict(scope='static source/bytecode review only; no compiler, tests, game fixture or desktop execution',
 base_source_manifest_sha256=source,base_compiled_input_files=1049,base_changed_files=[],
 previous_goal_turn_classification='progress',reason='Prior isolated candidate, four written cases and eight-file authoritative checkpoint are retained',
 keyboard=dict(candidate=str(paths['keyboard']),candidate_sha256=sha(paths['keyboard']),superseded_candidate=str(old_target),superseded_candidate_sha256=sha(old_target),
    defect_found_statically='Raster owner pixels alone do not prove visibility after GAME blind/environment overlays',
    changes=['Carry the frame render-environment key into the actual presented InputSurface',
    'Require current GAME/environment key and blind=0 before T target cycling; retain epoch, viewport, revision, incarnation and pixel ownership guards',
    'Reject resource-only unbound surfaces, blind modes1/2 and stale masked surfaces after a cure',
    'Extend the existing four written cases; do not replace old baseline assertions'],
    new_tests_written=4,tests_executed=0,applied=False),
 blind20=dict(candidate=str(paths['blind20']),candidate_sha256=sha(paths['blind20']),
    formal_contract=entries['blind20']['contract'],formal_contract_sha256=entries['blind20']['contract_sha256'],
    changes=['Add only observed skill20 to the existing target+coordinate C14 path, one-cell envelope and 500ms server reuse',
    'Add the learned-ID menu row; keep stale target and learned-book ownership gates',
    'Reuse original746 finite bank and existing S0A environment/rendering; keep costs, probability, duration and rejection server-owned',
    'Write six cases; source review corrected a mutable borrow overlapping request and makes retirement occur during an active finite effect',
    'Keep native per-frame integer millisecond effect timing; do not substitute a rational accumulated tick timeline'],
    new_tests_written=6,tests_executed=0,applied=False),
 normal_recognized_skill_ids=38,normal_missing_observed_skill_ids=36,
 runtime_qualification_granted=False,GUI_verified=False,whole_goal_complete=False,
 remaining=['No candidate has been compiled or executed; type correctness and assertions remain unproven',
 'Resume tests only after user permits: isolated components then formal server and normal GUI scope',
 'Do not reclassify candidate skill20 as part of normal38 or claim two-player blindness/physical sound/natural expiry',
 'G06.3d/controller96 and normal Debug rental entry/hotel/Enter/EOF/full GUI still require final-source execution'])
(out/'static-review.json').write_bytes(data(review))
files={'static-review.json':out/'static-review.json','static-inspector-failure.json':out/'static-inspector-failure.json',
    'checkpoint.py':Path(__file__),'base-compiled-inputs.json':manifest_path}
for k,p in paths.items():
    files[k+'/candidate.json']=p;files[k+'/candidate.zip']=p.parent/'candidate.zip';files[k+'/before.zip']=p.parent/'before.zip';files[k+'/preparer.py']=p.parent/'preparer.py'
contract=Path(entries['blind20']['contract']);files['formal/contract.json']=contract;files['formal/selected-methods.json']=Path(read(contract)['selected_methods'])
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in files.items())
record=dict(status='concrete_isolated_development_progress_tests_still_deferred',source_manifest_sha256=source,
 recorded_utc=datetime.now(timezone.utc).isoformat(),previous_goal_turn_classification='progress',
 candidates={k:dict(path=str(p),sha256=sha(p),files=len(entries[k]['candidate_files']),tests_written=entries[k]['new_tests_written'],applied=False,built=False,tested=False) for k,p in paths.items()},
 overlay_files_disjoint=True,overlay_total_files=7,new_tests_written=10,tests_executed=0,builds_executed=0,desktop_inputs=0,
 static_review=str(out/'static-review.json'),static_review_sha256=sha(out/'static-review.json'),
 raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),test_execution_deferred_by_user=True,
 whole_goal_complete=False,GUI_full_flow_verified=False)
(out/'evidence.json').write_bytes(data(record))
date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
work='依使用者要求僅做開發，測試及桌面操作仍暫緩。正常3E來源1049輸入未變；T鍵候選補呈現環境／致盲邊界，新增#20致盲請求與選單候選，沿用原746與S0A。兩候選七檔互斥、合10個已寫案例，均未編譯／測試／套用；實際施放、G06.3d及剩餘GUI仍待，不交人工。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p)
        v.setdefault('isolated_development_history',[]).append(dict(previous_keyboard_candidate=v.get('current_native_keyboard_target_candidate'),prior_desktop_checkpoint=v.get('last_goal_turn_evidence')))
        v['current_native_keyboard_target_candidate']=dict(candidate=str(paths['keyboard']),candidate_sha256=sha(paths['keyboard']),source_manifest_sha256=source,
            applied=False,built=False,tested=False,new_tests_written=4,static_review_path=str(out/'static-review.json'),static_review=review['keyboard'])
        v['current_native_blind20_candidate']=dict(candidate=str(paths['blind20']),candidate_sha256=sha(paths['blind20']),source_manifest_sha256=source,
            applied=False,built=False,tested=False,new_tests_written=6,static_review_path=str(out/'static-review.json'))
        v['current_isolated_development']=record
        v.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        v['current_focus']=dict(item='native_input_and_skill_gap_development',work=work,applies_to_current_source=False,goal_complete=False)
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
print(json.dumps(dict(evidence=str(out/'evidence.json'),base_source_unchanged=True,new_tests_written=10,tests_executed=0),ensure_ascii=False))
