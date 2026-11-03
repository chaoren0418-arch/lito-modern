"""Develop source-distinguishing recovery coverage and missing-character fallback.
Only write isolated candidates. No client, test, fixture or GUI is executed.
"""
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
prior=Path(w['current_native_cancel_recovery_candidate']['candidate']);old=read(prior)
assert sha(prior)==w['current_native_cancel_recovery_candidate']['candidate_sha256'] and old['new_tests_written']==10
qpath=Path(w['current_development_typecheck_qualification']['qualification']);q=read(qpath)
assert q['candidates']['cancel-recovery']['sha256']==sha(prior) and q['original_session_id']==18566
source=w['current_product_identity']['source_manifest_sha256'];assert source==old['base_source_manifest_sha256']
inputs=read(Path(q['run'])/'base-compiled-inputs.json');assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
runtime='LiTo381-Modern/src/modern_lito/native_login.rs'
pure='LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs'
wire='LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs'
formal='LiTo381-Modern/tests/native_teleport_failure.rs'
java='LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java'
names=[runtime,pure,wire,formal,java]
original={n:(R/n).read_bytes() for n in names}
previous={n:(prior.parent/n).read_bytes() if n in old['candidate_files'] else original[n] for n in names}
assert all(sha(prior.parent/n)==h for n,h in old['candidate_files'].items())
text={n:(b.decode('utf-8-sig').replace('\r\n','\n') if n in [formal,java] else b.decode('utf-8-sig')) for n,b in previous.items()}
def once(n,before,after):
    assert text[n].count(before)==1,(n,before[:100]);text[n]=text[n].replace(before,after)
once(runtime,'''                ensure!(self.state.characters.iter().any(|c|c.name==character),
                    "teleport character absent from destination directory");''','''                if !self.state.characters.iter().any(|c|c.name==character) {
                    // A fresh LOGIN recovery can bootstrap the default source
                    // after its source handoff expires. Do not expose that
                    // directory or fail before confirming the selected source.
                    ensure!(self.teleport.is_none() && self.recovery_character.is_some(),
                        "teleport character absent from destination directory");
                    self.refresh_expired_recovery_source()?;
                    return Ok(());
                }''')
once(runtime,'''    /// The existing eight-second continuation wait elapsed without any
    /// selection acknowledgement or inventory. The ordinary source handoff
    /// may also have expired, so its first GAME directory is not sufficient
    /// authority to publish a manual choice. Never retry partial admission.
    fn refresh_expired_recovery_source(&mut self)->Result<()> {
        ensure!(self.state.phase==LoginPhase::CharacterPending && self.game
            && self.state.game_connected && self.readiness_acked
            && self.teleport.is_none() && self.recovery_character.is_some()
            && self.state.selected_character.as_ref()==self.recovery_character.as_ref()
            && self.state.character_result.is_none() && self.state.inventory.batches==0
            && self.state.notice.is_none(),"recovery source refresh outside unused continuation");''','''    /// Refresh an unused LOGIN recovery after the existing eight-second wait,
    /// or after a complete first directory does not contain its character.
    /// Both can follow source-handoff expiry; partial admission cannot retry.
    fn refresh_expired_recovery_source(&mut self)->Result<()> {
        let waiting=self.state.phase==LoginPhase::CharacterPending
            && self.state.selected_character.as_ref()==self.recovery_character.as_ref();
        let absent=self.state.phase==LoginPhase::LoadingCharacters && self.directory.complete()
            && self.state.selected_character.is_none()
            && self.recovery_character.as_ref().is_some_and(|name|
                !self.state.characters.iter().any(|row|row.name==*name));
        ensure!((waiting || absent) && self.game
            && self.state.game_connected && self.readiness_acked
            && self.teleport.is_none() && self.recovery_character.is_some()
            && self.state.character_result.is_none() && self.state.inventory.batches==0
            && self.state.notice.is_none(),"recovery source refresh outside unused continuation");''')
