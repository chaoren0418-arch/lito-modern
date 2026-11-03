"""Pure future readback of retained real-actor mutation denial receipts.

The outer reader must bind the pinned fixture flow, original process/Future
cleanup, server trace and role observations. These functions start nothing.
"""
import json
from native_E149_item_attributes import observed_attributes
METHODS=['setCount','setEquipped','setIdentified','setSealed','setEnchantLevel','set_durability','setBless','setChargeCount']
CORE_FIELDS=['count','equipped','identified','sealed','enchant_level','durability','bless','charge_count']
SCHEMA='G06_E149_retired_bound_item_mutation_v1'
FIELDS={'schema','method','argument','observer_owner_id','session_identity','inventory_observer_id','actor_observer_id','item_id',
 'source_qualified_character','source_qualified_account','positive_guard_at_capture','owner_retired_before_attempt',
 'actor_attached_before_attempt','original_session_futures_completed_before_probe','attempted_real_setter',
 'argument_differs_from_original','rejection_class','rejection','before_state','after_state','source_observation',
 'full_native_owner_qualified','full_attribute_matrix_qualified'}

def require(ok,message):
 if not ok:raise RuntimeError(message)
def positive_id(value):return type(value) is int and value>0
def signature(value):return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':'))
def object_text(raw):
 require(type(raw) is str and len(raw)<=8_388_608,'missing/oversized original observation')
 value=json.loads(raw);require(type(value) is dict,'original observation is not an object');return value

