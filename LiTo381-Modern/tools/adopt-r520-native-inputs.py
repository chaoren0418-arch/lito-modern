from r491_common import *
import shutil
base=P.parent.parent; root=base/'lito脫殼資源/D5-settings-receive-contract.json'; dest=L/'r520-native-inputs';dest.mkdir(exist_ok=True)
assert pin(root)['sha256']=='8A6C74A34CFAC1BC1AFF04215F80E347853069ACFA611C076269443B4671DFD8'
d=read(root);refs={};occ=[];copies=[]
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   p=Path(v['path']);a=refs.setdefault(str(p),pin(p));assert (a['bytes'],a['sha256'])==(v['bytes'],v['sha256']),(v,a);occ.append(a)
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
walk(d)
def copy(p,name):
 q=dest/name;q.parent.mkdir(parents=True,exist_ok=True)
 if q.exists():assert pin(q)['sha256']==pin(p)['sha256']
 else:shutil.copyfile(p,q)
 copies.append(pin(q))
copy(root,root.name)
for e in d['outputs'].values() if isinstance(d['outputs'],dict) else d['outputs']:
 if isinstance(e,dict) and 'path' in e:copy(Path(e['path']),Path(e['path']).name)
ranges=read(base/'lito脫殼資源/D5-settings-receive/ranges.json');ins=read(base/'lito脫殼資源/D5-settings-receive/instruction-witnesses.json');walk(ranges);walk(ins)
pe=base/'lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe';assert pin(pe)['sha256']=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9';raw=pe.read_bytes();checks=[]
def rawwalk(v):
 if isinstance(v,dict):
  if 'raw_hex' in v and ('file_offset' in v or 'va' in v):
   n=v.get('file_offset');n=int(n,16) if isinstance(n,str) else n
   if n is None:n=int(str(v['va']),16)-0x400000
   b=bytes.fromhex(v['raw_hex']);assert raw[n:n+len(b)]==b;checks.append(dict(offset=n,bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper()))
  for x in v.values():rawwalk(x)
 elif isinstance(v,list):
  for x in v:rawwalk(x)
rawwalk(ranges);rawwalk(ins)
for i,e in enumerate(ranges['inherited_necessary_raw']):
 out=e['source_pin'];copy(Path(out['path']),Path('inherited')/f'{i:02d}-{Path(out["path"]).name}')
future=base/'lito伺服器/server-ordinary-room-map-selection-contract.json';assert pin(future)['sha256']=='3D0B5BAA46238ED5589C193FBDE22B966B945EFDEAB70D2A2C3933A62CA8E15F';copy(future,future.name)
save(L/'r520-native-input-adoption.json',dict(root=pin(root),full_contract=d,reference_occurrences=len(occ),unique_references=list(refs.values()),copies=copies,fixed_PE=pin(pe),raw_byte_checks=checks,new_ranges=len(ranges['new_ranges']),inherited_ranges=len(ranges['inherited_necessary_raw']),instruction_assertions=len(ins['assertions']),scope='Raw fixed PE bytes independently compared; complete conditions/unknowns retained. No disassembler/provider/Native/Java/SQL run, no old PASS replay.',Native_D5_branch_contract_adopted=True,Native_execution=False,Native_map_projection='full WORD FFFF/FF9C -> FF/9C; every other WORD AL+0C modulo256; 564 -> 64. Static only; no Modern runtime projection/resource binding.',Native_count='signed K min(8,max(0,i8K)); unread8 slots zeroed. Modern AD policy remains K<=8/exact end.',Native_phase6='prewrites room ID before subtype validity; never grants Modern ownership or bypasses room/epoch.',future_map_dependency_only=pin(future),C_D1_D5sub3_implemented=False))
print('R520 Native complete contract adopted:',len(refs),'unique refs;',len(checks),'fixed PE raw comparisons')
