"""Stage cancel retirement and one regression, preserving the original assertions."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
proof=w['current_native_cancel_retirement_counterexample'];assert proof['qualified_counterexample'] and proof['original_identities_terminal'] and sha(proof['qualification'])==proof['qualification_sha256']
source=w['current_product_identity']['source_manifest_sha256'];assert source==proof['source_manifest_sha256']
names=['LiTo381-Modern/src/modern_lito/native_login.rs','LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs']
old={n:(R/n).read_bytes() for n in names};text={n:b.decode('utf-8').replace('\r\n','\n') for n,b in old.items()}
runtime=text[names[0]];start=runtime.index('    pub fn cancel(&mut self) {');end=runtime.index('    fn accept(&mut self, packet: Packet)',start);cancel=runtime[start:end]
conditional='        if self.teleport.is_some()||self.recovery_character.is_some(){self.retire_teleport_scene();}\n';assert cancel.count(conditional)==1
cancel=cancel.replace(conditional,'');anchor='        self.teleport_recovery=self.teleport_recovery_candidate();\n';assert cancel.count(anchor)==1
cancel=cancel.replace(anchor,anchor+'        self.retire_teleport_scene();\n');new_runtime=runtime[:start]+cancel+runtime[end:]
regression='''
#[test]
fn cancel_live_world_retires_snapshot_without_losing_generation() {
    let listener=owned_listener();
    let mut c=reconnect_fixture(listener.local_addr().unwrap());
    c.state.phase=LoginPhase::WorldReady;
    c.rental_restart_quit_pending=false;
    c.state.world.learned_skills.insert(43);
    c.state.world.mp=Some([70,110]);
    assert!(c.teleport.is_none()&&c.recovery_character.is_none());
    c.cancel();c.poll();
    assert!(c.is_quiescent()&&c.credentials.is_none());
    assert!(c.state.world.map_id.is_none()&&c.state.world.mp.is_none());
    assert!(c.state.world.learned_skills.is_empty()&&c.state.world.effects.is_empty());
    assert!(c.state.joined_room.is_none()&&c.state.selected_character.is_none());
    assert!(c.state.services.is_none()&&c.state.inventory.owner.is_none());
    assert!(!c.state.game_connected&&c.state.rental_set.is_none());
    assert_eq!(c.state.world.generation,17);
    assert_eq!(c.transport.snapshot().epoch,0);
    assert_eq!(listener.accept().unwrap_err().kind(),std::io::ErrorKind::WouldBlock);
}
'''
new_tests=text[names[1]]+regression;assert new_tests.startswith(text[names[1]])
out=R/'維護工具/candidates'/('native-cancel-retirement-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
staged={names[0]:new_runtime,names[1]:new_tests};files={};diff=[]
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,b in old.items():z.writestr(n,b)
with zipfile.ZipFile(out/'before.zip') as z:assert all(z.read(n)==b for n,b in old.items())
for n,body in staged.items():
    p=out/'files'/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(body,encoding='utf-8',newline='\n');files[n]=str(p)
    diff.extend(difflib.unified_diff(text[n].splitlines(True),body.splitlines(True),fromfile=n+'/before',tofile=n+'/after'))
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='');(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
j=dict(status='prepared_cancel_retirement_candidate_unapplied',base_source_manifest_sha256=source,before={n:hashlib.sha256(b).hexdigest().upper() for n,b in old.items()},after={n:sha(p) for n,p in files.items()},candidate_files=files,original_counterexample=proof['qualification'],original_counterexample_sha256=proof['qualification_sha256'],related_haste_candidate=w['current_native_haste43_components']['candidate'],related_haste_candidate_sha256=w['current_native_haste43_components']['candidate_sha256'],runtime_files=1,existing_test_file_extended=1,new_test_functions=1,original_test_bytes_preserved_except_line_endings=True,protocol_or_gameplay_clocks_changed=False,source_applied=False,component_tests_executed=False,formal_repair_runtime_executed=False,GUI_verified=False,whole_goal_complete=False,review_sha256=sha(out/'review.diff'),before_archive_sha256=sha(out/'before.zip'))
(out/'candidate.json').write_bytes(data(j));assert all((R/n).read_bytes()==b for n,b in old.items());print(json.dumps(dict(candidate=str(out/'candidate.json'),files=2,applied=False),ensure_ascii=False))
