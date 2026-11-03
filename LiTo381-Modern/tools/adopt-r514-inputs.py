from r491_common import *
import shutil

contract=Path(r'D:\lineage開發\3-lito開發\lito伺服器\server-channel-room-entry-consumer-handoff.json')
assert pin(contract)['sha256']=='11D9483483A6863BF680927748BB7592DE5ADCB7D8AA73C77E5AD3863F04D5C6'
d=read(contract); refs={}
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ['path','bytes','sha256']):
   a=pin(v['path']);assert(a['bytes'],a['sha256'])==(v['bytes'],v['sha256']),v['path'];refs[v['path']]=a
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
walk(d)
root=P/'logs/continuation-20261003/r514-inputs';root.mkdir()
shutil.copyfile(contract,root/contract.name)
payload=[]
for e in read(d['manifest']['path'])['files']:
 source=Path(d['evidence_root'])/e['path'];a=pin(source);assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256'])
 target=root/e['path'];target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,target)
 payload.append(dict(original=a,snapshot=pin(target)))
examples=read(root/'evidence/existing-wire-examples.json')['examples']
extra={}
for e in examples:
 for k in ['original_frames','original_observations']:
  a=pin(e[k]['path']);assert(a['bytes'],a['sha256'])==(e[k]['bytes'],e[k]['sha256']);extra[a['path']]=a
golden=[]
for e in examples:
 for f in e['frames']:
  b=bytes.fromhex(f['body_hex'])
  if b[:2]!=b'\xce\x00':continue
  at=2
  def take(n):
   global at
   v=b[at:at+n];assert len(v)==n;at+=n;return v
  def z():
   global at
   end=b.index(0,at);v=b[at:end];at=end+1;return v.hex()
  room_id=int.from_bytes(take(4),'little');name=z();attributes=take(1)[0];password=z()
  users=int.from_bytes(take(2),'little');capacity=int.from_bytes(take(2),'little');mode=take(1)[0];map_id=int.from_bytes(take(2),'little')
  summary=list(take(5));owner=z();options=list(take(4));count=take(1)[0];selections=list(take(count));assert at==len(b)
  golden.append(dict(body_hex=b.hex(),room_id=room_id,name_hex=name,attributes=attributes,password_hex=password,users=users,capacity=capacity,mode=mode,map_id=map_id,summary=summary,owner_hex=owner,options=options,selections=selections,epoch=f['epoch']))
assert len(golden)==4
frames=read(examples[0]['original_frames']['path'])
print('frames type',type(frames).__name__)
if isinstance(frames,dict): print('frame keys',list(frames))
prior=read(L/'r513-test-inputs.json')
inputs=dict(setup_frames=prior['setup_frames'],golden_CE=golden,wire_examples=examples,
 known_prerequisite_input=pin(L/'r513-test-inputs.json'),
 new_Java_SQL=0,old_PASS_reruns=0,Native_GUI=0)
save(L/'r514-test-inputs.json',inputs)
save(L/'r514-input-adoption.json',dict(input=pin(contract),root_references=list(refs.values()),
 root_reference_count=len(refs),manifest_payload=payload,payload_count=len(payload),
 saved_frame_sources=list(extra.values()),four_CE_independent_fields=golden,
 adapter=read(root/'evidence/adapter-handoff.json'),new_runtime_calls=0,old_fixture_replays=0,
 Native_numeric_scene_guard_retained=True,result_exit_constraints=d['adopted_accepted'][2]))
print('adopted',len(refs),'root refs',len(payload),'payloads;4 saved CE field records')
