from r491_common import *
import shutil
base=P.parent.parent;dest=L/'r523-inputs';dest.mkdir(exist_ok=True);refs={};occ=[];copies=[]
roots=[(base/'lito脫殼資源/CD1-map-outbound-contract.json','9A2BA5AB692E92088E5B9BCA3C45A449EB04D7B3D2AD7AACB88EE7AAF43D598D'),(base/'lito伺服器/server-ordinary-room-DD-edit-network.json','A13ECBFE9DC9BFCBA1D32649C829FB960DA995591C59C8613BB403FA4D62C168'),(base/'lito脫殼資源/CDD-outbound-contract.json','7E09849B4F0D442E04457AEE7E6DBEEB3D106437D9689AE8060323CE36FCB59E')]
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   p=Path(v['path']);a=refs.setdefault(str(p),pin(p));assert(a['bytes'],a['sha256'])==(v['bytes'],v['sha256']),(v,a);occ.append(a)
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
def copy(p,name):
 q=dest/name;q.parent.mkdir(parents=True,exist_ok=True)
 if q.exists():assert pin(q)['sha256']==pin(p)['sha256']
 else:shutil.copyfile(p,q)
 copies.append(pin(q))
docs=[]
for p,sha in roots:
 assert pin(p)['sha256']==sha;d=read(p);docs.append(d);walk(d);copy(p,p.name)
for e in docs[0]['outputs']:copy(Path(e['path']),Path('BIN27')/Path(e['path']).name)
rp=base/'lito脫殼資源/CD1-map-outbound/ranges.json';ip=base/'lito脫殼資源/CD1-map-outbound/instruction-witnesses.json';ranges=read(rp);ins=read(ip);walk(ranges);walk(ins)
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
m=read(docs[1]['manifest']['path']);root=Path(docs[1]['evidence_root']);assert len(m['files'])==58
for e in m['files']:
 p=Path(e['path']);p=p if p.is_absolute() else root/p;a=pin(p);assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256']);refs[str(p)]=a;copy(p,Path('AD368-DD')/p.relative_to(root))
f=read(docs[1]['new_network']['frames']['path']);data=read(L/'r520-test-inputs.json');indices=[19,65,69,70,74,76,77,78,79,84,85];data['actual_C_DD']=f[76]['body_hex'];data['input_name']='EditAfterRoom';data['input_attributes']=40;data['input_password']='';data['input_capacity']=4;data['selected_frames']=[dict(index=i,frame=f[i]) for i in indices];data['prior_setup']=pin(L/'r520-test-inputs.json');data['context']='R523 ONE new own synthetic normal-entry actual Sender compact C_DD frame, saved AD368 Host perpeer D1reset→summary0→config1 reencoded. Independent rejected identical snapshot probe noACK, not sameoriginalHostflow. ASCII candidate policy only; no extended/mode/map/roomID. Future485F unaccepted/not adopted; no JavaSQL/NativeGUI run.';save(L/'r523-test-inputs.json',data)
save(L/'r523-input-adoption.json',dict(roots=[pin(p) for p,_ in roots],full_contracts=docs,reference_occurrences=len(occ),unique_references=list(refs.values()),copies=copies,fixed_PE=pin(pe),BIN27_raw_byte_checks=checks,BIN27_instruction_witnesses=len(ins) if isinstance(ins,list) else len(ins['assertions']),BIN27_conditional_transform='VA464DF1 selectedlookup450B90 twice, itemDWORD+4 equalsFFFFFFFF/FFFFFF9C retainbits else+500 wrapping32; cch low16LE call464E28. Algebra/tablekey-500 vs sourceordinal separate, no menu/resource binding or runtime selection.',BIN27_unknowns='Full root conditions preserved: table/list stability/click/count/bounds/room/owner/version/phase/caller unqualified; allocation46515D→4F118C STOP; ctor46313C→463B70 STOP; menu46B828virtual/46B849 allocation STOP; post464E40 virtualSTOP; actualtransport759058A0 STOP; adjacentmodeD1/2 step1/2 remainsunimplemented.',BIN27_prior_capture_scope='Old finite7 captureszero map hits retained as scoped historical input, not globalcensus; accepted newer C7network adopted separatelyR522. No old evidence rewrite or rerun.',AD368_DD_payload_files=58,test_inputs=pin(L/'r523-test-inputs.json'),future485F_adopted=False,Java_SQL_GUI=0,old_PASS_replays=0))
print('R523 adopted',len(refs),'unique refs;',len(checks),'BIN27 fixedPE raw comparisons;58 AD368 DD payloads; no new backend')
