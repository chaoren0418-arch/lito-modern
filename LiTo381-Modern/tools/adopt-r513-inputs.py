from r491_common import *
import shutil
B=Path(r'D:/lineage開發/3-lito開發/lito脫殼資源');scope=B/'status-family-50-51-52-56';p=B/'status-family-50-51-52-56-contract.json'
assert pin(p)['sha256']=='0DEEF1249E3F98A780994680575AF5EE0003DDEEDF6C9A95A5EC109D26184DCE'
pe=Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe').read_bytes();pins={};seen=set();raw=set();ranges={};excluded=[]
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ['path','bytes','sha256']):
   q=Path(v['path']);assert q.is_absolute();key=(str(q),v['sha256'].upper())
   if key not in pins:
    e=pin(q);assert(e['bytes'],e['sha256'])==(v['bytes'],key[1]),(v,e);pins[key]=e
   if q.suffix=='.json' and scope in q.parents and key not in seen:seen.add(key);walk(read(q))
   if q.parent==scope/'ranges':
    start=int(q.name.split('-')[0],16)-0x400000;b=q.read_bytes();assert pe[start:start+len(b)]==b;ranges[str(q)]=pins[key]
  if 'va' in v and 'raw_hex' in v and v.get('raw_backed',True):
   b=bytes.fromhex(v['raw_hex']);start=int(v['va'],16)-0x400000;assert pe[start:start+len(b)]==b;raw.add((v['va'],v['raw_hex']))
  for k,x in v.items():
   if k in ['previous_delivery','previous_manifest','historical_pre_batch_seal_metadata']:excluded.append(dict(key=k,value=x));continue
   walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
walk(read(p));pins[(str(p),pin(p)['sha256'])]=pin(p)
dest=L/'r513-inputs';dest.mkdir();shutil.copyfile(p,dest/p.name)
comparison=read(scope/'AD368-sealed-frame-comparison.json');capture=[json.loads(x) for x in Path(comparison['capture']['path']).read_text('utf-8-sig').splitlines()];vectors=[]
for f in comparison['frames']:
 b=Path(f['body']['path']).read_bytes();assert b==bytes.fromhex(capture[f['capture_line']-1]['plain']);assert capture[f['capture_line']-1]['wire_length']==len(b)+2
 c=read(scope/f'S{b[0]:02X}-fields.json');vectors.append(dict(capture_line=f['capture_line'],body_hex=b.hex(),pin=f['body'],raw=[int.from_bytes(b[x['body_offset']:x['body_offset']+x['width']],'little') for x in c['fields']],java=[int.from_bytes(b[x['body_offset']:x['body_offset']+x['width']],'little',signed=x['server_semantic_type'].startswith('i')) for x in c['fields']]))
assert len(vectors)==8
different=[]
for b in [bytes([80,1,254,3,252,5,250,29]),bytes([81,255])+bytes.fromhex('efcdabf1'),bytes([82,128,255,126,129,254]),bytes([86,128,127]),bytes([86,255,129])]:
 c=read(scope/f'S{b[0]:02X}-fields.json');different.append(dict(body_hex=b.hex(),raw=[int.from_bytes(b[x['body_offset']:x['body_offset']+x['width']],'little') for x in c['fields']],java=[int.from_bytes(b[x['body_offset']:x['body_offset']+x['width']],'little',signed=x['server_semantic_type'].startswith('i')) for x in c['fields']]))
old=read(L/'r512-test-inputs.json')
battle=read(L/'r512-inputs/server-battle-skill-network.json');frames=read(battle['new_network']['frames']['path']);tail=[dict(index=i+1,frame=f) for i,f in enumerate(frames) if f.get('epoch')=='Host-GAME' and f.get('direction')=='S' and f.get('body_hex','').startswith(('ce','d0'))]
assert [t['index'] for t in tail]==[66,69]
save(L/'r513-test-inputs.json',dict(eight_sealed=vectors,different=different,setup_frames=old['frames'],setup_scope='Known handshake/selection setup only; no old R512 assertions/scenarios, no JavaSQL',full_status_body=next(f['body_hex'] for f in old['frames'] if f.get('epoch')=='GAME-initial' and f.get('direction')=='S' and f.get('body_hex','').startswith('0c')),actual_next_room_frames=tail,battle_frames_pin=pin(battle['new_network']['frames']['path']),oracle='Independent Python field contract raw/Java domains; no Rust-derived values'))
save(L/'r513-input-adoption.json',dict(contract=pin(p),payloads=list(pins.values()),verified_complete_ranges=list(ranges.values()),unique_PE_raw_claims=len(raw),sealed_actual_frames=8,historical_summary_exclusions=excluded,new_wire_or_opcode=0,new_Java_SQL=0,GUI=0))
print('verified',len(pins),'pins',len(ranges),'complete raw slices',len(raw),'unique raw witnesses,8 sealed frames')
# Cross-group return/skill evidence is stored only, no new execution.
q=Path(r'D:/lineage開發/3-lito開發/lito伺服器/server-battle-return-skill-boundary.json');assert pin(q)['sha256']=='49A29B0678F40195DBD36D548F75B3A5CEF624E903462F6067B3D4ABAFC148F2';d=read(q);root=Path(d['evidence_root']);extra={}
def check(v):
 if isinstance(v,dict):
  if all(k in v for k in ['path','bytes','sha256']):
   a=Path(v['path']);a=a if a.is_absolute()else root/a;e=pin(a);assert(e['bytes'],e['sha256'])==(v['bytes'],v['sha256'].upper());extra[str(a)]=e
  for x in v.values():check(x)
 elif isinstance(v,list):
  for x in v:check(x)
check(d)
for f in read(d['manifest']['path'])['files']:
 a=(root/f['path']).resolve();assert root.resolve() in a.parents;check(dict(f,path=str(a)))
shutil.copyfile(q,dest/q.name);save(L/'r513-return-skill-evidence-adoption.json',dict(contract=pin(q),payloads=list(extra.values()),adopted_only=True,new_execution=0,interpretation='D9/CB/CA100/C8 preserves Java physical/source/skills without S1E; different physical C9 gives S_C8 -> S1F old bitmap -> S7C, new skills{1}. No Native owner/UI readiness and no inferred first-lobby hook.'))
print('stored return skill evidence',len(extra),'pins; no new execution')
