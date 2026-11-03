"""Prepare a separate integrated fixture overlay; never apply or run it."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];T=R/'維護工具'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
old_path=Path(w['current_E149_recovery_development']['candidate']);old=read(old_path)
assert sha(old_path)==w['current_E149_recovery_development']['candidate_sha256']
adapter_path=Path(w['current_E149_resource_bootstrap_development']['candidate']);adapter=read(adapter_path)
assert sha(adapter_path)==w['current_E149_resource_bootstrap_development']['candidate_sha256']
q=read(w['current_development_typecheck_qualification']['qualification']);inputs=read(Path(q['run'])/'base-compiled-inputs.json')
assert all(sha(R/n)==h for n,h in inputs.items())
java='LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java'
observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java'
bootstrap='LiTo381-Modern/tests/oracle/NativeRecoveryProductionBootstrap.java'
rust='LiTo381-Modern/tests/native_teleport_failure.rs'
after={java:(T/'templates/NativeTeleportFailureBootstrapHarness.java').read_bytes()}
text=(old_path.parent/observer).read_text('utf-8-sig')
before='static String observe(NativePeerHarness.Session session,GameHub hub)throws ReflectiveOperationException {\n        ClientSession c=session.owner();'
assert text.count(before)==1
after[observer]=text.replace(before,'static String observe(ClientSession c,GameHub hub)throws ReflectiveOperationException {').encode('utf-8')
text=Path(adapter['source']).read_text('utf-8-sig')
before='''        root=within(root,REBUILD_ROOT);within(config.source(),root);'''
assert text.count(before)==1
text=text.replace(before,'''        // This method owns all three supplied resources from entry, including
        // validation failure. The fixture must not close them a second time.
        ExecutorService clients=null;GameHub hub=null;NativeRecoveryProductionBootstrap application=null;
        try {
        root=within(root,REBUILD_ROOT);within(config.source(),root);''')
before='''        ExecutorService clients=Executors.newCachedThreadPool();List<ServerSocket> listeners=new ArrayList<>();
        GameStateStore state=new GameStateStore(config.stateFile(),config.statePersistenceEnabled(),config.ephemeralSourceProfiles());
        GameHub hub=null;NativeRecoveryProductionBootstrap application=null;
        try {'''
assert text.count(before)==1
text=text.replace(before,'''        clients=Executors.newCachedThreadPool();List<ServerSocket> listeners=new ArrayList<>();
        GameStateStore state=new GameStateStore(config.stateFile(),config.statePersistenceEnabled(),config.ephemeralSourceProfiles());''')
before='''            else {if(hub!=null)try{hub.close();}catch(Throwable cleanup){error.addSuppressed(cleanup);}clients.shutdownNow();}'''
assert text.count(before)==1
text=text.replace(before,'''            else {
                if(hub!=null)try{NativeServerLifetime.closeHub(hub);}catch(Throwable cleanup){error.addSuppressed(cleanup);}
                for(AutoCloseable resource:Arrays.asList(capture,maps,accounts))try{NativeServerLifetime.closeResource(resource);}
                    catch(Throwable cleanup){error.addSuppressed(cleanup);}
                if(clients!=null)try{NativeServerLifetime.finishClients(clients);}catch(Throwable cleanup){error.addSuppressed(cleanup);}
            }''')
text=text.replace('''    /** Ownership of accounts/maps/capture transfers only after native install.
     * Before then the caller retains cleanup responsibility. No listeners are
     * created or bound until both original resource readers have completed.
     */''','''    /** Takes ownership of supplied fixture resources at entry. Every failure
     * closes them through the original native lifetime helpers. No listeners
     * are created or bound until both original resource readers complete.
     */''')
after[bootstrap]=text.encode('utf-8')
text=(old_path.parent/rust).read_text('utf-8-sig')
before='assert_eq!(ready["observer_kind"],"G06_E149_direct_fields_v1");'
assert text.count(before)==1
text=text.replace(before,'''assert_eq!(ready["observer_kind"],"G06_E149_resources_direct_fields_v2");
 assert_eq!(ready["original_resource_factories_initialized"],true);
 assert_eq!(ready["native_server_lifetime_installed"],true);
 assert_eq!(ready["static_sql_access"],true);assert_eq!(ready["player_sql_access"],false);
 assert_eq!(ready["static_tables"],19);assert_eq!(ready["static_rows"],6277);
 assert_eq!(ready["full_main_process_qualified"],false);''')
text=text.replace('requires owned formal75','requires owned E149 original-resource')
before='let continued=c.snapshot().clone();c.select_rental_set(1)?;'
assert text.count(before)==1
text=text.replace(before,'''let continued=c.snapshot().clone();
  // Test-driver observation barrier only. The real continuation, inventory
  // and owner already arrived; no server clock or gameplay timer is edited.
  let character=continued.selected_character.as_deref().context("continued character absent")?;
  std::fs::write(root.join(format!("{role}-continuation-observe-request")),format!("{account}\\n{character}\\n測試乙"))?;
  let observe_end=Instant::now()+Duration::from_secs(8);
  loop {c.poll();ensure!(c.snapshot().phase==P::CharacterSelected
    && c.snapshot().selected_character==continued.selected_character
    && c.snapshot().inventory.owner==continued.inventory.owner,"continuation changed before observation");
   if root.join(format!("{role}-continuation-observed.json")).exists(){break}
   ensure!(Instant::now()<observe_end,"formal continuation observation not recorded");thread::sleep(Duration::from_millis(10));}
  c.select_rental_set(1)?;''')
after[rust]=text.encode('utf-8')
out=T/'candidates'/('native-E149-recovery-bootstrap-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
for n,blob in after.items():
    target=out/n;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(blob)
    previous=(old_path.parent/n).read_text('utf-8-sig') if (old_path.parent/n).exists() else Path(adapter['source']).read_text('utf-8-sig')
    (out/(Path(n).name+'.diff')).write_text(''.join(difflib.unified_diff(previous.splitlines(True),blob.decode('utf-8').splitlines(True),fromfile='previous/'+n,tofile='integrated/'+n)),encoding='utf-8')
contract=dict(status='development_only_no_fixture_execution',
    original_factory_sequence=['NativeServerLifetime.install','NativeDeItemTemplatesResources.read','NativeDeItemDefinitionsResources.read','NativeDeItemApplication.install'],
    original_session_sequence=['NativeServerLifetime.acceptSocket','ClientSession constructor','NativeServerLifetime.submitAccepted','Future.get'],
    cleanup=['NativeServerLifetime.closeHub','NativeServerLifetime.closeResource','NativeServerLifetime.finishClients','join acceptors','await original Futures'],
    changes=['separate overlay; existing thirteen files and all Runtime unchanged',
        'Require actual matching DB owner/import proofs and exact static SQL before initialization; no empty resource snapshots',
        'Original memory accounts/routes and embedded map factory, actual static JDBC only; player SQL forbidden',
        'Native bind after factory success, original accept/submit futures replace direct worker Threads',
        'Snapshot stage references under hub monitor, then release it before taking any inventory monitor',
        'All cleanup attempted, original primary error retained with suppressed cleanup errors; result only after successful exit barrier',
        'Resource bootstrap readiness marker is separate from stable direct-fields observer format',
        'Client CHARACTER_SELECTED waits for actual server LOBBY with selected character/inventory/active owner before next rental; product clocks and leases are unchanged',
        'Original recovery scenarios and 33/31-second natural waits unchanged'],
    limits=['not full Main initialization or full native owner qualification','no DB/test/Java class/GUI execution',
        'Future owned case orchestrator must supply proof hashes and pinned JDBC at runtime; live lease and orphan cleanup still require runtime verification'],
    test_cases_added=0,tests_executed=0,whole_goal_complete=False)
(out/'contract.json').write_bytes(data(contract))
record=dict(status='standalone_integrated_fixture_overlay_not_applied_not_tested',formal_target_sha256=old['formal_target_sha256'],
    formal_reference=old['formal_reference'],base_source_manifest_sha256=old['base_source_manifest_sha256'],
    predecessor_candidate=str(old_path),predecessor_candidate_sha256=sha(old_path),bootstrap_candidate=str(adapter_path),bootstrap_candidate_sha256=sha(adapter_path),
    candidate_files={n:digest(b) for n,b in after.items()},contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),
    existing_thirteen_file_overlay_unchanged=True,normal_source_modified=False,production_applied=False,
    Java_compiled=False,Rust_metadata_compiled=False,tests_executed=0,whole_goal_complete=False)
(out/'candidate.json').write_bytes(data(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
raw={**after,'candidate.json':(out/'candidate.json').read_bytes(),'contract.json':(out/'contract.json').read_bytes()}
with zipfile.ZipFile(out/'candidate.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,b in raw.items():z.writestr(n,b)
with zipfile.ZipFile(out/'candidate.zip') as z:assert all(z.read(n)==b for n,b in raw.items())
assert all(sha(R/n)==h for n,h in inputs.items()) and all(sha(n)==h for n,h in q['normal_product_pins'].items())
print(json.dumps(dict(candidate=str(out/'candidate.json'),sha256=sha(out/'candidate.json'),files=len(after),tests_executed=0),ensure_ascii=False))
