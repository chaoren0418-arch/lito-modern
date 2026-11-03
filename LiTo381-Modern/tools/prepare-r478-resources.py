from pathlib import Path
import json,hashlib,re,struct
from PIL import Image
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r478-mode-menu'
O.mkdir(exist_ok=False);(O/'source').mkdir();(O/'reference').mkdir()
def pin(p):
 b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
old=json.loads((L/'r476-native-menu-skin-binding.json').read_text(encoding='utf-8'))
idx=Path(old['LTsurf_index']['path']).read_bytes();pak=Path(old['LTsurf_pak']['path']).read_bytes()
members=[]
for i,row in enumerate(struct.iter_unpack('<I20sI',idx[4:])):
 off,name,size=row
 if name.split(b'\0')[0]==b'6231.img':
  raw=pak[off:off+size];w,h,flag,key=struct.unpack('<hhHH',raw[:8]);assert (w,h)==(7,10) and size==148
  rgba=bytearray()
  for v, in struct.iter_unpack('<H',raw[8:]):
   assert v<32768;rgba.extend((0,0,0,0) if v==key else tuple(((c<<3)|(c>>2)) for c in ((v>>10)&31,(v>>5)&31,v&31))+(255,))
  path=O/'source/6231.img';path.write_bytes(raw);ref=O/'reference/6231.png';Image.frombytes('RGBA',(w,h),bytes(rgba)).save(ref)
  members.append(dict(id=6231,name='6231.img',index_row=i,offset=off,bytes=size,sha256=hashlib.sha256(raw).hexdigest().upper(),raw=pin(path),reference_png=pin(ref),rgba_sha256=hashlib.sha256(rgba).hexdigest().upper(),width=w,height=h,mask_flag=flag,mask_word=key,localized_candidates_present=[]))
assert len(members)==1
m={**old,'state':'R478_SIX_PINNED_NATIVE_PANEL_ARROWS_THUMB','members':old['members']+members,'previous_skin_binding':pin(L/'r476-native-menu-skin-binding.json')}
(L/'r478-native-menu-skin-binding.json').write_text(json.dumps(m,ensure_ascii=False,indent=2),encoding='utf-8')
asm=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm');s=asm.read_text(encoding='utf-8')
assert pin(asm)['sha256']=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
fragments=[]
for ad in ['46B7D0','450090','4645D0','450230','450BC0','450940','4509A0','4630F0','465110','4646C0','4744F0','474440','41B030','477AB0','4A8BA0','4A9E50','46D2E0','4D4450','4D43A0','fn_LoadStringTable_Check']:
 name=('sub_'+ad) if not ad.startswith('fn_') else ad
 start=re.search('^'+name+r' proc near',s,re.M)
 if start is None:continue
 end=re.search(r'^\w+ proc near',s[start.end():],re.M);text=s[start.start():start.end()+end.start()]
 path=O/'source'/f'asm-{ad}-through-next-proc.txt';path.write_text(text,encoding='utf-8');fragments.append(pin(path))
(L/'r478-source-fragments.json').write_text(json.dumps(dict(original_disassembly=pin(asm),fragments=fragments),ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(dict(thumb=members[0],source_fragments=len(fragments)),ensure_ascii=False))
