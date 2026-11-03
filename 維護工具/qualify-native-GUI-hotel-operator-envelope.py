"""Compare newly compiled GUI fixture to the actual preceding fixture, byte for byte."""
from pathlib import Path
import hashlib,json
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current/native-installed-resource-profile-sky-gui'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
old=V/'prepared-20261020-021114-963185';new=V/'prepared-20261020-032053-361901'
a=read(old/'prepared.json');b=read(new/'prepared.json')
assert a['compiled_inputs']==b['compiled_inputs'] and len(b['compiled_inputs'])==1049
assert a['source_manifest_sha256']==b['source_manifest_sha256']=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
assert all(sha(R/n)==h for n,h in b['compiled_inputs'].items())
for key in ['fixture_origins','jar_sha256','catalog_sha256','normal_qualification_sha256','library_completion_sha256','passive_hotel_lease_registry_observation','scenarios']:
    assert a[key]==b[key],key
old_policy=dict(a['operator_wait_policy']);new_policy=dict(b['operator_wait_policy']);assert new_policy.pop('rental_hotel_return_overall_seconds')==1800 and new_policy==old_policy
insert=b'gui&&profile.equals("rental-hotel-return-reconnect")?1_800_000_000_000L:'
for name in ['NativePeerHarness.java','NativeTeleportHarness.java','NativeTeleportFailureHarness.java']:
    before=(old/'fixture-src'/name).read_bytes();after=(new/'fixture-src'/name).read_bytes()
    if name=='NativePeerHarness.java':assert after.count(insert)==1 and after.replace(insert,b'')==before
    else:assert after==before
assert b['fixture_compiler_exit_code']==0 and b['fixture_compiler_original_handle_signalled']
assert set(a['class_files'])==set(b['class_files'])
for name,digest in b['class_files'].items():assert sha(new/'classes'/name)==digest
for profile in ['Debug','Release']:
    for role in ['a','b']:assert a['products'][profile][role]['sha256']==b['products'][profile][role]['sha256']==sha(b['products'][profile][role]['path'])
record=dict(status='independently_reviewed_GUI_operator_envelope_only_no_runtime_acceptance',source_manifest_sha256=b['source_manifest_sha256'],before_prepared=str(old),after_prepared=str(new),prepared_sha256=sha(new/'prepared.json'),all1049compiled_inputs_unchanged=True,all_four_normal_products_unchanged=True,derived_Java_only_one_GUI_profile_outer_expression_changed=True,original_gameplay_authority_protocol_and_socket_waits_unchanged=True,compiler_original_handle_terminal=True,terminal_tool_chunk='e306b1',compiler_exit_code=0,GUI_runtime_rerun=False,GUI_qualification_granted=False)
(new/'orchestration-readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(record,ensure_ascii=False))
