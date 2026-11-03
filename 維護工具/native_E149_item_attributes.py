"""Pure readback for extended native item snapshots; no game/process APIs.

Only permanent state on a still-valid staged item is compared across adoption.
Timer/owner/cache diagnostics are observed separately and are not invariants.
Coverage of nonzero values alone never qualifies the full mutation matrix.
"""
PERMANENT_FIELDS={'elemental_kind','elemental_level','greater','inlay_cards',
 'magic_stone_item_id','magic_stone_level','magic_stone_time','awaken_power_ids'}
DIAGNOSTIC_FIELDS={'observer_actor_id','item_id','count_mutation_version','remaining_time','item_owner_id',
 'character_object_id','equip_window','temporarily_equipped','ability_dirty','now_lighting','match',
 'dark_item_present','magic_stone_generation','equipment_timer_generation'}

def require(ok,message):
 if not ok:raise RuntimeError(message)

def integer(value):return type(value) is int

def observed_attributes(inventory_state):
 rows=inventory_state['rows'];extended=inventory_state['native_attributes']
 require(len(rows)==len(extended),'extended inventory cardinality differs')
 item_ids={row['id'] for row in rows};result={}
 for row in extended:
  oid=row['id'];require(integer(oid) and oid in item_ids and oid not in result,'extended item identity differs')
  require(row['stable_observed'] is True,'native fields changed during observation')
  state=row['state'];require(state['present'] is True and state['schema']=='G06_E149_native_item_fields_v1','real native attribute state absent')
  permanent=state['permanent'];diagnostics=state['diagnostics'];abilities=state['cached_abilities']
  require(set(permanent)==PERMANENT_FIELDS and set(diagnostics)==DIAGNOSTIC_FIELDS,'native observation field set differs')
  for key in ['elemental_kind','elemental_level','greater','magic_stone_item_id','magic_stone_level']:
   require(integer(permanent[key]),'native attribute is not an integer: '+key)
  for key in ['inlay_cards','awaken_power_ids']:
   value=permanent[key];require(value is None or (type(value) is list and len(value)<=1024 and all(integer(n) for n in value)),'native integer collection differs')
  stamp=permanent['magic_stone_time']
  require(stamp is None or (type(stamp) is dict and set(stamp)=={'epoch_millis','nanos'} and integer(stamp['epoch_millis'])
   and integer(stamp['nanos']) and 0<=stamp['nanos']<1_000_000_000),'native timestamp observation differs')
  require(type(abilities) is dict and len(abilities)<=512
   and all(type(k) is str and k and (integer(v) or v is None) for k,v in abilities.items()),'cached ability observation differs')
  require(all(type(diagnostics[k]) is bool for k in ['temporarily_equipped','ability_dirty','now_lighting','match','dark_item_present']),
   'native diagnostic flag type differs')
  require(all(integer(v) for k,v in diagnostics.items() if k not in ['temporarily_equipped','ability_dirty','now_lighting','match','dark_item_present']),
   'native diagnostic counter type differs')
  result[oid]=state
 require(set(result)==item_ids,'extended inventory IDs missing');return result

def nondefault_coverage(states):
 states=list(states);values=[row['permanent'] for row in states]
 return dict(elemental=any(row['elemental_kind']!=0 or row['elemental_level']!=0 for row in values),
  greater=any(row['greater']!=0 for row in values),
  inlay_cards=any(any(n!=0 for n in (row['inlay_cards'] or [])) for row in values),
  magic_stone=any(row['magic_stone_item_id']!=0 or row['magic_stone_level']!=0 or row['magic_stone_time'] is not None for row in values),
  awakening=any(any(n!=0 for n in (row['awaken_power_ids'] or [])) for row in values),
  nonzero_cached_abilities=any(any(v is not None and v!=0 for v in row['cached_abilities'].values()) for row in states))

def compare_staged_adoption(before_inventory,after_inventory):
 before,after=observed_attributes(before_inventory),observed_attributes(after_inventory)
 require(before and set(before)==set(after),'staged native item identity changed')
 require(all(before[oid]['permanent']==after[oid]['permanent'] for oid in before),'permanent native attributes changed on staged adoption')
 return dict(scope='observed_permanent_fields_on_same_staged_item_IDs',objects=len(before),
  permanent_fields_equal=True,nondefault_features_observed=nondefault_coverage(before.values()),
  diagnostics_compared_as_invariants=False,ability_recomputation_requested=False,
  full_attribute_matrix_qualified=False,full_native_owner_qualified=False,
  remaining=['normal gameplay creation of nondefault states, expiry and mutation-denial cases',
   'validate cached effects against authoritative templates/expected modifiers without triggering getters',
   'dark-item payload, native timer leases and complete owner lifecycle'])
