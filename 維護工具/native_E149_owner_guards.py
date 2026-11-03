"""Pure readback of original owner guard receipts. No process or game APIs.

Receipt ordering is combined with pinned Java control flow, original observer
identities and terminal process evidence in the outer reader. It does not
qualify mutations or independently isolate every account/source guard branch.
"""
import json
SCHEMA='G06_E149_owner_guard_v1'
FIELDS={'schema','kind','guard_method','observer_owner_id','session_identity','inventory_observer_id',
 'source_qualified_character','source_qualified_account','owner_active_observed','owner_matches_session',
 'rejection','foreign_observer_owner_id','source_observation','item_mutation_attempted','full_native_owner_qualified'}

def require(ok,message):
 if not ok:raise RuntimeError(message)

def positive_id(value):return type(value) is int and value>0

def signature(row):return json.dumps(row,sort_keys=True,ensure_ascii=False,separators=(',',':'))

def verify_owner_guards(rows,trace,roles,source_identity):
 require(type(rows) is list and 7<=len(rows)<=8,'expected two live controls, one/two foreign rejections and four retirements')
 require(len(roles)==2 and {r['account'] for r in roles}=={'PeerFixture','PeerFixture2'},'guard role scope differs')
 owners={};sessions=set();selected={}
 for role in roles:
  for prefix in ['old','new']:
   oid,sid=role[prefix+'_owner_id'],role[prefix+'_session_id']
   require(positive_id(oid) and positive_id(sid) and oid not in owners and sid not in sessions,'duplicate selected owner/session identity')
   owners[oid]=dict(account=role['account'],session=sid,recovered=prefix=='new');sessions.add(sid)
  selected[role['new_owner_id']]=role['selected_observation']
 trace_rows={signature(row) for row in trace}
 active={};retired={};foreign=[];first_retired=None
 for index,row in enumerate(rows):
  require(type(row) is dict and set(row)==FIELDS and row['schema']==SCHEMA,'owner guard receipt shape differs')
  require(row['guard_method']=='NativeDeItemApplication.Owner.requireActive','different original authorization guard')
  require(row['item_mutation_attempted'] is False and row['full_native_owner_qualified'] is False,'guard receipt grants unsupported mutation/full-owner scope')
  oid,sid,iid=[row[k] for k in ['observer_owner_id','session_identity','inventory_observer_id']]
  require(all(positive_id(v) for v in [oid,sid,iid]) and oid in owners and owners[oid]['session']==sid,'guard receipt owner/session identity differs')
  require(type(row['owner_active_observed']) is bool and type(row['owner_matches_session']) is bool,'guard flag types differ')
  raw=row['source_observation'];require(type(raw) is str and len(raw)<=8_388_608,'guard source observation missing/too large')
  observation=json.loads(raw);require(type(observation) is dict and signature(observation) in trace_rows,'guard observation is absent from original server trace')
  owner=observation['source_item_owner'];require(type(owner) is dict,'original owner observation absent')
  require(observation['observer_kind']=='G06_E149_direct_fields_v1' and observation['stable_observed'] is True
   and owner['observer_owner_id']==oid and owner['active'] is True and owner['session_matches'] is True
   and observation['session_identity']==sid,'guard source observation is not this originally active owner/session')
  require(observation['test_account']==owners[oid]['account'] and observation['source']=='測試乙','guard source/account binding differs')
  require(row['source_qualified_account']==owner['account']==source_identity(observation['source'],observation['test_account'])
   and row['source_qualified_character']==owner['source']==source_identity(observation['source'],observation['character']),
   'guard source-qualified identity differs')
  kind=row['kind'];foreign_id=row['foreign_observer_owner_id']
  if kind=='active_accept':
   require(first_retired is None and owners[oid]['recovered'] and oid not in active,'duplicate/nonrecovered/late live guard control')
   require(row['owner_active_observed'] is True and row['owner_matches_session'] is True
    and row['rejection'] is None and foreign_id is None,'original live guard did not accept its own inventory')
   require(observation==selected[oid] and observation['phase']=='LOBBY','guard control did not precede this continuation ACK')
   inventory=observation['native_inventory_state']
   require(inventory['exists'] is True and inventory['stable_observed'] is True
    and inventory['native_owner_id']==oid and inventory['observer_inventory_id']==iid,'live guard checked another inventory')
   active[oid]=dict(index=index,inventory=iid,account=owners[oid]['account'])
  elif kind=='foreign_inventory_reject':
   require(first_retired is None and oid in active and positive_id(foreign_id) and foreign_id in owners
    and foreign_id!=oid and owners[foreign_id]['account']!=owners[oid]['account'],'foreign guard identity/order differs')
   require(observation==selected[oid] and row['owner_active_observed'] is True and row['owner_matches_session'] is True
    and row['rejection']=='source item inventory is staged or inactive','foreign inventory guard was not the expected live pointer-fence rejection')
   require(all(x['owner']!=oid for x in foreign),'duplicate foreign guard receipt for one live control')
   foreign.append(dict(owner=oid,foreign_owner=foreign_id,inventory=iid,index=index))
  elif kind=='retired_reject':
   first_retired=index if first_retired is None else first_retired
   require(oid not in retired and foreign_id is None and row['owner_active_observed'] is False
    and row['owner_matches_session'] is False and row['rejection']=='source item owner is stale','retired owner was not precisely rejected')
   ids={state.get('observer_inventory_id') for state in [observation['native_inventory_state'],observation['rental_inventory_state']] if state['exists']}
   require(iid in ids,'retired receipt inventory is absent from its retained original observation')
   terminal=[x for x in trace if x['session_identity']==sid][-1]
   require(terminal['stable_observed'] is True and terminal['source_item_disconnected'] is True
    and (terminal['source_item_owner'] is None or terminal['source_item_owner']['active'] is False),'original source trace did not observe owner retirement')
   retired[oid]=dict(index=index,inventory=iid,account=owners[oid]['account'])
  else:raise RuntimeError('unreviewed original owner guard receipt kind')
 require(set(active)==set(selected) and set(retired)==set(owners) and 1<=len(foreign)<=2,'guard matrix receipts incomplete')
 require(first_retired is not None and all(row['index']<first_retired for row in active.values()),'retirement receipts precede live controls')
 for row in foreign:
  require(row['inventory']==retired[row['foreign_owner']]['inventory'] and row['inventory']!=active[row['owner']]['inventory'],
   'foreign receipt inventory is not the actual other admission reference')
 return dict(scope='two_recovered_live_controls_foreign_inventory_fence_and_four_actual_retirements',
  active_controls=len(active),foreign_inventory_rejections=len(foreign),retired_rejections=len(retired),
  per_role=[dict(account=r['account'],old_owner=r['old_owner_id'],new_owner=r['new_owner_id'],
   recovered_inventory=active[r['new_owner_id']]['inventory'],both_actual_owner_admissions_retired=True) for r in roles],
  foreign_matches_and_post_rejection_positive_control_by_pinned_fixture_flow=True,
  retirements_after_original_Futures_and_before_fixture_close_by_pinned_flow=True,
  item_mutation_attempted=False,independent_account_source_branch_matrix_qualified=False,
  full_native_owner_qualified=False,full_attribute_matrix_qualified=False)