wire_added='''
#[cfg_attr(test, test)]
pub fn recovery_character_absent_from_first_directory_confirms_source_before_manual_choice() {
    let mut peer=FaultPeer::new();let mut c=expired_recovery_wait_on(&peer);
    c.state.phase=LoginPhase::LoadingCharacters;c.state.selected_character=None;
    c.state.inventory.clear();c.directory=Default::default();
    c.deadline=Some(Instant::now()+Duration::from_secs(8));
    peer.emit(vec![vec![3,0]]);pump(&mut c,|_|peer.opcodes().contains(&0xc8));
    assert_eq!(c.state.phase,LoginPhase::LoadingCharacters);
    assert!(c.state.characters.is_empty()&&c.recovery_character.is_none());
    assert!(c.state.selected_character.is_none()&&c.state.inventory.owner.is_none());
    assert!(!c.directory.complete()&&c.select_character(0).is_err());
    assert_eq!(*peer.commands.lock().unwrap(),vec![source_request("測試乙").unwrap().to_vec()]);
    peer.emit(vec![vec![3,0]]);pump(&mut c,|c|c.state.phase==LoginPhase::CharactersReady);
    assert_eq!(c.state.directory_generation,2);assert!(c.state.characters.is_empty());
    assert_eq!(peer.opcodes(),vec![0xc8]);stop(&mut c);peer.close();
}
'''
text[wire]+=wire_added

# Distinguish the requested source from the formal server's default. Select by
# received name, not a combo index, and retain every original cleanup assertion.
once(formal,''' let wrong_service=scenario=="wrong-service";let refused=scenario=="refused";let cancel_expiry=scenario=="cancel-expiry";let cancel_retry=scenario=="cancel-retry";
 let mut manual_selection=false;let mut closed_samples=0u32;let mut expiry_wait_ms=0u128;''',''' let wrong_service=scenario=="wrong-service";let refused=matches!(scenario.as_str(),"refused"|"refused-cancel-repeat");
 let dual_expiry=scenario=="cancel-expiry-source";
 let cancel_expiry=scenario=="cancel-expiry"||dual_expiry;
 let cancel_retry=matches!(scenario.as_str(),"cancel-retry"|"cancel-repeat");
 let repeat_cancel=matches!(scenario.as_str(),"cancel-repeat"|"refused-cancel-repeat");
 let mut manual_selection=false;let mut closed_samples=0u32;let mut expiry_wait_ms=0u128;
 let mut source_handoff_wait_ms=0u128;let mut cancellation_calls=0u32;''')
assert text[formal].count('c.select_source(0)?;')==2
text[formal]=text[formal].replace('c.select_source(0)?;','c.select_source(c.snapshot().sources.iter().position(|s|s.name=="測試乙").context("requested source not advertised")?)?;')
once(formal,'''   c.cancel();let end=Instant::now()+Duration::from_secs(3);''','''   c.cancel();cancellation_calls+=1;
   if repeat_cancel {for _ in 0..3 {c.cancel();cancellation_calls+=1;}}
   let end=Instant::now()+Duration::from_secs(3);''')
once(formal,'''  let failed=c.snapshot().clone();let elapsed=requested.elapsed().as_millis();let failed_epoch=c.transport().snapshot().epoch;''','''  if repeat_cancel {
   // Repeat after the original destination worker is also quiescent. This
   // must preserve an unused correlation without reviving any authority.
   for _ in 0..2 {c.cancel();cancellation_calls+=1;c.poll();}
  }
  let failed=c.snapshot().clone();let elapsed=requested.elapsed().as_millis();let failed_epoch=c.transport().snapshot().epoch;''')
