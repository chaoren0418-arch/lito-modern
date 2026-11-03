"""Read back integrated fixture compilation; no runtime qualification."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-E149-recovery-bootstrap-compile')
cohort=read(run/'cohort.json');cp=Path(cohort['candidate']);c=read(cp)
assert cohort['status']=='integrated_fixture_compilation_only_complete' and sha(cp)==cohort['candidate_sha256']
assert len(cohort['process_history'])==3 and [r['exit_code'] for r in cohort['process_history']]==[0,0,0]
assert all(sha(p)==h for p,h in cohort['input_pins'].items()) and len(cohort['classes'])==6
assert all(sha(p)==h for p,h in cohort['classes'].items())
for row in cohort['process_history']:
    assert all(sha(p)==h for p,h in row['logs'].items())
    assert Path(row['command'][0]).name in ['javac.exe','rustc.exe']
    if row['name']=='Java':assert '-proc:none' in row['command']
    else:assert '--emit=metadata' in row['command'] and '--test' in row['command']
for p,m in cohort['metadata'].items():assert sha(p)==m['sha256'] and Path(p).stat().st_size==m['bytes']==0
assert cohort['tests_executed']==cohort['classes_instantiated']==cohort['desktop_inputs']==0
assert not cohort['SQL_executed'] and not cohort['database_started'] and not cohort['original_factories_invoked']
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
ref=w['current_formal_reference_identity'];assert sha(ref['formal'])==sha(c['formal_reference'])==c['formal_target_sha256']
inputs=read(cohort['base_compiled_inputs']);assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
assert all(sha(p)==h for p,h in cohort['normal_product_pins'].items())
qpath=Path(cohort['reused_library_metadata_qualification']);q=read(qpath);assert sha(qpath)==cohort['reused_library_metadata_qualification_sha256']
assert all(q['candidates'][k]['sha256']==v['sha256'] for k,v in w['current_isolated_development']['candidates'].items())
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())

out=V/'native-E149-recovery-bootstrap-development'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
(out/'reader.py').write_bytes(Path(__file__).read_bytes())
commands=[];disassembly={}
for name in ['NativeTeleportFailureHarness','NativeRecoveryProductionBootstrap','NativeRecoveryProductionBootstrap$Session','NativeRecoveryReadonlySnapshot']:
    command=[r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javap.exe','-J-Dfile.encoding=UTF-8','-p','-c',
        '-classpath',str(run/'classes')+';'+c['formal_reference'],'local.lito381.server.'+name]
    result=subprocess.run(command,capture_output=True,cwd=out,creationflags=subprocess.CREATE_NO_WINDOW)
    (out/(name+'.stdout.txt')).write_bytes(result.stdout);(out/(name+'.stderr.txt')).write_bytes(result.stderr)
    commands.append(dict(command=command,exit_code=result.returncode));assert result.returncode==0
    disassembly[name]=result.stdout.decode('utf-8-sig')
(out/'commands.json').write_bytes(data(commands))
adapter=disassembly['NativeRecoveryProductionBootstrap'];fixture=disassembly['NativeTeleportFailureHarness'];observer=disassembly['NativeRecoveryReadonlySnapshot']
chain=['NativeServerLifetime.install:','NativeDeItemTemplatesResources.read:','NativeDeItemDefinitionsResources.read:','NativeDeItemApplication.install:']
positions=[adapter.index(n) for n in chain];assert positions==sorted(positions)
for n in ['NativeServerLifetime.bindMain:','NativeServerLifetime.bindSource:','NativeServerLifetime.acceptSocket:',
          'NativeServerLifetime.submitAccepted:','NativeServerLifetime.closeHub:','NativeServerLifetime.closeResource:','NativeServerLifetime.finishClients:',
          'java/sql/DriverManager.getConnection:']:assert n in adapter,n
assert 'java/util/concurrent/Future.get:' in disassembly['NativeRecoveryProductionBootstrap$Session']
for n in ['NativeRecoveryProductionBootstrap.open:','NativeRecoveryProductionBootstrap.bindLogin:','NativeRecoveryProductionBootstrap.bindGame:',
          'NativeRecoveryProductionBootstrap.accept:','NativeRecoveryProductionBootstrap.close:','NativeRecoveryProductionBootstrap$Session.await:']:
    assert n in fixture,n
assert 'java/net/ServerSocket.accept:' not in fixture and 'ClientSession."<init>"' not in fixture
assert 'GameHub.close:' not in adapter and 'ExecutorService.shutdownNow:' not in adapter
assert all(not re.search(r'java/lang/reflect/Field\.(?:set:|setInt:|setLong:|setBoolean:)',v) for v in [adapter,fixture,observer])
assert 'java/lang/reflect/Proxy' not in adapter and 'RuntimeInventory.items:' not in observer+fixture

# Observer is identical to the already-reviewed E149 source except accepting
# the actual ClientSession directly. Confirm its compiled original-method call
# set also matches, excluding the superseded Thread Session record accessor.
prior=Path(c['predecessor_candidate']);old=read(prior)
observer_name='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java'
old_source=(prior.parent/observer_name).read_text('utf-8-sig')
old_signature='static String observe(NativePeerHarness.Session session,GameHub hub)throws ReflectiveOperationException {\n        ClientSession c=session.owner();'
assert old_source.count(old_signature)==1
assert old_source.replace(old_signature,'static String observe(ClientSession c,GameHub hub)throws ReflectiveOperationException {')==(cp.parent/observer_name).read_text('utf-8-sig')
review_path=Path(w['current_E149_recovery_development']['readonly_observer_review']);review=read(review_path)
assert sha(review_path)==w['current_E149_recovery_development']['readonly_observer_review_sha256']
assert review['no_gameplay_method_calls_in_observer'] and review['no_reflective_game_field_setters'] and review['no_view_cache_refresh']
calls=lambda lines:{m.group(1) for line in lines if (m:=re.search(r'// (?:InterfaceMethod|Method) (.+)',line))}
def original_calls(lines):
    return {x for x in calls(lines) if x.split(':',1)[0].startswith('local/lito381/server/') and 'NativePeerHarness$Session.owner:' not in x}
assert original_calls(observer.splitlines())==original_calls(review['external_method_calls'])
fixture_source=(cp.parent/'LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java').read_text('utf-8-sig')
assert fixture_source.index('app=NativeRecoveryProductionBootstrap.open(')<fixture_source.index('app.bindLogin(')<fixture_source.index('G06_E149_resources_direct_fields_v2')
assert fixture_source.index('app.close();')<fixture_source.index('NativePeerHarness.write(root.resolve("result.json")')
assert 'new Thread(owner' not in fixture_source and 'full_game_reentry_verified' not in fixture_source
rust=(cp.parent/'LiTo381-Modern/tests/native_teleport_failure.rs').read_text('utf-8-sig')
assert 'continuation-observe-request' in rust and 'continuation-observed.json' in rust
assert 'continuation changed before observation' in rust
assert '"LOBBY".equals(String.valueOf(NativePeerHarness.field(current,"phase")))' in fixture_source
assert 'NativePeerHarness.write(observed,row)' in fixture_source and 'trace.write(row+' in fixture_source
for n in ['original_resource_factories_initialized','native_server_lifetime_installed','static_sql_access','player_sql_access','static_tables','static_rows','full_main_process_qualified']:
    assert 'assert_eq!(ready["'+n+'"]' in rust

failed=V/'native-E149-recovery-bootstrap-compile/20261023-204623-604762';fc=read(failed/'cohort.json')
assert fc['status']=='compile_failure_preserved' and len(fc['process_history'])==1 and fc['process_history'][0]['exit_code']==1
assert 'variable file is already defined' in (failed/'Java/stderr.log').read_text('utf-8-sig')
assert all(sha(p)==h for p,h in fc['input_pins'].items()) and fc['tests_executed']==fc['classes_instantiated']==0
identities=[cohort['pipeline'],*cohort['process_history'],fc['pipeline'],*fc['process_history']]
ids=','.join(str(x['pid']) for x in identities)
query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed=subprocess.check_output(['pwsh','-NoProfile','-Command',query]);(out/'owned-CIM.json').write_bytes(observed)
actual=json.loads(observed.decode('utf-8-sig')) if observed.strip() else [];actual=[actual] if isinstance(actual,dict) else actual
for row in identities:assert not any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in actual)
record=dict(status='integrated_E149_fixture_Java_and_Rust_metadata_compilation_only',candidate=str(cp),candidate_sha256=sha(cp),
    compile_run=str(run),compile_cohort_sha256=sha(run/'cohort.json'),observed_original_tool_chunk=a.terminal_chunk,observed_actual_exit_code=0,
    formal_target_sha256=c['formal_target_sha256'],source_manifest_sha256=c['base_source_manifest_sha256'],
    Java_classes=cohort['classes'],Java_compiler_exit_code=0,Rust_formal_metadata=cohort['metadata'],Rust_compiler_exit_codes=[0,0],
    reused_library_metadata_qualification=str(qpath),reused_library_metadata_qualification_sha256=sha(qpath),
    original_factory_chain_statically_verified=chain,original_native_accept_submit_future_cleanup_present=True,
    observer_body_unchanged_except_session_argument=True,observer_prior_readonly_review=str(review_path),observer_prior_readonly_review_sha256=sha(review_path),
    ready_requires_original_resource_initialization=True,result_after_cleanup_barrier=True,
    continuation_observation_barrier_source_complete=True,server_continuation_phase='LOBBY',client_continuation_phase='character_selected',
    failed_compile_run=str(failed),failed_compile_cohort_sha256=sha(failed/'cohort.json'),failed_compile_original_tool_chunk='b507e9',failed_compile_exit_code=1,
    original_identities_terminal=True,original_identity_count=len(identities),
    tests_executed=0,classes_instantiated=0,SQL_executed=False,database_started=False,original_factories_invoked=False,desktop_inputs=0,
    normal_source_unchanged=True,normal_PEs_unchanged=True,existing_thirteen_file_overlay_unchanged=True,
    standalone_overlay_files=4,production_applied=False,fixture_integration_source_complete=True,fixture_runtime_qualified=False,
    full_Main_process_qualified=False,full_native_owner_qualified=False,future_owned_case_orchestrator_pending=True,
    limitations=['Compilers and javap do not instantiate fixture classes or run native lifetime/factory/JDBC behavior',
        'Two Rust invocations cover only changed formal-case readiness assertions against existing qualified candidate library metadata; full library not rerun',
        'Six new classes do not establish DB startup/import, resource activation, session/wire/item continuity or terminal native cleanup',
        'Actual owner/import proofs, pinned JDBC runtime classpath and an owned case orchestrator still required',
        'Natural 33/31-second scenarios and current E149/full library/Release/full GUI remain unexecuted'],
    test_execution_deferred_by_user=True,whole_goal_complete=False,recorded_utc=datetime.now(timezone.utc).isoformat())
rejected=V/'native-E149-recovery-bootstrap-development/20261023-204904-101687'
assert read(rejected/'reader-rejection.json')['original_tool_chunk']=='5d3997'
record.update(prior_reader_rejection=str(rejected/'reader-rejection.json'),prior_reader_rejection_sha256=sha(rejected/'reader-rejection.json'),
    prior_reader_rejection_is_not_product_failure=True,
    map_factory_inspection=str(V/'native-E149-recovery-bootstrap-inspection/20261023-204127-462946/commands.json'),
    initial_javap_shell_argument_failure_original_chunk='96acda')
phase=V/'native-E149-recovery-reader-inspection/20261023-210435-261426'
assert 'Phase LOBBY;' in (phase/'phase.stdout.txt').read_text('utf-8-sig') and 'CHARACTER_SELECTED' not in (phase/'phase.stdout.txt').read_text('utf-8-sig')
discarded=T/'candidates/native-E149-recovery-bootstrap-20261023-210359-927625/candidate.json'
record.update(server_phase_inspection=str(phase/'command.json'),server_phase_inspection_sha256=sha(phase/'command.json'),
    superseded_uncompiled_phase_candidate=str(discarded),superseded_uncompiled_phase_candidate_sha256=sha(discarded),
    superseded_phase_candidate_reason='Static enum inspection: client CHARACTER_SELECTED corresponds to server LOBBY; rejected before any compilation or test execution')
(out/'qualified-development.json').write_bytes(data(record))
raw={'reader.py':Path(__file__),'candidate.json':cp,'contract.json':Path(c['contract']),'candidate.zip':cp.parent/'candidate.zip',
    'compiler.py':run/'compiler.py','cohort.json':run/'cohort.json','qualified-development.json':out/'qualified-development.json',
    'failed-cohort.json':failed/'cohort.json','failed-compiler.py':failed/'compiler.py','failed-stderr.log':failed/'Java/stderr.log',
    'failed-stdout.log':failed/'Java/stdout.log','failed-candidate.json':Path(fc['candidate']),
    'report-updater-before.py':T/'update-master-report-text-index-order.py','commands.json':out/'commands.json','owned-CIM.json':out/'owned-CIM.json'}
raw.update({'rejected-reader.py':rejected/'reader.py','reader-rejection.json':rejected/'reader-rejection.json',
    'map-factory-commands.json':Path(record['map_factory_inspection']),
    'map-factory.stdout.txt':Path(record['map_factory_inspection']).with_name('LitoMapRepository.stdout.txt')})
raw.update({'server-phase-command.json':phase/'command.json','server-phase.stdout.txt':phase/'phase.stdout.txt',
    'superseded-phase-candidate.json':discarded,'superseded-phase-candidate.zip':discarded.parent/'candidate.zip'})
for p in cohort['classes']:raw['classes/'+Path(p).name]=Path(p)
for row in cohort['process_history']:
    for p in row['logs']:raw[row['name']+'/'+Path(p).name]=Path(p)
for name in disassembly:
    for ext in ['stdout.txt','stderr.txt']:raw['bytecode/'+name+'.'+ext]=out/(name+'.'+ext)
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in raw.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in raw.items())
record.update(qualification=str(out/'qualified-development.json'),qualification_sha256=sha(out/'qualified-development.json'),raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),previous_goal_turn_evidence=w['last_goal_turn_evidence'])
(out/'evidence.json').write_bytes(data(record))
work='E149獨立G06 fixture補上真實接續觀測確認點：客戶端收到原物品後，等伺服器實際寫下穩定LOBBY／選角／active owner，再進下一個租裝房，避免20ms取樣漏掉接續階段；不改遊戲時鐘／租期。原'+a.terminal_chunk+' exit0：Java6class＋Debug／Release正式案例metadata各1，原程序身分皆終止。原先CHARACTER_SELECTED不屬伺服器enum，已在編譯前靜態退件，原稿與enum證據保留；原編譯失敗／reader拒絕亦保留。1049正常輸入、兩EXE及13檔舊候選不變，新4檔未套用。使用者暫緩DB／SQL／factory／功能／GUI測試。接續完成owned case orchestrator、獨立reader與隔離映像建置門檻；恢復測試後才驗原版6反例／候選14案、33／31秒、最終library／Release／完整GUI，整體未完成。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_E149_integrated_fixture_development']=record
        j['current_E149_resource_bootstrap_development'].update(future_fixture_integration_pending=False,
            integrated_fixture_development=record['qualification'],integrated_fixture_development_sha256=record['qualification_sha256'],integrated_fixture_runtime_qualified=False)
        j['current_E149_recovery_development'].update(standalone_integrated_fixture_development=record['qualification'],standalone_integrated_fixture_development_sha256=record['qualification_sha256'])
        j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='G06_3d_E149_original_lifetime_fixture_integration_development',work=work,applies_to_current_source=False,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep
        staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items()) and all(sha(R/n)==h for n,h in inputs.items()) and sha(ref['formal'])==ref['formal_sha256']
for n,b in staged.items():(R/n).write_bytes(b)
assert all((R/n).read_bytes()==b for n,b in staged.items())
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),Java_classes=6,Rust_formal_metadata_targets=2,tests_executed=0,SQL_executed=False,eight_files_read_back=True),ensure_ascii=False))
