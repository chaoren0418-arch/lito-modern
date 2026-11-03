"""Future pure receipt reader for eight original BoundItem morph guard calls.

Starts no processes and invokes no game code. The outer reader must bind the
exact compiled fixture, original Futures, trace, process cleanup and roles.
The existing 32-setter verifier remains unchanged and must qualify separately.
"""
import json
from native_E149_item_attributes import observed_attributes
from native_E149_mutation_denials import verify_retired_mutations,positive_id,signature,object_text,require
SCHEMA='G06_E149_retired_bound_morph_v1'
METHODS=['setItem','setItemId']
FIELDS={'schema','method','observer_owner_id','session_identity','inventory_observer_id','actor_observer_id','item_id',
 'original_item_id','original_template_observer_id','argument_kind','argument_template_observer_id','argument_item_id',
 'alternate_item_id','alternate_actor_observer_id','alternate_template_observer_id','source_qualified_character','source_qualified_account',
 'positive_guard_at_capture','owner_retired_before_attempt','both_actors_attached_before_attempt',
 'original_session_futures_completed_before_probe','attempted_real_setter','argument_differs_from_original',
 'rejection_class','rejection','before_state','after_state','source_observation','full_native_owner_qualified','full_attribute_matrix_qualified'}
STATE_FIELDS={'schema','retained_state','actor_native_item_id','template_native_item_id','alternate_actor_observer_id',
 'alternate_native_item_id','alternate_template_observer_id','alternate_template_native_item_id',
 'alternate_entry_observer_id','alternate_view_observer_id'}

