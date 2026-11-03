"""Prepare an unselected exact pre-copy observer, using fixed javap evidence."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,difflib,hashlib,json,re,zipfile
from native_E149_recovery_contract import R,SOURCE,FORMAL,read,sha,encoded,owned,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'

def prepare(api_path):
 w=read(R/'工作佇列.json');require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'],'original workload active')
 require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and sha(w['current_formal_reference_identity']['formal'])==FORMAL,'deferral/formal changed')
 api_path=owned(api_path.resolve(strict=True));api=read(api_path);require(api['formal_target_sha256']==FORMAL and api['game_classes_instantiated']==0 and all(x['exit_code']==0 for x in api['commands']),'original bytecode scope differs');check_pins(api['outputs'])
 item=api_path.parent/'NativeDeWorldKinds$Item.stdout.txt';copy=api_path.parent/'NativeDeItemStateCopy.stdout.txt';text=item.read_text('utf-8-sig');copy_text=copy.read_text('utf-8-sig');start=copy_text.index('  static void copy(');end=copy_text.index('  private static void copyFields(');body=copy_text[start:end]
 for marker in ['Class.getDeclaredFields:','instanceof','class java/util/concurrent/ScheduledFuture','String EquipmentTimerLease','String _magicStoneExpirySubmissionInFlight','String _magicStoneExpiryRetryScheduled','String _pendingMagicStoneExpiryGeneration','iflt          164','Method copyFields:']:
  require(marker in body,'original pre-copy condition differs: '+marker)
 require('isDone:' not in body and 'isCancelled:' not in body and body.index('active source item lifetime cannot be duplicated')<body.index('Method copyFields:') and body.index('pending source item expiry cannot be duplicated')<body.index('Method copyFields:'),'original pre-copy sequence differs')
 head=text[:text.index('  public local.lito381.server.NativeDeWorldKinds$Item(')];fields={}
 for modifiers,kind,name in re.findall(r'(?m)^  ((?:(?:private|public|protected|volatile|transient|final|static) )+)(.+?) (\w+);\r?$',head):
  if 'static ' in modifiers:continue
  kind=re.sub(r'<.*>','',kind)
  if kind.endswith('[]'):
   base=kind[:-2];kind='['+({'int':'I','byte':'B','long':'J','boolean':'Z'}.get(base,'L'+base+';'))
  require(name not in fields,'duplicate original field');fields[name]=kind
 require(0<len(fields)<=128 and fields.get('_pendingMagicStoneExpiryGeneration')=='long' and fields.get('_magicStoneExpirySubmissionInFlight')=='boolean' and fields.get('_magicStoneExpiryRetryScheduled')=='boolean','original declared field inventory differs')
 futures=[n for n,k in fields.items() if k=='java.util.concurrent.ScheduledFuture'];leases=[n for n,k in fields.items() if k.rsplit('$',1)[-1]=='EquipmentTimerLease']
 require(futures==['_magicStoneExpireFuture','_equipmentTimerFuture'] and leases==['_activeEquipmentTimer','_pendingEquipmentExpiry'],'fixed timer field inventory differs')
 ref=w['current_E149_fixture_binding'];require(sha(ref['qualification'])==ref['qualification_sha256'],'canonical binding changed');binding=read(ref['qualification']);prior=Path(binding['fixture_candidate']);old=read(prior);require(sha(prior)==binding['fixture_candidate_sha256'],'canonical fixture changed');check_pins({str(prior.parent/n):h for n,h in old['candidate_files'].items()})
 after={n:(prior.parent/n).read_bytes() for n in old['candidate_files']};observer='LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java';fragment=T/'templates/NativeRecoveryStateCopyPreconditions.fragment.java';source=fragment.read_text('utf-8');require(source.count('/*EXPECTED_FIELDS*/')==1,'manifest insertion anchor differs')
 source=source.replace('/*EXPECTED_FIELDS*/','\n        '.join('expected.put('+json.dumps(n)+','+json.dumps(kind)+');' for n,kind in fields.items()))
 current=after[observer].decode('utf-8-sig');anchor='    static long observationId(Object value) { return id(value); }';require(current.count(anchor)==1,'observer insertion anchor differs');current=current.replace(anchor,source+anchor)
 anchor='"cached_abilities",cachedAbilities(field(actor,"_ability")),"diagnostics",diagnostics);';require(current.count(anchor)==1,'native attribute sidecar anchor differs');current=current.replace(anchor,'"cached_abilities",cachedAbilities(field(actor,"_ability")),"diagnostics",diagnostics,\n            "state_copy_preconditions",stateCopyPreconditions(actor));');after[observer]=current.encode('utf-8')
 out=owned(T/'candidates'/('native-E149-state-copy-observer-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')));out.mkdir()
 for n,blob in after.items():
  target=owned(out/n,out);target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(blob)
 (out/'NativeRecoveryReadonlySnapshot.java.diff').write_text(''.join(difflib.unified_diff((prior.parent/observer).read_text('utf-8-sig').splitlines(True),current.splitlines(True),fromfile='canonical/'+observer,tofile='state-copy-candidate/'+observer)),encoding='utf-8')
 contract=dict(schema='E149-state-copy-preconditions-contract-v1',formal_target_sha256=FORMAL,original_API_inspection=str(api_path),original_API_inspection_sha256=sha(api_path),original_item_fields_source=str(item),original_item_fields_source_sha256=sha(item),original_copy_source=str(copy),original_copy_source_sha256=sha(copy),original_copy_method=body,declared_instance_fields=fields,declared_future_fields=futures,declared_lease_fields=leases,
  pending_boolean_fields=['_magicStoneExpirySubmissionInFlight','_magicStoneExpiryRetryScheduled'],pending_generation_field='_pendingMagicStoneExpiryGeneration',pending_generation_inclusive_minimum=0,
  future_presence_predicate='value instanceof ScheduledFuture, including done/cancelled references; no isDone/isCancelled call',lease_presence_predicate='declared field type simple name EquipmentTimerLease and nonnull value',
  active_rejection='active source item lifetime cannot be duplicated',pending_rejection='pending source item expiry cannot be duplicated',snapshot_is_copy_authorization=False,tests_executed=0)
 (out/'state-copy-contract.json').write_bytes(encoded(contract));(out/'contract.json').write_bytes(encoded(dict(read(old['contract']),state_copy_observer_contract=str(out/'state-copy-contract.json'),state_copy_observer_contract_sha256=sha(out/'state-copy-contract.json'),state_copy_observer_selected=False,tests_executed=0)))
 record=dict(old,status='state_copy_precondition_observer_source_not_selected_not_tested',predecessor_candidate=str(prior),predecessor_candidate_sha256=sha(prior),predecessor_fixture_binding=ref,contract=str(out/'contract.json'),contract_sha256=sha(out/'contract.json'),candidate_files={n:hashlib.sha256(blob).hexdigest().upper() for n,blob in after.items()},changed_files=[observer],
  state_copy_contract=str(out/'state-copy-contract.json'),state_copy_contract_sha256=sha(out/'state-copy-contract.json'),observer_fragment=str(fragment),observer_fragment_sha256=sha(fragment),declared_instance_field_count=len(fields),Rust_files_changed=0,Runtime_files_changed=0,Java_compiled=False,tests_executed=0,original_state_copy_invoked=False,original_timer_methods_invoked=False,integrated_into_producer_reader=False,integrated_into_active_fixture=False,whole_goal_complete=False)
 (out/'candidate.json').write_bytes(encoded(record));(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
 for name,paths in [('before.zip',{n:prior.parent/n for n in after}),('candidate.zip',{n:out/n for n in after})]:
  with zipfile.ZipFile(out/name,'x',zipfile.ZIP_DEFLATED) as z:
   for n,p in paths.items():z.write(p,n)
  with zipfile.ZipFile(out/name) as z:require(all(hashlib.sha256(z.read(n)).hexdigest().upper()==sha(p) for n,p in paths.items()),'candidate archive differs')
 print(json.dumps(dict(candidate=str(out/'candidate.json'),declared_instance_fields=len(fields),known_predicate_entries=7,changed_Java_sources=1,tests_executed=0),ensure_ascii=False))

if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('inspection',type=Path);a=p.parse_args();prepare(a.inspection)
