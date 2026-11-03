"""Audit qualified draw-state resources, exact PE evidence and isolated TCP captures."""
import datetime,hashlib,importlib.util,json,pefile
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'驗證紀錄/current/native-actor-states'
def read(p):return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def pin(p):return {'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p)}
def main():
 exe=ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe';eh=sha(exe)
 ref=read('驗證紀錄/current/reference-snapshot.json');jh=ref['sha256'];assert jh==sha(Path(ref['source']))==sha(Path(ref['snapshot']))
 ev=read('驗證紀錄/current/native-actor-states/native-state-raw.json');assert sha(Path(ev['native_image']['path']))==ev['native_image']['sha256']
 pe=pefile.PE(ev['native_image']['path']);n=0
 for key,s in ev['slices'].items():
  if 'bytes' in s and 'va' in s:
   b=bytes.fromhex(s['bytes']);assert pe.get_data(int(s['va'],16)-pe.OPTIONAL_HEADER.ImageBase,len(b))==b,key;n+=1
 assert n>=10
 spec=importlib.util.spec_from_file_location('actor',ROOT/'維護工具/verify-native-actor-layers.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod)
 static=read('LiTo381-Modern/target/current/verification/native-actor-states-reference.json');assert static['pixel_comparisons']==static['owner_surface_comparisons']==len(static['cases'])==28
 for s in static['sources']:mod.verify_source(s)
 pngs=[]
 for c in static['cases']:
  p=Path(c['png']).resolve();p.relative_to(ROOT)
  with Image.open(p) as im:assert im.mode=='RGBA' and im.size==(320,240) and hashlib.sha256(im.tobytes()).hexdigest()==c['rgba_sha256']
  assert (c['owner_pixels']==0)==(c['style']=='Hidden');pngs.append(pin(p))
 tcp=read('驗證紀錄/current/native-actor-states/receipt.json');assert tcp['exe_sha256']==eh and tcp['jar_sha256']==jh and len(tcp['cases'])==1
 case=tcp['cases'][0];client=case['client'];c=client['combat'];before=c['before'];own=before['status']['object_id'];capture=Path(case['server']['capture_file']);records=[json.loads(s) for s in capture.read_text(encoding='utf-8').splitlines()]
 packets=[r for r in records if r['note'].startswith('isolated_actor_state_')];assert len(packets)==9
 actual={int(p['note'].rsplit('_',1)[1]):bytes.fromhex(p['plain']) for p in packets};assert set(actual)==set(range(1,10))
 states={str(own):[False,False,False,False],'2':[False,False,False,False]};replay={0:{k:list(v) for k,v in states.items()}}
 expected=[(50,own,1,0),(50,own,0,1),(52,own,1),(50,own,0,0),(50,2,1,0),(52,2,1),(52,own,0),(52,2,0),(50,2,0,0)]
 for step,p in sorted(actual.items()):
  wanted=expected[step-1];assert p==bytes([wanted[0]])+wanted[1].to_bytes(4,'little')+bytes(wanted[2:])
  key=str(int.from_bytes(p[1:5],'little'));assert key in states
  if p[0]==50:states[key][:2]=[bool(p[5]),bool(p[6])]
  else:states[key][2]=bool(p[5])
  replay[step]={k:list(v) for k,v in states.items()}
 frames=c['frames'];assert set(f['state_step'] for f in frames)==set(range(10));by_step={i:[] for i in range(10)};pins={s['member']:s for s in static['sources']};runtime_pngs=[]
 for f in frames:
  step=f['state_step'];by_step[step].append(f);w=f['world'];ns=f['elapsed_ns'];assert f['fragment_composition']['actor_states'] and not f['fragment_composition']['see_invisible'];assert not f['audio'];assert w['hp']==before['hp'] and w['mp']==before['mp'] and w['attack_packets']==w['own_cast_packets']==0
  for key,st in replay[step].items():
   a=w['actors'][key];actual_state=[a['draw_state'][k] for k in ['green','dim','invisible','suppressed']];assert actual_state==st
   old=before['actors'][key];assert (a['target_revision']>old['target_revision'])==(key=='2' and step>=6)
   assert {k:v for k,v in a.items() if k not in ['draw_state','target_revision']}=={k:v for k,v in old.items() if k not in ['draw_state','target_revision']}
   style='Hidden' if st[3] or (st[2] and int(key)!=own) else 'Half' if st[2] else {'Green':(abs(12-(ns//50_000_000)%24)+16)//2} if st[0] else 'Dim' if st[1] else 'Normal'
   row=next(a for a in f['actors'] if a['id']==int(key));assert row['draw_style']==style
   if style=='Hidden':assert row['hidden'] and not row['rendered']
   else:assert row['rendered'] and row.get('layer_error') is None
  for a in f['actors']:
   if a['rendered']:
    for s in [a['body_source']]+[l['source'] for l in a['layers']]:pins[s['member']]=s
  if 'png' in f:
   p=Path(f['png']);p.relative_to(ROOT)
   with Image.open(p) as im:assert im.size==(800,600) and hashlib.sha256(im.tobytes()).hexdigest()==f['rgba_sha256']
   runtime_pngs.append(pin(p))
 for step,rows in by_step.items():assert len(rows)>=2 and rows[-1]['elapsed_ns']-rows[0]['elapsed_ns']>=650_000_000
 assert len(runtime_pngs)==10
 for s in pins.values():mod.verify_source(s)
 result={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':eh,'jar_sha256':jh,'independent_pixel_comparisons':28,'per_pixel_owner_comparisons':28,'reference':static,'pngs':pngs,'source_pins':list(pins.values()),'native_evidence':pin(OUT/'native-state-raw.json'),'native_byte_slices_verified':n,'runtime_frames':len(frames),'runtime_state_steps':10,'runtime_packets':9,'runtime_pngs':runtime_pngs,'capture':pin(capture),'gui_tested':False,'audio_device_tested':False,'natural_spell_gameplay':False,'full_native_render_equivalence':False,'limits':['Ordinary view only; see-invisible and GM perception unqualified.','RGB555 source tables; RGB565 parity not claimed.','Modern elapsed poison phase and selectable half-body coverage are explicit policies.','State packets injected through actual deployed encrypted writer; no natural status-skill acceptance.','Shared resource decoder; independent scalar pixel/ownership composition.']}
 (OUT/'states-receipt.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({'frames':len(frames),'steps':10,'packets':9,'pixel_comparisons':28,'byte_slices':n}))
if __name__=='__main__':main()