anchor='''  c.select_source(c.snapshot().sources.iter().position(|s|s.name=="測試乙").context("requested source not advertised")?)?;wait(&mut c,15,|s|s.phase==P::RedirectReady)?;c.begin_game()?;'''
# Initial selection has a different line prefix. Change only the recovery one.
assert text[formal].count(anchor)==1
once(formal,anchor,'''  c.select_source(c.snapshot().sources.iter().position(|s|s.name=="測試乙").context("requested source not advertised")?)?;wait(&mut c,15,|s|s.phase==P::RedirectReady)?;
  if dual_expiry {
   let wait_started=Instant::now();
   while wait_started.elapsed()<Duration::from_secs(31) {
    c.poll();ensure!(c.snapshot().phase==P::RedirectReady
     && c.snapshot().selected_source.as_deref()==Some("測試乙")
     && c.snapshot().selected_character.is_none() && c.snapshot().inventory.owner.is_none(),
     "source TTL wait published authority or changed source");
    thread::sleep(Duration::from_millis(20));
   }
   source_handoff_wait_ms=wait_started.elapsed().as_millis();
  }
  c.begin_game()?;''')
once(formal,'''  let continued=c.snapshot().clone();c.select_rental_set(1)?;''','''  ensure!(c.snapshot().selected_source.as_deref()==Some("測試乙"),"recovery source label changed");
  ensure!(!dual_expiry||source_handoff_wait_ms>=31000,"source handoff did not naturally expire");
  ensure!(!repeat_cancel||cancellation_calls>=2,"repeated cancellation was not exercised");
  let continued=c.snapshot().clone();c.select_rental_set(1)?;''')
once(formal,'"expiry_wait_ms":expiry_wait_ms,"full_game_reentry_verified":true',
     '"expiry_wait_ms":expiry_wait_ms,"requested_source":"測試乙","default_source":"測試甲","source_handoff_wait_ms":source_handoff_wait_ms,"cancellation_calls":cancellation_calls,"full_game_reentry_verified":true')
once(formal,'''assert_ne!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("cancel-expiry"));''','''assert!(!matches!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("cancel-expiry"|"cancel-expiry-source")));''')
once(formal,'''let expected=std::env::var("LITO_TELEPORT_REFERENCE_SHA").unwrap_or_else(|_|"bbc59308c43df89ee5029d3b3c802be8ad55fee65c41faca99a76912dd5dce43".into());assert_eq!(ready["jar_sha256"],expected);''','''let expected="75da69b0d2f7f9d394ee6bf2fdb396b153c75d9b938069b0999ed8dac4f50371";
 if let Ok(declared)=std::env::var("LITO_TELEPORT_REFERENCE_SHA"){assert_eq!(declared.to_lowercase(),expected);}
 assert_eq!(ready["jar_sha256"],expected);
 assert_eq!(ready["requested_source"],"測試乙");assert_eq!(ready["default_source"],"測試甲");
 assert_eq!(ready["scenario"],std::env::var("LITO_TELEPORT_SCENARIO").unwrap_or_else(|_|"timeout".into()));''')
text[formal]+='''
#[test]
#[ignore="requires owned formal75 harness; repeated cancel before and after quiescence"]
fn repeated_cancellation_preserves_only_one_authenticated_recovery() {
 assert_eq!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("cancel-repeat"));run_owned_harness();
}
#[test]
#[ignore="requires owned formal75 closed destination and failure/cancel recovery"]
fn failed_handoff_then_repeated_cancel_preserves_the_authenticated_recovery() {
 assert_eq!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("refused-cancel-repeat"));run_owned_harness();
}
#[test]
#[ignore="requires owned formal75 harness and actual 33s staged plus 31s source waits"]
fn expired_stage_and_source_handoff_require_the_requested_source_directory() {
 assert_eq!(std::env::var("LITO_TELEPORT_SCENARIO").as_deref(),Ok("cancel-expiry-source"));run_owned_harness();
}
'''
once(java,'''boolean cancelExpiry=scenario.equals("cancel-expiry");
  boolean refused=scenario.equals("refused");''','''boolean cancelExpiry=scenario.equals("cancel-expiry")||scenario.equals("cancel-expiry-source");
  boolean refused=scenario.equals("refused")||scenario.equals("refused-cancel-repeat");
  if(!Set.of("timeout","refused","wrong-service","cancel-retry","cancel-late","cancel-expiry","cancel-repeat","refused-cancel-repeat","cancel-expiry-source").contains(scenario))throw new IllegalArgumentException("unknown recovery scenario");''')
