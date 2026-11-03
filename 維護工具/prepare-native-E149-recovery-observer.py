"""Migrate G06 verification sources to fixed E149 and a read-only observer."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
ref=w['current_formal_reference_identity'];target=ref['formal_sha256']
assert target=='E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498'
assert sha(ref['formal'])==sha(ref['cached_reference'])==target
prior=Path(w['current_native_cancel_recovery_candidate']['candidate']);old=read(prior)
assert sha(prior)==w['current_native_cancel_recovery_candidate']['candidate_sha256'] and old['new_tests_written']==14
qpath=Path(w['current_development_typecheck_qualification']['qualification']);q=read(qpath)
assert q['candidates']['cancel-recovery']['sha256']==sha(prior) and q['original_session_id']==85804
inputs=read(Path(q['run'])/'base-compiled-inputs.json');assert all(sha(R/n)==h for n,h in inputs.items())
assert all(sha(prior.parent/n)==h for n,h in old['candidate_files'].items())
formal='LiTo381-Modern/tests/native_teleport_failure.rs'
java='LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java'
observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java'
template=R/'維護工具/templates/NativeRecoveryReadonlySnapshot.java'
assert not(R/observer).exists()
after={n:(prior.parent/n).read_bytes() for n in old['candidate_files']}
text=after[formal].decode('utf-8-sig');before='let expected="75da69b0d2f7f9d394ee6bf2fdb396b153c75d9b938069b0999ed8dac4f50371";'
assert text.count(before)==1;text=text.replace(before,'let expected="'+target.lower()+'";')
anchor='assert_eq!(ready["requested_source"],"測試乙");'
assert text.count(anchor)==1;text=text.replace(anchor,'assert_eq!(ready["observer_kind"],"G06_E149_direct_fields_v1");\n '+anchor)
after[formal]=text.encode('utf-8')
text=after[java].decode('utf-8-sig')
replacements=[
 ('for(var item:inventory.items())ids.add(String.valueOf(item.objectId()));','for(Long item:NativeRecoveryReadonlySnapshot.inventoryIds(inventory))ids.add(String.valueOf(item));'),
 ('NativePeerHarness.observe(session,hub)','NativeRecoveryReadonlySnapshot.observe(session,hub)'),
 ('"{\\"requested_source\\":','"{\\"observer_kind\\":\\"G06_E149_direct_fields_v1\\",\\"requested_source\\":'),
 ('"{\\"status\\":\\"sessions_closed\\",','"{\\"observer_direct_fields\\":true,\\"native_owner_runtime_qualified\\":false,\\"status\\":\\"sessions_closed\\",'),
]
for before,replacement in replacements:
    assert text.count(before)==1,before;text=text.replace(before,replacement)
assert '.items()' not in text and 'NativePeerHarness.observe(' not in text
after[java]=text.encode('utf-8');after[observer]=template.read_bytes()
for n,b in after.items():
    if n not in [formal,java,observer]:assert digest(b)==old['candidate_files'][n]
out=R/'維護工具/candidates'/('native-E149-recovery-observer-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
for n,b in after.items():
    p=out/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b)
    previous=(R/n).read_text('utf-8-sig') if (R/n).exists() else ''
    (out/(Path(n).name+'.diff')).write_text(''.join(difflib.unified_diff(previous.splitlines(True),b.decode('utf-8-sig').splitlines(True),fromfile=n,tofile=n)),encoding='utf-8')
original={n:(R/n).read_bytes() for n in after if (R/n).exists()}
for filename,files in [('before.zip',original),('candidate.zip',after)]:
    with zipfile.ZipFile(out/filename,'x',zipfile.ZIP_DEFLATED) as z:
        for n,b in files.items():z.writestr(n,b)
    with zipfile.ZipFile(out/filename) as z:assert all(z.read(n)==b for n,b in files.items())
closure=V/'native-formal-reference-refresh/20261023-192435-953238/G06-closure-20261023-193307-103834'
review=dict(status='E149_selected_G06_bridge_review_and_observer_development_only',formal_target_sha256=target,
    formal_reference=ref['cached_reference'],initial_comparison=ref['inspection'],initial_comparison_sha256=sha(ref['inspection']),
    closure_outputs={str(p):sha(p) for p in closure.iterdir() if p.is_file()},
    findings=['NativePlayerCommit.assign advances/retire previous viewer and player queues before assigning source/phase; selected also retires item and role owners',
        'NativeDeItemApplication.Owner checks session, object ID, selected character, source and account; adopt rejects capsule identity or a different admission owner',
        'validateReplacement rejects a copy if its live source mutation revision changed',
        'RuntimeInventory.items reaches a NativeDeItemInventory snapshot that may update cached EntryState.view; the new observer reads declared fields instead',
        'Observer takes only one inventory monitor at a time, records repeated direct-field observations/revisions and never treats stability as an atomic authority grant',
        'Existing memory GameHub constructor does not call NativeDeItemApplication.bootstrap/install; activation is therefore observed and remains a separate production-resource qualification gate',
        'Three existing new formal scenarios retain repeated cancellation and real 33s stage plus 31s fresh source waits; no clocks or item ownership are rewritten'],
    limitations=['Selected wire/source handoff methods match, but no full transitive/server runtime compatibility is inferred',
        'New production resource startup and full active item-owner lifecycle still need isolated real initialization and runtime verification',
        'NativeDe item raw bless and cached packet bless are recorded separately; wire/complete item comparison remains required'],
    tests_executed=0,game_or_fixture_executions=0,GUI_verified=False,whole_goal_complete=False)
(out/'contract.json').write_bytes(data(review))
record=dict(old,status='isolated_E149_G06_observer_migration_not_built_not_tested',
    predecessor_candidate=str(prior),predecessor_candidate_sha256=sha(prior),predecessor_metadata_qualification=str(qpath),
    predecessor_metadata_does_not_qualify_this_revision=True,formal_target_sha256=target,formal_reference=ref['cached_reference'],
    candidate_files={n:digest(b) for n,b in after.items()},base_files={n:digest(b) for n,b in original.items()},new_files=[observer],
    test_files=old['test_files']+[observer],new_tests_in_this_revision=0,new_library_cases_in_this_revision=0,new_formal_integration_cases=3,
    observer_template=str(template),observer_template_sha256=sha(template),
    before_archive_sha256=sha(out/'before.zip'),candidate_archive_sha256=sha(out/'candidate.zip'),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),
    changes=['All three Rust runtime/pure/wire files remain byte-identical to the preceding compiled G06 candidate',
        'Fix formal integration expectation to E149 and require its narrow observer readiness tag',
        'Replace broad observer and inventory.items with direct-fields inventory/session/stage snapshots without changing cache views or reading gameplay timers',
        'Record native-owner installation, session/character/source/account binding, raw item attributes and mutation revisions; do not infer full owner qualification'],
    actual_failure_reproduction_executed=False,builds_executed=0,tests_executed=0,production_applied=False,GUI_verified=False,whole_goal_complete=False)
for key in ['baseline_regression_tests','baseline_wire_regression_tests']:
    dest=out/Path(old[key]).name;dest.write_bytes(Path(old[key]).read_bytes());record[key]=str(dest);record[key+'_sha256']=sha(dest)
(out/'candidate.json').write_bytes(data(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
(out/'README.md').write_text('# E149 G06恢復驗證遷移\n\nRuntime／原元件案例維持前版位元組；正式Rust期待固定E149，Java改用限定的只讀快照。不要呼叫會更新NativeDe背包view的items()，也不借用會查詢遊戲狀態時鐘的通用observer。\n\n記錄新版item application是否安裝、session／object／character／source／account、背包owner及原始欄位／revision。兩次讀取相同僅是觀察穩定，不能視作原子授權快照。原memory GameHub沒有production resource bootstrap，不跳過／偽造owner，也不能以此授予完整新版owner資格。\n\n三候選合計13檔／24已寫案例；本G06組6檔（1 Runtime、5 QA），14案（11元件＋3新正式）、6原版元件反例。未編譯、未執行、未套用；Rust、Java對E149編譯後仍須正式runtime／完整library／Release／GUI。\n',encoding='utf-8')
assert all(sha(R/n)==h for n,h in inputs.items()) and sha(ref['formal'])==target
print(json.dumps(dict(candidate=str(out/'candidate.json'),sha256=sha(out/'candidate.json'),formal_target=target,files=len(after),tests_executed=0),ensure_ascii=False))
