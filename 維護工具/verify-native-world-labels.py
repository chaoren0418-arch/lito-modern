"""Audit own/hover label receipts against captured authority and pinned native bytes.

GDI glyph shape is host supplied; these checks do not claim independent native
font reconstruction or an actual GUI pointer test.
"""
import datetime, hashlib, json, pefile, capstone
from pathlib import Path
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-world-labels'
def read(p): return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest().upper()
def pin(p): return {'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p)}
def png(p,digest,size):
 p=Path(p);p.relative_to(ROOT)
 with Image.open(p) as im:
  assert im.mode=='RGBA' and im.size==size and hashlib.sha256(im.tobytes()).hexdigest()==digest
 return pin(p)
def main():
 exe=ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe';eh=sha(exe)
 ref=read('驗證紀錄/current/reference-snapshot.json');jh=ref['sha256'];assert jh==sha(Path(ref['source']))==sha(Path(ref['snapshot']))
 ev=read('驗證紀錄/current/native-world-labels/native-label-raw.json');assert sha(Path(ev['native_image']['path']))==ev['native_image']['sha256']
 pe=pefile.PE(ev['native_image']['path']);cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
 for key,s in ev['slices'].items():
  b=bytes.fromhex(s['bytes']);va=int(s['va'],16)
  assert pe.get_data(va-pe.OPTIONAL_HEADER.ImageBase,len(b))==b,key
  assert [f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in cs.disasm(b,va)]==s['instructions'],key
 static=read('LiTo381-Modern/target/current/verification/native-world-text-reference.json');assert len(static['cases'])==4
 pngs=[png(c['png'],c['rgba_sha256'],(240,40)) for c in static['cases']]
 faces={c['face'] for c in static['cases']};assert all(faces)
 for name,n in [('own',1),('hover',2)]:
  stem='LiTo381-Modern/target/current/verification/native-world-labels-'+name
  f=read(stem+'.json');assert len(f['world_labels']['lines'])==n
  pngs.append(png(ROOT/(stem+'.png'),f['rgba_sha256'],(800,600)))
 tcp=read('驗證紀錄/current/native-actor-states/receipt.json');assert tcp['exe_sha256']==eh and tcp['jar_sha256']==jh
 frames=tcp['cases'][0]['client']['combat']['frames'];counts={};runtime=[];sources=None
 for f in frames:
  step=f['state_step'];w=f['world'];own=w['status']['object_id'];labels=f['world_labels'];rows=labels['lines']
  assert labels['ownership']=='click_through' and labels['font_height']==12 and labels['charset']==136
  assert not labels['all_names'] and not labels['full_native_equivalence']
  ids=[r['line']['id'] for r in rows];assert ids==([own,2] if step<6 or step==9 else [own]),(step,ids)
  assert labels['cache_entries']<=256 and labels['cache_bytes']<=4<<20
  if sources is None:sources=labels['sources']
  assert labels['sources']==sources
  for r in rows:
   assert 'error' not in r and r['face'] in faces
   line=r['line'];actor=w['actors'][str(line['id'])]
   assert line['text']==actor['name'] and line['color']==0x7c00 and actor['tail_byte']==0
   assert line['kind']==('own' if line['id']==own else 'hover')
   assert r['advance']>0 and r['size'][0]>0 and r['size'][1]>0
  counts[str(step)]=counts.get(str(step),0)+1
  if 'png' in f:runtime.append(png(f['png'],f['rgba_sha256'],(800,600)))
 assert set(counts)==set(map(str,range(10))) and min(counts.values())>=2 and len(runtime)==10
 assert [s['sha256'] for s in sources]==['F06C94F47271FCB01F1D5AF63725DFA9A230B5F6BE3C0DD1D53CD37628ADB7DF','73243064A856FADE75DE110C5D25F6B1F19ED51C3BA7B1219BBE44ED7E84006B']
 result={'schema':1,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exe_sha256':eh,'jar_sha256':jh,'native_byte_slices':len(ev['slices']),'native_evidence':pin(OUT/'native-label-raw.json'),'static_pngs':pngs,'runtime_pngs':runtime,'runtime_frames':len(frames),'steps':counts,'host_faces':sorted(faces),'description_sources':sources,'hover_retired_after_hide':True,'explicit_fresh_hover_step':9,'gui_tested':False,'audio_device_tested':False,'full_native_equivalence':False,'limits':['Host GDI font mapping, not a reconstructed bitmap font.','Synthetic probe hover, not GUI mouse acceptance.','800x600 viewport adaptation and enabled nonempty hover titles are modern policies.','Emblems, badges, privileged all-name mode and complete markup remain pending.']}
 (OUT/'labels-receipt.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 print(json.dumps({k:result[k] for k in ['runtime_frames','steps','host_faces','native_byte_slices']}))
if __name__=='__main__':main()
