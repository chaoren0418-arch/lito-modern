"""Prepare isolated keyboard submission and failed-session retirement candidates, without applying."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json
R=Path(__file__).resolve().parents[1];P=R/'LiTo381-Modern';V=R/'驗證紀錄/current';T=R/'維護工具'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest().upper()
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
base=Path(w['current_native_catalog_library']['run']);inputs=read(base/'compiled-inputs.json');sources=read(base/'source-inputs.json')
assert len(inputs)==1049 and len(sources)==1048 and all(sha((R/n).read_bytes())==h for n,h in inputs.items())
out=T/'candidates'/('native-quantity-enter-failure-retirement-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
names=['native_login_window.rs','native_login.rs','native_rental_return_tests.rs'];prefix='LiTo381-Modern/src/modern_lito/'
before={n:(P/'src/modern_lito'/n).read_bytes()for n in names};after=dict(before)
body=before[names[0]].decode('utf-8');newline='\r\n'if'\r\n'in body else'\n'
anchor='                if world_focus && message.message == WM_KEYDOWN && message.wParam.0 == 0x1b {'
assert body.count(anchor)==1
insert='''                // Submit a received numeric NPC request from its quantity field.
                // The ordinary button path still validates owner, target and deadline.
                if message.message == WM_KEYDOWN && message.wParam.0 == 0x0d {
                    let quantity_field = {
                        let ui = state.0.borrow();
                        ui.rental.open && !ui.closing && !ui.inventory_view
                            && GetFocus() == ui.control(rental::QUANTITY)
                            && ui.controller.snapshot().merchant.input.as_ref().is_some_and(|input| input.mode == 0)
                    };
                    if quantity_field {
                        if message.lParam.0 & (1 << 30) == 0 {
                            let enabled = {
                                let ui = state.0.borrow();
                                IsWindowEnabled(ui.control(rental::ORDER)).as_bool()
                            };
                            if enabled { send(hwnd, WM_COMMAND, WPARAM(rental::ORDER as usize), LPARAM(0)); }
                        }
                        continue;
                    }
                }
'''.replace('\n',newline)
after[names[0]]=body.replace(anchor,insert+anchor).encode('utf-8')
body=before[names[1]].decode('utf-8');at=body.index('    fn fail(&mut self, error: String) {');end=body.index('    fn interrupt_world_actions',at);part=body[at:end]
old='        if self.teleport.is_some()||self.recovery_character.is_some(){self.retire_teleport_scene();}'
assert part.count(old)==1
part=part.replace(old,'        self.retire_teleport_scene();')
after[names[1]]=(body[:at]+part+body[end:]).encode('utf-8')
body=before[names[2]].decode('utf-8');newline='\r\n'if'\r\n'in body else'\n'
checks='''    assert!(c.snapshot().joined_room.is_none() && c.snapshot().rooms.is_empty());
    assert!(c.snapshot().world.map_id.is_none() && c.snapshot().selected_character.is_none());
    assert!(c.snapshot().rental_set.is_none() && c.snapshot().services.is_none());
    assert_eq!(c.snapshot().world.generation,17);
'''.replace('\n',newline)
for name in ['unacknowledged_return_timeout_cannot_open_login_or_keep_credentials','missing_source_return_timeout_cannot_connect_an_owned_login_peer']:
    at=body.index('fn '+name+'()');end=body.index(newline+'}',at);body=body[:end]+newline+checks.rstrip('\r\n')+body[end:]
after[names[2]]=body.encode('utf-8')
diff=[]
for n in names:
    (out/(n+'.before')).write_bytes(before[n]);(out/n).write_bytes(after[n])
    diff.extend(difflib.unified_diff(before[n].decode('utf-8').splitlines(True),after[n].decode('utf-8').splitlines(True),fromfile=prefix+n+'/before',tofile=prefix+n+'/candidate'))
(out/'review.diff').write_text(''.join(diff),encoding='utf-8',newline='')
candidate_sources=dict(sources)
for n in names:candidate_sources[prefix+n]=sha(after[n])
manifest=(json.dumps(candidate_sources,ensure_ascii=False,indent=2)+'\n').encode('utf-8');(out/'candidate-source-inputs.json').write_bytes(manifest)
assert all(sha((R/n).read_bytes())==h for n,h in inputs.items())
record=dict(status='isolated_candidate_not_applied_or_compiled',before_source_manifest_sha256=source,candidate_source_manifest_sha256=sha(manifest),source_files=1048,compiled_input_files=1049,
    candidate=str(out),changed_files=[prefix+n for n in names],before_files={prefix+n:sha(before[n])for n in names},after_files={prefix+n:sha(after[n])for n in names},
    keyboard_scope='Enter only in focused received numeric NPC quantity input; one keydown, same ordinary ORDER handler and all original owner/target/deadline validation; no repeat submission',
    failure_scope='capture optional teleport correlation first, then retire world/services/room/character authority on all failure paths; existing Failed/error/credentials/transport handling retained',
    regression_scope='extend two existing negative timeout cases with retirement and unchanged generation assertions; all original assertions and test names retained',
    game_or_protocol_clocks_changed=False,formal_JAR_modified=False,production_source_modified=False,compiled=False,GUI_verified=False,whole_goal_complete=False,
    required_before_apply='Original active controller session33337 must finish and its original scope be read independently; then candidate negative regressions against unchanged baseline before applying')
(out/'candidate.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(dict(candidate=str(out),candidate_source_manifest_sha256=sha(manifest),production_source_modified=False),ensure_ascii=False))