def verify_retired_mutations(rows,trace,roles,source_identity):
 require(type(rows) is list and len(rows)==32,'exactly 32 original retained-actor mutation attempts required')
 require(len(roles)==2 and {r['account'] for r in roles}=={'PeerFixture','PeerFixture2'},'mutation role scope differs')
 owners={};sessions=set()
 for role in roles:
  for prefix in ['old','new']:
   oid,sid=role[prefix+'_owner_id'],role[prefix+'_session_id']
   require(positive_id(oid) and positive_id(sid) and oid not in owners and sid not in sessions,'original owner/session identities overlap')
   owners[oid]=dict(account=role['account'],session=sid);sessions.add(sid)
 trace_rows={signature(value) for value in trace};seen={};actor_ids=set();store_ids=set();checks=[]
 for index,row in enumerate(rows):
  require(type(row) is dict and set(row)==FIELDS and row['schema']==SCHEMA,'mutation receipt shape differs')
  require(row['method']==METHODS[index%8],'setter order or method differs')
  oid,sid,iid,aid,item=[row[key] for key in ['observer_owner_id','session_identity','inventory_observer_id','actor_observer_id','item_id']]
  require(all(positive_id(value) for value in [oid,sid,iid,aid,item]) and oid in owners and sid==owners[oid]['session'],'mutation identity differs')
  require(all(row[key] is True for key in ['positive_guard_at_capture','owner_retired_before_attempt','actor_attached_before_attempt',
   'original_session_futures_completed_before_probe','attempted_real_setter','argument_differs_from_original']),'mutation precondition absent')
  require(row['full_native_owner_qualified'] is False and row['full_attribute_matrix_qualified'] is False,'mutation receipt grants unsupported full scope')
  require(row['rejection_class']=='java.lang.IllegalStateException' and row['rejection']=='source item owner is stale',
   'validation, detached actor or another error is not owner-retirement denial')
  before=object_text(row['before_state']);require(row['before_state']==row['after_state'],'observed inventory/item changed during rejected setter')
  require(set(before)=={'schema','store_observer_id','entry_observer_id','actor_observer_id','entry_view_observer_id','actor_core','inventory'}
   and before['schema']=='G06_E149_retained_mutation_state_v1','retained state schema differs')
  require(all(positive_id(before[key]) for key in ['store_observer_id','entry_observer_id','entry_view_observer_id'])
   and before['actor_observer_id']==aid,'retained store/entry/actor identity differs')
  core=before['actor_core'];require(set(core)==set(CORE_FIELDS)|{'template_observer_id'}
   and positive_id(core['template_observer_id']),'retained actor core fields differ')
  require(all(type(core[key]) is bool for key in ['equipped','identified','sealed'])
   and all(type(core[key]) is int for key in ['count','enchant_level','durability','bless','charge_count']),'retained core value types differ')
  key=CORE_FIELDS[METHODS.index(row['method'])];original=core[key];argument=row['argument']
  require(type(argument) is type(original) and argument!=original,'same-value/no-op or wrong-typed argument')
  expected=not original if type(original) is bool else ((2 if original==1 else 1) if key=='count' else (1 if original==0 else 0))
  require(argument==expected,'argument differs from pinned original probe algorithm')
  state=before['inventory'];require(state['exists'] is True and state['stable_observed'] is True
   and state['backing']=='local.lito381.server.NativeDeItemInventory' and state['observer_inventory_id']==iid
   and state['native_owner_id']==oid and state['rental_character'] is None and state['rental_account'] is None,'retained inventory no longer has original unstaged owner')
  attributes=observed_attributes(state);require(item in attributes and attributes[item]['diagnostics']['observer_actor_id']==aid,'retained actor absent from current inventory')
  source=object_text(row['source_observation']);require(signature(source) in trace_rows,'positive capture absent from original server trace')
  owner=source['source_item_owner'];require(source['observer_kind']=='G06_E149_direct_fields_v1' and source['stable_observed'] is True
   and source['session_identity']==sid and source['test_account']==owners[oid]['account'] and source['source']=='測試乙'
   and owner['observer_owner_id']==oid and owner['active'] is True and owner['session_matches'] is True,'capture was not this original active owner/session')
  require(row['source_qualified_character']==owner['source']==source_identity(source['source'],source['character'])
   and row['source_qualified_account']==owner['account']==source_identity(source['source'],source['test_account']),'mutation source-qualified identity differs')
  inventories=[value for value in [source['native_inventory_state'],source['rental_inventory_state']]
   if value['exists'] and value.get('observer_inventory_id')==iid]
  require(len(inventories)==1 and inventories[0]['native_owner_id']==oid and inventories[0]['stable_observed'] is True,'capture inventory reference differs')
  original_attributes=observed_attributes(inventories[0]);require(item in original_attributes
   and original_attributes[item]['diagnostics']['observer_actor_id']==aid,'capture did not retain this real native actor')
  terminal=[value for value in trace if value['session_identity']==sid][-1]
  require(terminal['stable_observed'] is True and terminal['source_item_disconnected'] is True
   and (terminal['source_item_owner'] is None or terminal['source_item_owner']['active'] is False),'original source trace did not observe retirement')
  target=(sid,iid,aid,item,before['store_observer_id'],before['entry_observer_id'])
  if index%8==0:
   require(oid not in seen and aid not in actor_ids and before['store_observer_id'] not in store_ids,'duplicate owner/actor/store target')
   seen[oid]=target;actor_ids.add(aid);store_ids.add(before['store_observer_id']);group_owner=oid
  require(oid==group_owner and seen[oid]==target,'one eight-setter group switched owner or retained target')
  checks.append(dict(owner=oid,actor=aid,item_id=item,method=row['method'],changed_argument=True,exact_stale_owner_rejection=True,observed_state_unchanged=True))
 require(set(seen)==set(owners),'four original owner admissions not covered')
 return dict(scope='eight_bound_setter_entries_on_four_real_retired_admissions',actual_mutation_attempts=32,
  exact_stale_owner_rejections=32,observed_state_unchanged=True,checks=checks,
  full_native_owner_qualified=False,full_attribute_matrix_qualified=False,
  remaining=['active mutation and extended attribute matrix','isolated source/account branches','staged/detached actors and natural timer expiry'])
