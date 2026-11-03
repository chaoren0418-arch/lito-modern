from r491_common import *
import shutil
contract=Path(r'D:\lineage開發\3-lito開發\lito脫殼資源\SCE-navigation-contract.json')
assert pin(contract)['sha256']=='2256D57DC068C669E41BFFACB9F39D0E26E83C652D8B987F7524B3AAA2B178B4'
d=read(contract);verified=[];excluded=[];cache={};witnesses={}
baseline=next(x for x in read(L/'r513-production-source-snapshot.json')['source_pins'] if x['path'].endswith('native_city_runtime.rs'))
def walk(v,location='root'):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   path=v['path'];a=cache.setdefault(path,pin(path))
   if(a['bytes'],a['sha256'])!=(v['bytes'],v['sha256']):
    if path.endswith('native_city_runtime.rs'):
     frozen=pin(baseline['snapshot']);assert(frozen['bytes'],frozen['sha256'])==(v['bytes'],v['sha256'])
     excluded.append(dict(location=location,historical=v,current=a,accepted_R513_frozen=frozen,reason='Historical comparison; live Modern source now implements R514; fixed R513 guard remains available'))
    elif 'historical_prebatch_aggregates' in location:
     excluded.append(dict(location=location,historical=v,current=a,reason='Explicit historical pre-batch mutable aggregate, excluded from current core acceptance'))
    else: raise AssertionError((location,v,a))
   else:verified.append(dict(location=location,pin=a))
  if 'raw_hex' in v and v.get('file_offset'):
   witnesses[(v['file_offset'],v['raw_hex'])]=v
  for k,x in v.items():walk(x,location+'.'+k)
 elif isinstance(v,list):
  for i,x in enumerate(v):walk(x,location+f'[{i}]')
walk(d)
root=L/'r514-inputs/native';root.mkdir()
shutil.copyfile(contract,root/contract.name)
outputs=[]
for e in d['reusable_outputs']:
 source=Path(e['path']);target=root/source.name;shutil.copyfile(source,target);outputs.append(pin(target))
pe=Path(d['sources'][0]['path']).read_bytes();raw_checks=[]
for (offset,raw_hex),v in witnesses.items():
 raw=bytes.fromhex(raw_hex);at=int(offset,16);assert pe[at:at+len(raw)]==raw
 raw_checks.append(dict(va=v.get('va'),file_offset=offset,raw_hex=raw_hex,matches=True))
slices=[]
for e in d['reusable_outputs']:
 q=Path(e['path'])
 if q.suffix=='.bin' and '-' in q.stem:
  address=q.stem.split('-')[0]
  if len(address)==8:
   at=int(address,16)-0x400000;raw=q.read_bytes();assert pe[at:at+len(raw)]==raw;slices.append(dict(source=e,va=address,matches=True))
assert 'navigation mode side effects unqualified; retire source epoch' in Path(baseline['snapshot']).read_text('utf-8')
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r514-native-evidence-adoption.json',dict(input=pin(contract),snapshot=pin(root/contract.name),
 reference_occurrences_verified=len(verified),verified=verified,explicit_historical_exclusions=excluded,
 fixed_PE_raw_witnesses=raw_checks,new_raw_ranges=slices,reused_outputs=outputs,
 Native_dispatch=d['dispatch']['eligible_condition'],Native_scene=d['scene'],
 historical_Java_not_AD368_authority=True,AD368_authority=pin(L/'r514-inputs/server-channel-room-entry-consumer-handoff.json'),
 original_and_Java_runs=0,old_PASS_replays=0,Native_navigation_guard_retained=True,
 policy='Modern owned room receipt scenes are separate; Native numeric phase, mode2 private/virtual/UI/callback graph remains unqualified'))
print('Native adoption',len(verified),'matching reference occurrences;',len(excluded),'historical exclusions;',len(raw_checks),'root raw witnesses;',len(slices),'new raw ranges match fixed PE')
