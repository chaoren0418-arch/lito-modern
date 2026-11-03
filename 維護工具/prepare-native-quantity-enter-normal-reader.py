"""Prepare the next normal-product reader without changing any active QA or product."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
origin=T/'qualify-native-installed-resource-profile-normal.py';body=origin.read_text('utf-8-sig')
start=body.index("assert manifest=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'")
end=body.index('release=tomllib.loads',start)
scope='''assert manifest=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert candidate['before_source_manifest_sha256']==application['before_source_manifest_sha256']=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
assert candidate['candidate_source_manifest_sha256']==manifest and not candidate['game_or_protocol_clocks_changed']
review=read(application['candidate_review']);assert sha(application['candidate_review'])==application['candidate_review_sha256']
assert review['candidate_sha256']==application['candidate_sha256']and review['only_fail_retirement_conditional_removed']and review['key_repeat_suppressed']
old=read(app/'before-compiled-inputs.json');assert set(old)==set(inputs)and len(old)==1049
changed={n for n,h in inputs.items()if old[n]!=h}
expected_changes={'LiTo381-Modern/src/modern_lito/'+n for n in ['native_login_window.rs','native_login.rs','native_rental_return_tests.rs']}
assert changed==set(application['changed_files'])==set(candidate['after_files'])==expected_changes
assert application['created_files']==[]and application['after_files']==candidate['after_files']
assert application['normal_product_source_modified']and not application['diagnostic_API_extension']and not application['clocks_changed']
verify_archive(app/'before.zip',application['before_files']);assert sha(app/'before.zip')==application['before_archive_sha256']
verify_archive(app/'applied-payload.zip',application['after_files']);assert sha(app/'applied-payload.zip')==application['payload_archive_sha256']
with zipfile.ZipFile(app/'before.zip')as z:before_sources={n:z.read(n)for n in expected_changes}
for n in expected_changes:assert hashlib.sha256(before_sources[n]).hexdigest().upper()==old[n]==candidate['before_files'][n]
ui_name='LiTo381-Modern/src/modern_lito/native_login_window.rs';ui=(ROOT/ui_name).read_bytes();before_ui=before_sources[ui_name]
at=ui.index(b'                // Submit a received numeric NPC request from its quantity field.')
end=ui.index(b'                if world_focus && message.message == WM_KEYDOWN && message.wParam.0 == 0x1b {',at)
insert=ui[at:end];assert ui[:at]+ui[end:]==before_ui
for token in [b'WM_KEYDOWN',b'message.wParam.0 == 0x0d',b'!ui.closing',b'!ui.inventory_view',b'GetFocus() == ui.control(rental::QUANTITY)',b'input.mode == 0',b'message.lParam.0 & (1 << 30) == 0',b'IsWindowEnabled(ui.control(rental::ORDER))',b'send(hwnd, WM_COMMAND, WPARAM(rental::ORDER as usize), LPARAM(0))']:assert token in insert
assert insert.count(b'send(')==1
name='LiTo381-Modern/src/modern_lito/native_login.rs';before_login=before_sources[name];login=(ROOT/name).read_bytes()
at=before_login.index(b'    fn fail(&mut self, error: String) {');end=before_login.index(b'    fn interrupt_world_actions',at)
old_if=b'        if self.teleport.is_some()||self.recovery_character.is_some(){self.retire_teleport_scene();}'
assert before_login[at:end].count(old_if)==1 and login==before_login[:at]+before_login[at:end].replace(old_if,b'        self.retire_teleport_scene();')+before_login[end:]
assert login.index(b'if let Some(recovery)=self.teleport_recovery_candidate()',at)<login.index(b'        self.retire_teleport_scene();',at)
name='LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs';tests=(ROOT/name).read_bytes()
added=b'    assert!(c.snapshot().joined_room.is_none() && c.snapshot().rooms.is_empty());\\n    assert!(c.snapshot().world.map_id.is_none() && c.snapshot().selected_character.is_none());\\n    assert!(c.snapshot().rental_set.is_none() && c.snapshot().services.is_none());\\n    assert_eq!(c.snapshot().world.generation,17);\\n'
assert tests.count(added)==2 and tests.replace(added,b'')==before_sources[name]
assert old['LiTo381-Modern/src/modern_lito/native_rental_window.rs']==inputs['LiTo381-Modern/src/modern_lito/native_rental_window.rs']
prior=read(application['before_normal_qualification']);assert sha(application['before_normal_qualification'])==application['before_normal_qualification_sha256']
assert prior['source_manifest_sha256']==application['before_source_manifest_sha256']and prior['passed_per_profile']==19 and prior['recorded_owned_processes_all_terminal']
negative=application['negative_baseline'];assert sha(negative['qualification'])==negative['qualification_sha256']
assert negative['source_manifest_sha256']==application['before_source_manifest_sha256']and negative['baseline_counterexamples_verified']and negative['original_source_and_library_PE_restored']
'''
body=body[:start]+scope+body[end:]
anchor="files['qualifier.py']=Path(__file__)";assert body.count(anchor)==1
body=body.replace(anchor,anchor+";files.update({'application.json':app/'application.json','candidate.json':Path(application['candidate']),'candidate-review.json':Path(application['candidate_review']),'negative-baseline.json':Path(negative['qualification'])})")
target=T/'qualify-native-quantity-enter-failure-retirement-normal.py';assert not target.exists()
compile(body,str(target),'exec');target.write_text(body,encoding='utf-8',newline='\n')
out=V/'native-quantity-enter-failure-tooling'/('normal-reader-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
record=dict(origin=str(origin),origin_sha256=hashlib.sha256(origin.read_bytes()).hexdigest().upper(),prepared=str(target),prepared_sha256=hashlib.sha256(target.read_bytes()).hexdigest().upper(),new_source_manifest_sha256='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6',original_normal19_PE_commands_sources_archives_and_terminal_checks_retained=True,only_scope_specific_three_file_and_negative_baseline_binding_replaced=True,active_QA_changed=False,runtime_executed=False)
(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(record,ensure_ascii=False))
