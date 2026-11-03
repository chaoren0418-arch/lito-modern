"""Prepare guard-probe Java source, without invoking guards or application code."""
from pathlib import Path
from datetime import datetime,timezone
import difflib,hashlib,json,zipfile
from native_E149_recovery_contract import R,FORMAL,read,sha,encoded
T=R/'維護工具';V=R/'驗證紀錄/current'
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
binding=read(w['current_E149_fixture_binding']['qualification'])
assert sha(w['current_E149_fixture_binding']['qualification'])==w['current_E149_fixture_binding']['qualification_sha256']
prior=Path(binding['fixture_candidate']);old=read(prior);assert sha(prior)==binding['fixture_candidate_sha256']
assert all(sha(prior.parent/n)==h for n,h in old['candidate_files'].items())
assert sha(w['current_formal_reference_identity']['formal'])==FORMAL
inspection=V/'native-E149-owner-guard-contract/20261023-224907-224079/inspection.json'
proof=read(inspection);assert proof['formal_target_sha256']==FORMAL and all(x['exit_code']==0 for x in proof['commands'])
assert all(sha(n)==h for n,h in proof['outputs'].items())
life=(inspection.parent/'NativePlayerLifetime.stdout.txt').read_text('utf-8-sig')
start=life.index('  boolean admits();');body=life[start:life.index('\n  boolean processes();',start)]
assert 'Mode.OPEN:' in body and 'ThreadLocal.get:' in body # only DRAINING needs thread-local commit identity
owner=V/'native-E149-item-gameplay-reachability/20261023-224206-735514/NativeDeItemApplication$Owner.stdout.txt'
owner_body=owner.read_text('utf-8-sig')
for message in ['source item owner is stale','source item inventory is staged or inactive']:
 assert message in owner_body
after={n:(prior.parent/n).read_bytes() for n in old['candidate_files']}
observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java'
source=after[observer].decode('utf-8-sig')
assert source.count('    private static String json(Object value) {')==1
source=source.replace('    private static String json(Object value) {','    static String json(Object value) {')
anchor='    private static LinkedHashMap<String,Object> object(Object... pairs) {';assert source.count(anchor)==1
source=source.replace(anchor,'    static long observationId(Object value) { return id(value); }\n'+anchor)
after[observer]=source.encode('utf-8')
harness='LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java'
source=after[harness].decode('utf-8-sig');fragment=T/'templates/NativeRecoveryOwnerGuards.fragment.java'
anchor=' static String stagedSnapshot(GameHub hub)throws Exception {';assert source.count(anchor)==1
source=source.replace(anchor,fragment.read_text('utf-8')+anchor)
anchor='    var previous=new IdentityHashMap<ClientSession,String>();';assert source.count(anchor)==1
source=source.replace(anchor,anchor+'\n    var ownerReferences=new IdentityHashMap<NativeDeItemApplication.Owner,Object[]>();\n    int continuedOwnerControls=0;')
anchor='var stages=Files.newBufferedWriter(root.resolve("staged-trace.jsonl")))';assert source.count(anchor)==1
source=source.replace(anchor,'var stages=Files.newBufferedWriter(root.resolve("staged-trace.jsonl"));var ownerProofs=Files.newBufferedWriter(root.resolve("owner-guard-trace.jsonl")))')
anchor='        String row=NativeRecoveryReadonlySnapshot.observe(session.owner(),hub);';assert source.count(anchor)==1
source=source.replace(anchor,anchor+'\n        rememberOwner(session.owner(),row,ownerReferences);')
anchor='           trace.write(row+"\\n");trace.flush();NativePeerHarness.write(observed,row);';assert source.count(anchor)==1
source=source.replace(anchor,'           continuedOwnerGuards(current,row,ownerProofs,ownerReferences);continuedOwnerControls++;\n'+anchor)
anchor='      Thread.sleep(20);\n     }\n    }';assert source.count(anchor)==1
source=source.replace(anchor,'      Thread.sleep(20);\n     }\n     require(continuedOwnerControls==2,"expected one active-owner control per real recovery");\n     require(sessions.size()==expectedSessions&&sessions.stream().allMatch(NativeRecoveryProductionBootstrap.Session::complete),"original session futures not terminal before retirement guards");\n     retiredOwnerGuards(ownerProofs,ownerReferences);\n    }')
anchor='\\"native_item_attribute_observer_kind\\":\\"G06_E149_native_item_fields_v1\\",';assert source.count(anchor)==2
source=source.replace(anchor,anchor+'\\"owner_guard_probe_kind\\":\\"G06_E149_owner_guard_v1\\",')
after[harness]=source.encode('utf-8')
assert {n for n,b in after.items() if hashlib.sha256(b).hexdigest().upper()!=old['candidate_files'][n]}=={observer,harness}
out=T/'candidates'/('native-E149-owner-guard-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
for name,blob in after.items():
 path=out/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(blob)
 if blob!=(prior.parent/name).read_bytes():
  (out/(Path(name).name+'.diff')).write_text(''.join(difflib.unified_diff((prior.parent/name).read_text('utf-8-sig').splitlines(True),blob.decode('utf-8').splitlines(True),fromfile='prior/'+name,tofile='guard-candidate/'+name)),encoding='utf-8')
record=dict(old,status='owner_guard_probe_source_not_active_not_tested',predecessor_candidate=str(prior),predecessor_candidate_sha256=sha(prior),
 candidate_files={n:hashlib.sha256(b).hexdigest().upper() for n,b in after.items()},changed_files=[observer,harness],
 guard_fragment=str(fragment),guard_fragment_sha256=sha(fragment),guard_API_bytecode_inspection=str(inspection),guard_API_bytecode_inspection_sha256=sha(inspection),
 guard_owner_bytecode=str(owner),guard_owner_bytecode_sha256=sha(owner),Rust_files_changed=0,Runtime_files_changed=0,
 all_Rust_inputs_byte_identical=True,Java_compiled=False,tests_executed=0,original_owner_guards_invoked=False,
 guard_scope=['real continued owner positive control before ACK','real different-account inventory pointer rejection when observed',
  'all four original selected GAME owner admissions rejected after their original Futures finish, before hub shutdown'],
 excluded=['no fake owners or field writes','no adopt/retire or item mutation method calls','no full account/source/payload/timer-denial matrix'],
 integrated_into_producer_reader=False,integrated_into_active_fixture=False,production_applied=False,whole_goal_complete=False)
(out/'candidate.json').write_bytes(encoded(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out/'candidate.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n,b in after.items():z.writestr(n,b)
 z.write(out/'candidate.json','candidate.json')
print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_Java_sources=2,Rust_sources_changed=0,tests_executed=0,guard_calls_executed=0),ensure_ascii=False))
