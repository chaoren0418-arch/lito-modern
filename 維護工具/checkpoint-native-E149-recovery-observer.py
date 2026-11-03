"""Publish concrete G06 recovery development, retaining all old qualifications."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib, json, zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
candidate=R/'維護工具/candidates/native-E149-recovery-observer-20261023-194101-605228/candidate.json'
c=read(candidate);assert sha(candidate)=='D73662E11AF734AECB693F3871506EBD91170BCE2BD713D1EB376B74CC4BC21F'
prior=Path(c['predecessor_candidate']);assert sha(prior)==c['predecessor_candidate_sha256']
assert str(prior)==w['current_native_cancel_recovery_candidate']['candidate']
qpath=Path(w['current_development_typecheck_qualification']['qualification']);q=read(qpath)
source=w['current_product_identity']['source_manifest_sha256'];assert source==c['base_source_manifest_sha256']==q['source_manifest_sha256']
inputs=read(Path(q['run'])/'base-compiled-inputs.json');assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
assert all(sha(p)==h for p,h in q['normal_product_pins'].items())
assert c['tests_executed']==c['builds_executed']==0 and not c['production_applied']
assert all(sha(R/n)==h for n,h in c['base_files'].items())
assert all(sha(candidate.parent/n)==h for n,h in c['candidate_files'].items())
for k in ['before_archive','candidate_archive']:
    assert sha(candidate.parent/('before.zip' if k=='before_archive' else 'candidate.zip'))==c[k+'_sha256']
for k in ['baseline_regression_tests','baseline_wire_regression_tests','contract']:
    assert sha(c[k])==c[k+'_sha256']
contract=read(c['contract'])
assert sha(contract['initial_comparison'])==contract['initial_comparison_sha256']
assert all(sha(p)==h for p,h in contract['closure_outputs'].items())
assert c['formal_target_sha256']==w['current_formal_reference_identity']['formal_sha256']==sha(w['current_formal_reference_identity']['formal'])
assert all(not(R/n).exists() for n in c['new_files'])
runtime=c['runtime_files'][0];pure,wire,formal,java,observer=c['test_files']
old=read(prior)
assert (candidate.parent/pure).read_bytes().startswith((prior.parent/pure).read_bytes())
assert (candidate.parent/wire).read_bytes().startswith((R/wire).read_bytes())
count=lambda b:b.count(b'#[test]')+b.count(b'#[cfg_attr(test, test)]')
assert sum(count((candidate.parent/n).read_bytes())-count((R/n).read_bytes() if (R/n).exists() else b'') for n in c['test_files'])==14
assert count(Path(c['baseline_regression_tests']).read_bytes())-count((R/pure).read_bytes())==3
assert count(Path(c['baseline_wire_regression_tests']).read_bytes())-count((R/wire).read_bytes())==3
body=(candidate.parent/runtime).read_text('utf-8')
helper=body.split('    fn refresh_expired_recovery_source(&mut self)->Result<()> {',1)[1].split('    fn begin_rental_return_reconnect',1)[0]
assert helper.count('self.transport.send(')==1 and 'source_request(source)?' in helper
assert helper.index('self.transport.send(')<helper.index('self.recovery_character=None')
assert 'self.state.phase=LoginPhase::LoadingCharacters' in helper and 'Duration::from_secs(8)' in helper
assert 'self.state.phase=LoginPhase::CharactersReady' not in helper and 'self.transport.start' not in helper
assert all(s in helper for s in ['self.state.inventory.batches==0','self.state.character_result.is_none()', 'self.teleport.is_none()', 'self.state.sources.iter().any', 'self.state.characters.clear()', 'self.directory=super::native_game::Directory::default()'])
ftext=(candidate.parent/formal).read_text('utf-8');jtext=(candidate.parent/java).read_text('utf-8')
obs=(candidate.parent/observer).read_text('utf-8')
assert '.items()' not in jtext and 'NativePeerHarness.observe(' not in jtext
assert 'NativeRecoveryReadonlySnapshot.observe(session,hub)' in jtext and 'NativeRecoveryReadonlySnapshot.inventoryIds(inventory)' in jtext
assert '.items()' not in obs and '.set(' not in obs and 'poisonActive' not in obs
assert all(s in obs for s in ['mutationRevision','source_item_application_active','raw_native_bless','sourceItemOwner','full_native_owner_qualification_inferred'])
assert c['formal_target_sha256'].lower() in ftext
assert sha(c['observer_template'])==c['observer_template_sha256']
assert all(c['candidate_files'][n]==old['candidate_files'][n] for n in [runtime,pure,wire])
assert ftext.count('c.select_source(0)')==0 and ftext.count('position(|s|s.name=="測試乙")')==2
assert 'Duration::from_secs(31)' in ftext and 'Duration::from_secs(33)' in ftext
assert 'login.classic.server.names=測試甲,測試乙' in jtext
assert 'authority_mutations\\\":0' in jtext and 'database.read.only=true' in jtext and 'state.persistence.enabled=false' in jtext
registry={k:dict(v) for k,v in w['current_isolated_development']['candidates'].items()}
registry['cancel-recovery']=dict(path=str(candidate),sha256=sha(candidate),files=6,tests_written=14,
    applied=False,built=False,tested=False,metadata_typechecked=False,predecessor_metadata_qualification=str(qpath))
seen=set();runtime_count=test_count=0
for entry in registry.values():
    p=Path(entry['path']);ca=read(p);assert sha(p)==entry['sha256']
    assert not seen.intersection(ca['candidate_files']);seen.update(ca['candidate_files'])
    assert all(sha(p.parent/n)==h for n,h in ca['candidate_files'].items())
    runtime_count+=len(ca['runtime_files']);test_count+=len(ca['test_files'])
assert len(seen)==13 and runtime_count==6 and test_count==7
out=V/'native-development-continuation'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir()
review=dict(scope='static source and retained formal bytecode only; no compiler/test/GUI execution',
    candidate=str(candidate),candidate_sha256=sha(candidate),source_manifest_sha256=source,
    defect=c['static_defect'],contract=c['contract'],contract_sha256=c['contract_sha256'],
    reviewed=['Predecessor cancellation and one-shot binding fixes retained, including all six appended cases',
        'The unchanged response deadline or a complete missing-character first recovery directory can trigger one refresh; explicit destination teleports still reject',
        'Only the exact advertised selected source is sent once through ordinary GAME C8; no auto-login, new transport or C9',
        'Unconfirmed directory and selection are retired only after enqueue succeeds; failure invokes existing full fail retirement',
        'Fresh S03/SCB completion is required before manual choice; partial admission and invalid context cannot retry',
        'Original game readiness, source/account authentication, staged handoff ordering, server 30s TTL and rental lifetimes remain untouched',
        'Eleven written component cases plus three new ignored formal cases cover source distinction, repeated cancel and actual dual TTL waits; no case executed',
        'Synthetic response-deadline fixtures do not claim natural 30s or rental expiry; late actual admission race still requires formal execution'],
    static_review_does_not_prove_runtime_defect=True,tests_executed=0,metadata_typechecked=False,production_applied=False,whole_goal_complete=False)
(out/'static-review.json').write_bytes(data(review))
raw={'candidate.json':candidate,'candidate.zip':candidate.parent/'candidate.zip','before-source.zip':candidate.parent/'before.zip',
    'preparer.py':candidate.parent/'preparer.py','baseline-regression-tests.rs':Path(c['baseline_regression_tests']),
    'baseline-wire-regression-tests.rs':Path(c['baseline_wire_regression_tests']),'contract.json':Path(c['contract']),
    'observer-template.java':Path(c['observer_template']),'static-review.json':out/'static-review.json',
    'checkpoint.py':Path(__file__),'predecessor-candidate.json':prior,'predecessor-typecheck.json':qpath,
    'base-compiled-inputs.json':Path(q['run'])/'base-compiled-inputs.json'}
preparation_failures=[]
checkpoint_failure=V/'native-development-preparation-failures/E149-observer-new-QA-file-98538d'
for p in checkpoint_failure.iterdir():
    assert p.is_file();raw['E149-checkpoint-failure/'+p.name]=p
preparation_failures.append(dict(path=str(checkpoint_failure/'failure.json'),sha256=sha(checkpoint_failure/'failure.json'),original_tool_chunk='98538d'))
for suffix in ['default-source-line-endings-57991e','default-source-prefix-0ca238']:
    folder=V/'native-development-preparation-failures'/suffix
    failure=read(folder/'failure.json');assert failure['actual_exit_code']==1 and failure['tests_executed']==0
    preparation_failures.append(dict(path=str(folder/'failure.json'),sha256=sha(folder/'failure.json'),original_tool_chunk=failure['original_tool_chunk']))
    for p in folder.iterdir():
        assert p.is_file();raw['preparation-failure/'+suffix+'/'+p.name]=p
partial=R/'維護工具/candidates/native-recovery-default-source-20261023-191718-408389'
assert not(partial/'candidate.json').exists()
for n in ['candidate.zip','before.zip']:
    raw['partial-preparation/'+n]=partial/n
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in raw.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items())
record=dict(status='G06_E149_recovery_readonly_observer_development_tests_deferred',source_manifest_sha256=source,
    previous_goal_turn_classification='progress',previous_goal_turn_evidence=w['last_goal_turn_evidence'],
    candidates=registry,overlay_files_disjoint=True,overlay_total_files=13,runtime_files=6,test_files=7,new_tests_written=24,
    tests_executed=0,builds_executed=0,desktop_inputs=0,metadata_typecheck_matches_current_registry=False,
    predecessor_metadata_typecheck=str(qpath),new_revision_requires_metadata_compilation=True,formal_target_migration_pending=True,current_formal_target_sha256=c['formal_target_sha256'],current_formal_target_runtime_qualified=False,
    static_review=str(out/'static-review.json'),static_review_sha256=sha(out/'static-review.json'),
    raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),test_execution_deferred_by_user=True,
    preparation_failures=preparation_failures,partial_preparation_retained=str(partial),
    GUI_full_flow_verified=False,whole_goal_complete=False,recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'evidence.json').write_bytes(data(record))
work='G06的Rust正式期待已改固定E149，Java新增限定只讀observer，避開通用遊戲時鐘getter與會更新view的items()，記錄新版item application／owner、session／角色／來源／帳號、原始物品欄位及revision。Runtime／11元件與3新正式案維持前版；三候選13檔／24案，這次observer遷移未編譯、未執行、未套用。原memory GameHub未安裝production item application，完整新版owner生命周期資格另需原始resource bootstrap，不能由snapshot或metadata推論。依使用者要求桌面／測試暫緩，先續Rust／Java E149隔離編譯與開發；再正式原版6反例／14案、實際自然33／31秒及最終library／Release／完整GUI，整體未完成。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j.setdefault('isolated_development_history',[]).append(dict(prior_registry=j['current_isolated_development'],prior_evidence=j['last_goal_turn_evidence']))
        current=dict(candidate=str(candidate),candidate_sha256=sha(candidate),source_manifest_sha256=source,applied=False,built=False,tested=False,
            metadata_typechecked=False,new_tests_written=14,new_tests_in_this_revision=0,baseline_additional_cases=6,
            static_review_path=str(out/'static-review.json'),predecessor_metadata_qualification=str(qpath))
        j['current_native_cancel_recovery_candidate']=current
        current.update(formal_target_sha256=c['formal_target_sha256'],current_formal_target_qualified=False)
        j['current_E149_recovery_development']=dict(candidate=str(candidate),candidate_sha256=sha(candidate),contract=c['contract'],contract_sha256=c['contract_sha256'],formal_target_sha256=c['formal_target_sha256'],compile_complete=False,runtime_qualified=False,full_native_owner_initialization_pending=True)
        j['current_isolated_development']=record
        j['current_development_typecheck_qualification']['matches_current_candidate_registry']=False
        j['current_development_typecheck_active']['status']='historical_terminal_predecessor_overlay_only'
        j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='G06_3d_expired_recovery_source_confirmation',work=work,applies_to_current_source=False,goal_complete=False)
        j['current_source_goal_validation'].setdefault('G06-room-service-branches',{}).update(complete=False,development_candidate=current)
        for item in j.get('items',[]):
            if item.get('id')=='G06-room-service-branches':item.update(complete=False,development_candidate=current)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep
        staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),candidate=str(candidate),candidate_files=13,cases_written=24,tests_executed=0,normal_source_unchanged=True),ensure_ascii=False))
