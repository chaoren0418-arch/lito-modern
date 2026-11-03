"""Independent original IMG pixels, raw PE slices and actual TCP metadata audit."""
import argparse, datetime, hashlib, importlib.util, json, struct
from pathlib import Path
import pefile, capstone
from PIL import Image
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-label-updates'
def read(p):return json.loads((ROOT/p).read_text(encoding='utf-8-sig'))
def sha(b):return hashlib.sha256(b).hexdigest().upper()
def pin(p):return {'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())}
def image(p,digest):
 p=Path(p);p.relative_to(ROOT)
 with Image.open(p) as im:
  assert im.mode=='RGBA';rgba=im.tobytes();assert sha(rgba)==digest.upper();size=im.size
 return rgba,size,pin(p)
def originals():
 spec=importlib.util.spec_from_file_location('native_icon_reference',ROOT/'維護工具/verify-native-inventory-icons.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 expected=read('LiTo381-Modern/tests/fixtures/native-clan-markers-current.json')['cases'];images=[];sources=[]
 for row in expected:
  id=row['id'];name=f'{id}.img'
  for suffix in ['c','h','j','e','']:
   candidate=f'{id}{suffix}.img';assert m.member(f'Sprite{sum(candidate.encode())%16:02}',candidate) is None
   if suffix:assert m.member('LiTo/LTsurf',candidate) is None and m.member('Sprite',candidate) is None
  raw=m.member('LiTo/LTsurf',name);assert sha(raw)==row['sha256'];assert struct.unpack_from('<4H',raw)==(24,18,1,0);assert len(raw)==872
  rgba=bytearray()
  for (v,) in struct.iter_unpack('<H',raw[8:]):
   assert v<32768
   rgba.extend([0,0,0,0] if v==0 else [((v>>s)&31)*8+((v>>s)&31)//4 for s in [10,5,0]]+[255])
  assert sha(rgba)==row['rgba_sha256'];images.append(bytes(rgba));sources.append({'archive':'LiTo/LTsurf','member':name,'bytes':len(raw),'sha256':sha(raw),'rgba_sha256':sha(rgba)})
 return images,sources
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--static-only',action='store_true');args=parser.parse_args()
 ev=read('驗證紀錄/current/native-label-updates/native-update-raw.json');assert sha(Path(ev['native_image']['path']).read_bytes())==ev['native_image']['sha256']
 pe=pefile.PE(ev['native_image']['path']);cs=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
 for key,s in ev['slices'].items():
  b=bytes.fromhex(s['bytes']);va=int(s['va'],16);ins=list(cs.disasm(b,va));assert sum(i.size for i in ins)==len(b),key
  assert pe.get_data(va-pe.OPTIONAL_HEADER.ImageBase,len(b))==b,key
  assert [f'{i.address:08X}: {i.mnemonic} {i.op_str}' for i in ins]==s['instructions'],key
 s=ev['surface_archive_string'];b=bytes.fromhex(s['bytes']);assert b==b'LiTo\\LTSurf\0' and pe.get_data(int(s['va'],16)-pe.OPTIONAL_HEADER.ImageBase,len(b))==b
 originals_rgba,sources=originals();static=read('LiTo381-Modern/target/current/verification/native-clan-markers.json');assert static['sources']['members']==sources
 for key,ext in [('index','idx'),('pak','pak')]:
  p=Path(r'D:\lineage開發\0-客戶端')/f'LiTo/LTsurf.{ext}';assert static['sources'][key]=={'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())}
 pngs=[];assert len(static['cases'])==21
 for case in static['cases']:
  rgba,size,p=image(case['png'],case['rgba_sha256']);assert size==(64,48);expected=bytearray([24,48,72,255]*64*48);source=originals_rgba[case['phase']];x,y=case['anchor']
  for sy in range(18):
   for sx in range(24):
    start=(sy*24+sx)*4;dx,dy=x+sx,y+sy
    if 0<=dx<64 and 0<=dy<48 and source[start+3]:expected[(dy*64+dx)*4:(dy*64+dx)*4+4]=source[start:start+4]
  assert rgba==expected,(case['phase'],case['placement']);pngs.append(p)
 for name in ['native-label-update-title','native-clan-marker-world']:
  f=read(f'LiTo381-Modern/target/current/verification/{name}.json');_,size,p=image(ROOT/f'LiTo381-Modern/target/current/verification/{name}.png',f['rgba_sha256']);assert size==(800,600);pngs.append(p)
 result={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'independent_pixel_comparisons':21,'static_pngs':pngs,'native_byte_slices':len(ev['slices']),'sources':static['sources'],'gui_tested':False,'audio_device_tested':False,'full_native_equivalence':False,'natural_clan_gameplay':False}
 if not args.static_only:
  ref=read('驗證紀錄/current/reference-snapshot.json');exe=ROOT/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe';eh=sha(exe.read_bytes());jh=ref['sha256'];assert sha(Path(ref['source']).read_bytes())==sha(Path(ref['snapshot']).read_bytes())==jh
  oracle=read('LiTo381-Modern/tests/fixtures/native-label-updates-current.json');assert oracle['jar_sha256'].upper()==jh
  tcp=read('驗證紀錄/current/native-label-updates/receipt.json');assert tcp['exe_sha256']==eh and tcp['jar_sha256']==jh and len(tcp['cases'])==1
  case=tcp['cases'][0];frames=case['client']['combat']['frames'];counts={};phases=set();runtime=[]
  own=case['client']['combat']['before']['status']['object_id'];capture=Path(case['server']['capture_file']);records=[json.loads(s) for s in capture.read_text(encoding='utf-8').splitlines()]
  packets=[r for r in records if r['note'].startswith('isolated_label_update_')];assert len(packets)==10
  actual={int(p['note'].rsplit('_',1)[1]):bytes.fromhex(p['plain']) for p in packets};assert set(actual)==set(range(1,11))
  d=lambda n:struct.pack('<I',n)
  s=lambda t:t.encode('cp950')+b'\0'
  title_packet=lambda t:b'\x2f'+d(2)+s(t)
  pledge_packet=lambda clan,title,emblem:b'\x69'+d(2)+s(clan)+s(title)+d(emblem)
  clan_packet=lambda who,emblem,text,active:b'\xfa'+d(who)+d(emblem)+s(text)+d(active)
  wanted=[title_packet('稱號更新'),pledge_packet('測試盟','盟員稱號',0xffffffff),clan_packet(2,0x80000000,'新血盟',1),title_packet(''),clan_packet(2,0,'',0),clan_packet(own,7,'自身血盟',1),clan_packet(2,0,'12345678901234567890',1),pledge_packet('','',0),clan_packet(2,0,'',0),clan_packet(own,0,'',0)]
  assert [actual[i] for i in range(1,11)]==wanted
  setup=[r for r in records if r['note']=='isolated_label_setup'];assert len(setup)==1;setup=bytes.fromhex(setup[0]['plain']);assert setup[0]==0x0b and struct.unpack_from('<I',setup,5)[0]==2
  expected=[('','',0,False),('稱號更新','',0,False),('盟員稱號','測試盟',0xffffffff,False),('新血盟','測試盟',0x80000000,True),('','測試盟',0x80000000,True),('','測試盟',0,False),('','測試盟',0,False),('1234567890123456','測試盟',0,True),('','',0,True),('','',0,False),('','',0,False)]
  identity=None
  for f in frames:
   step=f['label_step'];counts[str(step)]=counts.get(str(step),0)+1;w=f['world'];own=w['status']['object_id'];bot=w['actors']['2'];a=w['actors'][str(own)];title,clan,emblem,active=expected[step]
   assert (bot['title'],bot['clan'],bot['emblem'],bot['clan_marker_active'])==(title,clan,emblem,active)
   assert a['clan_marker_active']==(6<=step<=9);assert w['hp']==[50,50] and w['attack_packets']==w['own_cast_packets']==w['local_corrections']==0
   ident=(bot['incarnation'],bot['target_revision'],bot['motion_revision'],bot['action_sequence'],bot['x'],bot['y'])
   if identity is None:identity=ident
   assert ident==identity and f['selection_retained']
   lines=[r for r in f['world_labels']['lines'] if r['line']['id']==2];assert len(lines)==(2 if title else 1) and all('error' not in r for r in lines)
   if title:assert lines[0]['line']['kind']=='title' and lines[0]['line']['text']==title
   markers=f['clan_markers'];assert markers['ownership']=='click_through' and len(markers['rows'])==int(active)
   if active:
    row=markers['rows'][0];mark=row['marker'];assert row['drawn'] and 'error' not in row and mark['id']==2
    body=next(r for r in f['actors'] if r['id']==2);assert mark['anchor']==[body['screen'][0]+13,body['screen'][1]-62];phases.add(mark['phase']);assert markers['sources']==static['sources']
   # Steps are contiguous, but time spent waiting for packets is unrecorded.
   # Clock boundaries are covered by the deterministic Rust vector test.
   if 'png' in f:
    rgba,size,p=image(f['png'],f['rgba_sha256']);assert size==(800,600);runtime.append(p)
    if active:
     x,y=mark['anchor'];src=originals_rgba[mark['phase']];covered=0
     for sy in range(18):
      for sx in range(24):
       i=(sy*24+sx)*4;dx,dy=x+sx,y+sy
       if src[i+3] and 0<=dx<800 and 0<=dy<600:assert rgba[(dy*800+dx)*4:(dy*800+dx)*4+4]==src[i:i+4];covered+=1
     assert covered>0
  assert set(counts)==set(map(str,range(11))) and min(counts.values())>=2;assert phases==set(range(7)),phases
  result.update(exe_sha256=eh,jar_sha256=jh,runtime_frames=len(frames),runtime_steps=counts,runtime_pngs=runtime,runtime_phases=sorted(phases),tcp_metadata_packets=10,tcp_setup_packets=1,selection_retained=True,capture=pin(capture))
 (OUT/('static-audit.json' if args.static_only else 'updates-receipt.json')).write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 print(json.dumps({k:v for k,v in result.items() if k not in ['static_pngs','runtime_pngs','sources']}))
if __name__=='__main__':main()
