"""Develop G06.3d cancellation recovery without executing a client or test."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
digest=lambda b:hashlib.sha256(b).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
source=w['current_product_identity']['source_manifest_sha256'];assert source=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
manifest=read(V/'native-installed-resource-profile-normal/application-20261020-123323-122063/applied-compiled-inputs.json')
assert len(manifest)==1049 and all(sha(R/n)==h for n,h in manifest.items())
names=['LiTo381-Modern/src/modern_lito/native_login.rs','LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs']
original={n:(R/n).read_bytes() for n in names};text={n:b.decode('utf-8-sig') for n,b in original.items()}
def replace(n,before,after):
    assert text[n].count(before)==1,(n,before[:120]);text[n]=text[n].replace(before,after)
runtime,tests=names
replace(runtime,'''        self.teleport_recovery=self.teleport_recovery_candidate();
        self.retire_teleport_scene();''','''        if let Some(recovery)=self.teleport_recovery_candidate() {
            self.teleport_recovery=Some(recovery);
        }
        // Repeated cancellation must not erase an unused correlation merely
        // because the first call already retired its pending teleport. This
        // stores no password and never schedules authentication or gameplay.
        self.retire_teleport_scene();''')
replace(runtime,'''        self.state.selected_source = Some(source.name.clone());
        if let Some(recovery)=self.teleport_recovery.take(){
            if recovery.source==source.name && self.credentials.as_ref().is_some_and(|(account,_)|**account==recovery.account) {
                self.recovery_character=Some(recovery.character);
            }
        }''','''        let source_name=source.name.clone();
        self.state.selected_source = Some(source_name.clone());
        if let Some(character)=self.take_teleport_recovery(&source_name) {
            self.recovery_character=Some(character);
        }''')
replace(runtime,'    fn teleport_recovery_candidate(&self)->Option<TeleportRecovery> {','''    /// Consume the correlation for exactly one explicit source choice. Even
    /// a rejected identity binding consumes it; correcting fields later cannot
    /// revive a retired choice. The regular LOGIN/source protocol owns sends.
    fn take_teleport_recovery(&mut self, source: &str) -> Option<String> {
        let recovery=self.teleport_recovery.take()?;
        (self.login_peer==Some(recovery.login_peer) && recovery.source==source
            && self.credentials.as_ref().is_some_and(|(account,_)|**account==recovery.account))
            .then_some(recovery.character)
    }
    fn teleport_recovery_candidate(&self)->Option<TeleportRecovery> {''')
text[tests]+='''
// Pure controller state cases: no listener, transport, fixture, or credentials
// outside these synthetic values. Runtime authentication remains separately required.
fn staged_cancel_recovery() -> LoginController {
    let mut c=LoginController::default();
    c.login_peer=Some("127.0.0.1:31001".parse().unwrap());
    c.credentials=Some((Zeroizing::new("RecoveryFixture".into()),Zeroizing::new("TestOnlyPassword".into())));
    c.state.selected_source=Some("測試乙".into());c.state.selected_character=Some("RecoveryCharacter".into());
    c.state.phase=LoginPhase::HandoffClosing;c.state.game_connected=true;
    c.state.world.generation=17;c.state.world.map_id=Some(500);c.state.world.learned_skills.insert(43);
    c.state.world.mp=Some([70,110]);
    let owner=super::super::native_merchant::Owner{connection:1,generation:17,incarnation:2,control_interruptions:0,
        inventory:super::super::native_inventory::Owner{connection:1,selection:1,character:"RecoveryCharacter".into()}};
    let mut pending=super::super::native_teleport::Pending::new(owner,"RecoveryCharacter".into(),"TeleportMe","LT_TELEPORT_B").unwrap();
    pending.destination=Some("127.0.0.2:31002".parse().unwrap());c.teleport=Some(pending);c
}
fn assert_cancelled_scope_retired(c:&LoginController) {
    assert!(c.credentials.is_none()&&c.teleport.is_none()&&c.recovery_character.is_none());
    assert!(c.state.world.map_id.is_none()&&c.state.world.mp.is_none()&&c.state.world.learned_skills.is_empty());
    assert!(c.state.world.actors.is_empty()&&c.state.world.effects.is_empty()&&c.state.inventory.owner.is_none());
    assert!(c.state.selected_character.is_none()&&c.state.joined_room.is_none()&&c.state.services.is_none());
    assert!(c.state.environment.epoch.is_none()&&!c.state.game_connected&&c.rental_return_reconnect.is_none());
    assert!(c.is_quiescent());assert_eq!(c.state.world.generation,17);assert_eq!(c.transport.snapshot().epoch,0);
}
#[test]
fn repeated_cancel_keeps_only_the_unused_recovery_correlation() {
    let mut c=staged_cancel_recovery();c.cancel();
    for _ in 0..3 {
        assert_cancelled_scope_retired(&c);
        let recovery=c.teleport_recovery.as_ref().expect("unused correlation survived cancel");
        assert_eq!(recovery.login_peer,"127.0.0.1:31001".parse().unwrap());
        assert_eq!((&recovery.account,&recovery.source,&recovery.character),
            (&"RecoveryFixture".to_owned(),&"測試乙".to_owned(),&"RecoveryCharacter".to_owned()));
        c.cancel();c.poll();
    }
    assert_eq!(c.state.phase,LoginPhase::Closed);
}
#[test]
fn failure_then_cancel_does_not_erase_or_restore_retired_authority() {
    let mut c=staged_cancel_recovery();c.fail("synthetic handoff failure".into());c.cancel();c.poll();
    assert_cancelled_scope_retired(&c);assert!(c.teleport_recovery.is_some());
    assert_eq!(c.state.phase,LoginPhase::Failed);assert_eq!(c.state.error.as_deref(),Some("synthetic handoff failure"));
}
#[test]
fn recovery_binding_mismatches_consume_the_one_shot_without_revival() {
    for kind in 0..5 {
        let mut c=staged_cancel_recovery();c.cancel();
        c.credentials=Some((Zeroizing::new("RecoveryFixture".into()),Zeroizing::new("TestOnlyPassword".into())));
        let source=if kind==2 {"測試甲"} else {"測試乙"};
        match kind {
            0=>c.login_peer=Some("127.0.0.9:31001".parse().unwrap()),
            1=>c.credentials=Some((Zeroizing::new("OtherFixture".into()),Zeroizing::new("TestOnlyPassword".into()))),
            2=>{},3=>c.credentials=None,_=>c.login_peer=None,
        }
        assert!(c.take_teleport_recovery(source).is_none(),"binding {kind}");
        assert!(c.teleport_recovery.is_none());
        c.login_peer=Some("127.0.0.1:31001".parse().unwrap());
        c.credentials=Some((Zeroizing::new("RecoveryFixture".into()),Zeroizing::new("TestOnlyPassword".into())));
        assert!(c.take_teleport_recovery("測試乙").is_none());
        assert_eq!(c.transport.snapshot().epoch,0);assert!(c.state.world.map_id.is_none());
    }
}
#[test]
fn matching_recovery_is_taken_once_and_later_cancel_cannot_revive_it() {
    let mut c=staged_cancel_recovery();c.cancel();
    c.credentials=Some((Zeroizing::new("RecoveryFixture".into()),Zeroizing::new("TestOnlyPassword".into())));
    assert_eq!(c.take_teleport_recovery("測試乙").as_deref(),Some("RecoveryCharacter"));
    assert!(c.take_teleport_recovery("測試乙").is_none());c.cancel();c.poll();
    assert_cancelled_scope_retired(&c);assert!(c.teleport_recovery.is_none());
}
#[test]
fn ordinary_cancel_without_an_authorized_destination_cannot_create_recovery() {
    let mut c=staged_cancel_recovery();c.teleport.as_mut().unwrap().destination=None;
    c.cancel();c.cancel();c.poll();assert_cancelled_scope_retired(&c);assert!(c.teleport_recovery.is_none());
}
#[test]
fn a_new_authorized_handoff_replaces_only_the_old_correlation() {
    let mut c=staged_cancel_recovery();
    c.teleport_recovery=Some(TeleportRecovery{login_peer:"127.0.0.9:31009".parse().unwrap(),
        account:"OldFixture".into(),source:"舊來源".into(),character:"OldCharacter".into()});
    c.cancel();let r=c.teleport_recovery.as_ref().unwrap();assert_eq!(r.character,"RecoveryCharacter");
    assert_eq!(r.account,"RecoveryFixture");assert_eq!(r.source,"測試乙");assert_cancelled_scope_retired(&c);
}
'''
out=R/'維護工具/candidates'/('native-teleport-cancel-idempotence-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
after={n:s.encode('utf-8') for n,s in text.items()}
for n,b in after.items():
    p=out/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b)
    (out/(Path(n).name+'.diff')).write_text(''.join(difflib.unified_diff(original[n].decode('utf-8-sig').splitlines(True),text[n].splitlines(True),fromfile=n,tofile=n)),encoding='utf-8',newline='\n')
for filename,files in [('before.zip',original),('candidate.zip',after)]:
    with zipfile.ZipFile(out/filename,'x',zipfile.ZIP_DEFLATED) as z:
        for n,b in files.items():z.writestr(n,b)
    with zipfile.ZipFile(out/filename) as z:assert all(z.read(n)==b for n,b in files.items())
record=dict(status='isolated_G06_cancel_recovery_development_not_built_not_tested',base_source_manifest_sha256=source,
 candidate_files={n:digest(b) for n,b in after.items()},base_files={n:digest(b) for n,b in original.items()},
 before_archive_sha256=sha(out/'before.zip'),candidate_archive_sha256=sha(out/'candidate.zip'),runtime_files=[runtime],test_files=[tests],
 new_tests_written=6,tests_executed=0,builds_executed=0,production_applied=False,GUI_verified=False,whole_goal_complete=False,
 static_defect='cancel unconditionally replaced the saved one-shot recovery with None after its first call had retired teleport and credentials',
 changes=['Repeated cancel preserves unused correlation without retaining password or world authority',
 'Pure private one-shot consumption additionally rechecks exact LOGIN peer, account and source; all mismatches consume',
 'No new send/start, no timeout or gameplay expiry change, no protocol/server/resource edits'],
 actual_failure_reproduction_executed=False,required_later=['Isolated baseline/candidate six appended cases; all original rental-return and teleport cases retained',
 'Original formal75 repeated-cancel/failure then fresh LOGIN to same or changed peer/account/source',
 'G06 controller96, final-source Debug/Release full library and normal GUI handoff/cancel recovery'])
(out/'candidate.json').write_bytes(data(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
baseline_tests=text[tests].split('#[test]\nfn recovery_binding_mismatches_consume_the_one_shot_without_revival()',1)[0]
assert baseline_tests.count('#[test]')==original[tests].decode('utf-8-sig').count('#[test]')+2
(out/'baseline-regression-tests.rs').write_text(baseline_tests,encoding='utf-8',newline='\n')
record['baseline_regression_tests']=str(out/'baseline-regression-tests.rs')
record['baseline_regression_tests_sha256']=sha(out/'baseline-regression-tests.rs')
record['baseline_additional_cases']=2
record['required_later'][0]='Isolated baseline original cancellation regressions (two extracted cases) must fail semantically; candidate all six appended cases, all original return/teleport cases retained'
(out/'candidate.json').write_bytes(data(record))
(out/'README.md').write_text('# G06.3d 連續取消之回復關聯候選\n\n靜態發現：第一次cancel保存關聯後清空teleport及帳密，第二次cancel卻用None覆蓋尚未使用的關聯。候選只在新的已授權傳送存在時替換關聯；清空世界、帳密、背包、操作等原退役邏輯保留。\n\n取用仍由明確來源選擇觸發，額外在取用點核對LOGIN peer／account／source，無論成功或不符都只取一次；沒有新增自動登入、發包、伺服器狀態或時限。\n\n一Runtime檔與一既有測試檔，新增六案例。依使用者要求未編譯、未執行、未套用；靜態缺口不是已重現的runtime故障，也不代表G06.3d驗收完成。\n',encoding='utf-8')
assert all(sha(R/n)==h for n,h in manifest.items())
print(json.dumps(dict(candidate=str(out/'candidate.json'),tests_written=6,tests_executed=0,production_applied=False),ensure_ascii=False))
