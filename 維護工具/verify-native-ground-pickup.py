"""Audit actual ground pickup authority, action15 frames and immutable source pins."""
import datetime,hashlib,json,struct
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'驗證紀錄/current/native-ground-pickup';SOURCE=Path(r'D:\lineage開發\0-客戶端')
def read(p):return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def png(p,h):
 p=Path(p).resolve();p.relative_to(ROOT)
 with Image.open(p) as im:
  assert im.mode=='RGBA' and im.size==(800,600) and hashlib.sha256(im.tobytes()).hexdigest().upper()==h.upper()
 return {'path':p.relative_to(ROOT).as_posix(),'sha256':sha(p)}
def sprites(names):
 b=(SOURCE/'Sprite.idx').read_bytes();assert len(b)==4+struct.unpack_from('<I',b)[0]*28
 rows={}
 for at in range(4,len(b),28):
  name=b[at+4:at+24].split(b'\0',1)[0].decode('ascii').lower()
  if name in names:
   assert name not in rows;offset,size=struct.unpack_from('<I',b,at)[0],struct.unpack_from('<I',b,at+24)[0];assert 0<size<=8<<20
   with (SOURCE/'Sprite.pak').open('rb') as f:f.seek(offset);data=f.read(size)
   assert len(data)==size;rows[name]={'name':name,'bytes':size,'sha256':hashlib.sha256(data).hexdigest().upper()}
 assert set(rows)==names;return list(rows.values())
def replay_inventory(path):
 path=Path(path).resolve();path.relative_to(ROOT);items={};packets=[]
 for line in path.read_text(encoding='utf-8-sig').splitlines():
  r=json.loads(line)
  if r['direction']!='server_to_client' or r['opcode'] not in ['0x7C','0x16','0x17','0x18','0x19','0x1A','0x0E','0x6F']:continue
  raw=bytes.fromhex(r['plain']);at=1
  def take(n):
   nonlocal at
   assert at+n<=len(raw);b=raw[at:at+n];at+=n;return b
  def c():return take(1)[0]
  def h():return int.from_bytes(take(2),'little')
  def d():return int.from_bytes(take(4),'little')
  def text():
   nonlocal at
   end=raw.index(0,at);assert end-at<=255
   return take(end-at+1)[:-1].decode('cp950')
  def detail():return list(take(c()))
  def item():return {'id':d(),'use_type':c(),'charge':c(),'icon':h(),'blessing':c(),'count':d(),'identified':c(),'name':text(),'detail':detail()}
  op=raw[0]
  if op in [0x16,0x7c]:
   n=c() if op==0x7c else 1
   for _ in range(n):
    i=item();items[str(i['id'])]=i
  else:
   id=str(d());assert id in items
   if op==0x17:del items[id]
   elif op==0x18:items[id]['name']=text()
   elif op==0x19:items[id]['charge']=c()
   elif op==0x1a:items[id]['count']=d()
   elif op==0x0e:items[id]['blessing']=c() & 127
   elif op==0x6f:items[id].update(name=text(),count=d(),detail=detail())
  assert at==len(raw);packets.append({'opcode':r['opcode'],'plain':r['plain']})
 return items,{'path':path.relative_to(ROOT).as_posix(),'sha256':sha(path),'inventory_packets':packets}
def main():
 r=read('驗證紀錄/current/native-ground-pickup/receipt.json');ref=read('驗證紀錄/current/reference-snapshot.json')
 assert r['exe_sha256']==sha(ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe')
 assert r['jar_sha256']==ref['sha256']==sha(Path(ref['snapshot']))==sha(Path(ref['source']))
 assert len(r['cases'])==2;cases=[];members=set()
 for case in r['cases']:
  c,s=case['client'],case['server'];probe=c['combat'];stale=case['mode'].endswith('stale');rows=probe['pickup_actions'];w=probe['result']['world'];own=str(s['world_object']);ground=str(rows[0]['choice']['item']['id'])
  assert case['verified'] and c['worker_quiescent'] and c['inventory_shutdown_cleared'] and s['actual_client_sessions'] and not s['sql_access'] and not s['state_persistent']
  replay,capture=replay_inventory(s['capture_file']);assert replay==probe['result']['inventory']['items']
  regenerated={str(i['id']):i for i in s['inventory']}
  assert set(replay)==set(regenerated)
  for id,item in replay.items():assert {k:v for k,v in item.items() if k!='detail'}=={k:v for k,v in regenerated[id].items() if k!='detail'}
  assert w['hp']==[50,50] and w['own_attack_packets']==w['own_cast_packets']==0 and len(rows)==2
  if stale:
   assert s['forced_pickup_removal']==int(ground) and w['actors'][ground]['experience']==4
   assert all(x['feedback']=='NoResponse' and x['before']==x['after'] for x in rows)
  else:
   assert s['forced_pickup_removal']==0 and ground not in w['actors']
   assert rows[0]['after']['ground']['experience']==2 and rows[1]['after']['ground'] is None
   assert rows[0]['choice']['item']['incarnation']!=rows[1]['choice']['item']['incarnation']
   assert all(x['feedback']=='ObservedUpdates' for x in rows)
  events=set();action_sequences=set();pngs=[];drawn=0
  for f in probe['frames']:
   if f.get('png'):pngs.append(png(f['png'],f['rgba_sha256']))
   assert f['audio_events_dropped']==0
   for a in f['actors']:
    if a['id']==int(own) and a.get('action')==15:
     assert a['rendered'];drawn+=1;members.add(a['sprite_member']);action_sequences.add(f['world']['actors'][own]['action_sequence'])
     for e in a['entered_events']:
      k=json.dumps(e,sort_keys=True);assert k not in events;events.add(k)
    if a['id']==int(ground):assert a['rendered'] and a['phase']=='ground_item';members.add(a['sprite_member'])
  assert len(action_sequences)==(0 if stale else 2)
  last=next(a for a in probe['frames'][-1]['actors'] if a['id']==int(own));assert last['phase']=='idle'
  cases.append({'mode':case['mode'],'frames':len(probe['frames']),'pngs':pngs,'action15_samples':drawn,'action15_sequences':len(action_sequences),'events_once':len(events),'stale_fixture_forced_server_removal':s['forced_pickup_removal'],'capture':capture,'regenerated_detail_differences':[{'id':int(id),'last_received':item['detail'],'regenerated':regenerated[id]['detail']} for id,item in replay.items() if item['detail']!=regenerated[id]['detail']]})
 native=read('驗證紀錄/current/native-reagent-skills/pickup-native-recon.json');assert sha(Path(native['native_image']['path']))==native['native_image']['sha256']
 result={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],'cases':cases,'frames':sum(c['frames'] for c in cases),'pngs':sum(len(c['pngs']) for c in cases),'sources':sprites(members),'new_independent_pixel_comparisons':0,'native_auxiliary_equivalent':False,'auxiliary':'explicit zero; ignored by deployed battle acceptance; legacy membership not implemented','gui_tested':False,'audio_device_tested':False,'complete_ground_system':False}
 (OUT/'pickup-visual-receipt.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:result[k] for k in ['frames','pngs','sources']}))
if __name__=='__main__':main()
