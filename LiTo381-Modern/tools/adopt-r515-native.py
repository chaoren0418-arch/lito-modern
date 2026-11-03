from r491_common import *
import shutil,re
contract=Path(r'D:\lineage開發\3-lito開發\lito脫殼資源\SCD-member-comparison-contract.json')
assert pin(contract)['sha256']=='1C2519F9F1B88C9E68FDF963FFAFF85366CEC755254613C1926A68E7ED9BC192'
d=read(contract);verified=[];excluded=[];raw={};cache={}
def walk(v,location='root'):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   q=v['path']
   if q not in cache:cache[q]=pin(q)
   a=cache[q]
   if(a['bytes'],a['sha256'])!=(v['bytes'],v['sha256']):
    assert 'historical_prebatch_aggregates' in location,(location,v,a)
    excluded.append(dict(location=location,historical=v,current=a,reason='Explicit mutable historical prebatch aggregate, not current core'))
   else:verified.append(dict(location=location,pin=a))
  if v.get('file_offset') and 'raw_hex' in v:raw[(v['file_offset'],v['raw_hex'])]=v
  for k,x in v.items():walk(x,location+'.'+k)
 elif isinstance(v,list):
  for i,x in enumerate(v):walk(x,location+f'[{i}]')
walk(d);pe=Path(d['sources'][0]['path']).read_bytes();witness=[]
for(offset,hexraw),v in raw.items():
 b=bytes.fromhex(hexraw);at=int(offset,16);assert pe[at:at+len(b)]==b;witness.append(dict(va=v.get('va'),offset=offset,raw_hex=hexraw,matches=True))
root=L/'r515-inputs/native';root.mkdir();shutil.copyfile(contract,root/contract.name);outputs=[];new_slices=[];inherited=[]
for e in d['reusable_outputs']:
 q=Path(e['path']);target=root/q.name;shutil.copyfile(q,target);outputs.append(pin(target))
 if q.suffix=='.bin':
  m=re.match(r'(?:VA-)?([0-9A-Fa-f]{8})',q.stem)
  if m:
   at=int(m.group(1),16)-0x400000;b=q.read_bytes();assert pe[at:at+len(b)]==b;new_slices.append(dict(source=e,va=m.group(1),matches=True))
for item in d['inherited_raw']:
 e=item['range'];b=Path(e['path']).read_bytes();at=int(item['va'],16)-0x400000;assert pe[at:at+len(b)]==b;inherited.append(dict(source=e,va=item['va'],matches=True))
save(L/'r515-native-evidence-adoption.json',dict(input=pin(contract),snapshot=pin(root/contract.name),verified=verified,matching_reference_occurrences=len(verified),explicit_historical_exclusions=excluded,root_raw_witnesses=witness,new_raw_slices=new_slices,inherited_raw_slices=inherited,reused_outputs=outputs,comparison=d['comparison'],identity=d['identity_and_incremental_ready'],direct_boundaries=d['direct_boundaries'],instruction_assertions_source_report_only=d['verification']['instruction_assertions'],original_operations=0,old_PASS_replays=0,
 Modern_policy='Epoch+room+kind+raw alias. Same-kind/raw-alias duplicate CD preserves existing row and newer D1 state; every packet retained separately within a bounded receipt budget. Cross-kind rows remain distinct; unknown/ambiguous D1 is ignored, never merged or buffered. Runtime Native lead/fold table authority remains unqualified.',
 Native_guard_retained=True,table_authority='Unknown private initialization/external return, current epoch, locale/generation. Disk zeros, ASCII equality, locale guesses cannot qualify Native comparison tables.'))
print('Native adoption',len(verified),'matching refs;',len(excluded),'historical exclusions;',len(witness),'root raw;',len(new_slices),'new raw;',len(inherited),'inherited raw;81source assertions not replayed')
