"""Record current formal drift without transferring old runtime qualification."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
inspection=V/'native-formal-reference-refresh/20261023-192435-953238/evidence.json';i=read(inspection)
assert sha(i['formal'])==i['formal_sha256']==sha(i['cached_reference'])==i['cached_reference_sha256']
assert i['formal_sha256']=='E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
assert not i['runtime_compatibility_verified'] and i['tests_executed']==i['game_or_fixture_executions']==i['desktop_inputs']==0
assert sha(i['raw_archive'])==i['raw_archive_sha256']
qpath=Path(w['current_development_typecheck_qualification']['qualification']);q=read(qpath)
assert q['original_session_id']==85804 and q['terminal_tool_chunk']=='8467bb' and q['compiler_invocations']==12
assert all(sha(R/n)==h for n,h in read(Path(q['run'])/'base-compiled-inputs.json').items())
assert all(sha(p)==h for p,h in q['normal_product_pins'].items())
callee=inspection.parent/'callee-review-20261023-192757-011398'
assert all(row['exit_code']==0 for row in read(callee/'commands.json'))
out=inspection.parent/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
review=dict(scope='static follow-up only, not a behavioral or transitive equivalence proof',formal_sha256=i['formal_sha256'],
    inspected_callees=['NativePlayerCommit.scene -> assign; full owner/commit closure remains to review',
        'NativeDeItemApplication.replace may adopt/validate inventory ownership and reject stale assignment; these semantics are not covered by the old plain-field qualification',
        'characterSelect(byte[],boolean) now wraps sourcePoisonBody$characterSelect with CombatPoisonLifecycle.open/finish/failed; delegated body and retirement closure still require review'],
    remaining=['Trace NativePlayerCommit.assign and ownership invalidation on source/phase change',
        'Trace NativeDeItemApplication.Owner/adopt/validateReplacement and destination item continuity',
        'Trace sourcePoisonBody$characterSelect and scope completion/rollback',
        'Derive a fixed E149 fixture/client expectation without copying old runtime results',
        'When user resumes tests, execute current-target formal recovery/GUI with actual natural waits and full wire/server/item readback'],
    javap_outputs={str(p):sha(p) for p in callee.iterdir() if p.is_file()},tests_executed=0,fixture_classes_executed=0,
    candidate_Java_compiled_against='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371',
    current_E149_Java_compilation_completed=False,current_E149_runtime_qualified=False)
(out/'callee-review.json').write_bytes(data(review))
record=dict(status='current_formal_changed_E149_migration_required_tests_deferred',formal=str(i['formal']),formal_sha256=i['formal_sha256'],
    cached_reference=i['cached_reference'],cached_reference_sha256=i['cached_reference_sha256'],previous_formal_sha256=i['previous_formal_sha256'],
    source_manifest_sha256=w['current_product_identity']['source_manifest_sha256'],inspection=str(inspection),inspection_sha256=sha(inspection),
    selected_methods_identical={n:sum(v['constant_pool_normalized_identical'] for v in rows.values()) for n,rows in i['selected_method_comparison'].items()},
    selected_methods_compared={n:len(rows) for n,rows in i['selected_method_comparison'].items()},
    callee_review=str(out/'callee-review.json'),callee_review_sha256=sha(out/'callee-review.json'),
    candidate_typecheck_qualification=str(qpath),candidate_typecheck_qualification_sha256=sha(qpath),
    current_formal_compatibility_verified=False,old_75_runtime_qualifications_transfer=False,
    tests_executed=0,game_or_fixture_executions=0,desktop_inputs=0,normal_source_modified=False,normal_EXEs_modified=False,
    test_execution_deferred_by_user=True,previous_goal_turn_classification='progress',previous_goal_turn_evidence=w['last_goal_turn_evidence'],
    whole_goal_complete=False,recorded_utc=datetime.now(timezone.utc).isoformat())
raw={'inspection.json':inspection,'inspection-raw.zip':Path(i['raw_archive']),'callee-review.json':out/'callee-review.json',
    'typecheck-qualified.json':qpath,'checkpoint.py':Path(__file__)}
for p in callee.iterdir():
    if p.is_file():raw['callees/'+p.name]=p
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in raw.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items())
record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'))
(out/'evidence.json').write_bytes(data(record))
work='正式JAR已由75DA69B0變為E1498871，已固定新參考並靜態比對：GameHub來源／租裝交接6及Protocol8方法一致，ClientSession10僅3一致，新增場景提交／物品owner／選角生命週期函式仍待完整審查。現行三候選12檔／24案例的Rust兩版各6項與Java3class只完成編譯，Java仍綁75；原75的API／GUI資格保留原範圍，不移記E149。下一步先續E149委派函式與fixture契約遷移，依使用者要求桌面／測試暫緩；恢復後驗6個原版反例、11元件＋3新正式案及實際自然33／31秒、E149相關API／完整library／Release／GUI，整體未完成。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_formal_reference_identity']=record
        j['current_product_identity'].update(formal_target_sha256=i['formal_sha256'],current_formal_target_runtime_qualification_pending=True)
        j['current_isolated_development'].update(formal_target_migration_pending=True,current_formal_target_sha256=i['formal_sha256'],current_formal_target_runtime_qualified=False)
        for k in ['current_native_cancel_recovery_candidate','current_native_keyboard_target_candidate','current_native_blind20_candidate']:
            j[k].update(current_formal_target_sha256=i['formal_sha256'],current_formal_target_qualified=False,formal_migration_review=str(out/'callee-review.json'))
        j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='G06_3d_current_E149_formal_contract_migration',work=work,applies_to_current_source=False,goal_complete=False)
        j['current_source_goal_validation'].setdefault('G06-room-service-branches',{}).update(complete=False,current_formal_target_sha256=i['formal_sha256'],current_formal_target_qualified=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep
        staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert sha(i['formal'])==i['formal_sha256'] and all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),formal_sha256=i['formal_sha256'],runtime_compatibility_verified=False,tests_executed=0),ensure_ascii=False))
