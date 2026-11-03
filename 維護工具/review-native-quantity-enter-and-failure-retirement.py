"""Read the three-file candidate independently; compilation/runtime still required."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re
R=Path(__file__).resolve().parents[1];C=R/'維護工具/candidates/native-quantity-enter-failure-retirement-20261020-034156-331023'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda b:hashlib.sha256(b).hexdigest().upper()
c=read(C/'candidate.json');w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert c['before_source_manifest_sha256']==source
base=Path(w['current_native_catalog_library']['run']);original=read(base/'source-inputs.json');assert sha((json.dumps(original,ensure_ascii=False,indent=2)+'\n').encode('utf-8'))==source
assert len(c['changed_files'])==3
for n in c['changed_files']:
    name=Path(n).name;assert sha((R/n).read_bytes())==sha((C/(name+'.before')).read_bytes())==c['before_files'][n]
    assert sha((C/name).read_bytes())==c['after_files'][n]
ui=(C/'native_login_window.rs').read_bytes();before=(C/'native_login_window.rs.before').read_bytes()
start=ui.index(b'                // Submit a received numeric NPC request from its quantity field.')
end=ui.index(b'                if world_focus && message.message == WM_KEYDOWN && message.wParam.0 == 0x1b {',start)
insert=ui[start:end];assert ui[:start]+ui[end:]==before
for token in [b'WM_KEYDOWN',b'message.wParam.0 == 0x0d',b'!ui.closing',b'!ui.inventory_view',b'GetFocus() == ui.control(rental::QUANTITY)',b'input.mode == 0',b'message.lParam.0 & (1 << 30) == 0',b'IsWindowEnabled(ui.control(rental::ORDER))',b'send(hwnd, WM_COMMAND, WPARAM(rental::ORDER as usize), LPARAM(0))']:assert token in insert
assert insert.count(b'send(')==1
runtime=(C/'native_login.rs').read_bytes();before=(C/'native_login.rs.before').read_bytes();a=before.index(b'    fn fail(&mut self, error: String) {');b=before.index(b'    fn interrupt_world_actions',a)
old=b'        if self.teleport.is_some()||self.recovery_character.is_some(){self.retire_teleport_scene();}'
assert before[a:b].count(old)==1 and runtime==before[:a]+before[a:b].replace(old,b'        self.retire_teleport_scene();')+before[b:]
assert runtime.index(b'if let Some(recovery)=self.teleport_recovery_candidate()',a)<runtime.index(b'        self.retire_teleport_scene();',a)
tests=(C/'native_rental_return_tests.rs').read_bytes();before=(C/'native_rental_return_tests.rs.before').read_bytes()
added=b'    assert!(c.snapshot().joined_room.is_none() && c.snapshot().rooms.is_empty());\n    assert!(c.snapshot().world.map_id.is_none() && c.snapshot().selected_character.is_none());\n    assert!(c.snapshot().rental_set.is_none() && c.snapshot().services.is_none());\n    assert_eq!(c.snapshot().world.generation,17);\n'
assert tests.count(added)==2 and tests.replace(added,b'')==before
assert re.findall(rb'#\[test\]\s*fn (\w+)',tests)==re.findall(rb'#\[test\]\s*fn (\w+)',before)
new=dict(original);new.update(c['after_files']);assert sha((json.dumps(new,ensure_ascii=False,indent=2)+'\n').encode('utf-8'))==c['candidate_source_manifest_sha256']
record=dict(status='three_file_candidate_static_scope_reviewed_not_runtime_qualified',before_source_manifest_sha256=source,candidate_source_manifest_sha256=c['candidate_source_manifest_sha256'],candidate_sha256=sha((C/'candidate.json').read_bytes()),reviewed_utc=datetime.now(timezone.utc).isoformat(),only_focused_quantity_Enter_same_ordinary_command_path=True,key_repeat_suppressed=True,original_owner_target_quantity_and_deadline_checks_unchanged=True,only_fail_retirement_conditional_removed=True,optional_recovery_captured_before_retirement=True,original_two_negative_tests_extended_without_removing_assertions=True,test_inventory_unchanged=True,normal_source_still_unchanged=True,compiled=False,GUI_verified=False,whole_goal_complete=False)
(C/'review.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(record,ensure_ascii=False))
