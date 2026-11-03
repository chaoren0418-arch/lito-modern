from r491_common import *
import shutil
base=P.parent.parent
roots=[base/'lito脫殼資源/SCC-catalog-contract.json',base/'lito脫殼資源/SCC-catalog-lifecycle-contract.json',base/'lito伺服器/server-SCC-compatibility-comparison.json',base/'lito伺服器/server-SCC-publication-visibility-supplement.json']
expected=['E376B067F4E8BACD7F0DC01F2A615BE0E2B0B71C2FA3DC59E001EC0330D3F9B5','3ADB8FD6D3F48E6BB444C00BF3808A1F1A758E82C2EB2E9CEDD4F45CFAB38367','D7FD0DE1748ABE871A7CB3EB416146CDE2FFFF1201003D5A2187F234E3131A63','1D6F4348EAB19C212459E573F2A86F648CD88A724C7B7D1DD1F8A1D24F7A405B']
dest=L/'r518-inputs';dest.mkdir();cache={};occurrences=[];raw=[];copies=[];docs=[]
def verify(e,base=None):
 q=Path(e['path']);q=q if q.is_absolute() else base/q
 if str(q) not in cache:cache[str(q)]=pin(q)
 a=cache[str(q)];assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256']),(e,a)
 occurrences.append(a);return q
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):verify(v)
  if v.get('raw_hex') and v.get('file_offset'):raw.append(v)
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
def copy(q,to):
 to.parent.mkdir(parents=True,exist_ok=True);assert not to.exists();shutil.copyfile(q,to);copies.append(pin(to))
for i,q in enumerate(roots):
 assert pin(q)['sha256']==expected[i];d=read(q);docs.append(d);walk(d);copy(q,dest/q.name)
 for e in d.get('reusable_outputs',[]):
  source=verify(e);copy(source,dest/str(i)/source.name)
  if source.suffix=='.json':walk(read(source))
 if 'manifest' in d:
  manifest=read(d['manifest']['path']);root=Path(d['evidence_root']);copy(Path(d['manifest']['path']),dest/str(i)/'manifest.json')
  for e in manifest['files']:
   source=verify(e,root);copy(source,dest/str(i)/source.relative_to(root))
   if source.suffix=='.json':walk(read(source))
pe=Path(docs[0]['sources'][0]['path']).read_bytes();slices=[]
for d in docs[:2]:
 for e in d['reusable_outputs']:
  q=Path(e['path'])
  if q.suffix=='.bin':
   va=int(q.name.split('-')[0],16);b=q.read_bytes();assert pe[va-0x400000:va-0x400000+len(b)]==b;slices.append(dict(pin=pin(q),va=va,matches=True))
for v in raw:
 b=bytes.fromhex(v['raw_hex']);at=int(v['file_offset'],16);assert pe[at:at+len(b)]==b
old=read(L/'r517-test-inputs.json');old515=read(L/'r515-test-inputs.json');row=old['next_CC'];room2=bytes.fromhex(row);room2=room2[:2]+(2).to_bytes(4,'little')+room2[6:]
save(L/'r518-test-inputs.json',dict(setup_frames=old['setup_frames'],CE=old['CE'],CC_saved=row,CC_synthetic_room2=room2.hex(),CF=old['CF'],next_real=old515['Guest_next_after_saved_D1'],prior_setup=pin(L/'r517-test-inputs.json'),prior_next=pin(L/'r515-test-inputs.json'),saved_dynamic_trace=pin(L/'r517-inputs/dynamic/frames.json'),context='ONE new Modern normal-entry synthetic case. Captured CC0 room1 and CF2 bodies reused; room2 CC is explicitly synthetic ID rewrite, not a saved Java publication. Sequence/cipher reencoded; no actual Java cross-peer/current context claim.'))
save(L/'r518-input-adoption.json',dict(roots=[pin(q) for q in roots],reference_occurrences=len(occurrences),unique_references=list(cache.values()),copies=copies,new_raw_slices=slices,raw_byte_witness_occurrences=len(raw),unique_raw_byte_witnesses=len({(x['file_offset'],x['raw_hex']) for x in raw}),raw_bytes_match_fixed_PE=True,instruction_tests='Producer assertions only; raw equality independently checked, no disassembly assertion rerun.',contracts=docs,test_inputs=pin(L/'r518-test-inputs.json'),Java_SQL_GUI=0,old_PASS_replays=0))
print('R518 adopted',len(occurrences),'references;',len(cache),'unique;',len(slices),'new slices;',len(raw),'raw witnesses')
