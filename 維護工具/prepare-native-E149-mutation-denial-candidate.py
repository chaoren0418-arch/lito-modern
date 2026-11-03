"""Prepare real-retired-actor setter probes. Does not invoke any game API."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,ast,difflib,hashlib,json,re,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
METHODS=['setCount','setEquipped','setIdentified','setSealed','setEnchantLevel','set_durability','setBless','setChargeCount']

def prepare(inspection):
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'original workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'this preparer is development-only')
 require(sha(w['current_formal_reference_identity']['formal'])==FORMAL,'formal target changed')
 inspection=owned(inspection.resolve(strict=True));proof=read(inspection)
 require(proof['status']=='original_mutation_entry_bytecode_only' and proof['formal_target_sha256']==FORMAL
  and proof['game_classes_instantiated']==0 and all(row['exit_code']==0 for row in proof['commands']),'original API inspection missing')
 check_pins(proof['outputs']);body=(inspection.parent/'NativeDeItemInventory$BoundItem.stdout.txt').read_text('utf-8-sig')
 for method in METHODS:
  match=re.search(r'(?m)^  public void '+re.escape(method)+r'\([^\n]+\);\r?$',body);require(match is not None,'original BoundItem setter absent: '+method)
  following=re.search(r'(?m)^  (?:public|private) ',body[match.end():]);code=body[match.end():match.end()+following.start()] if following else body[match.end():]
  require('ifnonnull     13' in code and 'Method check:()V' in code and 'monitorenter' in code,'bound store/guard branch differs')
  bound=code[code.index('Method check:()V'):]
  require('Method local/lito381/server/NativeDeWorldKinds$Item.'+method+':' in bound,'guard no longer precedes original mutator')
 identity=read(w['current_E149_fixture_binding']['qualification']);require(sha(w['current_E149_fixture_binding']['qualification'])==w['current_E149_fixture_binding']['qualification_sha256'],'fixture binding changed')
 prior=Path(identity['fixture_candidate']);old=read(prior);require(sha(prior)==identity['fixture_candidate_sha256'],'canonical fixture candidate changed')
 after={name:(prior.parent/name).read_bytes() for name in old['candidate_files']}
 require(all(hashlib.sha256(blob).hexdigest().upper()==old['candidate_files'][name] for name,blob in after.items()),'canonical fixture sources changed')
 observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java';harness='LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java'
 observer_fragment=T/'templates/NativeRecoveryMutationDenialObserver.fragment.java';probe_fragment=T/'templates/NativeRecoveryMutationDenials.fragment.java'
 text=after[observer].decode('utf-8-sig');anchor='    static long observationId(Object value) { return id(value); }';require(text.count(anchor)==1,'observer insertion anchor differs')
 after[observer]=text.replace(anchor,observer_fragment.read_text('utf-8')+anchor).encode('utf-8')
 text=after[harness].decode('utf-8-sig');anchor=' static String stagedSnapshot(GameHub hub)throws Exception {';require(text.count(anchor)==1,'harness insertion anchor differs')
 text=text.replace(anchor,probe_fragment.read_text('utf-8')+anchor)
 replacements={
  '    int continuedOwnerControls=0;':'    int continuedOwnerControls=0;\n    var mutationTargets=new IdentityHashMap<NativeDeItemApplication.Owner,Object[]>();',
  'var ownerProofs=Files.newBufferedWriter(root.resolve("owner-guard-trace.jsonl")))':'var ownerProofs=Files.newBufferedWriter(root.resolve("owner-guard-trace.jsonl"));var mutationProofs=Files.newBufferedWriter(root.resolve("item-mutation-denial-trace.jsonl")))',
  '        rememberOwner(session.owner(),row,ownerReferences);':'        rememberOwner(session.owner(),row,ownerReferences);\n        rememberMutationTarget(session.owner(),row,mutationTargets);',
  '     retiredOwnerGuards(ownerProofs,ownerReferences);':'     retiredOwnerGuards(ownerProofs,ownerReferences);\n     require(retiredMutationDenials(mutationProofs,mutationTargets)==32,"native retired mutation matrix incomplete");',
 }
 for before,replacement in replacements.items():
  require(text.count(before)==1,'mutation insertion anchor differs: '+before);text=text.replace(before,replacement)
 anchor='\\"owner_guard_probe_kind\\":\\"G06_E149_owner_guard_v1\\",';require(text.count(anchor)==2,'ready/result guard tag anchor differs')
 text=text.replace(anchor,anchor+'\\"retired_item_mutation_probe_kind\\":\\"G06_E149_retired_bound_item_mutation_v1\\",\\"retired_item_mutation_methods\\":8,\\"retired_item_mutation_targets\\":4,')
 after[harness]=text.encode('utf-8')
 require({name for name,blob in after.items() if hashlib.sha256(blob).hexdigest().upper()!=old['candidate_files'][name]}=={observer,harness},'unexpected candidate source delta')
 out=owned(T/'candidates'/('native-E149-mutation-denial-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir()
 for name,blob in after.items():
  path=owned(out/name,out);path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(blob)
  if blob!=(prior.parent/name).read_bytes():
   (out/(Path(name).name+'.diff')).write_text(''.join(difflib.unified_diff((prior.parent/name).read_text('utf-8-sig').splitlines(True),blob.decode('utf-8').splitlines(True),fromfile='canonical/'+name,tofile='mutation-candidate/'+name)),encoding='utf-8')
 contract=dict(read(old['contract']),retired_item_mutation_probe_kind='G06_E149_retired_bound_item_mutation_v1',
  original_bound_setters=METHODS,expected_actual_owner_targets=4,planned_mutation_attempts=32,
  captures='original attached BoundItem references, positively guarded while their actual session is active',
  invocation_barrier='all original session Futures terminal, existing four retirement guards complete, before app.close',
  acceptance='exact stale-owner IllegalStateException, changed argument, full retained before/after observation equal',
  no_fake_owner_or_actor=True,no_reflective_game_field_write=True,not_executed=True,full_native_owner_qualified=False,
  remaining=['extended attribute setters and normal GAME creation of nondefault states','source/account branch isolation',
   'staged and detached actor denial','original timer leases and natural expiry','separate pure receipt reader and canonical producer/reader integration'])
 (out/'contract.json').write_bytes(encoded(contract))
 record=dict(old,status='retired_bound_item_mutation_denial_source_not_active_not_tested',
  predecessor_candidate=str(prior),predecessor_candidate_sha256=sha(prior),contract=str(out/'contract.json'),
  candidate_files={name:hashlib.sha256(blob).hexdigest().upper() for name,blob in after.items()},changed_files=[observer,harness],
  mutation_fragments={str(path):sha(path) for path in [probe_fragment,observer_fragment]},
  mutation_API_inspection=str(inspection),mutation_API_inspection_sha256=sha(inspection),
  mutation_methods=METHODS,planned_native_owner_targets=4,planned_retired_setter_attempts=32,
  Rust_files_changed=0,Runtime_files_changed=0,all_Rust_inputs_byte_identical=True,Java_compiled=False,
  guard_scope=old['guard_scope']+['eight actual bound setters denied for each of four retired original admissions'],
  excluded=['no fake owner, actor, template or inventory','no reflective game field writes or clocks injected',
   'no qualification for extended attribute mutation, isolated source/account branches, timers or full owner lifecycle'],
  tests_executed=0,original_mutation_setters_invoked=False,integrated_into_producer_reader=False,
  integrated_into_active_fixture=False,production_applied=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
 with zipfile.ZipFile(out/'candidate.zip','x',zipfile.ZIP_DEFLATED) as z:
  for name,blob in after.items():z.writestr(name,blob)
  for name in ['candidate.json','contract.json','preparer.py']:z.write(out/name,name)
 print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_Java_sources=2,Rust_sources_changed=0,planned_setter_attempts=32,tests_executed=0),ensure_ascii=False))

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('inspection',type=Path);a=ap.parse_args();prepare(a.inspection)
if __name__=='__main__':main()
