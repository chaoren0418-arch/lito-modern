from r491_common import *
import shutil
B=Path(r'D:/lineage開發/3-lito開發/lito脫殼資源');dest=L/'r510-inputs';dest.mkdir(exist_ok=True)
names=[('native-icon-draw-contract.json','9BA690374734FECFE86CF95651E864515FFFDC78D71FE58C556974625AD95803'),('skill-pane-layout-contract.json','45EBDED431F925A0F38066702E67F0AD79E9139939D793E201A96EEBD15E681F'),('skill-pane-render-initialization-contract.json','00262DDFF19C9392CFAEDDC53F38DE46725D99D0011A48F3FEF046C02754C1DE'),('skill-pane-frame-coordinate-contract.json','8B4AFC6C4C884099AD092459B1608B2586E4A4EE6DB22F1FBBA642EE4DC6A7CE')]
checked={};seen=set();pe=Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe').read_bytes();instructions={};failures=[]
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ['path','bytes','sha256']):
   p=Path(v['path']);assert p.is_absolute(),p;key=(str(p),v['sha256'].upper());actual=checked.get(key)
   if not actual:
    actual=pin(p)
    if actual['bytes']!=v['bytes'] or actual['sha256']!=key[1]:
     failures.append(dict(expected=v,actual=actual));save(L/'r510-transitive-pin-failures.json',failures)
     assert p in [B/'completeness-audit.json',B/'handoff.json'],(key,actual)
     # Mutable global progress referenced by an older extraction report.
     # Exclude it from qualified input; never promote it to an asset oracle.
     return
    checked[key]=actual
   scoped=p.parent.name in ['graphics-parity','skill-pane-layout','skill-pane-render-initialization','skill-pane-frame-coordinate'] or 'graphics-parity' in p.parts
   if p.suffix=='.json' and scoped and key not in seen:seen.add(key);walk(read(p))
  if 'va' in v and 'raw_hex' in v:
   h=bytes.fromhex(v['raw_hex']);a=int(v['va'],16)-0x400000;assert pe[a:a+len(h)]==h,(v['va'],v['raw_hex']);instructions[(v['va'],v['raw_hex'])]=True
  for e in v.values():walk(e)
 elif isinstance(v,list):
  for e in v:walk(e)
contracts=[]
for name,sha in names:
 p=B/name;q=pin(p);assert q['sha256']==sha;walk(read(p));shutil.copyfile(p,dest/name);contracts.append(q)
save(L/'r510-input-adoption.json',dict(contracts=contracts,verified_payload_pins=list(checked.values()),excluded_mutable_global_historical_references=failures,recursive_scope='Current graphics-parity/skill-pane batch directories only; earlier contracts and manifests pinned as direct references, not recursively expanded into historical mutable global progress',verified_unique_instruction_bytes=len(instructions),fixed_PE=pin(Path(r'D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe')),previous_R509=pin(L/'r509-character-selection-channel-handoff.json'),GUI=0,wire_changes=0))
def short(v):
 if isinstance(v,dict):return {k:short(e) for k,e in v.items() if not any(x in k for x in ['evidence','ranges','witnesses','references','reads','source'])}
 if isinstance(v,list):return [short(e) for e in v]
 return v
for name,_ in names[1:]:print(name,json.dumps(short(read(B/name)),ensure_ascii=False))
print('Verified payloads',len(checked),'unique instruction bytes',len(instructions))
