"""Develop expired-recovery source confirmation; no compilation or tests."""
from pathlib import Path
from datetime import datetime, timezone
import difflib, hashlib, json, re, zipfile

R = Path(__file__).resolve().parents[1]; P = R/'LiTo381-Modern'; V = R/'驗證紀錄/current'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
digest = lambda b: hashlib.sha256(b).hexdigest().upper()
data = lambda v: (json.dumps(v, ensure_ascii=False, indent=2)+'\n').encode('utf-8')
w = read(R/'工作佇列.json')
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
source = w['current_product_identity']['source_manifest_sha256']
prior = Path(w['current_native_cancel_recovery_candidate']['candidate']); old = read(prior)
assert sha(prior) == w['current_native_cancel_recovery_candidate']['candidate_sha256']
assert old['base_source_manifest_sha256'] == source and old['new_tests_written'] == 6
q = read(w['current_development_typecheck_qualification']['qualification'])
assert Path(q['candidates']['cancel-recovery']['path']) == prior and q['tests_executed'] == 0
inputs = read(Path(q['run'])/'base-compiled-inputs.json')
assert len(inputs) == 1049 and all(sha(R/n) == h for n,h in inputs.items())
assert all(sha(prior.parent/n) == h for n,h in old['candidate_files'].items())
runtime = 'LiTo381-Modern/src/modern_lito/native_login.rs'
pure = 'LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs'
wire = 'LiTo381-Modern/src/modern_lito/native_teleport_controller_tests.rs'
names = [runtime, pure, wire]
original = {n: (R/n).read_bytes() for n in names}
previous = {n: (prior.parent/n).read_bytes() if n in old['candidate_files'] else original[n] for n in names}
text = {n: b.decode('utf-8-sig') for n,b in previous.items()}
def once(n, before, after):
    assert text[n].count(before) == 1
    text[n] = text[n].replace(before, after)
once(runtime, '''                // A fresh LOGIN can outlive the server's staged handoff. The
                // protocol has no explicit "stage expired" response. With no
                // selection ack or inventory, retire this one expectation and
                // return to the current directory; do not invent authority or
                // send a second C_C9 while server continuation is pending.
                self.recovery_character=None;self.state.selected_character=None;
                self.state.inventory.clear();self.deadline=None;
                self.state.phase=LoginPhase::CharactersReady;''', '''                // No response establishes that the first directory belongs to
                // our selected source. Retire the unused expectation and ask
                // GAME for that source's directory before allowing selection.
                // This sends C_C8 once, never a speculative C_C9.
                if let Err(error)=self.refresh_expired_recovery_source() {
                    self.fail(format!("expired recovery source refresh: {error:#}"));
                }''')
once(runtime, '    fn begin_rental_return_reconnect(&mut self)->Result<()> {', '''    /// The existing eight-second continuation wait elapsed without any
    /// selection acknowledgement or inventory. The ordinary source handoff
    /// may also have expired, so its first GAME directory is not sufficient
    /// authority to publish a manual choice. Never retry partial admission.
    fn refresh_expired_recovery_source(&mut self)->Result<()> {
        ensure!(self.state.phase==LoginPhase::CharacterPending && self.game
            && self.state.game_connected && self.readiness_acked
            && self.teleport.is_none() && self.recovery_character.is_some()
            && self.state.selected_character.as_ref()==self.recovery_character.as_ref()
            && self.state.character_result.is_none() && self.state.inventory.batches==0
            && self.state.notice.is_none(),"recovery source refresh outside unused continuation");
        let source=self.state.selected_source.as_ref().filter(|s|!s.is_empty())
            .context("recovery source missing")?;
        ensure!(self.state.sources.iter().any(|entry|entry.name==*source),
            "recovery source no longer advertised");
        // Preserve all state if enqueue fails; the poll caller then follows
        // the ordinary failure retirement path. No second transport starts.
        self.transport.send(self.transport.snapshot().epoch,source_request(source)?)?;
        self.recovery_character=None;self.state.selected_character=None;
        self.state.inventory.clear();self.state.characters.clear();
        self.directory=super::native_game::Directory::default();
        self.state.character_result=None;self.state.classic_entry_result=None;
        self.confirm_initial_source=false;
        self.state.phase=LoginPhase::LoadingCharacters;
        self.deadline=Some(Instant::now()+Duration::from_secs(8));
        Ok(())
    }
    fn begin_rental_return_reconnect(&mut self)->Result<()> {''')

