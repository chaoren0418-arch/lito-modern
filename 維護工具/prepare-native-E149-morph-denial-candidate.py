"""Prepare two real-template morph denials; never initialize game classes."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,difflib,hashlib,json,re,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def method(text,name):
 starts=list(re.finditer(r'(?m)^  (?:public|private|protected) [^\r\n]*\([^\r\n]*\);\r?$',text))
 rows=[text[m.start():starts[i+1].start() if i+1<len(starts) else len(text)] for i,m in enumerate(starts) if ' '+name+'(' in m.group()]
 require(len(rows)==1,'original method absent/ambiguous: '+name);return rows[0]

def prepare(inspection):
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'original workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'deferral/formal changed')
 inspection=owned(inspection.resolve(strict=True));proof=read(inspection);require(proof['status']=='original_morph_entry_bytecode_only' and proof['formal_target_sha256']==FORMAL and all(x['exit_code']==0 for x in proof['commands']) and proof['game_classes_instantiated']==0,'original morph inspection missing');check_pins(proof['outputs'])
 prior_api=V/'native-E149-mutation-denial-contract/20261024-000326-681068/inspection.json';pa=read(prior_api);require(pa['formal_target_sha256']==FORMAL,'bound API target differs');check_pins(pa['outputs']);bound=(prior_api.parent/'NativeDeItemInventory$BoundItem.stdout.txt').read_text('utf-8-sig')
 bodies={name:method(bound,name) for name in ['setItem','setItemId']}
 for name,body in bodies.items():
  require('ifnonnull     13' in body and body.index('Method check:()V')<body.index('class java/lang/IllegalArgumentException') and 'monitorenter' in body,'original morph guard must precede value validation')
 require('morph requires native definition and captured source template together' in bodies['setItem'] and 'source item ID must match captured template' in bodies['setItemId'],'original validation messages changed')
 base_item=(inspection.parent/'NativeDeWorldKinds$Item.stdout.txt').read_text('utf-8-sig');template=(inspection.parent/'NativeDeItemTemplate.stdout.txt').read_text('utf-8-sig')
 require('private int _itemId;' in base_item and 'private int _itemId;' in template and 'Method setGreater:(I)V' in method(base_item,'setItem'),'original item/template field or extended setter route changed')
 ref=w['current_E149_fixture_binding'];require(sha(ref['qualification'])==ref['qualification_sha256'],'canonical binding changed');binding=read(ref['qualification']);prior=Path(binding['fixture_candidate']);old=read(prior);require(sha(prior)==binding['fixture_candidate_sha256'],'canonical candidate changed');check_pins({str(prior.parent/n):h for n,h in old['candidate_files'].items()})
 after={n:(prior.parent/n).read_bytes() for n in old['candidate_files']};harness='LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java';observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java'
 fragments=[T/'templates/NativeRecoveryMorphDenials.fragment.java',T/'templates/NativeRecoveryMorphObserver.fragment.java']
 s=after[harness].decode('utf-8-sig');anchor=' static String stagedSnapshot(GameHub hub)throws Exception {';require(s.count(anchor)==1,'harness insertion anchor differs');s=s.replace(anchor,fragments[0].read_text('utf-8')+anchor)
 replacements={
  '    var mutationTargets=new IdentityHashMap<NativeDeItemApplication.Owner,Object[]>();':'    var mutationTargets=new IdentityHashMap<NativeDeItemApplication.Owner,Object[]>();\n    var morphTargets=new IdentityHashMap<NativeDeItemApplication.Owner,Object[]>();',
  'var mutationProofs=Files.newBufferedWriter(root.resolve("item-mutation-denial-trace.jsonl")))':'var mutationProofs=Files.newBufferedWriter(root.resolve("item-mutation-denial-trace.jsonl"));var morphProofs=Files.newBufferedWriter(root.resolve("item-morph-denial-trace.jsonl")))',
  '        rememberMutationTarget(session.owner(),row,mutationTargets);':'        rememberMutationTarget(session.owner(),row,mutationTargets);\n        rememberMorphTarget(session.owner(),row,mutationTargets,morphTargets);',
  '     require(retiredMutationDenials(mutationProofs,mutationTargets)==32,"native retired mutation matrix incomplete");':'     require(retiredMutationDenials(mutationProofs,mutationTargets)==32,"native retired mutation matrix incomplete");\n     require(retiredMorphDenials(morphProofs,morphTargets)==8,"native retired morph matrix incomplete");',
 }
 for anchor,replacement in replacements.items():require(s.count(anchor)==1,'morph insertion anchor differs: '+anchor);s=s.replace(anchor,replacement)
 anchor='\\"retired_item_mutation_targets\\":4,';require(s.count(anchor)==2,'two original ready/result markers required')
 s=s.replace(anchor,anchor+'\\"retired_item_morph_probe_kind\\":\\"G06_E149_retired_bound_morph_v1\\",\\"retired_item_morph_methods\\":2,\\"retired_item_morph_targets\\":4,')
 after[harness]=s.encode('utf-8');s=after[observer].decode('utf-8-sig');anchor='    static long observationId(Object value) { return id(value); }';require(s.count(anchor)==1,'observer anchor differs');after[observer]=s.replace(anchor,fragments[1].read_text('utf-8')+anchor).encode('utf-8')
 require({n for n,blob in after.items() if hashlib.sha256(blob).hexdigest().upper()!=old['candidate_files'][n]}=={harness,observer},'unexpected source delta')
 out=owned(T/'candidates'/('native-E149-morph-denial-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir()
 for n,blob in after.items():
  target=owned(out/n,out);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(blob)
  if blob!=(prior.parent/n).read_bytes():(out/(Path(n).name+'.diff')).write_text(''.join(difflib.unified_diff((prior.parent/n).read_text('utf-8-sig').splitlines(True),blob.decode('utf-8').splitlines(True),fromfile='canonical/'+n,tofile='morph-candidate/'+n)),encoding='utf-8')
 contract=dict(read(old['contract']),retired_item_morph_probe_kind='G06_E149_retired_bound_morph_v1',original_morph_methods=['setItem','setItemId'],original_morph_bodies=bodies,expected_original_owner_targets=4,planned_additional_morph_attempts=8,planned_aggregate_retired_setter_attempts=40,
  argument_source='two actually existing attached BoundItems in each positively admitted original inventory; alternative template and catalog ID retained by identity, not constructed or replaced',
  acceptance='exact stale-owner IllegalStateException before template/ID validation and all observed state unchanged; validation/detached/staged errors do not qualify',
  original_eight_setter_probe_unchanged=True,original_thirty_two_trace_schema_unchanged=True,no_fake_game_object=True,no_reflective_game_field_write=True,tests_executed=0,
  remaining=['canonical producer/reader/suite binding for new eight receipts','actual existence/capture of four two-template inventories','extended permanent setters normal GAME reachability','isolated source/account branches','timer lease and natural expiry','full G06 and Main'])
 (out/'contract.json').write_bytes(encoded(contract));record=dict(old,status='retired_bound_morph_denial_source_not_active_not_tested',predecessor_candidate=str(prior),predecessor_candidate_sha256=sha(prior),predecessor_fixture_binding=ref,
  contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),candidate_files={n:hashlib.sha256(blob).hexdigest().upper() for n,blob in after.items()},changed_files=[harness,observer],morph_fragments={str(p):sha(p) for p in fragments},morph_API_inspection=str(inspection),morph_API_inspection_sha256=sha(inspection),original_bound_API_inspection=str(prior_api),original_bound_API_inspection_sha256=sha(prior_api),
  morph_methods=['setItem','setItemId'],planned_morph_attempts=8,planned_aggregate_retired_setter_attempts=40,Rust_files_changed=0,Runtime_files_changed=0,all_Rust_inputs_byte_identical=True,Java_compiled=False,original_morph_setters_invoked=False,
  tests_executed=0,integrated_into_producer_reader=False,integrated_into_active_fixture=False,production_applied=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
 for name,sources in [('before.zip',{n:prior.parent/n for n in after}),('candidate.zip',{n:out/n for n in after})]:
  with zipfile.ZipFile(out/name,'x',zipfile.ZIP_DEFLATED) as z:
   for n,p in sources.items():z.write(p,n)
  with zipfile.ZipFile(out/name) as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in sources.items()),'candidate archive readback differs')
 print(json.dumps(dict(candidate=str(out/'candidate.json'),changed_Java_sources=2,Rust_changed=0,additional_morph_attempts_planned=8,aggregate_attempts_planned=40,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('inspection',type=Path);a=p.parse_args();prepare(a.inspection)
