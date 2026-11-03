"""Prepare a public production-rlib probe; retain the prior candidate flow assertions."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json,difflib
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current';sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
out=V/'native-text-index-order-tooling'/('production43-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
origin=T/'native-haste43-live-body.rs';before=origin.read_text('utf-8-sig');body=before.replace('candidate::','lito_modern::modern_lito::').replace('candidate_haste43_','current_haste43_').replace('"candidate label"','"current production label"')
assert all(line.replace('candidate::','lito_modern::modern_lito::').replace('candidate_haste43_','current_haste43_').replace('"candidate label"','"current production label"') in body for line in before.splitlines() if 'ensure!' in line)
target_body=T/'native-haste43-production-body.rs';assert not target_body.exists();target_body.write_text(body,encoding='utf-8',newline='\n');(out/'body-before.rs').write_bytes(origin.read_bytes());(out/'body.diff').write_text(''.join(difflib.unified_diff(before.splitlines(True),body.splitlines(True),fromfile=origin.name,tofile=target_body.name)),encoding='utf-8')
origin=T/'run-current-haste43-live-candidate.py';old=origin.read_text('utf-8-sig');new=old.replace('A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6','3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628')
new=new.replace("choices=['self-peer','mp-refused','cancel-retirement','cancel-repaired']","choices=['self-peer','mp-refused','cancel-repaired']").replace(";ap.add_argument('--controller-candidate',type=Path)",'')
start=new.index("component=w['current_native_haste43_components']");end=new.index("normal=Path(w['current_native_installed_resource_profile_normal_qualification']['run'])",start)
new=new[:start]+'''from text_index_order_runtime_gate import require_idle_runtime
require_idle_runtime(R)
if args.scenario!='self-peer':
    positive=w['current_native_haste43_production_self_peer_formal75'];assert positive['source_manifest_sha256']==source and positive['qualified'] and positive['actual_formal_casts']==4 and sha(positive['qualification'])==positive['qualification_sha256']
'''+new[end:]
new=new.replace("    assert all(sha(candidate['candidate_files'][n])==h for n,h in candidate['after'].items())\n",'')
new=new.replace("out=V/'native-haste43-live-candidate'","out=V/'native-haste43-live-current'")
start=new.index("controller=P/'src/modern_lito/native_login.rs'");end=new.index("fixture=P/'tests/oracle/NativePeerHarness.java'",start)
new=new[:start]+'''body=T/'native-haste43-production-body.rs';(out/'live-body.rs').write_bytes(body.read_bytes())
(out/'main.rs').write_text('include!(r#"'+str(out/'live-body.rs')+'"#);\\n',encoding='utf-8',newline='\\n')
'''+new[end:]
new=new.replace("[out/'native_login.rs',out/'live-body.rs',out/'main.rs',out/'NativePeerHarness.java',out/'runner.py']","[out/'live-body.rs',out/'main.rs',out/'NativePeerHarness.java',out/'runner.py',body]")
start=new.index("state=dict(status='preparing_off_tree_haste43_live_candidate'");end=new.index("save(out/'cohort.json',state);print",start)
new=new[:start]+'''state=dict(status='preparing_current_production_haste43',source_manifest_sha256=source,pipeline=dict(pid=os.getpid(),creation_utc=birth(k.GetCurrentProcess())),process_history=[],owned_processes=[],profiles={},compiled_inputs=inputs,generated_source_pins=dict(extra_pins),fixture_origin=str(fixture),fixture_sha256=sha(fixture),jar=str(jar),jar_sha256=sha(jar),catalog=str(catalog),catalog_sha256=sha(catalog),normal_qualification=w['current_native_installed_resource_profile_normal_qualification']['qualification'],normal_qualification_sha256=w['current_native_installed_resource_profile_normal_qualification']['qualification_sha256'],normal_product_modified=False,production_rlib_used=True,production_source_overlays_used=False,candidate_applied=False,GUI_verified=False,whole_goal_complete=False)
state['scenario']=args.scenario;state['fixture_profile']='heal19-refused' if args.scenario=='mp-refused' else 'melee'
'''+new[end:]
new=new.replace("broad=w['current_native_catalog_broad_qualification']['profiles'][profile];build=Path(broad['run'])/'build/stdout.log'","build=normal/profile/'test-build/stdout.log'")
new=new.replace('off_tree_haste43_live_candidate','current_production_haste43').replace('candidate_haste43_live_','current_production_haste43_').replace('candidate_haste43_','current_haste43_')
new=new.replace('"""Off-tree controller prefix plus candidate modules against unmodified formal75.\n\nNo product source is applied. The original default memory fixture is unmodified.\n"""','"""Current public production-rlib APIs against unmodified formal75; no Runtime overlays."""')
assert 'controller_proposal' not in new and 'controller_candidate' not in new and "candidate['" not in new and 'component[' not in new and "out/'native_login.rs'" not in new
ast.parse(new);target=T/'run-current-production-haste43.py';assert not target.exists();target.write_text(new,encoding='utf-8',newline='\n');(out/'runner-before.py').write_bytes(origin.read_bytes());(out/'runner.diff').write_text(''.join(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile=origin.name,tofile=target.name)),encoding='utf-8')
record=dict(source_manifest_sha256='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628',original_body=str(T/'native-haste43-live-body.rs'),original_body_sha256=sha(T/'native-haste43-live-body.rs'),current_body=str(target_body),current_body_sha256=sha(target_body),original_runner=str(origin),original_runner_sha256=sha(origin),current_runner=str(target),current_runner_sha256=sha(target),production_controller_and_helpers_copied=False,production_rlib_linked_directly=True,all_flow_guards_preserved=True,full_library_completion_required=True,fixture_and_clocks_unchanged=True,executed=False,reader_still_required_before_execution=True,whole_goal_complete=False)
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');(out/'preparer.py').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(preparation=str(out/'preparation.json'),executed=False,reader_still_required=True),ensure_ascii=False))