def verify_retired_morphs(rows,core_rows,trace,roles,source_identity):
 # This is deliberately not called during source preparation/AST compilation.
 core_result=verify_retired_mutations(core_rows,trace,roles,source_identity)
 require(type(rows) is list and len(rows)==8,'eight original morph attempts required')
 owners={}
 for row in core_rows:owners.setdefault(row['observer_owner_id'],[]).append(row)
 require(len(owners)==4,'four independently qualified core targets required')
 trace_rows={signature(x) for x in trace};seen=set();checks=[];group=None
 for index,row in enumerate(rows):
  require(type(row) is dict and set(row)==FIELDS and row['schema']==SCHEMA,'morph receipt shape differs')
  require(row['method']==METHODS[index%2],'morph method/order differs')
  oid=row['observer_owner_id'];require(positive_id(oid) and oid in owners,'morph owner not one of the original core targets')
  core=owners[oid][-1]
  for key in ['session_identity','inventory_observer_id','actor_observer_id','item_id','source_qualified_character','source_qualified_account','source_observation']:
   require(row[key]==core[key],'morph switched original target/capture: '+key)
  require(all(row[k] is True for k in ['positive_guard_at_capture','owner_retired_before_attempt','both_actors_attached_before_attempt',
   'original_session_futures_completed_before_probe','attempted_real_setter','argument_differs_from_original']),'morph precondition absent')
  require(row['full_native_owner_qualified'] is False and row['full_attribute_matrix_qualified'] is False,'unsupported full morph qualification')
  require(row['rejection_class']=='java.lang.IllegalStateException' and row['rejection']=='source item owner is stale',
   'accepted call, validation error or detached/staged denial is not owner retirement')
  before=object_text(row['before_state']);require(row['before_state']==row['after_state'],'morph changed observed item/inventory state')
  require(set(before)==STATE_FIELDS and before['schema']=='G06_E149_retained_morph_state_v1','morph state schema differs')
  state=object_text(before['retained_state']);require(before['retained_state']==core['after_state'],'morph did not continue original unchanged core state')
  current,alternative=row['original_item_id'],row['argument_item_id']
  require(positive_id(current) and positive_id(alternative) and current!=alternative,'same/invalid native item ID argument')
  original_template,alternate_template=row['original_template_observer_id'],row['alternate_template_observer_id']
  require(positive_id(original_template) and positive_id(alternate_template) and original_template!=alternate_template,'same/invalid actual template identity')
  require(state['actor_core']['template_observer_id']==original_template and before['actor_native_item_id']==before['template_native_item_id']==current,
   'original actor/template identity differs')
  require(before['alternate_native_item_id']==before['alternate_template_native_item_id']==alternative
   and before['alternate_template_observer_id']==alternate_template,'alternative template ID differs')
  require(all(positive_id(before[k]) for k in ['alternate_entry_observer_id','alternate_view_observer_id'])
   and before['alternate_entry_observer_id']!=state['entry_observer_id'] and before['alternate_view_observer_id']!=state['entry_view_observer_id'],
   'alternative reuses original entry/view')
  alternate_actor=row['alternate_actor_observer_id'];alternate_item=row['alternate_item_id']
  require(positive_id(alternate_actor) and positive_id(alternate_item) and alternate_actor!=row['actor_observer_id'] and alternate_item!=row['item_id']
   and before['alternate_actor_observer_id']==alternate_actor,'alternative reuses original actor/item')
  if row['method']=='setItem':
   require(row['argument_kind']=='retained_actual_template' and row['argument_template_observer_id']==alternate_template,'template argument is not the retained actual alternative')
  else:require(row['argument_kind']=='retained_actual_template_item_id' and row['argument_template_observer_id'] is None,'ID argument is not the retained actual alternative')
  inventory=state['inventory'];attributes=observed_attributes(inventory);records={x['id']:x for x in inventory['rows']}
  for item,actor,native_id in [(row['item_id'],row['actor_observer_id'],current),(alternate_item,alternate_actor,alternative)]:
   require(item in attributes and item in records and records[item]['catalog_id']==native_id
    and attributes[item]['diagnostics']['observer_actor_id']==actor and attributes[item]['diagnostics']['item_id']==native_id,
    'original/alternative actor absent from retained inventory or template mismatched')
  capture=object_text(row['source_observation']);require(signature(capture) in trace_rows,'morph positive capture absent from original trace')
  captures=[value for value in [capture['native_inventory_state'],capture['rental_inventory_state']]
   if value['exists'] and value.get('observer_inventory_id')==row['inventory_observer_id']]
  require(len(captures)==1,'original morph inventory capture ambiguous/missing')
  original_attributes=observed_attributes(captures[0]);original_records={x['id']:x for x in captures[0]['rows']}
  for item,actor,native_id in [(row['item_id'],row['actor_observer_id'],current),(alternate_item,alternate_actor,alternative)]:
   require(item in original_attributes and original_records[item]['catalog_id']==native_id
    and original_attributes[item]['diagnostics']['observer_actor_id']==actor and original_attributes[item]['diagnostics']['item_id']==native_id,
    'alternative was not an original positively observed attached actor')
  target=(oid,row['session_identity'],row['inventory_observer_id'],row['actor_observer_id'],row['item_id'],current,original_template,alternate_item,alternate_actor,alternate_template,alternative,row['before_state'])
  if index%2==0:
   require(oid not in seen,'duplicate morph owner group');seen.add(oid);group=target
  require(target==group,'one morph pair switched original or alternative identity/state')
  checks.append(dict(owner=oid,actor=row['actor_observer_id'],item_id=row['item_id'],method=row['method'],
   actual_other_template=True,changed_argument=True,exact_stale_owner_rejection=True,observed_state_unchanged=True))
 require(seen==set(owners),'four original admissions not covered by morph denials')
 return dict(scope='two_guarded_morph_entries_on_four_real_retired_admissions',core_mutations=core_result,
  actual_morph_attempts=8,aggregate_retired_setter_attempts=40,exact_stale_owner_rejections=8,checks=checks,
  full_native_owner_qualified=False,full_attribute_matrix_qualified=False,
  remaining=['canonical outer binding and original processes/trace/Futures','successful normal GAME nondefault attribute creation',
   'isolated source/account branch matrix','active leases, staged/detached actors and natural expiry'])