once(java,'login.classic.server.names=測試乙,測試甲','login.classic.server.names=測試甲,測試乙')
once(java,'''var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試乙",memoryBacking);routes.put("測試甲",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));''','''var routes=new LinkedHashMap<String,AccountRepository>();routes.put("測試甲",new MemoryAccountRepository("PeerFixture","TestOnlyPassword"));routes.put("測試乙",memoryBacking);''')
once(java,'// Both players use source 0; the hub shares its initial memory child.',
     '// Both players explicitly request the nondefault source 測試乙; memory backing matches the existing qualified peer fixture.')
once(java,'''login.setSoTimeout((cancelExpiry||gui)?180000:90000);town.setSoTimeout((cancelExpiry||gui)?180000:90000);''',
     '''login.setSoTimeout(scenario.equals("cancel-expiry-source")?420000:(cancelExpiry||gui)?180000:90000);town.setSoTimeout(scenario.equals("cancel-expiry-source")?420000:(cancelExpiry||gui)?180000:90000);''')
once(java,'((cancelExpiry||gui)?320_000_000_000L:200_000_000_000L)',
     '(scenario.equals("cancel-expiry-source")?440_000_000_000L:(cancelExpiry||gui)?320_000_000_000L:200_000_000_000L)')
once(java,'''"{\\"port\\":"+login.getLocalPort()+''','''"{\\"requested_source\\":\\"測試乙\\",\\"default_source\\":\\"測試甲\\",\\"scenario\\":\\""+scenario+"\\",\\"port\\":"+login.getLocalPort()+''')

out=R/'維護工具/candidates'/('native-recovery-default-source-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
after={n:t.encode('utf-8') for n,t in text.items()}
for n,b in after.items():
    p=out/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b)
    (out/(Path(n).name+'.diff')).write_text(''.join(difflib.unified_diff(original[n].decode('utf-8-sig').replace('\r\n','\n').splitlines(True),text[n].replace('\r\n','\n').splitlines(True),fromfile=n,tofile=n)),encoding='utf-8')
for name,files in [('before.zip',original),('candidate.zip',after)]:
    with zipfile.ZipFile(out/name,'x',zipfile.ZIP_DEFLATED) as z:
        for n,b in files.items():z.writestr(n,b)
    with zipfile.ZipFile(out/name) as z:assert all(z.read(n)==b for n,b in files.items())
assert after[pure]==previous[pure] and after[wire].startswith(previous[wire])
baseline_pure=Path(old['baseline_regression_tests']).read_bytes()
baseline_wire=Path(old['baseline_wire_regression_tests']).read_bytes()+wire_added.encode('utf-8')
(out/'baseline-regression-tests.rs').write_bytes(baseline_pure)
(out/'baseline-wire-regression-tests.rs').write_bytes(baseline_wire)
contract=dict(predecessor_contract=old['contract'],predecessor_contract_sha256=sha(old['contract']),
    qualified_source_distinguishing_fixture=str(P/'tests/oracle/NativePeerHarness.java'),qualified_source_distinguishing_fixture_sha256=sha(P/'tests/oracle/NativePeerHarness.java'),
    inference='A complete recovery directory may contain no matching original character after source TTL expiry. Refresh the exact selected source once before publishing a manual directory. Live explicit teleport destinations and partial admission still reject.',
    formal_cases=['cancel-repeat','refused-cancel-repeat','cancel-expiry-source'],requested_source='測試乙',default_source='測試甲',
    natural_waits_seconds={'staged_loadout':33,'fresh_LOGIN_source_handoff':31},server_lifetimes_modified=False,
    product_deadlines_modified=False,dual_expiry_fixture_watchdog_seconds=440,required_owned_runner_lease_seconds=460,
    tests_executed=0,game_or_fixture_executions=0,GUI_verified=False)
