from r491_common import *
import shutil,re
roots=[Path(r'D:\lineage開發\3-lito開發\lito伺服器\server-room-D0-consumer-routing-supplement.json'),Path(r'D:\lineage開發\3-lito開發\lito脫殼資源\SD0-receive-contract.json'),Path(r'D:\lineage開發\3-lito開發\lito伺服器\server-ordinary-room-exit-commit-network.json')]
expected=['7BF90D85A297E8AAA9D90D3BD4AF11DFD13992D685FF7211CBD1A9433CA898E6','50A26C6526BC3BE8365DA435AE50117A25265EDE1AB0DC1E2F170BAEA95006C4','FB4EB76E965BA1C094AD7BD542A4CD7542DD5F99987CF884B041AD9C9676D44D']
cache={};verified=[];raw=[];copied=[]
def verify(e,base=None):
 q=Path(e['path']);q=q if q.is_absolute() else base/q
 if str(q) not in cache:cache[str(q)]=pin(q)
 a=cache[str(q)];assert(a['bytes'],a['sha256'])==(e['bytes'],e['sha256']),(e,a)
 verified.append(a);return q
def walk(v):
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):verify(v)
  if v.get('file_offset') and v.get('raw_hex'):raw.append(v)
  for k,x in v.items():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
root=L/'r516-inputs';root.mkdir()
docs=[]
def copy(q,destination):
 destination.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(q,destination);copied.append(pin(destination))
for i,q in enumerate(roots):
 assert pin(q)['sha256']==expected[i];d=read(q);docs.append(d);walk(d);copy(q,root/q.name)
 if 'manifest' in d:
  manifest=read(d['manifest']['path']);base=Path(d['evidence_root'])
  for e in manifest['files']:verify(e,base)
  copy(Path(d['manifest']['path']),root/str(i)/'manifest.json')
native=docs[1];pe=Path(native['sources'][0]['path']).read_bytes()
for v in raw:
 b=bytes.fromhex(v['raw_hex']);at=int(v['file_offset'],16);assert pe[at:at+len(b)]==b
new_raw=[]
for e in native['reusable_outputs']:
 q=verify(e);copy(q,root/'native'/q.name)
 if q.suffix=='.bin':
  va=int(q.name.split('-')[0],16);b=q.read_bytes();assert pe[va-0x400000:va-0x400000+len(b)]==b;new_raw.append(dict(pin=pin(q),va=va,matches=True))
route=read(docs[0]['contract']['path']);walk(route);copy(Path(docs[0]['contract']['path']),root/'route-boundary.json')
commit=docs[2]
for k in ['new_network','internal_commit','write_boundaries','poststates','consumer_routing','cleanup','protected','accounting','correction_history']:
 e=commit[k]['frames'] if k=='new_network' else commit[k];q=verify(e);copy(q,root/'dynamic'/q.name)
 # The projection and internal observer proof have different evidence levels.
saved=read(L/'r514-test-inputs.json');members=read(L/'r515-test-inputs.json')
save(L/'r516-test-inputs.json',dict(setup_frames=saved['setup_frames'],CE=saved['golden_CE'][0]['body_hex'],participants=members['vectors'],ArenaBot='d0010000004172656e61426f7400',setup_provenance=pin(L/'r514-test-inputs.json'),participant_provenance=pin(L/'r515-test-inputs.json'),only_new_case=True))
save(L/'r516-input-adoption.json',dict(roots=[pin(q) for q in roots],verified_reference_occurrences=len(verified),unique_verified_references=list(cache.values()),copied=copied,root_raw_occurrences=len(raw),unique_root_raw=len({(v['file_offset'],v['raw_hex']) for v in raw}),new_raw_slices=new_raw,native_wire=native['wire'],native_routing=native['routing'],native_consumer_gap=native['consumer_gap'],native_instruction_assertions_producer_only=native['verification']['instruction_assertions'],route_policy=route['routing'],dynamic_network=commit['new_network'],dynamic_findings=commit['findings'],dynamic_unknowns=commit['unknowns'],levels='Pinned fixed PE/static contracts and archived Java dynamic observations adopted read-only. No Java, SQL, Native or old PASS rerun; new Modern loopback program evidence remains separate.',test_inputs=pin(L/'r516-test-inputs.json')))
print('Adopted',len(verified),'refs;',len(cache),'unique;',len(raw),'raw witnesses;',len(new_raw),'raw slices; no producer calls')
