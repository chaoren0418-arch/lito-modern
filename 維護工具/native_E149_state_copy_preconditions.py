"""Pure future classifier of original pre-copy reads, never an authority grant."""
def require(ok,message):
 if not ok:raise RuntimeError(message)

def classify(snapshot,contract):
 require(contract['schema']=='E149-state-copy-preconditions-contract-v1' and contract['snapshot_is_copy_authorization'] is False,'wrong original contract')
 require(type(snapshot) is dict and set(snapshot)=={'schema','actor_observer_id','declared_class','declared_fields_scanned','entries','snapshot_is_copy_authorization'} and snapshot['schema']=='G06_E149_state_copy_preconditions_v1','pre-copy observation schema differs')
 require(type(snapshot['actor_observer_id']) is int and snapshot['actor_observer_id']>0 and snapshot['declared_class']=='local.lito381.server.NativeDeWorldKinds$Item' and snapshot['snapshot_is_copy_authorization'] is False,'pre-copy identity/authority differs')
 fields=contract['declared_instance_fields'];names=snapshot['declared_fields_scanned'];rows=snapshot['entries']
 require(type(names) is list and len(names)==len(fields)<=128 and all(type(n) is str for n in names) and len(set(names))==len(names) and set(names)==set(fields),'full original nonstatic field scan missing')
 require(type(rows) is list and len(rows)<=len(names),'pre-copy rows exceed field bound')
 pending=contract['pending_boolean_fields'];generation=contract['pending_generation_field'];mandatory=set(contract['declared_future_fields']+contract['declared_lease_fields']+pending+[generation]);seen=set();previous=-1;blockers=[]
 for row in rows:
  require(type(row) is dict and set(row)=={'field','scan_index','declared_type','present','scheduled_future_instance','reference_observer_id','pending_value'},'pre-copy field row differs')
  name=row['field'];index=row['scan_index'];require(type(name) is str and name in fields and name not in seen and type(index) is int and previous<index<len(names) and names[index]==name and row['declared_type']==fields[name],'pre-copy field identity/order differs');seen.add(name);previous=index
  present,future=row['present'],row['scheduled_future_instance'];require(type(present) is bool and type(future) is bool and (not future or present),'pre-copy presence types differ')
  lease=fields[name].rsplit('$',1)[-1]=='EquipmentTimerLease';require(name in mandatory or future,'unreviewed nonpredicate field row')
  if name in contract['declared_future_fields']:require(future==present,'nonnull ScheduledFuture declaration lacks runtime instance')
  if (future or lease) and present:require(type(row['reference_observer_id']) is int and row['reference_observer_id']>0,'missing actual future/lease identity')
  else:require(row['reference_observer_id'] is None,'unexpected reference identity')
  value=row['pending_value']
  if name in pending:require(present and type(value) is bool,'pending flag type differs')
  elif name==generation:require(present and type(value) is int and -(2**63)<=value<2**63,'pending generation type differs')
  else:require(value is None,'unreviewed scalar observation')
  if future or (lease and present):blockers.append(dict(field=name,reason='nonnull_future_or_declared_lease',rejection=contract['active_rejection']))
  elif (name in pending and value is True) or (name==generation and value>=contract['pending_generation_inclusive_minimum']):
   blockers.append(dict(field=name,reason='pending_expiry_flag_or_nonnegative_generation',rejection=contract['pending_rejection']))
 require(mandatory<=seen,'known original pre-copy condition missing')
 return dict(scope='observed_original_StateCopy_preflight_conditions_only',actor_observer_id=snapshot['actor_observer_id'],declared_fields=len(names),blockers=blockers,
  observed_first_rejection_by_recorded_order=blockers[0]['rejection'] if blockers else None,clear_of_observed_blockers=not blockers,
  future_completion_or_cancellation_inferred=False,original_copy_invoked=False,copy_authorized=False,full_native_owner_qualified=False,
  limitations=['Repeated observations are not an atomic source-copy operation','No blockers does not grant owner/adoption/copy authorization','Actual guarded copy denial, normal GAME timers and natural expiry remain unverified'])