pure_added = '''
// These cases stage a controller response deadline; they do not emulate the
// formal server's 30s source TTL or the rental loadout's natural lifetime.
fn unused_recovery_wait() -> LoginController {
    let mut c=LoginController::default();c.game=true;c.readiness_acked=true;
    c.state.phase=LoginPhase::CharacterPending;c.state.game_connected=true;
    c.state.selected_source=Some("測試乙".into());
    c.state.sources=vec![Source{name:"測試乙".into(),endpoint:"127.0.0.1:31002".parse().unwrap()}];
    c.state.selected_character=Some("RecoveryCharacter".into());
    c.recovery_character=Some("RecoveryCharacter".into());
    c.state.world.generation=17;c.state.inventory.begin(0,"RecoveryCharacter").unwrap();
    c.deadline=Some(Instant::now()-Duration::from_millis(1));c
}
#[test]
fn expired_recovery_send_failure_never_publishes_the_unconfirmed_directory() {
    let mut c=unused_recovery_wait();c.poll();
    assert_eq!(c.state.phase,LoginPhase::Failed);
    assert!(c.state.error.as_deref().unwrap().contains("expired recovery source refresh"));
    assert_cancelled_scope_retired(&c);assert!(c.teleport_recovery.is_none());
    assert!(c.state.characters.is_empty());assert_eq!(c.transport.snapshot().epoch,0);
}
#[test]
fn partial_or_unbound_recovery_cannot_request_another_source_directory() {
    for kind in 0..10 {
        let mut c=unused_recovery_wait();
        match kind {
            0=>c.state.character_result=Some(0),1=>c.state.inventory.batches=1,
            2=>c.state.selected_source=None,3=>c.state.sources.clear(),
            4=>c.state.phase=LoginPhase::CharacterSelected,5=>c.state.game_connected=false,
            6=>c.game=false,7=>c.readiness_acked=false,
            8=>c.state.selected_character=Some("AnotherCharacter".into()),
            _=>c.recovery_character=None,
        }
        let before=serde_json::to_value(&c.state).unwrap();let deadline=c.deadline;
        let recovery=c.recovery_character.clone();
        assert!(c.refresh_expired_recovery_source().is_err(),"binding {kind}");
        assert_eq!(serde_json::to_value(&c.state).unwrap(),before);
        assert_eq!(c.deadline,deadline);assert_eq!(c.recovery_character,recovery);
        assert_eq!(c.transport.snapshot().epoch,0);assert!(c.is_quiescent());
    }
}
'''
text[pure] += pure_added
wire_added = '''
// Seed only the unused post-bootstrap expectation against the encrypted peer.
// This is synthetic controller coverage, not authenticated formal TTL proof.
fn expired_recovery_wait_on(peer:&FaultPeer)->LoginController {
    let mut c=LoginController::default();c.start(peer.peer,Limits::default()).unwrap();
    pump(&mut c,|c|c.state.phase==LoginPhase::ReadyForLogin);
    c.game=true;c.readiness_acked=true;c.state.game_connected=true;
    c.state.selected_source=Some("測試乙".into());
    c.state.sources=vec![Source{name:"測試乙".into(),endpoint:peer.peer}];
    c.recovery_character=Some("TestKnight".into());
    c.state.selected_character=Some("TestKnight".into());
    c.state.phase=LoginPhase::CharacterPending;c.state.world.generation=17;
    c.state.inventory.begin(c.transport.snapshot().epoch,"TestKnight").unwrap();
    c.directory.start(0).unwrap();
    c.deadline=Some(Instant::now()-Duration::from_millis(1));c
}
#[cfg_attr(test, test)]
pub fn expired_recovery_confirms_source_before_publishing_a_new_directory() {
    let mut peer=FaultPeer::new();let mut c=expired_recovery_wait_on(&peer);c.poll();
    assert_eq!(c.state.phase,LoginPhase::LoadingCharacters);
    assert!(c.state.characters.is_empty()&&c.state.selected_character.is_none());
    assert!(c.recovery_character.is_none()&&c.state.inventory.owner.is_none());
    assert!(!c.directory.complete());assert_eq!(c.state.world.generation,17);
    assert!(c.select_character(0).is_err());
    pump(&mut c,|_|peer.opcodes().contains(&0xc8));
    let expected=source_request("測試乙").unwrap();
    assert_eq!(*peer.commands.lock().unwrap(),vec![expected.to_vec()]);
    for _ in 0..8 {c.poll();assert_eq!(c.state.phase,LoginPhase::LoadingCharacters);}
    assert_eq!(peer.opcodes(),vec![0xc8]);
    // S03 zero rows is a complete new directory; readiness remains bound to
    // this GAME epoch. Do not send another readiness or character selection.
    peer.emit(vec![vec![3,0]]);pump(&mut c,|c|c.state.phase==LoginPhase::CharactersReady);
    assert!(c.state.characters.is_empty());assert_eq!(c.state.directory_generation,1);
    assert_eq!(peer.opcodes(),vec![0xc8]);stop(&mut c);peer.close();
}
#[cfg_attr(test, test)]
pub fn cancelling_the_expired_recovery_refresh_cannot_start_another_transport() {
    let mut peer=FaultPeer::new();let mut c=expired_recovery_wait_on(&peer);c.poll();
    assert_eq!(c.state.phase,LoginPhase::LoadingCharacters);
    pump(&mut c,|_|peer.opcodes().contains(&0xc8));let epoch=c.transport.snapshot().epoch;
    c.cancel();c.cancel();pump(&mut c,|c|c.is_quiescent());
    assert!(c.recovery_character.is_none()&&c.teleport_recovery.is_none());retired(&c);
    assert_eq!(c.state.phase,LoginPhase::Closed);assert_eq!(c.transport.snapshot().epoch,epoch);
    assert_eq!(peer.accepted.load(Ordering::Acquire),1);assert_eq!(peer.opcodes(),vec![0xc8]);peer.close();
}
'''
text[wire] += wire_added
after = {n: t.encode('utf-8') for n,t in text.items()}
assert after[pure].startswith(previous[pure]) and after[wire].startswith(original[wire])
out = R/'維護工具/candidates'/('native-recovery-source-refresh-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
for n,b in after.items():
    dest=out/n;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(b)
    (out/(Path(n).name+'.diff')).write_text(''.join(difflib.unified_diff(original[n].decode('utf-8-sig').splitlines(True),text[n].splitlines(True),fromfile=n,tofile=n)),encoding='utf-8')
for name,files in [('before.zip',original),('candidate.zip',after)]:
    with zipfile.ZipFile(out/name,'x',zipfile.ZIP_DEFLATED) as z:
        for n,b in files.items():z.writestr(n,b)
    with zipfile.ZipFile(out/name) as z:assert all(z.read(n)==b for n,b in files.items())

# Retain selected existing formal bytecode, rather than running a server or
# disassembler. Pin the original disassembly and the original 30s causal bound.
contract_root=V/'native-source-handoff-expiry/contract-20261019-174028-047309'
disassembly=contract_root/'ClientSession.stdout.txt'
assert sha(disassembly)=='964DC949F7EA6C575A8E725D025EA67B94B73ABF81C200BBE2BA190D72248CF1'
jar=P/'target/current/server-reference/formal-server-75DA69B0.jar'
assert sha(jar)=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
lines=disassembly.read_text('utf-8-sig').splitlines()
starts=[i for i,s in enumerate(lines) if re.match(r'^  \S.*\(.*\).*;$',s)]
selected={}
for j,i in enumerate(starts):
    if any((' '+name+'(') in lines[i] for name in ['login','completeGameBootstrap','autoSelectPendingRentalBattleHandoff','requestLitoCharacterInfo']):
        selected[lines[i]]='\n'.join(lines[i:starts[j+1] if j+1<len(starts) else len(lines)])+'\n'
assert len(selected)==4
(out/'selected-formal-methods.json').write_bytes(data(selected))
contract=dict(formal_jar=str(jar),formal_jar_sha256=sha(jar),original_disassembly=str(disassembly),original_disassembly_sha256=sha(disassembly),
    selected_methods=str(out/'selected-formal-methods.json'),selected_methods_sha256=sha(out/'selected-formal-methods.json'),
    source_TTL_existing_evidence=str(contract_root/'causal-timing-refinement.json'),source_TTL_existing_evidence_sha256=sha(contract_root/'causal-timing-refinement.json'),
    inference='After an unused recovery times out the initial GAME directory has no new source-confirmation guarantee. Formal GAME C8 in LOBBY authenticates/canonicalizes the source and sends a new directory; it does not itself send a character-selection request.',
    limitation='Static inference only; no staged-expiry behavior was newly reproduced. Pending admission can still race the refresh; late incompatible packets must fail and retire through existing gates.',
    game_or_fixture_executions=0,tests_executed=0)
(out/'contract.json').write_bytes(data(contract))
# Baseline-readable cases contain no references to the new private helper.
baseline_pure=(prior.parent/'baseline-regression-tests.rs').read_bytes()+pure_added.split('#[test]\nfn partial_or_unbound_recovery_',1)[0].encode('utf-8')
(out/'baseline-regression-tests.rs').write_bytes(baseline_pure)
(out/'baseline-wire-regression-tests.rs').write_bytes(original[wire]+wire_added.encode('utf-8'))
record=dict(status='isolated_G06_recovery_source_refresh_development_not_built_not_tested',base_source_manifest_sha256=source,
    predecessor_candidate=str(prior),predecessor_candidate_sha256=sha(prior),predecessor_metadata_qualification=w['current_development_typecheck_qualification']['qualification'],
    predecessor_metadata_does_not_qualify_this_revision=True,candidate_files={n:digest(b) for n,b in after.items()},base_files={n:digest(b) for n,b in original.items()},
    runtime_files=[runtime],test_files=[pure,wire],new_tests_written=10,new_tests_in_this_revision=4,retained_prior_new_tests=6,
    baseline_regression_tests=str(out/'baseline-regression-tests.rs'),baseline_regression_tests_sha256=sha(out/'baseline-regression-tests.rs'),
    baseline_wire_regression_tests=str(out/'baseline-wire-regression-tests.rs'),baseline_wire_regression_tests_sha256=sha(out/'baseline-wire-regression-tests.rs'),baseline_additional_cases=5,
    before_archive_sha256=sha(out/'before.zip'),candidate_archive_sha256=sha(out/'candidate.zip'),contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),
    static_defect='Unused recovery timeout published the first GAME directory as CharactersReady without repeating public C8 for the selected source after source-handoff expiry.',
    changes=['Retain predecessor repeated-cancel and one-shot binding fixes', 'After unchanged continuation response deadline send one current-source C8 and clear the unconfirmed directory',
        'Keep CharactersReady and manual C9 blocked until a new server directory completes', 'No refresh when partial admission, a different character, an absent/unadvertised source or invalid GAME state is present',
        'Send failure follows full fail retirement; cancellation of refresh cannot re-login; no new connection or rental lifetime change'],
    builds_executed=0,tests_executed=0,production_applied=False,GUI_verified=False,whole_goal_complete=False,actual_failure_reproduction_executed=False,
    required_later=['Original five expected-red cancellation/expiry cases, candidate ten appended cases and original library cases',
        'Formal75 quick/cancel/failed handoff with live stage and actual natural stage expiry plus source TTL expiry; exact C8/source/directory assertions',
        'Final source full Debug/Release library, controller branches, normal GUI full flow and Release regression'])
(out/'candidate.json').write_bytes(data(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
(out/'README.md').write_text('# G06.3d 回復逾時後重新確認來源\n\n保留前候選六案與取消／一次性關聯修補，再新增四案。原回復等待逾時沒有選角確認或背包批次時，先送一次公有C8並清空未重新確認的清單；新清單完成前不開放選角、不送C9。有部分接續資料則不重試。原8秒回應時限、30秒來源TTL及租期維持。\n\n三個檔案（1 Runtime、2 test），十個新增案例與五個原版反例對照。尚未編譯、執行或套用；前版metadata資格不能移記此修訂。靜態推論及合成控制器案例不代替正式自然到期、GUI或G06.3d驗收。\n',encoding='utf-8')
assert all(sha(R/n)==h for n,h in inputs.items())
assert all(sha(p)==h for p,h in q['normal_product_pins'].items())
print(json.dumps(dict(candidate=str(out/'candidate.json'),sha256=sha(out/'candidate.json'),tests_written=10,new_cases=4,tests_executed=0,production_applied=False),ensure_ascii=False))
