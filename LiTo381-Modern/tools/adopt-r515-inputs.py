from r491_common import *
import shutil
contract=Path(r'D:\lineage開發\3-lito開發\lito伺服器\server-room-participant-consumer-handoff.json')
assert pin(contract)['sha256']=='EF3FF32941FF07DD0276FCB885A67560ECC47CBE0B660AB778EC5F60CA46C65F'
d=read(contract);refs={}
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   a=pin(v['path']);assert(a['bytes'],a['sha256'])==(v['bytes'],v['sha256']),v['path'];refs[v['path']]=a
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
walk(d);root=L/'r515-inputs';root.mkdir();shutil.copyfile(contract,root/contract.name);payload=[]
for e in read(d['manifest']['path'])['files']:
 source=Path(d['evidence_root'])/e['path'];a=pin(source);assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256'])
 target=root/e['path'];target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,target);payload.append(dict(original=a,snapshot=pin(target)))
examples=read(root/'evidence/existing-S_CD-D1-wire.json')['examples'];vectors=[];frame_pins=[]
for index,e in enumerate(examples):
 verify=pin(e['original_frames']['path']);assert(verify['bytes'],verify['sha256'])==(e['original_frames']['bytes'],e['original_frames']['sha256']);frame_pins.append(verify)
 frames=read(e['original_frames']['path'])
 for row in e['rows']:
  b=bytes.fromhex(row['frame']['body_hex']);assert frames[row['global_frame_index']]['body_hex']==b.hex()
  at=1;fields=[]
  for field in row['decoded']['fields']:
   raw=bytes.fromhex(field['hex']);assert(field['offset'],field['bytes'])==(at,len(raw));assert b[at:at+len(raw)]==raw
   ty=field['type'];value=field['value']
   if ty in ('c','d'):assert int.from_bytes(raw,'little')==(value & (255 if ty=='c' else 0xffffffff))
   elif ty=='s':assert raw[-1]==0 and raw[:-1].decode('cp950')==value
   elif ty in ('b12','b5'):assert list(raw)==value
   else:raise AssertionError(ty)
   fields.append(dict(field=field['field'],offset=at,type=ty,raw_hex=raw.hex(),value=value));at+=len(raw)
  assert at==len(b)==row['decoded']['body_bytes'];assert row['frame']['frame_bytes']==len(b)+2
  vectors.append(dict(example=index,frame_index=row['global_frame_index']+1,epoch=row['frame']['epoch'],body_hex=b.hex(),opcode=b[0],fields=fields))
guest=read(examples[0]['original_frames']['path']);tail=next(dict(frame_index=i+1,frame=f) for i,f in enumerate(guest) if i>93 and f.get('epoch')=='Guest-GAME' and f.get('direction')=='S')
assert(tail['frame_index'],tail['frame']['body_hex'])==(95,'e103')
save(L/'r515-test-inputs.json',dict(vectors=vectors,Guest_next_after_saved_D1=tail,Host_retained_first_boundary=dict(frame_index=69,opcode='D0',room_id=1,not_generic_exit=True),fixture_or_network_calls=0))
save(L/'r515-input-adoption.json',dict(input=pin(contract),root_refs=list(refs.values()),root_ref_count=len(refs),payload=payload,payload_count=len(payload),saved_frame_sources=frame_pins,vectors=pin(L/'r515-test-inputs.json'),vector_count=len(vectors),CD_count=sum(v['opcode']==205 for v in vectors),D1_count=sum(v['opcode']==209 for v in vectors),all_offsets_raw_type_value_end_verified=True,consumer_handoff=read(root/'evidence/consumer-handoff.json'),new_fixture_calls=0,new_Java_SQL=0,old_PASS_replays=0))
assert len(refs)==16 and len(payload)==9 and len(vectors)==16
print('Adopted16 refs/9 payloads;8CD+8D1 complete offset/raw/type/value/end verified;Guest next saved frame95 E1/03, Host D0 retained')