(out/'contract.json').write_bytes(data(contract))
record=dict(status='isolated_G06_default_source_recovery_and_formal_QA_not_built_not_tested',base_source_manifest_sha256=source,
    predecessor_candidate=str(prior),predecessor_candidate_sha256=sha(prior),predecessor_metadata_qualification=str(qpath),predecessor_metadata_does_not_qualify_this_revision=True,
    candidate_files={n:digest(b) for n,b in after.items()},base_files={n:digest(b) for n,b in original.items()},runtime_files=[runtime],test_files=[pure,wire,formal,java],
    new_tests_written=14,new_tests_in_this_revision=4,retained_prior_new_tests=10,new_library_cases_in_this_revision=1,new_formal_integration_cases=3,
    baseline_regression_tests=str(out/'baseline-regression-tests.rs'),baseline_regression_tests_sha256=sha(out/'baseline-regression-tests.rs'),
    baseline_wire_regression_tests=str(out/'baseline-wire-regression-tests.rs'),baseline_wire_regression_tests_sha256=sha(out/'baseline-wire-regression-tests.rs'),baseline_additional_cases=6,
    before_archive_sha256=sha(out/'before.zip'),candidate_archive_sha256=sha(out/'candidate.zip'),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),
    static_defect='Recovery missing its character failed before its existing timeout refresh; original formal recovery QA selected source index0, identical to the default, and did not expire the fresh LOGIN source handoff.',
    line_endings_normalized_only=[formal,java],
    changes=['Retain all predecessor cancellation and timeout fixes; refresh a complete missing-character recovery directory once',
        'Explicit destination teleport still rejects absent character; partial admission never retries',
        'Formal fixture defaults to 測試甲 while client selects received 測試乙 by name',
        'Add repeated cancel, failure then repeated cancel and dual natural expiry ignored integration cases against fixed formal75',
        'Fresh source TTL wait is actual 31s after redirect, in addition to the actual 33s stage wait; no authority or clock writes'],
    builds_executed=0,tests_executed=0,production_applied=False,actual_failure_reproduction_executed=False,GUI_verified=False,whole_goal_complete=False,
    required_later=['Original six expected-red component cases; candidate eleven appended component and three new formal cases plus all original cases',
        'Fresh source confirmation must be proven by wire C8/directory and formal server source, with complete item continuity and actual stage/source expiration timestamps',
        'Negative peer/account/source bindings, final-source full library both profiles, Release regression and complete normal GUI'])
(out/'candidate.json').write_bytes(data(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
(out/'README.md').write_text('# G06.3d 非預設來源恢復\n\n在前版取消／回復逾時修補上，補完整首批清單找不到原角色時的一次C8來源確認。保留部分接續與正式目的地拒絕；只在LOGIN回復期待尚未獲得授權時取用。\n\n正式測試fixture預設甲、按收到名稱選乙。新增一個加密loopback案例及三個ignored正式整合案例：連續取消、拒絕目的地後連續取消、原租裝階段自然33秒與新LOGIN來源交接自然31秒。兩個等待不互相代替，不改伺服器時鐘、TTL或租期。\n\n本次1 Runtime與4測試檔，共14個新增案例（11元件、3正式），原版6個元件反例對照。未執行、未套用；前版metadata不移記。Java/Rust須先隔離編譯。正式雙重到期fixture上限440秒、runner須保留460秒及身分核對清理，產品8/30/60秒與租期維持。\n',encoding='utf-8')
assert all(sha(R/n)==h for n,h in inputs.items()) and all(sha(p)==h for p,h in q['normal_product_pins'].items())
print(json.dumps(dict(candidate=str(out/'candidate.json'),sha256=sha(out/'candidate.json'),new_tests_written=14,tests_executed=0),ensure_ascii=False))
